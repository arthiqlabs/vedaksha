"""MCP transport tests: stdio dispatch, HTTP auth."""
from __future__ import annotations

import json
import threading
import urllib.error
import urllib.request

import pytest

from vedaksha.mcp.server import McpServer, serve_http


@pytest.fixture(scope="module")
def server() -> McpServer:
    return McpServer()


def test_handle_raw_tools_list(server: McpServer) -> None:
    resp = json.loads(server.handle_raw(
        '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
    ))
    assert len(resp["result"]["tools"]) == 15


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

    # Correct token -> 200 with 15 tools.
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/", data=body, method="POST",
        headers={"Authorization": "Bearer sekret"},
    )
    with urllib.request.urlopen(req, timeout=5) as r:
        payload = json.loads(r.read())
    assert len(payload["result"]["tools"]) == 15
