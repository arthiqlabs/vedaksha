// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `khagola::stations`: the series against the daily walk of `station`, the
//! counts against the synodic periods, and the step against the shortest
//! interval between stations.
//!
//! The daily walk is `station` itself, re-seeded a second past each station.
//! The engine's apparent positions and daily motion are the callback.

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};

use common::{jd, longitude_and_rate};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::khagola::{station, station_step_days, stations};

const ONE_SECOND: f64 = 1.0 / 86_400.0;

const PLANETS: [(Graha, Body); 5] = [
    (Graha::Mercury, Body::Mercury),
    (Graha::Venus, Body::Venus),
    (Graha::Mars, Body::Mars),
    (Graha::Jupiter, Body::Jupiter),
    (Graha::Saturn, Body::Saturn),
];

/// Every station in `[from, to)` by `station`, re-seeded a second past each.
fn daily_walk(
    graha: Graha,
    f: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    from: f64,
    to: f64,
) -> Vec<f64> {
    let mut out = Vec::new();
    let mut seed = from;
    // `station` looks 800 days ahead; a gap longer than that restarts from its end.
    while seed < to {
        match station(seed, graha, f) {
            Some(s) if s < to => {
                out.push(s);
                seed = s + ONE_SECOND;
            }
            Some(_) => break,
            None => seed += 800.0,
        }
    }
    out
}

/// Sidereal periods in days (Mercury to Saturn) and the Earth's; the synodic
/// period is `1 / |1/P - 1/P_earth|`.
fn synodic_days(graha: Graha) -> f64 {
    let p: f64 = match graha {
        Graha::Mercury => 87.969,
        Graha::Venus => 224.701,
        Graha::Mars => 686.980,
        Graha::Jupiter => 4332.589,
        Graha::Saturn => 10_759.22,
        _ => unreachable!(),
    };
    1.0 / (1.0 / p - 1.0 / 365.256_36).abs()
}

/// What the series must be, over `[from, to)`: the daily walk's stations to
/// the bisection's resolution, two per synodic period, and with a step under
/// half of the shortest gap; returns that gap and the station count.
fn check_span(graha: Graha, body: Body, from: f64, to: f64) -> (usize, f64, bool) {
    let p = AnalyticalProvider::new();
    let reads = AtomicUsize::new(0);
    let f = |t: f64| {
        reads.fetch_add(1, Ordering::Relaxed);
        longitude_and_rate(&p, body, t)
    };
    let walk = daily_walk(graha, &f, from, to);
    let walk_reads = reads.swap(0, Ordering::Relaxed);
    let series = stations(from, to, graha, &f);
    let series_reads = reads.load(Ordering::Relaxed);

    assert_eq!(
        series.len(),
        walk.len(),
        "{graha:?}: stations missed or invented"
    );
    let worst = series
        .iter()
        .zip(&walk)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    // The bisection's own tolerance is 1e-8 d; the rate itself is a difference
    // of positions a day apart, good to some 1e-12 deg/day, which at the
    // flattest rate curve moves the zero by a few 1e-8 d.
    assert!(
        worst < 1.0e-6,
        "{graha:?}: series differs from the daily walk by {worst} d"
    );
    assert!(
        series.windows(2).all(|w| w[0] < w[1]),
        "{graha:?}: not in time order"
    );

    let gaps: Vec<f64> = series.windows(2).map(|w| w[1] - w[0]).collect();
    let min_gap = gaps.iter().copied().fold(f64::MAX, f64::min);
    let at_min = gaps
        .iter()
        .position(|&g| g == min_gap)
        .map(|i| 0.5 * (series[i] + series[i + 1]))
        .unwrap();
    let retrograde = f(at_min).unwrap().1 < 0.0;
    let step = station_step_days(graha).unwrap();
    println!(
        "{graha:?}: {} stations, shortest interval {min_gap:.2} d ({}), step {step} d; reads {walk_reads} daily, {series_reads} series, worst difference {worst:e} d",
        series.len(),
        if retrograde { "retrograde" } else { "direct" }
    );
    assert!(
        step < 0.5 * min_gap,
        "{graha:?}: step {step} d, shortest interval {min_gap} d"
    );
    // Two stations per synodic period, within the ends of the span.
    let expected = 2.0 * (to - from) / synodic_days(graha);
    assert!(
        (series.len() as f64 - expected).abs() <= 2.0,
        "{graha:?}: {} stations, {expected:.1} from the synodic period",
        series.len()
    );
    (series.len(), min_gap, retrograde)
}

#[test]
fn the_series_is_the_daily_walk_over_six_years() {
    let from = jd(2020, 1, 1, 0.0);
    let to = jd(2026, 1, 1, 0.0);
    for (graha, body) in PLANETS {
        let (n, _, _) = check_span(graha, body, from, to);
        assert!(n >= 4, "{graha:?}: {n} stations in 6 years");
    }
}

#[test]
fn the_luminaries_and_nodes_have_no_stations() {
    let p = AnalyticalProvider::new();
    let from = jd(2020, 1, 1, 0.0);
    for (graha, body) in [
        (Graha::Sun, Body::Sun),
        (Graha::Moon, Body::Moon),
        (Graha::Rahu, Body::TrueNode),
        (Graha::Ketu, Body::TrueSouthNode),
    ] {
        let f = |t: f64| longitude_and_rate(&p, body, t);
        assert!(
            stations(from, from + 3000.0, graha, &f).is_empty(),
            "{graha:?}"
        );
        assert_eq!(station_step_days(graha), None);
    }
}

/// A rate that is a cosine of period 100 days: zeros at T0+25 and T0+75.
fn cosine(t0: f64) -> impl Fn(f64) -> Option<(f64, f64)> {
    move |t: f64| {
        let w = std::f64::consts::TAU * (t - t0) / 100.0;
        Some((
            100.0 + 5.0 * w.sin(),
            5.0 * std::f64::consts::TAU / 100.0 * w.cos(),
        ))
    }
}

#[test]
fn a_known_rate_has_its_stations_where_it_crosses_zero() {
    let t0 = jd(2020, 1, 1, 0.0);
    let f = cosine(t0);
    // Mars samples every 29 days: 0, 29, 58, 87, ... so both zeros fall inside a
    // step, the second after a sign that has not changed since the first.
    let found = stations(t0, t0 + 200.0, Graha::Mars, &f);
    assert_eq!(found.len(), 4, "{found:?}");
    for (i, t) in found.iter().enumerate() {
        let want = t0 + 25.0 + 50.0 * i as f64;
        assert!((t - want).abs() < 1.0e-6, "station {i}: {t} vs {want}");
    }
    // Half open at the end, and starting where it is told.
    assert!(stations(t0, t0 + 24.0, Graha::Mars, &f).is_empty());
    assert_eq!(stations(t0, t0 + 26.0, Graha::Mars, &f).len(), 1);
    assert_eq!(stations(t0 + 26.0, t0 + 200.0, Graha::Mars, &f).len(), 3);
    assert!(stations(t0, t0, Graha::Mars, &f).is_empty());
    assert!(stations(t0, t0 + 100.0, Graha::Mars, &|_| None).is_empty());
}

/// A rate that is exactly zero at a sample instant is a station there, once.
#[test]
fn a_rate_exactly_zero_at_a_sample_is_one_station() {
    let t0 = jd(2020, 1, 1, 0.0);
    // Zero at t0 + 29 (Mars's second sample) and at t0 itself for the second case.
    let f = |t: f64| Some((100.0, (t - t0 - 29.0) / 100.0));
    let found = stations(t0, t0 + 200.0, Graha::Mars, &f);
    assert_eq!(found, vec![t0 + 29.0]);
    let g = |t: f64| Some((100.0, (t - t0) / 100.0));
    let at_start = stations(t0, t0 + 200.0, Graha::Mars, &g);
    assert_eq!(at_start, vec![t0]);
    // `station` agrees about both.
    assert_eq!(station(t0, Graha::Mars, &g), Some(t0));
}
