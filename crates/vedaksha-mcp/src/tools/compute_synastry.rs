// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_synastry` — cross-chart aspects between two charts.
//!
//! Wraps [`vedaksha_astro::synastry::find_synastry_aspects`], which is pure
//! over longitudes: no ephemeris, no observer, no time. The tool therefore
//! takes the two charts' longitudes directly.
//!
//! # Named, not indexed
//!
//! The underlying function reports `chart_a_body` / `chart_b_body` as indices
//! into the slices it was handed. Indices are meaningless to a calling agent —
//! it would have to remember the order it packed the slices in. This tool takes
//! a map of graha name to longitude on each side and emits the names back, the
//! same convention `compute_bhavas` uses for its `planets` map.

use std::collections::BTreeMap;

use serde::Deserialize;
use vedaksha_astro::aspects::AspectType;

use crate::validation::McpError;

/// Widest orb multiplier the tool accepts.
///
/// Derived, not picked: at `orb_factor = 5.0` the five major aspect windows
/// already cover the entire 0–180° separation range, so every pair of grahas
/// aspects every other pair and the answer carries no information. With the
/// Lilly orbs the tool uses — conjunction/trine/opposition 8°, square 7°,
/// sextile 6° — a factor of 5 gives half-widths of 40°, 35° and 30°, so the
/// windows are [0,40] ∪ [30,90] ∪ [55,125] ∪ [80,160] ∪ [140,180] = [0,180].
/// Anything beyond that is certainly a caller error, so it is rejected with a
/// named parameter rather than answered with noise.
pub const MAX_ORB_FACTOR: f64 = 5.0;

/// Which family of aspects to test for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectSet {
    /// The five Ptolemaic aspects: conjunction, sextile, square, trine, opposition.
    Major,
    /// The five majors plus the six minors the engine knows.
    All,
}

impl AspectSet {
    /// The aspect types this set selects.
    #[must_use]
    pub const fn types(self) -> &'static [AspectType] {
        match self {
            Self::Major => AspectType::MAJOR,
            Self::All => AspectType::ALL,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeSynastryInput {
    /// Graha name to sidereal longitude for the first chart.
    pub chart_a: BTreeMap<String, f64>,
    /// Graha name to sidereal longitude for the second chart.
    pub chart_b: BTreeMap<String, f64>,
    /// `"major"` (default) or `"all"`.
    pub aspect_set: Option<String>,
    /// Multiplier on the default orbs; 1.0 = standard.
    pub orb_factor: Option<f64>,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_synastry",
        description: "Compute synastry — the aspects each graha in one chart makes to each graha \
            in another chart. Every graha in chart A is tested against every graha in chart B \
            (the two charts do not need the same graha names), and each hit is returned with its \
            aspect type, orb in degrees and strength (1.0 at exact, falling linearly to 0.0 at the \
            orb boundary). Orbs are the traditional Lilly values — conjunction, trine and \
            opposition 8°, square 7°, sextile 6°, minors 2° — scaled by orb_factor. Longitudes \
            only: this tool needs no birth time, place or ephemeris.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "chart_a": {
                    "type": "object",
                    "description": "Map of graha name to sidereal longitude [0, 360) for the \
                                    first chart, e.g. {\"Sun\": 10.0, \"Moon\": 130.0}.",
                    "additionalProperties": { "type": "number" }
                },
                "chart_b": {
                    "type": "object",
                    "description": "Map of graha name to sidereal longitude [0, 360) for the \
                                    second chart. Need not use the same names as chart_a.",
                    "additionalProperties": { "type": "number" }
                },
                "aspect_set": {
                    "type": "string",
                    "description": "'major' (default) tests the five Ptolemaic aspects; 'all' \
                                    adds semi-sextile, semi-square, quintile, sesquiquadrate, \
                                    bi-quintile and quincunx.",
                    "enum": ["major", "all"],
                    "default": "major"
                },
                "orb_factor": {
                    "type": "number",
                    "description": "Multiplier on the default orbs. 1.0 (default) is standard, \
                                    0.5 is tight. Must be greater than 0 and at most 5.0, above \
                                    which the major-aspect windows cover every separation.",
                    "default": 1.0
                }
            },
            "required": ["chart_a", "chart_b"]
        }),
        annotations: super::ToolAnnotations::READ_ONLY,
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "aspects": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "chart_a_planet": {
                                "type": "string",
                                "description": "Graha from the first chart."
                            },
                            "chart_b_planet": {
                                "type": "string",
                                "description": "Graha from the second chart."
                            },
                            "aspect_type": {
                                "type": "string",
                                "description": "Aspect name."
                            },
                            "orb": {
                                "type": "number",
                                "description": "Departure from exact, in degrees."
                            },
                            "strength": {
                                "type": "number",
                                "description": "Normalised strength in [0, 1], 1 at exact."
                            }
                        },
                        "required": [
                            "chart_a_planet",
                            "chart_b_planet",
                            "aspect_type",
                            "orb",
                            "strength"
                        ]
                    }
                }
            },
            "required": [
                "aspects"
            ]
        })),
        structured_key: Some("aspects"),
    }
}

fn validate_chart(param: &'static str, chart: &BTreeMap<String, f64>) -> Result<(), McpError> {
    if chart.is_empty() {
        return Err(McpError::invalid_parameter(
            param,
            "must contain at least one graha name to longitude entry",
        ));
    }
    for (name, lon) in chart {
        if !lon.is_finite() || !(0.0..360.0).contains(lon) {
            return Err(McpError::invalid_parameter(
                param,
                &format!("longitude for '{name}' must be a finite number in [0, 360)"),
            ));
        }
    }
    Ok(())
}

/// Validate a [`ComputeSynastryInput`] and resolve its defaults.
///
/// # Errors
/// Returns [`McpError`] when either chart is empty or carries a non-finite or
/// out-of-range longitude, when `aspect_set` is not `"major"` or `"all"`, or
/// when `orb_factor` is outside `(0, MAX_ORB_FACTOR]`.
pub fn validate(input: &ComputeSynastryInput) -> Result<(AspectSet, f64), McpError> {
    validate_chart("chart_a", &input.chart_a)?;
    validate_chart("chart_b", &input.chart_b)?;

    let aspect_set = match input.aspect_set.as_deref().unwrap_or("major") {
        "major" => AspectSet::Major,
        "all" => AspectSet::All,
        other => {
            return Err(McpError::invalid_parameter(
                "aspect_set",
                &format!("unknown aspect set '{other}'; expected 'major' or 'all'"),
            ));
        }
    };

    let orb_factor = input.orb_factor.unwrap_or(1.0);
    if !orb_factor.is_finite() || orb_factor <= 0.0 || orb_factor > MAX_ORB_FACTOR {
        return Err(McpError::invalid_parameter(
            "orb_factor",
            &format!("must be a finite number greater than 0 and at most {MAX_ORB_FACTOR}"),
        ));
    }

    Ok((aspect_set, orb_factor))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputeSynastryInput {
        ComputeSynastryInput {
            chart_a: BTreeMap::from([("Sun".to_string(), 10.0)]),
            chart_b: BTreeMap::from([("Moon".to_string(), 130.0)]),
            aspect_set: None,
            orb_factor: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input_and_defaults_to_major_at_unit_orb() {
        let (set, orb_factor) = validate(&valid_input()).expect("valid");
        assert_eq!(set, AspectSet::Major);
        assert!((orb_factor - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn major_set_is_five_aspects_and_all_is_eleven() {
        // Derived from `AspectType`: MAJOR is conjunction/sextile/square/trine/
        // opposition = 5; ALL adds semi-sextile, quincunx, semi-square,
        // sesquiquadrate, quintile and bi-quintile = 5 + 6 = 11.
        assert_eq!(AspectSet::Major.types().len(), 5);
        assert_eq!(AspectSet::All.types().len(), 11);
    }

    #[test]
    fn validate_rejects_empty_chart() {
        let mut input = valid_input();
        input.chart_b = BTreeMap::new();
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_nan_longitude() {
        let mut input = valid_input();
        input.chart_a.insert("Mars".to_string(), f64::NAN);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_longitude_at_360() {
        let mut input = valid_input();
        input.chart_b.insert("Ketu".to_string(), 360.0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_unknown_aspect_set() {
        let mut input = valid_input();
        input.aspect_set = Some("ptolemaic".to_string());
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_accepts_all_aspect_set() {
        let mut input = valid_input();
        input.aspect_set = Some("all".to_string());
        assert_eq!(validate(&input).expect("valid").0, AspectSet::All);
    }

    #[test]
    fn validate_bounds_orb_factor() {
        let mut input = valid_input();
        input.orb_factor = Some(0.0);
        assert!(
            validate(&input).is_err(),
            "zero orb factor must be rejected"
        );
        input.orb_factor = Some(-1.0);
        assert!(validate(&input).is_err(), "negative must be rejected");
        input.orb_factor = Some(MAX_ORB_FACTOR);
        assert!(validate(&input).is_ok(), "the bound itself is accepted");
        // Smallest representable step above the bound.
        input.orb_factor = Some(MAX_ORB_FACTOR + 0.001);
        assert!(
            validate(&input).is_err(),
            "above the bound must be rejected"
        );
        input.orb_factor = Some(f64::INFINITY);
        assert!(validate(&input).is_err(), "infinity must be rejected");
    }

    #[test]
    fn definition_requires_both_charts() {
        let def = definition();
        assert_eq!(def.name, "compute_synastry");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(names, vec!["chart_a", "chart_b"]);
    }
}
