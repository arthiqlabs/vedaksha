// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Chart orchestrator — top-level entry point that assembles a complete chart.
//!
//! Given pre-computed planet positions, geographic/temporal parameters, and a
//! configuration, [`compute_chart`] wires together houses, aspects, and
//! dignities into a single [`ComputedChart`] result.
//!
//! This module does **not** depend on any ephemeris provider. The caller is
//! responsible for obtaining planet positions (e.g. via an `EphemerisProvider`
//! or MCP server) and passing them in.

use crate::aspects::{Aspect, AspectType, BodyPosition, find_aspects};
use crate::dignity::{DignityPlanet, RulershipScheme, dignity_of, sign_of};
use crate::houses::{HouseCusps, HouseSystem, compute_houses};
use serde::{Deserialize, Serialize};

/// Configuration for chart computation.
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// House system to use.
    pub house_system: HouseSystem,
    /// If `Some`, apply sidereal zodiac with the given ayanamsha.
    /// If `None`, use the tropical zodiac.
    pub ayanamsha: Option<crate::sidereal::Ayanamsha>,
    /// Which rulership scheme for essential dignities.
    pub rulership_scheme: RulershipScheme,
    /// Which aspect types to detect.
    pub aspect_types: Vec<AspectType>,
    /// Multiplier for default orbs (1.0 = standard).
    pub orb_factor: f64,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            house_system: HouseSystem::Placidus,
            ayanamsha: None, // Tropical
            rulership_scheme: RulershipScheme::Traditional,
            aspect_types: AspectType::MAJOR.to_vec(),
            orb_factor: 1.0,
        }
    }
}

/// A planet's position in the chart with all computed attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPlanet {
    /// Display name of the planet (e.g. "Sun", "Moon").
    pub name: String,
    /// Ecliptic longitude in degrees (sidereal if ayanamsha applied).
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Heliocentric distance in AU.
    pub distance: f64,
    /// Daily speed in degrees/day.
    pub speed: f64,
    /// Whether the planet is retrograde (speed < 0).
    pub retrograde: bool,
    /// Name of the zodiac sign the planet occupies.
    pub sign: String,
    /// 0-based sign index (0 = Aries .. 11 = Pisces).
    pub sign_index: u8,
    /// House number (1-12) the planet falls in.
    pub house: u8,
    /// Essential dignity state description.
    pub dignity: String,
}

/// Complete computed chart result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedChart {
    /// All planets with their computed positions, signs, houses, and dignities.
    pub planets: Vec<ChartPlanet>,
    /// The 12 house cusps (plus ASC and MC).
    pub houses: HouseCusps,
    /// All detected aspects between planets.
    pub aspects: Vec<Aspect>,
    /// Human-readable summary of the configuration used.
    pub config_summary: String,
}

/// Compute a full chart from planet positions.
///
/// # Arguments
///
/// * `planet_data` — Vec of `(name, longitude, latitude, distance, speed)`.
///   Longitudes are tropical ecliptic degrees.
/// * `ramc`        — Right Ascension of MC in degrees.
/// * `geo_latitude` — Geographic latitude in degrees.
/// * `obliquity`   — Obliquity of the ecliptic in degrees.
/// * `jd`          — Julian Day (for ayanamsha computation).
/// * `config`      — Chart configuration.
#[must_use]
pub fn compute_chart(
    planet_data: &[(String, f64, f64, f64, f64)],
    ramc: f64,
    geo_latitude: f64,
    obliquity: f64,
    jd: f64,
    config: &ChartConfig,
) -> ComputedChart {
    // 1. Apply ayanamsha if sidereal
    let ayanamsha_offset = config
        .ayanamsha
        .map_or(0.0, |a| crate::sidereal::ayanamsha_value(a, jd));

    // 2. Compute houses
    let houses = compute_houses(ramc, geo_latitude, obliquity, config.house_system);

    // 3. Build planet positions with sign, house, dignity
    let mut planets = Vec::new();
    let mut body_positions = Vec::new();

    for (name, lon, lat, dist, speed) in planet_data {
        let sidereal_lon = vedaksha_math::angle::normalize_degrees(*lon - ayanamsha_offset);
        let sign = sign_of(sidereal_lon);
        let house = determine_house(sidereal_lon, &houses);

        // Map planet name to DignityPlanet for dignity lookup
        let dignity = match name_to_dignity_planet(name) {
            Some(dp) => format!("{:?}", dignity_of(dp, sign, config.rulership_scheme)),
            None => "Peregrine".to_string(),
        };

        planets.push(ChartPlanet {
            name: name.clone(),
            longitude: sidereal_lon,
            latitude: *lat,
            distance: *dist,
            speed: *speed,
            retrograde: *speed < 0.0,
            sign: sign.name().to_string(),
            sign_index: sign as u8,
            house,
            dignity,
        });

        body_positions.push(BodyPosition {
            longitude: sidereal_lon,
            speed: *speed,
        });
    }

    // 4. Find aspects
    let aspects = find_aspects(&body_positions, &config.aspect_types, config.orb_factor);

    let config_summary = format!(
        "Houses: {:?}, Zodiac: {}, Rulership: {:?}",
        config.house_system,
        config
            .ayanamsha
            .map_or_else(|| "Tropical".to_string(), |a| format!("{a:?}")),
        config.rulership_scheme,
    );

    ComputedChart {
        planets,
        houses,
        aspects,
        config_summary,
    }
}

/// Determine which house (1-12) a planet is in based on house cusps.
#[allow(clippy::cast_possible_truncation)]
fn determine_house(longitude: f64, houses: &HouseCusps) -> u8 {
    for i in 0..12 {
        let cusp = houses.cusps[i];
        let next_cusp = houses.cusps[(i + 1) % 12];

        if next_cusp > cusp {
            // Normal case: cusp range does not cross 0°/360°
            if longitude >= cusp && longitude < next_cusp {
                return (i as u8) + 1;
            }
        } else {
            // Wraps around 0°/360°
            if longitude >= cusp || longitude < next_cusp {
                return (i as u8) + 1;
            }
        }
    }
    1 // fallback
}

/// Map a planet name string to its [`DignityPlanet`] variant.
fn name_to_dignity_planet(name: &str) -> Option<DignityPlanet> {
    match name.to_lowercase().as_str() {
        "sun" => Some(DignityPlanet::Sun),
        "moon" => Some(DignityPlanet::Moon),
        "mars" => Some(DignityPlanet::Mars),
        "mercury" => Some(DignityPlanet::Mercury),
        "jupiter" => Some(DignityPlanet::Jupiter),
        "venus" => Some(DignityPlanet::Venus),
        "saturn" => Some(DignityPlanet::Saturn),
        "uranus" => Some(DignityPlanet::Uranus),
        "neptune" => Some(DignityPlanet::Neptune),
        "pluto" => Some(DignityPlanet::Pluto),
        _ => None,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a planet tuple.
    fn planet(name: &str, lon: f64, speed: f64) -> (String, f64, f64, f64, f64) {
        (name.to_string(), lon, 0.0, 1.0, speed)
    }

    /// Standard test parameters: RAMC=30, lat=45, obliquity=23.44, JD=J2000.
    const RAMC: f64 = 30.0;
    const LAT: f64 = 45.0;
    const OBL: f64 = 23.44;
    const JD: f64 = 2_451_545.0;

    #[test]
    fn compute_chart_returns_planets_and_houses() {
        let data = vec![planet("Sun", 130.0, 1.0), planet("Moon", 45.0, 13.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets.len(), 2);
        assert_eq!(chart.houses.cusps.len(), 12);
        assert!(!chart.config_summary.is_empty());
    }

    #[test]
    fn planets_have_correct_sign_assignments() {
        // 130° = Leo (sign index 4)
        let data = vec![planet("Sun", 130.0, 1.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].sign, "Leo");
        assert_eq!(chart.planets[0].sign_index, 4);
    }

    #[test]
    fn retrograde_flag_from_negative_speed() {
        let data = vec![planet("Mars", 200.0, -0.3), planet("Venus", 100.0, 1.2)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert!(chart.planets[0].retrograde, "Mars should be retrograde");
        assert!(!chart.planets[1].retrograde, "Venus should be direct");
    }

    #[test]
    fn aspects_detected_between_planets() {
        // Sun at 0° and Moon at 120° should form a trine
        let data = vec![planet("Sun", 0.0, 1.0), planet("Moon", 120.0, 13.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert!(
            chart
                .aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Trine),
            "Expected a trine between Sun and Moon"
        );
    }

    #[test]
    fn sidereal_mode_shifts_longitudes() {
        let data = vec![planet("Sun", 130.0, 1.0)];
        let mut config = ChartConfig::default();
        config.ayanamsha = Some(crate::sidereal::Ayanamsha::Lahiri);

        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &config);

        // Lahiri ayanamsha at J2000 is ~23.856°, so 130 − 23.856 ≈ 106.144°
        let expected = vedaksha_math::angle::normalize_degrees(
            130.0 - crate::sidereal::ayanamsha_value(crate::sidereal::Ayanamsha::Lahiri, JD),
        );
        assert!(
            (chart.planets[0].longitude - expected).abs() < 0.01,
            "Sidereal longitude mismatch: got {}, expected {}",
            chart.planets[0].longitude,
            expected
        );
    }

    #[test]
    fn dignity_computed_for_known_planet() {
        // Sun at 130° (Leo) should have Domicile dignity
        let data = vec![planet("Sun", 130.0, 1.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].dignity, "Domicile");
    }

    #[test]
    fn unknown_body_gets_peregrine() {
        let data = vec![planet("Chiron", 100.0, 0.05)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].dignity, "Peregrine");
    }

    #[test]
    fn determine_house_basic() {
        // Create simple equal house cusps at 0, 30, 60, ...
        let cusps: [f64; 12] = core::array::from_fn(|i| (i as f64) * 30.0);
        let houses = HouseCusps {
            cusps,
            asc: 0.0,
            mc: 270.0,
            system: HouseSystem::Equal,
            polar_fallback: false,
        };

        assert_eq!(determine_house(15.0, &houses), 1);
        assert_eq!(determine_house(45.0, &houses), 2);
        assert_eq!(determine_house(350.0, &houses), 12);
    }
}
