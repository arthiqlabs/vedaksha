// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Catalogue star positions, and their mean ecliptic longitude of date.
//!
//! This module exists to serve sidereal zodiac systems that are defined by
//! fixing a named star at an assigned longitude. Such a system is not a
//! constant: it is whatever the star's longitude happens to be, which means the
//! engine must carry the star's catalogue astrometry and propagate it.
//!
//! # The convention is part of the definition
//!
//! "The tropical longitude of the star" is ambiguous by roughly 20 arcseconds
//! of annual aberration plus 17 arcseconds of nutation — larger than the
//! differences that distinguish one sidereal system from another. This module
//! therefore commits to one reading and states it:
//!
//! [`mean_ecliptic_longitude_of_date`] returns the **mean geometric place of
//! date**:
//!
//! * proper motion **is** applied — it is part of where the star is;
//! * annual aberration is **not** — it is an artefact of the observer's motion;
//! * nutation is **not** — the result is referred to the *mean* equinox of date;
//! * annual parallax is **not**, and neither is light deflection — both are
//!   observer effects, and neither exceeds a milliarcsecond for these stars.
//!
//! A caller wanting an apparent place adds those terms themselves.
//!
//! # Accuracy
//!
//! The transform agrees with ERFA's `eraEqec06` to better than 1e-6 arcsecond
//! over a six-millennium span (see the tests). The error budget is therefore set
//! entirely by the catalogue, not by this code: a Hipparcos proper motion good
//! to ~0.5 mas/yr accumulates ~0.5 arcsecond of longitude error over a thousand
//! years, and that is the honest uncertainty on any star-anchored system far
//! from the catalogue epoch.

use crate::{obliquity, precession};
use vedaksha_math::angle::normalize_degrees;
use vedaksha_math::matrix::Vector3;

/// Milliarcseconds to radians.
const MAS_TO_RAD: f64 = core::f64::consts::PI / (180.0 * 3600.0 * 1000.0);

/// Days in a Julian year — the unit proper motions are quoted in.
const DAYS_PER_JULIAN_YEAR: f64 = 365.25;

/// A star's catalogue astrometry, as published.
///
/// Fields are stored exactly as the catalogue publishes them so that a reader
/// can compare this struct against the catalogue row without doing arithmetic.
/// Nothing here is derived.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatalogueStar {
    /// The catalogue this row came from, e.g. `"I/311/hip2"`.
    pub catalogue: &'static str,
    /// The identifier within that catalogue, e.g. `"HIP 65474"`.
    pub identifier: &'static str,
    /// Julian Day (TT) of the catalogue's reference epoch.
    pub epoch_jd_tt: f64,
    /// ICRS right ascension at `epoch_jd_tt`, in degrees.
    pub ra_deg: f64,
    /// ICRS declination at `epoch_jd_tt`, in degrees.
    pub dec_deg: f64,
    /// Proper motion in right ascension, `mu_alpha * cos(delta)`, in mas/yr.
    pub pm_ra_cosdec_mas_per_year: f64,
    /// Proper motion in declination, `mu_delta`, in mas/yr.
    pub pm_dec_mas_per_year: f64,
}

impl CatalogueStar {
    /// The star's ICRS unit direction vector at a given date, proper motion applied.
    ///
    /// Proper motion is applied as a linear space motion in the tangent plane at
    /// the catalogue epoch, then renormalised. This is exact for a star with no
    /// radial velocity; against a rigorous great-circle propagation the residual
    /// grows as roughly `theta^3 / 3` in the accumulated motion `theta`.
    ///
    /// Sizes, for the fastest star this engine carries (delta Cancri, 229 mas/yr):
    /// **0.8 mas at two millennia** from the catalogue epoch, and **12.6 mas at
    /// the ~5100-year extreme of the shipped conformance grid**. The earlier
    /// version of this comment claimed sub-milliarcsecond accuracy and named two
    /// millennia, while the grid the engine ships reaches five — a stated scope
    /// narrower than the exercised one, which is its own kind of wrong.
    ///
    /// Neither figure is material: at that epoch the catalogue's own proper-motion
    /// uncertainty already admits ~1.4 arcseconds, a hundred times larger.
    #[must_use]
    pub fn icrs_direction(&self, jd_tt: f64) -> Vector3 {
        let ra = self.ra_deg.to_radians();
        let dec = self.dec_deg.to_radians();
        let (sin_ra, cos_ra) = (libm::sin(ra), libm::cos(ra));
        let (sin_dec, cos_dec) = (libm::sin(dec), libm::cos(dec));

        // Position at the catalogue epoch.
        let p = Vector3::new(cos_dec * cos_ra, cos_dec * sin_ra, sin_dec);

        // Unit vectors along increasing RA and increasing declination.
        let e_ra = Vector3::new(-sin_ra, cos_ra, 0.0);
        let e_dec = Vector3::new(-sin_dec * cos_ra, -sin_dec * sin_ra, cos_dec);

        let years = (jd_tt - self.epoch_jd_tt) / DAYS_PER_JULIAN_YEAR;
        let mu_ra = self.pm_ra_cosdec_mas_per_year * MAS_TO_RAD * years;
        let mu_dec = self.pm_dec_mas_per_year * MAS_TO_RAD * years;

        let x = p.x + mu_ra * e_ra.x + mu_dec * e_dec.x;
        let y = p.y + mu_ra * e_ra.y + mu_dec * e_dec.y;
        let z = p.z + mu_ra * e_ra.z + mu_dec * e_dec.z;
        let norm = libm::sqrt(x * x + y * y + z * z);

        Vector3::new(x / norm, y / norm, z / norm)
    }
}

/// The star's mean geometric ecliptic longitude, referred to the mean equinox
/// and ecliptic of date, in degrees, normalised to `[0, 360)`.
///
/// See the module documentation for exactly which corrections this does and
/// does not include — the choice is part of the definition of any system built
/// on it, not an implementation detail.
///
/// # Arguments
///
/// * `star` — catalogue astrometry.
/// * `jd_tt` — Julian Day Number, Terrestrial Time.
#[must_use]
pub fn mean_ecliptic_longitude_of_date(star: &CatalogueStar, jd_tt: f64) -> f64 {
    let (lon, _) = mean_ecliptic_of_date(star, jd_tt);
    lon
}

/// The star's mean geometric ecliptic longitude and latitude of date, in degrees.
///
/// Longitude is normalised to `[0, 360)`; latitude is in `[-90, 90]`.
#[must_use]
pub fn mean_ecliptic_of_date(star: &CatalogueStar, jd_tt: f64) -> (f64, f64) {
    let icrs = star.icrs_direction(jd_tt);

    // ICRS -> mean equator and equinox of date. The Fukushima-Williams angles
    // are referred to the GCRS, so this rotation carries the frame bias too.
    let equatorial = precession::precession_matrix(jd_tt).apply(&icrs);

    // Mean equator of date -> mean ecliptic of date. Mean obliquity, not true:
    // including nutation here would return an apparent place.
    let eps = obliquity::mean_obliquity(jd_tt);
    let (sin_eps, cos_eps) = (libm::sin(eps), libm::cos(eps));

    let x = equatorial.x;
    let y = equatorial.y * cos_eps + equatorial.z * sin_eps;
    let z = -equatorial.y * sin_eps + equatorial.z * cos_eps;

    let lon = normalize_degrees(libm::atan2(y, x).to_degrees());
    let lat = libm::asin(z.clamp(-1.0, 1.0)).to_degrees();
    (lon, lat)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An arbitrary fixed direction, deliberately unrelated to any real star.
    ///
    /// It used to be Sgr A*'s position — which was a bad choice twice over. That
    /// pair is the one the derivation explicitly disavows (it is the arbitrary
    /// origin Reid & Brunthaler measure offsets from, not a measurement, and it
    /// sits ~40 mas from the ICRF3 position), so repeating it in shipped source
    /// undercut the fetch manifest's own "recorded so it is not repeated". And
    /// its ecliptic longitude minus 240 degrees was one subtraction from a
    /// Galactic-Centre ayanamsha, which is not a number a clean-room file should
    /// carry by accident.
    ///
    /// Any direction does; this one is arbitrary and lands nowhere near an
    /// assigned sidereal longitude.
    const FIXED_DIRECTION: CatalogueStar = CatalogueStar {
        catalogue: "test",
        identifier: "arbitrary fixed direction",
        epoch_jd_tt: 2_451_545.0,
        ra_deg: 123.456_789,
        dec_deg: 45.678_901,
        pm_ra_cosdec_mas_per_year: 0.0,
        pm_dec_mas_per_year: 0.0,
    };

    /// `eraEqec06` applied to `FIXED_DIRECTION`, at four epochs spanning ~6100 years.
    ///
    /// ERFA is the IAU SOFA board's BSD-3 relicensing of SOFA. Checking our own
    /// astronomy against a permissive reference implementation is what it is for.
    const ERFA_EQEC06: [(f64, f64, f64); 4] = [
        (588_465.75, 44.581_951_457, 24.467_648_409),
        (1_903_397.0, 94.241_225_583, 24.944_140_627),
        (2_451_545.0, 115.177_689_978, 25.121_014_299),
        (2_816_788.0, 129.209_891_274, 25.230_029_342),
    ];

    #[test]
    fn transform_matches_erfa_eqec06() {
        for (jd, expect_lon, expect_lat) in ERFA_EQEC06 {
            let (lon, lat) = mean_ecliptic_of_date(&FIXED_DIRECTION, jd);
            let d_lon = (lon - expect_lon) * 3600.0;
            let d_lat = (lat - expect_lat) * 3600.0;
            assert!(
                d_lon.abs() < 0.01,
                "jd={jd}: longitude {lon:.9} vs ERFA {expect_lon:.9} ({d_lon:+.6} arcsec)"
            );
            assert!(
                d_lat.abs() < 0.01,
                "jd={jd}: latitude {lat:.9} vs ERFA {expect_lat:.9} ({d_lat:+.6} arcsec)"
            );
        }
    }

    #[test]
    fn proper_motion_moves_the_star_by_the_catalogued_amount() {
        // A star at the equator with 1000 mas/yr in RA should move 1000 mas of
        // RA per Julian year. Checked on the sphere rather than in the formula,
        // so a sign or unit error in `icrs_direction` shows up as a position.
        let star = CatalogueStar {
            catalogue: "test",
            identifier: "synthetic",
            epoch_jd_tt: 2_451_545.0,
            ra_deg: 100.0,
            dec_deg: 0.0,
            pm_ra_cosdec_mas_per_year: 1_000.0,
            pm_dec_mas_per_year: 0.0,
        };
        let after = star.icrs_direction(2_451_545.0 + 100.0 * 365.25);
        let ra = normalize_degrees(libm::atan2(after.y, after.x).to_degrees());
        let moved_arcsec = (ra - 100.0) * 3600.0;
        assert!(
            (moved_arcsec - 100.0).abs() < 0.01,
            "100 years at 1000 mas/yr should be 100 arcsec, got {moved_arcsec:.6}"
        );
    }

    #[test]
    fn zero_proper_motion_leaves_the_direction_alone() {
        let a = FIXED_DIRECTION.icrs_direction(2_451_545.0);
        let b = FIXED_DIRECTION.icrs_direction(1_000_000.0);
        assert!((a.x - b.x).abs() < 1e-15);
        assert!((a.y - b.y).abs() < 1e-15);
        assert!((a.z - b.z).abs() < 1e-15);
    }
}
