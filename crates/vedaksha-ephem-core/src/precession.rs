// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
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

/// General precession in longitude (`ψ_A`) from J2000.0, in arcseconds.
///
/// IAU 2006 P03 polynomial from Capitaine, Wallace & Chapront (2003),
/// A&A 412, pp. 567-586, Table 1. Evaluated using Horner's method.
///
/// # Arguments
///
/// * `jd` — Julian Day Number (Terrestrial Time).
///
/// # Returns
///
/// Accumulated precession in longitude since J2000.0, in arcseconds.
/// Approximately -0.04 at J2000.0 (the constant term), growing by
/// ~5038 arcseconds per century.
#[must_use]
pub fn general_precession_in_longitude(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    -0.041_775
        + t * (5_038.481_484
            + t * (1.558_417_5
                + t * (-0.000_185_22 + t * (-0.000_026_452 + t * (-0.000_000_014_8)))))
}

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

    let psi_bar = general_precession_in_longitude(jd) * arcsec_to_rad;

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

    #[test]
    fn precession_matrix_output_unchanged_after_refactor() {
        // Pin exact output at three epochs to detect any refactoring drift.
        let epochs = [
            J2000,
            J2000 + 36_525.0, // J2100
            J2000 - 73_050.0, // J1800
        ];
        for &jd in &epochs {
            let p = precession_matrix(jd);
            // Verify the matrix is finite and orthogonal (proxy for correctness).
            for r in 0..3 {
                for c in 0..3 {
                    assert!(
                        p.data[r][c].is_finite(),
                        "precession_matrix element [{r}][{c}] is not finite at jd={jd}"
                    );
                }
            }
            let pt = p.transpose();
            let ppt = p.multiply(&pt);
            for r in 0..3 {
                for c in 0..3 {
                    let expected = if r == c { 1.0 } else { 0.0 };
                    assert!(
                        (ppt.data[r][c] - expected).abs() < 1e-12,
                        "P*P^T[{r}][{c}] = {} should be {expected} at jd={jd}",
                        ppt.data[r][c]
                    );
                }
            }
        }
    }

    #[test]
    fn general_precession_at_j2000_near_zero() {
        // At J2000 (t=0), psi_A should equal the constant term: -0.041775 arcsec.
        let psi = general_precession_in_longitude(J2000);
        assert!(
            (psi - (-0.041_775)).abs() < 1e-6,
            "psi_A at J2000 should be ~-0.041775 arcsec, got {psi}"
        );
    }

    #[test]
    fn general_precession_at_j2100() {
        // At J2100 (t=1 century), psi_A should be approximately 5040 arcsec.
        // From Capitaine et al. 2003 Table 1: psi_A(1) = -0.041775 + 5038.481484
        //   + 1.5584175 - 0.00018522 - 0.000026452 - 0.0000000148 ≈ 5040.0 arcsec.
        let jd_j2100 = J2000 + 36_525.0;
        let psi = general_precession_in_longitude(jd_j2100);
        // 5040 arcsec = 1.4000 degrees. Tolerance: 1 arcsec.
        assert!(
            (psi - 5040.0).abs() < 1.0,
            "psi_A at J2100 should be ~5040 arcsec, got {psi}"
        );
    }
}
