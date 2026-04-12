# WASM Chart Computation

**Date**: 2026-04-11
**Status**: Approved
**Scope**: Add `compute_natal_chart` function to `vedaksha-wasm` that accepts birth data as JSON and returns a complete natal chart, using `AnalyticalProvider` internally for zero-data-file ephemeris computation.

## Motivation

The `vedaksha-wasm` crate currently exports individual utility functions (dasha, nakshatra, varga, houses, aspects, sidereal conversion, localization) but cannot compute a natal chart end-to-end because it has no access to planetary positions. The `AnalyticalProvider` (VSOP87A + ELP/MPP02) now exists in `vedaksha-ephem-core` with zero data file requirements, making it suitable for WASM environments.

This bridges the gap: a single function call from JavaScript/TypeScript that takes birth data and returns a complete Jyotish chart.

## Design

### Function Signature

```rust
#[wasm_bindgen]
pub fn compute_natal_chart(config_json: &str) -> Result<String, JsError>
```

### Input (JSON)

```json
{
  "year": 1990,
  "month": 3,
  "day": 15,
  "hour": 14,
  "minute": 30,
  "second": 0,
  "latitude": 28.6139,
  "longitude": 77.2090,
  "ayanamsha": "Lahiri",
  "house_system": "Placidus",
  "bodies": ["Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "MeanNode", "TrueNode"]
}
```

**Required fields**: `year`, `month`, `day`, `hour`, `minute`, `latitude`, `longitude`

**Optional fields with defaults**:
- `second`: 0
- `ayanamsha`: "Lahiri"
- `house_system`: "Placidus"
- `bodies`: ["Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "MeanNode", "TrueNode"]

**Input is UTC**. The caller is responsible for timezone conversion before calling. Timezone databases do not belong in a WASM ephemeris.

### Output (JSON)

The full `ComputedChart` from `vedaksha_astro::chart::compute_chart()`, serialized via serde. Contains:

- `planets`: Array of planet objects, each with `name`, `longitude` (sidereal), `latitude`, `distance`, `speed`, `retrograde`, `sign`, `sign_index`, `house`, `dignity`
- `houses`: Object with `cusps` (12 longitudes), `asc`, `mc`, `system`, `polar_fallback`
- `aspects`: Array of detected aspects between planets
- `ayanamsha`: The ayanamsha value used (degrees)
- `julian_day`: The computed JD for the input date

### Internal Pipeline

```
1. Parse input JSON → NatalChartInput struct (serde, with defaults)
2. Validate: year in range, month 1-12, day 1-31, lat -90..90, lon -180..180
3. Convert calendar → JD:  vedaksha_ephem_core::julian::calendar_to_jd()
4. Create AnalyticalProvider (unit struct, zero cost)
5. For each body in input.bodies:
     apparent_position(&provider, body, jd) → ecliptic lon/lat/dist/speed
6. Compute sidereal time: vedaksha_ephem_core::sidereal_time module
     → RAMC (Right Ascension of MC) from geographic longitude + sidereal time
7. Compute obliquity: vedaksha_ephem_core::obliquity::mean_obliquity(jd)
8. Build planet_data vec: [(name, lon_deg, lat_deg, dist_au, speed_deg_per_day)]
9. Build ChartConfig from input ayanamsha + house system
10. compute_chart(planet_data, ramc, latitude, obliquity, jd, &config)
11. Serialize ComputedChart + metadata to JSON
12. Return JSON string
```

### Error Handling

Returns `JsError` with descriptive message for:
- Invalid JSON (parse failure)
- Missing required fields
- Date out of `AnalyticalProvider` range (-2000 to +3000 CE)
- Unknown ayanamsha or house system string
- Ephemeris computation failure (body not available, etc.)

## Dependencies

### New dependency for `vedaksha-wasm`:

```toml
vedaksha-ephem-core = { version = "0.2.0", path = "../vedaksha-ephem-core" }
```

### Existing dependencies already available:
- `vedaksha-astro` — `compute_chart`, `ChartConfig`, house systems, sidereal
- `vedaksha-vedic` — not directly needed for this function (dasha etc. are separate calls)
- `serde`, `serde_json` — JSON parsing/serialization
- `wasm-bindgen` — WASM export

## Files Modified

| File | Change |
|------|--------|
| `crates/vedaksha-wasm/Cargo.toml` | Add `vedaksha-ephem-core` dependency |
| `crates/vedaksha-wasm/src/lib.rs` | Add `compute_natal_chart` function, `NatalChartInput` struct, body name parser |

## What This Does NOT Include

- **No timezone handling** — input is UTC
- **No streaming or async** — synchronous single call
- **No caching** — each call is independent
- **No changes to `vedaksha-astro` or `vedaksha-ephem-core`** — consuming existing APIs
- **No WASI target build** — works in `wasm32-unknown-unknown` and native; WASI packaging is separate
- **No new crate or module** — single function addition to existing WASM crate

## Testing

### 1. Native Unit Test — Known Chart

Call the inner parsing and computation logic (not the `wasm_bindgen`-annotated function) with known birth data. Verify:
- Correct number of planets in output
- Moon is in the expected nakshatra (cross-check with known chart)
- House cusps are reasonable (ASC in expected quadrant for the given latitude/time)
- Ayanamsha value matches `get_ayanamsha` for the same date

### 2. Native Unit Test — Defaults

Call with only required fields. Verify defaults are applied: Lahiri ayanamsha, Placidus houses, all 9 graha + nodes.

### 3. Error Cases

- Missing `year` field → error
- `month: 13` → error (or handled by JD conversion)
- Unknown `ayanamsha: "FooBar"` → error
- Unknown `house_system: "Topocentric"` → error
- Date far outside range (year -5000) → error from AnalyticalProvider

### 4. WASM Integration Test

Using `wasm-bindgen-test`:
- Call `compute_natal_chart` with valid JSON string
- Parse output JSON
- Verify it contains `planets`, `houses`, `aspects` keys

## Risk Assessment

**Low risk.** This is a wiring task — connecting existing tested components through a new entry point. The `AnalyticalProvider`, `apparent_position`, `compute_chart`, and all serialization are already tested. The new code is input parsing, pipeline orchestration, and JSON output construction.

The main risk is getting the RAMC computation right (sidereal time + geographic longitude → RAMC). The existing `sidereal_time` module and house computation tests validate this path.
