"""Self-hostable MCP server for the Vedākṣha engine.

Two transports over the same engine:

* **stdio** — one JSON-RPC message per line on stdin/stdout, for Claude Desktop,
  Cursor, VS Code and other MCP clients.
* **HTTP** — MCP Streamable HTTP / JSON-RPC 2.0, with **bearer-token auth on by
  default** and bound to ``127.0.0.1``. This is a deliberate divergence from the
  Rust engine's HTTP server, which ships with no auth and binds ``0.0.0.0`` —
  this package exists to be self-hosted, so it is safe by default.

Run with ``python -m vedaksha.mcp`` (stdio) or ``python -m vedaksha.mcp --http``.
"""
from __future__ import annotations

from .server import McpServer, serve_http, serve_stdio

__all__ = ["McpServer", "serve_http", "serve_stdio"]
