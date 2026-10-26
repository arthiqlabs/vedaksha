# v10.0.1

Patch release. No engine change: every computed value is identical to
v10.0.0. This release carries registry metadata and a new MCP Registry name.

## Changed

- The official MCP Registry entry is now
  `io.github.arthiqlabs/vedaksha-vedic-astrology`. Registry search matches
  server names only, so the old name `io.github.arthiqlabs/vedaksha` was not
  found by a search for "vedic" or "astrology". The new name still contains
  "vedaksha". The Docker image's `io.modelcontextprotocol.server.name` label
  carries the new name. The old entry stays listed at 10.0.0 and is marked
  deprecated in favour of the new one.
- PyPI: the package lists Documentation, Changelog and Issues links, and no
  longer carries the `License :: Other/Proprietary License` classifier,
  because its licence is declared as `BUSL-1.1`.
- crates.io: `vedaksha` and `vedaksha-vedic` are listed under the
  `date-and-time` category, and `vedaksha` uses the keyword
  `vedic-astrology` in place of `vedic`.
- The LICENSE link at the end of each crate README and the Python README is
  reference-style, so viewers that auto-link raw URLs no longer take the
  trailing `).` into the link.

## Added

- `skills/vedaksha/SKILL.md`, an Agent Skill that tells an agent how to call
  the engine and which inputs cost accuracy when wrong.

## Compatibility

Identical behaviour to v10.0.0. MCP clients start the server from the Docker
image or the PyPI package, not from the registry name, so existing client
configurations keep working.
