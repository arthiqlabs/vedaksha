# Lunar node theory review — open questions

> **BOTH QUESTIONS ARE NOW CLOSED. Do not act on the conclusions below without
> reading the follow-ups.** Finding 1 (TrueNode) was resolved in v7.5.0. Finding 2
> (MeanNode) was resolved on 2026-09-01 and is **not** a Vedaksha defect — see
> `2026-09-01-mean-node-divergence-resolved.md`. That document also corrects a
> reasoning error in the companion file
> `2026-08-29-lunar-node-self-consistency-results.md`, which was cited here as
> ruling out a J2000-frame cause: the self-consistency check it reports compared
> Vedaksha against Vedaksha and therefore had **no power** to detect that error,
> since a shared frame mistake would cancel on both sides. The hypothesis was
> recorded as eliminated while still untested.

**Date:** 2026-08-29
**Origin:** an independent oracle-parity harness (`vedaksha-parity`) called Vedaksha through its
published PyPI package (v7.4.0) and reported two measured divergences on real, third-party-sourced
birth data. Per that harness's own discipline, every reference engine is treated as a sealed box —
this document states quantities, values, deltas and theory questions only, never how a reference
computes anything internally. Nothing here is a claim that Vedaksha is wrong; it is a set of
questions for Vedaksha's own theory review to resolve against published sources, the same
discipline this repo already applies to its own ayanamsha and precession work.

**Ground rule for resolving either question below:** per `DATA_PROVENANCE.md` and this project's
consistent practice (see the ayanamsha re-derivation, `docs/audit/2026-08-17-ayanamsha-cleanroom/`),
a divergence against any external reference is never closed by adjusting a constant to match that
reference's output. It is closed by deriving the correct value independently from a cited primary
source (Meeus, Chapront, Capitaine/Wallace/Chapront, or equivalent), and the fix cites that source
— never the reference that raised the question.

---

## Question 1 — Does `Body::TrueNode`'s truncated series explain the measured divergence, and if so, should the default change?

**Measured:** across 200 real birth instants (1452–1999 CE), Vedaksha's `TrueNode` sidereal
longitude differed from an independent reference by more than 60 arcsec in 85.5% of cases, up to a
maximum of 976.4 arcsec (≈0.27°). The delta did not correlate cleanly with time-distance from
J2000 (r ≈ 0.15) — a pattern consistent with a periodic or higher-order term rather than a constant
rate offset.

**What Vedaksha's own source already says, independent of any reference engine:**

- `crates/vedaksha-ephem-core/src/nodes.rs:42-71` (`true_node`) is documented, in its own doc
  comment, as using only "the 5 largest perturbation terms from Meeus Ch. 47," and states
  explicitly: *"Residual vs full lunar theory: ~0.09° max for modern dates. Improving beyond this
  requires verified coefficients from Chapront's complete lunar node series — not attempted here to
  avoid sign/argument transcription errors."*
- The same file provides a materially more accurate node function, `true_node_osculating`
  (`nodes.rs:73-117`), computed directly from the ELP/MPP02 lunar ephemeris rather than a truncated
  periodic series. It is independently validated against the public-domain JPL Horizons `OM`
  quantity to a mean of 0.00008° and max of 0.00017° (≈0.6 arcsec) — see the test
  `osculating_node_vs_jpl_horizons`, `nodes.rs:376-425`.
- Vedaksha's own test suite already bounds and quantifies the gap between these two functions:
  `osculating_node_close_to_meeus_true_node` (`nodes.rs:332-362`) and `every_node_method_shares_one_frame`
  (`crates/vedaksha-ephem-core/tests/node_frame.rs:187-209`) assert the truncated-Meeus vs.
  full-osculating divergence stays under 0.35° (1260 arcsec) across 1900–2100, and this bound is
  documented as a **bounded libration**, not a growing error — matching the non-linear, weakly
  time-correlated shape of the measured divergence.

**The question:** is the measured Finding-1 divergence consistent, in magnitude and in its
non-linear time behavior, with the already-documented and already-bounded gap between the
5-term truncated series and the full osculating computation? If so, this is not evidence that
either formula is computed incorrectly relative to its own citation — both already have primary
sources and passing tests. It is a question of which one `Body::TrueNode` should return by default
for a jyotish consumer, and whether the ≤0.35° residual of the current default is disclosed clearly
enough given that a nakshatra pada is only a few degrees wide. This project has precedent for
resolving exactly this class of question by making a hidden choice an explicit, tested parameter
(see the `compute_vargas` tradition parameter, v7.4.0) rather than by changing which formula
computes what silently.

---

## Question 2 — Where does the ~50 arcsec/year linear drift in sidereal `MeanNode` come from?

**Measured:** across 168 evaluable birth instants (1899–1999 CE), Vedaksha's `MeanNode` sidereal
longitude (Vedaksha configured to its own `TrueChitra` ayanamsha) differed from an independent
reference by more than 60 arcsec in 99.4% of cases, every one in the same signed direction. The
delta correlates with years-from-J2000 almost perfectly linearly (r = 1.00): delta ≈ −50.35 ×
(years from J2000) − 2.5 arcsec.

**What Vedaksha's own source already says, independent of any reference engine:**

- A delta that grows linearly at very close to 50 arcsec/year, with a near-zero intercept, is the
  numerical signature of **general precession in longitude** — Vedaksha's own cited IAU 2006 P03
  rate is `5_028.796_195` arcsec/century (≈50.288 arcsec/year), from Capitaine, Wallace & Chapront
  (2003), A&A 412, eq. (37) (`crates/vedaksha-ephem-core/src/precession.rs:39-72`). Vedaksha's own
  `TrueChitra` ayanamsha, self-timed between two of its own committed fixture values (B1900.0 and
  J2000.0 in `crates/vedaksha-astro/tests/fixtures/ayanamsha.json`), moves at ≈50.23 arcsec/year —
  essentially the same rate (the small difference is Spica's own catalogue proper motion).
- Vedaksha's own source separately documents, in plain language, exactly what a mismatch of this
  size and shape looks like: `nodes.rs:100-111`, on `true_node_osculating_j2000`, states that
  mixing a J2000-referred node value with an of-date quantity produces a gap of *"roughly 50.3
  arcsec per year from J2000."* That figure is a near-exact match to the measured −50.35
  arcsec/year rate.
- A full trace of every currently-reachable production call path — `compute_natal_chart` and
  `compute_vargas` via the MCP server (`crates/vedaksha-mcp/src/server.rs`), the WASM surface
  (`crates/vedaksha-wasm/src/lib.rs`), and the published Python package (a thin transport shim over
  the same MCP server, `bindings/python/engine/src/lib.rs`) — found a single, consistently-threaded
  `jd` used for both the node's longitude (`coordinates.rs:169`, via `nodes::node_longitude`) and
  the ayanamsha subtraction (`chart.rs:139-141`, `ayanamsha_value`). No J2000-vs-of-date mixing was
  found in this trace. `true_node_osculating_j2000` itself is unreachable from any `Body` variant,
  MCP tool, or the WASM/Python surface — its only caller is its own unit test.
- Also worth noting as a real, separate gap this review surfaced: **no test in this codebase
  currently checks a node's *sidereal* longitude against an independent value at widely-separated
  epochs.** The one end-to-end sidereal-node test (`crates/vedaksha-wasm/tests/mcp_surface_parity.rs:945-990`)
  checks that the WASM and MCP surfaces agree with each other at one date, 1985 — not that either
  is correct against theory or an oracle. Since both surfaces share the same underlying crates, a
  shared defect would not be caught by that test regardless of where it lives. This gap is why an
  external harness using real multi-decade-old birth dates found something CI could not have.

**The question:** since the mixing mechanism this magnitude and shape would suggest could not be
located in the code paths reached by the published package's `compute_natal_chart`/`compute_vargas`
tools, where else could it arise?

- Could there be a discrepancy between the git tag `v7.4.0`'s source and the actual wasm blob
  shipped in the published PyPI package at the time the harness ran (a build/packaging staleness
  question, not a source-code question — this project has had wasm-binding build gotchas before)?
  This is checkable by installing the current PyPI package and confirming its output matches what
  the current `nodes.rs`/`sidereal.rs` source, evaluated directly, produces for the same `jd` — a
  self-consistency check requiring no reference-engine data at all. **See the addendum below: this
  check was run on 2026-08-29 and the package agrees with source.**
- Is there a code path this review did not check — e.g. a different MCP tool, a caching layer, or
  version skew between what was actually installed and what this review's line numbers describe?
- Independent of the above: is `TrueChitra`'s own precession/proper-motion model (IAU 2006 P03 plus
  van Leeuwen 2007 Hipparcos proper motion, `crates/vedaksha-astro/src/sidereal.rs:203-214`,
  `crates/vedaksha-ephem-core/src/stars.rs:129-152`) the theoretically correct one for a
  "True Chitrapaksha" system as classically defined, independent of what any other implementation
  does? This project's own precedent (the ayanamsha re-derivation) is to re-verify the convention
  against a primary source rather than assume the existing implementation is correct because it
  passes its own tests.

**Explicitly not concluded here:** whether the defect (if any) is in Vedaksha, in a packaging step,
or is a legitimate difference in what "True Chitrapaksha" means between two independent
implementations. This review's own code trace found no reproducible defect in the current,
reachable source — that absence is reported as-is, not stretched into either a confirmation or a
denial.

---

## Addendum — self-consistency check (2026-08-29), completed

Vedaksha talking to itself, no reference-engine data involved: computed `mean_node(jd)` and
`ayanamsha_value(TrueChitra, jd)` two ways at three widely-separated epochs (B1900.0, 1950,
J2000.0) — (a) directly from the current `main` source via a throwaway Rust test, and (b) through
the same published PyPI package (`vedaksha==7.4.0`) the harness called. Full method and numbers in
[`2026-08-29-lunar-node-self-consistency-results.md`](2026-08-29-lunar-node-self-consistency-results.md)
in this same directory.

**Result: the package agrees with source across the full century tested, to within nutation-scale
noise (+17.41″ at 1900, −3.37″ at 1950, −14.07″ at 2000) — bounded, not growing.** This rules out a
stale or divergent packaging build as the explanation for Finding 2, and it rules out a
J2000-vs-of-date frame confusion inside Vedaksha's own theory layer: if that mechanism were the
cause, a century of separation would show on the order of 5,000 arcsec of drift between the
package and source, not the observed few-to-seventeen arcsec of nutation. The mechanism behind
Finding 2, if real, therefore does not live in a build/packaging gap or in the theory-layer
functions checked here — it remains open to whichever of the two remaining candidates above (an
unreached code path, or a genuine convention difference in how "True Chitrapaksha" is realized)
the next round of review takes up.
