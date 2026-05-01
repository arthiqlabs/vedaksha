// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-mcp
//!
//! MCP (Model Context Protocol) server for Vedākṣha.
//!
//! Provides 12 tools for AI agents to compute charts, dashas,
//! vargas, transits, and emit graph data.

#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

pub mod server;
pub mod tools;
pub mod validation;
