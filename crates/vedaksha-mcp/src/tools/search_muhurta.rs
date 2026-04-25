// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `search_muhurta` — search for auspicious time windows (muhurta) within a
//! given period.

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Input parameters for the `search_muhurta` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchMuhurtaInput {
    /// Start of the search window as a Julian Day (TDB).
    pub start_jd: f64,
    /// End of the search window as a Julian Day (TDB).
    pub end_jd: f64,
    /// Geographic latitude in degrees \[-90, +90\].
    pub latitude: f64,
    /// Geographic longitude in degrees \[-180, +180\], east positive.
    pub longitude: f64,
    /// Minimum quality score (0.0–1.0) for a muhurta to be included.
    /// Defaults to 0.5 when absent.
    pub min_quality: Option<f64>,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "search_muhurta",
        description: "Search for auspicious time windows (muhurta) within a given period for a \
            geographic location. Returns ranked muhurta candidates with quality scores based on \
            tithi, nakshatra, yoga, karana, and planetary positions.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "start_jd": {
                    "type": "number",
                    "description": "Start of the search window as a Julian Day (TDB)"
                },
                "end_jd": {
                    "type": "number",
                    "description": "End of the search window as a Julian Day (TDB)"
                },
                "latitude": {
                    "type": "number",
                    "description": "Geographic latitude in degrees [-90, +90]"
                },
                "longitude": {
                    "type": "number",
                    "description": "Geographic longitude in degrees [-180, +180], east positive"
                },
                "min_quality": {
                    "type": "number",
                    "description": "Minimum quality score [0.0, 1.0] for muhurta inclusion (default 0.5)",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.5
                }
            },
            "required": ["start_jd", "end_jd", "latitude", "longitude"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &SearchMuhurtaInput) -> Result<(), McpError> {
    validation::validate_search_span(input.start_jd, input.end_jd)?;
    validation::validate_latitude(input.latitude)?;
    validation::validate_longitude(input.longitude)?;

    if let Some(q) = input.min_quality {
        if !q.is_finite() || !(0.0..=1.0).contains(&q) {
            return Err(McpError::invalid_parameter(
                "min_quality",
                "min_quality must be a finite number in [0.0, 1.0]",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::JD_MIN;

    fn valid_input() -> SearchMuhurtaInput {
        SearchMuhurtaInput {
            start_jd: 2_451_545.0,
            end_jd: 2_451_545.0 + 30.0,
            latitude: 13.08,
            longitude: 80.27,
            min_quality: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_invalid_latitude() {
        let mut input = valid_input();
        input.latitude = 91.0;
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
    fn validate_rejects_jd_range_too_large() {
        let mut input = valid_input();
        input.end_jd = input.start_jd + 40_000.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "SEARCH_RANGE_TOO_LARGE");
    }

    #[test]
    fn validate_rejects_start_jd_below_min() {
        let mut input = valid_input();
        input.start_jd = JD_MIN - 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_min_quality_out_of_range() {
        let mut input = valid_input();
        input.min_quality = Some(1.5);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_min_quality_zero() {
        let mut input = valid_input();
        input.min_quality = Some(0.0);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"start_jd"));
        assert!(names.contains(&"end_jd"));
        assert!(names.contains(&"latitude"));
        assert!(names.contains(&"longitude"));
    }
}
