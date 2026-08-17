// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Cross-surface parity: a `vedaksha-wasm` export and the `vedaksha-mcp` tool
//! of the same name must emit the SAME JSON SHAPE.
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
//! # Scope: 12 of the MCP surface's 17 tools
//!
//! Compared here — every tool that exists on BOTH surfaces:
//!
//! | tool | wasm export |
//! |---|---|
//! | `compute_panchanga`    | `compute_panchanga`    |
//! | `compute_synastry`     | `compute_synastry`     |
//! | `compute_composite`    | `compute_composite`    |
//! | `compute_ashtakavarga` | `compute_ashtakavarga` |
//! | `compute_bhavas`       | `compute_bhavas`       |
//! | `compute_combustion`   | `compute_combustion`   |
//! | `compute_dasha`        | `compute_dasha`        |
//! | `compute_drishti`      | `compute_drishti`      |
//! | `compute_gochara`      | `compute_gochara`      |
//! | `compute_karakas`      | `compute_karakas`      |
//! | `compute_natal_chart`  | `compute_natal_chart`  |
//! | `compute_shadbala`     | `compute_shadbala`     |
//!
//! The five remaining MCP tools are NOT compared, because there is nothing on
//! the wasm surface to compare them against:
//!
//! * `compute_transit`, `search_transits`, `search_muhurta`, `emit_graph` —
//!   no `#[wasm_bindgen]` export of any name implements them. No transit
//!   search, no muhurta search and no graph emission is reachable from the
//!   wasm surface at all. (`vedaksha-wasm` used to declare a `vedaksha-graph`
//!   dependency with the `emitters` feature while `src/lib.rs` never named
//!   `vedaksha_graph`; it bought no export and has been dropped.)
//! * `compute_vargas` — the wasm surface has `compute_varga` (singular), and
//!   it is a DIFFERENT SHAPE, not a renamed twin: it takes one longitude and
//!   one varga name and returns a bare `u8` sign index, while the MCP tool
//!   takes a whole chart and a list of vargas and returns a JSON divisional
//!   chart. There are no leaf paths to line up.
//!
//! Note that "same tool on both surfaces" is a claim about the OUTPUT shape,
//! not the input one: several pairs deliberately take different arguments
//! (`compute_natal_chart` takes a Julian Day on MCP and a calendar date on
//! wasm; `compute_karakas` takes capitalised graha keys on wasm and lowercase
//! scalars on MCP; `compute_combustion` splits longitudes and retrograde
//! flags into two JSON objects on wasm and one flat object on MCP). Each case
//! below therefore hand-maps the arguments and then demands that the payloads
//! match exactly. Extend this file when you change a tool's emitted shape.
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

// ── The nine remaining shared tools ──────────────────────────────────────────
//
// Every expected figure below is derived in the comment above the assertion
// that uses it, from the engine's own rules — masks, orb tables, house lists,
// dasha year counts. None is copied out of a run.

/// Bhinna Ashtakavarga and its Sarva roll-up.
///
/// Derivation of the two totals asserted:
///   `bhinna_ashtakavarga` walks all 12 signs and, for each of the 8
///   contributors, adds one bindu when bit `(sign − contributor_sign) mod 12`
///   of that contributor's mask is set. Over a full circuit of 12 signs each
///   bit index is hit exactly once, so a table's `total` is the summed
///   popcount of its 8 masks and is INDEPENDENT of where the grahas actually
///   are. For the Sun row `[1995, 1572, 1995, 3892, 1328, 2144, 1995, 3628]`
///   the popcounts are 8, 4, 8, 7, 4, 3, 8, 6 → 48. Doing the same for the
///   other six rows gives 49, 39, 54, 56, 52, 39; Sarvashtakavarga is the
///   per-sign sum of all seven tables, so its 12 entries sum to
///   48+49+39+54+56+52+39 = 337.
/// Because those totals are position-independent they cannot show that the
/// input was read at all, so the spread across signs is asserted too: eight
/// distinct sign indices must produce a non-uniform Sarva row.
#[test]
fn compute_ashtakavarga_surfaces_agree() {
    let args = serde_json::json!({
        "sun": 3, "moon": 7, "mars": 1, "mercury": 10,
        "jupiter": 5, "venus": 8, "saturn": 11, "lagna": 0
    });
    let out = vedaksha_wasm::compute_ashtakavarga(&args.to_string()).expect("valid input");
    compare("ashtakavarga", "compute_ashtakavarga", args, &out);

    let v: Value = serde_json::from_str(&out).unwrap();
    let tables = v["tables"].as_array().expect("tables array");
    assert_eq!(tables.len(), 7, "seven bhinna tables, got {v}");
    assert_eq!(tables[0]["planet"], "Sun");
    assert_eq!(tables[0]["total"], 48, "summed popcount of the Sun masks");
    let sarva: Vec<u64> = v["sarvashtakavarga"]
        .as_array()
        .expect("sarva array")
        .iter()
        .map(|x| x.as_u64().unwrap())
        .collect();
    assert_eq!(sarva.len(), 12);
    assert_eq!(sarva.iter().sum::<u64>(), 337, "48+49+39+54+56+52+39");
    assert!(
        sarva.iter().min() != sarva.iter().max(),
        "sanity: the sign positions must move bindus around, got {sarva:?}"
    );
}

/// The whole-sign bhava chart, with grahas placed.
///
/// Derivation, with `lagna_sign = floor(218.5 / 30) = 7` (Scorpio) and
/// `planet_bhava(sign) = (sign + 12 − 7) mod 12 + 1`:
///   Jupiter 5.0°   → sign 0  → bhava (0+5) mod 12 + 1 = 6
///   Ketu    350.0° → sign 11 → bhava (11+5) mod 12 + 1 = 5
///   Mars    200.4° → sign 6  → bhava (6+5) mod 12 + 1 = 12
///   Sun     218.0° → sign 7  → bhava (7+5) mod 12 + 1 = 1
/// Emitted in sorted-name order on both surfaces — MCP deserialises into a
/// `BTreeMap`, and `serde_json`'s object map is a `BTreeMap` too (the
/// `preserve_order` feature is not enabled anywhere in this workspace).
/// Bhava 1 is both a kendra (1/4/7/10) and a trikona (1/5/9), and neither a
/// dusthana (6/8/12) nor an upachaya (3/6/10/11), so all four flags are
/// exercised on the first house alone.
#[test]
fn compute_bhavas_surfaces_agree() {
    let planets = serde_json::json!({
        "Jupiter": 5.0, "Ketu": 350.0, "Mars": 200.4, "Sun": 218.0
    });
    let out = vedaksha_wasm::compute_bhavas(218.5, &planets.to_string()).expect("valid input");
    compare(
        "bhavas",
        "compute_bhavas",
        serde_json::json!({ "ascendant": 218.5, "planets": planets }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["lagna_sign"], 7, "218.5° is Scorpio");
    let houses = v["houses"].as_array().expect("houses array");
    assert_eq!(houses.len(), 12);
    assert_eq!(houses[0]["sign"], 7);
    assert_eq!(houses[0]["is_kendra"], true);
    assert_eq!(houses[0]["is_trikona"], true);
    assert_eq!(houses[0]["is_dusthana"], false);
    assert_eq!(houses[0]["is_upachaya"], false);
    let placed = v["planets"].as_array().expect("planets array");
    assert_eq!(placed.len(), 4, "four grahas placed, got {v}");
    for (i, (name, sign, bhava)) in [
        ("Jupiter", 0, 6),
        ("Ketu", 11, 5),
        ("Mars", 6, 12),
        ("Sun", 7, 1),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(placed[i]["planet"], *name);
        assert_eq!(placed[i]["sign"], *sign);
        assert_eq!(placed[i]["bhava"], *bhava);
    }
}

/// Combustion, covering all three states and the retrograde-narrowed orb.
///
/// Derivation with the Sun at 100°. `combustion_state` compares the shortest
/// arc to the graha's orb: `< orb/3` → DeeplyCombust, `< orb` → Combust, else
/// None. Orbs are Moon 12°, Mars 17° direct / 8° retrograde, Mercury 14°/12°,
/// Jupiter 11°, Venus 10°/8°, Saturn 16°.
///   Moon    108.0° → sep 8.0°;  8 ≥ 12/3 = 4 and 8 < 12   → Combust
///   Mars    109.0° → sep 9.0°;  retrograde orb 8, 9 ≥ 8   → None
///                    (the discriminator: at the 17° direct orb this same
///                     placement would be Combust, so a dropped or misspelled
///                     `mars_retrograde` flips the answer)
///   Mercury 102.0° → sep 2.0°;  2 < 14/3 = 4.666…         → DeeplyCombust
///   Jupiter 150.0° → sep 50.0°                             → None
///   Venus   107.5° → sep 7.5°;  7.5 ≥ 10/3 and 7.5 < 10   → Combust
///   Saturn  200.0° → sep 100.0°                            → None
/// Six entries, always in the fixed Moon→Saturn order; the Sun is not one of
/// them (it is never combust relative to itself).
#[test]
fn compute_combustion_surfaces_agree_across_all_three_states() {
    let out = vedaksha_wasm::compute_combustion(
        r#"{"sun":100.0,"moon":108.0,"mars":109.0,"mercury":102.0,
            "jupiter":150.0,"venus":107.5,"saturn":200.0}"#,
        r#"{"mars":true}"#,
    )
    .expect("valid input");
    compare(
        "combustion",
        "compute_combustion",
        serde_json::json!({
            "sun": 100.0, "moon": 108.0, "mars": 109.0, "mercury": 102.0,
            "jupiter": 150.0, "venus": 107.5, "saturn": 200.0,
            "mars_retrograde": true
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 6, "the six combustible grahas, got {v}");
    for (i, (name, state, sep)) in [
        ("Moon", "Combust", 8.0),
        ("Mars", "None", 9.0),
        ("Mercury", "DeeplyCombust", 2.0),
        ("Jupiter", "None", 50.0),
        ("Venus", "Combust", 7.5),
        ("Saturn", "None", 100.0),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(arr[i]["planet"], *name);
        assert_eq!(arr[i]["state"], *state, "at {name}: {v}");
        assert!((arr[i]["degrees_from_sun"].as_f64().unwrap() - sep).abs() < 1e-9);
    }
}

/// The Vimshottari tree. `levels = 2` deliberately: it is not the MCP
/// default of 3, so a `levels` argument dropped on one surface changes the
/// payload instead of silently landing on the same default — and two levels
/// already exercise the nesting (9 × 9 periods) without the 9× larger tree
/// three levels would build.
///
/// Derivation. A nakshatra spans 360/27 = 13.333…°, so Moon 100.0° falls in
/// index floor(100 / 13.333…) = 7 = Pushya, whose Vimshottari lord is
/// `LORDS[7 mod 9]` = Saturn. The elapsed fraction is
/// (100 − 7 × 13.333…) / 13.333… = 0.5, so `initial_balance` = 0.5 and the
/// first (partial) maha dasha runs 19 years × 365.25 d × 0.5 = 3469.875 d
/// from `birth_jd`. The nine maha dashas then follow the fixed cycle from
/// Saturn, so the second is Mercury; sub-periods restart from the parent
/// lord, so the first antar dasha of the Saturn maha is Saturn's.
#[test]
fn compute_dasha_surfaces_agree() {
    let birth_jd = 2_446_231.937_5_f64;
    let out = vedaksha_wasm::compute_dasha(100.0, birth_jd, 2).expect("valid input");
    compare(
        "dasha-vimshottari",
        "compute_dasha",
        serde_json::json!({
            "system": "Vimshottari",
            "moon_longitude": 100.0,
            "birth_jd": birth_jd,
            "levels": 2
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["moon_nakshatra"], "Pushya");
    assert!((v["initial_balance"].as_f64().unwrap() - 0.5).abs() < 1e-9);
    let maha = v["maha_dashas"].as_array().expect("maha array");
    assert_eq!(maha.len(), 9, "one full 120-year cycle");
    assert_eq!(maha[0]["lord"], "Saturn");
    assert_eq!(maha[1]["lord"], "Mercury");
    assert!((maha[0]["duration_days"].as_f64().unwrap() - 3469.875).abs() < 1e-6);
    assert!((maha[0]["start_jd"].as_f64().unwrap() - birth_jd).abs() < 1e-9);
    let antar = maha[0]["sub_periods"].as_array().expect("sub_periods");
    assert_eq!(antar.len(), 9);
    assert_eq!(
        antar[0]["lord"], "Saturn",
        "sub-periods start at the parent"
    );
    assert_eq!(antar[0]["level"], 2);
    assert!(
        antar[0]["sub_periods"].as_array().unwrap().is_empty(),
        "levels = 2 must stop at antar"
    );
}

/// Graded graha drishti, covering the plain grade and all three overrides.
///
/// Derivation. `aspect_strength` returns a non-`None` grade for exactly seven
/// house distances — 3, 4, 5, 7, 8, 9, 10 — and `find_vedic_aspects` walks
/// distances 1..=12 for each graha in the order given, so the payload is
/// 9 grahas × 7 grades = 63 aspects in a fixed layout: graha `p` occupies
/// indices `7p..7p+7`, and within a graha the order is 3, 4, 5, 7, 8, 9, 10.
/// The aspected sign is `(sign + houses − 1) mod 12` (houses are counted
/// inclusively in Jyotish). Grades: the 7th is Full for everyone; 3/10 are
/// Quarter except Saturn (Full), 4/8 are ThreeQuarter except Mars (Full),
/// 5/9 are Half except Jupiter (Full). Signs used below are
/// `floor(longitude / 30)`: Sun 10° → 0, Mars 200° → 6, Jupiter 250° → 8,
/// Saturn 170° → 5.
#[test]
fn compute_drishti_surfaces_agree() {
    let args = serde_json::json!({
        "sun": 10.0, "moon": 100.0, "mars": 200.0, "mercury": 40.0,
        "jupiter": 250.0, "venus": 320.0, "saturn": 170.0,
        "rahu": 80.0, "ketu": 260.0
    });
    let out = vedaksha_wasm::compute_drishti(&args.to_string()).expect("valid input");
    compare("drishti", "compute_drishti", args, &out);

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(
        arr.len(),
        63,
        "9 grahas × 7 graded distances, got {}",
        arr.len()
    );

    // Sun (graha 0) at sign 0, its first emitted distance: 3 houses away.
    assert_eq!(arr[0]["aspecting_planet"], "Sun");
    assert_eq!(arr[0]["aspecting_sign"], 0);
    assert_eq!(arr[0]["houses_away"], 3);
    assert_eq!(arr[0]["aspected_sign"], 2, "(0 + 3 − 1) mod 12");
    assert_eq!(
        arr[0]["strength"], "Quarter",
        "3rd is Quarter for non-Saturn"
    );

    // The three special-aspect overrides, at index 7·p + offset with the
    // offsets 3→0, 4→1, 5→2, 7→3, 8→4, 9→5, 10→6.
    let mars_4th = &arr[7 * 2 + 1];
    assert_eq!(mars_4th["aspecting_planet"], "Mars");
    assert_eq!(mars_4th["houses_away"], 4);
    assert_eq!(
        mars_4th["strength"], "Full",
        "Mars overrides the 4th to Full"
    );
    assert_eq!(mars_4th["aspected_sign"], 9, "(6 + 4 − 1) mod 12");

    let jupiter_5th = &arr[7 * 4 + 2];
    assert_eq!(jupiter_5th["aspecting_planet"], "Jupiter");
    assert_eq!(jupiter_5th["houses_away"], 5);
    assert_eq!(jupiter_5th["strength"], "Full");
    assert_eq!(jupiter_5th["aspected_sign"], 0, "(8 + 5 − 1) mod 12");

    let saturn_3rd = &arr[7 * 6];
    assert_eq!(saturn_3rd["aspecting_planet"], "Saturn");
    assert_eq!(saturn_3rd["houses_away"], 3);
    assert_eq!(saturn_3rd["strength"], "Full");
    assert_eq!(saturn_3rd["aspected_sign"], 7, "(5 + 3 − 1) mod 12");

    // The 7th is Full for every graha, including the nodes.
    for p in 0..9 {
        let seventh = &arr[7 * p + 3];
        assert_eq!(seventh["houses_away"], 7);
        assert_eq!(seventh["strength"], "Full", "7th must be Full: {seventh}");
    }
}

/// Gochara with the raw geometry (`school = "Geometry"`).
///
/// Derivation, all against `natal_reference_sign = 0` so the house from the
/// reference is simply `sign + 1`, with the BPHS Ch.29 favourable-house lists
/// and vedha pairs. A vedha candidate is another graha standing in sign
/// `(0 + vedha_house − 1) mod 12`.
///   Sun     sign 2  → house 3;  favourable [3,6,10,11];      vedha 3→9  → sign 8  → Moon
///   Moon    sign 8  → house 9;  favourable [1,3,6,7,10,11]   → unfavourable, no vedha
///   Mars    sign 5  → house 6;  favourable [3,6,11];         vedha 6→9  → sign 8  → Moon
///   Mercury sign 3  → house 4;  favourable [2,4,6,8,10,11];  vedha 4→3  → sign 2  → Sun
///   Jupiter sign 4  → house 5;  favourable [2,5,7,9,11];     vedha 5→4  → sign 3  → Mercury
///   Venus   sign 10 → house 11; favourable [1,2,3,4,5,8,9,11,12]; vedha 11→6 → sign 5 → Mars
///   Saturn  sign 11 → house 12; favourable [3,6,11]          → unfavourable, no vedha
/// Seven entries — Rahu and Ketu are never returned, BPHS Ch.29 being silent
/// on the nodes.
#[test]
fn compute_gochara_surfaces_agree_on_raw_geometry() {
    let args = serde_json::json!({
        "sun": 2, "moon": 8, "mars": 5, "mercury": 3,
        "jupiter": 4, "venus": 10, "saturn": 11,
        "natal_reference_sign": 0, "school": "Geometry", "vedha_table": "Bphs29"
    });
    let out = vedaksha_wasm::compute_gochara(&args.to_string()).expect("valid input");
    compare("gochara-geometry", "compute_gochara", args, &out);

    let v: Value = serde_json::from_str(&out).unwrap();
    let entries = v["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 7, "seven grahas, no nodes, got {v}");
    for (i, (graha, house, effect, vedha)) in [
        ("Sun", 3, "Favourable", vec!["Moon"]),
        ("Moon", 9, "Unfavourable", vec![]),
        ("Mars", 6, "Favourable", vec!["Moon"]),
        ("Mercury", 4, "Favourable", vec!["Sun"]),
        ("Jupiter", 5, "Favourable", vec!["Mercury"]),
        ("Venus", 11, "Favourable", vec!["Mars"]),
        ("Saturn", 12, "Unfavourable", vec![]),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(entries[i]["graha"], *graha);
        assert_eq!(entries[i]["house_from_natal"], *house);
        assert_eq!(entries[i]["classical_effect"], *effect);
        assert_eq!(
            entries[i]["vedha_candidates"],
            serde_json::json!(vedha),
            "at {graha}: {v}"
        );
    }
}

/// The same transits under `school = "Parashari"`, whose exemption list drops
/// the Sun↔Moon and Jupiter↔Mercury vedha pairs. Same input, different
/// payload — which is what makes this a real second case rather than a
/// re-run: if `school` were dropped on one surface, that surface would fall
/// back to the `"Geometry"` default and the two would disagree here while
/// still agreeing above.
///
/// Derivation from the Geometry result above: the Sun's `[Moon]` and
/// Jupiter's `[Mercury]` are struck out; Mercury's `[Sun]`, Mars's `[Moon]`
/// and Venus's `[Mars]` are not exempt pairs and survive.
#[test]
fn compute_gochara_surfaces_agree_under_the_parashari_exemptions() {
    let args = serde_json::json!({
        "sun": 2, "moon": 8, "mars": 5, "mercury": 3,
        "jupiter": 4, "venus": 10, "saturn": 11,
        "natal_reference_sign": 0, "school": "Parashari"
    });
    let out = vedaksha_wasm::compute_gochara(&args.to_string()).expect("valid input");
    compare("gochara-parashari", "compute_gochara", args, &out);

    let v: Value = serde_json::from_str(&out).unwrap();
    let entries = v["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 7);
    assert_eq!(entries[0]["graha"], "Sun");
    assert!(
        entries[0]["vedha_candidates"]
            .as_array()
            .unwrap()
            .is_empty(),
        "Sun↔Moon is exempt under Parashari: {v}"
    );
    assert_eq!(entries[4]["graha"], "Jupiter");
    assert!(
        entries[4]["vedha_candidates"]
            .as_array()
            .unwrap()
            .is_empty(),
        "Jupiter↔Mercury is exempt under Parashari: {v}"
    );
    assert_eq!(
        entries[3]["vedha_candidates"],
        serde_json::json!(["Sun"]),
        "Mercury↔Sun is NOT exempt, so it must survive: {v}"
    );
    assert_eq!(entries[2]["vedha_candidates"], serde_json::json!(["Moon"]));
    assert_eq!(entries[5]["vedha_candidates"], serde_json::json!(["Mars"]));
}

/// Chara Karakas on the 8-graha scheme — the branch that adds Rahu and the
/// Pitrikaraka role, and the only one where a longitude is not used as-is.
///
/// Derivation. Ranking is by degrees within the sign, descending, with Rahu
/// reflected (30 − d) because it moves retrograde:
///   Sun 25° → 25, Moon 50° → 20, Mars 75° → 15, Mercury 100° → 10,
///   Jupiter 125° → 5, Venus 152° → 2, Saturn 181° → 1,
///   Rahu 202° → 22 reflected to 8.
/// Descending: Sun 25, Moon 20, Mars 15, Mercury 10, Rahu 8, Jupiter 5,
/// Venus 2, Saturn 1 — zipped onto the eight roles Atmakaraka, Amatyakaraka,
/// Bhratrikaraka, Matrikaraka, Pitrikaraka, Putrakaraka, Gnatikaraka,
/// Darakaraka. Rahu lands on Pitrikaraka, the role the 7-scheme does not
/// have, so a mis-wired scheme argument cannot produce this payload.
#[test]
fn compute_karakas_surfaces_agree_on_the_eight_graha_scheme() {
    let out = vedaksha_wasm::compute_karakas(
        r#"{"Sun":25.0,"Moon":50.0,"Mars":75.0,"Mercury":100.0,
            "Jupiter":125.0,"Venus":152.0,"Saturn":181.0,"Rahu":202.0}"#,
        "8",
    )
    .expect("valid input");
    compare(
        "karakas-8",
        "compute_karakas",
        serde_json::json!({
            "sun": 25.0, "moon": 50.0, "mars": 75.0, "mercury": 100.0,
            "jupiter": 125.0, "venus": 152.0, "saturn": 181.0,
            "rahu": 202.0, "scheme": "8"
        }),
        &out,
    );

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 8, "eight assignments under scheme '8', got {v}");
    for (i, (planet, karaka, degrees)) in [
        ("Sun", "Atmakaraka", 25.0),
        ("Moon", "Amatyakaraka", 20.0),
        ("Mars", "Bhratrikaraka", 15.0),
        ("Mercury", "Matrikaraka", 10.0),
        ("Rahu", "Pitrikaraka", 8.0),
        ("Jupiter", "Putrakaraka", 5.0),
        ("Venus", "Gnatikaraka", 2.0),
        ("Saturn", "Darakaraka", 1.0),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(arr[i]["planet"], *planet);
        assert_eq!(arr[i]["karaka"], *karaka, "at {planet}: {v}");
        assert!((arr[i]["degrees_in_sign"].as_f64().unwrap() - degrees).abs() < 1e-9);
    }
}

/// Shadbala, with both optional flags set and non-zero aspect counts so the
/// kala and drik components are live rather than sitting on their defaults.
///
/// Derivation of the asserted figures — the two that are fixed constants and
/// the two that are structural identities:
///   `naisargika_bala` is a per-graha constant: Sun 60.0, Saturn 8.57.
///   `total` is defined as naisargika + dig + sthana + kala + cheshta + drik,
///     so it must equal the sum of the six emitted components exactly.
///   `uccha_bala` is the same value as `sthana_bala`.
/// Those hold for every graha in the payload, so they are checked for all
/// three rather than for a chosen one. Output order follows input order.
#[test]
fn compute_shadbala_surfaces_agree() {
    let args = serde_json::json!({
        "planets": [
            { "planet": "Sun", "sign": 2, "longitude": 60.6, "bhava": 10,
              "speed": 0.9556, "average_speed": 0.9856,
              "benefic_aspect_count": 1, "malefic_aspect_count": 2 },
            { "planet": "Moon", "sign": 5, "longitude": 165.0, "bhava": 12,
              "speed": 13.2, "average_speed": 13.176,
              "benefic_aspect_count": 2, "malefic_aspect_count": 0 },
            { "planet": "Saturn", "sign": 7, "longitude": 225.0, "bhava": 4,
              "speed": -0.05, "average_speed": 0.033,
              "benefic_aspect_count": 0, "malefic_aspect_count": 1 }
        ],
        "is_daytime": true,
        "moon_phase_waxing": true
    });
    let out = vedaksha_wasm::compute_shadbala(&args.to_string()).expect("valid input");
    compare("shadbala", "compute_shadbala", args, &out);

    let v: Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 3, "one row per supplied graha, got {v}");
    assert_eq!(arr[0]["planet"], "Sun");
    assert_eq!(arr[1]["planet"], "Moon");
    assert_eq!(arr[2]["planet"], "Saturn");
    assert!((arr[0]["naisargika_bala"].as_f64().unwrap() - 60.0).abs() < 1e-9);
    assert!((arr[2]["naisargika_bala"].as_f64().unwrap() - 8.57).abs() < 1e-9);
    for row in arr {
        let f = |k: &str| row[k].as_f64().unwrap();
        let six = f("sthana_bala")
            + f("dig_bala")
            + f("kala_bala")
            + f("cheshta_bala")
            + f("naisargika_bala")
            + f("drik_bala");
        assert!(
            (f("total") - six).abs() < 1e-9,
            "total must be the sum of the six components: {row}"
        );
        assert!(
            (f("uccha_bala") - f("sthana_bala")).abs() < 1e-9,
            "uccha_bala is sthana_bala: {row}"
        );
    }
}

/// The natal chart, SIDEREAL. Lahiri rather than the MCP schema's `Tropical`
/// default on purpose: the frame defect this release fixed — feeding the
/// TT Julian Day to the Earth-rotation term instead of the UT1 one — moves
/// the ascendant, the MC and all twelve cusps, and the tropical path exercises
/// neither the ayanamsha subtraction nor the sidereal `config_summary`.
///
/// The two surfaces take different arguments here: MCP takes a UT1 Julian Day
/// directly, wasm takes a calendar date it turns into one via
/// `julian::calendar_to_jd(year, month, day + h/24 + m/1440 + s/86400)`. That
/// same function is called here to produce the MCP argument, so both sides
/// land on bit-identical `jd` and any divergence below is the payload's, not
/// the clock's. Wasm also defaults to nine bodies against MCP's fixed ten, so
/// the tenth (`TrueNodeOsculating`) is named explicitly.
///
/// This case found a real divergence when it was first written: wasm emitted a
/// top-level `ayanamsha_value` and MCP emitted no such key, so a sidereal MCP
/// caller could not read the offset that had been applied to every longitude in
/// the payload it had just received. That was fixed on the *surface* rather than
/// in this test — `call_compute_natal` now reports the value, tropical included
/// as an explicit `0.0` — and the case reverted to a plain [`compare`]. Recorded
/// because the temptation in that moment is to relax the comparison, and the
/// whole worth of this file is that it does not.
///
/// Derived sanity facts:
///   * ten bodies in, ten planet rows out; twelve house cusps.
///   * `config_summary` is `format!("Houses: {:?}, Zodiac: {}, Rulership: {:?}")`
///     over the resolved config, so the sidereal request must surface as
///     `Zodiac: Lahiri` — under the schema default it would read
///     `Zodiac: Tropical`, and that string is the cheapest proof the frame
///     argument survived the trip on both surfaces.
///   * `ayanamsha_value` must sit in [23.6, 23.7]: Lahiri is 23°51′ (23.853°)
///     at J2000, and this instant is 5313.06 d = 14.55 yr earlier, so at the
///     ~50.3″/yr general-precession rate the offset is 23.853 − 14.55 ×
///     0.01397 = 23.65°.
///   * each planet's emitted `sign_index` must be `floor(longitude / 30)`.
#[test]
fn compute_natal_chart_surfaces_agree_sidereal() {
    // 1985-06-15 10:30:00 UT.
    let jd =
        vedaksha_ephem_core::julian::calendar_to_jd(1985, 6, 15.0 + 10.0 / 24.0 + 30.0 / 1440.0);
    let out = vedaksha_wasm::compute_natal_chart(
        &serde_json::json!({
            "year": 1985, "month": 6, "day": 15, "hour": 10, "minute": 30, "second": 0,
            "latitude": 28.6, "longitude": 77.2,
            "ayanamsha": "Lahiri", "house_system": "Placidus",
            "bodies": ["Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter",
                       "Saturn", "MeanNode", "TrueNode", "TrueNodeOsculating"]
        })
        .to_string(),
    )
    .expect("valid input");
    compare(
        "natal-sidereal",
        "compute_natal_chart",
        serde_json::json!({
            "julian_day": jd, "latitude": 28.6, "longitude": 77.2,
            "ayanamsha": "Lahiri", "house_system": "Placidus"
        }),
        &out,
    );
    let wasm: Value = serde_json::from_str(&out).expect("wasm output is JSON");

    assert_eq!(
        wasm["config_summary"], "Houses: Placidus, Zodiac: Lahiri, Rulership: Traditional",
        "sanity: the sidereal frame must have reached the chart config"
    );
    let planets = wasm["planets"].as_array().expect("planets array");
    assert_eq!(planets.len(), 10, "ten bodies requested, ten rows back");
    assert_eq!(wasm["houses"]["cusps"].as_array().unwrap().len(), 12);
    let ayanamsha = wasm["ayanamsha_value"].as_f64().expect("a number");
    assert!(
        (23.6..23.7).contains(&ayanamsha),
        "sanity: Lahiri in mid-1985 is ≈23.65°, got {ayanamsha}"
    );
    for planet in planets {
        let lon = planet["longitude"].as_f64().unwrap();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected = (lon / 30.0).floor() as u64;
        assert_eq!(
            planet["sign_index"].as_u64().unwrap(),
            expected,
            "sanity: sign_index must be floor(longitude/30): {planet}"
        );
    }
}
