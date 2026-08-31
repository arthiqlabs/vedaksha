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
// old wrapper (investigation finding #1). The guard was moved to read
// `elp_mpp02::ELP_GEOCENTRIC_CALLS`, an atomic counter incremented at the
// `elp_geocentric` call site.
//
// That still undercounted: the counter lived in `elp_geocentric`, one of
// three public entry points into `vur_series` (the other two,
// `elp_geocentric_of_date` and `elp_geocentric_with_fit`, call `vur_series`
// directly and never passed through it). `Body::TrueNode`'s osculating
// computation goes through `elp_geocentric_of_date` and never through
// `compute_state` at all, so it was invisible twice over. Fixed by moving
// the increment into `vur_series` itself, the one function all three public
// entry points share — it now counts every real ELP evaluation regardless
// of entry point or whether a `compute_state` arm was involved.
//
// Wave 2 (position-only ELP, docs/audit/2026-08-29-perf-investigation.md
// item #5) added two more public entry points, `elp_geocentric_position`
// and `elp_geocentric_position_of_date`, each routing through their own
// sibling function `vur_series_position` rather than `vur_series` -- so
// there are now five public entry points and two increment sites
// (`vur_series` and `vur_series_position`), not three and one. Both sites
// increment the same counter, so this guard still counts every real ELP
// evaluation regardless of which pipeline (position+velocity or
// position-only) served it.

use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::analytical::elp_mpp02::ELP_GEOCENTRIC_CALLS;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{apparent_positions, ecliptic_position};

/// `ELP_GEOCENTRIC_CALLS` is process-global and libtest runs the tests in this
/// file on threads of one process, so a reset-then-measure pair has to be
/// serialised or the two tests read each other's evaluations.
static COUNTER: Mutex<()> = Mutex::new(());

/// Take the counter lock and zero the counter. The guard must be held until
/// the count has been read back.
fn begin_count() -> MutexGuard<'static, ()> {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    ELP_GEOCENTRIC_CALLS.store(0, Ordering::Relaxed);
    guard
}

#[test]
fn chart_evaluates_moon_only_a_few_times() {
    let _guard = begin_count();

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
    // handful, though "handful" needs the corrected counter above to state
    // accurately — history of this number:
    //   - trait-level-only wrapper (pre-Wave-0): 6 (counted only
    //     `compute_state(Body::Moon, ·)`, missing the EMB path entirely).
    //   - Wave 0's `elp_geocentric`-site counter: 9, then, after Wave 1
    //     removed the EMB/Moon cancelling pair (`EphemerisProvider::earth_state`
    //     returning VSOP87A's Earth series directly): 6. Both undercounted —
    //     `Body::TrueNode`'s osculating computation runs through
    //     `elp_geocentric_of_date`, which never passed through the counter
    //     site or `compute_state` at all.
    //   - `vur_series`-site counter (current): **15**, the first count that
    //     includes the node path. One Moon-body light-time evaluation per
    //     central-difference timestep (6, as before) plus TrueNode's
    //     osculating computation (3, one per central-difference timestep) and
    //     the intermediate values `true_node_osculating` needs beyond a bare
    //     position.
    //
    // The threshold is set at 2x the corrected measured count rather than
    // pinning the exact number, so it still catches a real regression without
    // being brittle to noise; the exact-count assertion lives in
    // `sun_position_evaluates_the_lunar_series_not_at_all` below, where zero
    // is the only defensible value.
    assert!(
        elp_evals <= 30,
        "chart evaluated the ELP/MPP02 series {elp_evals} times (expected a handful, ~15); \
         the light-time Earth-extrapolation or batch memoization regressed"
    );
}

/// The sharpest case for investigation finding #1, pinned exactly.
///
/// `ecliptic_position(Sun, jd)` is the workhorse of `search_transits`' coarse
/// scan and `search_muhurta`'s solar scan. The Sun is the SSB origin on this
/// provider, so essentially the whole evaluation is the Earth anchor.
///
/// Before Wave 1 it cost **2** full 35,758-term ELP/MPP02 evaluations — one
/// inside `compute_state(EarthMoonBarycenter, ·)` building the EMB, one for
/// `compute_state(Moon, ·)` subtracting it straight back off — to produce a
/// position that VSOP87A's 2,202-term Earth series fully determines. Measured
/// with criterion's `ecliptic_position_sun` on aarch64: **531.65 µs before,
/// 11.21 µs after** — 47.4×.
///
/// Zero is asserted exactly rather than as a bound. There is no lunar quantity
/// anywhere in a solar position on this provider, so any non-zero count is a
/// reintroduced round trip, not noise.
#[test]
fn sun_position_evaluates_the_lunar_series_not_at_all() {
    let _guard = begin_count();

    let ana = AnalyticalProvider::new();
    let sun = ecliptic_position(&ana, Body::Sun, 2_460_676.5).expect("Sun is supported");

    let elp_evals = ELP_GEOCENTRIC_CALLS.load(Ordering::Relaxed);
    println!(
        "ecliptic_position(Sun) ELP/MPP02 evaluations: {elp_evals} (longitude {})",
        sun.longitude
    );
    assert_eq!(
        elp_evals, 0,
        "ecliptic_position(Sun, jd) evaluated the ELP/MPP02 lunar series {elp_evals} times. \
         A solar position contains no lunar quantity on this provider: the Earth anchor is \
         VSOP87A's Earth series directly (`AnalyticalProvider::earth_state`). A non-zero \
         count means the EMB-minus-Moon round trip is back — either the override was lost, \
         or a wrapper (e.g. `CachingProvider`) stopped forwarding `earth_state` to it and \
         fell through to the trait default."
    );
}
