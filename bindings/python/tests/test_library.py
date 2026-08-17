"""Library-surface tests: the wasm engine loads and answers on this platform."""
from __future__ import annotations

import pytest

from vedaksha import Vedaksha, __version__
from vedaksha.errors import ToolError


@pytest.fixture(scope="module")
def vk() -> Vedaksha:
    return Vedaksha()


def test_version_matches_the_package_metadata() -> None:
    """`__version__` must equal the version pip actually installed.

    This used to hardcode the literal, which meant every release bumped it by
    hand and a missed bump failed CI *after* the tag was cut. Comparing against
    the installed distribution's metadata pins the real invariant — that the
    constant and the wheel agree — and needs no edit at release time.
    """
    from importlib.metadata import version

    assert __version__ == version("vedaksha")


def test_lists_the_engine_tool_catalog(vk: Vedaksha) -> None:
    # No hardcoded count. The engine's registry is the definition of the
    # catalog, and its exactness is already pinned on the Rust side by
    # `snapshot_matches_current_tool_definitions`, which compares names,
    # descriptions AND schemas. What is worth asserting here is that this
    # binding sees a non-empty catalog carrying the tools it documents.
    names = {t["name"] for t in vk.list_tools()}
    assert names, "the engine returned an empty tool catalog"
    assert {"compute_natal_chart", "compute_panchanga"} <= names


def test_natal_chart_has_expected_shape(vk: Vedaksha) -> None:
    chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6139, longitude=77.2090)
    assert "planets" in chart
    assert "houses" in chart
    assert chart["julian_day"] == 2451545.0


def test_call_tool_generic(vk: Vedaksha) -> None:
    result = vk.call_tool(
        "compute_panchanga",
        jd=2451545.0, sun=280.0, moon=120.4,
        latitude=28.6139, longitude=77.2090,
    )
    assert isinstance(result, dict) and result


def test_tool_error_is_structured(vk: Vedaksha) -> None:
    with pytest.raises(ToolError) as exc:
        vk.call_tool("compute_natal_chart")  # missing required args
    assert exc.value.code != 0
    assert "julian_day" in exc.value.message or exc.value.code == -32602
