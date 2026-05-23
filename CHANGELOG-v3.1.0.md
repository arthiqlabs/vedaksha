# Vedākṣha v3.1.0 — Performance + Panchanga Ending Times

**Release date:** 2026-05-23

A performance-focused minor release (no breaking API changes) plus one
correctness/feature addition to muhurta. All numerical results are preserved
within the existing oracle / paper-example tolerances; SIMD and reassociation
changes are validated against them.

## Performance — ephemeris / chart

Full natal chart **~205 ms → ~7.7 ms (≈27×)** on an AVX2 build; single Moon
position **1.59 ms → 339 µs**; per-chart lunar-series evaluations cut **123 → 6**.

- `sincos` for the VSOP87A and ELP/MPP02 series (one argument reduction).
- `[profile.release]`: fat LTO, `codegen-units = 1`, `panic = "abort"`, strip.
- Memoizing ephemeris provider + new batch `coordinates::apparent_positions`,
  so a chart shares one provider across bodies and daily-motion timesteps.
- SIMD lunar kernel via the `wide` crate (`f64x4::sin_cos`), accuracy pinned
  to `libm` within < 1e-12.
- Phase factorization in ELP/MPP02 (precompute argument values once per call).
- **Light-time Earth extrapolation** — anchor Earth once per call and
  first-order extrapolate to the retarded time, instead of re-evaluating the
  lunar series in every planet's light-time loop.
- Time-only frame hoisting (`CelestialFrame` / `frame_for`): nutation,
  precession and obliquity computed once per timestamp, shared across bodies.
- Distributed x86 artifacts (linux binary + ghcr.io image) now target
  `x86-64-v3` (AVX2) so the SIMD kernel runs 4-wide; WASM built with
  `+simd128`. **Note:** the linux binary / Docker image now require an AVX2
  (Haswell 2013+) CPU.

## Performance — dasha / transit / muhurta

- **Dasha:** `Vec::with_capacity` in the sub-period recursion.
- **Transit:** coarse-longitude cache shared across natal × aspect scans
  (~50× fewer ephemeris calls for multi-target searches) + bisection
  longitude memoization.
- **Muhurta:** the 0.5-day scan now uses a position-only ephemeris path
  (`coordinates::ecliptic_position`), skipping the daily-motion it discarded.

## Features

- **Muhurta tithi / nakshatra ending times** — `search_muhurta` now reports
  the exact Julian Day at which each reported window's tithi and nakshatra
  end (`tithi_end_jd` / `nakshatra_end_jd`), computed by Newton refinement of
  the boundary crossing with the Moon/Sun daily motion as the derivative
  (`vedaksha_vedic::muhurta::compute_tithi_end` / `compute_nakshatra_end`).

## New public API (additive, non-breaking)

- `coordinates::apparent_positions`, `coordinates::ecliptic_position`
- `coordinates::CelestialFrame`, `coordinates::frame_for`
- `vedaksha_vedic::muhurta::compute_tithi_end`, `compute_nakshatra_end`
  (and `tithi_end_jd` / `nakshatra_end_jd` fields on `MuhurtaAssessment`)

## Known follow-ups

- Muhurta/panchanga still ignores `lat`/`lon`, so **Lagna** and the local-
  sunrise factors (**Rahu Kalam / Yamaganda / Gulika**) are not yet computed.
