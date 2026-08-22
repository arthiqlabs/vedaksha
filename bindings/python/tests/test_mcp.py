# SPDX-License-Identifier: BUSL-1.1
"""MCP transport tests: stdio dispatch, HTTP auth."""
from __future__ import annotations

import json
import threading
import urllib.error
import urllib.request

import pytest

from vedaksha import Vedaksha
from vedaksha.mcp.server import McpServer, serve_http


@pytest.fixture(scope="module")
def server() -> McpServer:
    return McpServer()


def test_handle_raw_tools_list_matches_the_library_catalog(server: McpServer) -> None:
    # The invariant, not a count: the raw JSON-RPC surface must offer exactly
    # the set `Vedaksha.list_tools()` offers. A count cannot see a surface that
    # silently drops one tool while gaining another; a set equality can, and it
    # never needs editing when the engine gains a tool.
    resp = json.loads(server.handle_raw(
        '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
    ))
    names = {t["name"] for t in resp["result"]["tools"]}
    assert names, "the engine returned an empty tool catalog"
    assert names == {t["name"] for t in Vedaksha().list_tools()}


def test_handle_raw_bad_json_is_parse_error(server: McpServer) -> None:
    resp = json.loads(server.handle_raw("not json"))
    assert resp["error"]["code"] == -32700


def test_http_requires_token_by_default() -> None:
    with pytest.raises(ValueError):
        serve_http(require_auth=True, token=None)


def _free_port() -> int:
    import socket

    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def test_http_auth_enforced() -> None:
    port = _free_port()
    t = threading.Thread(
        target=serve_http,
        kwargs={"host": "127.0.0.1", "port": port, "token": "sekret"},
        daemon=True,
    )
    t.start()

    import time

    time.sleep(0.4)
    body = b'{"jsonrpc":"2.0","id":1,"method":"tools/list"}'

    # No token -> 401.
    req = urllib.request.Request(f"http://127.0.0.1:{port}/", data=body, method="POST")
    with pytest.raises(urllib.error.HTTPError) as exc:
        urllib.request.urlopen(req, timeout=5)
    assert exc.value.code == 401

    # Correct token -> 200 with the same tool set the library exposes.
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/", data=body, method="POST",
        headers={"Authorization": "Bearer sekret"},
    )
    with urllib.request.urlopen(req, timeout=5) as r:
        payload = json.loads(r.read())
    names = {t["name"] for t in payload["result"]["tools"]}
    assert names == {t["name"] for t in Vedaksha().list_tools()}
