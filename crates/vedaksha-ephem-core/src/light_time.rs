// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Iterative light-time correction.
//!
//! Corrects for the finite travel time of light from the body to the observer
//! by iterating to find the retarded position (where the body was when the
//! observed light was emitted).
//!
//! Source: Meeus, *Astronomical Algorithms*, 2nd ed., Chapter 33.

use crate::aberration::C_AU_PER_DAY;
use crate::bodies::Body;
use crate::error::ComputeError;
use crate::jpl::{EphemerisProvider, Position, StateVector};

/// Maximum iterations for light-time convergence.
const MAX_ITERATIONS: usize = 10;

/// Convergence threshold in days (~86 nanoseconds — well below a milliarcsecond).
const CONVERGENCE_THRESHOLD: f64 = 1e-12;

/// Apply iterative light-time correction.
///
/// Starting from an initial guess of τ = 0, this function iterates:
/// 1. Compute the body's position at `jd − τ`.
/// 2. Compute the new light-time `τ′ = |body − observer| / c`.
/// 3. Repeat until |τ′ − τ| < 1e-12 days (convergence threshold).
///
/// Convergence is typically reached in 2–3 iterations for solar-system
/// bodies.
///
/// # Arguments
/// * `provider`   — ephemeris data source
/// * `body`       — body for which the correction is computed
/// * `earth_pos`  — observer (Earth) position in AU at `jd`
/// * `jd`         — Julian Day of observation (TDB)
///
/// # Returns
/// A tuple of:
/// * `StateVector` — body state at the retarded time `jd − τ`
/// * `f64`         — light-time `τ` in days
///
/// # Errors
/// Propagates any [`ComputeError`] returned by the ephemeris provider.
///
/// Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 33.
pub fn light_time_correction(
    provider: &dyn EphemerisProvider,
    body: Body,
    earth_pos: &Position,
    jd: f64,
) -> Result<(StateVector, f64), ComputeError> {
    let mut tau = 0.0_f64;

    for _ in 0..MAX_ITERATIONS {
        let body_state = provider.compute_state(body, jd - tau)?;

        let dx = body_state.position.x - earth_pos.x;
        let dy = body_state.position.y - earth_pos.y;
        let dz = body_state.position.z - earth_pos.z;
        let distance = libm::sqrt(dx * dx + dy * dy + dz * dz);

        let tau_new = distance / C_AU_PER_DAY;

        if (tau_new - tau).abs() < CONVERGENCE_THRESHOLD {
            return Ok((body_state, tau_new));
        }
        tau = tau_new;
    }

    // Return last computed state even if not fully converged (MAX_ITERATIONS reached).
    let body_state = provider.compute_state(body, jd - tau)?;
    Ok((body_state, tau))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the speed of light constant.
    ///
    /// 299 792.458 km/s × 86 400 s/day / 149 597 870.700 km/AU ≈ 173.1446 AU/day.
    #[test]
    fn speed_of_light_au_per_day() {
        let c_km_s = 299_792.458_f64;
        let seconds_per_day = 86_400.0_f64;
        let km_per_au = 149_597_870.700_f64;
        let expected = c_km_s * seconds_per_day / km_per_au;
        assert!(
            (C_AU_PER_DAY - expected).abs() < 1e-4,
            "C_AU_PER_DAY={C_AU_PER_DAY} should be ≈{expected:.6}"
        );
    }

    /// Light-time for a body at 1 AU from the observer.
    ///
    /// τ = 1 AU / (173.144 AU/day) ≈ 0.005_775 days ≈ 499 seconds.
    #[test]
    fn light_time_one_au() {
        let distance_au = 1.0_f64;
        let tau = distance_au / C_AU_PER_DAY;
        // Between 0.00575 and 0.00580 days (≈498–501 seconds)
        assert!(
            (0.005_75..=0.005_80).contains(&tau),
            "light-time at 1 AU: expected ~0.00578 days, got {tau:.6} days"
        );
        // Also check in seconds
        let tau_seconds = tau * 86_400.0;
        assert!(
            (498.0..=501.0).contains(&tau_seconds),
            "light-time at 1 AU: expected ~499 s, got {tau_seconds:.1} s"
        );
    }

    /// Light-time for the Moon (mean distance ≈ 0.002_570 AU).
    ///
    /// τ ≈ 0.002_570 / 173.144 ≈ 1.484e-5 days ≈ 1.28 seconds.
    #[test]
    fn light_time_moon_distance() {
        let moon_au = 384_400.0_f64 / 149_597_870.700_f64; // mean distance in AU
        let tau = moon_au / C_AU_PER_DAY;
        // Expect roughly 1.28 seconds = 1.28/86400 ≈ 1.48e-5 days
        assert!(
            (1.0e-5..=2.0e-5).contains(&tau),
            "light-time for Moon: expected ~1.48e-5 days, got {tau:.2e} days"
        );
    }
}
