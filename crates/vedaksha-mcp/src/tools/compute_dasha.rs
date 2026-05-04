// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_dasha` — Vedic dasha period computation tool.
//!
//! Supports five classical systems: three Moon-longitude based
//! (Vimshottari, Ashtottari, Yogini) and two Lagna-sign based
//! (Chara, Narayana).

use serde::{Deserialize, Serialize};

use crate::validation::{self, McpError};

/// Maximum number of dasha levels supported (Moon-based systems).
const MAX_LEVELS: u8 = 5;

/// Dasha system selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashaSystem {
    Vimshottari,
    Ashtottari,
    Yogini,
    Chara,
    Narayana,
}

impl DashaSystem {
    pub fn parse(s: &str) -> Result<Self, McpError> {
        match s.to_ascii_lowercase().as_str() {
            "vimshottari" => Ok(Self::Vimshottari),
            "ashtottari" => Ok(Self::Ashtottari),
            "yogini" => Ok(Self::Yogini),
            "chara" => Ok(Self::Chara),
            "narayana" => Ok(Self::Narayana),
            _ => Err(McpError::invalid_parameter(
                "system",
                "must be one of: Vimshottari, Ashtottari, Yogini, Chara, Narayana",
            )),
        }
    }

    /// Whether this system uses the natal Moon's sidereal longitude
    /// (`true`) or the lagna sign (`false`) as its anchor.
    pub fn is_moon_based(self) -> bool {
        matches!(self, Self::Vimshottari | Self::Ashtottari | Self::Yogini)
    }
}

/// Input parameters for the `compute_dasha` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeDashaInput {
    /// Dasha system. Defaults to `"Vimshottari"`.
    pub system: Option<String>,
    /// Birth Julian Day (TDB) used as the dasha epoch.
    pub birth_jd: f64,
    /// Natal Moon sidereal longitude in degrees \[0, 360).
    /// Required for Vimshottari, Ashtottari, Yogini.
    pub moon_longitude: Option<f64>,
    /// Lagna (ascendant) sign 1–12 (1 = Aries).
    /// Required for Chara, Narayana.
    pub lagna_sign: Option<u8>,
    /// Number of nested dasha levels (1–5). Defaults to 3.
    /// Ignored by Chara and Narayana, which return a single level.
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
        description: "Compute Vedic dasha (planetary period) sequences. Supports five \
            classical systems: Vimshottari, Ashtottari, Yogini (Moon-longitude based, \
            require `moon_longitude`); Chara, Narayana (Lagna-sign based, require \
            `lagna_sign`). Returns a JSON dasha tree with start/end Julian Days.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "system": {
                    "type": "string",
                    "description": "Dasha system selector",
                    "enum": ["Vimshottari", "Ashtottari", "Yogini", "Chara", "Narayana"],
                    "default": "Vimshottari"
                },
                "birth_jd": {
                    "type": "number",
                    "description": "Birth Julian Day (TDB) used as the dasha epoch"
                },
                "moon_longitude": {
                    "type": "number",
                    "description": "Natal Moon sidereal longitude in degrees [0, 360). Required for Vimshottari, Ashtottari, Yogini.",
                    "minimum": 0,
                    "maximum": 360
                },
                "lagna_sign": {
                    "type": "integer",
                    "description": "Lagna (ascendant) sign 1–12 (1 = Aries). Required for Chara, Narayana.",
                    "minimum": 1,
                    "maximum": 12
                },
                "levels": {
                    "type": "integer",
                    "description": "Number of nested dasha levels (1–5). Ignored by Chara and Narayana.",
                    "minimum": 1,
                    "maximum": 5,
                    "default": 3
                }
            },
            "required": ["birth_jd"]
        }),
    }
}

/// Resolve the system selector and validate every input field.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeDashaInput) -> Result<DashaSystem, McpError> {
    let system = match &input.system {
        Some(s) => DashaSystem::parse(s)?,
        None => DashaSystem::Vimshottari,
    };

    validation::validate_jd(input.birth_jd)?;

    if system.is_moon_based() {
        let moon = input.moon_longitude.ok_or_else(|| {
            McpError::invalid_parameter(
                "moon_longitude",
                "required for Vimshottari, Ashtottari, and Yogini",
            )
        })?;
        if !moon.is_finite() || !(0.0..360.0).contains(&moon) {
            return Err(McpError::invalid_parameter(
                "moon_longitude",
                "must be a finite number in [0, 360)",
            ));
        }
    } else {
        let sign = input.lagna_sign.ok_or_else(|| {
            McpError::invalid_parameter("lagna_sign", "required for Chara and Narayana")
        })?;
        if !(1..=12).contains(&sign) {
            return Err(McpError::invalid_parameter(
                "lagna_sign",
                "must be an integer in [1, 12]",
            ));
        }
    }

    if let Some(levels) = input.levels {
        if levels == 0 || levels > MAX_LEVELS {
            return Err(McpError::invalid_parameter(
                "levels",
                &format!("must be between 1 and {MAX_LEVELS}"),
            ));
        }
    }

    Ok(system)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moon_input() -> ComputeDashaInput {
        ComputeDashaInput {
            system: None,
            birth_jd: 2_451_545.0, // J2000
            moon_longitude: Some(123.45),
            lagna_sign: None,
            levels: None,
        }
    }

    fn lagna_input() -> ComputeDashaInput {
        ComputeDashaInput {
            system: Some("Chara".to_string()),
            birth_jd: 2_451_545.0,
            moon_longitude: None,
            lagna_sign: Some(1),
            levels: None,
        }
    }

    #[test]
    fn validate_accepts_default_vimshottari() {
        let s = validate(&moon_input()).unwrap();
        assert_eq!(s, DashaSystem::Vimshottari);
    }

    #[test]
    fn validate_accepts_explicit_ashtottari() {
        let mut input = moon_input();
        input.system = Some("Ashtottari".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Ashtottari);
    }

    #[test]
    fn validate_accepts_yogini_case_insensitive() {
        let mut input = moon_input();
        input.system = Some("yogini".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Yogini);
    }

    #[test]
    fn validate_accepts_chara_with_lagna() {
        assert_eq!(validate(&lagna_input()).unwrap(), DashaSystem::Chara);
    }

    #[test]
    fn validate_accepts_narayana_with_lagna() {
        let mut input = lagna_input();
        input.system = Some("Narayana".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Narayana);
    }

    #[test]
    fn validate_rejects_unknown_system() {
        let mut input = moon_input();
        input.system = Some("DoesNotExist".to_string());
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_moon_based_without_moon_longitude() {
        let mut input = moon_input();
        input.moon_longitude = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_lagna_based_without_lagna_sign() {
        let mut input = lagna_input();
        input.lagna_sign = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_moon_longitude_below_zero() {
        let mut input = moon_input();
        input.moon_longitude = Some(-1.0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_moon_longitude_equal_to_360() {
        let mut input = moon_input();
        input.moon_longitude = Some(360.0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_moon_longitude_nan() {
        let mut input = moon_input();
        input.moon_longitude = Some(f64::NAN);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_lagna_sign_zero() {
        let mut input = lagna_input();
        input.lagna_sign = Some(0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_lagna_sign_above_twelve() {
        let mut input = lagna_input();
        input.lagna_sign = Some(13);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_birth_jd_out_of_range() {
        let mut input = moon_input();
        input.birth_jd = 0.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "DATE_OUT_OF_RANGE"
        );
    }

    #[test]
    fn validate_rejects_levels_zero() {
        let mut input = moon_input();
        input.levels = Some(0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_levels_above_max() {
        let mut input = moon_input();
        input.levels = Some(6);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_accepts_max_levels() {
        let mut input = moon_input();
        input.levels = Some(5);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_dasha");
        let required: Vec<&str> = def.input_schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"birth_jd"));
        // moon_longitude / lagna_sign are conditional on `system` and not in
        // the JSON-Schema `required` list — agents discover the requirement
        // through the field descriptions.
        let systems = def.input_schema["properties"]["system"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(systems.len(), 5);
    }
}
