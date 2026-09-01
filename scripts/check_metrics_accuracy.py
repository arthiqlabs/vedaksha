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
    return {
        "comparisons": int(comparisons.group(1)),
        "meanArcsec": float(mean.group(1)),
        "maxArcsec": float(mx.group(1)),
        "maxBody": mx.group(2),
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
