// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! RAG-optimized natural language text emitter.
//!
//! Produces a structured English description of the chart graph suitable
//! for vector embedding in RAG pipelines. Each entity (planet, sign, house,
//! nakshatra, yoga, dasha period, pattern, fixed star) is described in a
//! human-readable sentence. Aspect relationships are woven inline.

use crate::GraphEmitter;
use vedaksha_graph::{ChartGraph, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

/// Emits `ChartGraph` as RAG-optimized natural language text.
pub struct EmbeddingTextEmitter;

impl GraphEmitter for EmbeddingTextEmitter {
    type Output = String;

    #[allow(clippy::too_many_lines)]
    fn emit(&self, graph: &ChartGraph) -> Result<String, String> {
        let mut lines: Vec<String> = Vec::new();

        // Chart header
        for node in &graph.nodes {
            if node.node_type == NodeType::Chart {
                if let NodeProperties::Chart {
                    julian_day,
                    latitude,
                    longitude,
                    ..
                } = &node.properties
                {
                    lines.push(format!(
                        "Chart computed at JD {julian_day:.4} (lat {latitude:.2}, lon {longitude:.2})."
                    ));
                }
            }
        }

        // Planets
        for node in &graph.nodes {
            if node.node_type == NodeType::Planet {
                if let NodeProperties::Planet {
                    name,
                    longitude,
                    speed,
                    retrograde,
                    house,
                    ..
                } = &node.properties
                {
                    let retro_str = if *retrograde { ", retrograde" } else { "" };
                    let sign_name = planet_sign_name(graph, &node.id);
                    let sign_part = sign_name
                        .as_deref()
                        .map_or(String::new(), |s| format!(" {s}"));
                    lines.push(format!(
                        "{name} at {longitude:.1}°{sign_part} (House {house}){retro_str}, speed {speed:.2}°/day."
                    ));

                    // Aspect edges from this planet
                    for edge in graph.edges_from(&node.id) {
                        if edge.edge_type == EdgeType::Aspects {
                            if let EdgeProperties::Aspect {
                                aspect_type,
                                orb,
                                applying,
                                ..
                            } = &edge.properties
                            {
                                let applying_str =
                                    if *applying { "applying" } else { "separating" };
                                let target_name = node_name_by_id(graph, &edge.to)
                                    .unwrap_or_else(|| edge.to.to_string());
                                lines.push(format!(
                                    "{name} aspects {target_name} ({aspect_type}, orb {orb:.1}°, {applying_str})."
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Signs (brief)
        for node in &graph.nodes {
            if node.node_type == NodeType::Sign {
                if let NodeProperties::Sign {
                    name,
                    element,
                    modality,
                    ..
                } = &node.properties
                {
                    lines.push(format!("{name} is a {modality} {element} sign."));
                }
            }
        }

        // Houses
        for node in &graph.nodes {
            if node.node_type == NodeType::House {
                if let NodeProperties::House {
                    number,
                    cusp_longitude,
                    system,
                } = &node.properties
                {
                    lines.push(format!(
                        "House {number} cusp at {cusp_longitude:.1}° ({system} system)."
                    ));
                }
            }
        }

        // Nakshatras
        for node in &graph.nodes {
            if node.node_type == NodeType::Nakshatra {
                if let NodeProperties::Nakshatra {
                    name, lord, deity, ..
                } = &node.properties
                {
                    lines.push(format!("{name} nakshatra: lord {lord}, deity {deity}."));
                }
            }
        }

        // Padas
        for node in &graph.nodes {
            if node.node_type == NodeType::Pada {
                if let NodeProperties::Pada {
                    nakshatra_index,
                    pada_number,
                    start_longitude,
                } = &node.properties
                {
                    lines.push(format!(
                        "Pada {pada_number} of nakshatra {nakshatra_index} starts at {start_longitude:.2}°."
                    ));
                }
            }
        }

        // Yogas
        for node in &graph.nodes {
            if node.node_type == NodeType::Yoga {
                if let NodeProperties::Yoga {
                    name,
                    yoga_type,
                    description,
                } = &node.properties
                {
                    lines.push(format!("{name} ({yoga_type}): {description}"));
                }
            }
        }

        // Dasha periods
        for node in &graph.nodes {
            if node.node_type == NodeType::DashaPeriod {
                if let NodeProperties::DashaPeriod {
                    lord,
                    level,
                    duration_days,
                    ..
                } = &node.properties
                {
                    let level_name = match level {
                        1 => "Mahadasha",
                        2 => "Antardasha",
                        3 => "Pratyantardasha",
                        _ => "Dasha sub-period",
                    };
                    lines.push(format!(
                        "{lord} {level_name} lasting {duration_days:.1} days."
                    ));
                }
            }
        }

        // Patterns
        for node in &graph.nodes {
            if node.node_type == NodeType::Pattern {
                if let NodeProperties::Pattern {
                    pattern_type,
                    description,
                } = &node.properties
                {
                    lines.push(format!("{pattern_type}: {description}"));
                }
            }
        }

        // Fixed stars
        for node in &graph.nodes {
            if node.node_type == NodeType::FixedStar {
                if let NodeProperties::FixedStar {
                    name,
                    longitude,
                    magnitude,
                    ..
                } = &node.properties
                {
                    lines.push(format!(
                        "Fixed star {name} at {longitude:.2}°, magnitude {magnitude:.1}."
                    ));
                }
            }
        }

        // Star conjunctions
        for edge in &graph.edges {
            if edge.edge_type == EdgeType::ConjunctStar {
                if let EdgeProperties::StarConjunction { orb } = &edge.properties {
                    let planet =
                        node_name_by_id(graph, &edge.from).unwrap_or_else(|| edge.from.to_string());
                    let star =
                        node_name_by_id(graph, &edge.to).unwrap_or_else(|| edge.to.to_string());
                    lines.push(format!(
                        "{planet} is conjunct fixed star {star} (orb {orb:.1}°)."
                    ));
                }
            }
        }

        Ok(lines.join("\n"))
    }
}

/// Look up the sign name for a planet node by following `PlacedIn` edges.
fn planet_sign_name(graph: &ChartGraph, planet_id: &vedaksha_graph::NodeId) -> Option<String> {
    for edge in graph.edges_from(planet_id) {
        if edge.edge_type == EdgeType::PlacedIn {
            if let Some(sign_node) = graph.find_node(&edge.to) {
                if let NodeProperties::Sign { name, .. } = &sign_node.properties {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Get a human-readable name for a node by ID.
fn node_name_by_id(graph: &ChartGraph, id: &vedaksha_graph::NodeId) -> Option<String> {
    graph.find_node(id).and_then(node_display_name)
}

fn node_display_name(node: &Node) -> Option<String> {
    match &node.properties {
        NodeProperties::Planet { name, .. }
        | NodeProperties::Sign { name, .. }
        | NodeProperties::Nakshatra { name, .. }
        | NodeProperties::Yoga { name, .. }
        | NodeProperties::FixedStar { name, .. } => Some(name.clone()),
        NodeProperties::DashaPeriod { lord, .. } => Some(lord.clone()),
        _ => None,
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

        // Chart node
        g.add_node(Node {
            id: NodeId::chart_scoped("test", "chart", "root"),
            node_type: NodeType::Chart,
            properties: NodeProperties::Chart {
                julian_day: 2_451_545.0,
                latitude: 28.61,
                longitude: 77.23,
                classification: DataClassification::Anonymous,
            },
        });

        // Mars planet
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

        // Jupiter planet
        g.add_node(Node {
            id: NodeId::chart_scoped("test", "planet", "jupiter"),
            node_type: NodeType::Planet,
            properties: NodeProperties::Planet {
                name: "Jupiter".to_string(),
                longitude: 40.2,
                latitude: 0.5,
                distance: 5.2,
                speed: 0.1,
                retrograde: false,
                sign_index: 1,
                house: 2,
            },
        });

        // Capricorn sign
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

        // Ruchaka Yoga
        g.add_node(Node {
            id: NodeId::chart_scoped("test", "yoga", "ruchaka"),
            node_type: NodeType::Yoga,
            properties: NodeProperties::Yoga {
                name: "Ruchaka Yoga".to_string(),
                yoga_type: "Pancha Mahapurusha".to_string(),
                description: "Mars in own/exalted sign in kendra.".to_string(),
            },
        });

        // Mars placed in Capricorn
        g.add_edge(Edge {
            edge_type: EdgeType::PlacedIn,
            from: NodeId::chart_scoped("test", "planet", "mars"),
            to: NodeId::global("sign", "capricorn"),
            properties: EdgeProperties::None,
        });

        // Mars aspects Jupiter
        g.add_edge(Edge {
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
    fn output_contains_planet_descriptions() {
        let graph = make_test_graph();
        let output = EmbeddingTextEmitter.emit(&graph).expect("emit");
        assert!(output.contains("Mars"), "output should mention Mars");
        assert!(output.contains("Jupiter"), "output should mention Jupiter");
        assert!(
            output.contains("280.5°"),
            "output should contain Mars longitude"
        );
        assert!(
            output.contains("retrograde"),
            "output should mention retrograde"
        );
    }

    #[test]
    fn output_is_non_empty_for_graph_with_nodes() {
        let graph = make_test_graph();
        let output = EmbeddingTextEmitter.emit(&graph).expect("emit");
        assert!(
            !output.is_empty(),
            "output should be non-empty for a graph with nodes"
        );
        // Should have multiple lines
        let line_count = output.lines().count();
        assert!(
            line_count >= 3,
            "output should have at least 3 lines, got {line_count}"
        );
    }

    #[test]
    fn output_contains_readable_english() {
        let graph = make_test_graph();
        let output = EmbeddingTextEmitter.emit(&graph).expect("emit");
        // Chart header
        assert!(
            output.contains("Chart computed at JD"),
            "should have chart header sentence"
        );
        // Aspect description
        assert!(
            output.contains("aspects Jupiter (Trine"),
            "should describe aspect in English"
        );
        // Yoga description
        assert!(
            output.contains("Ruchaka Yoga"),
            "should describe yoga by name"
        );
        // Sign description
        assert!(
            output.contains("Capricorn is a Cardinal Earth sign"),
            "should describe sign element and modality"
        );
    }
}
