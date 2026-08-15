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

use vedaksha_ephem_core::sidereal_time;
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
/// GMST is a function of **UT**, not TT: `sidereal_time::gmst`'s dominant term
/// (Meeus eq. 12.4), `360.98564736629 * (JD - 2451545.0)`, tracks the Earth's
/// physical rotation, which is measured in UT by definition. Converting
/// `jd_ut` to TT before calling it would *introduce* rotation error rather
/// than remove one — about 0.29° (≈69 s of time) at the present ΔT.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 12.
fn local_sidereal_degrees(jd_ut: f64, lon_deg_east: f64) -> f64 {
    normalize_degrees(rad_to_deg(sidereal_time::gmst(jd_ut)) + lon_deg_east)
}

/// Geometric altitude in degrees of a body at apparent equatorial coordinates
/// `(ra_deg, dec_deg)`, seen from `lat_deg` / `lon_deg_east` at `jd_ut`.
///
/// `sin(alt) = sin(φ)·sin(δ) + cos(φ)·cos(δ)·cos(H)`, `H` the local hour angle.
///
/// This is the **geometric** altitude — no atmospheric refraction is applied
/// here. Refraction (plus, for the Sun, semidiameter and horizon dip) is
/// folded into the `h0_deg` target that [`rise_set`] compares this value
/// against, not into this function.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13.
#[must_use]
pub fn geometric_altitude_deg(
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
/// ⚠️ `rise` and `set` are each the FIRST such event inside the scanned
/// 24 hours and are **not** guaranteed to be in chronological order: at a
/// longitude where the window opens during local daytime, the set precedes
/// the rise. A caller pairing them into a "daytime" must order them itself —
/// scanning forward from the rise instant is the reliable way.
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
        Some(geometric_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg)
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

        // Upper transit: hour angle crosses zero upward. `prev_ha < 0.0 && ha
        // >= 0.0` alone already excludes the lower-transit wrap (+180 to
        // −180): `hour_angle` is folded to (−180, 180], so that wrap makes
        // `prev_ha` positive (near +180), which fails `prev_ha < 0.0`. A
        // magnitude guard on `(ha - prev_ha)` would only matter if a body's
        // hour angle could jump close to 360° within one `SCAN_STEP_DAYS`
        // (5 min), which no real or injected rate does here.
        if out.transit.is_none() && prev_ha < 0.0 && ha >= 0.0 {
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

    /// GMST is otherwise only ever checked against this module's own use of
    /// it, which cannot catch a time-scale bug (feeding TT instead of UT into
    /// `sidereal_time::gmst`) — both sides of the comparison would be wrong
    /// the same way. Pin `local_sidereal_degrees` against a value published
    /// independently of this codebase instead: Greenwich Mean Sidereal Time
    /// at 2000-01-01 0h UT (JD 2,451,544.5) is 6h 39m 52.0708s = 99.96696°.
    /// Feeding TT (JD + ΔT) into `gmst` here — the bug this module used to
    /// have — shifts the result by ~0.27°, well outside this test's 0.01°
    /// tolerance, so this is the test that makes the UT-vs-TT choice
    /// testable.
    #[test]
    fn local_sidereal_degrees_matches_published_gmst_at_j2000() {
        let lst = local_sidereal_degrees(2_451_544.5, 0.0);
        assert!(
            libm::fabs(lst - 99.966_96) < 0.01,
            "GMST at JD 2451544.5 = {lst}, expected 99.96696 (published)"
        );
    }

    /// The Sun's altitude at its own transit must equal (90° − |lat − dec|)
    /// for an observer on the same meridian. The previous version of this
    /// test *located* transit by scanning with `local_sidereal_degrees` and
    /// then evaluated altitude with the same function, so a constant LST
    /// error would shift both sides together and stay green. Anchor
    /// independently instead: the published GMST from
    /// `local_sidereal_degrees_matches_published_gmst_at_j2000` (99.96696° at
    /// JD 2,451,544.5) is used directly as this fake body's RA, which puts it
    /// on the Greenwich meridian (hour angle 0) at that exact instant by
    /// construction — no LST search involved.
    #[test]
    fn altitude_at_transit_matches_the_closed_form() {
        let lat = 51.4778;
        let ra = 99.966_96; // published GMST at JD 2451544.5, degrees.
        let dec = 0.0;
        let alt = geometric_altitude_deg(ra, dec, 2_451_544.5, lat, 0.0);
        // 90 - |lat - dec| = 90 - 51.4778 = 38.5222.
        let expected = 90.0 - libm::fabs(lat - dec);
        assert!(
            libm::fabs(alt - expected) < 0.01,
            "altitude at transit = {alt}, expected ~{expected}"
        );
    }

    /// A fake Sun fixed on the celestial equator rises ~6h and sets ~18h local
    /// apparent time at the equator, and the bracketing must find both plus the
    /// transit between them, in order.
    ///
    /// The window must start before the body's rise, or the first rise the
    /// scan can see belongs to the *next* cycle and lands after this
    /// transit/set — order is only guaranteed within one continuous arc. This
    /// starts the scan at JD 2,451,544.5, UT midnight on 2000-01-01 — the
    /// natural start of a UT day — rather than at a magic offset tuned to
    /// land the rise a fixed distance into the window. It works with margin
    /// to spare: rise is ~11.23 h into the window, transit ~17.27 h, set
    /// ~23.31 h, giving a day length of ~12.078 h — comfortably inside the
    /// window and inside the assertion below, with none of the three events
    /// close to either boundary.
    #[test]
    fn equatorial_sun_rises_before_it_transits_before_it_sets() {
        let sun = |_jd: f64| Some((0.0_f64, 0.0_f64));
        let rs = rise_set(2_451_544.5, 0.0, 0.0, SUN_STANDARD_ALTITUDE_DEG, &sun);
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
    /// wrong or panicking. The body still crosses the meridian once a day
    /// even though it never rises, so `transit` must still be `Some`.
    #[test]
    fn polar_night_yields_no_rise_and_no_set() {
        let sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        let rs = rise_set(2_451_545.0, 78.22, 15.65, SUN_STANDARD_ALTITUDE_DEG, &sun);
        assert!(rs.rise.is_none(), "polar night must not report a sunrise");
        assert!(rs.set.is_none(), "polar night must not report a sunset");
        assert!(
            rs.transit.is_some(),
            "a body that never rises still transits once a day"
        );
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
            libm::fabs(dec_jun - 23.4) < 0.5,
            "June solstice declination = {dec_jun}, expected ~+23.4"
        );
        assert!(
            libm::fabs(dec_dec + 23.4) < 0.5,
            "December solstice declination = {dec_dec}, expected ~-23.4"
        );
    }

    /// The Sun's right ascension must track the seasons: it is ~0°/360° at
    /// the March equinox, ~90° at the June solstice, ~180° at the September
    /// equinox and ~270° at the December solstice. The declination test above
    /// discards RA with `_`, and every `rise_set` test above injects a fixed
    /// RA via a fake closure, so this catches gross RA bugs (swapped
    /// axes, a stuck or unrotated value, a wrong sign) that those tests
    /// cannot.
    ///
    /// It does **not** catch dropping the obliquity rotation specifically:
    /// `RA = atan2(sin λ·cos ε − tan β·sin ε, cos λ)` happens to equal λ
    /// exactly at λ = 0°/90°/180°/270°, which is why the equinoxes and
    /// solstices are exactly where that term's effect vanishes. See
    /// `sun_right_ascension_reflects_the_obliquity_rotation` below for the
    /// test that targets the rotation itself, at a date away from those
    /// points.
    #[test]
    fn sun_right_ascension_tracks_the_seasons() {
        let provider = vedaksha_ephem_core::analytical::AnalyticalProvider;
        let cases = [
            (2_451_623.5, 0.0, "March equinox (2000-03-20)"),
            (2_451_716.5, 90.0, "June solstice (2000-06-21)"),
            (2_451_809.5, 180.0, "September equinox (2000-09-22)"),
            (2_451_899.5, 270.0, "December solstice (2000-12-21)"),
        ];
        for (jd, expected_ra, label) in cases {
            let (ra, _) =
                sun_equatorial_deg(&provider, jd).unwrap_or_else(|| panic!("{label} position"));
            // Fold the angular difference to (0, 180] so the March case
            // (expected 0°, actual RA near either 0° or 360°) is checked by
            // the wrap explicitly rather than passing by accident of range.
            let diff = normalize_degrees(ra - expected_ra);
            let diff = if diff > 180.0 { 360.0 - diff } else { diff };
            assert!(
                diff < 2.0,
                "{label}: RA = {ra}, expected ~{expected_ra} (diff {diff})"
            );
        }
    }

    /// Targets the obliquity rotation itself, which
    /// `sun_right_ascension_tracks_the_seasons` cannot: away from the
    /// cardinal points — here, 2000-05-04, roughly midway through the
    /// March→June quadrant — the "reduction to the equator" `RA − λ` is near
    /// its maximum (on the order of ε, ~2–3°), not near zero. The expected
    /// RA is computed independently from the same ecliptic longitude λ and
    /// obliquity ε the production code reads, but via a fresh `atan2` call in
    /// this test rather than by invoking `sun_equatorial_deg` — so a dropped
    /// `cos ε` / `sin ε` term in the production formula is not mirrored in
    /// the expectation and the test fails.
    #[test]
    fn sun_right_ascension_reflects_the_obliquity_rotation() {
        use vedaksha_ephem_core::{bodies::Body, coordinates, obliquity};

        let provider = vedaksha_ephem_core::analytical::AnalyticalProvider;
        let jd = 2_451_668.5; // 2000-05-04

        let pos = coordinates::ecliptic_position(&provider, Body::Sun, jd)
            .expect("mid-quadrant position");
        let eps = obliquity::mean_obliquity(jd);
        // Sun's ecliptic latitude is negligible (~arcseconds), so this omits
        // the `tan(beta) * sin(eps)` term that `sun_equatorial_deg` carries.
        let expected_ra = normalize_degrees(rad_to_deg(libm::atan2(
            libm::sin(pos.longitude) * libm::cos(eps),
            libm::cos(pos.longitude),
        )));

        let (ra, _) = sun_equatorial_deg(&provider, jd).expect("mid-quadrant position");
        let diff = normalize_degrees(ra - expected_ra);
        let diff = if diff > 180.0 { 360.0 - diff } else { diff };
        assert!(
            diff < 0.05,
            "RA = {ra}, expected ~{expected_ra} from the obliquity-rotated formula (diff {diff})"
        );
    }

    /// `horizon_dip_deg` must be 0 at and below sea level (never an imaginary
    /// dip from a negative `sqrt`) and strictly more negative as elevation
    /// grows.
    #[test]
    fn horizon_dip_deg_is_zero_at_sea_level_and_grows_with_elevation() {
        assert_eq!(
            horizon_dip_deg(0.0),
            0.0,
            "dip must be 0 exactly at sea level"
        );
        assert_eq!(
            horizon_dip_deg(-100.0),
            0.0,
            "dip must be 0 below sea level, not imaginary"
        );
        let dip_100 = horizon_dip_deg(100.0);
        let dip_400 = horizon_dip_deg(400.0);
        assert!(
            dip_100 < 0.0,
            "dip at 100 m must be negative, got {dip_100}"
        );
        assert!(
            dip_400 < dip_100,
            "dip must grow more negative with elevation: dip(400)={dip_400} dip(100)={dip_100}"
        );
    }

    /// `sun_rise_set` at a nonzero elevation lowers the effective horizon
    /// (via `horizon_dip_deg`), so the Sun must appear to rise earlier and
    /// set later than at sea level, for the same fake equatorial Sun and
    /// window used above.
    #[test]
    fn sun_rise_set_at_elevation_extends_the_day() {
        let sun = |_jd: f64| Some((0.0_f64, 0.0_f64));
        let sea_level = sun_rise_set(2_451_544.5, 0.0, 0.0, 0.0, &sun);
        let elevated = sun_rise_set(2_451_544.5, 0.0, 0.0, 2000.0, &sun);
        let (r0, s0) = (
            sea_level.rise.expect("sea-level sun must rise"),
            sea_level.set.expect("sea-level sun must set"),
        );
        let (r1, s1) = (
            elevated.rise.expect("elevated sun must rise"),
            elevated.set.expect("elevated sun must set"),
        );
        assert!(
            r1 < r0,
            "elevated rise ({r1}) must be earlier than sea-level rise ({r0})"
        );
        assert!(
            s1 > s0,
            "elevated set ({s1}) must be later than sea-level set ({s0})"
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
