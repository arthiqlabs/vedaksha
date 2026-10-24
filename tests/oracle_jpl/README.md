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

Vedaksha's BUSL-1.1 position rests on a documented clean-room trail (see
[`docs/audit/`](../../docs/audit)). Horizons output is a public-domain US
Government work, so using it as reference data raises no copyleft question.
Reference values here are astronomical facts, not anyone's code.

## Contract — read before regenerating

The generator's docstring is authoritative; the essentials:

* **Time scale is UT, and Horizons' UT is not one thing.** `apparent_position`
  takes `jd` in UT1 and converts internally via `delta_t::ut1_to_tt`. Horizons
  OBSERVER tables read input JD as UT and report `Date_________JDUT`, but its
  manual (<https://ssd.jpl.nasa.gov/horizons/manual.html>, the three time
  systems, and "Universal Time (TDB -> UT Conversion)") defines UT as **UT1
  before 1962 and UTC from 1962**, and converts with the Stephenson, Morrison
  and Hohenkerk (2016) series before 1962 and with its EOP file after. Getting the time scale wrong moves
  the Moon ~35″. Between UTC and UT1 lies DUT1, up to 0.9 s, which is up to
  0.5″ of Moon longitude and cannot be recovered by any Delta T table because
  this engine is handed UT1 and carries no EOP data (`MAINTENANCE.md` §1).
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

* **1900–2025** — 15,350 comparisons, mean 0.099″, max 0.501″ (the Moon,
  1906); the worst non-Moon row is 0.265″. What this is, measured:
  * **A constant frame difference of 0.05 to 0.1″ in longitude, shared by every
    body.** Horizons' "true ecliptic and equinox of date" is built from IAU 1976
    precession (Lieske 1979), the IAU 1976 mean obliquity and the IAU 1980
    nutation, with the ICRS frame bias unapplied (the fitted tilt between the two
    frames is under 0.002″ where the bias would show 0.007″ and 0.021″) and EOP
    pole offsets from 1962;
    this engine uses IAU 2006 precession (which carries the frame bias), the
    IAU 2006 obliquity and IAU 2000 nutation. Emulating the Horizons chain on
    top of this engine's vectors collapses the 1900–1960 residual of the outer
    planets to a constant 0.066″ with a scatter of 0.001″, and the latitude
    residual (0.027″ mean) to 0.000″. It is a difference of convention, not an
    error of either side, and the 0.099″ mean is therefore a conservative
    figure for the engine's own accuracy.
  * **The Moon's extra 0.05″ of mean, and its 0.5″ maxima, are time scale**, as
    above: Horizons' UT is UTC from 1962, and its pre-1962 Delta T runs 0.8 to
    1.0 s above the USNO series in 1900–1910. The Moon's residual over
    1972–2025 is its speed times `engine Delta T − (TT − UTC)`, plus the
    frame difference above, to 0.005″ (correlation 1.000).
  * **Light deflection by the Sun** is applied (it was the whole of the 1.19″
    maximum before). Planets within two degrees of the Sun agree to 0.21″ at
    worst.
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
