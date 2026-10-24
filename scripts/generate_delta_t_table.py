#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Generate `crates/vedaksha-ephem-core/src/delta_t_table.rs` from the USNO files.

The Delta T (TT - UT1) table behind `delta_t.rs` for 1900 to the present is two
public-domain USNO files, concatenated:

* `historic_deltat.data` -- "Historic Delta T and LOD values from McCarthy and
  Babcock (1986)", half-year rows from 1657.0. The rows from 1900.0 to 1973.0
  are used.
  https://maia.usno.navy.mil/ser7/historic_deltat.data
* `deltat.data` -- "Monthly determinations of Delta T (TT - UT1) since 1973",
  one row per month, the value at 0h on the first of the month. Every row is
  used.
  https://maia.usno.navy.mil/ser7/deltat.data
* `deltat.preds` -- "Long-term predictions of Delta T (TT - UT1) from
  finals.data", one row per quarter with its stated error. The rows dated after
  the last monthly row are used, through the file's last row.
  https://maia.usno.navy.mil/ser7/deltat.preds

Both are products of the IERS Rapid Service/Prediction Center at the U.S. Naval
Observatory (Distribution Statement A: approved for public release; a US
Government work). Over their overlap, 1973.5 to 1984.0 (22 half-year rows), the
two agree to within 6 milliseconds (largest 5.2 ms, at 1981.5), which is the
evidence that reading the half-year rows as a value at the instant 1900.0,
1900.5, ... is right.

This script does no network access. Fetch the two files yourself, into a
directory of their own, and pass their paths:

    python3 -I scripts/generate_delta_t_table.py \\
        --historic /path/to/historic_deltat.data \\
        --monthly  /path/to/deltat.data \\
        --preds    /path/to/deltat.preds \\
        --retrieved 2026-10-24                    # writes delta_t_table.rs
    python3 -I scripts/generate_delta_t_table.py ... --verify   # compare only

`--verify` regenerates and compares against the committed file, exiting
non-zero on a difference. It is not wired into CI: `deltat.data` gains a row
every month, so a fresh download is expected to differ from the committed
snapshot, and the point of `--verify` is to check a snapshot against the very
files it was made from (their sha256 sums are in the generated header).

The predicted rows are knots like the others. They are a prediction, not a
measurement: the file's error column grows from 0.03 s in 2022 to 1 s by 2032.75.

The knot time is a decimal year, `year + (JD - JD(Jan 1)) / length of year`,
the same function `delta_t.rs` uses to turn a Julian Day into a year, so a knot
at the first of a month is hit exactly. A half-year row is at the decimal year
printed in the file.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "crates" / "vedaksha-ephem-core" / "src" / "delta_t_table.rs"

HISTORIC_URL = "https://maia.usno.navy.mil/ser7/historic_deltat.data"
MONTHLY_URL = "https://maia.usno.navy.mil/ser7/deltat.data"
PREDS_URL = "https://maia.usno.navy.mil/ser7/deltat.preds"

# Half-year rows are taken from this decimal year (inclusive) to the year the
# monthly file takes over (inclusive: 1973.0 is the last half-year row, and the
# first monthly row is 1973-02-01).
HISTORIC_FIRST = 1900.0
HISTORIC_LAST = 1973.0


def decimal_year(year: int, month: int, day: int) -> float:
    start = datetime.date(year, 1, 1).toordinal()
    end = datetime.date(year + 1, 1, 1).toordinal()
    return year + (datetime.date(year, month, day).toordinal() - start) / (end - start)


def group(text: str) -> str:
    """Digit-group the fractional part of a decimal literal in threes, as clippy wants."""
    whole, frac = text.split(".")
    chunks = [frac[i : i + 3] for i in range(0, len(frac), 3)]
    return f"{whole}.{'_'.join(chunks)}"


def read_historic(path: pathlib.Path) -> list[tuple[float, str]]:
    rows = []
    for line in path.read_text().splitlines():
        parts = line.split()
        if len(parts) < 2:
            continue
        try:
            year = float(parts[0])
        except ValueError:
            continue
        if HISTORIC_FIRST <= year <= HISTORIC_LAST:
            rows.append((year, parts[1]))
    if not rows or rows[0][0] != HISTORIC_FIRST or rows[-1][0] != HISTORIC_LAST:
        raise SystemExit(f"{path}: expected half-year rows {HISTORIC_FIRST} to {HISTORIC_LAST}")
    years = [y for y, _ in rows]
    if any(b - a != 0.5 for a, b in zip(years, years[1:])):
        raise SystemExit(f"{path}: half-year rows are not contiguous")
    return rows


def read_monthly(path: pathlib.Path) -> list[tuple[int, int, str]]:
    rows = []
    for line in path.read_text().splitlines():
        parts = line.split()
        if len(parts) != 4:
            continue
        year, month, day = int(parts[0]), int(parts[1]), int(parts[2])
        if day != 1:
            raise SystemExit(f"{path}: expected first-of-month rows, saw day {day}")
        rows.append((year, month, parts[3]))
    if not rows or rows[0][:2] != (1973, 2):
        raise SystemExit(f"{path}: expected the first row to be 1973-02")
    months = [y * 12 + m for y, m, _ in rows]
    if any(b - a != 1 for a, b in zip(months, months[1:])):
        raise SystemExit(f"{path}: monthly rows are not contiguous")
    return rows


MJD_EPOCH = datetime.date(1858, 11, 17)  # MJD 0


def read_preds(path: pathlib.Path, after: datetime.date) -> list[tuple[int, int, int, str]]:
    """Rows of deltat.preds dated after `after`, as (year, month, day, TT-UT)."""
    rows = []
    for line in path.read_text().splitlines():
        parts = line.split()
        if len(parts) < 3:
            continue
        try:
            mjd = float(parts[0])
        except ValueError:
            continue  # header
        if mjd != int(mjd):
            raise SystemExit(f"{path}: expected whole-day MJD rows, saw {parts[0]}")
        date = MJD_EPOCH + datetime.timedelta(days=int(mjd))
        if date > after:
            rows.append((date.year, date.month, date.day, parts[2]))
    if not rows:
        raise SystemExit(f"{path}: no rows after {after}")
    dates = [datetime.date(y, m, d) for y, m, d, _ in rows]
    if any(b <= a for a, b in zip(dates, dates[1:])):
        raise SystemExit(f"{path}: rows are not strictly increasing in time")
    return rows


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(
    historic: pathlib.Path, monthly: pathlib.Path, preds: pathlib.Path, retrieved: str
) -> str:
    hist = read_historic(historic)
    mon = read_monthly(monthly)
    last_year, last_month, _ = mon[-1]
    pre = read_preds(preds, datetime.date(last_year, last_month, 1))
    out = []
    out.append(
        "// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.\n"
        "// Vedaksha — Vision from Vedas\n"
        "// SPDX-License-Identifier: BUSL-1.1\n"
        "// Contact: info@arthiq.net | https://vedaksha.net\n"
        "\n"
        "//! Delta T (TT - UT1) knots, 1900 to 2033.75. GENERATED, do not edit.\n"
        "//!\n"
        "//! Written by `scripts/generate_delta_t_table.py` from three public-domain files\n"
        "//! of the IERS Rapid Service/Prediction Center at the U.S. Naval Observatory\n"
        "//! (Distribution Statement A, approved for public release), retrieved\n"
        f"//! {retrieved}:\n"
        "//!\n"
        f"//! * <{HISTORIC_URL}>\n"
        "//!   \"Historic Delta T and LOD values from McCarthy and Babcock (1986)\",\n"
        f"//!   half-year rows 1900.0 to 1973.0 ({len(hist)} rows,\n"
        f"//!   sha256 `{sha256(historic)}`).\n"
        f"//! * <{MONTHLY_URL}>\n"
        "//!   \"Monthly determinations of Delta T (TT - UT1) since 1973\", the value at\n"
        f"//!   0h on the first of each month, 1973-02-01 to {last_year}-{last_month:02d}-01\n"
        f"//!   ({len(mon)} rows, sha256 `{sha256(monthly)}`).\n"
        f"//! * <{PREDS_URL}>\n"
        "//!   \"Long-term predictions of Delta T (TT - UT1) from finals.data\", the\n"
        f"//!   quarterly rows after {last_year}-{last_month:02d}-01, {pre[0][0]}-{pre[0][1]:02d}-{pre[0][2]:02d} to\n"
        f"//!   {pre[-1][0]}-{pre[-1][1]:02d}-{pre[-1][2]:02d} ({len(pre)} rows, sha256 `{sha256(preds)}`).\n"
        "//!   A prediction: the file states an error of 0.03 s in 2022 growing to 1 s by 2032.75.\n"
        "//!\n"
        "//! Each knot is `(decimal year, seconds)`. A decimal year is\n"
        "//! `year + (JD - JD(Jan 1)) / length of year`, the function `delta_t.rs` uses to\n"
        "//! turn a Julian Day into a year. See `delta_t.rs` for how the knots are used.\n"
        "\n"
        "/// Delta T knots from 1900.0 to the last monthly row, strictly increasing in\n"
        "/// time. Measured.\n"
        "pub(super) static MODERN: &[(f64, f64)] = &[\n"
        "    // McCarthy and Babcock (1986), USNO historic_deltat.data, half-year rows.\n"
    )
    for year, value in hist:
        out.append(f"    ({group(f'{year:.1f}')}, {value}),\n")
    out.append("    // USNO deltat.data, monthly rows (0h on the first of the month).\n")
    for year, month, value in mon:
        out.append(f"    ({group(f'{decimal_year(year, month, 1):.9f}')}, {value}),\n")
    out.append("];\n\n")
    out.append(
        "/// Predicted Delta T knots after the last monthly row, strictly increasing in\n"
        "/// time, continuing [`MODERN`]. A prediction, not a measurement.\n"
        "pub(super) static PREDICTED: &[(f64, f64)] = &[\n"
        "    // USNO deltat.preds, quarterly rows.\n"
    )
    for year, month, day, value in pre:
        out.append(f"    ({group(f'{decimal_year(year, month, day):.9f}')}, {value}),\n")
    out.append("];\n")
    return "".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("--historic", type=pathlib.Path, required=True)
    ap.add_argument("--monthly", type=pathlib.Path, required=True)
    ap.add_argument("--preds", type=pathlib.Path, required=True)
    ap.add_argument("--retrieved", required=True, help="date the files were fetched, YYYY-MM-DD")
    ap.add_argument("--verify", action="store_true", help="compare with the committed file; do not write")
    ap.add_argument("--out", type=pathlib.Path, default=OUTPUT)
    args = ap.parse_args()

    text = render(args.historic, args.monthly, args.preds, args.retrieved)
    if args.verify:
        committed = args.out.read_text()
        if committed != text:
            print(f"DRIFT: {args.out} differs from what these inputs generate", file=sys.stderr)
            return 1
        print(f"OK: {args.out.name} matches the inputs")
        return 0
    args.out.write_text(text)
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
