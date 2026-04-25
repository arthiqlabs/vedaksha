// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedākṣha MCP server — stdio and HTTP transports.
//!
//! ## Stdio (default)
//! Reads JSON-RPC 2.0 requests from stdin (one per line), writes responses
//! to stdout. Compatible with Claude Desktop, Claude Code, VS Code, Cursor.
//!
//! ## HTTP (Streamable HTTP transport)
//! Listens on a configurable port and accepts POST requests with JSON-RPC
//! bodies. Returns JSON responses. Compatible with Smithery, ChatGPT Actions,
//! and any MCP client supporting the Streamable HTTP transport.
//!
//! Usage:
//!   vedaksha-mcp                # stdio mode (default)
//!   vedaksha-mcp --http         # HTTP mode on port 3100
//!   vedaksha-mcp --http --port 8080  # HTTP mode on custom port

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--http") {
        #[cfg(feature = "http")]
        {
            let port = parse_port(&args);
            run_http(port);
        }
        #[cfg(not(feature = "http"))]
        {
            eprintln!("HTTP transport not available. Build with --features http");
            std::process::exit(1);
        }
    } else {
        run_stdio();
    }
}

fn run_stdio() {
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

#[cfg(feature = "http")]
fn run_http(port: u16) {
    use tiny_http::{Method, Response, Server};

    let addr = format!("0.0.0.0:{port}");
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        }
    };

    eprintln!("Vedākṣha MCP server listening on http://{addr}/mcp");

    let mcp = vedaksha_mcp::server::McpServer::new();

    for mut request in server.incoming_requests() {
        let path = request.url().to_string();

        // Health check
        if path == "/health" {
            let _ = request.respond(
                Response::from_string(r#"{"status":"ok"}"#).with_header(content_type_json()),
            );
            continue;
        }

        // Only accept POST to /mcp (or root /)
        if path != "/mcp" && path != "/" {
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            continue;
        }

        if *request.method() != Method::Post {
            // GET on the MCP endpoint: return server info
            if *request.method() == Method::Get {
                let info = serde_json::json!({
                    "name": "vedaksha-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                    "transport": "streamable-http",
                    "endpoint": "/mcp",
                });
                let _ = request.respond(
                    Response::from_string(info.to_string())
                        .with_header(content_type_json())
                        .with_header(cors_header()),
                );
                continue;
            }
            let _ =
                request.respond(Response::from_string("Method Not Allowed").with_status_code(405));
            continue;
        }

        // Read the POST body
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            let _ = request.respond(
                Response::from_string(r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#)
                    .with_status_code(400)
                    .with_header(content_type_json()),
            );
            continue;
        }

        let response_json = mcp.handle_request(&body);

        let _ = request.respond(
            Response::from_string(response_json)
                .with_header(content_type_json())
                .with_header(cors_header()),
        );
    }
}

#[cfg(feature = "http")]
fn content_type_json() -> tiny_http::Header {
    "Content-Type: application/json"
        .parse::<tiny_http::Header>()
        .unwrap()
}

#[cfg(feature = "http")]
fn cors_header() -> tiny_http::Header {
    "Access-Control-Allow-Origin: *"
        .parse::<tiny_http::Header>()
        .unwrap()
}

fn parse_port(args: &[String]) -> u16 {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" {
            if let Some(val) = args.get(i + 1) {
                return val.parse().unwrap_or(3100);
            }
        }
    }
    3100
}
