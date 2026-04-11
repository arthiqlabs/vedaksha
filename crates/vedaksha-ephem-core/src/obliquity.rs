// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Obliquity of the ecliptic.
//!
//! Source: Capitaine, Wallace & Chapront (2003), A&A 412, pp. 567-586.

use crate::julian;

/// Mean obliquity of the ecliptic (IAU 2006), in radians.
///
/// Source: Capitaine, Wallace & Chapront (2003), eq. 37.
#[must_use]
pub fn mean_obliquity(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    // Polynomial in arcseconds
    let eps0_arcsec = 84_381.406
        + t * (-46.836_769
            + t * (-0.000_183_1
                + t * (0.002_003_40 + t * (-0.000_000_576 + t * (-0.000_000_043_4)))));
    // Convert arcseconds to radians
    eps0_arcsec * core::f64::consts::PI / (180.0 * 3600.0)
}

/// True obliquity of the ecliptic, in radians.
///
/// True obliquity = mean obliquity + nutation in obliquity (deps).
#[must_use]
pub fn true_obliquity(jd: f64, deps: f64) -> f64 {
    mean_obliquity(jd) + deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::J2000;

    /// Tolerance for degree-level comparisons.
    const DEG_EPS: f64 = 0.001 * core::f64::consts::PI / 180.0; // 0.001 degrees in radians

    #[test]
    fn mean_obliquity_at_j2000() {
        // At J2000 (T=0), eps0 = 84381.406 arcsec ≈ 23.4392911 degrees
        let eps = mean_obliquity(J2000);
        let expected_deg = 23.439_291_1_f64;
        let eps_deg = eps * 180.0 / core::f64::consts::PI;
        assert!(
            (eps_deg - expected_deg).abs() < 0.001,
            "mean_obliquity at J2000: expected {expected_deg}°, got {eps_deg}°"
        );
        // Also check in radians
        let expected_rad = expected_deg * core::f64::consts::PI / 180.0;
        assert!(
            (eps - expected_rad).abs() < DEG_EPS,
            "mean_obliquity at J2000 (rad): expected {expected_rad}, got {eps}"
        );
    }

    #[test]
    fn mean_obliquity_one_century_later() {
        // One century after J2000 (T=1), the linear term is -46.836769 arcsec/century
        // so obliquity should decrease by approximately 46.8 arcseconds
        let eps_j2000 = mean_obliquity(J2000);
        let jd_j2100 = J2000 + 36_525.0;
        let eps_j2100 = mean_obliquity(jd_j2100);

        let diff_arcsec = (eps_j2000 - eps_j2100) * (180.0 * 3600.0) / core::f64::consts::PI;
        // Should be approximately 46.8 arcseconds (within 1 arcsecond tolerance for higher-order terms)
        assert!(
            (diff_arcsec - 46.836_769_f64).abs() < 1.0,
            "obliquity change over one century: expected ~46.8 arcsec, got {diff_arcsec} arcsec"
        );
        // And the sign: obliquity should be smaller one century later
        assert!(
            eps_j2100 < eps_j2000,
            "obliquity should decrease from J2000 to J2100"
        );
    }

    #[test]
    fn true_obliquity_zero_deps_equals_mean() {
        let jd = J2000 + 1000.0;
        let mean = mean_obliquity(jd);
        let true_obl = true_obliquity(jd, 0.0);
        assert!(
            (true_obl - mean).abs() < f64::EPSILON * 10.0,
            "true_obliquity with deps=0 should equal mean_obliquity: mean={mean}, true={true_obl}"
        );
    }

    #[test]
    fn true_obliquity_nonzero_deps_adds_correction() {
        let jd = J2000 + 5000.0;
        let deps = 1.0e-5_f64; // small nutation correction in radians (~2 arcsec)
        let mean = mean_obliquity(jd);
        let true_obl = true_obliquity(jd, deps);
        assert!(
            (true_obl - mean - deps).abs() < f64::EPSILON * 10.0,
            "true_obliquity should add deps correction: expected {}, got {}",
            mean + deps,
            true_obl
        );
    }
}
