// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Node and Edge type definitions for the astrological property graph.

use serde::{Deserialize, Serialize};

use crate::classification::DataClassification;
use crate::ids::NodeId;

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

/// A node in the astrological property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Discriminant for the kind of node.
    pub node_type: NodeType,
    /// Type-specific payload.
    pub properties: NodeProperties,
}

/// The type of a graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Chart,
    Planet,
    Sign,
    House,
    Nakshatra,
    Pada,
    Pattern,
    DashaPeriod,
    Yoga,
    FixedStar,
}

/// Properties specific to each node type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeProperties {
    /// Chart metadata.
    Chart {
        julian_day: f64,
        latitude: f64,
        longitude: f64,
        classification: DataClassification,
    },
    /// Planetary body.
    Planet {
        name: String,
        longitude: f64,
        latitude: f64,
        distance: f64,
        speed: f64,
        retrograde: bool,
        sign_index: u8,
        house: u8,
    },
    /// Zodiac sign.
    Sign {
        name: String,
        index: u8,
        element: String,
        modality: String,
    },
    /// House in a house system.
    House {
        number: u8,
        cusp_longitude: f64,
        system: String,
    },
    /// Lunar mansion.
    Nakshatra {
        name: String,
        index: u8,
        lord: String,
        deity: String,
    },
    /// Sub-division of a nakshatra.
    Pada {
        nakshatra_index: u8,
        pada_number: u8,
        start_longitude: f64,
    },
    /// Geometric pattern (e.g. Grand Trine, T-Square).
    Pattern {
        pattern_type: String,
        description: String,
    },
    /// A period in the Vimshottari or other dasha system.
    DashaPeriod {
        lord: String,
        level: u8,
        start_jd: f64,
        end_jd: f64,
        duration_days: f64,
    },
    /// An astrological yoga.
    Yoga {
        name: String,
        yoga_type: String,
        description: String,
    },
    /// A fixed star.
    FixedStar {
        name: String,
        longitude: f64,
        latitude: f64,
        magnitude: f64,
    },
}

// ---------------------------------------------------------------------------
// Edges
// ---------------------------------------------------------------------------

/// An edge (relationship) in the astrological property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Kind of relationship.
    pub edge_type: EdgeType,
    /// Source node.
    pub from: NodeId,
    /// Target node.
    pub to: NodeId,
    /// Optional properties on this edge.
    pub properties: EdgeProperties,
}

/// The type of a graph edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Planet is placed in Sign.
    PlacedIn,
    /// Planet occupies House.
    Occupies,
    /// Planet aspects Planet.
    Aspects,
    /// Planet rules Sign (domicile ruler).
    Rules,
    /// Planet disposits Planet (dispositor chain).
    Disposits,
    /// House cusp falls in Sign.
    CuspOf,
    /// Node belongs to Chart.
    BelongsTo,
    /// Planet participates in Pattern.
    PartOfPattern,
    /// Planet is in Nakshatra.
    InNakshatra,
    /// Planet is conjunct a `FixedStar`.
    ConjunctStar,
    /// `DashaPeriod` is ruled by Planet.
    DashaLord,
    /// `DashaPeriod` contains child `DashaPeriod`.
    ContainsPeriod,
    /// Chart has Yoga.
    HasYoga,
}

/// Properties on edges (optional, depends on edge type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeProperties {
    /// No additional properties.
    None,
    /// Aspect edge properties.
    Aspect {
        aspect_type: String,
        orb: f64,
        applying: bool,
        strength: f64,
    },
    /// Conjunction with fixed star.
    StarConjunction { orb: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_planet_node() -> Node {
        Node {
            id: NodeId::chart_scoped("abc", "planet", "mars"),
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
        }
    }

    fn sample_aspect_edge() -> Edge {
        Edge {
            edge_type: EdgeType::Aspects,
            from: NodeId::chart_scoped("abc", "planet", "mars"),
            to: NodeId::chart_scoped("abc", "planet", "jupiter"),
            properties: EdgeProperties::Aspect {
                aspect_type: "Trine".to_string(),
                orb: 2.3,
                applying: true,
                strength: 0.85,
            },
        }
    }

    #[test]
    fn create_planet_node() {
        let node = sample_planet_node();
        assert_eq!(node.node_type, NodeType::Planet);
        if let NodeProperties::Planet {
            name, retrograde, ..
        } = &node.properties
        {
            assert_eq!(name, "Mars");
            assert!(!retrograde);
        } else {
            panic!("expected Planet properties");
        }
    }

    #[test]
    fn create_aspect_edge() {
        let edge = sample_aspect_edge();
        assert_eq!(edge.edge_type, EdgeType::Aspects);
        if let EdgeProperties::Aspect { orb, applying, .. } = &edge.properties {
            assert!((*orb - 2.3).abs() < f64::EPSILON);
            assert!(applying);
        } else {
            panic!("expected Aspect properties");
        }
    }

    #[test]
    fn node_serializes_to_json() {
        let node = sample_planet_node();
        let json = serde_json::to_string(&node).expect("serialize node");
        assert!(json.contains("Mars"));
        assert!(json.contains("Planet"));
    }

    #[test]
    fn edge_serializes_to_json() {
        let edge = sample_aspect_edge();
        let json = serde_json::to_string(&edge).expect("serialize edge");
        assert!(json.contains("Trine"));
        assert!(json.contains("Aspects"));
    }
}
