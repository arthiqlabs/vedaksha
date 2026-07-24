"""SPK-tier tests. Skipped when the DE440s kernel is unavailable, unless
VEDAKSHA_REQUIRE_KERNEL is set (CI sets it) — a skipped ephemeris test must not
be mistaken for a passing one.
"""
from __future__ import annotations

import os
import pathlib

import pytest

from vedaksha import Vedaksha
from vedaksha.errors import EngineError

# Look for the kernel in the repo's data/ dir or an env override.
_CANDIDATES = [
    os.environ.get("VEDAKSHA_SPK_PATH"),
    str(pathlib.Path(__file__).parents[3] / "data" / "de440s.bsp"),
]
_KERNEL = next((p for p in _CANDIDATES if p and pathlib.Path(p).exists()), None)
_REQUIRE = os.environ.get("VEDAKSHA_REQUIRE_KERNEL")

pytestmark = pytest.mark.skipif(
    _KERNEL is None and not _REQUIRE,
    reason="DE440s kernel not found (set VEDAKSHA_SPK_PATH or VEDAKSHA_REQUIRE_KERNEL)",
)


@pytest.fixture(scope="module")
def vk_spk() -> Vedaksha:
    if _KERNEL is None:
        pytest.fail("VEDAKSHA_REQUIRE_KERNEL set but no kernel found")
    vk = Vedaksha()
    with open(_KERNEL, "rb") as f:
        vk.load_ephemeris(f.read())
    return vk


def test_kernel_loads(vk_spk: Vedaksha) -> None:
    assert vk_spk.ephemeris_loaded


def test_range_covers_2000(vk_spk: Vedaksha) -> None:
    lo, hi = vk_spk.ephemeris_range()
    assert lo < 2451545.0 < hi


def test_moon_state_plausible(vk_spk: Vedaksha) -> None:
    s = vk_spk.state_vector("moon", 2461041.5)
    dist = (s["x"] ** 2 + s["y"] ** 2 + s["z"] ** 2) ** 0.5
    # Geocentric Moon ~ 0.0026 AU; this is heliocentric-frame but small.
    assert dist < 0.05


def test_unloaded_engine_raises() -> None:
    # Isolated engine (not the process-wide default, which other tests populate).
    from vedaksha._engine import Engine

    vk = Vedaksha(engine=Engine())
    assert not vk.ephemeris_loaded
    with pytest.raises(EngineError):
        vk.state_vector("moon", 2451545.0)
