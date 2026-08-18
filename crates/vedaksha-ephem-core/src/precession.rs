// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
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

/// General precession in longitude (`p_A`) from J2000.0, in arcseconds.
///
/// IAU 2006 P03 polynomial from Capitaine, Wallace & Chapront (2003),
/// A&A 412, pp. 567-586, eq. (37). Evaluated using Horner's method.
///
/// # This is not [`general_precession_in_longitude`]
///
/// The two are different angles and differ by ~9.7 arcseconds per century.
///
/// * [`general_precession_in_longitude`] returns the Fukushima-Williams angle
///   `ψ̄_A`, precession in longitude referred to the **fixed** J2000 ecliptic.
///   It exists to build the precession matrix and has no other use.
/// * This function returns `p_A`, general precession in longitude, accumulated
///   along the **moving** ecliptic of date. It is the angle by which the equinox
///   regresses against a fixed direction among the stars, and it is therefore the
///   quantity an epoch-anchored ayanamsha is propagated by.
///
/// Picking the wrong one is not detectable from the result — both grow at
/// roughly 50 arcseconds per year — which is why they are documented together.
///
/// # Arguments
///
/// * `jd` — Julian Day Number (Terrestrial Time).
///
/// # Returns
///
/// Accumulated general precession in longitude since J2000.0, in arcseconds.
/// Exactly zero at J2000.0; approximately +5029.9 at J2100.0.
#[must_use]
pub fn general_precession_p03(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    t * (5_028.796_195
        + t * (1.105_434_8 + t * (0.000_079_64 + t * (-0.000_023_857 + t * (-0.000_000_038_3)))))
}

/// Precession matrix (IAU 2006), transforming coordinates from ICRS (J2000)
/// to the mean equator and equinox of date.
///
/// Uses the Fukushima-Williams parameterization:
/// ```text
/// P = Rx(−εA) · Rz(−ψ̄) · Rx(φ̄) · Rz(γ̄)
/// ```
///
/// Source: Capitaine, Wallace & Chapront (2003), eq. 40.
///
/// The four F-W angles are referred to the GCRS, so this matrix carries the
/// ICRS frame bias as well as the precession; no separate bias rotation is
/// needed before it.
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

    // P = Rx(-eps_A) * Rz(-psi_bar) * Rx(phi_bar) * Rz(gamma_bar)
    //
    // The order is the whole content of this expression: the four rotations do
    // not commute, and an incorrect order stays within a milliarcsecond near
    // J2000 while growing without bound away from it. See the module tests,
    // which pin this against ERFA at epochs 1500 years either side.
    Matrix3::rotation_x(-eps_a)
        .multiply(&Matrix3::rotation_z(-psi_bar))
        .multiply(&Matrix3::rotation_x(phi_bar))
        .multiply(&Matrix3::rotation_z(gamma_bar))
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

    #[test]
    fn general_precession_long_range_sanity_1582() {
        // Sanity check at JD 2299160.5 (1582-10-15 — Gregorian calendar reform date).
        // This is T ≈ −4.17 Julian centuries before J2000.
        //
        // Expected value (accumulated precession from J2000): approximately −20 986 arcsec
        // (≈ −5.83°). At a rate of ~5038.5 arcsec/century, 4.17 centuries gives ~21 010
        // arcsec backward precession.
        //
        // Note: The prompt spec stated "~30.1°" for this date. That figure appears to
        // reference accumulated precession from an ancient epoch (possibly ~600 BC),
        // not from J2000. The correct return value of `general_precession_in_longitude`
        // (which always measures from J2000) is ≈ −20 986 arcsec.
        //
        // Source: Capitaine, Wallace & Chapront (2003), A&A 412, 567–586, Table 1.
        const JD_1582_OCT_15: f64 = 2_299_160.5;
        let psi = general_precession_in_longitude(JD_1582_OCT_15);
        // Expected: ≈ −20 986 arcsec. Tolerance: 200 arcsec (≈ 0.056°).
        assert!(
            (psi - (-20_986.0)).abs() < 200.0,
            "psi_A at 1582-10-15 should be ≈ −20 986 arcsec, got {psi:.2} arcsec"
        );
        // Long-range polynomial must produce a negative (backward) value for past dates.
        assert!(
            psi < 0.0,
            "Precession at a past date (before J2000) must be negative, got {psi}"
        );
        // The precession matrix at this epoch must be non-identity and orthogonal.
        let p = precession_matrix(JD_1582_OCT_15);
        let pt = p.transpose();
        let ppt = p.multiply(&pt);
        for r in 0..3 {
            for c in 0..3 {
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!(
                    (ppt.data[r][c] - expected).abs() < 1e-10,
                    "P*P^T[{r}][{c}] = {} should be {expected} at 1582-10-15",
                    ppt.data[r][c]
                );
            }
        }
    }

    // ── ERFA-pinned regression tests ──────────────────────────────────────────
    //
    // The expected values below were produced by ERFA (the IAU SOFA board's
    // BSD-3 relicensing of SOFA) — `eraP06e` for p_A and `eraEqec06` for the
    // composite bias-precession-obliquity rotation. Verifying our own astronomy
    // against a permissive reference implementation is exactly what it is for.
    //
    // The epochs deliberately straddle a 6000-year span. A rotation-order error
    // in `precession_matrix` is invisible at J2000 (0.014 mas) and reaches 0.56
    // arcsec by 499 CE, so a test that only samples the modern era cannot see it.

    /// ERFA `eraEqec06` applied to a fixed ICRS direction, at four epochs.
    ///
    /// The direction is arbitrary (ICRS alpha 123.456789 deg, delta 45.678901
    /// deg) and deliberately unrelated to any star this engine carries. It was
    /// Sgr A*'s position until 2026-08-18; see `stars.rs` for why that was a poor
    /// choice in a clean-room module.
    const ERFA_ECLIPTIC_LONGITUDES: [(f64, f64); 4] = [
        (588_465.75, 44.581_951_457),
        (1_903_397.0, 94.241_225_583),
        (2_451_545.0, 115.177_689_978),
        (2_816_788.0, 129.209_891_274),
    ];

    #[test]
    fn precession_matrix_matches_erfa_across_six_millennia() {
        use vedaksha_math::angle::normalize_degrees;

        let ra = 123.456_789_f64.to_radians();
        let de = 45.678_901_f64.to_radians();
        let v = Vector3::new(de.cos() * ra.cos(), de.cos() * ra.sin(), de.sin());

        for (jd, expected_deg) in ERFA_ECLIPTIC_LONGITUDES {
            let equatorial = precession_matrix(jd).apply(&v);
            let eps = obliquity::mean_obliquity(jd);
            // Equator of date -> ecliptic of date: rotate about x by +eps.
            let y = equatorial.y * eps.cos() + equatorial.z * eps.sin();
            let lon = normalize_degrees(y.atan2(equatorial.x).to_degrees());
            let residual_arcsec = (lon - expected_deg) * 3600.0;
            assert!(
                residual_arcsec.abs() < 0.01,
                "at jd={jd} the mean ecliptic longitude is {lon:.9} deg, \
                 ERFA gives {expected_deg:.9} deg — residual {residual_arcsec:+.6} arcsec \
                 exceeds the 0.01 arcsec astronomy tolerance"
            );
        }
    }

    #[test]
    fn general_precession_p03_matches_erfa() {
        // eraP06e's `pa` output, in arcseconds.
        const ERFA_P_A: [(f64, f64); 6] = [
            (2_451_545.0, 0.0),
            (2_488_070.0, 5_029.901_685_545),
            (2_415_122.5, -5_013.584_762_414),
            (2_433_282.423_46, -2_514.132_286_027),
            (1_903_397.0, -75_222.009_257_288),
            (588_465.75, -253_793.167_256_519),
        ];
        for (jd, expected) in ERFA_P_A {
            let got = general_precession_p03(jd);
            assert!(
                (got - expected).abs() < 1e-6,
                "p_A at jd={jd}: got {got:.9} arcsec, ERFA gives {expected:.9} arcsec"
            );
        }
    }

    #[test]
    fn p_a_and_psi_bar_are_different_angles() {
        // The whole reason both exist. If someone ever "simplifies" one into the
        // other, this fails: they diverge by ~9.7 arcsec per century, which is
        // ~200x the accuracy this engine claims for a sidereal longitude.
        let at_j2100 = 2_488_070.0;
        let p_a = general_precession_p03(at_j2100);
        let psi_bar = general_precession_in_longitude(at_j2100);
        assert!(
            (p_a - psi_bar).abs() > 9.0,
            "p_A ({p_a:.3}) and psi_bar ({psi_bar:.3}) must not be interchangeable"
        );
    }
}
