// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_combustion` — per-planet combustion state relative to the Sun.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeCombustionInput {
    pub sun: f64,
    pub moon: f64,
    pub mars: f64,
    pub mercury: f64,
    pub jupiter: f64,
    pub venus: f64,
    pub saturn: f64,
    #[serde(default)]
    pub mercury_retrograde: bool,
    #[serde(default)]
    pub venus_retrograde: bool,
    #[serde(default)]
    pub mars_retrograde: bool,
    #[serde(default)]
    pub jupiter_retrograde: bool,
    #[serde(default)]
    pub saturn_retrograde: bool,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_combustion",
        description: "Compute combustion state for each planet relative to the Sun per BPHS Ch.7 \
            vv.28-29. Returns Combust, DeeplyCombust, or None for Moon, Mars, Mercury, Jupiter, \
            Venus, Saturn with degrees of separation.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "sun":     { "type": "number", "description": "Sidereal longitude of Sun [0, 360)" },
                "moon":    { "type": "number", "description": "Sidereal longitude of Moon [0, 360)" },
                "mars":    { "type": "number", "description": "Sidereal longitude of Mars [0, 360)" },
                "mercury": { "type": "number", "description": "Sidereal longitude of Mercury [0, 360)" },
                "jupiter": { "type": "number", "description": "Sidereal longitude of Jupiter [0, 360)" },
                "venus":   { "type": "number", "description": "Sidereal longitude of Venus [0, 360)" },
                "saturn":  { "type": "number", "description": "Sidereal longitude of Saturn [0, 360)" },
                "mercury_retrograde": { "type": "boolean", "default": false },
                "venus_retrograde":   { "type": "boolean", "default": false },
                "mars_retrograde":    { "type": "boolean", "default": false },
                "jupiter_retrograde": { "type": "boolean", "default": false },
                "saturn_retrograde":  { "type": "boolean", "default": false }
            },
            "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"]
        }),
    }
}

fn validate_lon(name: &'static str, value: f64) -> Result<(), McpError> {
    if !value.is_finite() || !(0.0..360.0).contains(&value) {
        return Err(McpError::invalid_parameter(name, "must be a finite number in [0, 360)"));
    }
    Ok(())
}

/// Validate a [`ComputeCombustionInput`].
///
/// # Errors
/// Returns [`McpError`] when any longitude is non-finite or outside [0, 360).
pub fn validate(input: &ComputeCombustionInput) -> Result<(), McpError> {
    validate_lon("sun", input.sun)?;
    validate_lon("moon", input.moon)?;
    validate_lon("mars", input.mars)?;
    validate_lon("mercury", input.mercury)?;
    validate_lon("jupiter", input.jupiter)?;
    validate_lon("venus", input.venus)?;
    validate_lon("saturn", input.saturn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeCombustionInput {
        ComputeCombustionInput {
            sun: 123.4,
            moon: 45.6,
            mars: 200.1,
            mercury: 310.5,
            jupiter: 5.3,
            venus: 92.1,
            saturn: 250.8,
            mercury_retrograde: false,
            venus_retrograde: false,
            mars_retrograde: false,
            jupiter_retrograde: false,
            saturn_retrograde: false,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_nan_longitude() {
        let mut input = valid_input();
        input.sun = f64::NAN;
        assert_eq!(validate(&input).unwrap_err().error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_longitude_at_360() {
        let mut input = valid_input();
        input.moon = 360.0;
        assert_eq!(validate(&input).unwrap_err().error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_combustion");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"sun"));
        assert!(names.contains(&"moon"));
    }
}
