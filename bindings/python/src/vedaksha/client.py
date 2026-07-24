"""High-level Vedākṣha client.

A thin, typed facade over the wasm engine's JSON-RPC surface. Every astrology
computation is an MCP tool call; this class handles the JSON-RPC envelope, the
double-encoded result payload, and error translation, and adds convenience
methods for the common tools.

    >>> from vedaksha import Vedaksha
    >>> vk = Vedaksha()
    >>> chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6, longitude=77.2)
    >>> [t["name"] for t in vk.list_tools()][:3]
    ['compute_natal_chart', 'compute_dasha', 'compute_karakas']
"""
from __future__ import annotations

from typing import Any

from ._engine import Engine, default_engine
from .errors import ToolError

# NAIF ids for the bodies DE440s carries, for the SPK ephemeris tier.
NAIF_IDS: dict[str, int] = {
    "sun": 10, "moon": 301, "mercury": 1, "venus": 2, "mars": 4,
    "jupiter": 5, "saturn": 6, "uranus": 7, "neptune": 8, "pluto": 9,
    "earth_moon_barycenter": 3,
}


class Vedaksha:
    """The primary entry point for computing with the Vedākṣha engine."""

    def __init__(self, engine: Engine | None = None) -> None:
        self._engine = engine or default_engine()
        self._next_id = 0

    def _rpc(self, method: str, params: dict[str, Any] | None = None) -> Any:
        self._next_id += 1
        req: dict[str, Any] = {"jsonrpc": "2.0", "id": self._next_id, "method": method}
        if params is not None:
            req["params"] = params
        resp = self._engine.call_json(req)
        if "error" in resp:
            err = resp["error"]
            raise ToolError(err.get("code", 0), err.get("message", "unknown"), err.get("data"))
        return resp.get("result")

    # -- introspection ------------------------------------------------------

    def list_tools(self) -> list[dict[str, Any]]:
        """Return the engine's tool catalog (name, description, input schema)."""
        return self._rpc("tools/list").get("tools", [])

    def call_tool(self, name: str, **arguments: Any) -> Any:
        """Call any tool by name, returning its decoded result object.

        The engine wraps a tool's result as a JSON string inside
        ``content[0].text``; this decodes that inner payload for you.
        """
        result = self._rpc("tools/call", {"name": name, "arguments": arguments})
        content = result.get("content", [])
        if content and content[0].get("type") == "text":
            import json

            return json.loads(content[0]["text"])
        return result

    # -- convenience wrappers for the common tools --------------------------

    def natal_chart(
        self,
        julian_day: float,
        latitude: float,
        longitude: float,
        *,
        house_system: str | None = None,
        ayanamsha: str | None = None,
    ) -> dict[str, Any]:
        """Compute a natal chart (analytical ephemeris tier)."""
        args: dict[str, Any] = {
            "julian_day": julian_day,
            "latitude": latitude,
            "longitude": longitude,
        }
        if house_system is not None:
            args["house_system"] = house_system
        if ayanamsha is not None:
            args["ayanamsha"] = ayanamsha
        return self.call_tool("compute_natal_chart", **args)

    # ``compute_natal_chart`` is the one tool that takes time + location and
    # computes everything. The other 14 tools have heterogeneous inputs — many
    # take precomputed planetary longitudes rather than a time — so rather than
    # ship fragile typed wrappers that would drift from the engine's schemas,
    # call them by name with ``call_tool`` and discover their inputs with
    # ``list_tools``. Example:
    #
    #     vk.call_tool("compute_dasha", birth_jd=2451545.0, moon_longitude=120.4)
    #     vk.call_tool("compute_panchanga", jd=2451545.0, sun=280.0, moon=120.4)

    # -- sub-arcsecond ephemeris (SPK tier) ---------------------------------

    def load_ephemeris(self, kernel: bytes) -> None:
        """Load a DE440s SPK kernel for sub-arcsecond positions.

        Pass the raw bytes of ``de440s.bsp`` (~32 MB). Without this, the SPK
        methods raise; the MCP tools above always work on the analytical tier.
        """
        self._engine.load_spk(kernel)

    @property
    def ephemeris_loaded(self) -> bool:
        return self._engine.spk_loaded

    def ephemeris_range(self) -> tuple[float, float]:
        """Return the loaded kernel's ``(jd_min, jd_max)`` coverage."""
        return self._engine.spk_range()

    def state_vector(self, body: str | int, julian_day: float) -> dict[str, float]:
        """Return the ICRS state of ``body`` at ``julian_day`` (sub-arcsecond).

        ``body`` is a NAIF id or a name in :data:`NAIF_IDS`. Requires a kernel
        loaded via :meth:`load_ephemeris`.
        """
        naif = body if isinstance(body, int) else NAIF_IDS[body.lower()]
        x, y, z, vx, vy, vz = self._engine.spk_state(naif, julian_day)
        return {"x": x, "y": y, "z": z, "vx": vx, "vy": vy, "vz": vz}
