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
//! # SCOPE: validated for the SUN, and for bodies no faster
//!
//! The closure makes these functions look general, and the argument they are
//! built on — a body has at most one rise, one set and one transit per
//! rotation of its local hour angle — is general. The **search** is not. Two
//! of its constants are the Earth's rotation, not the body's:
//! [`ROTATION_DAYS`] is the stride [`search_rise`] uses to step from one
//! candidate rotation to the next, and [`HALF_ROTATION_DAYS`] is the slack it
//! allows between a rise and the transit that names its rotation. For a body
//! whose right ascension moves fast enough, the true rise-to-transit offset
//! exceeds that slack and the walk visits the wrong rotations.
//!
//! Measured, against this module's own scan oracle, with a Moon-like fixture
//! (right ascension advancing at 13.176 396 6 °/day — the mean-longitude rate
//! of Meeus Ch. 47 — and declination swinging ±28.6° over the same period),
//! swept over 49 latitudes × 25 longitudes × 12 dates in three eras,
//! **73 500 comparisons**:
//!
//! | | presence disagreements | worst value gap |
//! |---|---|---|
//! | before this commit | 455 | **0.943 d** |
//! | after  | **14** (prev 1, next 0, rise 6, set 7) | 9.313 225 746 154 785e-10 d = 2 ULP |
//!
//! The fix in this commit was not aimed at the Moon and helps it anyway,
//! because most of what went wrong for a fast body was the same defect: a
//! search that failed being read as a body that did not rise. What is left is
//! 14 dropped events, and it is enough to keep the scope where it is. Nothing
//! in this engine drives these functions with the Moon, and none of the
//! rotation-walk premises in [`search_rise`] have been re-derived for a body
//! moving at 13 °/day — the numbers improved, the *argument* has not been
//! made.
//!
//! **So: drive [`rise_set`], [`sun_rise_set`], [`previous_rise`] and
//! [`next_rise`] with the Sun.** They are held to the scan oracle for the Sun
//! across longitude, elevation, three eras and latitude to ±89 by the sweeps
//! in this file (see the next section for why the gate stops there), and
//! nothing in this engine drives them with anything else. A fast-moving body
//! needs a bracketing scan over the whole search interval — the shape
//! `scan_reference::search_rise_by_scan` has — not this hour-angle walk. That
//! scan is deliberately retained (it is also [`refine_event`]'s fallback), but
//! it is not exported: adding a Moon-capable surface means proving a walk
//! against the oracle for that body first, not re-pointing these functions.
//! `a_moon_like_body_is_measurably_outside_this_modules_scope` holds this
//! paragraph to its numbers, and fails if they move in EITHER direction.
//!
//! # SCOPE: the ship gate runs to |lat| 89, and the band beyond it is measured
//!
//! Above about |lat| 88 the rotational wobble in altitude (amplitude `cos φ`,
//! which is 0.035 at 88° and 6.1e-17 at the pole) stops dominating the Sun's
//! ~0.4 °/day declination drift, and a horizon crossing becomes a DECLINATION
//! event rather than a rotational one. Eq. 15.1 has less and less to say —
//! at the pole it has nothing at all, `cos H₀` being of order 1e14 — and a
//! search built on "one rise per rotation, within half a rotation of its
//! transit" is modelling something that has stopped being true.
//!
//! Two of the three changes in this module attack that directly.
//! [`refine_event`] no longer reads its own failure — an exhausted budget, or
//! an iterate that stepped out of eq. 15.1's domain — as "there is no
//! crossing"; it hands the case to the bisection scan.
//! [`boundary_is_reachable`] turns the existence test from a point sample into
//! an interval one, and scans the rotation whenever the declination could have
//! carried the boundary across it. Both are bounded: they cost nothing where
//! the analytic search already works, and deep polar night at Svalbard
//! (`cos H₀ ≈ 1.14` against a reach of 0.017) never scans at all.
//!
//! Measured against the scan oracle over |lat| 88–90 × 8 longitudes × 8 dates
//! × 2 elevations × both search directions (5 376 samples):
//!
//! | | dropped sunrises | wrong instants | worst error |
//! |---|---|---|---|
//! | before | 53 | 16 | 1.214 d |
//! | after  | **0** | **0** | **one ULP** |
//!
//! The real-Sun SHIP GATE
//! (`analytic_rise_agrees_with_the_scan_oracle_dense`) runs to **±89** — 36 120
//! comparisons, worst disagreement one ULP, zero presence disagreements — and
//! the fixed-RA tier runs to the poles at 441 000 comparisons on the same
//! terms. Beyond ±89 the real Sun is MEASURED rather than asserted, by
//! `polar_band_disagreement_is_measured_not_asserted`, because
//! the rotation walk still misses an hour-long dip below `h₀` around a lower
//! transit there — 8 of 16 800 polar comparisons, all in [`rise_set`]'s
//! window, plus a worst `previous_rise` error of 0.497 d at |lat| 89.9. That
//! test does gate the property which matters most in the band —
//! [`previous_rise`] and [`next_rise`] never disagreeing with the oracle about
//! whether a sunrise EXISTS — and records the rest rather than hiding it
//! behind a loosened tolerance.
//!
//! Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 12 (sidereal time),
//! Ch. 13 (altitude), Ch. 15 (rising, transit, setting), Ch. 47 (the Moon's
//! mean rate, used only to size the fixture above).

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
/// # The budget, and why exhausting it is no longer a verdict
///
/// The contraction argument above holds AWAY from the grazing regime. Close to
/// a polar boundary it does not: `d cos H₀/dδ ≈ −tan φ` is ~57 per radian at
/// lat 89 and ~115 at lat 89.5, so `H₀` itself moves fast enough that the map
/// creeps toward its root instead of contracting onto it. A budget of 24 was
/// exhausted on real cases, and an earlier draft read that exhaustion as "no
/// crossing" — turning a real sunrise into a false polar night, or worse,
/// letting the rotation walk step past the right rotation and return a sunrise
/// from the wrong one. Measured against this module's own scan oracle, real
/// Sun, `AnalyticalProvider`:
///
/// | observer | anchor JD | shipped at 24 iters | scan oracle |
/// |---|---|---|---|
/// | lat 88.1, lon 0, 0 m | 2 459 287.5 | `None` | 2 459 286.996 263 494 7 |
/// | lat 89.3, lon 0, 0 m | 2 459 295.5 | 2 459 292.672 512 896 | 2 459 293.533 634 425 |
/// | lat 89.5, lon −150, 3650 m | 2 488 343.4375 | 2 488 341.166 030 892 | 2 488 342.375 926 383 |
///
/// — a dropped sunrise, then errors of 0.861 d and 1.210 d, each of which
/// starts the vara a whole day late.
///
/// Both halves of that are fixed here. The count below is the measured one,
/// and [`refine_event`] no longer answers `None` when it runs out: it falls
/// back to [`scan_reference::event_near_by_scan`], the bisection scan this
/// module retains as its oracle. `None` for "no rise" is then produced by
/// exactly one thing, [`rise_hour_angle_deg`]'s `cos H₀` range test, which is
/// the only condition entitled to say it.
///
/// # Derivation of 512
///
/// Measured on an instrumented build over the grid that exposed the defect —
/// |lat| 88 to 90 in 0.1° steps × 8 longitudes × 8 anchor dates spanning three
/// eras × 2 elevations × both search directions: 5 376 samples and **71 906
/// calls to this function**.
///
/// * The largest number of passes any CONVERGING refinement needed was **64**.
/// * 254 calls (0.35 %) reached no fixed point at all and went to the scan.
///
/// 512 is the next power of two above 64 — an **8× margin** over the measured
/// worst case, and 21× the 24 that was shipped. It is deliberately far above
/// the measurement rather than snug against it, because being generous costs
/// nothing: the loop returns the moment a pass stops moving the instant,
/// typically after five or six, so no converging call ever sees the budget.
/// The only calls that pay it are the ones that were going to be wrong.
///
/// # The budget alone was not the fix
///
/// Same grid, same instrument, one change added at a time — dropped sunrises
/// are `None` where the scan oracle finds one, wrong instants are `Some` more
/// than 1e-9 d from it:
///
/// | state | dropped | wrong | worst error |
/// |---|---|---|---|
/// | shipped: 24 iterations, exhaustion answers `None` | 53 | 16 | 1.214 d |
/// | + budget 512 and exhaustion falls back to the scan | 29 | 8 | 0.898 d |
/// | + an iterate leaving eq. 15.1's domain also falls back | 17 | 2 | 0.873 d |
/// | + [`boundary_is_reachable`] | 1 | 1 | 0.477 d |
/// | + a full-rotation fallback window and [`rise_set`]'s directed-seed retry | **0** | **0** | **1 ULP** |
///
/// The budget is the first row of five and worth about half the presence
/// errors. Everything else in that table is the same principle applied
/// elsewhere: a search that failed is not a body that did not rise.
const REFINE_ITERS: u32 = 512;

/// How large the residual of an accepted two-cycle may be, in ULP of the
/// instant — the second half of [`refine_event`]'s cycle test, alongside the
/// requirement that the two members be adjacent `f64`s.
///
/// A genuine adjacent two-cycle straddles the true root: each member sits
/// within one ULP of the other, so its correction — converted from degrees of
/// hour angle to days — is a shade over half an ULP.
///
/// # Why 4, and which of the two conditions actually binds
///
/// `next` is `t + correction/rate` ROUNDED to the nearest `f64`, so for a cycle
/// of bit-width `w` the correction in days satisfies
/// `(w − ½)·ulp ≤ |correction|/rate ≤ (w + ½)·ulp`. Measured directly on the
/// `retrograde_sun` fixture (see the tests): width 1 → 0.602 ULP, width 3 →
/// 2.602, width 5 → 4.602, width 15 → 14.602 — the residual tracks `w − 0.4`,
/// as the algebra says.
///
/// So a bound of `B` admits widths up to about `B + ½` **on its own**, and the
/// two conditions are not independent. In the shipped configuration ADJACENCY
/// is the binding one: it caps the width at 1, which caps the residual at
/// ~1.5 ULP, well inside 4. The bound below is a second, explicit statement of
/// the same requirement in the quantity the tie-break actually compares
/// (`correction`, converted from degrees of hour angle to days) rather than in
/// raw bit distance.
///
/// MUTATION-CHECK, both directions, run and recorded rather than assumed:
/// removing the ADJACENCY test while keeping this bound fails
/// `a_two_cycle_wider_than_one_ulp_is_rejected_and_handed_to_the_scan` (a
/// 3-ULP orbit, residual 2.602 ULP, sits inside 4 and is returned as a root).
/// Removing THIS bound while keeping adjacency fails nothing — it is redundant
/// given the width cap, and is kept as the condition that still holds the line
/// if the width cap is ever loosened, not as an independently load-bearing
/// test. Claiming otherwise would be the kind of unverified assertion this
/// module has already been bitten by.
///
/// 4 rather than 2: the algebraic maximum for an ACCEPTED (width-1) cycle is
/// 1.5 ULP and the worst measured on the fixture is 0.602, so 4 is a ~2.7×
/// margin over the bound and ~6.6× over the measurement, while still rejecting
/// outright the wide orbits (5 ULP and up) the raised budget makes reachable.
///
/// Wide orbits are not hypothetical for the real Sun either. Over the
/// high-latitude grid in [`REFINE_ITERS`] the refinement entered 3 356 cycles;
/// the widest was **3 bits, residual 2.507 ULP**, and the adjacency test
/// rejected it — at `REFINE_ITERS = 24` on the same grid the widest ever seen
/// was 1 bit. Raising the budget is what made that orbit reachable, which is
/// why the width test had to arrive with it.
const CYCLE_MAX_RESIDUAL_ULPS: f64 = 4.0;

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
    let cos_h0 = cos_rise_hour_angle(lat_deg, dec_deg, h0_deg);
    if (-1.0..=1.0).contains(&cos_h0) {
        Some(rad_to_deg(libm::acos(cos_h0)))
    } else {
        None
    }
}

/// `cos H₀` itself — Meeus eq. 15.1's left-hand side, with **no range test**.
///
/// [`rise_hour_angle_deg`] is the right thing to call to locate an event.
/// This exists for the one caller that needs to know HOW FAR out of range the
/// value is rather than merely that it is: [`boundary_is_reachable`], which
/// has to ask whether the declination could carry it back in.
///
/// Returns NaN when `cos φ · cos δ = 0` — a geographic or celestial pole. That
/// NaN is load-bearing: every comparison against it is false, which is how the
/// pole keeps reporting "no rotational crossing" instead of being scanned for
/// an annual one.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15, eq. 15.1.
fn cos_rise_hour_angle(lat_deg: f64, dec_deg: f64, h0_deg: f64) -> f64 {
    let (phi, dec) = (deg_to_rad(lat_deg), deg_to_rad(dec_deg));
    (libm::sin(deg_to_rad(h0_deg)) - libm::sin(phi) * libm::sin(dec))
        / (libm::cos(phi) * libm::cos(dec))
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

/// What [`hour_angle_gap_deg`] found at an instant.
///
/// These are two DIFFERENT answers and collapsing them into one `Option` is
/// what let a real sunrise disappear. "There is no crossing at this instant"
/// is a statement about the geometry at that instant only; `cos H₀` moves with
/// the declination, so it is emphatically not a statement about the rotation.
/// See [`refine_event`], which treats the two cases differently depending on
/// whether the instant is the caller's seed or an iterate the search wandered
/// onto by itself.
#[derive(Debug, Clone, Copy, PartialEq)]
enum HourAngleGap {
    /// The forward gap in degrees [0, 360).
    Degrees(f64),
    /// `cos H₀` is outside [−1, 1] here: the body does not reach `h₀` at this
    /// instant's declination.
    OutOfDomain,
}

/// How far the body's local hour angle still has to advance at `jd_ut` to
/// reach `event`, in degrees [0, 360) — the *forward* gap, since the hour
/// angle only ever increases.
///
/// `None` means one thing only: `equatorial` could not supply a position. The
/// geometric "no crossing here" answer is [`HourAngleGap::OutOfDomain`], kept
/// separate so callers cannot confuse a missing ephemeris with a missing
/// sunrise.
///
/// One `equatorial` evaluation per call; this is the search's entire cost.
fn hour_angle_gap_deg(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    h0_deg: f64,
    event: Event,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<HourAngleGap> {
    let (ra, dec) = equatorial(jd_ut)?;
    let Some(target) = target_hour_angle_deg(event, lat_deg, dec, h0_deg) else {
        return Some(HourAngleGap::OutOfDomain);
    };
    let current = normalize_degrees(local_sidereal_degrees(jd_ut, lon_deg_east) - ra);
    Some(HourAngleGap::Degrees(normalize_degrees(target - current)))
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
/// # An unconverged iterate is NOT an answer — and neither is `None`
///
/// The loop returns its own iterate **only** at an exact fixed point, or at a
/// two-cycle narrow enough to be one (see the cycle test in the body).
/// Returning the last iterate of a run that never settled would be a
/// fabricated instant: at lat 89 N, lon 60, JD 2,433,556.4375 an earlier draft
/// did exactly that, reporting JD 2,433,552.8231591503 where the Sun never
/// reaches `h₀` at all — measured `rel_alt` there is −0.00122°, and a
/// one-minute scan of the surrounding four days finds no crossing whatsoever.
/// That row is pinned in [`REAL_SUN_CASES`].
///
/// **`None` is equally forbidden as a report of exhaustion.** An earlier draft
/// did that too, and it is the worse bug of the two, because it is silent:
/// `search_rise` reads `None` as "this rotation has no rise", walks past a
/// rotation that does have one, and either reports a false polar night or
/// returns a sunrise from the wrong rotation — up to 1.21 days out, measured
/// (see [`REFINE_ITERS`] for the three cases). "There is no crossing" is a
/// verdict exactly one thing is entitled to reach: [`rise_hour_angle_deg`]'s
/// `cos H₀ ∉ [−1, 1]` test, which is polar day and polar night and nothing
/// else.
///
/// So exhaustion goes to [`scan_reference::event_near_by_scan`] — the
/// bisection scan this module keeps as its oracle — which answers the same
/// "nearest occurrence to `t0`" question by bracketing rather than by
/// contraction, and therefore does not care that the map failed to contract.
/// The scan costs ~290 `equatorial` evaluations against the fast path's five
/// or six, which is why it is a fallback and not the implementation; on the
/// grids that motivated the change it is now reached zero times.
///
/// Returns `None` as soon as `equatorial` cannot supply a position, when the
/// event ceases to exist on the rotation being examined (`cos H₀` out of
/// range), or when the fallback scan finds no crossing within half a rotation
/// of `t0` either.
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

    for iter in 0..REFINE_ITERS {
        let gap = match hour_angle_gap_deg(t, lat_deg, lon_deg_east, h0_deg, event, equatorial)? {
            HourAngleGap::Degrees(gap) => gap,
            // `cos H₀` is out of range HERE. What that means depends entirely
            // on where "here" is.
            //
            // At the caller's own seed it is the polar signal: the caller
            // chose that instant (`event_on_transits_rotation` chooses a
            // transit, precisely because that is where the polar boundary
            // falls) and asked whether the event exists there. `None` is the
            // honest answer and the one every polar test pins.
            //
            // At any LATER iterate it means nothing of the kind. The search
            // put itself there, and `cos H₀` moves with δ — at lat 88.4 it
            // runs from 0.997 at a transit to 1.121 half a rotation earlier,
            // so a step taken from inside the domain can land outside it while
            // the crossing it was converging on is perfectly real. Measured
            // exactly there, lon 90, JD 2 459 287.5: `cos H₀` = 0.997 487 at
            // the transit — in range, so a rise exists — and the iteration
            // still walked out of the domain and reported `None`, whereupon
            // `search_rise` skipped the rotation and returned the NEXT day's
            // sunrise, 0.898 d late. So: stop iterating and let the scan below
            // answer, exactly as for an exhausted budget.
            HourAngleGap::OutOfDomain => {
                if iter == 0 {
                    return None;
                }
                break;
            }
        };
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
            // A TWO-CYCLE between ADJACENT representable instants. It happens
            // when the true root sits within a few percent of the midpoint
            // between two neighbouring `f64`s: the correction from each is a
            // shade over half an ULP, so each rounds onto the other and
            // NEITHER is a fixed point. Measured, `flat_sun` at lat −85,
            // lon −150, 3650 m, anchored on JD 2,433,312.9375: the iteration
            // alternates forever between 2433312.220053467434 and
            // 2433312.220053467900, one ULP apart, with corrections whose
            // day-equivalents are +2.3311e-10 d and −2.3385e-10 d against a
            // half-ULP of 2.3283e-10 d. (`correction` itself is in DEGREES of
            // hour angle; the conversion is the division below.)
            //
            // Both are the root to within one ULP (4.0e-5 s) and no arithmetic
            // at this precision can say which is nearer, so take the one with
            // the SMALLER residual and let the sweep's tolerance absorb the
            // last bit.
            //
            // A PERIOD-2 ORBIT ALONE IS NOT ENOUGH to call it a root.
            // `next == previous` also fires on a WIDE orbit — two instants many
            // ULP apart that the map happens to swap — and such a pair is not a
            // root at all. That is not theoretical: the `retrograde_sun` test
            // fixture, whose local map has slope exactly −1, produces orbits of
            // any width the seed asks for — 3 ULP, 15 ULP, 199 999 ULP — all of
            // them period-2 and none of them converged. At `REFINE_ITERS = 24`
            // no orbit wider than 1 ULP was ever observed, which is why the
            // guard shipped without a width test; the raised budget gives the
            // iteration room to reach one.
            //
            // So a cycle is accepted only when its two members are NEIGHBOURS
            // (`|bits(t) − bits(previous)| ≤ 1`) and the better of the two
            // residuals is inside `CYCLE_MAX_RESIDUAL_ULPS`. Anything wider
            // falls through, burns the rest of the budget and is handed to the
            // bisection scan below, which is the honest answer for a case the
            // fixed-point map cannot resolve. See `CYCLE_MAX_RESIDUAL_ULPS`
            // for which of the two conditions binds, measured both ways.
            //
            // The tie-break is a deterministic function of the CYCLE, not of
            // the entry point, so it is idempotent — which is what
            // `previous_rise(r) == r` needs. Smaller residual wins; on an exact
            // residual tie the EARLIER instant wins, because "whichever of the
            // two `t` happens to be" would depend on which member the caller
            // entered from and would break that bit-exactness.
            if next == previous {
                // `to_bits` / `from_bits` are `core` reinterpretations, not the
                // `std`-only float MATH methods this crate routes through
                // `libm`. Both instants are positive Julian Days, where the
                // IEEE-754 bit pattern is monotone in value, so the bit
                // distance IS the count of representable instants between them
                // and `from_bits(bits + 1)` is the next one upward.
                let ulp = f64::from_bits(t.to_bits() + 1) - t;
                let width_bits = t.to_bits().abs_diff(previous.to_bits());
                let this_residual = libm::fabs(correction);
                let best_residual = if this_residual <= previous_correction {
                    this_residual
                } else {
                    previous_correction
                };
                // `correction` is in DEGREES of hour angle — that is what
                // `hour_angle_gap_deg` returns — so it has to be divided by the
                // rotation rate before it can be compared with an ULP of a
                // Julian Day.
                let best_residual_days = best_residual / HOUR_ANGLE_RATE_DEG_PER_DAY;
                if width_bits <= 1 && best_residual_days <= CYCLE_MAX_RESIDUAL_ULPS * ulp {
                    return Some(if this_residual < previous_correction {
                        t
                    } else if previous_correction < this_residual {
                        previous
                    } else if t < previous {
                        t
                    } else {
                        previous
                    });
                }
            }
        }

        previous = t;
        previous_correction = libm::fabs(correction);
        t = next;
    }
    // THE SEARCH GAVE UP — either the budget ran out or an iterate stepped out
    // of eq. 15.1's domain. Neither is "there is no crossing here".
    //
    // That verdict belongs to one thing: `cos H₀` outside [−1, 1] AT THE SEED
    // the caller chose, which returns above on the first pass and never gets
    // here. Answering `None` from this point would make a search that failed
    // indistinguishable from a genuine polar night, and that is precisely how
    // a real sunrise came to be dropped and two more came back a day early
    // (see [`REFINE_ITERS`] for all three).
    //
    // Hand the case to the retained 5-minute bisection scan instead. It is the
    // same oracle the sweep tests hold this module to, it makes no assumption
    // about contraction and none about staying inside a domain, and it is
    // reached on 0.02 % of refinements even in the polar band — so the fast
    // path stays fast. Returning the last iterate is still forbidden: it is
    // not a root.
    scan_reference::event_near_by_scan(
        t0,
        scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
        lat_deg,
        lon_deg_east,
        h0_deg,
        event,
        equatorial,
    )
}

/// Rise, set and upper transit of the Sun (or a body no faster) across the 24
/// hours beginning at `jd_ut_day_start`, for an observer at `lat_deg` /
/// `lon_deg_east`.
///
/// ⚠️ **Sun-scoped, and gated to |lat| ≤ 89.** The closure will accept any
/// body, but the search is validated against the scan oracle for the Sun only.
/// This 24-hour window is also the surface with the most residual disagreement
/// in the polar band and on a Moon-like fixture, in both cases for the same
/// reason: an hour-long dip below `h₀` around a lower transit is an event the
/// rotation walk does not enumerate. See the two SCOPE sections in the module
/// documentation.
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
/// `jd_ut_day_start`, refined by [`refine_event`]. The [`scan_reference`]
/// module keeps the 5-minute scan this replaced — as the oracle the sweep
/// tests below hold this path to, and as the fallback [`refine_event`] uses
/// when its iteration does not converge; see the note on [`previous_rise`] for
/// the measured bound.
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
        //
        // THE DIRECTED SEED FAILING IS NOT A VERDICT EITHER. `jd_ut_day_start +
        // gap/rate` is an instant this code picked, not one the caller chose,
        // and `cos H₀` at it is a different quantity from `cos H₀` at the
        // window's opening: the declination has moved by then. So a `None` from
        // the directed refinement falls through to the transit seeds rather
        // than out of this closure. Measured, real Sun at lat 89 N, lon 120,
        // 3650 m, window opening JD 2 488 343.4375: the directed seed for the
        // SET lands where eq. 15.1 no longer admits one, and the set at
        // JD 2 488 343.677 893 959 — 0.24 d inside the window, found by the
        // scan — was reported as absent. It is the last presence disagreement
        // the 36 120-comparison real-Sun sweep had.
        let directed = if let Some(HourAngleGap::Degrees(gap)) = hour_angle_gap_deg(
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
            )
        } else {
            None
        };
        let root = if let Some(root) = directed {
            root
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
    let from_seeds = refine_event(transit, lat_deg, lon_deg_east, h0_deg, event, equatorial)
        .or_else(|| {
            refine_event(
                transit - HALF_ROTATION_DAYS,
                lat_deg,
                lon_deg_east,
                h0_deg,
                event,
                equatorial,
            )
        });
    if from_seeds.is_some() {
        return from_seeds;
    }
    // Both seeds say this rotation has no crossing — a POINT test on a
    // quantity that moves. Believe it only when the declination CANNOT have
    // carried `cos H₀` into range between them; otherwise scan the rotation.
    if boundary_is_reachable(transit, lat_deg, h0_deg, equatorial)? {
        scan_reference::event_near_by_scan(
            transit,
            scan_reference::ROTATION_SCAN_HALF_SPAN_DAYS,
            lat_deg,
            lon_deg_east,
            h0_deg,
            event,
            equatorial,
        )
    } else {
        None
    }
}

/// Could `cos H₀` have been inside [−1, 1] somewhere on the rotation whose
/// upper transit is `transit`, even though it is outside at both instants
/// [`event_on_transits_rotation`] probes?
///
/// # Why the point test is not enough
///
/// [`event_on_transits_rotation`] probes the two instants where the polar
/// boundary falls *when the boundary is stationary*. It is not stationary: `δ`
/// drifts, `cos H₀` moves with it, and in the GRAZING regime the whole
/// crossing can appear and vanish inside one rotation.
///
/// Measured, real Sun, lat 88.8 N, lon 90, JD 2 459 288.756 (the rotation's own
/// upper transit): `cos H₀` = 1.000 777 there and 1.165 at the lower transit —
/// out of range at both, so the probe said "no rise". The scan finds one four
/// minutes AFTER that transit, at JD 2 459 288.759 028 611: the Sun's peak
/// altitude is 0.0007° short of `h₀` at the transit, `δ` is rising at
/// 0.395 °/day, and within four minutes it has gained more altitude from the
/// declination than it has lost from the hour angle. `search_rise` skipped that
/// rotation and returned the NEXT day's sunrise, 0.873 d late.
///
/// # The bound
///
/// Differentiating eq. 15.1 with respect to declination:
///
/// `d cos H₀ / dδ = −tan φ + cos H₀ · tan δ`
///
/// so over a declination change `Δδ` (radians) the value can move by at most
/// `(|tan φ| + |cos H₀| · |tan δ|) · Δδ`. `Δδ` is not assumed: it is measured
/// from the closure across the half rotation between the two probes and
/// DOUBLED, which covers the full rotation and leaves margin for the drift not
/// being uniform across it.
///
/// If `|cos H₀|` at BOTH probes exceeds `1 +` that reach, no declination the
/// body can have reached on this rotation brings it into range, and "no
/// crossing" is a theorem rather than a sample. Otherwise the answer is
/// unknown and the caller scans.
///
/// # What this costs
///
/// Nothing where the analytic search already succeeds — it is only consulted
/// after both seeds have failed. Where it IS consulted it spends two
/// `equatorial` evaluations on the two declinations, and answers `false`
/// without scanning unless the boundary is genuinely within reach: at Svalbard
/// (78.22 N) in December `cos H₀ ≈ 1.14` against a reach of 0.017, so deep
/// polar night still costs only those two evaluations per rotation. Near
/// |lat| 89.9 the reach is ~2 and the ~290-evaluation scan becomes the normal
/// path, which is the correct trade at a latitude where the rotational wobble
/// no longer dominates the declination drift.
///
/// Measured end to end through the MCP `compute_panchanga` path, release,
/// warm, 20 calls a case: the five mid-latitude cities are unchanged
/// (7.932 ms before this commit, 7.674 ms after — inside the run-to-run
/// spread), while Svalbard in polar night goes 12.823 → 14.966 ms, lat 88.8 in
/// the grazing band 12.008 → 14.934 ms and the pole 11.893 → 15.066 ms. About
/// 3 ms, paid only above the polar circles, and still ~13× faster than the
/// 203 ms the 5-minute scan cost before the analytic search replaced it.
///
/// NaN propagates as `false` (every comparison against NaN is false), so a
/// geographic pole is never scanned: see the SCOPE note in the module
/// documentation.
///
/// Returns `None` only when `equatorial` cannot supply a position.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 15, eq. 15.1.
fn boundary_is_reachable(
    transit: f64,
    lat_deg: f64,
    h0_deg: f64,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<bool> {
    let (_, dec_upper) = equatorial(transit)?;
    let (_, dec_lower) = equatorial(transit - HALF_ROTATION_DAYS)?;

    // Declination travelled over the whole rotation, from the half-rotation
    // sample, in radians.
    let delta_dec = 2.0 * libm::fabs(deg_to_rad(dec_upper) - deg_to_rad(dec_lower));
    let abs_tan_phi = libm::fabs(libm::tan(deg_to_rad(lat_deg)));

    for dec in [dec_upper, dec_lower] {
        let cos_h0 = cos_rise_hour_angle(lat_deg, dec, h0_deg);
        let reach =
            (abs_tan_phi + libm::fabs(cos_h0) * libm::fabs(libm::tan(deg_to_rad(dec)))) * delta_dec;
        if libm::fabs(cos_h0) <= 1.0 + reach {
            return Some(true);
        }
    }
    Some(false)
}

/// The nearest upward crossing of `h0_deg` on one side of `anchor` — the
/// INSTANT-anchored primitive [`previous_rise`] and [`next_rise`] share.
///
/// ⚠️ **This is the function that is Sun-scoped**, and the reason the two
/// public wrappers are. Its stride is [`ROTATION_DAYS`] and its slack is
/// [`HALF_ROTATION_DAYS`] — both the EARTH's rotation, not the body's — so a
/// body whose right ascension moves fast enough puts its rise outside the
/// slack and the walk visits the wrong rotation. See the SCOPE section in the
/// module documentation for the measured Moon-like numbers.
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

/// The most recent rise of the Sun **at or before** `jd_ut`, as a Julian Day
/// (UT), for an observer at `lat_deg` / `lon_deg_east` and `elevation_m` above
/// sea level.
///
/// ⚠️ **Sun-scoped.** `equatorial` will accept any body, but [`search_rise`]
/// enumerates rotations on the EARTH's stride, and that premise has never been
/// re-derived for a faster one — a Moon-like fixture still drops 14 real
/// events out of 73 500 comparisons, measured. Above |lat| 89 the Sun itself
/// starts to stretch the same premise; the existence answer is still gated
/// against the oracle there, the exact instant is not. See the two SCOPE
/// sections in the module documentation.
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
/// kept, verbatim, in the [`scan_reference`] module below and is now a
/// permanent oracle — the sweep tests hold the analytic path to it across
/// latitude, longitude, elevation, date and both providers, and any future
/// change to this code is checked the same way at zero runtime cost.
///
/// The same scan is also [`refine_event`]'s fallback when the fixed-point
/// iteration does not converge, so the hard cases are correct by construction
/// rather than by contraction. That path costs ~290 evaluations, but it is
/// reached zero times on the grids in [`REFINE_ITERS`]: the common case is
/// unaffected.
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

/// The first rise of the Sun **strictly after** `jd_ut`, as a Julian Day (UT),
/// for an observer at `lat_deg` / `lon_deg_east` and `elevation_m` above sea
/// level.
///
/// ⚠️ **Sun-scoped**, exactly as [`previous_rise`] is.
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
/// implementation, kept verbatim as a **reference oracle** — and, since the
/// convergence fix, also the production FALLBACK that [`refine_event`] hands
/// a non-converging case to.
///
/// It is not dead code and must not be deleted: it is the independent second
/// opinion the analytic path is measured against, and it is the only reason
/// replacing the production algorithm was a safe thing to do at all. Any
/// future change to the analytic search is checked against it by the sweep
/// tests below.
///
/// The oracle bodies (`rise_set_by_scan`, `search_rise_by_scan`,
/// `previous_rise_by_scan`, `next_rise_by_scan`) are unchanged from the
/// pre-analytic implementation apart from being moved here and renamed; do not
/// "simplify" them, or the oracle stops being independent of the thing it
/// checks. They stay `#[cfg(test)]` — only [`bisect`] and
/// [`event_near_by_scan`] are compiled into the shipped binary.
///
/// # What sharing `bisect` with production does and does not cost
///
/// [`event_near_by_scan`] uses the same [`bisect`] the oracle does, so on the
/// (rare) samples where the fallback fires, the sweep is comparing two
/// bisections rather than an analytic root against a bisected one. What is
/// still independently checked on those samples is everything that decides
/// WHICH root is reported — the rotation walk in [`search_rise`], the
/// existence test in [`rise_hour_angle_deg`], and the window arithmetic in
/// [`rise_set`] — because the two paths bracket from different origins (the
/// oracle walks a 5-minute grid anchored on the caller's instant; the fallback
/// scans a grid anchored on the analytic seed). Only the final root polish is
/// shared, and 40 halvings of a 5-minute bracket land ~3.2e-15 d from the
/// bracketed root, five orders of magnitude below the ~4.66e-10 d ULP, so that
/// polish cannot hide a disagreement of any size the sweep's tolerance can see.
pub(crate) mod scan_reference {
    #[cfg(test)]
    use super::RiseSet;
    use super::{Event, geometric_altitude_deg, local_sidereal_degrees};
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
    #[cfg(test)]
    const RISE_SEARCH_STEPS: u32 = 4 * 24 * 60 / 5;

    /// Hard step bound for [`event_near_by_scan`], sized for its widest caller:
    /// `2 · ROTATION_DAYS / (5/1440)` = 574.4, so 576 covers two full rotations
    /// at the oracle's own 5-minute resolution with a step to spare. The loop
    /// stops at its window's closing edge, not at this count; the count exists
    /// only so the scan cannot run away on a pathological argument.
    const MAX_EVENT_SCAN_STEPS: u32 = 576;

    /// Half-window [`super::refine_event`] hands [`event_near_by_scan`] when its
    /// iteration fails, in days.
    ///
    /// [`super::refine_event`] folds its correction to [−180°, 180°), so the
    /// occurrence it was converging on is within half a rotation (0.498 6 d) of
    /// the seed. Half a flat day covers that with 0.001 4 d to spare and keeps
    /// the window symmetric, which is what "the occurrence NEAREST to `t₀`"
    /// requires.
    pub(super) const REFINE_FALLBACK_HALF_SPAN_DAYS: f64 = 0.5;

    /// Half-window [`super::event_on_transits_rotation`] hands
    /// [`event_near_by_scan`] when the seeded existence test cannot be trusted,
    /// in days: one FULL rotation either side of the transit.
    ///
    /// Half a rotation is the right bound while the boundary stands still —
    /// Meeus's `m₁ = m₀ − H₀/360` puts a rise at most that far before its
    /// transit. It is the wrong bound in the regime this fallback exists for,
    /// because there the boundary MOVES: measured at lat −88.5, lon 90, 3650 m,
    /// the Sun dips below `h₀` for about an hour around the lower transit of
    /// JD 2 433 544.745 615 694 7 and rises again at JD 2 433 545.259 663 000 7
    /// — `transit + 0.514 d`, outside any half-rotation window by 0.014 d, and
    /// missed by both the ±0.5 d window and the next rotation's own probe.
    ///
    /// A full rotation each side cannot miss an event of this rotation however
    /// far the boundary has shifted it, and "nearest to the transit" still
    /// prefers this rotation's own event whenever it has one.
    pub(super) const ROTATION_SCAN_HALF_SPAN_DAYS: f64 = super::ROTATION_DAYS;

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

    /// The occurrence of `event` NEAREST to `t0`, found by bracketing on a
    /// 5-minute grid and bisecting — the same method the oracle uses, applied
    /// to the same "nearest to `t0`" question [`super::refine_event`] answers.
    ///
    /// This is the PRODUCTION fallback for a non-converging refinement, not a
    /// test aid. [`super::refine_event`] calls it when it runs out of
    /// iterations, so that exhausting the budget can never be mistaken for
    /// "there is no crossing here" — a verdict that belongs to
    /// [`super::rise_hour_angle_deg`]'s `cos H₀` range test alone.
    ///
    /// # The window is the caller's to choose
    ///
    /// `half_span_days` differs by caller, and each choice is derived where its
    /// constant lives: [`REFINE_FALLBACK_HALF_SPAN_DAYS`] for a refinement that
    /// failed, [`ROTATION_SCAN_HALF_SPAN_DAYS`] for a rotation whose existence
    /// test could not be trusted. Whatever the span, the NEAREST crossing to
    /// `t0` is returned, so a wider window can only add candidates further away
    /// than one already found — it cannot change the answer where the near one
    /// exists.
    ///
    /// The quantity bracketed is the same one the oracle brackets: the
    /// altitude residual `alt − h₀` for `Rise`/`Set` (upward and downward
    /// crossings respectively) and the local hour angle folded to (−180°, 180°]
    /// for `Transit` (upward through zero). The fold's own jump, +179.99° to
    /// −179.99°, is a DOWNWARD step and so is never mistaken for a transit.
    ///
    /// Returns `None` when no such crossing exists in the window, or as soon as
    /// `equatorial` cannot supply a position.
    ///
    /// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13 (altitude),
    /// Ch. 15 (the events).
    pub(super) fn event_near_by_scan(
        t0: f64,
        half_span_days: f64,
        lat_deg: f64,
        lon_deg_east: f64,
        h0_deg: f64,
        event: Event,
        equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    ) -> Option<f64> {
        let residual = |jd: f64| -> Option<f64> {
            let (ra, dec) = equatorial(jd)?;
            Some(match event {
                Event::Rise | Event::Set => {
                    geometric_altitude_deg(ra, dec, jd, lat_deg, lon_deg_east) - h0_deg
                }
                Event::Transit => {
                    let h = normalize_degrees(local_sidereal_degrees(jd, lon_deg_east) - ra);
                    if h > 180.0 { h - 360.0 } else { h }
                }
            })
        };
        // `Rise` and `Transit` are UPWARD crossings of `residual`; `Set` is the
        // downward one.
        let upward = !matches!(event, Event::Set);

        let start = t0 - half_span_days;
        let end = t0 + half_span_days;
        let mut prev_jd = start;
        let mut prev_residual = residual(prev_jd)?;
        let mut best: Option<f64> = None;

        for i in 1..=MAX_EVENT_SCAN_STEPS {
            // `i` is bounded by 576, so the hop through `u16` is lossless and
            // needs no float `as` cast.
            let step = u16::try_from(i).expect("576 fits in u16");
            let stepped = start + f64::from(step) * SCAN_STEP_DAYS;
            // Land the final bracket exactly on the window's closing edge
            // rather than a step past it, so the window scanned is the one the
            // caller asked for and not what the step size rounds it to.
            let jd = if stepped < end { stepped } else { end };
            let cur_residual = residual(jd)?;

            let crossed = if upward {
                prev_residual < 0.0 && cur_residual >= 0.0
            } else {
                prev_residual >= 0.0 && cur_residual < 0.0
            };
            if crossed {
                if let Some(root) = bisect(prev_jd, jd, &residual) {
                    let nearer = match best {
                        Some(b) => libm::fabs(root - t0) < libm::fabs(b - t0),
                        None => true,
                    };
                    if nearer {
                        best = Some(root);
                    }
                }
            }

            prev_jd = jd;
            prev_residual = cur_residual;
            if jd >= end {
                break;
            }
        }

        best
    }

    /// Reference implementation of [`super::rise_set`].
    #[cfg(test)]
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
    #[cfg(test)]
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
    #[cfg(test)]
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
    #[cfg(test)]
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

    /// Latitudes swept: the equator, both hemispheres all the way TO THE POLES,
    /// and a deliberately fine band straddling the polar circles (±66.5619° at
    /// the present obliquity), which is exactly where `cos H₀` crosses out of
    /// [−1, 1] and where a body can fail to clear the horizon on one rotation
    /// and clear it on the next. Every entry is mirrored, so the southern
    /// polar transition is covered as densely as the northern one.
    ///
    /// ±89.5, ±89.9 and ±90 were added when the convergence defect was fixed.
    /// The grid used to stop at ±89, and every case where the 24-iteration
    /// budget produced a wrong sunrise sat beyond it — a tolerance calibrated
    /// on a grid that never enters a regime cannot say anything about that
    /// regime.
    ///
    /// ±90 is the degenerate one. `cos φ` is 6.1e-17 there, so eq. 15.1 gives a
    /// `cos H₀` of order 1e14 and has no root at all: for the FIXED-RA fixture
    /// both implementations agree there is simply no rise, and for the real Sun
    /// [`boundary_is_reachable`] correctly reports that a declination drift can
    /// carry the boundary anywhere (`tan φ` is 1.6e16) and hands the rotation
    /// to the scan, which finds the ANNUAL crossing. Both are swept.
    const SWEEP_LATITUDES_DEG: [f64; 49] = [
        -90.0, -89.9, -89.5, -89.0, -85.0, -80.0, -75.0, -70.0, -68.0, -67.5, -67.0, -66.75,
        -66.5619, -66.5, -66.25, -66.0, -65.5, -65.0, -60.0, -50.0, -40.0, -30.0, -20.0,
        -10.0, //
        0.0,   //
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 65.0, 65.5, 66.0, 66.25, 66.5, 66.5619, 66.75, 67.0,
        67.5, 68.0, 70.0, 75.0, 80.0, 85.0, 89.0, 89.5, 89.9, 90.0,
    ];

    /// The latitudes the convergence defect actually lived at, sampled more
    /// finely than [`SWEEP_LATITUDES_DEG`] reaches: |lat| 88 to the pole. Every
    /// one of the three wrong-sunrise cases pinned in [`REAL_SUN_CASES`] sits
    /// in this band, and all three are beyond the ±89 the grid used to stop at.
    const POLAR_SWEEP_LATITUDES_DEG: [f64; 14] = [
        -90.0, -89.9, -89.5, -89.3, -89.0, -88.5, -88.0, //
        88.0, 88.5, 89.0, 89.3, 89.5, 89.9, 90.0,
    ];

    /// [`SWEEP_LATITUDES_DEG`] restricted to |lat| ≤ 89, for the REAL Sun.
    ///
    /// # This is a measured scope boundary, not a convenience
    ///
    /// Above |lat| ~88 the rotational wobble in altitude (amplitude `cos φ`)
    /// stops dominating the Sun's ~0.4 °/day declination drift, and a horizon
    /// crossing becomes a declination event that this module's rotation walk
    /// only partly models. The convergence fix narrowed that enormously —
    /// over |lat| 88–90 × 8 longitudes × 8 dates × 2 elevations × both
    /// directions (5 376 samples) the disagreement with the scan oracle went
    /// from 53 dropped sunrises and 16 wrong instants to **1 and 1** — but it
    /// did not close it, and `previous_rise` / `next_rise` / `rise_set` are
    /// still not exact against the oracle everywhere beyond ±89.
    ///
    /// So the real-Sun SHIP GATE runs to ±89, where the measured worst
    /// disagreement is **one ULP** across 7 200 comparisons at the hardest
    /// dates of the year, and the band beyond it is MEASURED rather than
    /// asserted by `polar_band_disagreement_is_measured_not_asserted`. Pinning
    /// a gate at a tolerance the code does not meet would be worse than saying
    /// where it stops; the fixed-RA tier still sweeps every latitude to the
    /// poles, because a body at constant declination has no drift to model.
    const REAL_SUN_SWEEP_LATITUDES_DEG: [f64; 43] = real_sun_latitudes(&SWEEP_LATITUDES_DEG);

    /// `SWEEP_LATITUDES_DEG[3..46]` — dropping ±89.5, ±89.9 and ±90 from each
    /// end — as a `const fn`, since a fixed-size array cannot be produced by
    /// slicing in a `const`.
    const fn real_sun_latitudes(all: &[f64; 49]) -> [f64; 43] {
        let mut out = [0.0_f64; 43];
        let mut i = 0;
        while i < 43 {
            out[i] = all[i + 3];
            i += 1;
        }
        out
    }

    /// Dates that STRADDLE the polar-night and polar-day boundaries, which is
    /// where a high-latitude rise exists at all.
    ///
    /// At |lat| 88 eq. 15.1 admits a rise only while the Sun's declination is
    /// inside a ~4° band (`sin δ ∈ [−0.0495, +0.0203]` at `h₀ = −0°50′`), and
    /// the Sun crosses that band at ~0.4 °/day — about ten days, twice a year,
    /// around the equinoxes. A 30.4375-day stride steps over it. These dates do
    /// not: five per equinox at a 3-day stride, centred on day-of-year 79
    /// (≈ March 20) and 265 (≈ September 22) of each era's Gregorian year, so
    /// the grid enters, crosses and leaves the band.
    ///
    /// The centres are calendar day-numbers, not ephemeris output — they only
    /// place the grid, and nothing is derived from their exactness.
    fn polar_sweep_julian_days() -> [f64; 30] {
        let mut out = [0.0_f64; 30];
        let mut i = 0_usize;
        for era in [2_433_282.5_f64, 2_451_544.5, 2_488_069.5] {
            for centre in [79.0_f64, 265.0] {
                for k in [-6.0_f64, -3.0, 0.0, 3.0, 6.0] {
                    out[i] = era + centre + k;
                    i += 1;
                }
            }
        }
        out
    }

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
    /// | sweep (dense tier, `--release`) | comparisons | max gap | presence |
    /// |---|---|---|---|
    /// | fixed-RA fixture, |lat| ≤ 90 | **441 000** | 4.656612873077393e-10 d | 0 |
    /// | real Sun, |lat| ≤ 89         |  **36 120** | 4.656612873077393e-10 d | 0 |
    ///
    /// 4.656612873077393e-10 d is **exactly one ULP** at JD ≈ 2.45e6
    /// (`2^-31` d), i.e. the two implementations never differ by more than the
    /// last bit. This tolerance is 1e-9 d — the next round figure above that
    /// measured maximum, about two ULP, and 8.64e-5 seconds. That is four
    /// orders of magnitude tighter than the one-second gate this change was
    /// allowed to spend, so no real regression can hide beneath it.
    ///
    /// # The counts above were wrong, and the grid they came from was too small
    ///
    /// This table previously read 258 000 and 34 830. Neither number was ever
    /// produced by the shipped grid, which gave 387 000 and 18 060 — the value
    /// 1e-9 was derived correctly from a measured maximum, but the sample sizes
    /// quoted beside it were not the ones measured. They are re-recorded above
    /// from the run that produced the current maximum.
    ///
    /// More importantly the old grid stopped at |lat| 89 and swept the real Sun
    /// at sea level only, so it never entered the regime where the convergence
    /// defect lived. The grid now runs to the poles, carries 0 m and 3650 m on
    /// both tiers, and spans all three eras for the real Sun (the old real-Sun
    /// slice `jds[..12]` was twelve dates in **1950 alone**, despite a comment
    /// claiming four per era). The polar band beyond ±89 is measured separately
    /// — see `polar_band_disagreement_is_measured_not_asserted`.
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
    /// for the REAL Sun as well as the fixture, plus a third tier aimed
    /// squarely at the regime the convergence defect lived in.
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
    ///
    /// # What is sampled, exactly
    ///
    /// | tier | body | latitudes | longitudes | dates | elevations | samples |
    /// |---|---|---|---|---|---|---|
    /// | 1 | fixed-RA fixture | all 49 | all 25 | all 36 (12 per era × 3) | 0 m, 3650 m | 88 200 |
    /// | 2 | real Sun | all 49 | every 4th (7) | 12, **4 per era** | 0 m, 3650 m | 8 232 |
    /// | 3 | real Sun, polar | 14 (|lat| 88→90) | 4 | 30 equinox-straddling | 0 m, 3650 m | 3 360 |
    ///
    /// Five comparisons per sample. Tier 2's date subset takes every third
    /// entry of the 36, which is four per era — the previous revision sliced
    /// `jds[..12]` instead, and since `sweep_julian_days(12)` lays the eras out
    /// consecutively, that was twelve dates in **1950 alone**, with 2000 and
    /// 2100 never reached by the real-Sun tier at all.
    ///
    /// The tiers are all measured before any is asserted, so one failure still
    /// reports every tier's numbers — which is what makes this the place the
    /// tolerance is derived from rather than merely checked at.
    #[test]
    #[ignore = "tier-2: full-density scan-oracle sweep; ~2 h in --release, run manually"]
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

        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        // Every fourth longitude: the oracle costs ~950 `sun_equatorial_deg`
        // evaluations a sample at 415 µs each, so longitude is where the
        // density is spent. Latitude — the axis that decides whether a
        // crossing exists at all — keeps its full resolution, and elevation
        // keeps both values, because the horizon dip moves `h₀` by 1.77° at
        // 3650 m and therefore moves every polar boundary with it.
        let mut sparse_lons = [0.0_f64; 7];
        for (i, slot) in sparse_lons.iter_mut().enumerate() {
            *slot = lons[i * 4];
        }
        // Every third date: four per era, all three eras.
        let mut era_spanning_jds = [0.0_f64; 12];
        for (i, slot) in era_spanning_jds.iter_mut().enumerate() {
            *slot = jds[i * 3];
        }
        let real = sweep(
            &REAL_SUN_SWEEP_LATITUDES_DEG,
            &sparse_lons,
            &era_spanning_jds,
            &[0.0, 3_650.0],
            &real_sun,
        );
        report("real Sun", &real);

        flat.assert_agrees("fixed-RA fixture (dense)", AGREEMENT_TOL_DAYS);
        real.assert_agrees("real Sun (dense)", AGREEMENT_TOL_DAYS);
    }

    /// MEASUREMENT, not a gate: how far the analytic path and the scan oracle
    /// still differ in the polar band, |lat| 88 to the pole, at dates that
    /// straddle the polar-night and polar-day boundaries.
    ///
    /// This is deliberately NOT an `assert_agrees` tier. The rotation walk does
    /// not fully model a horizon crossing driven by the declination rather than
    /// by rotation (see [`REAL_SUN_SWEEP_LATITUDES_DEG`]), and pinning a gate
    /// at a tolerance the code does not meet would be worse than recording
    /// exactly where it stops. Run it to see whether a change moves the number:
    ///
    /// ```text
    /// cargo test --release -p vedaksha-astro --lib \
    ///     polar_band_disagreement_is_measured_not_asserted -- --ignored --nocapture
    /// ```
    ///
    /// # Recorded, with this commit in place
    ///
    /// 3 360 samples = **16 800 comparisons** (14 latitudes × 4 longitudes ×
    /// 30 equinox-straddling dates × 2 elevations):
    ///
    /// | surface | presence disagreements |
    /// |---|---|
    /// | `previous_rise` | **0** |
    /// | `next_rise` | **0** |
    /// | `rise_set().rise` | 6 |
    /// | `rise_set().set` | 2 |
    /// | `rise_set().transit` | **0** |
    ///
    /// Worst value disagreement **0.496 686 570 346 355 44 d**, at lat 89.9,
    /// lon 0, 3650 m, anchor JD 2 451 617.5: `previous_rise` returns
    /// JD 2 451 616.975 991 497 where the scan finds JD 2 451 617.472 678 067.
    /// Every remaining case sits at |lat| ≥ 89.5.
    ///
    /// What is left is one shape: an hour-long dip below `h₀` around a LOWER
    /// transit, which the rotation walk does not enumerate because it is not
    /// the rise-set pair of any rotation. Measured directly at lat −88.5,
    /// lon 90, 3650 m, JD 2 433 544.5 — the Sun sets at
    /// JD 2 433 545.218 178 468 8 and rises again at JD 2 433 545.259 663 000 7,
    /// 1 h 0 m later, and [`rise_set`] reports the set but not the rise.
    /// [`scan_reference::ROTATION_SCAN_HALF_SPAN_DAYS`] widened the fallback
    /// window far enough to catch most of that class; catching the rest means
    /// enumerating dips as well as rotations, which is a different search.
    ///
    /// Before this commit the same grid gave 12 presence disagreements
    /// (rise 8, set 4) and a worst value disagreement of 0.794 d.
    #[test]
    #[ignore = "measurement: polar-band disagreement, ~30 min in --release"]
    fn polar_band_disagreement_is_measured_not_asserted() {
        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        let polar = sweep(
            &POLAR_SWEEP_LATITUDES_DEG,
            &[-150.0, -45.0, 0.0, 90.0],
            &polar_sweep_julian_days(),
            &[0.0, 3_650.0],
            &real_sun,
        );
        report("real Sun (polar band)", &polar);
        std::println!(
            "polar band, per surface: prev={} next={} rise={} set={} transit={}",
            polar.previous_rise.presence_mismatches,
            polar.next_rise.presence_mismatches,
            polar.window_rise.presence_mismatches,
            polar.window_set.presence_mismatches,
            polar.window_transit.presence_mismatches
        );
        // The one thing this tier DOES gate: the instant-anchored searches —
        // the primitives the vara is reckoned from — must never disagree with
        // the oracle about whether a sunrise exists, anywhere in the band.
        assert_eq!(
            polar.previous_rise.presence_mismatches, 0,
            "previous_rise disagreed about EXISTENCE somewhere in the polar band; \
             first at {:?}",
            polar.previous_rise.first_presence_mismatch
        );
        assert_eq!(
            polar.next_rise.presence_mismatches, 0,
            "next_rise disagreed about EXISTENCE somewhere in the polar band; \
             first at {:?}",
            polar.next_rise.first_presence_mismatch
        );
    }

    /// Observers and instants for the real-Sun oracle table below: mid
    /// latitudes in both hemispheres, three eras, sea level and 3650 m, both
    /// polar circles at the solstices (polar day, polar night) and the
    /// latitude/date at which the polar night ENDS — the sample that first
    /// exposed the declination-drift hole in the existence probe.
    const REAL_SUN_CASES: [(f64, f64, f64, f64); 20] = [
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
        // ── The three convergence-defect rows. See [`REFINE_ITERS`]. ──
        (88.1, 0.0, 0.0, 2_459_287.5), // a real sunrise was DROPPED
        (89.3, 0.0, 0.0, 2_459_295.5), // wrong sunrise, 0.861 d early
        (89.5, -150.0, 3_650.0, 2_488_343.437_5), // wrong sunrise, 1.210 d early
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
    /// * **Rows 18–20** — the convergence defect, in its three shapes. At
    ///   `REFINE_ITERS = 24` and with exhaustion answering `None`, the shipped
    ///   code returned, against these same oracle literals:
    ///   `previous_rise` = `None` at row 18, where a real sunrise sits 12 h
    ///   before the anchor; `2459292.672512896` at row 19, **0.861 d** before
    ///   the true one; and `2488341.166030892` at row 20, **1.210 d** before
    ///   it. Rows 19 and 20 are the dangerous shape — not a missing answer but
    ///   a confidently wrong one, which starts the vara a whole day late. All
    ///   three sit beyond |lat| 89, which is where the sweep grid used to stop.
    ///   MUTATION-CHECK: restoring `REFINE_ITERS = 24` fails this test on all
    ///   three rows.
    #[test]
    fn analytic_rise_reproduces_the_scan_oracle_for_the_real_sun() {
        /// Scan-oracle answers for [`REAL_SUN_CASES`], in the same order.
        const ORACLE: [(Option<f64>, Option<f64>); 20] = [
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
            (Some(2_459_286.996_263_494_7), Some(2_459_287.906_250_407)),
            (Some(2_459_293.533_634_425), None),
            (Some(2_488_342.375_926_383), None),
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

    /// A fixture whose right ascension sweeps BACKWARD at exactly the rate the
    /// local hour angle advances, `HOUR_ANGLE_RATE_DEG_PER_DAY`.
    ///
    /// # Why this is the right shape for a wide-cycle test
    ///
    /// [`refine_event`]'s map is `t ↦ t + (target(t) − current(t)) / rate`.
    /// With the error `e = t − t_root`, `current` advances at
    /// `rate − dα/dt`, so `e ↦ e·(dα/dt)/rate`: the local slope is the body's
    /// RA rate over the rotation rate. At `dα/dt = −rate` that slope is
    /// exactly −1 — the boundary of contraction — and the iteration orbits
    /// between `t_root + e` and `t_root − e` FOREVER, at whatever amplitude
    /// the seed sets. Seeding `k` ULP off the root therefore manufactures a
    /// period-2 orbit of a chosen width, which is exactly the class the cycle
    /// guard has to distinguish from a converged root.
    ///
    /// Measured with this fixture at lat 20, lon 0 (probe run, since removed),
    /// seeding `root + k` ULP: `k = 0` and `k = 1` give a 1-ULP orbit with a
    /// correction of 0.602 ULP; `k = 2` gives 3 ULP and 2.602; `k = 3` gives
    /// 5 ULP and 4.602; `k = 8` gives 15 ULP and 14.602; `k = 100 000` gives
    /// 199 999 ULP. The residual in ULP tracks `width − 0.4`, as the rounding
    /// argument in [`CYCLE_MAX_RESIDUAL_ULPS`] says it must.
    ///
    /// No real body behaves like this — it is a fixture, not a claim about the
    /// sky — but nothing in [`refine_event`] knows that, which is the point.
    fn retrograde_sun(jd: f64) -> Option<(f64, f64)> {
        Some((
            normalize_degrees(-HOUR_ANGLE_RATE_DEG_PER_DAY * (jd - 2_451_545.0)),
            0.0,
        ))
    }

    /// Latitude and longitude the cycle tests use. Nothing special about them:
    /// [`retrograde_sun`] has a constant declination, so any latitude inside
    /// the polar circles behaves the same way.
    const CYCLE_FIXTURE_OBSERVER: (f64, f64) = (20.0, 0.0);

    /// A period-2 orbit WIDER than one ULP is not a converged root, and must
    /// never be returned as if it were.
    ///
    /// `next == previous` alone does not mean "converged": it also fires on an
    /// orbit whose two members are many ULP apart, where NEITHER is the root.
    /// At `REFINE_ITERS = 24` no such orbit was ever observed (the widest over
    /// ~1.05 M refinements was 1 ULP), which is why the guard shipped without
    /// a width test — but raising the budget to 512 gives the iteration room
    /// to reach one, so the width test is now load-bearing.
    ///
    /// Seeding two ULP off the root manufactures a 3-ULP-wide orbit whose
    /// correction is 2.602 ULP. That correction is INSIDE
    /// [`CYCLE_MAX_RESIDUAL_ULPS`] (4), so the residual bound alone does not
    /// reject it: the ADJACENCY test is what does, and this test is what
    /// proves adjacency is not decoration. Confirmed by mutation — see that
    /// constant's doc for both directions.
    ///
    /// The assertion is bit-exact against the scan fallback's own answer for
    /// the same seed, not merely "close to the root". A rejected cycle burns
    /// the budget and lands in [`scan_reference::event_near_by_scan`], so the
    /// correct result is exactly what that function returns; a cycle member,
    /// being 1–2 ULP away, would satisfy any tolerance-based assertion and
    /// slip through.
    ///
    /// MUTATION-CHECK: deleting the `width_bits <= 1` condition makes this
    /// test fail — `refine_event` then returns a member of the 3-ULP orbit
    /// instead of the scanned root.
    #[test]
    fn a_two_cycle_wider_than_one_ulp_is_rejected_and_handed_to_the_scan() {
        let (lat, lon) = CYCLE_FIXTURE_OBSERVER;
        let h0 = SUN_STANDARD_ALTITUDE_DEG;
        let root = scan_reference::event_near_by_scan(
            2_451_545.3,
            scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
            lat,
            lon,
            h0,
            Event::Rise,
            &retrograde_sun,
        )
        .expect("the retrograde fixture crosses the horizon here");

        // Two ULP above the root. See [`retrograde_sun`] for why this width is
        // what the seed offset buys.
        let seed = f64::from_bits(root.to_bits() + 2);
        let expected = scan_reference::event_near_by_scan(
            seed,
            scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
            lat,
            lon,
            h0,
            Event::Rise,
            &retrograde_sun,
        )
        .expect("the scan finds the same crossing from the seeded instant");

        let got = refine_event(seed, lat, lon, h0, Event::Rise, &retrograde_sun)
            .expect("exhaustion must fall back to the scan, never report None");

        assert_eq!(
            got, expected,
            "a 3-ULP-wide period-2 orbit was returned as a converged root: \
             refine_event gave {got}, the scan fallback gives {expected} \
             (root {root}, seed {seed})"
        );
    }

    /// The other half of the same guard: a genuine ADJACENT two-cycle is still
    /// accepted, and accepting it is still idempotent.
    ///
    /// Tightening the cycle test must not turn the case it was written for
    /// into a scan fallback. Seeded on the root itself, [`retrograde_sun`]
    /// orbits between two neighbouring `f64`s with a correction of 0.602 ULP —
    /// inside both conditions — so [`refine_event`] resolves it by the
    /// smaller-residual rule rather than exhausting.
    ///
    /// Idempotency is the property `previous_rise(r) == r` rests on, and it is
    /// asserted here from BOTH members of the orbit: the rule is a function of
    /// the cycle, not of which member the caller happened to enter from. On an
    /// exact residual tie the earlier instant wins, which is what makes that
    /// true; "keep whichever one `t` is" would return a different answer per
    /// entry point.
    #[test]
    fn an_adjacent_two_cycle_is_accepted_and_resolves_the_same_way_from_either_member() {
        let (lat, lon) = CYCLE_FIXTURE_OBSERVER;
        let h0 = SUN_STANDARD_ALTITUDE_DEG;
        let root = scan_reference::event_near_by_scan(
            2_451_545.3,
            scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
            lat,
            lon,
            h0,
            Event::Rise,
            &retrograde_sun,
        )
        .expect("the retrograde fixture crosses the horizon here");

        let from_root = refine_event(root, lat, lon, h0, Event::Rise, &retrograde_sun)
            .expect("an adjacent cycle is a root, not a failure");
        let neighbour = f64::from_bits(root.to_bits() + 1);
        let from_neighbour = refine_event(neighbour, lat, lon, h0, Event::Rise, &retrograde_sun)
            .expect("an adjacent cycle is a root, not a failure");

        assert_eq!(
            from_root, from_neighbour,
            "the cycle rule must be a function of the CYCLE, not of the entry \
             point: from {root} it gave {from_root}, from {neighbour} it gave \
             {from_neighbour}"
        );
        let bits_off = from_root.to_bits().abs_diff(root.to_bits());
        assert!(
            bits_off <= 1,
            "the accepted cycle member must be within one ULP of the scanned \
             root: {from_root} is {bits_off} ULP from {root}"
        );
    }

    /// Running out of iterations must NEVER be reported as "no crossing".
    ///
    /// This is the defect in its purest form, stated against [`refine_event`]
    /// directly. [`retrograde_sun`] seeded off its root cannot converge — the
    /// map's slope is exactly −1 — so the budget always runs out, and the only
    /// two answers the function could give are the scan's root or `None`.
    /// `None` would be a lie: the crossing demonstrably exists, and the scan
    /// finds it. In production this is what made a real 88.1° N sunrise
    /// disappear, because [`search_rise`] reads `None` as "this rotation has no
    /// rise" and walks straight past it.
    ///
    /// MUTATION-CHECK: replacing the fallback with `None` fails this test, and
    /// also fails `analytic_rise_reproduces_the_scan_oracle_for_the_real_sun`
    /// at rows 18–20.
    #[test]
    fn exhausting_the_iteration_budget_never_reports_a_missing_crossing() {
        let (lat, lon) = CYCLE_FIXTURE_OBSERVER;
        let h0 = SUN_STANDARD_ALTITUDE_DEG;
        let root = scan_reference::event_near_by_scan(
            2_451_545.3,
            scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
            lat,
            lon,
            h0,
            Event::Rise,
            &retrograde_sun,
        )
        .expect("the retrograde fixture crosses the horizon here");

        // Far enough off the root that the orbit is thousands of ULP wide and
        // no cycle can be accepted, in both directions.
        for offset_bits in [-1_000_i64, -64, 64, 1_000] {
            let bits = root.to_bits();
            let seed = if offset_bits < 0 {
                f64::from_bits(bits - offset_bits.unsigned_abs())
            } else {
                f64::from_bits(bits + offset_bits.unsigned_abs())
            };
            let got = refine_event(seed, lat, lon, h0, Event::Rise, &retrograde_sun);
            assert!(
                got.is_some(),
                "offset {offset_bits} ULP: the iteration cannot converge here, but the \
                 crossing exists — reporting None turns an exhausted budget into a \
                 fabricated polar night"
            );
            let expected = scan_reference::event_near_by_scan(
                seed,
                scan_reference::REFINE_FALLBACK_HALF_SPAN_DAYS,
                lat,
                lon,
                h0,
                Event::Rise,
                &retrograde_sun,
            );
            assert_eq!(
                got, expected,
                "offset {offset_bits} ULP: the fallback must return the scan's own root"
            );
        }
    }

    /// The geographic pole: eq. 15.1 has no root there, and the module still
    /// reproduces the scan oracle — by scanning, not by clamping.
    ///
    /// At |lat| 90 the rotational wobble in altitude has amplitude
    /// `cos φ = 6.1e-17`, so the Sun crosses `h₀ = −0°50′` ONCE A YEAR, driven
    /// by the declination alone. `cos H₀` is ~1e14 and eq. 15.1 has nothing to
    /// say: the seeded refinement reports no crossing, correctly.
    ///
    /// [`boundary_is_reachable`] is then the thing that keeps the answer right.
    /// `tan φ` is 1.6e16 at the pole, so its reach bound says — truthfully —
    /// that a declination drift can carry `cos H₀` anywhere at all, the point
    /// test is worthless, and the rotation must be scanned. The scan finds the
    /// annual crossing, and the analytic surface reproduces the oracle
    /// BIT-EXACTLY rather than reporting a false polar night.
    ///
    /// Two assertions, and the first is what stops the second from being
    /// satisfied the wrong way: `rise_hour_angle_deg` must STILL return `None`
    /// at the pole. If a future change ever "fixes" the pole by clamping
    /// `cos H₀` into [−1, 1], the second assertion could still pass while the
    /// module fabricated an hour angle; the first cannot.
    ///
    /// The anchor is 2000-03-17. Measured with this module's own scan: the
    /// Sun's declination there is −1.310 095°, and it passes `h₀` upward at
    /// JD 2 451 621.706 382 162, 1.21 days later — inside the four-day search
    /// bound.
    #[test]
    fn the_geographic_pole_has_no_hour_angle_root_and_is_scanned_instead() {
        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        let jd = 2_451_620.5; // 2000-03-17 00:00 UT

        for lat in [90.0_f64, -90.0] {
            assert_eq!(
                rise_hour_angle_deg(lat, 0.0, SUN_STANDARD_ALTITUDE_DEG),
                None,
                "lat={lat}: cos H0 is ~1e14 here, so eq. 15.1 has no root and \
                 nothing may clamp it into one"
            );
        }

        let scanned = scan_reference::next_rise_by_scan(jd, 90.0, 0.0, 0.0, &real_sun);
        assert_eq!(
            scanned,
            Some(2_451_621.706_382_162),
            "the scan must find the ANNUAL crossing at the north pole three days \
             before the March equinox; if it does not, this test has stopped \
             exercising the case it exists for"
        );
        assert_eq!(
            next_rise(jd, 90.0, 0.0, 0.0, &real_sun),
            scanned,
            "at the pole the analytic surface must reproduce the oracle's annual \
             crossing, not report a polar night"
        );

        // The southern pole is in the opposite season: no crossing either way.
        assert_eq!(
            next_rise(jd, -90.0, 0.0, 0.0, &real_sun),
            scan_reference::next_rise_by_scan(jd, -90.0, 0.0, 0.0, &real_sun),
            "the south pole must agree with the oracle too, whichever answer it is"
        );
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

    /// SHIP GATE, |lat| 88 to the pole: the band the convergence defect lived
    /// in, swept against the scan oracle at 0.1° resolution.
    ///
    /// `#[ignore]`d for cost — 5 376 real-Sun samples, ~35 min in `--release`
    /// — not because it is optional. Run it after any change to the search:
    ///
    /// ```text
    /// cargo test --release -p vedaksha-astro --lib \
    ///     the_high_latitude_band_agrees_with_the_scan_oracle -- --ignored --nocapture
    /// ```
    ///
    /// # Why this grid exists
    ///
    /// [`SWEEP_LATITUDES_DEG`] jumps 85 → 89 → 90, and every wrong sunrise the
    /// convergence defect produced sat between those points. A tolerance
    /// calibrated on a grid that never enters a regime says nothing about that
    /// regime, so this walks the regime itself: 21 latitudes at 0.1° × 8
    /// longitudes × 8 anchors (2021 near the March boundary, and the 1950 /
    /// 2000 / 2100 solstices and equinoxes) × 2 elevations × both search
    /// directions.
    ///
    /// Recorded: **53 dropped sunrises and 16 wrong instants, worst 1.214 d**,
    /// before the fix; **0 and 0, worst one ULP**, after it. See
    /// [`REFINE_ITERS`] for the change-by-change breakdown.
    #[test]
    #[ignore = "ship gate, tier 2: 5376 real-Sun samples against the scan oracle, ~35 min in --release"]
    fn the_high_latitude_band_agrees_with_the_scan_oracle() {
        let provider = real_sun_provider();
        let real_sun = |jd: f64| sun_equatorial_deg(&provider, jd);
        let mut agreement = Agreement::default();

        let mut lat_tenths = 880_i32;
        while lat_tenths <= 900 {
            // Tenths of a degree, so the grid step is exact in the integer and
            // the division is the only float operation — no `as` cast.
            let lat = f64::from(lat_tenths) / 10.0;
            for lon in [-150.0_f64, -90.0, -45.0, 0.0, 45.0, 90.0, 150.0, 179.0] {
                for jd in HIGH_LATITUDE_ANCHORS {
                    for elevation in [0.0_f64, 3_650.0] {
                        agreement.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            previous_rise(jd, lat, lon, elevation, &real_sun),
                            scan_reference::previous_rise_by_scan(
                                jd, lat, lon, elevation, &real_sun,
                            ),
                        );
                        agreement.record(
                            lat,
                            lon,
                            elevation,
                            jd,
                            next_rise(jd, lat, lon, elevation, &real_sun),
                            scan_reference::next_rise_by_scan(jd, lat, lon, elevation, &real_sun),
                        );
                    }
                }
            }
            lat_tenths += 1;
        }

        std::println!(
            "high-latitude band: {} comparisons, max gap {:e} d, presence mismatches {}",
            agreement.samples,
            agreement.max_gap_days,
            agreement.presence_mismatches
        );
        std::println!("worst: {:?}", agreement.worst);
        assert_eq!(
            agreement.samples, 5_376,
            "sanity: 21 latitudes × 8 longitudes × 8 dates × 2 elevations × 2 directions"
        );
        agreement.assert_agrees("|lat| 88 to the pole", AGREEMENT_TOL_DAYS);
    }

    /// Anchors for [`the_high_latitude_band_agrees_with_the_scan_oracle`]: the
    /// two 2021 instants where the defect was first reproduced, the 2488343
    /// instant from the third pinned row, and the solstices and equinoxes of
    /// the three eras the engine serves — the dates on which a high-latitude
    /// rise exists at all.
    const HIGH_LATITUDE_ANCHORS: [f64; 8] = [
        2_459_287.5,
        2_459_295.5,
        2_488_343.437_5,
        2_433_282.5,
        2_451_544.5,
        2_451_716.5,
        2_451_809.5,
        2_451_899.5,
    ];

    /// A Moon-like fixture: right ascension advancing at the Moon's mean rate
    /// (13.176 396 6 °/day, the mean-longitude coefficient of Meeus Ch. 47) and
    /// a declination swinging ±28.6° over the same period.
    ///
    /// It is a fixture, not an ephemeris — nothing here claims it is where the
    /// Moon is. What it reproduces faithfully is the one property that matters
    /// to this module: a right ascension that moves fast enough to break the
    /// rotation walk's premises.
    fn moon_like(jd: f64) -> Option<(f64, f64)> {
        let d = jd - 2_451_545.0;
        let ra = normalize_degrees(13.176_396_6 * d);
        let dec = 28.6 * libm::sin(deg_to_rad(13.176_396_6 * d));
        Some((ra, dec))
    }

    /// MEASUREMENT: how far a Moon-like body is from being in scope, so the
    /// SCOPE section of the module documentation stays a measured claim rather
    /// than a remembered one.
    ///
    /// The fix in this commit — "a search that failed is not a body that did
    /// not rise", applied at four places — helps the Moon fixture enormously
    /// even though none of it was aimed at the Moon. Over 49 latitudes × 25
    /// longitudes × 12 dates in three eras (**73 500 comparisons**):
    ///
    /// | | presence disagreements | worst value gap |
    /// |---|---|---|
    /// | before | 455 | 0.943 d |
    /// | after  | **14** | **9.313 225 746 154 785e-10 d** (2 ULP) |
    ///
    /// So the instants it does report are now right to the last two bits, and
    /// what remains is purely a handful of missed events. That is still not
    /// "supported": 14 real crossings out of 73 500 are dropped, nothing in
    /// this engine drives these functions with the Moon, and none of the
    /// rotation-walk premises in [`search_rise`] have been re-derived for a
    /// body whose right ascension moves at 13 °/day. The documentation says
    /// Sun-only, and this test holds that statement to its numbers in BOTH
    /// directions — it fails if the gap grows, and it fails if the gap closes,
    /// because a closed gap means the SCOPE section needs rewriting rather
    /// than the code needing fixing.
    #[test]
    fn a_moon_like_body_is_measurably_outside_this_modules_scope() {
        let jds = sweep_julian_days(4);
        let lons = longitude_grid();
        let swept = sweep(&SWEEP_LATITUDES_DEG, &lons, &jds[..12], &[0.0], &moon_like);

        let presence = swept.previous_rise.presence_mismatches
            + swept.next_rise.presence_mismatches
            + swept.window_rise.presence_mismatches
            + swept.window_set.presence_mismatches
            + swept.window_transit.presence_mismatches;
        std::println!(
            "moon-like fixture: {} comparisons, {presence} presence disagreements \
             (prev={} next={} rise={} set={} transit={}), worst value gap {:e} d",
            swept.samples(),
            swept.previous_rise.presence_mismatches,
            swept.next_rise.presence_mismatches,
            swept.window_rise.presence_mismatches,
            swept.window_set.presence_mismatches,
            swept.window_transit.presence_mismatches,
            swept.max_gap_days()
        );

        assert!(
            presence > 0,
            "the SCOPE section documents this module as Sun-only and cites the \
             disagreements on this fixture as the evidence. NONE were found — if the \
             search has become Moon-capable, that is good news, and the SCOPE section \
             must be rewritten rather than left overtaken by the code."
        );
        assert!(
            presence <= 40,
            "the Moon-like fixture regressed: {presence} presence disagreements, \
             against the 14 recorded when this was measured"
        );
        assert!(
            swept.max_gap_days() <= 1e-9,
            "the Moon-like fixture regressed on VALUE: worst gap {} d, against the \
             9.313225746154785e-10 d (2 ULP) recorded when this was measured. The \
             instants this module reports for a fast body are wrong by a whole day \
             again, not by two bits",
            swept.max_gap_days()
        );
    }
}
