#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Assert every tracked source file carries an SPDX-License-Identifier.

GitHub cannot detect BUSL-1.1 -- its licence API has no entry for it, which is
why this repository, hashicorp/terraform and cockroachdb/cockroach all report
"Other". The compensating mechanism, and what Terraform (1,704 files) and Vault
(5,376) rely on, is the SPDX short-form identifier in each source header: that
is what scancode, REUSE, licensee and SBOM generators actually read.

This exists because a one-off sweep is not a guarantee. The sweep that added
these tags only touched files that already carried an older licence comment, so
27 files were silently missed -- the whole Python package among them, which is
what ships to PyPI. A new file with no header would drift the same way.

Exit 0 clean, 1 on any file missing the tag.
"""

from __future__ import annotations

import pathlib
import subprocess
import sys

TAG = "SPDX-License-Identifier: BUSL-1.1"
SUFFIXES = {".rs", ".py", ".sh", ".ts", ".tsx", ".mjs", ".js"}
# Generated coefficient blobs get their header from the generators, which are
# themselves checked here; nothing else is exempt.
EXEMPT: set[str] = set()
HEAD_BYTES = 400


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    tracked = subprocess.run(
        ["git", "ls-files"], cwd=root, capture_output=True, text=True, check=True
    ).stdout.split()

    missing = []
    checked = 0
    for rel in tracked:
        if rel in EXEMPT:
            continue
        path = root / rel
        if path.suffix not in SUFFIXES or not path.is_file():
            continue
        checked += 1
        if TAG not in path.read_text(errors="replace")[:HEAD_BYTES]:
            missing.append(rel)

    if missing:
        print(f"error: {len(missing)} of {checked} source files lack '{TAG}'", file=sys.stderr)
        for rel in missing:
            print(f"  {rel}", file=sys.stderr)
        print(
            "\nAdd the tag to the file header, below any shebang and any existing\n"
            "copyright lines. Python: '# SPDX-License-Identifier: BUSL-1.1' may sit\n"
            "above the module docstring, which stays the first statement.",
            file=sys.stderr,
        )
        return 1

    print(f"spdx headers: {checked} source files, all tagged")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
