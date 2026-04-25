// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_natal_chart` — natal chart computation tool.

use serde::{Deserialize, Serialize};

use crate::validation::{self, McpError};

/// Input parameters for the `compute_natal_chart` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeNatalInput {
    /// Julian Day number (TDB).
    pub julian_day: f64,
    /// Geographic latitude in degrees \[-90, +90\].
    pub latitude: f64,
    /// Geographic longitude in degrees \[-180, +180\], east positive.
    pub longitude: f64,
    /// House system (e.g. `"Placidus"`, `"Koch"`, `"WholeSign"`).
    pub house_system: Option<String>,
    /// Ayanamsha for sidereal output (e.g. `"Lahiri"`, `"Tropical"`).
    pub ayanamsha: Option<String>,
}

/// Output of the `compute_natal_chart` tool.
#[derive(Debug, Clone, Serialize)]
pub struct ComputeNatalOutput {
    /// Full `ChartGraph` serialised to JSON.
    pub chart_json: serde_json::Value,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_natal_chart",
        description: "Compute a natal astrological chart for a given time and location. \
            Returns a ChartGraph in JSON format containing planetary positions, house cusps, \
            aspects, nakshatras, and dignities.",
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
                "house_system": {
                    "type": "string",
                    "description": "House system: Placidus, Koch, Equal, WholeSign, etc.",
                    "default": "Placidus"
                },
                "ayanamsha": {
                    "type": "string",
                    "description": "Ayanamsha system for sidereal: Lahiri, FaganBradley, Tropical, etc.",
                    "default": "Tropical"
                }
            },
            "required": ["julian_day", "latitude", "longitude"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeNatalInput) -> Result<(), McpError> {
    validation::validate_jd(input.julian_day)?;
    validation::validate_latitude(input.latitude)?;
    validation::validate_longitude(input.longitude)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{JD_MAX, JD_MIN};

    fn valid_input() -> ComputeNatalInput {
        ComputeNatalInput {
            julian_day: 2_451_545.0, // J2000
            latitude: 28.6,
            longitude: 77.2,
            house_system: None,
            ayanamsha: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_jd_out_of_range() {
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
        input.latitude = 95.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_rejects_invalid_longitude() {
        let mut input = valid_input();
        input.longitude = -200.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_natal_chart");
        let required = def.input_schema["required"].as_array().unwrap();
        let required_names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(required_names.contains(&"julian_day"));
        assert!(required_names.contains(&"latitude"));
        assert!(required_names.contains(&"longitude"));
    }
}
