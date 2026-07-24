"""Regenerate the native-parity fixture from the in-repo Rust engine.

Feeds each JSON-RPC request through the native `vedaksha-mcp` binary in stdio
mode (which calls the same `McpServer::handle_request` the wasm build calls) and
records its exact output. The parity test then asserts the wasm build reproduces
these byte-for-byte. Run from this directory with the repo's Rust toolchain:

    python generate_fixture.py
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[3]  # repo root (…/vedaksha)

MCP_CASES = [
    ("natal_j2000", {
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "compute_natal_chart", "arguments": {
            "julian_day": 2451545.0, "latitude": 28.6139, "longitude": 77.2090}}}),
    ("natal_1985", {
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "compute_natal_chart", "arguments": {
            "julian_day": 2446270.104166667, "latitude": 40.7128, "longitude": -74.0060}}}),
    ("panchanga", {
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "compute_panchanga", "arguments": {
            "jd": 2451545.0, "sun": 280.0, "moon": 120.4}}}),
    ("tools_list", {"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
]


def run_native(request: dict) -> str:
    """One JSON-RPC request → response, via `vedaksha-mcp` stdio mode."""
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--release", "--bin", "vedaksha-mcp"],
        input=(json.dumps(request) + "\n").encode(),
        capture_output=True,
        cwd=str(ROOT),
        check=False,
    )
    if proc.returncode != 0:
        sys.exit(f"vedaksha-mcp failed: {proc.stderr.decode()[:500]}")
    return proc.stdout.decode().strip()


def main() -> int:
    fixture = {}
    for label, req in MCP_CASES:
        fixture[label] = {"request": req, "response": run_native(req)}
        print(f"  {label}: {len(fixture[label]['response'])} bytes")

    out = HERE / "native_fixture.json"
    out.write_text(json.dumps(fixture, indent=1))
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
