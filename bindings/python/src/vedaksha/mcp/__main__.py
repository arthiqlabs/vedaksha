"""``python -m vedaksha.mcp`` entry point."""
from __future__ import annotations

import argparse
import os
import sys

from .server import McpServer, serve_http, serve_stdio


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m vedaksha.mcp",
        description="Self-hosted Vedākṣha MCP server (15 tools).",
    )
    parser.add_argument("--http", action="store_true", help="serve over HTTP instead of stdio")
    parser.add_argument("--host", default="127.0.0.1", help="HTTP bind host (default: localhost)")
    parser.add_argument("--port", type=int, default=3100, help="HTTP port (default: 3100)")
    parser.add_argument(
        "--insecure-no-auth",
        action="store_true",
        help="disable HTTP bearer-token auth (only behind a trusted boundary)",
    )
    args = parser.parse_args(argv)

    server = McpServer()

    if not args.http:
        serve_stdio(server)
        return 0

    token = os.environ.get("VEDAKSHA_MCP_TOKEN")
    require_auth = not args.insecure_no_auth
    if require_auth and not token:
        print(
            "error: set VEDAKSHA_MCP_TOKEN, or pass --insecure-no-auth to run "
            "without auth (only behind a trusted boundary)",
            file=sys.stderr,
        )
        return 2
    if not require_auth and args.host != "127.0.0.1":
        print(
            "warning: serving with NO AUTH on a non-localhost interface "
            f"({args.host}); anyone who can reach this port can use the engine",
            file=sys.stderr,
        )

    serve_http(
        server,
        host=args.host,
        port=args.port,
        token=token,
        require_auth=require_auth,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
