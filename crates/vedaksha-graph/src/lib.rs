// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-graph
//!
//! Graph data model and ontology for astrological charts.
//!
//! Defines the property graph structure (nodes, edges, IDs) that represents
//! a computed chart. This graph is consumed by vedaksha-emit for output
//! to Neo4j, `SurrealDB`, JSON-LD, etc.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod chart_graph;
pub mod classification;
pub mod ids;
pub mod ontology;

// Re-exports
pub use chart_graph::ChartGraph;
pub use classification::DataClassification;
pub use ids::NodeId;
pub use ontology::{Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};
