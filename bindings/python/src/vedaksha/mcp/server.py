"""MCP server transports over the wasm engine.

The engine already speaks JSON-RPC 2.0 (it *is* an MCP server compiled to wasm),
so these transports are thin: read a request, hand the raw bytes to
``Engine.call_json``, write the response. The only real logic here is the HTTP
auth layer the Rust engine lacks.
"""
from __future__ import annotations

import hmac
import json
import sys
from typing import Any

from .._engine import Engine, default_engine


class McpServer:
    """Wraps an :class:`~vedaksha._engine.Engine` for one-request dispatch."""

    def __init__(self, engine: Engine | None = None) -> None:
        self._engine = engine or default_engine()

    def handle(self, request: dict[str, Any]) -> dict[str, Any]:
        return self._engine.call_json(request)

    def handle_raw(self, raw: str) -> str:
        """Dispatch a raw JSON-RPC string, returning a raw JSON string.

        Malformed JSON yields a JSON-RPC parse-error response rather than an
        exception, matching the protocol's requirement that the transport
        always return a well-formed envelope.
        """
        try:
            request = json.loads(raw)
        except json.JSONDecodeError:
            return json.dumps(
                {"jsonrpc": "2.0", "id": None,
                 "error": {"code": -32700, "message": "Parse error"}}
            )
        return json.dumps(self.handle(request))


def serve_stdio(server: McpServer | None = None) -> None:
    """Serve MCP over stdin/stdout, one JSON message per line."""
    server = server or McpServer()
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        sys.stdout.write(server.handle_raw(line) + "\n")
        sys.stdout.flush()


def serve_http(
    server: McpServer | None = None,
    *,
    host: str = "127.0.0.1",
    port: int = 3100,
    token: str | None = None,
    require_auth: bool = True,
) -> None:
    """Serve MCP over HTTP (JSON-RPC 2.0 on POST /).

    Auth is on by default: requests must carry ``Authorization: Bearer <token>``.
    Set ``require_auth=False`` only behind a trusted network boundary.
    """
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    server = server or McpServer()

    if require_auth and not token:
        raise ValueError(
            "HTTP auth is required but no token was given; set VEDAKSHA_MCP_TOKEN "
            "or pass --insecure-no-auth to disable (only behind a trusted boundary)"
        )

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def _send(self, code: int, body: bytes, ctype: str = "application/json") -> None:
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def _authorized(self) -> bool:
            if not require_auth:
                return True
            header = self.headers.get("Authorization", "")
            prefix = "Bearer "
            if not header.startswith(prefix):
                return False
            # Constant-time compare to avoid leaking the token via timing.
            return hmac.compare_digest(header[len(prefix):], token or "")

        def do_POST(self) -> None:
            if not self._authorized():
                self._send(401, b'{"error":"unauthorized"}')
                return
            length = int(self.headers.get("Content-Length", 0))
            raw = self.rfile.read(length).decode("utf-8", "replace")
            resp = server.handle_raw(raw).encode()
            self._send(200, resp)

        def do_GET(self) -> None:
            self._send(200, b'{"status":"ok","service":"vedaksha-mcp"}')

        def log_message(self, *args: Any) -> None:  # silence default logging
            pass

    httpd = ThreadingHTTPServer((host, port), Handler)
    auth_state = "auth required" if require_auth else "AUTH DISABLED"
    print(f"vedaksha-mcp HTTP on http://{host}:{port}  ({auth_state})", file=sys.stderr)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        httpd.shutdown()
