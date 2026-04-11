// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Transit search engine with adaptive step + bisection refinement.
//!
//! Finds all times in a date range when a transiting planet forms a
//! specified aspect to a natal position. Also computes solar and lunar
//! returns.
//!
//! Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 3 (bisection method).

use vedaksha_math::angle;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A transit event — a moment when a transiting planet forms an aspect to a natal point.
#[derive(Debug, Clone)]
pub struct TransitEvent {
    /// The transiting body.
    pub transiting_body: String,
    /// The natal body/point being aspected.
    pub natal_body: String,
    /// Type of aspect.
    pub aspect_type: String,
    /// Exact Julian Day of the aspect.
    pub exact_jd: f64,
    /// Whether the transit is applying (`true`) or separating (`false`) at the exact moment.
    pub applying: bool,
    /// Orb at the exact moment (should be ~0).
    pub exact_orb: f64,
}

/// Configuration for a transit search.
#[derive(Debug, Clone)]
pub struct TransitSearchConfig {
    /// Natal positions: (body name, longitude in degrees).
    pub natal_positions: Vec<(String, f64)>,
    /// Start of search range (Julian Day).
    pub start_jd: f64,
    /// End of search range (Julian Day).
    pub end_jd: f64,
    /// Which transiting bodies to track: (name, body index for the callback).
    pub transiting_bodies: Vec<(String, usize)>,
    /// Which aspect types to search for: (name, angle in degrees).
    pub aspect_types: Vec<(String, f64)>,
    /// Maximum orb for detection (degrees).
    pub max_orb: f64,
    /// Step size for coarse scan (days).
    pub step_size: f64,
}

// ---------------------------------------------------------------------------
// Core search
// ---------------------------------------------------------------------------

/// Search for transits in a date range.
///
/// Uses adaptive step search: coarse scan at `step_size`-day intervals,
/// then bisection refinement when a sign change is detected in the
/// aspect function.
///
/// # Arguments
/// * `config` — search configuration
/// * `get_longitude` — callback returning ecliptic longitude (degrees)
///   for a body index at a Julian Day, or `None` on failure.
///
/// # Returns
/// Transit events found, sorted by Julian Day.
pub fn search_transits(
    config: &TransitSearchConfig,
    get_longitude: &dyn Fn(usize, f64) -> Option<f64>,
) -> Vec<TransitEvent> {
    let mut events = Vec::new();

    for (body_name, body_idx) in &config.transiting_bodies {
        for (natal_name, natal_lon) in &config.natal_positions {
            for (aspect_name, aspect_angle) in &config.aspect_types {
                // For an aspect at angle A, there are two crossing points
                // on the ecliptic: natal+A and natal+(360-A). Both produce
                // the same angular_separation. Conjunction (0) and
                // opposition (180) have only one distinct crossing each.
                let mut search_angles = vec![*aspect_angle];
                if *aspect_angle > 0.0 && *aspect_angle < 180.0 {
                    search_angles.push(360.0 - *aspect_angle);
                }

                for &search_angle in &search_angles {
                    scan_for_crossings(
                        config,
                        *body_idx,
                        body_name,
                        natal_name,
                        *natal_lon,
                        aspect_name,
                        *aspect_angle,
                        search_angle,
                        get_longitude,
                        &mut events,
                    );
                }
            }
        }
    }

    events.sort_by(|a, b| {
        a.exact_jd
            .partial_cmp(&b.exact_jd)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    events
}

/// Scan a single directed angle for zero-crossings of the aspect function.
#[allow(clippy::too_many_arguments)]
fn scan_for_crossings(
    config: &TransitSearchConfig,
    body_idx: usize,
    body_name: &str,
    natal_name: &str,
    natal_lon: f64,
    aspect_name: &str,
    aspect_angle: f64,
    search_angle: f64,
    get_longitude: &dyn Fn(usize, f64) -> Option<f64>,
    events: &mut Vec<TransitEvent>,
) {
    let step = config.step_size;
    let mut t = config.start_jd;
    let mut prev_lon = get_longitude(body_idx, t);

    while t < config.end_jd {
        let next_t = (t + step).min(config.end_jd);
        let curr_lon = get_longitude(body_idx, next_t);

        if let (Some(prev), Some(curr)) = (prev_lon, curr_lon) {
            let prev_f = aspect_function(prev, natal_lon, search_angle);
            let curr_f = aspect_function(curr, natal_lon, search_angle);

            // Sign change (or exact zero crossing) with proximity guard.
            let sign_change = (prev_f * curr_f < 0.0) || (prev_f != 0.0 && curr_f == 0.0);
            if sign_change && prev_f.abs() < config.max_orb + 5.0 {
                if let Some(exact_jd) = bisect_transit(
                    t,
                    next_t,
                    natal_lon,
                    search_angle,
                    body_idx,
                    get_longitude,
                    50,
                ) {
                    if let Some(exact_lon) = get_longitude(body_idx, exact_jd) {
                        let orb =
                            (angle::angular_separation(exact_lon, natal_lon) - aspect_angle).abs();
                        if orb <= config.max_orb {
                            events.push(TransitEvent {
                                transiting_body: body_name.to_owned(),
                                natal_body: natal_name.to_owned(),
                                aspect_type: aspect_name.to_owned(),
                                exact_jd,
                                applying: curr_f.abs() < prev_f.abs(),
                                exact_orb: orb,
                            });
                        }
                    }
                }
            }
        }

        prev_lon = curr_lon;
        t = next_t;
    }
}

// ---------------------------------------------------------------------------
// Solar / Lunar returns
// ---------------------------------------------------------------------------

/// Find the exact moment of a solar return (Sun returns to natal longitude).
///
/// # Arguments
/// * `natal_sun_longitude` — natal Sun longitude in degrees
/// * `search_start_jd` — approximate date (near birthday)
/// * `get_sun_longitude` — callback returning Sun longitude at a JD
///
/// Searches +/- 1 day from `search_start_jd` using bisection.
pub fn solar_return(
    natal_sun_longitude: f64,
    search_start_jd: f64,
    get_sun_longitude: &dyn Fn(f64) -> Option<f64>,
) -> Option<f64> {
    // Sun moves ~1 deg/day — search a 2-day window.
    find_return(
        natal_sun_longitude,
        search_start_jd - 1.0,
        search_start_jd + 1.0,
        get_sun_longitude,
        60,
    )
}

/// Find the exact moment of a lunar return (Moon returns to natal longitude).
///
/// # Arguments
/// * `natal_moon_longitude` — natal Moon longitude in degrees
/// * `search_start_jd` — approximate date
/// * `get_moon_longitude` — callback returning Moon longitude at a JD
///
/// Searches forward in 1-day steps for up to 30 days.
pub fn lunar_return(
    natal_moon_longitude: f64,
    search_start_jd: f64,
    get_moon_longitude: &dyn Fn(f64) -> Option<f64>,
) -> Option<f64> {
    // Moon moves ~13 deg/day, full cycle ~27.3 days.
    let mut t = search_start_jd;
    for _ in 0..30 {
        let lon = get_moon_longitude(t)?;
        let next_lon = get_moon_longitude(t + 1.0)?;
        if longitude_crosses(lon, next_lon, natal_moon_longitude) {
            return find_return(natal_moon_longitude, t, t + 1.0, get_moon_longitude, 60);
        }
        t += 1.0;
    }
    None
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Signed aspect function using directed difference.
///
/// Computes `normalize_degrees(transit_lon - natal_lon) - target_angle`,
/// wrapped to `(-180, 180]`. A zero-crossing indicates the transit body
/// is at exactly the target angular distance from the natal point.
fn aspect_function(transit_lon: f64, natal_lon: f64, target_angle: f64) -> f64 {
    let diff = angle::normalize_degrees(transit_lon - natal_lon) - target_angle;
    angle::normalize_degrees_signed(diff)
}

/// Bisection to find exact transit moment.
fn bisect_transit(
    mut t_low: f64,
    mut t_high: f64,
    natal_lon: f64,
    target_angle: f64,
    body_idx: usize,
    get_longitude: &dyn Fn(usize, f64) -> Option<f64>,
    max_iter: u32,
) -> Option<f64> {
    let threshold = 1e-8; // ~0.86 microseconds

    for _ in 0..max_iter {
        let t_mid = f64::midpoint(t_low, t_high);
        if (t_high - t_low) < threshold {
            return Some(t_mid);
        }

        let lon_low = get_longitude(body_idx, t_low)?;
        let lon_mid = get_longitude(body_idx, t_mid)?;

        let f_low = aspect_function(lon_low, natal_lon, target_angle);
        let f_mid = aspect_function(lon_mid, natal_lon, target_angle);

        if f_low * f_mid < 0.0 {
            t_high = t_mid;
        } else {
            t_low = t_mid;
        }
    }

    Some(f64::midpoint(t_low, t_high))
}

/// Check whether the target longitude is crossed between two consecutive
/// longitude samples (accounting for the 360-to-0 wraparound).
fn longitude_crosses(lon1: f64, lon2: f64, target: f64) -> bool {
    let d1 = angle::normalize_degrees(target - lon1);
    let d2 = angle::normalize_degrees(target - lon2);
    // One sample ahead of target, the other behind (modulo 360).
    (d1 < 180.0) != (d2 < 180.0)
}

/// Bisection to find the exact moment a body reaches a target longitude.
fn find_return(
    target_lon: f64,
    t_start: f64,
    t_end: f64,
    get_longitude: &dyn Fn(f64) -> Option<f64>,
    max_iter: u32,
) -> Option<f64> {
    let mut low = t_start;
    let mut high = t_end;
    let threshold = 1e-10; // sub-millisecond

    for _ in 0..max_iter {
        let mid = f64::midpoint(low, high);
        if (high - low) < threshold {
            return Some(mid);
        }

        let lon_low = get_longitude(low)?;
        let lon_mid = get_longitude(mid)?;

        let d_low = angle::normalize_degrees(target_lon - lon_low);
        let d_mid = angle::normalize_degrees(target_lon - lon_mid);

        if (d_low < 180.0) == (d_mid < 180.0) {
            low = mid;
        } else {
            high = mid;
        }
    }

    Some(f64::midpoint(low, high))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_JD: f64 = 2_451_545.0; // J2000.0

    /// Synthetic linear motion: body 0 at 1 deg/day, body 1 at 13 deg/day.
    fn linear_longitude(body_idx: usize, jd: f64) -> Option<f64> {
        let speed = match body_idx {
            0 => 1.0,
            1 => 13.0,
            _ => 0.5,
        };
        Some(angle::normalize_degrees(speed * (jd - BASE_JD)))
    }

    fn sun_longitude(jd: f64) -> Option<f64> {
        Some(angle::normalize_degrees(1.0 * (jd - BASE_JD)))
    }

    fn moon_longitude(jd: f64) -> Option<f64> {
        Some(angle::normalize_degrees(13.0 * (jd - BASE_JD)))
    }

    // -- Transit search tests -----------------------------------------------

    #[test]
    fn find_conjunction_with_natal_point() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalSun".into(), 30.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 60.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Conjunction".into(), 0.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty(), "Should find at least one conjunction");
        let first = &events[0];
        assert!(
            (first.exact_jd - (BASE_JD + 30.0)).abs() < 0.5,
            "Expected near day 30, got {}",
            first.exact_jd
        );
    }

    #[test]
    fn find_opposition() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 0.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 200.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Opposition".into(), 180.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty(), "Should find opposition");
        let first = &events[0];
        assert!(
            (first.exact_jd - (BASE_JD + 180.0)).abs() < 0.5,
            "Expected near day 180, got {}",
            first.exact_jd
        );
    }

    #[test]
    fn find_trine() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 0.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 150.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Trine".into(), 120.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty(), "Should find trine");
        let first = &events[0];
        assert!(
            (first.exact_jd - (BASE_JD + 120.0)).abs() < 0.5,
            "Expected near day 120, got {}",
            first.exact_jd
        );
    }

    #[test]
    fn find_square() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 0.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 120.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Square".into(), 90.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty(), "Should find square");
        let first = &events[0];
        assert!(
            (first.exact_jd - (BASE_JD + 90.0)).abs() < 0.5,
            "Expected near day 90, got {}",
            first.exact_jd
        );
    }

    #[test]
    fn find_sextile() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 0.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 80.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Sextile".into(), 60.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty(), "Should find sextile");
        let first = &events[0];
        assert!(
            (first.exact_jd - (BASE_JD + 60.0)).abs() < 0.5,
            "Expected near day 60, got {}",
            first.exact_jd
        );
    }

    #[test]
    fn no_events_outside_range() {
        // Natal at 90 deg, search only first 10 days — Sun won't reach 90 yet.
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 90.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 10.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Conjunction".into(), 0.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(events.is_empty(), "Should find no events in short range");
    }

    #[test]
    fn events_sorted_by_jd() {
        // Multiple aspects should come back sorted.
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 0.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 200.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![
                ("Sextile".into(), 60.0),
                ("Square".into(), 90.0),
                ("Trine".into(), 120.0),
                ("Opposition".into(), 180.0),
            ],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(events.len() >= 4, "Should find multiple aspects");
        for w in events.windows(2) {
            assert!(
                w[0].exact_jd <= w[1].exact_jd,
                "Events not sorted: {} > {}",
                w[0].exact_jd,
                w[1].exact_jd
            );
        }
    }

    #[test]
    fn orb_at_exact_moment_is_small() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 30.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 60.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Conjunction".into(), 0.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        assert!(!events.is_empty());
        assert!(
            events[0].exact_orb < 0.001,
            "Orb should be near zero, got {}",
            events[0].exact_orb
        );
    }

    #[test]
    fn multiple_bodies_and_natal_points() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalSun".into(), 30.0), ("NatalMoon".into(), 60.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 60.0,
            transiting_bodies: vec![("Sun".into(), 0), ("Moon".into(), 1)],
            aspect_types: vec![("Conjunction".into(), 0.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &linear_longitude);
        // At least Sun-NatalSun conjunction (day 30) should exist.
        assert!(!events.is_empty(), "Should find events for multiple bodies");
    }

    #[test]
    fn callback_returning_none_is_handled() {
        let config = TransitSearchConfig {
            natal_positions: vec![("NatalPoint".into(), 30.0)],
            start_jd: BASE_JD,
            end_jd: BASE_JD + 60.0,
            transiting_bodies: vec![("Sun".into(), 0)],
            aspect_types: vec![("Conjunction".into(), 0.0)],
            max_orb: 1.0,
            step_size: 1.0,
        };
        let events = search_transits(&config, &|_, _| None);
        assert!(events.is_empty(), "Should handle None gracefully");
    }

    // -- Solar / Lunar return tests -----------------------------------------

    #[test]
    fn solar_return_linear() {
        let natal_lon = 280.0;
        // Sun at 1 deg/day reaches 280 at day 280. Search near there.
        let start_jd = BASE_JD + 280.0;
        let result = solar_return(natal_lon, start_jd, &sun_longitude);
        assert!(result.is_some(), "Should find solar return");
        let return_jd = result.unwrap();
        assert!(
            (return_jd - (BASE_JD + 280.0)).abs() < 0.01,
            "Expected near day 280, got offset {}",
            return_jd - BASE_JD
        );
    }

    #[test]
    fn lunar_return_within_cycle() {
        let natal_lon = 90.0;
        // Moon at 13 deg/day reaches 90 at day ~6.92.
        let result = lunar_return(natal_lon, BASE_JD, &moon_longitude);
        assert!(result.is_some(), "Should find lunar return");
        let return_jd = result.unwrap();
        let expected = BASE_JD + 90.0 / 13.0;
        assert!(
            (return_jd - expected).abs() < 0.1,
            "Expected near day {:.2}, got offset {:.2}",
            90.0 / 13.0,
            return_jd - BASE_JD
        );
    }

    #[test]
    fn lunar_return_none_callback() {
        let result = lunar_return(90.0, BASE_JD, &|_| None);
        assert!(result.is_none(), "Should return None on callback failure");
    }

    #[test]
    fn longitude_crosses_basic() {
        assert!(longitude_crosses(350.0, 10.0, 0.0));
        assert!(!longitude_crosses(10.0, 20.0, 0.0));
    }
}
