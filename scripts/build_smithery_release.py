#!/usr/bin/env python3
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedaksha — Vision from Vedas
# SPDX-License-Identifier: BUSL-1.1
"""Lay out the Smithery stdio release: an MCPB bundle directory and its payload.

Smithery lists Vedaksha as a local stdio server, distributed as an MCPB bundle
holding the two release binaries. Its CLI cannot publish such a bundle: the
release payload's serverCard needs `inputSchema` on every tool, and the MCPB
manifest schema rejects that key, so a card built from the manifest is always
refused. The release is therefore a direct `PUT /servers/<name>/releases` with
a payload carrying the full tool list, which this script writes next to the
bundle directory for `mcpb pack` and the upload to consume.

Both come from the committed `tools/mcp-tools.json` at the tag, which the drift
guard holds equal to what the binary answers to `tools/list`, so the card
Smithery shows is the surface the binary serves.

Usage: build_smithery_release.py VERSION BIN_DIR OUT_DIR
  BIN_DIR holds vedaksha-mcp-x86_64-unknown-linux-gnu and
  vedaksha-mcp-aarch64-apple-darwin, as attached to the GitHub Release.
  Writes OUT_DIR/bundle/ (manifest.json + server/) and OUT_DIR/payload.json.

Exit 0 on success, 1 on any mismatch between VERSION and the inputs.
"""

from __future__ import annotations

import json
import pathlib
import shutil
import stat
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Release asset name -> name inside the bundle. Only these two targets are
# built by release.yml, so the bundle covers Linux x86_64 and Apple silicon.
BINARIES = {
    "vedaksha-mcp-x86_64-unknown-linux-gnu": "vedaksha-mcp-linux-x64",
    "vedaksha-mcp-aarch64-apple-darwin": "vedaksha-mcp-darwin-arm64",
}

# The keys Smithery's ServerCard tool schema allows (additionalProperties: false).
CARD_TOOL_KEYS = (
    "name", "title", "icons", "description", "inputSchema",
    "outputSchema", "annotations", "execution", "_meta",
)


def main(argv: list[str]) -> int:
    if len(argv) != 4:
        print(__doc__, file=sys.stderr)
        return 1
    version, bin_dir, out_dir = argv[1], pathlib.Path(argv[2]), pathlib.Path(argv[3])

    snapshot = json.loads((ROOT / "tools" / "mcp-tools.json").read_text())
    if snapshot["engineVersion"] != version:
        print(f"tools/mcp-tools.json is {snapshot['engineVersion']}, not {version}", file=sys.stderr)
        return 1
    server = json.loads((ROOT / "server.json").read_text())
    if server["version"] != version:
        print(f"server.json is {server['version']}, not {version}", file=sys.stderr)
        return 1
    tools = snapshot["tools"]

    bundle = out_dir / "bundle"
    (bundle / "server").mkdir(parents=True, exist_ok=False)
    for asset, inner in BINARIES.items():
        dest = bundle / "server" / inner
        shutil.copyfile(bin_dir / asset, dest)
        dest.chmod(dest.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    linux = "${__dirname}/server/" + BINARIES["vedaksha-mcp-x86_64-unknown-linux-gnu"]
    darwin = "${__dirname}/server/" + BINARIES["vedaksha-mcp-aarch64-apple-darwin"]
    manifest = {
        "manifest_version": "0.3",
        "name": "vedaksha",
        "display_name": "Vedaksha",
        "version": version,
        "description": server["description"],
        "author": {"name": "ArthIQ Labs", "email": "info@arthiq.net", "url": "https://arthiq.net"},
        "homepage": server["websiteUrl"],
        "repository": {"type": "git", "url": server["repository"]["url"]},
        "license": "BUSL-1.1",
        "server": {
            "type": "binary",
            "entry_point": "server/" + BINARIES["vedaksha-mcp-x86_64-unknown-linux-gnu"],
            "mcp_config": {
                "command": linux,
                "args": ["--stdio"],
                "platform_overrides": {"darwin": {"command": darwin, "args": ["--stdio"]}},
            },
        },
        "compatibility": {"platforms": ["darwin", "linux"]},
        # The MCPB manifest takes name and description only; the schemas go in
        # the payload's serverCard below.
        "tools": [{"name": t["name"], "description": t["description"]} for t in tools],
    }
    (bundle / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    payload = {
        "type": "stdio",
        "runtime": "binary",
        "serverCard": {
            "serverInfo": {"name": "vedaksha-mcp", "version": version},
            "tools": [{k: t[k] for k in CARD_TOOL_KEYS if k in t} for t in tools],
        },
    }
    (out_dir / "payload.json").write_text(json.dumps(payload) + "\n")
    print(f"bundle for {version}: {len(tools)} tools, {len(BINARIES)} binaries")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
