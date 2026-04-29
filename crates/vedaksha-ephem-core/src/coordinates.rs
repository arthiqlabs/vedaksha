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
fn compute_ecliptic(
    provider: &dyn EphemerisProvider,
    body: Body,
    jd_ut: f64,
) -> Result<EclipticCoords, ComputeError> {
    let jd = delta_t::ut1_to_tt(jd_ut);

    // Step 1: planetary-aberration-form light-time iteration.
    //
    // We compute the geocentric vector with target AND observer at the
    // retarded time t-τ. For solar-system bodies this single step provides
    // the apparent direction in the observer's instantaneous rest frame —
    // adding stellar aberration on top would double-count.
    let geo = light_time_geocentric(provider, body, jd)?;

    // Step 2: precession (J2000 → mean equator of date)
    let prec = precession::precession_matrix(jd);

    // Step 3: nutation (mean → true equator of date)
    let (dpsi, deps) = nutation::nutation(jd);
    let eps_a = obliquity::mean_obliquity(jd);
    let eps_true = obliquity::true_obliquity(jd, deps);

    let nut_matrix = Matrix3::rotation_x(-eps_a - deps)
        .multiply(&Matrix3::rotation_z(-dpsi))
        .multiply(&Matrix3::rotation_x(eps_a));

    // Combined: true equatorial of date = N · P · v_J2000_eq.
    // The provider returns vectors already in the J2000 mean equator frame
    // (analytical: ecliptic→equatorial via fixed J2000 obliquity; SPK:
    // ICRF, which agrees with J2000 mean to <25 mas — small enough to
    // ignore at this level of accuracy).
    let combined = nut_matrix.multiply(&prec);

    let geo_vec = Vector3::new(geo[0], geo[1], geo[2]);
    let true_eq = combined.apply(&geo_vec);

    // Step 4: rotate true equatorial of date → ecliptic of date
    let ecl_vec = Matrix3::rotation_x(eps_true).apply(&true_eq);

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
    let mut tau = 0.0_f64;
    for _ in 0..10 {
        let target_state = provider.compute_state(body, jd - tau)?;
        let target_pos = if body == Body::Moon {
            // Convert Moon-rel-EMB to Moon-geocentric.
            [
                target_state.position.x * MOON_GEO_FACTOR,
                target_state.position.y * MOON_GEO_FACTOR,
                target_state.position.z * MOON_GEO_FACTOR,
            ]
        } else {
            // Body is barycentric; subtract Earth barycentric position
            // at the SAME retarded time t-τ (planetary aberration form).
            let (earth_at_ret, _) = earth_state(provider, jd - tau)?;
            [
                target_state.position.x - earth_at_ret.x,
                target_state.position.y - earth_at_ret.y,
                target_state.position.z - earth_at_ret.z,
            ]
        };

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

    // Final iteration if not converged: recompute at last tau.
    let target_state = provider.compute_state(body, jd - tau)?;
    let target_pos = if body == Body::Moon {
        [
            target_state.position.x * MOON_GEO_FACTOR,
            target_state.position.y * MOON_GEO_FACTOR,
            target_state.position.z * MOON_GEO_FACTOR,
        ]
    } else {
        let (earth_at_ret, _) = earth_state(provider, jd - tau)?;
        [
            target_state.position.x - earth_at_ret.x,
            target_state.position.y - earth_at_ret.y,
            target_state.position.z - earth_at_ret.z,
        ]
    };
    Ok(target_pos)
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
