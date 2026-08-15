// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-wasm
//!
//! WASM bindings for Vedākṣha, enabling browser-based astronomical
//! and astrological computation.

use wasm_bindgen::prelude::*;

/// Compute Vimshottari Dasha periods from Moon's sidereal longitude.
///
/// # Arguments
/// * `moon_longitude` — Moon's sidereal longitude in degrees [0, 360)
/// * `birth_jd` — Julian Day of birth, in **UT1** (Universal Time) — not TT,
///   not TDB, i.e. the same scale the MCP `compute_dasha` tool takes. Dasha
///   computation is ephemeris-free, so the epoch is carried through rather
///   than converted and every returned `start_jd`/`end_jd` is on this same
///   UT1 scale.
/// * `levels` — Depth of sub-periods (1-5, default 3)
///
/// # Returns
/// JSON string with the complete dasha tree.
#[wasm_bindgen]
pub fn compute_dasha(moon_longitude: f64, birth_jd: f64, levels: u8) -> Result<String, JsError> {
    let levels = levels.clamp(1, 5);
    let dasha =
        vedaksha_vedic::dasha::vimshottari::compute_vimshottari(moon_longitude, birth_jd, levels);
    serde_json::to_string(&dasha).map_err(|e| JsError::new(&e.to_string()))
}

/// Get the nakshatra and pada for a sidereal longitude.
///
/// # Arguments
/// * `sidereal_longitude` — Sidereal longitude in degrees [0, 360)
///
/// # Returns
/// JSON string with nakshatra name, index, pada, dasha lord.
#[wasm_bindgen]
pub fn get_nakshatra(sidereal_longitude: f64) -> Result<String, JsError> {
    let nak = vedaksha_vedic::nakshatra::Nakshatra::from_longitude(sidereal_longitude);
    let pada = vedaksha_vedic::nakshatra::Nakshatra::pada_from_longitude(sidereal_longitude);
    let lord = nak.dasha_lord();

    let result = serde_json::json!({
        "nakshatra": nak.name(),
        "index": nak.index(),
        "pada": pada,
        "dasha_lord": format!("{lord:?}"),
        "start_longitude": nak.start_longitude(),
        "end_longitude": nak.end_longitude(),
    });

    serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Compute the varga (divisional chart) sign for a longitude.
///
/// # Arguments
/// * `longitude` — Sidereal longitude in degrees
/// * `varga` — Varga name: "Rashi", "Navamsha", "Dashamsha", etc.
///
/// # Returns
/// Sign index (0-11) in the divisional chart.
#[wasm_bindgen]
pub fn compute_varga(longitude: f64, varga: &str) -> Result<u8, JsError> {
    let varga_type = parse_varga_type(varga)?;
    Ok(vedaksha_vedic::varga::varga_sign(longitude, varga_type))
}

/// Compute house cusps.
///
/// # Arguments
/// * `ramc` — Right Ascension of MC in degrees
/// * `latitude` — Geographic latitude in degrees
/// * `obliquity` — Obliquity of the ecliptic in degrees
/// * `system` — House system: "Placidus", "Equal", "WholeSign", etc.
///
/// # Returns
/// JSON string with 12 cusp longitudes, ASC, MC.
#[wasm_bindgen]
pub fn compute_houses(
    ramc: f64,
    latitude: f64,
    obliquity: f64,
    system: &str,
) -> Result<String, JsError> {
    let house_system = parse_house_system(system)?;
    let cusps = vedaksha_astro::houses::compute_houses(ramc, latitude, obliquity, house_system);

    let result = serde_json::json!({
        "cusps": cusps.cusps,
        "asc": cusps.asc,
        "mc": cusps.mc,
        "system": format!("{:?}", cusps.system),
        "polar_fallback": cusps.polar_fallback,
    });

    serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Find aspects between a set of planetary positions.
///
/// # Arguments
/// * `positions_json` — JSON array of {longitude: number, speed: number}
/// * `major_only` — If true, only check major (Ptolemaic) aspects
///
/// # Returns
/// JSON string with array of detected aspects.
#[wasm_bindgen]
pub fn find_aspects(positions_json: &str, major_only: bool) -> Result<String, JsError> {
    let raw_positions: Vec<serde_json::Value> = serde_json::from_str(positions_json)
        .map_err(|e| JsError::new(&format!("Invalid positions JSON: {e}")))?;

    let positions: Vec<vedaksha_astro::aspects::BodyPosition> = raw_positions
        .iter()
        .map(|v| vedaksha_astro::aspects::BodyPosition {
            longitude: v["longitude"].as_f64().unwrap_or(0.0),
            speed: v["speed"].as_f64().unwrap_or(0.0),
        })
        .collect();

    let aspect_types = if major_only {
        vedaksha_astro::aspects::AspectType::MAJOR
    } else {
        vedaksha_astro::aspects::AspectType::ALL
    };

    let aspects = vedaksha_astro::aspects::find_aspects(&positions, aspect_types, 1.0);

    let result: Vec<serde_json::Value> = aspects
        .iter()
        .map(|a| {
            serde_json::json!({
                "body1": a.body1_index,
                "body2": a.body2_index,
                "type": format!("{:?}", a.aspect_type),
                "orb": a.orb,
                "applying": a.motion == vedaksha_astro::aspects::AspectMotion::Applying,
                "strength": a.strength,
            })
        })
        .collect();

    serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Convert tropical longitude to sidereal.
///
/// # Arguments
/// * `tropical_longitude` — Tropical longitude in degrees
/// * `ayanamsha` — Ayanamsha system: "Lahiri", "FaganBradley", "Krishnamurti", etc.
/// * `jd` — Julian Day for computation
#[wasm_bindgen]
pub fn tropical_to_sidereal(
    tropical_longitude: f64,
    ayanamsha: &str,
    jd: f64,
) -> Result<f64, JsError> {
    let system = parse_ayanamsha(ayanamsha)?;
    Ok(vedaksha_astro::sidereal::tropical_to_sidereal(
        tropical_longitude,
        system,
        jd,
    ))
}

/// Get the ayanamsha value in degrees for a given date.
#[wasm_bindgen]
pub fn get_ayanamsha(ayanamsha: &str, jd: f64) -> Result<f64, JsError> {
    let system = parse_ayanamsha(ayanamsha)?;
    Ok(vedaksha_astro::sidereal::ayanamsha_value(system, jd))
}

/// Get the zodiac sign for a longitude.
///
/// # Returns
/// JSON with sign name and index.
#[wasm_bindgen]
pub fn get_sign(longitude: f64) -> String {
    let sign = vedaksha_astro::dignity::sign_of(longitude);
    serde_json::json!({
        "name": sign.name(),
        "index": sign as u8,
    })
    .to_string()
}

/// Get localized name for a planet.
#[wasm_bindgen]
pub fn planet_name(index: usize, language: &str) -> Result<String, JsError> {
    let lang = parse_language(language)?;
    Ok(vedaksha::locale::planets::planet_name(index, lang).to_string())
}

/// Get localized name for a zodiac sign.
#[wasm_bindgen]
pub fn sign_name(index: usize, language: &str) -> Result<String, JsError> {
    let lang = parse_language(language)?;
    Ok(vedaksha::locale::signs::sign_name(index, lang).to_string())
}

/// Get localized name for a nakshatra.
#[wasm_bindgen]
pub fn nakshatra_name(index: usize, language: &str) -> Result<String, JsError> {
    let lang = parse_language(language)?;
    Ok(vedaksha::locale::nakshatras::nakshatra_name(index, lang).to_string())
}

// --- Natal chart ---

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

fn default_ayanamsha() -> String {
    "Lahiri".to_string()
}
fn default_house_system() -> String {
    "Placidus".to_string()
}

fn default_bodies() -> Vec<String> {
    vec![
        "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "MeanNode", "TrueNode",
    ]
    .into_iter()
    .map(String::from)
    .collect()
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
        "truenodeosculating" | "true_node_osculating" | "osculating_node" => {
            Some(Body::TrueNodeOsculating)
        }
        _ => None,
    }
}

fn compute_natal_chart_inner(input: NatalChartInput) -> Result<String, String> {
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_ephem_core::coordinates;
    use vedaksha_ephem_core::jpl::EphemerisProvider;
    use vedaksha_ephem_core::julian;
    use vedaksha_ephem_core::nutation;
    use vedaksha_ephem_core::obliquity;
    use vedaksha_ephem_core::sidereal_time;

    // Parse config
    let ayanamsha_system = ayanamsha_from_str(&input.ayanamsha)
        .map_err(|_| format!("Unknown ayanamsha: {}", input.ayanamsha))?;
    let house_system = house_system_from_str(&input.house_system)
        .map_err(|_| format!("Unknown house system: {}", input.house_system))?;

    // Calendar to JD (UTC)
    let day_fraction = input.day as f64
        + input.hour as f64 / 24.0
        + input.minute as f64 / 1440.0
        + input.second as f64 / 86400.0;
    let jd = julian::calendar_to_jd(input.year, input.month, day_fraction);

    // Range check
    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();
    if jd < jd_min || jd > jd_max {
        return Err(format!(
            "Date out of range: JD {jd:.1} outside [{jd_min:.0}, {jd_max:.0}]"
        ));
    }

    // Resolve bodies
    let body_names = if input.bodies.is_empty() {
        default_bodies()
    } else {
        input.bodies
    };

    // Compute positions via the batch API: one shared memoizing provider so
    // the ELP/MPP02 lunar series pulled into every planet's light-time
    // correction is evaluated once per timestamp instead of once per body.
    // Output is bit-identical to per-body computation.
    let bodies: Vec<vedaksha_ephem_core::bodies::Body> = body_names
        .iter()
        .map(|name| body_from_name(name).ok_or_else(|| format!("Unknown body: {name}")))
        .collect::<Result<_, _>>()?;
    let results = coordinates::apparent_positions(&provider, &bodies, jd);
    let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::with_capacity(body_names.len());
    for (name, (_body, res)) in body_names.iter().zip(results.iter()) {
        let pos = res
            .as_ref()
            .map_err(|e| format!("Failed to compute {name}: {e}"))?;
        planet_data.push((
            name.clone(),
            pos.ecliptic.longitude.to_degrees(),
            pos.ecliptic.latitude.to_degrees(),
            pos.ecliptic.distance,
            pos.longitude_speed,
        ));
    }

    // Sidereal time -> RAMC.
    //
    // Two time scales, deliberately: nutation and obliquity are dynamical and
    // take TT, while sidereal time is the Earth's *rotation* and takes UT1
    // (`jd`). Passing `jd_tt` as the rotational argument — which this did
    // until the UT-vs-TT fix — adds ΔT worth of rotation instead of removing
    // it: 0.289° (17.3′) at today's ΔT ≈ 69 s, straight onto the ascendant,
    // the MC and all twelve cusps. Must stay identical to the MCP path
    // (`vedaksha-mcp/src/server.rs`); `mcp_surface_parity` enforces it.
    let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let eps_true = obliquity::true_obliquity(jd_tt, deps);
    let geo_lon_rad = input.longitude * core::f64::consts::PI / 180.0;
    let last = sidereal_time::local_sidereal_time(jd, geo_lon_rad, dpsi, eps_true);
    let ramc_deg = last * 180.0 / core::f64::consts::PI;

    // Obliquity in degrees
    let obliquity_deg = obliquity::mean_obliquity(jd_tt) * 180.0 / core::f64::consts::PI;

    // Chart config
    let config = vedaksha_astro::chart::ChartConfig {
        house_system,
        ayanamsha: Some(ayanamsha_system),
        rulership_scheme: vedaksha_astro::dignity::RulershipScheme::Traditional,
        aspect_types: vedaksha_astro::aspects::AspectType::MAJOR.to_vec(),
        orb_factor: 1.0,
    };

    // Compute chart
    let chart = vedaksha_astro::chart::compute_chart(
        &planet_data,
        ramc_deg,
        input.latitude,
        obliquity_deg,
        jd,
        &config,
    );

    let ayanamsha_value = vedaksha_astro::sidereal::ayanamsha_value(ayanamsha_system, jd);

    // Serialize
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

/// Compute a complete natal chart from birth data.
///
/// # Arguments
/// * `config_json` — JSON string with birth data and optional configuration.
///
/// Required: `year`, `month`, `day`, `hour`, `minute`, `latitude`, `longitude`
/// Optional: `second` (0), `ayanamsha` ("Lahiri"), `house_system` ("Placidus"),
///           `bodies` (default 9 Jyotish graha + nodes)
///
/// Input datetime is **UTC** — a civil clock reading, not TT and not TDB.
/// It is used directly as the UT1 rotational argument for sidereal time (and
/// hence the ascendant, the MC and all twelve house cusps), and converted to
/// TT internally for the dynamical terms (planetary positions, nutation,
/// obliquity). UTC stands in for UT1 here: leap seconds hold |UT1 − UTC| ≤
/// 0.9 s, worth 360.98564736629 °/day × 0.9 s / 86400 s = 0.0038° (13.5″) of
/// RAMC — two orders below the 0.289° a TT/TDB argument would cost.
///
/// # Returns
/// JSON string with planets, houses, aspects, ayanamsha value, Julian Day.
#[wasm_bindgen]
pub fn compute_natal_chart(config_json: &str) -> Result<String, JsError> {
    let input: NatalChartInput = serde_json::from_str(config_json)
        .map_err(|e| JsError::new(&format!("Invalid input JSON: {e}")))?;
    compute_natal_chart_inner(input).map_err(|e| JsError::new(&e))
}

// --- Helper parsers ---
//
// Each parser has an inner `_inner` variant returning `Result<T, &'static str>`
// (native-compatible, no JsError construction), and a public wrapper that
// converts the error to `JsError` for wasm-bindgen callers.
//
// Tests exercise the inner functions directly so they can run on native targets
// without triggering wasm-bindgen's "non-wasm targets" panic.

fn house_system_from_str(s: &str) -> Result<vedaksha_astro::houses::HouseSystem, &'static str> {
    match s.to_lowercase().as_str() {
        "placidus" => Ok(vedaksha_astro::houses::HouseSystem::Placidus),
        "koch" => Ok(vedaksha_astro::houses::HouseSystem::Koch),
        "equal" => Ok(vedaksha_astro::houses::HouseSystem::Equal),
        "wholesign" | "whole_sign" => Ok(vedaksha_astro::houses::HouseSystem::WholeSign),
        "campanus" => Ok(vedaksha_astro::houses::HouseSystem::Campanus),
        "regiomontanus" => Ok(vedaksha_astro::houses::HouseSystem::Regiomontanus),
        "porphyry" => Ok(vedaksha_astro::houses::HouseSystem::Porphyry),
        "morinus" => Ok(vedaksha_astro::houses::HouseSystem::Morinus),
        "alcabitius" => Ok(vedaksha_astro::houses::HouseSystem::Alcabitius),
        "sripathi" => Ok(vedaksha_astro::houses::HouseSystem::Sripathi),
        _ => Err("unknown house system"),
    }
}

fn parse_house_system(s: &str) -> Result<vedaksha_astro::houses::HouseSystem, JsError> {
    house_system_from_str(s).map_err(|_| JsError::new(&format!("Unknown house system: {s}")))
}

fn ayanamsha_from_str(s: &str) -> Result<vedaksha_astro::sidereal::Ayanamsha, &'static str> {
    match s.to_lowercase().as_str() {
        "lahiri" => Ok(vedaksha_astro::sidereal::Ayanamsha::Lahiri),
        "faganbradley" | "fagan_bradley" => Ok(vedaksha_astro::sidereal::Ayanamsha::FaganBradley),
        "krishnamurti" => Ok(vedaksha_astro::sidereal::Ayanamsha::Krishnamurti),
        "raman" => Ok(vedaksha_astro::sidereal::Ayanamsha::Raman),
        "tropical" => Ok(vedaksha_astro::sidereal::Ayanamsha::Tropical),
        _ => Err("unknown ayanamsha"),
    }
}

fn parse_ayanamsha(s: &str) -> Result<vedaksha_astro::sidereal::Ayanamsha, JsError> {
    ayanamsha_from_str(s).map_err(|_| JsError::new(&format!("Unknown ayanamsha: {s}")))
}

fn varga_type_from_str(s: &str) -> Result<vedaksha_vedic::varga::VargaType, &'static str> {
    match s.to_lowercase().as_str() {
        "rashi" | "d1" | "d-1" => Ok(vedaksha_vedic::varga::VargaType::Rashi),
        "hora" | "d2" | "d-2" => Ok(vedaksha_vedic::varga::VargaType::Hora),
        "drekkana" | "d3" | "d-3" => Ok(vedaksha_vedic::varga::VargaType::Drekkana),
        "navamsha" | "d9" | "d-9" => Ok(vedaksha_vedic::varga::VargaType::Navamsha),
        "dashamsha" | "d10" | "d-10" => Ok(vedaksha_vedic::varga::VargaType::Dashamsha),
        "dwadashamsha" | "d12" | "d-12" => Ok(vedaksha_vedic::varga::VargaType::Dwadashamsha),
        "shashtiamsha" | "d60" | "d-60" => Ok(vedaksha_vedic::varga::VargaType::Shashtiamsha),
        _ => Err("unknown varga"),
    }
}

fn parse_varga_type(s: &str) -> Result<vedaksha_vedic::varga::VargaType, JsError> {
    varga_type_from_str(s).map_err(|_| JsError::new(&format!("Unknown varga: {s}")))
}

fn language_from_str(s: &str) -> Result<vedaksha::locale::Language, &'static str> {
    match s.to_lowercase().as_str() {
        "en" | "english" => Ok(vedaksha::locale::Language::English),
        "hi" | "hindi" => Ok(vedaksha::locale::Language::Hindi),
        "sa" | "sanskrit" => Ok(vedaksha::locale::Language::Sanskrit),
        "ta" | "tamil" => Ok(vedaksha::locale::Language::Tamil),
        "te" | "telugu" => Ok(vedaksha::locale::Language::Telugu),
        "kn" | "kannada" => Ok(vedaksha::locale::Language::Kannada),
        "bn" | "bengali" => Ok(vedaksha::locale::Language::Bengali),
        _ => Err("unknown language"),
    }
}

fn parse_language(s: &str) -> Result<vedaksha::locale::Language, JsError> {
    language_from_str(s).map_err(|_| JsError::new(&format!("Unknown language: {s}")))
}

fn compute_karakas_inner(positions_json: &str, scheme: &str) -> Result<String, String> {
    use vedaksha_vedic::karaka::{KarakaInput, KarakaScheme};

    let pos: serde_json::Value =
        serde_json::from_str(positions_json).map_err(|e| format!("invalid positions JSON: {e}"))?;

    let get = |key: &str| -> Result<f64, String> {
        pos.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))
    };

    let karaka_scheme = match scheme {
        "8" => KarakaScheme::Eight,
        "7" | "" => KarakaScheme::Seven,
        other => return Err(format!("unknown scheme '{other}'; use '7' or '8'")),
    };

    let rahu = if karaka_scheme == KarakaScheme::Eight {
        Some(get("Rahu")?)
    } else {
        pos.get("Rahu").and_then(|v| v.as_f64())
    };

    let input = KarakaInput {
        sun: get("Sun")?,
        moon: get("Moon")?,
        mars: get("Mars")?,
        mercury: get("Mercury")?,
        jupiter: get("Jupiter")?,
        venus: get("Venus")?,
        saturn: get("Saturn")?,
        rahu,
        scheme: karaka_scheme,
    };

    let assignments = vedaksha_vedic::karaka::compute_karakas(&input);
    serde_json::to_string(&assignments).map_err(|e| e.to_string())
}

/// Compute Jaimini Chara Karaka assignments from sidereal planet longitudes.
///
/// # Arguments
/// * `positions_json` — JSON object with keys `"Sun"`, `"Moon"`, `"Mars"`,
///   `"Mercury"`, `"Jupiter"`, `"Venus"`, `"Saturn"`, and optionally `"Rahu"`.
///   All values are sidereal longitudes in degrees [0, 360).
/// * `scheme` — `"7"` (default, Sun–Saturn) or `"8"` (adds Rahu + Pitrikaraka).
///
/// # Returns
/// JSON array of `{ "planet": "...", "karaka": "...", "degrees_in_sign": f64 }`.
#[wasm_bindgen]
pub fn compute_karakas(positions_json: &str, scheme: &str) -> Result<String, JsError> {
    compute_karakas_inner(positions_json, scheme).map_err(|e| JsError::new(&e))
}

fn compute_combustion_inner(positions_json: &str, retro_json: &str) -> Result<String, String> {
    use vedaksha_vedic::combustion::{CombustionState, combustion_state};
    use vedaksha_vedic::graha::Graha;

    let pos: serde_json::Value =
        serde_json::from_str(positions_json).map_err(|e| format!("invalid positions JSON: {e}"))?;
    let retro: serde_json::Value =
        serde_json::from_str(retro_json).map_err(|e| format!("invalid retro JSON: {e}"))?;

    let get_lon = |key: &str| -> Result<f64, String> {
        pos.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))
    };
    let get_bool =
        |key: &str| -> bool { retro.get(key).and_then(|v| v.as_bool()).unwrap_or(false) };

    let sun = get_lon("sun")?;
    let moon_lon = get_lon("moon")?;
    let mars_lon = get_lon("mars")?;
    let mercury_lon = get_lon("mercury")?;
    let jupiter_lon = get_lon("jupiter")?;
    let venus_lon = get_lon("venus")?;
    let saturn_lon = get_lon("saturn")?;

    let mars_retro = get_bool("mars");
    let mercury_retro = get_bool("mercury");
    let jupiter_retro = get_bool("jupiter");
    let venus_retro = get_bool("venus");
    let saturn_retro = get_bool("saturn");

    let sep = |lon: f64| -> f64 {
        let diff = (lon - sun).abs() % 360.0;
        if diff > 180.0 { 360.0 - diff } else { diff }
    };

    let entries: &[(Graha, f64, bool, &str)] = &[
        (Graha::Moon, moon_lon, false, "Moon"),
        (Graha::Mars, mars_lon, mars_retro, "Mars"),
        (Graha::Mercury, mercury_lon, mercury_retro, "Mercury"),
        (Graha::Jupiter, jupiter_lon, jupiter_retro, "Jupiter"),
        (Graha::Venus, venus_lon, venus_retro, "Venus"),
        (Graha::Saturn, saturn_lon, saturn_retro, "Saturn"),
    ];

    let results: Vec<serde_json::Value> = entries
        .iter()
        .map(|(planet, lon, retro_flag, name)| {
            let state = combustion_state(*planet, *lon, sun, *retro_flag);
            let state_str = match state {
                CombustionState::None => "None",
                CombustionState::Combust => "Combust",
                CombustionState::DeeplyCombust => "DeeplyCombust",
            };
            serde_json::json!({
                "planet": name,
                "state": state_str,
                "degrees_from_sun": sep(*lon),
            })
        })
        .collect();

    serde_json::to_string(&results).map_err(|e| e.to_string())
}

/// Compute combustion state for the 6 combustible planets relative to the Sun.
///
/// # Arguments
/// * `positions_json` — JSON object with lowercase keys: `"sun"`, `"moon"`, `"mars"`,
///   `"mercury"`, `"jupiter"`, `"venus"`, `"saturn"`. Values are sidereal longitudes [0, 360).
/// * `retro_json` — JSON object with boolean keys `"mars"`, `"mercury"`, `"jupiter"`,
///   `"venus"`, `"saturn"`. Absent keys default to `false`.
///
/// # Returns
/// JSON array of `{ "planet", "state", "degrees_from_sun" }` for the 6 combustible planets.
#[wasm_bindgen]
pub fn compute_combustion(positions_json: &str, retro_json: &str) -> Result<String, JsError> {
    compute_combustion_inner(positions_json, retro_json).map_err(|e| JsError::new(&e))
}

fn compute_shadbala_inner(input_json: &str) -> Result<String, String> {
    use vedaksha_vedic::graha::{Graha, GrahaPosition};
    use vedaksha_vedic::shadbala::{ShadbalaPlanetData, compute_shadbala_full};

    let v: serde_json::Value =
        serde_json::from_str(input_json).map_err(|e| format!("invalid JSON: {e}"))?;

    let is_daytime = v
        .get("is_daytime")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let moon_phase_waxing = v
        .get("moon_phase_waxing")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let planets_arr = v
        .get("planets")
        .and_then(|x| x.as_array())
        .ok_or_else(|| "missing 'planets' array".to_string())?;

    let parse_planet_name = |name: &str| -> Result<Graha, String> {
        match name.to_lowercase().as_str() {
            "sun" => Ok(Graha::Sun),
            "moon" => Ok(Graha::Moon),
            "mars" => Ok(Graha::Mars),
            "mercury" => Ok(Graha::Mercury),
            "jupiter" => Ok(Graha::Jupiter),
            "venus" => Ok(Graha::Venus),
            "saturn" => Ok(Graha::Saturn),
            other => Err(format!("unknown planet '{other}'")),
        }
    };

    let mut planet_data: Vec<ShadbalaPlanetData> = Vec::with_capacity(planets_arr.len());
    for entry in planets_arr {
        let planet_name = entry
            .get("planet")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "missing 'planet' field".to_string())?;
        let planet = parse_planet_name(planet_name)?;
        let sign = entry.get("sign").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
        let longitude = entry
            .get("longitude")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0);
        let bhava = entry.get("bhava").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
        let speed = entry.get("speed").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let average_speed = entry
            .get("average_speed")
            .and_then(|x| x.as_f64())
            .unwrap_or(1.0);
        let benefic = entry
            .get("benefic_aspect_count")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as u32;
        let malefic = entry
            .get("malefic_aspect_count")
            .and_then(|x| x.as_u64())
            .unwrap_or(0) as u32;
        planet_data.push(ShadbalaPlanetData {
            position: GrahaPosition {
                planet,
                sign,
                longitude,
                bhava,
            },
            speed,
            average_speed,
            benefic_aspect_count: benefic,
            malefic_aspect_count: malefic,
        });
    }

    let results = compute_shadbala_full(&planet_data, is_daytime, moon_phase_waxing);
    serde_json::to_string(&results).map_err(|e| e.to_string())
}

/// Compute full Shadbala for all supplied planets.
///
/// # Arguments
/// * `input_json` — JSON object with `"planets"` array plus optional `"is_daytime"` and
///   `"moon_phase_waxing"` booleans. Each planet: `planet` (string), `sign` (0–11),
///   `longitude` (0–360), `bhava` (1–12), `speed`, `average_speed`,
///   optional `benefic_aspect_count`, `malefic_aspect_count`.
///
/// # Returns
/// JSON array of Shadbala objects including `uccha_bala`, `ishta_phala`, `kashta_phala`.
#[wasm_bindgen]
pub fn compute_shadbala(input_json: &str) -> Result<String, JsError> {
    compute_shadbala_inner(input_json).map_err(|e| JsError::new(&e))
}

fn compute_ashtakavarga_inner(input_json: &str) -> Result<String, String> {
    use vedaksha_vedic::ashtakavarga::{
        BhinnaAshtakavargaInput, bhinna_ashtakavarga, sarvashtakavarga,
    };

    let v: serde_json::Value =
        serde_json::from_str(input_json).map_err(|e| format!("invalid JSON: {e}"))?;

    let get_sign = |key: &str| -> Result<u8, String> {
        let n = v
            .get(key)
            .and_then(|x| x.as_u64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))?;
        if n > 11 {
            return Err(format!("'{key}' must be 0–11, got {n}"));
        }
        Ok(n as u8)
    };

    let input = BhinnaAshtakavargaInput {
        sun: get_sign("sun")?,
        moon: get_sign("moon")?,
        mars: get_sign("mars")?,
        mercury: get_sign("mercury")?,
        jupiter: get_sign("jupiter")?,
        venus: get_sign("venus")?,
        saturn: get_sign("saturn")?,
        lagna: get_sign("lagna")?,
    };

    let tables = bhinna_ashtakavarga(&input);
    let sarva = sarvashtakavarga(&tables);

    serde_json::to_string(&serde_json::json!({
        "tables": tables,
        "sarvashtakavarga": sarva,
    }))
    .map_err(|e| e.to_string())
}

/// Compute Bhinna Ashtakavarga and Sarvashtakavarga from sign positions.
///
/// # Arguments
/// * `input_json` — JSON object with integer sign-index fields: `"sun"`, `"moon"`, `"mars"`,
///   `"mercury"`, `"jupiter"`, `"venus"`, `"saturn"`, `"lagna"`. Values 0–11.
///
/// # Returns
/// JSON object: `{ "tables": [...], "sarvashtakavarga": [u8; 12] }`.
#[wasm_bindgen]
pub fn compute_ashtakavarga(input_json: &str) -> Result<String, JsError> {
    compute_ashtakavarga_inner(input_json).map_err(|e| JsError::new(&e))
}

fn compute_gochara_inner(input_json: &str) -> Result<String, String> {
    use vedaksha_vedic::gochara::{
        SchoolProfile, TransitPositions, VedhaTable, apply_vedha_exemptions, compute_gochara,
    };

    let v: serde_json::Value =
        serde_json::from_str(input_json).map_err(|e| format!("invalid JSON: {e}"))?;

    let get_sign = |key: &str| -> Result<u8, String> {
        let n = v
            .get(key)
            .and_then(|x| x.as_u64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))?;
        if n > 11 {
            return Err(format!("'{key}' must be 0–11, got {n}"));
        }
        Ok(n as u8)
    };

    let transits = TransitPositions {
        sun: get_sign("sun")?,
        moon: get_sign("moon")?,
        mars: get_sign("mars")?,
        mercury: get_sign("mercury")?,
        jupiter: get_sign("jupiter")?,
        venus: get_sign("venus")?,
        saturn: get_sign("saturn")?,
    };
    let natal_reference_sign = get_sign("natal_reference_sign")?;

    let table = match v
        .get("vedha_table")
        .and_then(|x| x.as_str())
        .unwrap_or("Bphs29")
    {
        "Bphs29" => VedhaTable::Bphs29,
        other => return Err(format!("unknown vedha_table '{other}'")),
    };
    let school = match v
        .get("school")
        .and_then(|x| x.as_str())
        .unwrap_or("Geometry")
    {
        "Geometry" => SchoolProfile::Geometry,
        "Parashari" => SchoolProfile::Parashari,
        other => return Err(format!("unknown school '{other}'")),
    };

    let mut entries = compute_gochara(&transits, natal_reference_sign, table);
    for entry in entries.iter_mut() {
        apply_vedha_exemptions(entry, school);
    }

    serde_json::to_string(&serde_json::json!({ "entries": entries })).map_err(|e| e.to_string())
}

/// Compute Gochara (transit interpretation) per BPHS Ch.29.
///
/// # Arguments
/// * `input_json` — JSON object with integer sign-index fields for the seven
///   transiting grahas (`"sun"`, `"moon"`, …, `"saturn"`), `"natal_reference_sign"`
///   (0–11), and optional `"vedha_table"` ("Bphs29") and `"school"` ("Geometry"
///   or "Parashari").
///
/// # Returns
/// JSON object: `{ "entries": [GrahaGochara, …] }` for the seven non-nodal grahas.
#[wasm_bindgen]
pub fn compute_gochara(input_json: &str) -> Result<String, JsError> {
    compute_gochara_inner(input_json).map_err(|e| JsError::new(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests use the inner `_from_str` helpers which return `Result<T, &'static str>`
    // and are safe to call on native targets (no JsError construction).

    #[test]
    fn parse_house_systems() {
        assert!(house_system_from_str("placidus").is_ok());
        assert!(house_system_from_str("koch").is_ok());
        assert!(house_system_from_str("equal").is_ok());
        assert!(house_system_from_str("wholesign").is_ok());
        assert!(house_system_from_str("whole_sign").is_ok());
        assert!(house_system_from_str("campanus").is_ok());
        assert!(house_system_from_str("regiomontanus").is_ok());
        assert!(house_system_from_str("porphyry").is_ok());
        assert!(house_system_from_str("morinus").is_ok());
        assert!(house_system_from_str("alcabitius").is_ok());
        assert!(house_system_from_str("sripathi").is_ok());
    }

    #[test]
    fn parse_ayanamshas() {
        assert!(ayanamsha_from_str("lahiri").is_ok());
        assert!(ayanamsha_from_str("faganbradley").is_ok());
        assert!(ayanamsha_from_str("fagan_bradley").is_ok());
        assert!(ayanamsha_from_str("krishnamurti").is_ok());
        assert!(ayanamsha_from_str("raman").is_ok());
        assert!(ayanamsha_from_str("tropical").is_ok());
    }

    #[test]
    fn parse_varga_types() {
        assert!(varga_type_from_str("rashi").is_ok());
        assert!(varga_type_from_str("d1").is_ok());
        assert!(varga_type_from_str("d-1").is_ok());
        assert!(varga_type_from_str("hora").is_ok());
        assert!(varga_type_from_str("d2").is_ok());
        assert!(varga_type_from_str("drekkana").is_ok());
        assert!(varga_type_from_str("d3").is_ok());
        assert!(varga_type_from_str("navamsha").is_ok());
        assert!(varga_type_from_str("d9").is_ok());
        assert!(varga_type_from_str("dashamsha").is_ok());
        assert!(varga_type_from_str("d10").is_ok());
        assert!(varga_type_from_str("dwadashamsha").is_ok());
        assert!(varga_type_from_str("d12").is_ok());
        assert!(varga_type_from_str("shashtiamsha").is_ok());
        assert!(varga_type_from_str("d60").is_ok());
    }

    #[test]
    fn parse_languages() {
        assert!(language_from_str("en").is_ok());
        assert!(language_from_str("english").is_ok());
        assert!(language_from_str("hi").is_ok());
        assert!(language_from_str("hindi").is_ok());
        assert!(language_from_str("sa").is_ok());
        assert!(language_from_str("sanskrit").is_ok());
        assert!(language_from_str("ta").is_ok());
        assert!(language_from_str("tamil").is_ok());
        assert!(language_from_str("te").is_ok());
        assert!(language_from_str("telugu").is_ok());
        assert!(language_from_str("kn").is_ok());
        assert!(language_from_str("kannada").is_ok());
        assert!(language_from_str("bn").is_ok());
        assert!(language_from_str("bengali").is_ok());
    }

    #[test]
    fn compute_natal_chart_inner_known_chart() {
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
                "Sun".into(),
                "Moon".into(),
                "Mercury".into(),
                "Venus".into(),
                "Mars".into(),
                "Jupiter".into(),
                "Saturn".into(),
                "MeanNode".into(),
                "TrueNode".into(),
            ],
        };
        let result = compute_natal_chart_inner(input);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let output: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(output["planets"].is_array());
        assert!(output["houses"].is_object());
        assert!(output["aspects"].is_array());
        assert!(output["julian_day"].is_number());
        assert!(output["ayanamsha_value"].is_number());

        let planets = output["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 9);

        let asc = output["houses"]["asc"].as_f64().unwrap();
        assert!(asc > 0.0 && asc < 360.0, "ASC out of range: {asc}");

        let ayan = output["ayanamsha_value"].as_f64().unwrap();
        assert!(
            (ayan - 23.856).abs() < 0.1,
            "Lahiri should be ~23.856°, got {ayan}"
        );
    }

    /// A **served** sidereal request must produce a **sidereal** chart —
    /// the `vedaksha-wasm` twin of
    /// `vedaksha-mcp/tests/invariants.rs::served_sidereal_request_yields_a_sidereal_chart`.
    ///
    /// This surface exposes exactly the same chart path as the MCP handler
    /// (`compute_chart` fed a UT1 RAMC and a `ChartConfig`), so it inherits
    /// the same gap: nothing checked that the caller's `ayanamsha` string
    /// reached the config. `compute_natal_chart_inner_known_chart` above looks
    /// like it covers this and does not — `ayanamsha_value` is serialised from
    /// `ayanamsha_system` directly, on a separate line from the `ChartConfig`,
    /// so forcing `ayanamsha: Some(Ayanamsha::Tropical)` into the config
    /// leaves that assertion reading a healthy 23.856° while every longitude
    /// in the chart is tropical.
    ///
    /// # The property
    ///
    /// Two charts at the same instant, observer and house system, differing
    /// only in the `ayanamsha` field:
    ///
    /// ```text
    /// sidereal_x = normalize(tropical_x − ayanamsha_value(Lahiri, jd))
    /// ```
    ///
    /// for `asc`, `mc`, all twelve cusps and every planet longitude. No
    /// external oracle — the right-hand side comes from the same public
    /// `ayanamsha_value` a caller would use, and the identity is the
    /// definition of a sidereal frame.
    ///
    /// # Derivation
    ///
    /// 2000-01-01 00:00 UTC is `calendar_to_jd(2000, 1, 1.0)` = 2451544.5, so
    /// this is the same instant and observer as the MCP twin and must produce
    /// the same numbers:
    ///
    /// - `ayanamsha_value(Lahiri, 2451544.5)` = 23.857073774210527°
    /// - tropical `asc` = 255.288134034110612°, sidereal `asc` = 231.431060259900079°
    /// - 255.288134034110612 − 23.857073774210527 = 231.431060259900085
    ///
    /// Tolerance 1e-9° is a floating-point allowance: the observed residual is
    /// below 1e-13°, and the failure it must catch is 23.857°.
    ///
    /// # Mutation, measured
    ///
    /// Forcing `ayanamsha: Some(vedaksha_astro::sidereal::Ayanamsha::Tropical)`
    /// into the `ChartConfig` collapses the measured offset from
    /// 23.857073774211° to 0° and fails this test on `asc`, `mc`, all twelve
    /// cusps and every planet — while `compute_natal_chart_inner_known_chart`
    /// stays green.
    #[test]
    fn served_sidereal_request_yields_a_sidereal_chart() {
        use vedaksha_astro::sidereal::{Ayanamsha, ayanamsha_value};
        use vedaksha_math::angle::normalize_degrees;

        let served = |ayanamsha: &str| -> serde_json::Value {
            let input = NatalChartInput {
                year: 2000,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                latitude: 28.6139,
                longitude: 77.209,
                ayanamsha: ayanamsha.to_string(),
                house_system: "Placidus".to_string(),
                bodies: default_bodies(),
            };
            let json = compute_natal_chart_inner(input)
                .unwrap_or_else(|e| panic!("compute_natal_chart_inner({ayanamsha}) failed: {e}"));
            serde_json::from_str(&json).unwrap()
        };

        let trop = served("Tropical");
        let sid = served("Lahiri");

        let jd = trop["julian_day"].as_f64().expect("julian_day missing");
        assert!(
            (jd - 2_451_544.5).abs() < 1e-9,
            "expected calendar_to_jd(2000, 1, 1.0) = 2451544.5, got {jd}"
        );
        assert!(
            (sid["julian_day"].as_f64().unwrap() - jd).abs() < 1e-9,
            "the two charts must be at the same instant"
        );

        // Guard: a degenerate ayanamsha makes every assertion below vacuous.
        let ayan = ayanamsha_value(Ayanamsha::Lahiri, jd);
        assert!(
            ayan > 20.0,
            "ayanamsha_value(Lahiri, {jd}) = {ayan}°, expected ~23.86°. With a \
             near-zero ayanamsha this test cannot tell a sidereal chart from a \
             tropical one."
        );

        // Signed shortest separation, so the 0°/360° wrap cannot mask a
        // failure or manufacture one.
        let sep = |a: f64, b: f64| -> f64 {
            let d = normalize_degrees(a - b);
            if d > 180.0 { d - 360.0 } else { d }
        };
        const TOL: f64 = 1e-9;

        let mut angles: Vec<(String, f64, f64)> = vec![
            (
                "asc".to_string(),
                trop["houses"]["asc"].as_f64().expect("houses.asc missing"),
                sid["houses"]["asc"].as_f64().expect("houses.asc missing"),
            ),
            (
                "mc".to_string(),
                trop["houses"]["mc"].as_f64().expect("houses.mc missing"),
                sid["houses"]["mc"].as_f64().expect("houses.mc missing"),
            ),
        ];
        for i in 0..12 {
            angles.push((
                format!("cusp {}", i + 1),
                trop["houses"]["cusps"][i]
                    .as_f64()
                    .unwrap_or_else(|| panic!("houses.cusps[{i}] missing")),
                sid["houses"]["cusps"][i]
                    .as_f64()
                    .unwrap_or_else(|| panic!("houses.cusps[{i}] missing")),
            ));
        }

        for (label, t, s) in &angles {
            let offset = sep(*t, *s);
            assert!(
                (offset - ayan).abs() < TOL,
                "served {label}: tropical {t}°, sidereal {s}° — offset {offset}°, \
                 expected the ayanamsha {ayan}° (±{TOL}). An offset of 0° means \
                 the requested ayanamsha never reached the ChartConfig and the \
                 caller was served a tropical chart under a sidereal label."
            );
        }

        let tp = trop["planets"].as_array().expect("planets missing");
        let sp = sid["planets"].as_array().expect("planets missing");
        assert_eq!(tp.len(), sp.len());
        assert_eq!(tp.len(), 9, "expected the 9 default Jyotish bodies");
        for (t, s) in tp.iter().zip(sp.iter()) {
            let name = t["name"].as_str().unwrap_or("<unnamed>");
            let (tl, sl) = (
                t["longitude"].as_f64().expect("planet longitude missing"),
                s["longitude"].as_f64().expect("planet longitude missing"),
            );
            let offset = sep(tl, sl);
            assert!(
                (offset - ayan).abs() < TOL,
                "served planet {name}: tropical {tl}°, sidereal {sl}° — offset \
                 {offset}°, expected the ayanamsha {ayan}° (±{TOL})"
            );
            assert_eq!(
                t["house"], s["house"],
                "served planet {name} changed house between frames — a uniform \
                 rotation of the zodiac cannot move a planet between houses"
            );
        }

        // Secondary, deliberately last: the frame the chart *reports*. Weaker
        // than the arithmetic above, so it runs only after the numbers have
        // already proved the frame.
        assert_eq!(
            trop["config_summary"].as_str().unwrap(),
            "Houses: Placidus, Zodiac: Tropical, Rulership: Traditional"
        );
        assert_eq!(
            sid["config_summary"].as_str().unwrap(),
            "Houses: Placidus, Zodiac: Lahiri, Rulership: Traditional",
            "the served chart reports a frame other than the one requested"
        );
    }

    #[test]
    fn compute_natal_chart_inner_defaults() {
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
            bodies: vec![],
        };
        let result = compute_natal_chart_inner(input);
        assert!(result.is_ok());
        let output: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(output["planets"].as_array().unwrap().len(), 9);
    }

    #[test]
    fn compute_natal_chart_inner_error_cases() {
        let input = NatalChartInput {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0,
            latitude: 28.0,
            longitude: 77.0,
            ayanamsha: "FooBar".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec!["Sun".into()],
        };
        assert!(compute_natal_chart_inner(input).is_err());

        let input = NatalChartInput {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0,
            latitude: 28.0,
            longitude: 77.0,
            ayanamsha: "Lahiri".to_string(),
            house_system: "Topocentric".to_string(),
            bodies: vec!["Sun".into()],
        };
        assert!(compute_natal_chart_inner(input).is_err());
    }

    #[test]
    fn unknown_house_system_errors() {
        assert!(house_system_from_str("geocentric").is_err());
        assert!(house_system_from_str("").is_err());
        assert!(house_system_from_str("topocentric").is_err());
    }

    #[test]
    fn unknown_language_errors() {
        assert!(language_from_str("fr").is_err());
        assert!(language_from_str("").is_err());
        assert!(language_from_str("japanese").is_err());
    }

    mod shadbala_tests {
        use super::*;

        #[test]
        fn compute_shadbala_jupiter_retrograde() {
            let input = r#"{
                "planets": [{
                    "planet": "Jupiter", "sign": 3, "longitude": 105.0,
                    "bhava": 4, "speed": -0.05, "average_speed": 0.08,
                    "benefic_aspect_count": 2, "malefic_aspect_count": 1
                }],
                "is_daytime": true,
                "moon_phase_waxing": true
            }"#;
            let result = compute_shadbala_inner(input).unwrap();
            let arr: serde_json::Value = serde_json::from_str(&result).unwrap();
            let sb = &arr[0];
            assert_eq!(sb["planet"], "Jupiter");
            assert!(sb["total"].as_f64().unwrap() > 0.0);
            assert!(sb["uccha_bala"].as_f64().is_some());
            assert!(sb["ishta_phala"].as_f64().is_some());
            assert!(sb["kashta_phala"].as_f64().is_some());
            let ishta = sb["ishta_phala"].as_f64().unwrap();
            let kashta = sb["kashta_phala"].as_f64().unwrap();
            assert!((ishta + kashta - 60.0).abs() < 0.001);
        }

        #[test]
        fn compute_shadbala_missing_planets_errors() {
            let input = r#"{"is_daytime": true}"#;
            assert!(compute_shadbala_inner(input).is_err());
        }
    }
}

#[cfg(test)]
mod karaka_tests {
    #[test]
    fn compute_karakas_7_scheme_returns_json_array() {
        let positions = r#"{"Sun":25.0,"Moon":20.0,"Mars":15.0,"Mercury":10.0,"Jupiter":5.0,"Venus":2.0,"Saturn":1.0}"#;
        let result = super::compute_karakas_inner(positions, "7").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 7);
        assert_eq!(arr[0]["karaka"].as_str().unwrap(), "Atmakaraka");
        assert_eq!(arr[0]["planet"].as_str().unwrap(), "Sun");
    }

    #[test]
    fn compute_karakas_8_scheme_returns_eight_items() {
        let positions = r#"{"Sun":25.0,"Moon":20.0,"Mars":15.0,"Mercury":10.0,"Jupiter":5.0,"Venus":2.0,"Saturn":1.0,"Rahu":310.0}"#;
        let result = super::compute_karakas_inner(positions, "8").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 8);
    }

    #[test]
    fn compute_karakas_rejects_missing_planet() {
        // Moon is missing
        let positions =
            r#"{"Sun":25.0,"Mars":15.0,"Mercury":10.0,"Jupiter":5.0,"Venus":2.0,"Saturn":1.0}"#;
        assert!(super::compute_karakas_inner(positions, "7").is_err());
    }
}

/// Compute the panchanga — the five limbs of the Vedic almanac — for an instant.
///
/// # Arguments
/// * `jd` — Julian Day (UT) of the instant
/// * `sun` — Sun's sidereal longitude in degrees [0, 360)
/// * `moon` — Moon's sidereal longitude in degrees [0, 360)
/// * `latitude` — Observer latitude in degrees [-90, 90]. Required — the vara
///   is reckoned from local sunrise, so it depends on the observer, not `jd`
///   alone.
/// * `longitude` — Observer longitude in degrees [-180, 180], east positive
/// * `elevation_m` — Observer elevation above sea level in metres, in
///   [−500, 9000] (default the caller's own 0.0 for sea level). Lowers the
///   horizon by the dip and so moves sunrise — at 3650 m (Lhasa) 9.2 minutes
///   earlier, measured — which can change the vara itself, not merely refine
///   the Kalam windows.
/// * `tz_offset_minutes` — Offset of the observer's civil clock from UT, in
///   minutes, in [−720, 840] (UTC−12:00 to UTC+14:00); used only to name the
///   vara's weekday
///
/// # Returns
/// JSON string with tithi, vara (weekday reckoned from local sunrise, with
/// its lord and the Rahu and Gulika Kalam windows as Julian Days),
/// nakshatra, yoga and karana.
///
/// `vara.from_sunrise` reports HOW the weekday was reckoned. `true` means it
/// came from an actual local sunrise — the Vedic definition. `false` means no
/// sunrise exists to reckon from (the polar day or polar night, above about
/// ±66.5° latitude, or an ephemeris the engine could not evaluate) and the
/// value is the observer's local **civil** weekday, emitted as a documented
/// fallback. Those are different quantities, and a caller presenting a vara at
/// high latitude must check this flag: an unflagged civil weekday is exactly
/// the defect the sunrise reckoning exists to fix.
///
/// `vara.rahu_kalam == null` is NOT the same signal. The Kalam windows are
/// also null when a sunrise WAS found but the following sunset was not, where
/// `from_sunrise` is still `true`.
///
/// The key is spelled and positioned identically in `vedaksha-mcp`'s
/// `compute_panchanga`; the two surfaces are compared by exact JSON equality.
///
/// # Errors
/// Returns [`JsError`] when a longitude is outside [0, 360), `jd` is not
/// finite, an observer coordinate is out of range or non-finite,
/// `elevation_m` is outside [−500, 9000], or `tz_offset_minutes` is outside
/// [−720, 840].
#[wasm_bindgen]
pub fn compute_panchanga(
    jd: f64,
    sun: f64,
    moon: f64,
    latitude: f64,
    longitude: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
) -> Result<String, JsError> {
    compute_panchanga_inner(
        jd,
        sun,
        moon,
        latitude,
        longitude,
        elevation_m,
        tz_offset_minutes,
    )
    .map_err(|e| JsError::new(&e))
}

fn compute_panchanga_inner(
    jd: f64,
    sun: f64,
    moon: f64,
    latitude: f64,
    longitude: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
) -> Result<String, String> {
    use vedaksha_astro::riseset::sun_equatorial_deg;
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_vedic::muhurta::{Paksha, Weekday, compute_tithi};
    use vedaksha_vedic::nakshatra::Nakshatra;
    use vedaksha_vedic::panchanga::{compute_karana, compute_panchanga_yoga};

    if !jd.is_finite() {
        return Err("jd must be a finite number".to_string());
    }
    for (name, lon) in [("sun", sun), ("moon", moon)] {
        if !lon.is_finite() || !(0.0..360.0).contains(&lon) {
            return Err(format!("{name} must be a finite number in [0, 360)"));
        }
    }
    if !latitude.is_finite() || !(-90.0..=90.0).contains(&latitude) {
        return Err("latitude must be a finite number in [-90, 90]".to_string());
    }
    if !longitude.is_finite() || !(-180.0..=180.0).contains(&longitude) {
        return Err("longitude must be a finite number in [-180, 180]".to_string());
    }
    // Bounds, not just finiteness — and the SAME bounds `vedaksha-mcp`'s
    // `validation::validate_elevation_m` applies (−500 m, below the Dead Sea
    // shore, to 9000 m, above Everest). An absurd finite value such as 1e9
    // would otherwise be accepted and produce a horizon dip of −926°, a
    // sunrise search that can never find a crossing, reported as a
    // polar-style fallback rather than as the bad input it is.
    if !elevation_m.is_finite() || !(-500.0..=9000.0).contains(&elevation_m) {
        return Err(
            "elevation_m must be a finite number of metres above sea level in [-500, 9000]"
                .to_string(),
        );
    }
    // `tz_offset_minutes` is validated here against the same bound
    // `vedaksha-mcp`'s `compute_panchanga::validate` applies via
    // `validation::validate_tz_offset_minutes` (−720..=840): one engine, one
    // contract. (Until this fix, `vedaksha-mcp`'s `search_muhurta` called
    // that validator but `compute_panchanga` did not — so the wasm caller
    // and the MCP `compute_panchanga` caller disagreed; `search_muhurta`
    // was never the mismatched one.) Real UTC offsets run from −12:00
    // (Baker Island) to +14:00 (Kiribati's Line Islands); anything outside
    // that names no real observer's civil clock.
    if !(-720..=840).contains(&tz_offset_minutes) {
        return Err(
            "tz_offset_minutes must be between -720 and 840 (UTC-12:00 to UTC+14:00)".to_string(),
        );
    }

    let tithi = compute_tithi(moon, sun);
    // `AnalyticalProvider` as a plain local, matching the pattern used in
    // `vedaksha-mcp/src/server.rs::call_compute_panchanga`.
    let provider = AnalyticalProvider;
    let sun_eq = |j: f64| sun_equatorial_deg(&provider, j);
    // ONE sunrise scan for both the vara and the kalam windows — mirrors the
    // MCP handler exactly: `kalam_windows` returns the vara it derived
    // internally, so there is no second, separate `vara_at` call here
    // re-running the same day scan a second time.
    let reckoning = vedaksha_vedic::muhurta::kalam_windows(
        jd,
        latitude,
        longitude,
        elevation_m,
        tz_offset_minutes,
        &sun_eq,
    );
    let (weekday, kalams) = (reckoning.vara, reckoning.windows);
    let nakshatra = Nakshatra::from_longitude(moon);
    let pada = Nakshatra::pada_from_longitude(moon);
    let yoga = compute_panchanga_yoga(sun, moon);
    let karana = compute_karana(moon, sun);

    let weekday_name = match weekday {
        Weekday::Sunday => "Sunday",
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
    };
    let paksha = match tithi.paksha() {
        Paksha::Shukla => "Shukla",
        Paksha::Krishna => "Krishna",
    };

    let out = serde_json::json!({
        "tithi": {
            "number": tithi.number,
            "name": tithi.name,
            "paksha": paksha,
            "lord": tithi.lord(),
        },
        "vara": {
            "weekday": weekday_name,
            // Mirrors `vedaksha-mcp`'s `call_compute_panchanga` key for key:
            // `true` = reckoned from an actual local sunrise (a vara),
            // `false` = the civil-weekday fallback of the polar day/night,
            // which is a different quantity. The Python conformance harness
            // compares the two surfaces' JSON by exact equality, so the
            // spelling and the position of this key must not drift.
            "from_sunrise": reckoning.from_sunrise,
            "lord": weekday.lord(),
            "rahu_kalam_slot": weekday.rahu_kalam_slot(),
            "gulika_kalam_slot": weekday.gulika_kalam_slot(),
            "rahu_kalam": kalams.map(|(r, _)| serde_json::json!({
                "start_jd": r.start_jd, "end_jd": r.end_jd
            })),
            "gulika_kalam": kalams.map(|(_, g)| serde_json::json!({
                "start_jd": g.start_jd, "end_jd": g.end_jd
            })),
        },
        "nakshatra": {
            "index": nakshatra.index(),
            "name": nakshatra.name(),
            "pada": pada,
        },
        "yoga": {
            "index": yoga.index,
            "name": yoga.name,
            "remaining_degrees": yoga.remaining_degrees,
        },
        "karana": {
            "index": karana.index,
            "name": karana.name,
            "is_fixed": karana.is_fixed,
        },
    });
    serde_json::to_string(&out).map_err(|e| format!("serialization failed: {e}"))
}

/// Compute graded graha drishti (Vedic sign aspects) for the nine grahas.
///
/// # Arguments
/// * `positions_json` — JSON object of graha name to sidereal longitude, e.g.
///   `{"sun":10.0,"moon":100.0,...}`. All nine grahas are required.
///
/// # Returns
/// JSON array of aspects with aspecting/aspected sign, strength and house distance.
///
/// # Errors
/// Returns [`JsError`] when the JSON is malformed or a graha is missing or out of range.
#[wasm_bindgen]
pub fn compute_drishti(positions_json: &str) -> Result<String, JsError> {
    compute_drishti_inner(positions_json).map_err(|e| JsError::new(&e))
}

fn compute_drishti_inner(positions_json: &str) -> Result<String, String> {
    use vedaksha_vedic::drishti::{AspectStrength, VedicPlanet, find_vedic_aspects};

    let pos: serde_json::Value =
        serde_json::from_str(positions_json).map_err(|e| format!("invalid positions JSON: {e}"))?;

    let get_sign = |key: &str| -> Result<u8, String> {
        let lon = pos
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| format!("missing or invalid field '{key}'"))?;
        if !lon.is_finite() || !(0.0..360.0).contains(&lon) {
            return Err(format!("{key} must be a finite number in [0, 360)"));
        }
        Ok((lon / 30.0).floor() as u8 % 12)
    };

    let placements = [
        (VedicPlanet::Sun, get_sign("sun")?),
        (VedicPlanet::Moon, get_sign("moon")?),
        (VedicPlanet::Mars, get_sign("mars")?),
        (VedicPlanet::Mercury, get_sign("mercury")?),
        (VedicPlanet::Jupiter, get_sign("jupiter")?),
        (VedicPlanet::Venus, get_sign("venus")?),
        (VedicPlanet::Saturn, get_sign("saturn")?),
        (VedicPlanet::Rahu, get_sign("rahu")?),
        (VedicPlanet::Ketu, get_sign("ketu")?),
    ];

    let planet_name = |p: VedicPlanet| match p {
        VedicPlanet::Sun => "Sun",
        VedicPlanet::Moon => "Moon",
        VedicPlanet::Mars => "Mars",
        VedicPlanet::Mercury => "Mercury",
        VedicPlanet::Jupiter => "Jupiter",
        VedicPlanet::Venus => "Venus",
        VedicPlanet::Saturn => "Saturn",
        VedicPlanet::Rahu => "Rahu",
        VedicPlanet::Ketu => "Ketu",
    };

    let aspects: Vec<serde_json::Value> = find_vedic_aspects(&placements)
        .into_iter()
        .map(|a| {
            let strength = match a.strength {
                AspectStrength::Full => "Full",
                AspectStrength::ThreeQuarter => "ThreeQuarter",
                AspectStrength::Half => "Half",
                AspectStrength::Quarter => "Quarter",
                AspectStrength::None => "None",
            };
            serde_json::json!({
                "aspecting_planet": planet_name(a.aspecting_planet),
                "aspecting_sign": a.aspecting_sign,
                "aspected_sign": a.aspected_sign,
                "strength": strength,
                "houses_away": a.houses_away,
            })
        })
        .collect();

    serde_json::to_string(&aspects).map_err(|e| format!("serialization failed: {e}"))
}

/// Compute the whole-sign bhava (house) chart from an ascendant.
///
/// # Arguments
/// * `ascendant` — sidereal longitude of the ascendant in degrees [0, 360)
/// * `planets_json` — JSON object of graha name to sidereal longitude, or `"{}"`
///   to omit placements
///
/// # Returns
/// JSON string with the lagna sign, the twelve bhavas with their
/// kendra/trikona/dusthana/upachaya classification, and any placed grahas.
///
/// # Errors
/// Returns [`JsError`] when the JSON is malformed or a longitude is out of range.
#[wasm_bindgen]
pub fn compute_bhavas(ascendant: f64, planets_json: &str) -> Result<String, JsError> {
    compute_bhavas_inner(ascendant, planets_json).map_err(|e| JsError::new(&e))
}

fn compute_bhavas_inner(ascendant: f64, planets_json: &str) -> Result<String, String> {
    use vedaksha_vedic::bhava::{
        compute_bhavas, is_dusthana, is_kendra, is_trikona, is_upachaya, planet_bhava,
    };

    if !ascendant.is_finite() || !(0.0..360.0).contains(&ascendant) {
        return Err("ascendant must be a finite number in [0, 360)".to_string());
    }
    let supplied: serde_json::Value =
        serde_json::from_str(planets_json).map_err(|e| format!("invalid planets JSON: {e}"))?;

    let chart = compute_bhavas(ascendant);

    let houses: Vec<serde_json::Value> = (1u8..=12)
        .map(|bhava| {
            serde_json::json!({
                "bhava": bhava,
                "sign": chart.house_signs[(bhava - 1) as usize],
                "is_kendra": is_kendra(bhava),
                "is_trikona": is_trikona(bhava),
                "is_dusthana": is_dusthana(bhava),
                "is_upachaya": is_upachaya(bhava),
            })
        })
        .collect();

    let mut placements = Vec::new();
    if let Some(map) = supplied.as_object() {
        for (name, value) in map {
            let lon = value
                .as_f64()
                .ok_or_else(|| format!("longitude for '{name}' must be a number"))?;
            if !lon.is_finite() || !(0.0..360.0).contains(&lon) {
                return Err(format!(
                    "longitude for '{name}' must be a finite number in [0, 360)"
                ));
            }
            let sign = (lon / 30.0).floor() as u8 % 12;
            placements.push(serde_json::json!({
                "planet": name,
                "sign": sign,
                "bhava": planet_bhava(sign, &chart),
            }));
        }
    }

    let out = serde_json::json!({
        "lagna_sign": chart.lagna_sign,
        "houses": houses,
        "planets": placements,
    });
    serde_json::to_string(&out).map_err(|e| format!("serialization failed: {e}"))
}

#[cfg(test)]
mod combustion_tests {
    use super::*;

    #[test]
    fn compute_combustion_moon_combust() {
        let pos = r#"{"sun":0.0,"moon":5.0,"mars":100.0,"mercury":200.0,"jupiter":300.0,"venus":50.0,"saturn":150.0}"#;
        let retro = r#"{}"#;
        let result = compute_combustion_inner(pos, retro).unwrap();
        let arr: serde_json::Value = serde_json::from_str(&result).unwrap();
        let moon = &arr[0];
        assert_eq!(moon["planet"], "Moon");
        assert_eq!(moon["state"], "Combust");
    }

    #[test]
    fn compute_combustion_moon_not_combust() {
        let pos = r#"{"sun":0.0,"moon":20.0,"mars":100.0,"mercury":200.0,"jupiter":300.0,"venus":50.0,"saturn":150.0}"#;
        let retro = r#"{}"#;
        let result = compute_combustion_inner(pos, retro).unwrap();
        let arr: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(arr[0]["state"], "None");
    }

    #[test]
    fn compute_combustion_missing_field_errors() {
        let pos = r#"{"sun":0.0}"#;
        let retro = r#"{}"#;
        assert!(compute_combustion_inner(pos, retro).is_err());
    }
}

#[cfg(test)]
mod ashtakavarga_tests {
    use super::*;

    #[test]
    fn compute_ashtakavarga_canonical_sun_total() {
        let input = r#"{"sun":3,"moon":7,"mars":1,"mercury":10,"jupiter":5,"venus":8,"saturn":11,"lagna":0}"#;
        let result = compute_ashtakavarga_inner(input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        let sun_total = v["tables"][0]["total"].as_u64().unwrap();
        assert_eq!(sun_total, 48, "Sun total must be 48");
    }

    #[test]
    fn compute_ashtakavarga_sarva_grand_total() {
        // Grand total = 48+49+39+54+56+52+39 = 337
        let input = r#"{"sun":3,"moon":7,"mars":1,"mercury":10,"jupiter":5,"venus":8,"saturn":11,"lagna":0}"#;
        let result = compute_ashtakavarga_inner(input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        let sarva = v["sarvashtakavarga"].as_array().unwrap();
        let total: u64 = sarva.iter().map(|x| x.as_u64().unwrap_or(0)).sum();
        assert_eq!(total, 337);
    }

    #[test]
    fn compute_ashtakavarga_missing_field_errors() {
        let input = r#"{"sun":0}"#;
        assert!(compute_ashtakavarga_inner(input).is_err());
    }
}

#[cfg(test)]
mod gochara_tests {
    use super::*;

    #[test]
    fn compute_gochara_returns_seven_entries() {
        let input = r#"{
            "sun":0,"moon":4,"mars":2,"mercury":6,"jupiter":8,"venus":10,"saturn":6,
            "natal_reference_sign":0
        }"#;
        let result = compute_gochara_inner(input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        let entries = v["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 7);
    }

    #[test]
    fn compute_gochara_parashari_school_strips_sun_moon_vedha() {
        // Reference sign 0; Moon at sign 0 (1st house, vedha 5);
        // Sun at sign 4 (5th house) — geometric mutual vedha pair.
        let input = r#"{
            "sun":4,"moon":0,"mars":1,"mercury":2,"jupiter":3,"venus":5,"saturn":6,
            "natal_reference_sign":0,
            "school":"Parashari"
        }"#;
        let result = compute_gochara_inner(input).unwrap();
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        let moon = v["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["graha"] == "Moon")
            .unwrap();
        let candidates = moon["vedha_candidates"].as_array().unwrap();
        for c in candidates {
            assert_ne!(c.as_str().unwrap(), "Sun");
        }
    }

    #[test]
    fn compute_gochara_rejects_bad_sign() {
        let input = r#"{
            "sun":12,"moon":0,"mars":0,"mercury":0,"jupiter":0,"venus":0,"saturn":0,
            "natal_reference_sign":0
        }"#;
        assert!(compute_gochara_inner(input).is_err());
    }
}

#[cfg(test)]
mod panchanga_drishti_bhava_tests {
    use super::*;

    // Chennai coordinates (13.08N, 80.27E) matching the MCP `compute_panchanga`
    // test fixture at
    // `crates/vedaksha-mcp/src/tools/compute_panchanga.rs::valid_input`.
    // The `sun`/`moon` sidereal longitudes below (280.0, 223.3238) match
    // `compute_panchanga_inner_j2000_is_saturday`'s existing fixture.

    #[test]
    fn compute_panchanga_inner_j2000_is_saturday() {
        let out =
            compute_panchanga_inner(2_451_545.0, 280.0, 223.3238, 13.08, 80.27, 0.0, 330).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["vara"]["weekday"], "Saturday");
        assert_eq!(v["vara"]["lord"], "Saturn");
        assert_eq!(v["nakshatra"]["name"], "Anuradha");
        assert_eq!(v["tithi"]["paksha"], "Krishna");
        // Rahu/Gulika Kalam must be present as real time windows, not just
        // the slot number.
        assert!(v["vara"]["rahu_kalam"]["start_jd"].is_number());
        assert!(v["vara"]["rahu_kalam"]["end_jd"].is_number());
        assert!(v["vara"]["gulika_kalam"]["start_jd"].is_number());
        assert!(v["vara"]["gulika_kalam"]["end_jd"].is_number());
    }

    /// Pins the vara to a real sunrise-based derivation, not the naive
    /// UT-civil-day weekday — this is the MUTATION-CHECK for "revert vara to
    /// `ut_weekday_from_jd(jd)`".
    ///
    /// All figures below were confirmed by direct call to
    /// `compute_panchanga_inner` (release build; see the task-5 derivation
    /// run), not hand-derived — the same methodology
    /// `vedaksha-vedic::muhurta`'s own tests use (e.g.
    /// `kalam_windows_selects_the_elevation_aware_vara_not_the_sea_level_one`).
    ///
    /// At `jd = 2_451_545.5` (2000-01-02 00:00 UT), Chennai (13.08N, 80.27E),
    /// elevation 0, `tz_offset_minutes = 330` (IST):
    /// - `ut_weekday_from_jd(jd)` = Sunday (the naive UT-civil-day answer —
    ///   `jd`'s fractional part is exactly 0.5, i.e. UT midnight, so the
    ///   civil day has just turned over).
    /// - The real, sunrise-based vara is Saturday: local sunrise in Chennai
    ///   on 2000-01-01 (JD ≈ 2451544.542 UT, Rahu/Gulika Kalam anchored
    ///   there) has not yet been followed by the next sunrise at `jd`, so
    ///   the previous day's vara (Saturday) is still current — a real
    ///   sunrise-to-sunrise vara diverges from the naive civil-day one by
    ///   design.
    #[test]
    fn compute_panchanga_inner_vara_is_sunrise_based_not_ut_civil_day() {
        use vedaksha_vedic::muhurta::ut_weekday_from_jd;

        let jd = 2_451_545.5;
        let out = compute_panchanga_inner(jd, 280.0, 223.3238, 13.08, 80.27, 0.0, 330).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();

        assert_eq!(v["vara"]["weekday"], "Saturday");
        assert_eq!(v["vara"]["lord"], "Saturn");
        assert_eq!(v["vara"]["rahu_kalam_slot"], 3);
        assert_eq!(v["vara"]["gulika_kalam_slot"], 1);

        // The naive UT-civil-day answer at this instant is a DIFFERENT
        // weekday. A reverted `ut_weekday_from_jd(jd)` implementation would
        // report "Sunday" here, not "Saturday" — this assertion is the
        // mutation-check.
        assert_ne!(
            v["vara"]["weekday"].as_str().unwrap(),
            format!("{:?}", ut_weekday_from_jd(jd)),
            "the sunrise-based vara must differ from the naive UT-civil-day \
             weekday at this instant, or this test cannot distinguish a \
             correct implementation from one reverted to ut_weekday_from_jd"
        );

        // Window instants (epsilon 1e-6 days ≈ 0.09 s — wide enough to
        // absorb the ~1-2 ULP cross-compilation float noise observed
        // between this crate's and vedaksha-mcp's independently compiled
        // binaries for the same inputs (see task-5 report), tight enough
        // that a wrong eighth-slot (which differs by whole eighths of a
        // ~12h daytime, i.e. tens of minutes) still fails loudly.
        let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(close(
            v["vara"]["gulika_kalam"]["start_jd"].as_f64().unwrap(),
            2_451_544.542_324_732
        ));
        assert!(close(
            v["vara"]["gulika_kalam"]["end_jd"].as_f64().unwrap(),
            2_451_544.601_552_454
        ));
        assert!(close(
            v["vara"]["rahu_kalam"]["start_jd"].as_f64().unwrap(),
            2_451_544.660_780_174_7
        ));
        assert!(close(
            v["vara"]["rahu_kalam"]["end_jd"].as_f64().unwrap(),
            2_451_544.720_007_896
        ));
    }

    /// MUTATION-CHECK for "drop `tz_offset_minutes`" (e.g. hardcoding 0
    /// internally instead of threading the caller's value through to
    /// `kalam_windows`).
    ///
    /// At the same `jd`/observer as the test above, confirmed by direct
    /// call: `tz_offset_minutes = 330` (IST) names the vara "Saturday", but
    /// `tz_offset_minutes = -330` (a symmetric offset the OTHER way) names
    /// it "Friday" — a different weekday, different lord, and different
    /// Rahu/Gulika slots. A `0`-hardcoded implementation would report
    /// "Saturday" for BOTH calls (confirmed separately: `tz_offset_minutes
    /// = 0` also yields "Saturday" here), so this test would pass through
    /// that mutation only if it failed to check the `-330` case — which is
    /// exactly why both calls are asserted.
    #[test]
    fn compute_panchanga_inner_vara_depends_on_tz_offset_minutes() {
        let jd = 2_451_545.5;
        let plus = compute_panchanga_inner(jd, 280.0, 223.3238, 13.08, 80.27, 0.0, 330).unwrap();
        let minus = compute_panchanga_inner(jd, 280.0, 223.3238, 13.08, 80.27, 0.0, -330).unwrap();
        let vp: serde_json::Value = serde_json::from_str(&plus).unwrap();
        let vm: serde_json::Value = serde_json::from_str(&minus).unwrap();

        assert_eq!(vp["vara"]["weekday"], "Saturday");
        assert_eq!(vm["vara"]["weekday"], "Friday");
        assert_ne!(vp["vara"]["weekday"], vm["vara"]["weekday"]);
        assert_ne!(
            vp["vara"]["gulika_kalam_slot"],
            vm["vara"]["gulika_kalam_slot"]
        );
    }

    /// Polar case: at 85N in northern-hemisphere winter (JD 2_451_545.0 =
    /// 2000-01-01 12:00 UT), the sun does not rise, so `kalam_windows`
    /// returns `None` for the kalam pair. Confirmed by direct call: the
    /// vara still falls back to the local civil weekday ("Saturday", same
    /// as `ut_weekday_from_jd(2_451_545.0)` since `tz_offset_minutes = 0`
    /// here).
    ///
    /// This pins the MCP contract's null-vs-absent choice: `rahu_kalam` and
    /// `gulika_kalam` must be present keys holding JSON `null`, not omitted
    /// — `serde_json::Value` indexing (`v["vara"]["rahu_kalam"]`) cannot
    /// tell "null" from "missing" apart (both come back as `Value::Null`),
    /// so this test uses `.get()` on the `vara` object explicitly.
    #[test]
    fn compute_panchanga_inner_polar_case_kalam_is_present_null() {
        let out = compute_panchanga_inner(2_451_545.0, 280.0, 223.3238, 85.0, 0.0, 0.0, 0).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["vara"]["weekday"], "Saturday");

        let vara = v["vara"].as_object().unwrap();
        assert!(vara.contains_key("rahu_kalam"), "key must be present");
        assert!(vara.contains_key("gulika_kalam"), "key must be present");
        assert!(vara["rahu_kalam"].is_null());
        assert!(vara["gulika_kalam"].is_null());
    }

    /// FIX 1. `vara.from_sunrise` must be `true` wherever a sunrise actually
    /// bounds the vara. Same observer and instant as
    /// `compute_panchanga_inner_j2000_is_saturday` above, and the exact
    /// mirror of `vedaksha-mcp`'s
    /// `compute_panchanga_from_sunrise_is_true_at_a_mid_latitude` — the two
    /// surfaces are compared by exact JSON equality, so the key must exist,
    /// be spelled the same, and carry the same value on both.
    #[test]
    fn compute_panchanga_inner_from_sunrise_is_true_at_a_mid_latitude() {
        let out =
            compute_panchanga_inner(2_451_545.0, 280.0, 223.3238, 13.08, 80.27, 0.0, 330).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["vara"]["from_sunrise"], true);
        assert!(v["vara"]["rahu_kalam"]["start_jd"].is_number());
    }

    /// FIX 1, the case that matters. Above the Arctic Circle in local summer
    /// the Sun never sets and so never rises: the reported `weekday` is the
    /// observer's CIVIL weekday, a different quantity from a vara, and this
    /// flag is the only thing that says so. Emitting the fallback unflagged
    /// was the original UT-weekday defect surviving at high latitude.
    ///
    /// Ny-Ålesund (78.22 N, 15.65 E) at JD 2459016.0 = 2020-06-15 12:00 UT,
    /// midnight sun. Verified by direct call: `previous_rise` is `None`
    /// there. `tz_offset_minutes = 60` names the fallback Monday, and
    /// 2020-06-15 was a Monday.
    ///
    /// This is also the discriminating half of the pair: a flag hardcoded to
    /// `true` fails here, one hardcoded to `false` fails the mid-latitude
    /// test above.
    #[test]
    fn compute_panchanga_inner_from_sunrise_is_false_in_the_polar_summer() {
        let out = compute_panchanga_inner(2_459_016.0, 84.0, 200.0, 78.22, 15.65, 0.0, 60).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["vara"]["from_sunrise"], false,
            "the midnight sun has no sunrise to reckon a vara from"
        );
        assert_eq!(v["vara"]["weekday"], "Monday", "the civil weekday fallback");

        // Present and boolean on this branch too — `serde_json` indexing
        // cannot tell an absent key from a null one.
        let vara = v["vara"].as_object().unwrap();
        assert!(vara.contains_key("from_sunrise"));
        assert!(vara["from_sunrise"].is_boolean());

        // And `rahu_kalam: null` is NOT the same signal — it is null here,
        // but it is also null in the sunrise-found/sunset-missing case where
        // `from_sunrise` is true, which is why the flag exists separately.
        assert!(vara["rahu_kalam"].is_null());
    }

    /// FIX 3. `tz_offset_minutes` is validated by `vedaksha-mcp`'s
    /// `compute_panchanga::validate` (`validation::validate_tz_offset_minutes`,
    /// −720..=840) and, until this fix, was not validated at all here: one
    /// engine, two contracts, with the wasm caller silently getting a wrong
    /// vara where the MCP `compute_panchanga` caller got an error. (MCP's
    /// `search_muhurta` called the same validator throughout; the gap was
    /// specific to `compute_panchanga`, on both surfaces at once until this
    /// fix.) The boundaries are the real extremes of civil time — UTC−12:00
    /// (Baker Island) and UTC+14:00 (Kiribati's Line Islands).
    #[test]
    fn compute_panchanga_inner_validates_tz_offset_minutes_like_the_mcp_surface() {
        let call =
            |tz: i32| compute_panchanga_inner(2_451_545.0, 280.0, 223.3238, 13.08, 80.27, 0.0, tz);
        assert!(call(-720).is_ok(), "UTC-12:00 is a real offset");
        assert!(call(840).is_ok(), "UTC+14:00 is a real offset");
        assert!(call(-721).is_err());
        assert!(call(841).is_err());
        assert!(call(100_000).is_err());
    }

    /// FIX 3. `elevation_m` was finiteness-checked only, on both surfaces. A
    /// finite but absurd 1e9 m yields a horizon dip of −0.0293·√1e9 = −926°,
    /// a "sunrise" the search can never find — reported as a polar-style
    /// fallback rather than as the bad input it is. Bounds mirror
    /// `vedaksha-mcp`'s `validation::ELEVATION_MIN_M`/`ELEVATION_MAX_M`:
    /// −500 m (below the −430 m Dead Sea shore, the lowest exposed land) to
    /// 9000 m (above Everest's 8848.86 m).
    #[test]
    fn compute_panchanga_inner_validates_elevation_range_like_the_mcp_surface() {
        let call =
            |e: f64| compute_panchanga_inner(2_451_545.0, 280.0, 223.3238, 13.08, 80.27, e, 330);
        assert!(call(-500.0).is_ok());
        assert!(call(0.0).is_ok());
        assert!(call(3650.0).is_ok(), "Lhasa is a real observer");
        assert!(call(9000.0).is_ok());
        assert!(call(-500.1).is_err());
        assert!(call(9000.1).is_err());
        assert!(call(1e9).is_err());
        assert!(call(f64::NAN).is_err());
    }

    #[test]
    fn compute_panchanga_inner_rejects_out_of_range() {
        assert!(compute_panchanga_inner(2_451_545.0, 360.0, 10.0, 13.08, 80.27, 0.0, 330).is_err());
        assert!(compute_panchanga_inner(f64::NAN, 10.0, 10.0, 13.08, 80.27, 0.0, 330).is_err());
        assert!(compute_panchanga_inner(2_451_545.0, 10.0, 10.0, 91.0, 80.27, 0.0, 330).is_err());
        assert!(compute_panchanga_inner(2_451_545.0, 10.0, 10.0, 13.08, 200.0, 0.0, 330).is_err());
        assert!(
            compute_panchanga_inner(2_451_545.0, 10.0, 10.0, 13.08, 80.27, f64::NAN, 330).is_err()
        );
    }

    #[test]
    fn compute_drishti_inner_seventh_is_opposite_sign() {
        let positions = r#"{"sun":5.0,"moon":35.0,"mars":65.0,"mercury":95.0,
            "jupiter":125.0,"venus":155.0,"saturn":185.0,"rahu":215.0,"ketu":35.0}"#;
        let out = compute_drishti_inner(positions).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let seventh = v
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["aspecting_planet"] == "Sun" && a["houses_away"] == 7)
            .expect("Sun casts a 7th aspect");
        // Sun at 5 deg is in Aries (0); the 7th is Libra (6).
        assert_eq!(seventh["aspected_sign"], 6);
        assert_eq!(seventh["strength"], "Full");
    }

    #[test]
    fn compute_drishti_inner_rejects_missing_graha() {
        assert!(compute_drishti_inner(r#"{"sun":5.0}"#).is_err());
    }

    #[test]
    fn compute_bhavas_inner_places_grahas() {
        let out = compute_bhavas_inner(95.0, r#"{"Mars":200.4}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        // 95 deg is Cancer (3), so the lagna is Cancer and the 1st bhava is Cancer.
        assert_eq!(v["lagna_sign"], 3);
        assert_eq!(v["houses"][0]["sign"], 3);
        assert_eq!(v["houses"][0]["is_kendra"], true);
        // Mars at 200.4 deg is Libra (6); from a Cancer lagna that is the 4th.
        assert_eq!(v["planets"][0]["sign"], 6);
        assert_eq!(v["planets"][0]["bhava"], 4);
    }

    #[test]
    fn compute_bhavas_inner_rejects_bad_input() {
        assert!(compute_bhavas_inner(360.0, "{}").is_err());
        assert!(compute_bhavas_inner(10.0, "not json").is_err());
    }
}
