// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Every declared `outputSchema` is checked against a real response.
//!
//! MCP requires a tool that advertises an output schema to return
//! `structuredContent` conforming to it. A schema written from a sample and
//! never re-checked is exactly the kind of prose that rots, so this drives the
//! production server — `McpServer::handle_request`, the same entry point the
//! binary uses — and validates what actually comes back.
//!
//! The validator below covers the JSON Schema subset these schemas use:
//! `type`, `properties`, `required` and `items`. It is deliberately small, and
//! [`validator_rejects_what_it_should`] plants four defects to prove it can
//! fail — a validator that returns "valid" for everything would make every
//! assertion here meaningless.

use serde_json::{Value, json};
use vedaksha_mcp::server::McpServer;
use vedaksha_mcp::tools::tool_definitions;

/// Validate `value` against the JSON Schema subset used by our output schemas.
/// Returns the path of the first violation, or `None` when it conforms.
fn violation(value: &Value, schema: &Value, path: &str) -> Option<String> {
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        let ok = match ty {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            // Serde emits whole-valued f64 as `1.0`, which is still a number;
            // `is_i64`/`is_u64` alone would reject a legitimate integer field.
            "integer" => {
                value.is_i64() || value.is_u64() || value.as_f64().is_some_and(|f| f.fract() == 0.0)
            }
            "number" => value.is_number(),
            other => return Some(format!("{path}: schema names unknown type '{other}'")),
        };
        if !ok {
            return Some(format!("{path}: expected {ty}, found {value}"));
        }
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for key in required.iter().filter_map(Value::as_str) {
            if value.get(key).is_none() {
                return Some(format!("{path}: required property '{key}' is missing"));
            }
        }
    }

    if let (Some(props), Some(obj)) = (
        schema.get("properties").and_then(Value::as_object),
        value.as_object(),
    ) {
        for (key, sub_schema) in props {
            if let Some(sub_value) = obj.get(key) {
                if let Some(v) = violation(sub_value, sub_schema, &format!("{path}.{key}")) {
                    return Some(v);
                }
            }
        }
    }

    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        for (i, element) in arr.iter().enumerate() {
            if let Some(v) = violation(element, items, &format!("{path}[{i}]")) {
                return Some(v);
            }
        }
    }

    None
}

fn call(server: &McpServer, tool: &str, args: Value) -> Value {
    let request = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": tool, "arguments": args }
    });
    let raw = server.handle_request(&serde_json::to_string(&request).unwrap());
    let response: Value = serde_json::from_str(&raw).unwrap();
    assert!(
        response.get("error").is_none_or(Value::is_null),
        "{tool} returned an error: {}",
        response["error"]
    );
    response["result"].clone()
}

const JD: f64 = 2_451_545.0;
const LAT: f64 = 28.6139;
const LON: f64 = 77.2090;

/// A valid call for every tool that declares an output schema.
fn sample_arguments() -> Vec<(&'static str, Value)> {
    let lons = json!({
        "sun": 10.0, "moon": 100.0, "mars": 200.0, "mercury": 25.0,
        "jupiter": 300.0, "venus": 40.0, "saturn": 250.0
    });
    let signs = json!({
        "sun": 0, "moon": 3, "mars": 6, "mercury": 0,
        "jupiter": 10, "venus": 1, "saturn": 8
    });
    let planets: Vec<Value> = [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ]
    .iter()
    .zip([10.0, 100.0, 200.0, 25.0, 300.0, 40.0, 250.0])
    .map(|(name, longitude): (&&str, f64)| {
        json!({
            "planet": name, "sign": (longitude / 30.0) as u8, "longitude": longitude,
            "bhava": ((longitude / 30.0) as u8 % 12) + 1,
            "speed": 0.9, "average_speed": 1.0,
            "benefic_aspect_count": 1, "malefic_aspect_count": 0
        })
    })
    .collect();

    let mut with_lagna = signs.clone();
    with_lagna["lagna"] = json!(2);
    let mut with_reference = signs.clone();
    with_reference["natal_reference_sign"] = json!(3);
    let mut with_nodes = lons.clone();
    with_nodes["rahu"] = json!(150.0);
    with_nodes["ketu"] = json!(330.0);

    let chart_a = json!({ "Sun": 10.0, "Moon": 100.0, "Mars": 200.0 });
    let chart_b = json!({ "Sun": 23.0, "Moon": 113.0, "Mars": 213.0 });
    let positions_a = json!({
        "Sun": { "longitude": 10.0, "speed": 1.0 },
        "Moon": { "longitude": 100.0, "speed": 13.0 }
    });
    let positions_b = json!({
        "Sun": { "longitude": 23.0, "speed": 1.0 },
        "Moon": { "longitude": 113.0, "speed": 13.0 }
    });

    vec![
        (
            "compute_natal_chart",
            json!({ "julian_day": JD, "latitude": LAT, "longitude": LON }),
        ),
        (
            "compute_dasha",
            json!({ "birth_jd": JD, "moon_longitude": 100.0 }),
        ),
        ("compute_karakas", lons.clone()),
        ("compute_combustion", lons),
        (
            "compute_shadbala",
            json!({ "planets": planets, "is_daytime": true, "moon_phase_waxing": true }),
        ),
        (
            "compute_transit",
            json!({ "natal_jd": JD, "natal_lat": LAT, "natal_lon": LON, "transit_jd": JD + 365.0 }),
        ),
        (
            "search_transits",
            json!({
                "natal_positions": [{ "name": "Sun", "longitude": 10.0 }],
                "start_jd": JD, "end_jd": JD + 30.0
            }),
        ),
        (
            "search_muhurta",
            json!({ "start_jd": JD, "end_jd": JD + 2.0, "latitude": LAT, "longitude": LON }),
        ),
        ("compute_ashtakavarga", with_lagna),
        ("compute_gochara", with_reference),
        (
            "compute_panchanga",
            json!({ "jd": JD, "sun": 10.0, "moon": 100.0, "latitude": LAT, "longitude": LON }),
        ),
        ("compute_drishti", with_nodes),
        ("compute_bhavas", json!({ "ascendant": 15.0 })),
        (
            "compute_synastry",
            json!({ "chart_a": chart_a, "chart_b": chart_b }),
        ),
        (
            "compute_composite",
            json!({ "chart_a": positions_a, "chart_b": positions_b }),
        ),
    ]
}

#[test]
fn every_declared_output_schema_matches_a_real_response() {
    let server = McpServer::new();

    for (tool, args) in sample_arguments() {
        let definition = tool_definitions()
            .into_iter()
            .find(|t| t.name == tool)
            .unwrap_or_else(|| panic!("no such tool: {tool}"));
        let schema = definition
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{tool} is in the sample set but declares no output schema"));

        let result = call(&server, tool, args);
        let structured = result.get("structuredContent").unwrap_or_else(|| {
            panic!("{tool} declares an output schema but returned no structuredContent")
        });

        assert_eq!(
            violation(structured, schema, tool),
            None,
            "{tool}: structuredContent does not conform to its declared outputSchema"
        );
    }
}

/// Every tool is either covered by the sample set above or declares no schema.
/// Without this, adding a tool with a schema and forgetting the sample would
/// leave it silently unchecked.
#[test]
fn no_tool_declares_a_schema_without_a_sample_call() {
    let covered: Vec<&str> = sample_arguments().iter().map(|(name, _)| *name).collect();
    for tool in tool_definitions() {
        if tool.output_schema.is_some() {
            assert!(
                covered.contains(&tool.name),
                "{} declares an output schema but no sample call exercises it",
                tool.name
            );
        }
    }
}

/// A tool with no output schema must not emit structuredContent either: the
/// pair is what makes the promise checkable.
#[test]
fn tools_without_a_schema_emit_no_structured_content() {
    let server = McpServer::new();
    let chart = call(
        &server,
        "compute_natal_chart",
        json!({ "julian_day": JD, "latitude": LAT, "longitude": LON }),
    );
    let chart_json: Value =
        serde_json::from_str(chart["content"][0]["text"].as_str().unwrap()).unwrap();

    for (tool, args) in [
        (
            "emit_graph",
            json!({ "chart_json": chart_json, "format": "jsonld", "latitude": LAT, "longitude": LON }),
        ),
        (
            "compute_vargas",
            json!({ "julian_day": JD, "latitude": LAT, "longitude": LON, "divisions": ["D9"], "planet_longitude": 105.3 }),
        ),
    ] {
        let definition = tool_definitions()
            .into_iter()
            .find(|t| t.name == tool)
            .unwrap();
        assert!(
            definition.output_schema.is_none(),
            "{tool} now declares an output schema — add it to the sample set above"
        );
        let result = call(&server, tool, args);
        assert!(
            result.get("structuredContent").is_none(),
            "{tool} emitted structuredContent with no schema to validate it against"
        );
    }
}

/// The validator must be able to fail, or every assertion above is vacuous.
#[test]
fn validator_rejects_what_it_should() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": { "type": "integer" },
            "items": { "type": "array", "items": { "type": "object",
                "properties": { "name": { "type": "string" } }, "required": ["name"] } }
        },
        "required": ["count", "items"]
    });

    assert_eq!(
        violation(
            &json!({ "count": 1, "items": [{ "name": "a" }] }),
            &schema,
            "$"
        ),
        None,
        "a conforming value must pass"
    );

    for (bad, expected) in [
        (
            json!({ "items": [] }),
            "required property 'count' is missing",
        ),
        (json!({ "count": "one", "items": [] }), "expected integer"),
        (json!({ "count": 1, "items": {} }), "expected array"),
        (
            json!({ "count": 1, "items": [{ "label": "a" }] }),
            "required property 'name' is missing",
        ),
    ] {
        let found = violation(&bad, &schema, "$").expect("planted defect must be caught");
        assert!(
            found.contains(expected),
            "expected a violation mentioning '{expected}', got '{found}'"
        );
    }
}
