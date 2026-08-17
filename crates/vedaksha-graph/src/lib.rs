// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
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

//!
//! **Not `no_std`.** This crate declared `#![cfg_attr(not(feature = "std"),
//! no_std)]` until v5.0.0 and could not honour it: `--no-default-features`
//! failed to compile, so the attribute was a claim no build ever checked.
//! `vedaksha-math` is the one crate in this workspace that is genuinely
//! `no_std`, and CI proves it. The `std` feature and the `alloc` shims are
//! kept — they gate real optional code and are the scaffolding a future
//! `no_std` effort would build on — but the claim is gone until something
//! verifies it.

#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod chart_graph;
pub mod classification;
pub mod ids;
pub mod ontology;

#[cfg(feature = "emitters")]
pub mod emitters;

#[cfg(feature = "from-chart")]
pub mod from_chart;

// Re-exports
pub use chart_graph::ChartGraph;
pub use classification::DataClassification;
pub use ids::NodeId;
pub use ontology::{Edge, EdgeProperties, EdgeType, Node, NodeProperties, NodeType};

#[cfg(feature = "from-chart")]
pub use from_chart::chart_to_graph;
