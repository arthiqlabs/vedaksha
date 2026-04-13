// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Annual aberration correction.
//!
//! Computes the first-order annual aberration shift due to the finite speed
//! of light and the Earth's orbital velocity.
//!
//! Source: Meeus, *Astronomical Algorithms*, 2nd ed., Chapter 23.

/// Speed of light in AU/day.
///
/// Derived from 299 792.458 km/s × 86 400 s/day ÷ 149 597 870.700 km/AU.
pub const C_AU_PER_DAY: f64 = 173.144_632_674;

/// Apply first-order annual aberration in Cartesian coordinates.
///
/// Returns the aberration displacement vector (in AU) to be added to the
/// geometric position vector to obtain the apparent position.
///
/// The first-order formula (Meeus eq. 23.2 in vector form) is:
/// ```text
/// Δpᵢ = (vᵢ / c) · r  −  pᵢ · (p⋅v) / (c · r)
/// ```
/// where `r = |p|` is the distance in AU and `c` is the speed of light.
///
/// # Arguments
/// * `geo_pos`   — geocentric position of the body in AU (ICRS)
/// * `earth_vel` — Earth's barycentric velocity in AU/day (ICRS)
///
/// # Returns
/// Aberration displacement `[Δx, Δy, Δz]` in AU. Add to `geo_pos` for the
/// apparent position.  Returns `[0.0; 3]` when `r < 1e-20`.
#[must_use]
pub fn aberration_correction(geo_pos: &[f64; 3], earth_vel: &[f64; 3]) -> [f64; 3] {
    let r = libm::sqrt(geo_pos[0] * geo_pos[0] + geo_pos[1] * geo_pos[1] + geo_pos[2] * geo_pos[2]);
    if r < 1e-20 {
        return [0.0; 3];
    }

    let p_dot_v = geo_pos[0] * earth_vel[0] + geo_pos[1] * earth_vel[1] + geo_pos[2] * earth_vel[2];

    [
        earth_vel[0] / C_AU_PER_DAY * r - geo_pos[0] * p_dot_v / (C_AU_PER_DAY * r),
        earth_vel[1] / C_AU_PER_DAY * r - geo_pos[1] * p_dot_v / (C_AU_PER_DAY * r),
        earth_vel[2] / C_AU_PER_DAY * r - geo_pos[2] * p_dot_v / (C_AU_PER_DAY * r),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Radians per arcsecond.
    const ARCSEC_RAD: f64 = core::f64::consts::PI / (180.0 * 3600.0);

    /// Zero Earth velocity produces zero correction.
    #[test]
    fn zero_velocity_zero_correction() {
        let pos = [1.0_f64, 0.0, 0.0];
        let vel = [0.0_f64; 3];
        let corr = aberration_correction(&pos, &vel);
        assert_eq!(corr, [0.0; 3]);
    }

    /// A typical Earth orbital velocity (~0.017 AU/day) produces a correction
    /// of roughly 20 arcseconds in magnitude.
    ///
    /// The constant of aberration κ = v/c ≈ 0.017 / 173.14 ≈ 20.5 arcsec.
    #[test]
    fn typical_velocity_gives_about_20_arcsec() {
        // Body at 1 AU along +X; Earth moving at typical orbital speed along +Y.
        let pos = [1.0_f64, 0.0, 0.0];
        let vel = [0.0_f64, 0.017_202, 0.0]; // ~Earth mean orbital speed AU/day

        let corr = aberration_correction(&pos, &vel);

        // |Δp| ≈ (v/c) * r  since p⋅v = 0 in this configuration.
        let mag = libm::sqrt(corr[0] * corr[0] + corr[1] * corr[1] + corr[2] * corr[2]);
        // At r=1 AU the displacement equals v/c AU; convert to arcsec
        let mag_arcsec = mag / (1.0 * ARCSEC_RAD);

        // Expect 18–23 arcseconds
        assert!(
            (18.0..=23.0).contains(&mag_arcsec),
            "expected ~20 arcsec, got {mag_arcsec:.3} arcsec"
        );
    }

    /// Correction is roughly along the velocity vector when the body is far
    /// off the velocity direction (p⋅v ≈ 0).
    #[test]
    fn correction_direction_along_velocity() {
        // Body at 1 AU along +X; velocity entirely along +Y.
        let pos = [1.0_f64, 0.0, 0.0];
        let vel = [0.0_f64, 0.017_202, 0.0];

        let corr = aberration_correction(&pos, &vel);

        // The leading term vᵢ/c * r dominates; the Y component should be positive.
        assert!(
            corr[1] > 0.0,
            "Y correction should be positive (velocity is in +Y): {corr:?}"
        );
        // The X component should be zero (p⋅v = 0 and vₓ = 0).
        assert!(corr[0].abs() < 1e-14, "X correction should be ~0: {corr:?}");
    }
}
