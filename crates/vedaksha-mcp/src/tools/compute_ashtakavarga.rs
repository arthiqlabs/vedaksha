// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_ashtakavarga` — Bhinna Ashtakavarga and Sarvashtakavarga.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeAshtakavargaInput {
    /// Sign index of Sun (0 = Aries … 11 = Pisces).
    pub sun: u8,
    /// Sign index of Moon.
    pub moon: u8,
    /// Sign index of Mars.
    pub mars: u8,
    /// Sign index of Mercury.
    pub mercury: u8,
    /// Sign index of Jupiter.
    pub jupiter: u8,
    /// Sign index of Venus.
    pub venus: u8,
    /// Sign index of Saturn.
    pub saturn: u8,
    /// Sign index of Lagna (Ascendant).
    pub lagna: u8,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_ashtakavarga",
        description: "Compute Bhinna Ashtakavarga (raw bindu tables) and Sarvashtakavarga \
            for all 7 planets from sign positions. Source: BPHS Ch.66 vv.13-68. \
            Trikona/Ekadhipatya Shodhana and Pinda Sadhana are not included.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "sun":     { "type": "integer", "minimum": 0, "maximum": 11, "description": "Sign index of Sun (0=Aries…11=Pisces)" },
                "moon":    { "type": "integer", "minimum": 0, "maximum": 11 },
                "mars":    { "type": "integer", "minimum": 0, "maximum": 11 },
                "mercury": { "type": "integer", "minimum": 0, "maximum": 11 },
                "jupiter": { "type": "integer", "minimum": 0, "maximum": 11 },
                "venus":   { "type": "integer", "minimum": 0, "maximum": 11 },
                "saturn":  { "type": "integer", "minimum": 0, "maximum": 11 },
                "lagna":   { "type": "integer", "minimum": 0, "maximum": 11, "description": "Sign index of Lagna (Ascendant)" }
            },
            "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "lagna"]
        }),
    }
}

fn validate_sign(name: &'static str, value: u8) -> Result<(), McpError> {
    if value > 11 {
        return Err(McpError::invalid_parameter(name, "must be 0–11 (Aries=0 … Pisces=11)"));
    }
    Ok(())
}

/// Validate a [`ComputeAshtakavargaInput`].
///
/// # Errors
/// Returns [`McpError`] when any sign index is > 11.
pub fn validate(input: &ComputeAshtakavargaInput) -> Result<(), McpError> {
    validate_sign("sun",     input.sun)?;
    validate_sign("moon",    input.moon)?;
    validate_sign("mars",    input.mars)?;
    validate_sign("mercury", input.mercury)?;
    validate_sign("jupiter", input.jupiter)?;
    validate_sign("venus",   input.venus)?;
    validate_sign("saturn",  input.saturn)?;
    validate_sign("lagna",   input.lagna)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeAshtakavargaInput {
        ComputeAshtakavargaInput {
            sun: 3, moon: 7, mars: 1,
            mercury: 10, jupiter: 5,
            venus: 8, saturn: 11, lagna: 0,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_sign_12() {
        let mut input = valid_input();
        input.sun = 12;
        assert_eq!(validate(&input).unwrap_err().error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_all_zeros() {
        let input = ComputeAshtakavargaInput {
            sun: 0, moon: 0, mars: 0, mercury: 0,
            jupiter: 0, venus: 0, saturn: 0, lagna: 0,
        };
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_eight_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_ashtakavarga");
        let required = def.input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 8);
    }
}
