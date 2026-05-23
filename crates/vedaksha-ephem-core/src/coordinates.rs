// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
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
use crate::jpl::{EphemerisProvider, Position, Velocity};
use crate::nutation;
use crate::obliquity;
use crate::precession;
use vedaksha_math::matrix::{Matrix3, Vector3};

/// Earth-Moon mass ratio (DE440/441 value).
const EMRAT: f64 = 81.300_568_94;

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

/// Compute the barycentric position and velocity of Earth at a given Julian Day.
///
/// `Earth = EMB + Moon_relative_to_EMB * (-1 / (1 + EMRAT))`
///
/// The SPK file stores EMB (target=3, center=0) and Moon (target=301, center=3).
fn earth_state(
    provider: &dyn EphemerisProvider,
    jd: f64,
) -> Result<(Position, Velocity), ComputeError> {
    let emb = provider.compute_state(Body::EarthMoonBarycenter, jd)?;
    let moon = provider.compute_state(Body::Moon, jd)?;

    let factor = 1.0 / (1.0 + EMRAT);

    let pos = Position {
        x: emb.position.x - moon.position.x * factor,
        y: emb.position.y - moon.position.y * factor,
        z: emb.position.z - moon.position.z * factor,
    };
    let vel = Velocity {
        x: emb.velocity.x - moon.velocity.x * factor,
        y: emb.velocity.y - moon.velocity.y * factor,
        z: emb.velocity.z - moon.velocity.z * factor,
    };
    Ok((pos, vel))
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
    }
}

fn compute_ecliptic_with_frame(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_ut: f64,
    frame: &CelestialFrame,
) -> Result<EclipticCoords, ComputeError> {
    let jd = delta_t::ut1_to_tt(jd_ut);

    // Step 1: planetary-aberration-form light-time iteration.
    //
    // We compute the geocentric vector with target AND observer at the
    // retarded time t-τ. For solar-system bodies this single step provides
    // the apparent direction in the observer's instantaneous rest frame —
    // adding stellar aberration on top would double-count.
    let geo = light_time_geocentric(provider, body, jd)?;

    // Steps 2-3: apply the precomputed nutation·precession frame (the
    // provider returns J2000 mean-equatorial vectors; SPK ICRF agrees to
    // <25 mas — negligible here). `frame` must correspond to this `jd`.
    let geo_vec = Vector3::new(geo[0], geo[1], geo[2]);
    let true_eq = frame.combined.apply(&geo_vec);

    // Step 4: rotate true equatorial of date → ecliptic of date
    let ecl_vec = Matrix3::rotation_x(frame.eps_true).apply(&true_eq);

    // Step 5: extract spherical coordinates
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
    // expensive ELP/MPP02 lunar series (via `earth_state`) at every iteration
    // and for every body. Instead, anchor Earth's state once at the
    // observation time and extrapolate to first order:
    //   Earth(t−τ) ≈ Earth(t) + v_Earth·(−τ).
    // τ is at most a few hours, so the (second-derivative) error in Earth's
    // position is sub-milliarcsec on the apparent direction. The anchor's
    // `compute_state(Moon, jd)` is shared across bodies by the memoizing
    // provider, so a full chart evaluates the lunar series ~once per timestep
    // instead of ~75 times. The Moon itself does not use `earth_state` (it
    // scales the rel-EMB vector directly), so its own position is unaffected.
    let earth_anchor = if body == Body::Moon {
        None
    } else {
        Some(earth_state(provider, jd)?)
    };

    let mut tau = 0.0_f64;
    for _ in 0..10 {
        let target_pos = retarded_geocentric(provider, body, jd, tau, earth_anchor)?;
        let r = libm::sqrt(
            target_pos[0] * target_pos[0]
                + target_pos[1] * target_pos[1]
                + target_pos[2] * target_pos[2],
        );
        let tau_new = r / crate::aberration::C_AU_PER_DAY;

        if (tau_new - tau).abs() < 1e-12 {
            return Ok(target_pos);
        }
        tau = tau_new;
    }

    // Not converged in 10 iterations: evaluate once more at the last τ.
    retarded_geocentric(provider, body, jd, tau, earth_anchor)
}

/// Geocentric vector `[x, y, z]` (AU, J2000 mean equatorial) to `body` at the
/// retarded time `jd − tau`.
///
/// For the Moon, scales the rel-EMB vector by `MOON_GEO_FACTOR`. For other
/// bodies, subtracts Earth's position obtained by first-order extrapolation of
/// `earth_anchor` (Earth's `(position, velocity)` at the observation time `jd`)
/// to `jd − tau` — see [`light_time_geocentric`].
fn retarded_geocentric(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
    tau: f64,
    earth_anchor: Option<(Position, Velocity)>,
) -> Result<[f64; 3], ComputeError> {
    let target_state = provider.compute_state(body, jd - tau)?;
    if body == Body::Moon {
        return Ok([
            target_state.position.x * MOON_GEO_FACTOR,
            target_state.position.y * MOON_GEO_FACTOR,
            target_state.position.z * MOON_GEO_FACTOR,
        ]);
    }
    let (earth_pos, earth_vel) = earth_anchor.expect("non-Moon body has an Earth anchor");
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
    // Daily motion uses a ±0.5-day central difference, so three timestamps are
    // involved; build each one's time-only frame once.
    let frame = frame_for(delta_t::ut1_to_tt(jd));
    let frame_before = frame_for(delta_t::ut1_to_tt(jd - 0.5));
    let frame_after = frame_for(delta_t::ut1_to_tt(jd + 0.5));
    apparent_position_with_frames(provider, body, jd, &frame, &frame_before, &frame_after)
}

/// Apparent ecliptic coordinates **without** daily motion.
///
/// Identical to [`apparent_position`]'s `.ecliptic`, but skips the two extra
/// half-day evaluations that the central-difference `longitude_speed` needs —
/// roughly a third of the work. Use this when only position is required (e.g.
/// muhurta/panchanga sweeps, raw ephemeris position queries); use
/// [`apparent_position`] when daily motion / retrograde state is needed.
///
/// # Errors
/// Returns [`ComputeError`] if the provider cannot compute the body's state.
pub fn ecliptic_position(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
) -> Result<EclipticCoords, ComputeError> {
    let frame = frame_for(delta_t::ut1_to_tt(jd));
    compute_ecliptic_with_frame(provider, body, jd, &frame)
}

/// [`apparent_position`] with the three central-difference frames supplied by
/// the caller, so a batch can build them once and reuse them across all bodies
/// rather than recomputing nutation/precession/obliquity per body.
fn apparent_position_with_frames(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd: f64,
    frame: &CelestialFrame,
    frame_before: &CelestialFrame,
    frame_after: &CelestialFrame,
) -> Result<ApparentPosition, ComputeError> {
    let ecliptic = compute_ecliptic_with_frame(provider, body, jd, frame)?;

    // Step 9: Daily motion via central difference (half-day step).
    let pos_before = compute_ecliptic_with_frame(provider, body, jd - 0.5, frame_before)?;
    let pos_after = compute_ecliptic_with_frame(provider, body, jd + 0.5, frame_after)?;

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
/// reused. The dominant saving is the ELP/MPP02 lunar series pulled into
/// [`earth_state`] during every planet's light-time correction: at each shared
/// timestamp it is now evaluated once rather than once per body.
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
    // Time-only frames are body-independent — build the three central-difference
    // frames once and reuse them across every body.
    let frame = frame_for(delta_t::ut1_to_tt(jd));
    let frame_before = frame_for(delta_t::ut1_to_tt(jd - 0.5));
    let frame_after = frame_for(delta_t::ut1_to_tt(jd + 0.5));
    bodies
        .iter()
        .map(|&body| {
            (
                body,
                apparent_position_with_frames(
                    &cached,
                    body,
                    jd,
                    &frame,
                    &frame_before,
                    &frame_after,
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
