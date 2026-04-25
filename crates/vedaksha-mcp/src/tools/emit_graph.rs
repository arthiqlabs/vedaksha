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
    /// A `ChartGraph` as previously returned by `compute_natal_chart` or
    /// a similar tool.
    pub chart_json: serde_json::Value,
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
        description: "Convert a ChartGraph JSON (as produced by compute_natal_chart or \
            compute_vargas) into a target output format for downstream storage or retrieval. \
            Supports Neo4j Cypher, SurrealDB SurrealQL, JSON-LD, plain JSON, and RAG \
            embedding text.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "chart_json": {
                    "type": "object",
                    "description": "ChartGraph JSON as returned by compute_natal_chart or compute_vargas"
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
