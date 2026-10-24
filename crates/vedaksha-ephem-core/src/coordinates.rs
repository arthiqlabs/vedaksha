// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Coordinate transformation pipeline.
//!
//! Chains light-time correction, precession (IAU 2006), nutation (IAU 2000B),
//! annual aberration, and ecliptic rotation to produce apparent ecliptic
//! positions from JPL SPK ephemeris data.
//!
//! Source: Meeus, *Astronomical Algorithms*, 2nd ed., Chapters 22--23, 33.

use core::f64::consts::PI;

use crate::bodies::Body;
use crate::delta_t;
use crate::error::ComputeError;
use crate::jpl::{EMRAT, EphemerisProvider, MOON_DISTANCE_REL_TOL, Position, StateVector};
use crate::nutation;
use crate::obliquity;
use crate::precession;
use vedaksha_math::matrix::{Matrix3, Vector3};

/// Geocentric-to-EMB conversion factor for Moon: |Moon-geocentric| = factor × |Moon-rel-EMB|.
const MOON_GEO_FACTOR: f64 = (1.0 + EMRAT) / EMRAT;

/// Ecliptic coordinates of a celestial body.
#[derive(Debug, Clone, Copy)]
pub struct EclipticCoords {
    /// Ecliptic longitude in radians [0, 2pi)
    pub longitude: f64,
    /// Ecliptic latitude in radians [-pi/2, pi/2]
    pub latitude: f64,
    /// Distance from Earth in AU
    pub distance: f64,
}

/// Full apparent position with ecliptic coordinates and daily motion.
#[derive(Debug, Clone, Copy)]
pub struct ApparentPosition {
    /// Apparent ecliptic coordinates
    pub ecliptic: EclipticCoords,
    /// Daily motion in ecliptic longitude (degrees/day, positive = direct)
    pub longitude_speed: f64,
}

/// Compute apparent ecliptic coordinates (without speed) for a body at a
/// given Julian Day.
///
/// Pipeline:
/// 1. Light-time correction (planetary aberration formulation): produce
///    the geocentric vector `target(t-τ) - earth(t-τ)`. This already
///    captures both light-travel-time delay and observer-motion aberration
///    in one step; no separate stellar-aberration formula is needed for
///    solar-system bodies (cf. Meeus, *Astronomical Algorithms* 2nd ed.,
///    Ch. 33; Explanatory Supplement to the Astronomical Almanac §7.4).
/// 2. Precession (J2000 → mean equator of date)
/// 3. Nutation (mean → true equator of date)
/// 4. Rotate from true equatorial to ecliptic of date using true obliquity
/// 5. Extract longitude, latitude, distance
///
/// The Moon special case: SPK / Analytical providers expose the Moon
/// relative to EMB. Multiplying by `(1+EMRAT)/EMRAT` gives the geocentric
/// vector (Earth-EMB-Moon are collinear, so direction is preserved and
/// only magnitude changes).
/// The time-only part of the coordinate pipeline: the combined
/// nutation·precession rotation (J2000 mean equatorial → true equatorial of
/// date) and the true obliquity. These depend only on the timestamp, not the
/// body, so a chart computes them once per distinct time and reuses them
/// across all bodies (see [`frame_for`], [`apparent_positions`]).
#[derive(Clone, Copy, Debug)]
pub struct CelestialFrame {
    /// N · P — true equatorial of date from J2000 mean equatorial.
    combined: Matrix3,
    /// True obliquity of date (radians).
    eps_true: f64,
    /// Nutation in longitude Δψ (radians). The matrix above already carries
    /// it for state-vector bodies; the lunar nodes never become vectors, so
    /// they need the scalar to reach the same true-equinox-of-date frame.
    dpsi: f64,
}

impl CelestialFrame {
    /// True obliquity of date, in radians — the angle this frame rotates the
    /// true equator of date onto the ecliptic of date by, and therefore the one
    /// to pass to [`ecliptic_to_equatorial_deg`] for a position this pipeline
    /// produced.
    #[must_use]
    pub fn true_obliquity(&self) -> f64 {
        self.eps_true
    }
}

/// Rotate ecliptic coordinates of date onto the equator of date.
///
/// Takes ecliptic longitude `lon_deg`, latitude `lat_deg` and the obliquity
/// `obliquity_deg`, all in degrees, and returns `(right_ascension_deg ∈
/// [0, 360), declination_deg ∈ [−90, 90])`:
///
/// `sin δ = sin β·cos ε + cos β·sin ε·sin λ`,
/// `α = atan2(sin λ·cos ε − tan β·sin ε, cos λ)`.
///
/// Pass the **true** obliquity for apparent positions from
/// [`ecliptic_position`] / [`apparent_position`] — that pipeline rotates the
/// true equator onto the ecliptic by it (see [`CelestialFrame::true_obliquity`]),
/// and this is the inverse of that rotation. Mean obliquity would leave the
/// nutation in obliquity (up to ~9.2″) in the result.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13, eqs. 13.3–13.4.
#[must_use]
pub fn ecliptic_to_equatorial_deg(lon_deg: f64, lat_deg: f64, obliquity_deg: f64) -> (f64, f64) {
    let (lambda, beta, eps) = (
        lon_deg.to_radians(),
        lat_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let (sin_eps, cos_eps) = (libm::sin(eps), libm::cos(eps));
    let ra = libm::atan2(
        libm::sin(lambda) * cos_eps - libm::tan(beta) * sin_eps,
        libm::cos(lambda),
    );
    (
        ra.to_degrees().rem_euclid(360.0),
        declination_deg(lon_deg, lat_deg, obliquity_deg),
    )
}

/// Declination in degrees of a point at ecliptic longitude `lon_deg` and
/// latitude `lat_deg`, for obliquity `obliquity_deg` (all degrees):
/// `sin δ = sin β·cos ε + cos β·sin ε·sin λ`.
///
/// The declination half of [`ecliptic_to_equatorial_deg`] — see there for
/// which obliquity to pass.
///
/// `lon_deg` must be **tropical** longitude of date. A sidereal longitude is
/// offset by the ayanamsha (~24° today) and gives a wrong declination with no
/// error: add the ayanamsha back first. Classical strength reckonings that depend on a
/// graha's distance north or south of the equator (Ayana Bala, for one) want
/// exactly this quantity.
///
/// Source: Meeus, *Astronomical Algorithms* 2nd ed., Ch. 13, eq. 13.4.
#[must_use]
pub fn declination_deg(lon_deg: f64, lat_deg: f64, obliquity_deg: f64) -> f64 {
    let (lambda, beta, eps) = (
        lon_deg.to_radians(),
        lat_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let sin_dec =
        libm::sin(beta) * libm::cos(eps) + libm::cos(beta) * libm::sin(eps) * libm::sin(lambda);
    libm::asin(sin_dec.clamp(-1.0, 1.0)).to_degrees()
}

/// Build the [`CelestialFrame`] for a given TT Julian Day.
///
/// Identical arithmetic to the inline form previously in `compute_ecliptic`;
/// extracted so it can be hoisted out of the per-body loop.
#[must_use]
pub fn frame_for(jd_tt: f64) -> CelestialFrame {
    let prec = precession::precession_matrix(jd_tt);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let eps_a = obliquity::mean_obliquity(jd_tt);
    let eps_true = obliquity::true_obliquity(jd_tt, deps);

    let nut_matrix = Matrix3::rotation_x(-eps_a - deps)
        .multiply(&Matrix3::rotation_z(-dpsi))
        .multiply(&Matrix3::rotation_x(eps_a));

    CelestialFrame {
        combined: nut_matrix.multiply(&prec),
        eps_true,
        dpsi,
    }
}

/// Compute apparent ecliptic coordinates from an already-converted TT Julian
/// Day.
///
/// This is the single point in the pipeline that touches `jpl`/`nodes`
/// timestamps: `jd_tt` must already be Terrestrial Time. No caller in this
/// file may hand this a UT1 value — the public UT1-addressed functions below
/// convert exactly once, at their own boundary, before reaching here.
fn compute_ecliptic_with_frame(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_tt: f64,
    frame: &CelestialFrame,
) -> Result<EclipticCoords, ComputeError> {
    // Step 0: the lunar nodes are directions, not bodies.
    //
    // A node has no position, so it has no light-time, no aberration and no
    // distance, and its longitude is already referred to the ecliptic of
    // date. Routing it through the steps below — which subtract Earth's
    // barycentric position from the target's and then precess the result —
    // treats the direction as a place one AU away and moves the node by tens
    // of degrees. That is what this branch exists to prevent; see the tests
    // in `tests/node_frame.rs`.
    //
    // The only correction a node does take is nutation in longitude: the node
    // functions return the *mean* equinox of date and every other longitude
    // this function returns is referred to the *true* equinox of date.
    if let Some(mean_of_date_deg) = crate::nodes::node_longitude(body, jd_tt) {
        let mut longitude = mean_of_date_deg.to_radians() + frame.dpsi;
        if longitude < 0.0 {
            longitude += 2.0 * PI;
        } else if longitude >= 2.0 * PI {
            longitude -= 2.0 * PI;
        }
        return Ok(EclipticCoords {
            longitude,
            // A node lies in the ecliptic by definition, and has no distance.
            latitude: 0.0,
            distance: 0.0,
        });
    }

    // Step 1: planetary-aberration-form light-time iteration.
    //
    // We compute the geocentric vector with target AND observer at the
    // retarded time t-τ. For solar-system bodies this single step provides
    // the apparent direction in the observer's instantaneous rest frame —
    // adding stellar aberration on top would double-count.
    let geo = light_time_geocentric(provider, body, jd_tt)?;

    Ok(spherical_of(&ecliptic_of_date(geo, frame)))
}

/// Steps 2-4 of [`compute_ecliptic_with_frame`]: rotate a J2000
/// mean-equatorial vector into the ecliptic of date.
///
/// Apply the precomputed nutation·precession frame (the provider returns
/// J2000 mean-equatorial vectors; SPK ICRF agrees to <25 mas — negligible
/// here), then rotate true equatorial of date → ecliptic of date. `frame` must
/// correspond to the observation time. The map is linear, so
/// [`moon_apparent_position`] also passes a velocity through it.
fn ecliptic_of_date(geo: [f64; 3], frame: &CelestialFrame) -> Vector3 {
    let geo_vec = Vector3::new(geo[0], geo[1], geo[2]);
    let true_eq = frame.combined.apply(&geo_vec);
    Matrix3::rotation_x(frame.eps_true).apply(&true_eq)
}

/// Step 5 of [`compute_ecliptic_with_frame`]: spherical coordinates of an
/// ecliptic-of-date vector.
fn spherical_of(ecl_vec: &Vector3) -> EclipticCoords {
    let distance =
        libm::sqrt(ecl_vec.x * ecl_vec.x + ecl_vec.y * ecl_vec.y + ecl_vec.z * ecl_vec.z);
    let mut longitude = libm::atan2(ecl_vec.y, ecl_vec.x);
    if longitude < 0.0 {
        longitude += 2.0 * PI;
    }
    let latitude = libm::asin(ecl_vec.z / distance);

    EclipticCoords {
        longitude,
        latitude,
        distance,
    }
}

/// Compute the geocentric vector to `body` at observation time `jd` (TT),
/// applying planetary-aberration-form light-time correction.
///
/// For Moon: scales the rel-EMB vector returned by providers up to a true
/// geocentric vector via the (1+EMRAT)/EMRAT factor. Because Earth–EMB–Moon
/// are collinear, this preserves direction and corrects the distance.
///
/// Returns `[x, y, z]` in AU, in the J2000 mean equatorial frame.
fn light_time_geocentric(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
) -> Result<[f64; 3], ComputeError> {
    // For non-Moon bodies the geocentric vector subtracts Earth's barycentric
    // position at the retarded time t−τ. Re-evaluating Earth there pulls the
    // provider's `earth_state` at every iteration and for every body — which on
    // the SPK path is two kernel lookups and on the analytical path used to be
    // two full ELP/MPP02 lunar evaluations. Instead, anchor Earth's state once
    // at the observation time and extrapolate to first order:
    //   Earth(t−τ) ≈ Earth(t) + v_Earth·(−τ).
    // τ is at most a few hours, so the (second-derivative) error in Earth's
    // position is sub-milliarcsec on the apparent direction. The anchor is
    // shared across bodies by the memoizing provider, so a full chart resolves
    // Earth ~once per timestep instead of ~75 times. The Moon itself does not
    // use `earth_state` (it scales the rel-EMB vector directly), so its own
    // position is unaffected.
    //
    // Only the Moon's position is ever used on this path — its velocity has
    // no consumer — so its evaluation is `moon_position`, not
    // `compute_state`, skipping a velocity computation that would only be
    // discarded (`docs/audit/2026-08-29-perf-investigation.md` #5).
    if body == Body::Moon {
        let pos = moon_light_time(provider, jd, |t| provider.moon_position(t), moon_geocentric)?;
        return Ok(moon_geocentric(&pos));
    }
    let earth_anchor = provider.earth_state(jd)?;

    let mut tau = 0.0_f64;
    for _ in 0..LIGHT_TIME_MAX_ITER {
        let target_pos = retarded_geocentric(provider, body, jd, tau, earth_anchor)?;
        let tau_new = norm3(&target_pos) / crate::aberration::C_AU_PER_DAY;

        if (tau_new - tau).abs() < LIGHT_TIME_TOL {
            return Ok(target_pos);
        }
        tau = tau_new;
    }

    // Not converged in 10 iterations: evaluate once more at the last τ.
    retarded_geocentric(provider, body, jd, tau, earth_anchor)
}

/// Convergence threshold of the light-time iteration on `τ`, in days.
const LIGHT_TIME_TOL: f64 = 1e-12;

/// Iterations of the light-time loop before one final evaluation at the last
/// `τ`, converged or not.
const LIGHT_TIME_MAX_ITER: usize = 10;

/// `|v|`, written once so every light-time distance rounds the same way.
fn norm3(v: &[f64; 3]) -> f64 {
    libm::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}

/// Geocentric Moon vector from the provider's rel-EMB position, scaled by
/// `MOON_GEO_FACTOR`.
fn moon_geocentric(pos: &Position) -> [f64; 3] {
    [
        pos.x * MOON_GEO_FACTOR,
        pos.y * MOON_GEO_FACTOR,
        pos.z * MOON_GEO_FACTOR,
    ]
}

/// The Moon's light-time iteration: the provider evaluation (`eval`) at the
/// retarded instant `jd − τ` that the iteration converges on, `jd` in TT.
///
/// `geo` maps an evaluation to the geocentric vector whose length sets `τ`.
/// `eval` is `moon_position` for a position read and `compute_state` for
/// [`moon_apparent_position`]; either way the result is bit for bit the
/// evaluation the plain iteration (the non-Moon loop in
/// [`light_time_geocentric`], which is how the Moon was iterated too through
/// 9.4.1) returns, at less cost. Two savings, both exact:
///
/// 1. **No second evaluation at the same instant.** For the Moon `τ` is about
///    1.3 s, and its change between iterations is often above
///    [`LIGHT_TIME_TOL`] but far below the spacing of doubles near a modern
///    `jd` (about 4.7e-10 day). So the next retarded instant is frequently
///    the same double as the last one, and the plain loop evaluates it again
///    to learn what it already knows. Since the Moon's vector depends only on
///    the instant `jd − τ` (unlike other bodies', whose Earth extrapolation
///    uses `τ` itself), a repeat evaluation returns the identical vector,
///    hence the identical `τ`, and the plain loop then stops and returns it.
///    Returning the previous evaluation directly is that same answer.
/// 2. **A distance-only first step**, where the provider offers
///    [`EphemerisProvider::moon_distance`]. The plain loop's first
///    evaluation, at `jd` itself, is used only for its length. See
///    [`moon_distance_first_step`] for when the distance can stand in for it
///    and why the result is then unchanged; when it cannot, this falls back to
///    the plain loop's own start.
fn moon_light_time<S: Copy>(
    provider: &dyn EphemerisProvider,
    jd: f64,
    eval: impl Fn(f64) -> Result<S, ComputeError>,
    geo: impl Fn(&S) -> [f64; 3],
) -> Result<S, ComputeError> {
    let mut tau = 0.0_f64;
    // The last evaluation and the bit pattern of the instant it was made at.
    let mut prev: Option<(u64, S)> = None;
    let mut first_iter = 0;
    match moon_distance_first_step(provider, jd, &eval, &geo)? {
        Some(FirstStep::Converged(s)) => return Ok(s),
        Some(FirstStep::Continue {
            tau: tau2,
            instant_bits,
            eval: s,
        }) => {
            tau = tau2;
            prev = Some((instant_bits, s));
            first_iter = 2;
        }
        None => {}
    }

    for _ in first_iter..LIGHT_TIME_MAX_ITER {
        let t = jd - tau;
        if let Some((bits, s)) = prev {
            if bits == t.to_bits() {
                return Ok(s);
            }
        }
        let s = eval(t)?;
        let tau_new = norm3(&geo(&s)) / crate::aberration::C_AU_PER_DAY;
        if (tau_new - tau).abs() < LIGHT_TIME_TOL {
            return Ok(s);
        }
        tau = tau_new;
        prev = Some((t.to_bits(), s));
    }

    // Not converged in 10 iterations: evaluate once more at the last τ.
    let t = jd - tau;
    if let Some((bits, s)) = prev {
        if bits == t.to_bits() {
            return Ok(s);
        }
    }
    eval(t)
}

/// Outcome of [`moon_distance_first_step`]: the plain iteration's first two
/// steps, done with a distance-only first step.
enum FirstStep<S> {
    /// The plain loop would return this evaluation at its second step.
    Converged(S),
    /// The plain loop continues from `τ = tau` with this evaluation, made at
    /// the instant whose bit pattern is `instant_bits`, as its last.
    Continue {
        tau: f64,
        instant_bits: u64,
        eval: S,
    },
}

/// The plain iteration's first two steps, the first from
/// [`EphemerisProvider::moon_distance`] instead of a full evaluation at `jd`,
/// or `None` where that cannot be shown to give the plain loop's answer.
///
/// The plain loop's first step evaluates at `jd` and keeps only
/// `τ₁ = |g(jd)| / c`. The distance gives `τ₁'`, which differs from `τ₁` by
/// at most `E` (twice [`MOON_DISTANCE_REL_TOL`], relative). The unknown `τ₁`
/// therefore lies in `[τ₁' − E, τ₁' + E]`, and the substitution is exact
/// when every value in that interval leads the plain loop the same way:
///
/// - **Its first step must not stop.** The plain loop stops at step one when
///   `|τ₁ − 0| < LIGHT_TIME_TOL`; requiring `τ₁' − E ≥ LIGHT_TIME_TOL` rules
///   that out for every candidate.
/// - **The second instant must be one double.** Floating-point subtraction is
///   monotone, so if `jd − (τ₁' − E)` and `jd − (τ₁' + E)` round to the same
///   double, `jd − τ₁` rounds to it too. The second evaluation, and the
///   `τ₂` it gives, are then exactly the plain loop's.
/// - **The second stop test must agree.** The set of `x` with
///   `|τ₂ − x| < LIGHT_TIME_TOL` is an interval `2·LIGHT_TIME_TOL` wide. With
///   `2E` narrower than that, `[τ₁' − E, τ₁' + E]` cannot contain it, so the
///   test agreeing at both ends means it agrees everywhere between.
///
/// `E` is about 3e-17 day against a double spacing near 4.7e-10 day, so the
/// fallback (`None`, after one wasted evaluation in the last case) is
/// vanishingly rare.
fn moon_distance_first_step<S: Copy>(
    provider: &dyn EphemerisProvider,
    jd: f64,
    eval: &impl Fn(f64) -> Result<S, ComputeError>,
    geo: &impl Fn(&S) -> [f64; 3],
) -> Result<Option<FirstStep<S>>, ComputeError> {
    let Some(distance) = provider.moon_distance(jd) else {
        return Ok(None);
    };
    let tau1 = distance * MOON_GEO_FACTOR / crate::aberration::C_AU_PER_DAY;
    let e = 2.0 * MOON_DISTANCE_REL_TOL * tau1;
    if !tau1.is_finite() || tau1 - e < LIGHT_TIME_TOL || e >= LIGHT_TIME_TOL {
        return Ok(None);
    }
    let t1 = jd - tau1;
    if (jd - (tau1 - e)).to_bits() != t1.to_bits() || (jd - (tau1 + e)).to_bits() != t1.to_bits() {
        return Ok(None);
    }

    let s = eval(t1)?;
    let tau2 = norm3(&geo(&s)) / crate::aberration::C_AU_PER_DAY;
    let stops_lo = (tau2 - (tau1 - e)).abs() < LIGHT_TIME_TOL;
    let stops_hi = (tau2 - (tau1 + e)).abs() < LIGHT_TIME_TOL;
    Ok(match (stops_lo, stops_hi) {
        (true, true) => Some(FirstStep::Converged(s)),
        (false, false) => Some(FirstStep::Continue {
            tau: tau2,
            instant_bits: t1.to_bits(),
            eval: s,
        }),
        _ => None,
    })
}

/// Geocentric vector `[x, y, z]` (AU, J2000 mean equatorial) to a non-Moon
/// `body` at the retarded time `jd − tau`.
///
/// Subtracts Earth's position obtained by first-order extrapolation of
/// `earth_anchor` (Earth's state at the observation time `jd`) to `jd − tau`
/// — see [`light_time_geocentric`]. The Moon is iterated by
/// [`moon_light_time`] instead.
fn retarded_geocentric(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
    tau: f64,
    earth: StateVector,
) -> Result<[f64; 3], ComputeError> {
    let target_state = provider.compute_state(body, jd - tau)?;
    let (earth_pos, earth_vel) = (earth.position, earth.velocity);
    let dt = -tau;
    Ok([
        target_state.position.x - (earth_pos.x + earth_vel.x * dt),
        target_state.position.y - (earth_pos.y + earth_vel.y * dt),
        target_state.position.z - (earth_pos.z + earth_vel.z * dt),
    ])
}

/// Compute the apparent ecliptic position of a body at a given Julian Day.
///
/// Pipeline:
/// 1. Get Earth's barycentric position (EMB + Earth-relative-to-EMB)
/// 2. Apply light-time correction to get body's geometric position
/// 3. Compute geocentric position vector
/// 4. Apply precession (J2000 -> date)
/// 5. Apply nutation (mean -> true equatorial)
/// 6. Apply annual aberration
/// 7. Rotate from equatorial to ecliptic using true obliquity
/// 8. Extract longitude, latitude, distance
/// 9. Compute daily motion via numerical differentiation
///
/// Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 22-23, 33.
///
/// # Errors
/// Returns [`ComputeError`] if the ephemeris provider cannot compute the
/// required state vectors (e.g., body not available or date out of range).
pub fn apparent_position(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
) -> Result<ApparentPosition, ComputeError> {
    // Daily motion uses a ±0.5-day central difference, so three UT1 timestamps
    // are involved; each is converted to TT exactly once, here, and the TT
    // value is what flows into both the frame and the ephemeris query below.
    let jd_tt = delta_t::ut1_to_tt(jd);
    let jd_tt_before = delta_t::ut1_to_tt(jd - 0.5);
    let jd_tt_after = delta_t::ut1_to_tt(jd + 0.5);
    let frame = frame_for(jd_tt);
    let frame_before = frame_for(jd_tt_before);
    let frame_after = frame_for(jd_tt_after);
    apparent_position_with_frames(
        provider,
        body,
        jd_tt,
        &frame,
        jd_tt_before,
        &frame_before,
        jd_tt_after,
        &frame_after,
    )
}

/// Half-width, in days, of the central difference that measures how fast the
/// precession-nutation frame turns, for [`moon_apparent_position`].
///
/// The frame is a smooth, slow rotation whose whole contribution to a
/// longitude speed is about 4e-5 °/day, so a difference this narrow leaves no
/// visible error in it; the end-to-end check in `tests/moon_light_time.rs`
/// bounds the whole speed, frame term included, at 5e-7 °/day.
const FRAME_RATE_HALF_STEP: f64 = 0.05;

/// The Moon's apparent position with `longitude_speed` from the analytic
/// derivative of the ephemeris, not a central difference.
///
/// `jd` is **UT1**, as for [`apparent_position`]. The position fields are bit
/// for bit what [`ecliptic_position`]`(provider, Body::Moon, jd)` returns,
/// provided the provider's `compute_state(Body::Moon, ·)` position equals its
/// `moon_position` to the bit, as it does for
/// [`crate::analytical::AnalyticalProvider`] and for any provider that keeps
/// the trait's default `moon_position`.
///
/// # How the speed is formed
///
/// The geocentric vector is `g(t) = M(t − τ(t))`, the Moon at the retarded
/// instant, with the provider's own velocity `v` there (for
/// [`crate::analytical::AnalyticalProvider`], ELP/MPP02's analytic time
/// derivative). Differentiating the light-time equation `τ = |g| / c` gives
/// `τ̇ = (ĝ·v)(1 − τ̇) / c`, so `ġ = v · (1 − τ̇) = v / (1 + ĝ·v / c)`. That
/// velocity goes through the frame of date like the position does. The frame
/// itself turns (precession and nutation, a few times 1e-5 °/day in
/// longitude), and that rate is measured by a central difference of the cheap
/// frame alone over ±[`FRAME_RATE_HALF_STEP`], applied to the fixed vector `g`.
/// The longitude rate is then `(x·ẏ − y·ẋ) / (x² + y²)` in the ecliptic of
/// date.
///
/// The speed is per day of TT. [`apparent_position`]'s is per UT1 day; the
/// two differ by the rate of change of Delta T, below 1e-7 relative.
///
/// # Agreement with [`apparent_position`]
///
/// [`apparent_position`]'s `longitude_speed` is a ±0.5-day central difference,
/// whose truncation error for the Moon reaches about 1e-2 °/day. This read is
/// the derivative itself, so the two differ by that error. Measured by
/// `tests/moon_light_time.rs` over 1900 to 2100: worst 8.5e-3 °/day against
/// the one-day difference, with no bias, and worst 1.3e-7 °/day against a
/// Richardson-extrapolated difference of [`ecliptic_position_tt`] (the
/// tolerance stated there is 5e-7). Expect the 1e-2 shift when moving a
/// consumer from [`apparent_position`]'s speed to this one.
///
/// # Cost
///
/// One position-and-velocity evaluation at the retarded instant, plus, on a
/// provider with [`EphemerisProvider::moon_distance`], one distance-only
/// evaluation. [`apparent_position`] makes three light-time-corrected reads.
///
/// # Errors
/// Returns [`ComputeError`] if the provider cannot compute the Moon's state.
pub fn moon_apparent_position(
    provider: &dyn EphemerisProvider,
    jd: f64,
) -> Result<ApparentPosition, ComputeError> {
    let jd_tt = delta_t::ut1_to_tt(jd);
    let frame = frame_for(jd_tt);
    let state = moon_light_time(
        provider,
        jd_tt,
        |t| provider.compute_state(Body::Moon, t),
        |s: &StateVector| moon_geocentric(&s.position),
    )?;
    let g = moon_geocentric(&state.position);
    let v = [
        state.velocity.x * MOON_GEO_FACTOR,
        state.velocity.y * MOON_GEO_FACTOR,
        state.velocity.z * MOON_GEO_FACTOR,
    ];

    let ecl = ecliptic_of_date(g, &frame);
    let ecliptic = spherical_of(&ecl);

    // Light-time factor 1 − τ̇ = 1 / (1 + ĝ·v / c).
    let radial_rate = (g[0] * v[0] + g[1] * v[1] + g[2] * v[2]) / norm3(&g);
    let light_time_factor = 1.0 / (1.0 + radial_rate / crate::aberration::C_AU_PER_DAY);
    let ecl_rate = ecliptic_of_date(
        [
            v[0] * light_time_factor,
            v[1] * light_time_factor,
            v[2] * light_time_factor,
        ],
        &frame,
    );

    // The frame's own turning, applied to the fixed vector g.
    let ahead = ecliptic_of_date(g, &frame_for(jd_tt + FRAME_RATE_HALF_STEP));
    let behind = ecliptic_of_date(g, &frame_for(jd_tt - FRAME_RATE_HALF_STEP));
    let span = 2.0 * FRAME_RATE_HALF_STEP;
    let x_rate = ecl_rate.x + (ahead.x - behind.x) / span;
    let y_rate = ecl_rate.y + (ahead.y - behind.y) / span;

    let longitude_rate = (ecl.x * y_rate - ecl.y * x_rate) / (ecl.x * ecl.x + ecl.y * ecl.y);

    Ok(ApparentPosition {
        ecliptic,
        longitude_speed: longitude_rate * 180.0 / PI,
    })
}

/// Apparent ecliptic coordinates **without** daily motion.
///
/// Identical to [`apparent_position`]'s `.ecliptic`, but skips the two extra
/// half-day evaluations that the central-difference `longitude_speed` needs —
/// roughly a third of the work. Use this when only position is required (e.g.
/// muhurta/panchanga sweeps, raw ephemeris position queries); use
/// [`apparent_position`] when daily motion / retrograde state is needed.
///
/// `jd` is **UT1**; it is converted to TT exactly once, here. See
/// [`ecliptic_position_tt`] for the TT-addressed sibling used for
/// cross-implementation comparison.
///
/// # Errors
/// Returns [`ComputeError`] if the provider cannot compute the body's state.
pub fn ecliptic_position(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
) -> Result<EclipticCoords, ComputeError> {
    let jd_tt = delta_t::ut1_to_tt(jd);
    let frame = frame_for(jd_tt);
    compute_ecliptic_with_frame(provider, body, jd_tt, &frame)
}

/// TT-addressed sibling of [`ecliptic_position`]: `jd_tt` is **Terrestrial
/// Time**, not UT1, and this function applies **no Delta T conversion at
/// all** — the value is passed straight into the pipeline that
/// [`ecliptic_position`] itself reaches only after converting.
///
/// # Why this exists
/// Every other published entry point to the analytical path is UT1-addressed
/// and converts to TT internally via [`delta_t::ut1_to_tt`]. Outside the
/// modern era where Delta T is a measured quantity (roughly 1900-2100),
/// Delta T itself is an extrapolation, and two independent implementations
/// generally use two different extrapolations. Comparing this engine's
/// UT1-addressed output against another implementation's UT1-addressed
/// output therefore measures the *sum* of two entangled error sources — the
/// analytical theories' own truncation error, and the disagreement between
/// the two Delta T tables — with no way to apportion the residual between
/// them.
///
/// Addressing both sides in TT removes Delta T from the comparison entirely:
/// neither side performs a UT1->TT conversion, so the two independent
/// extrapolations never enter the picture, and the only remaining limit is
/// each reference ephemeris's own coverage span. This is the analytical-path
/// counterpart of [`crate::jpl::reader::SpkReader::compute_state`]'s
/// TDB-addressed raw entry point, which applies no Delta T conversion for the
/// same reason.
///
/// The tiny remaining TT/TDB distinction (the analytical theories are
/// formally TDB-referenced) is deliberately not corrected here: the periodic
/// TT-TDB difference peaks at roughly 1.7 ms (Fairhead & Bretagnon 1990), and
/// even for the Moon — the fastest-moving body this engine computes, at a
/// mean rate of about 13.176 deg/day — that maps to at most on the order of
/// 47,435 arcsec/day x (1.7e-3 s / 86,400 s/day) ~= 9.3e-4 arcsec, under one
/// milliarcsecond. Every other body moves slower, so its TT/TDB error is
/// smaller still.
///
/// # Why this is body positions only, not a chart
/// Houses and the ascendant depend on Earth's instantaneous rotation angle
/// (sidereal time), which is fundamentally a function of UT1, not TT or TDB —
/// there is no TT-addressed reformulation of "which point of the ecliptic is
/// on the eastern horizon right now" that means anything. A "TT-addressed
/// chart" is not a well-defined request, so this entry point is deliberately
/// restricted to body positions and is not exposed as an MCP tool. Do not
/// extend it to houses/ascendant/panchanga.
///
/// # Errors
/// Returns [`ComputeError`] if the provider cannot compute the body's state.
pub fn ecliptic_position_tt(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_tt: f64,
) -> Result<EclipticCoords, ComputeError> {
    let frame = frame_for(jd_tt);
    compute_ecliptic_with_frame(provider, body, jd_tt, &frame)
}

/// [`apparent_position`] with the three central-difference **TT** timestamps
/// and frames supplied by the caller, so a batch can build them once and
/// reuse them across all bodies rather than recomputing
/// nutation/precession/obliquity per body.
///
/// All three `jd_tt*` parameters must already be TT; no conversion happens
/// in this function.
#[allow(clippy::too_many_arguments)]
fn apparent_position_with_frames(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_tt: f64,
    frame: &CelestialFrame,
    jd_tt_before: f64,
    frame_before: &CelestialFrame,
    jd_tt_after: f64,
    frame_after: &CelestialFrame,
) -> Result<ApparentPosition, ComputeError> {
    let ecliptic = compute_ecliptic_with_frame(provider, body, jd_tt, frame)?;

    // Step 9: Daily motion via central difference (half-day step).
    let pos_before = compute_ecliptic_with_frame(provider, body, jd_tt_before, frame_before)?;
    let pos_after = compute_ecliptic_with_frame(provider, body, jd_tt_after, frame_after)?;

    // Longitude speed with wrap-around handling
    let mut speed_rad = pos_after.longitude - pos_before.longitude;
    if speed_rad > PI {
        speed_rad -= 2.0 * PI;
    } else if speed_rad < -PI {
        speed_rad += 2.0 * PI;
    }
    let speed_deg_per_day = speed_rad * 180.0 / PI;

    Ok(ApparentPosition {
        ecliptic,
        longitude_speed: speed_deg_per_day,
    })
}

/// Compute apparent ecliptic positions for many bodies at a single instant.
///
/// Batch entry point for chart computation. All bodies share one memoizing
/// provider ([`crate::cache::CachingProvider`]), so state lookups that recur
/// across bodies and across the daily-motion timesteps are evaluated once and
/// reused. The dominant saving is the Earth anchor
/// ([`EphemerisProvider::earth_state`]) that every planet's light-time
/// correction needs: at each shared timestamp it is now resolved once rather
/// than once per body.
///
/// Results are **bit-identical** to calling [`apparent_position`] per body —
/// only redundant work is removed. One entry is returned per input body, in
/// order; a per-body error (e.g. Pluto on the analytical provider) is returned
/// in place rather than aborting the whole chart.
#[cfg(feature = "std")]
pub fn apparent_positions(
    provider: &dyn EphemerisProvider,
    bodies: &[Body],
    jd: f64,
) -> Vec<(Body, Result<ApparentPosition, ComputeError>)> {
    let cached = crate::cache::CachingProvider::new(provider);
    // Time-only frames are body-independent — convert each of the three UT1
    // timestamps to TT exactly once and build its frame once, then reuse both
    // across every body.
    let jd_tt = delta_t::ut1_to_tt(jd);
    let jd_tt_before = delta_t::ut1_to_tt(jd - 0.5);
    let jd_tt_after = delta_t::ut1_to_tt(jd + 0.5);
    let frame = frame_for(jd_tt);
    let frame_before = frame_for(jd_tt_before);
    let frame_after = frame_for(jd_tt_after);
    bodies
        .iter()
        .map(|&body| {
            (
                body,
                apparent_position_with_frames(
                    &cached,
                    body,
                    jd_tt,
                    &frame,
                    jd_tt_before,
                    &frame_before,
                    jd_tt_after,
                    &frame_after,
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ecliptic_to_equatorial_deg`] must be the exact inverse of the rotation
    /// the production pipeline applies (`Matrix3::rotation_x(eps_true)`, true
    /// equator → ecliptic, in `compute_ecliptic_with_frame`). Build the
    /// ecliptic unit vector, undo that rotation with the same matrix type, and
    /// read RA/Dec off the equatorial vector: an independent formulation that
    /// catches a sign or axis slip in the closed form.
    #[test]
    fn ecliptic_to_equatorial_inverts_the_pipeline_rotation() {
        let mut worst = 0.0_f64;
        let mut n = 0;
        for eps in [0.0_f64, 23.439_291_1, 23.5, 24.2] {
            for lat_i in -8..=8 {
                let lat = f64::from(lat_i) * 10.0; // −80..80
                for lon_i in 0..72 {
                    let lon = f64::from(lon_i) * 5.0 + 0.37;
                    let ecl = Vector3::new(
                        libm::cos(lat.to_radians()) * libm::cos(lon.to_radians()),
                        libm::cos(lat.to_radians()) * libm::sin(lon.to_radians()),
                        libm::sin(lat.to_radians()),
                    );
                    let eq = Matrix3::rotation_x(eps.to_radians())
                        .transpose()
                        .apply(&ecl);
                    let ra_ref = libm::atan2(eq.y, eq.x).to_degrees().rem_euclid(360.0);
                    let dec_ref = libm::asin(eq.z).to_degrees();

                    let (ra, dec) = ecliptic_to_equatorial_deg(lon, lat, eps);
                    let dra = ((ra - ra_ref + 180.0).rem_euclid(360.0) - 180.0).abs();
                    worst = worst.max(dra).max((dec - dec_ref).abs());
                    assert!((declination_deg(lon, lat, eps) - dec).abs() < f64::EPSILON * 64.0);
                    n += 1;
                }
            }
        }
        assert_eq!(n, 4 * 17 * 72);
        assert!(worst < 1e-9, "worst RA/Dec disagreement {worst}°");
    }

    /// On the ecliptic at λ = 90° the declination is +ε, at λ = 270° it is −ε,
    /// and at the equinoxes it is 0 — the solstice/equinox definitions.
    #[test]
    fn declination_at_the_solstices_is_the_obliquity() {
        let eps = 23.439_291_1;
        assert!((declination_deg(90.0, 0.0, eps) - eps).abs() < 1e-12);
        assert!((declination_deg(270.0, 0.0, eps) + eps).abs() < 1e-12);
        assert!(declination_deg(0.0, 0.0, eps).abs() < 1e-12);
        assert!(declination_deg(180.0, 0.0, eps).abs() < 1e-12);
        let (ra, _) = ecliptic_to_equatorial_deg(90.0, 0.0, eps);
        assert!((ra - 90.0).abs() < 1e-12);
    }

    #[test]
    fn emrat_is_positive() {
        assert!(EMRAT > 80.0 && EMRAT < 82.0);
    }

    #[test]
    fn ecliptic_coords_fields_accessible() {
        let ec = EclipticCoords {
            longitude: 1.0,
            latitude: 0.5,
            distance: 1.0,
        };
        assert!((ec.longitude - 1.0).abs() < f64::EPSILON);
        assert!((ec.latitude - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn batch_matches_per_body_bit_for_bit() {
        use crate::analytical::AnalyticalProvider;

        let provider = AnalyticalProvider::new();
        let jd = 2_460_676.5;
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

        let batch = apparent_positions(&provider, &bodies, jd);
        assert_eq!(batch.len(), bodies.len());

        for (i, &body) in bodies.iter().enumerate() {
            let single = apparent_position(&provider, body, jd).expect("per-body should succeed");
            let (batch_body, batch_res) = &batch[i];
            assert_eq!(*batch_body, body);
            let b = batch_res.as_ref().expect("batch body should succeed");

            // The memoizing batch path must be bit-identical to per-body.
            assert_eq!(
                b.ecliptic.longitude.to_bits(),
                single.ecliptic.longitude.to_bits(),
                "{body:?} longitude differs"
            );
            assert_eq!(
                b.ecliptic.latitude.to_bits(),
                single.ecliptic.latitude.to_bits(),
                "{body:?} latitude differs"
            );
            assert_eq!(
                b.ecliptic.distance.to_bits(),
                single.ecliptic.distance.to_bits(),
                "{body:?} distance differs"
            );
            assert_eq!(
                b.longitude_speed.to_bits(),
                single.longitude_speed.to_bits(),
                "{body:?} speed differs"
            );
        }
    }

    #[test]
    fn moon_longitude_at_j2000_matches_jpl_horizons() {
        // Verifies that nutation is applied to the Moon through the full pipeline.
        // The `apparent_position` path calls `compute_ecliptic_with_frame` which
        // applies the combined precession × nutation matrix; this test confirms
        // the Moon is NOT exempt from that path.
        //
        // Oracle: JPL Horizons DE441 apparent ecliptic longitude of the Moon at
        //   JD 2451545.0 **UT** (Horizons' own `Date_________JDUT` column, per
        //   `scripts/generate_horizons_oracle.py`'s provenance record — not
        //   TT, despite this being J2000.0; do not feed this value into
        //   `ecliptic_position_tt`, which would introduce Delta T's ~32
        //   arcsec-scale error at this epoch), in the true ecliptic and
        //   equinox of date. `apparent_position` below is UT1-addressed and
        //   converts internally, which is why it lands within tolerance.
        //   Query: COMMAND='301', CENTER='500@399', EPHEM_TYPE='OBSERVER',
        //   QUANTITIES='31', REF_PLANE='ECLIPTIC', STEP='1d', START='2000-01-01',
        //   STOP='2000-01-02'.
        //   Result: apparent ecliptic longitude ≈ 223.3238° (JD 2451545.0 UT).
        //   Source: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/).
        //   Tolerance: 0.006° (≈ 22 arcsec) accounts for ELP/MPP02 vs. DE441
        //   lunar theory residual (~17 arcsec max for modern dates) plus rounding.
        use crate::analytical::AnalyticalProvider;
        let provider = AnalyticalProvider::new();
        let jd_j2000 = 2_451_545.0_f64;

        let pos = apparent_position(&provider, Body::Moon, jd_j2000)
            .expect("Moon position at J2000 should succeed");

        // EclipticCoords::longitude is in radians; convert to degrees for comparison.
        let got_deg = pos.ecliptic.longitude.to_degrees();
        let expected_deg = 223.3238_f64;
        let mut diff = (got_deg - expected_deg).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 0.006,
            "Moon longitude at J2000.0 should be ≈223.3238° (JPL Horizons DE441); \
             got {got_deg:.4}°, diff={diff:.4}°"
        );
    }

    /// Absolute-oracle test for [`ecliptic_position_tt`] itself, not merely
    /// for the UT1 sibling it is proved bit-identical to elsewhere in this
    /// file. Every other check on the TT entry point is relative to the UT1
    /// path (see `ecliptic_position_tt_matches_ut1_at_converted_time`), so a
    /// bug shared by both — e.g. in `compute_ecliptic_with_frame`, which
    /// both funnel through — would cancel out and go undetected. This test
    /// calls `ecliptic_position_tt` directly and checks it against the same
    /// external Horizons anchor the UT1 test above uses.
    ///
    /// The anchor (`moon_longitude_at_j2000_matches_jpl_horizons`, above) is
    /// pinned at JD 2451545.0 **UT**, not TT (see that test's comment, and
    /// item 4 of the adversarial-review report this test was added for).
    /// There is no independently-fetched Horizons value at a *TT* epoch in
    /// this repo, so this test reaches the same physical instant a different
    /// way: `delta_t::ut1_to_tt(2451545.0)` is the TT Julian Day of that same
    /// instant, using this engine's own (already-tested-elsewhere) Delta T
    /// table — the same table `apparent_position` itself uses at its UT1
    /// boundary. Calling `ecliptic_position_tt` at that TT instant and
    /// checking it against the Horizons UT longitude is therefore a genuine
    /// exercise of the TT code path against an external, non-self-referential
    /// number; it is not the tautological substitution the sibling
    /// bit-identity test performs, because it never calls the UT1-addressed
    /// `ecliptic_position` at all.
    #[test]
    fn ecliptic_position_tt_matches_jpl_horizons_at_j2000() {
        use crate::analytical::AnalyticalProvider;
        let provider = AnalyticalProvider::new();
        let jd_tt = delta_t::ut1_to_tt(2_451_545.0_f64);

        let pos = ecliptic_position_tt(&provider, Body::Moon, jd_tt)
            .expect("Moon position at J2000 (TT) should succeed");

        let got_deg = pos.longitude.to_degrees();
        let expected_deg = 223.3238_f64;
        let mut diff = (got_deg - expected_deg).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 0.006,
            "ecliptic_position_tt(Moon, ut1_to_tt(2451545.0)) should be ≈223.3238° \
             (JPL Horizons DE441, same physical instant as the UT1 anchor above); \
             got {got_deg:.4}°, diff={diff:.4}°"
        );
    }

    #[test]
    fn ecliptic_position_matches_apparent_position() {
        use crate::analytical::AnalyticalProvider;

        let provider = AnalyticalProvider::new();
        let jd = 2_460_676.5;
        for body in [Body::Sun, Body::Moon, Body::Mars, Body::Jupiter] {
            let full = apparent_position(&provider, body, jd).expect("apparent_position");
            let pos = ecliptic_position(&provider, body, jd).expect("ecliptic_position");
            // Position-only path must equal apparent_position's ecliptic exactly.
            assert_eq!(
                pos.longitude.to_bits(),
                full.ecliptic.longitude.to_bits(),
                "{body:?} longitude"
            );
            assert_eq!(
                pos.latitude.to_bits(),
                full.ecliptic.latitude.to_bits(),
                "{body:?} latitude"
            );
            assert_eq!(
                pos.distance.to_bits(),
                full.ecliptic.distance.to_bits(),
                "{body:?} distance"
            );
        }
    }

    /// Change-detector, not a correctness property: `ecliptic_position(x)`
    /// reduces by substitution to `ecliptic_position_tt(ut1_to_tt(x))`, so
    /// this holds for any implementation, correct or broken — it constrains
    /// no value. It stays useful for catching an accidental behavioural
    /// change between the two entry points (e.g. one gaining an extra
    /// conversion, or the two diverging on which bodies error). Correctness
    /// against an external oracle is [`moon_longitude_at_j2000_matches_jpl_horizons`]
    /// for the UT1 path and `ecliptic_position_tt_matches_jpl_horizons_at_j2000`
    /// for the TT path.
    #[test]
    fn ecliptic_position_tt_matches_ut1_at_converted_time() {
        use crate::analytical::AnalyticalProvider;

        let provider = AnalyticalProvider::new();
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
            Body::Pluto,
            Body::MeanNode,
            Body::TrueNode,
        ];
        // A spread of dates: deep past (well before the measured-Delta-T
        // era), the measured era itself, present, and far future.
        let dates_ut1 = [
            990_000.0,   // 1300 BCE-ish (deep past, Delta T heavily extrapolated)
            1_500_000.0, // antiquity
            2_000_000.0, // ~ 763 BCE
            2_305_447.5, // 1600-01-01
            2_400_000.5, // 1858-11-17 (MJD epoch)
            2_451_545.0, // J2000.0 (measured era)
            2_460_676.5, // 2024-11-24 (measured era)
            2_500_000.5, // 2132 CE
            2_600_000.5, // 2405 CE (far future, extrapolated Delta T)
        ];

        for jd_ut1 in dates_ut1 {
            let jd_tt = delta_t::ut1_to_tt(jd_ut1);
            for &body in &bodies {
                let ut1_result = ecliptic_position(&provider, body, jd_ut1);
                let tt_result = ecliptic_position_tt(&provider, body, jd_tt);
                match (ut1_result, tt_result) {
                    (Ok(ut1), Ok(tt)) => {
                        assert_eq!(
                            ut1.longitude.to_bits(),
                            tt.longitude.to_bits(),
                            "{body:?} at jd_ut1={jd_ut1}: longitude differs"
                        );
                        assert_eq!(
                            ut1.latitude.to_bits(),
                            tt.latitude.to_bits(),
                            "{body:?} at jd_ut1={jd_ut1}: latitude differs"
                        );
                        assert_eq!(
                            ut1.distance.to_bits(),
                            tt.distance.to_bits(),
                            "{body:?} at jd_ut1={jd_ut1}: distance differs"
                        );
                    }
                    (Err(_), Err(_)) => {
                        // Both paths must fail together (e.g. Pluto outside
                        // the analytical provider's supported range).
                    }
                    other => panic!(
                        "{body:?} at jd_ut1={jd_ut1}: UT1 and TT paths disagreed on success: {other:?}"
                    ),
                }
            }
        }
    }

    /// The TT path must apply *no* Delta T conversion: feeding it a UT1 value
    /// directly (as if it were TT) must disagree with the true UT1 path by an
    /// amount consistent with Delta T at that epoch — not silently agree,
    /// which would indicate a conversion was applied somewhere on the "TT"
    /// path after all.
    #[test]
    fn ecliptic_position_tt_applies_no_delta_t_conversion() {
        use crate::analytical::AnalyticalProvider;

        let provider = AnalyticalProvider::new();
        let jd_ut1 = 2_305_447.5; // 1600-01-01: far enough from J2000 that
        // Delta T (~120 s here) produces an easily-measurable longitude shift
        // for the Moon (fast mover), well above float noise.
        let delta_t_days = delta_t::ut1_to_tt(jd_ut1) - jd_ut1;
        assert!(
            delta_t_days.abs() > 1e-4,
            "test epoch's Delta T ({delta_t_days} days) is too small to distinguish \
             from float noise; pick a date further from the measured era"
        );

        let body = Body::Moon;
        let true_ut1 = ecliptic_position(&provider, body, jd_ut1).expect("UT1 path");
        // Misuse the TT entry point by handing it the raw UT1 value.
        let misfed_tt = ecliptic_position_tt(&provider, body, jd_ut1).expect("TT path");

        // If the TT path silently converted, these would be bit-identical
        // to true_ut1. They must not be: the TT path took `jd_ut1` literally
        // as TT, which is `delta_t_days` away from the correct TT instant.
        assert_ne!(
            true_ut1.longitude.to_bits(),
            misfed_tt.longitude.to_bits(),
            "TT path appears to have applied a Delta T conversion internally"
        );

        // Use the Moon's own instantaneous longitude speed at this epoch
        // (from `apparent_position`, not a hardcoded mean-motion constant) to
        // predict the shift `delta_t_days` should produce. This is accurate
        // to well under 1% over a single delta_t_days-sized step, so the
        // tolerance below can be tight: a few percent catches a sign-flipped
        // conversion (~200% off), a halved or doubled Delta T (~50%/100%
        // off), or a wrong-body/wrong-epoch mixup, while still tolerating the
        // second-order error from using the speed at `jd_ut1` rather than the
        // true mean speed over the (jd_ut1, jd_ut1 + delta_t_days) interval.
        let speed_deg_per_day = apparent_position(&provider, body, jd_ut1)
            .expect("UT1 path speed")
            .longitude_speed;
        let predicted_shift_deg = speed_deg_per_day.abs() * delta_t_days.abs();
        let mut observed_shift_deg =
            (true_ut1.longitude.to_degrees() - misfed_tt.longitude.to_degrees()).abs();
        if observed_shift_deg > 180.0 {
            observed_shift_deg = 360.0 - observed_shift_deg;
        }
        assert!(
            observed_shift_deg > predicted_shift_deg * 0.95
                && observed_shift_deg < predicted_shift_deg * 1.05,
            "observed shift {observed_shift_deg}deg is not within 5% of the \
             Delta-T-predicted shift {predicted_shift_deg}deg (from the Moon's own \
             longitude_speed at this epoch) — this band is tight enough to catch a \
             sign-flipped conversion, or a halved/doubled Delta T, not just a no-op"
        );
    }
}

/// The Moon's light-time iteration against the plain iteration it replaced.
///
/// [`moon_light_time`] claims to return, bit for bit, what the plain loop
/// returns. These tests hold a verbatim copy of that loop and of the steps
/// that followed it as they stood at 9.4.1, so the claim is checked against
/// the old arithmetic itself and not against a refactor of it. The copy is
/// architecture-independent, unlike a pinned table of output bits.
#[cfg(all(test, feature = "std"))]
mod moon_light_time_tests {
    use super::*;
    use crate::analytical::AnalyticalProvider;
    use crate::jpl::Velocity;

    /// 9.4.1's Moon read: the plain light-time loop, then steps 2-5, verbatim.
    fn plain_ecliptic_moon(
        provider: &dyn EphemerisProvider,
        jd_tt: f64,
        frame: &CelestialFrame,
    ) -> Result<EclipticCoords, ComputeError> {
        let retarded = |tau: f64| -> Result<[f64; 3], ComputeError> {
            let moon_pos = provider.moon_position(jd_tt - tau)?;
            Ok([
                moon_pos.x * MOON_GEO_FACTOR,
                moon_pos.y * MOON_GEO_FACTOR,
                moon_pos.z * MOON_GEO_FACTOR,
            ])
        };
        let geo = 'lt: {
            let mut tau = 0.0_f64;
            for _ in 0..10 {
                let target_pos = retarded(tau)?;
                let r = libm::sqrt(
                    target_pos[0] * target_pos[0]
                        + target_pos[1] * target_pos[1]
                        + target_pos[2] * target_pos[2],
                );
                let tau_new = r / crate::aberration::C_AU_PER_DAY;

                if (tau_new - tau).abs() < 1e-12 {
                    break 'lt target_pos;
                }
                tau = tau_new;
            }
            retarded(tau)?
        };

        let geo_vec = Vector3::new(geo[0], geo[1], geo[2]);
        let true_eq = frame.combined.apply(&geo_vec);
        let ecl_vec = Matrix3::rotation_x(frame.eps_true).apply(&true_eq);
        let distance =
            libm::sqrt(ecl_vec.x * ecl_vec.x + ecl_vec.y * ecl_vec.y + ecl_vec.z * ecl_vec.z);
        let mut longitude = libm::atan2(ecl_vec.y, ecl_vec.x);
        if longitude < 0.0 {
            longitude += 2.0 * PI;
        }
        let latitude = libm::asin(ecl_vec.z / distance);
        Ok(EclipticCoords {
            longitude,
            latitude,
            distance,
        })
    }

    /// 9.4.1's `apparent_position(Moon)`: three plain reads and the ±0.5-day
    /// central difference, verbatim.
    fn plain_apparent_moon(provider: &dyn EphemerisProvider, jd: f64) -> ApparentPosition {
        let jd_tt = delta_t::ut1_to_tt(jd);
        let jd_tt_before = delta_t::ut1_to_tt(jd - 0.5);
        let jd_tt_after = delta_t::ut1_to_tt(jd + 0.5);
        let ecliptic = plain_ecliptic_moon(provider, jd_tt, &frame_for(jd_tt)).unwrap();
        let pos_before =
            plain_ecliptic_moon(provider, jd_tt_before, &frame_for(jd_tt_before)).unwrap();
        let pos_after =
            plain_ecliptic_moon(provider, jd_tt_after, &frame_for(jd_tt_after)).unwrap();
        let mut speed_rad = pos_after.longitude - pos_before.longitude;
        if speed_rad > PI {
            speed_rad -= 2.0 * PI;
        } else if speed_rad < -PI {
            speed_rad += 2.0 * PI;
        }
        ApparentPosition {
            ecliptic,
            longitude_speed: speed_rad * 180.0 / PI,
        }
    }

    /// `n` UT1 instants spread over 1900 to 2100, the fractional day stepped
    /// by the golden ratio so no two share a time of day.
    fn instants(n: u32) -> impl Iterator<Item = f64> {
        let start = 2_415_020.5;
        let span = 73_048.0;
        (0..n).map(move |i| {
            let frac = (f64::from(i) * 0.618_033_988_749_894_9).fract();
            start + span * f64::from(i) / f64::from(n) + frac
        })
    }

    fn assert_same_bits(got: &EclipticCoords, want: &EclipticCoords, what: &str) {
        assert_eq!(
            got.longitude.to_bits(),
            want.longitude.to_bits(),
            "{what}: longitude"
        );
        assert_eq!(
            got.latitude.to_bits(),
            want.latitude.to_bits(),
            "{what}: latitude"
        );
        assert_eq!(
            got.distance.to_bits(),
            want.distance.to_bits(),
            "{what}: distance"
        );
    }

    /// Forwards every method except `moon_distance`, so it takes the trait
    /// default and the provider offers no cheaper distance.
    struct WithoutDistance<'a>(&'a dyn EphemerisProvider);

    impl EphemerisProvider for WithoutDistance<'_> {
        fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
            self.0.compute_state(body, jd)
        }
        fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
            self.0.earth_state(jd)
        }
        fn moon_position(&self, jd: f64) -> Result<Position, ComputeError> {
            self.0.moon_position(jd)
        }
        fn time_range(&self) -> (f64, f64) {
            self.0.time_range()
        }
    }

    #[test]
    fn ecliptic_position_moon_is_bit_identical_to_the_plain_iteration() {
        let analytical = AnalyticalProvider::new();
        let without = WithoutDistance(&analytical);
        let mut n = 0;
        for jd in instants(4_000) {
            let jd_tt = delta_t::ut1_to_tt(jd);
            let want = plain_ecliptic_moon(&analytical, jd_tt, &frame_for(jd_tt)).unwrap();
            let got = ecliptic_position(&analytical, Body::Moon, jd).unwrap();
            assert_same_bits(&got, &want, &format!("jd {jd}"));
            let got = ecliptic_position(&without, Body::Moon, jd).unwrap();
            assert_same_bits(&got, &want, &format!("jd {jd}, no distance hook"));
            n += 1;
        }
        assert_eq!(n, 4_000);
    }

    #[test]
    fn apparent_positions_moon_is_bit_identical_to_the_plain_iteration() {
        let analytical = AnalyticalProvider::new();
        for jd in instants(1_000) {
            let want = plain_apparent_moon(&analytical, jd);
            let got = apparent_positions(&analytical, &[Body::Moon], jd)
                .remove(0)
                .1
                .unwrap();
            assert_same_bits(&got.ecliptic, &want.ecliptic, &format!("jd {jd}"));
            assert_eq!(
                got.longitude_speed.to_bits(),
                want.longitude_speed.to_bits(),
                "jd {jd}: speed"
            );
        }
    }

    /// The new read's position is [`ecliptic_position`]'s, to the bit.
    #[test]
    fn moon_apparent_position_has_ecliptic_positions_bits() {
        let analytical = AnalyticalProvider::new();
        for jd in instants(1_000) {
            let got = moon_apparent_position(&analytical, jd).unwrap();
            let want = ecliptic_position(&analytical, Body::Moon, jd).unwrap();
            assert_same_bits(&got.ecliptic, &want, &format!("jd {jd}"));
        }
    }

    /// A Moon on a fixed direction whose distance grows linearly in time, so a
    /// test can place `τ` and its change wherever a guard needs it. Its
    /// distance hook is exact.
    struct LinearMoon {
        r0: f64,
        rate: f64,
        epoch: f64,
    }

    impl LinearMoon {
        fn rel_emb(&self, jd: f64) -> f64 {
            (self.r0 + self.rate * (jd - self.epoch)) / MOON_GEO_FACTOR
        }
    }

    impl EphemerisProvider for LinearMoon {
        fn compute_state(&self, _body: Body, jd: f64) -> Result<StateVector, ComputeError> {
            Ok(StateVector {
                position: Position {
                    x: self.rel_emb(jd),
                    y: 0.0,
                    z: 0.0,
                },
                velocity: Velocity {
                    x: self.rate / MOON_GEO_FACTOR,
                    y: 0.0,
                    z: 0.0,
                },
            })
        }
        fn moon_distance(&self, jd: f64) -> Option<f64> {
            Some(self.rel_emb(jd))
        }
        fn time_range(&self) -> (f64, f64) {
            (f64::MIN, f64::MAX)
        }
    }

    fn plain_geo(provider: &dyn EphemerisProvider, jd_tt: f64) -> [f64; 3] {
        let frame = frame_for(jd_tt);
        let e = plain_ecliptic_moon(provider, jd_tt, &frame).unwrap();
        [e.longitude, e.latitude, e.distance]
    }

    fn new_geo(provider: &dyn EphemerisProvider, jd_tt: f64) -> [f64; 3] {
        let e = ecliptic_position_tt(provider, Body::Moon, jd_tt).unwrap();
        [e.longitude, e.latitude, e.distance]
    }

    fn first_step_taken(provider: &dyn EphemerisProvider, jd: f64) -> bool {
        let eval = |t: f64| provider.moon_position(t);
        moon_distance_first_step(provider, jd, &eval, &moon_geocentric)
            .unwrap()
            .is_some()
    }

    /// Each of the three guards in [`moon_distance_first_step`] declines when
    /// its condition fails, and the read then still matches the plain loop.
    #[test]
    fn distance_first_step_falls_back_where_it_cannot_prove_the_answer() {
        let c = crate::aberration::C_AU_PER_DAY;
        let jd = 2_460_000.5;

        // Baseline: an ordinary Moon-like case takes the distance step.
        let ordinary = LinearMoon {
            r0: 0.002_57,
            rate: 1e-6,
            epoch: jd,
        };
        assert!(first_step_taken(&ordinary, jd));
        assert_eq!(new_geo(&ordinary, jd), plain_geo(&ordinary, jd));

        // Guard 1: τ₁ below the stop threshold, so the plain loop stops at
        // its first step.
        let near = LinearMoon {
            r0: 1e-12 * c / 2.0,
            rate: 0.0,
            epoch: jd,
        };
        assert!(!first_step_taken(&near, jd));
        assert_eq!(new_geo(&near, jd), plain_geo(&near, jd));

        // Guard 2: near jd = 0.001 doubles are 2e-19 day apart, far finer than
        // the distance's error bound, so the second instant is not one double.
        let fine_jd = 0.001;
        let fine = LinearMoon {
            r0: 0.002_57,
            rate: 1e-6,
            epoch: fine_jd,
        };
        assert!(!first_step_taken(&fine, fine_jd));
        assert_eq!(new_geo(&fine, fine_jd), plain_geo(&fine, fine_jd));

        // Guard 3: choose the distance rate so |τ₂ − τ₁| lands on the stop
        // threshold, inside the distance's error bound.
        let r0 = 0.002_57;
        let tau1 = r0 / c;
        let delta = jd - (jd - tau1);
        let boundary = LinearMoon {
            r0,
            rate: 1e-12 * c / delta,
            epoch: jd,
        };
        assert!(
            !first_step_taken(&boundary, jd),
            "the stop test should be ambiguous across the distance's error bound"
        );
        assert_eq!(new_geo(&boundary, jd), plain_geo(&boundary, jd));
    }
}
