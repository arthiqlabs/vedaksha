#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
"""Assert metrics.json's accuracy figures against what the oracle tests print.

# Why this exists

The accuracy residuals were hand-maintained constants on the website from the
first release until 2026-09-01. Nothing checked them, so nothing noticed when
they went unverified from v7.0.0 through v9.1.0 -- four releases during which
the site presented numbers whose last re-measurement predated three of them.

Publishing them in metrics.json fixes WHERE they live. It does not, on its own,
fix the thing that made them rot: a number nobody re-derives is a number that
drifts. This script is the re-derivation. It runs the two oracle tests, reads
the mean, max, max-body and comparison count each one PRINTS, and fails if
metrics.json disagrees.

That makes the published figures a claim the engine can falsify, rather than
prose maintained by memory.

# What it does NOT do

It does not judge whether the residuals are good. The tests own that: each
asserts its own ceiling and fails independently if accuracy regresses. This
script only asserts that what we PUBLISH equals what we MEASURE. Both can be
true while the engine is inaccurate, and that is the tests' problem, not this
script's.

It also does not check `accuracy.quantity` or `accuracy.targetConvention`.
Those are scope prose -- what the figure measures (apparent geocentric
ecliptic longitude, not latitude/distance/speed) and what convention the
comparison uses (barycentres, not planet centres) -- not numbers the oracle
tests print. There is nothing in `--nocapture` output to assert them against;
they are re-derivations of facts fixed by the fixture's own generator
(`scripts/generate_horizons_oracle.py`) and by `oracle_comparison.rs`'s field
list, not by a per-run measurement. Every *numeric* field in the accuracy
block -- including each `perBody[].comparisons` -- is covered by `compare()`
above; only this pair of descriptive strings is exempt, deliberately.

# Running it

    python3 scripts/check_metrics_accuracy.py

Requires `data/de440s.bsp` (fetch with `bash scripts/download_de440s.sh`).
Without it the SPK oracle cannot run; the script reports that and exits 1
rather than passing vacuously on a check it never performed -- an unrun check
reported as a pass is worse than a known gap.
"""
from __future__ import annotations

import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
METRICS = ROOT / "metrics.json"
KERNEL = ROOT / "data" / "de440s.bsp"

# Floating-point figures are published rounded to three decimals, which is how
# the tests print them, so equality is exact rather than approximate.
TOLERANCE = 0.0005


def run_oracle(test: str) -> str:
    """Run one release-only oracle test and return its stdout."""
    cmd = [
        "cargo", "test", "-p", "vedaksha-ephem-core", "--release", "--locked",
        "--test", test, "--", "--include-ignored", "--nocapture",
    ]
    # Merged, deliberately: cargo writes its own progress to stderr while the
    # test's report goes to stdout, and which stream carries which line is not
    # a contract worth depending on. Parsing the union is stable against that.
    proc = subprocess.run(
        cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    if proc.returncode != 0:
        print(f"FAIL: {test} did not pass; accuracy figures cannot be confirmed")
        print(proc.stdout[-3000:])
        sys.exit(1)
    return proc.stdout


def parse_per_body(era: str) -> list[dict]:
    """Pull the `Body  n  mean"  max"` table out of a measured-era block.

    Shared by both oracle tests: they print the table in the identical shape
    (`analytical_oracle.rs` and `oracle_comparison.rs` both write it via the
    same `{:<10} {:>7} {:>10.3} {:>10.3}` format), so one parser covers both.
    """
    rows = re.findall(
        r"^([A-Za-z]+)\s+(\d+)\s+([\d.]+)\s+([\d.]+)\s*$", era, re.MULTILINE
    )
    return [
        {
            "body": body,
            "comparisons": int(n),
            "meanArcsec": float(mean),
            "maxArcsec": float(mx),
        }
        for body, n, mean, mx in rows
    ]


def parse_spk(out: str) -> dict:
    """Pull the measured-era block out of oracle_comparison's report."""
    era = out.split("--- Measured-")[1] if "--- Measured-" in out else ""
    comparisons = re.search(r"Comparisons:\s+(\d+)", era)
    mean = re.search(r"Mean error:\s+([\d.]+)", era)
    mx = re.search(r"Max error:\s+([\d.]+) arcseconds \((\w+) at (\d{4}-\w{3}-\d{2})", era)
    if not (comparisons and mean and mx):
        print("FAIL: could not parse oracle_comparison output; its format changed")
        print(era[:1500])
        sys.exit(1)
    # The per-body table lives after the measured-era summary but before the
    # predicted-ΔT era block, so scope the table search to just that span.
    body_era = era.split("--- Predicted-")[0] if "--- Predicted-" in era else era
    per_body = parse_per_body(body_era)
    return {
        "comparisons": int(comparisons.group(1)),
        "meanArcsec": float(mean.group(1)),
        "maxArcsec": float(mx.group(1)),
        "maxBody": mx.group(2),
        "perBody": per_body,
    }


def parse_analytical(out: str) -> dict:
    """Pull the measured-era block out of analytical_oracle's report.

    Scoped to the measured-era section, not searched across the whole report:
    both tests print a full-range block FIRST, whose max is the Moon in 2099 at
    tens of arcseconds. That figure is Delta T prediction divergence rather than
    ephemeris error, and a regex run over the whole output silently matches it
    instead -- which is exactly what this parser did on its first draft, and
    what the comparison below then correctly rejected.
    """
    era = out.split("--- Measured-")[1] if "--- Measured-" in out else ""
    comparisons = re.search(r"Comparisons:\s+(\d+)", era)
    mean = re.search(r'Mean error:\s+([\d.]+)"', era)
    mx = re.search(r'Max error:\s+([\d.]+)" \((\w+) at (\d{4}-\w{3}-\d{2})', era)
    if not (comparisons and mean and mx):
        print("FAIL: could not parse analytical_oracle output; its format changed")
        print(era[:1500])
        sys.exit(1)
    return {
        "comparisons": int(comparisons.group(1)),
        "meanArcsec": float(mean.group(1)),
        "maxArcsec": float(mx.group(1)),
        "maxBody": mx.group(2),
        "perBody": parse_per_body(era),
    }


def compare(label: str, published: dict, measured: dict) -> list[str]:
    problems = []
    for key in ("comparisons", "maxBody"):
        if published.get(key) != measured[key]:
            problems.append(
                f"{label}.{key}: metrics.json says {published.get(key)!r}, "
                f"the test measured {measured[key]!r}"
            )
    for key in ("meanArcsec", "maxArcsec"):
        pub = published.get(key)
        if not isinstance(pub, (int, float)) or abs(pub - measured[key]) > TOLERANCE:
            problems.append(
                f"{label}.{key}: metrics.json says {pub}, "
                f"the test measured {measured[key]}"
            )
    problems += compare_per_body(label, published.get("perBody"), measured["perBody"])
    return problems


def compare_per_body(label: str, published: object, measured: list[dict]) -> list[str]:
    """Check every row of a `perBody` table against the matching measured row.

    A body published but not measured, or measured but not published, is a
    failure in either direction — that is the whole point of the guard.
    """
    problems = []
    if not isinstance(published, list):
        return [f"{label}.perBody: metrics.json has no `perBody` array"]

    published_by_body = {}
    for row in published:
        if not isinstance(row, dict) or "body" not in row:
            problems.append(f"{label}.perBody: a published row is missing `body`")
            continue
        published_by_body[row["body"]] = row
    measured_by_body = {row["body"]: row for row in measured}

    for body, pub_row in published_by_body.items():
        if body not in measured_by_body:
            problems.append(
                f"{label}.perBody[{body}]: published but not in the measured "
                f"per-body table"
            )
            continue
        m_row = measured_by_body[body]
        if pub_row.get("comparisons") != m_row["comparisons"]:
            problems.append(
                f"{label}.perBody[{body}].comparisons: metrics.json says "
                f"{pub_row.get('comparisons')!r}, the test measured {m_row['comparisons']!r}"
            )
        for key in ("meanArcsec", "maxArcsec"):
            pub = pub_row.get(key)
            if not isinstance(pub, (int, float)) or abs(pub - m_row[key]) > TOLERANCE:
                problems.append(
                    f"{label}.perBody[{body}].{key}: metrics.json says {pub}, "
                    f"the test measured {m_row[key]}"
                )

    for body in measured_by_body:
        if body not in published_by_body:
            problems.append(
                f"{label}.perBody[{body}]: measured by the test but not "
                f"published in metrics.json"
            )

    return problems


def main() -> int:
    if not KERNEL.exists():
        print(f"FAIL: {KERNEL} is absent, so the SPK oracle cannot run.")
        print("Fetch it with `bash scripts/download_de440s.sh`.")
        print("Exiting 1 rather than reporting a pass for a check that never ran.")
        return 1

    metrics = json.loads(METRICS.read_text())
    accuracy = metrics.get("accuracy")
    if not accuracy:
        print("FAIL: metrics.json has no `accuracy` block")
        return 1

    print("Running oracle_comparison (SPK path)...")
    spk = parse_spk(run_oracle("oracle_comparison"))
    print("Running analytical_oracle (analytical path)...")
    analytical = parse_analytical(run_oracle("analytical_oracle"))

    problems = compare("accuracy.spk", accuracy.get("spk", {}), spk)
    problems += compare("accuracy.analytical", accuracy.get("analytical", {}), analytical)

    if problems:
        print("\nFAIL: metrics.json disagrees with what the engine measures.\n")
        for p in problems:
            print(f"  - {p}")
        print(
            "\nvedaksha.net serves these figures live. Update metrics.json to the "
            "measured values above, in the same commit as whatever moved them."
        )
        return 1

    print(
        f"\nok: metrics.json matches the engine\n"
        f"  SPK        {spk['meanArcsec']}\" mean, {spk['maxArcsec']}\" max "
        f"({spk['maxBody']}), n={spk['comparisons']}\n"
        f"  analytical {analytical['meanArcsec']}\" mean, "
        f"{analytical['maxArcsec']}\" max ({analytical['maxBody']}), "
        f"n={analytical['comparisons']}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
