// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Cross-surface parity: `vedaksha-wasm::compute_panchanga` and
//! `vedaksha-mcp`'s `compute_panchanga` tool must emit the SAME JSON SHAPE.
//!
//! # Why this file exists
//!
//! The two are independent hand-written `serde_json::json!` literals over one
//! shared engine, and callers are told they are interchangeable. Nothing
//! enforced that. The Python conformance harness
//! (`bindings/python/tests/conformance/`) is often assumed to — it does not:
//! it hosts `vedaksha-mcp` itself inside the wasm blob
//! (`bindings/python/engine` wraps `McpServer::handle_request`) and compares
//! that against the native `vedaksha-mcp`, i.e. one source compiled twice.
//! This crate's `compute_panchanga_inner` is never in that comparison.
//!
//! So a key added to one surface and misspelled — or forgotten — on the other
//! shipped green. `vara.from_sunrise` is exactly such a key.
//!
//! # What is asserted
//!
//! The full set of leaf key-paths, exactly, plus every non-numeric leaf value
//! exactly and every numeric leaf to 1e-9. Numbers are compared with a
//! tolerance rather than bit-for-bit because the two surfaces are separately
//! compiled crates and may inline the same arithmetic differently — a
//! documented ~1–2 ULP effect (see
//! `compute_panchanga_inner_vara_is_sunrise_based_not_ut_civil_day` in
//! `src/lib.rs`). 1e-9 d is ≈ 86 µs, far below any real disagreement: a wrong
//! slot moves a Kalam window by tens of minutes.

use serde_json::Value;

/// Every leaf as `(dotted.path, value)`.
fn leaves(v: &Value, path: String, out: &mut Vec<(String, Value)>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                leaves(child, format!("{path}.{k}"), out);
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                leaves(child, format!("{path}[{i}]"), out);
            }
        }
        other => out.push((path, other.clone())),
    }
}

/// The MCP surface's `compute_panchanga` payload for one set of arguments.
fn mcp_panchanga(arguments: Value) -> Value {
    let server = vedaksha_mcp::server::McpServer::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "compute_panchanga", "arguments": arguments }
    });
    let response: Value = serde_json::from_str(&server.handle_request(&request.to_string()))
        .expect("MCP response is JSON");
    assert!(
        response.get("error").is_none(),
        "MCP call errored: {response}"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("MCP tool result carries a text payload");
    serde_json::from_str(text).expect("the payload is JSON")
}

fn compare(label: &str, arguments: Value, wasm_json: &str) {
    let mcp = mcp_panchanga(arguments);
    let wasm: Value = serde_json::from_str(wasm_json).expect("wasm output is JSON");

    let (mut m, mut w) = (Vec::new(), Vec::new());
    leaves(&mcp, String::new(), &mut m);
    leaves(&wasm, String::new(), &mut w);

    let mpaths: Vec<&String> = m.iter().map(|(p, _)| p).collect();
    let wpaths: Vec<&String> = w.iter().map(|(p, _)| p).collect();
    assert_eq!(
        mpaths, wpaths,
        "[{label}] the two surfaces emit different key sets — a key added, \
         renamed or dropped on one surface only.\n  mcp : {mcp:#}\n  wasm: {wasm:#}"
    );

    for ((path, mv), (_, wv)) in m.iter().zip(w.iter()) {
        match (mv.as_f64(), wv.as_f64()) {
            (Some(a), Some(b)) => assert!(
                (a - b).abs() < 1e-9,
                "[{label}] {path}: mcp {a} vs wasm {b}"
            ),
            _ => assert_eq!(mv, wv, "[{label}] {path} differs"),
        }
    }
}

/// This crate's own `compute_panchanga` export, called the way a JS host
/// would. Going through the `#[wasm_bindgen]` entry point rather than the
/// private `compute_panchanga_inner` is deliberate: the exported function is
/// what actually ships, so this also catches an argument reordered between
/// the export and the implementation.
fn panchanga_wasm(
    jd: f64,
    sun: f64,
    moon: f64,
    latitude: f64,
    longitude: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
) -> Result<String, &'static str> {
    vedaksha_wasm::compute_panchanga(
        jd,
        sun,
        moon,
        latitude,
        longitude,
        elevation_m,
        tz_offset_minutes,
    )
    .map_err(|_| "compute_panchanga rejected the input")
}

/// Mid-latitude: a real sunrise, real Kalam windows, `from_sunrise` true.
#[test]
fn compute_panchanga_surfaces_agree_at_a_mid_latitude() {
    let out =
        panchanga_wasm(2_451_545.0, 280.0, 223.3238, 13.08, 80.27, 0.0, 330).expect("valid input");
    compare(
        "chennai",
        serde_json::json!({
            "jd": 2_451_545.0, "sun": 280.0, "moon": 223.3238,
            "latitude": 13.08, "longitude": 80.27,
            "elevation_m": 0.0, "tz_offset_minutes": 330
        }),
        &out,
    );
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["vara"]["from_sunrise"], true, "sanity: the true branch");
}

/// Polar summer: no sunrise, null Kalam windows, `from_sunrise` false. The
/// branch where the two surfaces are most likely to diverge, and the one the
/// fallback flag exists for.
#[test]
fn compute_panchanga_surfaces_agree_in_the_polar_summer() {
    let out = panchanga_wasm(2_459_016.0, 84.0, 200.0, 78.22, 15.65, 0.0, 60).expect("valid input");
    compare(
        "ny-alesund",
        serde_json::json!({
            "jd": 2_459_016.0, "sun": 84.0, "moon": 200.0,
            "latitude": 78.22, "longitude": 15.65,
            "elevation_m": 0.0, "tz_offset_minutes": 60
        }),
        &out,
    );
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["vara"]["from_sunrise"], false, "sanity: the false branch");
    assert!(v["vara"]["rahu_kalam"].is_null());
}

/// At altitude, where the elevation-aware sunrise is the one both surfaces
/// must land on.
#[test]
fn compute_panchanga_surfaces_agree_at_altitude() {
    let out = panchanga_wasm(
        2_459_015.451_521_861,
        84.0,
        200.0,
        29.65,
        91.13,
        3650.0,
        480,
    )
    .expect("valid input");
    compare(
        "lhasa",
        serde_json::json!({
            "jd": 2_459_015.451_521_861_f64, "sun": 84.0, "moon": 200.0,
            "latitude": 29.65, "longitude": 91.13,
            "elevation_m": 3650.0, "tz_offset_minutes": 480
        }),
        &out,
    );
}
