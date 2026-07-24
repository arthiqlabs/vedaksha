"""Low-level wasmtime host for the Vedākṣha engine.

Loads ``vedaksha.wasm`` and wraps its C ABI (see ``engine/src/lib.rs``). This is
the only module that touches wasmtime; everything above it speaks JSON or typed
objects. One :class:`Engine` owns one wasm instance and is **not** thread-safe —
create one per thread, or guard it with a lock (:class:`vedaksha.mcp` does the
latter).
"""
from __future__ import annotations

import importlib.resources
import json
import struct
import threading
from typing import Any

from .errors import EngineError, EngineNotAvailable

_ABI_VERSION = 1

# Error codes mirrored from engine/src/lib.rs.
_ERR = {
    -1: "invalid UTF-8 in request",
    -2: "unknown NAIF body id",
    -3: "no SPK kernel loaded",
    -4: "ephemeris computation failed (out of range?)",
    -5: "malformed SPK/DAF file",
    -6: "no pending response",
    -7: "output buffer too small",
}


def _load_wasm_bytes() -> bytes:
    """Read the engine blob shipped as package data."""
    res = importlib.resources.files(__package__).joinpath("_wasm/vedaksha.wasm")
    return res.read_bytes()


class Engine:
    """A single loaded instance of the Vedākṣha wasm engine."""

    def __init__(self) -> None:
        try:
            from wasmtime import Config, Instance, Module, Store
            from wasmtime import Engine as WtEngine
        except ImportError as exc:  # pragma: no cover
            raise EngineNotAvailable(
                "wasmtime is required to run the Vedākṣha engine; "
                "install `vedaksha` (which depends on it) or `pip install wasmtime`"
            ) from exc

        cfg = Config()
        # macOS: wasmtime's default Mach-port exception handler can conflict
        # with host process supervisors (and some sandboxes SIGKILL on it).
        # POSIX signal traps are equivalent for our purposes.
        try:
            cfg.macos_use_mach_ports = False
        except Exception:  # pragma: no cover - non-macOS builds may lack it
            pass

        self._lock = threading.Lock()
        wt_engine = WtEngine(cfg)
        module = Module(wt_engine, _load_wasm_bytes())
        self._store = Store(wt_engine)
        instance = Instance(self._store, module, [])
        ex = instance.exports(self._store)

        self._mem = ex["memory"]
        self._alloc = ex["vk_alloc"]
        self._free = ex["vk_free"]
        self._mcp_request = ex["vk_mcp_request"]
        self._mcp_take = ex["vk_mcp_take"]
        self._spk_load = ex["vk_spk_load"]
        self._spk_loaded = ex["vk_spk_loaded"]
        self._spk_state = ex["vk_spk_state"]
        self._spk_range = ex["vk_spk_range"]

        abi = ex["vk_abi_version"](self._store)
        if abi != _ABI_VERSION:
            raise EngineError(
                f"engine ABI version {abi} != expected {_ABI_VERSION}; "
                "the shipped wasm blob and this Python package are mismatched"
            )

    # -- memory helpers -----------------------------------------------------

    def _write(self, data: bytes) -> int:
        ptr = self._alloc(self._store, len(data))
        self._mem.write(self._store, data, ptr)
        return ptr

    def _read(self, ptr: int, length: int) -> bytes:
        return bytes(self._mem.read(self._store, ptr, ptr + length))

    @staticmethod
    def _fail(code: int, context: str) -> EngineError:
        return EngineError(f"{context}: {_ERR.get(code, f'error {code}')}")

    # -- MCP surface --------------------------------------------------------

    def call_json(self, request: dict[str, Any]) -> dict[str, Any]:
        """Send one JSON-RPC 2.0 request object; return the parsed response."""
        req = json.dumps(request).encode()
        with self._lock:
            rptr = self._write(req)
            try:
                n = self._mcp_request(self._store, rptr, len(req))
            finally:
                self._free(self._store, rptr, len(req))
            if n < 0:
                raise self._fail(n, "mcp_request")
            obuf = self._alloc(self._store, n)
            try:
                got = self._mcp_take(self._store, obuf, n)
                if got < 0:
                    raise self._fail(got, "mcp_take")
                raw = self._read(obuf, got)
            finally:
                self._free(self._store, obuf, n)
        return json.loads(raw)

    # -- SPK surface --------------------------------------------------------

    def load_spk(self, kernel: bytes) -> None:
        """Install a DE440s SPK kernel for the sub-arcsecond ephemeris tier."""
        with self._lock:
            ptr = self._write(kernel)
            try:
                rc = self._spk_load(self._store, ptr, len(kernel))
            finally:
                self._free(self._store, ptr, len(kernel))
        if rc != 0:
            raise self._fail(rc, "load_spk")

    @property
    def spk_loaded(self) -> bool:
        with self._lock:
            return bool(self._spk_loaded(self._store))

    def spk_state(self, naif_id: int, jd: float) -> tuple[float, ...]:
        """Return ``(x, y, z, vx, vy, vz)`` in AU / AU-per-day, ICRS."""
        with self._lock:
            out = self._alloc(self._store, 48)
            try:
                rc = self._spk_state(self._store, naif_id, jd, out)
                if rc != 0:
                    raise self._fail(rc, "spk_state")
                return struct.unpack("<6d", self._read(out, 48))
            finally:
                self._free(self._store, out, 48)

    def spk_range(self) -> tuple[float, float]:
        with self._lock:
            out = self._alloc(self._store, 16)
            try:
                rc = self._spk_range(self._store, out)
                if rc != 0:
                    raise self._fail(rc, "spk_range")
                lo, hi = struct.unpack("<2d", self._read(out, 16))
                return lo, hi
            finally:
                self._free(self._store, out, 16)


_default: Engine | None = None
_default_lock = threading.Lock()


def default_engine() -> Engine:
    """Return a lazily-created process-wide engine instance."""
    global _default
    if _default is None:
        with _default_lock:
            if _default is None:
                _default = Engine()
    return _default
