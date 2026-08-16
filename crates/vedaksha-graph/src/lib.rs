// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-graph
//!
//! Graph data model and ontology for astrological charts.
//!
//! Defines the property graph structure (nodes, edges, IDs) that represents
//! a computed chart. Enable the `emitters` feature to also pull in the
//! Neo4j Cypher, `SurrealDB`, JSON-LD, JSON, and RAG embedding-text emitters
//! (formerly the standalone `vedaksha-emit` crate in v2.x).

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Raising the MSRV to 1.89 put let-chains (stable in 1.88) inside the supported
// range, so clippy now offers to fold every `if cond { if let … }` into
// `if cond && let …`. That is a stylistic rewrite of working, tested code, not a
// defect — so it is declined here rather than churned through the emitters.
#![allow(clippy::collapsible_if)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod chart_graph;
pub mod classification;
pub mod ids;
pub mod ontology;

#[cfg(feature = "emitters")]
pub mod emitters;

// Re-exports
pub use chart_graph::ChartGraph;
pub use classification::DataClassification;
pub use ids::NodeId;
pub use ontology::{Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};
