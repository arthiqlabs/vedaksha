// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_drishti` — graha drishti (Vedic sign aspects) with graded strength.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeDrishtiInput {
    pub sun: f64,
    pub moon: f64,
    pub mars: f64,
    pub mercury: f64,
    pub jupiter: f64,
    pub venus: f64,
    pub saturn: f64,
    pub rahu: f64,
    pub ketu: f64,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_drishti",
        description: "Compute graha drishti — Vedic sign aspects — for all nine grahas. Unlike \
            Western aspects, drishti is cast from sign to sign and is asymmetric: every graha \
            aspects the 7th from itself, and Mars additionally aspects the 4th and 8th, Jupiter \
            the 5th and 9th, Saturn the 3rd and 10th. Returns each aspect with its graded \
            strength (Full, ThreeQuarter, Half, Quarter) and the house distance.",
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
                "rahu":    { "type": "number", "description": "Sidereal longitude of Rahu [0, 360)" },
                "ketu":    { "type": "number", "description": "Sidereal longitude of Ketu [0, 360)" }
            },
            "required": [
                "sun", "moon", "mars", "mercury", "jupiter",
                "venus", "saturn", "rahu", "ketu"
            ]
        }),
    }
}

fn validate_lon(name: &'static str, value: f64) -> Result<(), McpError> {
    if !value.is_finite() || !(0.0..360.0).contains(&value) {
        return Err(McpError::invalid_parameter(
            name,
            "must be a finite number in [0, 360)",
        ));
    }
    Ok(())
}

/// Validate a [`ComputeDrishtiInput`].
///
/// # Errors
/// Returns [`McpError`] when any longitude is non-finite or outside [0, 360).
pub fn validate(input: &ComputeDrishtiInput) -> Result<(), McpError> {
    validate_lon("sun", input.sun)?;
    validate_lon("moon", input.moon)?;
    validate_lon("mars", input.mars)?;
    validate_lon("mercury", input.mercury)?;
    validate_lon("jupiter", input.jupiter)?;
    validate_lon("venus", input.venus)?;
    validate_lon("saturn", input.saturn)?;
    validate_lon("rahu", input.rahu)?;
    validate_lon("ketu", input.ketu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeDrishtiInput {
        ComputeDrishtiInput {
            sun: 10.0,
            moon: 100.0,
            mars: 190.0,
            mercury: 20.0,
            jupiter: 130.0,
            venus: 40.0,
            saturn: 280.0,
            rahu: 60.0,
            ketu: 240.0,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_nan_longitude() {
        let mut input = valid_input();
        input.mars = f64::NAN;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_longitude_at_360() {
        let mut input = valid_input();
        input.ketu = 360.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn definition_requires_all_nine_grahas() {
        let def = definition();
        assert_eq!(def.name, "compute_drishti");
        let required = def.input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 9);
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"rahu"));
        assert!(names.contains(&"ketu"));
    }
}
