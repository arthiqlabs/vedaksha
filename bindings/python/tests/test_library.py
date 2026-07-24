"""Library-surface tests: the wasm engine loads and answers on this platform."""
from __future__ import annotations

import pytest

from vedaksha import Vedaksha, __version__
from vedaksha.errors import ToolError


@pytest.fixture(scope="module")
def vk() -> Vedaksha:
    return Vedaksha()


def test_version() -> None:
    assert __version__ == "4.0.0"


def test_lists_fifteen_tools(vk: Vedaksha) -> None:
    tools = vk.list_tools()
    assert len(tools) == 15
    assert "compute_natal_chart" in {t["name"] for t in tools}


def test_natal_chart_has_expected_shape(vk: Vedaksha) -> None:
    chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6139, longitude=77.2090)
    assert "planets" in chart
    assert "houses" in chart
    assert chart["julian_day"] == 2451545.0


def test_call_tool_generic(vk: Vedaksha) -> None:
    result = vk.call_tool("compute_panchanga", jd=2451545.0, sun=280.0, moon=120.4)
    assert isinstance(result, dict) and result


def test_tool_error_is_structured(vk: Vedaksha) -> None:
    with pytest.raises(ToolError) as exc:
        vk.call_tool("compute_natal_chart")  # missing required args
    assert exc.value.code != 0
    assert "julian_day" in exc.value.message or exc.value.code == -32602
