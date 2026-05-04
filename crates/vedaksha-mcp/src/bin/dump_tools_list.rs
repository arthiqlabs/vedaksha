// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Dumps the canonical MCP `tools/list` response body to stdout as
//! pretty-printed JSON. Used to regenerate the committed snapshot at
//! `tools/mcp-tools.json` whenever the MCP tool surface changes.
//!
//! The snapshot doubles as the source of truth for the introspection-only
//! MCP endpoint hosted at `vedaksha.net/api/mcp` — that endpoint reads it
//! at request time so the live `tools/list` response can never drift from
//! the Rust `tool_definitions()` registry. A drift-guard test in this crate
//! fails CI if the snapshot is out of date.
//!
//! Regenerate with:
//!   cargo run -p vedaksha-mcp --bin dump-tools-list > tools/mcp-tools.json

use vedaksha_mcp::tools::tool_definitions;

fn main() {
    let tools: Vec<serde_json::Value> = tool_definitions()
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    let body = serde_json::json!({
        "engineVersion": env!("CARGO_PKG_VERSION"),
        "tools": tools,
    });

    println!("{}", serde_json::to_string_pretty(&body).unwrap());
}
