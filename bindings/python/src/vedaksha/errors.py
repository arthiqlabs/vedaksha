"""Exception hierarchy for the Vedākṣha Python package."""
from __future__ import annotations


class VedakshaError(Exception):
    """Base class for all Vedākṣha errors."""


class EngineNotAvailable(VedakshaError):
    """The wasm runtime (wasmtime) could not be loaded."""


class EngineError(VedakshaError):
    """The engine returned a low-level (ABI) error."""


class ToolError(VedakshaError):
    """A JSON-RPC tool call returned an error response.

    Carries the JSON-RPC error ``code`` and any structured ``data`` the engine
    attached, so callers can branch on machine-readable failure modes.
    """

    def __init__(self, code: int, message: str, data: object | None = None) -> None:
        super().__init__(f"[{code}] {message}")
        self.code = code
        self.message = message
        self.data = data
