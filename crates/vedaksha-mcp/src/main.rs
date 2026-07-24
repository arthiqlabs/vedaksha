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
//! ### HTTP authentication (on by default)
//! In HTTP mode every `POST` must carry `Authorization: Bearer <token>`, where
//! the token is read from `VEDAKSHA_MCP_TOKEN`. The server refuses to start
//! without a token unless `--insecure-no-auth` is passed. `/health` and the
//! informational `GET` are always open (no computation, no data). This is a
//! deliberate break from the previous open-by-default server: the endpoint
//! computes on request and must not be exposed unauthenticated.
//!
//! Usage:
//!   vedaksha-mcp                       # stdio mode (default)
//!   VEDAKSHA_MCP_TOKEN=… vedaksha-mcp --http          # HTTP on :3100, auth on
//!   VEDAKSHA_MCP_TOKEN=… vedaksha-mcp --http --port 8080
//!   vedaksha-mcp --http --insecure-no-auth            # no auth (trusted net only)
//!   vedaksha-mcp --http --host 127.0.0.1              # override bind host
//!
//! Environment:
//!   VEDAKSHA_MCP_TOKEN        bearer token required of every POST
//!   VEDAKSHA_MCP_HOST         bind host (default 0.0.0.0; --host overrides)
//!   VEDAKSHA_MCP_CORS_ORIGIN  Access-Control-Allow-Origin value (default *)

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--http") {
        #[cfg(feature = "http")]
        {
            let port = parse_port(&args);
            run_http(port, &args);
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

/// Resolved HTTP auth policy: `Some(token)` requires that bearer token on every
/// POST; `None` means auth is explicitly disabled via `--insecure-no-auth`.
#[cfg(feature = "http")]
struct AuthConfig {
    token: Option<String>,
}

#[cfg(feature = "http")]
impl AuthConfig {
    /// Build the policy from flags + env, exiting with a clear message if HTTP
    /// mode is requested without a token and without `--insecure-no-auth`.
    fn resolve(args: &[String]) -> Self {
        let insecure = args.iter().any(|a| a == "--insecure-no-auth");
        let token = std::env::var("VEDAKSHA_MCP_TOKEN")
            .ok()
            .filter(|t| !t.is_empty());

        match (insecure, token) {
            (true, _) => {
                eprintln!(
                    "WARNING: --insecure-no-auth set; the MCP HTTP endpoint is UNAUTHENTICATED. \
                     Only run this behind a trusted network boundary."
                );
                Self { token: None }
            }
            (false, Some(t)) => Self { token: Some(t) },
            (false, None) => {
                eprintln!(
                    "error: HTTP mode requires authentication. Set VEDAKSHA_MCP_TOKEN to a \
                     bearer token, or pass --insecure-no-auth to run without auth (only behind \
                     a trusted boundary)."
                );
                std::process::exit(1);
            }
        }
    }

    /// True if `header_value` (the raw `Authorization` header) presents the
    /// correct bearer token. Always true when auth is disabled.
    fn authorized(&self, header_value: Option<&str>) -> bool {
        let Some(expected) = self.token.as_deref() else {
            return true; // auth disabled
        };
        let Some(v) = header_value else {
            return false;
        };
        let Some(presented) = v.strip_prefix("Bearer ") else {
            return false;
        };
        constant_time_eq(presented.as_bytes(), expected.as_bytes())
    }
}

/// Length-then-content constant-time comparison, to avoid leaking the token
/// via response timing. (The length check is standard and acceptable.)
#[cfg(feature = "http")]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(feature = "http")]
fn run_http(port: u16, args: &[String]) {
    use tiny_http::{Method, Response, Server};

    let auth = AuthConfig::resolve(args);
    let host = parse_host(args);
    let addr = format!("{host}:{port}");
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        }
    };

    let auth_state = if auth.token.is_some() {
        "auth required"
    } else {
        "AUTH DISABLED"
    };
    eprintln!("Vedākṣha MCP server listening on http://{addr}/mcp  ({auth_state})");

    let mcp = vedaksha_mcp::server::McpServer::new();

    for mut request in server.incoming_requests() {
        let path = request.url().to_string();

        // Health check — always open (liveness only, no computation).
        if path == "/health" {
            let _ = request.respond(
                Response::from_string(r#"{"status":"ok"}"#).with_header(content_type_json()),
            );
            continue;
        }

        // Only accept /mcp (or root /)
        if path != "/mcp" && path != "/" {
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            continue;
        }

        // CORS preflight: browsers preflight because we require the
        // Authorization header. Answer with the allowed origin + headers.
        if *request.method() == Method::Options {
            let _ = request.respond(
                Response::from_string("")
                    .with_status_code(204)
                    .with_header(cors_origin_header())
                    .with_header(cors_preflight_headers()),
            );
            continue;
        }

        if *request.method() != Method::Post {
            // GET on the MCP endpoint: informational only, always open.
            if *request.method() == Method::Get {
                let info = serde_json::json!({
                    "name": "vedaksha-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                    "transport": "streamable-http",
                    "endpoint": "/mcp",
                    "auth": if auth.token.is_some() { "bearer" } else { "none" },
                });
                let _ = request.respond(
                    Response::from_string(info.to_string())
                        .with_header(content_type_json())
                        .with_header(cors_origin_header()),
                );
                continue;
            }
            let _ =
                request.respond(Response::from_string("Method Not Allowed").with_status_code(405));
            continue;
        }

        // Authenticate the POST (the only path that computes).
        let authz = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Authorization"))
            .map(|h| h.value.as_str().to_string());
        if !auth.authorized(authz.as_deref()) {
            let _ = request.respond(
                Response::from_string(r#"{"error":"unauthorized"}"#)
                    .with_status_code(401)
                    .with_header(content_type_json())
                    .with_header(cors_origin_header()),
            );
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
                .with_header(cors_origin_header()),
        );
    }
}

#[cfg(feature = "http")]
fn content_type_json() -> tiny_http::Header {
    "Content-Type: application/json"
        .parse::<tiny_http::Header>()
        .unwrap()
}

/// The `Access-Control-Allow-Origin` value, from `VEDAKSHA_MCP_CORS_ORIGIN`
/// (default `*`). With bearer auth mandatory, a permissive origin still can't
/// be used without the token; operators can lock it down here.
#[cfg(feature = "http")]
fn cors_origin_header() -> tiny_http::Header {
    let origin = std::env::var("VEDAKSHA_MCP_CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
    format!("Access-Control-Allow-Origin: {origin}")
        .parse::<tiny_http::Header>()
        .unwrap_or_else(|_| {
            "Access-Control-Allow-Origin: *"
                .parse::<tiny_http::Header>()
                .unwrap()
        })
}

#[cfg(feature = "http")]
fn cors_preflight_headers() -> tiny_http::Header {
    "Access-Control-Allow-Headers: Authorization, Content-Type"
        .parse::<tiny_http::Header>()
        .unwrap()
}

#[cfg(feature = "http")]
fn parse_host(args: &[String]) -> String {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--host" {
            if let Some(val) = args.get(i + 1) {
                return val.clone();
            }
        }
    }
    std::env::var("VEDAKSHA_MCP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string())
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

#[cfg(all(test, feature = "http"))]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_matches_semantics() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"toshi")); // same length, differ
        assert!(!constant_time_eq(b"token", b"tok")); // length differs
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn disabled_auth_allows_anything() {
        let auth = AuthConfig { token: None };
        assert!(auth.authorized(None));
        assert!(auth.authorized(Some("Bearer whatever")));
        assert!(auth.authorized(Some("garbage")));
    }

    #[test]
    fn enabled_auth_requires_exact_bearer_token() {
        let auth = AuthConfig {
            token: Some("s3cr3t".to_string()),
        };
        assert!(auth.authorized(Some("Bearer s3cr3t")));
        assert!(!auth.authorized(Some("Bearer wrong")));
        assert!(!auth.authorized(Some("s3cr3t"))); // missing "Bearer " prefix
        assert!(!auth.authorized(Some("bearer s3cr3t"))); // case-sensitive scheme
        assert!(!auth.authorized(None));
        assert!(!auth.authorized(Some("")));
    }
}
