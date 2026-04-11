// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Mean and True North Node of the Moon.
//!
//! Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 47.

use crate::julian;

/// Compute the mean longitude of the ascending node of the Moon (Rahu).
///
/// Source: Meeus Ch. 47, eq. 47.7.
#[must_use]
pub fn mean_node(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    let omega = 125.044_547_9
        + t * (-1_934.136_289_1
            + t * (0.002_075_4 + t * (1.0 / 467_441.0 + t * (-1.0 / 60_616_000.0))));
    vedaksha_math::angle::normalize_degrees(omega)
}

/// Compute the true longitude of the ascending node.
///
/// True node = mean node + principal perturbation terms.
/// Source: Meeus Ch. 47.
#[must_use]
pub fn true_node(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    let mean = mean_node(jd);

    // Principal perturbation terms (simplified)
    let d = vedaksha_math::angle::deg_to_rad(297.850_192_1 + 445_267.111_403_4 * t);
    let m = vedaksha_math::angle::deg_to_rad(357.529_109_2 + 35_999.050_290_9 * t);
    let mp = vedaksha_math::angle::deg_to_rad(134.963_396_4 + 477_198.867_505_5 * t);
    let f = vedaksha_math::angle::deg_to_rad(93.272_095_0 + 483_202.017_523_3 * t);

    // Perturbation in longitude (degrees)
    let correction =
        -1.4979 * libm::sin(2.0 * (d - f)) - 0.1500 * libm::sin(m) - 0.1226 * libm::sin(2.0 * d)
            + 0.1176 * libm::sin(2.0 * f)
            - 0.0801 * libm::sin(2.0 * (mp - f));

    vedaksha_math::angle::normalize_degrees(mean + correction)
}

/// South Node (Ketu) = North Node + 180° (mean).
#[must_use]
pub fn south_node_mean(jd: f64) -> f64 {
    vedaksha_math::angle::normalize_degrees(mean_node(jd) + 180.0)
}

/// South Node (Ketu) = North Node + 180° (true).
#[must_use]
pub fn south_node_true(jd: f64) -> f64 {
    vedaksha_math::angle::normalize_degrees(true_node(jd) + 180.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 = JD 2451545.0
    const J2000: f64 = 2_451_545.0;

    #[test]
    fn mean_node_at_j2000_approx_125_04() {
        let mn = mean_node(J2000);
        assert!(
            (mn - 125.04).abs() < 0.5,
            "Mean node at J2000 expected ~125.04°, got {mn:.4}°"
        );
    }

    #[test]
    fn mean_node_always_in_range() {
        // Test several dates spanning millennia
        for offset in [-36525.0_f64, -10000.0, 0.0, 10000.0, 36525.0] {
            let jd = J2000 + offset;
            let mn = mean_node(jd);
            assert!(
                (0.0..360.0).contains(&mn),
                "Mean node out of [0,360) at JD {jd}: {mn:.4}°"
            );
        }
    }

    #[test]
    fn mean_node_retrograde_over_time() {
        // The lunar node is retrograde: decreases ~19.3°/year (mod 360)
        // Over one year (365.25 days), the node moves back ~19.3°
        let jd1 = J2000;
        let jd2 = J2000 + 365.25;
        let mn1 = mean_node(jd1);
        let mn2 = mean_node(jd2);
        // Compute signed difference accounting for wrap
        let mut diff = mn2 - mn1;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }
        // Should be approximately -19.3° (retrograde)
        assert!(
            (diff + 19.3).abs() < 1.0,
            "Mean node annual motion expected ~-19.3°, got {diff:.4}°"
        );
    }

    #[test]
    fn true_node_differs_from_mean_node() {
        let mn = mean_node(J2000);
        let tn = true_node(J2000);
        // Perturbation should make them differ (but not by huge amounts)
        let diff = (tn - mn).abs();
        assert!(diff > 0.0, "True node should differ from mean node");
        assert!(
            diff < 5.0,
            "True node perturbation unexpectedly large: {diff:.4}°"
        );
    }

    #[test]
    fn south_node_is_north_plus_180() {
        let mn = mean_node(J2000);
        let sn = south_node_mean(J2000);
        let expected = vedaksha_math::angle::normalize_degrees(mn + 180.0);
        assert!(
            (sn - expected).abs() < 1e-10,
            "South node should be north + 180°: south={sn:.6}, north+180={expected:.6}"
        );
    }

    #[test]
    fn true_node_at_j2000_close_to_mean_node() {
        let mn = mean_node(J2000);
        let tn = true_node(J2000);
        let mut diff = (tn - mn).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 3.0,
            "True node at J2000 should be within 3° of mean node, diff={diff:.4}°"
        );
    }
}
