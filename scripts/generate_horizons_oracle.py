#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedākṣha — Vision from Vedas
# Licensed under BSL 1.1. See LICENSE file.
"""Generate the JPL Horizons oracle fixture for `oracle_comparison.rs`.

Fetches apparent geocentric ecliptic longitudes from the NASA/JPL Horizons
API and writes `tests/oracle_jpl/reference_positions.json`, the independent
reference the SpkReader accuracy test compares against.

Why Horizons (and not a third-party ephemeris library): Vedākṣha's BSL 1.1
position rests on a documented clean-room trail. Horizons output is a NASA
public-domain US Government work, so using it as reference data carries no
copyleft question. It is also an *independent* kernel from the one under
test — Horizons serves DE441, while `SpkReader` reads the bundled DE440s.

## Frame and time-scale contract

These must match `coordinates::apparent_position` exactly or the comparison
is meaningless:

* **Time scale — UT.** `apparent_position(provider, body, jd)` takes `jd` in
  UT1 (see `compute_ecliptic_with_frame`, whose parameter is `jd_ut` and
  which converts via `delta_t::ut1_to_tt`). Horizons OBSERVER tables read
  input JD as UT and report `Date_________JDUT`. Both sides therefore speak
  UT and no ΔT correction is applied here.
* **Target — barycentres.** `Body::naif_id()` returns barycentre IDs
  (Mercury=1, Venus=2, Mars=4, Jupiter=5 … Pluto=9), because `SpkReader`
  reads DE440s barycentre segments. We query the same IDs. Fetching planet
  *centres* (199, 499, 599 …) instead would inject a spurious ~0.1″ offset
  for the outer planets — right at the magnitude we are trying to measure.
* **Coordinates — apparent, ecliptic of date.** `QUANTITIES='31'` yields
  `ObsEcLon`/`ObsEcLat`: apparent longitude/latitude in the true ecliptic
  and equinox of date, geocentric (`CENTER='500@399'`). This is what
  `apparent_position` returns.

Sanity anchor: the Moon at JD 2451545.0 must come back as 223.3238°, the
value independently cited in `coordinates.rs`.

## Usage

    python3 scripts/generate_horizons_oracle.py            # regenerate
    python3 scripts/generate_horizons_oracle.py --verify    # drift check

`--verify` regenerates into a temp buffer and compares the sha256 against
the committed `.sha256` sidecar, exiting non-zero on drift. Mirrors the
contract of `generate_vsop87a.py` / `generate_elpmpp02.py`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

HORIZONS_API = "https://ssd.jpl.nasa.gov/api/horizons.api"

# (name, Horizons COMMAND). Must mirror `Body::naif_id()` in bodies.rs —
# barycentre IDs for the planets, since that is what DE440s stores and what
# SpkReader therefore returns.
BODIES: list[tuple[str, str]] = [
    ("Sun", "10"),
    ("Moon", "301"),
    ("Mercury", "1"),
    ("Venus", "2"),
    ("Mars", "4"),
    ("Jupiter", "5"),
    ("Saturn", "6"),
    ("Uranus", "7"),
    ("Neptune", "8"),
    ("Pluto", "9"),
]

# DE440s spans ~1849–2150; stay well inside so no row is skipped as
# out-of-range by the consuming test.
DEFAULT_START = "1900-01-01"
DEFAULT_STOP = "2100-01-01"
DEFAULT_STEP = "30d"

REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "tests" / "oracle_jpl" / "reference_positions.json"


def build_url(command: str, start: str, stop: str, step: str) -> str:
    params = {
        "format": "text",
        "COMMAND": f"'{command}'",
        "OBJ_DATA": "'NO'",
        "MAKE_EPHEM": "'YES'",
        "EPHEM_TYPE": "'OBSERVER'",
        "CENTER": "'500@399'",  # geocentric
        "START_TIME": f"'{start}'",
        "STOP_TIME": f"'{stop}'",
        "STEP_SIZE": f"'{step}'",
        "QUANTITIES": "'31,20'",  # 31 = ObsEcLon/ObsEcLat, 20 = range/range-rate
        "REF_SYSTEM": "'J2000'",
        "ANG_FORMAT": "'DEG'",
        "CAL_FORMAT": "'BOTH'",  # emit both calendar date and JD
        "CSV_FORMAT": "'YES'",
    }
    return f"{HORIZONS_API}?{urllib.parse.urlencode(params, safe=chr(39))}"


def fetch(url: str, retries: int = 3) -> str:
    """GET with linear backoff. Raises on final failure."""
    last: Exception | None = None
    for attempt in range(1, retries + 1):
        try:
            with urllib.request.urlopen(url, timeout=120) as resp:
                return resp.read().decode("utf-8", errors="replace")
        except (urllib.error.URLError, TimeoutError) as exc:  # transient
            last = exc
            if attempt < retries:
                time.sleep(3 * attempt)
    raise RuntimeError(f"Horizons fetch failed after {retries} attempts: {last}")


def parse_rows(text: str, body: str) -> list[dict]:
    """Extract the $$SOE/$$EOE block into oracle rows.

    Row layout (CSV_FORMAT + CAL_FORMAT='BOTH' + QUANTITIES='31,20'):
        0 date(UT)  1 JDUT  2 solar-presence  3 lunar-presence
        4 ObsEcLon  5 ObsEcLat  6 delta(AU)  7 deldot(km/s)
    """
    rows: list[dict] = []
    in_data = False
    for line in text.splitlines():
        if line.startswith("$$SOE"):
            in_data = True
            continue
        if line.startswith("$$EOE"):
            break
        if not in_data:
            continue
        cols = [c.strip() for c in line.split(",")]
        if len(cols) < 8:
            continue
        try:
            rows.append(
                {
                    "date": cols[0],
                    "jd": float(cols[1]),
                    "body": body,
                    "ref_longitude": float(cols[4]),
                    "ref_latitude": float(cols[5]),
                    # Horizons emits ~15 significant digits of range; 9 decimal
                    # places of an AU is ~0.15 m, far past anything we compare.
                    "ref_distance": round(float(cols[6]), 9),  # AU
                    "ref_speed": float(cols[7]),  # km/s
                }
            )
        except ValueError:
            # Horizons emits 'n.a.' for some quantities near singularities.
            continue
    if not rows:
        raise RuntimeError(f"no rows parsed for {body} — Horizons response shape changed?")
    return rows


def verify_anchor() -> None:
    """Fail loudly if the Moon@J2000 anchor drifts from the cited value.

    Guards against a silent frame or time-scale regression in the query: a
    UT/TT mixup alone would move the Moon by ~35″. Deliberately its own
    fetch rather than a lookup into the generated grid — J2000 does not fall
    on an arbitrary start/step, and an anchor that silently skips itself is
    worse than no anchor.
    """
    expected = 223.3238
    text = fetch(build_url("301", "JD2451545.0", "JD2451546.0", "1d"))
    rows = parse_rows(text, "Moon")
    got = rows[0]["ref_longitude"]
    if abs(got - expected) > 0.001:
        raise RuntimeError(
            f"Moon@J2000 anchor is {got:.4f}°, expected ≈{expected}° (cited in "
            f"coordinates.rs). The Horizons query's frame or time scale has "
            f"drifted — do NOT commit this fixture."
        )
    print(f"anchor OK: Moon@J2000 = {got:.4f}° (expected ≈{expected}°)")


def generate(start: str, stop: str, step: str) -> bytes:
    verify_anchor()
    all_rows: list[dict] = []
    for name, command in BODIES:
        print(f"fetching {name} (COMMAND='{command}') …", flush=True)
        text = fetch(build_url(command, start, stop, step))
        rows = parse_rows(text, name)
        print(f"  {len(rows)} rows")
        all_rows.extend(rows)

    all_rows.sort(key=lambda r: (r["body"], r["jd"]))

    payload = {
        "_provenance": {
            "source": "NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/)",
            "kernel": "DE441 (Horizons default) — independent of the DE440s under test",
            "license": "Public domain (US Government work)",
            "generator": "scripts/generate_horizons_oracle.py",
            "query": (
                "EPHEM_TYPE=OBSERVER, CENTER=500@399 (geocentric), QUANTITIES=31,20, "
                "REF_SYSTEM=J2000, ANG_FORMAT=DEG — ObsEcLon/ObsEcLat are apparent "
                "longitude/latitude in the true ecliptic and equinox of date"
            ),
            "time_scale": "UT (Horizons JDUT), matching apparent_position's jd_ut argument",
            "targets": "Barycentre NAIF IDs, mirroring Body::naif_id() and DE440s segments",
            "grid": f"{start} to {stop}, step {step}",
            "row_count": len(all_rows),
        },
        "rows": all_rows,
    }
    # Compact but newline-terminated; stable key order so the sha256 is
    # reproducible across runs.
    return (json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n").encode()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--verify", action="store_true", help="check for drift; do not write")
    ap.add_argument("--start", default=DEFAULT_START)
    ap.add_argument("--stop", default=DEFAULT_STOP)
    ap.add_argument("--step", default=DEFAULT_STEP)
    args = ap.parse_args()

    blob = generate(args.start, args.stop, args.step)
    digest = hashlib.sha256(blob).hexdigest()
    sidecar = OUTPUT.with_suffix(".json.sha256")

    if args.verify:
        if not sidecar.exists():
            print(f"ERROR: no sidecar at {sidecar}", file=sys.stderr)
            return 1
        expected = sidecar.read_text().split()[0].strip()
        if digest != expected:
            print(
                f"DRIFT: regenerated sha256 {digest} != committed {expected}\n"
                f"Horizons output changed for the same query. Investigate before "
                f"regenerating — this may be a real upstream ephemeris update.",
                file=sys.stderr,
            )
            return 1
        print(f"OK: {OUTPUT.name} matches committed sha256 ({digest[:12]}…)")
        return 0

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(blob)
    sidecar.write_text(f"{digest}  {OUTPUT.name}\n")
    size_mb = len(blob) / 1024 / 1024
    print(f"\nwrote {OUTPUT} ({size_mb:.2f} MB)")
    print(f"wrote {sidecar}")
    print(f"sha256 {digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
