// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! MCP tool schemas, input/output types, and validation entry-points.

use serde::{Deserialize, Serialize};

pub mod compute_ashtakavarga;
pub mod compute_combustion;
pub mod compute_dasha;
pub mod compute_shadbala;
pub mod compute_karakas;
pub mod compute_natal;
pub mod compute_transit;
pub mod compute_vargas;
pub mod emit_graph;
pub mod search_muhurta;
pub mod search_transits;

/// Metadata that describes a single MCP tool to an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: serde_json::Value,
}

/// Return the registry of all currently available tools.
#[must_use]
pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        compute_natal::definition(),
        compute_dasha::definition(),
        compute_karakas::definition(),
        compute_combustion::definition(),
        compute_shadbala::definition(),
        compute_vargas::definition(),
        emit_graph::definition(),
        compute_transit::definition(),
        search_transits::definition(),
        search_muhurta::definition(),
        compute_ashtakavarga::definition(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tool_definitions_have_non_empty_name_and_description() {
        for tool in tool_definitions() {
            assert!(!tool.name.is_empty(), "tool name must not be empty");
            assert!(
                !tool.description.is_empty(),
                "description for '{}' must not be empty",
                tool.name
            );
        }
    }

    #[test]
    fn tool_definitions_produce_valid_json_schemas() {
        for tool in tool_definitions() {
            // Must have a "type" field at the root.
            assert!(
                tool.input_schema.get("type").is_some(),
                "schema for '{}' must have a 'type' field",
                tool.name
            );
            // Must have a "properties" object.
            assert!(
                tool.input_schema.get("properties").is_some(),
                "schema for '{}' must have a 'properties' field",
                tool.name
            );
        }
    }

    #[test]
    fn exactly_eleven_tools_are_registered() {
        assert_eq!(tool_definitions().len(), 11);
    }

    #[test]
    fn tool_names_are_unique() {
        let defs = tool_definitions();
        let mut names: Vec<&str> = defs.iter().map(|t| t.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), defs.len(), "tool names must be unique");
    }
}
