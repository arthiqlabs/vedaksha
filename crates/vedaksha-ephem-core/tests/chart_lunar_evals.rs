// Regression guard: a full chart must not re-evaluate the (expensive)
// ELP/MPP02 lunar series many times. With the memoizing batch provider and
// the light-time Earth-extrapolation, the Moon is evaluated only a handful of
// times per chart (anchors at the central-difference timesteps + the Moon
// body's own light-time iterations), not ~75×.

use std::cell::Cell;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::apparent_positions;
use vedaksha_ephem_core::error::ComputeError;
use vedaksha_ephem_core::jpl::{EphemerisProvider, StateVector};

struct Counter<'a> {
    inner: &'a AnalyticalProvider,
    moon: Cell<usize>,
}
impl EphemerisProvider for Counter<'_> {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        if body == Body::Moon {
            self.moon.set(self.moon.get() + 1);
        }
        self.inner.compute_state(body, jd)
    }
    fn time_range(&self) -> (f64, f64) {
        self.inner.time_range()
    }
}

#[test]
fn chart_evaluates_moon_only_a_few_times() {
    let ana = AnalyticalProvider::new();
    let counter = Counter {
        inner: &ana,
        moon: Cell::new(0),
    };
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
    let _ = apparent_positions(&counter, &bodies, 2_460_676.5);

    let moon_evals = counter.moon.get();
    println!("chart lunar-series evaluations: {moon_evals}");
    // Pre-optimization this was ~75 (memoized) / ~123 (naive). The anchor +
    // extrapolation collapses it to a handful (≈ 3 timesteps × a couple of
    // Moon-body light-time iterations). Guard generously against regression.
    assert!(
        moon_evals <= 15,
        "chart evaluated the lunar series {moon_evals} times (expected a handful); \
         the light-time Earth-extrapolation or batch memoization regressed"
    );
}
