// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Neo4j Cypher emitter.
//!
//! Emits `MERGE` for global nodes (Sign, Nakshatra, `FixedStar`) and
//! `CREATE` for chart-scoped nodes. Edges are created with `MATCH`+`CREATE`.

use crate::GraphEmitter;
use vedaksha_graph::{ChartGraph, Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

/// Emits `ChartGraph` as Neo4j Cypher statements.
pub struct CypherEmitter;

impl GraphEmitter for CypherEmitter {
    type Output = String;

    fn emit(&self, graph: &ChartGraph) -> Result<String, String> {
        let mut statements: Vec<String> = Vec::new();

        for node in &graph.nodes {
            statements.push(emit_node_cypher(node));
        }

        for edge in &graph.edges {
            statements.push(emit_edge_cypher(edge));
        }

        Ok(statements.join("\n"))
    }
}

fn emit_node_cypher(node: &Node) -> String {
    let label = node_type_label(node.node_type);
    let id = &node.id;
    let verb = if is_global_node(node.node_type) {
        "MERGE"
    } else {
        "CREATE"
    };
    let props = node_properties_cypher(&node.properties);
    format!("{verb} (n:{label} {{id: '{id}'{props}}})")
}

fn emit_edge_cypher(edge: &Edge) -> String {
    let rel_type = edge_type_label(edge.edge_type);
    let edge_props = edge_properties_cypher(&edge.properties);
    format!(
        "MATCH (a {{id: '{from}'}}), (b {{id: '{to}'}}) CREATE (a)-[:{rel_type}{edge_props}]->(b)",
        from = edge.from,
        to = edge.to,
    )
}

fn is_global_node(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Sign | NodeType::Nakshatra | NodeType::FixedStar
    )
}

fn node_type_label(t: NodeType) -> &'static str {
    match t {
        NodeType::Chart => "Chart",
        NodeType::Planet => "Planet",
        NodeType::Sign => "Sign",
        NodeType::House => "House",
        NodeType::Nakshatra => "Nakshatra",
        NodeType::Pada => "Pada",
        NodeType::Pattern => "Pattern",
        NodeType::DashaPeriod => "DashaPeriod",
        NodeType::Yoga => "Yoga",
        NodeType::FixedStar => "FixedStar",
    }
}

fn edge_type_label(t: EdgeType) -> &'static str {
    match t {
        EdgeType::PlacedIn => "PLACED_IN",
        EdgeType::Occupies => "OCCUPIES",
        EdgeType::Aspects => "ASPECTS",
        EdgeType::Rules => "RULES",
        EdgeType::Disposits => "DISPOSITS",
        EdgeType::CuspOf => "CUSP_OF",
        EdgeType::BelongsTo => "BELONGS_TO",
        EdgeType::PartOfPattern => "PART_OF_PATTERN",
        EdgeType::InNakshatra => "IN_NAKSHATRA",
        EdgeType::ConjunctStar => "CONJUNCT_STAR",
        EdgeType::DashaLord => "DASHA_LORD",
        EdgeType::ContainsPeriod => "CONTAINS_PERIOD",
        EdgeType::HasYoga => "HAS_YOGA",
    }
}

/// Produce a Cypher property string (comma-prefixed) from node properties.
#[allow(clippy::too_many_lines)]
fn node_properties_cypher(props: &NodeProperties) -> String {
    match props {
        NodeProperties::Chart {
            julian_day,
            latitude,
            longitude,
            classification,
        } => {
            let cls = format!("{classification:?}");
            format!(
                ", julian_day: {julian_day}, latitude: {latitude}, longitude: {longitude}, classification: '{cls}'"
            )
        }
        NodeProperties::Planet {
            name,
            longitude,
            latitude,
            distance,
            speed,
            retrograde,
            sign_index,
            house,
        } => {
            let name = escape_single_quotes(name);
            format!(
                ", name: '{name}', longitude: {longitude}, latitude: {latitude}, distance: {distance}, speed: {speed}, retrograde: {retrograde}, sign_index: {sign_index}, house: {house}"
            )
        }
        NodeProperties::Sign {
            name,
            index,
            element,
            modality,
        } => {
            let name = escape_single_quotes(name);
            let element = escape_single_quotes(element);
            let modality = escape_single_quotes(modality);
            format!(
                ", name: '{name}', index: {index}, element: '{element}', modality: '{modality}'"
            )
        }
        NodeProperties::House {
            number,
            cusp_longitude,
            system,
        } => {
            let system = escape_single_quotes(system);
            format!(", number: {number}, cusp_longitude: {cusp_longitude}, system: '{system}'")
        }
        NodeProperties::Nakshatra {
            name,
            index,
            lord,
            deity,
        } => {
            let name = escape_single_quotes(name);
            let lord = escape_single_quotes(lord);
            let deity = escape_single_quotes(deity);
            format!(", name: '{name}', index: {index}, lord: '{lord}', deity: '{deity}'")
        }
        NodeProperties::Pada {
            nakshatra_index,
            pada_number,
            start_longitude,
        } => {
            format!(
                ", nakshatra_index: {nakshatra_index}, pada_number: {pada_number}, start_longitude: {start_longitude}"
            )
        }
        NodeProperties::Pattern {
            pattern_type,
            description,
        } => {
            let pt = escape_single_quotes(pattern_type);
            let desc = escape_single_quotes(description);
            format!(", pattern_type: '{pt}', description: '{desc}'")
        }
        NodeProperties::DashaPeriod {
            lord,
            level,
            start_jd,
            end_jd,
            duration_days,
        } => {
            let lord = escape_single_quotes(lord);
            format!(
                ", lord: '{lord}', level: {level}, start_jd: {start_jd}, end_jd: {end_jd}, duration_days: {duration_days}"
            )
        }
        NodeProperties::Yoga {
            name,
            yoga_type,
            description,
        } => {
            let name = escape_single_quotes(name);
            let yt = escape_single_quotes(yoga_type);
            let desc = escape_single_quotes(description);
            format!(", name: '{name}', yoga_type: '{yt}', description: '{desc}'")
        }
        NodeProperties::FixedStar {
            name,
            longitude,
            latitude,
            magnitude,
        } => {
            let name = escape_single_quotes(name);
            format!(
                ", name: '{name}', longitude: {longitude}, latitude: {latitude}, magnitude: {magnitude}"
            )
        }
    }
}

/// Produce a Cypher property map string for an edge (space-prefixed if non-empty).
fn edge_properties_cypher(props: &EdgeProperties) -> String {
    match props {
        EdgeProperties::None => String::new(),
        EdgeProperties::Aspect {
            aspect_type,
            orb,
            applying,
            strength,
        } => {
            let at = escape_single_quotes(aspect_type);
            format!(
                " {{aspect_type: '{at}', orb: {orb}, applying: {applying}, strength: {strength}}}"
            )
        }
        EdgeProperties::StarConjunction { orb } => {
            format!(" {{orb: {orb}}}")
        }
    }
}

fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_graph::{
        classification::DataClassification, ids::NodeId, ontology::EdgeProperties,
    };

    fn make_test_graph() -> ChartGraph {
        let chart_id = NodeId::chart_scoped("test", "chart", "root");
        let mut g = ChartGraph::new(chart_id, DataClassification::Anonymous);

        // Chart-scoped planet node
        g.add_node(vedaksha_graph::ontology::Node {
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

        // Global sign node
        g.add_node(vedaksha_graph::ontology::Node {
            id: NodeId::global("sign", "capricorn"),
            node_type: NodeType::Sign,
            properties: NodeProperties::Sign {
                name: "Capricorn".to_string(),
                index: 9,
                element: "Earth".to_string(),
                modality: "Cardinal".to_string(),
            },
        });

        // Global nakshatra node
        g.add_node(vedaksha_graph::ontology::Node {
            id: NodeId::global("nakshatra", "shravana"),
            node_type: NodeType::Nakshatra,
            properties: NodeProperties::Nakshatra {
                name: "Shravana".to_string(),
                index: 22,
                lord: "Moon".to_string(),
                deity: "Vishnu".to_string(),
            },
        });

        // PlacedIn edge (no properties)
        g.add_edge(vedaksha_graph::ontology::Edge {
            edge_type: EdgeType::PlacedIn,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::global("sign", "capricorn"),
            properties: EdgeProperties::None,
        });

        // Aspects edge with properties
        g.add_edge(vedaksha_graph::ontology::Edge {
            edge_type: EdgeType::Aspects,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::chart_scoped("test", "planet", "jupiter"),
            properties: EdgeProperties::Aspect {
                aspect_type: "Trine".to_string(),
                orb: 2.3,
                applying: true,
                strength: 0.85,
            },
        });

        g
    }

    #[test]
    fn global_node_uses_merge() {
        let graph = make_test_graph();
        let output = CypherEmitter.emit(&graph).expect("emit");
        // Sign and Nakshatra are global — should use MERGE
        let lines: Vec<&str> = output.lines().collect();
        let sign_line = lines
            .iter()
            .find(|l| l.contains(":Sign"))
            .expect("sign line");
        assert!(
            sign_line.starts_with("MERGE"),
            "Sign node should use MERGE, got: {sign_line}"
        );
        let nak_line = lines
            .iter()
            .find(|l| l.contains(":Nakshatra"))
            .expect("nakshatra line");
        assert!(
            nak_line.starts_with("MERGE"),
            "Nakshatra node should use MERGE, got: {nak_line}"
        );
    }

    #[test]
    fn chart_scoped_node_uses_create() {
        let graph = make_test_graph();
        let output = CypherEmitter.emit(&graph).expect("emit");
        let lines: Vec<&str> = output.lines().collect();
        let planet_line = lines
            .iter()
            .find(|l| l.contains(":Planet"))
            .expect("planet line");
        assert!(
            planet_line.starts_with("CREATE"),
            "Planet node should use CREATE, got: {planet_line}"
        );
    }

    #[test]
    fn output_contains_correct_label() {
        let graph = make_test_graph();
        let output = CypherEmitter.emit(&graph).expect("emit");
        assert!(
            output.contains(":Planet"),
            "output should contain :Planet label"
        );
        assert!(
            output.contains(":Sign"),
            "output should contain :Sign label"
        );
        assert!(
            output.contains(":Nakshatra"),
            "output should contain :Nakshatra label"
        );
    }

    #[test]
    fn edge_output_has_correct_relationship_type() {
        let graph = make_test_graph();
        let output = CypherEmitter.emit(&graph).expect("emit");
        assert!(
            output.contains(":PLACED_IN"),
            "output should contain :PLACED_IN relationship"
        );
        assert!(
            output.contains(":ASPECTS"),
            "output should contain :ASPECTS relationship"
        );
    }

    #[test]
    fn properties_included_in_output() {
        let graph = make_test_graph();
        let output = CypherEmitter.emit(&graph).expect("emit");
        // Planet properties
        assert!(
            output.contains("longitude: 280.5"),
            "should include longitude"
        );
        assert!(
            output.contains("retrograde: true"),
            "should include retrograde flag"
        );
        // Aspect edge properties
        assert!(
            output.contains("aspect_type: 'Trine'"),
            "should include aspect_type"
        );
        assert!(output.contains("orb: 2.3"), "should include orb");
    }
}
