// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-mcp
//!
//! MCP (Model Context Protocol) server for Vedaksha.
//!
//! Provides tools for AI agents to compute charts, dashas, vargas, transits,
//! synastry and composites, and to emit graph data. The catalog is
//! [`tools::tool_definitions`]; agents discover it with one `tools/list` call,
//! so no count is repeated in prose that would have to be kept in step.
#![deny(unsafe_code)]

pub mod server;
pub mod tools;
pub mod validation;
