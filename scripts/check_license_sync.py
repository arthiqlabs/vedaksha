#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
"""Assert every packaged copy of LICENSE matches the workspace root.

Each crate carries its own LICENSE, and that copy -- not the workspace root --
is what `cargo package` archives into the .crate. Editing the root alone
therefore corrects nothing that a licensee receives.

That is not hypothetical. The 6.0.1 licence fix was applied to the root, passed
every existing check, and was caught only by unpacking a built .crate and
reading the text inside it: eight copies still carried the superseded terms. A
release whose entire purpose was to distribute corrected terms would have
shipped the old ones.

crates.io publishes cannot be withdrawn, so this runs before the tag, not after.

Exit 0 clean, 1 on any drift.
"""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CANONICAL = ROOT / "LICENSE"


def main() -> int:
    if not CANONICAL.is_file():
        print(f"error: {CANONICAL} is missing", file=sys.stderr)
        return 1

    want = CANONICAL.read_bytes()
    copies = sorted(
        p for p in ROOT.rglob("LICENSE")
        if p != CANONICAL and "target" not in p.parts and ".git" not in p.parts
    )

    if not copies:
        print("error: no per-crate LICENSE copies found. Refusing to pass "
              "vacuously -- either the layout changed or this check is looking "
              "in the wrong place.", file=sys.stderr)
        return 1

    drifted = [p for p in copies if p.read_bytes() != want]
    for p in drifted:
        print(f"::error::{p.relative_to(ROOT)} differs from the workspace root "
              f"LICENSE. Copy the root over it; do not edit it in place.")

    if drifted:
        print(f"\n{len(drifted)} of {len(copies)} copies drifted. The .crate "
              f"archives the per-crate copy, so this is what a licensee would "
              f"receive.", file=sys.stderr)
        return 1

    print(f"license in sync: {len(copies)} copies match the workspace root")
    return 0


if __name__ == "__main__":
    sys.exit(main())
