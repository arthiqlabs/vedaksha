// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-mcp
//!
//! MCP (Model Context Protocol) server for Vedākṣha.
//!
//! Provides tools for AI agents to compute charts, dashas, vargas, transits,
//! synastry and composites, and to emit graph data. The catalog is
//! [`tools::tool_definitions`]; agents discover it with one `tools/list` call,
//! so no count is repeated in prose that would have to be kept in step.

#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

pub mod server;
pub mod tools;
pub mod validation;
