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
use vedaksha_math::angle::{deg_to_rad, normalize_degrees, normalize_degrees_signed, rad_to_deg};

/// Standard geometric altitude of the Sun's centre at rise/set: −0°50′,
/// covering mean atmospheric refraction (34′) plus the solar semidiameter
/// (16′).
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
pub const SUN_STANDARD_ALTITUDE_DEG: f64 = -50.0 / 60.0;

/// Rate at which a body's local hour angle advances, in degrees per **UT**
/// day.
///
/// This is the coefficient of `(JD − 2451545.0)` in Meeus eq. 12.4 — the GMST
/// rate — copied from the single place this engine implements that equation,
/// `vedaksha_ephem_core::sidereal_time::gmst`, so the two cannot drift apart.
/// The local hour angle is `H = LST − α`, so its true rate is this minus
/// `dα/dt`; for the Sun that is at most ~1.02 °/day (≈0.28 %), and for a body
/// held at fixed right ascension it is exactly zero. The residual is absorbed
/// by the iteration in [`refine_event`], which re-evaluates `α` every pass —
/// this constant only sets the step size, never the answer.
const HOUR_ANGLE_RATE_DEG_PER_DAY: f64 = 360.985_647_366_29;

/// One full turn of the local hour angle, in days:
/// `360 / 360.98564736629` = 0.9972695663290739 d. For a body at fixed right
/// ascension this is exactly the rise-to-rise interval; for the real Sun it is
/// the same quantity before the `dα/dt` correction, and is used only as a
/// stride when a search has to skip to the next rotation.
const ROTATION_DAYS: f64 = 360.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;

/// Half of [`ROTATION_DAYS`] — the offset from a body's upper transit to its
/// LOWER transit, which is the second seed
/// [`event_on_transits_rotation`] tries, and the slack
/// [`search_rise`] allows on its transit walk because a rise sits at most this
/// far before the transit that names its rotation.
const HALF_ROTATION_DAYS: f64 = 180.0 / HOUR_ANGLE_RATE_DEG_PER_DAY;

/// Iterations of the Meeus Ch. 15 correction in [`refine_event`].
///
/// The iteration is a contraction: one pass multiplies the time error by
/// `|dα/dt − dH₀/dt| / 360.98564736629`. For the Sun `dα/dt ≤ ~1.02 °/day`
/// and `dH₀/dt` stays within a few °/day except in the grazing regime, so the
/// factor is ~0.003 and each pass gains roughly 2.5 decimal digits. The
/// shortest-signed fold caps the starting error at half a rotation (0.4986 d)
/// whatever seed a caller hands in, so even at a contraction factor of 0.1 —
/// far worse than anything away from the grazing regime — eight passes already
/// reach the ~4.7e-10 d ULP at the Julian Days this engine serves. The loop
/// exits early, typically after five or six passes, the moment a pass no
/// longer moves the instant.
///
/// 24 is deliberately generous, because exhausting this count is now a
/// VERDICT, not a fallback: [`refine_event`] returns `None` rather than its
/// last iterate, so the count has to be high enough that "did not converge"
/// means "there is no crossing here" and never "the budget was too tight".
/// Iterations past convergence cost nothing — the loop has already returned.
const REFINE_ITERS: u32 = 24;

/// How far from its anchor instant [`previous_rise`] / [`next_rise`] will look
/// before giving up, in days.
///
/// Consecutive rises of a body are one rotation apart — one solar day for the
/// real Sun, one *sidereal* day ([`ROTATION_DAYS`]) for a body held at fixed
/// right ascension — so a single day of search always suffices away from the
/// polar circles. Four days keeps margin for the latitudes just inside those
/// circles, where a body can fail to clear the horizon on one rotation and
/// clear it on the next. Unchanged from the bound the 5-minute scan this
/// replaced enforced as `4 · 24 · 60 / 5` = 1152 steps, so the two search
/// horizons are identical and the scan remains a valid oracle.
const RISE_SEARCH_DAYS: f64 = 4.0;

/// Maximum candidate rotations [`search_rise`] examines.
///
/// Each round advances the transit that names the rotation by exactly one
/// [`ROTATION_DAYS`], and the walk may start up to half a rotation from the
/// anchor, so `⌈(4 + 0.4986) / 0.9972695663⌉ = 5` rounds already cover
/// [`RISE_SEARCH_DAYS`]. 8 is that with margin; the day bound inside the loop —
/// not this count — is what actually terminates the search.
const RISE_SEARCH_ROUNDS: u32 = 8;

/// Slack allowed past the closing edge of [`rise_set`]'s 24-hour window, in
/// days. Matches the `+ 1e-9` the 5-minute scan carried on its own loop
/// condition, so a root sitting on the window's last representable instant is
/// accepted by both implementations alike.
const WINDOW_EDGE_SLACK_DAYS: f64 = 1e-9;

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

/// Which horizon or meridian event a search is converging on. Each is a
/// *local hour angle* the body must reach, which is what makes the search
/// analytic: `Rise` is `−H₀`, `Set` is `+H₀`, `Transit` is `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    /// Upward crossing of the target altitude — local hour angle `−H₀`.
    Rise,
    /// Downward crossing of the target altitude — local hour angle `+H₀`.
    Set,
    /// Upper meridian transit — local hour angle `0`.
    Transit,
}

/// The local hour angle `H₀`, in degrees [0, 180], at which a body of
/// declination `dec_deg` reaches altitude `h0_deg` for an observer at
/// `lat_deg` — Meeus eq. 15.1:
///
/// `cos H₀ = (sin h₀ − sin φ · sin δ) / (cos φ · cos δ)`
///
/// Returns `None` when `cos H₀` falls outside [−1, 1]: the body never reaches
/// that altitude on this rotation, which is exactly polar day (the body never
/// descends to it) and polar night (never ascends to it). **No clamp is
/// applied** — clamping would turn "no crossing exists" into a fabricated
/// instant, and this out-of-range test is the only signal that distinguishes
/// the two. The same test also rejects the NaN produced when
/// `cos φ · cos δ = 0` (observer exactly at a geographic pole, or body exactly
/// at a celestial pole), because a range check is false for NaN.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15, eq. 15.1.
fn rise_hour_angle_deg(lat_deg: f64, dec_deg: f64, h0_deg: f64) -> Option<f64> {
    let (phi, dec) = (deg_to_rad(lat_deg), deg_to_rad(dec_deg));
    let cos_h0 = (libm::sin(deg_to_rad(h0_deg)) - libm::sin(phi) * libm::sin(dec))
        / (libm::cos(phi) * libm::cos(dec));
    if (-1.0..=1.0).contains(&cos_h0) {
        Some(rad_to_deg(libm::acos(cos_h0)))
    } else {
        None
    }
}

/// The local hour angle, in degrees [0, 360), the body must reach for `event`.
///
/// `None` propagates [`rise_hour_angle_deg`]'s polar signal for `Rise` and
/// `Set`. `Transit` is always defined — a body that never rises still crosses
/// the meridian once a rotation — so it never returns `None` here.
fn target_hour_angle_deg(event: Event, lat_deg: f64, dec_deg: f64, h0_deg: f64) -> Option<f64> {
    match event {
        Event::Transit => Some(0.0),
        Event::Rise => {
            rise_hour_angle_deg(lat_deg, dec_deg, h0_deg).map(|h0| normalize_degrees(-h0))
        }
        Event::Set => rise_hour_angle_deg(lat_deg, dec_deg, h0_deg),
    }
}

/// How far the body's local hour angle still has to advance at `jd_ut` to
/// reach `event`, in degrees [0, 360) — the *forward* gap, since the hour
/// angle only ever increases.
///
/// One `equatorial` evaluation per call; this is the search's entire cost.
fn hour_angle_gap_deg(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    event: Event,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let (ra, dec) = equatorial(jd_ut)?;
    let target = target_hour_angle_deg(event, lat_deg, dec, h0_deg)?;
    let current = normalize_degrees(local_sidereal_degrees(jd_ut, lon_deg_east) - ra);
    Some(normalize_degrees(target - current))
}

/// Converge on the occurrence of `event` NEAREST to `t0`, by Meeus's Ch. 15
/// correction: re-evaluate the body's `(α, δ)` at the current estimate,
/// recompute the target hour angle from eq. 15.1, and step by the shortest
/// signed hour-angle gap divided by the rotation rate.
///
/// Folding the gap to [−180, 180) is what "nearest" means, so the caller owns
/// the choice of rotation and hands in a `t0` that already sits on it:
/// [`rise_set`] takes a *directed* (unfolded) step from the window's opening
/// instant, and [`search_rise`] seeds from the rotation's own transit (see
/// [`event_on_transits_rotation`]).
///
/// The iteration is a contraction with factor `|dα/dt − dH₀/dt| / 360.9856`
/// (see [`REFINE_ITERS`]), so it converges to the `f64` nearest the true root
/// — the same value a bisection converges to — and then stops moving.
///
/// # An unconverged iterate is NOT an answer
///
/// The loop returns a value **only** when it reaches its exact fixed point.
/// Running out of iterations returns `None`, because the last iterate is not a
/// crossing and reporting it would be a fabricated instant. That is not a
/// theoretical concern: at lat 89 N, lon 60, JD 2,433,556.4375 an earlier
/// draft returned JD 2,433,552.8231591503 where the Sun never reaches `h₀` at
/// all — measured `rel_alt` there is −0.00122°, and a one-minute scan of the
/// surrounding four days finds no crossing whatsoever. The scan oracle
/// correctly said `None`; the sweep caught the difference.
///
/// The cause is that `cos H₀` at lat 89 moves ~1 per DEGREE of declination
/// (`d cos H₀/dδ ≈ −tan φ ≈ −57` per radian), so with the Sun's ~0.39 °/day
/// drift the crossing can cease to exist within minutes of the instant where
/// eq. 15.1 still admits it. There is then no fixed point to find, and the
/// only honest report is that there is no rise.
///
/// Returns `None` when the iteration does not reach its fixed point, as soon
/// as `equatorial` cannot supply a position, or when the event ceases to exist
/// on the rotation being examined (`cos H₀` out of range).
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
fn refine_event(
    t0: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    event: Event,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let mut t = t0;
    // The iterate before `t`, and the size of the correction that produced
    // `t`. `NAN` compares unequal to everything, so the first pass cannot
    // spuriously report a cycle.
    let mut previous = f64::NAN;
    let mut previous_correction = f64::INFINITY;

    for _ in 0..REFINE_ITERS {
        let gap = hour_angle_gap_deg(t, lat_deg, lon_deg_east, h0_deg, event, equatorial)?;
        let correction = normalize_degrees_signed(gap);
        let next = t + correction / HOUR_ANGLE_RATE_DEG_PER_DAY;

        // Exact float comparisons, deliberately: these are fixed-point and
        // cycle tests, not approximate ones, and neither needs a tolerance.
        #[allow(clippy::float_cmp)]
        {
            // Converged. The correction was smaller than half an ULP at this
            // magnitude, so the iteration has landed on the nearest
            // representable instant. This is also what makes
            // `previous_rise(r) == r` bit-exact for a rise `r`: the gap at `r`
            // is the residual of a root already correct to within half an ULP,
            // so the first correction cannot move it.
            if next == t {
                return Some(t);
            }
            // A TWO-CYCLE between adjacent representable instants. It happens
            // when the true root sits within a few percent of the midpoint
            // between two neighbouring `f64`s: the correction from each is a
            // shade over half an ULP, so each rounds onto the other and
            // NEITHER is a fixed point. Measured, `flat_sun` at lat −85,
            // lon −150, 3650 m, anchored on JD 2,433,312.9375: the iteration
            // alternates forever between 2433312.220053467434 and
            // 2433312.220053467900, one ULP apart, with corrections
            // +2.3311e-10 d and −2.3385e-10 d against a half-ULP of
            // 2.3283e-10 d.
            //
            // Both are the root to within one ULP (4.0e-5 s) and no arithmetic
            // at this precision can say which is nearer, so take the one with
            // the SMALLER residual and let the sweep's tolerance — set at two
            // ULP, above the measured maximum — absorb the last bit. The rule
            // is a deterministic function of the cycle, so it is idempotent:
            // re-entering from either member returns the same instant, which
            // is what `previous_rise(r) == r` needs.
            if next == previous {
                return Some(if libm::fabs(correction) <= previous_correction {
                    t
                } else {
                    previous
                });
            }
        }

        previous = t;
        previous_correction = libm::fabs(correction);
        t = next;
    }
    // Out of iterations, and not in a cycle either: there is no crossing to
    // report here. See the note above — returning `t` fabricates one.
    None
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
/// ⚠️ A second consequence of "first event in the window": when the body's
/// rotation period is SHORTER than the 24 h scanned, two rises fall inside one
/// window and only the earlier is reported. That is always the case for a body
/// at fixed right ascension (one sidereal day, 0.99727 d) and true for the real
/// Sun over roughly half the year. **Do not enumerate successive rises by
/// stepping this window along the calendar** — the second rise of a window is
/// invisible to that idiom, so the walk settles on a stale event. Use
/// [`previous_rise`] / [`next_rise`], which anchor on the instant instead of on
/// a calendar boundary and cannot skip a crossing.
///
/// # How it is found
///
/// Analytically, not by scanning. Each of the three events is a *local hour
/// angle* the body must reach — `−H₀` for the rise, `+H₀` for the set, `0`
/// for the transit, with `H₀` from Meeus eq. 15.1 — so the first occurrence
/// inside the window is one forward step of `gap / 360.98564736629` days from
/// `jd_ut_day_start`, refined by [`refine_event`]. The `#[cfg(test)]`
/// `scan_reference` module keeps the 5-minute scan this replaced, and the
/// sweep tests below hold the two to agreement; see the note on
/// [`previous_rise`] for the measured bound.
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
    let window_end = jd_ut_day_start + 1.0 + WINDOW_EDGE_SLACK_DAYS;

    let first_in_window = |event: Event| -> Option<f64> {
        // The DIRECTED first estimate: the hour-angle gap is already in
        // [0, 360), i.e. measured forward from the window's opening instant,
        // so this lands inside `[start, start + 0.99727)` by construction and
        // can never name an event on an earlier rotation.
        //
        // When `cos H₀` is out of range at the window's opening instant the
        // event may still occur later inside the window — the declination
        // drifts, and at the polar transitions that is exactly what happens —
        // so fall back to seeding from the transits of the (at most two)
        // rotations the 24-hour window touches, where the declination that
        // decides the boundary is the one actually sampled. See
        // [`event_on_transits_rotation`].
        let root = if let Some(gap) = hour_angle_gap_deg(
            jd_ut_day_start,
            lat_deg,
            lon_deg_east,
            h0_deg,
            event,
            equatorial,
        ) {
            refine_event(
                jd_ut_day_start + gap / HOUR_ANGLE_RATE_DEG_PER_DAY,
                lat_deg,
                lon_deg_east,
                h0_deg,
                event,
                equatorial,
            )?
        } else {
            let transit = refine_event(
                jd_ut_day_start,
                lat_deg,
                lon_deg_east,
                h0_deg,
                Event::Transit,
                equatorial,
            )?;
            event_on_transits_rotation(transit, lat_deg, lon_deg_east, h0_deg, event, equatorial)
                .or_else(|| {
                    event_on_transits_rotation(
                        transit + ROTATION_DAYS,
                        lat_deg,
                        lon_deg_east,
                        h0_deg,
                        event,
                        equatorial,
                    )
                })?
        };
        // The refinement can pull a root that started a hair inside the window
        // back across its opening edge. The scan could not see such an event
        // either — its first bracket opens at `jd_ut_day_start` — so the first
        // event in the window is then the one a rotation later.
        let root = if root < jd_ut_day_start {
            refine_event(
                root + ROTATION_DAYS,
                lat_deg,
                lon_deg_east,
                h0_deg,
                event,
                equatorial,
            )?
        } else {
            root
        };
        if root >= jd_ut_day_start && root <= window_end {
            Some(root)
        } else {
            None
        }
    };

    RiseSet {
        rise: first_in_window(Event::Rise),
        set: first_in_window(Event::Set),
        transit: first_in_window(Event::Transit),
    }
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
/// Ephemeris position and obliquity are dynamical quantities, defined on
/// TT, not UT — unlike [`local_sidereal_degrees`]'s GMST, which tracks
/// Earth's physical rotation and is defined on UT. `ecliptic_position`
/// already does its own `jd_ut` → TT conversion internally (it takes UT,
/// like this function does), but `obliquity::mean_obliquity` does not, so
/// this converts once via `delta_t::ut1_to_tt` before calling it — passing
/// `jd_ut` there would evaluate the obliquity polynomial ~69 s of time
/// (present-day ΔT) into the past.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13.
#[must_use]
pub fn sun_equatorial_deg(
    provider: &dyn vedaksha_ephem_core::jpl::EphemerisProvider,
    jd_ut: f64,
) -> Option<(f64, f64)> {
    use vedaksha_ephem_core::{bodies::Body, coordinates, delta_t, obliquity};

    let pos = coordinates::ecliptic_position(provider, Body::Sun, jd_ut).ok()?;
    let (lambda, beta) = (pos.longitude, pos.latitude);
    let jd_tt = delta_t::ut1_to_tt(jd_ut);
    let eps = obliquity::mean_obliquity(jd_tt);

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

/// The occurrence of `event` on the ONE rotation whose upper transit is
/// `transit`, or `None` if that rotation has none.
///
/// # Why the seed has to be a transit
///
/// `cos H₀` depends on the instant only through the declination, and `δ`
/// drifts — by up to ~0.4 °/day for the Sun. Within one rotation that is
/// enough to move `cos H₀` across the ±1 boundary, so an instant picked at an
/// arbitrary phase of the rotation can report "no rise" for a rotation that
/// does have one. Measured directly: at lat 68 N, lon 0, anchored on
/// JD 2,433,282.5 (1950-01-01), the scan oracle finds a rise at
/// JD 2,433,285.9891848518 — the day the polar night ends — while probing that
/// same rotation half a rotation earlier gives `cos H₀ ≈ 1.003`, out of range,
/// and reports the Sun as never rising.
///
/// The two instants where the probe is *not* arbitrary are the rotation's own
/// transits, because they are where the two boundaries actually happen:
///
/// * Polar night ends with `cos H₀` falling through +1, i.e. `H₀ → 0`: the Sun
///   first grazes `h₀` at the moment of UPPER transit, so `δ` there is `δ` at
///   the event.
/// * Polar day ends with `cos H₀` rising through −1, i.e. `H₀ → 180°`: the Sun
///   first dips to `h₀` at LOWER transit, half a rotation away, so `δ` there
///   is the one that decides.
///
/// Both are therefore tried, upper first. Each is only a starting instant:
/// [`refine_event`] re-evaluates `(α, δ)` — and hence `H₀` — at every pass, so
/// two seeds that admit the event converge onto the same fixed point.
///
/// # Why this cannot stray to a neighbouring rotation
///
/// From the upper transit the first correction is `−H₀ / rate` with
/// `H₀ ∈ [0, 180]`, so the rise it seeks is at most half a rotation earlier;
/// from the lower transit the first correction lands on the same instant. The
/// event returned therefore belongs to `transit`'s own rotation, which is what
/// lets [`search_rise`] enumerate rotations by their transits and know it has
/// skipped none.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15 (`m₀` the
/// transit, `m₁ = m₀ − H₀/360` the rise, `m₂ = m₀ + H₀/360` the set).
fn event_on_transits_rotation(
    transit: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    event: Event,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    refine_event(transit, lat_deg, lon_deg_east, h0_deg, event, equatorial).or_else(|| {
        refine_event(
            transit - HALF_ROTATION_DAYS,
            lat_deg,
            lon_deg_east,
            h0_deg,
            event,
            equatorial,
        )
    })
}

/// The nearest upward crossing of `h0_deg` on one side of `anchor` — the
/// INSTANT-anchored primitive [`previous_rise`] and [`next_rise`] share.
///
/// Unlike [`rise_set`], whose 24 h window is fixed by the caller and which
/// therefore reports only the FIRST rise inside it, this measures from
/// `anchor` itself. A rotation shorter than 24 h therefore cannot hide a
/// second rise behind the first: there is no window for two rises to share.
///
/// # How it is found
///
/// By ENUMERATING ROTATIONS, one per iteration, each identified by its upper
/// transit. A rotation has exactly one rise or none, and
/// [`event_on_transits_rotation`] returns that rotation's own rise (Meeus
/// `m₁ = m₀ − H₀/360`), so walking the transits outward from `anchor` visits
/// every candidate in order and can skip none.
///
/// The walk starts at the transit nearest `anchor` and steps by
/// [`ROTATION_DAYS`], re-converging the transit each time so it stays exact —
/// the Sun's transits are one *solar* day apart, slightly more than one
/// rotation, and re-converging absorbs that difference rather than letting it
/// accumulate.
///
/// ## Why rotations, and not "the nearest rise, then step"
///
/// Two cheaper schemes were tried and both are wrong, in ways the scan oracle
/// caught:
///
/// * A DIRECTED first step — "the rise at or behind `anchor` is
///   `anchor − (360 − g) / rate`, with `g ∈ [0, 360)` the forward hour-angle
///   gap" — is exact in real arithmetic and wrong in floating point, because
///   it is discontinuous exactly at `anchor`: `g` sits a hair above 0 or a
///   hair below 360 depending only on which way the last rounding of a
///   converged root went, and the two answers are a whole rotation apart. That
///   is defect #4 in analytic clothing;
///   `anchoring_exactly_on_a_rise_respects_the_strictness_of_each_side` failed
///   at lon 0 with `previous_rise(rise)` returning `rise − 0.99727 d`.
/// * Folding to the NEAREST rise from a free-running cursor fixes that but can
///   skip a rotation, because "nearest" is ambiguous when the cursor sits near
///   a lower transit and because a rotation whose `cos H₀` is out of range at
///   the cursor's phase may still have a rise at its transit. Measured: at lat
///   68 N, lon 0, JD 2,433,282.5, that scheme returned `None` where the oracle
///   finds JD 2,433,285.9891848518 — the day the polar night ends.
///
/// Enumerating by transit has neither hazard: the identity of each rotation is
/// unambiguous, its rise is sought from an instant on that same rotation, and
/// the existence test is evaluated where the polar boundary actually falls.
///
/// ## Acceptance
///
/// `forward = true` accepts a crossing strictly after `anchor`;
/// `forward = false` accepts one at or before it. The test is applied to the
/// refined root itself, so the returned instant satisfies its inequality
/// against `anchor` by construction rather than by an argument about window
/// arithmetic. A rejected root simply means the walk has not reached the right
/// rotation yet.
///
/// Returns `None` when no accepted crossing is found within
/// [`RISE_SEARCH_DAYS`] of `anchor` (polar day and polar night), or as soon as
/// `equatorial` fails to supply a position.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
fn search_rise(
    anchor: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    forward: bool,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let stride = if forward {
        ROTATION_DAYS
    } else {
        -ROTATION_DAYS
    };
    // The rotation containing `anchor`, named by its upper transit. A transit
    // always exists — a body that never rises still crosses the meridian — so
    // this is `None` only when `equatorial` has no position at all.
    let mut transit = refine_event(
        anchor,
        lat_deg,
        lon_deg_east,
        h0_deg,
        Event::Transit,
        equatorial,
    )?;

    for _ in 0..RISE_SEARCH_ROUNDS {
        // A rise sits at most half a rotation before its transit, so a transit
        // this far out can no longer carry one inside the horizon.
        if libm::fabs(transit - anchor) > RISE_SEARCH_DAYS + HALF_ROTATION_DAYS {
            return None;
        }

        if let Some(root) = event_on_transits_rotation(
            transit,
            lat_deg,
            lon_deg_east,
            h0_deg,
            Event::Rise,
            equatorial,
        ) {
            let accepted = if forward {
                root > anchor
            } else {
                root <= anchor
            };
            if accepted {
                return if libm::fabs(root - anchor) <= RISE_SEARCH_DAYS {
                    Some(root)
                } else {
                    // Inside the round budget but outside the four-day horizon
                    // the 5-minute walk enforced. Reporting it would widen the
                    // contract silently.
                    None
                };
            }
        }
        // Either this rotation has no rise (polar day/night), or its rise is
        // on the wrong side of `anchor`. Both mean: go to the next rotation.
        transit = refine_event(
            transit + stride,
            lat_deg,
            lon_deg_east,
            h0_deg,
            Event::Transit,
            equatorial,
        )?;
    }

    None
}

/// The most recent rise of a body **at or before** `jd_ut`, as a Julian Day
/// (UT), for an observer at `lat_deg` / `lon_deg_east` and `elevation_m` above
/// sea level.
///
/// Specialised to the Sun's horizon: [`SUN_STANDARD_ALTITUDE_DEG`] plus
/// [`horizon_dip_deg`], the same target [`sun_rise_set`] applies.
///
/// This is the correct primitive for reckoning a day that begins at sunrise —
/// the Vedic vara. Walking [`sun_rise_set`]'s 24 h window back along the
/// calendar is NOT: two sunrises fall in one such window whenever the
/// inter-sunrise gap is under a day, the second is never reported, and the walk
/// settles on a sunrise a whole day too early. See the ⚠️ note on [`rise_set`].
///
/// Returns `None` when the body does not rise within the search bound
/// ([`RISE_SEARCH_DAYS`] = 4 days) — polar night, polar day, or an
/// `equatorial` closure that cannot supply a position.
///
/// # Cost, and how the answer is checked
///
/// Found analytically from Meeus eq. 15.1, not by scanning the day: see
/// [`search_rise`]. That is a handful of `equatorial` evaluations where the
/// 5-minute scan this replaced needed up to 1152 of them. The scan itself is
/// kept, verbatim, as the `#[cfg(test)]` `scan_reference` module below and is
/// now a permanent oracle — the sweep tests hold the analytic path to it
/// across latitude, longitude, date and both providers, and any future change
/// to this code is checked the same way at zero runtime cost.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
#[must_use]
pub fn previous_rise(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let h0 = SUN_STANDARD_ALTITUDE_DEG + horizon_dip_deg(elevation_m);
    search_rise(jd_ut, lat_deg, lon_deg_east, h0, false, equatorial)
}

/// The first rise of a body **strictly after** `jd_ut`, as a Julian Day (UT),
/// for an observer at `lat_deg` / `lon_deg_east` and `elevation_m` above sea
/// level.
///
/// The forward counterpart of [`previous_rise`], sharing its horizon target and
/// its search bound. Pairing the two around one instant yields the half-open
/// interval `[previous_rise, next_rise)` that contains it — containment holding
/// by construction, since each side is accepted only if it satisfies its own
/// inequality against `jd_ut`.
///
/// Returns `None` when the body does not rise within the search bound.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15.
#[must_use]
pub fn next_rise(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let h0 = SUN_STANDARD_ALTITUDE_DEG + horizon_dip_deg(elevation_m);
    search_rise(jd_ut, lat_deg, lon_deg_east, h0, true, equatorial)
}

/// The 5-minute brute-force scan that used to be the production
/// implementation, kept verbatim as a **reference oracle**.
///
/// It is not dead code and must not be deleted: it is the independent second
/// opinion the analytic path is measured against. Because it costs nothing at
/// runtime (`#[cfg(test)]`), keeping it is free, and it is the only reason
/// replacing the production algorithm was a safe thing to do at all. Any
/// future change to the analytic search is checked against it by the sweep
/// tests below.
///
/// The bodies are unchanged from the pre-analytic implementation apart from
/// being moved here and renamed; do not "simplify" them, or the oracle stops
/// being independent of the thing it checks.
#[cfg(test)]
pub(crate) mod scan_reference {
    use super::{RiseSet, geometric_altitude_deg, local_sidereal_degrees};
    use vedaksha_math::angle::normalize_degrees;

    /// Coarse scan step when bracketing a horizon crossing: 5 minutes.
    const SCAN_STEP_DAYS: f64 = 5.0 / 1440.0;

    /// Bisection iterations. 40 halvings of a 5-minute bracket reach far below
    /// float resolution; the loop is bounded rather than tolerance-driven so
    /// it cannot spin on a pathological closure.
    const BISECT_ITERS: u32 = 40;

    /// Steps the outward walk takes before giving up: `4 · 24 · 60 / 5` =
    /// 1152, four days at five minutes a step — the same four-day horizon
    /// `super::RISE_SEARCH_DAYS` now expresses directly, so the oracle and the
    /// production path look exactly as far.
    const RISE_SEARCH_STEPS: u32 = 4 * 24 * 60 / 5;

    /// Refine a bracketed root of `f` in `[lo, hi]` by bisection.
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

    /// Reference implementation of [`super::rise_set`].
    pub(crate) fn rise_set_by_scan(
        jd_ut_day_start: f64,
        lat_deg: f64,
        lon_deg_east: f64,
        h0_deg: f64,
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> RiseSet {
        let rel_alt = |jd: f64| -> Option<f64> {
            let (ra, dec) = equatorial(jd)?;
            Some(geometric_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg)
        };
        let hour_angle = |jd: f64| -> Option<f64> {
            let (ra, _) = equatorial(jd)?;
            let h = normalize_degrees(local_sidereal_degrees(jd, lon_deg_east) - ra);
            Some(if h > 180.0 { h - 360.0 } else { h })
        };
        let alt_and_hour_angle = |jd: f64| -> Option<(f64, f64)> {
            let (ra, dec) = equatorial(jd)?;
            let alt = geometric_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg;
            let h = normalize_degrees(local_sidereal_degrees(jd, lon_deg_east) - ra);
            Some((alt, if h > 180.0 { h - 360.0 } else { h }))
        };

        let mut out = RiseSet {
            rise: None,
            set: None,
            transit: None,
        };

        let mut prev_jd = jd_ut_day_start;
        let Some((mut prev_alt, mut prev_ha)) = alt_and_hour_angle(prev_jd) else {
            return out;
        };

        let mut jd = jd_ut_day_start + SCAN_STEP_DAYS;
        while jd <= jd_ut_day_start + 1.0 + 1e-9 {
            let Some((alt, ha)) = alt_and_hour_angle(jd) else {
                return out;
            };

            if out.rise.is_none() && prev_alt < 0.0 && alt >= 0.0 {
                out.rise = bisect(prev_jd, jd, &rel_alt);
            } else if out.set.is_none() && prev_alt >= 0.0 && alt < 0.0 {
                out.set = bisect(prev_jd, jd, &rel_alt);
            }

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

    /// Reference implementation of [`super::search_rise`] — the outward walk
    /// from the anchor instant, at 5-minute resolution.
    pub(crate) fn search_rise_by_scan(
        anchor: f64,
        lat_deg: f64,
        lon_deg_east: f64,
        h0_deg: f64,
        forward: bool,
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> Option<f64> {
        let rel_alt = |jd: f64| -> Option<f64> {
            let (ra, dec) = equatorial(jd)?;
            Some(geometric_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg)
        };

        let step = if forward {
            SCAN_STEP_DAYS
        } else {
            -SCAN_STEP_DAYS
        };
        let mut near = if forward {
            anchor
        } else {
            anchor + SCAN_STEP_DAYS
        };
        let mut near_alt = rel_alt(near)?;

        for _ in 0..RISE_SEARCH_STEPS {
            let far = near + step;
            let far_alt = rel_alt(far)?;
            let (lo, hi, lo_alt, hi_alt) = if forward {
                (near, far, near_alt, far_alt)
            } else {
                (far, near, far_alt, near_alt)
            };
            if lo_alt < 0.0 && hi_alt >= 0.0 {
                if let Some(root) = bisect(lo, hi, &rel_alt) {
                    let accepted = if forward {
                        root > anchor
                    } else {
                        root <= anchor
                    };
                    if accepted {
                        return Some(root);
                    }
                }
            }
            near = far;
            near_alt = far_alt;
        }

        None
    }

    /// Reference implementation of [`super::previous_rise`].
    pub(crate) fn previous_rise_by_scan(
        jd_ut: f64,
        lat_deg: f64,
        lon_deg_east: f64,
        elevation_m: f64,
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> Option<f64> {
        let h0 = super::SUN_STANDARD_ALTITUDE_DEG + super::horizon_dip_deg(elevation_m);
        search_rise_by_scan(jd_ut, lat_deg, lon_deg_east, h0, false, equatorial)
    }

    /// Reference implementation of [`super::next_rise`].
    pub(crate) fn next_rise_by_scan(
        jd_ut: f64,
        lat_deg: f64,
        lon_deg_east: f64,
        elevation_m: f64,
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> Option<f64> {
        let h0 = super::SUN_STANDARD_ALTITUDE_DEG + super::horizon_dip_deg(elevation_m);
        search_rise_by_scan(jd_ut, lat_deg, lon_deg_east, h0, true, equatorial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GMST is otherwise only ever checked against this module's own use of
    /// it, which cannot catch a time-scale bug (feeding TT instead of UT into
    /// `sidereal_time::gmst`) — both sides of the comparison would be wrong
    /// the same way. Pin `local_sidereal_degrees` against a value derived
    /// independently of this codebase, not against a number merely recalled
    /// from memory — a prior version of this constant (99.96696°) was
    /// exactly that, and it was wrong by 3.00″.
    ///
    /// Two independent closed forms, evaluated by hand here rather than by
    /// calling `sidereal_time::gmst` (which would just compare the function
    /// to itself), agree to 7 decimal places at JD 2,451,544.5
    /// (2000-01-01 00:00 UT1), where Tu = (JD − 2,451,545.0) / 36525 =
    /// −1.369472e-5:
    /// - IAU 1982: `GMST_seconds = 24110.54841 + 8640184.812866·Tu +
    ///   0.093104·Tu² − 6.2e-6·Tu³`
    /// - Meeus, *Astronomical Algorithms* 2nd ed., eq. 12.4 — the same
    ///   formula `sidereal_time::gmst` implements
    ///
    /// Both give 23992.2707s of time = 6h 39m 52.271s = 99.967795°.
    /// Feeding TT (JD + ΔT) into `gmst` here — the bug this module used to
    /// have — shifts the result by ~0.27°, far outside this test's
    /// tolerance, so this is the test that makes the UT-vs-TT choice
    /// testable.
    #[test]
    fn local_sidereal_degrees_matches_published_gmst_at_j2000() {
        let lst = local_sidereal_degrees(2_451_544.5, 0.0);
        // Measured residual: |99.9677946868551 - 99.967795| = 3.131449e-7
        // degrees — purely from rounding the derived value to 6 decimal
        // places for this literal, not a modeling gap. Tolerance is set
        // just above that measured number (1e-6), not to a round figure
        // sized to hide bad reference data.
        assert!(
            libm::fabs(lst - 99.967_795) < 1e-6,
            "GMST at JD 2451544.5 = {lst}, expected 99.967795 (IAU 1982 / Meeus eq. 12.4)"
        );
    }

    /// The engine reaches local sidereal time by two paths, and they must not
    /// disagree by a time scale.
    ///
    /// * This module's [`local_sidereal_degrees`], which passes **UT1** and is
    ///   pinned above against the independently derived IAU 1982 / Meeus 12.4
    ///   value (99.967795° at JD 2,451,544.5). Mean sidereal time.
    /// * `vedaksha_ephem_core::sidereal_time::local_sidereal_time`, which the
    ///   MCP and WASM chart surfaces call to build the RAMC that drives the
    ///   ascendant, the MC and all twelve house cusps. Apparent sidereal time.
    ///
    /// Mean and apparent differ by exactly one thing: the equation of the
    /// equinoxes, `Δψ · cos(ε_true)`. Δψ is bounded by the 18.6-year nutation
    /// term at about ±17.2″, so the equation of the equinoxes cannot exceed
    /// roughly 17.2″ · cos(23.4°) ≈ 15.8″ ≈ 0.0044°. It can NEVER be ΔT.
    ///
    /// Until the UT-vs-TT fix the two paths disagreed by ΔT — the chart
    /// surfaces converted to TT before calling `local_sidereal_time`, adding
    /// ΔT worth of Earth rotation instead of removing it (~0.27° at J2000,
    /// ~0.29° today). Feeding `jd_tt` to `local_sidereal_time` below
    /// reproduces that failure; see the mutation note on the tolerance.
    ///
    /// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 12.
    #[test]
    fn ephem_core_lst_agrees_with_ut_gmst_to_the_equation_of_the_equinoxes() {
        // Real UT1 Julian Days spanning the eras the engine serves, at
        // longitudes on both sides of Greenwich.
        let cases = [
            (2_433_282.5, 0.0),      // 1950-01-01 00:00 UT, Greenwich
            (2_451_544.5, 77.209),   // 2000-01-01 00:00 UT, Delhi
            (2_461_100.0, -74.006),  // 2026-02-28 12:00 UT, New York
            (2_488_069.5, 151.2093), // 2100-01-01 00:00 UT, Sydney
        ];

        // Tolerance = the physical bound on |Δψ·cos(ε)|, 17.2″ / 3600 =
        // 0.0047778°, NOT a round figure chosen to make the test pass.
        //
        // Measured residuals for the four cases above (apparent − mean, from
        // this test instrumented to print them):
        //   JD 2433282.5  lon    0.0000°  −8.416072689101e-4°  =  −3.029786″
        //   JD 2451544.5  lon   77.2090°  −3.550561239877e-3°  = −12.782020″
        //   JD 2461100.0  lon  −74.0060°   1.860283755832e-3°  =   6.697022″
        //   JD 2488069.5  lon  151.2093°   8.382244558618e-4°  =   3.017608″
        // Largest measured: 12.782020″ (0.00355°), under the bound.
        //
        // Mutation-measured for contrast — the same four cases with `jd_tt`
        // passed as the rotational argument (the pre-fix behaviour), with the
        // engine's own ΔT alongside:
        //   JD 2433282.5  ΔT  29.117 s  →  0.1208100534°  (7.25′)
        //   JD 2451544.5  ΔT  63.807 s  →  0.2630419328°  (15.78′)
        //   JD 2461100.0  ΔT  70.100 s  →  0.2947433232°  (17.68′)
        //   JD 2488069.5  ΔT 141.463 s  →  0.5918825517°  (35.51′)
        // Every one of those exceeds this bound by 25× to 124×, so the wrong
        // convention cannot satisfy this test at any epoch the engine serves.
        // Assertion 2 below pins each residual to Δψ·cos(ε) exactly, so the
        // slack in this bound cannot hide anything either.
        const EQ_EQUINOXES_MAX_DEG: f64 = 17.2 / 3600.0;

        for (jd_ut1, lon_deg_east) in cases {
            let mean = local_sidereal_degrees(jd_ut1, lon_deg_east);

            // The chart surfaces' exact call shape: dynamical quantities at
            // TT, the rotational argument at UT1.
            let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd_ut1);
            let (dpsi, deps) = vedaksha_ephem_core::nutation::nutation(jd_tt);
            let eps_true = vedaksha_ephem_core::obliquity::true_obliquity(jd_tt, deps);
            let apparent = normalize_degrees(rad_to_deg(sidereal_time::local_sidereal_time(
                jd_ut1,
                deg_to_rad(lon_deg_east),
                dpsi,
                eps_true,
            )));

            // Signed shortest angular separation, in degrees.
            let raw = normalize_degrees(apparent - mean);
            let diff = if raw > 180.0 { raw - 360.0 } else { raw };

            // 1. The gap must be the equation of the equinoxes, not ΔT.
            assert!(
                libm::fabs(diff) < EQ_EQUINOXES_MAX_DEG,
                "LST paths disagree by {diff}° at JD {jd_ut1} (lon {lon_deg_east}°); \
                 the equation of the equinoxes cannot exceed {EQ_EQUINOXES_MAX_DEG}°. \
                 A gap near 0.27–0.29° means a TT Julian Day reached a rotational \
                 argument."
            );

            // 2. Stronger: the gap must equal Δψ·cos(ε_true) to float
            // precision, so no other term can hide inside the bound above.
            let eq_equinoxes_deg = rad_to_deg(dpsi * libm::cos(eps_true));
            assert!(
                libm::fabs(diff - eq_equinoxes_deg) < 1e-12,
                "at JD {jd_ut1}: apparent − mean = {diff}°, but Δψ·cos(ε) = \
                 {eq_equinoxes_deg}°"
            );
        }
    }

    /// The Sun's altitude at its own transit must equal (90° − |lat − dec|)
    /// for an observer on the same meridian. The previous version of this
    /// test *located* transit by scanning with `local_sidereal_degrees` and
    /// then evaluated altitude with the same function, so a constant LST
    /// error would shift both sides together and stay green. Anchor
    /// independently instead: the GMST derived in
    /// `local_sidereal_degrees_matches_published_gmst_at_j2000` (99.967795°
    /// at JD 2,451,544.5, from IAU 1982 / Meeus eq. 12.4) is used directly as
    /// this fake body's RA, which puts it on the Greenwich meridian (hour
    /// angle 0) at that exact instant by construction — no LST search
    /// involved.
    #[test]
    fn altitude_at_transit_matches_the_closed_form() {
        let lat = 51.4778;
        let ra = 99.967_795; // GMST at JD 2451544.5, degrees (IAU 1982 / Meeus 12.4).
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
        use vedaksha_ephem_core::{bodies::Body, coordinates, delta_t, obliquity};

        let provider = vedaksha_ephem_core::analytical::AnalyticalProvider;
        let jd = 2_451_668.5; // 2000-05-04

        let pos = coordinates::ecliptic_position(&provider, Body::Sun, jd)
            .expect("mid-quadrant position");
        // Mirrors `sun_equatorial_deg`: obliquity is a dynamical quantity
        // and wants TT, unlike `ecliptic_position`'s UT parameter (which
        // converts internally).
        let eps = obliquity::mean_obliquity(delta_t::ut1_to_tt(jd));
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

    /// A Sun held at fixed right ascension, as `vedaksha_vedic::muhurta`'s own
    /// tests use it. Its rises recur once per SIDEREAL rotation, which is what
    /// makes the two-rises-in-one-window case below reproducible at will.
    fn flat_sun(_jd: f64) -> Option<(f64, f64)> {
        Some((0.0, 0.0))
    }

    /// One sidereal rotation in days: `360 / 360.98564736629`, the GMST rate
    /// of Meeus eq. 12.4 as implemented by
    /// `vedaksha_ephem_core::sidereal_time::gmst`. With `flat_sun`'s RA and
    /// declination both fixed, the altitude depends on the local hour angle
    /// `H = LST − RA` alone, so one full cycle of `H` — and hence one
    /// rise-to-rise interval — takes exactly this long: 0.9972695663290739 d,
    /// about 3 min 56 s SHORT of the 1.0 d window `rise_set` scans. That
    /// shortfall is precisely why two rises fit in one window.
    const SIDEREAL_ROTATION_DAYS: f64 = 0.997_269_566_329_073_9;

    /// THE BUG CLASS, stated directly against this module rather than through
    /// a caller: because one sidereal rotation is shorter than the 24 h
    /// [`rise_set`] scans, TWO rises fall inside a single window and
    /// [`sun_rise_set`] — first event per window, by contract — reports only
    /// the earlier one. Any walk that enumerates sunrises by stepping that
    /// window along the calendar therefore skips the later one silently.
    ///
    /// Measured directly with this fixture (probe run, since removed) at lat
    /// −9°, lon 159.4°, window `[JD 2451553.5, 2451554.5]`: rises land at JD
    /// 2451553.5025420883 and JD 2451554.4998116544 — both inside the window,
    /// 0.99727 d apart. `sun_rise_set` returns the first; `next_rise` anchored
    /// on it finds the second; `previous_rise` anchored at the window's close
    /// returns the second, which is the one a sunrise-reckoned day needs.
    #[test]
    fn two_rises_share_one_scan_window_and_only_the_instant_anchored_search_sees_both() {
        let (lat, lon) = (-9.0, 159.4);
        let window_start = 2_451_553.5;
        let window_end = window_start + 1.0;

        let first = sun_rise_set(window_start, lat, lon, 0.0, &flat_sun)
            .rise
            .expect("the fixture rises every sidereal day at this latitude");
        let second = next_rise(first, lat, lon, 0.0, &flat_sun).expect("a second rise follows");

        assert!(
            second < window_end,
            "precondition: the second rise ({second}) must fall inside the SAME \
             window [{window_start}, {window_end}] that `sun_rise_set` reported \
             only {first} from — otherwise this test does not exercise the bug"
        );
        assert!(
            libm::fabs((second - first) - SIDEREAL_ROTATION_DAYS) < 1e-6,
            "consecutive rises must be one sidereal rotation apart: got {} d, \
             expected {SIDEREAL_ROTATION_DAYS} d",
            second - first
        );

        let latest = previous_rise(window_end, lat, lon, 0.0, &flat_sun)
            .expect("a rise precedes the window's close");
        assert!(
            libm::fabs(latest - second) < 1e-9,
            "previous_rise must return the LATER of the two rises in the window \
             ({second}), not the one `sun_rise_set` reports ({first}) — got {latest}"
        );
    }

    /// `[previous_rise(t), next_rise(t))` must contain `t` for every `t`, and
    /// its width must be one rise-to-rise spacing — never two. Swept across a
    /// full 360° of longitude at 1° steps and three latitudes; the muhurta
    /// crate carries the same sweep at 0.01° resolution against the caller.
    #[test]
    fn previous_and_next_rise_bracket_the_instant_across_a_longitude_sweep() {
        let jd = 2_451_554.75;
        let mut checked = 0_u32;
        for lat in [-45.0_f64, -9.0, 0.0, 33.0] {
            let mut lon_i = -180_i32;
            while lon_i <= 180 {
                let lon = f64::from(lon_i);
                let prev = previous_rise(jd, lat, lon, 0.0, &flat_sun)
                    .unwrap_or_else(|| panic!("lat={lat} lon={lon}: a rise must precede {jd}"));
                let next = next_rise(jd, lat, lon, 0.0, &flat_sun)
                    .unwrap_or_else(|| panic!("lat={lat} lon={lon}: a rise must follow {jd}"));
                assert!(
                    prev <= jd && jd < next,
                    "lat={lat} lon={lon}: [{prev}, {next}) does not contain {jd}"
                );
                assert!(
                    libm::fabs((next - prev) - SIDEREAL_ROTATION_DAYS) < 1e-6,
                    "lat={lat} lon={lon}: bracket width {} d is not one sidereal \
                     rotation ({SIDEREAL_ROTATION_DAYS} d)",
                    next - prev
                );
                checked += 1;
                lon_i += 1;
            }
        }
        assert_eq!(checked, 4 * 361, "sanity: 4 latitudes × 361 longitudes");
    }

    /// The inequalities are strict in the one direction each and inclusive in
    /// the other, so anchoring exactly ON a rise must not return that same
    /// instant from `next_rise`, but MUST return it — exactly — from
    /// `previous_rise` ("at or before").
    ///
    /// Regression for the sub-ULP-negative-altitude bug: `search_rise`'s old
    /// backward seed tested `rel_alt(anchor)` via the `[anchor -
    /// SCAN_STEP_DAYS, anchor]` bracket's `hi_alt >= 0.0` guard, which a
    /// rise's own sub-ULP-negative residual fails, so the bracket containing
    /// `anchor` was skipped and `previous_rise` silently returned the
    /// PREVIOUS rotation's rise -- a bracket TWO rotations wide instead of
    /// one. The old assertion here (`prev <= rise`) could not catch this: it
    /// is satisfied by `rise - one_rotation` too. This version asserts
    /// bit-exact equality and a one-rotation bracket width instead.
    ///
    /// Two longitudes are checked because the sign of `rel_alt(anchor)`'s
    /// sub-ULP residual is not guaranteed by construction — it depends on
    /// which way the bisection's last halving happened to round for that
    /// specific instant, not on longitude as such. Measured directly against
    /// THIS test's own anchors (`flat_sun` fixture, lat 0, window start JD
    /// 2,451,544.5 — MUTATION-CHECK output, see below): `rel_alt(rise) =
    /// -8.40e-9` deg at lon 0 (the sign that triggers the bug) and `+1.13e-7`
    /// deg at lon 174 (the sign that happened to pass by luck on the old
    /// code). Both are exercised so the fix is proven independent of which
    /// sign the residual takes.
    ///
    /// Rotation spacing is derived, not assumed to be 1.0 d: this fixture's
    /// Sun is held at fixed right ascension, so consecutive rises are one
    /// SIDEREAL day apart -- `360 / 360.98564736629` (GMST rate, Meeus eq.
    /// 12.4) = `SIDEREAL_ROTATION_DAYS`, derived where that constant is
    /// defined above -- not one solar day.
    #[test]
    fn anchoring_exactly_on_a_rise_respects_the_strictness_of_each_side() {
        for lon in [0.0_f64, 174.0_f64] {
            let lat = 0.0;
            let rise = sun_rise_set(2_451_544.5, lat, lon, 0.0, &flat_sun)
                .rise
                .unwrap_or_else(|| panic!("lon={lon}: the fixture rises at the equator"));

            let next = next_rise(rise, lat, lon, 0.0, &flat_sun)
                .unwrap_or_else(|| panic!("lon={lon}: a rise follows"));
            assert!(
                next > rise,
                "lon={lon}: next_rise must be STRICTLY after its anchor: anchor {rise}, got {next}"
            );

            let prev = previous_rise(rise, lat, lon, 0.0, &flat_sun)
                .unwrap_or_else(|| panic!("lon={lon}: a rise is at or before"));
            assert_eq!(
                prev, rise,
                "lon={lon}: previous_rise(rise) must equal rise EXACTLY (bit-equal): \
                 anchor {rise}, got {prev} -- a mismatch of one rotation here is the \
                 sub-ULP-negative-altitude bug this test guards"
            );

            // Width must be ONE rotation, not two: the bug this test guards
            // produces exactly a two-rotation width by returning the
            // previous rotation's rise instead of `rise` itself.
            let width = next - prev;
            assert!(
                libm::fabs(width - SIDEREAL_ROTATION_DAYS) < 1e-6,
                "lon={lon}: [previous_rise(rise), next_rise(rise)) width = {width} d, \
                 expected one sidereal rotation ({SIDEREAL_ROTATION_DAYS} d) -- a width \
                 near {} d would mean previous_rise returned the WRONG (earlier) rotation",
                2.0 * SIDEREAL_ROTATION_DAYS
            );
        }
    }

    /// Polar night: the search must run out its bound and report `None` rather
    /// than inventing a crossing. Same observer and body as
    /// `polar_night_yields_no_rise_and_no_set`.
    #[test]
    fn polar_night_yields_no_previous_or_next_rise() {
        let sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        assert_eq!(previous_rise(2_451_545.0, 78.22, 15.65, 0.0, &sun), None);
        assert_eq!(next_rise(2_451_545.0, 78.22, 15.65, 0.0, &sun), None);
    }

    /// A closure that cannot supply a position must degrade to `None` here
    /// too, not panic and not report an arbitrary instant.
    #[test]
    fn unavailable_ephemeris_yields_no_previous_or_next_rise() {
        let sun = |_jd: f64| None;
        assert_eq!(previous_rise(2_451_545.0, 0.0, 0.0, 0.0, &sun), None);
        assert_eq!(next_rise(2_451_545.0, 0.0, 0.0, 0.0, &sun), None);
    }

    /// `elevation_m` must reach the horizon target here as it does in
    /// [`sun_rise_set`]: a lower horizon moves the rise earlier, so the most
    /// recent rise seen from altitude is later than (or equal to) the
    /// sea-level one only when they belong to different rotations — what is
    /// unambiguous is that for a fixed instant the elevated rise precedes the
    /// sea-level one within the same rotation.
    #[test]
    fn previous_rise_honours_elevation() {
        let (lat, lon) = (0.0, 0.0);
        let jd = 2_451_545.0;
        let sea = previous_rise(jd, lat, lon, 0.0, &flat_sun).expect("sea-level rise");
        let high = previous_rise(jd, lat, lon, 2_000.0, &flat_sun).expect("elevated rise");
        assert!(
            high < sea,
            "a 2000 m horizon dip must move the rise EARLIER: sea={sea}, elevated={high}"
        );
        assert!(
            sea - high < 0.01,
            "the two must belong to the same rotation, not differ by a day: \
             sea={sea}, elevated={high}"
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

    // ── Agreement with the brute-force scan oracle ────────────────────────────
    //
    // The analytic search replaced a 5-minute scan plus bisection. The scan is
    // not gone — it lives, verbatim, in `super::scan_reference` — and these
    // sweeps are the reason replacing it was safe: every event the production
    // code reports is checked against an implementation that shares no code
    // with it beyond `geometric_altitude_deg` and `local_sidereal_degrees`.

    /// Running tally of how far the analytic search and the scan oracle differ
    /// over a grid.
    #[derive(Debug, Default)]
    struct Agreement {
        samples: u64,
        /// Largest `|analytic − scan|` in days, over samples where BOTH
        /// reported an instant.
        max_gap_days: f64,
        /// `(lat, lon, elevation_m, jd, analytic, scan)` at `max_gap_days`.
        worst: Option<(f64, f64, f64, f64, f64, f64)>,
        /// Samples where exactly one of the two reported an instant. These are
        /// NOT folded into `max_gap_days` — a presence disagreement is a
        /// different (and worse) failure than a numeric one, so it is counted
        /// and asserted separately rather than being averaged away.
        presence_mismatches: u64,
        /// `(lat, lon, elevation_m, jd, analytic, scan)` at the first
        /// presence mismatch.
        first_presence_mismatch: Option<(f64, f64, f64, f64, Option<f64>, Option<f64>)>,
    }

    impl Agreement {
        fn record(
            &mut self,
            lat: f64,
            lon: f64,
            elevation_m: f64,
            jd: f64,
            analytic: Option<f64>,
            scan: Option<f64>,
        ) {
            self.samples += 1;
            match (analytic, scan) {
                (Some(a), Some(s)) => {
                    let gap = libm::fabs(a - s);
                    if gap > self.max_gap_days {
                        self.max_gap_days = gap;
                        self.worst = Some((lat, lon, elevation_m, jd, a, s));
                    }
                }
                (None, None) => {}
                _ => {
                    self.presence_mismatches += 1;
                    if self.first_presence_mismatch.is_none() {
                        self.first_presence_mismatch =
                            Some((lat, lon, elevation_m, jd, analytic, scan));
                    }
                }
            }
        }

        /// Panic unless every sample agreed to within `tol_days` and no sample
        /// disagreed about whether the event exists at all.
        fn assert_agrees(&self, what: &str, tol_days: f64) {
            assert_eq!(
                self.presence_mismatches, 0,
                "{what}: {} of {} samples disagree about whether the event EXISTS \
                 (analytic vs scan oracle); first at {:?}",
                self.presence_mismatches, self.samples, self.first_presence_mismatch
            );
            assert!(
                self.max_gap_days <= tol_days,
                "{what}: worst analytic-vs-scan disagreement {} d ({} s) over {} \
                 samples exceeds {tol_days} d; worst case (lat, lon, elevation_m, \
                 jd, analytic, scan) = {:?}",
                self.max_gap_days,
                self.max_gap_days * 86_400.0,
                self.samples,
                self.worst
            );
        }
    }

    /// Every comparison the sweep makes, kept apart so each surface reports its
    /// own worst case.
    #[derive(Debug, Default)]
    struct SweepResult {
        previous_rise: Agreement,
        next_rise: Agreement,
        window_rise: Agreement,
        window_set: Agreement,
        window_transit: Agreement,
    }

    impl SweepResult {
        fn samples(&self) -> u64 {
            self.previous_rise.samples
                + self.next_rise.samples
                + self.window_rise.samples
                + self.window_set.samples
                + self.window_transit.samples
        }

        fn max_gap_days(&self) -> f64 {
            let mut worst = self.previous_rise.max_gap_days;
            for a in [
                &self.next_rise,
                &self.window_rise,
                &self.window_set,
                &self.window_transit,
            ] {
                if a.max_gap_days > worst {
                    worst = a.max_gap_days;
                }
            }
            worst
        }

        fn assert_agrees(&self, label: &str, tol_days: f64) {
            self.previous_rise.assert_agrees("previous_rise", tol_days);
            self.next_rise.assert_agrees("next_rise", tol_days);
            self.window_rise.assert_agrees("rise_set().rise", tol_days);
            self.window_set.assert_agrees("rise_set().set", tol_days);
            self.window_transit
                .assert_agrees("rise_set().transit", tol_days);
            assert!(
                self.samples() > 0,
                "{label}: the sweep ran zero samples — a grid is empty"
            );
        }
    }

    /// Compare the analytic path against `scan_reference` at every point of
    /// the given grid, for the instant-anchored searches AND the 24-hour
    /// window.
    fn sweep(
        lats: &[f64],
        lons: &[f64],
        jds: &[f64],
        elevations: &[f64],
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> SweepResult {
        let mut out = SweepResult::default();
        for &lat in lats {
            for &lon in lons {
                for &jd in jds {
                    for &elevation in elevations {
                        out.previous_rise.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            previous_rise(jd, lat, lon, elevation, equatorial),
                            scan_reference::previous_rise_by_scan(
                                jd, lat, lon, elevation, equatorial,
                            ),
                        );
                        out.next_rise.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            next_rise(jd, lat, lon, elevation, equatorial),
                            scan_reference::next_rise_by_scan(jd, lat, lon, elevation, equatorial),
                        );

                        let h0 = SUN_STANDARD_ALTITUDE_DEG + horizon_dip_deg(elevation);
                        let analytic = rise_set(jd, lat, lon, h0, equatorial);
                        let scanned =
                            scan_reference::rise_set_by_scan(jd, lat, lon, h0, equatorial);
                        out.window_rise.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            analytic.rise,
                            scanned.rise,
                        );
                        out.window_set
                            .record(lat, lon, elevation, jd, analytic.set, scanned.set);
                        out.window_transit.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            analytic.transit,
                            scanned.transit,
                        );
                    }
                }
            }
        }
        out
    }

    /// Latitudes swept: the equator, both hemispheres out to ±89°, and a
    /// deliberately fine band straddling the polar circles (±66.5619° at the
    /// present obliquity), which is exactly where `cos H₀` crosses out of
    /// [−1, 1] and where a body can fail to clear the horizon on one rotation
    /// and clear it on the next. Every entry is mirrored, so the southern
    /// polar transition is covered as densely as the northern one.
    const SWEEP_LATITUDES_DEG: [f64; 43] = [
        -89.0, -85.0, -80.0, -75.0, -70.0, -68.0, -67.5, -67.0, -66.75, -66.5619, -66.5, -66.25,
        -66.0, -65.5, -65.0, -60.0, -50.0, -40.0, -30.0, -20.0, -10.0, //
        0.0,   //
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 65.0, 65.5, 66.0, 66.25, 66.5, 66.5619, 66.75, 67.0,
        67.5, 68.0, 70.0, 75.0, 80.0, 85.0, 89.0,
    ];

    /// Julian Days spanning a full year in each of three eras — 1950, 2000 and
    /// 2100. A year matters because the Sun's declination sets `H₀` and so
    /// sets how close a latitude is to its own polar transition; the eras
    /// matter because ΔT and the nutation/obliquity terms differ across them.
    /// The 30.4375-day stride is a twelfth of a Julian year, and its
    /// fractional part cycles the anchor through twelve different times of day
    /// rather than pinning every sample to UT midnight.
    fn sweep_julian_days(dates_per_era: u32) -> [f64; 36] {
        let mut out = [0.0_f64; 36];
        let mut i = 0_usize;
        // 1950-01-01, 2000-01-01, 2100-01-01, all 00:00 UT.
        for era in [2_433_282.5_f64, 2_451_544.5, 2_488_069.5] {
            for k in 0..dates_per_era {
                out[i] = era + f64::from(k) * 30.4375;
                i += 1;
            }
        }
        out
    }

    /// The real Sun, from the engine's own analytical provider.
    fn real_sun_provider() -> vedaksha_ephem_core::analytical::AnalyticalProvider {
        vedaksha_ephem_core::analytical::AnalyticalProvider
    }

    /// Longitude grid: −180° to +180° in 15° steps.
    fn longitude_grid() -> [f64; 25] {
        let mut lons = [0.0_f64; 25];
        for (i, slot) in lons.iter_mut().enumerate() {
            // `i` is bounded by 25, so the hop through `u8` is lossless and
            // needs no float `as` cast.
            let step = u8::try_from(i).expect("25 fits in u8");
            *slot = -180.0 + f64::from(step) * 15.0;
        }
        lons
    }

    /// Print a sweep's outcome. Only reached under `--nocapture` in the
    /// `#[ignore]`d tier, where the numbers in the tolerance derivation come
    /// from.
    fn report(label: &str, r: &SweepResult) {
        std::println!(
            "{label}: {} comparisons, max gap {:e} d ({:e} s), presence mismatches {}",
            r.samples(),
            r.max_gap_days(),
            r.max_gap_days() * 86_400.0,
            r.previous_rise.presence_mismatches
                + r.next_rise.presence_mismatches
                + r.window_rise.presence_mismatches
                + r.window_set.presence_mismatches
                + r.window_transit.presence_mismatches
        );
        std::println!(
            "{label}: worst prev={:?}\n{label}: worst next={:?}\n{label}: worst \
             rise={:?}\n{label}: worst set={:?}\n{label}: worst transit={:?}",
            r.previous_rise.worst,
            r.next_rise.worst,
            r.window_rise.worst,
            r.window_set.worst,
            r.window_transit.worst
        );
    }

    /// Tolerance for analytic-vs-scan agreement, in days.
    ///
    /// **DERIVED, not chosen.** Both implementations converge on the same root
    /// of the same continuous function: the bisection halves a 5-minute
    /// bracket 40 times (≈3.2e-15 d, far below the ~4.66e-10 d ULP at these
    /// Julian Days) and the analytic iteration runs to its own fixed point, so
    /// both land on the `f64` nearest the true root. The residual is therefore
    /// ULP-scale by construction, not model-scale, and the measured maxima say
    /// exactly that:
    ///
    /// | sweep (dense tier, `--release`) | comparisons | max gap |
    /// |---|---|---|
    /// | fixed-RA fixture | 258,000 | 4.656612873077393e-10 d |
    /// | real Sun         |  34,830 | 4.656612873077393e-10 d |
    ///
    /// 4.656612873077393e-10 d is **exactly one ULP** at JD ≈ 2.45e6
    /// (`2^-31` d), i.e. the two implementations never differ by more than the
    /// last bit. This tolerance is 1e-9 d — the next round figure above that
    /// measured maximum, about two ULP, and 8.64e-5 seconds. That is four
    /// orders of magnitude tighter than the one-second gate this change was
    /// allowed to spend, so no real regression can hide beneath it.
    const AGREEMENT_TOL_DAYS: f64 = 1e-9;

    /// SHIP GATE, committed tier. The analytic search must agree with the
    /// brute-force scan oracle across latitude (including both polar
    /// transitions and beyond them), the full range of longitude, twelve dates
    /// spanning a year, and three eras.
    ///
    /// The fixed-RA fixture carries this tier alone, because the scan oracle
    /// is affordable only for a closure that returns constants: one scan
    /// sample needs ~950 `equatorial` evaluations, and `sun_equatorial_deg`
    /// costs 15.59 ms per evaluation in a debug profile (measured, against
    /// 415 µs in `--release`) — 15 seconds for a single real-Sun sample. The
    /// real Sun is covered instead by
    /// `analytic_rise_reproduces_the_scan_oracle_for_the_real_sun`, which
    /// compares against literals the oracle produced, and at full density by
    /// the `#[ignore]`d tier below.
    ///
    /// The fixture is not a weaker case than the Sun for the property under
    /// test: its rises recur once per SIDEREAL rotation, so it exercises the
    /// two-rises-in-one-window geometry on every sample rather than on half
    /// the year, and its declination is what makes each latitude sit where it
    /// does relative to its own polar transition.
    #[test]
    fn analytic_rise_agrees_with_the_scan_oracle() {
        let jds = sweep_julian_days(4);
        let lons = longitude_grid();
        let flat = sweep(
            &SWEEP_LATITUDES_DEG,
            &lons,
            &jds[..12],
            &[0.0, 3_650.0],
            &flat_sun,
        );
        flat.assert_agrees("fixed-RA fixture", AGREEMENT_TOL_DAYS);
    }

    /// SHIP GATE, dense tier: the same comparison at full grid resolution and
    /// for the REAL Sun as well as the fixture.
    ///
    /// `#[ignore]`d because the oracle is the 5-minute scan — the very cost
    /// this change removed — and the real Sun costs 415 µs per ephemeris
    /// evaluation even in `--release`. This is where the numbers in
    /// [`AGREEMENT_TOL_DAYS`]'s derivation come from. Run it deliberately, in
    /// release:
    ///
    /// ```text
    /// cargo test --release -p vedaksha-astro --lib \
    ///     analytic_rise_agrees_with_the_scan_oracle_dense -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "tier-2: full-density scan-oracle sweep; tens of minutes in --release, run manually"]
    fn analytic_rise_agrees_with_the_scan_oracle_dense() {
        let jds = sweep_julian_days(12);
        let lons = longitude_grid();

        let flat = sweep(
            &SWEEP_LATITUDES_DEG,
            &lons,
            &jds,
            &[0.0, 3_650.0],
            &flat_sun,
        );
        report("fixed-RA fixture", &flat);
        flat.assert_agrees("fixed-RA fixture (dense)", AGREEMENT_TOL_DAYS);

        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        // Every fourth longitude and four dates per era: 43 × 7 × 12 = 3612
        // samples, five comparisons each. The grid is thinner than the
        // fixture's only because the oracle costs ~950 `sun_equatorial_deg`
        // evaluations a sample at 415 µs each — tens of minutes of wall clock
        // as it stands. Latitude, which is the axis that decides whether a
        // crossing exists at all, keeps its full resolution.
        let mut sparse_lons = [0.0_f64; 7];
        for (i, slot) in sparse_lons.iter_mut().enumerate() {
            *slot = lons[i * 4];
        }
        let real = sweep(
            &SWEEP_LATITUDES_DEG,
            &sparse_lons,
            &jds[..12],
            &[0.0],
            &real_sun,
        );
        report("real Sun", &real);
        real.assert_agrees("real Sun (dense)", AGREEMENT_TOL_DAYS);
    }

    /// Observers and instants for the real-Sun oracle table below: mid
    /// latitudes in both hemispheres, three eras, sea level and 3650 m, both
    /// polar circles at the solstices (polar day, polar night) and the
    /// latitude/date at which the polar night ENDS — the sample that first
    /// exposed the declination-drift hole in the existence probe.
    const REAL_SUN_CASES: [(f64, f64, f64, f64); 17] = [
        (13.0827, 80.2707, 0.0, 2_451_545.0),    // Chennai, 2000
        (21.3069, -157.8583, 0.0, 2_459_015.75), // Honolulu, 2020
        (51.4778, -0.0015, 100.0, 2_461_100.0),  // Greenwich, 2026
        (-33.8688, 151.2093, 20.0, 2_433_282.5), // Sydney, 1950
        (64.1466, -21.9426, 50.0, 2_488_069.5),  // Reykjavik, 2100
        (66.5619, 25.0, 0.0, 2_451_544.5),       // Arctic circle, January
        (66.5619, 25.0, 0.0, 2_451_716.5),       // Arctic circle, June solstice
        (68.0, 0.0, 0.0, 2_433_282.5),           // where the polar night ends
        (-66.5619, 140.0, 0.0, 2_451_544.5),     // Antarctic circle, January
        (-70.0, -60.0, 0.0, 2_451_899.5),        // Antarctic, December solstice
        (78.22, 15.65, 0.0, 2_451_545.0),        // Svalbard, polar night
        (0.0, 0.0, 0.0, 2_451_544.5),            // equator
        (-9.0, 87.668, 0.0, 2_459_113.4),        // the two-sunrise regression
        (29.65, 91.13, 3_650.0, 2_459_015.454),  // Lhasa, at elevation
        (-45.0, 170.5, 0.0, 2_488_069.5),        // Otago, 2100
        (85.0, -100.0, 0.0, 2_451_809.5),        // high Arctic, September equinox
        (89.0, 60.0, 0.0, 2_433_556.4375),       // where a draft fabricated a rise
    ];

    /// Generator for the literal table in
    /// `analytic_rise_reproduces_the_scan_oracle_for_the_real_sun`.
    ///
    /// The literals in that test are the SCAN ORACLE's answers, produced by
    /// this function and pasted in — never hand-written and never copied from
    /// the analytic path they exist to check. Re-run it in `--release` (the
    /// oracle needs ~660 ephemeris evaluations per row, at 415 µs each) if the
    /// table ever has to be re-recorded:
    ///
    /// ```text
    /// cargo test --release -p vedaksha-astro --lib \
    ///     record_the_real_sun_scan_oracle_table -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "generator: prints the scan-oracle literals for the table below"]
    fn record_the_real_sun_scan_oracle_table() {
        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        for (lat, lon, elevation, jd) in REAL_SUN_CASES {
            let prev = scan_reference::previous_rise_by_scan(jd, lat, lon, elevation, &real_sun);
            let next = scan_reference::next_rise_by_scan(jd, lat, lon, elevation, &real_sun);
            std::println!("        ({prev:?}, {next:?}),");
        }
    }

    /// SHIP GATE, real-Sun committed tier. The analytic path must reproduce
    /// the SCAN ORACLE's answer for the real Sun at every case in
    /// [`REAL_SUN_CASES`] — including the three that are `None` because the
    /// Sun does not rise, and the one that is `None` in one direction and
    /// `Some` in the other.
    ///
    /// The literals are the oracle's own output, printed by
    /// `record_the_real_sun_scan_oracle_table` in `--release` and pasted
    /// unmodified. They are not hand-derived and are not the analytic path's
    /// output; comparing against them is comparing against the scan, at a cost
    /// a debug profile can afford (the oracle itself cannot be run here — see
    /// `analytic_rise_agrees_with_the_scan_oracle`).
    ///
    /// Two rows are load-bearing, and both are here because the scan oracle
    /// caught a draft of the analytic search getting them wrong:
    ///
    /// * **Row 8** — lat 68 N, lon 0, JD 2,433,282.5. The polar night has not
    ///   ended, so `previous_rise` is `None`, but it ends 3.49 days later and
    ///   `next_rise` is `Some(2433285.9891848518)`. A draft returned `None`
    ///   for BOTH, because it tested `cos H₀` at an arbitrary phase of each
    ///   rotation, where the declination was still 0.07° short of admitting a
    ///   crossing. See [`event_on_transits_rotation`].
    /// * **Row 17** — lat 89 N, lon 60, JD 2,433,556.4375, both `None`: the
    ///   Sun does not reach `h₀` anywhere in the four days before this
    ///   instant. A draft reported `Some(2433552.8231591503)`, which is not a
    ///   crossing at all — `rel_alt` there is −0.00122° and a one-minute scan
    ///   of the whole window finds no sign change. It was an unconverged
    ///   iterate being returned as an answer; see [`refine_event`].
    #[test]
    fn analytic_rise_reproduces_the_scan_oracle_for_the_real_sun() {
        /// Scan-oracle answers for [`REAL_SUN_CASES`], in the same order.
        const ORACLE: [(Option<f64>, Option<f64>); 17] = [
            (Some(2_451_544.542_326_138_4), Some(2_451_545.542_597_106_7)),
            (Some(2_459_015.159_156_623_3), Some(2_459_016.159_253_377_5)),
            (Some(2_461_099.781_587_543), Some(2_461_100.780_101_655)),
            (Some(2_433_282.282_305_157_7), Some(2_433_283.282_818_015_7)),
            (Some(2_488_068.969_565_544_3), Some(2_488_069.968_919_513)),
            (Some(2_451_543.879_405_976_3), Some(2_451_544.878_051_822)),
            (None, None),
            (None, Some(2_433_285.989_184_851_8)),
            (None, None),
            (None, None),
            (None, None),
            (Some(2_451_543.749_334_572), Some(2_451_544.749_668_686_7)),
            (Some(2_459_112.499_999_42), Some(2_459_113.499_583_688_6)),
            (Some(2_459_015.448_311_249_7), Some(2_459_016.448_384_337)),
            (Some(2_488_069.204_676_534), Some(2_488_070.205_307_214_5)),
            (Some(2_451_808.980_359_035_5), Some(2_451_809.992_955_033_7)),
            (None, None),
        ];

        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);

        for (case, (want_prev, want_next)) in REAL_SUN_CASES.iter().zip(ORACLE) {
            let (lat, lon, elevation, jd) = *case;
            let got_prev = previous_rise(jd, lat, lon, elevation, &real_sun);
            let got_next = next_rise(jd, lat, lon, elevation, &real_sun);

            for (label, got, want) in [
                ("previous_rise", got_prev, want_prev),
                ("next_rise", got_next, want_next),
            ] {
                match (got, want) {
                    (Some(g), Some(w)) => assert!(
                        libm::fabs(g - w) <= AGREEMENT_TOL_DAYS,
                        "{label} at lat={lat} lon={lon} elev={elevation} jd={jd}: \
                         analytic {g} vs scan oracle {w}, gap {} d ({} s)",
                        libm::fabs(g - w),
                        libm::fabs(g - w) * 86_400.0
                    ),
                    (None, None) => {}
                    _ => panic!(
                        "{label} at lat={lat} lon={lon} elev={elevation} jd={jd}: \
                         analytic {got:?} but the scan oracle says {want:?} — the two \
                         disagree about whether the Sun rises at all"
                    ),
                }
            }
        }
    }

    /// SHIP GATE, the anchor class the grid sweep cannot reach.
    ///
    /// `vedaksha_vedic::muhurta::kalam_windows` — and through it
    /// `compute_panchanga` — opens its 24-hour window EXACTLY on a sunrise
    /// that [`previous_rise`] produced, and reads `.set` from it. That anchor
    /// puts the body precisely on the horizon at the window's opening instant,
    /// where "the first event inside the window" is decided by a sub-ULP
    /// residual whose sign is not fixed by construction. A grid of round
    /// latitudes and longitudes never lands on such an instant, so the sweep
    /// above cannot exercise it; this does, at every point of a longitude
    /// sweep and at four latitudes, against the same scan oracle.
    #[test]
    fn the_window_agrees_with_the_oracle_when_opened_exactly_on_a_rise() {
        let mut checked = 0_u32;
        for lat in [-45.0_f64, -9.0, 0.0, 33.0, 55.0] {
            let mut lon_i = -180_i32;
            while lon_i <= 180 {
                let lon = f64::from(lon_i);
                let rise = previous_rise(2_451_554.75, lat, lon, 0.0, &flat_sun)
                    .unwrap_or_else(|| panic!("lat={lat} lon={lon}: the fixture rises"));

                let analytic = sun_rise_set(rise, lat, lon, 0.0, &flat_sun);
                let scanned = scan_reference::rise_set_by_scan(
                    rise,
                    lat,
                    lon,
                    SUN_STANDARD_ALTITUDE_DEG,
                    &flat_sun,
                );

                for (label, got, want) in [
                    ("set", analytic.set, scanned.set),
                    ("transit", analytic.transit, scanned.transit),
                    ("rise", analytic.rise, scanned.rise),
                ] {
                    match (got, want) {
                        (Some(g), Some(w)) => assert!(
                            libm::fabs(g - w) <= AGREEMENT_TOL_DAYS,
                            "lat={lat} lon={lon}: window opened on the rise {rise}: \
                             {label} analytic {g} vs scan oracle {w}, gap {} d",
                            libm::fabs(g - w)
                        ),
                        (None, None) => {}
                        _ => panic!(
                            "lat={lat} lon={lon}: window opened on the rise {rise}: \
                             {label} analytic {got:?} vs scan oracle {want:?} — the two \
                             disagree about whether the event is inside the window"
                        ),
                    }
                }
                checked += 1;
                lon_i += 1;
            }
        }
        assert_eq!(checked, 5 * 361, "sanity: 5 latitudes × 361 longitudes");
    }

    /// Polar day and polar night must come out of the ANALYTIC path as `None`
    /// from the `cos H₀` range test itself, not from a search that merely ran
    /// out of steps.
    ///
    /// MUTATION-CHECK: clamping `cos_h0` into [−1, 1] in
    /// `rise_hour_angle_deg` — the single most plausible way to "fix" a
    /// domain error on `acos` — makes this test fail at every latitude below,
    /// because a clamped `cos H₀ = ±1` fabricates a grazing crossing where the
    /// body never reaches the horizon at all.
    #[test]
    fn cos_h0_out_of_range_is_the_polar_signal_and_never_a_fabricated_instant() {
        // Northern polar night / southern polar day at δ = −23°, and the
        // mirror image at δ = +23°.
        for (lat, dec) in [
            (78.22_f64, -23.0_f64), // Svalbard, polar night
            (78.22, 23.0),          // Svalbard, midnight sun
            (-78.22, -23.0),        // Antarctic, midnight sun
            (-78.22, 23.0),         // Antarctic, polar night
            (89.9, 0.0),            // essentially at the pole
        ] {
            assert_eq!(
                rise_hour_angle_deg(lat, dec, SUN_STANDARD_ALTITUDE_DEG),
                None,
                "lat={lat} dec={dec}: cos H0 is outside [-1, 1] here, so there is \
                 no rise hour angle to report"
            );
            let sun = move |_jd: f64| Some((0.0_f64, dec));
            assert_eq!(
                previous_rise(2_451_545.0, lat, 15.65, 0.0, &sun),
                None,
                "lat={lat} dec={dec}: previous_rise must be None"
            );
            assert_eq!(
                next_rise(2_451_545.0, lat, 15.65, 0.0, &sun),
                None,
                "lat={lat} dec={dec}: next_rise must be None"
            );
            let rs = rise_set(2_451_545.0, lat, 15.65, SUN_STANDARD_ALTITUDE_DEG, &sun);
            assert_eq!(rs.rise, None, "lat={lat} dec={dec}: no rise");
            assert_eq!(rs.set, None, "lat={lat} dec={dec}: no set");
            assert!(
                rs.transit.is_some(),
                "lat={lat} dec={dec}: a body that never rises still transits"
            );
        }
    }

    /// `rise_hour_angle_deg` must be `None`, not a panic and not `Some(NaN)`,
    /// where `cos φ · cos δ` is exactly zero and the quotient is NaN. A range
    /// check is false for NaN, which is why no separate zero test is needed —
    /// but that is a property of the code, so pin it.
    #[test]
    fn a_degenerate_geometry_yields_none_rather_than_nan() {
        for (lat, dec) in [(90.0_f64, 0.0_f64), (-90.0, 0.0), (0.0, 90.0), (0.0, -90.0)] {
            assert_eq!(
                rise_hour_angle_deg(lat, dec, SUN_STANDARD_ALTITUDE_DEG),
                None,
                "lat={lat} dec={dec}: cos(phi)*cos(dec) is 0 here"
            );
        }
    }

    /// The `f64` the analytic iteration settles on must be a FIXED POINT: one
    /// more pass of the refinement must return the identical bits. Without
    /// that, `previous_rise(r) == r` would hold only by luck of where the
    /// iteration happened to stop.
    #[test]
    fn a_converged_rise_is_a_fixed_point_of_the_refinement() {
        for lat in [0.0_f64, 33.0, -45.0, 60.0] {
            for lon in [0.0_f64, -120.0, 77.2] {
                let anchor = 2_451_545.0;
                let rise = previous_rise(anchor, lat, lon, 0.0, &flat_sun)
                    .unwrap_or_else(|| panic!("lat={lat} lon={lon}: the fixture rises"));
                let again = refine_event(
                    rise,
                    lat,
                    lon,
                    SUN_STANDARD_ALTITUDE_DEG,
                    Event::Rise,
                    &flat_sun,
                )
                .expect("the rise exists, so refining from it must too");
                assert_eq!(
                    again, rise,
                    "lat={lat} lon={lon}: refining from a converged rise moved it"
                );
            }
        }
    }
}
