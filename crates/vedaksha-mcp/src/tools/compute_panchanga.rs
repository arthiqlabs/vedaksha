// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_panchanga` — the five limbs of the Vedic almanac for an instant.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputePanchangaInput {
    /// Julian Day (UT). Determines the vara (weekday).
    pub jd: f64,
    /// Sidereal longitude of the Sun, degrees [0, 360).
    pub sun: f64,
    /// Sidereal longitude of the Moon, degrees [0, 360).
    pub moon: f64,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_panchanga",
        description: "Compute the panchanga — the five limbs of the Vedic almanac — for an \
            instant: tithi (lunar day, with paksha and lord), vara (weekday, with lord and Rahu \
            Kalam slot), nakshatra (with pada), yoga (one of the 27 nithya yogas, with degrees \
            remaining), and karana (half-tithi). Takes sidereal longitudes; the caller is \
            responsible for timezone conversion, as all times are UT.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "jd":   { "type": "number", "description": "Julian Day (UT), used for the vara" },
                "sun":  { "type": "number", "description": "Sidereal longitude of Sun [0, 360)" },
                "moon": { "type": "number", "description": "Sidereal longitude of Moon [0, 360)" }
            },
            "required": ["jd", "sun", "moon"]
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

/// Validate a [`ComputePanchangaInput`].
///
/// # Errors
/// Returns [`McpError`] when `jd` is non-finite or a longitude is outside [0, 360).
pub fn validate(input: &ComputePanchangaInput) -> Result<(), McpError> {
    if !input.jd.is_finite() {
        return Err(McpError::invalid_parameter("jd", "must be a finite number"));
    }
    validate_lon("sun", input.sun)?;
    validate_lon("moon", input.moon)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputePanchangaInput {
        ComputePanchangaInput {
            jd: 2_451_545.0,
            sun: 280.0,
            moon: 220.0,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_non_finite_jd() {
        let mut input = valid_input();
        input.jd = f64::INFINITY;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_longitude_at_360() {
        let mut input = valid_input();
        input.moon = 360.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_panchanga");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"jd"));
        assert!(names.contains(&"sun"));
        assert!(names.contains(&"moon"));
    }
}
