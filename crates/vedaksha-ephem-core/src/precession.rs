// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! IAU 2006 precession matrix.
//!
//! Uses the Fukushima-Williams 4-angle parameterization.
//!
//! Source: Capitaine, Wallace & Chapront (2003), A&A 412, pp. 567-586.

use crate::julian;
use crate::obliquity;
use vedaksha_math::matrix::Matrix3;

/// Precession matrix (IAU 2006), transforming coordinates from ICRS (J2000)
/// to the mean equator and equinox of date.
///
/// Uses the Fukushima-Williams parameterization:
/// ```text
/// P = Rz(γ̄) · Rx(−φ̄) · Rz(−ψ̄) · Rx(εA)
/// ```
///
/// Source: Capitaine, Wallace & Chapront (2003), eq. 40.
#[allow(clippy::similar_names)]
pub fn precession_matrix(jd: f64) -> Matrix3 {
    let t = julian::centuries_from_j2000(jd);

    let arcsec_to_rad = core::f64::consts::PI / (180.0 * 3600.0);

    let gamma_bar = (-0.052_928
        + t * (10.556_378
            + t * (0.493_204_4
                + t * (-0.000_312_38 + t * (-0.000_002_788 + t * 0.000_000_026_0)))))
        * arcsec_to_rad;

    let phi_bar = (84_381.412_819
        + t * (-46.811_016
            + t * (0.051_126_8
                + t * (0.000_532_89 + t * (-0.000_000_440 + t * (-0.000_000_017_6))))))
        * arcsec_to_rad;

    let psi_bar = (-0.041_775
        + t * (5_038.481_484
            + t * (1.558_417_5
                + t * (-0.000_185_22 + t * (-0.000_026_452 + t * (-0.000_000_014_8))))))
        * arcsec_to_rad;

    let eps_a = obliquity::mean_obliquity(jd);

    // P = Rz(gamma_bar) * Rx(-phi_bar) * Rz(-psi_bar) * Rx(eps_A)
    Matrix3::rotation_z(gamma_bar)
        .multiply(&Matrix3::rotation_x(-phi_bar))
        .multiply(&Matrix3::rotation_z(-psi_bar))
        .multiply(&Matrix3::rotation_x(eps_a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::J2000;
    use vedaksha_math::matrix::{Matrix3, Vector3};

    /// Tolerance for near-identity check at J2000.
    const IDENTITY_EPS: f64 = 1e-6;

    /// Tight tolerance for orthogonality check.
    const ORTHO_EPS: f64 = 1e-12;

    fn assert_mat_near_identity(m: &Matrix3, eps: f64) {
        let identity = Matrix3::IDENTITY;
        for r in 0..3 {
            for c in 0..3 {
                assert!(
                    (m.data[r][c] - identity.data[r][c]).abs() < eps,
                    "matrix[{r}][{c}] = {} should be close to identity element {}",
                    m.data[r][c],
                    identity.data[r][c]
                );
            }
        }
    }

    #[test]
    fn precession_matrix_near_identity_at_j2000() {
        // At T=0, all angles are effectively zero (the constant terms cancel
        // because the Fukushima-Williams matrix reduces to identity at J2000)
        let p = precession_matrix(J2000);
        assert_mat_near_identity(&p, IDENTITY_EPS);
    }

    #[test]
    fn precession_matrix_is_orthogonal() {
        // For any epoch, P * P^T should be the identity matrix
        let jd = J2000 + 10_000.0; // ~27 years after J2000
        let p = precession_matrix(jd);
        let pt = p.transpose();
        let ppt = p.multiply(&pt);
        for r in 0..3 {
            for c in 0..3 {
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!(
                    (ppt.data[r][c] - expected).abs() < ORTHO_EPS,
                    "P*P^T[{r}][{c}] = {} should be {expected}",
                    ppt.data[r][c]
                );
            }
        }
    }

    #[test]
    fn precession_matrix_differs_from_identity_at_j2100() {
        // One century after J2000, precession should have moved coordinates noticeably
        let jd_j2100 = J2000 + 36_525.0;
        let p = precession_matrix(jd_j2100);

        // The off-diagonal elements should be non-negligible (psi_bar ~ 5038 arcsec/century ~ 0.024 rad)
        let max_off_diag = p.data[0][1]
            .abs()
            .max(p.data[1][0].abs())
            .max(p.data[0][2].abs())
            .max(p.data[2][0].abs());
        assert!(
            max_off_diag > 1e-3,
            "P at J2100 should differ measurably from identity; max off-diag = {max_off_diag}"
        );
    }

    #[test]
    fn precession_preserves_vector_length() {
        // Applying a rotation matrix must preserve vector norm
        let jd = J2000 + 18_262.5; // ~50 years after J2000
        let p = precession_matrix(jd);

        let v = Vector3::new(0.8, 0.5, 0.33);
        let original_len = v.length();
        let rotated = p.apply(&v);
        let rotated_len = rotated.length();

        assert!(
            (rotated_len - original_len).abs() < 1e-12,
            "precession should preserve vector length: original={original_len}, rotated={rotated_len}"
        );
    }
}
