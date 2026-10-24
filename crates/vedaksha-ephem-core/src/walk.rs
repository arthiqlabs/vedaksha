// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Interpolated apparent positions for scans over long spans of time.
//!
//! A search for the next time the Moon enters a rashi or a nakshatra, or for a
//! planet's station, steps through time asking for the body's apparent
//! ecliptic longitude at each step and then refines the step that brackets the
//! event. Every one of those asks goes through the whole apparent-place
//! pipeline: the series, the light-time iteration, precession, nutation and
//! aberration. For the Moon that is about 60 µs a time, and a search that wants
//! a second or a millisecond of accuracy asks thousands of times.
//!
//! The apparent longitude, latitude and distance are smooth functions of time.
//! [`BodyWalk`] samples them at the Chebyshev nodes of a short segment (about
//! twelve samples per four days for the Moon), keeps the polynomial, and answers
//! every time in the segment from it at the cost of a polynomial evaluation,
//! about 50 ns. The apparent-place pipeline is reached only to fit a segment,
//! the first time a time inside it is asked for.
//!
//! # Cost model
//!
//! Fitting a segment costs `nodes` calls of
//! [`ecliptic_position`](crate::coordinates::ecliptic_position). A walk pays
//! off when it is asked about more than `nodes` times per segment: for the
//! default Moon walk, more than three times a day on average when each ask
//! would be one `ecliptic_position`, and more than about once a day when each
//! would be an [`apparent_position`](crate::coordinates::apparent_position),
//! which is three of them (measured: a Moon ingress scan at 1.9 asks a day makes
//! a quarter of the provider calls through a walk). A scan that samples the
//! Moon once a day, and never again, costs more through a walk than directly. The break-even for the usual search shapes (an hourly bracket scan,
//! then bisection to the second) is a factor of several in the walk's favour.
//!
//! # Accuracy
//!
//! The interpolant differs from [`ecliptic_position`] by interpolation error
//! only. For the default Moon walk the bound is
//! [`BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC`] in longitude and latitude and
//! [`BodyWalk::MOON_DISTANCE_ERROR_METRES`] in distance, over the whole
//! supported range; the tests `moon_walk_stays_within_the_stated_bounds` and
//! `bounds_hold_across_the_delta_t_table_joints` measure it against direct
//! evaluation at random instants and around the joints of the Delta T model.
//! The fit corrects for the rounding of a Julian date (see [`crate::chebyshev`]),
//! which otherwise floors the error at the body's speed times 2.3e-10 day.
//!
//! The speed [`BodyWalk::longitude_speed`] returns is the interpolant's
//! derivative, per TT day (it differs from the per-UT1-day rate by the rate of
//! change of Delta T, below 1e-7 relative), and agrees with a central
//! difference of the direct positions to within
//! [`BodyWalk::MOON_SPEED_ERROR_DEG_PER_DAY`].
//!
//! # Why the grid is in Terrestrial Time
//!
//! [`ecliptic_position`] takes UT1 and converts to TT with [`delta_t`], a table
//! that steps: it is constant for days or months and then jumps, by up to a
//! tenth of a second near 1900. The Moon moves 0.55 arcsec a second, so the UT1
//! apparent longitude has steps of up to 0.06 arcsec that no polynomial follows.
//! The walk therefore interpolates the smooth function, the apparent place as a
//! function of TT ([`ecliptic_position_tt`]), and applies the same
//! [`delta_t::ut1_to_tt`] to the time it is asked about. It reproduces the
//! direct call's treatment of Delta T exactly, steps included, whatever the
//! table does.
//!
//! # Threads
//!
//! A walk is `Send + Sync`: the provider is `&(dyn EphemerisProvider + Sync)`
//! and each segment is a [`OnceLock`], so a closure over a walk is the
//! `Fn + Sync` callback the scans of `vedaksha-astro` and `vedaksha-vedic`
//! take, and threads asking for different segments fit them in parallel. Two
//! threads asking for the same unfitted segment fit it once; the second waits.
//!
//! # Why not the planets
//!
//! The Moon and the Sun are smooth functions of time at the bound above. The
//! planets are not, near their conjunctions with the Sun:
//! [`solar_deflection`](crate::deflection::solar_deflection) returns zero when
//! the straight ray from the planet to the observer crosses the Sun's disc, so
//! the apparent longitude steps where the planet's elongation crosses the
//! Sun's limb (Jupiter, JD 2454457.49435 TT: 1.089 arcsec between two samples
//! 1e-7 day apart, at an elongation of 0.2709 degree). No polynomial follows a
//! step, and a shorter segment or more nodes do not shrink it: the segment that
//! holds one is off by the step. The Moon and the Sun are not deflected and
//! have none. Do not build a walk for a planet; the segment tests below that
//! use planets pick instants away from a disc crossing.
//!
//! # Determinism
//!
//! Segments sit on a fixed grid (`J2000 + i · segment_days`, in TT) and are
//! fitted at fixed nodes, so the answer at a time does not depend on the span
//! the walk was created for, nor on the order times are asked in.

use core::f64::consts::{PI, TAU};
use std::sync::OnceLock;

use crate::bodies::Body;
use crate::chebyshev::Chebyshev;
use crate::coordinates::{EclipticCoords, ecliptic_position_tt};
use crate::delta_t::ut1_to_tt;
use crate::jpl::EphemerisProvider;

/// J2000.0, the origin of the segment grid.
const GRID_ORIGIN_JD: f64 = 2_451_545.0;

/// Segment length and node count of a walk.
///
/// The angle a body moves between two consecutive nodes must be below half a
/// turn, or the longitude cannot be unwrapped; for the Moon at the default size
/// it is about 3.5 degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkSpec {
    /// Segment length, in days of TT.
    pub days: f64,
    /// Samples per segment.
    pub nodes: usize,
}

impl WalkSpec {
    /// The size the Moon bounds on [`BodyWalk`] are measured at.
    pub const MOON: Self = Self {
        days: 4.0,
        nodes: 12,
    };
}

/// One segment's polynomial: unwrapped longitude (radians), latitude
/// (radians), distance (AU).
type Segment = Chebyshev<3>;

/// Apparent ecliptic coordinates of one body, interpolated from short fitted
/// segments. See the [module documentation](self).
pub struct BodyWalk<'p> {
    provider: &'p (dyn EphemerisProvider + Sync),
    body: Body,
    spec: WalkSpec,
    /// The span asked for, in UT1.
    start: f64,
    end: f64,
    /// Index (on the global TT grid) of `segments[0]`.
    first_index: i64,
    /// `None` inside the cell means the provider could not supply a node.
    segments: Vec<OnceLock<Option<Segment>>>,
}

impl<'p> BodyWalk<'p> {
    /// Largest difference, in arcseconds, between the Moon's interpolated
    /// apparent longitude or latitude and [`ecliptic_position`]'s, for a walk of
    /// [`WalkSpec::MOON`], anywhere in the supported range.
    pub const MOON_LONGITUDE_ERROR_ARCSEC: f64 = 5.0e-5;

    /// Largest difference, in metres, between the interpolated apparent
    /// distance and the direct one, same conditions.
    pub const MOON_DISTANCE_ERROR_METRES: f64 = 0.05;

    /// Largest difference, in degrees per day, between the interpolant's
    /// derivative and a central difference of direct longitudes over a
    /// 0.01-day step, same conditions.
    pub const MOON_SPEED_ERROR_DEG_PER_DAY: f64 = 1.0e-5;

    /// A walk over `[jd_ut_start, jd_ut_end]` (UT1) with the Moon's default
    /// segment size, [`WalkSpec::MOON`]. Use it for the Moon; other bodies move
    /// more slowly and want [`Self::with_spec`] and longer segments.
    ///
    /// Nothing is evaluated until a time is asked for.
    ///
    /// # Panics
    ///
    /// If `jd_ut_end < jd_ut_start`.
    #[must_use]
    pub fn moon(
        provider: &'p (dyn EphemerisProvider + Sync),
        jd_ut_start: f64,
        jd_ut_end: f64,
    ) -> Self {
        Self::with_spec(provider, Body::Moon, jd_ut_start, jd_ut_end, WalkSpec::MOON)
    }

    /// A walk over `[jd_ut_start, jd_ut_end]` (UT1) for any body, with explicit
    /// segment size. No error bound is stated for sizes other than
    /// [`WalkSpec::MOON`] on the Moon; measure. Segments longer than about eight
    /// days are measurably short of the nutation series' fastest terms (periods
    /// of a few days), whatever the body, at 12 nodes: a sixteen-day segment of
    /// the Sun is off by 1e-4 arcsec, an eight-day one by well under 1e-5.
    ///
    /// # Panics
    ///
    /// If `jd_ut_end < jd_ut_start`, `spec.days <= 0` or `spec.nodes < 2`.
    #[must_use]
    pub fn with_spec(
        provider: &'p (dyn EphemerisProvider + Sync),
        body: Body,
        jd_ut_start: f64,
        jd_ut_end: f64,
        spec: WalkSpec,
    ) -> Self {
        assert!(jd_ut_end >= jd_ut_start, "a walk needs end >= start");
        assert!(spec.days > 0.0 && spec.nodes >= 2, "invalid walk spec");
        let first_index = index_of(ut1_to_tt(jd_ut_start), spec.days);
        let last_index = index_of(ut1_to_tt(jd_ut_end), spec.days);
        let count = (last_index - first_index + 1) as usize;
        Self {
            provider,
            body,
            spec,
            start: jd_ut_start,
            end: jd_ut_end,
            first_index,
            segments: (0..count).map(|_| OnceLock::new()).collect(),
        }
    }

    /// The span this walk answers for, `(start, end)`, in UT1.
    #[must_use]
    pub fn span(&self) -> (f64, f64) {
        (self.start, self.end)
    }

    /// Number of segments fitted so far; each cost `nodes` calls of
    /// [`ecliptic_position`].
    #[must_use]
    pub fn segments_fitted(&self) -> usize {
        self.segments.iter().filter(|c| c.get().is_some()).count()
    }

    /// The segment polynomial covering `jd_tt`, fitting it if this is the first
    /// time it is asked for. `None` if the time is outside the span or the
    /// provider could not supply every node.
    fn segment(&self, jd_ut: f64, jd_tt: f64) -> Option<&Segment> {
        if !(jd_ut >= self.start && jd_ut <= self.end) {
            return None;
        }
        let index = index_of(jd_tt, self.spec.days);
        let cell = self.segments.get((index - self.first_index) as usize)?;
        cell.get_or_init(|| self.fit(index)).as_ref()
    }

    fn fit(&self, index: i64) -> Option<Segment> {
        let lo = GRID_ORIGIN_JD + index as f64 * self.spec.days;
        let hi = GRID_ORIGIN_JD + (index + 1) as f64 * self.spec.days;
        // Nodes arrive in order of decreasing time, a fraction of a turn apart,
        // so each longitude is unwrapped against the one before it.
        let mut previous: Option<f64> = None;
        let sample = |jd: f64| -> Result<[f64; 3], ()> {
            let c = ecliptic_position_tt(self.provider, self.body, jd).map_err(|_| ())?;
            let longitude = match previous {
                None => c.longitude,
                Some(p) => p + wrap_signed(c.longitude - p),
            };
            previous = Some(longitude);
            Ok([longitude, c.latitude, c.distance])
        };
        Segment::try_fit(lo, hi, self.spec.nodes, sample).ok()
    }

    /// Apparent ecliptic coordinates at `jd_ut` (UT1), longitude in `[0, 2π)`:
    /// what [`ecliptic_position`](crate::coordinates::ecliptic_position)
    /// returns, to within the interpolation bound.
    ///
    /// `None` if `jd_ut` is outside the span, or the apparent place could not
    /// be computed for one of the segment's nodes (the provider's range ends
    /// inside the segment): callers fall back to `ecliptic_position` for that
    /// time.
    #[must_use]
    pub fn ecliptic(&self, jd_ut: f64) -> Option<EclipticCoords> {
        self.ecliptic_at(jd_ut, ut1_to_tt(jd_ut))
    }

    /// [`Self::ecliptic`] addressed in TT, as
    /// [`ecliptic_position_tt`] is: `jd_tt` is Terrestrial Time and no Delta T
    /// is applied. The span check is against the walk's span converted to TT.
    #[must_use]
    pub fn ecliptic_tt(&self, jd_tt: f64) -> Option<EclipticCoords> {
        self.ecliptic_at(crate::delta_t::tt_to_ut1(jd_tt), jd_tt)
    }

    fn ecliptic_at(&self, jd_ut: f64, jd_tt: f64) -> Option<EclipticCoords> {
        let [longitude, latitude, distance] = self.segment(jd_ut, jd_tt)?.value(jd_tt);
        Some(EclipticCoords {
            longitude: longitude.rem_euclid(TAU),
            latitude,
            distance,
        })
    }

    /// Apparent ecliptic longitude at `jd_ut`, radians in `[0, 2π)`. See
    /// [`Self::ecliptic`].
    #[must_use]
    pub fn longitude(&self, jd_ut: f64) -> Option<f64> {
        self.ecliptic(jd_ut).map(|c| c.longitude)
    }

    /// Rate of change of apparent longitude at `jd_ut`, in degrees per day
    /// (positive is direct), from the interpolant's derivative. See
    /// [`Self::ecliptic`] for `None`.
    #[must_use]
    pub fn longitude_speed(&self, jd_ut: f64) -> Option<f64> {
        let jd_tt = ut1_to_tt(jd_ut);
        let rate = self.segment(jd_ut, jd_tt)?.derivative(jd_tt)[0];
        Some(rate * 180.0 / PI)
    }
}

/// Index of the grid segment containing `jd`.
fn index_of(jd: f64, days: f64) -> i64 {
    libm::floor((jd - GRID_ORIGIN_JD) / days) as i64
}

/// `x` brought into `(-π, π]` by whole turns.
fn wrap_signed(x: f64) -> f64 {
    let mut y = x.rem_euclid(TAU);
    if y > PI {
        y -= TAU;
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytical::{AnalyticalProvider, ChebyshevProvider};
    use crate::coordinates::ecliptic_position;

    const ARCSEC: f64 = 206_264.806_247_096_36;
    const JD_LO: f64 = 1_000_000.0;
    const JD_HI: f64 = 2_800_000.0;

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

    fn wrapped_arcsec(a: f64, b: f64) -> f64 {
        wrap_signed(a - b).abs() * ARCSEC
    }

    /// Rate of the direct TT-addressed longitude by a central difference, in
    /// degrees per day. TT-addressed so the Delta T table's steps stay out of it.
    fn direct_speed_tt(provider: &dyn EphemerisProvider, body: Body, jd_tt: f64) -> f64 {
        let step = 0.005;
        let lo = ecliptic_position_tt(provider, body, jd_tt - step)
            .unwrap()
            .longitude;
        let hi = ecliptic_position_tt(provider, body, jd_tt + step)
            .unwrap()
            .longitude;
        wrap_signed(hi - lo) / (2.0 * step) * 180.0 / PI
    }

    /// The stated Moon bounds, against direct evaluation, at random instants in
    /// 1900 to 2100 and across the whole supported range.
    #[test]
    fn moon_walk_stays_within_the_stated_bounds() {
        let provider = AnalyticalProvider::new();
        let walk = BodyWalk::moon(&provider, JD_LO, JD_HI);
        let mut worst = [0.0_f64; 3];
        let mut worst_speed = 0.0_f64;
        for (lo, hi) in [(2_415_020.0, 2_488_070.0), (JD_LO, JD_HI)] {
            for jd in instants(0x57A1 ^ lo as u64, 250, lo, hi) {
                let want = ecliptic_position(&provider, Body::Moon, jd).unwrap();
                let got = walk.ecliptic(jd).unwrap();
                assert!(
                    (0.0..TAU).contains(&got.longitude),
                    "longitude {} not in [0, 2pi)",
                    got.longitude
                );
                worst[0] = worst[0].max(wrapped_arcsec(want.longitude, got.longitude));
                worst[1] = worst[1].max((want.latitude - got.latitude).abs() * ARCSEC);
                worst[2] =
                    worst[2].max((want.distance - got.distance).abs() * crate::jpl::AU_KM * 1000.0);
                let jd_tt = ut1_to_tt(jd);
                let speed = walk.longitude_speed(jd).unwrap();
                worst_speed =
                    worst_speed.max((speed - direct_speed_tt(&provider, Body::Moon, jd_tt)).abs());
            }
        }
        println!(
            "Moon walk: longitude {:.2e}, latitude {:.2e} arcsec, distance {:.2e} m, speed {:.2e} deg/day",
            worst[0], worst[1], worst[2], worst_speed
        );
        assert!(
            worst[0] <= BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC,
            "longitude off by {:e} arcsec",
            worst[0]
        );
        assert!(
            worst[1] <= BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC,
            "latitude off by {:e} arcsec",
            worst[1]
        );
        assert!(
            worst[2] <= BodyWalk::MOON_DISTANCE_ERROR_METRES,
            "distance off by {:e} m",
            worst[2]
        );
        assert!(
            worst_speed <= BodyWalk::MOON_SPEED_ERROR_DEG_PER_DAY,
            "speed off by {worst_speed:e} deg/day"
        );
    }

    /// The Delta T table steps. The walk applies the same table to the time it
    /// is asked about and interpolates the smooth TT function, so on either
    /// side of a step it still agrees with the direct call to the same bound.
    /// (Interpolating the UT1 function instead would be off by the step, up to
    /// 0.06 arcsec.)
    #[test]
    fn bounds_hold_across_the_delta_t_table_joints() {
        let provider = AnalyticalProvider::new();
        let walk = BodyWalk::moon(&provider, 2_415_020.0, 2_470_000.0);

        // Find instants where ut1_to_tt jumps, by scanning at half-day steps.
        let mut joints = Vec::new();
        let mut jd = 2_415_020.0;
        while jd < 2_470_000.0 && joints.len() < 40 {
            let jump = (ut1_to_tt(jd + 0.5) - ut1_to_tt(jd) - 0.5).abs() * 86_400.0;
            if jump > 1e-3 {
                joints.push(jd);
            }
            jd += 0.5;
        }
        assert!(
            joints.len() >= 10,
            "found only {} Delta T steps; the model changed",
            joints.len()
        );

        let mut worst = 0.0_f64;
        let mut biggest_step = 0.0_f64;
        for &j in &joints {
            let step = (ut1_to_tt(j + 0.5) - ut1_to_tt(j) - 0.5).abs() * 86_400.0;
            biggest_step = biggest_step.max(step);
            for k in 0..=50 {
                let jd = j + 0.5 * f64::from(k) / 50.0;
                let want = ecliptic_position(&provider, Body::Moon, jd).unwrap();
                let got = walk.ecliptic(jd).unwrap();
                worst = worst.max(wrapped_arcsec(want.longitude, got.longitude));
            }
        }
        println!(
            "{} Delta T steps, largest {biggest_step:.3} s ({:.2e} arcsec of Moon); worst walk error across them {worst:.2e} arcsec",
            joints.len(),
            biggest_step * 0.55
        );
        assert!(
            worst <= BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC,
            "off by {worst:e} arcsec across a Delta T step"
        );
        // The test would pass vacuously if the steps were too small to matter.
        assert!(
            biggest_step * 0.55 > 1e-3 * 0.55,
            "steps too small to prove anything"
        );
    }

    #[test]
    fn answers_do_not_depend_on_the_span_or_the_order_of_questions() {
        let provider = AnalyticalProvider::new();
        let wide = BodyWalk::moon(&provider, JD_LO, JD_HI);
        let jds = instants(0xABCD, 80, 2_440_000.0, 2_460_000.0);
        let narrow_first: Vec<[u64; 3]> = jds
            .iter()
            .map(|&jd| {
                let c = BodyWalk::moon(&provider, jd - 1.0, jd + 1.0)
                    .ecliptic(jd)
                    .unwrap();
                [
                    c.longitude.to_bits(),
                    c.latitude.to_bits(),
                    c.distance.to_bits(),
                ]
            })
            .collect();
        for (&jd, want) in jds.iter().zip(&narrow_first).rev() {
            let c = wide.ecliptic(jd).unwrap();
            assert_eq!(
                [
                    c.longitude.to_bits(),
                    c.latitude.to_bits(),
                    c.distance.to_bits()
                ],
                *want,
                "jd {jd}"
            );
        }
    }

    #[test]
    fn a_walk_fits_lazily_and_only_once_per_segment() {
        let provider = AnalyticalProvider::new();
        let walk = BodyWalk::moon(&provider, 2_460_000.0, 2_460_400.0);
        assert_eq!(
            walk.segments_fitted(),
            0,
            "building a walk evaluates nothing"
        );
        let _ = walk.ecliptic(2_460_100.3).unwrap();
        assert_eq!(walk.segments_fitted(), 1);
        for k in 0..500 {
            let _ = walk.ecliptic(2_460_100.0 + 0.0001 * f64::from(k)).unwrap();
        }
        assert!(
            walk.segments_fitted() <= 2,
            "{} segments for times inside one",
            walk.segments_fitted()
        );
        let _ = walk.ecliptic(2_460_300.0).unwrap();
        assert!(walk.segments_fitted() >= 2);
    }

    #[test]
    fn times_outside_the_span_are_not_answered() {
        let provider = AnalyticalProvider::new();
        let walk = BodyWalk::moon(&provider, 2_460_000.0, 2_460_010.0);
        assert!(walk.ecliptic(2_459_999.0).is_none());
        assert!(walk.ecliptic(2_460_011.0).is_none());
        assert!(walk.longitude_speed(2_460_011.0).is_none());
        assert!(walk.ecliptic(2_460_005.0).is_some());
        assert_eq!(walk.span(), (2_460_000.0, 2_460_010.0));
    }

    /// A segment that covers the Moon's passage through 0 / 2 pi still agrees
    /// with the direct call: the unwrapping works.
    #[test]
    fn longitude_is_continuous_across_zero() {
        let provider = AnalyticalProvider::new();
        let start = 2_460_000.0;
        let walk = BodyWalk::moon(&provider, start, start + 60.0);
        // Find a day where the direct longitude wraps.
        let mut crossing = None;
        let mut prev = ecliptic_position(&provider, Body::Moon, start)
            .unwrap()
            .longitude;
        for d in 1..=60 {
            let now = ecliptic_position(&provider, Body::Moon, start + f64::from(d))
                .unwrap()
                .longitude;
            if now < prev {
                crossing = Some(start + f64::from(d));
                break;
            }
            prev = now;
        }
        let crossing = crossing.expect("the Moon crosses 0 within 60 days");
        let mut worst = 0.0_f64;
        let mut saw_low = false;
        let mut saw_high = false;
        for k in 0..=400 {
            let jd = crossing - 1.0 + 1.0 * f64::from(k) / 200.0;
            let want = ecliptic_position(&provider, Body::Moon, jd)
                .unwrap()
                .longitude;
            let got = walk.longitude(jd).unwrap();
            saw_low |= got < 0.2;
            saw_high |= got > TAU - 0.2;
            worst = worst.max(wrapped_arcsec(want, got));
        }
        assert!(saw_low && saw_high, "the samples did not straddle 0");
        assert!(
            worst <= BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC,
            "off by {worst:e} arcsec near 0"
        );
    }

    #[test]
    fn tt_addressing_matches_the_tt_pipeline() {
        let provider = AnalyticalProvider::new();
        let walk = BodyWalk::moon(&provider, 2_450_000.0, 2_470_000.0);
        let mut worst = 0.0_f64;
        for jd_tt in instants(0x7717, 100, 2_455_000.0, 2_465_000.0) {
            let want = ecliptic_position_tt(&provider, Body::Moon, jd_tt).unwrap();
            let got = walk.ecliptic_tt(jd_tt).unwrap();
            worst = worst.max(wrapped_arcsec(want.longitude, got.longitude));
        }
        assert!(
            worst <= BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC,
            "off by {worst:e} arcsec"
        );
    }

    /// Elongation from the Sun in degrees, from the exact apparent places.
    fn elongation_deg(exact: &dyn EphemerisProvider, body: Body, jd: f64) -> f64 {
        let a = ecliptic_position(exact, body, jd).unwrap();
        let s = ecliptic_position(exact, Body::Sun, jd).unwrap();
        let cos = a.latitude.sin() * s.latitude.sin()
            + a.latitude.cos() * s.latitude.cos() * (a.longitude - s.longitude).cos();
        cos.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// A walk is not special to the Moon or to the exact-series provider: a
    /// planet with longer segments, and the Chebyshev provider underneath.
    ///
    /// The planet is only asked about where it stands well away from the Sun.
    /// Close to a conjunction the Sun's light deflection peaks as `1 / elongation`
    /// (and switches off across the Sun's disc, see
    /// `a_planet_walk_misses_the_step_where_the_sun_deflection_switches_off`), a
    /// shape 12 nodes over 8 days do not hold: Saturn at 2459236.9, three days
    /// before its conjunction, is off by 4e-3 arcsec.
    #[test]
    fn other_bodies_and_other_providers_work() {
        let exact = AnalyticalProvider::new();
        let fitted = ChebyshevProvider::new();
        // Slow bodies still move through the nutation series, whose shortest
        // terms have periods of days, so segments cannot be much longer than
        // the Moon's: 16 days at this node count is measurably short of them.
        let spec = WalkSpec {
            days: 8.0,
            nodes: 12,
        };
        for body in [Body::Mars, Body::Saturn, Body::Sun] {
            let walk = BodyWalk::with_spec(&fitted, body, 2_455_000.0, 2_465_000.0, spec);
            let mut worst = 0.0_f64;
            let mut asked = 0;
            for jd in instants(0x0B0D, 120, 2_455_000.0, 2_465_000.0) {
                // A segment is 8 days, 8 days of the fastest planet is 12 degrees.
                if body != Body::Sun
                    && (elongation_deg(&exact, body, jd - 8.0) < 30.0
                        || elongation_deg(&exact, body, jd) < 30.0
                        || elongation_deg(&exact, body, jd + 8.0) < 30.0)
                {
                    continue;
                }
                let want = ecliptic_position(&exact, body, jd).unwrap();
                let got = walk.ecliptic(jd).unwrap();
                worst = worst.max(wrapped_arcsec(want.longitude, got.longitude));
                asked += 1;
            }
            println!(
                "{body:?} through the Chebyshev provider: {worst:.2e} arcsec, {asked} instants"
            );
            assert!(
                asked >= 50,
                "{body:?}: only {asked} instants away from the Sun"
            );
            assert!(worst < 5e-5, "{body:?} off by {worst:e} arcsec");
        }
    }

    /// Why the planets are not walked. `solar_deflection` returns zero when the
    /// straight ray crosses the Sun's disc, so Jupiter's apparent longitude
    /// steps by about 1.09 arcsec where its elongation crosses the Sun's limb
    /// (2454457.49435 TT, 0.2709 degree). The walk's segment holding it is off
    /// by the step, at every node count and segment size.
    #[test]
    fn a_planet_walk_misses_the_step_where_the_sun_deflection_switches_off() {
        use crate::coordinates::ecliptic_position_tt;
        use crate::delta_t::tt_to_ut1;

        let provider = AnalyticalProvider::new();
        let step_tt = 2_454_457.494_347_5;
        let lon = |jd_tt: f64| {
            ecliptic_position_tt(&provider, Body::Jupiter, jd_tt)
                .unwrap()
                .longitude
        };
        // The step itself: the longitude falls by about an arcsecond across 4e-6
        // day, where it moves 3e-3 arcsec.
        let drop = (lon(step_tt - 2.0e-6) - lon(step_tt + 2.0e-6)) * ARCSEC;
        assert!(
            (0.9..1.3).contains(&drop),
            "no step of about 1.09 arcsec at {step_tt}: {drop}"
        );

        let around = tt_to_ut1(step_tt);
        for spec in [
            WalkSpec {
                days: 8.0,
                nodes: 12,
            },
            WalkSpec {
                days: 2.0,
                nodes: 16,
            },
        ] {
            let walk =
                BodyWalk::with_spec(&provider, Body::Jupiter, around - 20.0, around + 20.0, spec);
            let mut worst = 0.0_f64;
            for k in -40..=40 {
                let jd_tt = step_tt + f64::from(k) * 0.01;
                let want = lon(jd_tt);
                let got = walk.ecliptic_tt(jd_tt).unwrap().longitude;
                worst = worst.max(wrapped_arcsec(want, got));
            }
            assert!(
                worst > 0.3,
                "{spec:?}: the walk followed a step it cannot (worst {worst} arcsec)"
            );
            // Away from it, a few segments off, the same walk is as good as ever.
            let far = (-20..=-10).chain(10..=20).map(|d| {
                let jd_tt = step_tt + f64::from(d);
                wrapped_arcsec(lon(jd_tt), walk.ecliptic_tt(jd_tt).unwrap().longitude)
            });
            let far = far.fold(0.0_f64, f64::max);
            assert!(far < 1.0e-3, "{spec:?}: {far} arcsec away from the step");
        }
    }

    /// The walk is shared between threads: that is what lets a closure over it
    /// be the `Fn + Sync` callback the scans take.
    #[test]
    fn a_walk_is_send_and_sync() {
        fn both<T: Send + Sync>() {}
        both::<BodyWalk<'static>>();
    }

    /// Threads asking for the same segments, in different orders, get the bits a
    /// single thread gets, and no segment is fitted twice.
    #[test]
    fn threads_sharing_a_walk_agree_with_one_thread() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_460_000.0, 2_460_200.0);
        let jds = instants(0x7EAD, 400, lo, hi);
        let alone = BodyWalk::moon(&provider, lo, hi);
        let want: Vec<[u64; 2]> = jds
            .iter()
            .map(|&jd| {
                [
                    alone.longitude(jd).unwrap().to_bits(),
                    alone.longitude_speed(jd).unwrap().to_bits(),
                ]
            })
            .collect();

        let shared = BodyWalk::moon(&provider, lo, hi);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..6)
                .map(|t| {
                    let (shared, jds, want) = (&shared, &jds, &want);
                    scope.spawn(move || {
                        // Each thread walks the same instants in its own order.
                        let mut order: Vec<usize> = (0..jds.len()).collect();
                        order.rotate_left(t * 61);
                        if t % 2 == 1 {
                            order.reverse();
                        }
                        for i in order {
                            let got = [
                                shared.longitude(jds[i]).unwrap().to_bits(),
                                shared.longitude_speed(jds[i]).unwrap().to_bits(),
                            ];
                            assert_eq!(got, want[i], "thread {t}, jd {}", jds[i]);
                        }
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
        });
        assert_eq!(
            shared.segments_fitted(),
            alone.segments_fitted(),
            "the same segments were fitted"
        );
    }

    #[test]
    #[should_panic(expected = "invalid walk spec")]
    fn a_degenerate_spec_is_refused() {
        let provider = AnalyticalProvider::new();
        let _ = BodyWalk::with_spec(
            &provider,
            Body::Moon,
            0.0,
            1.0,
            WalkSpec {
                days: 1.0,
                nodes: 1,
            },
        );
    }

    #[test]
    #[should_panic(expected = "end >= start")]
    fn a_backwards_span_is_refused() {
        let provider = AnalyticalProvider::new();
        let _ = BodyWalk::moon(&provider, 2.0, 1.0);
    }
}
