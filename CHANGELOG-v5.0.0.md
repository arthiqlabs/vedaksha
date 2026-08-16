# Vedākṣha v5.0.0 — The ascendant, the vara, and a sunrise the engine never had

**Release date:** 2026-08-16

A major release, and **unlike v4.0.0 it changes numerical results.** Three long-standing
defects are corrected. Two of them move output that every existing caller already has:

- **every chart's ascendant, MC and house cusps shift** (measured 5.18′–30.23′, varying
  with latitude), and
- **sidereal charts change house placement for roughly 77% of planets.**

If you store computed charts, they will not match a recomputation under v5. Read
[Breaking](#breaking) before upgrading.

To be precise about what this is and is not: v5 **corrects answers that were wrong**. It
does **not** make the ephemeris more accurate. Residuals against the JPL Horizons oracle are
unchanged — mean 0.880″ overall, 0.106″ in the measured-ΔT era.

One small caveat, disclosed rather than glossed. The `wide` SIMD dependency moved from 0.7.33
to 1.6.1 (its 0.7 line is end-of-life), and that is **not** bit-identical in the lunar theory:
**6 of 21,915 analytical-oracle rows differ, all of them the Moon** — at most 1 ULP in
longitude (2.29 × 10⁻⁵ µas), 4 ULP in latitude, 28 ULP in speed. That is far below any
meaningful precision and moves no accuracy figure, but the SPK path remains byte-identical and
we would rather state the difference than let a bit-reproducibility claim quietly stop being
true. The analytical digest is pinned by `analytical_bit_digest.rs` so any future drift is
caught deliberately.

---

## Two corrections that move every chart

**The ascendant was built on the wrong time scale.** Sidereal time measures Earth's rotation,
so it is defined on UT1 — the engine converted to Terrestrial Time first, adding ΔT worth of
rotation where it should have removed it. The RAMC error is uniform (~0.29° today); the
resulting ascendant error is not, because it varies with latitude — measured between 5.18′ and
30.23′ across three test charts. Every ascendant, MC and house cusp the engine has returned
carries it.

**Sidereal charts mixed two coordinate frames.** `compute_chart` rotated the planets by the
ayanamsha and left the house cusps tropical, then placed the rotated planets against the
unrotated cusps. With Lahiri (~24.2°) that is 0.81 of a house: **77.4% of placements (669 of
864 measured) fell in the wrong bhava.** The default `Tropical` configuration was unaffected,
which is precisely why it survived this long.

Both are detailed under [Breaking](#breaking).

## And a vara is not a weekday

`weekday_from_jd` returned the **Universal Time** calendrical weekday. A *vara* — the Vedic
weekday — runs from local sunrise to local sunrise, so it depends on where the observer is
standing. The UT day boundary falls at a different local clock time at every longitude, so
between that boundary and local midnight the engine reported the following day's weekday.

Worse, the signature took only `jd`. There was no parameter through which a caller could
supply latitude or longitude, so **every consumer received the UT answer and had no way to
correct it.** It reached all four shipped surfaces.

The engine could not have computed the right answer: it had **no sunrise routine at all**.

v5 adds one, and derives the vara from it.

---

## Breaking

**`weekday_from_jd` → `ut_weekday_from_jd`.** Renamed deliberately and documented as the UT
calendrical weekday, explicitly *not* a vara. The old name is what invited the misuse — a
downstream integrator reported having copied the same derivation into five places because it
looked like a settled primitive. A rename is the one change an existing caller cannot
silently ignore.

**`compute_panchanga` now requires `latitude` and `longitude`.** A vara without an observer
is meaningless. `elevation_m` and `tz_offset_minutes` are optional additions.

**`assess_muhurta` now takes the vara; `search_muhurta` takes the observer.**
`assess_muhurta(jd, moon, sun, weekday)` no longer derives the weekday itself — a vara needs
an observer and that function has none, so the caller must supply it. `search_muhurta` gained
the observer parameters and derives it. No backwards-compatibility wrapper was added, on
purpose: a shim would have defaulted to the UT weekday, which is the exact defect being
removed. A compile error is better than a silently wrong vara.

**`kalam_windows` returns a `KalamReckoning`** — carrying the vara, the two windows and
`from_sunrise` — so the vara and the windows derive from one sunrise rather than two.

**RAMC is now built from UT1, not TT.** Sidereal time measures Earth's rotation and is
defined on UT; the engine was converting to Terrestrial Time first, adding ΔT worth of
rotation instead of removing it. This moves **every** ascendant, MC and cusp. The RAMC shift
is uniform (~0.29° today); the resulting ascendant shift is not — it varies with latitude,
measured between 5.18′ and 30.23′ on three test charts.

**Sidereal charts no longer mix coordinate frames.** `compute_chart` rotated planets by the
ayanamsha but left the house cusps tropical, then placed the rotated planets against the
unrotated cusps. With Lahiri (~24.2°) that displaced planets by 0.81 of a house: **77.4% of
placements (669 of 864 measured) were wrong.** The default `Tropical` configuration was
unaffected, which is why it survived. Whole-Sign is re-anchored on the rotated ascendant
rather than rigidly rotated, because its cusps *are* sign boundaries and move with the frame.

**`julian_day` is documented as UT1, not TDB.** Six tool schemas said TDB while the code
treated the value as UT1. A caller obeying the published schema was supplying the wrong time
scale. `SpkReader::compute_state` and `Vedaksha.state_vector` genuinely do take TDB and are
now labelled as such.

**MSRV 1.85 → 1.89.**

---

## Added

**A sunrise/sunset primitive** (`vedaksha-astro::riseset`): `previous_rise`, `next_rise`,
`rise_set`, `sun_rise_set`, `horizon_dip_deg`. Meeus Ch. 15's analytic hour-angle method with
a bisection fallback, anchored on the instant rather than a calendar boundary so containment
holds by construction. Polar day and night return `None` rather than a fabricated instant.

**Rahu Kalam and Gulika Kalam as real time windows** — `(start_jd, end_jd)` rather than a
slot index a caller could not place on a clock. Gulika Kalam was previously computed and
tested but reachable from **no shipped surface at all**, while the README advertised it.

**`vara.from_sunrise`** — a boolean distinguishing a genuine sunrise-reckoned vara from the
local-civil-weekday fallback used inside the polar day and night, where the sunrise-to-sunrise
reckoning is undefined. Previously that substitution was silent.

**Two new MCP tools:** `compute_synastry` and `compute_composite`. Both pair grahas by name
rather than by array position, which for `compute_composite` also removes a panic that
mismatched input lengths would otherwise have triggered.

---

## Performance

Measured on aarch64 (Apple Silicon), `--release`. These are latency improvements only; no
numerical result changes.

| Operation | Before | After | |
|---|---|---|---|
| `compute_panchanga` | 203.5 ms | 7.75 ms | **26.2×** |
| `search_transits` | 450.06 ms | 151.77 ms | **2.97×** |
| `compute_transit` | 51.52 ms | 25.30 ms | **2.04×** |
| `compute_natal_chart` | 25.81 ms | 12.98 ms | **1.99×** |
| 8 concurrent requests | 1180 ms | 203 ms | **5.80×** |
| Served muhurta request | 1483.0 ms | 785.6 ms | **1.89×** |
| Muhurta scan, 365-day | 3754.1 ms | 458.2 ms | **8.19×** |

`compute_panchanga` came from replacing a 5-minute brute-force sunrise scan (~660 ephemeris
evaluations) with the analytic bracket (~25). The chart and transit gains came from computing
same-instant body sets through `apparent_positions`, which hoists the per-timestamp
nutation·precession frame instead of rebuilding it per body. The muhurta gains came from
evaluating each body once per instant instead of up to four times, and from scanning candidate
days across threads. Concurrency came from serving HTTP on a bounded worker pool rather than a
serial accept loop.

Every one of these is **bit-identical** to the previous implementation — verified by
comparing served JSON byte-for-byte, and by an agreement sweep against the retained
brute-force scanner whose maximum disagreement was one ULP.

**Where these numbers come from,** since this project prefers a stated provenance to an
unattributed figure. The batch-versus-per-body comparison is reproducible in-repo:
`cargo bench -p vedaksha-ephem-core` (`benches/ephemeris.rs`), with its bit-identity asserted
by `batch_matches_per_body_bit_for_bit` in `coordinates.rs`. The agreement sweep is
`analytic_rise_agrees_with_the_scan_oracle_dense`, gated behind the non-default
`derivation-sweeps` feature with its recorded output beside it. The remaining
figures — served `compute_panchanga`, `search_transits`, the muhurta request and the
concurrency number — were measured against the running HTTP surface on aarch64 in
`--release` during development; they are not regenerated by a committed benchmark, and the
ascendant-shift range and the 669-of-864 placement count are development measurements in the
same sense. Treat those as reported, not as artifacts you can re-run from this repository.

---

## Notes

The brute-force sunrise scanner is retained rather than deleted, in two roles. Its
oracle-comparison harness is `#[cfg(test)]` and exists to check the analytic path against it —
that harness caught three defects during development. The scan routine itself ships, and is
the fallback the production path takes when iteration exhausts its budget or leaves the
analytic method's domain, so a hard case is answered correctly rather than reported as no
sunrise.

Above 89° latitude the analytic path is routed through that scanner, because near the pole a
sunrise can be attributed to the wrong rotation. Below that threshold the fast path is
untouched — measured at zero routings across five representative cities.
