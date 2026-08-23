# Vedaksha v7.2.0

**Shadbala's positional component was one-fifth of itself, and said otherwise.** Plus a rotation
that was not quite a rotation, a release-only guard that was never called, and the lint gate that
would have shown both.

Values change on `compute_shadbala`. Nothing else moves by more than 1e-8 arcsec.

---

## Sthana Bala was Uchcha Bala under another name

`sthana_bala(planet, sign, longitude)` returned the exaltation term alone and discarded the
`sign` the MCP tool required from every caller. The tool advertised "full six-fold Shadbala",
the struct field was documented "own/exalted/friend sign", and `Shadbala.sthana_bala` and
`Shadbala.uccha_bala` carried the same number in every response ever served.

Four of the five sub-components BPHS Ch. 27 defines are now computed, each as a function of its
own with its defining rule in the doc comment:

| | rule | range |
|---|---|---|
| `uccha_bala` | `(180 - arc) / 3` from the exaltation degree | 0-60 |
| `ojhayugma_bala` | Moon and Venus want even signs, the other five odd; tested on rasi and navamsa | 0-30 |
| `kendradi_bala` | kendra 60, panapara 30, apoklima 15 | 15-60 |
| `drekkana_bala` | male grahas in the first third of the sign, neuter the second, female the third | 0 or 15 |

`Shadbala` reports all four separately, so a caller can see the composition instead of one
opaque number. Two placements that differ only in house no longer score identically.

**Saptavargaja Bala, the fifth, is still not computed.** It needs a Moolatrikona degree table and
a panchadha maitri derivation that this project does not hold from a primary source, and the
citation rule here forbids taking either from a modern edition's worked table. The gap is now
stated in the module docs, the README and the tool description rather than contradicted by them.
Kala Bala is partial for the same reason and is disclosed the same way: Nathonnatha and Paksha
only, no Tribhaga, Varsha/Masa/Vara/Hora, Ayana or Yuddha Bala.

### A second defect the fix would have introduced

`ishta_kashta_phala` was being handed the Sthana composite. That was harmless only while Sthana
and Uchcha were the same number; making Sthana real drives `uchcha_rasmi` to 12.4 and clamps
`ishta_phala` to its ceiling of 60. BPHS Ch. 28 v. 5 builds the Rasmis from Uchcha Bala, and it
now receives it. Caught by planting the defect and finding no test failed, then adding one that
does.

### Also removed

Eight private dignity helpers — own-sign, exaltation, debilitation and friendship tables — that
were written for Saptavargaja Bala, never called, never tested, and whose friendship match arm
was asymmetric and incomplete. And five `*Output` structs in `vedaksha-mcp` that described
response shapes the server does not emit: `ComputeDashaOutput` claimed `compute_dasha` returns
`{"dasha_json": ...}`, which it never has.

## The ecliptic-to-equatorial rotation was not orthogonal

`COS_EPS` sat 5.74e-12 — 26,083 ulps — above `cos(OBLIQUITY_J2000)`, leaving
`COS_EPS^2 + SIN_EPS^2` a part in 1e11 from unity. Corrected, and both constants are now checked
against the obliquity they claim, and the transform against length preservation.

The obliquity itself did **not** change. `COS_EPS`/`SIN_EPS` always encoded 84381.448 arcsec, the
IAU 1976 value, which is the frame VSOP87A and ELP/MPP02 are defined against; the
`OBLIQUITY_J2000` constant beside them read 84381.406 (IAU 2006), was dead code behind an
`#[allow]`, and was wrong. Rotating with the IAU 2006 value would have introduced a 0.042 arcsec
frame error against the theory being evaluated, so the documentation moved to the values rather
than the reverse.

`analytical_bit_digest` moved and was re-pinned. Measured across all 21,915 rows: every row
changed, worst delta 1.64e-10, **worst angular change 1.02e-8 arcsec** — eight orders of magnitude
inside the analytical path's own 0.239 arcsec mean error against Horizons.

## `emit_graph` could not keep a fabricated observer out of the graph

The tool requires `latitude` and `longitude`, and its own comment says why: "a chart
result records neither the observer nor, reliably, the instant", so defaulting them to zero
"would put a fabricated observer into the emitted graph". It then range-checked neither.
`latitude: 9999, longitude: -1e9` reached the Chart node unaltered — every other tool in the
surface rejects those.

The instant was worse. `julian_day` was read with `unwrap_or(f64::NAN)`, and serde_json renders
a non-finite float as `null`, so a `chart_json` without one produced `"julian_day": null` in the
Chart node and no error at all. Both are now rejected, with a test for each and one that the
valid case still passes.

## The Cypher and SurrealQL emitters escaped one character out of several

`emit_graph` accepts a caller-supplied `ChartGraph`, so node names and ids are not all ours. Both
emitters escaped the single quote and nothing else, and escaping is not composable that way:
escaping introduces backslashes, so an unescaped backslash in the input either changed the value
(`\r` arrived as a carriage return) or, at the end of a string, consumed the closing quote and
let the remainder run on as statement text. A raw newline split the statement outright.

Both now escape the backslash first, then the quote, then the C0 controls that have a literal
spelling, and drop the ones that do not.

`sanitize_surreal_id` was the sharper case: SurrealDB spells a complex record ID `⟨...⟩`, and the
function — named "sanitize", documented as "Replaces characters that need escaping" — replaced
nothing whatsoever, so an id containing `⟩` closed the record ID early. It now strips the
brackets and controls.

Four tests, each verified to fail against the v7.1.1 escaping and pass against this one. The
property one asserts that no input can leave an unescaped quote in the output, rather than
checking a list of payloads somebody thought of.

## `compute_gochara` emitted a field it could never fill

`ashtakavarga_score` appeared as `null` on every entry of every response. `compute_gochara`
hardcodes `None`, and no MCP input could change that: Bhinna Ashtakavarga is built from the
**natal** chart, which the tool is not given. It is now skipped when absent rather than
serialised as `null`, and the field's documentation says who is expected to fill it. Populating
it would mean taking a whole natal chart as input, which is a feature and not a fix.

## Ketu is now a body, in Rahu's frame

There was no south-node variant of [`Body`], so Ketu never went through the pipeline. The only
Ketu on offer was `nodes::south_node_*`, which is the theory layer and carries no nutation —
pairing it with a `Body`-routed Rahu left the two up to 17.2 arcsec off 180 degrees apart. None of
the three functions was called by anything, so nothing noticed.

`Body::MeanSouthNode`, `Body::TrueSouthNode` and `Body::TrueSouthNodeOsculating` now resolve
through `nodes::node_longitude` exactly as their north counterparts do. Rahu and Ketu therefore
share one frame and one nutation term, and are exactly opposite at 1e-9 degrees across four
epochs and all three methods. This is the same fix v7.0.0 applied to the north nodes, one layer
out: put the concept behind the enum so there is only one frame it can be in.

**This adds variants to a public enum**, so external code that matches `Body` exhaustively will
need new arms. It ships in a minor release by explicit decision rather than oversight — no known
consumer is on `^7`, and the alternative was leaving Ketu in a second frame indefinitely.

`light_time::light_time_correction` is the other unwired path and is **documented, not
restructured**. It duplicates the light-time iteration `coordinates.rs` performs internally in
the planetary-aberration form the pipeline needs; nothing calls the public one, so the Horizons
oracle does not cover it. The module now says so, which is what stops a change there being
mistaken for a change to a served value.

## `Nakshatra::guna` carried a citation naming neither a chapter nor a rule

"Source: BPHS standard assignment" is precisely what this project's citation rule forbids —
[`Self::gana`] and [`Self::nadi`] beside it carry BPHS Ch. 20. The accessor was also the only one
in the module with no reference anywhere in the tree: no caller, no test.

The citation now says plainly that this one table is untraced, and a test pins the property that
can be checked without a source — that the 27 nakshatras partition nine ways to each guna. That
catches an edit losing or duplicating one; it does not make any individual assignment right, and
the doc says that too.

## Every public `Result` and panicking FFI entry point now documents it

`missing_errors_doc` and `missing_panics_doc` were **allowed** in the first cut of this release's
lint table, because 22 sites in `vedaksha-wasm` and `bindings/python/engine` had no such sections
— those two crates carried no pedantic lints at all until the table existed. They are now
**enforced**: sixteen `# Errors` sections on the wasm surface, each naming what that function
actually rejects rather than repeating a formula, and six `# Panics` sections on the FFI
boundary, all of which panic on exactly one thing — a poisoned state mutex.

## Answering an external parity report

graha-lab measured vedaksha 7.0.0 against two reference ephemerides and raised three items. Two
are closed by our own documentation: `TrueChitra` is the star's **mean geometric place of date**,
which `sidereal.rs` has stated — including the "roughly 20 arcseconds of aberration plus 17 of
nutation" they went on to measure — since v6.0.0; and `TrueNode`'s 185.61 arcsec residual is
inside the ~0.09 degree bound `bodies.rs` publishes for a 5-term series.

The third reproduces. On the **analytical** path only, the outer bodies carry more error than the
inner ones, and the README now publishes the per-body table rather than only the 0.239 arcsec
mean that hid it. Their proposed cause — the 2:5 Jupiter-Saturn great inequality — is not what
the data shows: the excess grows monotonically outward (Jupiter 1.33x, Saturn 1.39x, Uranus
1.48x, Neptune 2.79x the inner-body floor), which no resonance term in that pair can produce.
Their instrument stopped at Saturn, so the pair looked singled out.

The mechanism is our own truncation rule. `generate_vsop87a.py` drops terms below a **uniform
absolute amplitude of 1e-7 AU** for all eight planets — Saturn is not cut harder, it retains the
most terms of any planet — and a uniform absolute cut leaves a residual that scales with the
orbit: 9x the threshold at 1 AU, 60x at Jupiter, 116x at Saturn, 249x at Uranus, 734x at Neptune.

Nothing was regenerated. A tighter threshold trades coefficient-set size for accuracy, and the
size is what makes this provider viable in WASM and at the edge; that is a ratified product
decision, not a patch. On `SpkReader` the outer planets are already the *best* bodies —
Jupiter 0.066 arcsec, Saturn 0.064 arcsec against the same oracle — and the README now says so.

## A guard that was never called

`require_release_profile` in `riseset.rs` panics rather than skipping, and its doc comment
explains at length why a silent skip would be worse. It was called by none of the four
`derivation-sweeps` tests it was written for. Now wired into all four, and verified to fire.

## The lint gate that would have found it

`cargo clippy --workspace`, what CI gated, was clean. `--all-targets` carried 153 warnings,
because crate-level `#![allow]` attributes never reach `tests/`, `benches/` or `examples/`. Two
of the 153 were real: the dead guard above, and five imports orphaned by this release's own
deletions.

The policy moved to a `[workspace.lints]` table, which does cover every target, and the 33
crate-level attributes it replaces are gone. CI now gates
`cargo clippy --workspace --all-targets --all-features -- -D warnings`.

`missing_errors_doc` and `missing_panics_doc` are allowed for now: 44 hits, all in
`vedaksha-wasm` and `bindings/python/engine`, which carried no pedantic lints at all until this
table existed. Writing those sections is worth doing and is not this change.

## Compatibility

A `vedaksha-vedic = "7"` dependent keeps compiling. A `vedaksha-ephem-core = "7"` dependent that
matches [`Body`] **exhaustively** does not — see the Ketu section above. That is a breaking
change shipping in a minor version, by decision rather than oversight: it is the fix that puts
Rahu and Ketu in one frame, no known consumer is on `^7`, and holding it for an v8.0.0 would mean
two releases in as many days. Add the three arms, or a wildcard.

Everything else is additive or deprecated-in-place:

- `sthana_bala(planet, sign, longitude)` keeps its signature and its v7 behaviour — Uchcha Bala,
  sign discarded — and is deprecated in favour of `uccha_bala`. The composite is
  `sthana_bala_full(planet, longitude, bhava)`.
- `compute_shadbala(positions, lagna_sign)` is kept and deprecated. It still totals the three
  components reachable from a bare `GrahaPosition`; the struct no longer claims that is six.
- `Shadbala` gained four fields. JSON consumers see additions only.
- **`compute_shadbala` now rejects a `sign` that disagrees with `longitude`.** Nothing read
  `sign` before, so a caller could contradict itself and never learn. Both are consumed now.

Nine manifests declared both `license` and `license-file`, warning on every build. `license-file`
is gone; `license = "BUSL-1.1"` is a standard SPDX expression and is what every registry reads.
Verified by unpacking a built `.crate`: the LICENSE is still archived.

## Validation

Full validation at this tree — `cargo test --workspace --release --locked -- --include-ignored`,
the same command `full-validation.yml` runs: **35 targets, 1,093 passed, 0 failed, 0 ignored.**
The README badge moves 1,069 → 1,093, re-measured rather than inferred from the delta.

Also green: `cargo clippy --workspace --all-targets --all-features -- -D warnings` at zero, with
`missing_errors_doc` and `missing_panics_doc` enforced; `cargo fmt --all --check`; the six version
places; publish-order; licence sync across nine copies; SPDX headers on 176 files; the WASM–MCP
surface parity test; `ruff`; and the Python conformance fixture regenerated against a rebuilt
7.2.0 blob, 20 passed.

`analytical_bit_digest` was re-pinned once, for the `COS_EPS` correction, with the measurement
recorded beside the new constant.

Three fixes were verified by planting the defect they prevent and confirming the test fails:
the Sthana composite, the Ishta/Kashta input, and all four emitter-escaping guards. Two of those
guards did not exist until the planted defect showed nothing caught it.
