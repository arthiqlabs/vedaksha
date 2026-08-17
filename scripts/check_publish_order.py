#!/usr/bin/env python3
"""Assert release.yml publishes workspace crates in dependency order.

`cargo publish` resolves each dependency against what is already ON crates.io.
A path dependency is not enough: if crate A depends on crate B and A is
published first, cargo fails with "failed to select a version for the
requirement B = ^x.y.z" — after everything before A has already gone out, and
crates.io publishes cannot be withdrawn.

That happened in v5.0.1. `vedaksha-graph` gained an optional `vedaksha-astro`
dependency while still sitting before astro in the publish list, so the release
died halfway: math and ephem-core shipped at the new version while the other
five stayed behind.

Nothing checked the ordering, so this does. Run by CI on every push.

Exit 0 if the order is sound, 1 with an explanation otherwise.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"

# Only these sections gate publishing. dev-dependencies do not: cargo does not
# require them to resolve on crates.io, which is why a crate can list a
# later-published sibling under [dev-dependencies] and still publish fine.
GATING_SECTIONS = {"dependencies", "build-dependencies"}


def publish_order() -> list[str]:
    """The crate list from release.yml's publish loop."""
    text = RELEASE_WORKFLOW.read_text()
    match = re.search(r"for crate in ([^;]+); do", text)
    if not match:
        sys.exit("could not find the publish loop in release.yml")
    return match.group(1).split()


def gating_deps(crate: str) -> list[str]:
    """Workspace siblings `crate` needs on crates.io before it can publish."""
    manifest = ROOT / "crates" / crate / "Cargo.toml"
    deps: list[str] = []
    section: str | None = None
    for line in manifest.read_text().splitlines():
        header = re.match(r"^\[([^\]]+)\]", line)
        if header:
            section = header.group(1)
            continue
        if section in GATING_SECTIONS:
            dep = re.match(r"^(vedaksha[a-z-]*)\s*=", line)
            if dep:
                deps.append(dep.group(1))
    return deps


def main() -> int:
    order = publish_order()
    position = {crate: i for i, crate in enumerate(order)}

    problems: list[str] = []
    for crate in order:
        for dep in gating_deps(crate):
            if dep not in position:
                problems.append(
                    f"{crate} depends on {dep}, which release.yml never publishes"
                )
            elif position[dep] > position[crate]:
                problems.append(
                    f"{crate} (position {position[crate] + 1}) depends on {dep} "
                    f"(position {position[dep] + 1}) — the dependency publishes later"
                )

    if problems:
        print("release.yml publish order is unsound:\n", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        print(
            "\nReorder the `for crate in ...` loop in .github/workflows/release.yml "
            "so every crate follows everything it depends on.",
            file=sys.stderr,
        )
        return 1

    print(f"publish order is sound ({len(order)} crates):")
    for i, crate in enumerate(order, 1):
        deps = ", ".join(gating_deps(crate)) or "—"
        print(f"  {i}. {crate:<22} needs: {deps}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
