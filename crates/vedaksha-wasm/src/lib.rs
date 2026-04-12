// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-wasm
//!
//! WASM bindings for Vedākṣa, enabling browser-based astronomical
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

fn default_ayanamsha() -> String { "Lahiri".to_string() }
fn default_house_system() -> String { "Placidus".to_string() }

fn default_bodies() -> Vec<String> {
    vec!["Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "MeanNode", "TrueNode"]
        .into_iter().map(String::from).collect()
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

fn compute_natal_chart_inner(input: NatalChartInput) -> Result<String, String> {
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_ephem_core::coordinates;
    use vedaksha_ephem_core::julian;
    use vedaksha_ephem_core::jpl::EphemerisProvider;
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
        return Err(format!("Date out of range: JD {jd:.1} outside [{jd_min:.0}, {jd_max:.0}]"));
    }

    // Resolve bodies
    let body_names = if input.bodies.is_empty() { default_bodies() } else { input.bodies };

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
        &planet_data, ramc_deg, input.latitude, obliquity_deg, jd, &config,
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
            year: 2000, month: 1, day: 1, hour: 12, minute: 0, second: 0,
            latitude: 28.6139, longitude: 77.209,
            ayanamsha: "Lahiri".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec![
                "Sun".into(), "Moon".into(), "Mercury".into(), "Venus".into(),
                "Mars".into(), "Jupiter".into(), "Saturn".into(),
                "MeanNode".into(), "TrueNode".into(),
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
        assert!((ayan - 23.856).abs() < 0.1, "Lahiri should be ~23.856°, got {ayan}");
    }

    #[test]
    fn compute_natal_chart_inner_defaults() {
        let input = NatalChartInput {
            year: 1990, month: 6, day: 15, hour: 10, minute: 30, second: 0,
            latitude: 51.5074, longitude: -0.1278,
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
            year: 2000, month: 1, day: 1, hour: 12, minute: 0, second: 0,
            latitude: 28.0, longitude: 77.0,
            ayanamsha: "FooBar".to_string(),
            house_system: "Placidus".to_string(),
            bodies: vec!["Sun".into()],
        };
        assert!(compute_natal_chart_inner(input).is_err());

        let input = NatalChartInput {
            year: 2000, month: 1, day: 1, hour: 12, minute: 0, second: 0,
            latitude: 28.0, longitude: 77.0,
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
}
