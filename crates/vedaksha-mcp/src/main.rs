// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedākṣha MCP server — stdio transport.
//!
//! Reads JSON-RPC 2.0 requests from stdin (one per line) and writes
//! responses to stdout. This is the standard MCP stdio transport
//! compatible with Claude Desktop, Claude Code, VS Code, and Cursor.

use std::io::{self, BufRead, Write};

fn main() {
    let server = vedaksha_mcp::server::McpServer::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = server.handle_request(trimmed);
        let _ = writeln!(stdout, "{response}");
        let _ = stdout.flush();
    }
}
