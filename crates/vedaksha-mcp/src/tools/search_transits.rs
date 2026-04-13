// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `search_transits` — search for transiting planet–natal planet aspects
//! within a time window.

use serde::{Deserialize, Serialize};

use crate::validation::{self, McpError};

/// A natal planet position for transit-to-natal aspect computation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NatalPosition {
    /// Planet name (e.g. `"Mars"`, `"Venus"`).
    pub name: String,
    /// Natal sidereal longitude in degrees \[0, 360\).
    pub longitude: f64,
}

/// Input parameters for the `search_transits` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchTransitsInput {
    /// Natal planet positions to check transits against.
    pub natal_positions: Vec<NatalPosition>,
    /// Start of the search window as a Julian Day (TDB).
    pub start_jd: f64,
    /// End of the search window as a Julian Day (TDB).
    pub end_jd: f64,
    /// Transiting bodies to include (e.g. `["Mars", "Jupiter"]`).
    /// Defaults to all planets when absent.
    pub bodies: Option<Vec<String>>,
    /// Aspect types to filter (e.g. `["conjunction", "opposition", "trine"]`).
    /// Defaults to major aspects when absent.
    pub aspects: Option<Vec<String>>,
    /// Maximum orb in degrees (default 1.0).
    pub max_orb: Option<f64>,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "search_transits",
        description: "Search for transiting planet–natal planet aspects within a time window. \
            Supply natal positions and a Julian Day range; receive a list of exact transit \
            moments with aspect type, orb, and applying/separating status.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "natal_positions": {
                    "type": "array",
                    "description": "Array of natal planet positions to check transits against",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Planet name (e.g. Mars, Venus)"
                            },
                            "longitude": {
                                "type": "number",
                                "description": "Natal sidereal longitude in degrees [0, 360)"
                            }
                        },
                        "required": ["name", "longitude"]
                    },
                    "minItems": 1
                },
                "start_jd": {
                    "type": "number",
                    "description": "Start of the search window as a Julian Day (TDB)"
                },
                "end_jd": {
                    "type": "number",
                    "description": "End of the search window as a Julian Day (TDB)"
                },
                "bodies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Transiting bodies to include. Defaults to all planets."
                },
                "aspects": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Aspect types to filter (e.g. conjunction, opposition, trine). Defaults to major aspects."
                },
                "max_orb": {
                    "type": "number",
                    "description": "Maximum orb in degrees (default 1.0)",
                    "default": 1.0
                }
            },
            "required": ["natal_positions", "start_jd", "end_jd"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &SearchTransitsInput) -> Result<(), McpError> {
    validation::validate_search_span(input.start_jd, input.end_jd)?;

    if input.natal_positions.is_empty() {
        return Err(McpError::invalid_parameter(
            "natal_positions",
            "at least one natal position must be provided",
        ));
    }

    for pos in &input.natal_positions {
        if pos.name.is_empty() {
            return Err(McpError::invalid_parameter(
                "natal_positions",
                "planet name must not be empty",
            ));
        }
        if !pos.longitude.is_finite() || !(0.0..360.0).contains(&pos.longitude) {
            return Err(McpError::invalid_parameter(
                "natal_positions",
                &format!(
                    "longitude {} for '{}' must be in [0, 360)",
                    pos.longitude, pos.name
                ),
            ));
        }
    }

    if let Some(orb) = input.max_orb {
        if !orb.is_finite() || orb <= 0.0 || orb > 30.0 {
            return Err(McpError::invalid_parameter(
                "max_orb",
                "max_orb must be a positive finite number not exceeding 30 degrees",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::JD_MIN;

    fn valid_input() -> SearchTransitsInput {
        SearchTransitsInput {
            natal_positions: vec![NatalPosition {
                name: "Mars".into(),
                longitude: 45.0,
            }],
            start_jd: 2_451_545.0,
            end_jd: 2_451_545.0 + 365.0,
            bodies: None,
            aspects: None,
            max_orb: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_jd_range_too_large() {
        let mut input = valid_input();
        input.end_jd = input.start_jd + 40_000.0; // > 100 years
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
    fn validate_rejects_empty_natal_positions() {
        let mut input = valid_input();
        input.natal_positions = vec![];
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_invalid_natal_longitude() {
        let mut input = valid_input();
        input.natal_positions[0].longitude = 400.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_zero_orb() {
        let mut input = valid_input();
        input.max_orb = Some(0.0);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"natal_positions"));
        assert!(names.contains(&"start_jd"));
        assert!(names.contains(&"end_jd"));
    }
}
