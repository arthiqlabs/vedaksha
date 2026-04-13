// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `SurrealDB` `SurrealQL` emitter.
//!
//! Emits `CREATE` statements for nodes and `RELATE` statements for edges.
//! Global nodes (Sign, Nakshatra, `FixedStar`) use `INSERT IGNORE` semantics
//! via `CREATE ... IF NOT EXISTS` style (or plain CREATE — `SurrealDB` is idempotent
//! when using record IDs).

use crate::GraphEmitter;
use vedaksha_graph::{ChartGraph, Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

/// Emits `ChartGraph` as `SurrealDB` `SurrealQL` statements.
pub struct SurrealEmitter;

impl GraphEmitter for SurrealEmitter {
    type Output = String;

    fn emit(&self, graph: &ChartGraph) -> Result<String, String> {
        let mut statements: Vec<String> = Vec::new();

        for node in &graph.nodes {
            statements.push(emit_node_surreal(node));
        }

        for edge in &graph.edges {
            statements.push(emit_edge_surreal(edge));
        }

        Ok(statements.join("\n"))
    }
}

fn emit_node_surreal(node: &Node) -> String {
    let table = node_type_table(node.node_type);
    // SurrealDB record ID: table:⟨id⟩ — sanitize the node ID for use as record ID
    let record_id = sanitize_surreal_id(&node.id.to_string());
    let props = node_properties_surreal(&node.properties);
    if props.is_empty() {
        format!("CREATE {table}:{record_id};")
    } else {
        format!("CREATE {table}:{record_id} SET {props};")
    }
}

fn emit_edge_surreal(edge: &Edge) -> String {
    let rel_table = edge_type_table(edge.edge_type);
    let from_id = sanitize_surreal_id(&edge.from.to_string());
    let to_id = sanitize_surreal_id(&edge.to.to_string());
    let props = edge_properties_surreal(&edge.properties);

    // RELATE source->relation->target
    let relate = format!("RELATE node:{from_id}->{rel_table}->node:{to_id}");
    if props.is_empty() {
        format!("{relate};")
    } else {
        format!("{relate} SET {props};")
    }
}

fn node_type_table(t: NodeType) -> &'static str {
    match t {
        NodeType::Chart => "chart",
        NodeType::Planet => "planet",
        NodeType::Sign => "sign",
        NodeType::House => "house",
        NodeType::Nakshatra => "nakshatra",
        NodeType::Pada => "pada",
        NodeType::Pattern => "pattern",
        NodeType::DashaPeriod => "dasha_period",
        NodeType::Yoga => "yoga",
        NodeType::FixedStar => "fixed_star",
    }
}

fn edge_type_table(t: EdgeType) -> &'static str {
    match t {
        EdgeType::PlacedIn => "placed_in",
        EdgeType::Occupies => "occupies",
        EdgeType::Aspects => "aspects",
        EdgeType::Rules => "rules",
        EdgeType::Disposits => "disposits",
        EdgeType::CuspOf => "cusp_of",
        EdgeType::BelongsTo => "belongs_to",
        EdgeType::PartOfPattern => "part_of_pattern",
        EdgeType::InNakshatra => "in_nakshatra",
        EdgeType::ConjunctStar => "conjunct_star",
        EdgeType::DashaLord => "dasha_lord",
        EdgeType::ContainsPeriod => "contains_period",
        EdgeType::HasYoga => "has_yoga",
    }
}

/// Sanitize a node ID string for use as a `SurrealDB` record ID.
/// Replaces characters that need escaping; `SurrealDB` allows `⟨...⟩` for complex IDs.
fn sanitize_surreal_id(id: &str) -> String {
    // Wrap in backtick-style escape using SurrealDB's ⟨⟩ notation for complex IDs
    format!("⟨{id}⟩")
}

#[allow(clippy::too_many_lines)]
fn node_properties_surreal(props: &NodeProperties) -> String {
    match props {
        NodeProperties::Chart {
            julian_day,
            latitude,
            longitude,
            classification,
        } => {
            let cls = format!("{classification:?}");
            format!(
                "julian_day = {julian_day}, latitude = {latitude}, longitude = {longitude}, classification = '{cls}'"
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
            let name = escape_surreal_string(name);
            format!(
                "name = '{name}', longitude = {longitude}, latitude = {latitude}, distance = {distance}, speed = {speed}, retrograde = {retrograde}, sign_index = {sign_index}, house = {house}"
            )
        }
        NodeProperties::Sign {
            name,
            index,
            element,
            modality,
        } => {
            let name = escape_surreal_string(name);
            let element = escape_surreal_string(element);
            let modality = escape_surreal_string(modality);
            format!(
                "name = '{name}', index = {index}, element = '{element}', modality = '{modality}'"
            )
        }
        NodeProperties::House {
            number,
            cusp_longitude,
            system,
        } => {
            let system = escape_surreal_string(system);
            format!("number = {number}, cusp_longitude = {cusp_longitude}, system = '{system}'")
        }
        NodeProperties::Nakshatra {
            name,
            index,
            lord,
            deity,
        } => {
            let name = escape_surreal_string(name);
            let lord = escape_surreal_string(lord);
            let deity = escape_surreal_string(deity);
            format!("name = '{name}', index = {index}, lord = '{lord}', deity = '{deity}'")
        }
        NodeProperties::Pada {
            nakshatra_index,
            pada_number,
            start_longitude,
        } => {
            format!(
                "nakshatra_index = {nakshatra_index}, pada_number = {pada_number}, start_longitude = {start_longitude}"
            )
        }
        NodeProperties::Pattern {
            pattern_type,
            description,
        } => {
            let pt = escape_surreal_string(pattern_type);
            let desc = escape_surreal_string(description);
            format!("pattern_type = '{pt}', description = '{desc}'")
        }
        NodeProperties::DashaPeriod {
            lord,
            level,
            start_jd,
            end_jd,
            duration_days,
        } => {
            let lord = escape_surreal_string(lord);
            format!(
                "lord = '{lord}', level = {level}, start_jd = {start_jd}, end_jd = {end_jd}, duration_days = {duration_days}"
            )
        }
        NodeProperties::Yoga {
            name,
            yoga_type,
            description,
        } => {
            let name = escape_surreal_string(name);
            let yt = escape_surreal_string(yoga_type);
            let desc = escape_surreal_string(description);
            format!("name = '{name}', yoga_type = '{yt}', description = '{desc}'")
        }
        NodeProperties::FixedStar {
            name,
            longitude,
            latitude,
            magnitude,
        } => {
            let name = escape_surreal_string(name);
            format!(
                "name = '{name}', longitude = {longitude}, latitude = {latitude}, magnitude = {magnitude}"
            )
        }
    }
}

fn edge_properties_surreal(props: &EdgeProperties) -> String {
    match props {
        EdgeProperties::None => String::new(),
        EdgeProperties::Aspect {
            aspect_type,
            orb,
            applying,
            strength,
        } => {
            let at = escape_surreal_string(aspect_type);
            format!(
                "aspect_type = '{at}', orb = {orb}, applying = {applying}, strength = {strength}"
            )
        }
        EdgeProperties::StarConjunction { orb } => {
            format!("orb = {orb}")
        }
    }
}

fn escape_surreal_string(s: &str) -> String {
    s.replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_graph::{
        classification::DataClassification,
        ids::NodeId,
        ontology::{Edge, EdgeProperties, Node, NodeProperties},
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

        g.add_edge(Edge {
            edge_type: EdgeType::Aspects,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::chart_scoped("test", "planet", "jupiter"),
            properties: EdgeProperties::Aspect {
                aspect_type: "Trine".to_string(),
                orb: 2.3,
                applying: false,
                strength: 0.7,
            },
        });

        g
    }

    #[test]
    fn node_creates_with_correct_type() {
        let graph = make_test_graph();
        let output = SurrealEmitter.emit(&graph).expect("emit");
        // Planet node
        assert!(
            output.contains("CREATE planet:"),
            "should create planet record"
        );
        // Sign node
        assert!(output.contains("CREATE sign:"), "should create sign record");
    }

    #[test]
    fn relate_syntax_is_correct() {
        let graph = make_test_graph();
        let output = SurrealEmitter.emit(&graph).expect("emit");
        assert!(
            output.contains("RELATE node:"),
            "edges should use RELATE syntax"
        );
        assert!(
            output.contains("->placed_in->"),
            "should use placed_in relation table"
        );
        assert!(
            output.contains("->aspects->"),
            "should use aspects relation table"
        );
    }

    #[test]
    fn properties_included_in_surreal_output() {
        let graph = make_test_graph();
        let output = SurrealEmitter.emit(&graph).expect("emit");
        assert!(
            output.contains("longitude = 280.5"),
            "should include longitude"
        );
        assert!(
            output.contains("retrograde = true"),
            "should include retrograde"
        );
        assert!(
            output.contains("aspect_type = 'Trine'"),
            "should include aspect_type"
        );
        assert!(output.contains("orb = 2.3"), "should include orb value");
    }
}
