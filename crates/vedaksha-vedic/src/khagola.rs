// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Khagola solvers — ingress, station, heliacal visibility, eclipse.
//!
//! # Sources
//!
//! - Meeus, *Astronomical Algorithms*, 2nd ed.: Ch. 12 (sidereal time, via
//!   [`vedaksha_ephem_core::sidereal_time::gmst`]), Ch. 13 (altitude from
//!   hour angle), Ch. 15 (the Sun's standard rise/set altitude, shared with
//!   [`vedaksha_astro::riseset`]), Ch. 40 (topocentric parallax, oblate-Earth
//!   vector form), Ch. 54 (eclipse shadow-cone geometry).
//! - Espenak, F., NASA/GSFC Eclipse Web Site, "Enlargement of Earth's
//!   Shadows" (the Five Millennium Canon of Lunar Eclipses), equations 1-3 to
//!   1-6: the penumbral and umbral radii in Chauvenet's and in Danjon's
//!   convention. See [`ShadowEnlargement`].
//! - Surya Siddhanta: the classical grahana (eclipse) and udaya/asta
//!   (heliacal rise/set) theory behind the event definitions.
//!
//! # Contract
//!
//! Locale-free canonical values (Julian Days UT, degrees, rashi indices).
//! Every ephemeris query is a caller callback; the engine holds no tables
//! here. [`crate::muhurta::refine_crossing`] is the shared longitude solver
//! — this module brackets, it refines.
//!
//! # Frames and approximations (read before citing a result)
//!
//! - Altitudes use MEAN equinox LST (`gmst` + longitude): nutation (±1")
//!   is below any arcus-visionis criterion.
//! - Topocentric parallax uses the oblate Earth (WGS84 `1/298.257223`,
//!   equatorial radius 6378.137 km), exact vector form — no series.
//! - Constants: AU 149597870.7 km (IAU 2012, exact by definition);
//!   R_sun 695700 km (IAU 2015 nominal); R_moon 1737.4 km (standard mean).
//! - Heliacal visibility is GEOMETRY the caller thresholds: the classical
//!   arcus visionis (extinction, magnitude, season) is the caller's
//!   `critical_altitude_deg`, not an engine table.
//!
//! # Eclipses: what is reported
//!
//! - A **lunar** eclipse is reported when the Moon's limb reaches the
//!   penumbra, that is, when the penumbral magnitude is positive. There is no
//!   larger floor: the shallowest eclipse in NASA/GSFC's 2001 to 2100 catalogue
//!   (2027-07-18, penumbral magnitude 0.0014) is found. The shadow radii use
//!   Danjon's enlargement by default and Chauvenet's on request
//!   ([`ShadowEnlargement`]). A lunar eclipse is geocentric: it is returned
//!   whether or not the Moon is above the observer's horizon.
//! - A **solar** eclipse is reported at a place only if the Sun stands above
//!   the horizon for some part of it. "Above the horizon" means the Sun's
//!   topocentric centre is at or above the standard rise/set altitude of
//!   -50 arcminutes (34' of mean refraction plus the 16' mean semidiameter:
//!   the upper limb on the refracted horizon; Meeus Ch. 15, the value
//!   [`vedaksha_astro::riseset::SUN_STANDARD_ALTITUDE_DEG`] already uses for
//!   sunrise and sunset). The reported sparsha and moksha are the visible
//!   ones, clipped to that sunrise or sunset where the geometric contact falls
//!   outside; [`SolarHorizon`] carries the unclipped contacts and says what was
//!   clipped. Refraction is not applied to the Sun-Moon separation itself.
//!
//! # Heliacal events: Surya Siddhanta ch. IX and X
//!
//! Read in the Sanskrit, in the text of the Bibliotheca Indica edition (F. Hall,
//! 1854) and its later editions as transcribed for GRETIL by M. Yano, whose
//! verse numbers these are (<https://gretil.sub.uni-goettingen.de/>, file
//! `sa_sUryasiddhAnta`). Graded as the
//! other doctrine modules grade: (a) the verse states it; (b) the verse states
//! it and this module takes the exact spherical form of its arithmetic; (c) not
//! in the verse, adopted here.
//!
//! - IX.2-3 (a): Jupiter, Mars, Saturn, and Venus and Mercury while retrograde,
//!   set in the west when their longitude exceeds the Sun's and rise in the east
//!   when it is less; the Moon, and Venus and Mercury while direct, set in the
//!   east when less and rise in the west when greater. [`HeliacalSide`].
//! - IX.4-5 (b): take the Sun and the planet at the moment of sunrise (east) or
//!   sunset (west) and apply the drikkarma, the correction of the longitude for
//!   the latitude (VII.7-10, which VII.11 names as the rule for the setting and
//!   rising of the grahas); the oblique ascensions of the two, in respirations,
//!   divided by sixty, are the kalamsha (degrees of time). [`kalamsha_deg`]
//!   takes the difference of the sidereal times at which the two rise (set), the
//!   body itself at its own `(alpha, delta)`, which has the latitude exactly in
//!   it, rather than the text's sine-table arithmetic.
//! - IX.6 (a): Jupiter 11, Saturn 15, Mars 17. IX.7 (a): Venus 8 for "setting in
//!   the west and rising in the east", 10 for the reverse. IX.8 (a): Mercury 12
//!   when retrograde (`vakri`), 14 when fast-moving. X.1 (a): the Moon is seen
//!   in the west at 12. IX.7 pairs Venus's figures with the horizon; IX.2-3 pairs
//!   the horizons with the motions, so 8 is the retrograde figure and 10 the
//!   direct one (b), the pairing IX.8 gives Mercury outright. The figures are
//!   [`crate::combustion::combustion_orb_deg`]'s.
//! - IX.9 (a): at more kalamsha than these a graha is seen, at less it is not.
//!   The verse is silent on equality; "at least" is adopted (c).
//! - Not implemented: IX.10-11, the days to a heliacal event from the rates (the
//!   finders step the sunrises instead), and IX.12-18, the asterisms and stars.
//!
//! There is no independent published dataset of the days the grahas are first
//! and last seen to check these against: almanacs print conjunctions and
//! elongations, not the day a planet is seen. `tests/khagola_heliacal.rs` checks
//! the rule by hand-worked vectors, a model sky, an independent computation of
//! the kalamsha from the rising times, and properties the rule implies.

use vedaksha_astro::riseset::SUN_STANDARD_ALTITUDE_DEG;
use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::obliquity::mean_obliquity;
use vedaksha_ephem_core::sidereal_time::gmst;

use crate::combustion::combustion_orb_deg;
use crate::graha::Graha;
use crate::muhurta::refine_crossing;

// ── shared helpers ──────────────────────────────────────────────

/// Scan windows (documented, not tuned): ingress 400 d, station 800 d,
/// heliacal 600 d, eclipse 8 syzygies per kind. The range forms
/// ([`graha_ingresses`], [`stations`]) have none, and [`next_udaya`] and
/// [`next_asta`] look 1000 d ahead.
const INGRESS_STEPS: u32 = 400;
const STATION_STEPS: u32 = 800;
const HELIACAL_WINDOW_D: f64 = 600.0;
const GRAHANA_SYZYGIES: u32 = 8;

/// Rashi index (0–11) of a sidereal longitude.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rashi_idx(lon_deg: f64) -> u8 {
    (lon_deg.rem_euclid(360.0) / 30.0).floor() as u8 % 12
}

/// (angle, rate) adapter over a position-only callback, rate by central
/// difference (h = 0.002 d ≈ 3 min), branch-cut unwrapped. For the smooth
/// scanning angles here (Sun, Moon, elongation) the truncation error is
/// ~1e-7 °/day — and the rate only steers the refinement, never sets its root.
fn numeric_angle_rate(pos: &(dyn Fn(f64) -> Option<f64> + Sync), t: f64) -> Option<(f64, f64)> {
    let h = 0.002;
    let (a0, a1) = (pos(t - h)?, pos(t + h)?);
    let mut d = a1 - a0;
    if d > 180.0 {
        d -= 360.0;
    }
    if d < -180.0 {
        d += 360.0;
    }
    Some((pos(t)?, d / (2.0 * h)))
}
/// Same rule as `kala_mana`'s scanner: a step wider than 180° is the 0°
/// wrap, a crossing only for target 0°.
fn crossed(e0: f64, e1: f64, target: f64) -> bool {
    if target == 0.0 {
        e1 < e0 && (e0 > 300.0 || e1 < 60.0)
    } else {
        (e1 - e0).abs() < 180.0 && ((e0 < target && e1 >= target) || (e0 >= target && e1 < target))
    }
}

/// Bisection root of `f` (sign-changing) on `[a, b]`, 50 iterations.
fn bisect(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64) -> Option<f64> {
    let mut lo = a;
    let mut hi = b;
    let mut flo = f(lo)?;
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        let fmid = f(mid)?;
        if flo == 0.0 {
            return Some(lo);
        }
        if flo.signum() == fmid.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Bisection to `tol_d` days of a sign-changing `f` on `[a, b]`.
fn bisect_to(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64, tol_d: f64) -> Option<f64> {
    let (mut lo, mut hi) = (a, b);
    let flo = f(lo)?;
    while hi - lo > tol_d {
        let mid = 0.5 * (lo + hi);
        if f(mid)?.signum() == flo.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// The root of `f` on `[a, b]`, `f(a)` and `f(b)` of opposite sign, by the
/// Illinois form of regula falsi: for a smooth, nearly linear `f` (the Sun's
/// altitude across an hour or two) it needs a handful of evaluations where
/// bisection needs thirty. Stops when `|f|` is under `f_tol` or the bracket is
/// under a nanoday, or after 60 passes.
fn illinois(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64, f_tol: f64) -> Option<f64> {
    let (mut a, mut b) = (a, b);
    let (mut fa, mut fb) = (f(a)?, f(b)?);
    let mut side = 0_i8;
    let mut c = b;
    for _ in 0..60 {
        c = (a * fb - b * fa) / (fb - fa);
        let fc = f(c)?;
        if fc.abs() < f_tol || (b - a).abs() < 1.0e-9 {
            return Some(c);
        }
        if fc.signum() == fb.signum() {
            b = c;
            fb = fc;
            if side == -1 {
                fa *= 0.5;
            }
            side = -1;
        } else {
            a = c;
            fa = fc;
            if side == 1 {
                fb *= 0.5;
            }
            side = 1;
        }
    }
    Some(c)
}

/// Minimum of convex `f` on `[a, b]` by ternary search, 60 iterations.
fn ternary_min(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64) -> Option<f64> {
    let mut lo = a;
    let mut hi = b;
    for _ in 0..60 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        let (f1, f2) = (f(m1)?, f(m2)?);
        if f1 > f2 {
            lo = m1;
        } else {
            hi = m2;
        }
    }
    Some(0.5 * (lo + hi))
}

// ── ingress ─────────────────────────────────────────────────────

/// How far an ingress scan may look.
#[derive(Clone, Copy)]
enum IngressWindow {
    /// At most this many one-day steps after the seed (the single-ingress
    /// functions: 400).
    Days(u32),
    /// Until a sample reaches this JD.
    Until(f64),
}

/// One second in days: the nudge that steps a walk past a crossing.
const ONE_SECOND_D: f64 = 1.0 / 86_400.0;

/// Next ingress of a tropical longitude into a sidereal rashi after `jd_start`:
/// `(jd, rashi)`.
///
/// Samples the rashi on a grid of whole days from the seed and refines the edge
/// between the two samples that straddle it. With `speed_bound` `None` every
/// step is one day (no body moves 30°/day). With `Some(v)`, `v` an upper bound
/// on the sidereal longitude's speed in degrees a day, a step is as many whole
/// days as the nearer rashi edge is `v` away, at least one: no crossing can
/// fall inside a step that long, so the one-day step is kept for the last day
/// before an edge only. Both ways sample the same grid and straddle a crossing
/// with the same two samples, so the refinement starts from the same instant and
/// the answer is identical.
///
/// Retrograde crossings (nodes, vakri) refine on the sign-flipped angle and
/// report the rashi entered downward. `rate` from the callback seeds the
/// direction; the crossing instant's own sign rules where a station sits inside
/// the bracket. A bound the callback exceeds is caught: a rashi change after a
/// long step falls back to the one-day step from the sample before it.
///
/// Returns `None` where the callback is unavailable or no ingress falls in
/// `window`.
fn scan_ingress(
    jd_start: f64,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    window: IngressWindow,
    speed_bound: Option<f64>,
) -> Option<(f64, u8)> {
    let sidereal = |t: f64| {
        let (lon, rate) = longitude(t)?;
        Some((
            (lon - true_ayanamsha_value(ayanamsha, t)).rem_euclid(360.0),
            rate,
        ))
    };
    let rashi = |t: f64| sidereal(t).map(|(lon, _)| rashi_idx(lon));
    let mut t0 = jd_start;
    let mut lon0 = sidereal(t0)?.0;
    let mut i0 = rashi_idx(lon0);
    // Step off an exact boundary: a start within a second BEFORE an edge
    // would otherwise report that edge as the "next" ingress.
    if rashi(t0 + ONE_SECOND_D)? != i0 {
        t0 += ONE_SECOND_D;
        lon0 = sidereal(t0)?.0;
        i0 = rashi_idx(lon0);
    }
    let mut t = t0;
    let mut lon = lon0;
    let mut steps = 0_u32;
    loop {
        match window {
            IngressWindow::Days(max) => {
                if steps >= max {
                    return None;
                }
            }
            IngressWindow::Until(end) => {
                if t >= end {
                    return None;
                }
            }
        }
        // Whole days to the earliest a crossing can fall.
        let days = match speed_bound {
            Some(v) => {
                let r = lon.rem_euclid(30.0);
                (r.min(30.0 - r) / v).floor().max(1.0)
            }
            None => 1.0,
        };
        let mut t1 = t + days;
        let mut lon1 = sidereal(t1)?.0;
        if days > 1.0 && rashi_idx(lon1) != i0 {
            // The bound did not hold: look at the day after `t` instead.
            t1 = t + 1.0;
            lon1 = sidereal(t1)?.0;
        }
        steps += 1;
        let i1 = rashi_idx(lon1);
        if i1 != i0 {
            // Entered i1. Direct up-crossing refines edge i1*30; a
            // down-crossing (i1 == i0 - 1 mod 12) refines edge (i1+1)*30
            // on the flipped angle.
            let direct = i1 == (i0 + 1) % 12;
            let edge = if direct {
                f64::from(i1) * 30.0
            } else {
                f64::from((i1 + 1) % 12) * 30.0
            };
            let rate_now = sidereal(t1)?.1;
            let angle_at = |tt: f64| {
                let (lon, rate) = sidereal(tt)?;
                if direct || rate_now >= 0.0 {
                    Some((lon, rate))
                } else {
                    Some(((-lon).rem_euclid(360.0), -rate))
                }
            };
            let target = if direct || rate_now >= 0.0 {
                edge
            } else {
                (-edge).rem_euclid(360.0)
            };
            let jd = refine_crossing(target, t, &angle_at)?;
            return Some((jd, i1));
        }
        t = t1;
        lon = lon1;
    }
}

/// Next ingress of a tropical longitude into a sidereal rashi: `(jd, rashi)`.
///
/// One-day steps over a 400-day window; see [`scan_ingress`].
fn next_ingress(
    jd_start: f64,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    scan_ingress(
        jd_start,
        ayanamsha,
        longitude,
        IngressWindow::Days(INGRESS_STEPS),
        None,
    )
}

/// Next solar ingress (sankranti): `(jd, rashi entered)`, sidereal.
#[must_use]
pub fn sun_ingress(
    jd_start: f64,
    ayanamsha: Ayanamsha,
    sun: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    next_ingress(jd_start, ayanamsha, sun)
}

/// Next ingress of `graha` into a sidereal rashi: `(jd, rashi entered)`.
///
/// `graha` labels which body `longitude` serves (the math needs only the
/// callback); the nodes' retrograde crossings are handled, not rejected.
/// Residual risk, stated: where a station sits inside the 1-day detection
/// bracket, the edge is picked by the crossing instant's rate sign — a
/// station exactly ON the edge refines the wrong side's edge.
///
/// Looks 400 days ahead: a slow graha (shani spends about 900 days in a rashi)
/// can return `None` with its next entry further off. For a range, or for any
/// graha without that limit, use [`graha_ingresses`].
#[must_use]
pub fn graha_ingress(
    jd_start: f64,
    graha: Graha,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    let _ = graha;
    next_ingress(jd_start, ayanamsha, longitude)
}

/// An upper bound, in degrees a day, on how fast the sidereal longitude of
/// `graha` changes over any window of a day or more: what [`graha_ingresses`]
/// takes as the most a graha can travel, and so the longest step that cannot
/// cross a rashi edge. It is for the apparent geocentric longitude and the daily
/// motion the engine reports, with the ayanamsha's drift (at most 0.00015
/// degrees a day) inside the margin.
///
/// | graha | bound | basis |
/// |---|---|---|
/// | Sun | 1.05 | Kepler: `n (1+e)^2 / (1-e^2)^1.5` at perihelion is 1.019; measured 1.020 |
/// | Moon | 16.0 | under 16 at perigee (as `vedaksha_astro::moon_riseset` bounds it); measured 15.39 |
/// | Mercury | 2.4 | two-body supremum 2.288; measured 2.202 |
/// | Venus | 1.35 | two-body supremum 1.275; measured 1.259 |
/// | Mars | 0.85 | two-body supremum 0.795; measured 0.791 |
/// | Jupiter | 0.26 | two-body supremum 0.245; measured 0.242 |
/// | Saturn | 0.14 | two-body supremum 0.134; measured 0.130 |
/// | Rahu, Ketu | 0.30 | the osculating node, measured -0.258 to +0.032; the mean node moves 0.053 |
///
/// The two-body supremum is the largest rate at which the line from the Earth
/// to the planet turns when both move on Kepler ellipses (Earth's `a = 1.0000`,
/// `e = 0.0167`), over every phase of each and every relative orientation of
/// the two orbits. The ellipses' `a` and `e` are the rounded mean values:
/// Mercury 0.3871 and 0.2056, Venus 0.7233 and 0.0068, Mars 1.5237 and 0.0934,
/// Jupiter 5.2034 and 0.0484, Saturn 9.5371 and 0.0542. The "measured" figures
/// are the largest daily motion this crate's analytical positions give from
/// 1900 to 2100, sampled every 0.25 to 2 days. The margin over the larger of
/// the two is 3 to 7 per cent for the planets and Sun, 16 per cent for the
/// nodes, whose motion is not Keplerian. The 1900 to 2100 figures were measured
/// once (2026-10-24); a test holds the bounds over one to ten years per graha
/// from 2010 (`tests/khagola_ingress.rs`).
///
/// A bound that is too small is caught when a long step lands in another
/// rashi (see [`graha_ingresses`]); a bound that is too large only costs
/// reads. The Rahu and Ketu figure covers the osculating node and the mean node.
#[must_use]
pub const fn ingress_speed_bound_deg_per_day(graha: Graha) -> f64 {
    match graha {
        Graha::Sun => 1.05,
        Graha::Moon => 16.0,
        Graha::Mercury => 2.4,
        Graha::Venus => 1.35,
        Graha::Mars => 0.85,
        Graha::Jupiter => 0.26,
        Graha::Saturn => 0.14,
        Graha::Rahu | Graha::Ketu => 0.30,
    }
}

/// Every ingress of `graha` into a sidereal rashi in `[jd_start, jd_end)`, in
/// time order, as `(jd, rashi entered)`: the vakri re-entries and the nodes'
/// downward crossings included, no 400-day limit.
///
/// Each entry is what [`graha_ingress`] returns when re-seeded one second past
/// the previous one (the first is seeded at `jd_start`) and, where that finds
/// none in its 400 days, from the window's end. The two sample the same grid of
/// whole days from the seed, straddle a crossing with the same two samples and
/// refine from the same one, so they agree to the bit (`tests/khagola_ingress.rs`
/// walks nine spans of 6 months to 10 years across every graha and finds no
/// difference);
/// at worst they differ by the refinement's own resolution, about 0.1 ms. The cost
/// differs: where [`graha_ingress`] reads the callback once a day, this
/// steps as many whole days as the nearer rashi edge is away at the graha's
/// speed bound ([`ingress_speed_bound_deg_per_day`]), and reads it daily only
/// within a day's travel of an edge. Measured on this crate's analytical
/// positions from 2001 to 2043 (once, 2026-10-24): the 37 shani
/// entries cost 15,968 reads one day at a time and 1,118 here (7 per cent);
/// the 78 of Jupiter 15,883 and 2,133 (13 per cent); 12 years of Mars (77
/// entries) 4,911 and 1,533 (31 per cent).
///
/// `longitude` is the tropical longitude and daily motion in degrees at a
/// JD (UT), as for [`graha_ingress`]. The walk ends early where the callback
/// returns `None` or the rate vanishes at a crossing. The speed bound is for an
/// apparent geocentric longitude; a callback that moves faster is caught for a
/// single missed jump (a long step that lands in another rashi falls back to
/// one-day steps) but not for a loop that enters and leaves a rashi between two
/// samples.
#[must_use]
pub fn graha_ingresses(
    jd_start: f64,
    jd_end: f64,
    graha: Graha,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Vec<(f64, u8)> {
    let bound = ingress_speed_bound_deg_per_day(graha);
    let mut found = Vec::new();
    let mut seed = jd_start;
    while seed < jd_end {
        let Some((jd, rashi)) = scan_ingress(
            seed,
            ayanamsha,
            longitude,
            IngressWindow::Until(jd_end),
            Some(bound),
        ) else {
            break;
        };
        if jd >= jd_end {
            break;
        }
        found.push((jd, rashi));
        // A second past the crossing, as the one-ingress walk is re-seeded;
        // never back behind the seed, so the loop always advances.
        seed = (jd + ONE_SECOND_D).max(seed + ONE_SECOND_D);
    }
    found
}

// ── station ─────────────────────────────────────────────────────

/// Next station (speed sign change) of a star-planet: the JD of the turning
/// point, located by bisection on the rate bracket. The Sun, Moon and nodes
/// do not station — `None` for them, as for an unavailable callback or an
/// empty 800-day window. For every station in a range see [`stations`].
#[must_use]
pub fn station(
    jd_start: f64,
    graha: Graha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => return None,
        Graha::Mars | Graha::Mercury | Graha::Jupiter | Graha::Venus | Graha::Saturn => {}
    }
    let rate = |t: f64| longitude(t).map(|(_, r)| r);
    let mut t0 = jd_start;
    let mut r0 = rate(t0)?;
    if r0 == 0.0 {
        return Some(t0);
    }
    for _ in 0..STATION_STEPS {
        let t1 = t0 + 1.0;
        let r1 = rate(t1)?;
        if r1 == 0.0 {
            return Some(t1);
        }
        if r0.signum() != r1.signum() {
            let f = |t: f64| rate(t);
            return bisect(&f, t0, t1);
        }
        t0 = t1;
        r0 = r1;
    }
    None
}

/// Resolution of a station found by [`stations`]: 1e-8 day, about a
/// millisecond. [`station`] bisects 50 times, to the spacing of a Julian Day
/// near 2.4 million (4.7e-10 d); the two agree to within this.
const STATION_TOL_D: f64 = 1.0e-8;

/// The step, in days, at which [`stations`] reads a star-planet's rate: under
/// half the shortest interval between two of its stations, so that no step
/// can hold two. `None` for the Sun, Moon and nodes, which do not station.
///
/// The shortest interval between consecutive stations, in days, measured
/// from the daily walk of [`station`] over 1900 to 2100 on this crate's
/// analytical positions (apparent geocentric longitude and its daily rate), and
/// the step, a whole number of days under half of it:
///
/// | graha | stations | shortest interval | step |
/// |---|---|---|---|
/// | Mercury | 1260 | 19.76 | 9 |
/// | Venus | 250 | 40.89 | 20 |
/// | Mars | 188 | 59.93 | 29 |
/// | Jupiter | 366 | 117.47 | 58 |
/// | Saturn | 386 | 133.62 | 66 |
///
/// The shortest interval is in every case a retrograde stretch. The counts
/// agree with two stations per synodic period (from the sidereal periods, 1261,
/// 250, 187, 366 and 386 in the 73,048 days). The step is under half the
/// interval, not the interval, so that a retrograde somewhere outside 1900 to
/// 2100 a little shorter than these still cannot put two stations in one step.
/// The figures above were measured once (2026-10-24); `tests/khagola_station.rs`
/// checks the series against the daily walk over 2020 to 2026.
#[must_use]
pub const fn station_step_days(graha: Graha) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => None,
        Graha::Mercury => Some(9.0),
        Graha::Venus => Some(20.0),
        Graha::Mars => Some(29.0),
        Graha::Jupiter => Some(58.0),
        Graha::Saturn => Some(66.0),
    }
}

/// Every station (speed sign change) of a star-planet in `[jd_from, jd_to)`,
/// in time order. Empty for the Sun, Moon and nodes, which do not station.
///
/// Reads the rate once per [`station_step_days`] and bisects each step whose
/// ends differ in sign; a step shorter than half the shortest gap between two
/// stations holds at most one. Each station is what [`station`] returns when
/// re-seeded just past the previous one, to the bisection's own resolution,
/// and no station is skipped: over 1900 to 2100 and the five star-planets it
/// finds the same 2450 stations as the daily walk of [`station`], to 5e-8 days,
/// with 95,000 reads of the callback against 495,000 (measured once,
/// 2026-10-24).
///
/// A rate that is exactly zero at a sample is a station there. The walk ends
/// early where the callback returns `None`.
#[must_use]
pub fn stations(
    jd_from: f64,
    jd_to: f64,
    graha: Graha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Vec<f64> {
    let mut found = Vec::new();
    let Some(step) = station_step_days(graha) else {
        return found;
    };
    // Half-open: an empty or inverted range holds nothing, not even a zero
    // rate at `jd_from`.
    if jd_from >= jd_to {
        return found;
    }
    let rate = |t: f64| longitude(t).map(|(_, r)| r);
    let mut t0 = jd_from;
    let Some(mut r0) = rate(t0) else {
        return found;
    };
    if r0 == 0.0 {
        found.push(t0);
        // Past the zero, so the next step compares against a signed rate.
        t0 += ONE_SECOND_D;
        let Some(r) = rate(t0) else {
            return found;
        };
        r0 = r;
    }
    while t0 < jd_to {
        let t1 = t0 + step;
        let Some(r1) = rate(t1) else {
            break;
        };
        if r1 == 0.0 {
            if t1 < jd_to {
                found.push(t1);
            }
            t0 = t1 + ONE_SECOND_D;
            let Some(r) = rate(t0) else {
                break;
            };
            r0 = r;
            continue;
        }
        if r0.signum() != r1.signum() {
            let f = |t: f64| rate(t);
            let Some(root) = bisect_to(&f, t0, t1, STATION_TOL_D) else {
                break;
            };
            if root < jd_to {
                found.push(root);
            }
        }
        t0 = t1;
        r0 = r1;
    }
    found
}

// ── heliacal visibility ─────────────────────────────────────────

/// Altitude (degrees) of an equatorial body over the observer.
fn altitude_deg(jd_ut: f64, lat_deg: f64, lon_east_deg: f64, ra_deg: f64, dec_deg: f64) -> f64 {
    let lst = gmst(jd_ut).to_degrees() + lon_east_deg;
    let h = (lst - ra_deg).to_radians();
    let (phi, dec) = (lat_deg.to_radians(), dec_deg.to_radians());
    (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

/// Next sunrise/sunset pair bracketing: returns the event JD or `None`.
fn sun_event(
    jd_start: f64,
    window_d: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    rising: bool,
) -> Option<f64> {
    // 6-hour strides: no sunrise/sunset is skipped outside the poles.
    let mut t = jd_start;
    let alt = |tt: f64| {
        let (ra, dec) = sun_eq(tt)?;
        Some(altitude_deg(tt, lat_deg, lon_east_deg, ra, dec))
    };
    let mut a0 = alt(t)?;
    let end = jd_start + window_d;
    while t < end {
        let t1 = (t + 0.25).min(end);
        let a1 = alt(t1)?;
        let is_event = if rising {
            a0 < 0.0 && a1 >= 0.0
        } else {
            a0 >= 0.0 && a1 < 0.0
        };
        if is_event {
            let f = |tt: f64| alt(tt);
            return bisect(&f, t, t1);
        }
        t = t1;
        a0 = a1;
    }
    None
}

/// First sunrise at which `graha` stands at least `critical_altitude_deg`
/// above the horizon — the geometric content of heliacal rise (udaya).
/// The criterion (extinction, magnitude, season) is the caller's: pass the
/// tradition's arcus visionis, typically ~10°. `None` for the Sun and nodes
/// (no visible body), an unavailable callback, or an empty 600-day window. This
/// is the first qualifying sunrise, not the change from invisibility to
/// visibility; for that, and for the classical kalamsha criterion, see
/// [`next_udaya`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn heliacal_rise(
    jd_start: f64,
    graha: Graha,
    latitude_deg: f64,
    longitude_deg_east: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    critical_altitude_deg: f64,
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Rahu | Graha::Ketu => return None,
        _ => {}
    }
    let mut t = jd_start;
    let end = jd_start + HELIACAL_WINDOW_D;
    while t < end {
        let rise = sun_event(t, end - t, latitude_deg, longitude_deg_east, sun_eq, true)?;
        let (ra, dec) = graha_eq(rise)?;
        if altitude_deg(rise, latitude_deg, longitude_deg_east, ra, dec) >= critical_altitude_deg {
            return Some(rise);
        }
        t = rise + 0.5;
    }
    None
}

/// Last sunset at which `graha` still stands at least
/// `critical_altitude_deg` above the horizon — the geometric content of
/// heliacal set (asta). Window semantics: the LAST qualifying sunset in the
/// 600-day window; a caller hunting an apparition's end past the window
/// recalls with a later `jd_start`. `None` cases as in [`heliacal_rise`]. For
/// the end of the next apparition see [`next_asta`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn heliacal_set(
    jd_start: f64,
    graha: Graha,
    latitude_deg: f64,
    longitude_deg_east: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    critical_altitude_deg: f64,
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Rahu | Graha::Ketu => return None,
        _ => {}
    }
    let mut t = jd_start;
    let end = jd_start + HELIACAL_WINDOW_D;
    let mut last: Option<f64> = None;
    while t < end {
        let Some(set) = sun_event(t, end - t, latitude_deg, longitude_deg_east, sun_eq, false)
        else {
            break;
        };
        let (ra, dec) = graha_eq(set)?;
        if altitude_deg(set, latitude_deg, longitude_deg_east, ra, dec) >= critical_altitude_deg {
            last = Some(set);
        }
        t = set + 0.5;
    }
    last
}

/// The horizon at which a heliacal appearance is looked for.
///
/// Surya Siddhanta IX.2-3: Jupiter, Mars and Saturn, and Venus and Mercury
/// while retrograde, set in the west (evening) and rise in the east (morning);
/// the Moon, and Venus and Mercury while direct, set in the east and rise in the
/// west. So the same planet has its udaya on one side and its asta on the
/// other, and which depends on its motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeliacalSide {
    /// The morning, at sunrise: the graha rises ahead of the Sun in the east.
    East,
    /// The evening, at sunset: the graha sets after the Sun in the west.
    West,
}

/// An observer's place: geodetic latitude and east longitude, degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub latitude_deg: f64,
    pub longitude_deg_east: f64,
}

/// When a graha counts as visible at a sunrise or sunset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeliacalCriterion {
    /// Its altitude over the geometric horizon at the Sun's geometric
    /// sunrise or sunset is at least this many degrees: the caller's arcus
    /// visionis, the criterion of [`heliacal_rise`] and [`heliacal_set`].
    Altitude { critical_altitude_deg: f64 },
    /// The classical rule, Surya Siddhanta IX.5-9 and X.1: its kalamsha (see
    /// [`kalamsha_deg`]) is at least the graha's limit, which is
    /// [`crate::combustion::combustion_orb_deg`] (Jupiter 11, Saturn 15, Mars
    /// 17, Venus 10 and 8 retrograde, Mercury 14 and 12 retrograde, the Moon
    /// 12). Not defined for the Sun and nodes.
    Kalamsha,
}

/// The search window of [`next_udaya`] and [`next_asta`]: 1000 days. A graha
/// that is visible on a side of the Sun for a stretch of every synodic period
/// has its next udaya within a period (Mars's is the longest, 780 days on
/// average and under 810) plus the invisible stretch.
const HELIACAL_EVENT_WINDOW_D: f64 = 1000.0;

/// Kalamsha (time-degrees) of a graha at the Sun's sunrise (`East`) or sunset
/// (`West`) for an observer at `latitude_deg`: how far, in degrees of the
/// equator, the graha rises ahead of the Sun (east) or sets after it (west).
/// Negative when it is on the wrong side. `sun_eq` and `graha_eq` are
/// `(right ascension, declination)` in degrees, at that sunrise or sunset.
///
/// Surya Siddhanta IX.4-5: take the longitudes of the Sun and of the planet at
/// the moment of sunrise (east) or sunset (west), with the planet's
/// latitude put into its longitude (drikkarma, VII.7-11); the oblique
/// ascensions, in respirations, of the two risings (west: of the two settings,
/// each longitude increased by six signs), divided by sixty, are the
/// kalamsha. The oblique ascension of the point of the ecliptic that rises
/// with a body is the sidereal time at which that body rises, so this is the
/// difference between the sidereal times of the planet's rising and the Sun's:
/// a body at `(alpha, delta)` rises at `alpha - H0` and sets at `alpha + H0`,
/// `cos H0 = -tan(phi) tan(delta)`. The result does not depend on longitude. It
/// is the exact spherical form of the text's rule, not its sine-table
/// arithmetic.
///
/// `None` where either body does not rise and set that day (`|tan phi tan
/// delta| >= 1`: beyond about 66 degrees of latitude for the Sun).
#[must_use]
pub fn kalamsha_deg(
    side: HeliacalSide,
    latitude_deg: f64,
    sun_eq: (f64, f64),
    graha_eq: (f64, f64),
) -> Option<f64> {
    let tan_phi = latitude_deg.to_radians().tan();
    let h0 = |dec_deg: f64| {
        let c = -tan_phi * dec_deg.to_radians().tan();
        (c.abs() < 1.0).then(|| c.acos().to_degrees())
    };
    let (h_sun, h_graha) = (h0(sun_eq.1)?, h0(graha_eq.1)?);
    let wrap = |x: f64| (x + 180.0).rem_euclid(360.0) - 180.0;
    Some(match side {
        HeliacalSide::East => wrap((sun_eq.0 - h_sun) - (graha_eq.0 - h_graha)),
        HeliacalSide::West => wrap((graha_eq.0 + h_graha) - (sun_eq.0 + h_sun)),
    })
}

/// Whether the graha's ecliptic longitude is falling at `t`, read from the
/// central difference of the longitude implied by its `(ra, dec)` over a day
/// (mean obliquity of date; the sign does not care about nutation).
fn is_retrograde(graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync), t: f64) -> Option<bool> {
    let lon = |tt: f64| {
        let (ra, dec) = graha_eq(tt)?;
        let eps = mean_obliquity(tt);
        let (ra, dec) = (ra.to_radians(), dec.to_radians());
        Some(
            (ra.sin() * eps.cos() + dec.tan() * eps.sin())
                .atan2(ra.cos())
                .to_degrees(),
        )
    };
    let d = (lon(t + 0.5)? - lon(t - 0.5)? + 180.0).rem_euclid(360.0) - 180.0;
    Some(d < 0.0)
}

/// The Sun's altitude crossing the geometric horizon inside `[a, b]`, upward
/// (`rising`) or downward: the instant, or `None` where it does not cross there
/// (or the callback is unavailable).
fn horizon_crossing(
    a: f64,
    b: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    rising: bool,
) -> Option<f64> {
    let alt = |tt: f64| {
        let (ra, dec) = sun_eq(tt)?;
        Some(altitude_deg(tt, lat_deg, lon_east_deg, ra, dec))
    };
    let (fa, fb) = (alt(a)?, alt(b)?);
    let crosses = if rising {
        fa < 0.0 && fb >= 0.0
    } else {
        fa >= 0.0 && fb < 0.0
    };
    if !crosses {
        return None;
    }
    illinois(&alt, a, b, 1.0e-9)
}

/// The next sunrise (`rising`) or sunset after `prev`, about a day on.
fn following_sun_event(
    prev: f64,
    end: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    rising: bool,
) -> Option<f64> {
    // The next event is a day (+-2.4 h) after the last, outside the polar
    // regions; if the Sun does not cross there, search with the general scan.
    if let Some(t) = horizon_crossing(
        prev + 0.9,
        prev + 1.1,
        lat_deg,
        lon_east_deg,
        sun_eq,
        rising,
    ) {
        return Some(t);
    }
    let from = prev + 0.5;
    sun_event(from, end - from, lat_deg, lon_east_deg, sun_eq, rising)
}

/// Whether `graha` is visible at the sunrise or sunset `t` under `criterion`.
#[allow(clippy::too_many_arguments)]
fn visible_at(
    t: f64,
    graha: Graha,
    side: HeliacalSide,
    place: Place,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    criterion: HeliacalCriterion,
) -> Option<bool> {
    let g = graha_eq(t)?;
    Some(match criterion {
        HeliacalCriterion::Altitude {
            critical_altitude_deg,
        } => {
            altitude_deg(t, place.latitude_deg, place.longitude_deg_east, g.0, g.1)
                >= critical_altitude_deg
        }
        HeliacalCriterion::Kalamsha => {
            let retrograde = match graha {
                Graha::Mercury | Graha::Venus => is_retrograde(graha_eq, t)?,
                _ => false,
            };
            let limit = combustion_orb_deg(graha, retrograde)?;
            kalamsha_deg(side, place.latitude_deg, sun_eq(t)?, g)? >= limit
        }
    })
}

/// Which of the two heliacal events to look for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HeliacalEvent {
    Udaya,
    Asta,
}

/// Shared walk of [`next_udaya`] and [`next_asta`] over successive sunrises or
/// sunsets.
#[allow(clippy::too_many_arguments)]
fn heliacal_event(
    event: HeliacalEvent,
    jd_start: f64,
    graha: Graha,
    side: HeliacalSide,
    place: Place,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    criterion: HeliacalCriterion,
) -> Option<f64> {
    if matches!(graha, Graha::Sun | Graha::Rahu | Graha::Ketu) {
        return None;
    }
    let (lat, lon) = (place.latitude_deg, place.longitude_deg_east);
    let rising = side == HeliacalSide::East;
    let end = jd_start + HELIACAL_EVENT_WINDOW_D;
    let visible = |t: f64| visible_at(t, graha, side, place, sun_eq, graha_eq, criterion);
    let mut t = sun_event(jd_start, end - jd_start, lat, lon, sun_eq, rising)?;
    // The morning or evening before `jd_start`, which an udaya at the first
    // one needs to be a change.
    let mut was: Option<(f64, bool)> = None;
    if event == HeliacalEvent::Udaya {
        if let Some(before) = horizon_crossing(t - 1.1, t - 0.9, lat, lon, sun_eq, rising) {
            was = Some((before, visible(before)?));
        }
    }
    loop {
        let now = visible(t)?;
        match (event, was) {
            (HeliacalEvent::Udaya, Some((_, false))) if now => return Some(t),
            (HeliacalEvent::Asta, Some((last, true))) if !now => return Some(last),
            _ => {}
        }
        was = Some((t, now));
        t = following_sun_event(t, end, lat, lon, sun_eq, rising)?;
        if t >= end {
            return None;
        }
    }
}

/// The next udaya (heliacal rising) of `graha` on `side` after `jd_start`: the
/// sunrise (`East`) or sunset (`West`) of the first day it is visible after a
/// day it was not.
///
/// Within one apparition the answer is the same from every instant of the
/// invisibility before it, including the hours between the last invisible
/// day's sunrise and the first visible one's; from inside an apparition it is
/// the NEXT apparition's. The day before `jd_start` is read to know whether
/// the first one after it is a change. `sun_eq` and `graha_eq` return `(right
/// ascension, declination)` in degrees at a JD (UT); the criterion is the
/// caller's altitude or the classical kalamsha ([`HeliacalCriterion`]).
///
/// Which side an udaya falls on is Surya Siddhanta IX.2-3 (see
/// [`HeliacalSide`]): for Mars, Jupiter and Saturn it is `East`; for Venus and
/// Mercury it is `East` after inferior conjunction (retrograde) and `West`
/// after superior conjunction (direct); for the Moon it is `West`. A side on
/// which it never happens returns `None` after the 1000-day window.
///
/// `None` for the Sun and nodes, an unavailable callback, or no change in the
/// window. The instant is the sunrise or sunset to within a millisecond, not the
/// moment the criterion is crossed, which falls between two days.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn next_udaya(
    jd_start: f64,
    graha: Graha,
    side: HeliacalSide,
    place: Place,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    criterion: HeliacalCriterion,
) -> Option<f64> {
    heliacal_event(
        HeliacalEvent::Udaya,
        jd_start,
        graha,
        side,
        place,
        sun_eq,
        graha_eq,
        criterion,
    )
}

/// The next asta (heliacal setting) of `graha` on `side` at or after
/// `jd_start`: the sunrise (`East`) or sunset (`West`) of the last day it is
/// visible before a day it is not.
///
/// From inside an apparition this is that apparition's last day, never a later
/// one's; from an invisibility, the end of the next apparition. The returned
/// day is at or after `jd_start`. Otherwise as [`next_udaya`]: the side on
/// which an asta falls is Surya Siddhanta IX.2-3 (`West` for Mars, Jupiter,
/// Saturn and for Venus and Mercury before inferior conjunction, `East` for the
/// Moon and for Venus and Mercury before superior conjunction); the first
/// day it is NOT visible follows the returned one by a day.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn next_asta(
    jd_start: f64,
    graha: Graha,
    side: HeliacalSide,
    place: Place,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    criterion: HeliacalCriterion,
) -> Option<f64> {
    heliacal_event(
        HeliacalEvent::Asta,
        jd_start,
        graha,
        side,
        place,
        sun_eq,
        graha_eq,
        criterion,
    )
}

// ── eclipse ─────────────────────────────────────────────────────

/// Geocentric ecliptic geometry for the lunar computation.
#[derive(Debug, Clone, Copy)]
pub struct LunarGeometry {
    pub sun_lon_deg: f64,
    pub sun_dist_au: f64,
    pub moon_lon_deg: f64,
    pub moon_lat_deg: f64,
    pub moon_dist_au: f64,
}

/// Geocentric equatorial geometry for the solar computation.
#[derive(Debug, Clone, Copy)]
pub struct SolarGeometry {
    pub sun_ra_deg: f64,
    pub sun_dec_deg: f64,
    pub sun_dist_au: f64,
    pub moon_ra_deg: f64,
    pub moon_dec_deg: f64,
    pub moon_dist_au: f64,
}

/// Eclipse kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrahanaKind {
    LunarPenumbral,
    LunarPartial,
    LunarTotal,
    SolarPartial,
    SolarTotal,
    SolarAnnular,
}

/// An eclipse: the sparsha/madhya/moksha triple with kind and magnitude.
///
/// Lunar contacts are penumbral C1/greatest/C4 (umbral U1–U4 are NOT
/// separately returned); `magnitude` is the umbral magnitude (lunar: negative
/// for a penumbral eclipse, > 1 for total) or the fraction of the Sun's
/// diameter covered at greatest eclipse (solar; the formula
/// `(s_sun + s_moon - separation) / (2 s_sun)`, so > 1 for total).
///
/// For a solar eclipse `sparsha_jd` and `moksha_jd` are the VISIBLE contacts,
/// clipped to sunrise and sunset; `horizon` says whether they were, and holds
/// the unclipped ones. `madhya_jd` is always the instant of least separation
/// at the place, even when the Sun is below the horizon then.
#[derive(Debug, Clone, Copy)]
pub struct Grahana {
    pub kind: GrahanaKind,
    pub sparsha_jd: f64,
    pub madhya_jd: f64,
    pub moksha_jd: f64,
    pub magnitude: f64,
    /// Lunar only: the penumbral magnitude, the fraction of the Moon's
    /// diameter inside the penumbra at greatest eclipse
    /// `(R_penumbra + s_moon - gamma) / (2 s_moon)`. Positive for every
    /// lunar eclipse this module reports. `None` for a solar eclipse.
    pub penumbral_magnitude: Option<f64>,
    /// Solar only: how the horizon bounds what the observer sees. `None` for
    /// a lunar eclipse, which is geocentric and not tested against a horizon.
    pub horizon: Option<SolarHorizon>,
}

/// How the observer's horizon bounds a solar eclipse: what [`grahana`] clipped
/// to report the visible part, and what a caller needs to say "drishya".
///
/// "Above the horizon" is the Sun's topocentric centre at or above
/// [`vedaksha_astro::riseset::SUN_STANDARD_ALTITUDE_DEG`] (-50 arcminutes), the
/// standard sunrise and sunset altitude (Meeus Ch. 15). If the Sun sets and
/// rises again inside one eclipse (a midnight-sun dip at high latitude) the
/// visible span runs from the first to the last visible instant, sampled at
/// 15 minute steps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarHorizon {
    /// First contact (JD UT) at the place, whether or not the Sun was up.
    pub geometric_sparsha_jd: f64,
    /// Last contact (JD UT) at the place, whether or not the Sun was up.
    pub geometric_moksha_jd: f64,
    /// The eclipse is already in progress when the Sun rises: the reported
    /// `sparsha_jd` is the moment of sunrise, not first contact.
    pub sparsha_at_sunrise: bool,
    /// The Sun sets before the eclipse ends: the reported `moksha_jd` is the
    /// moment of sunset, not last contact.
    pub moksha_at_sunset: bool,
    /// The Sun is above the horizon at `madhya_jd`, so the eclipse is seen at
    /// its greatest.
    pub madhya_visible: bool,
    /// The largest magnitude reached while the Sun is above the horizon, same
    /// definition as [`Grahana::magnitude`]. Equals it when `madhya_visible`.
    pub visible_magnitude: f64,
}

/// How Earth's shadow is enlarged for its atmosphere in a lunar eclipse.
///
/// The Moon's equatorial horizontal parallax `Pm`, the Sun's geocentric
/// semidiameter `Ss` and its parallax `Ps` give the radii of the penumbra and
/// umbra at the Moon's distance. Source: Espenak, F., NASA/GSFC Eclipse Web
/// Site, "Enlargement of Earth's Shadows" (the Five Millennium Canon of Lunar
/// Eclipses), which gives both conventions.
///
/// | convention | penumbral radius | umbral radius |
/// |---|---|---|
/// | Danjon (eqs. 1-5, 1-6) | `1.01 Pm + Ss + Ps` | `1.01 Pm - Ss + Ps` |
/// | Chauvenet (eqs. 1-3, 1-4) | `1.02 (0.998340 Pm + Ss + Ps)` | `1.02 (0.998340 Pm - Ss + Ps)` |
///
/// Danjon raises the radius of the Earth by the height of an opaque layer of
/// air (about 1/85, with `1.01` also absorbing the oblateness at latitude 45
/// degrees), so both shadows grow by the same absolute amount; Chauvenet
/// scales both by 1/50 and takes the oblateness through `0.998340`, the
/// Astronomical Almanac's factor. The two differ by about 0.026 in penumbral
/// magnitude and 0.005 in umbral magnitude.
///
/// [`Danjon`](Self::Danjon) is the default: NASA/GSFC's published catalogue,
/// which is the check on this module, is computed with it, as is the French
/// almanac since 1951.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowEnlargement {
    /// `Rp = 1.01 Pm + Ss + Ps`, `Ru = 1.01 Pm - Ss + Ps`.
    #[default]
    Danjon,
    /// `Rp = 1.02 (0.998340 Pm + Ss + Ps)`, `Ru = 1.02 (0.998340 Pm - Ss + Ps)`.
    Chauvenet,
}

impl ShadowEnlargement {
    /// `(penumbral radius, umbral radius)` in degrees at the Moon's distance.
    fn radii_deg(self, pi_m: f64, pi_s: f64, s_s: f64) -> (f64, f64) {
        match self {
            Self::Danjon => (1.01 * pi_m + s_s + pi_s, 1.01 * pi_m - s_s + pi_s),
            Self::Chauvenet => (
                1.02 * (0.998_340 * pi_m + s_s + pi_s),
                1.02 * (0.998_340 * pi_m - s_s + pi_s),
            ),
        }
    }
}

const AU_KM: f64 = 149_597_870.7;
const R_EARTH_KM: f64 = 6378.137;
const FLATTENING: f64 = 1.0 / 298.257_223;
const R_SUN_KM: f64 = 695_700.0;
const R_MOON_KM: f64 = 1737.4;

/// Angular semidiameter (degrees) of `radius_km` at `dist_au`.
fn semidiameter_deg(radius_km: f64, dist_au: f64) -> f64 {
    (radius_km / (dist_au * AU_KM)).asin().to_degrees()
}

/// Horizontal parallax (degrees) at `dist_au`.
fn parallax_deg(dist_au: f64) -> f64 {
    (R_EARTH_KM / (dist_au * AU_KM)).asin().to_degrees()
}

/// Angular separation of two ecliptic points (degrees).
fn ecl_sep(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let (l1, b1, l2, b2) = (
        lon1.to_radians(),
        lat1.to_radians(),
        lon2.to_radians(),
        lat2.to_radians(),
    );
    (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos())
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// Topocentric RA/dec (degrees) from geocentric, via the oblate-Earth
/// vector (Meeus Ch. 40, exact form): station from geodetic latitude,
/// `theta` = mean LST in radians, distance in AU.
fn topocentric_ra_dec(
    ra_deg: f64,
    dec_deg: f64,
    dist_au: f64,
    lat_deg: f64,
    theta_rad: f64,
) -> (f64, f64) {
    let e2 = 2.0 * FLATTENING - FLATTENING * FLATTENING;
    let (phi_sin, phi_cos) = lat_deg.to_radians().sin_cos();
    let big_n = R_EARTH_KM / (1.0 - e2 * phi_sin * phi_sin).sqrt();
    let st = (
        (big_n * phi_cos) * theta_rad.cos(),
        (big_n * phi_cos) * theta_rad.sin(),
        (big_n * (1.0 - e2) * phi_sin),
    );
    let (ra, dec) = (ra_deg.to_radians(), dec_deg.to_radians());
    let d_km = dist_au * AU_KM;
    let geo = (
        d_km * dec.cos() * ra.cos(),
        d_km * dec.cos() * ra.sin(),
        d_km * dec.sin(),
    );
    let topo = (geo.0 - st.0, geo.1 - st.1, geo.2 - st.2);
    let ra_t = topo.1.atan2(topo.0).to_degrees().rem_euclid(360.0);
    let dec_t = (topo.2 / (topo.0.hypot(topo.1).hypot(topo.2)))
        .asin()
        .to_degrees();
    (ra_t, dec_t)
}

/// Topocentric Sun–Moon separation (degrees).
fn topo_sep(g: &SolarGeometry, jd_ut: f64, lat_deg: f64, lon_east_deg: f64) -> f64 {
    let theta = (gmst(jd_ut).to_degrees() + lon_east_deg).to_radians();
    let (sun_ra, sun_dec) =
        topocentric_ra_dec(g.sun_ra_deg, g.sun_dec_deg, g.sun_dist_au, lat_deg, theta);
    let (moon_ra, moon_dec) = topocentric_ra_dec(
        g.moon_ra_deg,
        g.moon_dec_deg,
        g.moon_dist_au,
        lat_deg,
        theta,
    );
    ecl_sep(sun_ra, sun_dec, moon_ra, moon_dec)
}

/// Next lunar eclipse at/after `jd_start` (geocentric shadow cone): the Moon's
/// limb reaches the penumbra, `gamma < R_penumbra + s_moon`, with the shadow
/// radii of `shadow`.
fn next_lunar(
    jd_start: f64,
    shadow: ShadowEnlargement,
    lunar: &(dyn Fn(f64) -> Option<LunarGeometry> + Sync),
) -> Option<Grahana> {
    let elong = |t: f64| {
        let g = lunar(t)?;
        Some((g.moon_lon_deg - g.sun_lon_deg).rem_euclid(360.0))
    };
    let gamma = |t: f64| {
        let g = lunar(t)?;
        Some(ecl_sep(
            g.moon_lon_deg,
            g.moon_lat_deg,
            (g.sun_lon_deg + 180.0).rem_euclid(360.0),
            0.0,
        ))
    };
    let mut t = jd_start;
    for _ in 0..GRAHANA_SYZYGIES {
        // Next full moon: first 180-crossing after t.
        let mut t0 = t;
        let mut e0 = elong(t0)?;
        let mut full: Option<f64> = None;
        for _ in 0..40 {
            let t1 = t0 + 1.0;
            let e1 = elong(t1)?;
            if crossed(e0, e1, 180.0) {
                let pos = |tt: f64| elong(tt);
                let f = |tt: f64| numeric_angle_rate(&pos, tt);
                full = refine_crossing(180.0, t0, &f);
                break;
            }
            t0 = t1;
            e0 = e1;
        }
        let fm = full?;
        let gmin_t = ternary_min(&gamma, fm - 2.0, fm + 2.0)?;
        let gmin = gamma(gmin_t)?;
        let g = lunar(gmin_t)?;
        let pi_m = parallax_deg(g.moon_dist_au);
        let pi_s = parallax_deg(g.sun_dist_au);
        let s_s = semidiameter_deg(R_SUN_KM, g.sun_dist_au);
        let s_m = semidiameter_deg(R_MOON_KM, g.moon_dist_au);
        let (penumbra, umbra) = shadow.radii_deg(pi_m, pi_s, s_s);
        if gmin < penumbra + s_m {
            let c1 = {
                let f = |tt: f64| gamma(tt).map(|x| x - (penumbra + s_m));
                bisect(&f, fm - 2.0, gmin_t)?
            };
            let c4 = {
                let f = |tt: f64| gamma(tt).map(|x| x - (penumbra + s_m));
                bisect(&f, gmin_t, fm + 2.0)?
            };
            let kind = if gmin < umbra - s_m {
                GrahanaKind::LunarTotal
            } else if gmin < umbra + s_m {
                GrahanaKind::LunarPartial
            } else {
                GrahanaKind::LunarPenumbral
            };
            return Some(Grahana {
                kind,
                sparsha_jd: c1,
                madhya_jd: gmin_t,
                moksha_jd: c4,
                magnitude: (umbra + s_m - gmin) / (2.0 * s_m),
                penumbral_magnitude: Some((penumbra + s_m - gmin) / (2.0 * s_m)),
                horizon: None,
            });
        }
        t = fm + 15.0;
    }
    None
}

/// How far the Sun's centre stands above the standard rise/set altitude
/// ([`SUN_STANDARD_ALTITUDE_DEG`]) for the observer, degrees: non-negative while
/// the Sun counts as above the horizon. The Sun's position is topocentric
/// (parallax of 8.8" at most), the altitude geometric; refraction and
/// semidiameter are inside the standard altitude.
fn sun_horizon_margin(g: &SolarGeometry, jd_ut: f64, lat_deg: f64, lon_east_deg: f64) -> f64 {
    let theta = (gmst(jd_ut).to_degrees() + lon_east_deg).to_radians();
    let (ra, dec) = topocentric_ra_dec(g.sun_ra_deg, g.sun_dec_deg, g.sun_dist_au, lat_deg, theta);
    altitude_deg(jd_ut, lat_deg, lon_east_deg, ra, dec) - SUN_STANDARD_ALTITUDE_DEG
}

/// Spacing of the horizon samples across a solar eclipse: 15 minutes. The Sun's
/// altitude changes at most 15 degrees an hour, so a dip below the horizon
/// shorter than this is not seen (only possible near the poles).
const HORIZON_SAMPLE_D: f64 = 15.0 / 1440.0;

/// The first and last instants in `[a, b]` at which `margin >= 0`, found on a
/// grid of at most [`HORIZON_SAMPLE_D`] and refined to a tenth of a second:
/// `(any_up, start, end)`. `any_up` is false (and `start`, `end` meaningless)
/// where no grid instant is up. `None` where the callback is unavailable.
fn up_span(
    margin: &(dyn Fn(f64) -> Option<f64> + Sync),
    a: f64,
    b: f64,
) -> Option<(bool, f64, f64)> {
    const TOL_D: f64 = 1.0e-6;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (((b - a) / HORIZON_SAMPLE_D).ceil() as usize).max(1);
    #[allow(clippy::cast_precision_loss)]
    let at = |i: usize| a + (b - a) * (i as f64) / (n as f64);
    let mut up = Vec::with_capacity(n + 1);
    for i in 0..=n {
        up.push(margin(at(i))? >= 0.0);
    }
    let Some(first) = up.iter().position(|&u| u) else {
        return Some((false, a, a));
    };
    let last = up.iter().rposition(|&u| u)?;
    let start = if first == 0 {
        a
    } else {
        bisect_to(margin, at(first - 1), at(first), TOL_D)?
    };
    let end = if last == n {
        b
    } else {
        bisect_to(margin, at(last), at(last + 1), TOL_D)?
    };
    Some((true, start, end))
}

/// Next solar eclipse at/after `jd_start`, topocentric at the observer, seen
/// with the Sun above the horizon for some part of it.
///
/// A syzygy whose eclipse falls wholly below the horizon is skipped (it counts
/// toward the [`GRAHANA_SYZYGIES`] searched), so the answer is the next
/// eclipse the observer could see.
fn next_solar(
    jd_start: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    solar: &(dyn Fn(f64) -> Option<SolarGeometry> + Sync),
) -> Option<Grahana> {
    // Bracket by RA conjunction (ΔRA wraps +180 → −180 at new moon).
    let dra = |t: f64| {
        let g = solar(t)?;
        Some((g.moon_ra_deg - g.sun_ra_deg).rem_euclid(360.0))
    };
    let sep = |t: f64| {
        let g = solar(t)?;
        Some(topo_sep(&g, t, lat_deg, lon_east_deg))
    };
    let margin = |t: f64| {
        let g = solar(t)?;
        Some(sun_horizon_margin(&g, t, lat_deg, lon_east_deg))
    };
    // Magnitude at an instant: the fraction of the Sun's diameter covered.
    let magnitude_at = |t: f64| {
        let g = solar(t)?;
        let s_s = semidiameter_deg(R_SUN_KM, g.sun_dist_au);
        let s_m = semidiameter_deg(R_MOON_KM, g.moon_dist_au);
        Some((s_s + s_m - topo_sep(&g, t, lat_deg, lon_east_deg)) / (2.0 * s_s))
    };
    let mut t = jd_start;
    for _ in 0..GRAHANA_SYZYGIES {
        let mut t0 = t;
        let mut e0 = dra(t0)?;
        let mut conj: Option<f64> = None;
        for _ in 0..40 {
            let t1 = t0 + 1.0;
            let e1 = dra(t1)?;
            if crossed(e0, e1, 0.0) {
                let pos = |tt: f64| dra(tt);
                let f = |tt: f64| numeric_angle_rate(&pos, tt);
                conj = refine_crossing(0.0, t0, &f);
                break;
            }
            t0 = t1;
            e0 = e1;
        }
        let nm = conj?;
        let smin_t = ternary_min(&sep, nm - 1.5, nm + 1.5)?;
        let smin = sep(smin_t)?;
        let g = solar(smin_t)?;
        let s_s = semidiameter_deg(R_SUN_KM, g.sun_dist_au);
        let s_m = semidiameter_deg(R_MOON_KM, g.moon_dist_au);
        if smin < s_s + s_m {
            let c1 = {
                let f = |tt: f64| sep(tt).map(|x| x - (s_s + s_m));
                bisect(&f, nm - 1.5, smin_t)?
            };
            let c4 = {
                let f = |tt: f64| sep(tt).map(|x| x - (s_s + s_m));
                bisect(&f, smin_t, nm + 1.5)?
            };
            let (any_up, vis_start, vis_end) = up_span(&margin, c1, c4)?;
            if !any_up {
                // The eclipse happens at night here: not an eclipse to see.
                t = nm + 15.0;
                continue;
            }
            let kind = if smin + s_s <= s_m {
                GrahanaKind::SolarTotal
            } else if smin + s_m <= s_s {
                GrahanaKind::SolarAnnular
            } else {
                GrahanaKind::SolarPartial
            };
            let magnitude = (s_s + s_m - smin) / (2.0 * s_s);
            let madhya_visible = margin(smin_t)? >= 0.0;
            let visible_magnitude = if madhya_visible {
                magnitude
            } else {
                magnitude_at(vis_start)?.max(magnitude_at(vis_end)?)
            };
            return Some(Grahana {
                kind,
                sparsha_jd: vis_start,
                madhya_jd: smin_t,
                moksha_jd: vis_end,
                magnitude,
                penumbral_magnitude: None,
                horizon: Some(SolarHorizon {
                    geometric_sparsha_jd: c1,
                    geometric_moksha_jd: c4,
                    sparsha_at_sunrise: vis_start > c1,
                    moksha_at_sunset: vis_end < c4,
                    madhya_visible,
                    visible_magnitude,
                }),
            });
        }
        t = nm + 15.0;
    }
    None
}

/// Next eclipse (grahana) at/after `jd_start`: the sparsha/madhya/moksha
/// triple, solar or lunar, whichever comes first. Lunar shadow radii use
/// [`ShadowEnlargement::Danjon`]; [`grahana_with`] takes another convention.
///
/// Lunar is geocentric (shadow cone at the Moon — no observer needed, and no
/// horizon test); solar is topocentric at (`latitude_deg`,
/// `longitude_deg_east`) and reported only if the Sun is above the observer's
/// horizon for some part of it, with sparsha and moksha clipped to sunrise and
/// sunset (see the module note and [`SolarHorizon`]). Either callback
/// returning `None` throughout skips that kind. Returns `None` where neither
/// kind occurs in the next 8 syzygies each; a solar syzygy whose eclipse is
/// wholly below the horizon counts among the 8.
///
/// A lunar eclipse is reported when the Moon's limb reaches the penumbra
/// (penumbral magnitude above zero), with no larger floor.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn grahana(
    jd_start: f64,
    latitude_deg: f64,
    longitude_deg_east: f64,
    lunar: &(dyn Fn(f64) -> Option<LunarGeometry> + Sync),
    solar: &(dyn Fn(f64) -> Option<SolarGeometry> + Sync),
) -> Option<Grahana> {
    grahana_with(
        jd_start,
        latitude_deg,
        longitude_deg_east,
        ShadowEnlargement::default(),
        lunar,
        solar,
    )
}

/// [`grahana`] with the lunar shadow enlargement chosen: the same search, the
/// penumbral and umbral radii of `shadow`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn grahana_with(
    jd_start: f64,
    latitude_deg: f64,
    longitude_deg_east: f64,
    shadow: ShadowEnlargement,
    lunar: &(dyn Fn(f64) -> Option<LunarGeometry> + Sync),
    solar: &(dyn Fn(f64) -> Option<SolarGeometry> + Sync),
) -> Option<Grahana> {
    match (
        next_lunar(jd_start, shadow, lunar),
        next_solar(jd_start, latitude_deg, longitude_deg_east, solar),
    ) {
        (Some(l), Some(s)) => Some(if l.sparsha_jd <= s.sparsha_jd { l } else { s }),
        (Some(l), None) => Some(l),
        (None, Some(s)) => Some(s),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: f64 = 2_460_000.0;

    #[test]
    fn sun_ingress_finds_the_next_edge() {
        // Tropical sun 100° + 1°/day; sidereal ≈ 75.8 + 1°/day -> edge 90
        // (Karka, rashi 3) ~14.2 d on.
        let f = |t: f64| Some((100.0 + (t - T0), 1.0));
        let (jd, rashi) = sun_ingress(T0, Ayanamsha::IndianOfficial, &f).unwrap();
        assert_eq!(rashi, 3);
        assert!((jd - (T0 + 14.2)).abs() < 0.1, "jd = {jd}");
    }

    #[test]
    fn retrograde_ingress_reports_the_rashi_entered_downward() {
        // Nodes: tropical 100° falling 0.05°/day; sidereal ≈ 75.8 falling:
        // edge 60 downward into rashi 1, ~316 d on (inside the 400 d window).
        let f = |t: f64| Some((100.0 - 0.05 * (t - T0), -0.05));
        let (jd, rashi) = graha_ingress(T0, Graha::Rahu, Ayanamsha::IndianOfficial, &f).unwrap();
        assert_eq!(rashi, 1);
        assert!((jd - (T0 + 316.0)).abs() < 5.0, "jd = {jd}");
    }

    #[test]
    fn station_bisects_the_sign_change() {
        // lon = 100 + 5 sin(2π(t−T0)/100): rate zero at T0+25.
        let f = |t: f64| {
            let w = core::f64::consts::TAU * (t - T0) / 100.0;
            Some((
                100.0 + 5.0 * w.sin(),
                5.0 * core::f64::consts::TAU / 100.0 * w.cos(),
            ))
        };
        let st = station(T0, Graha::Mars, &f).unwrap();
        assert!((st - (T0 + 25.0)).abs() < 0.01, "station = {st}");
        // Luminaries and nodes never station.
        assert_eq!(station(T0, Graha::Sun, &f), None);
        assert_eq!(station(T0, Graha::Moon, &f), None);
        assert_eq!(station(T0, Graha::Rahu, &f), None);
    }

    #[test]
    fn constant_motion_never_stations() {
        let f = |t: f64| Some((100.0 + (t - T0), 1.0));
        assert_eq!(station(T0, Graha::Mars, &f), None);
    }

    #[test]
    fn heliacal_rise_finds_first_qualifying_sunrise() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // H(T0) = 280° increasing 360°/day: the morning sun is already up
        // (alt +10°), so the first sunrise forward is the NEXT one, H=630°,
        // +0.9724 d on. Graha 30° east of the sun in RA stands at alt +30°
        // there — qualifies.
        let lst0 = gmst(T0).to_degrees();
        let ra0 = lst0 - 280.0;
        let sun = move |t: f64| Some((ra0 + (t - T0), 0.0));
        let graha = move |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra - 30.0, 0.0))
        };
        let rise = heliacal_rise(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0).unwrap();
        assert!((rise - T0 - 0.9724).abs() < 0.02, "rise = {rise}");
    }

    #[test]
    fn heliacal_set_returns_last_qualifying_sunset_in_window() {
        // Graha 30° west of the sun: alt +30° at every sunset, −30° at
        // every sunrise. Every sunset in the 600-day window qualifies, so
        // the last sits near the window's end (window semantics, module doc).
        let sun = |t: f64| Some((280.0 + 1.0 * (t - T0), 0.0));
        let graha = |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra + 30.0, 0.0))
        };
        let set = heliacal_set(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0).unwrap();
        assert!((set - T0 - 599.5).abs() < 2.0, "set = {set}");
    }

    #[test]
    fn conjunct_graha_never_rises_heliacally() {
        // Graha glued to the sun: alt ~0 at every sunrise < 10° -> None.
        // (Full 600-day scan; stubs are cheap.)
        let sun = |t: f64| Some((280.0 + 1.0 * (t - T0), 0.0));
        let graha = |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra, 0.0))
        };
        assert_eq!(
            heliacal_rise(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0),
            None
        );
        assert_eq!(
            heliacal_rise(T0, Graha::Sun, 0.0, 0.0, &sun, &graha, 10.0),
            None
        );
    }

    #[test]
    fn lunar_total_eclipse_triple_ordered() {
        // New moon T0−10, full T0+5, moon lat 0.3°: total (gamma 0.3 <
        // umbra − moon radius ≈ 0.44).
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 0.3,
                moon_dist_au: 0.00257,
            })
        };
        let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
        let g = grahana(T0, 0.0, 0.0, &lunar, &no_solar).unwrap();
        assert_eq!(g.kind, GrahanaKind::LunarTotal);
        assert!(g.sparsha_jd < g.madhya_jd && g.madhya_jd < g.moksha_jd);
        assert!(g.magnitude > 1.0);
        assert!((g.madhya_jd - (T0 + 5.0)).abs() < 0.5);
    }

    #[test]
    fn high_latitude_full_moon_is_no_eclipse() {
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 5.0,
                moon_dist_au: 0.00257,
            })
        };
        let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
        assert!(grahana(T0, 0.0, 0.0, &lunar, &no_solar).is_none());
    }

    #[test]
    fn solar_partial_eclipse_at_subsolar_observer() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // Conjunction at T0 (RA), moon dec 0.1°: central sep 0.1° < 0.53°
        // but > |s_m − s_s| ≈ 0.007° -> partial. Observer subsolar at T0
        // so parallax ~0 and the geometry is exact.
        let lon_east = (-gmst(T0).to_degrees()).rem_euclid(360.0);
        let solar = move |t: f64| {
            Some(SolarGeometry {
                sun_ra_deg: 0.0,
                sun_dec_deg: 0.0,
                sun_dist_au: 1.0,
                moon_ra_deg: 13.0 * (t - T0),
                moon_dec_deg: 0.1,
                moon_dist_au: 0.00257,
            })
        };
        let no_lunar: fn(f64) -> Option<LunarGeometry> = |_| None;
        let g = grahana(T0 - 1.0, 0.0, lon_east, &no_lunar, &solar).unwrap();
        assert_eq!(g.kind, GrahanaKind::SolarPartial);
        assert!(g.sparsha_jd < g.madhya_jd && g.madhya_jd < g.moksha_jd);
        assert!((g.madhya_jd - T0).abs() < 0.1, "madhya = {}", g.madhya_jd);
    }

    #[test]
    fn earliest_sparsha_wins_between_kinds() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // Solar conjunction T0, lunar full moon T0+5 (total): solar first.
        // Subsolar observer again, so parallax cannot hide the solar event.
        let lon_east = (-gmst(T0).to_degrees()).rem_euclid(360.0);
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 0.3,
                moon_dist_au: 0.00257,
            })
        };
        let solar = |t: f64| {
            Some(SolarGeometry {
                sun_ra_deg: 0.0,
                sun_dec_deg: 0.0,
                sun_dist_au: 1.0,
                moon_ra_deg: 13.0 * (t - T0),
                moon_dec_deg: 0.1,
                moon_dist_au: 0.00257,
            })
        };
        let g = grahana(T0 - 1.0, 0.0, lon_east, &lunar, &solar).unwrap();
        assert!(g.sparsha_jd < T0 + 4.0, "sparsha = {}", g.sparsha_jd);
    }

    #[test]
    fn illinois_finds_a_nonlinear_root_in_a_few_reads() {
        use core::sync::atomic::{AtomicU32, Ordering};
        // x^3 - 2 on [1, 2] (root 1.259 921 049 894 873) and exp(10 x) - 2 on
        // [0, 1] (root ln(2) / 10), both convex and rising, and each reflected
        // in x so that it is convex and falling. Plain regula falsi leaves one
        // end of the bracket where it is for dozens of passes on a convex
        // function, and which end depends on the direction, so every one of the
        // four is run; Illinois needs about 20 reads to a 1e-12 residual where
        // plain regula falsi needs over 30.
        let cases: [(fn(f64) -> f64, f64, f64, f64); 2] = [
            (|x| x * x * x - 2.0, 1.0, 2.0, 1.259_921_049_894_873),
            (|x| (10.0 * x).exp() - 2.0, 0.0, 1.0, 2.0_f64.ln() / 10.0),
        ];
        for (f, a, b, root) in cases {
            for mirrored in [false, true] {
                let n = AtomicU32::new(0);
                let g = |x: f64| {
                    n.fetch_add(1, Ordering::Relaxed);
                    Some(if mirrored { f(-x) } else { f(x) })
                };
                let (lo, hi) = if mirrored { (-b, -a) } else { (a, b) };
                let found = illinois(&g, lo, hi, 1.0e-12).unwrap();
                let want = if mirrored { -root } else { root };
                let reads = n.load(Ordering::Relaxed);
                assert!(
                    (found - want).abs() < 1.0e-9,
                    "root = {found}, wanted {want}"
                );
                assert!(
                    reads <= 22,
                    "{reads} reads, mirrored {mirrored}, on [{lo}, {hi}]"
                );
            }
        }
    }

    #[test]
    fn bisect_to_stops_at_its_tolerance() {
        let f = |x: f64| Some(x - 0.3);
        let root = bisect_to(&f, 0.0, 1.0, 1.0e-6).unwrap();
        assert!((root - 0.3).abs() <= 1.0e-6, "root = {root}");
        // The bracket halved 20 times is about 1e-6 wide.
        let coarse = bisect_to(&f, 0.0, 1.0, 0.1).unwrap();
        assert!((coarse - 0.3).abs() <= 0.1);
        assert!((coarse - 0.3).abs() > 1.0e-6);
    }

    /// The span over which the Sun counts as up inside an eclipse: first and
    /// last instants with a non-negative margin, refined to the grid's 15
    /// minute cell.
    #[test]
    fn up_span_finds_sunrise_sunset_and_the_dip() {
        let near = |a: f64, b: f64| assert!((a - b).abs() < 2.0e-6, "{a} vs {b}");
        // Up throughout, down throughout.
        assert_eq!(up_span(&|_| Some(1.0), 0.0, 0.2), Some((true, 0.0, 0.2)));
        assert!(!up_span(&|_| Some(-1.0), 0.0, 0.2).unwrap().0);
        // Sunrise during the span: margin crosses zero upward at 0.07.
        let (up, a, b) = up_span(&|t| Some(t - 0.07), 0.0, 0.2).unwrap();
        assert!(up);
        near(a, 0.07);
        assert_eq!(b, 0.2);
        // Sunset during it: downward at 0.13.
        let (up, a, b) = up_span(&|t| Some(0.13 - t), 0.0, 0.2).unwrap();
        assert!(up);
        assert_eq!(a, 0.0);
        near(b, 0.13);
        // Up only in the middle (the Sun rises and sets inside the span).
        let (up, a, b) = up_span(&|t| Some(0.04 - (t - 0.1).abs()), 0.0, 0.2).unwrap();
        assert!(up);
        near(a, 0.06);
        near(b, 0.14);
        // A dip below the horizon between two up stretches: the span runs from
        // the first up instant to the last.
        let (up, a, b) = up_span(&|t| Some((t - 0.1).abs() - 0.03), 0.0, 0.2).unwrap();
        assert!(up);
        assert_eq!((a, b), (0.0, 0.2));
        // A missing callback is a missing answer.
        assert_eq!(up_span(&|_| None, 0.0, 0.2), None);
    }

    /// Penumbral and umbral radii, worked by hand for Pm = 0.95, Ss = 0.2666
    /// and Ps = 0.00244 degrees.
    /// Danjon: Rp = 1.01 * 0.95 + 0.2666 + 0.00244 = 0.9595 + 0.26904 = 1.22854;
    /// Ru = 0.9595 - 0.2666 + 0.00244 = 0.69534.
    /// Chauvenet: Rp = 1.02 * (0.998340 * 0.95 + 0.26904) = 1.02 * 1.217463
    /// = 1.241812; Ru = 1.02 * (0.948423 - 0.2666 + 0.00244) = 1.02 * 0.684263
    /// = 0.697948.
    #[test]
    fn shadow_radii_follow_the_published_formulas() {
        let (rp, ru) = ShadowEnlargement::Danjon.radii_deg(0.95, 0.00244, 0.2666);
        assert!(
            (rp - 1.22854).abs() < 1.0e-9 && (ru - 0.69534).abs() < 1.0e-9,
            "{rp} {ru}"
        );
        let (rp, ru) = ShadowEnlargement::Chauvenet.radii_deg(0.95, 0.00244, 0.2666);
        assert!(
            (rp - 1.241_812_26).abs() < 1.0e-8 && (ru - 0.697_948_26).abs() < 1.0e-8,
            "{rp} {ru}"
        );
        assert_eq!(ShadowEnlargement::default(), ShadowEnlargement::Danjon);
    }

    /// The Sun's margin over the standard rise/set altitude, worked by hand
    /// for an observer on the equator with the Sun on the equator at 1 AU: on
    /// the meridian its altitude is 90 degrees, so the margin is 90 + 50/60; at
    /// hour angle 90 degrees it is on the geometric horizon (less the Sun's
    /// parallax of 8.8 arcseconds), so the margin is 50/60 - 0.0024.
    #[test]
    fn sun_horizon_margin_by_hand() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        let lon_east = (-gmst(T0).to_degrees()).rem_euclid(360.0);
        let sun_at = |ra_deg: f64| SolarGeometry {
            sun_ra_deg: ra_deg,
            sun_dec_deg: 0.0,
            sun_dist_au: 1.0,
            moon_ra_deg: ra_deg,
            moon_dec_deg: 0.0,
            moon_dist_au: 0.00257,
        };
        // Local sidereal time is 0 at T0 for this longitude: RA 0 is the meridian.
        let up = sun_horizon_margin(&sun_at(0.0), T0, 0.0, lon_east);
        assert!(
            (up - (90.0 + 50.0 / 60.0)).abs() < 1.0e-4,
            "margin on the meridian {up}"
        );
        // RA 270 puts the Sun at hour angle 90 degrees (west), on the horizon.
        let set = sun_horizon_margin(&sun_at(270.0), T0, 0.0, lon_east);
        assert!(
            (set - (50.0 / 60.0 - 0.0024)).abs() < 5.0e-4,
            "margin at the horizon {set}"
        );
        // RA 90 is hour angle -90 (east): the same.
        let rise = sun_horizon_margin(&sun_at(90.0), T0, 0.0, lon_east);
        assert!((rise - set).abs() < 1.0e-6, "{rise} vs {set}");
        // Below the horizon the margin is negative: hour angle 180.
        assert!(sun_horizon_margin(&sun_at(180.0), T0, 0.0, lon_east) < -89.0);
    }
}
