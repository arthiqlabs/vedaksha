# Sidereal nutation and chara-karaka theory review — open questions

**Date:** 2026-08-29
**Origin:** the same oracle-parity harness (`vedaksha-parity`,
<https://github.com/arthiqlabs/vedaksha-parity>) as the lunar-node review, on Vedaksha v7.5.0. That
harness is maintained by the same authors as this engine; what is independent is the methodology
and the reference ephemerides, not the party — it reads Vedaksha only through its published
interfaces. Per that harness's own discipline, every reference engine is a sealed
box — this document states quantities, values, deltas and theory questions only, never how a
reference computes anything internally. **Unlike the two questions below, the first is fully
resolved entirely from Vedaksha's own source** — no reference-engine data was needed to explain it,
only to notice it.

---

## Question 1 — Sidereal chart positions carry an uncancelled nutation-in-longitude term

**Measured:** across 1782 evaluable (body, birth-instant) pairs, Vedaksha's sidereal longitude
(`IndianOfficial` ayanamsha) differed from an independent reference by a small amount — mean near
zero, amplitude roughly ±18-20 arcsec — uniformly across all bodies, with best-fit periodicity at
**18.70 years** (r² = 0.93). The same 200-record sample compared *tropically* instead (Vedaksha's
own `tropical = sidereal + ayanamsha_value` sum against the reference's native tropical output)
passed 92.8% of cases at sub-arcsecond precision. The oscillation is present only when both sides
are compared sidereally.

**This is fully explained by Vedaksha's own documented design, with no reference-engine data
needed:**

- `crates/vedaksha-astro/src/sidereal.rs:14-26` (module doc): *"**Mean ayanamsha, always.** Nutation
  in longitude is never included. A caller who wants the true ayanamsha adds nutation themselves...
  An engine that is silent about which one it returns gets compared against the wrong column."*
  Repeated on the function itself (`sidereal.rs:908`) and on `tropical_to_sidereal`
  (`sidereal.rs:945`), and in the Python binding: *"Values are the **mean** ayanamsha — add
  nutation in longitude yourself for the true one"* (`bindings/python/src/vedaksha/client.py:84-85`).
- `crates/vedaksha-astro/src/chart.rs:139-141,183`: `compute_chart` computes
  `sidereal_lon = tropical_lon − ayanamsha_offset`, where `ayanamsha_offset` is exactly the
  mean-only `sidereal::ayanamsha_value` (chart.rs:121-122 documents `lon` as "tropical ecliptic
  degrees" with no further qualification).
- The `tropical_lon` fed into `compute_chart` is **not** mean-of-date — it comes from
  `coordinates::apparent_positions` (called at `crates/vedaksha-mcp/src/server.rs:334`, feeding
  `compute_chart` at `server.rs:394-401`), whose own module doc states it "Chains light-time
  correction, precession (IAU 2006), **nutation (IAU 2000B)**, annual aberration... to produce
  apparent ecliptic positions" (`crates/vedaksha-ephem-core/src/coordinates.rs:6-10`) — confirmed
  mechanically: `frame_for` builds a nutation×precession matrix (`coordinates.rs:131-146`) applied
  to every body's geocentric vector to produce a **true-equinox-of-date** longitude.
- So every sidereal chart position is `(mean_tropical + Δψ) − ayanamsha_mean`, carrying an
  uncancelled `+Δψ` (nutation-in-longitude). The principal term of Vedaksha's own IAU 2000B series
  — the one whose argument depends solely on the lunar node's mean longitude, i.e. the classical
  18.6-year nutation cycle — has coefficient `-172_064_161` (units of 0.1 μas,
  `crates/vedaksha-ephem-core/src/nutation.rs:28`), converting to **17.2064161 arcsec**. This
  matches the harness's measured ~18-20 arcsec amplitude and 18.70-year period closely enough that
  no other candidate mechanism is needed.
- **No existing test could have caught this.** `ayanamsha_fixture.rs` checks `ayanamsha_value`
  alone against an independent mean-ayanamsha derivation, never touching nutation or a full chart.
  `chart.rs`'s own sidereal tests construct their "expected" value by calling the same
  `ayanamsha_value` function under test, so they are self-consistent by construction and
  structurally cannot detect a bias shared by both sides. `coordinates.rs`'s one external-oracle
  test (`moon_longitude_at_j2000_matches_jpl_horizons`) is tropical-only and never applies an
  ayanamsha. No test anywhere checks a full sidereal *chart* longitude against an external or
  theoretical value at more than one epoch.

**The question this raises, for Vedaksha's own team to decide:** should `compute_chart`'s sidereal
subtraction use the **true** ayanamsha (mean + nutation-in-longitude) to stay internally consistent
with the true-of-date tropical longitude it already receives — matching what "the daily tables most
panchanga-makers consume" actually use, per `sidereal.rs`'s own module doc — rather than leaving
this mismatch live in the one code path (`compute_chart`) that was never covered by the doc
warning's own scope (that warning describes `ayanamsha_value` as a standalone quantity; nothing
documents that the *whole chart's* delivered sidereal positions inherit the same gap)? This is not
a "which convention is correct" question the way the TrueNode question was — `sidereal.rs`'s own
prose already states plainly which of mean/true is used by real published tables. It reads as an
internal-consistency gap between a documented API contract and what the chart-assembly code
actually does with it, not a genuine theory dispute.

**Scale, so it's not over- or under-stated:** ~17-20 arcsec is roughly 0.14-0.17% of a nakshatra
pada (3°20′ = 12,000″) — small relative to Vedaksha's own sub-arcsecond tropical accuracy claim,
but the whole point of that claim is tropical; this is the first evidence this project has that its
*sidereal* output, which is what most Jyotish consumers of this engine actually read, has never been
validated end-to-end against anything external.

---

## Question 2 — Chara-karaka rank of Rahu near a sign boundary

**Measured:** of 198 evaluable chara-karaka rankings (8-karaka scheme, `compute_karakas`), 156
(78.8%) matched an independent reference on all 8 ranks; 42 had at least one rank disagreement — 37
were adjacent-rank swaps, and 5 were full or near-full reorders. In every one of the 5 full-reorder
cases, Vedaksha's own computed Rahu degree-within-sign sat within about a degree of a sign boundary
(0.062°, 0.618°, 29.098°, 29.103°, 29.538°). The harness's own diagnostic — re-ranking with the
opposite direct/reflected convention applied to Rahu's degree alone — reproduced the reference's
exact rank in 4 of the 5 cases.

**What Vedaksha's own source says, independent of the reference:**

- Rahu's degree **is** reflected in the current implementation: `rahu_degrees_in_sign`
  (`crates/vedaksha-vedic/src/karaka.rs:72-80`) returns `30.0 - d` (with an explicit `d == 0.0 → 0.0`
  special case), called only for the 8-karaka scheme (`karaka.rs:103-108`).
- The **entire module's citation is one line**: `"Source: Jaimini Sutras 1.1."`
  (`karaka.rs:13`). Nothing in the file ties the *reflection rule specifically* to a chapter or
  verse — the doc comment states the reflection as a bare assertion ("using a reflected degree...
  because it moves retrograde," `karaka.rs:10-11,72-75`) with no citation of its own, distinct from
  the module-level Jaimini citation that covers the karaka scheme in general.
- **No prior audit trail exists for this at all.** Neither `DATA_PROVENANCE.md` nor `docs/audit/`
  contains any mention of chara-karaka methodology, Rahu's treatment, or Jaimini's own rule for
  reading a retrograde/nodal body's degree — unlike the ayanamsha and lunar-theory subsystems, which
  each have a dedicated, primary-sourced derivation record.
- **The boundary itself is undertested regardless of which convention is correct.** The one
  existing test for `rahu_degrees_in_sign` (`karaka.rs:243-248`) checks a mid-sign value (310° →
  20°) and the exact `0.0°` special case, but nothing exercises Rahu's degree approaching either
  edge of a sign from within a few arcseconds — exactly the regime the 5 full-reorder cases sit in.
  Separately, the `d == 0.0` check is an exact floating-point equality comparison, which is fragile
  by construction: a value that should be exactly zero but arrives as a tiny residual (e.g. `1e-15`)
  from upstream normalization would silently fall through to the `30.0 - d` branch instead of the
  intended `0.0`, producing a near-maximal (Atmakaraka-adjacent) rank instead of the intended
  near-minimal one. This is a real code-quality gap independent of the convention question below.

**The question this raises:** does Jaimini Sutras 1.1 (or whichever specific verse governs karaka
ranking for a retrograde/nodal body) actually call for a reflected degree, a direct degree, or does
it not address the nodes at all — making the reflection convention itself an inference this project
made without a specific citation? This needs a primary-source check the way the ayanamsha
re-derivation got one; right now the only warrant for the current behavior is the doc comment's own
say-so. Separately, and regardless of that answer, the `d == 0.0` exact-equality special case should
be replaced with an epsilon-tolerant boundary check before it ships a rank flip on floating-point
noise alone.

**Explicitly not concluded here:** which convention (direct or reflected) is correct — that is a
Jaimini-sutra reading question this report cannot answer, and the harness's own diagnostic result
(4 of 5 cases reconciled) is suggestive, not dispositive.

---

## Recommended next steps

1. **Question 1** looks like a real, fixable defect with a clear direction (use true ayanamsha —
   mean + nutation-in-longitude — inside `compute_chart`'s sidereal subtraction) and a clean,
   entirely-internal justification. This is Amit's call on whether/how to proceed, same as the
   TrueNode fix.
2. **Question 2** needs an actual primary-source check (does Jaimini Sutras 1.1, or another
   specifically-cited verse, address a retrograde/nodal body's karaka degree at all) before any code
   change — this is exactly the kind of divergence this project's own discipline says should be
   answered from a classical source, not by adopting whichever convention happens to match an
   external comparison.
