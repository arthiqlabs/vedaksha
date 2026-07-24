// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_bhavas` — whole-sign bhava chart with kendra/trikona classification.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeBhavasInput {
    /// Sidereal longitude of the ascendant, degrees [0, 360).
    pub ascendant: f64,
    /// Optional graha longitudes to place into bhavas, keyed by graha name.
    #[serde(default)]
    pub planets: std::collections::BTreeMap<String, f64>,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_bhavas",
        description: "Compute the whole-sign bhava (house) chart from an ascendant. In the Vedic \
            whole-sign system the entire sign containing the ascendant is the 1st bhava, the next \
            sign the 2nd, and so on — houses do not have cusps within signs. Returns the sign of \
            each of the twelve bhavas with its kendra / trikona / dusthana / upachaya \
            classification, and optionally places supplied grahas into their bhavas.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "ascendant": {
                    "type": "number",
                    "description": "Sidereal longitude of the ascendant [0, 360)"
                },
                "planets": {
                    "type": "object",
                    "description": "Optional map of graha name to sidereal longitude, \
                                    e.g. {\"Mars\": 200.4}. Each is placed into its bhava.",
                    "additionalProperties": { "type": "number" }
                }
            },
            "required": ["ascendant"]
        }),
    }
}

/// Validate a [`ComputeBhavasInput`].
///
/// # Errors
/// Returns [`McpError`] when the ascendant or any supplied graha longitude is
/// non-finite or outside [0, 360).
pub fn validate(input: &ComputeBhavasInput) -> Result<(), McpError> {
    if !input.ascendant.is_finite() || !(0.0..360.0).contains(&input.ascendant) {
        return Err(McpError::invalid_parameter(
            "ascendant",
            "must be a finite number in [0, 360)",
        ));
    }
    for (name, lon) in &input.planets {
        if !lon.is_finite() || !(0.0..360.0).contains(lon) {
            return Err(McpError::invalid_parameter(
                "planets",
                &format!("longitude for '{name}' must be a finite number in [0, 360)"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeBhavasInput {
        ComputeBhavasInput {
            ascendant: 95.0,
            planets: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_accepts_supplied_planets() {
        let mut input = valid_input();
        input.planets.insert("Mars".to_string(), 200.4);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_ascendant() {
        let mut input = valid_input();
        input.ascendant = 360.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_bad_planet_longitude() {
        let mut input = valid_input();
        input.planets.insert("Venus".to_string(), f64::NAN);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn definition_only_requires_ascendant() {
        let def = definition();
        assert_eq!(def.name, "compute_bhavas");
        let required = def.input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].as_str().unwrap(), "ascendant");
    }
}
