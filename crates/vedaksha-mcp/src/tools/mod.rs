// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! MCP tool schemas, input/output types, and validation entry-points.

use serde::{Deserialize, Serialize};

pub mod compute_ashtakavarga;
pub mod compute_bhavas;
pub mod compute_combustion;
pub mod compute_composite;
pub mod compute_dasha;
pub mod compute_drishti;
pub mod compute_gochara;
pub mod compute_karakas;
pub mod compute_natal;
pub mod compute_panchanga;
pub mod compute_shadbala;
pub mod compute_synastry;
pub mod compute_transit;
pub mod compute_vargas;
pub mod emit_graph;
pub mod search_muhurta;
pub mod search_transits;

/// Behaviour hints an agent can act on before it calls a tool.
///
/// Only the two hints that carry meaning here are modelled. The MCP
/// specification defines `destructiveHint` and `idempotentHint` as meaningful
/// **only when `readOnlyHint` is false**, and every tool on this surface is
/// read-only, so emitting them would be noise dressed as information.
///
/// That every tool is read-only is not an accident of the current set: the
/// engine is stateless by construction, because determinism is what makes the
/// oracles, the bit-identity claims and the exact-equality Python fixture mean
/// anything. A tool that mutated state would break that invariant long before
/// it reached this annotation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// The tool computes and returns; it changes nothing an agent can observe.
    pub read_only_hint: bool,
    /// The tool touches no external entity — no network, no third-party API.
    /// The only inputs are its arguments and the committed ephemeris data.
    pub open_world_hint: bool,
}

impl ToolAnnotations {
    /// A pure computation over its arguments: no writes, no outside world.
    pub const READ_ONLY: Self = Self {
        read_only_hint: true,
        open_world_hint: false,
    };
}

/// Metadata that describes a single MCP tool to an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: serde_json::Value,
    /// Declared per tool rather than defaulted centrally, so that adding a
    /// tool that is *not* read-only is a compile error the author must answer
    /// rather than a wrong hint they inherit in silence.
    pub annotations: ToolAnnotations,
    /// JSON Schema for `structuredContent`, or `None` where the tool has no
    /// settled output contract.
    ///
    /// MCP requires a tool that declares an output schema to return structured
    /// content conforming to it, so declaring one for a response shape that is
    /// not yet honest would make the server non-conformant rather than better
    /// documented. `None` is the truthful answer there.
    pub output_schema: Option<serde_json::Value>,
    /// Key to nest the result under in `structuredContent` when the tool
    /// answers with a JSON array.
    ///
    /// `structuredContent` must be an object, and six tools here return an
    /// array. Wrapping happens at the boundary rather than in the handlers, so
    /// `content[].text` keeps the exact shape callers already parse —
    /// KundaliMCP among them.
    pub structured_key: Option<&'static str>,
}

impl ToolDefinition {
    /// The tool as MCP `tools/list` serialises it, camelCased for the wire.
    ///
    /// This projection existed in three places — the server handler, the
    /// `dump-tools-list` binary and the snapshot drift guard — so adding
    /// `annotations` desynchronised them and the guard failed on a snapshot
    /// that was correctly regenerated. One function now, so a new field
    /// reaches every consumer or none.
    #[must_use]
    pub fn to_wire(&self) -> serde_json::Value {
        let mut v = serde_json::json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
            "annotations": self.annotations,
        });
        // Omitted rather than sent as null: a client that sees the key will
        // expect to validate against it.
        if let Some(schema) = &self.output_schema {
            v["outputSchema"] = schema.clone();
        }
        v
    }

    /// Shape a tool result for `structuredContent`, which MCP requires to be
    /// an object. Array results are nested under [`Self::structured_key`].
    ///
    /// Returns `None` when the tool declares no output schema, so the two
    /// always travel together: structured content with no schema to check it
    /// against is a promise nothing verifies.
    #[must_use]
    pub fn structured_content(&self, value: &serde_json::Value) -> Option<serde_json::Value> {
        self.output_schema.as_ref()?;
        match (value, self.structured_key) {
            (serde_json::Value::Object(_), _) => Some(value.clone()),
            (serde_json::Value::Array(_), Some(key)) => {
                Some(serde_json::json!({ key: value.clone() }))
            }
            _ => None,
        }
    }
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
        compute_gochara::definition(),
        compute_panchanga::definition(),
        compute_drishti::definition(),
        compute_bhavas::definition(),
        compute_synastry::definition(),
        compute_composite::definition(),
    ]
}

/// The ayanamsha names the engine accepts, as a JSON-Schema `enum` array.
///
/// Generated from [`vedaksha_astro::sidereal::Ayanamsha::ALL`] rather than
/// hand-listed, so this schema cannot drift from the systems the engine has —
/// which is exactly what happened before the re-derivation, when the schema
/// named three systems and the engine had forty-four.
#[must_use]
pub fn ayanamsha_schema_enum() -> serde_json::Value {
    serde_json::Value::Array(
        vedaksha_astro::sidereal::Ayanamsha::ALL
            .iter()
            .map(|a| serde_json::Value::String(a.key().to_string()))
            .collect(),
    )
}

/// A description of the ayanamsha parameter that names every system and the
/// primary it is derived from.
///
/// The provenance is carried into the schema rather than left in a doc file
/// because it is the point: an agent choosing a sidereal system should be able
/// to see what defines it. Systems marked `[star]` track a live star, so their
/// values follow catalogue astrometry and will move when a catalogue is
/// superseded.
#[must_use]
pub fn ayanamsha_schema_description() -> String {
    use core::fmt::Write as _;
    let mut s = String::from(
        "Sidereal zodiac system. Eleven systems, each derived forward from a \
primary source and none tuned to match another implementation. The chart's \
true_ayanamsha_value output field reports the TRUE ayanamsha (mean plus nutation in \
longitude) already applied — nutation is included, do not add it yourself. Pass \
Tropical for no rotation. Systems: ",
    );
    for (i, a) in vedaksha_astro::sidereal::Ayanamsha::SIDEREAL
        .iter()
        .enumerate()
    {
        // Joined on " | ", not "; ": two primary_source() strings legitimately
        // contain a semicolon, and joining on one silently split those entries
        // in half for anything parsing this description.
        if i > 0 {
            s.push_str(" | ");
        }
        let star = if a.is_star_anchored() { " [star]" } else { "" };
        let _ = write!(s, "{}{star} = {}", a.key(), a.primary_source());
    }
    s.push('.');
    s
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

    // There is deliberately no `assert_eq!(tool_definitions().len(), N)` here.
    // A hardcoded count is strictly weaker than
    // `snapshot_matches_current_tool_definitions` below, which compares names,
    // descriptions AND schemas against the committed snapshot: the count
    // cannot see a tool renamed, a schema edited, or one tool swapped for
    // another, all of which that assertion catches. It only added a second
    // number to keep in step by hand.

    /// The wire projection is what three consumers serialise — the server's
    /// `tools/list`, the snapshot binary and the drift guard. A field added to
    /// `ToolDefinition` but forgotten here would reach none of them, and the
    /// snapshot guard could not tell, because it uses this same projection.
    #[test]
    fn wire_projection_carries_every_field() {
        for tool in tool_definitions() {
            let wire = tool.to_wire();
            for key in ["name", "description", "inputSchema", "annotations"] {
                assert!(
                    wire.get(key).is_some(),
                    "'{}' is missing '{key}' on the wire",
                    tool.name
                );
            }
            assert_eq!(
                wire["annotations"]["readOnlyHint"], true,
                "'{}' is advertised as read-only; a tool that writes must not \
                 carry ToolAnnotations::READ_ONLY",
                tool.name
            );
        }
    }

    #[test]
    fn tool_names_are_unique() {
        let defs = tool_definitions();
        let mut names: Vec<&str> = defs.iter().map(|t| t.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), defs.len(), "tool names must be unique");
    }

    /// Drift guard: the committed `tools/mcp-tools.json` snapshot at the
    /// workspace root must match what `dump-tools-list` would produce now.
    /// The snapshot powers `vedaksha.net/api/mcp` (introspection-only MCP
    /// endpoint) and the portal's `/docs/mcp` page; if it falls behind the
    /// Rust registry, agents see stale `tools/list` responses.
    ///
    /// To fix a failure, regenerate the snapshot:
    ///   cargo run -p vedaksha-mcp --bin dump-tools-list > tools/mcp-tools.json
    #[test]
    fn snapshot_matches_current_tool_definitions() {
        let snapshot_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/mcp-tools.json");
        let snapshot_raw = std::fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", snapshot_path.display()));
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_raw).expect("snapshot is valid JSON");

        let live_tools: Vec<serde_json::Value> = tool_definitions()
            .iter()
            .map(ToolDefinition::to_wire)
            .collect();
        let live = serde_json::json!({
            "engineVersion": env!("CARGO_PKG_VERSION"),
            "tools": live_tools,
        });

        assert_eq!(
            snapshot, live,
            "tools/mcp-tools.json is out of date — regenerate with: \
             cargo run -p vedaksha-mcp --bin dump-tools-list > tools/mcp-tools.json"
        );
    }
}
