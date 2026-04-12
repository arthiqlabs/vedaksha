# WASM Chart Computation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `compute_natal_chart(config_json) -> JSON` to `vedaksha-wasm` — a single function that takes birth data and returns a complete natal chart using `AnalyticalProvider` for zero-data-file ephemeris computation.

**Architecture:** Parse JSON input with serde (defaults via `#[serde(default)]`), convert calendar date to JD, compute planetary positions via `AnalyticalProvider` + `apparent_position`, compute sidereal time for RAMC, pipe everything through `compute_chart`, serialize result.

**Tech Stack:** Rust, `wasm-bindgen`, `vedaksha-ephem-core` (AnalyticalProvider, coordinates, julian, sidereal_time, obliquity, nutation), `vedaksha-astro` (chart, sidereal)

**Spec:** `docs/superpowers/specs/2026-04-11-wasm-chart-computation-design.md`

---

## File Structure

| File | Change |
|------|--------|
| `crates/vedaksha-wasm/Cargo.toml` | Add `vedaksha-ephem-core` dependency |
| `crates/vedaksha-wasm/src/lib.rs` | Add `compute_natal_chart` function, `NatalChartInput` struct, `body_from_name` helper |

---

### Task 1: Add `vedaksha-ephem-core` dependency to WASM crate

**Files:**
- Modify: `crates/vedaksha-wasm/Cargo.toml`

- [ ] **Step 1: Add the dependency**

In `crates/vedaksha-wasm/Cargo.toml`, add to the `[dependencies]` section:

```toml
vedaksha-ephem-core = { version = "0.2.0", path = "../vedaksha-ephem-core" }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p vedaksha-wasm`

Expected: Compiles without errors. The new dependency is available but unused.

- [ ] **Step 3: Commit**

```bash
git add crates/vedaksha-wasm/Cargo.toml
git commit -m "chore(wasm): add vedaksha-ephem-core dependency

Required for AnalyticalProvider access in compute_natal_chart."
```

---

### Task 2: Implement `compute_natal_chart` function

**Files:**
- Modify: `crates/vedaksha-wasm/src/lib.rs`

- [ ] **Step 1: Write a native test for the inner computation logic**

Add to the `mod tests` block at the bottom of `lib.rs`:

```rust
    #[test]
    fn compute_natal_chart_inner_known_chart() {
        // Known birth data: 2000-01-01 12:00 UTC, Delhi (28.6139°N, 77.2090°E)
        let input = NatalChartInput {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0,
            latitude: 28.6139,
            longitude: 77.209,
            ayanamsha: "Lahiri".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec![
                "Sun".into(), "Moon".into(), "Mercury".into(), "Venus".into(),
                "Mars".into(), "Jupiter".into(), "Saturn".into(),
                "MeanNode".into(), "TrueNode".into(),
            ],
        };

        let result = compute_natal_chart_inner(input);
        assert!(result.is_ok(), "compute_natal_chart_inner failed: {:?}", result.err());

        let output: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();

        // Verify structure
        assert!(output["planets"].is_array(), "missing planets array");
        assert!(output["houses"].is_object(), "missing houses object");
        assert!(output["aspects"].is_array(), "missing aspects array");
        assert!(output["julian_day"].is_number(), "missing julian_day");
        assert!(output["ayanamsha_value"].is_number(), "missing ayanamsha_value");

        // Verify planet count matches requested bodies
        let planets = output["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 9, "expected 9 planets, got {}", planets.len());

        // Verify Ascendant is reasonable for Delhi at noon
        // Delhi at noon UTC → local sidereal time puts ASC roughly in 10-40° range
        let asc = output["houses"]["asc"].as_f64().unwrap();
        assert!(asc > 0.0 && asc < 360.0, "ASC out of range: {asc}");

        // Verify ayanamsha is ~23.856° (Lahiri at J2000)
        let ayan = output["ayanamsha_value"].as_f64().unwrap();
        assert!((ayan - 23.856).abs() < 0.1, "Lahiri ayanamsha should be ~23.856°, got {ayan}");
    }

    #[test]
    fn compute_natal_chart_inner_defaults() {
        // Only required fields — verify defaults are applied
        let input = NatalChartInput {
            year: 1990,
            month: 6,
            day: 15,
            hour: 10,
            minute: 30,
            second: 0,
            latitude: 51.5074,
            longitude: -0.1278,
            ayanamsha: "Lahiri".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec![],  // empty = use defaults
        };

        let result = compute_natal_chart_inner(input);
        assert!(result.is_ok());

        let output: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        let planets = output["planets"].as_array().unwrap();
        // Default bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, MeanNode, TrueNode
        assert_eq!(planets.len(), 9, "default should produce 9 planets");
    }

    #[test]
    fn compute_natal_chart_inner_error_cases() {
        // Unknown ayanamsha
        let input = NatalChartInput {
            year: 2000, month: 1, day: 1, hour: 12, minute: 0, second: 0,
            latitude: 28.0, longitude: 77.0,
            ayanamsha: "FooBar".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec!["Sun".into()],
        };
        assert!(compute_natal_chart_inner(input).is_err());

        // Unknown house system
        let input = NatalChartInput {
            year: 2000, month: 1, day: 1, hour: 12, minute: 0, second: 0,
            latitude: 28.0, longitude: 77.0,
            ayanamsha: "Lahiri".to_string(),
            house_system: "Topocentric".to_string(),
            bodies: vec!["Sun".into()],
        };
        assert!(compute_natal_chart_inner(input).is_err());
    }
```

- [ ] **Step 2: Define the `NatalChartInput` struct and body parser**

Add these above the existing `// --- Helper parsers ---` section in `lib.rs`:

```rust
/// Input for natal chart computation.
#[derive(serde::Deserialize)]
struct NatalChartInput {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    #[serde(default)]
    second: u32,
    latitude: f64,
    longitude: f64,
    #[serde(default = "default_ayanamsha")]
    ayanamsha: String,
    #[serde(default = "default_house_system")]
    house_system: String,
    #[serde(default)]
    bodies: Vec<String>,
}

fn default_ayanamsha() -> String { "Lahiri".to_string() }
fn default_house_system() -> String { "Placidus".to_string() }

fn default_bodies() -> Vec<String> {
    vec![
        "Sun", "Moon", "Mercury", "Venus", "Mars",
        "Jupiter", "Saturn", "MeanNode", "TrueNode",
    ].into_iter().map(String::from).collect()
}

fn body_from_name(name: &str) -> Option<vedaksha_ephem_core::bodies::Body> {
    use vedaksha_ephem_core::bodies::Body;
    match name.to_lowercase().as_str() {
        "sun" => Some(Body::Sun),
        "moon" => Some(Body::Moon),
        "mercury" => Some(Body::Mercury),
        "venus" => Some(Body::Venus),
        "mars" => Some(Body::Mars),
        "jupiter" => Some(Body::Jupiter),
        "saturn" => Some(Body::Saturn),
        "uranus" => Some(Body::Uranus),
        "neptune" => Some(Body::Neptune),
        "meannode" | "mean_node" | "rahu" => Some(Body::MeanNode),
        "truenode" | "true_node" => Some(Body::TrueNode),
        _ => None,
    }
}
```

- [ ] **Step 3: Implement `compute_natal_chart_inner`**

Add this function below the struct definitions:

```rust
/// Inner implementation (no wasm_bindgen, testable on native targets).
fn compute_natal_chart_inner(input: NatalChartInput) -> Result<String, String> {
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_ephem_core::coordinates;
    use vedaksha_ephem_core::julian;
    use vedaksha_ephem_core::jpl::EphemerisProvider;
    use vedaksha_ephem_core::nutation;
    use vedaksha_ephem_core::obliquity;
    use vedaksha_ephem_core::sidereal_time;

    // 1. Parse ayanamsha and house system
    let ayanamsha_system = ayanamsha_from_str(&input.ayanamsha)
        .map_err(|_| format!("Unknown ayanamsha: {}", input.ayanamsha))?;
    let house_system = house_system_from_str(&input.house_system)
        .map_err(|_| format!("Unknown house system: {}", input.house_system))?;

    // 2. Convert calendar date to Julian Day (UTC)
    let day_fraction = input.day as f64
        + input.hour as f64 / 24.0
        + input.minute as f64 / 1440.0
        + input.second as f64 / 86400.0;
    let jd = julian::calendar_to_jd(input.year, input.month, day_fraction);

    // 3. Create analytical provider
    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();
    if jd < jd_min || jd > jd_max {
        return Err(format!(
            "Date out of range: JD {jd:.1} is outside [{jd_min:.0}, {jd_max:.0}]"
        ));
    }

    // 4. Resolve bodies (use defaults if empty)
    let body_names = if input.bodies.is_empty() {
        default_bodies()
    } else {
        input.bodies
    };

    // 5. Compute planetary positions
    let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::new();
    for name in &body_names {
        let body = body_from_name(name)
            .ok_or_else(|| format!("Unknown body: {name}"))?;

        let pos = coordinates::apparent_position(&provider, body, jd)
            .map_err(|e| format!("Failed to compute {name}: {e}"))?;

        planet_data.push((
            name.clone(),
            pos.ecliptic.longitude.to_degrees(),
            pos.ecliptic.latitude.to_degrees(),
            pos.ecliptic.distance,
            pos.longitude_speed,
        ));
    }

    // 6. Compute sidereal time and RAMC
    let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let eps_true = obliquity::true_obliquity(jd_tt, deps);
    let geo_lon_rad = input.longitude * core::f64::consts::PI / 180.0;
    let last = sidereal_time::local_sidereal_time(jd_tt, geo_lon_rad, dpsi, eps_true);
    let ramc_deg = last * 180.0 / core::f64::consts::PI;

    // 7. Compute obliquity in degrees
    let obliquity_deg = obliquity::mean_obliquity(jd_tt) * 180.0 / core::f64::consts::PI;

    // 8. Build chart config
    let config = vedaksha_astro::chart::ChartConfig {
        house_system,
        ayanamsha: Some(ayanamsha_system),
        rulership_scheme: vedaksha_astro::dignity::RulershipScheme::Traditional,
        aspect_types: vedaksha_astro::aspects::AspectType::MAJOR.to_vec(),
        orb_factor: 1.0,
    };

    // 9. Compute chart
    let chart = vedaksha_astro::chart::compute_chart(
        &planet_data,
        ramc_deg,
        input.latitude,
        obliquity_deg,
        jd,
        &config,
    );

    // 10. Build output with metadata
    let ayanamsha_value = vedaksha_astro::sidereal::ayanamsha_value(ayanamsha_system, jd);

    let output = serde_json::json!({
        "planets": chart.planets,
        "houses": {
            "cusps": chart.houses.cusps,
            "asc": chart.houses.asc,
            "mc": chart.houses.mc,
            "system": format!("{:?}", chart.houses.system),
            "polar_fallback": chart.houses.polar_fallback,
        },
        "aspects": chart.aspects.iter().map(|a| serde_json::json!({
            "body1": a.body1_index,
            "body2": a.body2_index,
            "type": format!("{:?}", a.aspect_type),
            "orb": a.orb,
            "applying": a.motion == vedaksha_astro::aspects::AspectMotion::Applying,
            "strength": a.strength,
        })).collect::<Vec<_>>(),
        "ayanamsha_value": ayanamsha_value,
        "julian_day": jd,
        "config_summary": chart.config_summary,
    });

    serde_json::to_string(&output).map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Add the `wasm_bindgen` entry point**

Add this function in the main body of `lib.rs` (near the other `#[wasm_bindgen]` functions):

```rust
/// Compute a complete natal chart from birth data.
///
/// # Arguments
/// * `config_json` — JSON string with birth data and optional configuration.
///
/// Required fields: `year`, `month`, `day`, `hour`, `minute`, `latitude`, `longitude`
/// Optional: `second` (0), `ayanamsha` ("Lahiri"), `house_system` ("Placidus"),
///           `bodies` (["Sun","Moon","Mercury","Venus","Mars","Jupiter","Saturn","MeanNode","TrueNode"])
///
/// Input datetime is UTC. The caller is responsible for timezone conversion.
///
/// # Returns
/// JSON string with complete chart: planets (with signs, houses, dignities),
/// house cusps, aspects, ayanamsha value, Julian Day.
#[wasm_bindgen]
pub fn compute_natal_chart(config_json: &str) -> Result<String, JsError> {
    let input: NatalChartInput = serde_json::from_str(config_json)
        .map_err(|e| JsError::new(&format!("Invalid input JSON: {e}")))?;
    compute_natal_chart_inner(input).map_err(|e| JsError::new(&e))
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --lib -p vedaksha-wasm`

Expected: All existing tests pass + 3 new tests pass (`compute_natal_chart_inner_known_chart`, `compute_natal_chart_inner_defaults`, `compute_natal_chart_inner_error_cases`).

- [ ] **Step 6: Run full workspace tests**

Run: `cargo test --workspace --lib`

Expected: All tests pass. No regressions.

- [ ] **Step 7: Commit**

```bash
git add crates/vedaksha-wasm/src/lib.rs
git commit -m "feat(wasm): add compute_natal_chart for end-to-end chart computation

Accepts birth data as JSON (date, time, location, optional config),
computes planetary positions via AnalyticalProvider (zero data files),
returns complete natal chart with planets, houses, aspects, and
dignities. Input is UTC."
```

---

### Task 3: Update DATA_PROVENANCE.md

**Files:**
- Modify: `DATA_PROVENANCE.md`

- [ ] **Step 1: Update WASM section**

In `DATA_PROVENANCE.md`, Section 13 (WASM & Python Bindings), find:

```
| **WASM chart computation** | Not available (no embedded ephemeris) | `compute_chart` callable from JS with embedded DE441 data | **MISSING** |
```

Replace with:

```
| **WASM chart computation** | `compute_natal_chart` via AnalyticalProvider (VSOP87A + ELP/MPP02) | `compute_chart` callable from JS with embedded DE441 data | **OK** |
```

Also find:

```
| **WASM functions** | 11 functions (dasha, nakshatra, varga, houses, aspects, sidereal, i18n) | Chart computation via embedded ephemeris (spec Section 8) | **DEV** |
```

Replace with:

```
| **WASM functions** | 12 functions (chart, dasha, nakshatra, varga, houses, aspects, sidereal, i18n) | Chart computation via embedded ephemeris (spec Section 8) | **OK** |
```

- [ ] **Step 2: Commit**

```bash
git add DATA_PROVENANCE.md
git commit -m "docs: update DATA_PROVENANCE for WASM chart computation

compute_natal_chart now available via AnalyticalProvider.
WASM chart computation marked OK."
```
