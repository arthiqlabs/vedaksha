// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
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

use crate::aberration;
use crate::bodies::Body;
use crate::delta_t;
use crate::error::ComputeError;
use crate::jpl::{EphemerisProvider, Position, Velocity};
use crate::light_time;
use crate::nutation;
use crate::obliquity;
use crate::precession;
use vedaksha_math::matrix::{Matrix3, Vector3};

/// Earth-Moon mass ratio (DE440/441 value).
const EMRAT: f64 = 81.300_568_94;

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
/// 1. Get Earth's barycentric position
/// 2. Apply light-time correction to get body's geometric position
/// 3. Compute geocentric position vector
/// 4. Apply precession (J2000 -> date)
/// 5. Apply nutation (mean -> true equatorial)
/// 6. Apply annual aberration
/// 7. Rotate from equatorial to ecliptic using true obliquity
/// 8. Extract longitude, latitude, distance
fn compute_ecliptic(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_ut: f64,
) -> Result<EclipticCoords, ComputeError> {
    // Convert UT → TT for ephemeris lookups (JPL ephemeris uses TDB ≈ TT).
    // Precession/nutation use TT as well. The Delta T correction is critical
    // for the Moon which moves ~0.5 arcsec/second.
    let jd = delta_t::ut1_to_tt(jd_ut);

    // Step 1: Earth's barycentric state
    let (earth_pos, earth_vel) = earth_state(provider, jd)?;

    // Step 2 & 3: Light-time correction and geocentric position
    //
    // The Moon is special: its SPK segment is relative to EMB (center=3),
    // while all other bodies are barycentric (center=0).
    let geo = if body == Body::Moon {
        // Earth relative to EMB = -Moon_rel_EMB / (1 + EMRAT)
        let moon_now = provider.compute_state(Body::Moon, jd)?;
        let earth_rel_emb = Position {
            x: -moon_now.position.x / (1.0 + EMRAT),
            y: -moon_now.position.y / (1.0 + EMRAT),
            z: -moon_now.position.z / (1.0 + EMRAT),
        };
        let (moon_lt, _tau) =
            light_time::light_time_correction(provider, Body::Moon, &earth_rel_emb, jd)?;
        // Moon geocentric = Moon_rel_EMB (exact identity, see spec derivation)
        [moon_lt.position.x, moon_lt.position.y, moon_lt.position.z]
    } else {
        let (body_state, _tau) = light_time::light_time_correction(provider, body, &earth_pos, jd)?;
        // Body is barycentric; subtract Earth barycentric position
        [
            body_state.position.x - earth_pos.x,
            body_state.position.y - earth_pos.y,
            body_state.position.z - earth_pos.z,
        ]
    };

    // Step 4: Precession matrix (J2000 -> mean equator of date)
    let prec = precession::precession_matrix(jd);

    // Step 5: Nutation matrix (mean -> true equatorial of date)
    // N = Rx(-eps_A - deps) * Rz(-dpsi) * Rx(eps_A)
    let (dpsi, deps) = nutation::nutation(jd);
    let eps_a = obliquity::mean_obliquity(jd);
    let eps_true = obliquity::true_obliquity(jd, deps);

    let nut_matrix = Matrix3::rotation_x(-eps_a - deps)
        .multiply(&Matrix3::rotation_z(-dpsi))
        .multiply(&Matrix3::rotation_x(eps_a));

    // Frame bias: ICRS -> mean J2000 (~23 mas correction)
    // Source: IERS Conventions (2010), eq. 5.4
    let bias = nutation::frame_bias_matrix();

    // Combined transformation: true equatorial of date = N * P * B * ICRS_vector
    let combined = nut_matrix.multiply(&prec).multiply(&bias);

    let geo_vec = Vector3::new(geo[0], geo[1], geo[2]);
    let equatorial = combined.apply(&geo_vec);

    // Step 6: Annual aberration
    // Transform Earth velocity to equatorial of date
    let vel_vec = Vector3::new(earth_vel.x, earth_vel.y, earth_vel.z);
    let vel_equatorial = combined.apply(&vel_vec);

    let eq_arr = [equatorial.x, equatorial.y, equatorial.z];
    let vel_eq_arr = [vel_equatorial.x, vel_equatorial.y, vel_equatorial.z];
    let aber = aberration::aberration_correction(&eq_arr, &vel_eq_arr);

    let apparent_eq = [
        eq_arr[0] + aber[0],
        eq_arr[1] + aber[1],
        eq_arr[2] + aber[2],
    ];

    // Step 7: Rotate from true equatorial to ecliptic using true obliquity
    // ecliptic = Rx(eps_true) * equatorial
    let rot_ecl = Matrix3::rotation_x(eps_true);
    let ecl_vec = rot_ecl.apply(&Vector3::new(
        apparent_eq[0],
        apparent_eq[1],
        apparent_eq[2],
    ));

    // Step 8: Extract spherical coordinates
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
    let ecliptic = compute_ecliptic(provider, body, jd)?;

    // Step 9: Daily motion via central difference (half-day step)
    let dt = 0.5;
    let pos_before = compute_ecliptic(provider, body, jd - dt)?;
    let pos_after = compute_ecliptic(provider, body, jd + dt)?;

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
}
