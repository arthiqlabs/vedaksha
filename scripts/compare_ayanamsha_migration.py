#!/usr/bin/env python3
"""
Post-freeze migration delta for the ayanamsha re-derivation. Section 6.4.

**This is not a gate. It has no pass/fail.** It exists to size a breaking change,
and it must be run only AFTER the derived values are frozen and committed, so
that it cannot influence them. It is deliberately a separate script from
`generate_ayanamsha.py` for the same reason.

    "If a derived value lands close to what we shipped, that is a welcome result
     and nothing more; if it lands 16 arcseconds away, we ship the derived value."

How it avoids reading the artefact
----------------------------------
The old constants are the contaminated artefact. This script never reads them.
It checks the baseline tag out into a throwaway git worktree, writes a small
driver into that worktree, and runs the OLD ENGINE'S OWN `ayanamsha_value`,
capturing outputs. Nothing reads the old source; the old code is driven, and only
its outputs cross.

Those outputs go into the migration note. Per section 6.6 they must never become
a test fixture: regression-testing against them would reinstate the
aim-at-the-known-answer failure the re-derivation exists to undo.

Usage:
    python3 scripts/compare_ayanamsha_migration.py [--baseline <commit-ish>]

Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURE = os.path.join(
    PROJECT_ROOT, "crates", "vedaksha-astro", "tests", "fixtures", "ayanamsha.json"
)

# Old variant name -> new variant name. Only pairs where a caller could
# plausibly have moved from one to the other; the delta is meaningful only for
# the first group.
#
# "renamed" — same system, re-derived. The delta is the breaking change.
# "rebased" — the new system answers a related but DIFFERENT defining condition.
#             The delta is reported for information and is not a regression.
MAPPING = [
    ("Lahiri", "IndianOfficial", "renamed"),
    ("FaganBradley", "FaganBradley", "renamed"),
    ("Krishnamurti", "Krishnamurti", "renamed"),
    ("Raman", "Raman", "renamed"),
    ("Yukteshwar", "Yukteshwar", "renamed"),
    ("SuryaSiddhanta", "SuryaSiddhanta", "renamed"),
    ("GalacticCenter0Sag", "GalacticCenter0Sag", "renamed"),
    ("TrueChitrapaksha", "TrueChitra", "renamed"),
    ("TrueRevati", "RevatiPaksha", "rebased"),
    ("TruePushya", "PushyaPaksha", "rebased"),
    ("TrueMula", "ChandraHari", "rebased"),
]

DRIVER = r'''
use vedaksha_astro::sidereal::{Ayanamsha, ayanamsha_value};

#[test]
fn dump_baseline_ayanamsha() {
    let epochs: Vec<f64> = std::env::var("VDK_EPOCHS")
        .unwrap()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let systems: Vec<(&str, Ayanamsha)> = vec![
%SYSTEMS%
    ];
    for jd in epochs {
        for (name, sys) in &systems {
            println!("BASELINE\t{name}\t{jd}\t{:.12}", ayanamsha_value(*sys, jd));
        }
    }
}
'''


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, text=True, **kw)


def main():
    ap = argparse.ArgumentParser()
    # The commit v5.0.2 pointed at. The tag no longer exists on origin; see the note
    # in check_spec_hygiene.py. This commit is an ancestor of main.
    ap.add_argument("--baseline", default="91869cea1578c6b38b6587a0cd403beca29eb9f3",
                    help="git ref of the shipped baseline (default: the v5.0.2 commit)")
    ap.add_argument("--output", default="-", help="write the markdown table here")
    args = ap.parse_args()

    with open(FIXTURE, encoding="utf-8") as fh:
        fixture = json.load(fh)
    rows = fixture["rows"]
    epochs = [r["jd_tt"] for r in rows]
    labels = {r["jd_tt"]: r["label"] for r in rows}
    new_values = {(s, r["jd_tt"]): v for r in rows for s, v in r["ayanamsha_deg"].items()}

    worktree = tempfile.mkdtemp(prefix="ayanamsha-baseline-")
    shutil.rmtree(worktree)
    baseline = {}
    try:
        print(f"checking out {args.baseline} into {worktree} ...", file=sys.stderr)
        run(["git", "worktree", "add", "--detach", worktree, args.baseline], cwd=PROJECT_ROOT)

        systems_src = "\n".join(
            f'        ("{old}", Ayanamsha::{old}),' for old, _, _ in MAPPING
        )
        driver_dir = os.path.join(worktree, "crates", "vedaksha-astro", "tests")
        os.makedirs(driver_dir, exist_ok=True)
        driver_path = os.path.join(driver_dir, "zz_baseline_dump.rs")
        with open(driver_path, "w", encoding="utf-8") as fh:
            fh.write(DRIVER.replace("%SYSTEMS%", systems_src))

        env = dict(os.environ, VDK_EPOCHS=",".join(repr(e) for e in epochs))
        print("building and running the baseline engine ...", file=sys.stderr)
        proc = subprocess.run(
            [
                "cargo", "test", "-p", "vedaksha-astro",
                "--test", "zz_baseline_dump", "--", "--nocapture",
            ],
            cwd=worktree, env=env, capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(proc.stdout[-4000:], file=sys.stderr)
            print(proc.stderr[-4000:], file=sys.stderr)
            print("FAILED to run the baseline engine", file=sys.stderr)
            return 1
        for line in proc.stdout.splitlines():
            if line.startswith("BASELINE\t"):
                _, name, jd, val = line.split("\t")
                baseline[(name, float(jd))] = float(val)
    finally:
        subprocess.run(
            ["git", "worktree", "remove", "--force", worktree],
            cwd=PROJECT_ROOT, capture_output=True, text=True,
        )

    if not baseline:
        print("no baseline values captured — refusing to emit an empty table", file=sys.stderr)
        return 1

    out = []
    out.append(f"### Delta against {args.baseline}, at J2000.0\n")
    out.append("| v5 name | v6 name | relation | delta (arcsec) |")
    out.append("|---|---|---|---|")
    j2000 = 2451545.0
    for old, new, relation in MAPPING:
        b = baseline.get((old, j2000))
        n = new_values.get((new, j2000))
        if b is None or n is None:
            out.append(f"| `{old}` | `{new}` | {relation} | not comparable |")
            continue
        out.append(f"| `{old}` | `{new}` | {relation} | {(n - b) * 3600.0:+.2f} |")

    out.append("\n### Range of the delta across the sampled epochs\n")
    out.append("| v5 name | v6 name | min (arcsec) | max (arcsec) | worst epoch |")
    out.append("|---|---|---|---|---|")
    for old, new, _ in MAPPING:
        deltas = []
        for jd in epochs:
            b, n = baseline.get((old, jd)), new_values.get((new, jd))
            if b is not None and n is not None:
                deltas.append(((n - b) * 3600.0, jd))
        if not deltas:
            continue
        lo = min(deltas)[0]
        hi = max(deltas)[0]
        worst = max(deltas, key=lambda d: abs(d[0]))
        out.append(
            f"| `{old}` | `{new}` | {lo:+.2f} | {hi:+.2f} | {labels[worst[1]]} ({worst[0]:+.2f}) |"
        )

    text = "\n".join(out) + "\n"
    if args.output == "-":
        print(text)
    else:
        with open(args.output, "w", encoding="utf-8") as fh:
            fh.write(text)
        print(f"wrote {args.output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
