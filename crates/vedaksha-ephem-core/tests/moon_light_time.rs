// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The Moon's light-time-corrected reads: unchanged answers, fewer series
//! evaluations, and the analytic-speed read.
//!
//! The light-time iteration for the Moon stopped re-evaluating the series at
//! an instant it had already evaluated, and takes its first step from the
//! distance series alone where the provider offers it
//! (`EphemerisProvider::moon_distance`). Both are claimed to change no output
//! bit. The bit-for-bit comparison against a verbatim copy of the old
//! iteration lives in `coordinates.rs` (`moon_light_time_tests`), where it can
//! reach the private frame; this file holds what the public API can show
//! (the end-to-end output bits are `analytical_bit_digest`'s to pin):
//!
//! - the evaluation counts the change exists to lower;
//! - the accuracy of `moon_apparent_position`'s analytic speed.

use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::{ELP_DISTANCE_CALLS, ELP_GEOCENTRIC_CALLS};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    apparent_position, apparent_positions, ecliptic_position, ecliptic_position_tt,
    moon_apparent_position,
};
use vedaksha_ephem_core::delta_t;

/// `n` UT1 instants spread over 1900 to 2100, the fractional day stepped by
/// the golden ratio so no two share a time of day.
fn instants(n: u32) -> impl Iterator<Item = f64> {
    let start = 2_415_020.5;
    let span = 73_048.0;
    (0..n).map(move |i| {
        let frac = (f64::from(i) * 0.618_033_988_749_894_9).fract();
        start + span * f64::from(i) / f64::from(n) + frac
    })
}

/// The ELP counters are process-global and libtest runs tests on threads of
/// one process, so every counting test holds this lock.
static COUNTER: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Full and distance-only ELP/MPP02 evaluations made by `read`.
fn count(read: impl FnOnce()) -> (u64, u64) {
    ELP_GEOCENTRIC_CALLS.store(0, Ordering::Relaxed);
    ELP_DISTANCE_CALLS.store(0, Ordering::Relaxed);
    read();
    (
        ELP_GEOCENTRIC_CALLS.load(Ordering::Relaxed),
        ELP_DISTANCE_CALLS.load(Ordering::Relaxed),
    )
}

/// A longitude-only Moon read costs one full series evaluation plus one
/// distance-only one, where 9.4.1 averaged 2.756 full evaluations and made
/// three in most reads.
#[test]
fn longitude_only_moon_read_evaluates_the_series_about_once() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 4_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            ecliptic_position(&p, Body::Moon, jd).unwrap();
        });
        assert_eq!(distance, 1, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("ecliptic_position(Moon): {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 2, "a read made {most_full} full evaluations");
    assert!(mean <= 1.01, "{mean} full evaluations per read");
}

/// `apparent_positions(Moon)` makes three light-time-corrected reads, each
/// now one full evaluation and one distance-only one, where 9.4.1 made six
/// full evaluations.
#[test]
fn apparent_positions_moon_evaluates_the_series_three_times() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 1_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            apparent_positions(&p, &[Body::Moon], jd)
                .remove(0)
                .1
                .unwrap();
        });
        assert_eq!(distance, 3, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("apparent_positions(Moon): {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 4, "a read made {most_full} full evaluations");
    assert!(mean <= 3.03, "{mean} full evaluations per read");
}

/// `moon_apparent_position` costs one position-and-velocity evaluation and
/// one distance-only one.
#[test]
fn moon_apparent_position_evaluates_the_series_about_once() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let n = 1_000_u32;
    let (mut full_total, mut most_full) = (0_u64, 0_u64);
    for jd in instants(n) {
        let (full, distance) = count(|| {
            moon_apparent_position(&p, jd).unwrap();
        });
        assert_eq!(distance, 1, "jd {jd}: {distance} distance-only evaluations");
        full_total += full;
        most_full = most_full.max(full);
    }
    let mean = full_total as f64 / f64::from(n);
    println!("moon_apparent_position: {mean} full evaluations per read, at most {most_full}");
    assert!(most_full <= 2, "a read made {most_full} full evaluations");
    assert!(mean <= 1.01, "{mean} full evaluations per read");
}

/// The analytic speed against two difference quotients of the same pipeline.
///
/// **Against a Richardson-extrapolated TT difference** of `ecliptic_position_tt`
/// (the function `moon_apparent_position` differentiates): `(4·D(h/2) − D(h))
/// / 3` with `D` the central difference and `h` = 0.08 day. A plain fine
/// difference cannot serve as the reference. The light-time iteration's
/// retarded instant `jd − τ` sits on the double grid, about 4.7e-10 day apart
/// near a modern `jd`, so the read longitude steps by up to about 6e-9° and a
/// central difference over `±h` carries noise near 6e-9 / 2h: 1.5e-4 °/day at
/// h = 2.5e-5 day, and still 1e-6 °/day at the step (about 0.005 day) where
/// it balances truncation. Extrapolation removes the `h²` truncation term and
/// allows a step wide enough that the grid noise is near 1e-7 °/day. Measured
/// over 1,500 instants, 1900 to 2100: [`RICHARDSON_TOL`] holds it with
/// margin, and is below what either part of the speed contributes on its own
/// (a dropped light-time factor reaches a few times 1e-6 °/day, a dropped
/// frame rate about 4e-5), so it would catch either one missing.
///
/// **Against `apparent_position`'s ±0.5-day UT1 difference**, the speed this
/// read sits beside: they differ by that difference's own truncation error,
/// worst 8.5e-3 °/day measured, with no bias.
#[test]
fn moon_apparent_position_speed_matches_difference_quotients() {
    let _guard = lock();
    let p = AnalyticalProvider::new();
    let longitude_tt = |jd_tt: f64| {
        ecliptic_position_tt(&p, Body::Moon, jd_tt)
            .unwrap()
            .longitude
    };
    let central = |jd_tt: f64, h: f64| {
        let mut d = longitude_tt(jd_tt + h) - longitude_tt(jd_tt - h);
        if d > std::f64::consts::PI {
            d -= 2.0 * std::f64::consts::PI;
        } else if d < -std::f64::consts::PI {
            d += 2.0 * std::f64::consts::PI;
        }
        d.to_degrees() / (2.0 * h)
    };
    let h = 0.08;
    let (mut worst_fine, mut worst_day, mut sum_day) = (0.0_f64, 0.0_f64, 0.0_f64);
    let n = 1_500_u32;
    for jd in instants(n) {
        let analytic = moon_apparent_position(&p, jd).unwrap().longitude_speed;

        let jd_tt = delta_t::ut1_to_tt(jd);
        let extrapolated = (4.0 * central(jd_tt, h / 2.0) - central(jd_tt, h)) / 3.0;
        worst_fine = worst_fine.max((analytic - extrapolated).abs());

        let day = apparent_position(&p, Body::Moon, jd)
            .unwrap()
            .longitude_speed;
        worst_day = worst_day.max((analytic - day).abs());
        sum_day += analytic - day;
    }
    let bias = sum_day / f64::from(n);
    println!(
        "analytic speed vs Richardson (h = {h} day): worst {worst_fine:e} °/day; \
         vs ±0.5-day: worst {worst_day:e} °/day, mean {bias:e} °/day"
    );
    assert!(
        worst_fine < RICHARDSON_TOL,
        "worst {worst_fine:e} °/day against the extrapolated difference"
    );
    assert!(
        worst_day < DAY_TOL,
        "worst {worst_day:e} °/day against ±0.5 day"
    );
    assert!(
        bias.abs() < DAY_TOL / 100.0,
        "mean difference {bias:e} °/day is a bias"
    );
}

/// Largest disagreement, °/day, between the analytic speed and the
/// Richardson-extrapolated difference.
const RICHARDSON_TOL: f64 = 5e-7;

/// Largest disagreement, °/day, between the analytic speed and
/// `apparent_position`'s ±0.5-day central difference: that difference's
/// truncation error, worst 8.5e-3 measured.
const DAY_TOL: f64 = 1e-2;
