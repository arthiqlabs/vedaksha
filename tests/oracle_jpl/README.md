# JPL Horizons oracle fixture

`reference_positions.json` is the independent reference that
[`oracle_comparison.rs`](../../crates/vedaksha-ephem-core/tests/oracle_comparison.rs)
measures `SpkReader` against. It is **generated, committed, and checksummed** —
regenerate with:

```bash
python3 scripts/generate_horizons_oracle.py           # rewrite + sidecar
python3 scripts/generate_horizons_oracle.py --verify   # drift check only
```

## Provenance

| | |
|---|---|
| Source | NASA/JPL Horizons System — <https://ssd.jpl.nasa.gov/horizons/> |
| Kernel | DE441 (Horizons' default) |
| Licence | Public domain (US Government work) |
| Grid | 1900-01-01 → 2100-01-01, step 30 d, 10 bodies |
| Rows | 24,350 (2,435 per body) |

Horizons serves **DE441**, while `SpkReader` reads the bundled **DE440s**. The
comparison is therefore against a genuinely independent kernel rather than a
restatement of our own input, and it exercises our DAF/SPK Chebyshev evaluation
and the whole apparent-place pipeline.

## Why Horizons and not another ephemeris library

Vedākṣha's BSL 1.1 position rests on a documented clean-room trail (see
[`docs/audit/`](../../docs/audit)). Horizons output is a public-domain US
Government work, so using it as reference data raises no copyleft question.
Reference values here are astronomical facts, not anyone's code.

## Contract — read before regenerating

The generator's docstring is authoritative; the essentials:

* **Time scale is UT.** `apparent_position` takes `jd` in UT1 and converts
  internally via `delta_t::ut1_to_tt`. Horizons OBSERVER tables read input JD as
  UT and report `Date_________JDUT`. Both sides speak UT; no ΔT correction is
  applied when building this file. Getting this wrong moves the Moon ~35″.
* **Targets are barycentres.** `Body::naif_id()` returns barycentre IDs
  (Mercury=1, Jupiter=5 …) because DE440s stores barycentre segments. The
  generator queries the same IDs. Fetching planet *centres* (199, 599 …) would
  inject a spurious ~0.1″ offset on the outer planets.
* **Coordinates are apparent, ecliptic of date** (`QUANTITIES='31'`,
  `CENTER='500@399'`), matching what `apparent_position` returns.

The generator refuses to write a fixture whose Moon-at-J2000 anchor disagrees
with the 223.3238° value independently cited in `coordinates.rs`.

## Interpreting the numbers

Residuals split at **2026**, where `delta_t.rs` stops interpolating measured
IERS values and starts extrapolating:

* **1900–2025** — 15,350 comparisons, mean 0.106″, max 1.184″. This is our
  ephemeris and apparent-place accuracy, and it is what the sub-arcsecond claim
  rests on.
* **2026–2100** — residuals grow with each body's angular rate because our
  Espenak & Meeus ΔT extrapolation and Horizons' ΔT prediction diverge (~68 s by
  2099 — the Moon picks up ~45″ of it, Pluto essentially none). That is a
  time-scale disagreement, not ephemeris error; ΔT past the measured record is
  unpredictable in principle.

`oracle_comparison.rs` asserts the two eras against separate budgets for exactly
this reason.

## Not covered here

This oracle covers body longitudes only. **House-cusp accuracy is not measured
against any reference.**

Two tests used to claim that ground — `comprehensive_comparison.rs` and
`extended_comparison.rs` — but they read `comprehensive_reference.json` /
`extended_reference.json`, fixtures generated from Swiss Ephemeris that had
never existed in this repository's git history. Both had therefore never run a
single assertion while reporting green. They were removed on 2026-07-16 rather
than revived: Horizons does not serve house cusps, ayanamshas, nutation or
tithis, so reviving them would have meant taking a Swiss Ephemeris dependency —
which is exactly what the clean-room position exists to avoid.

Covering house cusps would need a reference that is public-domain or
first-principles. That work is not done.
