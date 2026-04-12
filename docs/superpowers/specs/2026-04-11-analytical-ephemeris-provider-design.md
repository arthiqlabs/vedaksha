# Analytical Ephemeris Provider

**Date**: 2026-04-11
**Status**: Approved
**Scope**: New `AnalyticalProvider` implementing `EphemerisProvider` using VSOP87A + ELP/MPP02 analytical series — zero data files, `no_std` compatible, suitable for WASM and constrained deployment targets.

## Motivation

Vedaksha's only `EphemerisProvider` implementation is `SpkReader`, which reads the DE440s binary file (~31 MB) from disk. This works for server/container deployments but is a blocker for:

- **Cloudflare Workers**: 10 MB bundle limit
- **WASM in browser**: no filesystem access
- **Edge deployments**: minimal binary, no external data dependencies

An analytical provider compiles coefficients as constants in the source code, producing a self-contained binary with zero runtime data dependencies. This is the same approach used by Swiss Ephemeris's Moshier mode, which has served the astrology community for 30+ years.

## Architecture

### Two-Provider Model

| Environment | Provider | Accuracy | Data |
|-------------|----------|----------|------|
| Server / container | `SpkReader` (existing) | Sub-arcsecond | `de440s.bsp` on disk |
| WASM / Workers / edge | `AnalyticalProvider` (new) | ~0.5-5 arcsecond | Zero — compiled constants |

Both implement `EphemerisProvider`. All downstream code (`apparent_position()`, `compute_chart()`, house systems, aspects) is provider-agnostic.

### The Provider

```rust
/// Analytical ephemeris using VSOP87A (planets) and ELP/MPP02 (Moon).
///
/// Zero data files. All coefficients are compile-time constants.
/// Suitable for `no_std`, WASM, and constrained environments.
pub struct AnalyticalProvider;

impl EphemerisProvider for AnalyticalProvider {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError>;
    fn time_range(&self) -> (f64, f64);
}
```

- **Unit struct**: No fields, no constructor arguments, no I/O.
- **`no_std` compatible**: Pure math, `libm` for transcendentals, no allocations.
- **`time_range()`**: Returns (-2000 CE, +3000 CE) — the conservative intersection of VSOP87A's validated range (~-2000 to +6000 CE) and ELP/MPP02's validated range (~-3000 to +3000 CE). The Moon is the binding constraint.

### Body Support

| Body | Source | Status |
|------|--------|--------|
| Sun | Derived from VSOP87A Earth (negate heliocentric) | Supported |
| Moon | ELP/MPP02 (Chapront 2002) | Supported |
| Mercury | VSOP87A (Bretagnon & Francou 1988) | Supported |
| Venus | VSOP87A | Supported |
| Mars | VSOP87A | Supported |
| Jupiter | VSOP87A | Supported |
| Saturn | VSOP87A | Supported |
| Uranus | VSOP87A | Supported |
| Neptune | VSOP87A | Supported |
| Earth-Moon Barycenter | VSOP87A (direct) | Supported |
| Mean Node | Meeus polynomial (existing `nodes.rs`) | Supported |
| True Node | Meeus polynomial (existing `nodes.rs`) | Supported |
| Pluto | N/A | `BodyNotAvailable` |

## Coefficient Sources

### VSOP87A

**Source**: Bretagnon P., Francou G. (1988), "Planetary theories in rectangular and spherical variables. VSOP87 solutions", Astronomy & Astrophysics 202, pp. 309-315.

VSOP87A provides heliocentric ecliptic rectangular coordinates (X, Y, Z) referred to the ecliptic and equinox of J2000.0. Each coordinate is expressed as a Poisson series:

```
X = sum over alpha: X_alpha(t)
X_alpha(t) = t^alpha * sum_i [A_i * cos(B_i + C_i * t)]
```

where `t` is Julian millennia from J2000.0, alpha = 0..5, and each term is an `(A, B, C)` triple (amplitude, phase, frequency).

### ELP/MPP02

**Source**: Chapront J. (2002), "A new determination of lunar orbital parameters, precession constant and tidal acceleration from LLR measurements", Astronomy & Astrophysics 387, pp. 700-709.

ELP/MPP02 provides geocentric ecliptic coordinates (longitude, latitude, distance) for the Moon. The series is organized in three main groups, each with subgroups for the main problem, perturbation corrections, and Poisson terms.

### Truncation Budget

Target accuracy: **0.5 arcsecond for planets, 2 arcseconds for Moon, across 1800-2200 CE.**

Terms with amplitude below the threshold are dropped. The threshold is chosen per body so that the cumulative residual from all dropped terms stays within budget.

| Body | Full terms | Truncated terms | Amplitude threshold | Residual (1800-2200) |
|------|-----------|-----------------|--------------------|--------------------|
| Mercury | ~1100 | ~150-200 | TBD after transcription | < 0.5" |
| Venus | ~700 | ~100-150 | TBD after transcription | < 0.5" |
| Earth (EMB) | ~1100 | ~150-200 | TBD after transcription | < 0.5" |
| Mars | ~1500 | ~200-250 | TBD after transcription | < 0.5" |
| Jupiter | ~800 | ~100-150 | TBD after transcription | < 0.5" |
| Saturn | ~1000 | ~100-150 | TBD after transcription | < 0.5" |
| Uranus | ~600 | ~80-100 | TBD after transcription | < 0.5" |
| Neptune | ~400 | ~60-80 | TBD after transcription | < 0.5" |
| Moon (longitude) | ~14000 | ~1000-1500 | TBD after transcription | < 2" |
| Moon (latitude) | ~8000 | ~500-800 | TBD after transcription | < 2" |
| Moon (distance) | ~14000 | ~500-800 | TBD after transcription | < 2" |

**Note**: The "Full terms" column is approximate — exact counts depend on the series file version. "Truncated terms" are estimates based on similar implementations (SwissEph Moshier, IMCCE truncated series). The "TBD after transcription" thresholds will be determined during implementation by sorting terms by amplitude and cutting where cumulative residual exceeds the budget. Final values will be recorded in each coefficient file's header comment and updated in this table.

**Acceptance criteria**: If oracle tests show a body exceeding its stated residual, the truncation was too aggressive and terms must be added back.

## Coordinate Conversion Pipeline

### Planets (Mercury through Neptune)

```
VSOP87A(planet, jd)
  → heliocentric ecliptic rectangular [X, Y, Z] (J2000 ecliptic frame)
  → rotate ecliptic-to-equatorial (obliquity at J2000 = 84381.406")
  → heliocentric ICRS equatorial [X, Y, Z]
  → add Sun barycentric position
  → barycentric ICRS equatorial [X, Y, Z]
```

**Sun-SSB approximation**: The Sun is treated as coincident with the solar system barycenter. This introduces up to ~0.5 arcsecond error for inner planets due to the Sun's actual offset from the SSB (up to ~0.01 AU, driven primarily by Jupiter). This is within the truncation budget and documented in the code. If sub-arcsecond accuracy is ever needed, the fix is computing the Sun-SSB offset from the giant planets' positions.

### Sun

```
VSOP87A(Earth, jd)
  → heliocentric ecliptic rectangular [X, Y, Z]
  → negate → geocentric ecliptic position of Sun
  → rotate ecliptic-to-equatorial
  → barycentric ICRS (Sun ≈ SSB)
```

### Moon

```
ELP/MPP02(jd)
  → geocentric ecliptic [longitude, latitude, distance]
  → convert spherical to rectangular [X, Y, Z]
  → rotate ecliptic-to-equatorial
  → add Earth barycentric position
  → barycentric ICRS equatorial [X, Y, Z]
```

Earth barycentric is derived from VSOP87A(EMB) and the Moon's position using the EMRAT factor (81.30056894) already defined in `coordinates.rs`.

### Velocity

Time derivatives are computed analytically from the VSOP87A/ELP series derivatives (the derivative of `A * cos(B + C*t)` is `-A * C * sin(B + C*t)`). This avoids numerical differentiation and gives exact velocities to the same truncation accuracy as positions.

### Mean/True Node

Delegated to existing `vedaksha_ephem_core::nodes::mean_lunar_node()` and `true_lunar_node()`. These return ecliptic longitude only; the provider wraps them into a `StateVector` with latitude = 0, distance = 1 AU (conventional), and zero velocity.

## Module Structure

New directory `vedaksha-ephem-core/src/analytical/`:

```
analytical/
  mod.rs              — AnalyticalProvider struct, EphemerisProvider impl,
                        ecliptic-to-equatorial rotation, heliocentric-to-barycentric
  vsop87a.rs          — VSOP87A series evaluation function
  elp_mpp02.rs        — ELP/MPP02 series evaluation function
  coefficients/
    mod.rs            — re-exports
    mercury.rs        — Mercury VSOP87A coefficients (X0-X5, Y0-Y5, Z0-Z5)
    venus.rs          — Venus coefficients
    earth.rs          — Earth-Moon barycenter coefficients
    mars.rs           — Mars coefficients
    jupiter.rs        — Jupiter coefficients
    saturn.rs         — Saturn coefficients
    uranus.rs         — Uranus coefficients
    neptune.rs        — Neptune coefficients
    moon_longitude.rs — ELP/MPP02 longitude series (main + perturbation + Poisson)
    moon_latitude.rs  — ELP/MPP02 latitude series
    moon_distance.rs  — ELP/MPP02 distance series
```

Each coefficient file begins with a header comment documenting:
- Source paper and table number
- Truncation threshold (amplitude cutoff)
- Number of terms retained vs. full series
- Estimated residual error for 1800-2200 CE

## Testing

### 1. Per-Body Position Accuracy (vs DE440s)

For each supported body, compute position at 10 dates spanning 1900-2100. Compare against `SpkReader` + DE440s. Tolerances:

| Bodies | Longitude tolerance | Latitude tolerance |
|--------|-------------------|--------------------|
| Planets (Mercury-Neptune) | 2 arcseconds | 2 arcseconds |
| Sun | 2 arcseconds | 2 arcseconds |
| Moon | 5 arcseconds | 5 arcseconds |

### 2. Moon Nakshatra Boundary Test

Select 3-5 dates where DE440s places the Moon within 0.01 degrees of a nakshatra boundary (multiples of 13 degrees 20 minutes). Verify that `AnalyticalProvider` assigns the **same nakshatra** as `SpkReader`. This tests the decision boundary that matters most — dasha lord assignment. A 5 arcsecond error that doesn't cross a boundary is fine; a 1 arcsecond error that crosses one is catastrophic.

### 3. End-to-End Chart Equivalence

Run `compute_chart()` with both providers for 3 locations (Delhi 28.6N/77.2E, London 51.5N/0.1W, New York 40.7N/74.0W) at 2-3 dates. Compare:
- All 12 house cusps
- Ascendant and MC
- All planet longitudes

Tolerance: 0.01 degrees for house cusps/Asc/MC (they're sensitive to Earth position), 2 arcseconds for planets.

### 4. Oracle Regression

Run `comprehensive_comparison` substituting `AnalyticalProvider` for `SpkReader`. Expect degraded accuracy (arcseconds vs sub-arcsecond) but all positions within 1 degree. This validates the provider is usable for chart computation across the full oracle date range.

### 5. Node Delegation

Thin integration test: `AnalyticalProvider.compute_state(Body::MeanNode, jd)` and `Body::TrueNode` return the same values as calling `nodes::mean_lunar_node(jd)` and `nodes::true_lunar_node(jd)` directly.

### 6. Pluto Returns Error

`AnalyticalProvider.compute_state(Body::Pluto, jd)` returns `Err(ComputeError::BodyNotAvailable)`.

### 7. Time Range Honesty

Dates outside (-2000 CE, +3000 CE) return `Err(ComputeError::DateOutOfRange)`.

## What This Does NOT Include

- **No WASM integration**: The provider is usable from WASM, but wiring it into `vedaksha-wasm` exports is a separate spec.
- **No Pluto**: Returns `BodyNotAvailable`. Use `SpkReader` for Pluto.
- **No changes to `SpkReader`**: The existing file-based provider is untouched.
- **No changes to `EphemerisProvider` trait**: We're adding an implementation, not modifying the interface.
- **No changes to `coordinates.rs` or any astro-layer code**: Provider-agnostic by design.
- **No coefficient generation tooling**: Coefficients are transcribed from published tables with documented truncation. No build-time pipeline.
- **No embedded SPK (`include_bytes!`)**: The analytical provider eliminates the need for this.

## Files Created

| File | Purpose |
|------|---------|
| `src/analytical/mod.rs` | Provider struct, trait impl, frame conversion |
| `src/analytical/vsop87a.rs` | VSOP87A series evaluator |
| `src/analytical/elp_mpp02.rs` | ELP/MPP02 series evaluator |
| `src/analytical/coefficients/mod.rs` | Re-exports |
| `src/analytical/coefficients/mercury.rs` | Mercury VSOP87A coefficients |
| `src/analytical/coefficients/venus.rs` | Venus coefficients |
| `src/analytical/coefficients/earth.rs` | Earth-Moon barycenter coefficients |
| `src/analytical/coefficients/mars.rs` | Mars coefficients |
| `src/analytical/coefficients/jupiter.rs` | Jupiter coefficients |
| `src/analytical/coefficients/saturn.rs` | Saturn coefficients |
| `src/analytical/coefficients/uranus.rs` | Uranus coefficients |
| `src/analytical/coefficients/neptune.rs` | Neptune coefficients |
| `src/analytical/coefficients/moon_longitude.rs` | ELP/MPP02 longitude series |
| `src/analytical/coefficients/moon_latitude.rs` | ELP/MPP02 latitude series |
| `src/analytical/coefficients/moon_distance.rs` | ELP/MPP02 distance series |

## Files Modified

| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub mod analytical;` |

## Risk Assessment

**Medium risk.** The implementation is straightforward (evaluate trigonometric series, rotate frames), but the volume of coefficient data is large and transcription errors are the primary hazard. The per-body oracle tests against DE440s are the safety net — any transcription error that exceeds the residual budget will be caught.

The conversion pipeline (heliocentric ecliptic → barycentric ICRS) is well-understood and uses the same obliquity constant and EMRAT factor already in the codebase. The Sun-SSB approximation is documented and within budget.

## Clean Room Provenance

All coefficients are transcribed from peer-reviewed published papers:
- VSOP87A: Bretagnon & Francou (1988), A&A 202, 309-315
- ELP/MPP02: Chapront (2002), A&A 387, 700-709

No external software source code was referenced. The truncation decisions are independent engineering choices documented per-body.
