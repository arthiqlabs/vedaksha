// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
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
    // Each function validates inputs via the tool's own `validate()` function,
    // then either performs real computation or returns a validated stub pending
    // `EphemerisProvider` integration.

    fn call_compute_natal(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_natal::ComputeNatalInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_natal::validate(&input)?;

        // Full chart computation requires EphemerisProvider (SPK data file).
        // Returns validated acknowledgment; wire to SpkReader when data is embedded.
        Ok(serde_json::json!({
            "status": "validated",
            "julian_day": input.julian_day,
            "latitude": input.latitude,
            "longitude": input.longitude,
            "message": "Input validated. Full computation requires EphemerisProvider initialization."
        }))
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
            // Full chart computation requires EphemerisProvider — return a
            // validated stub until that integration is wired.
            Ok(serde_json::json!({
                "status": "validated",
                "message": "Input validated. Provide planet_longitude for direct varga computation, \
                    or await EphemerisProvider integration for full chart vargas."
            }))
        }
    }

    fn call_compute_transit(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::compute_transit::ComputeTransitInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::compute_transit::validate(&input)?;

        // Full computation requires EphemerisProvider.
        Ok(serde_json::json!({
            "status": "validated",
            "natal_jd": input.natal_jd,
            "transit_jd": input.transit_jd,
            "message": "Input validated. Full computation requires EphemerisProvider initialization."
        }))
    }

    fn call_search_transits(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::search_transits::SearchTransitsInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::search_transits::validate(&input)?;

        // Full computation requires EphemerisProvider.
        Ok(serde_json::json!({
            "status": "validated",
            "start_jd": input.start_jd,
            "end_jd": input.end_jd,
            "natal_position_count": input.natal_positions.len(),
            "message": "Input validated. Full transit search requires EphemerisProvider initialization."
        }))
    }

    fn call_search_muhurta(args: &serde_json::Value) -> Result<serde_json::Value, McpError> {
        let input: crate::tools::search_muhurta::SearchMuhurtaInput =
            serde_json::from_value(args.clone())
                .map_err(|e| McpError::invalid_parameter("arguments", &e.to_string()))?;
        crate::tools::search_muhurta::validate(&input)?;

        // Full computation requires EphemerisProvider.
        Ok(serde_json::json!({
            "status": "validated",
            "start_jd": input.start_jd,
            "end_jd": input.end_jd,
            "latitude": input.latitude,
            "longitude": input.longitude,
            "message": "Input validated. Full muhurta search requires EphemerisProvider initialization."
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
    fn tools_list_returns_seven_tools() {
        let s = server();
        let resp =
            s.handle_request(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":null}"#);
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let tools = val["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7, "expected exactly 7 tools");
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
        assert!(names.contains(&"compute_vargas"));
        assert!(names.contains(&"emit_graph"));
        assert!(names.contains(&"compute_transit"));
        assert!(names.contains(&"search_transits"));
        assert!(names.contains(&"search_muhurta"));
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
    fn compute_natal_with_valid_params_returns_validated_response() {
        let s = server();
        let resp = s.handle_request(
            r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{
                "name":"compute_natal_chart",
                "arguments":{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2}
            }}"#,
        );
        let val: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(val["result"].is_object(), "expected a result");
        // Content should be a text item.
        let text = val["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("validated"),
            "response should mention 'validated'"
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
    fn search_transits_valid_input_returns_validated_stub() {
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
        assert!(text.contains("validated"));
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
