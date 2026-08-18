# Vedaksha v6.0.1

**No code changed.** This release exists to put a corrected `LICENSE` into the published
artifacts. Behaviour, API surface and every computed value are identical to 6.0.0.

---

## Why a release for a licence file

`LICENSE` is packaged into every crate (`license-file` is set at the workspace root), and
into the npm package. crates.io publishes are immutable, so the text shipped with 6.0.0
cannot be corrected in place — a new version is the only way to distribute the corrected
terms to anyone who installs the code.

If you obtained Vedaksha from PyPI, nothing about your artifact changes: the wheel carries
the SPDX expression `BUSL-1.1`, not the licence text.

## What changed in the licence

**The commercial fee is charged once per organization, and covers every version.** The
previous wording said "one-time per organization" and then "perpetually for that version",
which are two different readings of the same sentence, and the second one is the reading a
licensee would have had to argue against. One purchase now explicitly covers unlimited
products and seats within the organization, perpetually, for the version licensed **and
every version released afterwards**. It is not charged again per release.

**The Change Date is four years, not five.** BSL 1.1 caps it at four: the Terms grant the
Change License on the Change Date "or the fourth anniversary of the first publicly available
distribution ... whichever comes first". With a five-year parameter, the four-year clause
governed and the parameter was dead text that contradicted the rest of the file. Four years
is what a licensee could already rely on, now stated as the parameter.

Neither change reduces any right. The fee clarification resolves an ambiguity in the
licensee's favour, and the Change Date now matches the date that already governed.

## How the corrected text is guaranteed to reach you

Each crate carries its own copy of `LICENSE`, and that copy — not the workspace root — is
what `cargo` archives into the `.crate`. Correcting the root alone changed nothing a
licensee would receive, and every existing check passed while eight copies still carried the
superseded terms. It was caught by unpacking a built `.crate` and reading the text inside.

`scripts/check_license_sync.py` now runs in CI and fails if any copy drifts from the root.
Because crates.io publishes cannot be withdrawn, it runs before the tag rather than after.

## Also in this release

Documentation only, none of it packaged into the crates:

- `MAINTENANCE.md` described upkeep for components this engine does not have — an EOP table,
  a leap-second table, an asteroid element set, and a general star catalogue. The engine
  takes UT1 from the caller, which is what removes the need for the first two; five
  catalogue stars serve the ayanamsha anchors; there is no asteroid surface. Those sections
  now say what is absent and why.
- The shipped kernel is DE440s and, read from its own segment headers, covers **1850–2150**.
  The durability discussion previously cited DE441 and 2400 CE.
- `README.md` reported a test count last measured at 5.0.2, and advertised the graph
  ontology's nine node types where a chart builds four. Both corrected.
- `CONTRIBUTING.md` listed Meeus as an acceptable source four lines above forbidding
  20th-century commercial editions. The prohibition is scoped to classical texts, which is
  what it was written about; the rule against using any book's worked-value table as a
  conformance oracle is separated out and still binds every source.
