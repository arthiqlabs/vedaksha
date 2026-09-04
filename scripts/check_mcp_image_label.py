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

It also asserts the OCI identifier's image tag matches server.json's own
"version" field. The registry forbids a separate "version" key on an OCI
package (see the `version` check below), so the tag inside the identifier
string is the only place that version lives — and nothing was asserting it.
It was last bumped by hand at v8.0.0 and then not touched through v8.1.0,
v9.0.0 and v9.1.0 — three releases where the MCP Registry's Docker install
instructions silently pointed at a stale image before anyone noticed by hand.

For every non-OCI package (pypi, npm, ...), `version` is an ordinary field
rather than a tag embedded in `identifier`, so it is asserted the same way
directly against server.json's version.
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
        registry_type = package.get("registryType")
        if registry_type != "oci":
            # Every other registryType (pypi, npm, nuget, ...) carries version
            # as its own required field rather than baking it into identifier,
            # so check that directly.
            version = package.get("version")
            if version is not None and version != server["version"]:
                print(
                    f"::error::{registry_type} package '{package.get('identifier')}' "
                    f"declares version '{version}' but server.json version is "
                    f"'{server['version']}'. Bump it alongside every other version "
                    f"place.",
                    file=sys.stderr,
                )
                return 1
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
        identifier = package.get("identifier", "")
        tail = identifier.rsplit("/", 1)[-1]
        if ":" not in tail:
            print(
                f"::error::OCI identifier '{identifier}' carries no "
                f"tag. The registry wants a canonical reference.",
                file=sys.stderr,
            )
            return 1
        tag = tail.rsplit(":", 1)[-1].lstrip("v")
        if tag != server["version"]:
            print(
                f"::error::OCI identifier '{identifier}' is tagged "
                f"'{tag}' but server.json version is '{server['version']}'. "
                f"Bump the tag alongside every other version place.",
                file=sys.stderr,
            )
            return 1

    print(f"ok: image label and OCI package agree with server.json ({name})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
