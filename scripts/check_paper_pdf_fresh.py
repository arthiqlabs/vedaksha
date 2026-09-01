#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Assert the committed technical-report PDF matches its committed LaTeX source.

`docs/paper/main.pdf` is committed alongside `docs/paper/main.tex` so a reader
browsing the repo on GitHub can read the paper without a LaTeX toolchain. A
committed build artifact drifts from its source the instant someone edits
`main.tex` without rebuilding and re-committing the PDF -- and this project
has spent real effort eliminating exactly that class of failure everywhere
else it appears (LICENSE copies, MCP tool snapshots, Cargo.lock).

`docs/paper/main.tex.sha256` records the SHA-256 of the `main.tex` that
`main.pdf` was built from. This check re-hashes the local `main.tex` and
fails if it no longer matches -- meaning the source moved since the PDF was
last built, and the committed PDF is now stale.

This is a purely local, no-network check, so it belongs in the per-push gate
tier alongside the other guard scripts.

To refresh after a legitimate edit to main.tex:
    cd docs/paper && make pdf && cp build/main.pdf main.pdf \\
        && shasum -a 256 main.tex > main.tex.sha256

Exit 0 clean, 1 on drift or a missing file.
"""

from __future__ import annotations

import hashlib
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
PAPER_DIR = ROOT / "docs" / "paper"
TEX = PAPER_DIR / "main.tex"
PDF = PAPER_DIR / "main.pdf"
SIDECAR = PAPER_DIR / "main.tex.sha256"


def main() -> int:
    for p in (TEX, PDF, SIDECAR):
        if not p.is_file():
            print(f"error: {p.relative_to(ROOT)} is missing", file=sys.stderr)
            return 1

    sidecar_text = SIDECAR.read_text()
    match = re.match(r"([0-9a-fA-F]{64})\s", sidecar_text)
    if not match:
        print(
            f"error: {SIDECAR.relative_to(ROOT)} does not start with a "
            f"64-hex-char SHA-256 digest -- got: {sidecar_text.strip()!r}",
            file=sys.stderr,
        )
        return 1
    recorded = match.group(1).lower()

    actual = hashlib.sha256(TEX.read_bytes()).hexdigest()

    if actual != recorded:
        print(
            f"::error::{TEX.relative_to(ROOT)} has changed since "
            f"{PDF.relative_to(ROOT)} was last built.\n"
            f"  recorded (sidecar): {recorded}\n"
            f"  actual (main.tex):  {actual}\n"
            f"The committed PDF is stale. Rebuild and re-record:\n"
            f"  cd docs/paper && make pdf && cp build/main.pdf main.pdf "
            f"&& shasum -a 256 main.tex > main.tex.sha256",
            file=sys.stderr,
        )
        return 1

    print(f"paper PDF fresh: {TEX.relative_to(ROOT)} matches {SIDECAR.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
