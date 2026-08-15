// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Rise, set and meridian transit of a body for a terrestrial observer.
//!
//! The body's apparent equatorial coordinates are supplied by the caller as a
//! closure, so this module needs no ephemeris provider and stays `no_std` and
//! wasm-clean — the same injection `vedaksha_vedic::muhurta::search_muhurta`
//! uses.
//!
//! Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 12 (sidereal time),
//! Ch. 13 (altitude), Ch. 15 (rising, transit, setting).

use vedaksha_ephem_core::{delta_t, sidereal_time};
use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

/// Standard geometric altitude of the Sun's centre at rise/set: −0°50′,
/// covering mean atmospheric refraction (34′) plus the solar semidiameter
/// (16′).
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
pub const SUN_STANDARD_ALTITUDE_DEG: f64 = -50.0 / 60.0;

/// Coarse scan step when bracketing a horizon crossing: 5 minutes.
const SCAN_STEP_DAYS: f64 = 5.0 / 1440.0;

/// Bisection iterations. 40 halvings of a 5-minute bracket reach far below
/// float resolution; the loop is bounded rather than tolerance-driven so it
/// cannot spin on a pathological closure.
const BISECT_ITERS: u32 = 40;

/// Rise, set and upper-meridian transit of a body, as Julian Days (UT).
///
/// Any field is `None` when that event does not occur within the scanned day —
/// polar day and polar night being the cases that matter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiseSet {
    /// Julian Day (UT) of rising, if it occurs.
    pub rise: Option<f64>,
    /// Julian Day (UT) of setting, if it occurs.
    pub set: Option<f64>,
    /// Julian Day (UT) of upper-meridian transit, if it occurs.
    pub transit: Option<f64>,
}

/// Dip of the horizon in degrees for an observer `elevation_m` above it.
///
/// `dip ≈ −1.76′·√(elevation_m)` = `−0.0293°·√(elevation_m)`. Returns 0 at or
/// below sea level rather than an imaginary dip.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 16.
#[must_use]
pub fn horizon_dip_deg(elevation_m: f64) -> f64 {
    if elevation_m > 0.0 {
        -0.0293 * libm::sqrt(elevation_m)
    } else {
        0.0
    }
}

/// Local mean sidereal time in degrees [0, 360) for a UT Julian Day at
/// east-positive `lon_deg_east`.
///
/// `sidereal_time::gmst` expects TT and returns radians, so the UT instant is
/// converted with ΔT first. Ignoring that conversion would bias every rise/set
/// by ΔT (about 69 s today, and far more for historical dates).
fn local_sidereal_degrees(jd_ut: f64, lon_deg_east: f64) -> f64 {
    let jd_tt = delta_t::ut1_to_tt(jd_ut);
    normalize_degrees(rad_to_deg(sidereal_time::gmst(jd_tt)) + lon_deg_east)
}

/// Apparent altitude in degrees of a body at apparent equatorial coordinates
/// `(ra_deg, dec_deg)`, seen from `lat_deg` / `lon_deg_east` at `jd_ut`.
///
/// `sin(alt) = sin(φ)·sin(δ) + cos(φ)·cos(δ)·cos(H)`, `H` the local hour angle.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13.
#[must_use]
pub fn apparent_altitude_deg(
    ra_deg: f64,
    dec_deg: f64,
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
) -> f64 {
    let hour_angle = deg_to_rad(normalize_degrees(
        local_sidereal_degrees(jd_ut, lon_deg_east) - ra_deg,
    ));
    let (phi, dec) = (deg_to_rad(lat_deg), deg_to_rad(dec_deg));
    let sin_alt = libm::sin(phi) * libm::sin(dec)
        + libm::cos(phi) * libm::cos(dec) * libm::cos(hour_angle);
    rad_to_deg(libm::asin(sin_alt.clamp(-1.0, 1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Sun's altitude at its own transit must equal (90° − |lat − dec|)
    /// for an observer on the same meridian. Checked at Greenwich with a
    /// stationary fake Sun so the geometry is isolated from any ephemeris.
    #[test]
    fn altitude_at_transit_matches_the_closed_form() {
        // A fixed Sun at RA 0°, dec 0°. It transits Greenwich when LST = 0.
        let lat = 51.4778;
        // Find the instant LST(jd) == 0 near J2000 by scanning; the altitude
        // there must be 90 - |lat - dec| = 90 - 51.4778 = 38.5222.
        let mut best_jd = 2_451_545.0;
        let mut best_lst = f64::MAX;
        let mut t = 2_451_545.0;
        while t < 2_451_546.0 {
            let lst = local_sidereal_degrees(t, 0.0);
            let d = libm::fabs(((lst + 180.0) % 360.0) - 180.0);
            if d < best_lst {
                best_lst = d;
                best_jd = t;
            }
            t += 1.0 / 86_400.0;
        }
        let alt = apparent_altitude_deg(0.0, 0.0, best_jd, lat, 0.0);
        assert!(
            libm::fabs(alt - 38.5222) < 0.01,
            "altitude at transit = {alt}, expected ~38.5222"
        );
    }
}
