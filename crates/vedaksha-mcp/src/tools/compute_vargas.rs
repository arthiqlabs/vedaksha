// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_vargas` — Vedic divisional chart (varga) computation tool.

use serde::{Deserialize, Serialize};

use crate::validation::{self, McpError};

/// Input parameters for the `compute_vargas` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeVargasInput {
    /// Julian Day number (TDB).
    pub julian_day: f64,
    /// Geographic latitude in degrees \[-90, +90\].
    pub latitude: f64,
    /// Geographic longitude in degrees \[-180, +180\], east positive.
    pub longitude: f64,
    /// Sidereal longitude of the planet in degrees \[0, 360\) for direct
    /// varga computation without a full ephemeris lookup.
    pub planet_longitude: Option<f64>,
    /// List of varga division codes to compute, e.g. `["D1", "D9", "D10"]`.
    pub divisions: Vec<String>,
    /// Ayanamsha system (e.g. `"Lahiri"`, `"Tropical"`).
    pub ayanamsha: Option<String>,
}

/// Output of the `compute_vargas` tool.
#[derive(Debug, Clone, Serialize)]
pub struct ComputeVargasOutput {
    /// Map from division code to its `ChartGraph` JSON representation.
    pub vargas_json: serde_json::Value,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_vargas",
        description: "Compute Vedic divisional charts (vargas) for a given time and location. \
            Supply a list of division codes (e.g. D1, D9, D10) and receive a ChartGraph JSON \
            for each, including planetary positions and dignities within each varga.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "julian_day": {
                    "type": "number",
                    "description": "Julian Day number (TDB)"
                },
                "latitude": {
                    "type": "number",
                    "description": "Geographic latitude in degrees [-90, +90]"
                },
                "longitude": {
                    "type": "number",
                    "description": "Geographic longitude in degrees [-180, +180], east positive"
                },
                "divisions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Varga division codes to compute, e.g. [\"D1\", \"D9\", \"D10\"]",
                    "minItems": 1
                },
                "planet_longitude": {
                    "type": "number",
                    "description": "Sidereal longitude of the planet in degrees [0, 360) for direct varga computation"
                },
                "ayanamsha": {
                    "type": "string",
                    "description": "Ayanamsha system for sidereal: Lahiri, FaganBradley, Tropical, etc.",
                    "default": "Lahiri"
                }
            },
            "required": ["julian_day", "latitude", "longitude", "divisions"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeVargasInput) -> Result<(), McpError> {
    validation::validate_jd(input.julian_day)?;
    validation::validate_latitude(input.latitude)?;
    validation::validate_longitude(input.longitude)?;

    if input.divisions.is_empty() {
        return Err(McpError::invalid_parameter(
            "divisions",
            "at least one division code must be provided",
        ));
    }

    for div in &input.divisions {
        if div.is_empty() {
            return Err(McpError::invalid_parameter(
                "divisions",
                "division codes must not be empty strings",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{JD_MAX, JD_MIN};

    fn valid_input() -> ComputeVargasInput {
        ComputeVargasInput {
            julian_day: 2_451_545.0,
            latitude: 13.08,
            longitude: 80.27,
            planet_longitude: None,
            divisions: vec!["D1".into(), "D9".into()],
            ayanamsha: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_jd_below_min() {
        let mut input = valid_input();
        input.julian_day = JD_MIN - 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_jd_above_max() {
        let mut input = valid_input();
        input.julian_day = JD_MAX + 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_invalid_latitude() {
        let mut input = valid_input();
        input.latitude = -91.5;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_rejects_invalid_longitude() {
        let mut input = valid_input();
        input.longitude = 200.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    #[test]
    fn validate_rejects_empty_divisions_list() {
        let mut input = valid_input();
        input.divisions = vec![];
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_empty_division_code() {
        let mut input = valid_input();
        input.divisions = vec!["D1".into(), String::new()];
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_requires_divisions() {
        let def = definition();
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"divisions"));
    }
}
