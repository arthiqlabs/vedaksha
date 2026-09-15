#!/usr/bin/env python3
# SPDX-License-Identifier: BUSL-1.1
"""Assert every intra-workspace dependency requires the CURRENT workspace version.

Locally a sibling crate is always resolved by `path`, so the `version` next to it
is never exercised by a build here. It matters only once the crate is on
crates.io, where it is the entire requirement: `vedaksha-astro = { version =
"9.0.0", ... }` publishes as `^9.0.0`, and cargo will pair that crate with any
9.x sibling a consumer's lockfile already holds.

That shipped in v9.2.0. `vedaksha-astro` 9.2.0 calls
`coordinates::ecliptic_to_equatorial_deg` and `CelestialFrame::true_obliquity`,
both new in `vedaksha-ephem-core` 9.2.0, while still declaring `^9.0.0`. A
consumer holding ephem-core 9.1.x who bumped only astro resolved a combination
that does not compile. Nothing here could see it: the workspace builds through
`path`, and a caret requirement satisfied a minor bump, so the pins were left at
9.0.0 for three releases. A downstream consumer found it.

Requiring exactly the workspace version is stricter than strictly necessary for
a release that adds no cross-crate API, and deliberately so: deciding per release
whether any crate started using a sibling's new item is the judgement that
failed. The cost is that every version bump moves these pins with it.

Checked: `[workspace.dependencies]` in the root manifest, and `dependencies`,
`build-dependencies` and `dev-dependencies` (including `target.*` tables) of
every member that publishes. A member with `publish = false` (the wasm crate,
the Python engine shim) is never resolved against crates.io and is skipped. A
path-only entry with no `version` is allowed in `dev-dependencies` only — cargo
strips those on publish — and is an error in any section that is resolved.

Exit 0 if every pin matches, 1 with a list otherwise.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SECTIONS = ("dependencies", "build-dependencies", "dev-dependencies")


def load(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def dep_tables(manifest: dict):
    """Yield (section_name, table) for every dependency table, target ones too."""
    for section in SECTIONS:
        if section in manifest:
            yield section, manifest[section]
    for target, body in manifest.get("target", {}).items():
        for section in SECTIONS:
            if section in body:
                yield f"target.{target}.{section}", body[section]


def main() -> int:
    root = load(ROOT / "Cargo.toml")
    workspace = root["workspace"]
    version = workspace["package"]["version"]

    members: dict[str, Path] = {}
    published: set[str] = set()
    for member in workspace["members"]:
        manifest_path = ROOT / member / "Cargo.toml"
        package = load(manifest_path)["package"]
        members[package["name"]] = manifest_path
        if package.get("publish", True) is not False:
            published.add(package["name"])

    problems: list[str] = []
    checked = 0

    def check(where: str, section: str, name: str, spec) -> None:
        nonlocal checked
        if name not in members:
            return
        if isinstance(spec, str):
            spec = {"version": spec}
        if spec.get("workspace") is True:
            return  # inherits from [workspace.dependencies], checked there
        checked += 1
        if "version" not in spec:
            if not section.endswith("dev-dependencies"):
                problems.append(
                    f"{where} [{section}] {name}: no version requirement, but this "
                    "section is resolved against crates.io on publish"
                )
            return
        if spec["version"] != version:
            problems.append(
                f"{where} [{section}] {name} = \"{spec['version']}\", "
                f"workspace version is \"{version}\""
            )

    for name, spec in workspace.get("dependencies", {}).items():
        check("Cargo.toml", "workspace.dependencies", name, spec)
    for crate, manifest_path in members.items():
        if crate not in published:
            continue
        rel = manifest_path.relative_to(ROOT)
        for section, table in dep_tables(load(manifest_path)):
            for name, spec in table.items():
                check(str(rel), section, name, spec)

    if checked == 0:
        print("check_internal_dep_versions: found no intra-workspace dependencies "
              "at all — the manifest layout changed and this check is blind")
        return 1
    if problems:
        print(f"Intra-workspace dependency requirements must equal the workspace "
              f"version {version} ({len(problems)} wrong):")
        for p in problems:
            print(f"  {p}")
        return 1
    print(f"OK: {checked} intra-workspace dependency requirements all = {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
