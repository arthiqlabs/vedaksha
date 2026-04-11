// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Plain JSON graph emitter — the canonical MCP format.
//!
//! Serializes `ChartGraph` directly to pretty-printed JSON using serde.
//! This is the simplest emitter and the default format for MCP tool responses.

use crate::GraphEmitter;
use vedaksha_graph::ChartGraph;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Emits `ChartGraph` as plain JSON (the canonical MCP format).
///
/// The output contains `nodes`, `edges`, `chart_id`, and `classification`
/// fields matching the `ChartGraph` structure exactly.
pub struct JsonGraphEmitter;

impl GraphEmitter for JsonGraphEmitter {
    type Output = String;

    fn emit(&self, graph: &ChartGraph) -> Result<String, String> {
        serde_json::to_string_pretty(graph).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_graph::{
        ChartGraph,
        classification::DataClassification,
        ids::NodeId,
        ontology::{Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType},
    };

    fn make_test_graph() -> ChartGraph {
        let chart_id = NodeId::chart_scoped("test", "chart", "root");
        let mut g = ChartGraph::new(chart_id, DataClassification::Anonymous);

        g.add_node(Node {
            id: NodeId::chart_scoped("test", "planet", "mars"),
            node_type: NodeType::Planet,
            properties: NodeProperties::Planet {
                name: "Mars".to_string(),
                longitude: 280.5,
                latitude: 1.2,
                distance: 1.52,
                speed: -0.23,
                retrograde: true,
                sign_index: 9,
                house: 10,
            },
        });

        g.add_node(Node {
            id: NodeId::global("sign", "capricorn"),
            node_type: NodeType::Sign,
            properties: NodeProperties::Sign {
                name: "Capricorn".to_string(),
                index: 9,
                element: "Earth".to_string(),
                modality: "Cardinal".to_string(),
            },
        });

        g.add_edge(Edge {
            edge_type: EdgeType::PlacedIn,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::global("sign", "capricorn"),
            properties: EdgeProperties::None,
        });

        g
    }

    #[test]
    fn output_is_valid_json() {
        let graph = make_test_graph();
        let output = JsonGraphEmitter.emit(&graph).expect("emit");
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert!(parsed.is_object(), "output should be a JSON object");
        assert!(parsed["nodes"].is_array(), "should have nodes array");
        assert!(parsed["edges"].is_array(), "should have edges array");
    }

    #[test]
    fn deserializes_back_to_chart_graph() {
        let graph = make_test_graph();
        let output = JsonGraphEmitter.emit(&graph).expect("emit");
        let restored: ChartGraph =
            serde_json::from_str(&output).expect("should deserialize back to ChartGraph");
        assert_eq!(
            restored.node_count(),
            graph.node_count(),
            "node count should match"
        );
        assert_eq!(
            restored.edge_count(),
            graph.edge_count(),
            "edge count should match"
        );
        assert_eq!(
            restored.chart_id.to_string(),
            graph.chart_id.to_string(),
            "chart_id should match"
        );
        assert_eq!(
            restored.classification, graph.classification,
            "classification should match"
        );
    }
}
