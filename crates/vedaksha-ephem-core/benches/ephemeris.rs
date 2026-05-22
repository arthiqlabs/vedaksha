// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Performance baseline for the analytical ephemeris hot paths.
//!
//! Run with `cargo bench -p vedaksha-ephem-core`. These establish a measured
//! baseline for the four cost centers identified in the perf audit:
//!   1. VSOP87A planetary Poisson series (`vsop87a_planet`)
//!   2. ELP/MPP02 lunar series — the dominant single evaluation (`elp_mpp02_moon`)
//!   3. Full chart via `apparent_position` (`apparent_position_full_chart`)
//!   4. A 365-day lunar scan, modelling transit/dasha sweeps (`moon_scan_365d`)

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::elp_geocentric;
use vedaksha_ephem_core::analytical::vsop87a::{Planet, vsop87a_heliocentric};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::apparent_position;

/// A representative contemporary epoch (≈ 2025), well inside the supported range.
const JD: f64 = 2_460_676.5;

/// Bodies a real Vedic chart needs that the analytical provider supports
/// (Pluto is excluded — not available in the analytical theory).
const CHART_BODIES: &[Body] = &[
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::MeanNode,
    Body::TrueNode,
];

fn bench_planet(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsop87a_planet");
    for planet in [
        Planet::Mercury,
        Planet::Earth,
        Planet::Jupiter,
        Planet::Neptune,
    ] {
        group.bench_function(format!("{planet:?}"), |b| {
            b.iter(|| vsop87a_heliocentric(black_box(planet), black_box(JD)));
        });
    }
    group.finish();
}

fn bench_moon(c: &mut Criterion) {
    // The ELP/MPP02 lunar series is the hottest evaluation in the engine
    // (tens of thousands of trig terms per call).
    c.bench_function("elp_mpp02_moon", |b| {
        b.iter(|| elp_geocentric(black_box(JD)));
    });
}

fn bench_chart(c: &mut Criterion) {
    let provider = AnalyticalProvider::new();
    c.bench_function("apparent_position_full_chart", |b| {
        b.iter(|| {
            for &body in CHART_BODIES {
                let p = apparent_position(&provider, black_box(body), black_box(JD))
                    .expect("chart body should be supported");
                black_box(p);
            }
        });
    });
}

fn bench_transit_scan(c: &mut Criterion) {
    // 365 daily moon positions — models transit/dasha scanning where the same
    // series is evaluated across many dates (the batch-amortization target).
    c.bench_function("moon_scan_365d", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for day in 0..365 {
                let m = elp_geocentric(black_box(JD + f64::from(day)));
                acc += m.x;
            }
            black_box(acc)
        });
    });
}

criterion_group!(
    benches,
    bench_planet,
    bench_moon,
    bench_chart,
    bench_transit_scan
);
criterion_main!(benches);
