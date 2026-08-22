#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
"""Block previously-shipped constants from re-entering a derivation spec.

A derivation spec is handed to an implementer who must not see the values being
replaced: a derivation that can see its target is not a derivation. Declaring
that a document contains no target is not a control -- an earlier draft of the
ayanamsha spec carried that declaration while stating two shipped constants
verbatim and several deltas from which the rest fell out by one subtraction.
This is the control.

The patterns are NOT stored in this file. They are extracted at run time from a
baseline git ref, so that:

  * no previously-shipped constant is written into the working tree by the very
    script that exists to keep them out of it, and
  * the check cannot drift out of date with what was actually shipped.

Usage:
    scripts/check_spec_hygiene.py SPEC.md [MORE.md ...]
    scripts/check_spec_hygiene.py --baseline <commit-ish> SPEC.md
    scripts/check_spec_hygiene.py --show-pattern-count     # audit, prints no values

Exit 0 clean, 1 on any hit. A hit is a blocking defect, not a style note.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

# The commit v5.0.2 pointed at, not the tag. A tag is a mutable ref and it did not
# survive the release cleanup: v5.0.0-v5.0.2 were fetched by CI on 2026-08-18 at
# 07:38 and were gone from origin by 22:06, which left this gate with zero patterns
# and turned it red. It refused to pass vacuously, which is the behaviour we want --
# but a control whose baseline can be deleted out from under it is fragile in the one
# direction that matters. This commit is an ancestor of main, so it is reachable from
# any full clone whatever happens to the tags.
DEFAULT_BASELINE = "91869cea1578c6b38b6587a0cd403beca29eb9f3"  # v5.0.2
DEFAULT_SOURCES = ["crates/vedaksha-astro/src/sidereal.rs"]
DEFAULT_ALLOWLIST = "scripts/spec_hygiene_allow.txt"

# Comparison phrasings that make a delta recoverable even when no digit appears:
# "derived X; reproduces the shipped value to within d" leaks the shipped value.
# These contain no constants, so they are safe to keep here.
PHRASE_PATTERNS = [
    r"reproduces the (currently )?shipped",
    r"differs from the shipped",
    r"the shipped (value|constant)",
    r"currently ships",
    r"adjusted by [+−-]?\d",
    r"\badrift\b",
    r"agrees with .{0,40}to (within )?[≈~]?\s*\d",
]


def baseline_text(ref: str, paths: list[str]) -> str:
    out = []
    for path in paths:
        try:
            out.append(
                subprocess.run(
                    ["git", "show", f"{ref}:{path}"],
                    capture_output=True, text=True, check=True,
                ).stdout
            )
        except subprocess.CalledProcessError:
            print(f"warning: {ref}:{path} not readable; skipping", file=sys.stderr)
    return "\n".join(out)


def dms(value: float) -> list[str]:
    """Render a decimal degree value the ways a spec would write it."""
    deg = int(value)
    minutes_f = abs(value - deg) * 60
    minutes = int(minutes_f)
    seconds = (minutes_f - minutes) * 60
    return [
        f"{deg}°{minutes:02d}'{seconds:.2f}",
        f"{deg}°{minutes:02d}'{seconds:.1f}",
        f"{deg}°{minutes:02d}'{round(seconds)}",
        f"{deg} deg {minutes:02d}' {seconds:.2f}",
    ]


def build_patterns(text: str) -> list[str]:
    """Derive literal patterns from the baseline source.

    Only the ayanamsha constants themselves are extracted -- named `*_DEG`
    constants and match-arm values. Sweeping up every float in the file also
    catches polynomial coefficients, tolerances and Julian Days, whose rounded
    forms then collide with ordinary prose (section numbers, version strings)
    and bury real hits in noise. A gate nobody can read is a gate nobody runs.
    """
    literals: set[str] = set()
    candidates: list[str] = []
    candidates += re.findall(r"const\s+[A-Z0-9_]+\s*:\s*f64\s*=\s*([\d_]+\.[\d_]+)", text)
    candidates += re.findall(r"=>\s*([\d_]+\.[\d_]+)\s*,", text)

    for raw in candidates:
        plain = raw.replace("_", "")
        try:
            value = float(plain)
        except ValueError:
            continue
        if not (0.0 < value < 40.0):
            continue
        literals.add(raw)
        literals.add(plain)
        for places in (3, 4, 5, 6):
            rounded = f"{value:.{places}f}".rstrip("0")
            if "." in rounded and len(rounded.split(".")[1]) >= 3:
                literals.add(rounded)
        literals.update(dms(value))

    return [re.escape(lit) for lit in sorted(literals)]


def load_allowlist(path: str) -> list[tuple[str, str]]:
    """Regexes for hits confirmed legitimate, with a reason for each.

    Needed because a primary source and a previously-shipped constant can be the
    same number -- when the shipped value was correct, quoting its primary is
    indistinguishable from leaking it. The gate cannot resolve that; a human can,
    once, in writing.
    """
    allows: list[tuple[str, str]] = []
    try:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                pattern, _, reason = line.partition("|")
                allows.append((pattern.strip(), reason.strip() or "(no reason given)"))
    except FileNotFoundError:
        pass
    return allows


def scan(path: str, patterns: list[str],
         allows: list[tuple[str, str]]) -> list[tuple[int, str, str]]:
    hits = []
    combined = [(p, "value") for p in patterns] + [(p, "phrase") for p in PHRASE_PATTERNS]
    with open(path, encoding="utf-8") as handle:
        for lineno, line in enumerate(handle, 1):
            if any(re.search(a, line) for a, _ in allows):
                continue
            for pattern, kind in combined:
                if re.search(pattern, line, re.IGNORECASE):
                    hits.append((lineno, kind, line.rstrip()))
                    break
    return hits


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("files", nargs="*", help="spec documents to check")
    parser.add_argument("--baseline", default=DEFAULT_BASELINE,
                        help=f"git ref holding the pre-derivation constants (default {DEFAULT_BASELINE})")
    parser.add_argument("--source", action="append", default=None,
                        help="path within the baseline ref to extract constants from")
    parser.add_argument("--allow-file", default=DEFAULT_ALLOWLIST,
                        help="regex|reason lines for hits confirmed legitimate")
    parser.add_argument("--show-pattern-count", action="store_true",
                        help="print how many patterns were derived, and no values")
    args = parser.parse_args()

    sources = args.source or DEFAULT_SOURCES
    patterns = build_patterns(baseline_text(args.baseline, sources))

    if args.show_pattern_count:
        print(f"{len(patterns)} value patterns derived from {args.baseline}:{','.join(sources)}")
        print(f"{len(PHRASE_PATTERNS)} phrase patterns")
        return 0

    if not patterns:
        print(f"error: no patterns derived from {args.baseline}. Refusing to pass vacuously.",
              file=sys.stderr)
        return 1

    if not args.files:
        parser.error("no files given")

    allows = load_allowlist(args.allow_file)
    failed = False
    for path in args.files:
        hits = scan(path, patterns, allows)
        if hits:
            failed = True
            print(f"\n[spec-hygiene] BLOCKED: {path}")
            for lineno, kind, line in hits:
                print(f"  {path}:{lineno}: [{kind}] {line[:120]}")
        else:
            print(f"[spec-hygiene] clean: {path}")

    if failed:
        print("\nA derivation spec must not contain the values it replaces, nor any\n"
              "comparison from which they can be recovered. Move observations to the\n"
              "post-freeze migration note; they are an observation, never a gate.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
