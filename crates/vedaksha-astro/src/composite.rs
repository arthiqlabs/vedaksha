// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Composite chart — midpoint method.
//!
//! Creates a composite chart by computing the midpoint of each
//! pair of corresponding planets from two natal charts.

use vedaksha_math::angle::normalize_degrees;

/// A position in the composite chart.
#[derive(Debug, Clone, Copy)]
pub struct CompositePosition {
    /// Midpoint longitude in degrees [0, 360)
    pub longitude: f64,
    /// Average speed in degrees/day
    pub speed: f64,
}

/// Compute composite chart positions using the midpoint method.
///
/// For each pair of corresponding planets (`chart_a`\[i\], `chart_b`\[i\]),
/// compute the shorter-arc midpoint.
///
/// # Arguments
/// * `lons_a` — longitudes from chart A in degrees
/// * `lons_b` — longitudes from chart B in degrees
/// * `speeds_a` — speeds from chart A in degrees/day
/// * `speeds_b` — speeds from chart B in degrees/day
///
/// # Panics
/// Panics if arrays have different lengths.
#[must_use]
pub fn compute_composite(
    lons_a: &[f64],
    lons_b: &[f64],
    speeds_a: &[f64],
    speeds_b: &[f64],
) -> Vec<CompositePosition> {
    assert_eq!(lons_a.len(), lons_b.len());
    assert_eq!(lons_a.len(), speeds_a.len());
    assert_eq!(lons_a.len(), speeds_b.len());

    lons_a
        .iter()
        .zip(lons_b.iter())
        .zip(speeds_a.iter().zip(speeds_b.iter()))
        .map(|((&lon_a, &lon_b), (&spd_a, &spd_b))| {
            let longitude = shorter_arc_midpoint(lon_a, lon_b);
            let speed = f64::midpoint(spd_a, spd_b);
            CompositePosition { longitude, speed }
        })
        .collect()
}

/// Compute the shorter-arc midpoint between two longitudes.
fn shorter_arc_midpoint(lon_a: f64, lon_b: f64) -> f64 {
    let diff = normalize_degrees(lon_b - lon_a);
    if diff <= 180.0 {
        normalize_degrees(lon_a + diff / 2.0)
    } else {
        normalize_degrees(lon_a + (diff - 360.0) / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    #[test]
    fn midpoint_of_10_and_30_is_20() {
        let result = shorter_arc_midpoint(10.0, 30.0);
        assert!((result - 20.0).abs() < EPS, "Expected 20.0, got {result}");
    }

    #[test]
    fn midpoint_wraps_around_zero() {
        // 350° and 10° — shorter arc crosses 0°; midpoint = 0°
        let result = shorter_arc_midpoint(350.0, 10.0);
        assert!((result - 0.0).abs() < EPS, "Expected 0.0, got {result}");
    }

    #[test]
    fn midpoint_of_0_and_180_is_90() {
        // diff = 180°, which is <= 180, so: 0 + 90 = 90°
        let result = shorter_arc_midpoint(0.0, 180.0);
        assert!((result - 90.0).abs() < EPS, "Expected 90.0, got {result}");
    }

    #[test]
    fn composite_preserves_planet_count() {
        let lons_a = [0.0_f64, 90.0, 180.0];
        let lons_b = [30.0_f64, 60.0, 210.0];
        let spds_a = [1.0_f64, 0.5, 0.8];
        let spds_b = [0.5_f64, 0.3, 0.2];
        let composite = compute_composite(&lons_a, &lons_b, &spds_a, &spds_b);
        assert_eq!(composite.len(), 3, "Planet count must be preserved");
    }

    #[test]
    fn composite_average_speed_is_correct() {
        let lons_a = [0.0_f64];
        let lons_b = [60.0_f64];
        let spds_a = [2.0_f64];
        let spds_b = [1.0_f64];
        let composite = compute_composite(&lons_a, &lons_b, &spds_a, &spds_b);
        assert_eq!(composite.len(), 1);
        assert!(
            (composite[0].speed - 1.5).abs() < EPS,
            "Expected average speed 1.5, got {}",
            composite[0].speed
        );
    }

    #[test]
    fn composite_longitude_correct_for_simple_pair() {
        let lons_a = [10.0_f64];
        let lons_b = [30.0_f64];
        let spds_a = [1.0_f64];
        let spds_b = [1.0_f64];
        let composite = compute_composite(&lons_a, &lons_b, &spds_a, &spds_b);
        assert!(
            (composite[0].longitude - 20.0).abs() < EPS,
            "Expected midpoint 20.0, got {}",
            composite[0].longitude
        );
    }

    #[test]
    fn composite_wraps_correctly() {
        let lons_a = [350.0_f64];
        let lons_b = [10.0_f64];
        let spds_a = [0.0_f64];
        let spds_b = [0.0_f64];
        let composite = compute_composite(&lons_a, &lons_b, &spds_a, &spds_b);
        assert!(
            (composite[0].longitude - 0.0).abs() < EPS,
            "Expected wrap-around midpoint 0.0, got {}",
            composite[0].longitude
        );
    }
}
