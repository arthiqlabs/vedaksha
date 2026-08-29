# Self-consistency check: published package vs. current source

**Date:** 2026-08-29. Companion to `2026-08-29-lunar-node-theory-review.md`. This check compares
Vedaksha against itself only — the published PyPI package (`vedaksha==7.4.0`, the same package
version the external harness called) against a direct evaluation of `mean_node` and
`ayanamsha_value(TrueChitra, ...)` from the `main` source tree. No reference-engine data of any
kind is used or needed for this comparison.

## Method

1. Installed `vedaksha==7.4.0` from PyPI into a clean venv and called
   `Vedaksha().natal_chart(julian_day=jd, latitude=28.6139, longitude=77.2090,
   ayanamsha="TrueChitra")` at three widely-separated epochs, reading the `MeanNode` planet
   entry's `longitude` field (the package's sidereal output, which includes nutation-in-longitude
   per `coordinates.rs`'s documented convention for `Body`-routed longitudes).
2. Wrote a throwaway Rust test (not committed) calling `vedaksha_ephem_core::nodes::mean_node(jd)`
   and `vedaksha_astro::sidereal::ayanamsha_value(Ayanamsha::TrueChitra, jd)` directly from the
   current `main` source at the same three `jd` values, and computed
   `sidereal = normalize(mean_node - ayanamsha)`. This value has **no** nutation term (the raw
   theory-layer subtraction), so it is expected to differ from the package's value by up to the
   nutation-in-longitude amplitude (documented elsewhere in this repo as bounded at ≈17.2 arcsec),
   not by more.

## Results

| Epoch | jd | Source (no nutation) | Package `MeanNode` (with nutation) | Difference |
|---|---|---|---|---|
| B1900.0 | 2415020.31352 | 236.7203305° | 236.7251662° | +17.41 arcsec |
| 1950 | 2433282.5 | 348.9696254° | 348.9686902° | −3.37 arcsec |
| J2000.0 | 2451545.0 | 101.2031886° | 101.1992794° | −14.07 arcsec |

The `ayanamsha_value` the package reports independently matches the committed test fixture
(`crates/vedaksha-astro/tests/fixtures/ayanamsha.json`) exactly at B1900.0 and J2000.0
(22.445977627282105° and 23.841359280278397° respectively, to all fixture digits).

## Conclusion

Across a full century (1900 → 2000), the published package and current source agree to within a
few to ~17 arcseconds — bounded by nutation, not growing with time. There is no ~50 arcsec/year
drift, or anything resembling one, between the package and the source. This rules out a
stale/divergent packaging build as the explanation for Finding 2 in the parity harness's report:
**the published package is not stale relative to source, and no J2000-vs-of-date frame confusion
exists between Vedaksha's own theory layer and what it ships.**

This does not resolve the finding — it narrows it. The remaining open question (see the theory
review doc) is whether the divergence lies in a code path this review has not yet reached, or in a
genuine difference between how `TrueChitra`/"True Chitrapaksha" is defined or realized between two
independent implementations — a question for further theory review, not for the packaging pipeline.
