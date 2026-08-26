#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
# Contact: info@arthiq.net | https://vedaksha.net
"""Assert the Docker image can be listed as an OCI package in the MCP Registry.

The registry refuses an OCI package unless the image carries

    LABEL io.modelcontextprotocol.server.name="<server.json name>"

and it checks that at publish time — after crates.io, npm, PyPI, GHCR and the
GitHub Release have already gone out irreversibly. That is exactly how v7.3.0
shipped: nine publish jobs green, the registry job red, and no way to fix the
image without rebuilding a tag that was already published.

None of the three rules that rejected v7.3.0 appear in the registry's own JSON
Schema, which the file validated against cleanly. So this checks the two things
that are checkable here: that the label exists, and that it agrees with
server.json.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LABEL = "io.modelcontextprotocol.server.name"


def main() -> int:
    server = json.loads((ROOT / "server.json").read_text())
    name = server["name"]
    packages = server.get("packages") or []
    dockerfile = (ROOT / "Dockerfile").read_text()

    match = re.search(rf'^LABEL\s+{re.escape(LABEL)}="([^"]+)"', dockerfile, re.MULTILINE)

    if not any(p.get("registryType") == "oci" for p in packages):
        # No OCI package declared, so the registry never inspects the image.
        print("ok: server.json declares no OCI package; image label not required")
        return 0

    if match is None:
        print(
            f"::error::server.json declares an OCI package, but Dockerfile has no "
            f'LABEL {LABEL}="{name}". The MCP Registry will reject the publish '
            f"AFTER every other surface has published.",
            file=sys.stderr,
        )
        return 1

    if match.group(1) != name:
        print(
            f"::error::Dockerfile labels the image '{match.group(1)}' but "
            f"server.json names the server '{name}'. The registry compares them.",
            file=sys.stderr,
        )
        return 1

    for package in packages:
        if package.get("registryType") != "oci":
            continue
        # Both rules the registry enforced against v7.3.0, and neither is in
        # its published schema — `version` is in fact REQUIRED there.
        for forbidden in ("registryBaseUrl", "version"):
            if forbidden in package:
                print(
                    f"::error::OCI package carries '{forbidden}'. The registry "
                    f"rejects it and wants the version inside 'identifier' "
                    f"instead, e.g. ghcr.io/owner/image:1.2.3",
                    file=sys.stderr,
                )
                return 1
        if ":" not in package.get("identifier", "").rsplit("/", 1)[-1]:
            print(
                f"::error::OCI identifier '{package.get('identifier')}' carries no "
                f"tag. The registry wants a canonical reference.",
                file=sys.stderr,
            )
            return 1

    print(f"ok: image label and OCI package agree with server.json ({name})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
