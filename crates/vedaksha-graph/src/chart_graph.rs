// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `ChartGraph` — a complete astrological chart as a property graph.

use serde::{Deserialize, Serialize};

use crate::classification::DataClassification;
use crate::ids::NodeId;
use crate::ontology::{Edge, EdgeType, Node, NodeType};

/// A complete astrological chart represented as a property graph.
///
/// Contains all nodes (planets, signs, houses, nakshatras, etc.)
/// and edges (aspects, placements, rulerships, etc.) for a single
/// chart computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartGraph {
    /// All nodes in the graph.
    pub nodes: Vec<Node>,
    /// All edges in the graph.
    pub edges: Vec<Edge>,
    /// The chart node's ID (root of the graph).
    pub chart_id: NodeId,
    /// Data classification level for this chart.
    pub classification: DataClassification,
}

impl ChartGraph {
    /// Create a new empty chart graph with the given chart ID.
    #[must_use]
    pub fn new(chart_id: NodeId, classification: DataClassification) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            chart_id,
            classification,
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Find a node by its ID.
    #[must_use]
    pub fn find_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| &n.id == id)
    }

    /// Find all nodes of a given type.
    #[must_use]
    pub fn nodes_of_type(&self, node_type: NodeType) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    /// Find all edges of a given type.
    #[must_use]
    pub fn edges_of_type(&self, edge_type: EdgeType) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.edge_type == edge_type)
            .collect()
    }

    /// Find all edges originating from a node.
    #[must_use]
    pub fn edges_from(&self, id: &NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| &e.from == id).collect()
    }

    /// Find all edges pointing to a node.
    #[must_use]
    pub fn edges_to(&self, id: &NodeId) -> Vec<&Edge> {
        self.edges.iter().filter(|e| &e.to == id).collect()
    }

    /// Total number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Serialize to JSON (the canonical MCP format).
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::{EdgeProperties, NodeProperties};

    fn make_graph() -> ChartGraph {
        let chart_id = NodeId::chart_scoped("test", "chart", "root");
        let mut g = ChartGraph::new(chart_id, DataClassification::Anonymous);

        g.add_node(Node {
            id: NodeId::chart_scoped("test", "planet", "mars"),
            node_type: NodeType::Planet,
            properties: NodeProperties::Planet {
                name: "Mars".to_string(),
                longitude: 45.5,
                latitude: 1.2,
                distance: 1.52,
                speed: 0.6,
                retrograde: false,
                sign_index: 1,
                house: 7,
            },
        });

        g.add_node(Node {
            id: NodeId::global("sign", "aries"),
            node_type: NodeType::Sign,
            properties: NodeProperties::Sign {
                name: "Aries".to_string(),
                index: 0,
                element: "Fire".to_string(),
                modality: "Cardinal".to_string(),
            },
        });

        g.add_edge(Edge {
            edge_type: EdgeType::PlacedIn,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::global("sign", "aries"),
            properties: EdgeProperties::None,
        });

        g
    }

    #[test]
    fn empty_graph_counts() {
        let g = ChartGraph::new(
            NodeId::chart_scoped("x", "chart", "root"),
            DataClassification::Anonymous,
        );
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn add_and_find_node() {
        let g = make_graph();
        let mars_id = NodeId::chart_scoped("test", "planet", "mars");
        let found = g.find_node(&mars_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().node_type, NodeType::Planet);
    }

    #[test]
    fn find_edges_by_type() {
        let g = make_graph();
        let placed = g.edges_of_type(EdgeType::PlacedIn);
        assert_eq!(placed.len(), 1);
        let aspects = g.edges_of_type(EdgeType::Aspects);
        assert!(aspects.is_empty());
    }

    #[test]
    fn nodes_of_type_filters() {
        let g = make_graph();
        let planets = g.nodes_of_type(NodeType::Planet);
        assert_eq!(planets.len(), 1);
        let signs = g.nodes_of_type(NodeType::Sign);
        assert_eq!(signs.len(), 1);
        let houses = g.nodes_of_type(NodeType::House);
        assert!(houses.is_empty());
    }

    #[test]
    fn to_json_produces_valid_json() {
        let g = make_graph();
        let json = g.to_json().expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed.is_object());
    }

    #[test]
    fn edges_from_and_to() {
        let g = make_graph();
        let mars_id = NodeId::chart_scoped("test", "planet", "mars");
        let aries_id = NodeId::global("sign", "aries");

        let from_mars = g.edges_from(&mars_id);
        assert_eq!(from_mars.len(), 1);

        let to_aries = g.edges_to(&aries_id);
        assert_eq!(to_aries.len(), 1);

        // No edges from aries in this graph.
        let from_aries = g.edges_from(&aries_id);
        assert!(from_aries.is_empty());
    }
}
