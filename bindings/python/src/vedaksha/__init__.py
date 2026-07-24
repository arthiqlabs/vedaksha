"""Vedākṣha — clean-room Vedic astronomy & Jyotish engine, hosted from Python.

This package runs the real Rust Vedākṣha engine (compiled to WebAssembly) via
``wasmtime``. It is **not** a reimplementation: every number it returns is the
Rust engine's own, bit-for-bit. That is what makes one architecture-independent
wheel possible — no Rust toolchain, no per-platform build, any OS and CPU
``wasmtime`` supports.

Quick start::

    from vedaksha import Vedaksha
    vk = Vedaksha()
    chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6, longitude=77.2)

See :class:`Vedaksha` for the full surface, ``vedaksha.mcp`` for a self-hostable
MCP server, and the ``vedaksha`` CLI.
"""
from __future__ import annotations

from .client import NAIF_IDS, Vedaksha
from .errors import (
    EngineError,
    EngineNotAvailable,
    ToolError,
    VedakshaError,
)

__version__ = "3.3.0"

__all__ = [
    "NAIF_IDS",
    "EngineError",
    "EngineNotAvailable",
    "ToolError",
    "Vedaksha",
    "VedakshaError",
    "__version__",
]
