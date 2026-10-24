#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Generate `crates/vedaksha-ephem-core/tests/common/tai_utc.rs` from USNO's leap-second file.

`tai-utc.dat` is the U.S. Naval Observatory's table of `TAI - UTC` since 1961,
https://maia.usno.navy.mil/ser7/tai-utc.dat, a public-domain US Government
product (Distribution Statement A). Each line gives, from a Julian Day at 0h
UTC, `TAI - UTC = a + (MJD - MJD0) * rate` seconds; before 1972 the rate is
non-zero (UTC then ran at a rate offset from atomic time, the "rubber second"),
from 1972 it is zero and the value is a whole number of leap seconds.

The two Horizons oracle tests use it to turn the fixture's UTC time tags into
TT: `TT = UTC + 32.184 s + (TAI - UTC)`. Horizons' manual says its UT is UTC from
1962, and the table agrees with Horizons' own `TDB-UT` output to 1.7 ms (the
periodic TDB - TT term) at all 1,679 oracle epochs from 1962-01-20 on.

This script does no network access. Fetch the file into a directory of its own
and pass its path:

    python3 -I scripts/generate_tai_utc_table.py \\
        --input /path/to/tai-utc.dat --retrieved 2026-10-24    # writes tai_utc.rs
    python3 -I scripts/generate_tai_utc_table.py ... --verify  # compare only

`--verify` regenerates and compares against the committed file, exiting non-zero
on a difference. It is not wired into CI: a new leap second adds a line to the
source file, so a fresh download is expected to differ from the committed
snapshot (whose sha256 is in the generated header).
"""

from __future__ import annotations

import argparse
import hashlib
import pathlib
import re
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "crates" / "vedaksha-ephem-core" / "tests" / "common" / "tai_utc.rs"
URL = "https://maia.usno.navy.mil/ser7/tai-utc.dat"

LINE = re.compile(
    r"=JD\s+([\d.]+)\s+TAI-UTC=\s*([\d.]+)\s*S \+ \(MJD - ([\d.]+)\) X ([\d.]+)\s*S"
)


def group(text: str) -> str:
    """Digit-group a decimal literal as clippy wants: threes either side of the point."""
    whole, _, frac = text.partition(".")
    frac = frac or "0"  # a float literal always has a fractional part
    if len(whole) >= 5:
        head = len(whole) % 3
        parts = ([whole[:head]] if head else []) + [whole[i : i + 3] for i in range(head, len(whole), 3)]
        whole = "_".join(parts)
    chunks = [frac[i : i + 3] for i in range(0, len(frac), 3)]
    return f"{whole}.{'_'.join(chunks)}"


def parse(path: pathlib.Path) -> list[tuple[str, str, str, str]]:
    rows = []
    for line in path.read_text().splitlines():
        m = LINE.search(line)
        if not m:
            if line.strip():
                raise SystemExit(f"{path}: unparsed line: {line!r}")
            continue
        rows.append(m.groups())
    if not rows:
        raise SystemExit(f"{path}: no rows")
    jds = [float(r[0]) for r in rows]
    if any(b <= a for a, b in zip(jds, jds[1:])):
        raise SystemExit(f"{path}: rows are not strictly increasing in time")
    if jds[0] > 2437300.5:
        raise SystemExit(f"{path}: expected the table to start by 1961-01-01")
    return rows


def render(path: pathlib.Path, retrieved: str) -> str:
    rows = parse(path)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    out = [
        "// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.\n"
        "// Vedaksha — Vision from Vedas\n"
        "// SPDX-License-Identifier: BUSL-1.1\n"
        "// Contact: info@arthiq.net | https://vedaksha.net\n"
        "\n"
        "//! TAI - UTC since 1961. GENERATED, do not edit.\n"
        "//!\n"
        "//! Written by `scripts/generate_tai_utc_table.py` from the U.S. Naval\n"
        "//! Observatory's leap-second file, a public-domain US Government product\n"
        "//! (Distribution Statement A, approved for public release):\n"
        "//!\n"
        f"//! * <{URL}>\n"
        f"//!   retrieved {retrieved}, {len(rows)} rows,\n"
        f"//!   sha256 `{digest}`.\n"
        "//!\n"
        "//! Each row is `(JD of 0h UTC at which it takes effect, a, MJD0, rate)`:\n"
        "//! from that day `TAI - UTC = a + (MJD - MJD0) * rate` seconds. Before 1972 the\n"
        "//! rate is non-zero (UTC ran at a rate offset from atomic time); from 1972 it\n"
        "//! is zero and `a` is a whole number of leap seconds. The last row, 2017-01-01,\n"
        "//! is 37 s and is still in force at the retrieval date.\n"
        "\n"
        "/// `TAI - UTC` rows in effect from the given Julian Day, strictly increasing.\n"
        "pub const TAI_UTC: &[(f64, f64, f64, f64)] = &[\n"
    ]
    for jd, a, mjd0, rate in rows:
        out.append(f"    ({group(jd)}, {group(a)}, {group(mjd0)}, {group(rate)}),\n")
    out.append("];\n")
    return "".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("--input", type=pathlib.Path, required=True)
    ap.add_argument("--retrieved", required=True, help="date the file was fetched, YYYY-MM-DD")
    ap.add_argument("--verify", action="store_true", help="compare with the committed file; do not write")
    ap.add_argument("--out", type=pathlib.Path, default=OUTPUT)
    args = ap.parse_args()

    text = render(args.input, args.retrieved)
    if args.verify:
        if args.out.read_text() != text:
            print(f"DRIFT: {args.out} differs from what this input generates", file=sys.stderr)
            return 1
        print(f"OK: {args.out.name} matches the input")
        return 0
    args.out.write_text(text)
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
