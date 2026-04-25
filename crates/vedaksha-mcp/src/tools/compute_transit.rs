// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_transit` — compute transiting planet positions relative to a
//! natal chart at a given moment.

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Input parameters for the `compute_transit` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeTransitInput {
    /// Julian Day of the natal chart (TDB).
    pub natal_jd: f64,
    /// Natal geographic latitude in degrees \[-90, +90\].
    pub natal_lat: f64,
    /// Natal geographic longitude in degrees \[-180, +180\], east positive.
    pub natal_lon: f64,
    /// Julian Day of the transit moment to compute (TDB).
    pub transit_jd: f64,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_transit",
        description: "Compute transiting planet positions relative to a natal chart at a specific \
            moment. Returns planet longitudes for the transit time alongside natal positions, \
            enabling aspect calculation between transit and natal placements.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "natal_jd": {
                    "type": "number",
                    "description": "Julian Day of the natal chart (TDB)"
                },
                "natal_lat": {
                    "type": "number",
                    "description": "Natal geographic latitude in degrees [-90, +90]"
                },
                "natal_lon": {
                    "type": "number",
                    "description": "Natal geographic longitude in degrees [-180, +180], east positive"
                },
                "transit_jd": {
                    "type": "number",
                    "description": "Julian Day of the transit moment to compute (TDB)"
                }
            },
            "required": ["natal_jd", "natal_lat", "natal_lon", "transit_jd"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeTransitInput) -> Result<(), McpError> {
    validation::validate_jd(input.natal_jd)?;
    validation::validate_latitude(input.natal_lat)?;
    validation::validate_longitude(input.natal_lon)?;
    validation::validate_jd(input.transit_jd)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{JD_MAX, JD_MIN};

    fn valid_input() -> ComputeTransitInput {
        ComputeTransitInput {
            natal_jd: 2_451_545.0,
            natal_lat: 28.6,
            natal_lon: 77.2,
            transit_jd: 2_451_910.0,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_natal_jd_below_min() {
        let mut input = valid_input();
        input.natal_jd = JD_MIN - 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_transit_jd_above_max() {
        let mut input = valid_input();
        input.transit_jd = JD_MAX + 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_invalid_natal_lat() {
        let mut input = valid_input();
        input.natal_lat = 95.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_rejects_invalid_natal_lon() {
        let mut input = valid_input();
        input.natal_lon = -200.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"natal_jd"));
        assert!(names.contains(&"natal_lat"));
        assert!(names.contains(&"natal_lon"));
        assert!(names.contains(&"transit_jd"));
    }
}
