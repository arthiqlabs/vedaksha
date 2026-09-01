# SPDX-License-Identifier: BUSL-1.1
"""Tests for the TT-addressed analytical surface (``Vedaksha.analytical_position``),
the Python-side counterpart of ``vk_analytical_position_tt`` / the Rust
``ecliptic_position_tt`` entry point.

Before this file, this surface had zero test coverage anywhere in the repo:
every existing check on it (see ``coordinates.rs``) is relative to the UT1
path, so a bug shared by both cancels out. This module adds an absolute check
against an external oracle, and confirms the documented Pluto behaviour.
"""
from __future__ import annotations

import pytest

from vedaksha import Vedaksha
from vedaksha.errors import EngineError

# JPL Horizons DE441 apparent geocentric ecliptic longitude of the Moon at
# JD 2451545.0 **UT** (the same anchor cited in
# `vedaksha-ephem-core/src/coordinates.rs`'s `moon_longitude_at_j2000_matches_jpl_horizons`
# and pinned by `scripts/generate_horizons_oracle.py`). That anchor's time
# scale is UT, not TT (see the adversarial-review fix to that comment), so it
# cannot be fed directly into a TT-addressed function. Instead this reaches
# the same physical instant the way the Rust-side
# `ecliptic_position_tt_matches_jpl_horizons_at_j2000` test does: converting
# JD 2451545.0 UT1 to TT via this engine's own Delta T table
# (`delta_t::ut1_to_tt`) and calling the TT entry point at that TT instant.
# `ut1_to_tt(2451545.0)` = 2451545.0007385127 (Delta T = 63.8075 s at J2000
# in this engine's table). Delta T is not exposed through the Python binding
# (only the TT-addressed entry point is), so this constant is transcribed
# from the Rust-side measurement rather than recomputed here.
_J2000_TT = 2_451_545.0007385127
_HORIZONS_MOON_LONGITUDE_DEG = 223.3238
# Same tolerance as the Rust-side anchor test: accounts for ELP/MPP02 vs.
# DE441 lunar theory residual (~17 arcsec max for modern dates) plus rounding.
_TOLERANCE_DEG = 0.006


def test_analytical_position_moon_matches_jpl_horizons_at_j2000() -> None:
    vk = Vedaksha()
    pos = vk.analytical_position("moon", _J2000_TT)

    diff = abs(pos["longitude"] - _HORIZONS_MOON_LONGITUDE_DEG)
    if diff > 180.0:
        diff = 360.0 - diff
    assert diff < _TOLERANCE_DEG, (
        f"analytical_position('moon', {_J2000_TT}) longitude "
        f"{pos['longitude']:.4f} deg should be within {_TOLERANCE_DEG} deg of "
        f"the JPL Horizons DE441 anchor {_HORIZONS_MOON_LONGITUDE_DEG} deg "
        f"(diff={diff:.4f} deg)"
    )
    # Sanity: the result really is degrees/AU, not radians or some other unit.
    assert 0.0 <= pos["longitude"] < 360.0
    assert -90.0 <= pos["latitude"] <= 90.0
    assert 0.0 < pos["distance"] < 1.0  # geocentric Moon, AU


def test_analytical_position_pluto_raises() -> None:
    # The analytical tier does not model Pluto (VSOP87/ELP have no Pluto
    # series), unlike the SPK tier, which does (NAIF_IDS lists it for both).
    # See the adversarial-review fix mapping ComputeError::BodyNotAvailable
    # to ERR_UNKNOWN_BODY ("unknown NAIF body id") in engine/src/lib.rs,
    # rather than the generic ERR_COMPUTE ("ephemeris computation failed (out
    # of range?)") that would misdirect a caller into hunting a date bug.
    vk = Vedaksha()
    with pytest.raises(EngineError, match="unknown NAIF body id"):
        vk.analytical_position("pluto", _J2000_TT)


def test_analytical_position_accepts_naif_id() -> None:
    # NAIF id 301 == Moon; must agree with the name-based lookup above.
    vk = Vedaksha()
    by_name = vk.analytical_position("moon", _J2000_TT)
    by_id = vk.analytical_position(301, _J2000_TT)
    assert by_name == by_id
