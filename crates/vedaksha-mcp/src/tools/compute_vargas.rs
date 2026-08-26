// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_vargas` — Vedic divisional chart (varga) computation tool.

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Input parameters for the `compute_vargas` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct ComputeVargasInput {
    /// Julian Day number in **UT1** (Universal Time) — not TT, not TDB.
    ///
    /// Same scale as `compute_natal_chart`'s `julian_day`: the engine
    /// converts to TT internally for the dynamical terms and uses UT1
    /// directly for the Earth-rotation term.
    pub julian_day: f64,
    /// Geographic latitude in degrees \[-90, +90\].
    pub latitude: f64,
    /// Geographic longitude in degrees \[-180, +180\], east positive.
    pub longitude: f64,
    /// Sidereal longitude of a single body in degrees \[0, 360\).
    ///
    /// When supplied, only that longitude is divided and no ephemeris is
    /// consulted; the result carries no graha name, no dignity and no bhava,
    /// because none of the three is defined without knowing which body it is.
    pub planet_longitude: Option<f64>,
    /// List of varga division codes to compute, e.g. `["D1", "D9", "D10"]`.
    pub divisions: Vec<String>,
    /// Sidereal system to rotate by before dividing. Defaults to `Tropical`,
    /// matching `compute_natal_chart`.
    pub ayanamsha: Option<String>,
    /// Which Parashari reading to use where the texts diverge: `modality`
    /// (default) or `element`. Affects D16, D20, D30 and D45 only.
    pub tradition: Option<String>,
}

/// The varga traditions this tool accepts.
pub const TRADITIONS: [&str; 2] = ["modality", "element"];

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_vargas",
        description: "Compute Vedic divisional charts (vargas). Given a time and place, \
            returns one chart per requested division: the varga lagna, and for each of the ten \
            bodies compute_natal_chart returns — the seven grahas plus the mean, true and \
            osculating lunar node — its rashi longitude, the sign it occupies within that varga, \
            its dignity in that sign, and its whole-sign bhava counted from the varga lagna. Ketu \
            is not listed separately: it is the node's opposite point, 180 degrees away. The \
            nodes carry no dignity, so that field is absent for them. Supply \
            planet_longitude instead to divide a single longitude without an ephemeris lookup, \
            in which case no graha name, dignity or bhava is returned because none is defined. \
            Vargas are classically read on a sidereal zodiac: pass an ayanamsha, or accept the \
            Tropical default this surface uses everywhere. Source: BPHS Ch. 6-7.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "julian_day": {
                    "type": "number",
                    "description": "Julian Day number in UT1 (Universal Time) \
                        — not TT, not TDB. Same scale as compute_natal_chart's \
                        julian_day: the engine converts to TT internally for the \
                        dynamical terms and uses UT1 directly for the Earth-rotation \
                        term."
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
                    "description": "Sidereal longitude of a single body in degrees [0, 360). \
                        When supplied, only this longitude is divided and no ephemeris is \
                        consulted; the result carries no graha name, dignity or bhava."
                },
                "ayanamsha": {
                    "type": "string",
                    "description": crate::tools::ayanamsha_schema_description(),
                    "enum": crate::tools::ayanamsha_schema_enum()
                },
                "tradition": {
                    "type": "string",
                    "enum": TRADITIONS,
                    "default": "modality",
                    "description": "Which Parashari reading to use where the texts diverge. \
                        'modality' (default) starts the division from a movable/fixed/dual \
                        sign; 'element' starts it from a fire/earth/air/water sign. This \
                        changes D16, D20, D30 and D45 only; every other varga is identical \
                        under both. Source: BPHS Ch. 6; Phala Deepika Ch. 2."
                }
            },
            "required": ["julian_day", "latitude", "longitude", "divisions"]
        }),
        annotations: super::ToolAnnotations::READ_ONLY,
        // Declarable at last. Through v7.3.1 this tool answered its own
        // documented call with a `status`/`message` stub, and no schema can be
        // honest about a contract that is not kept.
        //
        // `lagna_sign` and the per-placement `planet`, `dignity` and `bhava`
        // are absent on the planet_longitude path and required on neither, for
        // the reason the description gives: a bare longitude names no graha, so
        // it has no dignity, and there is no lagna to count a bhava from.
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "julian_day": {
                    "type": "number",
                    "description": "The Julian Day the vargas were computed for, echoed back."
                },
                "ayanamsha_value": {
                    "type": "number",
                    "description": "Mean ayanamsha applied before dividing, in degrees. Zero when Tropical."
                },
                "tradition": {
                    "type": "string",
                    "description": "The tradition actually used: 'modality' or 'element'."
                },
                "vargas": {
                    "type": "array",
                    "description": "One entry per requested division, in the order requested.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "division": {
                                "type": "string",
                                "description": "The division code as requested, e.g. D9."
                            },
                            "lagna_sign": {
                                "type": "integer",
                                "description": "Sign the ascendant occupies in this varga (0=Aries…11=Pisces). Absent when planet_longitude was supplied."
                            },
                            "placements": {
                                "type": "array",
                                "description": "One entry per graha, or a single entry for a supplied longitude.",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "planet": {
                                            "type": "string",
                                            "description": "Graha name. Absent for a supplied longitude."
                                        },
                                        "rashi_longitude": {
                                            "type": "number",
                                            "description": "The sidereal longitude that was divided, in degrees [0, 360)."
                                        },
                                        "varga_sign": {
                                            "type": "integer",
                                            "description": "Sign occupied within this varga (0=Aries…11=Pisces)."
                                        },
                                        "dignity": {
                                            "type": "string",
                                            "description": "Essential dignity in the varga sign. Absent for a supplied longitude, and for Rahu and Ketu, which have none."
                                        },
                                        "bhava": {
                                            "type": "integer",
                                            "description": "Whole-sign house counted from the varga lagna, 1-12. Absent when planet_longitude was supplied."
                                        }
                                    },
                                    "required": ["rashi_longitude", "varga_sign"]
                                }
                            }
                        },
                        "required": ["division", "placements"]
                    }
                }
            },
            "required": ["julian_day", "tradition", "vargas"]
        })),
        structured_key: None,
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

    if let Some(tradition) = &input.tradition {
        if !TRADITIONS.contains(&tradition.as_str()) {
            return Err(McpError::invalid_parameter(
                "tradition",
                &format!("must be one of: {}", TRADITIONS.join(", ")),
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
            tradition: None,
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
