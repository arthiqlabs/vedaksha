// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Python bindings for Vedākṣa via PyO3.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

/// Compute Vimshottari Dasha from Moon's sidereal longitude.
///
/// Args:
///     moon_longitude: Moon's sidereal longitude in degrees [0, 360)
///     birth_jd: Julian Day of birth
///     levels: Depth of sub-periods (1-5, default 3)
///
/// Returns:
///     JSON string with the dasha tree
#[pyfunction]
#[pyo3(signature = (moon_longitude, birth_jd, levels=3))]
fn compute_dasha(moon_longitude: f64, birth_jd: f64, levels: u8) -> PyResult<String> {
    let levels = levels.clamp(1, 5);
    let dasha = vedaksha_vedic::dasha::vimshottari::compute_vimshottari(
        moon_longitude, birth_jd, levels,
    );
    serde_json::to_string(&dasha)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Get nakshatra and pada for a sidereal longitude.
///
/// Args:
///     sidereal_longitude: Sidereal longitude in degrees [0, 360)
///
/// Returns:
///     dict with nakshatra name, index, pada, dasha_lord
#[pyfunction]
fn get_nakshatra(py: Python<'_>, sidereal_longitude: f64) -> PyResult<PyObject> {
    let nak = vedaksha_vedic::nakshatra::Nakshatra::from_longitude(sidereal_longitude);
    let pada = vedaksha_vedic::nakshatra::Nakshatra::pada_from_longitude(sidereal_longitude);
    let lord = nak.dasha_lord();

    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("nakshatra", nak.name())?;
    dict.set_item("index", nak.index())?;
    dict.set_item("pada", pada)?;
    dict.set_item("dasha_lord", format!("{lord:?}"))?;
    dict.set_item("start_longitude", nak.start_longitude())?;
    dict.set_item("end_longitude", nak.end_longitude())?;
    Ok(dict.into())
}

/// Compute varga (divisional chart) sign for a longitude.
///
/// Args:
///     longitude: Sidereal longitude in degrees
///     varga: Varga name ("Rashi", "Navamsha", "Dashamsha", etc.)
///
/// Returns:
///     Sign index (0-11)
#[pyfunction]
fn compute_varga(longitude: f64, varga: &str) -> PyResult<u8> {
    let varga_type = parse_varga(varga)?;
    Ok(vedaksha_vedic::varga::varga_sign(longitude, varga_type))
}

/// Compute house cusps.
///
/// Args:
///     ramc: Right Ascension of MC in degrees
///     latitude: Geographic latitude in degrees
///     obliquity: Obliquity of the ecliptic in degrees
///     system: House system name ("Placidus", "Equal", "WholeSign", etc.)
///
/// Returns:
///     dict with cusps (list of 12), asc, mc, polar_fallback
#[pyfunction]
#[pyo3(signature = (ramc, latitude, obliquity, system="Placidus"))]
fn compute_houses(py: Python<'_>, ramc: f64, latitude: f64, obliquity: f64, system: &str) -> PyResult<PyObject> {
    let house_system = parse_house_system(system)?;
    let cusps = vedaksha_astro::houses::compute_houses(ramc, latitude, obliquity, house_system);

    let dict = pyo3::types::PyDict::new(py);
    let cusps_list = pyo3::types::PyList::new(py, cusps.cusps.iter())?;
    dict.set_item("cusps", cusps_list)?;
    dict.set_item("asc", cusps.asc)?;
    dict.set_item("mc", cusps.mc)?;
    dict.set_item("system", format!("{:?}", cusps.system))?;
    dict.set_item("polar_fallback", cusps.polar_fallback)?;
    Ok(dict.into())
}

/// Convert tropical longitude to sidereal.
///
/// Args:
///     tropical_longitude: Tropical longitude in degrees
///     ayanamsha: Ayanamsha system ("Lahiri", "FaganBradley", etc.)
///     jd: Julian Day
///
/// Returns:
///     Sidereal longitude in degrees
#[pyfunction]
#[pyo3(signature = (tropical_longitude, ayanamsha="Lahiri", jd=2451545.0))]
fn tropical_to_sidereal(tropical_longitude: f64, ayanamsha: &str, jd: f64) -> PyResult<f64> {
    let system = parse_ayanamsha(ayanamsha)?;
    Ok(vedaksha_astro::sidereal::tropical_to_sidereal(tropical_longitude, system, jd))
}

/// Get ayanamsha value in degrees for a given date.
///
/// Args:
///     ayanamsha: System name ("Lahiri", "FaganBradley", etc.)
///     jd: Julian Day
///
/// Returns:
///     Ayanamsha value in degrees
#[pyfunction]
#[pyo3(signature = (ayanamsha="Lahiri", jd=2451545.0))]
fn get_ayanamsha(ayanamsha: &str, jd: f64) -> PyResult<f64> {
    let system = parse_ayanamsha(ayanamsha)?;
    Ok(vedaksha_astro::sidereal::ayanamsha_value(system, jd))
}

/// Convert calendar date to Julian Day.
///
/// Source: Jean Meeus, "Astronomical Algorithms" 2nd ed. (1998), Ch. 7, eq. 7.1.
///
/// Args:
///     year: Year (negative for BCE)
///     month: Month (1-12)
///     day: Day with fractional part for time
///
/// Returns:
///     Julian Day number
#[pyfunction]
fn calendar_to_jd(year: i32, month: u32, day: f64) -> f64 {
    let _ = vedaksha_math::angle::normalize_degrees(0.0); // ensure math crate is linked
    // We need to call ephem-core but can't without std feature...
    // Use a simple Meeus formula inline
    let (y, m) = if month <= 2 {
        (f64::from(year - 1), f64::from(month + 12))
    } else {
        (f64::from(year), f64::from(month))
    };
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day + b - 1524.5
}

/// Get planet name in a specified language.
///
/// Args:
///     index: Planet index (0=Sun, 1=Moon, 2=Mars, ...)
///     language: Language code ("en", "hi", "sa", "ta", "te", "kn", "bn")
///
/// Returns:
///     Localized planet name
#[pyfunction]
#[pyo3(signature = (index, language="en"))]
fn planet_name(index: usize, language: &str) -> PyResult<String> {
    let lang = parse_language(language)?;
    Ok(vedaksha_locale::planets::planet_name(index, lang).to_string())
}

/// Get sign name in a specified language.
#[pyfunction]
#[pyo3(signature = (index, language="en"))]
fn sign_name(index: usize, language: &str) -> PyResult<String> {
    let lang = parse_language(language)?;
    Ok(vedaksha_locale::signs::sign_name(index, lang).to_string())
}

/// Get nakshatra name in a specified language.
#[pyfunction]
#[pyo3(signature = (index, language="en"))]
fn nakshatra_name_i18n(index: usize, language: &str) -> PyResult<String> {
    let lang = parse_language(language)?;
    Ok(vedaksha_locale::nakshatras::nakshatra_name(index, lang).to_string())
}

/// Compute a complete natal chart from birth data.
///
/// Args:
///     year: Birth year (negative for BCE)
///     month: Birth month (1-12)
///     day: Birth day (1-31)
///     hour: Birth hour UTC (0-23)
///     minute: Birth minute (0-59)
///     latitude: Geographic latitude in degrees [-90, 90]
///     longitude: Geographic longitude in degrees [-180, 180]
///     ayanamsha: Ayanamsha system ("Lahiri", "FaganBradley", etc.)
///     house_system: House system ("Placidus", "WholeSign", etc.)
///
/// Returns:
///     JSON string with complete chart: planets, houses, aspects, ayanamsha value
#[pyfunction]
#[pyo3(signature = (year, month, day, hour, minute, latitude, longitude, ayanamsha="Lahiri", house_system="Placidus"))]
fn compute_natal_chart(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    latitude: f64,
    longitude: f64,
    ayanamsha: &str,
    house_system: &str,
) -> PyResult<String> {
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_ephem_core::bodies::Body;
    use vedaksha_ephem_core::coordinates;
    use vedaksha_ephem_core::jpl::EphemerisProvider;
    use vedaksha_ephem_core::nutation;
    use vedaksha_ephem_core::obliquity;
    use vedaksha_ephem_core::sidereal_time;

    let ayanamsha_system = parse_ayanamsha(ayanamsha)?;
    let house_sys = parse_house_system(house_system)?;

    // Calendar to JD (UTC)
    let day_fraction = day as f64
        + hour as f64 / 24.0
        + minute as f64 / 1440.0;
    let jd = calendar_to_jd(year, month, day_fraction);

    // Range check
    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();
    if jd < jd_min || jd > jd_max {
        return Err(PyValueError::new_err(format!(
            "Date out of range: JD {jd:.1} outside [{jd_min:.0}, {jd_max:.0}]"
        )));
    }

    // Compute positions for 9 standard Jyotish bodies
    let bodies: &[(&str, Body)] = &[
        ("Sun", Body::Sun),
        ("Moon", Body::Moon),
        ("Mercury", Body::Mercury),
        ("Venus", Body::Venus),
        ("Mars", Body::Mars),
        ("Jupiter", Body::Jupiter),
        ("Saturn", Body::Saturn),
        ("MeanNode", Body::MeanNode),
        ("TrueNode", Body::TrueNode),
    ];

    let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::new();
    for (name, body) in bodies {
        let pos = coordinates::apparent_position(&provider, *body, jd)
            .map_err(|e| PyValueError::new_err(format!("Failed to compute {name}: {e}")))?;
        planet_data.push((
            name.to_string(),
            pos.ecliptic.longitude.to_degrees(),
            pos.ecliptic.latitude.to_degrees(),
            pos.ecliptic.distance,
            pos.longitude_speed,
        ));
    }

    // Sidereal time → RAMC
    let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd);
    let (dpsi, deps) = nutation::nutation(jd_tt);
    let eps_true = obliquity::true_obliquity(jd_tt, deps);
    let geo_lon_rad = longitude * core::f64::consts::PI / 180.0;
    let last = sidereal_time::local_sidereal_time(jd_tt, geo_lon_rad, dpsi, eps_true);
    let ramc_deg = last * 180.0 / core::f64::consts::PI;

    // Obliquity in degrees
    let obliquity_deg = obliquity::mean_obliquity(jd_tt) * 180.0 / core::f64::consts::PI;

    // Chart config
    let config = vedaksha_astro::chart::ChartConfig {
        house_system: house_sys,
        ayanamsha: Some(ayanamsha_system),
        rulership_scheme: vedaksha_astro::dignity::RulershipScheme::Traditional,
        aspect_types: vedaksha_astro::aspects::AspectType::MAJOR.to_vec(),
        orb_factor: 1.0,
    };

    let chart = vedaksha_astro::chart::compute_chart(
        &planet_data, ramc_deg, latitude, obliquity_deg, jd, &config,
    );

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

    serde_json::to_string(&output).map_err(|e| PyValueError::new_err(e.to_string()))
}

// --- Parser helpers ---

fn parse_house_system(s: &str) -> PyResult<vedaksha_astro::houses::HouseSystem> {
    use vedaksha_astro::houses::HouseSystem;
    match s.to_lowercase().as_str() {
        "placidus" => Ok(HouseSystem::Placidus),
        "koch" => Ok(HouseSystem::Koch),
        "equal" => Ok(HouseSystem::Equal),
        "wholesign" | "whole_sign" => Ok(HouseSystem::WholeSign),
        "campanus" => Ok(HouseSystem::Campanus),
        "regiomontanus" => Ok(HouseSystem::Regiomontanus),
        "porphyry" => Ok(HouseSystem::Porphyry),
        "morinus" => Ok(HouseSystem::Morinus),
        "alcabitius" => Ok(HouseSystem::Alcabitius),
        "sripathi" => Ok(HouseSystem::Sripathi),
        _ => Err(PyValueError::new_err(format!("Unknown house system: {s}"))),
    }
}

fn parse_ayanamsha(s: &str) -> PyResult<vedaksha_astro::sidereal::Ayanamsha> {
    use vedaksha_astro::sidereal::Ayanamsha;
    match s.to_lowercase().as_str() {
        "lahiri" => Ok(Ayanamsha::Lahiri),
        "faganbradley" | "fagan_bradley" => Ok(Ayanamsha::FaganBradley),
        "krishnamurti" => Ok(Ayanamsha::Krishnamurti),
        "raman" => Ok(Ayanamsha::Raman),
        "tropical" => Ok(Ayanamsha::Tropical),
        _ => Err(PyValueError::new_err(format!("Unknown ayanamsha: {s}"))),
    }
}

fn parse_varga(s: &str) -> PyResult<vedaksha_vedic::varga::VargaType> {
    use vedaksha_vedic::varga::VargaType;
    match s.to_lowercase().as_str() {
        "rashi" | "d1" | "d-1" => Ok(VargaType::Rashi),
        "navamsha" | "d9" | "d-9" => Ok(VargaType::Navamsha),
        "dashamsha" | "d10" | "d-10" => Ok(VargaType::Dashamsha),
        "dwadashamsha" | "d12" | "d-12" => Ok(VargaType::Dwadashamsha),
        "shashtiamsha" | "d60" | "d-60" => Ok(VargaType::Shashtiamsha),
        _ => Err(PyValueError::new_err(format!("Unknown varga: {s}"))),
    }
}

fn parse_language(s: &str) -> PyResult<vedaksha_locale::Language> {
    use vedaksha_locale::Language;
    match s.to_lowercase().as_str() {
        "en" | "english" => Ok(Language::English),
        "hi" | "hindi" => Ok(Language::Hindi),
        "sa" | "sanskrit" => Ok(Language::Sanskrit),
        "ta" | "tamil" => Ok(Language::Tamil),
        "te" | "telugu" => Ok(Language::Telugu),
        "kn" | "kannada" => Ok(Language::Kannada),
        "bn" | "bengali" => Ok(Language::Bengali),
        _ => Err(PyValueError::new_err(format!("Unknown language: {s}"))),
    }
}

/// Vedākṣa Python module.
#[pymodule]
fn vedaksha(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_dasha, m)?)?;
    m.add_function(wrap_pyfunction!(get_nakshatra, m)?)?;
    m.add_function(wrap_pyfunction!(compute_varga, m)?)?;
    m.add_function(wrap_pyfunction!(compute_houses, m)?)?;
    m.add_function(wrap_pyfunction!(tropical_to_sidereal, m)?)?;
    m.add_function(wrap_pyfunction!(get_ayanamsha, m)?)?;
    m.add_function(wrap_pyfunction!(calendar_to_jd, m)?)?;
    m.add_function(wrap_pyfunction!(planet_name, m)?)?;
    m.add_function(wrap_pyfunction!(sign_name, m)?)?;
    m.add_function(wrap_pyfunction!(nakshatra_name_i18n, m)?)?;
    m.add_function(wrap_pyfunction!(compute_natal_chart, m)?)?;
    Ok(())
}
