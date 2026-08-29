# v8.0.0

## `compute_dasha`: Chara and Narayana dasha's `lagna_sign` was off by one sign — live since v2.4.0

**Severity: high. This is a real, long-lived correctness defect on a documented, live API
surface, not a naming or documentation change.** `compute_dasha`'s MCP schema documents
`lagna_sign` as 1-indexed ("Lagna (ascendant) sign 1–12, 1 = Aries"). The underlying
`vedaksha_vedic::dasha::chara::compute_chara` and `dasha::narayana::compute_narayana` functions
are both 0-indexed (0 = Aries) — correctly documented as such, and internally self-consistent
with their own test suites. The MCP dispatch layer (`compute_dasha`'s `call_compute_dasha`)
passed the 1-indexed input straight through with no conversion. Every caller who correctly
followed the documented schema received a Chara or Narayana dasha sequence starting exactly one
sign later than their true ascendant.

This was introduced when the Chara/Narayana dispatch was first added, in **v2.4.0**, and has
been present in every tagged release since — this is not a regression from any recent work in
this release; it is a defect this release happens to be the first to find and fix. Fixed by
converting the 1-indexed input to 0-indexed at the MCP boundary before dispatch, with a
regression test asserting the exact first-period sign for both Aries and Pisces boundary cases,
on both Chara and Narayana. No change to `chara.rs`/`narayana.rs` themselves, which were already
correct — the defect was entirely in the MCP dispatch layer.

## Chara Dasha: counting direction implemented (previously disclosed as incomplete)

`compute_chara` always counted forward through the 12 signs regardless of the lagna sign — its
own module doc already disclosed this as a known simplification ("full Jaimini treatment...not
yet implemented"), not a silent defect. This release completes it, from dedicated primary-source
research (Jaimini Sutras, no reference-engine consultation):

- **Base rule (Jaimini Sutras 1.1.25-26, high confidence):** odd lagna signs count forward, even
  signs count backward.
- **Fixed-sign exception (Jaimini Sutras 1.1.27, moderate confidence):** the sutra itself is a
  terse, two-word exception clause with no explicit list; the commentarial reconstruction —
  corroborated by two independently-styled sources — is that the four fixed (Sthira) signs invert
  the plain rule: Taurus and Scorpio (even) count forward; Leo and Aquarius (odd) count backward.
- **Explicitly not implemented:** a different, popular secondary-source mnemonic ("movable
  forward, fixed backward, dual forward-then-backward") was considered and rejected — it is
  internally inconsistent with sutras 25-26 for movable-even signs (e.g. Cancer). See
  `crates/vedaksha-vedic/src/dasha/chara.rs`'s module doc and `DATA_PROVENANCE.md` Fix 11 for the
  full citation and confidence discussion.

**This changes computed Chara Dasha output for every lagna sign except Aries, Gemini, Libra, and
Sagittarius** — the four odd, non-fixed signs, for which forward was already correct. This is a
feature completion of a previously-disclosed gap, not a silent behavior change, but the scale of
affected output (most charts) should not be understated.

## Naming symmetry for mean vs. true ayanamsha — breaking API rename

v7.6.0 added `true_ayanamsha_value`/`true_tropical_to_sidereal` (mean ayanamsha plus
nutation-in-longitude) alongside the existing `ayanamsha_value`/`tropical_to_sidereal`
(mean-only). That left an asymmetric pair: the "true" functions say what they are, but the
mean-only ones don't say anything — nothing in `ayanamsha_value`'s name signals it's mean-only,
which is exactly the kind of unlabeled-default ambiguity this project's own module
documentation warns about elsewhere ("an engine that is silent about which one it returns gets
compared against the wrong column"). This release makes the pairing fully symmetric, no
unlabeled default on either side.

**Renamed — published Rust API (`vedaksha-astro`, crates.io):**
- `sidereal::ayanamsha_value` → `sidereal::mean_ayanamsha_value`
- `sidereal::tropical_to_sidereal` → `sidereal::mean_tropical_to_sidereal`
- `sidereal::sidereal_to_tropical` → `sidereal::mean_sidereal_to_tropical`

`true_ayanamsha_value` and `true_tropical_to_sidereal` keep their existing names — they were
already correctly labeled. No function body, computed value, or test assertion value changed
anywhere; this is a pure identifier rename, independently verified against the diff and by a
full workspace test run before and after.

**Renamed — published wasm/npm API (`vedaksha-wasm`):**
- `tropical_to_sidereal` → `mean_tropical_to_sidereal`
- `get_ayanamsha` → `get_mean_ayanamsha`

Any direct caller of these three Rust functions or two wasm/JS exports must update to the new
names. Every other public function, every MCP tool name, and every MCP tool's input parameter
names (including the `ayanamsha` parameter that selects a sidereal *system*, e.g.
`"IndianOfficial"` — a different concept from mean-vs-true, and not touched by this release) are
unchanged.

## Served JSON field renamed: `ayanamsha_value` → `true_ayanamsha_value`

`compute_natal_chart` and `compute_vargas` (both the MCP and wasm surfaces) report an
`ayanamsha_value` field that — since v7.6.0 — actually reports the **true** ayanamsha (the exact
rotation applied to the chart). That created the same kind of ambiguity as above one layer up:
the served field shared a name with the library's `ayanamsha_value()` function, but the two mean
different things. The field is renamed to `true_ayanamsha_value` to match the quantity it
actually reports and the function that computes it. The MCP and wasm output schemas, the
`ayanamsha` input parameter's description text (which references this field by name), and the
Python binding's docstring are all updated to match. No computed value changed — this is a pure
key rename, independently verified (including by actually building the wasm engine and running
the Python conformance suite end-to-end, not just a static diff read).

## Chara-karaka module: dedicated sources record added

`docs/audit/2026-08-29-karaka-sources.md` records what was consulted for the citation rewritten
in v7.6.0 (Jaimini's own sutra text for the karaka-ranking chain and Rahu's textual inclusion,
Parashara's *Brihat Parashara Hora Shastra* for the specific "30 minus degree" arithmetic
Jaimini's own text leaves implicit) — the module had no dedicated derivation record before this,
unlike the ayanamsha and lunar-theory subsystems. No ranking behavior changed.

**Disclosed here, not previously stated anywhere:** `compute_karakas` performs no rotation
itself and is unaffected by v7.6.0's true-ayanamsha fix directly — but its typical caller chains
`compute_natal_chart`'s (now-corrected) sidereal output straight into it. For a chart where
Rahu's degree-within-sign sits within about the same ~17-20 arcsec margin of a sign boundary,
that correction can move it across the boundary and change its karaka rank. This is the intended,
correct effect of v7.6.0's fix reaching a typical caller more accurately — not a new defect in
`compute_karakas`, and not something either release's changelog stated on its own until now. See
`DATA_PROVENANCE.md`'s Fix 9/Fix 10 cross-reference note for the full explanation.

## Release-policy note

This release bundles two independent kinds of change, found and fixed together but distinct in
nature:

1. **Breaking API renames** (five Rust/wasm function names, one served JSON field name) — the
   reason for the major version bump. No computed value changes as part of the rename work
   itself; every value-level behavior change from the ayanamsha/nutation fix already shipped in
   v7.6.0.
2. **Real correctness fixes to `compute_dasha`'s Chara and Narayana systems** — a severity-high,
   multi-year-live off-by-one bug, and a feature completion of a previously-disclosed gap in
   Chara Dasha's direction logic. These change computed dasha output for real callers on a live
   API surface, independent of and unrelated to the ayanamsha renames above.

An earlier draft of this changelog described this release as "naming-only, no computed value
changes" — that was written before the dasha fixes above were found during this same release's
final review cycle, and was corrected here rather than left standing now that it is no longer
true. This release ships as a **major** version bump for the API renames; the dasha fixes are
bundled into the same release rather than split out, since both were found and fixed in the same
work session before the previous major-bump work was tagged.
