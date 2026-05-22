// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Memoizing ephemeris provider.
//!
//! Computing a full chart evaluates the same provider state many times over.
//! In particular, the light-time correction of every planet calls
//! [`crate::coordinates`]`::earth_state` at the observation epoch, which in
//! turn evaluates the (expensive) ELP/MPP02 lunar series; that lunar
//! evaluation is identical across all bodies at a shared timestamp. This
//! wrapper memoizes `compute_state` results keyed on `(body, jd.to_bits())`,
//! so repeated lookups at an identical time return the cached state.
//!
//! The key uses the exact IEEE-754 bit pattern of `jd`, so a cache hit
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
}

#[cfg(feature = "std")]
impl<'a> CachingProvider<'a> {
    /// Wrap `inner` with an empty memoization cache.
    #[must_use]
    pub fn new(inner: &'a dyn EphemerisProvider) -> Self {
        Self {
            inner,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Number of distinct `(body, time)` states currently cached.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.borrow().len()
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
        fn time_range(&self) -> (f64, f64) {
            (0.0, f64::MAX)
        }
    }

    #[test]
    fn identical_lookups_hit_cache() {
        let inner = CountingProvider {
            calls: Cell::new(0),
        };
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
        let inner = CountingProvider {
            calls: Cell::new(0),
        };
        let cached = CachingProvider::new(&inner);

        cached.compute_state(Body::Moon, 2_451_545.0).unwrap();
        cached.compute_state(Body::Moon, 2_451_545.5).unwrap();
        cached.compute_state(Body::Mars, 2_451_545.0).unwrap();

        assert_eq!(inner.calls.get(), 3);
        assert_eq!(cached.len(), 3);
    }
}
