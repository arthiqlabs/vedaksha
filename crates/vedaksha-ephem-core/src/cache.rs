// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Memoizing ephemeris provider.
//!
//! Computing a full chart evaluates the same provider state many times over.
//! In particular, the light-time correction of every planet needs Earth's own
//! state at the observation epoch, and that anchor is identical across all
//! bodies at a shared timestamp. This wrapper memoizes `compute_state` results
//! keyed on `(body, jd.to_bits())`, so repeated lookups at an identical time
//! return the cached state, and memoizes
//! [`EphemerisProvider::earth_state`] separately on `jd.to_bits()` alone —
//! forwarded to the inner provider so its own override survives the wrapping.
//!
//! Both keys use the exact IEEE-754 bit pattern of `jd`, so a cache hit
//! returns the **bit-identical** state vector the inner provider would have
//! produced — there is no accuracy trade-off, only deduplicated work.

#[cfg(feature = "std")]
use core::cell::RefCell;
#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::bodies::Body;
use crate::error::ComputeError;
use crate::jpl::{EphemerisProvider, StateVector};

/// An [`EphemerisProvider`] that memoizes the wrapped provider's
/// `compute_state` results for the lifetime of the wrapper.
///
/// Construct one per logical computation (a chart, a transit scan over a
/// fixed instant, …) so that repeated state lookups at identical timestamps
/// are served from the cache. Each cache hit is bit-identical to the inner
/// provider's output.
///
/// Errors are not cached: a failing lookup is retried on the next call.
#[cfg(feature = "std")]
pub struct CachingProvider<'a> {
    inner: &'a dyn EphemerisProvider,
    cache: RefCell<HashMap<(Body, u64), StateVector>>,
    /// Earth's own state, memoized separately.
    ///
    /// Earth is not a [`Body`] — it is reached through
    /// [`EphemerisProvider::earth_state`], which each provider may construct
    /// its own way. Forwarding to `inner.earth_state` rather than letting the
    /// trait default run against `self` is what keeps a provider's override
    /// (notably `AnalyticalProvider`'s, which skips two cancelling ELP/MPP02
    /// evaluations) reachable through the cache; this second map restores the
    /// memoization that forwarding would otherwise bypass.
    earth_cache: RefCell<HashMap<u64, StateVector>>,
}

#[cfg(feature = "std")]
impl<'a> CachingProvider<'a> {
    /// Wrap `inner` with an empty memoization cache.
    #[must_use]
    pub fn new(inner: &'a dyn EphemerisProvider) -> Self {
        Self {
            inner,
            cache: RefCell::new(HashMap::new()),
            earth_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Number of distinct `(body, time)` states currently cached.
    ///
    /// Counts the `compute_state` cache only; memoized Earth anchors are in
    /// their own map (see [`Self::earth_len`]).
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    /// Number of distinct times for which Earth's own state is cached.
    #[must_use]
    pub fn earth_len(&self) -> usize {
        self.earth_cache.borrow().len()
    }

    /// Whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.borrow().is_empty()
    }
}

#[cfg(feature = "std")]
impl EphemerisProvider for CachingProvider<'_> {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        let key = (body, jd.to_bits());
        if let Some(state) = self.cache.borrow().get(&key).copied() {
            return Ok(state);
        }
        let state = self.inner.compute_state(body, jd)?;
        self.cache.borrow_mut().insert(key, state);
        Ok(state)
    }

    /// Memoized forward to the wrapped provider's own `earth_state`.
    ///
    /// Deliberately **not** left to the trait default. The default body would
    /// run against `self`, decomposing Earth into two cached `compute_state`
    /// lookups — which is correct, but silently discards whatever the inner
    /// provider's override does. `AnalyticalProvider`'s override returns
    /// VSOP87A's Earth series directly instead of paying two cancelling
    /// ELP/MPP02 evaluations, and it must survive being wrapped.
    ///
    /// Errors are not cached, matching [`Self::compute_state`].
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        let key = jd.to_bits();
        if let Some(state) = self.earth_cache.borrow().get(&key).copied() {
            return Ok(state);
        }
        let state = self.inner.earth_state(jd)?;
        self.earth_cache.borrow_mut().insert(key, state);
        Ok(state)
    }

    fn time_range(&self) -> (f64, f64) {
        self.inner.time_range()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::jpl::{Position, Velocity};
    use core::cell::Cell;

    struct CountingProvider {
        calls: Cell<usize>,
        /// Counts calls to this provider's own `earth_state` **override**, so a
        /// wrapper that fell through to the trait default (which would reach
        /// `compute_state` instead) is distinguishable from one that forwarded.
        earth_calls: Cell<usize>,
    }

    impl CountingProvider {
        fn new() -> Self {
            Self {
                calls: Cell::new(0),
                earth_calls: Cell::new(0),
            }
        }
    }

    impl EphemerisProvider for CountingProvider {
        fn compute_state(&self, _body: Body, jd: f64) -> Result<StateVector, ComputeError> {
            self.calls.set(self.calls.get() + 1);
            Ok(StateVector {
                position: Position {
                    x: jd,
                    y: 0.0,
                    z: 0.0,
                },
                velocity: Velocity {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            })
        }

        /// A distinguishable override: `x = -jd`, so a value produced by this
        /// method can never be confused with one the default body would have
        /// assembled from `compute_state`.
        fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
            self.earth_calls.set(self.earth_calls.get() + 1);
            Ok(StateVector {
                position: Position {
                    x: -jd,
                    y: 0.0,
                    z: 0.0,
                },
                velocity: Velocity {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            })
        }

        fn time_range(&self) -> (f64, f64) {
            (0.0, f64::MAX)
        }
    }

    /// The wrapper must forward `earth_state` to the inner provider's override
    /// and memoize the result — not fall through to the trait default.
    ///
    /// This is the guard for the mistake that would silently undo Wave 1:
    /// `CachingProvider` implements `EphemerisProvider`, so omitting its
    /// `earth_state` compiles fine and quietly runs the default body against
    /// `self`, throwing away `AnalyticalProvider`'s override and reinstating
    /// two cancelling ELP/MPP02 evaluations per Earth anchor.
    #[test]
    fn earth_state_forwards_to_the_inner_override_and_memoizes() {
        let inner = CountingProvider::new();
        let cached = CachingProvider::new(&inner);

        let a = cached.earth_state(2_451_545.0).unwrap();
        let b = cached.earth_state(2_451_545.0).unwrap();

        // -jd is the override's signature; the default body would have given +jd.
        assert_eq!(
            a.position.x, -2_451_545.0,
            "fell through to the trait default"
        );
        assert_eq!(a.position.x.to_bits(), b.position.x.to_bits());
        assert_eq!(inner.earth_calls.get(), 1, "second lookup was not memoized");
        assert_eq!(
            inner.calls.get(),
            0,
            "earth_state reached compute_state, so it ran the default body"
        );
        assert_eq!(cached.earth_len(), 1);
        assert_eq!(cached.len(), 0, "the Earth anchor polluted the body cache");

        cached.earth_state(2_451_546.0).unwrap();
        assert_eq!(inner.earth_calls.get(), 2);
        assert_eq!(cached.earth_len(), 2);
    }

    #[test]
    fn identical_lookups_hit_cache() {
        let inner = CountingProvider::new();
        let cached = CachingProvider::new(&inner);

        let a = cached.compute_state(Body::Moon, 2_451_545.0).unwrap();
        let b = cached.compute_state(Body::Moon, 2_451_545.0).unwrap();

        // Bit-identical, and the inner provider was invoked exactly once.
        assert_eq!(a.position.x.to_bits(), b.position.x.to_bits());
        assert_eq!(inner.calls.get(), 1);
        assert_eq!(cached.len(), 1);
    }

    #[test]
    fn distinct_times_and_bodies_miss() {
        let inner = CountingProvider::new();
        let cached = CachingProvider::new(&inner);

        cached.compute_state(Body::Moon, 2_451_545.0).unwrap();
        cached.compute_state(Body::Moon, 2_451_545.5).unwrap();
        cached.compute_state(Body::Mars, 2_451_545.0).unwrap();

        assert_eq!(inner.calls.get(), 3);
        assert_eq!(cached.len(), 3);
    }
}
