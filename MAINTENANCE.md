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

The one leap-second table in the repository is test data, not engine: `crates/vedaksha-ephem-core/tests/common/tai_utc.rs` lets the Horizons oracle tests convert Horizons' UTC time tags to TT (§3, step 4). Nothing in a shipped crate reads it.

If your application converts civil UTC to UT1 before calling, that conversion is where leap seconds belong, and keeping the table current is your side of the boundary. IERS Bulletin C announces insertions about six months ahead. No leap second has been inserted since 2016-12-31, and the 27th CGPM (2022, Resolution 4) resolved to stop inserting them by 2035.

---

## 3. Delta T (TT − UT1)

**What:** The difference between Terrestrial Time (TT, uniform atomic time) and UT1 (Earth rotation time). The values are measurements to 2026-10 (the Stephenson, Morrison and Hohenkerk spline before 1962.0, the USNO rows from then), USNO's own predictions to 2033.75, and an extrapolation after that. `delta_t.rs` has the table of eras and the joins between them.

**Why it matters:** Converting between the Julian Day used in ephemeris computation (TDB/TT) and the UTC/UT1 used by humans requires Delta T. The extrapolation beyond USNO's predictions drifts over time as Earth's rotation deviates from the model.

**Impact if neglected:** The prediction error grows roughly quadratically. After 5 years without update, the error is typically < 2 seconds. After 20 years, it could be 10–30 seconds. For the Moon (fastest-moving body), 1 second ≈ 0.5 arcsecond.

**How to update:**
0. The pre-1962 Delta T is the Stephenson, Morrison and Hohenkerk (2016) spline, Table S15 of the electronic supplement to doi:10.1098/rspa.2016.0404 (Creative Commons Attribution 4.0; zip `rspa20160404supp2.zip`, sha256 and licence in `delta_t_table.rs`). It is static and needs refreshing only if a revision is adopted: JPL Horizons cites an "Addendum 2020" (Morrison, Zawilski, Hohenkerk and Stephenson, *Proceedings of the Royal Society A*, vol. 477, issue 2246, 2021-Feb-17) that has not been taken up here. Keep the zip beside the USNO files when regenerating; the generator reads `Table-S15.txt` from it.
1. Download the three USNO files, each into a directory of its own (they are untrusted input; never run anything from inside that directory):
   - `https://maia.usno.navy.mil/ser7/deltat.data`, monthly Delta T since 1973 (gains a row each month)
   - `https://maia.usno.navy.mil/ser7/historic_deltat.data`, half-year Delta T from 1657 (static; the rows from 1962.0 to 1973.0 are used)
   - `https://maia.usno.navy.mil/ser7/deltat.preds`, quarterly predictions to about seven years ahead (the rows after the last monthly row are used)
2. Regenerate the table: `python3 -I scripts/generate_delta_t_table.py --smh-zip <path> --historic <path> --monthly <path> --preds <path> --retrieved <YYYY-MM-DD>`. It writes `crates/vedaksha-ephem-core/src/delta_t_table.rs`, the spline, the measured knots and the predicted knots, with the sha256 of every input in its header. Update the retrieval date, row counts and the first and last dates in `delta_t.rs`'s module notes and in `DATA_PROVENANCE.md`. Beyond the last prediction `delta_t.rs` continues with the Espenak & Meeus expressions, shifted to meet it; nothing else needs changing.
3. Run `cargo test -p vedaksha-ephem-core --lib delta_t::`. `rows_of_the_usno_files_are_hit_exactly_at_their_own_instants` pins the last row, so update it to the new last row. If a leap second has been inserted since 2017, add it to the list in `monthly_rows_stay_within_the_leap_second_bound_of_utc` and extend that test's range.
4. Re-run the two oracles (`oracle_comparison.rs`, `analytical_oracle.rs`) and re-pin `metrics.json`. Only the rows before 1962-01-20 exercise the pre-1962 spline (the rest of the table is the USNO rows from 1962.0): from then Horizons' time tag is UTC, and the oracles convert it to TT with the leap-second count (`tests/common/tai_utc.rs`, from USNO `tai-utc.dat`; regenerate with `scripts/generate_tai_utc_table.py` if a leap second is ever inserted) and read the engine in TT. After 1962 the table's accuracy is covered by the `delta_t` unit tests, not by the oracles. Against Horizons before 1962 the Moon is the instrument: 1 s of Delta T is 0.55 arcsec of Moon longitude.

**Frequency:** Once a year keeps charts cast for the current year on measured values (the table's measurements end at the month it was made; USNO's predictions after it carry the error the file states, up to 1 s by 2033). Every 5 years is sufficient for astrological use elsewhere; every 1–2 years for research-grade.

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

**Push CI invokes no generator at all.** `ci.yml` runs only the guard scripts in `scripts/`, never a coefficient generator. The VSOP87A and
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
  three orders of magnitude inside the analytical path's mean error against Horizons (2.06
  arcsec then; the observer fix of 2026-08-20 showed that figure was a defect of ours and not
  truncation, and it is 0.160 arcsec at v10.0.0). The SPK path is untouched.
- **The values in v6.0.0 and v6.0.1 are the corrected ones.** Re-pinning changed no shipped
  behaviour; it recorded a change that had already been released.


### The VSOP87A frame rotation — `vedaksha-ephem-core::analytical`

`VSOP87A_TO_EQUATORIAL` is the matrix the VSOP87 authors publish for taking VSOP87A's
dynamical ecliptic and equinox of J2000 to the equatorial frame (FK5 J2000). It replaced
`Rx(84381.448″)` alone, which the Moon still uses (`ecliptic_to_equatorial`).

- **Value:** the matrix printed to twelve decimals in the authors' `vsop87.doc`, section
  "REFERENCE SYSTEM" (Bretagnon and Francou, distributed with the series at
  `https://ftp.imcce.fr/pub/ephem/planets/vsop87/`). It equals `Rz(−0.0990″) · Rx(84381.4091″)`.
  The authors state the matrix, not those two angles;
  `tests::vsop87a_matrix_is_rz_phi_rx_eps` pins that the matrix reproduces from them.
- **Established by:** reading the authors' documentation, then measuring against DE440s over
  1900 to 2100 (geometric heliocentric vectors, no light time or aberration). The mean angular
  residual went, for example, Mercury 0.102″ to 0.021″, Venus 0.090″ to 0.014″, Jupiter 0.236″
  to 0.170″. The full table is in the doc comment on the constant.
- **Already answered, do not re-search:** (1) VSOP2013's rotation constants (`eps = 84381.41136`,
  `phi = −0.05188` arcsec) belong to a theory fitted to a different integration and are not
  VSOP87A's. (2) Nothing read for this change gave a rotation from FK5 to the ICRS for a theory
  fitted to DE200, so none is applied; Venus and Earth still sit about 16 mas from DE440s, which
  is evidence and not a correction. (3) The outer planets' remaining residual is not the 1e-7 AU
  truncation: the untruncated series gives the same residuals to four decimals. The authors
  fitted the theory to DE200 (`vsop87.doc`, "PRECISION").
- **Invalidated by:** the theory, or its authors' published frame relation, changing. Nothing else.

### ELP/MPP02 and VSOP87A phases by angle addition — `analytical/phase_graph.rs`, `analytical/vsop87a.rs`

Both series build a term's sine and cosine from shared ones instead of calling a sine and
cosine per term.

- **Method, lunar:** each term's phase is a sum of multiples of a few fundamental arguments.
  The sine and cosine of the fundamentals are evaluated once, and every term's is built from
  them by the addition formulas through a prefix tree shared between terms (`phase_graph`).
- **Method, VSOP87A:** `cos(B + C t) = cos B cos(C t) − sin B sin(C t)`, with `cos B` and
  `sin B` taken once when the tables are built, and one sine and cosine per distinct frequency
  (Earth: 624 distinct among 2,202 terms).
- **Output movement, measured** by dumping every row before and after: the lunar series moves
  by at most 7.5e-7″ over 1610 to 2390 (1.2e-5″ over the whole supported range, near −1340);
  VSOP87A by at most 5.7e-8″ over 1900 to 2100. Both are far inside the series' own
  arcsecond-level fit, so no accuracy figure moves.
- **Speed, measured:** one ELP/MPP02 evaluation 188 µs to 56 µs; VSOP87A Mercury, Earth,
  Jupiter, Neptune 4.6, 6.1, 12.8, 6.4 µs to 1.8, 2.9, 6.0, 3.5 µs (`CHANGELOG-v10.0.0.md`).
- **Bits:** the lunar series no longer calls `simd_trig::sincos_f64x4`, so its bits are the same
  on aarch64 and x86_64. VSOP87A still calls that kernel, so `analytical_bit_digest` keeps one
  pin per architecture.
- **Invalidated by:** a change to the series tables themselves (a new ELP/MPP02 or VSOP87 fit).
  Nothing else.

### Fused multiply-add — measured and rejected

- **Result:** no gain outside the noise, so the `analytical::fused` gate was removed
  (`a65bae5`; `git show -s a65bae5` has the measurement).
- **Established by:** the same code with plain multiply-adds against the fused form,
  interleaved, on aarch64: no difference outside the noise of a loaded machine (about 5
  percent either way). The change FMA would have made to the output was at most 5.9e-9″ for
  the Moon and 8.2e-11″ for the planets.
- **Why it stays rejected:** the main consumer is x86_64 without FMA, so the gate would add
  output that depends on the target for no measured gain. Without it the lunar series uses
  only IEEE multiply and add and pure-Rust libm.
- **Invalidated by:** a measured gain outside the noise on a target consumers actually run.
  Even then the price is target-dependent output.

### Delta T sources and joins — `vedaksha-ephem-core::delta_t`

`delta_t.rs`'s module notes are the record; this is the pointer. Read "Where the values come
from", "Why the handover is at 1962.0" and "The joins" there, and `DATA_PROVENANCE.md` for the
sources, retrieval dates and checksums. In short: before −720 the Espenak and Meeus (2006)
long-term parabola; −720 to 1962.0 the Stephenson, Morrison and Hohenkerk (2016) spline
(Table S15); 1962.0 to 1973.0 the USNO half-year rows; 1973-02 to 2026-10 the USNO monthly
rows; to 2033.75 USNO's predictions (`deltat.preds`); after that the Espenak and Meeus
expressions shifted to meet the last prediction, so there is no step.

- **Already answered, do not re-search:** (1) Why the USNO rows, not the spline, from 1962.0:
  SMH2016 itself uses the atomic-time data over 1962 to 2015 only as a control, and USNO states
  an error of 0.001 s from 1962.0. (2) The joins: every one is continuous except two, a step of
  −74.5 s at −720 and +0.371 s at 1962.0. They are stated, not smoothed, because there is no
  published rule for blending these sources and none was invented. (3) Horizons has the same
  step between its own pre-1962 series and its Earth-orientation file, but at 1962-01-20 where
  the engine hands over at 1962.0. The 19 days between are where the two disagree by 0.371 s,
  which is 0.2″ of Moon longitude, and the oracle's worst SPK row sits in them.
- **Held by:** `delta_t_is_continuous_at_every_join` and `delta_t_has_no_unstated_step`.
- **Invalidated by:** adopting a revision of the spline (JPL Horizons cites an "Addendum 2020",
  which §3 step 0 records as not taken up), or a published rule for blending the sources.
  Refreshing the USNO rows is routine (§3), not an invalidation.

### The Horizons convention floor — `oracle_comparison.rs`, `analytical_oracle.rs`

After 1962-01-20 every body on the SPK path sits at 0.047″ to 0.054″ mean against Horizons
(Pluto 0.054″, at most 0.062″). Before it, the bodies sit at 0.138″ to 0.146″ mean on the SPK
path and 0.131″ to 0.153″ on the analytical path.

- **Cause:** Horizons builds its ecliptic of date from IAU 1976/1980 precession and nutation;
  this engine uses IAU 2006/2000B. The offset is shared by every body, so it is a difference of
  convention and not engine error. It is **not emulated**: the engine implements the newer
  models, and the offset is the other side's older convention.
- **Established by:** the report both oracle tests print, split at 1962-01-20 and by body; the
  ceilings in `oracle_comparison.rs` (0.10″ maximum and 0.07″ mean after 1962) are set from it.
- **Not part of the floor:** the worst SPK row, the Moon at 1962-01-06 (0.272″), is the Delta T
  handover of the previous entry (0.371 s is 0.2″ of Moon longitude). Without it the worst row
  is 0.251″ (Pluto).
- **Invalidated by:** Horizons changing its precession and nutation model, or this engine
  changing its own.

### Known bit-level drift: v10.0.0 (Delta T, nutation, deflection, VSOP87A frame, angle addition)

Every analytical row moved. The causes are six deliberate changes in one release, listed with
their measured effect in the doc comment on `EXPECTED_DIGEST` in
`crates/vedaksha-ephem-core/tests/analytical_bit_digest.rs` ("Why this moved at v10.0.0").

| Pin | v9.4.1 | v10.0.0 |
|---|---|---|
| `analytical_bit_digest`, aarch64 | `b8aecba3f726ebd2a516b5a63aee5a4647f45942bf3be6cc3e04c6b8d6701571` | `732350622f43d4ee27f1a2d9e5288f2b503353cb1df59ee0c1b367168ab86298` |
| `analytical_bit_digest`, x86_64 | `64dcc4cf991cf5b087dde0492882157b64503d87468ddd51922e59cb2afa7ab2` | `392fb9e71c3735dfda1fafc1798fd3dc094b6f93f2153f8c124fdf5c8468f3aa` |
| `lunar_series_bit_digest`, aarch64 | `e8b21c8863fa0065e125e16a4b3396371f7de6a386d19f35f233c07ac5e1394b` | `181ab376450f17b5986e682b665cf8a217f5f137fcd4fec2e439a057e516313b` |
| `lunar_series_bit_digest`, x86_64 | `a2af33c87957fd15249d6f5b714749a8623f0ad4593a1fbc1ce0d4b51c0cccfe` | `181ab376450f17b5986e682b665cf8a217f5f137fcd4fec2e439a057e516313b` |
| lunar series, position-only, aarch64 | `6edec0b1dd362d83a0a31e993b940b1119d8a49a9062a4afbf9ecd53ec255e93` | `53a769cb566760984de44bf9263e39dedd4b736d98bf349f2fe4cca519caa2e2` |
| lunar series, position-only, x86_64 | `57821af832a46e6de2cb9c6a20d1bbee6c4e654f55d7ec950850c8a699784363` | `53a769cb566760984de44bf9263e39dedd4b736d98bf349f2fe4cca519caa2e2` |

- **The lunar series is now one value on both architectures**, because it no longer calls the
  SIMD kernel whose rounding differs between AVX2 and NEON. The analytical provider still has
  one per architecture, because VSOP87A still calls it.
- **x86_64 digests can be measured on an Apple Silicon machine**, under Rosetta, with no remote
  box: `rustup target add x86_64-apple-darwin`, then
  `cargo test --release --target x86_64-apple-darwin -p vedaksha-ephem-core --test analytical_bit_digest --test lunar_series_bit_digest -- --include-ignored`.
  That route was validated before the v10.0.0 pins were taken from it: built at the v9.4.1 tag
  it reproduced all three of that release's x86_64 pins exactly. Re-validate it the same way
  after a toolchain or Rosetta change before trusting it for a new pin.

## 11. Validation Harness — Audit Backlog (2026-09-01)

**None of the following is a defect.** An independent verification pass over the validation
harness re-ran the accuracy suite and confirmed the published mean (0.103″ then, 0.096″ at
v10.0.0) is correct and reproducible for the quantity it actually measures (see the Accuracy section of `README.md`
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
   and max only. A single outlier (the 1.187″ Uranus max at the time of the audit; the 0.272″
   Moon row at 1962-01-06 at v10.0.0) says nothing about the shape of the distribution between
   the mean and that max. P50/P90/P99 would show whether error is
   concentrated in a few dates/bodies or spread evenly, which the current two numbers cannot
   distinguish.

6. **A per-quantity validation status page**, rather than one general "accuracy" figure
   standing in for the whole engine. As of this writing: SPK longitude is externally validated
   against Horizons; latitude and distance are not (see item 2); house cusps are not externally
   validated at all; ayanamsha values are re-derived from primary sources but not
   cross-checked against an independent implementation. A reader who sees "0.096″" without this
   breakdown can reasonably assume it covers more of the engine than it does.

## Quick Reference: Annual Maintenance Checklist

```
[ ] Delta T — re-download the three USNO files and regenerate the table (§3); USNO's predictions end at 2033.75
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

The Delta T rows are order-of-magnitude estimates, not measurements. Since v10.0.0 the table carries USNO's own predictions to 2033.75, each with the error the file states (0.33 s at 2027, 0.77 s at 2030, 1 s from 2032.75), and `delta_t.rs` treats everything after 2033.75 as a guess.

**Bottom line:** The math does not expire. The shipped DE440s kernel covers 1850–2150, which bounds the SPK path and nothing else. The things that decay are the ones that decay in any software project — toolchains and library dependencies.

---

*© 2026 ArthIQ Labs LLC*
*Vedaksha — Vision from Vedas*
*Contact: info@arthiq.net | https://vedaksha.net*
