// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_dasha` — Vedic dasha period computation tool.

use serde::{Deserialize, Serialize};

use crate::validation::{self, McpError};

/// Maximum number of dasha levels supported.
const MAX_LEVELS: u8 = 5;

/// Input parameters for the `compute_dasha` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeDashaInput {
    /// Natal Moon longitude in degrees \[0, 360).
    pub moon_longitude: f64,
    /// Birth Julian Day (TDB) used as the dasha epoch.
    pub birth_jd: f64,
    /// Dasha system (e.g. `"Vimshottari"`, `"Ashtottari"`).
    pub system: Option<String>,
    /// Number of dasha levels to compute (1–5).  Defaults to 3.
    pub levels: Option<u8>,
}

/// Output of the `compute_dasha` tool.
#[derive(Debug, Clone, Serialize)]
pub struct ComputeDashaOutput {
    /// Hierarchical dasha periods serialised to JSON.
    pub dasha_json: serde_json::Value,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_dasha",
        description: "Compute Vedic dasha (planetary period) sequences from a natal Moon \
            longitude and birth epoch. Returns a hierarchical JSON structure of dasha, \
            antardasha, and pratyantardasha periods with start/end Julian Days.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "moon_longitude": {
                    "type": "number",
                    "description": "Natal Moon longitude in degrees [0, 360)"
                },
                "birth_jd": {
                    "type": "number",
                    "description": "Birth Julian Day number (TDB) used as dasha epoch"
                },
                "system": {
                    "type": "string",
                    "description": "Dasha system: Vimshottari, Ashtottari, etc.",
                    "default": "Vimshottari"
                },
                "levels": {
                    "type": "integer",
                    "description": "Number of dasha levels to compute (1–5)",
                    "minimum": 1,
                    "maximum": 5,
                    "default": 3
                }
            },
            "required": ["moon_longitude", "birth_jd"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeDashaInput) -> Result<(), McpError> {
    // Moon longitude must be in [0, 360).
    if !input.moon_longitude.is_finite()
        || input.moon_longitude < 0.0
        || input.moon_longitude >= 360.0
    {
        return Err(McpError::invalid_parameter(
            "moon_longitude",
            "must be a finite number in [0, 360)",
        ));
    }

    validation::validate_jd(input.birth_jd)?;

    if let Some(levels) = input.levels {
        if levels == 0 || levels > MAX_LEVELS {
            return Err(McpError::invalid_parameter(
                "levels",
                &format!("must be between 1 and {MAX_LEVELS}"),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeDashaInput {
        ComputeDashaInput {
            moon_longitude: 123.45,
            birth_jd: 2_451_545.0, // J2000
            system: None,
            levels: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_moon_longitude_below_zero() {
        let mut input = valid_input();
        input.moon_longitude = -1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_moon_longitude_equal_to_360() {
        let mut input = valid_input();
        input.moon_longitude = 360.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_moon_longitude_nan() {
        let mut input = valid_input();
        input.moon_longitude = f64::NAN;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_birth_jd_out_of_range() {
        let mut input = valid_input();
        input.birth_jd = 0.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_levels_zero() {
        let mut input = valid_input();
        input.levels = Some(0);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_levels_above_max() {
        let mut input = valid_input();
        input.levels = Some(6);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_max_levels() {
        let mut input = valid_input();
        input.levels = Some(5);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_dasha");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"moon_longitude"));
        assert!(names.contains(&"birth_jd"));
    }
}
