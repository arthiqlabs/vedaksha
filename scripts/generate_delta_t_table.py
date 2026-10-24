#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Generate `crates/vedaksha-ephem-core/src/delta_t_table.rs`.

The Delta T (TT - UT1) behind `delta_t.rs` is built from four published sources:

* **Stephenson, Morrison and Hohenkerk 2016 (SMH2016)**, "Measurement of the
  Earth's rotation: 720 BC to AD 2015", Proc. R. Soc. A 472: 20160404,
  doi:10.1098/rspa.2016.0404. Table S15 of the electronic supplementary
  material gives the cubic spline they fitted to Delta T from -720.0 to 2016.0,
  as coefficients `a_0..a_3` for each interval `K_i .. K_{i+1}`. Used here from
  -720 to 1962.0. The article and its supplement are under Creative Commons
  Attribution 4.0 (https://creativecommons.org/licenses/by/4.0/); the
  supplement is the zip `rspa20160404supp2.zip`, figshare collection
  doi:10.6084/m9.figshare.c.3579767. The numbers are transcribed unchanged.
  The script reads `Table-S15.txt` out of the zip itself and records both
  sha256 sums.
* `historic_deltat.data` -- U.S. Naval Observatory, "Historic Delta T and LOD
  values from McCarthy and Babcock (1986)", half-year rows from 1657.0. The rows
  from 1962.0 to 1973.0 are used.
  https://maia.usno.navy.mil/ser7/historic_deltat.data
* `deltat.data` -- USNO, "Monthly determinations of Delta T (TT - UT1) since
  1973", one row per month, the value at 0h on the first of the month. Every row
  is used.
  https://maia.usno.navy.mil/ser7/deltat.data
* `deltat.preds` -- USNO, "Long-term predictions of Delta T (TT - UT1) from
  finals.data", one row per quarter with its stated error. The rows dated after
  the last monthly row are used, through the file's last row.
  https://maia.usno.navy.mil/ser7/deltat.preds

The USNO files are products of the IERS Rapid Service/Prediction Center
(Distribution Statement A: approved for public release; a US Government work).
Over the overlap of the first two, 1973.5 to 1984.0 (22 half-year rows), they
agree to within 6 milliseconds (largest 5.2 ms, at 1981.5), which is the
evidence that reading the half-year rows as a value at the instant 1962.0,
1962.5, ... is right.

Why the spline stops at 1962.0: SMH2016 says (section 2(a)(vii), "Atomic time
data") that "the values of TAI - UT1 are tabulated by the IERS at a daily interval
from 1962 to the present", and (section 7, Conclusion) that after AD 1600
timings of lunar occultations "provide the most accurate measure of the Earth's
rotation on time scales longer than about a year, until the introduction of the
atomic time scale from 1962 onwards". From 1962.0 the USNO rows, which state an
error of 0.001 s, are the definitive measurement, and the spline, which smooths
occultation timings through 3-year knots and which the authors used over 1962 to
2015 only "as a comparative control", is not.

This script does no network access. Fetch the files yourself, each into a
directory of its own, and pass their paths:

    python3 -I scripts/generate_delta_t_table.py \\
        --smh-zip  /path/to/rspa20160404supp2.zip \\
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

The knot time of the USNO and predicted rows is a decimal year,
`year + (JD - JD(Jan 1)) / length of year`, the same function `delta_t.rs` uses to
turn a Julian Day into a year, so a knot at the first of a month is hit exactly. A
half-year row is at the decimal year printed in the file. The spline's `Y` is
read as the same decimal year; see the note in `delta_t.rs` on how little that
choice matters.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import pathlib
import sys
import zipfile

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "crates" / "vedaksha-ephem-core" / "src" / "delta_t_table.rs"

HISTORIC_URL = "https://maia.usno.navy.mil/ser7/historic_deltat.data"
MONTHLY_URL = "https://maia.usno.navy.mil/ser7/deltat.data"
PREDS_URL = "https://maia.usno.navy.mil/ser7/deltat.preds"
SMH_DOI = "10.1098/rspa.2016.0404"
SMH_MEMBER = "Table-S15.txt"

# Half-year rows are taken from this decimal year (inclusive) to the year the
# monthly file takes over (inclusive: 1973.0 is the last half-year row, and the
# first monthly row is 1973-02-01). This is also where the spline stops.
HISTORIC_FIRST = 1962.0
HISTORIC_LAST = 1973.0


def decimal_year(year: int, month: int, day: int) -> float:
    start = datetime.date(year, 1, 1).toordinal()
    end = datetime.date(year + 1, 1, 1).toordinal()
    return year + (datetime.date(year, month, day).toordinal() - start) / (end - start)


def group(text: str) -> str:
    """Digit-group a decimal literal as clippy wants: threes either side of the point."""
    sign = "-" if text.startswith("-") else ""
    whole, _, frac = text.lstrip("-").partition(".")
    frac = frac or "0"  # a float literal always has a fractional part
    if len(whole) >= 5:
        head = len(whole) % 3
        parts = ([whole[:head]] if head else []) + [whole[i : i + 3] for i in range(head, len(whole), 3)]
        whole = "_".join(parts)
    chunks = [frac[i : i + 3] for i in range(0, len(frac), 3)]
    return f"{sign}{whole}.{'_'.join(chunks)}"


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


def read_spline(zip_path: pathlib.Path) -> tuple[list[tuple[str, ...]], str]:
    """Rows 1.. of Table S15 up to the handover, read out of the supplement zip.

    Returns the rows `(K_i, K_{i+1}, a0, a1, a2, a3)` as the strings printed in the
    table, for the intervals ending at or before HISTORIC_FIRST, and the sha256 of
    the member. The zip is read as bytes and never extracted or executed.
    """
    with zipfile.ZipFile(zip_path) as z:
        data = z.read(SMH_MEMBER)
    rows = []
    for line in data.decode("utf-8", errors="replace").splitlines():
        parts = line.split()
        if len(parts) == 7 and parts[0].isdigit():
            rows.append(tuple(parts[1:]))
    if len(rows) != 54:
        raise SystemExit(f"{zip_path}: expected 54 rows in {SMH_MEMBER}, found {len(rows)}")
    if rows[0][0] != "-720.0" or rows[-1][1] != "2016.0":
        raise SystemExit(f"{zip_path}: Table S15 does not span -720.0 to 2016.0")
    if any(a[1] != b[0] for a, b in zip(rows, rows[1:])):
        raise SystemExit(f"{zip_path}: Table S15 intervals are not contiguous")
    used = [r for r in rows if float(r[1]) <= HISTORIC_FIRST]
    if float(used[-1][1]) != HISTORIC_FIRST:
        raise SystemExit(f"{zip_path}: no spline interval ends at {HISTORIC_FIRST}")
    return used, hashlib.sha256(data).hexdigest()


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(
    smh_zip: pathlib.Path,
    historic: pathlib.Path,
    monthly: pathlib.Path,
    preds: pathlib.Path,
    retrieved: str,
) -> str:
    spline, member_sha = read_spline(smh_zip)
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
        "//! Delta T (TT - UT1) table, -720 to 2033.75. GENERATED, do not edit.\n"
        "//!\n"
        "//! Written by `scripts/generate_delta_t_table.py` from four published sources,\n"
        f"//! retrieved {retrieved}:\n"
        "//!\n"
        "//! * Stephenson, F. R., Morrison, L. V. and Hohenkerk, C. Y., \"Measurement of the\n"
        "//!   Earth's rotation: 720 BC to AD 2015\", Proc. R. Soc. A 472: 20160404 (2016),\n"
        f"//!   doi:{SMH_DOI}. Table S15 of the electronic supplementary material,\n"
        "//!   \"The Polynomial Coefficients for DT -720.0 to 2016.0\", intervals 1 to\n"
        f"//!   {len(spline)} (-720.0 to {HISTORIC_FIRST}) of 54. The supplement is the zip\n"
        f"//!   `rspa20160404supp2.zip` (sha256 `{sha256(smh_zip)}`),\n"
        f"//!   member `{SMH_MEMBER}` (sha256 `{member_sha}`), figshare collection\n"
        "//!   doi:10.6084/m9.figshare.c.3579767. Licensed Creative Commons Attribution 4.0\n"
        "//!   (<https://creativecommons.org/licenses/by/4.0/>); the numbers are transcribed\n"
        "//!   unchanged. Intervals 37 to 54 (1962 to 2016) are not used: see `delta_t.rs`.\n"
        "//! * U.S. Naval Observatory, IERS Rapid Service/Prediction Center (public domain,\n"
        "//!   Distribution Statement A, approved for public release):\n"
        f"//!   * <{HISTORIC_URL}>\n"
        "//!     \"Historic Delta T and LOD values from McCarthy and Babcock (1986)\",\n"
        f"//!     half-year rows {HISTORIC_FIRST} to {HISTORIC_LAST} ({len(hist)} rows,\n"
        f"//!     sha256 `{sha256(historic)}`).\n"
        f"//!   * <{MONTHLY_URL}>\n"
        "//!     \"Monthly determinations of Delta T (TT - UT1) since 1973\", the value at\n"
        f"//!     0h on the first of each month, 1973-02-01 to {last_year}-{last_month:02d}-01\n"
        f"//!     ({len(mon)} rows, sha256 `{sha256(monthly)}`).\n"
        f"//!   * <{PREDS_URL}>\n"
        "//!     \"Long-term predictions of Delta T (TT - UT1) from finals.data\", the\n"
        f"//!     quarterly rows after {last_year}-{last_month:02d}-01, {pre[0][0]}-{pre[0][1]:02d}-{pre[0][2]:02d} to\n"
        f"//!     {pre[-1][0]}-{pre[-1][1]:02d}-{pre[-1][2]:02d} ({len(pre)} rows, sha256 `{sha256(preds)}`).\n"
        "//!     A prediction: the file states an error of 0.03 s in 2022 growing to 1 s by 2032.75.\n"
        "//!\n"
        "//! The spline is evaluated as Table S15's notes state: for the year and fraction\n"
        "//! `Y` take row `i` with `K_i <= Y <= K_{i+1}`, form `t = (Y - K_i) / (K_{i+1} - K_i)`,\n"
        "//! and `DT = a_0 + a_1 t + a_2 t^2 + a_3 t^3` seconds. The USNO and predicted knots are\n"
        "//! `(decimal year, seconds)`. A decimal year is `year + (JD - JD(Jan 1)) / length of\n"
        "//! year`, the function `delta_t.rs` uses to turn a Julian Day into a year. See\n"
        "//! `delta_t.rs` for how the pieces are used and joined.\n"
        "\n"
        "/// One interval of the SMH2016 cubic spline: `(K_i, K_{i+1}, a_0, a_1, a_2, a_3)`.\n"
        "pub(super) type SplineRow = (f64, f64, f64, f64, f64, f64);\n"
        "\n"
        "/// Table S15 intervals 1 to "
        f"{len(spline)}, -720.0 to {HISTORIC_FIRST}, contiguous and increasing.\n"
        "pub(super) static SPLINE: &[SplineRow] = &[\n"
    )
    for k0, k1, a0, a1, a2, a3 in spline:
        out.append(
            f"    ({group(k0)}, {group(k1)}, {group(a0)}, {group(a1)}, {group(a2)}, {group(a3)}),\n"
        )
    out.append("];\n\n")
    out.append(
        f"/// Delta T knots from {HISTORIC_FIRST} to the last monthly row, strictly increasing in\n"
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
    ap.add_argument("--smh-zip", type=pathlib.Path, required=True)
    ap.add_argument("--historic", type=pathlib.Path, required=True)
    ap.add_argument("--monthly", type=pathlib.Path, required=True)
    ap.add_argument("--preds", type=pathlib.Path, required=True)
    ap.add_argument("--retrieved", required=True, help="date the files were fetched, YYYY-MM-DD")
    ap.add_argument("--verify", action="store_true", help="compare with the committed file; do not write")
    ap.add_argument("--out", type=pathlib.Path, default=OUTPUT)
    args = ap.parse_args()

    text = render(args.smh_zip, args.historic, args.monthly, args.preds, args.retrieved)
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
