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
// Raising the MSRV to 1.89 put let-chains (stable in 1.88) inside the supported
// range, so clippy now offers to fold every `if cond { if let … }` into
// `if cond && let …`. A stylistic rewrite of working, tested code, so declined.
#![allow(clippy::collapsible_if)]

pub mod server;
pub mod tools;
pub mod validation;
