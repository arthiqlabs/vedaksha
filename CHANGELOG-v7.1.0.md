# Vedaksha v7.1.0

**No computed value changes.** Every position, cusp, dasha and panchanga limb is identical to
7.0.0. This release adds a transport flag, publishes the server to the official MCP Registry, and
restores licence text we had been shipping incomplete.

---

## `--stdio`

`vedaksha-mcp` now accepts `--stdio` to select the stdio transport explicitly. It was previously
selected by the *absence* of `--http`, which is fine when a human types the command and useless
when something else assembles it — a container `CMD`, an MCP client's launch config, a registry
entry. Overriding a default `CMD` requires an argument to override it *with*, and there wasn't
one, so `ghcr.io/arthiqlabs/vedaksha-mcp` (whose `CMD` is `["--http"]`) could not be run over
stdio without also overriding the entrypoint.

Behaviour is unchanged in every existing invocation:

| invocation | transport |
|---|---|
| `vedaksha-mcp` | stdio (as before) |
| `vedaksha-mcp --stdio` | stdio |
| `vedaksha-mcp --http` | HTTP (as before) |
| `vedaksha-mcp --http --stdio` | exits 2 |

Passing both is a conflict rather than a silent preference: on an image whose `CMD` is
`["--http"]`, silently preferring one would make `docker run … --stdio` look like it worked while
serving HTTP.

Verified against the binary, not only the unit tests — `--stdio` and no arguments both answer
`tools/list` with all 17 tools, and their output is byte-identical, so the flag selects the
existing path rather than a parallel one.

## Published to the official MCP Registry

`server.json` at the repo root registers the server as `io.github.arthiqlabs/vedaksha` on
[registry.modelcontextprotocol.io](https://registry.modelcontextprotocol.io), the registry MCP
clients query for discovery. Publishing runs from the release workflow authenticated by GitHub
OIDC, so there is no token to store or rotate, and namespace ownership is proved by the workflow
itself.

`server.json`'s version is asserted by the existing `version-check` job rather than rewritten at
publish time, so the committed file cannot drift from the tag. That makes it the **sixth** place
a version bump touches.

The entry carries no `packages` block yet. The obvious candidate is the GHCR image, and `--stdio`
is what makes a stdio package entry expressible — but the invocation has to be run before it is
published to a registry that machines read, and that verification is still outstanding.

## The licence text was incomplete

`LICENSE` omitted the final ~1,100 characters of canonical BUSL-1.1: the MariaDB
trademark-permission paragraph and the entire **Covenants of Licensor** section. Everything up to
the warranty disclaimer matched and the file simply stopped there.

This was not cosmetic. MariaDB grants use of the licence text and the "Business Source License"
trademark *"as long as you comply with the Covenants of Licensor below"*, and our file carried the
trademark notice while omitting the covenants that permission is conditioned on. Covenant 4 is
"Not to modify this License in any other way", so the truncation was itself a modification.

Restored verbatim from the SPDX text; the body now matches canonical BUSL-1.1 after whitespace and
quote normalisation. Applied to all nine copies, because each crate carries its own and that copy
is what `cargo package` archives. The covenants were checked against what we ship before being
restored, since including them asserts compliance: Apache 2.0 as the Change License is
GPLv3-compatible, the Additional Use Grant restricts nothing the licence already grants, and a
Change Date is specified.

**Verified on the artifact:** `vedaksha-7.0.0.crate` downloaded from crates.io contains the
truncated text. 7.0.0 shipped without these sections; 7.1.0 is the first release carrying them.

One thing this does *not* fix: GitHub still reports the repository licence as "Other". Its licence
API has no BUSL entry at all — `bsl-1.0` in that list is the Boost Software License — so
`hashicorp/terraform`, `cockroachdb/cockroach` and every other Business Source project reads the
same way. Nothing in the file changes that.

## Repository and documentation

A code of conduct, issue forms and a pull-request template, taking GitHub's community profile from
71% to 100%. The accuracy-report form states the clean-room rule up front — no values pasted from
another implementation's output — and reminds reporters that public Julian Days are UT1, which is
the commonest source of a result that looks wrong and isn't.

`keywords` and `categories` added to `vedaksha-math`, `-astro`, `-vedic`, `-graph` and `-wasm`,
which had none and so could not be found by crates.io search. PyPI keywords widened and its
classifiers corrected to the Python versions CI actually tests. README gains real badges, a
section index and an architecture diagram.

## Validation

1,069 tests passed, 0 failed, 0 ignored in release with `--include-ignored` and
`VEDAKSHA_REQUIRE_FIXTURES=1`; per-push 1,063 passed with 5 ignored. Both re-measured at this
tree rather than carried over: the five added are the transport tests, and the two figures
reconcile as 1069 = 1063 + 5 ignored + 1 release-only oracle. Clippy at `-D warnings`,
format check, and the four generator drift checks all green. The Python parity fixture is
unchanged, which is the expected result for a release that moves no computed value.
