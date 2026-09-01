# Vedaksha — Ongoing Maintenance Guide

> **Annual review recommended: April 1st each year.**
> This document lists every component that may need periodic updates. If ArthIQ Labs LLC ceases maintenance, any fork maintainer can follow this guide to keep the platform accurate and secure.

---

## 1. Earth Orientation Parameters (EOP) — not carried

**What:** The IERS publishes measured UT1-UTC, polar motion and length-of-day in weekly bulletins.

**Status: this engine carries none of it, and there is nothing here to maintain.** There is no EOP table and no polar-motion correction anywhere in the workspace. This follows from the public contract: **every Julian Day on the public surfaces is UT1**, supplied by the caller. Because the engine never converts civil time to UT1, it never needs UT1-UTC.

**What that means for an integrator:** obtaining UT1 is *your* responsibility. If you feed UTC directly, you inherit up to 0.9 s of Earth rotation error, which is about 13.5 arcseconds of sidereal time. The resulting shift in the ascendant and the house cusps is of that order but not equal to it: it varies with latitude and with the sign rising. For most astrological work that is acceptable and is the usual practice; if it is not acceptable for yours, apply UT1-UTC from `finals2000A.all` before you call.

Polar motion is not applied either. It displaces the pole by under 0.5 arcsecond and is below the threshold of any astrological use.

---

## 2. Leap Seconds — not carried

**Status: there is no leap-second table in this engine, and no maintenance task attaches to one.** It follows from the same contract as §1: the engine is handed UT1 and never performs a UTC conversion, which is the only place a leap-second table would be consulted.

If your application converts civil UTC to UT1 before calling, that conversion is where leap seconds belong, and keeping the table current is your side of the boundary. IERS Bulletin C announces insertions about six months ahead. No leap second has been inserted since 2016-12-31, and the 27th CGPM (2022, Resolution 4) resolved to stop inserting them by 2035.

---

## 3. Delta T (TT − UT1)

**What:** The difference between Terrestrial Time (TT, uniform atomic time) and UT1 (Earth rotation time). Historical values are measured; future values are predicted using polynomial approximations.

**Why it matters:** Converting between the Julian Day used in ephemeris computation (TDB/TT) and the UTC/UT1 used by humans requires Delta T. The prediction polynomial drifts over time as Earth's rotation deviates from the model.

**Impact if neglected:** The prediction error grows roughly quadratically. After 5 years without update, the error is typically < 2 seconds. After 20 years, it could be 10–30 seconds. For the Moon (fastest-moving body), 1 second ≈ 0.5 arcsecond.

**How to update:**
1. Get the latest Delta T observations from: https://datacenter.iers.org/data/latestVersion/finals2000A.all (column for TT-UT1) or Espenak & Meeus tables
2. Update the polynomial coefficients in `crates/vedaksha-ephem-core/src/delta_t.rs`
3. Extend the historical measured table with new data points
4. Run accuracy tests to verify the new polynomial matches observed values

**Frequency:** Every 5 years is sufficient for astrological use. Every 1–2 years for research-grade.

---

## 4. JPL Planetary Ephemeris

**What:** NASA JPL periodically releases improved Development Ephemeris versions incorporating new observations.

**What ships:** `SpkReader` reads **DE440s**, the short-span kernel (`data/de440s.bsp`, 32,726,016 bytes, fetched by `scripts/download_de440s.sh` against a pinned SHA256). Read from the kernel's own segment headers, **it covers 1850-01-01 to 2150-01-01**. DE441 appears in this project only as the *oracle* — Horizons serves DE441, so the accuracy comparison is against a kernel we do not ship.

**Impact if neglected:** accuracy does not decay; coverage runs out. Requests outside 1850–2150 fail on the SPK path rather than degrading quietly, and `AnalyticalProvider` (VSOP87A + ELP/MPP02, no data files) is the path for dates beyond it. A future DE release would change inner-planet positions by well under a milliarcsecond — invisible to any astrological application.

**How to update:**
1. Check for new releases at: https://ssd.jpl.nasa.gov/planets/eph_export.html
2. Download the new SPK and update the URL and pinned SHA256 in `scripts/download_de440s.sh`
3. Re-run the oracle suite; regenerate `tests/oracle_jpl/reference_positions.json` only if the comparison kernel changed

**Frequency:** Only when NASA releases a new major version (every 5–15 years), or **before 2150** if the engine is still in service. DE440/441 were released in 2021.

---

## 5. Star Catalogue Data

**What:** The engine carries **five** catalogue stars, not a general star catalogue. They exist to serve the sidereal systems that are *defined* by fixing a named star, and nothing else: ζ Piscium, δ Cancri, Spica, λ Scorpii and Sgr A\*, in `crates/vedaksha-astro/src/sidereal.rs`. There is no fixed-star conjunction or paran feature.

**Why it matters:** five of the eleven ayanamshas track a live star, so their values depend on catalogue astrometry rather than on a published constant. Proper motion is applied by `vedaksha_ephem_core::stars`; see §7.

**Impact if neglected:** a Hipparcos proper motion good to ~0.5 mas/yr accumulates ~0.5 arcsecond of longitude per thousand years from the catalogue epoch. That is the honest uncertainty on a star-anchored system far from epoch, and no update removes it.

**How to update:** the rows are inputs to the derivation, so they change the same way every other ayanamsha input does — through `derivation-inputs.json` and the spec, never the Rust constants alone. See §7. `python3 scripts/generate_ayanamsha.py --check-catalogue` re-queries VizieR and confirms the committed rows still match.

**Frequency:** Only when a catalogue release supersedes Hipparcos for bright stars (Gaia, eventually).

---

## 6. Asteroid Orbital Elements — not applicable

**This engine computes no asteroids.** There are no orbital elements, no asteroid data file and no Chiron/Ceres/Juno/Vesta/Pallas surface. Nothing here requires maintenance. If asteroid support is ever added, this section becomes a real one and an MPC element source gets a row in `DATA_PROVENANCE.md`.

---

## 7. Ayanamsha Values

**What:** Two things can move an ayanamsha: an issuing authority revising a published definition, or a star catalogue being superseded.

**Why it matters:** Five of the eleven systems track a live star, so their values depend on catalogue astrometry rather than on a published constant. The other six rest on an anchor an authority published; the Indian official system is the only one whose authority still issues revisions.

**Impact if neglected:** An IAE revision would be well under an arcsecond. A catalogue change matters more: the ESA 1997 and van Leeuwen 2007 Hipparcos solutions differ by up to 3.3 mas/yr in proper motion, which is ~0.5 arcsecond of longitude per thousand years from the catalogue epoch. Gaia will eventually supersede Hipparcos for these stars.

**How to update — and the one rule that governs it:**

Every value is derived from `docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json`. **Change that file, never the Rust constants**, and never both independently. The derivation spec is normative: any change to an anchor, model, convention or declared assumption is a change to the derivation and belongs in `docs/audit/2026-08-17-ayanamsha-cleanroom/spec.md` first.

1. Record the change and its reasoning in the spec.
2. Update `derivation-inputs.json`.
3. Mirror it in `crates/vedaksha-astro/src/sidereal.rs` (the two are deliberately independent implementations; the fixture is what proves they agree).
4. `python3 scripts/generate_ayanamsha.py` to regenerate the fixture.
5. `cargo test -p vedaksha-astro` — the anchor, zero-year-inversion and cross-check tests all have to pass.
6. `python3 scripts/generate_ayanamsha.py --check-catalogue` to confirm the committed catalogue rows still match VizieR.

**Never** validate a change by comparing against another implementation's numbers. That is the reverse engineering the re-derivation exists to undo, and it is not made acceptable by citing a source afterwards.

**Frequency:** Check when a new Indian Astronomical Ephemeris is published, or when a major astrometric catalogue release supersedes Hipparcos for bright stars.

---

## 8. Rust Toolchain & Dependencies

**What:** Rust compiler updates, dependency security advisories, and breaking changes in upstream crates.

**How to update:**
```bash
rustup update
cargo update
cargo audit          # Check for known vulnerabilities
cargo deny check     # Check license compliance
```

**Frequency:** Quarterly. Security advisories should be checked monthly.

---

## 9. WASM Toolchain

**What:** `wasm-pack`, `wasm-bindgen`, and browser WASM standards evolve. The WASM Component Model is still maturing.

**How to update:**
```bash
cargo install wasm-pack --force
# Rebuild and test WASM targets
wasm-pack build --target web crates/vedaksha-wasm/
```

**Frequency:** With each release, or when targeting new WASM runtimes.

---

## MCP `tools/list` Snapshot

**What:** `tools/mcp-tools.json` is a committed snapshot of the canonical
`tools/list` JSON-RPC response body, generated from the Rust
`tool_definitions()` registry in `crates/vedaksha-mcp/src/tools/`.

**Why it exists:** The introspection-only MCP endpoint at
`vedaksha.net/api/mcp` reads it at request time, and the portal's
`/docs/mcp` page reads it for the rendered tool catalog. Both surfaces
are therefore guaranteed to match the Rust source as of the last commit
on `main`.

**When to regenerate:** Any time a tool is added, removed, renamed, or
its `inputSchema` / `description` changes. The drift-guard test
`tools::tests::snapshot_matches_current_tool_definitions` fails CI if
the snapshot is stale.

**How to regenerate:**

```bash
cargo run -p vedaksha-mcp --bin dump-tools-list > tools/mcp-tools.json
cargo test -p vedaksha-mcp tools::tests::snapshot_matches_current_tool_definitions
git add tools/mcp-tools.json
git commit -m "chore(mcp): regenerate tools/list snapshot"
```

The site repo (`arthiqlabs/vedaksha-site`) reads the snapshot from
`https://raw.githubusercontent.com/arthiqlabs/vedaksha/main/tools/mcp-tools.json`
at MCP-route cold start, so the snapshot must be on `main` for the live
endpoint to see the change. Redeploy the site repo (Cloudflare Workers,
via OpenNext) after merging here.

---

## 10. Derived Constants — Already Established, Do Not Re-derive

Some constants in this engine were fixed by expensive parameter searches. **Those searches
have been run. Their conclusions are recorded here and beside the constants themselves.**
Re-running them costs hours and tells you nothing new — do it only when the stated
invalidating condition is met.

### CI does not derive coefficients. Ever.

Stated first because it has been misread twice, both times from the same evidence: a CI job
that takes over an hour, and a stack full of `elp_mpp02::eval_pert_series`.

**Push CI invokes no generator at all.** `ci.yml` runs no script in `scripts/`. The VSOP87A and
ELP/MPP02 coefficients are committed binaries under
`crates/vedaksha-ephem-core/src/analytical/coefficients/` — 331 files, ~30,000 retained terms —
and they change only when a human runs a generator without `--verify` and commits the result.

What is slow is **evaluating** those constants, which is what computing a position means. In a
debug build, with no optimisation and no SIMD, one ELP/MPP02 evaluation is orders of magnitude
slower than in `--release`, and every osculating-node call pays three of them for its finite
difference. `analytical_bit_digest` is `#[ignore]`d and release-only for exactly this reason.

**The weekly job is the only place a generator runs, and it is read-only.**
`full-validation.yml` calls each generator with `--verify`, which re-fetches the primary,
regenerates into a *temp directory*, and compares SHA256s against the committed `.bin.sha256`
sidecars. It never writes to the working tree. Green means the committed coefficients still
match the primary; red means the primary moved or a sidecar was edited.

So: a slow CI run is not a derivation, a green weekly run is not a regeneration, and neither
one can change a shipped value. If a coefficient moved, it moved in a commit with a human's
name on it, and `analytical_bit_digest` will say so.

### `AGREEMENT_TOL_DAYS` — `vedaksha-astro::riseset`

The tolerance at which the analytic sunrise path is held to agree with the brute-force scan.

- **Value:** `1e-9` days
- **Established by:** 441,000 fixed-RA plus 36,120 real-Sun comparisons across latitude to
  both poles, the full longitude range, three eras and two elevations
- **Maximum disagreement measured:** exactly one ULP (4.656612873077393e-10 d), with zero
  presence disagreements. `1e-9` is the next clean figure above that measured maximum.
- **Re-derive with:** `cargo test --release -p vedaksha-astro --features derivation-sweeps -- --ignored --nocapture`
  (release only — the real-Sun path is ~45× slower in a debug build)
- **Invalidated by:** a change to the sunrise search algorithm itself. Nothing else.

### `ANALYTIC_LATITUDE_LIMIT_DEG` — `vedaksha-astro::riseset`

Above this latitude the analytic path defers to the brute-force scan, because near the pole a
sunrise can otherwise be attributed to the wrong rotation.

- **Value:** `89.0` degrees
- **Established by:** 9,408 comparisons over latitude 88.0–90.0 at 0.1° resolution. Clean
  through 89.8; the first wrong attribution appears at 89.9 (0.4967 d — nearly a whole vara).
  89.0 sits below the lowest latitude any measurement implicates.
- **After routing:** the polar band's 16,800 comparisons drop from 8 presence disagreements
  and a 0.4967 d worst gap to 2 and one ULP.
- **Invalidated by:** a change to the sunrise search algorithm itself.

### The IAU 2006 precession composition — `vedaksha-ephem-core::precession`

`precession_matrix` composes four Fukushima-Williams rotations, and **the order is the whole
content of the expression**. It is `Rx(−εA) · Rz(−ψ̄) · Rx(φ̄) · Rz(γ̄)`.

- **Established by:** comparison against ERFA's `eraEqec06` at four epochs spanning 6100
  years, in `precession_matrix_matches_erfa_across_six_millennia`.
- **Why it needs recording:** a wrong order is **0.014 mas at J2000 and 0.56 arcsecond at
  499 CE**. It shipped that way through v5 because every existing check sampled 1900–2100,
  where it hides. If you "simplify" this expression, that test is the only thing that will
  tell you.
- **Also:** `general_precession_in_longitude` returns the F-W angle `ψ̄_A`, **not** general
  precession `p_A`. They differ by ~9.7 arcsec/century and are not interchangeable; use
  `general_precession_p03` for anything sidereal. A test asserts they stay distinct.
- **Invalidated by:** adopting a different precession theory. Nothing else.

### Ayanamsha constants — read from primaries, not searched for

No parameter search established these, so there is nothing expensive to re-run — but there is
a rule, and it is stricter than the ones above.

- **Never validate an ayanamsha against another implementation's numbers.** Not Swiss
  Ephemeris, not any other engine, not a worked-value table from a published book. That is
  the reverse engineering the re-derivation exists to undo, and citing a source afterwards
  does not change what it is.
- **Change `docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json`, never the Rust
  constants alone**, and record any change to an anchor, model, convention or declared
  assumption in that directory's `spec.md` first.
- **What is already answered** (do not re-search): the search records for all 33 dropped
  systems, the definition/determination test that excludes every Babylonian system, and the
  declared assumptions DA-1 through DA-10. All in `spec.md`.
- **Invalidated by:** a new *Indian Astronomical Ephemeris* revision, or a star catalogue
  superseding Hipparcos for bright stars. See §7.

### The two oracles are DISJOINT — check you are using the right one

This has caused a real defect: a dependency upgrade was validated four times against an oracle
structurally incapable of seeing the code it changed.

| Oracle | Drives | Rows | Covers |
|---|---|---|---|
| `oracle_comparison.rs` | `SpkReader` (DE440s kernel) | 24,350 | the SPK path **only** |
| `analytical_oracle.rs` | `AnalyticalProvider` (VSOP87A + ELP/MPP02) | 21,915 | the analytical path **only** |

The analytical count is 21,915 and not 24,350 because `AnalyticalProvider` returns
`BodyNotAvailable` for Pluto — that is a real provider property, not an incomplete row filter.

**Before citing either as evidence, confirm it executes the code you changed.** A change
rewriting 90.67% of the lunar theory's output bits once left the SPK digest byte-identical.

### Known bit-level drift: `wide` 0.7.33 → 1.6.1

- 6 of 21,915 analytical-oracle rows differ, **all of them the Moon** (ELP/MPP02 is the only
  place `wide` is used): ≤1 ULP longitude, ≤4 ULP latitude, ≤28 ULP speed.
- Digest before: `e35c5e3ab95dcb35a816d77313a35fa8e773bd7637568bd450e98e3eaef7bb81`
- Digest after: `f943337e6dbfe1d7881a001749009e7aa322cbbff2c4aa4e89c4e1db4c266b80`
- Digest method: per-row lines → `tr -s ' '` → `LC_ALL=C sort` → `sha256`
- Accepted deliberately: the 0.7 line is end-of-life, and the difference is far below any
  meaningful precision. The SPK path remains byte-identical.

### Known bit-level drift: the v6 precession rotation-order correction

`ddf6810` corrected the order of the four Fukushima-Williams rotations in `precession_matrix`.
`apparent_position` composes that matrix, so **every analytical row moved** and
`analytical_bit_digest` went red at v6.0.0 — it stayed red through v6.0.1, because only the
scheduled Full Validation job runs it.

- Digest before: `f943337e6dbfe1d7881a001749009e7aa322cbbff2c4aa4e89c4e1db4c266b80`
- Digest after: `d7f8b5e1cff0f88592de285293f85d52cf6b851265738f8f542e27814ce67dff`
- Attributed by running the test either side of that single commit: **passes at `ddf6810~1`,
  fails at `ddf6810`.** Assume nothing here — the same symptom would appear for a real defect.
- Measured over all 21,915 rows: longitude max **0.00014 arcsec** (21,912 rows), latitude max
  **0.0010 arcsec** (21,915 rows), distance max 1.4e-14 AU, speed max 3.8e-7 °/day.
- Accepted: the correction moves results **toward** the IAU 2006 model, and 0.001 arcsec is
  three orders of magnitude inside VSOP87A's own 2.06 arcsec mean truncation error. The SPK
  path is untouched.
- **The values in v6.0.0 and v6.0.1 are the corrected ones.** Re-pinning changed no shipped
  behaviour; it recorded a change that had already been released.


## 11. Validation Harness — Audit Backlog (2026-09-01)

**None of the following is a defect.** An independent verification pass over the validation
harness re-ran the accuracy suite and confirmed the published 0.103″ mean is correct and
reproducible for the quantity it actually measures (see the Accuracy section of `README.md`
and `metrics.json`'s `accuracy` block). The six items below are the auditor's longer-term
suggestions for making the harness itself more thorough, recorded here so the next person to
audit it does not have to re-derive them from scratch. Nothing here changes a shipped number.

1. **An adversarial sampling oracle.** `generate_horizons_oracle.py` samples uniformly at
   30-day intervals across 1900-2100. It is not weighted toward the cases most likely to
   expose a defect: station/retrograde turning points, conjunctions, perigee/apogee, longitude
   wraparound (0/360), and Chebyshev segment boundaries in the SPK kernel. A second, targeted
   fixture built around those events would be a stronger regression net than more uniform
   dates.

2. **Publish latitude, distance and full Cartesian vector residuals.** The oracle fixture
   already carries `ref_latitude`, `ref_distance` and `ref_speed` — `oracle_comparison.rs`
   deserialises all three and marks them `#[allow(dead_code)]` (see the struct fields there).
   Only the headline longitude reduction is computed and published; the other three dimensions
   are measured for nothing. This is a reporting gap, not a measurement gap: the data already
   exists in the committed fixture.

3. **A physical planet-centre comparison mode**, published separately from the existing
   barycentre comparison. `targetConvention` in `metrics.json` already documents that querying
   Horizons IDs 199/299/499/599/699/799/899/999 (physical centres) instead of the
   planetary-system barycentre IDs would inject a spurious ~0.1″ offset for the outer planets —
   the same order of magnitude as the headline mean. Right now that fact is only asserted in
   prose; a second published comparison would make it a measured, reproducible number instead.

4. **A third, independently-implemented reference path.** The current comparison is
   SpkReader/DE440s vs. Horizons/DE441. DE440 and DE441 are sibling solutions fit from the same
   underlying JPL integration and much of the same observational dataset, not two independent
   determinations of the planetary orbits — so agreement between them is weaker evidence of
   correctness than it looks. Adding SPICE (NAIF's own toolkit) or a from-scratch DE440 reader
   as a third path, built independently of both the fixture generator and `SpkReader`, would
   give a comparison that is actually independent in the way the current one is assumed to be.

5. **Percentile statistics (P50/P90/P99).** `metrics.json`'s `accuracy` block publishes mean
   and max only. A single outlier (e.g. the 1.187″ Uranus max) says nothing about the shape of
   the distribution between the mean and that max. P50/P90/P99 would show whether error is
   concentrated in a few dates/bodies or spread evenly, which the current two numbers cannot
   distinguish.

6. **A per-quantity validation status page**, rather than one general "accuracy" figure
   standing in for the whole engine. As of this writing: SPK longitude is externally validated
   against Horizons; latitude and distance are not (see item 2); house cusps are not externally
   validated at all; ayanamsha values are re-derived from primary sources but not
   cross-checked against an independent implementation. A reader who sees "0.103″" without this
   breakdown can reasonably assume it covers more of the engine than it does.

## Quick Reference: Annual Maintenance Checklist

```
[ ] Delta T — compare predicted vs. observed, update if drift > 0.5s
[ ] JPL ephemeris — check if NASA released a new DE version (DE440s coverage ends 2150)
[ ] Ayanamsha — check for Indian Astronomical Ephemeris revisions (rare), and run `python3 scripts/generate_ayanamsha.py --check-catalogue`
[ ] cargo audit — run and resolve any advisories
[ ] cargo deny check — verify no disallowed licenses crept in
[ ] Run full test suite — cargo test --workspace
[ ] Run accuracy suite — compare against fresh JPL Horizons data
```

---

## What Happens If Nobody Maintains This

If Vedaksha is abandoned entirely, here is the degradation timeline:

| Years without maintenance | Impact |
|---|---|
| 0–2 years | **No noticeable impact.** All outputs remain accurate for any astrological purpose. |
| 2–5 years | **Negligible.** Delta T prediction drifts by ~1–2 seconds. Moon position may be off by ~1 arcsecond. No astrologer would notice. Dependencies may have unpatched vulnerabilities. |
| 5–10 years | **Minor.** Delta T drift grows to 5–10 seconds. Sidereal time off by a comparable amount. House cusps could shift by a few arcseconds. Still well below astrological significance. Rust edition may require a `cargo fix --edition`. |
| 10–20 years | **Moderate for precision users.** Delta T error could reach 30+ seconds. Moon error ~15 arcseconds. Planetary positions still accurate to < 1 arcsecond. Astrologically still perfectly usable. |
| 20+ years | **Core math remains valid.** IAU precession/nutation models hold for centuries and the Rust code will still compile and produce correct results. The shipped DE440s kernel covers to **2150**; past that the SPK path has no data and `AnalyticalProvider` is the only route. |

**Bottom line:** The math does not expire. The shipped DE440s kernel covers 1850–2150, which bounds the SPK path and nothing else. The things that decay are the ones that decay in any software project — toolchains and library dependencies.

---

*© 2026 ArthIQ Labs LLC*
*Vedaksha — Vision from Vedas*
*Contact: info@arthiq.net | https://vedaksha.net*
