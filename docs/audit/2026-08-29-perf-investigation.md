# Performance investigation — vedaksha numeric hot path

**Date:** 2026-08-29
**Method note.** Everything labelled *measured* comes from `cargo bench -p vedaksha-ephem-core` run on a MacBook Pro M5 Pro (aarch64, bench profile = fat LTO + `codegen-units=1`, load average 1.5 at start). Term counts come from the `record_count` headers of the coefficient blobs. Sparsity and trig-argument ranges were computed directly from the blob bytes. Items labelled *derived* are traced from the call graph and priced with measured per-evaluation costs; flagged as such.

**Caveat that applies to the whole report:** aarch64 NEON gives `wide::f64x4` two 2-lane vectors, not one 4-lane vector. On an AVX2 x86 build the SIMD ratios will differ. `simd_trig.rs`'s own module comment says "re-measure before trusting these on x86-64" — that applies to these numbers too.

---

## 1. Current state (verified by reading the code, not assumed)

### What already exists

v3.1.0 was a serious optimization release (`CHANGELOG-v3.1.0.md`: full natal chart 205 ms → 7.7 ms, ≈27× on an AVX2 build). It shipped: `sincos` instead of separate sin+cos, fat LTO + `codegen-units=1` + `panic=abort`, the `wide::f64x4::sin_cos` lunar kernel, ELP phase factorization (`reduce_arg`), the memoizing `CachingProvider`, batch `apparent_positions`, light-time Earth extrapolation, and time-only frame hoisting. The obvious moves are taken.

**A criterion suite exists** — `crates/vedaksha-ephem-core/benches/ephemeris.rs`, five benchmarks. It is the only bench file in the repo. Measured:

| benchmark | time | terms | ns/term |
|---|---|---|---|
| `vsop87a_planet/Mercury` | 7.20 µs | 1,485 | 4.85 |
| `vsop87a_planet/Earth` | 10.80 µs | 2,202 | 4.90 |
| `vsop87a_planet/Jupiter` | 21.56 µs | 4,434 | 4.86 |
| `vsop87a_planet/Neptune` | 12.70 µs | 2,636 | 4.82 |
| `elp_mpp02_moon` | 260.0 µs | 35,758 | 7.27 |
| `apparent_position_full_chart` (11 bodies, unbatched) | 19.31 ms | | |
| `apparent_positions_batch_chart` (11 bodies, batched) | 6.47 ms | | |
| `moon_scan_365d` | 95.04 ms | | 260.4 µs/day |

VSOP87A's cost per term is flat to within 2% across four planets spanning 3× in size — the loop is pure trig cost; term assembly (one FMA) hides entirely in its shadow.

Term inventory (from blob headers): Moon 35,758 (main 2,636 + pert 33,122); planets 29,902 (Saturn 7,512, Uranus 5,289, Mars 4,997, Jupiter 4,434, Neptune 2,636, Earth 2,202, Mercury 1,485, Venus 1,347). Total 65,660 — the Moon alone is 54%.

### Where `simd_trig` is actually wired in

Wired into **ELP/MPP02 only** — `elp_mpp02.rs:619` and `:746`. **VSOP87A does not use it**: `vsop87a.rs:79` calls scalar `libm::sincos` per term. The only `target_feature`/`std::arch`/`cfg(target_arch)` hit in the entire workspace is a *comment* in `vedaksha-ephem-core/Cargo.toml`. No runtime feature detection, no architecture-specific code.

The ELP vectorization SIMDs `sincos`, then `.to_array()`s and accumulates **scalar, in the original term order** (`elp_mpp02.rs:621-632`, `:748-755`). Summation order is deliberately preserved. That is why the measured benefit is modest, and it is the key fact for opportunity ranking.

### Parallelism

`rayon` appears only in `Cargo.lock`, as a `criterion` dev-dependency — no production code uses it. The one real parallel path is `vedaksha-vedic::muhurta::search_candidates_in_parallel`, hand-rolled with `std::thread::scope`, `MIN_CANDIDATES_PER_WORKER = 8`, per-worker memo, `available_parallelism().ok()?` → serial fallback on wasm32, and a `threaded_and_serial_scans_agree_bit_for_bit` test over 801 candidates.

### `no_std` reality

Only `vedaksha-math` is genuinely `no_std`. `vedaksha-ephem-core`, `vedaksha-astro`, and `vedaksha-vedic` each carry an explicit lib.rs note: *"Not `no_std`. This crate declared `#![cfg_attr(not(feature = "std"), no_std)]` until v5.0.0 and could not honour it."* The binding constraint on the ephemeris crates is **wasm32 (no threads)**, not `no_std`.

Toolchain pinned stable `1.97.1`, MSRV `1.89`.

---

## 2. Ranked opportunities

### #1 — Two full lunar-series evaluations per Earth position, cancelling to ~1 ULP

**Files:** `analytical/mod.rs:263-265` and `:218-232`; `coordinates.rs:68-88`.

`AnalyticalProvider::compute_state(EarthMoonBarycenter, jd)` returns `earth_to_emb(vsop_state(Earth,jd), moon_state(jd))` = `vsop_earth + moon_rel_emb/EMRAT`. That `moon_state(jd)` is a full 35,758-term `elp_geocentric`. Then `coordinates::earth_state` immediately computes `emb − moon/EMRAT`, calling ELP a *second* time — and the two terms cancel algebraically. The code says so itself (`analytical/mod.rs:216-217`): *"`earth_state` divides by the same `EMRAT`, so the two cancel exactly."*

The ELP call inside `compute_state(EMB, ·)` is an internal call. It never passes through `CachingProvider`, and it is invisible to the `chart_lunar_evals.rs` regression guard, which counts only trait-level `compute_state(Body::Moon, ·)`. Actual lunar evaluations per chart are roughly double what that guard measures.

This is specific to `AnalyticalProvider`. On the SPK path, EMB and Moon are independent kernel segments and nothing cancels.

**Impact (derived, corroborated).** Batched chart: ~12 ELP evaluations (3 timesteps × [1 EMB-internal + 1 `earth_state` Moon + 2 Moon-body light-time]) = 3.12 ms of the measured 6.47 ms. Six of the twelve are the cancellation → ~1.56 ms, ~24% of the batched chart. Unbatched: ~54 ELP evals → 14.0 ms of the measured 19.3 ms, of which 48 are cancellation. The model predicts a per-body unbatched average of 1.76 ms and gets 1.76 ms — it corroborates.

The sharpest case is `ecliptic_position(Sun, jd)` — the workhorse of `search_transits`, `search_muhurta`'s solar scan, and the sidereal paths. It pays exactly one `earth_state` and nothing else of consequence. That is ~520 µs, essentially all of it two cancelling lunar-series evaluations, to produce a position that VSOP87A-Earth (2,202 terms, ~11 µs) fully determines. Roughly 25–45× overhead on the single most-called position in the search paths.

**Fix:** give `EphemerisProvider` an `earth_state`-shaped method with a default impl doing today's `EMB − Moon/EMRAT`, overridden in `AnalyticalProvider` to return `vsop_state(Earth, jd)` directly.

**Determinism: theoretically ≲1 ULP, but need not actually move (verify, don't assume).** `(a+b)−b ≠ a` in general. Magnitude: `b/a ≈ 3.1e-5`, so ≲1 ULP of Earth's position ≈ 0.03 mm ≈ 4e-11 arcsec at 1 AU (corrected from an earlier "0.03 m" unit slip in this doc — mm, not m). `(a+b)−b` is exact whenever the addition's rounding error is strictly under `ulp(a)/2`, so the digest may not move at all in practice; measure the actual before/after digest rather than assuming a re-pin is required.

**Verify:** `analytical_oracle.rs` (21,915 rows vs Horizons) unchanged within tolerance; `analytical_bit_digest` re-pinned in a reviewed commit; a direct assertion that `earth_state` now equals `vsop_state(Earth)` bit-for-bit; `chart_lunar_evals.rs` fixed first (Wave 0) so the improvement is actually observable.

---

### #2 — ELP perturbation dot products are 70% multiplication by zero, and the fix is bit-identical

**File:** `elp_mpp02.rs:693-721` (`term_poc`).

Every one of the 33,122 perturbation terms does 13 multiply + 12 add for `phase` and again for `omega` — 861,172 multiply-adds per `elp_geocentric`. Measured density of the multiplier tables: mean 3.90 nonzero of 13 → 30.0% dense. Maximum observed 7. 97.0% of terms have ≤5 nonzeros. (Main series: 3.09 of 4, 77.2% dense — not worth touching.)

The module's own ablation (`simd_trig.rs:31`) puts "perturbation phase + ω 13-multiplier dot products" at 36.5% (35.0–38.1%) of `elp_mpp02_moon` — ~95 µs at today's 260 µs. Cutting the multiply-adds to 30% should recover ~60–70 µs, i.e. 23–27% off the lunar evaluation.

**Determinism: preserved, and provably so.** `fl(x + ±0) = x` for every finite `x` except `x = −0.0`, and `0.0 × A` is exactly `±0.0`. A partial sum can only be `−0.0` if every prior contribution was `−0.0` — which happens only for the 3 all-zero-multiplier terms, where both forms give a phase of `±0` and the resulting `±0` amplitude contribution changes nothing. A sparse rewrite that visits the nonzero multipliers in their original index order produces bit-identical phases. This makes `analytical_bit_digest` a clean pass/fail rather than a re-pin — prove it, don't assert it.

**Representation:** build a CSR-style side table at `LazyLock` load time — per-term offsets plus a flat array of `(arg_index: u8, multiplier: i16)`. Max |multiplier| measured is 75, so `i8` overflows. Prefer the load-time transform over regenerating the `.bin` blobs; the blobs are under coefficient-provenance CI gates and the win is identical.

---

### #3 — VSOP87A's scalar `sincos` is the obvious remaining SIMD target, and it is blocked on an accuracy question

`vsop87a.rs:73-82` is 29,902 terms per full pass at a measured 4.85 ns/term (~145 µs), and a chart does roughly 3 timesteps × 4 light-time iterations of them. Applying the ELP pattern (SIMD `sincos`, scalar in-order accumulation) should give ~30% off VSOP87A, ~0.5 ms off the batched chart. Not 4× — the array round-trip on both sides of the kernel and NEON's 2-lane reality are why ELP only nets ~1.5×.

**The blocker.** Measured trig argument magnitudes for `b + c*t`: max 67,831 rad over 1800–2200 CE, 1,356,541 rad over the provider's full JD range. `simd_trig::matches_libm_across_domain` validates |x| ≤ 3000. Large-argument reduction is exactly where a fast vector kernel and libm diverge most — libm does Payne–Hanek; vector kernels typically do a few-term Cody–Waite that loses digits linearly in the argument. **This must be measured before vectorizing VSOP87A.**

### #3b — and the same gap already exists for ELP, in production, today

Max |phase| actually fed to `sincos_f64x4` from `elp_mpp02.rs`, computed from the real argument polynomials:

| epoch | main | pert |
|---|---|---|
| J2000 (t = 0) | 52 | 34 |
| 2025 (t = 0.253 cy) | 20,922 | 14,259 |
| +3000 CE (t = 10) | 825,924 | 562,492 |
| −2000 CE (t = −40) | 3,303,561 | 2,249,820 |

The module doc asserts *"lunar phases stay well inside this"* (|x| ≤ 3000). That is true only within roughly a month of J2000. A contemporary 2025 chart already runs 7× beyond the validated domain; the provider's advertised range reaches ~1,100× beyond it. `analytical_oracle.rs` would catch a gross failure at its sampled dates, but it is an end-to-end accuracy test against a ~0.169″ theory floor — it cannot resolve the few-ULP-to-few-mas band where reduction typically degrades. **This is a correctness finding that happens to gate a performance item, and it has been shipping since v3.1.0.**

---

### #4 — jd-independent work recomputed every call (bit-identical, small)

- `build_args(fit)` (`elp_mpp02.rs:223`) runs on every `elp_geocentric` call and depends only on `fit`. Two variants → two `LazyLock<Args>`.
- `corrected_main_amplitude` (`:409`) is evaluated per main term per call and is a pure function of `(term, fit, is_distance)` — entirely jd-independent. 2,636 terms × ~8 flops plus a division. Precompute into `LazyLock<Vec<f64>>` per (series, fit).
- The `reduce_arg` sweeps are duplicated: `eval_main_series` reduces 4 args and `eval_pert_series` reduces 13, each called 3× per `vur_series` (V, U, r). Hoist to one computation.

Together ~3–6% of the lunar evaluation. Bit-identical. Bundle into #2's change, not worth its own task.

---

### #5 — After #1, the lunar velocity series has no consumer at all

`term_pao`/`term_poc` compute a second full dot product per term for `omega`, solely to produce the ELP velocity. Grepping every `.velocity` consumer in `crates/*/src` finds exactly two sites: `coordinates.rs:83-85` and `analytical/mod.rs:227-229` — both halves of the #1 cancellation. `retarded_geocentric` uses only `position` for the Moon; `apparent_position`'s `longitude_speed` is a central difference of positions, not a state velocity.

So once #1 lands, a position-only `elp_geocentric_position(jd)` drops the ω dot products (half of the 36.5% ablation → ~18%) plus the `dot` accumulation. Composes with #2: sparse + position-only ELP should be ~35–40% cheaper than today's. Caveat: `compute_state` returns a `StateVector`, so the trait still has to produce something — either add a position-only method or route only `retarded_geocentric`'s Moon branch to it. Position output is bit-identical.

---

### #6 — Light-time iteration re-evaluates the full series ~4× per body per timestep (NOT in scope — ratification-gated, see below)

`coordinates.rs:248-262` loops up to 10 times, each a full `compute_state(body, jd − τ)`. Using the target's own analytic velocity (already computed, currently discarded for planets) to extrapolate τ, mirroring the existing Earth-anchor extrapolation, could roughly halve iterations on the planetary half of every chart.

**Determinism: breaks it, and not at the ULP level.** The second-order term is ½·a·τ²: ~1 mas for the Sun, ~70 µas for Neptune — inside the theory's own ~0.169″ floor, but it changes every published number. **This item requires explicit ratification before implementation — it is intentionally excluded from this plan's Wave 0-2 scope.**

---

### #7 — Build/distribution gaps: free wins, no numeric-core change

**(a)** No `.cargo/config.toml` anywhere in the repo. `-C target-cpu=x86-64-v3` is set only in `Dockerfile` and `release.yml`'s `build-mcp` matrix. `.github/workflows/benchmarks.yml` runs `cargo bench --workspace` with no RUSTFLAGS — the tracked performance trend measures a configuration the project does not ship.

**(b)** The Python binding's wasm blob is built without `+simd128`. `release.yml`'s `wasm` job sets `RUSTFLAGS: "-C target-feature=+simd128"` for `vedaksha-wasm`, but `bindings/python/scripts/build-wasm.sh:15` runs a bare `cargo build --release --target wasm32-unknown-unknown -p vedaksha-py-engine` with no RUSTFLAGS — and both `ci.yml`'s `python` job and the Makefile's `wasm` target call that script. Per `release.yml`'s own comment, without `+simd128` `wide`'s `f64x4` "falls back to scalar." **The lunar kernel runs scalar inside the shipped Python package.** One line. Verify with a before/after measurement through the package and a digest check — the scalar and simd128 backends may not agree bit-for-bit.

**(c)** Runtime dispatch (`is_x86_feature_detected!` + a `#[target_feature]` kernel clone) — flagged, not recommended for this pass: two code paths whose float results must be *proven* identical is a real verification cost against modest reach. NOT in scope for Wave 0-2.

---

### #8 — Parallelism: `search_transits`'s coarse scan is the one gap

`search_transits`'s `coarse_scan` (`transits.rs:129-144`) walks the grid sequentially — 366 points for a 1-year, 1-day-step search, per transiting body. The outer loop over `transiting_bodies` is fully independent; the grid points are independent of one another. Embarrassingly parallel, currently serial.

**Recommendation: copy `muhurta.rs`'s `std::thread::scope` pattern, not `rayon`.** `vedaksha-astro` also compiles into `vedaksha-wasm` (no threads), so it needs the same `available_parallelism().ok()?` → serial-fallback shape, modeled on `search_candidates_in_parallel`'s existing `threaded_and_serial_scans_agree_bit_for_bit` test.

---

## 3. Dead ends and negative findings (do not re-investigate)

1. "Use SIMD" as generic advice is already spent — v3.1.0 did it and measured 27×.
2. `std::simd` is unavailable (stable 1.97.1, MSRV 1.89; portable SIMD is nightly). `wide` (already a dependency, `no_std`-safe) is the correct choice and already in use.
3. No faster trig library preserves bit-determinism — every implementation rounds transcendentals differently. Not worth further search.
4. Lane-parallel SIMD accumulation (4 independent accumulators + horizontal sum instead of scalar in-order) would give another ~1.5-2× on the lunar kernel but moves every output bit via reassociation. **This is a policy question — is bit-determinism a product guarantee or an engineering convenience? — not a technical one, and it must not be done incidentally. NOT in scope for Wave 0-2; requires explicit ratification, same as #6.**
5. `CachingProvider`'s `HashMap` is not a cost — single-digit µs against 6.47 ms.
6. Memory bandwidth is not the constraint — 2.44 MB of lunar tables at 260 µs/eval is ~9.4 GB/s, far below sustained rate.
7. "`round_int` on AVX2" (an old backlog note) appears to be stale — no such symbol, no such note anywhere in this workspace. Do not chase it.
8. `search_muhurta` needs no parallelism work — already threaded, already determinism-tested, correctly falls back on wasm32.
9. The scan paths already avoid the 3× central-difference cost — both MCP handlers deliberately use `ecliptic_position` (1 eval) over `apparent_position` (3 evals).

---

## 4. Scope for this plan: Wave 0 through Wave 2, explicitly excluding two ratification-gated items

**In scope:**
- **Wave 0** — verification only, no behavior change: extend SIMD domain validation, fix the lunar-eval counting guard, add bench RUSTFLAGS.
- **Wave 1** — free/near-free wins: #1 (EMB/Moon cancellation), #7b (missing +simd128), #2+#4 (sparse ELP dot products, bundled).
- **Wave 2** — needs the Wave 0 answer to proceed, but proceeds now on explicit direction: #5 (position-only ELP), #8 (threaded search_transits), #3 (VSOP87A vectorization — CONDITIONAL: if Wave 0's extended validation shows the existing SIMD trig kernel is NOT accurate enough at production's real argument range, this task converts into fixing that accuracy defect in the ALREADY-SHIPPED ELP kernel rather than extending SIMD to VSOP87A — do not proceed with new vectorization work on top of an unvalidated kernel).

**Explicitly OUT of scope, not to be implemented without separate ratification:**
- #6 — light-time target extrapolation (changes every published position at the sub-mas level).
- Dead-end #4 (lane-parallel SIMD accumulation) — changes every output bit via reassociation.
- #7c — runtime SIMD feature dispatch (two code paths needing bit-identical proof; not pursued this pass).

**Verification floor for anything touching the numeric core:** `analytical_oracle.rs` (21,915 rows vs the Horizons fixture) within existing tolerance, plus `analytical_bit_digest` — unchanged where the analysis says bit-identical (#2, #4, #5-position), deliberately re-pinned with a recorded reason where it does not (#1). Re-measure the criterion suite before and after under the shipped RUSTFLAGS, on an unloaded machine (never benchmark under load, per this project's own principles).
