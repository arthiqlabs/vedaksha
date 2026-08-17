// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `emit_graph` — convert a `ChartGraph` JSON to a target output format.

use serde::{Deserialize, Serialize};

use crate::validation::McpError;

/// Formats that the `emit_graph` tool can produce.
pub const VALID_FORMATS: &[&str] = &["cypher", "surreal", "jsonld", "json", "embedding"];

/// Input parameters for the `emit_graph` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct EmitGraphInput {
    /// Either a `ChartGraph` (`{nodes, edges, ...}`) or the output of
    /// `compute_natal_chart` (`{planets, houses, aspects, ...}`), which is
    /// converted to a graph on the way in.
    ///
    /// Until v5.0.1 only the first form worked, while the schema advertised the
    /// second — handing this tool a natal chart returned `missing field
    /// 'nodes'`, because nothing in the engine could build a graph from a
    /// computed chart.
    pub chart_json: serde_json::Value,
    /// Observer latitude in degrees, positive north. **Required when
    /// `chart_json` is a computed chart**, since a chart result does not record
    /// where it was cast and the graph's `Chart` node does. Ignored when a
    /// `ChartGraph` is supplied, which already carries its own chart node.
    pub latitude: Option<f64>,
    /// Observer longitude in degrees, positive east. Same rule as `latitude`.
    pub longitude: Option<f64>,
    /// Target output format: `cypher`, `surreal`, `jsonld`, `json`, or
    /// `embedding`.
    pub format: String,
    /// Optional classification tag attached to emitted nodes (e.g. a
    /// chart label or session ID).
    pub classification: Option<String>,
}

/// Output of the `emit_graph` tool.
#[derive(Debug, Clone, Serialize)]
pub struct EmitGraphOutput {
    /// Emitted content in the requested format (string for Cypher/SurrealQL/
    /// embedding, JSON value for `json`/`jsonld`).
    pub output: serde_json::Value,
    /// The format that was used.
    pub format: String,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "emit_graph",
        description: "Turn a chart into a queryable property graph. Accepts the output of \
            compute_natal_chart directly (pass latitude and longitude alongside it), or an \
            existing ChartGraph. Emits Neo4j Cypher, SurrealDB SurrealQL, JSON-LD, plain \
            JSON, or RAG embedding text.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "chart_json": {
                    "type": "object",
                    "description": "Either the output of compute_natal_chart ({planets, houses, aspects, ...}), which is converted to a graph here, or an existing ChartGraph ({nodes, edges, chart_id, classification})"
                },
                "latitude": {
                    "type": "number",
                    "minimum": -90,
                    "maximum": 90,
                    "description": "Observer latitude, degrees north. Required when chart_json is a computed chart; a chart result does not record where it was cast. Ignored for an existing ChartGraph."
                },
                "longitude": {
                    "type": "number",
                    "minimum": -180,
                    "maximum": 180,
                    "description": "Observer longitude, degrees east. Required when chart_json is a computed chart."
                },
                "format": {
                    "type": "string",
                    "enum": ["cypher", "surreal", "jsonld", "json", "embedding"],
                    "description": "Target output format"
                },
                "classification": {
                    "type": "string",
                    "description": "Optional label or session ID attached to emitted nodes"
                }
            },
            "required": ["chart_json", "format"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns [`McpError::invalid_parameter`] when `format` is not one of the
/// recognised values.
pub fn validate(input: &EmitGraphInput) -> Result<(), McpError> {
    let fmt = input.format.trim().to_lowercase();
    if !VALID_FORMATS.contains(&fmt.as_str()) {
        return Err(McpError::invalid_parameter(
            "format",
            &format!("must be one of: {}", VALID_FORMATS.join(", ")),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input(format: &str) -> EmitGraphInput {
        EmitGraphInput {
            chart_json: serde_json::json!({ "nodes": [], "edges": [] }),
            format: format.into(),
            classification: None,
            latitude: None,
            longitude: None,
        }
    }

    #[test]
    fn validate_accepts_cypher() {
        assert!(validate(&valid_input("cypher")).is_ok());
    }

    #[test]
    fn validate_accepts_surreal() {
        assert!(validate(&valid_input("surreal")).is_ok());
    }

    #[test]
    fn validate_accepts_jsonld() {
        assert!(validate(&valid_input("jsonld")).is_ok());
    }

    #[test]
    fn validate_accepts_json() {
        assert!(validate(&valid_input("json")).is_ok());
    }

    #[test]
    fn validate_accepts_embedding() {
        assert!(validate(&valid_input("embedding")).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_format() {
        let err = validate(&valid_input("turtle")).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(err.message.contains("format"));
    }

    #[test]
    fn validate_rejects_empty_format() {
        let err = validate(&valid_input("")).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_is_case_insensitive() {
        // "Cypher" (mixed case) should still pass.
        assert!(validate(&valid_input("Cypher")).is_ok());
        assert!(validate(&valid_input("JSON")).is_ok());
    }

    #[test]
    fn definition_has_format_enum() {
        let def = definition();
        let fmt_enum = def.input_schema["properties"]["format"]["enum"]
            .as_array()
            .unwrap();
        let values: Vec<&str> = fmt_enum.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(values.contains(&"cypher"));
        assert!(values.contains(&"embedding"));
    }
}

/// Rebuild a [`ComputedChart`] from `compute_natal_chart`'s published output.
///
/// The tool does not serialise `ComputedChart` directly — it emits a bespoke
/// projection, and the `aspects` array is where the two diverge: the tool
/// publishes `body1`/`body2`/`type`/`applying` where the struct has
/// `body1_index`/`body2_index`/`aspect_type`/`motion`. Deserialising the tool's
/// own output straight back into the struct therefore fails on
/// `missing field 'body1_index'`.
///
/// This adapter reads the **published** shape, which is the contract callers
/// actually see, so `emit_graph` consumes exactly what `compute_natal_chart`
/// hands them.
///
/// # Errors
///
/// Returns a message naming the field that did not match.
pub fn computed_chart_from_tool_output(
    value: &serde_json::Value,
) -> Result<vedaksha_astro::chart::ComputedChart, String> {
    use vedaksha_astro::aspects::{Aspect, AspectMotion, AspectType};

    let planets: Vec<vedaksha_astro::chart::ChartPlanet> =
        serde_json::from_value(value.get("planets").cloned().unwrap_or_default())
            .map_err(|e| format!("`planets`: {e}"))?;
    let houses: vedaksha_astro::houses::HouseCusps =
        serde_json::from_value(value.get("houses").cloned().unwrap_or_default())
            .map_err(|e| format!("`houses`: {e}"))?;

    let mut aspects = Vec::new();
    if let Some(list) = value.get("aspects").and_then(|v| v.as_array()) {
        for (i, a) in list.iter().enumerate() {
            let field = |k: &str| a.get(k).cloned().unwrap_or(serde_json::Value::Null);
            let idx = |k: &str| -> Result<usize, String> {
                field(k)
                    .as_u64()
                    .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
                    .ok_or_else(|| format!("`aspects[{i}].{k}` is not an index"))
            };
            let aspect_type: AspectType = serde_json::from_value(field("type"))
                .map_err(|e| format!("`aspects[{i}].type`: {e}"))?;
            aspects.push(Aspect {
                body1_index: idx("body1")?,
                body2_index: idx("body2")?,
                aspect_type,
                orb: field("orb")
                    .as_f64()
                    .ok_or_else(|| format!("`aspects[{i}].orb` is not a number"))?,
                motion: if field("applying").as_bool().unwrap_or(false) {
                    AspectMotion::Applying
                } else {
                    AspectMotion::Separating
                },
                strength: field("strength")
                    .as_f64()
                    .ok_or_else(|| format!("`aspects[{i}].strength` is not a number"))?,
            });
        }
    }

    Ok(vedaksha_astro::chart::ComputedChart {
        planets,
        houses,
        aspects,
        config_summary: value
            .get("config_summary")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}
