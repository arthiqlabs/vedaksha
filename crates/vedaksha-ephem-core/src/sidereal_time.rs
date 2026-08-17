// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Greenwich Mean and Apparent Sidereal Time.
//!
//! Implements GMST (eq. 12.4), GAST, and Local Apparent Sidereal Time.
//!
//! # Time scale — read before calling
//!
//! Sidereal time measures the **Earth's rotation**, and Earth rotation is
//! measured in **UT1** by definition. Every function in this module therefore
//! takes `jd_ut1`, a **UT1** Julian Day, as its rotational argument. Passing
//! TT (or TDB) adds ΔT worth of extra rotation instead of removing it:
//! at ΔT ≈ 69 s, `360.98564736629 °/day × 69.2 s / 86400 s` = **0.289°**
//! (≈ 17.3′) of error — which lands directly on the RAMC, and hence on the
//! ascendant, the MC and all twelve house cusps.
//!
//! The `dpsi` / `true_obliquity` arguments of [`gast`] and
//! [`local_sidereal_time`] are the **opposite** case: nutation and obliquity
//! are dynamical quantities and must be evaluated at **TT**. The split is
//! deliberate — do not "unify" the two time scales in a caller. The correct
//! call shape is:
//!
//! ```text
//! let jd_tt = delta_t::ut1_to_tt(jd_ut1);
//! let (dpsi, deps) = nutation::nutation(jd_tt);        // dynamical → TT
//! let eps_true = obliquity::true_obliquity(jd_tt, deps); // dynamical → TT
//! let last = local_sidereal_time(jd_ut1, lon, dpsi, eps_true); // rotational → UT1
//! ```
//!
//! Source: Meeus, *Astronomical Algorithms*, 2nd ed., Chapter 12.

use crate::julian;
use vedaksha_math::angle::{normalize_degrees, normalize_radians};

/// Greenwich Mean Sidereal Time (GMST) in radians, range [0, 2π).
///
/// Uses Meeus equation 12.4, which gives GMST in degrees as a polynomial
/// in Julian centuries T from J2000.0 plus a term proportional to the
/// fractional Julian day from J2000.
///
/// # Arguments
/// * `jd_ut1` — Julian Day in **UT1**. Not TT, not TDB. The dominant term
///   `360.98564736629 · (JD − 2451545.0)` *is* the Earth's physical rotation,
///   which is what UT1 measures; feeding TT introduces ~0.29° of error at the
///   present ΔT (≈69 s). See the module-level note.
#[must_use]
pub fn gmst(jd_ut1: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd_ut1);
    // Meeus eq. 12.4 — result in degrees
    let theta0 =
        280.460_618_37 + 360.985_647_366_29 * (jd_ut1 - julian::J2000) + 0.000_387_933 * t * t
            - t * t * t / 38_710_000.0;

    let theta0_normalized = normalize_degrees(theta0);
    theta0_normalized * core::f64::consts::PI / 180.0
}

/// Greenwich Apparent Sidereal Time (GAST) in radians, range [0, 2π).
///
/// GAST = GMST + equation of the equinoxes, where:
/// ```text
/// equation_of_equinoxes = Δψ · cos(ε_true)
/// ```
///
/// # Arguments
/// * `jd_ut1`          — Julian Day in **UT1** (rotational; see the module
///   note — passing TT costs ~0.29°)
/// * `dpsi`            — nutation in longitude (radians), evaluated at **TT**
/// * `true_obliquity`  — true obliquity of the ecliptic (radians), at **TT**
///
/// The mixed time scales are intentional: GMST is rotation (UT1), while
/// nutation and obliquity are dynamical (TT).
#[must_use]
pub fn gast(jd_ut1: f64, dpsi: f64, true_obliquity: f64) -> f64 {
    let theta = gmst(jd_ut1) + dpsi * libm::cos(true_obliquity);
    normalize_radians(theta)
}

/// Local Apparent Sidereal Time (LAST) in radians, range [0, 2π).
///
/// LST = GAST + observer longitude (east positive).
///
/// # Arguments
/// * `jd_ut1`          — Julian Day in **UT1** (rotational; see the module
///   note — passing TT costs ~0.29°, which lands on the ascendant, the MC and
///   every house cusp)
/// * `longitude_rad`   — observer's geographic longitude in radians (east positive)
/// * `dpsi`            — nutation in longitude (radians), evaluated at **TT**
/// * `true_obliquity`  — true obliquity of the ecliptic (radians), at **TT**
///
/// The mixed time scales are intentional: the Julian Day is rotation (UT1),
/// while nutation and obliquity are dynamical (TT).
#[must_use]
pub fn local_sidereal_time(jd_ut1: f64, longitude_rad: f64, dpsi: f64, true_obliquity: f64) -> f64 {
    let theta = gast(jd_ut1, dpsi, true_obliquity) + longitude_rad;
    normalize_radians(theta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::J2000;
    use core::f64::consts::PI;

    /// Tolerance for sidereal-time comparisons: 0.1 arcsecond in radians.
    const EPS_RAD: f64 = 0.1 * PI / (180.0 * 3600.0);

    /// GMST at J2000.0 (2000 January 1.5 TDB) should be approximately
    /// 280.46° ≈ 4.894 rad.
    ///
    /// Reference: Meeus eq. 12.4 with T=0: θ₀ = 280.46061837°.
    #[test]
    fn gmst_at_j2000() {
        let result = gmst(J2000);
        let expected_deg = 280.460_618_37_f64;
        let expected_rad = expected_deg * PI / 180.0;
        assert!(
            (result - expected_rad).abs() < EPS_RAD,
            "GMST at J2000: expected {expected_rad:.6} rad ({expected_deg:.4}°), got {result:.6} rad"
        );
    }

    /// GMST must always lie in [0, 2π).
    #[test]
    fn gmst_in_range() {
        // Test a handful of different Julian Days spanning several epochs.
        let jds = [
            J2000 - 36_525.0, // J1900
            J2000,            // J2000
            J2000 + 18_262.5, // ~J2050
            J2000 + 36_525.0, // J2100
            2_458_849.5,      // 2020 Jan 1.0 UT
        ];
        for jd in jds {
            let g = gmst(jd);
            assert!(
                (0.0..2.0 * PI).contains(&g),
                "GMST({jd}) = {g} is outside [0, 2π)"
            );
        }
    }

    /// GAST with zero nutation equals GMST.
    #[test]
    fn gast_zero_nutation_equals_gmst() {
        let jd = J2000 + 1000.0;
        let g = gmst(jd);
        let ga = gast(jd, 0.0, 0.4090928); // obliquity value doesn't matter when dpsi=0
        assert!(
            (ga - g).abs() < f64::EPSILON * 100.0,
            "GAST with dpsi=0 should equal GMST: GMST={g}, GAST={ga}"
        );
    }

    /// Local Apparent Sidereal Time at Greenwich (longitude=0) equals GAST.
    #[test]
    fn lst_at_greenwich_equals_gast() {
        let jd = J2000 + 500.0;
        let dpsi = 1.0e-5_f64;
        let eps = 0.409_092_8_f64; // ~23.44° in radians
        let ga = gast(jd, dpsi, eps);
        let lst = local_sidereal_time(jd, 0.0, dpsi, eps);
        assert!(
            (lst - ga).abs() < f64::EPSILON * 100.0,
            "LST at Greenwich should equal GAST: GAST={ga}, LST={lst}"
        );
    }
}
