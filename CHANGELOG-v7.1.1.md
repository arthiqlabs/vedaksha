# Vedaksha v7.1.1

**Licence identification only.** No code changes, no computed value changes, no API changes. This
release exists so the packaged sources carry the same licence identifier the repository does.

---

## Why a patch release for comments

7.1.0 shipped with `SPDX-License-Identifier` tags in the repository but not in the published
crates: the sweep landed after the tag. Anything scanning the repository saw the tags; anything
scanning the packaged sources — a vendored dependency, an SBOM built from a `.crate`, a
file-level licence scan of `~/.cargo/registry` — still read `// Licensed under BSL 1.1`.

That is not a wrong statement, since it names the same licence, but it is an inconsistent one,
and licence identification is a place where a machine reading two different strings for one
licence is the whole problem.

## What changed

Every `.rs` and `.py` source header now carries the SPDX short-form identifier:

```rust
// SPDX-License-Identifier: BUSL-1.1
```

replacing `// Licensed under BSL 1.1. See LICENSE file.` in 148 files. The repository previously
had **zero** SPDX tags, so per-file licensing was invisible to scancode, REUSE, licensee and SBOM
generators, all of which read exactly this tag.

`BUSL-1.1` is the SPDX identifier. "BSL 1.1" is MariaDB's human shorthand and carries no machine
meaning; the `BUSL` prefix exists because `BSL-1.0` is the unrelated Boost Software License.

Also corrected: `ontology/vedaksha-ontology.json` declared `"license": "BSL-1.1"`, an identifier
that does not exist in SPDX. It now reads `BUSL-1.1`.

The two coefficient generators emit that header into generated files, so they were updated in the
same pass — otherwise the next regeneration would revert roughly 130 of the tags.

## What deliberately did not change

- **The LICENSE files.** Their title is the canonical "Business Source License 1.1" and covenant 4
  of the licence forbids modifying the text. The `license` field in every manifest has read
  `BUSL-1.1` since well before this release.
- **`deny.toml`'s `BSL-1.0` entry**, which is Boost and correct as it stands.
- **Historical changelogs**, which are dated records of what was said at the time.

## Terms, unchanged and restated for the avoidance of doubt

- **Business Source License 1.1**, SPDX `BUSL-1.1`.
- **Non-commercial use is free** — personal, research, education, internal tools.
- **Commercial use requires a licence**: USD $500, charged once per organization, covering
  unlimited products and seats, perpetually, and every version released afterwards.
- **Converts to Apache 2.0 four years after each version's release**, per version. Four is the
  licence's own ceiling, not a choice: BSL 1.1 grants the Change License on the Change Date "or
  the fourth anniversary of the first publicly available distribution … whichever comes first".
  For 7.1.1, released 2026-08-22, the Change Date is **2030-08-22**.

## Validation

Scoped to the change rather than run in full, because nothing here can move a number: the only
diff in 147 `.rs` files is the header comment, and filtering the header lines out of the diff
leaves zero remaining lines.

Workspace build, `cargo test --workspace --release`, clippy at `-D warnings`, format check, the
six version places, publish-order, and the Python parity fixture — which is byte-identical, as it
must be. The DE440s-gated oracle comparisons and the network generator drift checks were not
re-run; they were green at 7.1.0 and no input to them changed.
