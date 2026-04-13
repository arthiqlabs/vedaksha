// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
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
///
/// Uses the 5 largest perturbation terms from Meeus Ch. 47.
/// Residual vs full lunar theory: ~0.09° max for modern dates.
/// Improving beyond this requires verified coefficients from
/// Chapront's complete lunar node series — not attempted here
/// to avoid sign/argument transcription errors.
///
/// Source: Meeus, *Astronomical Algorithms*, 2nd ed., Ch. 47.
#[must_use]
pub fn true_node(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    let mean = mean_node(jd);

    // Fundamental arguments (degrees → radians)
    let d = vedaksha_math::angle::deg_to_rad(297.850_192_1 + 445_267.111_403_4 * t);
    let m = vedaksha_math::angle::deg_to_rad(357.529_109_2 + 35_999.050_290_9 * t);
    let mp = vedaksha_math::angle::deg_to_rad(134.963_396_4 + 477_198.867_505_5 * t);
    let f = vedaksha_math::angle::deg_to_rad(93.272_095_0 + 483_202.017_523_3 * t);

    // Perturbation in longitude (degrees) — 5 principal terms.
    let correction =
        -1.4979 * libm::sin(2.0 * (d - f)) - 0.1500 * libm::sin(m) - 0.1226 * libm::sin(2.0 * d)
            + 0.1176 * libm::sin(2.0 * f)
            - 0.0801 * libm::sin(2.0 * (mp - f));

    vedaksha_math::angle::normalize_degrees(mean + correction)
}

/// Compute the true longitude of the ascending node from osculating
/// orbital elements derived from the Moon's position and velocity.
///
/// Standard osculating node method (Bate, Mueller & White 1971): given the
/// Moon's geocentric ecliptic position `(x, y, z)` and velocity
/// `(vx, vy, vz)`, find where the instantaneous orbital plane crosses
/// the ecliptic plane (z = 0, ascending).
///
/// The node direction is `r - (z/vz) * v`, with a sign correction to
/// select the ascending (not descending) node. This is exact to the
/// accuracy of the underlying lunar ephemeris.
///
/// **Important:** Uses ecliptic-of-date coordinates (not J2000), because
/// the astrological true node is the node on the ecliptic of date.
///
/// Requires the `analytical` module (ELP/MPP02).
///
/// Source: Standard orbital mechanics (Bate, Mueller & White 1971;
/// Montenbruck & Gill 2000).
#[must_use]
pub fn true_node_osculating(jd: f64) -> f64 {
    // Use a tight differentiation step for velocity (0.001 day ≈ 86 seconds).
    // The osculating node is sensitive to velocity direction.
    let moon = {
        let dt = 0.001_f64;
        let m0 = crate::analytical::elp_mpp02::elp_geocentric_of_date(jd);
        let mp = crate::analytical::elp_mpp02::elp_geocentric_of_date(jd + dt);
        let mm = crate::analytical::elp_mpp02::elp_geocentric_of_date(jd - dt);
        let inv_2dt = 1.0 / (2.0 * dt);
        crate::analytical::elp_mpp02::MoonRectangular {
            x: m0.x, y: m0.y, z: m0.z,
            vx: (mp.x - mm.x) * inv_2dt,
            vy: (mp.y - mm.y) * inv_2dt,
            vz: (mp.z - mm.z) * inv_2dt,
        }
    };

    // Avoid division by zero if Moon is exactly on the ecliptic
    // with zero vertical velocity (astronomically impossible, but safe).
    let vz = if moon.vz.abs() < 1e-15 { 1e-15 } else { moon.vz };

    // Find where the tangent line to the orbit crosses z = 0.
    let fac = moon.z / vz;
    let sgn = vz.signum();

    // Node direction vector (ascending node selected by sign of vz).
    let nx = (moon.x - fac * moon.vx) * sgn;
    let ny = (moon.y - fac * moon.vy) * sgn;

    let lon_rad = libm::atan2(ny, nx);
    let lon_deg = lon_rad * 180.0 / core::f64::consts::PI;
    vedaksha_math::angle::normalize_degrees(lon_deg)
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

/// South Node (Ketu) = North Node + 180° (osculating).
#[must_use]
pub fn south_node_osculating(jd: f64) -> f64 {
    vedaksha_math::angle::normalize_degrees(true_node_osculating(jd) + 180.0)
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

    #[test]
    fn osculating_node_at_j2000_close_to_mean_node() {
        let mn = mean_node(J2000);
        let osc = true_node_osculating(J2000);
        let mut diff = (osc - mn).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 3.0,
            "Osculating node at J2000 should be within 3° of mean node, diff={diff:.4}°"
        );
    }

    #[test]
    fn osculating_node_close_to_meeus_true_node() {
        // The osculating node and Meeus 5-term true node should agree
        // within ~0.1° (the Meeus approximation error).
        let tn = true_node(J2000);
        let osc = true_node_osculating(J2000);
        let mut diff = (osc - tn).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 0.5,
            "Osculating and Meeus true nodes should agree within 0.5°, diff={diff:.4}°"
        );
    }

    #[test]
    fn osculating_node_always_in_range() {
        for offset in [-3652.5_f64, 0.0, 3652.5, 7305.0, 10957.5] {
            let jd = J2000 + offset;
            let osc = true_node_osculating(jd);
            assert!(
                (0.0..360.0).contains(&osc),
                "Osculating node out of [0,360) at JD {jd}: {osc:.4}°"
            );
        }
    }

    #[test]
    fn osculating_node_vs_jpl_horizons() {
        // Oracle data: JPL Horizons DE441 osculating node longitude (J2000 ecliptic).
        // Query: COMMAND='301', CENTER='500@399', EPHEM_TYPE='ELEMENTS'.
        // The OM (longitude of ascending node) values are osculating orbital
        // elements from the DE441 numerical integration.
        //
        // Our osculating node uses ecliptic-of-date (for astrological use),
        // so small differences (~0.02°) are expected from the J2000↔date
        // frame rotation. Tolerance: 0.05°.
        //
        // Source: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/).
        let oracle = [
            (2451545.0, 123.984, "J2000"),       // 2000-01-01
            (2453006.0, 49.237,  "2004-01-01"),   // JD from Horizons
            (2455197.5, 290.923, "2010-01-01"),
            (2457388.5, 174.656, "2016-01-01"),
            (2459580.5, 60.937,  "2022-01-01"),
        ];

        for (jd, jpl_node, label) in &oracle {
            let osc = true_node_osculating(*jd);

            let mut diff = (osc - jpl_node).abs();
            if diff > 180.0 { diff = 360.0 - diff; }

            assert!(
                diff < 0.05,
                "{label}: osculating vs JPL DE441 diff too large: {diff:.4}° \
                 (ours={osc:.4}°, JPL={jpl_node:.3}°)"
            );
        }
    }

    #[test]
    fn osculating_node_multi_epoch_sanity() {
        // The osculating node is the Moon's instantaneous ascending node
        // derived from position + velocity. It differs from the Meeus
        // 5-term "true node" because:
        //   - Meeus smooths perturbations into 5 terms (~0.09° residual)
        //   - Osculating captures the full instantaneous orbital plane
        //
        // The two methods can differ by up to ~0.5° at any given moment.
        // This is NOT an error — it's the inherent difference between
        // a smoothed series and an instantaneous orbital element.
        //
        // For validation, we check:
        // 1. Values are in [0°, 360°)
        // 2. Osculating is within 3° of mean node (sanity bound)
        // 3. Osculating and Meeus agree within 0.5° (expected divergence)
        let epochs = [
            (2451545.0, "J2000"),
            (2453006.0, "2004-01-01"),
            (2455197.5, "2010-01-01"),
            (2457388.5, "2016-01-01"),
            (2459580.5, "2022-01-01"),
            (2461041.5, "2026-01-01"),
            (2461142.5, "2026-04-12"),
        ];

        for (jd, label) in &epochs {
            let mean = mean_node(*jd);
            let true_m = true_node(*jd);
            let osc = true_node_osculating(*jd);

            // Valid range
            assert!(
                (0.0..360.0).contains(&osc),
                "{label}: osculating out of range: {osc:.4}°"
            );

            // Osculating vs Meeus: within 0.5° (inherent method difference)
            let mut diff_meeus = (osc - true_m).abs();
            if diff_meeus > 180.0 { diff_meeus = 360.0 - diff_meeus; }
            assert!(
                diff_meeus < 0.5,
                "{label}: osculating vs Meeus diff too large: {diff_meeus:.4}°"
            );

            // Osculating vs mean: within 3° (sanity)
            let mut diff_mean = (osc - mean).abs();
            if diff_mean > 180.0 { diff_mean = 360.0 - diff_mean; }
            assert!(
                diff_mean < 3.0,
                "{label}: osculating vs mean diff too large: {diff_mean:.4}°"
            );
        }
    }

    #[test]
    fn osculating_south_node_is_north_plus_180() {
        let north = true_node_osculating(J2000);
        let south = south_node_osculating(J2000);
        let expected = vedaksha_math::angle::normalize_degrees(north + 180.0);
        assert!(
            (south - expected).abs() < 1e-10,
            "Osculating south node should be north + 180°"
        );
    }
}
