// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_dasha` — Vedic dasha period computation tool.
//!
//! Supports five classical systems: three Moon-longitude based
//! (Vimshottari, Ashtottari, Yogini) and two Lagna-sign based
//! (Chara, Narayana).

use serde::Deserialize;

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
    /// Parse a case-insensitive system name.
    ///
    /// # Errors
    /// Returns [`McpError::invalid_parameter`] if `s` does not match one of
    /// `Vimshottari`, `Ashtottari`, `Yogini`, `Chara`, or `Narayana`.
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
    #[must_use]
    pub fn is_moon_based(self) -> bool {
        matches!(self, Self::Vimshottari | Self::Ashtottari | Self::Yogini)
    }
}

/// Natal sign positions (0 = Aries .. 11 = Pisces) of the seven classical
/// grahas plus Rahu, as served by `compute_natal_chart`'s `sign_index`
/// field. Required, with every field, when `system` is `Chara` or
/// `Narayana` — their dasha durations are chart-dependent (see
/// [`crate::server`]'s `compute_dasha` dispatch and
/// `vedaksha_vedic::dasha::chara`'s module doc for the rule).
///
/// Ketu is deliberately NOT a field here: it is derived internally as
/// `(rahu + 6) % 12`, since the two lunar nodes are always exactly
/// opposite by definition and accepting both invites an
/// internally-inconsistent chart.
///
/// Every field is `Option` — not because any of them is optional (all eight
/// are required whenever `graha_signs` itself is required) — but so a
/// caller who omits one field gets a specific "which field" error from
/// [`validate`] rather than a generic JSON-deserialization failure that
/// does not name the missing field.
/// Natal sign of each graha, as supplied by the caller.
///
/// Fields are `Option<i32>`, not `u8`, deliberately. `u8` would reject a
/// negative value inside `serde` before [`validate`] ever runs, yielding
/// `invalid value: integer -1, expected u8` with no field name — and `-1` is
/// a plausible caller mistake, being exactly what an off-by-one from a
/// 1-indexed source produces. Widening the parse lets every range error reach
/// [`validate`] and come back naming the offending graha.
///
/// `deny_unknown_fields` is load-bearing rather than tidiness: `ketu` is
/// deliberately absent because it is derived from `rahu`, and silently
/// ignoring a supplied `ketu` would accept an internally inconsistent chart
/// while appearing to honour it. Rejecting it says so.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrahaSignsInput {
    /// Sun's natal sign, 0–11 (0 = Aries).
    pub sun: Option<i32>,
    /// Moon's natal sign, 0–11 (0 = Aries).
    pub moon: Option<i32>,
    /// Mars's natal sign, 0–11 (0 = Aries).
    pub mars: Option<i32>,
    /// Mercury's natal sign, 0–11 (0 = Aries).
    pub mercury: Option<i32>,
    /// Jupiter's natal sign, 0–11 (0 = Aries).
    pub jupiter: Option<i32>,
    /// Venus's natal sign, 0–11 (0 = Aries).
    pub venus: Option<i32>,
    /// Saturn's natal sign, 0–11 (0 = Aries).
    pub saturn: Option<i32>,
    /// Rahu's (north lunar node's) natal sign, 0–11 (0 = Aries).
    pub rahu: Option<i32>,
}

impl GrahaSignsInput {
    /// Convert to the library's [`vedaksha_vedic::dasha::chara::GrahaSigns`].
    ///
    /// # Panics
    /// Panics if any field is `None` or outside `[0, 11]`. Only call after
    /// [`validate`] has confirmed both — the same "validated above" contract
    /// `moon_longitude` and `lagna_sign` already rely on below.
    #[must_use]
    pub fn into_graha_signs(self) -> vedaksha_vedic::dasha::chara::GrahaSigns {
        // `validate` bounds every field to [0, 11], so each cast is exact.
        let sign = |v: Option<i32>| -> u8 {
            u8::try_from(v.expect("validated above")).expect("validated above")
        };
        vedaksha_vedic::dasha::chara::GrahaSigns {
            sun: sign(self.sun),
            moon: sign(self.moon),
            mars: sign(self.mars),
            mercury: sign(self.mercury),
            jupiter: sign(self.jupiter),
            venus: sign(self.venus),
            saturn: sign(self.saturn),
            rahu: sign(self.rahu),
        }
    }
}

/// Input parameters for the `compute_dasha` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeDashaInput {
    /// Dasha system. Defaults to `"Vimshottari"`.
    pub system: Option<String>,
    /// Birth Julian Day used as the dasha epoch, in **UT1** (Universal
    /// Time) — not TT, not TDB, i.e. the same scale as
    /// `compute_natal_chart`'s `julian_day`.
    ///
    /// This epoch is only added to, never converted, so every returned
    /// `start_jd`/`end_jd` is on the same UT1 scale as the input.
    pub birth_jd: f64,
    /// Natal Moon sidereal longitude in degrees \[0, 360).
    /// Required for Vimshottari, Ashtottari, Yogini.
    pub moon_longitude: Option<f64>,
    /// Lagna (ascendant) sign 1–12 (1 = Aries).
    /// Required for Chara, Narayana.
    pub lagna_sign: Option<u8>,
    /// Natal sign positions of the seven classical grahas plus Rahu.
    /// Required for Chara, Narayana; irrelevant otherwise.
    pub graha_signs: Option<GrahaSignsInput>,
    /// Number of nested dasha levels (1–5). Defaults to 3.
    /// Ignored by Chara and Narayana, which return a single level.
    pub levels: Option<u8>,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_dasha",
        description: "Compute Vedic dasha (planetary period) sequences. Supports five \
            classical systems: Vimshottari, Ashtottari, Yogini (Moon-longitude based, \
            require `moon_longitude`); Chara, Narayana (sign based, require BOTH \
            `lagna_sign` AND `graha_signs`, since each sign's period length is counted \
            to the sign its lord occupies in this chart). `graha_signs` takes the eight \
            `sign_index` values compute_natal_chart already returns; Ketu is derived \
            from Rahu and must not be supplied. Returns a JSON dasha tree with \
            start/end Julian Days.",
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
                    "description": "Birth Julian Day used as the dasha epoch, in \
                        UT1 (Universal Time) — not TT, not TDB, i.e. the same scale as \
                        compute_natal_chart's julian_day. This epoch is only added to, \
                        never converted: every returned start_jd/end_jd is on the same \
                        UT1 scale as the input."
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
                "graha_signs": {
                    "type": "object",
                    "description": "Natal sign positions of the seven classical grahas plus \
                        Rahu, 0-11 each (0 = Aries), as served by compute_natal_chart's \
                        sign_index. Required for Chara, Narayana, whose period lengths are \
                        chart-dependent — each sign's duration is the count from that sign to \
                        the sign its lord actually occupies. Sign indices are used rather than \
                        longitudes: they chain directly from compute_natal_chart's sign_index \
                        with no conversion, carry no tropical-vs-sidereal ambiguity, and avoid \
                        the boundary question of a planet at exactly 30.0 degrees. Ketu is not \
                        a field: it is derived as (rahu + 6) mod 12, since the two lunar nodes \
                        are always exactly opposite.",
                    "properties": {
                        "sun":     { "type": "integer", "description": "Sun's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "moon":    { "type": "integer", "description": "Moon's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "mars":    { "type": "integer", "description": "Mars's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "mercury": { "type": "integer", "description": "Mercury's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "jupiter": { "type": "integer", "description": "Jupiter's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "venus":   { "type": "integer", "description": "Venus's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "saturn":  { "type": "integer", "description": "Saturn's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "rahu":    { "type": "integer", "description": "Rahu's (north lunar node's) natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 }
                    },
                    "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu"]
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
        annotations: super::ToolAnnotations::READ_ONLY,
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "moon_nakshatra": {
                    "type": "string",
                    "description": "Birth nakshatra of the Moon, which sets the starting mahadasha."
                },
                "initial_balance": {
                    "type": "number",
                    "description": "Fraction of the first mahadasha already elapsed at birth, in [0, 1)."
                },
                "maha_dashas": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "lord": {
                                "type": "string",
                                "description": "Ruling graha of the period."
                            },
                            "level": {
                                "type": "integer",
                                "description": "Nesting depth: 1 = mahadasha, 2 = antardasha, 3 = pratyantardasha."
                            },
                            "start_jd": {
                                "type": "number",
                                "description": "Start of the period as a Julian Day."
                            },
                            "end_jd": {
                                "type": "number",
                                "description": "End of the period as a Julian Day."
                            },
                            "duration_days": {
                                "type": "number",
                                "description": "Length of the period in days."
                            },
                            "sub_periods": {
                                "type": "array",
                                "items": {
                                    "type": "object"
                                },
                                "description": "Nested periods one level down; empty at the deepest level requested."
                            }
                        },
                        "required": [
                            "lord",
                            "level",
                            "start_jd",
                            "end_jd",
                            "duration_days"
                        ]
                    }
                }
            },
            "required": [
                "moon_nakshatra",
                "initial_balance",
                "maha_dashas"
            ]
        })),
        structured_key: None,
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

        let graha_signs = input.graha_signs.ok_or_else(|| {
            McpError::invalid_parameter("graha_signs", "required for Chara and Narayana")
        })?;
        for (name, value) in [
            ("graha_signs.sun", graha_signs.sun),
            ("graha_signs.moon", graha_signs.moon),
            ("graha_signs.mars", graha_signs.mars),
            ("graha_signs.mercury", graha_signs.mercury),
            ("graha_signs.jupiter", graha_signs.jupiter),
            ("graha_signs.venus", graha_signs.venus),
            ("graha_signs.saturn", graha_signs.saturn),
            ("graha_signs.rahu", graha_signs.rahu),
        ] {
            let value = value.ok_or_else(|| {
                McpError::invalid_parameter(name, "required when system is Chara or Narayana")
            })?;
            // Both ends checked. A negative value is the likelier caller
            // mistake of the two -- it is what an off-by-one from a 1-indexed
            // source produces -- and reaching here at all is why the field is
            // parsed as `i32` rather than `u8`.
            if !(0..=11).contains(&value) {
                return Err(McpError::invalid_parameter(
                    name,
                    "must be an integer in [0, 11]",
                ));
            }
        }
    }

    if let Some(levels) = input.levels
        && (levels == 0 || levels > MAX_LEVELS)
    {
        return Err(McpError::invalid_parameter(
            "levels",
            &format!("must be between 1 and {MAX_LEVELS}"),
        ));
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
            graha_signs: None,
            levels: None,
        }
    }

    fn valid_graha_signs() -> GrahaSignsInput {
        GrahaSignsInput {
            sun: Some(9),
            moon: Some(2),
            mars: Some(1),
            mercury: Some(10),
            jupiter: Some(10),
            venus: Some(10),
            saturn: Some(4),
            rahu: Some(2),
        }
    }

    fn lagna_input() -> ComputeDashaInput {
        ComputeDashaInput {
            system: Some("Chara".to_string()),
            birth_jd: 2_451_545.0,
            moon_longitude: None,
            lagna_sign: Some(1),
            graha_signs: Some(valid_graha_signs()),
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
    fn validate_accepts_narayana_with_graha_signs() {
        let mut input = lagna_input();
        input.system = Some("Narayana".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Narayana);
    }

    #[test]
    fn validate_rejects_chara_without_graha_signs() {
        let mut input = lagna_input();
        input.graha_signs = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_chara_missing_rahu() {
        let mut input = lagna_input();
        let mut signs = valid_graha_signs();
        signs.rahu = None;
        input.graha_signs = Some(signs);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(
            err.message.contains("rahu"),
            "error should name the missing field, got: {}",
            err.message
        );
    }

    #[test]
    fn validate_rejects_chara_graha_sign_out_of_range() {
        let mut input = lagna_input();
        let mut signs = valid_graha_signs();
        signs.mars = Some(12);
        input.graha_signs = Some(signs);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(
            err.message.contains("mars"),
            "error should name the offending field, got: {}",
            err.message
        );
    }

    #[test]
    fn validate_moon_based_ignores_missing_graha_signs() {
        // graha_signs is irrelevant for Moon-based systems.
        assert!(validate(&moon_input()).is_ok());
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
