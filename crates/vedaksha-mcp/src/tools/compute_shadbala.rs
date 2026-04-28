// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_shadbala` — full six-fold planetary strength with Ishta/Kashta Phala.

use serde::Deserialize;

use crate::validation::McpError;

/// Per-planet data object in the input array.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanetEntry {
    pub planet: String,
    /// Sign index 0–11.
    pub sign: u8,
    /// Sidereal longitude [0, 360).
    pub longitude: f64,
    /// Bhava (house) 1–12.
    pub bhava: u8,
    /// Daily speed in degrees/day. Negative = retrograde.
    pub speed: f64,
    /// Average daily speed for this planet.
    pub average_speed: f64,
    #[serde(default)]
    pub benefic_aspect_count: u32,
    #[serde(default)]
    pub malefic_aspect_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeShidbalaInput {
    pub planets: Vec<PlanetEntry>,
    #[serde(default)]
    pub is_daytime: bool,
    #[serde(default)]
    pub moon_phase_waxing: bool,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_shadbala",
        description: "Compute full six-fold Shadbala (Sthana, Dig, Kala, Cheshta, Naisargika, \
            Drik Bala) for each planet, plus uccha_bala, ishta_phala, and kashta_phala per \
            BPHS Ch.27-28.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "planets": {
                    "type": "array",
                    "description": "Array of planet data objects.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "planet": { "type": "string", "description": "Planet name: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn" },
                            "sign":   { "type": "integer", "minimum": 0, "maximum": 11 },
                            "longitude": { "type": "number", "minimum": 0, "maximum": 360 },
                            "bhava": { "type": "integer", "minimum": 1, "maximum": 12 },
                            "speed": { "type": "number" },
                            "average_speed": { "type": "number" },
                            "benefic_aspect_count": { "type": "integer", "default": 0 },
                            "malefic_aspect_count": { "type": "integer", "default": 0 }
                        },
                        "required": ["planet", "sign", "longitude", "bhava", "speed", "average_speed"]
                    }
                },
                "is_daytime": { "type": "boolean", "default": false },
                "moon_phase_waxing": { "type": "boolean", "default": false }
            },
            "required": ["planets"]
        }),
    }
}

/// Parse a planet name string to `YogaPlanet`.
///
/// # Errors
/// Returns `McpError` for unknown names.
pub fn parse_planet(name: &str) -> Result<vedaksha_vedic::yoga::YogaPlanet, McpError> {
    use vedaksha_vedic::yoga::YogaPlanet;
    match name.to_lowercase().as_str() {
        "sun"     => Ok(YogaPlanet::Sun),
        "moon"    => Ok(YogaPlanet::Moon),
        "mars"    => Ok(YogaPlanet::Mars),
        "mercury" => Ok(YogaPlanet::Mercury),
        "jupiter" => Ok(YogaPlanet::Jupiter),
        "venus"   => Ok(YogaPlanet::Venus),
        "saturn"  => Ok(YogaPlanet::Saturn),
        other => Err(McpError::invalid_parameter(
            "planet",
            &format!("unknown planet '{other}'; expected Sun, Moon, Mars, Mercury, Jupiter, Venus, or Saturn"),
        )),
    }
}

/// Validate a [`ComputeShidbalaInput`].
///
/// # Errors
/// Returns [`McpError`] for unknown planet names, out-of-range values.
pub fn validate(input: &ComputeShidbalaInput) -> Result<(), McpError> {
    for entry in &input.planets {
        parse_planet(&entry.planet)?;
        if entry.sign > 11 {
            return Err(McpError::invalid_parameter("sign", "must be 0–11"));
        }
        if !entry.longitude.is_finite() || !(0.0..360.0).contains(&entry.longitude) {
            return Err(McpError::invalid_parameter(
                "longitude", "must be a finite number in [0, 360)",
            ));
        }
        if entry.bhava == 0 || entry.bhava > 12 {
            return Err(McpError::invalid_parameter("bhava", "must be 1–12"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_entry() -> PlanetEntry {
        PlanetEntry {
            planet: "Jupiter".to_string(),
            sign: 3,
            longitude: 105.0,
            bhava: 4,
            speed: -0.05,
            average_speed: 0.08,
            benefic_aspect_count: 2,
            malefic_aspect_count: 1,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        let input = ComputeShidbalaInput {
            planets: vec![valid_entry()],
            is_daytime: true,
            moon_phase_waxing: true,
        };
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_planet() {
        let mut entry = valid_entry();
        entry.planet = "Pluto".to_string();
        let input = ComputeShidbalaInput {
            planets: vec![entry],
            is_daytime: false,
            moon_phase_waxing: false,
        };
        assert_eq!(validate(&input).unwrap_err().error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_bad_sign() {
        let mut entry = valid_entry();
        entry.sign = 12;
        let input = ComputeShidbalaInput {
            planets: vec![entry],
            is_daytime: false,
            moon_phase_waxing: false,
        };
        assert_eq!(validate(&input).unwrap_err().error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_shadbala");
        let required = def.input_schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("planets")));
    }
}
