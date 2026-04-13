// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-emit
//!
//! Graph emitters that convert `ChartGraph` into various output formats.
//!
//! - **Neo4j Cypher** — CREATE/MERGE statements for Neo4j graph database
//! - **`SurrealDB`** — `SurrealQL` statements for `SurrealDB`
//! - **JSON-LD** — linked-data serialization with astronomical ontology
//! - **JSON** — plain JSON graph (canonical MCP format)
//! - **Embedding text** — structured natural-language output for RAG pipelines

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

pub mod cypher;
pub mod embedding_text;
pub mod json_graph;
pub mod jsonld;
pub mod surreal;

use vedaksha_graph::ChartGraph;

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
