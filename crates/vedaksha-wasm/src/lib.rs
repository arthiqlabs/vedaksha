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
/// * `birth_jd` — Julian Day of birth
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
    Ok(vedaksha_locale::planets::planet_name(index, lang).to_string())
}

/// Get localized name for a zodiac sign.
#[wasm_bindgen]
pub fn sign_name(index: usize, language: &str) -> Result<String, JsError> {
    let lang = parse_language(language)?;
    Ok(vedaksha_locale::signs::sign_name(index, lang).to_string())
}

/// Get localized name for a nakshatra.
#[wasm_bindgen]
pub fn nakshatra_name(index: usize, language: &str) -> Result<String, JsError> {
    let lang = parse_language(language)?;
    Ok(vedaksha_locale::nakshatras::nakshatra_name(index, lang).to_string())
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

    // Compute positions
    let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::new();
    for name in &body_names {
        let body = body_from_name(name).ok_or_else(|| format!("Unknown body: {name}"))?;
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

    // Sidereal time -> RAMC
    let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let eps_true = obliquity::true_obliquity(jd_tt, deps);
    let geo_lon_rad = input.longitude * core::f64::consts::PI / 180.0;
    let last = sidereal_time::local_sidereal_time(jd_tt, geo_lon_rad, dpsi, eps_true);
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
/// Input datetime is UTC.
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

fn language_from_str(s: &str) -> Result<vedaksha_locale::Language, &'static str> {
    match s.to_lowercase().as_str() {
        "en" | "english" => Ok(vedaksha_locale::Language::English),
        "hi" | "hindi" => Ok(vedaksha_locale::Language::Hindi),
        "sa" | "sanskrit" => Ok(vedaksha_locale::Language::Sanskrit),
        "ta" | "tamil" => Ok(vedaksha_locale::Language::Tamil),
        "te" | "telugu" => Ok(vedaksha_locale::Language::Telugu),
        "kn" | "kannada" => Ok(vedaksha_locale::Language::Kannada),
        "bn" | "bengali" => Ok(vedaksha_locale::Language::Bengali),
        _ => Err("unknown language"),
    }
}

fn parse_language(s: &str) -> Result<vedaksha_locale::Language, JsError> {
    language_from_str(s).map_err(|_| JsError::new(&format!("Unknown language: {s}")))
}

fn compute_karakas_inner(positions_json: &str, scheme: &str) -> Result<String, String> {
    use vedaksha_vedic::karaka::{KarakaInput, KarakaScheme};

    let pos: serde_json::Value = serde_json::from_str(positions_json)
        .map_err(|e| format!("invalid positions JSON: {e}"))?;

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
    use vedaksha_vedic::combustion::{combustion_state, CombustionState};
    use vedaksha_vedic::yoga::YogaPlanet;

    let pos: serde_json::Value = serde_json::from_str(positions_json)
        .map_err(|e| format!("invalid positions JSON: {e}"))?;
    let retro: serde_json::Value = serde_json::from_str(retro_json)
        .map_err(|e| format!("invalid retro JSON: {e}"))?;

    let get_lon = |key: &str| -> Result<f64, String> {
        pos.get(key).and_then(|v| v.as_f64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))
    };
    let get_bool = |key: &str| -> bool {
        retro.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    };

    let sun = get_lon("sun")?;
    let moon_lon = get_lon("moon")?;
    let mars_lon = get_lon("mars")?;
    let mercury_lon = get_lon("mercury")?;
    let jupiter_lon = get_lon("jupiter")?;
    let venus_lon = get_lon("venus")?;
    let saturn_lon = get_lon("saturn")?;

    let mars_retro    = get_bool("mars");
    let mercury_retro = get_bool("mercury");
    let jupiter_retro = get_bool("jupiter");
    let venus_retro   = get_bool("venus");
    let saturn_retro  = get_bool("saturn");

    let sep = |lon: f64| -> f64 {
        let diff = (lon - sun).abs() % 360.0;
        if diff > 180.0 { 360.0 - diff } else { diff }
    };

    let entries: &[(YogaPlanet, f64, bool, &str)] = &[
        (YogaPlanet::Moon,    moon_lon,    false,         "Moon"),
        (YogaPlanet::Mars,    mars_lon,    mars_retro,    "Mars"),
        (YogaPlanet::Mercury, mercury_lon, mercury_retro, "Mercury"),
        (YogaPlanet::Jupiter, jupiter_lon, jupiter_retro, "Jupiter"),
        (YogaPlanet::Venus,   venus_lon,   venus_retro,   "Venus"),
        (YogaPlanet::Saturn,  saturn_lon,  saturn_retro,  "Saturn"),
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
    use vedaksha_vedic::shadbala::{compute_shadbala_full, ShadbalaPlanetData};
    use vedaksha_vedic::yoga::{PlanetPosition, YogaPlanet};

    let v: serde_json::Value = serde_json::from_str(input_json)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    let is_daytime = v.get("is_daytime").and_then(|x| x.as_bool()).unwrap_or(false);
    let moon_phase_waxing = v.get("moon_phase_waxing").and_then(|x| x.as_bool()).unwrap_or(false);

    let planets_arr = v
        .get("planets")
        .and_then(|x| x.as_array())
        .ok_or_else(|| "missing 'planets' array".to_string())?;

    let parse_planet_name = |name: &str| -> Result<YogaPlanet, String> {
        match name.to_lowercase().as_str() {
            "sun"     => Ok(YogaPlanet::Sun),
            "moon"    => Ok(YogaPlanet::Moon),
            "mars"    => Ok(YogaPlanet::Mars),
            "mercury" => Ok(YogaPlanet::Mercury),
            "jupiter" => Ok(YogaPlanet::Jupiter),
            "venus"   => Ok(YogaPlanet::Venus),
            "saturn"  => Ok(YogaPlanet::Saturn),
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
        let longitude = entry.get("longitude").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let bhava = entry.get("bhava").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
        let speed = entry.get("speed").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let average_speed = entry.get("average_speed").and_then(|x| x.as_f64()).unwrap_or(1.0);
        let benefic = entry.get("benefic_aspect_count").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        let malefic = entry.get("malefic_aspect_count").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        planet_data.push(ShadbalaPlanetData {
            position: PlanetPosition { planet, sign, longitude, bhava },
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
    use vedaksha_vedic::ashtakavarga::{bhinna_ashtakavarga, sarvashtakavarga, BhinnaAshtakavargaInput};

    let v: serde_json::Value = serde_json::from_str(input_json)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    let get_sign = |key: &str| -> Result<u8, String> {
        let n = v.get(key).and_then(|x| x.as_u64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))?;
        if n > 11 {
            return Err(format!("'{key}' must be 0–11, got {n}"));
        }
        Ok(n as u8)
    };

    let input = BhinnaAshtakavargaInput {
        sun:     get_sign("sun")?,
        moon:    get_sign("moon")?,
        mars:    get_sign("mars")?,
        mercury: get_sign("mercury")?,
        jupiter: get_sign("jupiter")?,
        venus:   get_sign("venus")?,
        saturn:  get_sign("saturn")?,
        lagna:   get_sign("lagna")?,
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
        apply_vedha_exemptions, compute_gochara, SchoolProfile, TransitPositions, VedhaTable,
    };

    let v: serde_json::Value = serde_json::from_str(input_json)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    let get_sign = |key: &str| -> Result<u8, String> {
        let n = v.get(key).and_then(|x| x.as_u64())
            .ok_or_else(|| format!("missing or invalid field '{key}'"))?;
        if n > 11 {
            return Err(format!("'{key}' must be 0–11, got {n}"));
        }
        Ok(n as u8)
    };

    let transits = TransitPositions {
        sun:     get_sign("sun")?,
        moon:    get_sign("moon")?,
        mars:    get_sign("mars")?,
        mercury: get_sign("mercury")?,
        jupiter: get_sign("jupiter")?,
        venus:   get_sign("venus")?,
        saturn:  get_sign("saturn")?,
    };
    let natal_reference_sign = get_sign("natal_reference_sign")?;

    let table = match v.get("vedha_table").and_then(|x| x.as_str()).unwrap_or("Bphs29") {
        "Bphs29" => VedhaTable::Bphs29,
        other => return Err(format!("unknown vedha_table '{other}'")),
    };
    let school = match v.get("school").and_then(|x| x.as_str()).unwrap_or("Geometry") {
        "Geometry" => SchoolProfile::Geometry,
        "Parashari" => SchoolProfile::Parashari,
        other => return Err(format!("unknown school '{other}'")),
    };

    let mut entries = compute_gochara(&transits, natal_reference_sign, table);
    for entry in entries.iter_mut() {
        apply_vedha_exemptions(entry, school);
    }

    serde_json::to_string(&serde_json::json!({ "entries": entries }))
        .map_err(|e| e.to_string())
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
        let positions = r#"{"Sun":25.0,"Mars":15.0,"Mercury":10.0,"Jupiter":5.0,"Venus":2.0,"Saturn":1.0}"#;
        assert!(super::compute_karakas_inner(positions, "7").is_err());
    }
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
