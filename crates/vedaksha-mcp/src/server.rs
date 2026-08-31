// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
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
            .map(crate::tools::ToolDefinition::to_wire)
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
            "compute_shadbala" => Self::call_compute_shadbala(&arguments),
            "compute_ashtakavarga" => Self::call_compute_ashtakavarga(&arguments),
            "compute_gochara" => Self::call_compute_gochara(&arguments),
            "compute_vargas" => Self::call_compute_vargas(&arguments),
            "emit_graph" => Self::call_emit_graph(&arguments),
            "compute_transit" => Self::call_compute_transit(&arguments),
            "search_transits" => Self::call_search_transits(&arguments),
            "search_muhurta" => Self::call_search_muhurta(&arguments),
            "compute_panchanga" => Self::call_compute_panchanga(&arguments),
            "compute_drishti" => Self::call_compute_drishti(&arguments),
            "compute_bhavas" => Self::call_compute_bhavas(&arguments),
            "compute_synastry" => Self::call_compute_synastry(&arguments),
            "compute_composite" => Self::call_compute_composite(&arguments),
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
                // structuredContent is added alongside, never instead of, the
                // text block: it is what a schema-aware client validates,
                // while existing callers keep parsing exactly the string they
                // parse today.
                let structured = crate::tools::tool_definitions()
                    .iter()
                    .find(|t| t.name == tool_name)
                    .and_then(|t| t.structured_content(&value));

                let text = match value {
                    serde_json::Value::String(s) => s,
                    other => serde_json::to_string(&other).unwrap_or_default(),
                };
                let mut result = serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                });
                if let Some(structured) = structured {
                    result["structuredContent"] = structured;
                }
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(result),
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

    /// Positions, ascendant and dignities for one moment and place.
    ///
    /// Extracted so `compute_vargas` divides the very chart
    /// `compute_natal_chart` returns. A varga computed by a second
    /// implementation could disagree with the D-1 it is derived from, and
    /// nothing would catch it — the two surfaces have no oracle in common.
    ///
    /// Returns the chart and the ayanamsha actually applied, in degrees.
    fn compute_sidereal_chart(
        julian_day: f64,
        latitude: f64,
        geo_longitude: f64,
        ayanamsha_name: Option<&str>,
        house_system: vedaksha_astro::houses::HouseSystem,
    ) -> Result<(vedaksha_astro::chart::ComputedChart, f64), McpError> {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;
        use vedaksha_ephem_core::nutation;
        use vedaksha_ephem_core::obliquity;
        use vedaksha_ephem_core::sidereal_time;

        let provider = AnalyticalProvider;
        let jd = julian_day;

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

        // Every body is wanted at the same instant, so use the batch entry
        // point: it builds the three central-difference frames once instead of
        // once per body, and shares one memoizing provider so the ELP/MPP02
        // lunar series pulled in by each light-time correction is evaluated per
        // timestamp rather than per body. Bit-identical to `apparent_position`
        // per body — asserted by `batch_matches_per_body_bit_for_bit`.
        let body_list: Vec<Body> = bodies.iter().map(|(_, body)| *body).collect();
        let computed = coordinates::apparent_positions(&provider, &body_list, jd);

        let mut planet_data: Vec<(String, f64, f64, f64, f64)> = Vec::new();
        for ((name, _), (_, result)) in bodies.iter().zip(computed) {
            let pos = result.map_err(|e| {
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

        // Sidereal time → RAMC.
        //
        // Two time scales, deliberately: nutation and obliquity are dynamical
        // and take TT, while sidereal time is the Earth's *rotation* and takes
        // UT1 (`jd`). Passing `jd_tt` as the rotational argument — which this
        // did until the UT-vs-TT fix — adds ΔT worth of rotation instead of
        // removing it: 0.289° (17.3′) at today's ΔT ≈ 69 s, straight onto the
        // ascendant, the MC and all twelve cusps.
        let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd);
        let (dpsi, deps) = nutation::nutation(jd_tt);
        let eps_true = obliquity::true_obliquity(jd_tt, deps);
        let geo_lon_rad = geo_longitude * core::f64::consts::PI / 180.0;
        let last = sidereal_time::local_sidereal_time(jd, geo_lon_rad, dpsi, eps_true);
        let ramc_deg = last * 180.0 / core::f64::consts::PI;

        // Obliquity in degrees
        let obliquity_deg = obliquity::mean_obliquity(jd_tt) * 180.0 / core::f64::consts::PI;

        // Delegates to the engine's own parser so this surface cannot drift from
        // the systems the engine actually has. A name that Vedaksha 5 accepted
        // but that no longer names a system is refused with the engine's
        // disposition message, never silently remapped.
        let ayanamsha = match ayanamsha_name {
            Some(s) => {
                use std::str::FromStr as _;
                match vedaksha_astro::sidereal::Ayanamsha::from_str(s) {
                    Ok(vedaksha_astro::sidereal::Ayanamsha::Tropical) => None,
                    Ok(a) => Some(a),
                    Err(e) => {
                        return Err(McpError::invalid_parameter("ayanamsha", &e.to_string()));
                    }
                }
            }
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
            latitude,
            obliquity_deg,
            jd,
            &config,
        );

        // Report the offset that was applied, not just the name that was asked
        // for. Every longitude, the ascendant, the MC and all twelve cusps below
        // have been rotated by this value; without it a sidereal caller cannot
        // check the rotation or convert back to tropical. `None` is tropical,
        // which is a zero rotation — the same number `true_ayanamsha_value`
        // returns for `Ayanamsha::Tropical`, which is what the wasm surface
        // passes. Uses the TRUE ayanamsha so this field always equals the
        // rotation `compute_chart` actually performed (see
        // `sidereal::true_ayanamsha_value`) — must stay identical to the wasm
        // path (`vedaksha-wasm/src/lib.rs`); `mcp_surface_parity` enforces it.
        let ayanamsha_value = ayanamsha.map_or(0.0, |a| {
            vedaksha_astro::sidereal::true_ayanamsha_value(a, jd)
        });

        Ok((chart, ayanamsha_value))
    }

    fn call_compute_natal(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_natal::ComputeNatalInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_natal::validate(&input)?;

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

        let jd = input.julian_day;
        let (chart, ayanamsha_value) = Self::compute_sidereal_chart(
            jd,
            input.latitude,
            input.longitude,
            input.ayanamsha.as_deref(),
            house_system,
        )?;

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
            "true_ayanamsha_value": ayanamsha_value,
            "julian_day": jd,
            "config_summary": chart.config_summary,
        });

        Ok(output)
    }

    fn call_compute_dasha(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use crate::tools::compute_dasha::{ComputeDashaInput, DashaSystem};
        use vedaksha_vedic::dasha;

        let input: ComputeDashaInput = serde_json::from_value(args.clone())
            .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        let system = crate::tools::compute_dasha::validate(&input)?;

        // This tool performs no ephemeris computation of its own — every
        // graha position it uses (moon_longitude, or graha_signs for
        // Chara/Narayana) is caller-supplied, not derived here. That is NOT
        // the same as chart-independence: Chara/Narayana durations vary
        // with the supplied graha_signs (see dasha::chara's module doc).
        let levels = input.levels.unwrap_or(3).clamp(1, 5);
        let result = match system {
            DashaSystem::Vimshottari => {
                serde_json::to_value(dasha::vimshottari::compute_vimshottari(
                    input.moon_longitude.expect("validated above"),
                    input.birth_jd,
                    levels,
                ))
            }
            DashaSystem::Ashtottari => serde_json::to_value(dasha::ashtottari::compute_ashtottari(
                input.moon_longitude.expect("validated above"),
                input.birth_jd,
                levels,
            )),
            DashaSystem::Yogini => serde_json::to_value(dasha::yogini::compute_yogini(
                input.moon_longitude.expect("validated above"),
                input.birth_jd,
                levels,
            )),
            // `dasha::chara::compute_chara` and `dasha::narayana::compute_narayana`
            // take a 0-indexed lagna_sign, but this tool's schema documents
            // `lagna_sign` as 1-indexed (1 = Aries), matching `compute_natal_chart`'s
            // house-based convention. The `- 1` below is that conversion.
            // Shipping without it silently starts every Chara/Narayana sequence
            // one sign off from the true lagna -- exactly the defect that went
            // unnoticed from v2.4.0 through v8.0.0.
            DashaSystem::Chara => {
                let lagna_0indexed = input.lagna_sign.expect("validated above") - 1;
                let positions = input
                    .graha_signs
                    .expect("validated above")
                    .into_graha_signs();
                serde_json::to_value(dasha::chara::compute_chara(
                    lagna_0indexed,
                    input.birth_jd,
                    positions,
                ))
            }
            DashaSystem::Narayana => {
                let lagna_0indexed = input.lagna_sign.expect("validated above") - 1;
                let positions = input
                    .graha_signs
                    .expect("validated above")
                    .into_graha_signs();
                serde_json::to_value(dasha::narayana::compute_narayana(
                    lagna_0indexed,
                    input.birth_jd,
                    positions,
                ))
            }
        };

        result.map_err(|e| McpError::computation_failed(&e.to_string()))
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
        use vedaksha_vedic::combustion::{CombustionState, combustion_state};
        use vedaksha_vedic::graha::Graha;

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
            (Graha::Moon, input.moon, false, "Moon"),
            (Graha::Mars, input.mars, input.mars_retrograde, "Mars"),
            (
                Graha::Mercury,
                input.mercury,
                input.mercury_retrograde,
                "Mercury",
            ),
            (
                Graha::Jupiter,
                input.jupiter,
                input.jupiter_retrograde,
                "Jupiter",
            ),
            (Graha::Venus, input.venus, input.venus_retrograde, "Venus"),
            (
                Graha::Saturn,
                input.saturn,
                input.saturn_retrograde,
                "Saturn",
            ),
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

    fn call_compute_panchanga(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_astro::riseset::sun_equatorial_deg;
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_vedic::muhurta::{Paksha, Weekday, compute_tithi};
        use vedaksha_vedic::nakshatra::Nakshatra;
        use vedaksha_vedic::panchanga::{compute_karana, compute_panchanga_yoga};

        let input: crate::tools::compute_panchanga::ComputePanchangaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_panchanga::validate(&input)?;

        let tithi = compute_tithi(input.moon, input.sun);
        // `AnalyticalProvider` as a plain local — the pattern used at
        // server.rs:292, :868, :971 and :1092. There is no `Self::provider()`
        // accessor in this file; do not invent one.
        let provider = AnalyticalProvider;
        let sun_eq = |jd: f64| sun_equatorial_deg(&provider, jd);
        // ONE sunrise search for both the vara and the kalam windows:
        // `kalam_windows` returns the vara it derived internally, so there is
        // no second, separate `vara_at` call here repeating the same horizon
        // search. That mattered more when the search was a 5-minute scan and
        // still matters now that it is analytic (Meeus eq. 15.1, see
        // `vedaksha_astro::riseset`), because the ephemeris evaluation behind
        // it is the expensive part either way.
        let reckoning = vedaksha_vedic::muhurta::kalam_windows(
            input.jd,
            input.latitude,
            input.longitude,
            input.elevation_m,
            input.tz_offset_minutes,
            &sun_eq,
        );
        let (weekday, kalams) = (reckoning.vara, reckoning.windows);
        let nakshatra = Nakshatra::from_longitude(input.moon);
        let pada = Nakshatra::pada_from_longitude(input.moon);
        let yoga = compute_panchanga_yoga(input.sun, input.moon);
        let karana = compute_karana(input.moon, input.sun);

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

        Ok(serde_json::json!({
            "tithi": {
                "number": tithi.number,
                "name": tithi.name,
                "paksha": paksha,
                "lord": tithi.lord(),
            },
            "vara": {
                "weekday": weekday_name,
                // How `weekday` was reckoned. `true` = from an actual local
                // sunrise (a vara). `false` = the civil-weekday fallback,
                // which is what the polar day/night produces and is a
                // DIFFERENT quantity — emitting it unflagged would be the
                // original UT-weekday defect surviving at high latitude.
                // Deliberately not inferable from `rahu_kalam`: those are
                // null in a third case (sunrise found, sunset not) where this
                // is still true. Key and semantics are mirrored exactly in
                // `vedaksha-wasm`'s `compute_panchanga`.
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
        }))
    }

    fn call_compute_drishti(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::drishti::{AspectStrength, VedicPlanet, find_vedic_aspects};

        let input: crate::tools::compute_drishti::ComputeDrishtiInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_drishti::validate(&input)?;

        // Drishti is cast sign-to-sign, so the longitude only matters via its sign.
        // `validate` above pins every longitude to [0, 360), so the quotient is in
        // [0, 12) — the cast can neither truncate meaningfully nor lose a sign.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let sign = vedaksha_astro::dignity::sign_index;

        let placements = [
            (VedicPlanet::Sun, sign(input.sun)),
            (VedicPlanet::Moon, sign(input.moon)),
            (VedicPlanet::Mars, sign(input.mars)),
            (VedicPlanet::Mercury, sign(input.mercury)),
            (VedicPlanet::Jupiter, sign(input.jupiter)),
            (VedicPlanet::Venus, sign(input.venus)),
            (VedicPlanet::Saturn, sign(input.saturn)),
            (VedicPlanet::Rahu, sign(input.rahu)),
            (VedicPlanet::Ketu, sign(input.ketu)),
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

        Ok(serde_json::json!(aspects))
    }

    fn call_compute_bhavas(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::bhava::{
            compute_bhavas, is_dusthana, is_kendra, is_trikona, is_upachaya, planet_bhava,
        };

        let input: crate::tools::compute_bhavas::ComputeBhavasInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_bhavas::validate(&input)?;

        let chart = compute_bhavas(input.ascendant);

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

        let placements: Vec<serde_json::Value> = input
            .planets
            .iter()
            .map(|(name, lon)| {
                // `validate` pins every supplied longitude to [0, 360), so the
                // quotient is in [0, 12) and the cast is exact.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let planet_sign = vedaksha_astro::dignity::sign_index(*lon);
                serde_json::json!({
                    "planet": name,
                    "sign": planet_sign,
                    "bhava": planet_bhava(planet_sign, &chart),
                })
            })
            .collect();

        Ok(serde_json::json!({
            "lagna_sign": chart.lagna_sign,
            "houses": houses,
            "planets": placements,
        }))
    }

    fn call_compute_synastry(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_astro::aspects::BodyPosition;
        use vedaksha_astro::synastry::find_synastry_aspects;

        let input: crate::tools::compute_synastry::ComputeSynastryInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        let (aspect_set, orb_factor) = crate::tools::compute_synastry::validate(&input)?;

        // `find_synastry_aspects` reads only `longitude`; the speed field is
        // never touched (no applying/separating determination is made for a
        // cross-chart aspect), so 0.0 is the absence of a value here, not an
        // assumed one. BTreeMap iteration is sorted, so the emitted order is
        // deterministic for a given input.
        let names_a: Vec<&str> = input.chart_a.keys().map(String::as_str).collect();
        let names_b: Vec<&str> = input.chart_b.keys().map(String::as_str).collect();
        let chart_a: Vec<BodyPosition> = input
            .chart_a
            .values()
            .map(|&longitude| BodyPosition {
                longitude,
                speed: 0.0,
            })
            .collect();
        let chart_b: Vec<BodyPosition> = input
            .chart_b
            .values()
            .map(|&longitude| BodyPosition {
                longitude,
                speed: 0.0,
            })
            .collect();

        let aspects: Vec<serde_json::Value> =
            find_synastry_aspects(&chart_a, &chart_b, aspect_set.types(), orb_factor)
                .into_iter()
                .map(|a| {
                    serde_json::json!({
                        "chart_a_planet": names_a[a.chart_a_body],
                        "chart_b_planet": names_b[a.chart_b_body],
                        "aspect_type": format!("{:?}", a.aspect_type),
                        "orb": a.orb,
                        "strength": a.strength,
                    })
                })
                .collect();

        Ok(serde_json::json!(aspects))
    }

    fn call_compute_composite(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_astro::composite::compute_composite;

        let input: crate::tools::compute_composite::ComputeCompositeInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_composite::validate(&input)?;

        // `validate` has established that the two maps carry exactly the same
        // keys, and BTreeMap iterates in sorted key order, so index i names the
        // same graha on both sides and all four slices are the same length —
        // which is what the engine's `assert_eq!` on the lengths demands.
        let names: Vec<&str> = input.chart_a.keys().map(String::as_str).collect();
        let lons_a: Vec<f64> = input.chart_a.values().map(|b| b.longitude).collect();
        let lons_b: Vec<f64> = input.chart_b.values().map(|b| b.longitude).collect();
        let speeds_a: Vec<f64> = input.chart_a.values().map(|b| b.speed).collect();
        let speeds_b: Vec<f64> = input.chart_b.values().map(|b| b.speed).collect();

        let positions: Vec<serde_json::Value> =
            compute_composite(&lons_a, &lons_b, &speeds_a, &speeds_b)
                .into_iter()
                .zip(names)
                .map(|(position, name)| {
                    serde_json::json!({
                        "planet": name,
                        "longitude": position.longitude,
                        "speed": position.speed,
                    })
                })
                .collect();

        Ok(serde_json::json!(positions))
    }

    fn call_compute_shadbala(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::graha::GrahaPosition;
        use vedaksha_vedic::shadbala::{ShadbalaPlanetData, compute_shadbala_full};

        let input: crate::tools::compute_shadbala::ComputeShadbalaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_shadbala::validate(&input)?;

        let planet_data: Vec<ShadbalaPlanetData> = input
            .planets
            .iter()
            .map(|entry| {
                let planet = crate::tools::compute_shadbala::parse_planet(&entry.planet)
                    .expect("already validated");
                ShadbalaPlanetData {
                    position: GrahaPosition {
                        planet,
                        sign: entry.sign,
                        longitude: entry.longitude,
                        bhava: entry.bhava,
                    },
                    speed: entry.speed,
                    average_speed: entry.average_speed,
                    benefic_aspect_count: entry.benefic_aspect_count,
                    malefic_aspect_count: entry.malefic_aspect_count,
                }
            })
            .collect();

        let results =
            compute_shadbala_full(&planet_data, input.is_daytime, input.moon_phase_waxing);
        serde_json::to_value(&results).map_err(|e| McpError::computation_failed(&e.to_string()))
    }

    fn call_compute_ashtakavarga(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::ashtakavarga::{
            BhinnaAshtakavargaInput, bhinna_ashtakavarga, sarvashtakavarga,
        };

        let input: crate::tools::compute_ashtakavarga::ComputeAshtakavargaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_ashtakavarga::validate(&input)?;

        let av_input = BhinnaAshtakavargaInput {
            sun: input.sun,
            moon: input.moon,
            mars: input.mars,
            mercury: input.mercury,
            jupiter: input.jupiter,
            venus: input.venus,
            saturn: input.saturn,
            lagna: input.lagna,
        };

        let tables = bhinna_ashtakavarga(&av_input);
        let sarva = sarvashtakavarga(&tables);

        serde_json::to_value(serde_json::json!({
            "tables": tables,
            "sarvashtakavarga": sarva,
        }))
        .map_err(|e| McpError::computation_failed(&e.to_string()))
    }

    fn call_compute_gochara(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_vedic::gochara::{
            SchoolProfile, TransitPositions, VedhaTable, apply_vedha_exemptions, compute_gochara,
        };

        let input: crate::tools::compute_gochara::ComputeGocharaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_gochara::validate(&input)?;

        let table = match input.vedha_table.as_deref().unwrap_or("Bphs29") {
            "Bphs29" => VedhaTable::Bphs29,
            other => {
                return Err(McpError::invalid_parameter(
                    "vedha_table",
                    &format!("unknown table '{other}'"),
                ));
            }
        };
        let school = match input.school.as_deref().unwrap_or("Geometry") {
            "Geometry" => SchoolProfile::Geometry,
            "Parashari" => SchoolProfile::Parashari,
            other => {
                return Err(McpError::invalid_parameter(
                    "school",
                    &format!("unknown school '{other}'"),
                ));
            }
        };

        let transits = TransitPositions {
            sun: input.sun,
            moon: input.moon,
            mars: input.mars,
            mercury: input.mercury,
            jupiter: input.jupiter,
            venus: input.venus,
            saturn: input.saturn,
        };

        let mut entries = compute_gochara(&transits, input.natal_reference_sign, table);
        for entry in &mut entries {
            apply_vedha_exemptions(entry, school);
        }

        serde_json::to_value(serde_json::json!({ "entries": entries }))
            .map_err(|e| McpError::computation_failed(&e.to_string()))
    }

    fn call_compute_vargas(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_vargas::ComputeVargasInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_vargas::validate(&input)?;

        use vedaksha_vedic::varga::{VargaTradition, varga_sign_with_tradition};

        let tradition = match input.tradition.as_deref() {
            Some("element") => VargaTradition::Element,
            _ => VargaTradition::Modality,
        };
        let tradition_name = match tradition {
            VargaTradition::Element => "element",
            VargaTradition::Modality => "modality",
        };

        // Resolve the division codes once, before any ephemeris work: an
        // unknown code should cost the caller an error, not a chart.
        let divisions: Vec<(String, vedaksha_vedic::varga::VargaType)> = input
            .divisions
            .iter()
            .map(|name| {
                parse_varga_type(name)
                    .map(|t| (name.clone(), t))
                    .map_err(|e| McpError::invalid_parameter("divisions", &e))
            })
            .collect::<Result<_, _>>()?;

        // A bare longitude names no graha, so it has no dignity, and there is
        // no lagna to count a bhava from. It gets the same envelope with those
        // three absent rather than a different shape.
        if let Some(planet_lon) = input.planet_longitude {
            let vargas: Vec<serde_json::Value> = divisions
                .iter()
                .map(|(name, varga)| {
                    serde_json::json!({
                        "division": name,
                        "placements": [{
                            "rashi_longitude": planet_lon,
                            "varga_sign": varga_sign_with_tradition(planet_lon, *varga, tradition),
                        }],
                    })
                })
                .collect();
            return Ok(serde_json::json!({
                "julian_day": input.julian_day,
                "true_ayanamsha_value": 0.0,
                "tradition": tradition_name,
                "vargas": vargas,
            }));
        }

        // The documented path, which returned a stub through v7.3.1.
        //
        // Positions and the ascendant come from the same chart routine
        // compute_natal_chart uses, not a second implementation of it, so a
        // varga can never disagree with the D-1 it is derived from.
        // Whole-sign: a varga needs the ascendant's longitude, and nothing
        // about the D-1 cusps. Whole-sign is also the system the bhava counting
        // below assumes.
        let (chart, ayanamsha_value) = Self::compute_sidereal_chart(
            input.julian_day,
            input.latitude,
            input.longitude,
            input.ayanamsha.as_deref(),
            vedaksha_astro::houses::HouseSystem::WholeSign,
        )?;

        let vargas: Vec<serde_json::Value> = divisions
            .iter()
            .map(|(name, varga)| {
                let lagna_sign = varga_sign_with_tradition(chart.houses.asc, *varga, tradition);
                let placements: Vec<serde_json::Value> = chart
                    .planets
                    .iter()
                    .map(|planet| {
                        let varga_sign =
                            varga_sign_with_tradition(planet.longitude, *varga, tradition);
                        // Whole-sign bhavas counted from the varga lagna: the
                        // sign holding the lagna is the 1st, the next the 2nd.
                        let bhava = (varga_sign + 12 - lagna_sign) % 12 + 1;
                        let dignity = vedaksha_astro::chart::name_to_dignity_planet(&planet.name)
                            .map(|p| {
                                format!(
                                    "{:?}",
                                    vedaksha_astro::dignity::dignity_of(
                                        p,
                                        vedaksha_astro::dignity::Sign::from_index(varga_sign),
                                        vedaksha_astro::dignity::RulershipScheme::Traditional,
                                    )
                                )
                            });
                        let mut out = serde_json::json!({
                            "planet": planet.name,
                            "rashi_longitude": planet.longitude,
                            "varga_sign": varga_sign,
                            "bhava": bhava,
                        });
                        // Absent, not null: Rahu and Ketu have no essential
                        // dignity, and saying "none" would read as a value.
                        if let Some(dignity) = dignity {
                            out["dignity"] = serde_json::Value::String(dignity);
                        }
                        out
                    })
                    .collect();
                serde_json::json!({
                    "division": name,
                    "lagna_sign": lagna_sign,
                    "placements": placements,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "julian_day": input.julian_day,
            "true_ayanamsha_value": ayanamsha_value,
            "tradition": tradition_name,
            "vargas": vargas,
        }))
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

        // Both sweeps below want every body at one shared instant, so each goes
        // through the batch entry point: frames built once per instant instead
        // of once per body, and a memoizing provider shared across the set.
        // Bit-identical to per-body `apparent_position` — asserted by
        // `batch_matches_per_body_bit_for_bit`.
        let body_list: Vec<Body> = bodies.iter().map(|(_, body)| *body).collect();

        // Compute natal positions
        let natal_computed = coordinates::apparent_positions(&provider, &body_list, input.natal_jd);
        let mut natal_positions: Vec<serde_json::Value> = Vec::new();
        for ((name, _), (_, result)) in bodies.iter().zip(natal_computed) {
            let pos =
                result.map_err(|e| McpError::computation_failed(&format!("Natal {name}: {e}")))?;
            natal_positions.push(serde_json::json!({
                "name": name,
                "longitude": pos.ecliptic.longitude.to_degrees(),
                "latitude": pos.ecliptic.latitude.to_degrees(),
                "distance": pos.ecliptic.distance,
                "speed": pos.longitude_speed,
            }));
        }

        // Compute transit positions
        let transit_computed =
            coordinates::apparent_positions(&provider, &body_list, input.transit_jd);
        let mut transit_positions: Vec<serde_json::Value> = Vec::new();
        for ((name, _), (_, result)) in bodies.iter().zip(transit_computed) {
            let pos = result
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
        //
        // Position only. `search_transits` takes longitude and nothing else —
        // its `applying` flag comes from comparing successive orbs, not from a
        // speed — so skip the two extra half-day evaluations that
        // `apparent_position`'s central-difference `longitude_speed` needs and
        // use `ecliptic_position` (≈3× cheaper per step). Same reasoning, and
        // the same call, as `call_search_muhurta` below; the returned
        // `.longitude` is the identical value `apparent_position` would have
        // put in `.ecliptic.longitude`.
        let get_longitude = |body_idx: usize, jd: f64| -> Option<f64> {
            let (_, body) = all_bodies.get(body_idx)?;
            coordinates::ecliptic_position(&provider, *body, jd)
                .ok()
                .map(|pos| pos.longitude.to_degrees())
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
        use std::sync::Mutex;
        use vedaksha_astro::riseset::sun_equatorial_deg;
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates;

        /// One memo slot for the enrichment loop below: an instant, keyed on
        /// the JD's exact bit pattern, and whatever was computed there.
        type Slot<T> = Mutex<Option<(u64, T)>>;

        /// Read a slot, hitting only on a bit-for-bit identical JD.
        fn read_slot<T: Copy>(slot: &Slot<T>, jd: f64) -> Option<T> {
            slot.lock()
                .expect("memo slot")
                .and_then(|(bits, payload)| (bits == jd.to_bits()).then_some(payload))
        }

        let input: crate::tools::search_muhurta::SearchMuhurtaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::search_muhurta::validate(&input)?;

        let provider = AnalyticalProvider;
        let min_quality = input.min_quality.unwrap_or(0.5);

        // Muhurta needs sidereal Sun and Moon longitudes (Lahiri ayanamsha),
        // using the TRUE ayanamsha (mean + nutation-in-longitude) so the
        // quality scoring and nakshatra-boundary refinement below are rotated
        // consistently with the nutation already present in the tropical
        // positions `apparent_position`/`apparent_positions` return.
        // Position only — daily motion is not used, so skip the central-
        // difference work via `ecliptic_position` (≈3× cheaper per step).
        let get_moon_sidereal = |jd: f64| -> Option<f64> {
            let pos = coordinates::ecliptic_position(&provider, Body::Moon, jd).ok()?;
            let tropical_lon = pos.longitude.to_degrees();
            Some(vedaksha_astro::sidereal::true_tropical_to_sidereal(
                tropical_lon,
                vedaksha_astro::sidereal::Ayanamsha::IndianOfficial,
                jd,
            ))
        };

        let get_sun_sidereal = |jd: f64| -> Option<f64> {
            let pos = coordinates::ecliptic_position(&provider, Body::Sun, jd).ok()?;
            let tropical_lon = pos.longitude.to_degrees();
            Some(vedaksha_astro::sidereal::true_tropical_to_sidereal(
                tropical_lon,
                vedaksha_astro::sidereal::Ayanamsha::IndianOfficial,
                jd,
            ))
        };

        // `tz_offset_minutes` (default 0/UT) names the vara's weekday only —
        // the sunrise instant that bounds the vara depends only on
        // latitude/longitude and is unaffected by this offset.
        let tz_offset_minutes = input.tz_offset_minutes.unwrap_or(0);
        // `elevation_m` (default 0/sea level) DOES change which sunrise bounds
        // each candidate's vara, unlike `tz_offset_minutes` above: the horizon
        // dip moves sunrise, and a candidate whose vara flips also has its
        // `quality_score` shift by the weekday term. It is threaded through
        // for the same reason `compute_panchanga` takes it — so that the two
        // tools, given the same observer, cannot report different weekdays for
        // the same instant.
        let elevation_m = input.elevation_m.unwrap_or(0.0);
        let sun_eq = |jd: f64| sun_equatorial_deg(&provider, jd);
        let mut assessments = vedaksha_vedic::muhurta::search_muhurta(
            input.start_jd,
            input.end_jd,
            input.latitude,
            input.longitude,
            elevation_m,
            tz_offset_minutes,
            &get_moon_sidereal,
            &get_sun_sidereal,
            &sun_eq,
            min_quality,
        );

        // Enrich the reported windows with exact tithi/nakshatra ending times.
        // These need the Moon/Sun daily motion, so they use apparent_position
        // (position + speed) — but only for the few windows that passed the
        // quality filter, not the position-only scan above.
        //
        // One ephemeris evaluation per instant, shared by every consumer of
        // that instant. Three things asked the provider for the same evaluation
        // twice over:
        //
        //   * The Moon is wanted tropical by the tithi refinement (the
        //     ayanamsha cancels in the elongation) and sidereal by the
        //     nakshatra refinement. Those are one `true_tropical_to_sidereal`
        //     apart, so a single evaluation yields both.
        //   * Each refinement asks for its body at the candidate instant twice
        //     — once to identify the boundary it is aiming at, once on the
        //     first Newton step, which starts from that same instant.
        //   * The tithi refinement wants the Moon AND the Sun at each `t` it
        //     visits. Asked separately, that entered the provider twice per
        //     instant and rebuilt the same three central-difference frames both
        //     times.
        //
        // `apparent_position` is a ±0.5-day central difference, i.e. three
        // ephemeris evaluations a call, so these repeats are the dominant cost
        // of the enrichment loop. Three things collapse them, and all three are
        // reuse of an identical call rather than a different computation, so
        // values are unchanged rather than approximated:
        //
        //   1. `apparent_positions` (plural) serves the tithi refinement's
        //      Moon-and-Sun pair from ONE entry: it builds the three frames once
        //      and shares a memoizing provider across the pair, so the ELP/MPP02
        //      lunar series each light-time correction pulls in is evaluated per
        //      timestamp rather than per body. It is bit-identical to
        //      `apparent_position` per body — asserted by
        //      `batch_matches_per_body_bit_for_bit`.
        //   2. Memo slots keyed on the *bit pattern* of the JD. An identical
        //      `jd` hands back the identical `f64`s the provider already
        //      produced; any other `jd` — including one differing in the last
        //      ulp — misses and recomputes. `AnalyticalProvider` is a unit
        //      struct, so its output depends on nothing but `(body, jd)`.
        //   3. A slot PINNED to the candidate instant for the whole of one
        //      iteration. Both refinements start their Newton walk at `a.jd`,
        //      but the tithi walk steps away from it first, so with a single
        //      rolling slot the nakshatra refinement's opening ask for `a.jd`
        //      always missed. The pinned slot is primed once per candidate,
        //      by the one evaluation the tithi refinement was going to make
        //      anyway.
        //
        // The slots are `Mutex`, not `Cell`, because the callback types these
        // are passed as carry `+ Sync` — a `&Cell` is not shareable across
        // threads and so cannot appear inside one. The lock is never contended
        // (this loop is serial) and is taken a few thousand times per served
        // request against a request measured in seconds, so it does not show up
        // in the timings below.
        //
        // Slot layout: `(jd.to_bits(), payload)`, the payload being
        // `(tropical_longitude_deg, longitude_speed)` per body.
        // Moon and Sun together, from one batch evaluation.
        let pair_at = |jd: f64| -> Option<vedaksha_vedic::muhurta::MoonAndSun> {
            let computed = coordinates::apparent_positions(&provider, &[Body::Moon, Body::Sun], jd);
            let mut it = computed.into_iter();
            let moon = it.next()?.1.ok()?;
            let sun = it.next()?.1.ok()?;
            Some((
                (moon.ecliptic.longitude.to_degrees(), moon.longitude_speed),
                (sun.ecliptic.longitude.to_degrees(), sun.longitude_speed),
            ))
        };

        // Pinned to the candidate instant; primed once per iteration below.
        let pinned: Slot<vedaksha_vedic::muhurta::MoonAndSun> = Mutex::new(None);
        // Rolling slot for the instants the Newton walks step to.
        let pair_rolling: Slot<vedaksha_vedic::muhurta::MoonAndSun> = Mutex::new(None);
        // Rolling slot for the nakshatra walk, which wants the Moon alone —
        // evaluating the Sun alongside it would be work nothing reads.
        let moon_rolling: Slot<(f64, f64)> = Mutex::new(None);

        let moon_and_sun = |jd: f64| -> Option<vedaksha_vedic::muhurta::MoonAndSun> {
            if let Some(hit) = read_slot(&pinned, jd).or_else(|| read_slot(&pair_rolling, jd)) {
                return Some(hit);
            }
            let pair = pair_at(jd)?;
            *pair_rolling.lock().expect("memo slot") = Some((jd.to_bits(), pair));
            Some(pair)
        };

        let moon_pos_speed = |jd: f64| -> Option<(f64, f64)> {
            // The pinned and rolling PAIR slots are consulted first: whenever
            // the Moon is already known at this instant, the pair holding it is
            // the cheapest place to find it.
            if let Some(((lon, speed), _)) =
                read_slot(&pinned, jd).or_else(|| read_slot(&pair_rolling, jd))
            {
                return Some((lon, speed));
            }
            if let Some(hit) = read_slot(&moon_rolling, jd) {
                return Some(hit);
            }
            let p = coordinates::apparent_position(&provider, Body::Moon, jd).ok()?;
            let moon = (p.ecliptic.longitude.to_degrees(), p.longitude_speed);
            *moon_rolling.lock().expect("memo slot") = Some((jd.to_bits(), moon));
            Some(moon)
        };

        let moon_sid_speed = |jd: f64| -> Option<(f64, f64)> {
            let (tropical_lon, speed) = moon_pos_speed(jd)?;
            let sid = vedaksha_astro::sidereal::true_tropical_to_sidereal(
                tropical_lon,
                vedaksha_astro::sidereal::Ayanamsha::IndianOfficial,
                jd,
            );
            Some((sid, speed))
        };
        for a in &mut assessments {
            // Prime the pinned slot. This is the evaluation `compute_tithi_end`
            // makes on its first line, hoisted so it also survives the tithi
            // Newton walk and serves `compute_nakshatra_end`'s opening ask.
            *pinned.lock().expect("memo slot") = pair_at(a.jd).map(|p| (a.jd.to_bits(), p));
            a.tithi_end_jd = vedaksha_vedic::muhurta::compute_tithi_end(a.jd, &moon_and_sun);
            a.nakshatra_end_jd =
                vedaksha_vedic::muhurta::compute_nakshatra_end(a.jd, &moon_sid_speed);
        }

        let results_json: Vec<serde_json::Value> = assessments
            .iter()
            .map(|a| {
                serde_json::json!({
                    "jd": a.jd,
                    "nakshatra": a.nakshatra.name(),
                    "tithi_number": a.tithi.number,
                    "tithi_name": a.tithi.name,
                    "tithi_end_jd": a.tithi_end_jd,
                    "nakshatra_end_jd": a.nakshatra_end_jd,
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
            // Echoed alongside the other two observer coordinates it now sits
            // with. A caller comparing this tool's `weekday` against
            // `compute_panchanga`'s can otherwise not see which horizon this
            // answer was computed on, which is the whole reason the two could
            // disagree. `search_muhurta` has no `vedaksha-wasm` twin, so this
            // key has no mirror to keep in sync.
            "elevation_m": elevation_m,
            "min_quality": min_quality,
            "result_count": results_json.len(),
            "results": results_json,
        }))
    }

    fn call_emit_graph(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        use vedaksha_graph::emitters::GraphEmitter;

        let input: crate::tools::emit_graph::EmitGraphInput = serde_json::from_value(args.clone())
            .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::emit_graph::validate(&input)?;

        // Two accepted shapes. A ChartGraph passes straight through; the output
        // of `compute_natal_chart` is converted here. Before v5.0.1 only the
        // first worked while the schema documented the second, so the
        // documented call failed with `missing field 'nodes'`.
        let graph: vedaksha_graph::ChartGraph = if input.chart_json.get("nodes").is_some() {
            serde_json::from_value(input.chart_json)
                .map_err(|e| McpError::invalid_parameter("chart_json", &e.to_string()))?
        } else {
            let chart =
                crate::tools::emit_graph::computed_chart_from_tool_output(&input.chart_json)
                    .map_err(|e| {
                        McpError::invalid_parameter(
                            "chart_json",
                            &format!("not a ChartGraph (no `nodes`) and not a computed chart: {e}"),
                        )
                    })?;
            // A chart result records neither the observer nor, reliably, the
            // instant. Rather than default them to zero — which would put a
            // fabricated observer into the emitted graph — require them.
            let (Some(latitude), Some(longitude)) = (input.latitude, input.longitude) else {
                return Err(McpError::invalid_parameter(
                    "latitude",
                    "building a graph from a computed chart needs `latitude` and \
                     `longitude`: the chart does not record where it was cast, and the \
                     graph's Chart node does. Pass the same values used to compute it.",
                ));
            };
            // `unwrap_or(f64::NAN)` here through v7.1.1. serde_json renders a
            // non-finite float as `null`, so a chart_json without a
            // `julian_day` produced a Chart node with `"julian_day": null` and
            // no error — the same fabricated-observer problem the latitude and
            // longitude requirement just above exists to prevent.
            let julian_day = input
                .chart_json
                .get("julian_day")
                .and_then(serde_json::Value::as_f64)
                .filter(|jd| jd.is_finite())
                .ok_or_else(|| {
                    McpError::invalid_parameter(
                        "chart_json",
                        "building a graph from a computed chart needs a finite \
                         `julian_day` in that chart: the graph's Chart node records \
                         the instant, and it cannot be invented.",
                    )
                })?;
            let classification = match input.classification.as_deref() {
                Some("identified") => vedaksha_graph::DataClassification::Identified,
                Some("pseudonymized") => vedaksha_graph::DataClassification::Pseudonymized,
                _ => vedaksha_graph::DataClassification::Anonymous,
            };
            vedaksha_graph::from_chart::chart_to_graph(
                &chart,
                julian_day,
                latitude,
                longitude,
                vedaksha_astro::dignity::RulershipScheme::Traditional,
                classification,
            )
        };

        // Emit using the requested format (validate() already normalises case,
        // but validate() doesn't mutate, so normalise here as well).
        let fmt = input.format.trim().to_lowercase();
        let output = match fmt.as_str() {
            "cypher" => vedaksha_graph::emitters::cypher::CypherEmitter.emit(&graph),
            "surreal" => vedaksha_graph::emitters::surreal::SurrealEmitter.emit(&graph),
            "jsonld" => vedaksha_graph::emitters::jsonld::JsonLdEmitter.emit(&graph),
            "json" => vedaksha_graph::emitters::json_graph::JsonGraphEmitter.emit(&graph),
            "embedding" => {
                vedaksha_graph::emitters::embedding_text::EmbeddingTextEmitter.emit(&graph)
            }
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
    fn tools_list_returns_every_registered_tool() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let tools = val["result"]["tools"].as_array().unwrap();
        assert_eq!(
            tools.len(),
            crate::tools::tool_definitions().len(),
            "tools/list must expose the whole registry"
        );
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
        assert!(names.contains(&"compute_shadbala"));
        assert!(names.contains(&"compute_ashtakavarga"));
        assert!(names.contains(&"compute_gochara"));
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

    /// The chart reports the rotation it applied, not merely the name it was
    /// asked for.
    ///
    /// Every longitude, the ascendant, the MC and all twelve cusps in a sidereal
    /// response have been rotated by this offset; a caller that cannot read it
    /// cannot check the rotation or convert back to tropical. The wasm surface
    /// emitted `ayanamsha_value` and this one did not — a real divergence, found
    /// by `mcp_surface_parity` in `vedaksha-wasm`, which covers only the sidereal
    /// request. The tropical case is asserted here because that is the default,
    /// and 0.0 is a claim ("no rotation was applied"), not an absent answer.
    #[test]
    fn compute_natal_reports_the_ayanamsha_offset_it_applied() {
        let s = server();
        let chart = |args: &str| -> serde_json::Value {
            let resp = s.handle_request(&format!(
                r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{
                    "name":"compute_natal_chart","arguments":{args}}}}}"#
            ));
            let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
            let text = val["result"]["content"][0]["text"]
                .as_str()
                .unwrap_or_else(|| panic!("expected a result, got: {val}"));
            serde_json::from_str(text).unwrap()
        };

        let tropical = chart(r#"{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2}"#);
        assert_eq!(
            tropical["true_ayanamsha_value"].as_f64(),
            Some(0.0),
            "the tropical default is a zero rotation, stated explicitly"
        );

        let sidereal = chart(
            r#"{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2,"ayanamsha":"Lahiri"}"#,
        );
        let offset = sidereal["true_ayanamsha_value"].as_f64().expect("a number");
        assert!(
            (23.8..23.9).contains(&offset),
            "the Indian official ayanamsha at J2000 is 23°51′25″.53 ≈ 23.86°, got {offset}"
        );
        assert!(
            (offset
                - vedaksha_astro::sidereal::true_ayanamsha_value(
                    vedaksha_astro::sidereal::Ayanamsha::IndianOfficial,
                    2451545.0
                ))
            .abs()
                < 1e-9,
            "the reported offset must be the true (mean + nutation-in-longitude) \
             ayanamsha the library computed"
        );

        // And it must describe the payload it arrived with: the ascendant is
        // rotated by exactly this much relative to the tropical request.
        let (trop_asc, sid_asc) = (
            tropical["houses"]["asc"].as_f64().unwrap(),
            sidereal["houses"]["asc"].as_f64().unwrap(),
        );
        let applied = (trop_asc - sid_asc).rem_euclid(360.0);
        assert!(
            (applied - offset).abs() < 1e-9,
            "asc moved {applied}° but the report claims {offset}°"
        );
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

    fn dasha_text(s: &McpServer, args: &str) -> serde_json::Value {
        let resp = s.handle_request(&format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{
                "name":"compute_dasha","arguments":{args}
            }}}}"#
        ));
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected result, got: {val}");
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn compute_dasha_dispatches_ashtottari() {
        let s = server();
        let dasha = dasha_text(
            &s,
            r#"{"system":"Ashtottari","moon_longitude":45.0,"birth_jd":2451545.0}"#,
        );
        // AshtottariDasha has `starting_lord` (8-planet sequence) and `periods` —
        // distinct from VimshottariDasha which has `lord` and `maha_dashas`.
        assert!(dasha["starting_lord"].is_string());
        assert!(dasha["periods"].is_array());
    }

    #[test]
    fn compute_dasha_dispatches_yogini() {
        let s = server();
        let dasha = dasha_text(
            &s,
            r#"{"system":"Yogini","moon_longitude":45.0,"birth_jd":2451545.0}"#,
        );
        // YoginiDasha has `starting_yogini_index` and `maha_periods`.
        assert!(dasha["starting_yogini_index"].is_number());
        assert!(dasha["maha_periods"].is_array());
    }

    /// BPHS ch. 46's worked Aquarius-lagna chart (see
    /// `vedaksha_vedic::dasha::chara`'s `BPHS_46_CHART` test fixture), as a
    /// `graha_signs` JSON fragment for MCP-level requests.
    const BPHS_46_GRAHA_SIGNS_JSON: &str = r#""graha_signs":{
        "sun":9,"moon":2,"mars":1,"mercury":10,"jupiter":10,"venus":10,"saturn":4,"rahu":2
    }"#;

    /// The 1-indexed to 0-indexed `lagna_sign` conversion, at both ends of
    /// the range, for both Chara and Narayana.
    ///
    /// The schema documents `lagna_sign` as 1-indexed (1 = Aries);
    /// `compute_chara`/`compute_narayana` take 0-indexed. Shipping without
    /// that conversion is exactly the off-by-one defect that went unnoticed
    /// from v2.4.0 through v8.0.0. Both ends are checked because an
    /// off-by-two or sign-flipped conversion could pass an Aries-only check
    /// by coincidence.
    #[test]
    fn compute_dasha_chara_and_narayana_lagna_sign_is_1indexed() {
        let s = server();
        for system in ["Chara", "Narayana"] {
            let aries = dasha_text(
                &s,
                &format!(
                    r#"{{"system":"{system}","lagna_sign":1,"birth_jd":2451545.0,{BPHS_46_GRAHA_SIGNS_JSON}}}"#
                ),
            );
            let periods = periods_array(system, &aries);
            assert_eq!(
                periods[0]["sign_name"].as_str().unwrap(),
                "Aries",
                "{system} lagna_sign 1 must start on Aries, got: {periods:?}"
            );

            let pisces = dasha_text(
                &s,
                &format!(
                    r#"{{"system":"{system}","lagna_sign":12,"birth_jd":2451545.0,{BPHS_46_GRAHA_SIGNS_JSON}}}"#
                ),
            );
            let periods = periods_array(system, &pisces);
            assert_eq!(
                periods[0]["sign_name"].as_str().unwrap(),
                "Pisces",
                "{system} lagna_sign 12 must start on Pisces, got: {periods:?}"
            );
        }
    }

    /// Extract the sign-period array from a Chara or Narayana response.
    /// `compute_chara` serialises to a bare JSON array; `compute_narayana`
    /// serialises to an object with a `periods` field.
    fn periods_array<'a>(system: &str, dasha: &'a serde_json::Value) -> &'a Vec<serde_json::Value> {
        if let Some(arr) = dasha.as_array() {
            return arr;
        }
        dasha["periods"]
            .as_array()
            .unwrap_or_else(|| panic!("expected {system} response to be/contain an array: {dasha}"))
    }

    /// Duration VALUES, pinned against BPHS ch. 46's worked example — not
    /// just response shape. Previously this tool's tests asserted only
    /// `is_number()`/`is_array()`/`sign_name`, never a `duration_years`,
    /// which is exactly why a fully green gate covered chart-independent
    /// durations for this tool's entire life.
    #[test]
    fn compute_dasha_chara_duration_values_match_bphs_46() {
        let s = server();
        let dasha = dasha_text(
            &s,
            &format!(
                r#"{{"system":"Chara","lagna_sign":11,"birth_jd":2451545.0,{BPHS_46_GRAHA_SIGNS_JSON}}}"#
            ),
        );
        let periods = periods_array("Chara", &dasha);
        let mut by_sign = std::collections::HashMap::new();
        for p in periods {
            by_sign.insert(
                p["sign_name"].as_str().unwrap().to_string(),
                p["duration_years"].as_f64().unwrap(),
            );
        }
        // A representative sample, including both dual-lord signs.
        assert_eq!(by_sign["Aries"], 1.0);
        assert_eq!(by_sign["Taurus"], 9.0);
        assert_eq!(
            by_sign["Scorpio"], 6.0,
            "Scorpio is a dual-lord sign (Mars/Ketu)"
        );
        assert_eq!(
            by_sign["Aquarius"], 8.0,
            "Aquarius is a dual-lord sign (Saturn/Rahu)"
        );
        assert_eq!(by_sign["Pisces"], 1.0);
    }

    #[test]
    fn compute_dasha_narayana_duration_values_match_bphs_46() {
        let s = server();
        let dasha = dasha_text(
            &s,
            &format!(
                r#"{{"system":"Narayana","lagna_sign":11,"birth_jd":2451545.0,{BPHS_46_GRAHA_SIGNS_JSON}}}"#
            ),
        );
        let periods = periods_array("Narayana", &dasha);
        let mut by_sign = std::collections::HashMap::new();
        for p in periods {
            by_sign.insert(
                p["sign_name"].as_str().unwrap().to_string(),
                p["duration_years"].as_f64().unwrap(),
            );
        }
        assert_eq!(by_sign["Aries"], 1.0);
        assert_eq!(
            by_sign["Scorpio"], 6.0,
            "Scorpio is a dual-lord sign (Mars/Ketu)"
        );
        assert_eq!(
            by_sign["Aquarius"], 8.0,
            "Aquarius is a dual-lord sign (Saturn/Rahu)"
        );
    }

    /// The single most direct regression guard against the original defect:
    /// the same lagna with different graha positions must yield different
    /// durations. Before this fix, durations were read from a hardcoded
    /// table and this was impossible for ANY two charts.
    #[test]
    fn compute_dasha_chara_duration_depends_on_graha_signs() {
        let s = server();
        let bphs_chart = dasha_text(
            &s,
            &format!(
                r#"{{"system":"Chara","lagna_sign":1,"birth_jd":2451545.0,{BPHS_46_GRAHA_SIGNS_JSON}}}"#
            ),
        );
        let all_aries_chart = dasha_text(
            &s,
            r#"{"system":"Chara","lagna_sign":1,"birth_jd":2451545.0,
                "graha_signs":{"sun":0,"moon":0,"mars":0,"mercury":0,"jupiter":0,"venus":0,"saturn":0,"rahu":0}}"#,
        );
        let bphs_periods = periods_array("Chara", &bphs_chart);
        let aries_periods = periods_array("Chara", &all_aries_chart);
        assert_ne!(
            bphs_periods[0]["duration_years"], aries_periods[0]["duration_years"],
            "different graha_signs on the same lagna must not produce the same \
             durations -- that was exactly the original defect"
        );
    }

    #[test]
    fn compute_dasha_chara_without_graha_signs_returns_invalid_parameter() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{
                "name":"compute_dasha",
                "arguments":{"system":"Chara","lagna_sign":1,"birth_jd":2451545.0}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["error"].is_object(), "expected error, got: {val}");
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn compute_dasha_rejects_unknown_system() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{
                "name":"compute_dasha",
                "arguments":{"system":"BogusSystem","moon_longitude":45.0,"birth_jd":2451545.0}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["error"].is_object(), "expected error, got: {val}");
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn compute_dasha_chara_without_lagna_returns_invalid_parameter() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{
                "name":"compute_dasha",
                "arguments":{"system":"Chara","moon_longitude":45.0,"birth_jd":2451545.0}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["error"].is_object());
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "INVALID_PARAMETER"
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

    /// The documented workflow: compute a chart, hand it straight to
    /// `emit_graph`. Before v5.0.1 this returned `missing field 'nodes'` — the
    /// tool's own schema described a call that could not work, because nothing
    /// in the engine converted a computed chart into a graph.
    ///
    /// Note what the test above does NOT catch: it emits an *empty*
    /// `ChartGraph`, so it passes with zero nodes and zero edges. That is how a
    /// green suite coexisted with an unreachable feature. This one asserts the
    /// graph has real content drawn from the chart.
    #[test]
    fn emit_graph_consumes_compute_natal_chart_output() {
        let s = server();
        let call = |args: serde_json::Value| -> serde_json::Value {
            let req = serde_json::json!({
                "jsonrpc": "2.0", "id": 12, "method": "tools/call", "params": args
            });
            let resp = s.handle_request(&serde_json::to_string(&req).unwrap());
            serde_json::from_str(&resp).unwrap()
        };

        let natal = call(serde_json::json!({
            "name": "compute_natal_chart",
            "arguments": {"julian_day": 2451545.0, "latitude": 28.6, "longitude": 77.2}
        }));
        let chart: serde_json::Value = serde_json::from_str(
            natal["result"]["content"][0]["text"]
                .as_str()
                .unwrap_or_else(|| panic!("natal chart failed: {natal}")),
        )
        .unwrap();
        assert!(
            chart.get("nodes").is_none(),
            "premise: this is a chart, not a graph"
        );

        let emitted = call(serde_json::json!({
            "name": "emit_graph",
            "arguments": {
                "chart_json": chart, "format": "json",
                "latitude": 28.6, "longitude": 77.2
            }
        }));
        let text = emitted["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("emit_graph rejected a computed chart: {emitted}"));
        let graph: serde_json::Value = serde_json::from_str(text).unwrap();

        let nodes = graph["nodes"].as_array().expect("nodes array");
        let edges = graph["edges"].as_array().expect("edges array");
        // 1 chart + 12 signs + 12 houses + 10 planets.
        assert_eq!(
            nodes.len(),
            35,
            "expected a fully populated graph, got {}",
            nodes.len()
        );
        assert!(
            edges.len() > 30,
            "expected placement edges, got {}",
            edges.len()
        );
        assert!(
            text.contains("Sun") && text.contains("Aries"),
            "the graph must carry the chart's own planets and signs"
        );
    }

    /// Building from a computed chart without an observer must fail loudly
    /// rather than record latitude 0, longitude 0 in the graph's Chart node. A
    /// fabricated observer would be indistinguishable from a real one downstream.
    #[test]
    fn emit_graph_refuses_a_chart_without_an_observer() {
        let s = server();
        let req = serde_json::json!({
            "jsonrpc": "2.0", "id": 13, "method": "tools/call",
            "params": {"name": "emit_graph", "arguments": {
                "chart_json": {"planets": [], "houses": {
                    "cusps": [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0],
                    "asc": 0.0, "mc": 270.0,
                    "system": "WholeSign", "polar_fallback": false
                }, "aspects": [], "config_summary": "x"},
                "format": "json"
            }}
        });
        let val: serde_json::Value =
            serde_json::from_str(&s.handle_request(&serde_json::to_string(&req).unwrap())).unwrap();
        assert_eq!(
            val["error"]["data"]["error_code"].as_str().unwrap(),
            "INVALID_PARAMETER"
        );
        assert!(
            val["error"]["message"]
                .as_str()
                .unwrap()
                .contains("latitude"),
            "the error must name what is missing: {val}"
        );
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
        // One envelope for both input shapes as of v7.4.0; the
        // single-longitude path yields exactly one placement.
        assert_eq!(
            result["vargas"][0]["placements"][0]["varga_sign"]
                .as_u64()
                .unwrap(),
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
        let by_division = |code: &str| {
            result["vargas"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["division"] == code)
                .unwrap_or_else(|| panic!("no {code} in {result}"))
                .clone()
        };
        assert_eq!(
            by_division("D1")["placements"][0]["varga_sign"]
                .as_u64()
                .unwrap(),
            1,
            "45° should be Taurus (D1 sign 1)"
        );
        assert!(
            by_division("D9")["placements"][0]["varga_sign"].is_number(),
            "D9 result should be a number"
        );
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

    // 365-day transit search in debug mode runs unbounded — search iterates
    // each JD and re-evaluates the full ELP/MPP02 series per step. Release
    // mode runs in ~30s. Keep as a release-only smoke test via the full
    // validation workflow (cargo test --workspace --release runs without
    // --include-ignored).
    #[test]
    #[ignore = "release-only: 365-day search hangs in debug; runs in full-validation.yml"]
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

    /// FINDING 2 fix. `call_search_muhurta` reads `tz_offset_minutes` from
    /// the request and is supposed to pass it through to
    /// `vedaksha_vedic::muhurta::search_muhurta` — but nothing exercised
    /// that wiring: a reviewer mutated the handler to pass a hardcoded `0`
    /// instead of `input.tz_offset_minutes.unwrap_or(0)` and all 162
    /// existing `vedaksha-mcp` tests stayed green.
    ///
    /// Same instant/observer as `compute_panchanga_vara_uses_the_supplied_tz_offset`
    /// above and `far_east_and_far_west_cannot_share_a_vara_at_one_instant`
    /// in `vedaksha_vedic::muhurta` (jd 2459015.75, lat 0°, lon 165°E — the
    /// far-east case from a downstream consumer's report): both handlers
    /// derive the vara through the same
    /// `sun_equatorial_deg(&AnalyticalProvider, jd)` closure (see
    /// `call_compute_panchanga` and `call_search_muhurta` above), so the same
    /// jd/lat/lon must flip the same way here. The search window is collapsed
    /// to the single instant (`start_jd == end_jd == 2459015.75`) so exactly
    /// one candidate is evaluated, and `min_quality: 0.0` keeps it regardless
    /// of score.
    ///
    /// Confirmed empirically (not hand-derived): `search_muhurta` at
    /// tz_offset_minutes = +660 reports "Monday" for that one candidate; the
    /// same jd/lat/lon at tz_offset_minutes = 0 reports "Sunday" — the two
    /// must disagree, or the tz offset supplied by the caller is not
    /// reaching `vara_at` inside `search_muhurta`.
    #[test]
    fn search_muhurta_vara_uses_the_supplied_tz_offset() {
        let with_tz = serde_json::json!({
            "start_jd": 2_459_015.75, "end_jd": 2_459_015.75,
            "latitude": 0.0, "longitude": 165.0, "tz_offset_minutes": 660,
            "min_quality": 0.0
        });
        let without_tz = serde_json::json!({
            "start_jd": 2_459_015.75, "end_jd": 2_459_015.75,
            "latitude": 0.0, "longitude": 165.0, "tz_offset_minutes": 0,
            "min_quality": 0.0
        });
        let v1 = McpServer::call_search_muhurta(&with_tz).expect("valid input");
        let v2 = McpServer::call_search_muhurta(&without_tz).expect("valid input");

        assert_eq!(
            v1["result_count"].as_u64(),
            Some(1),
            "expected exactly one candidate at tz+660"
        );
        assert_eq!(
            v2["result_count"].as_u64(),
            Some(1),
            "expected exactly one candidate at tz 0"
        );

        let w1 = v1["results"][0]["weekday"]
            .as_str()
            .expect("weekday string");
        let w2 = v2["results"][0]["weekday"]
            .as_str()
            .expect("weekday string");
        assert_eq!(w1, "Monday");
        assert_eq!(w2, "Sunday");
        assert_ne!(
            w1, w2,
            "tz_offset_minutes must reach vara_at inside search_muhurta — \
             dropping it (e.g. hardcoding 0) must change this result"
        );
    }

    // ── compute_panchanga vara/kalam ──────────────────────────────────────────

    /// The panchanga's vara must follow the observer, not UT. Same instant,
    /// two observers either side of the UT day boundary.
    #[test]
    fn compute_panchanga_vara_follows_the_observer() {
        // 2020-06-14 20:00 Honolulu == 2020-06-15 06:00Z.
        let honolulu = serde_json::json!({
            "jd": 2_459_015.75, "sun": 84.0, "moon": 200.0,
            "latitude": 21.3069, "longitude": -157.8583, "tz_offset_minutes": -600
        });
        let v = McpServer::call_compute_panchanga(&honolulu).expect("valid input");
        assert_eq!(
            v["vara"]["weekday"], "Sunday",
            "20:00 Sunday evening in Honolulu is Ravivara, not Monday"
        );

        // `.is_number()` alone (what this test used to assert) passes for a
        // wrong anchor, a wrong slot, or a window running backwards. Pin the
        // three properties that actually define a Kalam window instead,
        // against values DERIVED from the sunrise/sunset primitives rather
        // than restated: with the Sun's own ephemeris at this observer,
        //
        //   sunrise = previous_rise(jd)          = JD 2459015.159156623
        //   sunset  = first set after sunrise    = JD 2459015.718493310
        //   daytime = sunset - sunrise           = 0.559336687 d (13.4241 h)
        //   eighth  = daytime / 8                = 0.069917086 d (100.68 min)
        //
        // Sunday's slots are rahu 8 and gulika 7 (Kalaprakashika; Muhurtha
        // Chintamani), so rahu spans [sunrise + 7·eighth, sunset] and gulika
        // [sunrise + 6·eighth, sunrise + 7·eighth) — adjacent, gulika first.
        // The derivation is re-run here from the primitives rather than
        // hardcoded, so this cannot drift from the engine while still
        // constraining it: a wrong anchor or a wrong slot inside
        // `kalam_windows` changes the engine's answer without changing the
        // reference.
        let sunrise =
            vedaksha_astro::riseset::previous_rise(2_459_015.75, 21.3069, -157.8583, 0.0, &|jd| {
                vedaksha_astro::riseset::sun_equatorial_deg(
                    &vedaksha_ephem_core::analytical::AnalyticalProvider,
                    jd,
                )
            })
            .expect("the Sun rises in Honolulu");
        let sunset =
            vedaksha_astro::riseset::sun_rise_set(sunrise, 21.3069, -157.8583, 0.0, &|jd| {
                vedaksha_astro::riseset::sun_equatorial_deg(
                    &vedaksha_ephem_core::analytical::AnalyticalProvider,
                    jd,
                )
            })
            .set
            .expect("and sets again");
        let eighth = (sunset - sunrise) / 8.0;

        let f = |path: [&str; 2]| {
            v["vara"][path[0]][path[1]]
                .as_f64()
                .unwrap_or_else(|| panic!("{path:?} must be a number"))
        };
        let (rahu_start, rahu_end) = (f(["rahu_kalam", "start_jd"]), f(["rahu_kalam", "end_jd"]));
        let (gulika_start, gulika_end) = (
            f(["gulika_kalam", "start_jd"]),
            f(["gulika_kalam", "end_jd"]),
        );

        // (1) Ordering — each window runs forwards.
        assert!(
            rahu_end > rahu_start,
            "rahu ran backwards: {rahu_start} .. {rahu_end}"
        );
        assert!(
            gulika_end > gulika_start,
            "gulika ran backwards: {gulika_start} .. {gulika_end}"
        );

        // (2) Width — each window is exactly one eighth of the daytime.
        // 1e-9 d ≈ 86 µs, far below the ~100 min a whole eighth spans.
        assert!(
            (rahu_end - rahu_start - eighth).abs() < 1e-9,
            "rahu width {} d != one eighth of the daytime {eighth} d",
            rahu_end - rahu_start
        );
        assert!(
            (gulika_end - gulika_start - eighth).abs() < 1e-9,
            "gulika width {} d != one eighth of the daytime {eighth} d",
            gulika_end - gulika_start
        );

        // (3) Anchor — each start is `sunrise + (slot - 1) · eighth` for the
        // slot the response itself reports, so a slot/window mismatch fails.
        let rahu_slot = v["vara"]["rahu_kalam_slot"].as_u64().expect("slot");
        let gulika_slot = v["vara"]["gulika_kalam_slot"].as_u64().expect("slot");
        assert_eq!((rahu_slot, gulika_slot), (8, 7), "Sunday's classical slots");
        #[expect(
            clippy::cast_precision_loss,
            reason = "slot is 1..=8 by construction (Weekday::rahu_kalam_slot / \
                      gulika_kalam_slot are exhaustive matches over literals in that \
                      range), so the u64 -> f64 conversion is exact"
        )]
        let anchor = |slot: u64| sunrise + (slot - 1) as f64 * eighth;
        assert!(
            (rahu_start - anchor(rahu_slot)).abs() < 1e-9,
            "rahu anchored at {rahu_start}, not at slot {rahu_slot}'s {}",
            anchor(rahu_slot)
        );
        assert!(
            (gulika_start - anchor(gulika_slot)).abs() < 1e-9,
            "gulika anchored at {gulika_start}, not at slot {gulika_slot}'s {}",
            anchor(gulika_slot)
        );

        // (4) Both windows lie inside the daytime they divide.
        assert!(gulika_start >= sunrise - 1e-9 && rahu_end <= sunset + 1e-9);
    }

    /// FIX 1. `vara.from_sunrise` must be `true` wherever a sunrise actually
    /// bounds the vara — the mid-latitude case, which is every observer this
    /// tool is normally asked about. Paired with
    /// `compute_panchanga_from_sunrise_is_false_in_the_polar_summer` below:
    /// a flag hardcoded to either constant fails one of the two.
    #[test]
    fn compute_panchanga_from_sunrise_is_true_at_a_mid_latitude() {
        // Chennai, J2000.0. `previous_rise` finds JD 2451544.542324732.
        let chennai = serde_json::json!({
            "jd": 2_451_545.0, "sun": 280.0, "moon": 223.3238,
            "latitude": 13.08, "longitude": 80.27, "tz_offset_minutes": 330
        });
        let v = McpServer::call_compute_panchanga(&chennai).expect("valid input");
        assert_eq!(v["vara"]["from_sunrise"], true);
        // Sanity: a real sunrise was found, so the windows exist too.
        assert!(v["vara"]["rahu_kalam"]["start_jd"].is_number());
    }

    /// FIX 1, the case that matters. Above the Arctic Circle in local summer
    /// the Sun never sets and so never rises: `previous_rise` returns `None`
    /// and the reported `weekday` is the observer's CIVIL weekday, a
    /// different quantity from a vara. Before this flag existed that fallback
    /// was emitted with no error and no marker — the original UT-weekday
    /// defect surviving at high latitude — and `rahu_kalam: null` was not a
    /// usable substitute, since the windows are also null when a sunrise WAS
    /// found but the sunset was not.
    ///
    /// Ny-Ålesund (78.22 N, 15.65 E) at JD 2459016.0 = 2020-06-15 12:00 UT,
    /// midnight sun. Verified by direct call: `previous_rise` there is
    /// `None`. `tz_offset_minutes = 60` (CEST-less Svalbard standard time)
    /// names the fallback Monday — 2020-06-15 was a Monday.
    #[test]
    fn compute_panchanga_from_sunrise_is_false_in_the_polar_summer() {
        let svalbard = serde_json::json!({
            "jd": 2_459_016.0, "sun": 84.0, "moon": 200.0,
            "latitude": 78.22, "longitude": 15.65, "tz_offset_minutes": 60
        });
        let v = McpServer::call_compute_panchanga(&svalbard).expect("valid input");
        assert_eq!(
            v["vara"]["from_sunrise"], false,
            "the midnight sun has no sunrise to reckon a vara from"
        );
        assert_eq!(v["vara"]["weekday"], "Monday", "the civil weekday fallback");

        // The key must be PRESENT and boolean on both branches, not omitted
        // on one — `serde_json` indexing cannot tell absent from null.
        let vara = v["vara"].as_object().expect("vara object");
        assert!(vara.contains_key("from_sunrise"));
        assert!(vara["from_sunrise"].is_boolean());
    }

    /// FIX 2. `search_muhurta` derived its vara at sea level while
    /// `compute_panchanga` was elevation-aware, so at altitude the two tools
    /// disagreed about the weekday for the same instant and observer, with no
    /// parameter a caller could set to reconcile them.
    ///
    /// Lhasa (29.65 N, 91.13 E, 3650 m), tz +480. Derived by direct call, not
    /// restated: the horizon dip at 3650 m is −0.0293·√3650 = −1.7702°, which
    /// puts sunrise on 2020-06-15 at JD 2459015.448311250 instead of the
    /// sea-level JD 2459015.454732473 — 9.2466 minutes earlier. (The review
    /// brief estimated ~8 minutes; the measured figure is 9.25.) Every
    /// instant in between is in one vara for a sea-level observer and the
    /// NEXT vara for the observer actually standing in Lhasa.
    ///
    /// The search window is collapsed to a single instant at the midpoint of
    /// that gap, JD 2459015.451521861, with `min_quality: 0.0` so the one
    /// candidate is kept regardless of score.
    #[test]
    fn search_muhurta_vara_uses_the_supplied_elevation() {
        let at = |elevation_m: f64| {
            serde_json::json!({
                "start_jd": 2_459_015.451_521_861_f64, "end_jd": 2_459_015.451_521_861_f64,
                "latitude": 29.65, "longitude": 91.13,
                "elevation_m": elevation_m, "tz_offset_minutes": 480,
                "min_quality": 0.0
            })
        };
        let sea = McpServer::call_search_muhurta(&at(0.0)).expect("valid input");
        let lhasa = McpServer::call_search_muhurta(&at(3650.0)).expect("valid input");

        assert_eq!(sea["result_count"].as_u64(), Some(1));
        assert_eq!(lhasa["result_count"].as_u64(), Some(1));

        assert_eq!(sea["results"][0]["weekday"], "Sunday");
        assert_eq!(
            lhasa["results"][0]["weekday"], "Monday",
            "elevation_m must reach the vara derivation inside search_muhurta \
             — dropping it (e.g. hardcoding 0.0) makes this Sunday too"
        );

        // The disagreement is not cosmetic: the vara feeds `quality_score`.
        // `assess_muhurta` adds +0.1 for Monday and nothing for Sunday, so
        // the two scores must differ by exactly that.
        let score = |v: &serde_json::Value| v["results"][0]["quality_score"].as_f64().unwrap();
        assert!(
            (score(&lhasa) - score(&sea) - 0.1).abs() < 1e-12,
            "expected the Monday/Sunday weekday term to shift the score by \
             0.1: sea {} vs Lhasa {}",
            score(&sea),
            score(&lhasa)
        );

        // And the answer says which horizon it was computed on.
        assert_eq!(sea["elevation_m"].as_f64(), Some(0.0));
        assert_eq!(lhasa["elevation_m"].as_f64(), Some(3650.0));
    }

    /// FIX 2's other half: the two tools must now AGREE, given the same
    /// observer. Same instant, same coordinates, same elevation — the vara
    /// `compute_panchanga` reports and the vara `search_muhurta` reports for
    /// its single candidate must be the same weekday, at altitude as well as
    /// at sea level.
    #[test]
    fn search_muhurta_and_compute_panchanga_agree_on_the_vara_at_altitude() {
        for elevation_m in [0.0_f64, 3650.0] {
            let jd = 2_459_015.451_521_861_f64;
            let p = McpServer::call_compute_panchanga(&serde_json::json!({
                "jd": jd, "sun": 84.0, "moon": 200.0,
                "latitude": 29.65, "longitude": 91.13,
                "elevation_m": elevation_m, "tz_offset_minutes": 480
            }))
            .expect("valid input");
            let s = McpServer::call_search_muhurta(&serde_json::json!({
                "start_jd": jd, "end_jd": jd,
                "latitude": 29.65, "longitude": 91.13,
                "elevation_m": elevation_m, "tz_offset_minutes": 480,
                "min_quality": 0.0
            }))
            .expect("valid input");
            assert_eq!(
                p["vara"]["weekday"], s["results"][0]["weekday"],
                "the two tools disagreed about the weekday at elevation \
                 {elevation_m} m for one instant and one observer"
            );
        }
    }

    /// The declared elevation bounds must be enforced, not decorative.
    #[test]
    fn compute_panchanga_and_search_muhurta_reject_an_absurd_elevation() {
        let p = serde_json::json!({
            "jd": 2_451_545.0, "sun": 84.0, "moon": 200.0,
            "latitude": 13.08, "longitude": 80.27, "elevation_m": 1e9
        });
        assert!(McpServer::call_compute_panchanga(&p).is_err());
        let s = serde_json::json!({
            "start_jd": 2_451_545.0, "end_jd": 2_451_546.0,
            "latitude": 13.08, "longitude": 80.27, "elevation_m": -1e9
        });
        assert!(McpServer::call_search_muhurta(&s).is_err());
    }

    /// The MCP wiring must actually pass `tz_offset_minutes` through to
    /// `vara_at`, not silently drop it (e.g. always call with 0). Same
    /// instant and observer position as `far_east_and_far_west_cannot_share_a_vara_at_one_instant`
    /// in `vedaksha_vedic::muhurta` (jd 2459015.75, lon 165°E — the far-east
    /// case from a downstream consumer's report), at UTC+11. Confirmed
    /// empirically (not hand-derived): calling `compute_panchanga` at
    /// tz_offset_minutes = +660 returns "Monday"; at the same jd/lat/lon with
    /// tz_offset_minutes forced to 0 it returns "Sunday" — the two must
    /// disagree, or the tz offset supplied by the caller is not reaching
    /// `vara_at`.
    #[test]
    fn compute_panchanga_vara_uses_the_supplied_tz_offset() {
        let with_tz = serde_json::json!({
            "jd": 2_459_015.75, "sun": 84.0, "moon": 200.0,
            "latitude": 0.0, "longitude": 165.0, "tz_offset_minutes": 660
        });
        let without_tz = serde_json::json!({
            "jd": 2_459_015.75, "sun": 84.0, "moon": 200.0,
            "latitude": 0.0, "longitude": 165.0, "tz_offset_minutes": 0
        });
        let v1 = McpServer::call_compute_panchanga(&with_tz).expect("valid input");
        let v2 = McpServer::call_compute_panchanga(&without_tz).expect("valid input");
        assert_eq!(v1["vara"]["weekday"], "Monday");
        assert_eq!(v2["vara"]["weekday"], "Sunday");
        assert_ne!(
            v1["vara"]["weekday"], v2["vara"]["weekday"],
            "tz_offset_minutes must reach vara_at — dropping it must change this result"
        );
    }

    /// Observer coordinates are required for a vara and must be rejected, not
    /// silently defaulted, when out of range.
    #[test]
    fn compute_panchanga_rejects_an_impossible_latitude() {
        let bad = serde_json::json!({
            "jd": 2_451_545.0, "sun": 84.0, "moon": 200.0,
            "latitude": 91.0, "longitude": 0.0
        });
        assert!(McpServer::call_compute_panchanga(&bad).is_err());
    }
}
