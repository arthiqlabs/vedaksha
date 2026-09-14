// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Rise, set and meridian transit of the **Moon** for a terrestrial observer.
//!
//! [`crate::riseset`] is validated for the Sun and says, with numbers, why its
//! hour-angle walk is not a Moon search: the Moon's right ascension advances
//! ~13 °/day and its declination swings up to ~±28.6° a fortnight, so the
//! Earth-rotation stride and slack that walk is built on stop holding. This
//! module is the search that doc asks for — a bracketing scan over the whole
//! window — made cheap by a bound rather than by a fine step.
//!
//! # How rise and set are found
//!
//! `f(t) = altitude(t) − h₀(t)`, with the Moon's geocentric apparent
//! `(α, δ, Δ)` from the caller's closure. A rise is `f` crossing upward, a set
//! downward.
//!
//! The window is cut into [`SAMPLE_STEP_DAYS`] intervals. An interval whose
//! endpoints share a sign can only hide a crossing if `f` travels to zero and
//! back inside it, which needs `|f(a)| + |f(b)| ≤ L·(b − a)` for any bound `L`
//! on `|df/dt|`. [`MAX_ALTITUDE_RATE_DEG_PER_DAY`] is such a bound, so every
//! interval that fails that inequality is discarded without looking inside,
//! and every other interval is halved until it is [`RESOLUTION_DAYS`] wide. A
//! bracket that narrow with a sign change is refined by regula falsi against
//! fresh Moon positions.
//!
//! **The guarantee that buys:** every crossing is found except one that lies
//! within [`RESOLUTION_DAYS`] (one minute) of another crossing — a graze in
//! which the Moon's limb touches the horizon for under a minute. No sampled
//! search does better, refraction near the horizon is uncertain by more than
//! that, and the scan oracle this module is measured against resolves only
//! half a minute.
//!
//! # How transit is found
//!
//! The local hour angle `H = LST − α` advances at 360.9856 °/day minus the
//! Moon's `dα/dt`, which stays between roughly 342 and 351 °/day, so `H` is
//! monotone and the upper transit (`H = 0`) recurs every ~1.035 d — at most
//! once in a 24-hour window. It is found by Newton steps on `H` at the mean
//! rate [`MEAN_HOUR_ANGLE_RATE_DEG_PER_DAY`], each re-evaluating `α`; the step
//! error contracts by `|1 − rate/mean| ≤ ~0.02` a pass.
//!
//! # Cost
//!
//! See `moon_rise_set_is_far_cheaper_than_a_four_minute_scan`: at mid
//! latitudes a day costs a few dozen Moon evaluations, against the 361 of a
//! four-minute scan before any bisection.
//!
//! # What is measured, and what is not
//!
//! Held to a 30-second bisection scan of the same `f` over a synthetic Moon
//! across latitudes and dates, and over the real `AnalyticalProvider` Moon at a
//! smaller grid — see the tests. Both compare this search with a scan of the
//! SAME model, so they prove the search, not the model: `h₀` and the
//! coordinates are Meeus's, and neither has been compared with an external
//! almanac.
//!
//! Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13 (altitude),
//! Ch. 15 (rising, transit, setting; the Moon's `h₀`), Ch. 47 (horizontal
//! parallax).

use crate::riseset::{RiseSet, geometric_altitude_deg, horizon_dip_deg};
use vedaksha_ephem_core::sidereal_time;
#[cfg(test)]
use vedaksha_math::angle::normalize_degrees;
use vedaksha_math::angle::normalize_degrees_signed;

/// Equatorial radius of the Earth used for the Moon's horizontal parallax,
/// `sin π = R / Δ`, in km.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 47.
pub const EARTH_EQUATORIAL_RADIUS_KM: f64 = 6378.14;

/// Mean atmospheric refraction at the horizon folded into the Moon's `h₀`:
/// 34′.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
const HORIZON_REFRACTION_DEG: f64 = 34.0 / 60.0;

/// Width of the intervals the window is first cut into: one hour.
///
/// Only a cost knob. Correctness comes from [`MAX_ALTITUDE_RATE_DEG_PER_DAY`]
/// and [`RESOLUTION_DAYS`]; a coarser step discards fewer intervals outright
/// and subdivides more.
pub const SAMPLE_STEP_DAYS: f64 = 1.0 / 24.0;

/// Crossings closer together than this are not separated: one minute.
pub const RESOLUTION_DAYS: f64 = 1.0 / 1440.0;

/// An upper bound on `|d(altitude − h₀)/dt|` for the Moon, in degrees per day.
///
/// Altitude is the angular distance of the Moon's direction from the horizon
/// plane, and distance to a great circle changes no faster than the point
/// moves. Relative to the observer the Moon's direction moves at most:
///
/// * 360.9856 °/day from the Earth's rotation (`ω·cos δ ≤ ω`),
/// * plus its own geocentric apparent motion, under 16 °/day at perigee,
/// * while `h₀ = 0.7275·π − 34′` moves under 0.02 °/day as the distance varies.
///
/// That sums to under 377.1 °/day; 400 leaves 6 % of margin. A closure that
/// moves faster than the Moon breaks this bound and with it the guarantee.
pub const MAX_ALTITUDE_RATE_DEG_PER_DAY: f64 = 400.0;

/// Mean advance of the Moon's local hour angle, degrees per UT day: the GMST
/// rate of Meeus eq. 12.4 minus the Moon's mean motion in longitude,
/// 13.176 396 °/day (Meeus Ch. 47). Sets the Newton step for the transit;
/// every pass re-evaluates `α`, so it never sets the answer.
const MEAN_HOUR_ANGLE_RATE_DEG_PER_DAY: f64 = 360.985_647_366_29 - 13.176_396;

/// Newton passes allowed for the transit. The contraction factor is ≤ ~0.02,
/// so from a ≤ 180° starting error the step falls below 1e-9 d in five or six;
/// the loop exits when a pass stops moving the instant.
const TRANSIT_ITERS: u32 = 16;

/// Regula-falsi passes allowed inside a one-minute bracket. Altitude is close
/// to linear across a minute, so three or four suffice; the bound only stops a
/// pathological closure from spinning.
const REFINE_ITERS: u32 = 60;

/// Convergence tolerance for both refinements, in days (~86 µs).
const TIME_TOL_DAYS: f64 = 1e-9;

/// The Moon's standard altitude at rise and set for an observer at sea level,
/// in degrees: `h₀ = 0.7275·π − 34′`, where `π = asin(R / Δ)` is the
/// horizontal parallax at geocentric distance `distance_km`.
///
/// `0.7275·π` is the parallax (which lowers the Moon, `π`) less its
/// semidiameter (`0.2725·π`), so this is the upper limb touching the refracted
/// horizon. Add [`horizon_dip_deg`] for an elevated observer.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15 and Ch. 47.
#[must_use]
pub fn moon_standard_altitude_deg(distance_km: f64) -> f64 {
    let parallax_deg = libm::asin(EARTH_EQUATORIAL_RADIUS_KM / distance_km).to_degrees();
    0.7275 * parallax_deg - HORIZON_REFRACTION_DEG
}

/// The Moon's geocentric apparent `(right_ascension_deg, declination_deg,
/// distance_km)` at `jd_ut` from `provider` — the closure
/// [`moon_rise_set`] wants.
///
/// Rotates [`vedaksha_ephem_core::coordinates::ecliptic_position`]'s apparent
/// ecliptic position onto the equator with the **true** obliquity, the inverse
/// of the rotation that pipeline applied.
#[must_use]
pub fn moon_equatorial(
    provider: &dyn vedaksha_ephem_core::jpl::EphemerisProvider,
    jd_ut: f64,
) -> Option<(f64, f64, f64)> {
    use vedaksha_ephem_core::{bodies::Body, coordinates, delta_t, jpl::AU_KM};

    let pos = coordinates::ecliptic_position(provider, Body::Moon, jd_ut).ok()?;
    let eps = coordinates::frame_for(delta_t::ut1_to_tt(jd_ut)).true_obliquity();
    let (ra, dec) = coordinates::ecliptic_to_equatorial_deg(
        pos.longitude.to_degrees(),
        pos.latitude.to_degrees(),
        eps.to_degrees(),
    );
    Some((ra, dec, pos.distance * AU_KM))
}

/// Moonrise, moonset and the Moon's upper-meridian transit in the 24 hours
/// from `jd_ut_day_start`, as Julian Days (UT).
///
/// `moon(jd_ut)` returns the Moon's geocentric apparent `(right_ascension_deg,
/// declination_deg, distance_km)` — [`moon_equatorial`] over a provider, or
/// anything equivalent — or `None` if unavailable. Rise and set are the upper
/// limb at the refracted horizon ([`moon_standard_altitude_deg`]) lowered by
/// the dip for `elevation_m`.
///
/// Each field is the FIRST such event in `[jd_ut_day_start,
/// jd_ut_day_start + 1)`, or `None` if there is none — which happens once a
/// month for each event anywhere, since the Moon's day is ~24 h 50 m, and for
/// days at a time at high latitude. As with [`crate::riseset::rise_set`], `rise`
/// and `set` are not in chronological order: a set can precede the rise.
///
/// A `None` from the closure ends the rise/set search at that instant: an
/// event already found before it stands (it is still the first), and any
/// event not yet found is `None` rather than one taken from beyond the gap,
/// which might not be the first. A `None` during the transit search makes
/// `transit` `None`.
///
/// See the module documentation for the method and its one-minute resolution
/// guarantee.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
#[must_use]
pub fn moon_rise_set(
    jd_ut_day_start: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    moon: &(dyn Fn(f64) -> Option<(f64, f64, f64)> + Sync),
) -> RiseSet {
    let dip = horizon_dip_deg(elevation_m);
    let f = |t: f64| -> Option<f64> {
        let (ra, dec, dist) = moon(t)?;
        Some(
            geometric_altitude_deg(ra, dec, t, lat_deg, lon_deg_east)
                - (moon_standard_altitude_deg(dist) + dip),
        )
    };

    let end = jd_ut_day_start + 1.0;
    let mut found = Crossings::default();
    // Truncation is exact: 1 / SAMPLE_STEP_DAYS is a small whole number.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = libm::round(1.0 / SAMPLE_STEP_DAYS) as u32;
    let mut a = jd_ut_day_start;
    let mut fa = f(a);
    for k in 1..=steps {
        if found.done() {
            break;
        }
        let b = if k == steps {
            end
        } else {
            jd_ut_day_start + f64::from(k) * SAMPLE_STEP_DAYS
        };
        let fb = f(b);
        let (Some(va), Some(vb)) = (fa, fb) else {
            found.gap = true;
            break;
        };
        bracket(&f, a, va, b, vb, &mut found);
        a = b;
        fa = fb;
    }

    RiseSet {
        rise: found.rise.filter(|&t| t < end),
        set: found.set.filter(|&t| t < end),
        transit: upper_transit(jd_ut_day_start, lon_deg_east, moon),
    }
}

#[derive(Default)]
struct Crossings {
    rise: Option<f64>,
    set: Option<f64>,
    /// The closure returned `None`; nothing later in the window may be used.
    gap: bool,
}

impl Crossings {
    fn done(&self) -> bool {
        self.gap || (self.rise.is_some() && self.set.is_some())
    }
}

/// `true` when the Moon counts as above the target altitude. Zero is above, so
/// a rise is `below → above` and a set is `above → below`; the scan oracle in
/// the tests uses the same predicate.
fn above(v: f64) -> bool {
    v >= 0.0
}

/// Find the crossings of `f` in `[a, b]`, left to right, recording the first
/// rise and first set. `fa`/`fb` are `f` at the ends.
fn bracket(
    f: &dyn Fn(f64) -> Option<f64>,
    a: f64,
    fa: f64,
    b: f64,
    fb: f64,
    found: &mut Crossings,
) {
    if found.done() {
        return;
    }
    let width = b - a;
    let sign_change = above(fa) != above(fb);
    if !sign_change && libm::fabs(fa) + libm::fabs(fb) > MAX_ALTITUDE_RATE_DEG_PER_DAY * width {
        return; // no room to reach zero and come back
    }
    if width <= RESOLUTION_DAYS {
        if sign_change {
            let slot = if above(fb) {
                &mut found.rise
            } else {
                &mut found.set
            };
            if slot.is_none() {
                *slot = refine(f, a, fa, b, fb);
                found.gap = slot.is_none();
            }
        }
        return; // a same-sign sliver narrower than the resolution: a graze
    }
    let m = 0.5 * (a + b);
    let Some(fm) = f(m) else {
        found.gap = true;
        return;
    };
    bracket(f, a, fa, m, fm, found);
    bracket(f, m, fm, b, fb, found);
}

/// Regula falsi with the Illinois modification inside a sign-changing bracket
/// no wider than [`RESOLUTION_DAYS`].
fn refine(f: &dyn Fn(f64) -> Option<f64>, a: f64, fa: f64, b: f64, fb: f64) -> Option<f64> {
    let (mut a, mut fa, mut b, mut fb) = (a, fa, b, fb);
    let mut side = 0_i8;
    for _ in 0..REFINE_ITERS {
        let t = (a * fb - b * fa) / (fb - fa);
        let t = if t > a && t < b { t } else { 0.5 * (a + b) };
        if b - a < TIME_TOL_DAYS {
            return Some(t);
        }
        let ft = f(t)?;
        if above(ft) == above(fa) {
            a = t;
            fa = ft;
            if side == -1 {
                fb *= 0.5;
            }
            side = -1;
        } else {
            b = t;
            fb = ft;
            if side == 1 {
                fa *= 0.5;
            }
            side = 1;
        }
        if libm::fabs(ft) == 0.0 {
            return Some(t);
        }
    }
    Some(0.5 * (a + b))
}

/// Signed local hour angle of the Moon in degrees, `[−180, 180)`.
fn hour_angle_deg(
    t: f64,
    lon_deg_east: f64,
    moon: &(dyn Fn(f64) -> Option<(f64, f64, f64)> + Sync),
) -> Option<f64> {
    let (ra, _, _) = moon(t)?;
    let lst = sidereal_time::gmst(t).to_degrees() + lon_deg_east;
    Some(normalize_degrees_signed(lst - ra))
}

/// The Moon's upper transit in `[start, start + 1)`, if any.
fn upper_transit(
    start: f64,
    lon_deg_east: f64,
    moon: &(dyn Fn(f64) -> Option<(f64, f64, f64)> + Sync),
) -> Option<f64> {
    let end = start + 1.0;
    // Forward gap from the hour angle at the start to the next H = 0.
    let h_start = hour_angle_deg(start, lon_deg_east, moon)?;
    let mut t = start + (-h_start).rem_euclid(360.0) / MEAN_HOUR_ANGLE_RATE_DEG_PER_DAY;
    for _ in 0..TRANSIT_ITERS {
        let step = -hour_angle_deg(t, lon_deg_east, moon)? / MEAN_HOUR_ANGLE_RATE_DEG_PER_DAY;
        t += step;
        if libm::fabs(step) < TIME_TOL_DAYS {
            break;
        }
    }
    // The seed can land a hair on the wrong side of `start` when a transit
    // sits at the very start; a converged instant decides membership.
    (t >= start && t < end).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicU32, Ordering};

    /// A synthetic Moon cheap enough for dense sweeps: Meeus Ch. 47's mean
    /// elements with the largest term of each of longitude (6.289° equation of
    /// centre), latitude (5.128°) and distance (20 905 km), on a fixed 23.44°
    /// obliquity. Declination swings about ±28.6° at its largest, the
    /// right-ascension rate varies with it, and the parallax varies across the
    /// month — every property the search's bounds depend on.
    fn synthetic_moon(t: f64) -> Option<(f64, f64, f64)> {
        let d = t - 2_451_545.0;
        let l = 218.316_447_7 + 13.176_396_48 * d;
        let m = (134.963_396_4 + 13.064_992_95 * d).to_radians();
        let f = (93.272_095 + 13.229_350_24 * d).to_radians();
        let lon = l + 6.289 * libm::sin(m);
        let lat = 5.128 * libm::sin(f);
        let dist = 385_000.56 - 20_905.355 * libm::cos(m);
        let (ra, dec) =
            vedaksha_ephem_core::coordinates::ecliptic_to_equatorial_deg(lon, lat, 23.44);
        Some((ra, dec, dist))
    }

    type Events = (Option<f64>, Option<f64>, Option<f64>);

    /// The oracle: the Moon sampled every `step_days` across the window, each
    /// sign change of `f` (rise/set) and each upward zero of the hour angle
    /// (transit) bisected. Same `f`, same `above` predicate, no bound, no
    /// skipping.
    fn scan_oracle(
        start: f64,
        lat: f64,
        lon: f64,
        elevation_m: f64,
        step_days: f64,
        moon: &(dyn Fn(f64) -> Option<(f64, f64, f64)> + Sync),
    ) -> Events {
        let dip = horizon_dip_deg(elevation_m);
        let both = |t: f64| {
            let (ra, dec, dist) = moon(t).unwrap();
            let alt = geometric_altitude_deg(ra, dec, t, lat, lon)
                - (moon_standard_altitude_deg(dist) + dip);
            let lst = sidereal_time::gmst(t).to_degrees() + lon;
            (alt, normalize_degrees_signed(lst - ra))
        };
        let bisect = |mut a: f64, mut b: f64, pick: &dyn Fn((f64, f64)) -> bool| {
            let side = pick(both(a));
            for _ in 0..60 {
                let m = 0.5 * (a + b);
                if pick(both(m)) == side {
                    a = m;
                } else {
                    b = m;
                }
            }
            0.5 * (a + b)
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let steps = libm::round(1.0 / step_days) as u32;
        let (mut rise, mut set, mut transit) = (None, None, None);
        let (mut a, mut va) = (start, both(start));
        for k in 1..=steps {
            let b = start + f64::from(k) * step_days;
            let vb = both(b);
            if above(va.0) != above(vb.0) {
                let slot = if above(vb.0) { &mut rise } else { &mut set };
                if slot.is_none() {
                    *slot = Some(bisect(a, b, &|v| above(v.0)));
                }
            }
            // H crosses 0 upward — not the ±180 wrap, where it jumps down.
            if transit.is_none() && va.1 < 0.0 && vb.1 >= 0.0 && vb.1 < 90.0 {
                transit = Some(bisect(a, b, &|v| v.1 >= 0.0));
            }
            (a, va) = (b, vb);
        }
        let end = start + 1.0;
        (
            rise.filter(|&t| t < end),
            set.filter(|&t| t < end),
            transit.filter(|&t| t < end),
        )
    }

    /// Presence must agree exactly; instants to within this. Measured worst
    /// is one ULP (4.66e-10 d); the bound is the refinement tolerance with room.
    const AGREEMENT_TOL_DAYS: f64 = 1e-8;

    #[derive(Default)]
    struct Tally {
        comparisons: u32,
        presence: u32,
        worst: f64,
        first: Option<String>,
        max_calls: u32,
    }

    impl Tally {
        fn add(&mut self, at: &str, ours: RiseSet, oracle: Events) {
            for (label, x, y) in [
                ("rise", ours.rise, oracle.0),
                ("set", ours.set, oracle.1),
                ("transit", ours.transit, oracle.2),
            ] {
                self.comparisons += 1;
                match (x, y) {
                    (Some(x), Some(y)) => self.worst = self.worst.max(libm::fabs(x - y)),
                    (None, None) => {}
                    _ => {
                        self.presence += 1;
                        self.first.get_or_insert_with(|| {
                            format!("{label} {at}: ours {x:?} oracle {y:?}")
                        });
                    }
                }
            }
        }

        fn assert_clean(&self, what: &str) {
            assert_eq!(
                self.presence, 0,
                "{what}: {} presence disagreements in {} comparisons; first: {:?}",
                self.presence, self.comparisons, self.first
            );
            assert!(
                self.worst < AGREEMENT_TOL_DAYS,
                "{what}: worst instant gap {} d",
                self.worst
            );
        }
    }

    /// Sweep the synthetic Moon over `lats` × 4 longitudes × 30 consecutive
    /// days (a lunar month: every declination, every no-rise and no-set day)
    /// × `elevations`, against a 30-second oracle.
    fn synthetic_sweep(lats: &[f64], elevations: &[f64]) -> Tally {
        let calls = AtomicU32::new(0);
        let moon = |t: f64| {
            calls.fetch_add(1, Ordering::Relaxed);
            synthetic_moon(t)
        };
        let mut tally = Tally::default();
        for &lat in lats {
            for lon_i in 0..4 {
                let lon = -180.0 + f64::from(lon_i) * 90.0 + 7.0;
                for day in 0..30 {
                    let start = 2_460_676.5 + f64::from(day);
                    for &elev in elevations {
                        calls.store(0, Ordering::Relaxed);
                        let ours = moon_rise_set(start, lat, lon, elev, &moon);
                        tally.max_calls = tally.max_calls.max(calls.load(Ordering::Relaxed));
                        let oracle =
                            scan_oracle(start, lat, lon, elev, 0.5 / 1440.0, &synthetic_moon);
                        tally.add(
                            &format!("lat {lat} lon {lon} jd {start} elev {elev}"),
                            ours,
                            oracle,
                        );
                    }
                }
            }
        }
        tally
    }

    /// Routine tier: latitudes −60..60 every 10°, two elevations — 9 360
    /// comparisons.
    #[test]
    fn synthetic_moon_agrees_with_the_scan_oracle() {
        let lats: Vec<f64> = (-6..=6).map(|i| f64::from(i) * 10.0).collect();
        let tally = synthetic_sweep(&lats, &[0.0, 2500.0]);
        assert_eq!(tally.comparisons, 13 * 4 * 30 * 2 * 3);
        tally.assert_clean("synthetic, |lat| ≤ 60");
    }

    /// Weekly tier: every whole degree from −80 to 80 at sea level — 57 960
    /// comparisons, including the high-latitude months where the Moon stays up
    /// or down for days and grazes the horizon.
    ///
    /// Measured 2026-09-14 (aarch64, release): zero presence disagreements,
    /// worst instant gap 4.66e-10 d (one ULP), at most 204 Moon evaluations in
    /// a day (lat 71–80); ≤ 115 at |lat| ≤ 60, mean ~55.
    #[test]
    #[ignore = "dense sweep, ~3 s in --release; runs in Full Validation"]
    fn synthetic_moon_agrees_with_the_scan_oracle_to_80_degrees() {
        let lats: Vec<f64> = (-80..=80).map(f64::from).collect();
        let tally = synthetic_sweep(&lats, &[0.0]);
        assert_eq!(tally.comparisons, 161 * 4 * 30 * 3);
        tally.assert_clean("synthetic, |lat| ≤ 80");
        assert!(
            tally.max_calls <= 250,
            "{} Moon evaluations in a day",
            tally.max_calls
        );
    }

    const OBSERVERS: [(f64, f64, f64); 5] = [
        (28.61, 77.21, 216.0),
        (13.08, 80.27, 6.0),
        (51.48, 0.0, 0.0),
        (-33.87, 151.21, 0.0),
        (59.91, 10.75, 20.0),
    ];

    fn real_sweep(observers: &[(f64, f64, f64)], days: &[f64]) -> Tally {
        let provider = vedaksha_ephem_core::analytical::AnalyticalProvider::new();
        let moon = |t: f64| moon_equatorial(&provider, t);
        let mut tally = Tally::default();
        for &(lat, lon, elev) in observers {
            for &day in days {
                let start = 2_460_676.5 + day;
                let ours = moon_rise_set(start, lat, lon, elev, &moon);
                let oracle = scan_oracle(start, lat, lon, elev, 1.0 / 1440.0, &moon);
                tally.add(&format!("lat {lat} lon {lon} jd {start}"), ours, oracle);
            }
        }
        tally
    }

    /// Routine tier over the real `AnalyticalProvider` Moon: two observers,
    /// two days, one-minute oracle.
    #[test]
    fn real_moon_agrees_with_the_scan_oracle() {
        let tally = real_sweep(&OBSERVERS[..2], &[0.0, 13.0]);
        assert_eq!(tally.comparisons, 2 * 2 * 3);
        tally.assert_clean("real Moon");
    }

    /// Weekly tier over the real Moon: five observers × six days across a month.
    #[test]
    #[ignore = "~180 000 real Moon evaluations; runs in Full Validation"]
    fn real_moon_agrees_with_the_scan_oracle_over_a_month() {
        let tally = real_sweep(&OBSERVERS, &[0.0, 5.0, 10.0, 15.0, 20.0, 25.0]);
        assert_eq!(tally.comparisons, 5 * 6 * 3);
        tally.assert_clean("real Moon, month");
    }

    /// `h₀` itself, which every other test shares with its oracle and so
    /// cannot check. At Δ = 385 000 km: π = asin(6378.14 / 385 000) =
    /// 0.949 21°, 0.7275·π = 0.690 55°, less 34′ (0.566 67°) = +0.123 88°.
    /// At perigee-like 356 500 km: π = 1.025 12°, h₀ = +0.179 11°.
    #[test]
    fn standard_altitude_matches_meeus_at_two_distances() {
        assert!((moon_standard_altitude_deg(385_000.0) - 0.123_88).abs() < 5e-5);
        assert!((moon_standard_altitude_deg(356_500.0) - 0.179_11).abs() < 5e-5);
    }

    /// A closure gap must not let a later crossing stand in for the first.
    /// Over a month at lat 28.6 the closure is withheld for 2.4 hours from
    /// mid-window and valid again after: every event the gapped search reports
    /// must equal the ungapped one and lie before the gap, and every event the
    /// ungapped search found from the gap onward must come back `None` — a
    /// search that stepped over the gap would report those.
    #[test]
    fn a_closure_gap_ends_the_search_instead_of_skipping_it() {
        let mut hidden = 0;
        for day in 0..30 {
            let start = 2_460_676.5 + f64::from(day);
            let gap_from = start + 0.5;
            let gapped = move |t: f64| {
                if (gap_from..gap_from + 0.1).contains(&t) {
                    None
                } else {
                    synthetic_moon(t)
                }
            };
            let full = moon_rise_set(start, 28.61, 77.21, 0.0, &synthetic_moon);
            let cut = moon_rise_set(start, 28.61, 77.21, 0.0, &gapped);
            for (label, f, c) in [("rise", full.rise, cut.rise), ("set", full.set, cut.set)] {
                match c {
                    Some(t) => {
                        assert_eq!(Some(t), f, "{label} day {day}");
                        assert!(t < gap_from, "{label} day {day} reported past the gap");
                    }
                    None => {
                        if f.is_some_and(|t| t >= gap_from) {
                            hidden += 1;
                        }
                    }
                }
                if let Some(t) = f
                    && t >= gap_from
                {
                    assert!(c.is_none(), "{label} day {day}: {c:?} from beyond the gap");
                }
            }
        }
        assert!(
            hidden >= 10,
            "only {hidden} events fell after the gap; the test has no teeth"
        );
    }

    /// A graze the hourly samples cannot see. A body at fixed position,
    /// observed from lat 60, culminates right at the horizon: its transit
    /// altitude sits `margin` degrees above `h₀`, so it is up for a few
    /// minutes either side of a transit inside an hour — both hourly samples
    /// around it are below, and only the subdivision finds the pair. The
    /// 0.001° case is up for only minutes; red-proved 2026-09-14, a search that
    /// stops subdividing at 15 minutes misses it (one at 7.5 minutes happens to
    /// land a subdivision point inside it). With `margin` negative it never
    /// clears, and neither side may report an event. Held to the 30-second oracle, and to the expected
    /// shape: rise before transit before set, all inside that hour.
    #[test]
    fn a_graze_between_hourly_samples_is_found() {
        let (lat, lon, dist) = (60.0, 0.0, 385_000.0);
        let start = 2_460_676.5;
        let h0 = moon_standard_altitude_deg(dist);
        for (transit_hour, margin, expect_events) in [
            (5.37, 0.02, true),
            (5.37, 0.05, true),
            (5.30, 0.001, true),
            (5.37, -0.02, false),
        ] {
            let transit_at = start + transit_hour / 24.0;
            let ra = normalize_degrees(sidereal_time::gmst(transit_at).to_degrees() + lon);
            // Transit altitude = 90 − (φ − δ) for a body south of the zenith.
            let dec = lat - 90.0 + h0 + margin;
            let body = move |_t: f64| Some((ra, dec, dist));
            let ours = moon_rise_set(start, lat, lon, 0.0, &body);
            let oracle = scan_oracle(start, lat, lon, 0.0, 0.5 / 1440.0, &body);
            let mut tally = Tally::default();
            tally.add(&format!("graze margin {margin}"), ours, oracle);
            tally.assert_clean("graze");
            if expect_events {
                let (r, s) = (ours.rise.expect("rise"), ours.set.expect("set"));
                assert!(
                    r < transit_at && transit_at < s && s - r < 1.0 / 24.0,
                    "margin {margin}: rise {r} set {s} transit {transit_at}"
                );
                let hour = (start + 5.0 / 24.0, start + 6.0 / 24.0);
                assert!(
                    r > hour.0 && s < hour.1,
                    "the graze must fall between two samples"
                );
            } else {
                assert!(
                    ours.rise.is_none() && ours.set.is_none(),
                    "margin {margin}: {ours:?}"
                );
            }
        }
    }

    /// The search exists to replace a four-minute scan (361 evaluations a day
    /// before any bisection). Count the closure calls over a month at mid
    /// latitude; measured worst 2026-09-14 was 115 at |lat| ≤ 60.
    #[test]
    fn moon_rise_set_is_far_cheaper_than_a_four_minute_scan() {
        let tally = synthetic_sweep(&[28.61], &[0.0]);
        tally.assert_clean("cost sweep");
        assert!(
            tally.max_calls <= 120,
            "worst day took {} Moon evaluations",
            tally.max_calls
        );
    }
}
