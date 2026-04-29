// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_gochara` — Gochara (transit interpretation) per BPHS Ch.29.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeGocharaInput {
    pub sun: u8,
    pub moon: u8,
    pub mars: u8,
    pub mercury: u8,
    pub jupiter: u8,
    pub venus: u8,
    pub saturn: u8,
    /// Sign index (0–11) of the natal reference point. The caller chooses
    /// whether this is the natal Moon's sign or the natal Lagna's sign.
    pub natal_reference_sign: u8,
    /// Optional vedha pair table. Currently `"Bphs29"` is the only
    /// supported value; defaults to `"Bphs29"` when absent.
    pub vedha_table: Option<String>,
    /// Optional school exemption profile applied to the raw vedha
    /// candidate list. `"Geometry"` (default) returns raw candidates;
    /// `"Parashari"` strips Sun/Moon and Jupiter/Mercury mutual vedha.
    pub school: Option<String>,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_gochara",
        description: "Compute Gochara (transit interpretation) for the seven grahas against a \
            natal reference sign per BPHS Ch.29. Returns favourable/unfavourable verdict, \
            house from natal, and raw vedha (obstruction) candidates per planet. The natal \
            reference sign is the caller's choice — typically the natal Moon's sign \
            (Chandra Gochara) or the natal Lagna's sign. Rahu and Ketu are not included.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "sun":     { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Sun (0=Aries … 11=Pisces)" },
                "moon":    { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Moon" },
                "mars":    { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Mars" },
                "mercury": { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Mercury" },
                "jupiter": { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Jupiter" },
                "venus":   { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Venus" },
                "saturn":  { "type": "integer", "minimum": 0, "maximum": 11, "description": "Transit sign index of Saturn" },
                "natal_reference_sign": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 11,
                    "description": "Sign index of the natal reference point — natal Moon for Chandra Gochara, natal Lagna for Lagna-based Gochara"
                },
                "vedha_table": {
                    "type": "string",
                    "enum": ["Bphs29"],
                    "default": "Bphs29",
                    "description": "Vedha pair table source"
                },
                "school": {
                    "type": "string",
                    "enum": ["Geometry", "Parashari"],
                    "default": "Geometry",
                    "description": "Exemption profile applied to the raw vedha candidate list"
                }
            },
            "required": [
                "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn",
                "natal_reference_sign"
            ]
        }),
    }
}

fn validate_sign(name: &'static str, value: u8) -> Result<(), McpError> {
    if value > 11 {
        return Err(McpError::invalid_parameter(
            name,
            "must be a sign index in [0, 11]",
        ));
    }
    Ok(())
}

/// Validate a [`ComputeGocharaInput`].
///
/// # Errors
///
/// Returns [`McpError`] when any sign index is out of range or when an
/// unknown table or school string is supplied.
pub fn validate(input: &ComputeGocharaInput) -> Result<(), McpError> {
    validate_sign("sun", input.sun)?;
    validate_sign("moon", input.moon)?;
    validate_sign("mars", input.mars)?;
    validate_sign("mercury", input.mercury)?;
    validate_sign("jupiter", input.jupiter)?;
    validate_sign("venus", input.venus)?;
    validate_sign("saturn", input.saturn)?;
    validate_sign("natal_reference_sign", input.natal_reference_sign)?;

    if let Some(table) = input.vedha_table.as_deref() {
        match table {
            "Bphs29" => {}
            other => {
                return Err(McpError::invalid_parameter(
                    "vedha_table",
                    &format!("unknown table '{other}'; expected 'Bphs29'"),
                ));
            }
        }
    }

    if let Some(school) = input.school.as_deref() {
        match school {
            "Geometry" | "Parashari" => {}
            other => {
                return Err(McpError::invalid_parameter(
                    "school",
                    &format!("unknown school '{other}'; expected 'Geometry' or 'Parashari'"),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeGocharaInput {
        ComputeGocharaInput {
            sun: 0,
            moon: 4,
            mars: 2,
            mercury: 6,
            jupiter: 8,
            venus: 10,
            saturn: 6,
            natal_reference_sign: 0,
            vedha_table: None,
            school: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_sign_out_of_range() {
        let mut input = valid_input();
        input.moon = 12;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_unknown_table() {
        let mut input = valid_input();
        input.vedha_table = Some("Saravali".into());
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_unknown_school() {
        let mut input = valid_input();
        input.school = Some("KP".into());
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_gochara");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"natal_reference_sign"));
        assert!(names.contains(&"sun"));
    }
}
