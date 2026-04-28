// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_karakas` — Jaimini Chara Karaka planet-role assignments.

use serde::{Deserialize, Serialize};

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeKarakasInput {
    pub sun: f64,
    pub moon: f64,
    pub mars: f64,
    pub mercury: f64,
    pub jupiter: f64,
    pub venus: f64,
    pub saturn: f64,
    /// Required when `scheme = "8"`.
    pub rahu: Option<f64>,
    /// `"7"` (default) or `"8"`.
    pub scheme: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComputeKarakasOutput {
    pub assignments: serde_json::Value,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_karakas",
        description: "Compute Jaimini Chara Karaka assignments from sidereal planet longitudes. \
            Ranks planets by degrees within their current sign: highest = Atmakaraka (soul \
            significator), lowest = Darakaraka (spouse significator). Supports 7-karaka \
            (Sun–Saturn) and 8-karaka (adds Rahu) schemes.",
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
                "rahu":    { "type": "number", "description": "Sidereal longitude of Rahu [0, 360). Required for scheme '8'." },
                "scheme":  {
                    "type": "string",
                    "description": "Karaka scheme: '7' (default, Sun–Saturn) or '8' (adds Rahu + Pitrikaraka)",
                    "enum": ["7", "8"],
                    "default": "7"
                }
            },
            "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"]
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

/// Validate a [`ComputeKarakasInput`].
///
/// # Errors
///
/// Returns [`McpError`] with `error_code = "INVALID_PARAMETER"` if any
/// longitude is out of range, if `scheme = "8"` is requested without `rahu`,
/// or if an unknown scheme string is supplied.
pub fn validate(input: &ComputeKarakasInput) -> Result<(), McpError> {
    validate_lon("sun", input.sun)?;
    validate_lon("moon", input.moon)?;
    validate_lon("mars", input.mars)?;
    validate_lon("mercury", input.mercury)?;
    validate_lon("jupiter", input.jupiter)?;
    validate_lon("venus", input.venus)?;
    validate_lon("saturn", input.saturn)?;

    if let Some(rahu) = input.rahu {
        validate_lon("rahu", rahu)?;
    }

    let scheme = input.scheme.as_deref().unwrap_or("7");
    match scheme {
        "7" => {}
        "8" => {
            if input.rahu.is_none() {
                return Err(McpError::invalid_parameter(
                    "rahu",
                    "must be provided when scheme is '8'",
                ));
            }
        }
        other => {
            return Err(McpError::invalid_parameter(
                "scheme",
                &format!("unknown scheme '{other}'; expected '7' or '8'"),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_7_input() -> ComputeKarakasInput {
        ComputeKarakasInput {
            sun: 25.0,
            moon: 20.0,
            mars: 15.0,
            mercury: 10.0,
            jupiter: 5.0,
            venus: 2.0,
            saturn: 1.0,
            rahu: None,
            scheme: None,
        }
    }

    #[test]
    fn validate_accepts_valid_7_input() {
        assert!(validate(&valid_7_input()).is_ok());
    }

    #[test]
    fn validate_rejects_nan_longitude() {
        let mut input = valid_7_input();
        input.sun = f64::NAN;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_longitude_out_of_range() {
        let mut input = valid_7_input();
        input.moon = 360.5;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_8_scheme_without_rahu() {
        let mut input = valid_7_input();
        input.scheme = Some("8".to_string());
        input.rahu = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_8_scheme_with_rahu() {
        let mut input = valid_7_input();
        input.scheme = Some("8".to_string());
        input.rahu = Some(180.0);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_scheme() {
        let mut input = valid_7_input();
        input.scheme = Some("9".to_string());
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_karakas");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"sun"));
        assert!(names.contains(&"moon"));
    }
}
