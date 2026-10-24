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

Horizons serves **DE441**, while `SpkReader` reads **DE440s** (downloaded by `scripts/download_de440s.sh`; it is not in the repository). The
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

* **Time scale: Horizons' UT is not one thing.** Its manual
  (<https://ssd.jpl.nasa.gov/horizons/manual.html>, the three time systems, and
  "Universal Time (TDB -> UT Conversion)") defines UT as **UT1 before 1962-01-20
  and UTC from then**, and converts with the Stephenson, Morrison and Hohenkerk
  (2016) series before 1962 and with its EOP file after. The engine is handed
  UT1 and carries no UT1 − UTC (`MAINTENANCE.md` §1), so feeding it a UTC tag as
  UT1 compares two different instants, up to 0.9 s or 0.5″ of Moon longitude
  apart, which no Delta T table can remove. The oracle tests therefore convert
  the rows from 1962-01-20 to TT with `TT = UTC + 32.184 s + (TAI − UTC)`
  (`common::horizons_jd_tt`) and read the engine with `ecliptic_position_tt`;
  the earlier rows go through `apparent_position` and the engine's Delta T. The
  leap-second counts are committed as data in
  `crates/vedaksha-ephem-core/tests/common/tai_utc.rs`, generated from USNO
  `tai-utc.dat` by `scripts/generate_tai_utc_table.py`, and
  `oracle_time_scale.rs` checks the conversion against Horizons' own `TDB-UT`
  to 1.7 ms, the TDB − TT term. Leaving the leap seconds out puts the Moon 23.4″
  out.
* **Targets are barycentres.** `Body::naif_id()` returns barycentre IDs
  (Mercury=1, Jupiter=5 …) because DE440s stores barycentre segments. The
  generator queries the same IDs. Fetching planet *centres* (199, 599 …) would
  inject a spurious ~0.1″ offset on the outer planets.
* **Coordinates are apparent, ecliptic of date** (`QUANTITIES='31,20'`,
  `CENTER='500@399'`; 31 is the longitude and latitude, 20 the range and range rate), matching what `apparent_position` returns.

The generator refuses to write a fixture whose Moon-at-J2000 anchor disagrees
with the 223.3238° value independently cited in `coordinates.rs`.

## Interpreting the numbers

The report splits at **2026** only to keep the published count and definition
(`MEASURED_DT_JD`). From 1962-01-20 both sides of that cut are compared in TT,
so they are like for like.

* **1900–2025** — 15,350 comparisons, mean 0.096″, max 0.272″ (the Moon,
  1962-01-06); by body the mean runs from 0.092″ to 0.100″. What this is:
  * **A frame offset shared by every body.** After 1962-01-20 every body sits at
    0.047″ to 0.054″ mean and at most 0.062″; before it, at 0.138″ to 0.146″.
    Horizons builds its "true ecliptic and equinox of date" from IAU 1976
    precession and obliquity and IAU 1980 nutation with the frame bias
    unapplied; this engine uses IAU 2006 precession (which carries the frame
    bias), the IAU 2006 obliquity and IAU 2000B nutation. It is a difference of
    convention, not an error of either side, and it is not emulated, so the
    mean is a conservative figure for the engine's own accuracy.
  * **The largest row is the Delta T handover, not the frame.** The Moon at
    1962-01-06 lies in the 19 days between 1962-01-01, where the engine hands
    over from the SMH2016 spline to the USNO rows, and 1962-01-20, where Horizons
    does. The two series differ by 0.371 s across it, which is 0.2″ of Moon
    longitude (`delta_t.rs`, "The joins"). Without that row the worst is 0.251″
    (Pluto), and the Moon's worst is 0.225″ (1900-05-31), the same as every other
    body's.
  * **Light deflection by the Sun** is applied. Before it was, the maximum was
    1.187″ (Uranus, 0.36° from the Sun). `solar_deflection_oracle.rs` holds the
    155 planet rows within two degrees of the Sun to the same agreement as every
    other row. Where the ray crosses the solar disc Horizons applies none, and
    neither does the engine.
* **2026–2100** — worst row 0.189″ (the Moon, 2099-10-04): the frame offset plus
  a small drift between the DE440s the engine reads and the DE441 Horizons
  serves. Before v10.0.0 these rows were compared through each side's own Delta T
  prediction and the Moon picked up ~45″ in 2099; that was one prediction
  against another, not ephemeris error, and it is gone with the mismatch that
  produced it. It also means this oracle says nothing about the accuracy of Delta
  T after 2033.75, which `delta_t.rs` states is a guess.

`oracle_comparison.rs` asserts separate ceilings for 1900–2025 (0.35″ maximum,
0.12″ mean), for the rows from 1962-01-20 (0.10″, 0.07″) and for the rows after
2025 (0.30″), each set from its observed figure.

## Not covered here

This oracle covers body longitudes only. **House-cusp accuracy is not measured
against any reference.**

Two tests used to claim that ground — `comprehensive_comparison.rs` and
`extended_comparison.rs` — but they read `comprehensive_reference.json` /
`extended_reference.json`, fixtures generated from a third-party ephemeris
library that had never existed in this repository's git history. Both had
therefore never run a single assertion while reporting green. They were removed
on 2026-07-16 rather than revived: Horizons does not serve house cusps,
ayanamshas, nutation or tithis, so reviving them would have meant taking a
dependency on that library, which is exactly what the clean-room position
exists to avoid.

Covering house cusps would need a reference that is public-domain or
first-principles. That work is not done.
