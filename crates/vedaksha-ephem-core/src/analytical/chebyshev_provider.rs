// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! An opt-in provider that fits the analytical series with Chebyshev
//! polynomials, as JPL's SPK kernels store them, and evaluates the polynomial.
//!
//! # What it is for
//!
//! One lunar position from the ELP/MPP02 series costs about 40 µs and a planet
//! from VSOP87A 6 to 20 µs. A polynomial evaluation costs about 50 ns. A
//! workload that asks for many positions close together in time (a scan of
//! rashi or nakshatra ingresses, a transit search at a fine step, the
//! bisection that refines an event) pays for each series evaluation again,
//! although the answer is a smooth function of time that a handful of samples
//! pin down. [`ChebyshevProvider`] samples the series at the Chebyshev nodes of
//! a short interval the first time a time inside it is asked for, keeps the
//! polynomial, and answers every later time in that interval from it.
//!
//! [`AnalyticalProvider`] is unchanged and stays the exact-series provider:
//! nothing selects this one except constructing it.
//!
//! # Cost model
//!
//! Fitting a segment costs `nodes` series evaluations (12 × 40 µs for a lunar
//! segment, so about 0.5 ms), after which every query in it is free by
//! comparison. A segment is only worth fitting if it will be asked about more
//! than `nodes` times, which for the lunar default (3 nodes per day) means
//! queries denser than about 3 per day: hourly and finer scans, repeated
//! bisection, dense charts of one period. A scan that samples the Moon once a
//! day and never revisits a segment is cheaper on [`AnalyticalProvider`].
//!
//! # Accuracy
//!
//! The polynomial interpolates the series at its nodes and is within the
//! stated bound of it everywhere in the segment; it adds no model error, only
//! interpolation error. The bounds are [`ChebyshevConfig::MOON_POSITION_ERROR_METRES`]
//! and [`ChebyshevConfig::PLANET_POSITION_ERROR_METRES`] (tighter ones hold
//! for 1500 to 2500 CE), and the test `fits_stay_within_the_stated_bounds`
//! measures them against the exact series, per body, over random instants
//! across the supported range. Velocity is the polynomial's derivative, which
//! converges more slowly than the value:
//! [`ChebyshevConfig::VELOCITY_ERROR_METRES_PER_DAY`].
//!
//! Two things in how a segment is fitted matter for that accuracy, and both are
//! in [`crate::chebyshev`]: node times are corrected for the rounding of a
//! Julian date to a multiple of 4.7e-10 day (uncorrected, the fit would be off
//! by that times the body's speed: 0.9 m for Mercury), and the polynomial is
//! evaluated at the same double the caller passed.
//!
//! # Determinism
//!
//! Segments sit on a fixed grid (`J2000 + i · segment_days`) and are fitted at
//! fixed nodes, so the answer for a time depends only on the time, never on
//! which times were asked before it or on how many threads asked.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use crate::bodies::Body;
use crate::chebyshev::Chebyshev;
use crate::error::ComputeError;
use crate::jpl::{EphemerisProvider, Position, StateVector, Velocity};

use super::vsop87a::Planet;
use super::{AnalyticalProvider, JD_MAX, JD_MIN, earth_to_emb, moon_position_rel_emb, vsop_state};

/// J2000.0, the origin of the segment grid.
const GRID_ORIGIN_JD: f64 = 2_451_545.0;

/// Slots in each body's segment cache (a power of two; direct-mapped by
/// segment index).
const CACHE_SLOTS: usize = 256;

/// Segment length and node count for one body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentSpec {
    /// Length of a segment, in days.
    pub days: f64,
    /// Nodes per segment (polynomial degree is one less).
    pub nodes: usize,
}

/// Segment sizes for every body the provider fits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChebyshevConfig {
    pub moon: SegmentSpec,
    pub earth: SegmentSpec,
    /// Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, in that order.
    pub planets: [SegmentSpec; 7],
}

impl ChebyshevConfig {
    /// Largest distance, in metres, between the Moon's fitted position and the
    /// exact series' anywhere in the supported range (about 6e-5 arcsec at the
    /// Moon's distance). Between 1500 and 2500 CE it is
    /// [`Self::MOON_POSITION_ERROR_MODERN_METRES`].
    ///
    /// Most of this is not interpolation error. The series itself is evaluated
    /// in `f64` and, far from J2000, rounds: the unwrapped longitude is a
    /// number of order 1e5 radians whose last bit is 2 cm of lunar distance.
    /// The polynomial smooths that rounding and so differs from it by about as
    /// much. Measured worst case over the supported range is 5e-2 m; the stated
    /// bound adds a factor of two.
    pub const MOON_POSITION_ERROR_METRES: f64 = 0.1;

    /// [`Self::MOON_POSITION_ERROR_METRES`] for 1500 to 2500 CE (about 1e-5
    /// arcsec). Measured worst case 7e-3 m.
    pub const MOON_POSITION_ERROR_MODERN_METRES: f64 = 0.02;

    /// Largest distance, in metres, between a fitted planet's (or Earth's, or
    /// the Earth-Moon barycentre's) position and the exact series' anywhere in
    /// the supported range, which at the nearest approach to Earth of the body
    /// it is worst for (Mercury) is under 1e-5 arcsec.
    ///
    /// As for the Moon, this is dominated by the series' own rounding, here the
    /// rounding of `B + C·t` with `C·t` of order 1e5 radians at the range edge,
    /// times an amplitude of a fraction of an AU. Measured worst case 1.5 m.
    pub const PLANET_POSITION_ERROR_METRES: f64 = 3.0;

    /// [`Self::PLANET_POSITION_ERROR_METRES`] for 1500 to 2500 CE. Measured
    /// worst case 0.16 m.
    pub const PLANET_POSITION_ERROR_MODERN_METRES: f64 = 0.4;

    /// Largest velocity error, in metres per day, over the supported range,
    /// for any fitted body. The velocity is the polynomial's derivative, which
    /// converges more slowly than the value and inherits the same rounding.
    pub const VELOCITY_ERROR_METRES_PER_DAY: f64 = 25.0;

    /// [`Self::VELOCITY_ERROR_METRES_PER_DAY`] for 1500 to 2500 CE. Measured
    /// worst case 0.8 m/day.
    pub const VELOCITY_ERROR_MODERN_METRES_PER_DAY: f64 = 3.0;
}

impl Default for ChebyshevConfig {
    /// Segment sizes chosen by measurement: for each body, the cheapest
    /// (days, nodes) whose error in 1500 to 2500 CE is at the rounding floor
    /// of the series itself. Shorter or fewer is measurably worse (the Moon at
    /// 4 days and 10 nodes is eight times worse than at 12); longer or more
    /// buys nothing.
    fn default() -> Self {
        let spec = |days, nodes| SegmentSpec { days, nodes };
        Self {
            moon: spec(4.0, 12),
            earth: spec(8.0, 12),
            planets: [
                spec(6.0, 12),  // Mercury
                spec(8.0, 12),  // Venus
                spec(24.0, 14), // Mars
                spec(32.0, 12), // Jupiter
                spec(32.0, 16), // Saturn
                spec(16.0, 12), // Uranus
                spec(16.0, 12), // Neptune
            ],
        }
    }
}

/// One cache entry: the segment index and its fitted polynomial.
struct Slot {
    index: i64,
    fit: Option<Chebyshev<3>>,
}

/// A body's segments, direct-mapped by index.
struct SegmentCache {
    spec: SegmentSpec,
    slots: Mutex<Vec<Slot>>,
    /// Segments fitted so far, counting refits after an eviction.
    fits: AtomicU64,
}

impl SegmentCache {
    fn new(spec: SegmentSpec) -> Self {
        assert!(spec.days > 0.0 && spec.nodes >= 2, "invalid segment spec");
        Self {
            spec,
            slots: Mutex::new(Vec::new()),
            fits: AtomicU64::new(0),
        }
    }

    /// Position (and, if `with_velocity`, its time derivative) at `jd`, fitting
    /// the segment containing `jd` from `sample` if it is not cached.
    fn lookup(
        &self,
        jd: f64,
        with_velocity: bool,
        sample: impl Fn(f64) -> Result<[f64; 3], ComputeError>,
    ) -> Result<([f64; 3], [f64; 3]), ComputeError> {
        let index = libm::floor((jd - GRID_ORIGIN_JD) / self.spec.days) as i64;
        let slot_of = index.rem_euclid(CACHE_SLOTS as i64) as usize;

        {
            let slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(Slot {
                index: i,
                fit: Some(fit),
            }) = slots.get(slot_of)
            {
                if *i == index {
                    return Ok(Self::read(fit, jd, with_velocity));
                }
            }
        }

        // Not cached: fit outside the lock. Two threads may fit the same
        // segment; they get the same polynomial, so whichever is stored wins.
        let lo = (GRID_ORIGIN_JD + index as f64 * self.spec.days).max(JD_MIN);
        let hi = (GRID_ORIGIN_JD + (index + 1) as f64 * self.spec.days).min(JD_MAX);
        // A segment clipped to nothing (an index boundary exactly on the edge
        // of the range) borrows the interval just inside it.
        let (lo, hi) = if hi - lo < 1e-6 {
            (hi - self.spec.days, hi)
        } else {
            (lo, hi)
        };
        let fit = Chebyshev::<3>::try_fit(lo, hi, self.spec.nodes, sample)?;
        self.fits.fetch_add(1, Ordering::Relaxed);
        let out = Self::read(&fit, jd, with_velocity);

        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        if slots.is_empty() {
            slots.extend((0..CACHE_SLOTS).map(|_| Slot {
                index: i64::MIN,
                fit: None,
            }));
        }
        slots[slot_of] = Slot {
            index,
            fit: Some(fit),
        };
        Ok(out)
    }

    fn read(fit: &Chebyshev<3>, jd: f64, with_velocity: bool) -> ([f64; 3], [f64; 3]) {
        let value = fit.value(jd);
        let derivative = if with_velocity {
            fit.derivative(jd)
        } else {
            [0.0; 3]
        };
        (value, derivative)
    }
}

/// The analytical ephemeris, answered from Chebyshev fits of the series.
///
/// See the [module documentation](self) for what it costs and what it buys.
pub struct ChebyshevProvider {
    inner: AnalyticalProvider,
    moon: SegmentCache,
    earth: SegmentCache,
    planets: [SegmentCache; 7],
}

const PLANET_ORDER: [Planet; 7] = [
    Planet::Mercury,
    Planet::Venus,
    Planet::Mars,
    Planet::Jupiter,
    Planet::Saturn,
    Planet::Uranus,
    Planet::Neptune,
];

impl ChebyshevProvider {
    /// A provider with the default segment sizes.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ChebyshevConfig::default())
    }

    /// A provider with explicit segment sizes.
    ///
    /// # Panics
    ///
    /// If a segment has a non-positive length or fewer than two nodes.
    #[must_use]
    pub fn with_config(config: ChebyshevConfig) -> Self {
        Self {
            inner: AnalyticalProvider::new(),
            moon: SegmentCache::new(config.moon),
            earth: SegmentCache::new(config.earth),
            planets: config.planets.map(SegmentCache::new),
        }
    }

    /// How many segments this provider has fitted so far, summed over bodies;
    /// a segment fitted again after the cache evicted it counts again.
    ///
    /// Each is `nodes` evaluations of the exact series, so this times the node
    /// count is the work the provider has spent that
    /// [`AnalyticalProvider`] would not have; compare it with the number of
    /// queries to see whether the access pattern is paying for the fits.
    #[must_use]
    pub fn segments_fitted(&self) -> u64 {
        let caches = [&self.moon, &self.earth].into_iter().chain(&self.planets);
        caches.map(|c| c.fits.load(Ordering::Relaxed)).sum()
    }

    fn check_range(jd: f64) -> Result<(), ComputeError> {
        if jd < JD_MIN || jd > JD_MAX {
            return Err(ComputeError::DateOutOfRange {
                jd,
                min: JD_MIN,
                max: JD_MAX,
            });
        }
        Ok(())
    }

    /// The Moon relative to the EMB, ICRS equatorial AU, and its velocity.
    fn moon(&self, jd: f64, with_velocity: bool) -> Result<([f64; 3], [f64; 3]), ComputeError> {
        self.moon.lookup(jd, with_velocity, |t| {
            let p = moon_position_rel_emb(t);
            Ok([p.x, p.y, p.z])
        })
    }

    /// Earth's centre, barycentric ICRS equatorial AU, and its velocity.
    fn earth(&self, jd: f64, with_velocity: bool) -> Result<([f64; 3], [f64; 3]), ComputeError> {
        self.earth.lookup(jd, with_velocity, |t| {
            let p = vsop_state(Planet::Earth, t).position;
            Ok([p.x, p.y, p.z])
        })
    }

    fn planet(
        &self,
        index: usize,
        jd: f64,
        with_velocity: bool,
    ) -> Result<([f64; 3], [f64; 3]), ComputeError> {
        let planet = PLANET_ORDER[index];
        self.planets[index].lookup(jd, with_velocity, |t| {
            let p = vsop_state(planet, t).position;
            Ok([p.x, p.y, p.z])
        })
    }
}

impl Default for ChebyshevProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn state_of((p, v): ([f64; 3], [f64; 3])) -> StateVector {
    StateVector {
        position: Position {
            x: p[0],
            y: p[1],
            z: p[2],
        },
        velocity: Velocity {
            x: v[0],
            y: v[1],
            z: v[2],
        },
    }
}

impl EphemerisProvider for ChebyshevProvider {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        Self::check_range(jd)?;
        match body {
            Body::Moon => Ok(state_of(self.moon(jd, true)?)),
            Body::EarthMoonBarycenter => {
                let earth = state_of(self.earth(jd, true)?);
                let moon = state_of(self.moon(jd, true)?);
                Ok(earth_to_emb(earth, moon))
            }
            Body::Mercury => Ok(state_of(self.planet(0, jd, true)?)),
            Body::Venus => Ok(state_of(self.planet(1, jd, true)?)),
            Body::Mars => Ok(state_of(self.planet(2, jd, true)?)),
            Body::Jupiter => Ok(state_of(self.planet(3, jd, true)?)),
            Body::Saturn => Ok(state_of(self.planet(4, jd, true)?)),
            Body::Uranus => Ok(state_of(self.planet(5, jd, true)?)),
            Body::Neptune => Ok(state_of(self.planet(6, jd, true)?)),
            // The Sun, the nodes and Pluto are not series evaluations; the
            // analytical provider answers (or refuses) them directly.
            _ => self.inner.compute_state(body, jd),
        }
    }

    /// Earth's own state, from its fit: VSOP87A's Earth, as
    /// [`AnalyticalProvider::earth_state`] returns it.
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        Self::check_range(jd)?;
        Ok(state_of(self.earth(jd, true)?))
    }

    fn moon_position(&self, jd: f64) -> Result<Position, ComputeError> {
        Self::check_range(jd)?;
        let (p, _) = self.moon(jd, false)?;
        Ok(Position {
            x: p[0],
            y: p[1],
            z: p[2],
        })
    }

    /// The length of [`Self::moon_position`]'s vector. It costs a polynomial
    /// evaluation, the same as the position, so unlike on the series provider
    /// there is nothing cheaper to return; it is still `Some`, and exact.
    fn moon_distance(&self, jd: f64) -> Option<f64> {
        let p = self.moon_position(jd).ok()?;
        Some(libm::sqrt(p.x * p.x + p.y * p.y + p.z * p.z))
    }

    fn time_range(&self) -> (f64, f64) {
        (JD_MIN, JD_MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinates::ecliptic_position;

    const MODERN: (f64, f64) = (2_451_545.0 - 182_625.0, 2_451_545.0 + 182_625.0);

    fn distance_m(a: &Position, b: &Position) -> f64 {
        let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);
        libm::sqrt(dx * dx + dy * dy + dz * dz) * crate::jpl::AU_KM * 1000.0
    }

    fn velocity_as_position(v: &Velocity) -> Position {
        Position {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }

    /// Deterministic instants in `[lo, hi]`.
    fn instants(seed: u64, count: usize, lo: f64, hi: f64) -> Vec<f64> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                lo + (state >> 11) as f64 / (1_u64 << 53) as f64 * (hi - lo)
            })
            .collect()
    }

    const FITTED: [Body; 9] = [
        Body::Moon,
        Body::EarthMoonBarycenter,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ];

    /// The stated bounds, against the exact series, per body and per era.
    #[test]
    fn fits_stay_within_the_stated_bounds() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        for (era, (lo, hi), count) in [("modern", MODERN, 200), ("full", (JD_MIN, JD_MAX), 400)] {
            for body in FITTED {
                let modern = era == "modern";
                let (pos_bound, vel_bound) = match (body == Body::Moon, modern) {
                    (true, true) => (
                        ChebyshevConfig::MOON_POSITION_ERROR_MODERN_METRES,
                        ChebyshevConfig::VELOCITY_ERROR_MODERN_METRES_PER_DAY,
                    ),
                    (true, false) => (
                        ChebyshevConfig::MOON_POSITION_ERROR_METRES,
                        ChebyshevConfig::VELOCITY_ERROR_METRES_PER_DAY,
                    ),
                    (false, true) => (
                        ChebyshevConfig::PLANET_POSITION_ERROR_MODERN_METRES,
                        ChebyshevConfig::VELOCITY_ERROR_MODERN_METRES_PER_DAY,
                    ),
                    (false, false) => (
                        ChebyshevConfig::PLANET_POSITION_ERROR_METRES,
                        ChebyshevConfig::VELOCITY_ERROR_METRES_PER_DAY,
                    ),
                };
                let (mut worst_p, mut worst_v) = (0.0_f64, 0.0_f64);
                for jd in instants(0xC0FF_EE00 + body as u64, count, lo, hi) {
                    let want = exact.compute_state(body, jd).unwrap();
                    let got = fitted.compute_state(body, jd).unwrap();
                    worst_p = worst_p.max(distance_m(&want.position, &got.position));
                    worst_v = worst_v.max(distance_m(
                        &velocity_as_position(&want.velocity),
                        &velocity_as_position(&got.velocity),
                    ));
                }
                println!(
                    "{body:?} {era}: worst position {worst_p:.2e} m (bound {pos_bound}), \
                     worst velocity {worst_v:.2e} m/day (bound {vel_bound})"
                );
                assert!(
                    worst_p <= pos_bound,
                    "{body:?} {era}: position {worst_p:e} m > {pos_bound}"
                );
                assert!(
                    worst_v <= vel_bound,
                    "{body:?} {era}: velocity {worst_v:e} m/day > {vel_bound}"
                );
            }
        }
    }

    /// Earth's own state, which the light-time iteration's observer anchor
    /// reads, is the VSOP87A Earth to the same bound.
    #[test]
    fn earth_state_follows_the_series_earth() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        let mut worst = 0.0_f64;
        for jd in instants(0xE4_27, 300, JD_MIN, JD_MAX) {
            let want = exact.earth_state(jd).unwrap();
            let got = fitted.earth_state(jd).unwrap();
            worst = worst.max(distance_m(&want.position, &got.position));
        }
        assert!(
            worst <= ChebyshevConfig::PLANET_POSITION_ERROR_METRES,
            "Earth off by {worst:e} m"
        );
    }

    /// The fits feed the geocentric pipeline: apparent longitude, latitude and
    /// distance through `ecliptic_position`, the call every scan makes.
    #[test]
    fn apparent_positions_agree_to_a_small_fraction_of_a_milliarcsecond() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        let arcsec = 206_264.806_247_096_36;
        // 1500 to 2500 CE in UT1; the Sun and the planets move so slowly that
        // their apparent error is dominated by the rounding floor of the fit.
        let mut worst = [0.0_f64; 3];
        for body in [Body::Moon, Body::Sun, Body::Mars, Body::Saturn] {
            for jd in instants(0xA11_CE, 150, MODERN.0, MODERN.1) {
                let a = ecliptic_position(&exact, body, jd).unwrap();
                let b = ecliptic_position(&fitted, body, jd).unwrap();
                let mut dlon = (a.longitude - b.longitude).abs();
                if dlon > core::f64::consts::PI {
                    dlon = core::f64::consts::TAU - dlon;
                }
                let dlat = (a.latitude - b.latitude).abs();
                let k = usize::from(body != Body::Moon);
                worst[k] = worst[k].max(dlon.max(dlat) * arcsec);
                if body == Body::Moon {
                    worst[2] =
                        worst[2].max((a.distance - b.distance).abs() * crate::jpl::AU_KM * 1000.0);
                }
            }
        }
        println!(
            "apparent: Moon {:.2e} arcsec, others {:.2e} arcsec, Moon distance {:.2e} m",
            worst[0], worst[1], worst[2]
        );
        // The bar for this change is 1e-4 arcsec and 1 m.
        assert!(
            worst[0] < 2e-5,
            "Moon apparent angle off by {:e} arcsec",
            worst[0]
        );
        assert!(
            worst[1] < 2e-5,
            "other bodies' apparent angle off by {:e} arcsec",
            worst[1]
        );
        assert!(
            worst[2] < 0.05,
            "Moon apparent distance off by {:e} m",
            worst[2]
        );
    }

    /// `moon_position` and `compute_state(Moon)` are one polynomial, and
    /// `moon_distance` is exactly its length.
    #[test]
    fn moon_reads_are_consistent_with_each_other() {
        let fitted = ChebyshevProvider::new();
        for jd in instants(0x600D, 100, JD_MIN, JD_MAX) {
            let p = fitted.moon_position(jd).unwrap();
            let s = fitted.compute_state(Body::Moon, jd).unwrap().position;
            assert_eq!(p.x.to_bits(), s.x.to_bits());
            assert_eq!(p.y.to_bits(), s.y.to_bits());
            assert_eq!(p.z.to_bits(), s.z.to_bits());
            let d = fitted.moon_distance(jd).unwrap();
            assert_eq!(
                d.to_bits(),
                libm::sqrt(p.x * p.x + p.y * p.y + p.z * p.z).to_bits()
            );
        }
    }

    /// The answer for a time is a function of the time alone: not of what was
    /// asked before it, nor of a cache slot being evicted by a segment that
    /// maps to the same slot.
    #[test]
    fn answers_do_not_depend_on_query_history_or_eviction() {
        let jds = instants(0xBEEF, 60, 2_440_000.0, 2_460_000.0);
        let fresh = |jd: f64| {
            ChebyshevProvider::new()
                .compute_state(Body::Moon, jd)
                .unwrap()
                .position
        };
        let reference: Vec<Position> = jds.iter().map(|&jd| fresh(jd)).collect();

        // Reverse order on one provider, with each query followed by one 256
        // segments away, which lands in the same direct-mapped slot and evicts it.
        let shared = ChebyshevProvider::new();
        let stride = 256.0 * ChebyshevConfig::default().moon.days;
        for (&jd, want) in jds.iter().zip(&reference).rev() {
            let got = shared.compute_state(Body::Moon, jd).unwrap().position;
            let _ = shared.compute_state(Body::Moon, jd + stride).unwrap();
            let again = shared.compute_state(Body::Moon, jd).unwrap().position;
            for (g, w) in [(got, want), (again, want)] {
                assert_eq!(g.x.to_bits(), w.x.to_bits());
                assert_eq!(g.y.to_bits(), w.y.to_bits());
                assert_eq!(g.z.to_bits(), w.z.to_bits());
            }
        }
    }

    /// Several threads asking about overlapping segments get the bits one
    /// thread gets.
    #[test]
    fn threads_agree_with_a_single_thread() {
        let jds = instants(0x7_7E4D, 400, 2_440_000.0, 2_445_000.0);
        let single: Vec<[u64; 3]> = {
            let p = ChebyshevProvider::new();
            jds.iter()
                .map(|&jd| {
                    let s = p.compute_state(Body::Moon, jd).unwrap().position;
                    [s.x.to_bits(), s.y.to_bits(), s.z.to_bits()]
                })
                .collect()
        };
        let shared = ChebyshevProvider::new();
        std::thread::scope(|scope| {
            for t in 0..4 {
                let (shared, jds, single) = (&shared, &jds, &single);
                scope.spawn(move || {
                    for k in (0..jds.len()).rev().skip(t).step_by(1) {
                        let s = shared.compute_state(Body::Moon, jds[k]).unwrap().position;
                        assert_eq!([s.x.to_bits(), s.y.to_bits(), s.z.to_bits()], single[k]);
                    }
                });
            }
        });
    }

    /// The first and last instants of the range, where the segment is clipped.
    #[test]
    fn the_range_edges_are_served_and_beyond_them_is_refused() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        for body in FITTED {
            for jd in [JD_MIN, JD_MIN + 1e-9, JD_MAX - 1e-9, JD_MAX] {
                let want = exact.compute_state(body, jd).unwrap();
                let got = fitted.compute_state(body, jd).unwrap();
                let bound = if body == Body::Moon {
                    ChebyshevConfig::MOON_POSITION_ERROR_METRES
                } else {
                    ChebyshevConfig::PLANET_POSITION_ERROR_METRES
                };
                assert!(
                    distance_m(&want.position, &got.position) <= bound,
                    "{body:?} at {jd}"
                );
            }
            for jd in [JD_MIN - 1.0, JD_MAX + 1.0] {
                assert!(matches!(
                    fitted.compute_state(body, jd),
                    Err(ComputeError::DateOutOfRange { .. })
                ));
            }
        }
    }

    /// Bodies the provider does not fit answer as the series provider does.
    #[test]
    fn unfitted_bodies_are_answered_by_the_series_provider() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        let jd = 2_460_000.5;
        let sun = fitted.compute_state(Body::Sun, jd).unwrap();
        let want = exact.compute_state(Body::Sun, jd).unwrap();
        assert_eq!(sun.position.x.to_bits(), want.position.x.to_bits());
        for body in [Body::Pluto, Body::MeanNode, Body::TrueNode] {
            assert!(matches!(
                fitted.compute_state(body, jd),
                Err(ComputeError::BodyNotAvailable { .. })
            ));
        }
    }

    /// A segment is fitted once and then reused: a thousand queries across
    /// three days (one or two 4-day segments) cost one or two fits, not a
    /// thousand evaluations, and a segment the direct-mapped cache evicted is
    /// fitted again.
    #[test]
    fn a_segment_is_fitted_once_and_refitted_after_eviction() {
        let fitted = ChebyshevProvider::new();
        let jd0 = 2_460_000.25;
        for k in 0..1000 {
            let _ = fitted.moon_position(jd0 + 0.003 * f64::from(k)).unwrap();
        }
        let fits = fitted.segments_fitted();
        assert!(
            (1..=2).contains(&fits),
            "{fits} fits for 1000 queries inside two segments"
        );

        // Same slot, different segment: evicts, so asking again refits.
        let stride = 256.0 * ChebyshevConfig::default().moon.days;
        let _ = fitted.moon_position(jd0 + stride).unwrap();
        assert_eq!(fitted.segments_fitted(), fits + 1);
        let _ = fitted.moon_position(jd0).unwrap();
        assert_eq!(
            fitted.segments_fitted(),
            fits + 2,
            "the evicted segment should be refitted"
        );

        // Reads inside a cached segment add none.
        let _ = fitted.moon_position(jd0 + 0.1).unwrap();
        assert_eq!(fitted.segments_fitted(), fits + 2);
    }

    #[test]
    #[should_panic(expected = "invalid segment spec")]
    fn a_degenerate_segment_is_refused() {
        let mut config = ChebyshevConfig::default();
        config.moon.nodes = 1;
        let _ = ChebyshevProvider::with_config(config);
    }
}
