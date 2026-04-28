// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! JSON-RPC 2.0 MCP server dispatcher.
//!
//! This module provides the core dispatch logic for the Model Context Protocol
//! server. It handles `initialize`, `tools/list`, and `tools/call` requests,
//! delegating computation to the individual tool handlers in [`crate::tools`].
//!
//! Transport (stdio, HTTP+SSE) is outside this module's scope. Callers feed
//! raw JSON-RPC strings in and receive JSON-RPC strings back.

use serde::{Deserialize, Serialize};

use crate::validation::McpError;

// ── Varga type parsing ────────────────────────────────────────────────────────

/// Parse a varga division string (e.g. `"D9"`, `"navamsha"`) into a
/// [`vedaksha_vedic::varga::VargaType`].
fn parse_varga_type(s: &str) -> Result<vedaksha_vedic::varga::VargaType, String> {
    use vedaksha_vedic::varga::VargaType;
    match s.to_lowercase().as_str() {
        "rashi" | "d1" | "d-1" => Ok(VargaType::Rashi),
        "hora" | "d2" | "d-2" => Ok(VargaType::Hora),
        "drekkana" | "d3" | "d-3" => Ok(VargaType::Drekkana),
        "chaturthamsha" | "d4" | "d-4" => Ok(VargaType::Chaturthamsha),
        "saptamsha" | "d7" | "d-7" => Ok(VargaType::Saptamsha),
        "navamsha" | "d9" | "d-9" => Ok(VargaType::Navamsha),
        "dashamsha" | "d10" | "d-10" => Ok(VargaType::Dashamsha),
        "dwadashamsha" | "d12" | "d-12" => Ok(VargaType::Dwadashamsha),
        "shodashamsha" | "d16" | "d-16" => Ok(VargaType::Shodashamsha),
        "vimshamsha" | "d20" | "d-20" => Ok(VargaType::Vimshamsha),
        "chaturvimshamsha" | "d24" | "d-24" => Ok(VargaType::ChaturVimshamsha),
        "saptavimshamsha" | "d27" | "d-27" => Ok(VargaType::Saptavimshamsha),
        "trimshamsha" | "d30" | "d-30" => Ok(VargaType::Trimshamsha),
        "khavedamsha" | "d40" | "d-40" => Ok(VargaType::Khavedamsha),
        "akshavedamsha" | "d45" | "d-45" => Ok(VargaType::Akshavedamsha),
        "shashtiamsha" | "d60" | "d-60" => Ok(VargaType::Shashtiamsha),
        _ => Err(format!("Unknown varga type: {s}")),
    }
}

// ── JSON-RPC 2.0 types ───────────────────────────────────────────────────────

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response (success or error).
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// ── Standard JSON-RPC error codes ────────────────────────────────────────────

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

// ── MCP server metadata ───────────────────────────────────────────────────────

/// MCP server metadata (returned in the `initialize` response).
#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub tools: Vec<serde_json::Value>,
}

// ── McpServer ─────────────────────────────────────────────────────────────────

/// The MCP server dispatcher.
///
/// Handles `initialize`, `tools/list`, and `tools/call` methods
/// per the MCP specification. All computation is stateless — no
/// fields are required.
pub struct McpServer {
    // No state needed — all computation is stateless.
}

impl McpServer {
    /// Create a new server instance.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Handle a raw JSON-RPC 2.0 request string and return a JSON-RPC response
    /// string.
    ///
    /// All errors (parse failures, unknown methods, validation failures) are
    /// encoded as JSON-RPC error responses rather than propagated as Rust
    /// errors, matching the protocol requirement that the transport always
    /// receives a well-formed response.
    #[must_use]
    pub fn handle_request(&self, request_json: &str) -> String {
        // 1. Parse the JSON-RPC request.
        let request: JsonRpcRequest = match serde_json::from_str(request_json) {
            Ok(r) => r,
            Err(e) => {
                return serde_json::to_string(&JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: PARSE_ERROR,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                })
                .unwrap_or_default();
            }
        };

        // 2. Dispatch to the appropriate handler.
        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request),
            "tools/list" => self.handle_tools_list(&request),
            "tools/call" => self.handle_tools_call(&request),
            _ => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        };

        serde_json::to_string(&response).unwrap_or_default()
    }

    // ── Method handlers ───────────────────────────────────────────────────────

    #[allow(clippy::unused_self)]
    fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id.clone(),
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "vedaksha-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        }
    }

    #[allow(clippy::unused_self)]
    fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let tools: Vec<serde_json::Value> = crate::tools::tool_definitions()
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                })
            })
            .collect();

        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: request.id.clone(),
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    #[allow(clippy::unused_self)]
    fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let Some(params) = &request.params else {
            return Self::error_response(&request.id, INVALID_PARAMS, "Missing params");
        };

        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        // Dispatch to the tool handler.
        let result = match tool_name {
            "compute_natal_chart" => Self::call_compute_natal(&arguments),
            "compute_dasha" => Self::call_compute_dasha(&arguments),
            "compute_karakas" => Self::call_compute_karakas(&arguments),
            "compute_combustion" => Self::call_compute_combustion(&arguments),
            "compute_vargas" => Self::call_compute_vargas(&arguments),
            "emit_graph" => Self::call_emit_graph(&arguments),
            "compute_transit" => Self::call_compute_transit(&arguments),
            "search_transits" => Self::call_search_transits(&arguments),
            "search_muhurta" => Self::call_search_muhurta(&arguments),
            _ => Err(McpError::invalid_parameter(
                "name",
                &format!("Unknown tool: {tool_name}"),
            )),
        };

        match result {
            Ok(value) => {
                // MCP requires content[].text to be a plain string.
                // If the tool returned a JSON string, use it directly.
                // If it returned an object/array, serialise to a JSON string.
                let text = match value {
                    serde_json::Value::String(s) => s,
                    other => serde_json::to_string(&other).unwrap_or_default(),
                };
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": text
                        }]
                    })),
                    error: None,
                }
            }
            Err(mcp_err) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_PARAMS,
                    message: mcp_err.message,
                    data: Some(serde_json::json!({
                        "error_code": mcp_err.error_code,
                        "suggested_action": mcp_err.suggested_action
                    })),
                }),
            },
        }
    }

    // ── Tool implementations ──────────────────────────────────────────────────
    //
    // Each function validates inputs via the tool's own `validate()` function
    // and dispatches to the underlying computation crates.

    fn call_compute_natal(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;
        use vedaksha_ephem_core::nutation;
        use vedaksha_ephem_core::obliquity;
        use vedaksha_ephem_core::sidereal_time;

        let input: crate::tools::compute_natal::ComputeNatalInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_natal::validate(&input)?;

        let provider = AnalyticalProvider;
        let jd = input.julian_day;

        // Compute positions for the 9 standard Jyotish bodies
        let bodies = [
            ("Sun", Body::Sun),
            ("Moon", Body::Moon),
            ("Mercury", Body::Mercury),
            ("Venus", Body::Venus),
            ("Mars", Body::Mars),
            ("Jupiter", Body::Jupiter),
            ("Saturn", Body::Saturn),
            ("MeanNode", Body::MeanNode),
            ("TrueNode", Body::TrueNode),
            ("TrueNodeOsculating", Body::TrueNodeOsculating),
        ];

        let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::new();
        for (name, body) in &bodies {
            let pos = coordinates::apparent_position(&provider, *body, jd).map_err(|e| {
                McpError::computation_failed(&format!("Failed to compute {name}: {e}"))
            })?;
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
        let geo_lon_rad = input.longitude * core::f64::consts::PI / 180.0;
        let last = sidereal_time::local_sidereal_time(jd_tt, geo_lon_rad, dpsi, eps_true);
        let ramc_deg = last * 180.0 / core::f64::consts::PI;

        // Obliquity in degrees
        let obliquity_deg = obliquity::mean_obliquity(jd_tt) * 180.0 / core::f64::consts::PI;

        // Parse house system and ayanamsha
        let house_system = match input.house_system.as_deref() {
            Some(s) => match s.to_lowercase().as_str() {
                "placidus" => vedaksha_astro::houses::HouseSystem::Placidus,
                "koch" => vedaksha_astro::houses::HouseSystem::Koch,
                "equal" => vedaksha_astro::houses::HouseSystem::Equal,
                "wholesign" | "whole_sign" => vedaksha_astro::houses::HouseSystem::WholeSign,
                "campanus" => vedaksha_astro::houses::HouseSystem::Campanus,
                "regiomontanus" => vedaksha_astro::houses::HouseSystem::Regiomontanus,
                "porphyry" => vedaksha_astro::houses::HouseSystem::Porphyry,
                "morinus" => vedaksha_astro::houses::HouseSystem::Morinus,
                "alcabitius" => vedaksha_astro::houses::HouseSystem::Alcabitius,
                "sripathi" => vedaksha_astro::houses::HouseSystem::Sripathi,
                _ => {
                    return Err(McpError::invalid_parameter(
                        "house_system",
                        &format!("Unknown: {s}"),
                    ));
                }
            },
            None => vedaksha_astro::houses::HouseSystem::Placidus,
        };

        let ayanamsha = match input.ayanamsha.as_deref() {
            Some(s) => match s.to_lowercase().as_str() {
                "lahiri" => Some(vedaksha_astro::sidereal::Ayanamsha::Lahiri),
                "faganbradley" | "fagan_bradley" => {
                    Some(vedaksha_astro::sidereal::Ayanamsha::FaganBradley)
                }
                "krishnamurti" => Some(vedaksha_astro::sidereal::Ayanamsha::Krishnamurti),
                "raman" => Some(vedaksha_astro::sidereal::Ayanamsha::Raman),
                "tropical" => None,
                _ => {
                    return Err(McpError::invalid_parameter(
                        "ayanamsha",
                        &format!("Unknown: {s}"),
                    ));
                }
            },
            None => None, // Tropical default
        };

        let config = vedaksha_astro::chart::ChartConfig {
            house_system,
            ayanamsha,
            rulership_scheme: vedaksha_astro::dignity::RulershipScheme::Traditional,
            aspect_types: vedaksha_astro::aspects::AspectType::MAJOR.to_vec(),
            orb_factor: 1.0,
        };

        let chart = vedaksha_astro::chart::compute_chart(
            &planet_data,
            ramc_deg,
            input.latitude,
            obliquity_deg,
            jd,
            &config,
        );

        // Build output JSON
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
            "julian_day": jd,
            "config_summary": chart.config_summary,
        });

        Ok(output)
    }

    fn call_compute_dasha(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_dasha::ComputeDashaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_dasha::validate(&input)?;

        // Dasha computation is ephemeris-free — compute directly.
        let levels = input.levels.unwrap_or(3).clamp(1, 5);
        let dasha = vedaksha_vedic::dasha::vimshottari::compute_vimshottari(
            input.moon_longitude,
            input.birth_jd,
            levels,
        );

        serde_json::to_value(&dasha).map_err(|e| McpError::computation_failed(&e.to_string()))
    }

    fn call_compute_karakas(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::karaka::{KarakaInput, KarakaScheme};

        let input: crate::tools::compute_karakas::ComputeKarakasInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_karakas::validate(&input)?;

        let scheme = match input.scheme.as_deref().unwrap_or("7") {
            "8" => KarakaScheme::Eight,
            _ => KarakaScheme::Seven,
        };

        let karaka_input = KarakaInput {
            sun: input.sun,
            moon: input.moon,
            mars: input.mars,
            mercury: input.mercury,
            jupiter: input.jupiter,
            venus: input.venus,
            saturn: input.saturn,
            rahu: input.rahu,
            scheme,
        };

        let assignments = vedaksha_vedic::karaka::compute_karakas(&karaka_input);
        serde_json::to_value(&assignments).map_err(|e| McpError::computation_failed(&e.to_string()))
    }

    fn call_compute_combustion(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::combustion::{combustion_state, CombustionState};
        use vedaksha_vedic::yoga::YogaPlanet;

        let input: crate::tools::compute_combustion::ComputeCombustionInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_combustion::validate(&input)?;

        let sun = input.sun;
        let sep = |lon: f64| -> f64 {
            let diff = (lon - sun).abs() % 360.0;
            if diff > 180.0 { 360.0 - diff } else { diff }
        };

        let results: Vec<serde_json::Value> = vec![
            (YogaPlanet::Moon,    input.moon,    false,                        "Moon"),
            (YogaPlanet::Mars,    input.mars,    input.mars_retrograde,        "Mars"),
            (YogaPlanet::Mercury, input.mercury, input.mercury_retrograde,     "Mercury"),
            (YogaPlanet::Jupiter, input.jupiter, input.jupiter_retrograde,     "Jupiter"),
            (YogaPlanet::Venus,   input.venus,   input.venus_retrograde,       "Venus"),
            (YogaPlanet::Saturn,  input.saturn,  input.saturn_retrograde,      "Saturn"),
        ]
        .into_iter()
        .map(|(planet, lon, retro, name)| {
            let state = combustion_state(planet, lon, sun, retro);
            let state_str = match state {
                CombustionState::None => "None",
                CombustionState::Combust => "Combust",
                CombustionState::DeeplyCombust => "DeeplyCombust",
            };
            serde_json::json!({
                "planet": name,
                "state": state_str,
                "degrees_from_sun": sep(lon),
            })
        })
        .collect();

        Ok(serde_json::json!(results))
    }

    fn call_compute_vargas(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_vargas::ComputeVargasInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_vargas::validate(&input)?;

        if let Some(planet_lon) = input.planet_longitude {
            // Compute varga sign for each requested division from the supplied
            // sidereal longitude — no EphemerisProvider required.
            let mut results = serde_json::Map::new();
            for division_name in &input.divisions {
                let varga_type = parse_varga_type(division_name)
                    .map_err(|e| McpError::invalid_parameter("divisions", &e))?;
                let sign = vedaksha_vedic::varga::varga_sign(planet_lon, varga_type);
                results.insert(division_name.clone(), serde_json::json!(sign));
            }
            Ok(serde_json::Value::Object(results))
        } else {
            // Without `planet_longitude`, return a validation-only response
            // listing the requested divisions. Direct varga computation
            // requires the planet's sidereal longitude.
            Ok(serde_json::json!({
                "status": "validated",
                "message": "Input validated. Provide planet_longitude for direct varga computation."
            }))
        }
    }

    fn call_compute_transit(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;

        let input: crate::tools::compute_transit::ComputeTransitInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_transit::validate(&input)?;

        let provider = AnalyticalProvider;

        // 9 standard Jyotish bodies
        let bodies = [
            ("Sun", Body::Sun),
            ("Moon", Body::Moon),
            ("Mercury", Body::Mercury),
            ("Venus", Body::Venus),
            ("Mars", Body::Mars),
            ("Jupiter", Body::Jupiter),
            ("Saturn", Body::Saturn),
            ("MeanNode", Body::MeanNode),
            ("TrueNode", Body::TrueNode),
            ("TrueNodeOsculating", Body::TrueNodeOsculating),
        ];

        // Compute natal positions
        let mut natal_positions: Vec<serde_json::Value> = Vec::new();
        for (name, body) in &bodies {
            let pos = coordinates::apparent_position(&provider, *body, input.natal_jd)
                .map_err(|e| McpError::computation_failed(&format!("Natal {name}: {e}")))?;
            natal_positions.push(serde_json::json!({
                "name": name,
                "longitude": pos.ecliptic.longitude.to_degrees(),
                "latitude": pos.ecliptic.latitude.to_degrees(),
                "distance": pos.ecliptic.distance,
                "speed": pos.longitude_speed,
            }));
        }

        // Compute transit positions
        let mut transit_positions: Vec<serde_json::Value> = Vec::new();
        for (name, body) in &bodies {
            let pos = coordinates::apparent_position(&provider, *body, input.transit_jd)
                .map_err(|e| McpError::computation_failed(&format!("Transit {name}: {e}")))?;
            transit_positions.push(serde_json::json!({
                "name": name,
                "longitude": pos.ecliptic.longitude.to_degrees(),
                "latitude": pos.ecliptic.latitude.to_degrees(),
                "distance": pos.ecliptic.distance,
                "speed": pos.longitude_speed,
            }));
        }

        // Compute transit-to-natal aspects using major aspect angles
        let major_aspects: &[(&str, f64)] = &[
            ("Conjunction", 0.0),
            ("Sextile", 60.0),
            ("Square", 90.0),
            ("Trine", 120.0),
            ("Opposition", 180.0),
        ];
        let max_orb = 1.0_f64;

        let mut aspects: Vec<serde_json::Value> = Vec::new();
        for (ti, t_pos) in transit_positions.iter().enumerate() {
            let t_lon = t_pos["longitude"].as_f64().unwrap_or(0.0);
            for (ni, n_pos) in natal_positions.iter().enumerate() {
                let n_lon = n_pos["longitude"].as_f64().unwrap_or(0.0);
                let raw_diff = ((t_lon - n_lon) % 360.0 + 360.0) % 360.0;
                let sep = if raw_diff > 180.0 {
                    360.0 - raw_diff
                } else {
                    raw_diff
                };
                for (aspect_name, aspect_angle) in major_aspects {
                    let orb = (sep - aspect_angle).abs();
                    if orb <= max_orb {
                        let t_speed = t_pos["speed"].as_f64().unwrap_or(0.0);
                        aspects.push(serde_json::json!({
                            "transit_body": t_pos["name"],
                            "transit_body_index": ti,
                            "natal_body": n_pos["name"],
                            "natal_body_index": ni,
                            "aspect_type": aspect_name,
                            "aspect_angle": aspect_angle,
                            "orb": orb,
                            "applying": t_speed > 0.0,
                        }));
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "natal_jd": input.natal_jd,
            "transit_jd": input.transit_jd,
            "natal_positions": natal_positions,
            "transit_positions": transit_positions,
            "transit_natal_aspects": aspects,
        }))
    }

    fn call_search_transits(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;

        let input: crate::tools::search_transits::SearchTransitsInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::search_transits::validate(&input)?;

        let provider = AnalyticalProvider;

        // Map body name strings to (name, Body) pairs for use with AnalyticalProvider.
        let all_bodies: &[(&str, Body)] = &[
            ("Sun", Body::Sun),
            ("Moon", Body::Moon),
            ("Mercury", Body::Mercury),
            ("Venus", Body::Venus),
            ("Mars", Body::Mars),
            ("Jupiter", Body::Jupiter),
            ("Saturn", Body::Saturn),
            ("MeanNode", Body::MeanNode),
            ("TrueNode", Body::TrueNode),
            ("TrueNodeOsculating", Body::TrueNodeOsculating),
        ];

        // Determine which bodies to track.
        let transiting_bodies: Vec<(String, usize)> = if let Some(ref requested) = input.bodies {
            requested
                .iter()
                .filter_map(|req_name| {
                    all_bodies
                        .iter()
                        .position(|(n, _)| n.eq_ignore_ascii_case(req_name))
                        .map(|idx| (all_bodies[idx].0.to_owned(), idx))
                })
                .collect()
        } else {
            all_bodies
                .iter()
                .enumerate()
                .map(|(idx, (name, _))| ((*name).to_owned(), idx))
                .collect()
        };

        // Map aspect name strings to (name, angle) pairs.
        let all_aspects: &[(&str, f64)] = &[
            ("Conjunction", 0.0),
            ("Sextile", 60.0),
            ("Square", 90.0),
            ("Trine", 120.0),
            ("Opposition", 180.0),
        ];

        let aspect_types: Vec<(String, f64)> = if let Some(ref requested) = input.aspects {
            requested
                .iter()
                .filter_map(|req_name| {
                    all_aspects
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(req_name))
                        .map(|(n, a)| ((*n).to_owned(), *a))
                })
                .collect()
        } else {
            all_aspects
                .iter()
                .map(|(n, a)| ((*n).to_owned(), *a))
                .collect()
        };

        let max_orb = input.max_orb.unwrap_or(1.0);

        // Build the search config for vedaksha_astro::transits.
        let config = vedaksha_astro::transits::TransitSearchConfig {
            natal_positions: input
                .natal_positions
                .iter()
                .map(|p| (p.name.clone(), p.longitude))
                .collect(),
            start_jd: input.start_jd,
            end_jd: input.end_jd,
            transiting_bodies,
            aspect_types,
            max_orb,
            step_size: 1.0, // 1-day coarse step; bisection refines to sub-minute precision
        };

        // Closure: look up body longitude from AnalyticalProvider by index.
        let get_longitude = |body_idx: usize, jd: f64| -> Option<f64> {
            let (_, body) = all_bodies.get(body_idx)?;
            coordinates::apparent_position(&provider, *body, jd)
                .ok()
                .map(|pos| pos.ecliptic.longitude.to_degrees())
        };

        let events = vedaksha_astro::transits::search_transits(&config, &get_longitude);

        let events_json: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "transiting_body": e.transiting_body,
                    "natal_body": e.natal_body,
                    "aspect_type": e.aspect_type,
                    "exact_jd": e.exact_jd,
                    "applying": e.applying,
                    "exact_orb": e.exact_orb,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "start_jd": input.start_jd,
            "end_jd": input.end_jd,
            "max_orb": max_orb,
            "event_count": events_json.len(),
            "events": events_json,
        }))
    }

    fn call_search_muhurta(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;

        let input: crate::tools::search_muhurta::SearchMuhurtaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::search_muhurta::validate(&input)?;

        let provider = AnalyticalProvider;
        let min_quality = input.min_quality.unwrap_or(0.5);

        // Muhurta needs sidereal Sun and Moon longitudes (Lahiri ayanamsha).
        let get_moon_sidereal = |jd: f64| -> Option<f64> {
            let pos = coordinates::apparent_position(&provider, Body::Moon, jd).ok()?;
            let tropical_lon = pos.ecliptic.longitude.to_degrees();
            Some(vedaksha_astro::sidereal::tropical_to_sidereal(
                tropical_lon,
                vedaksha_astro::sidereal::Ayanamsha::Lahiri,
                jd,
            ))
        };

        let get_sun_sidereal = |jd: f64| -> Option<f64> {
            let pos = coordinates::apparent_position(&provider, Body::Sun, jd).ok()?;
            let tropical_lon = pos.ecliptic.longitude.to_degrees();
            Some(vedaksha_astro::sidereal::tropical_to_sidereal(
                tropical_lon,
                vedaksha_astro::sidereal::Ayanamsha::Lahiri,
                jd,
            ))
        };

        let assessments = vedaksha_vedic::muhurta::search_muhurta(
            input.start_jd,
            input.end_jd,
            &get_moon_sidereal,
            &get_sun_sidereal,
            min_quality,
        );

        let results_json: Vec<serde_json::Value> = assessments
            .iter()
            .map(|a| {
                serde_json::json!({
                    "jd": a.jd,
                    "nakshatra": a.nakshatra.name(),
                    "tithi_number": a.tithi.number,
                    "tithi_name": a.tithi.name,
                    "weekday": format!("{:?}", a.weekday),
                    "quality_score": a.quality_score,
                    "factors": a.factors,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "start_jd": input.start_jd,
            "end_jd": input.end_jd,
            "latitude": input.latitude,
            "longitude": input.longitude,
            "min_quality": min_quality,
            "result_count": results_json.len(),
            "results": results_json,
        }))
    }

    fn call_emit_graph(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_emit::GraphEmitter;

        let input: crate::tools::emit_graph::EmitGraphInput = serde_json::from_value(args.clone())
            .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::emit_graph::validate(&input)?;

        // Parse the ChartGraph from the supplied JSON.
        let graph: vedaksha_graph::ChartGraph = serde_json::from_value(input.chart_json)
            .map_err(|e| McpError::invalid_parameter("chart_json", &e.to_string()))?;

        // Emit using the requested format (validate() already normalises case,
        // but validate() doesn't mutate, so normalise here as well).
        let fmt = input.format.trim().to_lowercase();
        let output = match fmt.as_str() {
            "cypher" => vedaksha_emit::cypher::CypherEmitter.emit(&graph),
            "surreal" => vedaksha_emit::surreal::SurrealEmitter.emit(&graph),
            "jsonld" => vedaksha_emit::jsonld::JsonLdEmitter.emit(&graph),
            "json" => vedaksha_emit::json_graph::JsonGraphEmitter.emit(&graph),
            "embedding" => vedaksha_emit::embedding_text::EmbeddingTextEmitter.emit(&graph),
            _ => Err(format!("Unknown format: {fmt}")),
        };

        match output {
            Ok(text) => Ok(serde_json::Value::String(text)),
            Err(e) => Err(McpError::computation_failed(&e)),
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn error_response(id: &serde_json::Value, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_graph::{ChartGraph, classification::DataClassification, ids::NodeId};

    fn server() -> McpServer {
        McpServer::new()
    }

    // ── initialize ────────────────────────────────────────────────────────────

    #[test]
    fn initialize_returns_correct_protocol_version() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(
            val["result"]["protocolVersion"].as_str().unwrap(),
            "2024-11-05"
        );
    }

    #[test]
    fn initialize_response_contains_server_info() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(
            val["result"]["serverInfo"]["name"].as_str().unwrap(),
            "vedaksha-mcp"
        );
    }

    // ── tools/list ────────────────────────────────────────────────────────────

    #[test]
    fn tools_list_returns_nine_tools() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let tools = val["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 9, "expected exactly 9 tools");
    }

    #[test]
    fn tools_list_includes_all_expected_tool_names() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":4,"method":"tools/list","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let names: Vec<&str> = val["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"compute_natal_chart"));
        assert!(names.contains(&"compute_dasha"));
        assert!(names.contains(&"compute_karakas"));
        assert!(names.contains(&"compute_vargas"));
        assert!(names.contains(&"emit_graph"));
        assert!(names.contains(&"compute_transit"));
        assert!(names.contains(&"search_transits"));
        assert!(names.contains(&"search_muhurta"));
        assert!(names.contains(&"compute_combustion"));
    }

    // ── tools/call — unknown tool ─────────────────────────────────────────────

    #[test]
    fn tools_call_unknown_tool_returns_error() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"no_such_tool","arguments":{}}}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["error"].is_object(), "expected an error response");
        assert_eq!(
            val["error"]["code"].as_i64().unwrap(),
            INVALID_PARAMS as i64
        );
    }

    // ── tools/call — invalid JSON ─────────────────────────────────────────────

    #[test]
    fn invalid_json_returns_parse_error() {
        let s = server();
        let resp = s.handle_request("this is not json");
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(val["error"]["code"].as_i64().unwrap(), PARSE_ERROR as i64);
        assert_eq!(val["id"], serde_json::Value::Null);
    }

    // ── unknown method ────────────────────────────────────────────────────────

    #[test]
    fn unknown_method_returns_method_not_found() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":6,"method":"unknown/method","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(
            val["error"]["code"].as_i64().unwrap(),
            METHOD_NOT_FOUND as i64
        );
    }

    // ── missing params ────────────────────────────────────────────────────────

    #[test]
    fn tools_call_missing_params_returns_invalid_params() {
        let s = server();
        let resp = s.handle_request(r#"{"jsonrpc":"2.0","id":7,"method":"tools/call"}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(
            val["error"]["code"].as_i64().unwrap(),
            INVALID_PARAMS as i64
        );
    }

    // ── compute_natal_chart ───────────────────────────────────────────────────

    #[test]
    fn compute_natal_with_valid_params_returns_chart() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{
                "name":"compute_natal_chart",
                "arguments":{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        let chart: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(chart["planets"].is_array(), "expected planets array");
        assert!(chart["houses"].is_object(), "expected houses object");
        assert!(chart["aspects"].is_array(), "expected aspects array");
        let planets = chart["planets"].as_array().unwrap();
        assert_eq!(planets.len(), 10, "expected 10 planets");
        let asc = chart["houses"]["asc"].as_f64().unwrap();
        assert!(asc > 0.0 && asc < 360.0, "ASC out of range: {asc}");
    }

    #[test]
    fn compute_natal_with_invalid_jd_returns_date_out_of_range() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{
                "name":"compute_natal_chart",
                "arguments":{"julian_day":1.0,"latitude":28.6,"longitude":77.2}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["error"].is_object());
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "DATE_OUT_OF_RANGE"
        );
    }

    // ── compute_dasha ─────────────────────────────────────────────────────────

    #[test]
    fn compute_dasha_with_valid_params_returns_dasha_tree() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{
                "name":"compute_dasha",
                "arguments":{"moon_longitude":45.0,"birth_jd":2451545.0,"levels":2}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        // The text is JSON-serialised VimshottariDasha — check for key fields.
        let dasha: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(
            dasha["maha_dashas"].is_array(),
            "expected maha_dashas array"
        );
    }

    // ── emit_graph ────────────────────────────────────────────────────────────

    #[test]
    fn emit_graph_with_json_format_works_end_to_end() {
        let s = server();

        // Build a minimal valid ChartGraph JSON.
        let chart_id = NodeId::chart_scoped("test", "chart", "root");
        let graph = ChartGraph::new(chart_id, DataClassification::Anonymous);
        let chart_json = serde_json::to_value(&graph).unwrap();

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "emit_graph",
                "arguments": {
                    "chart_json": chart_json,
                    "format": "json"
                }
            }
        });

        let resp = s.handle_request(&serde_json::to_string(&req).unwrap());
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        // The emitted output should itself be valid JSON containing nodes/edges.
        let emitted: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(emitted["nodes"].is_array());
        assert!(emitted["edges"].is_array());
    }

    // ── compute_vargas — real varga computation ───────────────────────────────

    #[test]
    fn compute_vargas_navamsha_returns_sign_index() {
        let s = server();
        // 0° Aries (movable sign) → first navamsha starts from Aries itself → sign 0
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{
                "name":"compute_vargas",
                "arguments":{
                    "julian_day":2451545.0,
                    "latitude":28.6,
                    "longitude":77.2,
                    "planet_longitude":0.0,
                    "divisions":["D9"]
                }
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        let result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            result["D9"].as_u64().unwrap(),
            0,
            "0° Aries navamsha should be sign 0 (Aries)"
        );
    }

    #[test]
    fn compute_vargas_rashi_and_navamsha_together() {
        let s = server();
        // 45° = 15° Taurus (D1 sign = 1), navamsha of Taurus (fixed) starts from Capricorn (9)
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{
                "name":"compute_vargas",
                "arguments":{
                    "julian_day":2451545.0,
                    "latitude":28.6,
                    "longitude":77.2,
                    "planet_longitude":45.0,
                    "divisions":["D1","D9"]
                }
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        let result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            result["D1"].as_u64().unwrap(),
            1,
            "45° should be Taurus (D1 sign 1)"
        );
        assert!(result["D9"].is_number(), "D9 result should be a number");
    }

    // ── search_transits validation ────────────────────────────────────────────

    #[test]
    fn search_transits_validates_jd_range() {
        let s = server();
        // Span > 100 years should be rejected.
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{
                "name":"search_transits",
                "arguments":{
                    "natal_positions":[{"name":"Mars","longitude":45.0}],
                    "start_jd":2451545.0,
                    "end_jd":2525000.0
                }
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(
            val["error"].is_object(),
            "expected an error for oversized range"
        );
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "SEARCH_RANGE_TOO_LARGE"
        );
    }

    #[test]
    fn search_transits_returns_actual_results() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{
                "name":"search_transits",
                "arguments":{
                    "natal_positions":[{"name":"Mars","longitude":45.0}],
                    "start_jd":2451545.0,
                    "end_jd":2451910.0
                }
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(
            data["event_count"].as_u64().is_some(),
            "expected event_count in response"
        );
        assert!(
            data["events"].is_array(),
            "expected events array in response"
        );
    }

    // ── search_muhurta validation ─────────────────────────────────────────────

    #[test]
    fn search_muhurta_validates_latitude() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{
                "name":"search_muhurta",
                "arguments":{
                    "start_jd":2451545.0,
                    "end_jd":2451575.0,
                    "latitude":95.0,
                    "longitude":77.2
                }
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(
            val["error"].is_object(),
            "expected an error for invalid latitude"
        );
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "INVALID_LATITUDE"
        );
    }
}
