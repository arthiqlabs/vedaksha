// SPDX-License-Identifier: BUSL-1.1
// Regression guard: a full chart must not re-evaluate the (expensive)
// ELP/MPP02 lunar series many times. With the memoizing batch provider and
// the light-time Earth-extrapolation, the Moon is evaluated only a handful of
// times per chart (anchors at the central-difference timesteps + the Moon
// body's own light-time iterations), not ~75×.
//
// Counting mechanism (fixed 2026-08-29, Wave 0 of
// docs/audit/2026-08-29-perf-investigation.md): this used to count only
// trait-level `compute_state(Body::Moon, ·)` calls via a wrapper around
// `AnalyticalProvider`. That undercounts by roughly half —
// `AnalyticalProvider::compute_state(EarthMoonBarycenter, jd)` also calls
// `moon_state(jd)` -> `elp_geocentric(jd)` internally (to build the EMB
// position, per `analytical/mod.rs`'s `earth_to_emb`), and that call never
// passes through `compute_state(Body::Moon, ·)` — it was invisible to the
// old wrapper (investigation finding #1). The guard now reads
// `elp_mpp02::ELP_GEOCENTRIC_CALLS`, an atomic counter incremented at the
// actual `elp_geocentric` call site, so it counts every real ELP evaluation
// regardless of which `compute_state` arm triggered it.

use std::sync::atomic::Ordering;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::ELP_GEOCENTRIC_CALLS;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::apparent_positions;

#[test]
fn chart_evaluates_moon_only_a_few_times() {
    // The counter is process-global, so reset it immediately before the
    // measured call. `cargo test` runs tests in this crate's `tests/`
    // binaries in separate processes per file by default, but resetting
    // here keeps the test correct even if that ever changes.
    ELP_GEOCENTRIC_CALLS.store(0, Ordering::Relaxed);

    let ana = AnalyticalProvider::new();
    let bodies = [
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
    let _ = apparent_positions(&ana, &bodies, 2_460_676.5);

    let elp_evals = ELP_GEOCENTRIC_CALLS.load(Ordering::Relaxed);
    println!("chart ELP/MPP02 evaluations: {elp_evals}");
    // Pre-optimization (naive, unmemoized) this was ~123. With the memoizing
    // batch provider and light-time Earth-extrapolation it collapses to a
    // handful. Measured 2026-08-29 for this exact chart: 9 real ELP
    // evaluations by this counter, versus 6 that the old trait-level-only
    // wrapper (which counted only `compute_state(Body::Moon, ·)`) reported —
    // the 3-call gap is `compute_state(EarthMoonBarycenter, ·)`'s internal
    // `moon_state` call (investigation finding #1: it computes a Moon
    // position solely to have it cancel, to 1 ULP, against
    // `coordinates::earth_state`'s own Moon evaluation). The threshold below
    // keeps the old guard's margin (2.5x the measured count) rather than
    // pinning the exact number, so it still catches a real regression
    // without being brittle to noise. This count is expected to drop once
    // Wave 1 removes the cancelling EMB/Moon pair.
    assert!(
        elp_evals <= 22,
        "chart evaluated the ELP/MPP02 series {elp_evals} times (expected a handful); \
         the light-time Earth-extrapolation or batch memoization regressed"
    );
}
