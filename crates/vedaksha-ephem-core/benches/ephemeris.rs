// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Performance baseline for the analytical ephemeris hot paths.
//!
//! Run with `cargo bench -p vedaksha-ephem-core`. These establish a measured
//! baseline for the four cost centers identified in the perf audit:
//!   1. VSOP87A planetary Poisson series (`vsop87a_planet`)
//!   2. ELP/MPP02 lunar series — the dominant single evaluation (`elp_mpp02_moon`)
//!   3. Full chart via `apparent_position` (`apparent_position_full_chart`)
//!   4. A 365-day lunar scan, modelling transit/dasha sweeps (`moon_scan_365d`)
//!   5. The position-only and distance-only lunar reads the light-time
//!      iteration makes (`elp_mpp02_moon_position`, `elp_mpp02_moon_distance`)
//!   6. The opt-in Chebyshev provider and the interpolated `BodyWalk`, against
//!      the same hourly scans on the exact-series provider (`scan_moon_hourly_30d`)

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use vedaksha_ephem_core::analytical::elp_mpp02::{
    elp_geocentric, elp_geocentric_distance, elp_geocentric_position,
};
use vedaksha_ephem_core::analytical::vsop87a::{Planet, vsop87a_heliocentric};
use vedaksha_ephem_core::analytical::{AnalyticalProvider, ChebyshevProvider};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    apparent_position, apparent_positions, ecliptic_position, moon_apparent_position,
};
use vedaksha_ephem_core::walk::BodyWalk;

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
        Planet::Saturn,
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

fn bench_moon_split_reads(c: &mut Criterion) {
    // The two lunar reads the light-time iteration makes: the full position at
    // the retarded instant and the distance-only first step.
    c.bench_function("elp_mpp02_moon_position", |b| {
        b.iter(|| elp_geocentric_position(black_box(JD)));
    });
    c.bench_function("elp_mpp02_moon_distance", |b| {
        b.iter(|| elp_geocentric_distance(black_box(JD)));
    });
}

fn bench_chebyshev(c: &mut Criterion) {
    // A query inside a segment that is already fitted: a polynomial evaluation.
    let warm = ChebyshevProvider::new();
    let _ = ecliptic_position(&warm, Body::Moon, JD).expect("warm");
    c.bench_function("chebyshev_ecliptic_position_moon_warm", |b| {
        b.iter(|| {
            let p = ecliptic_position(&warm, black_box(Body::Moon), black_box(JD))
                .expect("Moon position should be supported");
            black_box(p);
        });
    });
    // Fitting the Moon's segment: twelve series evaluations. A new provider per
    // iteration so every iteration pays it.
    c.bench_function("chebyshev_fit_moon_segment", |b| {
        use vedaksha_ephem_core::jpl::EphemerisProvider;
        b.iter(|| {
            let provider = ChebyshevProvider::new();
            black_box(provider.moon_position(black_box(JD)).expect("Moon"));
        });
    });
}

/// The shape the walk and the Chebyshev provider are for: a 30-day scan of the
/// Moon at an hourly step (720 reads), as a rashi or nakshatra ingress search
/// makes before it bisects. The same reads on the exact-series provider, on the
/// Chebyshev provider, and through a `BodyWalk`.
fn bench_scan(c: &mut Criterion) {
    const HOURS: usize = 24 * 30;
    let exact = AnalyticalProvider::new();
    let mut group = c.benchmark_group("scan_moon_hourly_30d");
    group.sample_size(20);
    group.bench_function("exact_series", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for h in 0..HOURS {
                let jd = JD + h as f64 / 24.0;
                acc += ecliptic_position(&exact, Body::Moon, black_box(jd))
                    .expect("Moon")
                    .longitude;
            }
            black_box(acc)
        });
    });
    group.bench_function("chebyshev_provider_cold", |b| {
        b.iter(|| {
            let provider = ChebyshevProvider::new();
            let mut acc = 0.0_f64;
            for h in 0..HOURS {
                let jd = JD + h as f64 / 24.0;
                acc += ecliptic_position(&provider, Body::Moon, black_box(jd))
                    .expect("Moon")
                    .longitude;
            }
            black_box(acc)
        });
    });
    group.bench_function("body_walk_cold", |b| {
        b.iter(|| {
            let walk = BodyWalk::moon(&exact, JD, JD + 31.0);
            let mut acc = 0.0_f64;
            for h in 0..HOURS {
                let jd = JD + h as f64 / 24.0;
                acc += walk.longitude(black_box(jd)).expect("in span");
            }
            black_box(acc)
        });
    });
    // The same walk asked again, as a bisection does: no fits, only polynomials.
    let walk = BodyWalk::moon(&exact, JD, JD + 31.0);
    for h in 0..HOURS {
        let _ = walk.longitude(JD + h as f64 / 24.0);
    }
    group.bench_function("body_walk_warm", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for h in 0..HOURS {
                let jd = JD + h as f64 / 24.0;
                acc += walk.longitude(black_box(jd)).expect("in span");
            }
            black_box(acc)
        });
    });
    group.finish();
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

fn bench_chart_batch(c: &mut Criterion) {
    // Same 11-body chart via the batch API, which shares one memoizing
    // provider across bodies and daily-motion timesteps. Compare against
    // `apparent_position_full_chart` to see the redundant-work saving.
    let provider = AnalyticalProvider::new();
    c.bench_function("apparent_positions_batch_chart", |b| {
        b.iter(|| {
            let out = apparent_positions(&provider, black_box(CHART_BODIES), black_box(JD));
            black_box(out);
        });
    });
}

fn bench_sun_position(c: &mut Criterion) {
    // `ecliptic_position(Sun, jd)` is the single most-called position in the
    // search paths — `search_transits`' coarse scan and `search_muhurta`'s
    // solar scan both walk it one grid point at a time, with no shared
    // memoizing provider across points. It is also the sharpest case for the
    // Earth-anchor cost: the Sun is the SSB origin, so essentially the entire
    // evaluation is `earth_state`.
    let provider = AnalyticalProvider::new();
    c.bench_function("ecliptic_position_sun", |b| {
        b.iter(|| {
            let p = ecliptic_position(&provider, black_box(Body::Sun), black_box(JD))
                .expect("Sun position should be supported");
            black_box(p);
        });
    });
}

fn bench_moon_reads(c: &mut Criterion) {
    // The three light-time-corrected Moon reads a lunar walk (rashi and
    // nakshatra entries) makes at every bracket step: longitude only, the
    // batch read with its central-difference speed, and the analytic-speed
    // read.
    let provider = AnalyticalProvider::new();
    c.bench_function("ecliptic_position_moon", |b| {
        b.iter(|| {
            let p = ecliptic_position(&provider, black_box(Body::Moon), black_box(JD))
                .expect("Moon position should be supported");
            black_box(p);
        });
    });
    c.bench_function("apparent_positions_moon", |b| {
        b.iter(|| black_box(apparent_positions(&provider, &[Body::Moon], black_box(JD))));
    });
    c.bench_function("moon_apparent_position", |b| {
        b.iter(|| {
            let p = moon_apparent_position(&provider, black_box(JD))
                .expect("Moon position should be supported");
            black_box(p);
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
    bench_moon_split_reads,
    bench_chebyshev,
    bench_scan,
    bench_chart,
    bench_chart_batch,
    bench_sun_position,
    bench_moon_reads,
    bench_transit_scan
);
criterion_main!(benches);
