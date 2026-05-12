// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Graph emitters that convert `ChartGraph` into various output formats.
//!
//! - **Neo4j Cypher** — CREATE/MERGE statements for Neo4j graph database
//! - **`SurrealDB`** — `SurrealQL` statements for `SurrealDB`
//! - **JSON-LD** — linked-data serialization with astronomical ontology
//! - **JSON** — plain JSON graph (canonical MCP format)
//! - **Embedding text** — structured natural-language output for RAG pipelines
//!
//! This module was the `vedaksha-emit` crate in v2.x; in v3.0.0 it folded
//! into `vedaksha-graph` behind the `emitters` feature.

pub mod cypher;
pub mod embedding_text;
pub mod json_graph;
pub mod jsonld;
pub mod surreal;

use crate::ChartGraph;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Trait for emitting a `ChartGraph` to a specific output format.
pub trait GraphEmitter {
    /// The output type produced by this emitter.
    type Output;

    /// Emit the entire chart graph to the output format.
    ///
    /// # Errors
    ///
    /// Returns an error string if emission fails.
    fn emit(&self, graph: &ChartGraph) -> Result<Self::Output, String>;
}
