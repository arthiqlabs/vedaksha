# Vedaksha v5.0.2 — completes the 5.0.1 release

**Release date:** 2026-08-17

A release-plumbing patch. **Identical in behaviour to v5.0.1** — same
computation, same API, same numbers.

## Why it exists

v5.0.1 published to PyPI, npm and Docker, and stopped partway through
crates.io. `vedaksha-graph` had gained an optional `vedaksha-astro` dependency
(the `from-chart` feature that made the property graph reachable), but the
release workflow still published graph *before* astro. `cargo publish` resolves
each dependency against crates.io rather than the local path, so it failed with
`failed to select a version for the requirement vedaksha-astro = ^5.0.1` — after
`vedaksha-math` and `vedaksha-ephem-core` had already gone out.

crates.io publishes cannot be withdrawn, only superseded, so v5.0.2 is the
complete set: all seven crates at one version, published in dependency order.

**5.0.0 and 5.0.1 are yanked on crates.io.** Nothing is wrong with 5.0.0's
computation, but leaving a partial 5.0.1 resolvable invites a mixed-version
build that nobody tested. Use 5.0.2.

## What changed

- `release.yml` publishes in dependency order: `math` → `ephem-core` → `astro`
  → `graph` → `vedic` → `mcp` → `vedaksha`.
- **`scripts/check_publish_order.py`**, run by CI on every push. It parses the
  publish loop and every crate's `[dependencies]`, and fails if any crate would
  be published before something it needs. `[dev-dependencies]` are excluded
  deliberately — they do not gate publishing, which is why `ephem-core`
  published cleanly despite listing later siblings there.
- The Python binding's version test compares `__version__` against the
  installed distribution's metadata instead of a hardcoded literal, so a
  version bump no longer needs a matching edit in the test — and can no longer
  fail CI after a tag has been cut.

Everything v5.0.1 introduced — `chart_to_graph`, `emit_graph` accepting a
computed chart, the source-comment and `SECURITY.md` corrections, the shorter
README — is in this release. See [CHANGELOG-v5.0.1.md](CHANGELOG-v5.0.1.md).

---

Verified at this commit: per-push 1,024 passed / 0 failed / 5 ignored; full
validation 1,030 passed / 0 failed / 0 ignored. Python binding 20 passed.
clippy `-D warnings` clean, `cargo fmt --check` clean, clean-room check clean.
