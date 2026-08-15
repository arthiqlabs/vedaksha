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
    let sin_alt =
        libm::sin(phi) * libm::sin(dec) + libm::cos(phi) * libm::cos(dec) * libm::cos(hour_angle);
    rad_to_deg(libm::asin(sin_alt.clamp(-1.0, 1.0)))
}

/// Refine a bracketed root of `f` in `[lo, hi]` by bisection.
///
/// `f` must change sign across the bracket. A fixed iteration count is used so
/// a closure that returns garbage cannot make this loop forever.
fn bisect(mut lo: f64, mut hi: f64, f: &dyn Fn(f64) -> Option<f64>) -> Option<f64> {
    let f_lo = f(lo)?;
    for _ in 0..BISECT_ITERS {
        let mid = 0.5 * (lo + hi);
        let f_mid = f(mid)?;
        if (f_lo < 0.0) == (f_mid < 0.0) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Rise, set and upper transit of a body across the 24 hours beginning at
/// `jd_ut_day_start`, for an observer at `lat_deg` / `lon_deg_east`.
///
/// `h0_deg` is the target apparent altitude of the body's centre — use
/// [`SUN_STANDARD_ALTITUDE_DEG`] (plus [`horizon_dip_deg`]) for the Sun.
/// `equatorial(jd_ut)` returns the body's apparent `(right_ascension_deg,
/// declination_deg)`, or `None` if unavailable at that instant.
///
/// A missing field means the event did not occur in the scanned interval.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
#[must_use]
pub fn rise_set(
    jd_ut_day_start: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> RiseSet {
    // Altitude relative to the target — zero exactly at a horizon crossing.
    let rel_alt = |jd: f64| -> Option<f64> {
        let (ra, dec) = equatorial(jd)?;
        Some(apparent_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg)
    };
    // Hour angle folded to (−180, 180] — zero exactly at upper transit.
    let hour_angle = |jd: f64| -> Option<f64> {
        let (ra, _) = equatorial(jd)?;
        let h = normalize_degrees(local_sidereal_degrees(jd, lon_deg_east) - ra);
        Some(if h > 180.0 { h - 360.0 } else { h })
    };

    let mut out = RiseSet {
        rise: None,
        set: None,
        transit: None,
    };

    let mut prev_jd = jd_ut_day_start;
    let (Some(mut prev_alt), Some(mut prev_ha)) = (rel_alt(prev_jd), hour_angle(prev_jd)) else {
        return out;
    };

    let mut jd = jd_ut_day_start + SCAN_STEP_DAYS;
    while jd <= jd_ut_day_start + 1.0 + 1e-9 {
        let (Some(alt), Some(ha)) = (rel_alt(jd), hour_angle(jd)) else {
            return out;
        };

        if out.rise.is_none() && prev_alt < 0.0 && alt >= 0.0 {
            out.rise = bisect(prev_jd, jd, &rel_alt);
        } else if out.set.is_none() && prev_alt >= 0.0 && alt < 0.0 {
            out.set = bisect(prev_jd, jd, &rel_alt);
        }

        // Upper transit: hour angle crosses zero upward. The magnitude guard
        // rejects the wrap from +180 to −180, which is lower transit.
        if out.transit.is_none() && prev_ha < 0.0 && ha >= 0.0 && (ha - prev_ha) < 180.0 {
            out.transit = bisect(prev_jd, jd, &hour_angle);
        }

        prev_jd = jd;
        prev_alt = alt;
        prev_ha = ha;
        jd += SCAN_STEP_DAYS;
    }

    out
}

/// The Sun's apparent equatorial `(right_ascension_deg, declination_deg)` at a
/// Julian Day (UT) — the adapter that turns an ephemeris provider into the
/// closure [`rise_set`] and [`sun_rise_set`] expect.
///
/// `coordinates::ecliptic_position` yields ecliptic **radians**, so this
/// rotates to the equator by the obliquity:
/// `sin δ = sin β·cos ε + cos β·sin ε·sin λ`,
/// `α = atan2(sin λ·cos ε − tan β·sin ε, cos λ)`.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13.
#[must_use]
pub fn sun_equatorial_deg(
    provider: &dyn vedaksha_ephem_core::jpl::EphemerisProvider,
    jd_ut: f64,
) -> Option<(f64, f64)> {
    use vedaksha_ephem_core::{bodies::Body, coordinates, obliquity};

    let pos = coordinates::ecliptic_position(provider, Body::Sun, jd_ut).ok()?;
    let (lambda, beta) = (pos.longitude, pos.latitude);
    let eps = obliquity::mean_obliquity(jd_ut);

    let sin_dec =
        libm::sin(beta) * libm::cos(eps) + libm::cos(beta) * libm::sin(eps) * libm::sin(lambda);
    let dec = libm::asin(sin_dec.clamp(-1.0, 1.0));
    let ra = libm::atan2(
        libm::sin(lambda) * libm::cos(eps) - libm::tan(beta) * libm::sin(eps),
        libm::cos(lambda),
    );

    Some((normalize_degrees(rad_to_deg(ra)), rad_to_deg(dec)))
}

/// [`rise_set`] specialised to the Sun: applies the standard −0°50′ altitude
/// plus the horizon dip for an observer `elevation_m` above sea level.
#[must_use]
pub fn sun_rise_set(
    jd_ut_day_start: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> RiseSet {
    let h0 = SUN_STANDARD_ALTITUDE_DEG + horizon_dip_deg(elevation_m);
    rise_set(jd_ut_day_start, lat_deg, lon_deg_east, h0, equatorial)
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

    /// A fake Sun fixed on the celestial equator rises ~6h and sets ~18h local
    /// apparent time at the equator, and the bracketing must find both plus the
    /// transit between them, in order.
    ///
    /// The window must start before the body's rise, or the first rise the
    /// scan can see belongs to the *next* cycle and lands after this
    /// transit/set — order is only guaranteed within one continuous arc. At
    /// JD 2,451,545.0 exactly (lon 0), GMST already has this fixed RA-0/dec-0
    /// body about 43 minutes past its rise (local sidereal time there is
    /// 280.7°, not 0°), so the window opens an hour earlier, comfortably
    /// before that rise (altitude ≈ −4.3° at that instant).
    #[test]
    fn equatorial_sun_rises_before_it_transits_before_it_sets() {
        let sun = |_jd: f64| Some((0.0_f64, 0.0_f64));
        let rs = rise_set(
            2_451_545.0 - 1.0 / 24.0,
            0.0,
            0.0,
            SUN_STANDARD_ALTITUDE_DEG,
            &sun,
        );
        let (r, t, s) = (
            rs.rise.expect("equatorial sun must rise"),
            rs.transit.expect("equatorial sun must transit"),
            rs.set.expect("equatorial sun must set"),
        );
        assert!(
            r < t && t < s,
            "expected rise < transit < set, got {r} {t} {s}"
        );
        // Day length at the equator with a -50' horizon is a little over 12 h.
        let day_hours = (s - r) * 24.0;
        assert!(
            (12.0..12.3).contains(&day_hours),
            "equatorial day length = {day_hours} h, expected ~12.1"
        );
    }

    /// Polar night: a deeply southern declination never clears the horizon at
    /// high northern latitude, so rise and set must be absent rather than
    /// wrong or panicking.
    #[test]
    fn polar_night_yields_no_rise_and_no_set() {
        let sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        let rs = rise_set(2_451_545.0, 78.22, 15.65, SUN_STANDARD_ALTITUDE_DEG, &sun);
        assert!(rs.rise.is_none(), "polar night must not report a sunrise");
        assert!(rs.set.is_none(), "polar night must not report a sunset");
    }

    /// The provider adapter must produce real equatorial coordinates, checked
    /// against a fact no ephemeris can get wrong: the Sun's declination is
    /// about +23.4° at the June solstice and about −23.4° at the December
    /// solstice. This catches a swapped or unrotated ecliptic→equatorial
    /// conversion, which would otherwise look plausible.
    #[test]
    fn sun_declination_tracks_the_solstices() {
        let provider = vedaksha_ephem_core::analytical::AnalyticalProvider;
        // 2000-06-21 and 2000-12-21, near the solstices.
        let (_, dec_jun) = sun_equatorial_deg(&provider, 2_451_716.5).expect("june position");
        let (_, dec_dec) = sun_equatorial_deg(&provider, 2_451_899.5).expect("december position");
        assert!(
            (dec_jun - 23.4).abs() < 0.5,
            "June solstice declination = {dec_jun}, expected ~+23.4"
        );
        assert!(
            (dec_dec + 23.4).abs() < 0.5,
            "December solstice declination = {dec_dec}, expected ~-23.4"
        );
    }

    /// A closure that cannot supply a position must degrade to `None`, not
    /// panic and not silently report an event at an arbitrary instant.
    #[test]
    fn unavailable_ephemeris_yields_no_events() {
        let sun = |_jd: f64| None;
        let rs = rise_set(2_451_545.0, 0.0, 0.0, SUN_STANDARD_ALTITUDE_DEG, &sun);
        assert_eq!(
            rs,
            RiseSet {
                rise: None,
                set: None,
                transit: None
            }
        );
    }
}
