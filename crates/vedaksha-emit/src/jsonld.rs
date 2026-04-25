// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! JSON-LD emitter.
//!
//! Emits a JSON-LD document with `@context` (vedaksha ontology namespace)
//! and `@graph` array containing all nodes with `@type` and `@id`.

use crate::GraphEmitter;
use vedaksha_graph::{ChartGraph, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Emits `ChartGraph` as a JSON-LD document.
pub struct JsonLdEmitter;

const ONTOLOGY_BASE: &str = "https://vedaksha.net/ontology/";

impl GraphEmitter for JsonLdEmitter {
    type Output = String;

    fn emit(&self, graph: &ChartGraph) -> Result<String, String> {
        let context = build_context();
        let graph_array = build_graph_array(graph);

        let doc = serde_json::json!({
            "@context": context,
            "@graph": graph_array,
        });

        serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())
    }
}

fn build_context() -> serde_json::Value {
    serde_json::json!({
        "vedaksha": ONTOLOGY_BASE,
        "xsd": "http://www.w3.org/2001/XMLSchema#",
        "Chart": "vedaksha:Chart",
        "Planet": "vedaksha:Planet",
        "Sign": "vedaksha:Sign",
        "House": "vedaksha:House",
        "Nakshatra": "vedaksha:Nakshatra",
        "Pada": "vedaksha:Pada",
        "Pattern": "vedaksha:Pattern",
        "DashaPeriod": "vedaksha:DashaPeriod",
        "Yoga": "vedaksha:Yoga",
        "FixedStar": "vedaksha:FixedStar",
        "PlacedIn": "vedaksha:PlacedIn",
        "Occupies": "vedaksha:Occupies",
        "Aspects": "vedaksha:Aspects",
        "Rules": "vedaksha:Rules",
        "Disposits": "vedaksha:Disposits",
        "CuspOf": "vedaksha:CuspOf",
        "BelongsTo": "vedaksha:BelongsTo",
        "PartOfPattern": "vedaksha:PartOfPattern",
        "InNakshatra": "vedaksha:InNakshatra",
        "ConjunctStar": "vedaksha:ConjunctStar",
        "DashaLord": "vedaksha:DashaLord",
        "ContainsPeriod": "vedaksha:ContainsPeriod",
        "HasYoga": "vedaksha:HasYoga",
    })
}

fn build_graph_array(graph: &ChartGraph) -> serde_json::Value {
    let mut items: Vec<serde_json::Value> = Vec::new();

    // Nodes
    for node in &graph.nodes {
        items.push(node_to_jsonld(node));
    }

    // Edges as reified triples
    for edge in &graph.edges {
        let edge_type = edge_type_term(edge.edge_type);
        let mut obj = serde_json::json!({
            "@type": edge_type,
            "from": { "@id": edge.from.to_string() },
            "to": { "@id": edge.to.to_string() },
        });

        match &edge.properties {
            EdgeProperties::None => {}
            EdgeProperties::Aspect {
                aspect_type,
                orb,
                applying,
                strength,
            } => {
                obj["aspectType"] = serde_json::Value::String(aspect_type.clone());
                obj["orb"] = serde_json::json!(orb);
                obj["applying"] = serde_json::json!(applying);
                obj["strength"] = serde_json::json!(strength);
            }
            EdgeProperties::StarConjunction { orb } => {
                obj["orb"] = serde_json::json!(orb);
            }
        }

        items.push(obj);
    }

    serde_json::Value::Array(items)
}

#[allow(clippy::too_many_lines)]
fn node_to_jsonld(node: &Node) -> serde_json::Value {
    let type_term = node_type_term(node.node_type);
    let id = node.id.to_string();

    let mut obj = serde_json::json!({
        "@type": type_term,
        "@id": id,
    });

    // Merge in type-specific properties
    match &node.properties {
        NodeProperties::Chart {
            julian_day,
            latitude,
            longitude,
            classification,
        } => {
            obj["julianDay"] = serde_json::json!(julian_day);
            obj["latitude"] = serde_json::json!(latitude);
            obj["longitude"] = serde_json::json!(longitude);
            obj["classification"] = serde_json::Value::String(format!("{classification:?}"));
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
            obj["name"] = serde_json::Value::String(name.clone());
            obj["longitude"] = serde_json::json!(longitude);
            obj["latitude"] = serde_json::json!(latitude);
            obj["distance"] = serde_json::json!(distance);
            obj["speed"] = serde_json::json!(speed);
            obj["retrograde"] = serde_json::json!(retrograde);
            obj["signIndex"] = serde_json::json!(sign_index);
            obj["house"] = serde_json::json!(house);
        }
        NodeProperties::Sign {
            name,
            index,
            element,
            modality,
        } => {
            obj["name"] = serde_json::Value::String(name.clone());
            obj["index"] = serde_json::json!(index);
            obj["element"] = serde_json::Value::String(element.clone());
            obj["modality"] = serde_json::Value::String(modality.clone());
        }
        NodeProperties::House {
            number,
            cusp_longitude,
            system,
        } => {
            obj["number"] = serde_json::json!(number);
            obj["cuspLongitude"] = serde_json::json!(cusp_longitude);
            obj["system"] = serde_json::Value::String(system.clone());
        }
        NodeProperties::Nakshatra {
            name,
            index,
            lord,
            deity,
        } => {
            obj["name"] = serde_json::Value::String(name.clone());
            obj["index"] = serde_json::json!(index);
            obj["lord"] = serde_json::Value::String(lord.clone());
            obj["deity"] = serde_json::Value::String(deity.clone());
        }
        NodeProperties::Pada {
            nakshatra_index,
            pada_number,
            start_longitude,
        } => {
            obj["nakshatraIndex"] = serde_json::json!(nakshatra_index);
            obj["padaNumber"] = serde_json::json!(pada_number);
            obj["startLongitude"] = serde_json::json!(start_longitude);
        }
        NodeProperties::Pattern {
            pattern_type,
            description,
        } => {
            obj["patternType"] = serde_json::Value::String(pattern_type.clone());
            obj["description"] = serde_json::Value::String(description.clone());
        }
        NodeProperties::DashaPeriod {
            lord,
            level,
            start_jd,
            end_jd,
            duration_days,
        } => {
            obj["lord"] = serde_json::Value::String(lord.clone());
            obj["level"] = serde_json::json!(level);
            obj["startJd"] = serde_json::json!(start_jd);
            obj["endJd"] = serde_json::json!(end_jd);
            obj["durationDays"] = serde_json::json!(duration_days);
        }
        NodeProperties::Yoga {
            name,
            yoga_type,
            description,
        } => {
            obj["name"] = serde_json::Value::String(name.clone());
            obj["yogaType"] = serde_json::Value::String(yoga_type.clone());
            obj["description"] = serde_json::Value::String(description.clone());
        }
        NodeProperties::FixedStar {
            name,
            longitude,
            latitude,
            magnitude,
        } => {
            obj["name"] = serde_json::Value::String(name.clone());
            obj["longitude"] = serde_json::json!(longitude);
            obj["latitude"] = serde_json::json!(latitude);
            obj["magnitude"] = serde_json::json!(magnitude);
        }
    }

    obj
}

fn node_type_term(t: NodeType) -> &'static str {
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

fn edge_type_term(t: EdgeType) -> &'static str {
    match t {
        EdgeType::PlacedIn => "PlacedIn",
        EdgeType::Occupies => "Occupies",
        EdgeType::Aspects => "Aspects",
        EdgeType::Rules => "Rules",
        EdgeType::Disposits => "Disposits",
        EdgeType::CuspOf => "CuspOf",
        EdgeType::BelongsTo => "BelongsTo",
        EdgeType::PartOfPattern => "PartOfPattern",
        EdgeType::InNakshatra => "InNakshatra",
        EdgeType::ConjunctStar => "ConjunctStar",
        EdgeType::DashaLord => "DashaLord",
        EdgeType::ContainsPeriod => "ContainsPeriod",
        EdgeType::HasYoga => "HasYoga",
    }
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

        g
    }

    #[test]
    fn output_has_context() {
        let graph = make_test_graph();
        let output = JsonLdEmitter.emit(&graph).expect("emit");
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert!(
            parsed["@context"].is_object(),
            "should have @context object"
        );
        assert_eq!(
            parsed["@context"]["vedaksha"],
            serde_json::Value::String(ONTOLOGY_BASE.to_string()),
            "context should include vedaksha namespace"
        );
    }

    #[test]
    fn output_has_graph_array() {
        let graph = make_test_graph();
        let output = JsonLdEmitter.emit(&graph).expect("emit");
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert!(parsed["@graph"].is_array(), "should have @graph array");
        // 2 nodes + 1 edge
        assert_eq!(
            parsed["@graph"].as_array().unwrap().len(),
            3,
            "graph array should contain 2 nodes + 1 edge"
        );
    }

    #[test]
    fn nodes_have_type_and_id() {
        let graph = make_test_graph();
        let output = JsonLdEmitter.emit(&graph).expect("emit");
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        let graph_arr = parsed["@graph"].as_array().unwrap();

        // Find the planet node
        let planet = graph_arr
            .iter()
            .find(|item| item["@type"] == "Planet")
            .expect("should find Planet node");

        assert!(planet["@id"].is_string(), "Planet node should have @id");
        assert!(
            planet["@id"].as_str().unwrap().contains("mars"),
            "Planet @id should reference mars"
        );
        assert_eq!(
            planet["@type"], "Planet",
            "Planet node should have @type = Planet"
        );

        // Verify the sign node
        let sign = graph_arr
            .iter()
            .find(|item| item["@type"] == "Sign")
            .expect("should find Sign node");

        assert!(sign["@id"].is_string(), "Sign node should have @id");
        assert_eq!(sign["@type"], "Sign", "Sign node should have @type = Sign");
    }
}
