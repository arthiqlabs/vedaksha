// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Chara Karakas — Jaimini planet-role assignments by degree within sign.
//!
//! Each planet's degree within its current sign determines its karaka rank.
//! Highest degree = Atmakaraka; lowest = Darakaraka. Rahu is included only
//! in the 8-karaka scheme, using a reflected degree (30° − degrees) because
//! it moves retrograde.
//!
//! Source: Jaimini Sutras 1.1; B.V. Raman, *Studies in Jaimini Astrology*, Ch. 2.

use serde::{Deserialize, Serialize};

use crate::yoga::YogaPlanet;

/// Karaka role assigned to a planet in the Jaimini scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Karaka {
    Atmakaraka,
    Amatyakaraka,
    Bhratrikaraka,
    Matrikaraka,
    /// Present only in the 8-karaka scheme.
    Pitrikaraka,
    Putrakaraka,
    Gnatikaraka,
    Darakaraka,
}

/// 7- or 8-planet karaka ranking scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KarakaScheme {
    /// Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn.
    Seven,
    /// Adds Rahu (reflected degree) to the 7 planets.
    Eight,
}

/// A single planet–karaka assignment with the degree used for ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarakaAssignment {
    pub planet: YogaPlanet,
    pub karaka: Karaka,
    /// Effective degrees within sign used for ranking (0.0–30.0).
    pub degrees_in_sign: f64,
}

/// Sidereal longitudes for the planets involved in karaka ranking.
#[derive(Debug, Clone)]
pub struct KarakaInput {
    pub sun: f64,
    pub moon: f64,
    pub mars: f64,
    pub mercury: f64,
    pub jupiter: f64,
    pub venus: f64,
    pub saturn: f64,
    /// Required when `scheme` is [`KarakaScheme::Eight`].
    pub rahu: Option<f64>,
    pub scheme: KarakaScheme,
}

/// Degrees traversed within the current sign (0.0–29.999…).
#[must_use]
pub(crate) fn degrees_in_sign(longitude: f64) -> f64 {
    longitude.rem_euclid(30.0)
}

/// Rahu's effective degree is reflected because it moves retrograde.
///
/// When Rahu is exactly at 0° of a sign (sign boundary), it has traversed
/// no degrees and ranks lowest (0.0), not highest (30.0 would be out-of-range).
#[must_use]
pub(crate) fn rahu_degrees_in_sign(longitude: f64) -> f64 {
    let d = degrees_in_sign(longitude);
    if d == 0.0 { 0.0 } else { 30.0 - d }
}

/// Compute Chara Karaka assignments ranked from Atmakaraka to Darakaraka.
///
/// Returns 7 assignments for [`KarakaScheme::Seven`] or 8 for
/// [`KarakaScheme::Eight`]. The slice is ordered Atmakaraka first,
/// Darakaraka last.
///
/// # Panics
///
/// Panics if `scheme` is `Eight` and `input.rahu` is `None`.
#[must_use]
pub fn compute_karakas(input: &KarakaInput) -> Vec<KarakaAssignment> {
    let mut candidates: Vec<(YogaPlanet, f64)> = vec![
        (YogaPlanet::Sun, degrees_in_sign(input.sun)),
        (YogaPlanet::Moon, degrees_in_sign(input.moon)),
        (YogaPlanet::Mars, degrees_in_sign(input.mars)),
        (YogaPlanet::Mercury, degrees_in_sign(input.mercury)),
        (YogaPlanet::Jupiter, degrees_in_sign(input.jupiter)),
        (YogaPlanet::Venus, degrees_in_sign(input.venus)),
        (YogaPlanet::Saturn, degrees_in_sign(input.saturn)),
    ];

    if input.scheme == KarakaScheme::Eight {
        let rahu_lon = input
            .rahu
            .expect("rahu longitude required for 8-karaka scheme");
        candidates.push((YogaPlanet::Rahu, rahu_degrees_in_sign(rahu_lon)));
    }

    // Stable sort: equal-degree planets retain insertion order (Sun first … Rahu last).
    // `partial_cmp` is total on finite f64; treat NaN as equal (should never occur).
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let roles: &[Karaka] = match input.scheme {
        KarakaScheme::Seven => &[
            Karaka::Atmakaraka,
            Karaka::Amatyakaraka,
            Karaka::Bhratrikaraka,
            Karaka::Matrikaraka,
            Karaka::Putrakaraka,
            Karaka::Gnatikaraka,
            Karaka::Darakaraka,
        ],
        KarakaScheme::Eight => &[
            Karaka::Atmakaraka,
            Karaka::Amatyakaraka,
            Karaka::Bhratrikaraka,
            Karaka::Matrikaraka,
            Karaka::Pitrikaraka,
            Karaka::Putrakaraka,
            Karaka::Gnatikaraka,
            Karaka::Darakaraka,
        ],
    };

    candidates
        .into_iter()
        .zip(roles.iter().copied())
        .map(|((planet, deg), karaka)| KarakaAssignment {
            planet,
            karaka,
            degrees_in_sign: deg,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input_7() -> KarakaInput {
        KarakaInput {
            sun: 25.0,
            moon: 20.0,
            mars: 15.0,
            mercury: 10.0,
            jupiter: 5.0,
            venus: 2.0,
            saturn: 1.0,
            rahu: None,
            scheme: KarakaScheme::Seven,
        }
    }

    fn test_input_8() -> KarakaInput {
        KarakaInput {
            sun: 25.0,
            moon: 20.0,
            mars: 15.0,
            mercury: 10.0,
            jupiter: 5.0,
            venus: 2.0,
            saturn: 1.0,
            rahu: Some(310.0),
            scheme: KarakaScheme::Eight,
        }
    }

    #[test]
    fn seven_karaka_returns_seven_assignments() {
        let result = compute_karakas(&test_input_7());
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn seven_karaka_first_is_atmakaraka() {
        let result = compute_karakas(&test_input_7());
        assert_eq!(result[0].karaka, Karaka::Atmakaraka);
        assert_eq!(result[0].planet, YogaPlanet::Sun);
    }

    #[test]
    fn seven_karaka_last_is_darakaraka() {
        let result = compute_karakas(&test_input_7());
        assert_eq!(result[6].karaka, Karaka::Darakaraka);
        assert_eq!(result[6].planet, YogaPlanet::Saturn);
    }

    #[test]
    fn seven_karaka_roles_in_order() {
        let result = compute_karakas(&test_input_7());
        let roles: Vec<Karaka> = result.iter().map(|a| a.karaka).collect();
        assert_eq!(
            roles,
            vec![
                Karaka::Atmakaraka,
                Karaka::Amatyakaraka,
                Karaka::Bhratrikaraka,
                Karaka::Matrikaraka,
                Karaka::Putrakaraka,
                Karaka::Gnatikaraka,
                Karaka::Darakaraka,
            ]
        );
    }

    #[test]
    fn eight_karaka_returns_eight_assignments() {
        let result = compute_karakas(&test_input_8());
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn eight_karaka_includes_pitrikaraka() {
        let result = compute_karakas(&test_input_8());
        let roles: Vec<Karaka> = result.iter().map(|a| a.karaka).collect();
        assert!(roles.contains(&Karaka::Pitrikaraka));
    }

    #[test]
    fn eight_karaka_last_is_darakaraka() {
        let result = compute_karakas(&test_input_8());
        assert_eq!(result[7].karaka, Karaka::Darakaraka);
    }

    #[test]
    fn degrees_in_sign_wraps_correctly() {
        assert!((degrees_in_sign(395.0) - 5.0).abs() < 1e-9);
        assert!((degrees_in_sign(30.0) - 0.0).abs() < 1e-9);
        assert!((degrees_in_sign(29.999) - 29.999).abs() < 1e-9);
    }

    #[test]
    fn rahu_degrees_reflected() {
        assert!((rahu_degrees_in_sign(310.0) - 20.0).abs() < 1e-9);
        // At sign boundary (longitude 0°), Rahu ranks lowest (0.0)
        assert!((rahu_degrees_in_sign(0.0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn degrees_in_sign_stored_in_assignment() {
        let result = compute_karakas(&test_input_7());
        assert!((result[0].degrees_in_sign - 25.0).abs() < 1e-9);
    }
}
