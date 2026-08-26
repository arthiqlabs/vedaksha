// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_composite` — the midpoint composite of two charts.
//!
//! Wraps [`vedaksha_astro::composite::compute_composite`], which is pure over
//! longitudes and speeds: no ephemeris, no observer, no time. The tool
//! therefore takes the two charts' positions directly.
//!
//! # Named, not indexed — and why that removes a panic
//!
//! The underlying function pairs `lons_a[i]` with `lons_b[i]` and asserts the
//! four slices are the same length. Reached through a positional API that
//! assertion is a live panic class: a caller who sends nine longitudes for one
//! chart and eight for the other takes the server down instead of getting an
//! error back. This tool takes a map of graha name to position on each side and
//! pairs by name, so a mismatch is an ordinary `INVALID_PARAMETER` that names
//! the grahas at fault, and the slices handed to the engine are equal-length by
//! construction.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::validation::McpError;

/// One graha's position in an input chart.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CompositeBody {
    /// Sidereal longitude in degrees [0, 360).
    pub longitude: f64,
    /// Daily motion in degrees/day; negative when retrograde. Defaults to 0.
    #[serde(default)]
    pub speed: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeCompositeInput {
    /// Graha name to position for the first chart.
    pub chart_a: BTreeMap<String, CompositeBody>,
    /// Graha name to position for the second chart. Same names as `chart_a`.
    pub chart_b: BTreeMap<String, CompositeBody>,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_composite",
        description: "Compute the midpoint composite chart of two charts: for each graha, the \
            shorter-arc midpoint of its longitude in the two charts, and the arithmetic mean of \
            its two speeds. The two charts must carry the SAME graha names — each graha is paired \
            with its namesake, not with whatever happens to sit at the same position in a list — \
            and a name present in one chart but not the other is an error naming that graha. \
            Longitudes only: this tool needs no birth time, place or ephemeris.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "chart_a": {
                    "type": "object",
                    "description": "Map of graha name to position for the first chart, e.g. \
                                    {\"Sun\": {\"longitude\": 350.0, \"speed\": 1.0}}. `speed` \
                                    is optional and defaults to 0.",
                    "additionalProperties": {
                        "type": "object",
                        "properties": {
                            "longitude": {
                                "type": "number",
                                "description": "Sidereal longitude [0, 360)"
                            },
                            "speed": {
                                "type": "number",
                                "description": "Daily motion in degrees/day; negative when retrograde"
                            }
                        },
                        "required": ["longitude"]
                    }
                },
                "chart_b": {
                    "type": "object",
                    "description": "Map of graha name to position for the second chart. Must have \
                                    exactly the same graha names as chart_a.",
                    "additionalProperties": {
                        "type": "object",
                        "properties": {
                            "longitude": {
                                "type": "number",
                                "description": "Sidereal longitude [0, 360)"
                            },
                            "speed": {
                                "type": "number",
                                "description": "Daily motion in degrees/day; negative when retrograde"
                            }
                        },
                        "required": ["longitude"]
                    }
                }
            },
            "required": ["chart_a", "chart_b"]
        }),
        annotations: super::ToolAnnotations::READ_ONLY,
    }
}

fn validate_chart(
    param: &'static str,
    chart: &BTreeMap<String, CompositeBody>,
) -> Result<(), McpError> {
    if chart.is_empty() {
        return Err(McpError::invalid_parameter(
            param,
            "must contain at least one graha name to position entry",
        ));
    }
    for (name, body) in chart {
        if !body.longitude.is_finite() || !(0.0..360.0).contains(&body.longitude) {
            return Err(McpError::invalid_parameter(
                param,
                &format!("longitude for '{name}' must be a finite number in [0, 360)"),
            ));
        }
        if !body.speed.is_finite() {
            return Err(McpError::invalid_parameter(
                param,
                &format!("speed for '{name}' must be a finite number"),
            ));
        }
    }
    Ok(())
}

/// Validate a [`ComputeCompositeInput`].
///
/// # Errors
/// Returns [`McpError`] when either chart is empty, when any longitude is
/// non-finite or outside [0, 360), when any speed is non-finite, or when the
/// two charts do not carry exactly the same graha names.
pub fn validate(input: &ComputeCompositeInput) -> Result<(), McpError> {
    validate_chart("chart_a", &input.chart_a)?;
    validate_chart("chart_b", &input.chart_b)?;

    let only_a: Vec<&str> = input
        .chart_a
        .keys()
        .filter(|k| !input.chart_b.contains_key(*k))
        .map(String::as_str)
        .collect();
    let only_b: Vec<&str> = input
        .chart_b
        .keys()
        .filter(|k| !input.chart_a.contains_key(*k))
        .map(String::as_str)
        .collect();

    if !only_a.is_empty() || !only_b.is_empty() {
        return Err(McpError::invalid_parameter(
            "chart_b",
            &format!(
                "the two charts must carry the same graha names; \
                 only in chart_a: [{}]; only in chart_b: [{}]",
                only_a.join(", "),
                only_b.join(", ")
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(longitude: f64, speed: f64) -> CompositeBody {
        CompositeBody { longitude, speed }
    }

    fn valid_input() -> ComputeCompositeInput {
        ComputeCompositeInput {
            chart_a: BTreeMap::from([("Sun".to_string(), body(350.0, 1.0))]),
            chart_b: BTreeMap::from([("Sun".to_string(), body(10.0, 1.0))]),
        }
    }

    #[test]
    fn validate_accepts_matching_charts() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_mismatched_graha_names() {
        // This is the case that panicked through the positional API: chart A
        // has two entries, chart B one, so `assert_eq!(lons_a.len(), …)` would
        // have fired inside the engine. Here it must be an ordinary error.
        let mut input = valid_input();
        input.chart_a.insert("Moon".to_string(), body(100.0, 13.0));
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(
            err.message.contains("Moon"),
            "the error must name the offending graha, got: {}",
            err.message
        );
    }

    #[test]
    fn validate_rejects_same_count_but_different_names() {
        // Equal lengths, so the engine's assertion would have passed and it
        // would silently have paired Moon with Mars. Pairing by name catches it.
        let mut input = valid_input();
        input.chart_a.insert("Moon".to_string(), body(100.0, 13.0));
        input.chart_b.insert("Mars".to_string(), body(200.0, 0.5));
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(err.message.contains("Moon") && err.message.contains("Mars"));
    }

    #[test]
    fn validate_rejects_empty_chart() {
        let mut input = valid_input();
        input.chart_a = BTreeMap::new();
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_out_of_range_longitude() {
        let mut input = valid_input();
        input.chart_b.insert("Sun".to_string(), body(360.0, 1.0));
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_non_finite_speed() {
        let mut input = valid_input();
        input
            .chart_a
            .insert("Sun".to_string(), body(350.0, f64::NAN));
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn speed_defaults_to_zero_when_omitted() {
        let parsed: CompositeBody =
            serde_json::from_value(serde_json::json!({ "longitude": 12.5 })).expect("parses");
        assert!((parsed.longitude - 12.5).abs() < f64::EPSILON);
        assert!(parsed.speed.abs() < f64::EPSILON);
    }

    #[test]
    fn definition_requires_both_charts() {
        let def = definition();
        assert_eq!(def.name, "compute_composite");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(names, vec!["chart_a", "chart_b"]);
    }
}
