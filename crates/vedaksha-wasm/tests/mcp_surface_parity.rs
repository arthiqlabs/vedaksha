// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Cross-surface parity: a `vedaksha-wasm` export and the `vedaksha-mcp` tool
//! of the same name must emit the SAME JSON SHAPE. Covered here:
//! `compute_panchanga`, `compute_synastry`, `compute_composite`.
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

/// The MCP surface's payload for one tool and one set of arguments.
fn mcp_tool(tool: &str, arguments: Value) -> Value {
    let server = vedaksha_mcp::server::McpServer::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": tool, "arguments": arguments }
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

fn compare(label: &str, tool: &str, arguments: Value, wasm_json: &str) {
    let mcp = mcp_tool(tool, arguments);
    let wasm: Value = serde_json::from_str(wasm_json).expect("wasm output is JSON");

    assert!(
        !matches!(&mcp, Value::Array(items) if items.is_empty()),
        "[{label}] the MCP surface returned an empty array — an empty payload \
         has no leaves, so this comparison would pass vacuously. Choose \
         arguments that produce output."
    );

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
        "compute_panchanga",
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
        "compute_panchanga",
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
        "compute_panchanga",
        serde_json::json!({
            "jd": 2_459_015.451_521_861_f64, "sun": 84.0, "moon": 200.0,
            "latitude": 29.65, "longitude": 91.13,
            "elevation_m": 3650.0, "tz_offset_minutes": 480
        }),
        &out,
    );
}

// ── compute_synastry / compute_composite ─────────────────────────────────────
//
// Both were pure, unreachable modules in `vedaksha-astro` until they were
// wired to these two surfaces, so they have never had a chance to drift — this
// is where that stays true. Every figure below is derived in the comment above
// it; none is copied from a run.

/// Cross-chart aspects, with one exact trine, one exact opposition and one
/// partial square so the fractional `orb` and `strength` leaves are compared
/// too, not just the integral ones.
///
/// Derivation, with `angular_separation` folded into [0, 180] and the Lilly
/// orbs the engine uses (trine/opposition 8°, square 7°, sextile 6°):
///   Moon 103 vs Mars 130 → 27°: |27−0|, |27−60|, |27−90| all exceed their
///     orbs, so no aspect.
///   Moon 103 vs Venus 190 → 87°: square, orb |87−90| = 3°, strength
///     1 − 3/7 = 0.571428…
///   Sun 10 vs Mars 130 → 120°: trine, orb 0, strength 1.
///   Sun 10 vs Venus 190 → 180°: opposition, orb 0, strength 1.
/// Three aspects, emitted in sorted-name order (Moon before Sun, Mars before
/// Venus) on both surfaces because both iterate a `BTreeMap`.
#[test]
fn compute_synastry_surfaces_agree_on_major_aspects() {
    let out = vedaksha_wasm::compute_synastry(
        r#"{"Sun":10.0,"Moon":103.0}"#,
        r#"{"Mars":130.0,"Venus":190.0}"#,
        "major",
        1.0,
    )
    .expect("valid input");
    compare(
        "synastry-major",
        "compute_synastry",
        serde_json::json!({
            "chart_a": { "Sun": 10.0, "Moon": 103.0 },
            "chart_b": { "Mars": 130.0, "Venus": 190.0 },
            "aspect_set": "major",
            "orb_factor": 1.0
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 3, "sanity: three aspects, got {v}");
    assert_eq!(arr[0]["chart_a_planet"], "Moon");
    assert_eq!(arr[0]["aspect_type"], "Square");
    assert!((arr[0]["orb"].as_f64().unwrap() - 3.0).abs() < 1e-9);
    assert!((arr[0]["strength"].as_f64().unwrap() - (1.0 - 3.0 / 7.0)).abs() < 1e-9);
    assert_eq!(arr[2]["aspect_type"], "Opposition");
}

/// The minor-aspect branch and a non-unit `orb_factor` — the two arguments the
/// "major"/1.0 case above leaves at their defaults, and therefore the two that
/// could be reordered or dropped on one surface without the first test noticing.
///
/// Derivation: Sun 0° vs Ketu 72.5° → separation 72.5°. The quintile sits at
/// 72° with a 2° default orb, halved by `orb_factor = 0.5` to 1.0°; the orb is
/// 0.5°, so it hits with strength 1 − 0.5/1.0 = 0.5. The nearest other angle,
/// the 60° sextile, is 12.5° away against a halved orb of 3° — no hit. So the
/// payload is exactly one quintile, and it is empty under "major".
#[test]
fn compute_synastry_surfaces_agree_on_minor_aspects_at_a_tight_orb() {
    let out = vedaksha_wasm::compute_synastry(r#"{"Sun":0.0}"#, r#"{"Ketu":72.5}"#, "all", 0.5)
        .expect("valid input");
    compare(
        "synastry-all-tight",
        "compute_synastry",
        serde_json::json!({
            "chart_a": { "Sun": 0.0 },
            "chart_b": { "Ketu": 72.5 },
            "aspect_set": "all",
            "orb_factor": 0.5
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 1, "sanity: exactly the quintile, got {v}");
    assert_eq!(arr[0]["aspect_type"], "Quintile");
    assert!((arr[0]["strength"].as_f64().unwrap() - 0.5).abs() < 1e-9);
}

/// The composite, covering both branches of the shorter-arc midpoint.
///
/// Derivation:
///   Moon 350° / 10° — diff = normalize(10 − 350) = 20° ≤ 180°, so the wrap
///     branch is taken: normalize(350 + 20/2) = normalize(360) = 0°. Speeds
///     13.2 and 12.0 → 12.6.
///   Sun 100° / 200° — diff = 100° ≤ 180°, no wrap: 100 + 50 = 150°. Speeds
///     1.0 and 0.9 → 0.95.
/// Emitted Moon first on both surfaces (sorted keys).
#[test]
fn compute_composite_surfaces_agree() {
    let chart_a = r#"{"Moon":{"longitude":350.0,"speed":13.2},
                      "Sun":{"longitude":100.0,"speed":1.0}}"#;
    let chart_b = r#"{"Moon":{"longitude":10.0,"speed":12.0},
                      "Sun":{"longitude":200.0,"speed":0.9}}"#;
    let out = vedaksha_wasm::compute_composite(chart_a, chart_b).expect("valid input");
    compare(
        "composite",
        "compute_composite",
        serde_json::json!({
            "chart_a": {
                "Moon": { "longitude": 350.0, "speed": 13.2 },
                "Sun":  { "longitude": 100.0, "speed": 1.0 }
            },
            "chart_b": {
                "Moon": { "longitude": 10.0, "speed": 12.0 },
                "Sun":  { "longitude": 200.0, "speed": 0.9 }
            }
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v[0]["planet"], "Moon");
    assert!(v[0]["longitude"].as_f64().unwrap().abs() < 1e-9);
    assert!((v[0]["speed"].as_f64().unwrap() - 12.6).abs() < 1e-9);
    assert_eq!(v[1]["planet"], "Sun");
    assert!((v[1]["longitude"].as_f64().unwrap() - 150.0).abs() < 1e-9);
    assert!((v[1]["speed"].as_f64().unwrap() - 0.95).abs() < 1e-9);
}

/// Mismatched graha names must come back as a validation error, never as a
/// panic. Through the positional `vedaksha_astro::composite` signature this is
/// `assert_eq!(lons_a.len(), lons_b.len())` firing inside the engine, which on
/// the MCP server is a process-level abort, not a JSON-RPC error. Pairing by
/// name is what makes it unreachable, and this pins it.
///
/// Only the MCP side is exercised here: the wasm export's error arm builds a
/// `JsError`, and `wasm_bindgen` panics with "cannot call wasm-bindgen imported
/// functions on non-wasm targets" if that is reached from a host test. The wasm
/// rejection is pinned instead by
/// `synastry_composite_tests::composite_rejects_mismatched_names_instead_of_panicking`
/// in `src/lib.rs`, which calls `compute_composite_inner` directly.
#[test]
fn compute_composite_rejects_mismatched_grahas() {
    let server = vedaksha_mcp::server::McpServer::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "compute_composite", "arguments": {
            "chart_a": { "Sun": { "longitude": 10.0 }, "Moon": { "longitude": 100.0 } },
            "chart_b": { "Sun": { "longitude": 20.0 } }
        }}
    });
    let response: Value = serde_json::from_str(&server.handle_request(&request.to_string()))
        .expect("MCP response is JSON");
    assert_eq!(
        response["error"]["data"]["error_code"], "INVALID_PARAMETER",
        "MCP must return a validation error, got {response}"
    );
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("Moon"),
        "the error must name the offending graha: {response}"
    );
}
