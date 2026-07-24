"""FastAPI REST projection of the engine's tool surface.

Generated from the engine's own tool list, not hand-authored: each tool becomes
``POST /v1/<tool_name>`` taking the tool's arguments as the JSON body and
returning the decoded result. Because it is generated from the same schemas that
drive MCP, it has no independent surface to drift — if MCP is correct, REST is.

Easiest surface for a non-Python client (e.g. a PHP team): ordinary JSON POST,
no JSON-RPC envelope, no double-decoding.

Requires the ``rest`` extra::

    pip install "vedaksha[rest]"
    python -m vedaksha.rest
"""
from __future__ import annotations

import os
from typing import Any, Optional

try:
    from fastapi import Body, FastAPI, Header, HTTPException
except ImportError as exc:  # pragma: no cover
    raise ImportError(
        "the REST server needs the `rest` extra: pip install 'vedaksha[rest]'"
    ) from exc

from .client import Vedaksha
from .errors import ToolError


def create_app(*, token: Optional[str] = None) -> "FastAPI":
    """Build the FastAPI app. If ``token`` is set, all tool routes require it."""
    app = FastAPI(
        title="Vedākṣha",
        version="3.3.0",
        description="REST projection of the Vedākṣha engine's 15 tools.",
    )
    vk = Vedaksha()
    tools = {t["name"]: t for t in vk.list_tools()}

    def _check_auth(authorization: Optional[str]) -> None:
        if token is None:
            return
        expected = f"Bearer {token}"
        if authorization != expected:
            raise HTTPException(status_code=401, detail="unauthorized")

    @app.get("/v1/tools")
    def list_tools(authorization: Optional[str] = Header(default=None)) -> list[dict[str, Any]]:
        _check_auth(authorization)
        return list(tools.values())

    def _make_route(tool_name: str):
        def route(
            arguments: dict[str, Any] = Body(default_factory=dict),
            authorization: Optional[str] = Header(default=None),
        ) -> Any:
            _check_auth(authorization)
            try:
                return vk.call_tool(tool_name, **arguments)
            except ToolError as exc:
                raise HTTPException(status_code=422, detail=exc.message) from exc

        return route

    for name, spec in tools.items():
        app.post(f"/v1/{name}", summary=spec.get("description", name))(_make_route(name))

    return app


def main(argv: list[str] | None = None) -> int:
    import argparse

    import uvicorn

    parser = argparse.ArgumentParser(prog="python -m vedaksha.rest")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--insecure-no-auth", action="store_true")
    args = parser.parse_args(argv)

    token = os.environ.get("VEDAKSHA_MCP_TOKEN")
    if not args.insecure_no_auth and not token:
        print("error: set VEDAKSHA_MCP_TOKEN or pass --insecure-no-auth")
        return 2

    app = create_app(token=None if args.insecure_no_auth else token)
    uvicorn.run(app, host=args.host, port=args.port)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
