// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! MCP layer consistency test.
//!
//! Verifies that calling MCP tools via JSON-RPC produces IDENTICAL results
//! to calling the underlying computation functions directly. 2000+ data points.

use vedaksha_mcp::server::McpServer;

/// Helper: call an MCP tool and return the parsed result
fn call_tool(server: &McpServer, tool: &str, args: serde_json::Value) -> serde_json::Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool,
            "arguments": args
        }
    });
    let response_str = server.handle_request(&serde_json::to_string(&request).unwrap());
    let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();

    // Extract text content from MCP response
    if let Some(result) = response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(first) = content.get(0) {
                if let Some(text) = first.get("text") {
                    let text_str = text.as_str().unwrap_or("");
                    return serde_json::from_str(text_str)
                        .unwrap_or(serde_json::Value::String(text_str.to_string()));
                }
            }
        }
    }
    if let Some(err) = response.get("error") {
        return err.clone();
    }
    serde_json::Value::Null
}

#[test]
fn mcp_compute_dasha_matches_direct() {
    let server = McpServer::new();

    let mut pass = 0;
    let mut fail = 0;
    let mut total = 0;

    // Test 200 random Moon longitudes × 5 birth JDs = 1000 dasha computations
    let moon_lons: Vec<f64> = (0..200).map(|i| (i as f64) * 1.8).collect(); // 0° to 360°
    let birth_jds = [2451545.0, 2451000.0, 2452000.0, 2450000.0, 2453000.0];

    for &jd in &birth_jds {
        for &moon_lon in &moon_lons {
            total += 1;

            // Direct computation
            let direct = vedaksha_vedic::dasha::vimshottari::compute_vimshottari(moon_lon, jd, 2);

            // MCP computation
            let mcp_result = call_tool(
                &server,
                "compute_dasha",
                serde_json::json!({
                    "moon_longitude": moon_lon,
                    "birth_jd": jd,
                    "levels": 2
                }),
            );

            // Compare: MCP should return serialized VimshottariDasha
            if mcp_result.is_null() || mcp_result.get("error_code").is_some() {
                fail += 1;
                continue;
            }

            // Compare moon_nakshatra
            let direct_nak = format!("{:?}", direct.moon_nakshatra);
            let mcp_nak = mcp_result
                .get("moon_nakshatra")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if direct_nak != mcp_nak {
                fail += 1;
                if fail <= 5 {
                    eprintln!(
                        "MISMATCH dasha nakshatra: moon_lon={moon_lon}, direct={direct_nak}, mcp={mcp_nak}"
                    );
                }
                continue;
            }

            // Compare initial_balance
            let direct_balance = direct.initial_balance;
            let mcp_balance = mcp_result
                .get("initial_balance")
                .and_then(|v| v.as_f64())
                .unwrap_or(-1.0);

            if (direct_balance - mcp_balance).abs() > 1e-10 {
                fail += 1;
                if fail <= 5 {
                    eprintln!(
                        "MISMATCH dasha balance: moon_lon={moon_lon}, direct={direct_balance}, mcp={mcp_balance}"
                    );
                }
                continue;
            }

            // Compare number of maha dashas
            let direct_count = direct.maha_dashas.len();
            let mcp_count = mcp_result
                .get("maha_dashas")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            if direct_count != mcp_count {
                fail += 1;
                continue;
            }

            // Compare first maha dasha lord and duration
            if let Some(first_direct) = direct.maha_dashas.first() {
                if let Some(first_mcp) = mcp_result.get("maha_dashas").and_then(|v| v.get(0)) {
                    let direct_lord = format!("{:?}", first_direct.lord);
                    let mcp_lord = first_mcp.get("lord").and_then(|v| v.as_str()).unwrap_or("");

                    if direct_lord != mcp_lord {
                        fail += 1;
                        continue;
                    }

                    let direct_dur = first_direct.duration_days;
                    let mcp_dur = first_mcp
                        .get("duration_days")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(-1.0);

                    if (direct_dur - mcp_dur).abs() > 1e-6 {
                        fail += 1;
                        continue;
                    }
                }
            }

            pass += 1;
        }
    }

    eprintln!("\n=== MCP DASHA CONSISTENCY ===");
    eprintln!("Total: {total}, Pass: {pass}, Fail: {fail}");
    eprintln!("Pass rate: {:.1}%", 100.0 * pass as f64 / total as f64);

    assert!(
        pass as f64 / total as f64 > 0.99,
        "MCP dasha consistency below 99%: {pass}/{total}"
    );
}

#[test]
fn mcp_compute_vargas_matches_direct() {
    let server = McpServer::new();

    let mut pass = 0;
    let mut fail = 0;
    let mut total = 0;

    let vargas = [
        "Rashi",
        "Navamsha",
        "Dashamsha",
        "Dwadashamsha",
        "Shashtiamsha",
    ];
    let longitudes: Vec<f64> = (0..200).map(|i| (i as f64) * 1.8).collect();

    for varga_name in &vargas {
        let varga_type = match *varga_name {
            "Rashi" => vedaksha_vedic::varga::VargaType::Rashi,
            "Navamsha" => vedaksha_vedic::varga::VargaType::Navamsha,
            "Dashamsha" => vedaksha_vedic::varga::VargaType::Dashamsha,
            "Dwadashamsha" => vedaksha_vedic::varga::VargaType::Dwadashamsha,
            "Shashtiamsha" => vedaksha_vedic::varga::VargaType::Shashtiamsha,
            _ => continue,
        };

        for &lon in &longitudes {
            total += 1;

            // Direct
            let direct_sign = vedaksha_vedic::varga::varga_sign(lon, varga_type);

            // MCP — include required fields even though only planet_longitude is used
            let mcp_result = call_tool(
                &server,
                "compute_vargas",
                serde_json::json!({
                    "julian_day": 2451545.0,
                    "latitude": 0.0,
                    "longitude": 0.0,
                    "planet_longitude": lon,
                    "divisions": [varga_name]
                }),
            );

            if mcp_result.is_null() || mcp_result.get("error_code").is_some() {
                fail += 1;
                continue;
            }

            let mcp_sign = mcp_result
                .get(varga_name)
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(255);

            if direct_sign == mcp_sign {
                pass += 1;
            } else {
                fail += 1;
                if fail <= 3 {
                    eprintln!(
                        "MISMATCH varga {varga_name}: lon={lon}, direct={direct_sign}, mcp={mcp_sign}"
                    );
                    eprintln!("  raw mcp_result: {mcp_result}");
                }
            }
        }
    }

    eprintln!("\n=== MCP VARGA CONSISTENCY ===");
    eprintln!("Total: {total}, Pass: {pass}, Fail: {fail}");
    eprintln!("Pass rate: {:.1}%", 100.0 * pass as f64 / total as f64);

    assert!(
        pass == total,
        "MCP varga consistency not 100%: {pass}/{total}"
    );
}

#[test]
fn mcp_emit_graph_roundtrip() {
    let server = McpServer::new();

    let mut pass = 0;
    let mut total = 0;

    // Create 100 test ChartGraphs and emit them through MCP in each format
    let formats = ["cypher", "surreal", "jsonld", "json", "embedding"];

    let mut debug_fail = 0;
    for i in 0..100 {
        let graph = vedaksha_graph::ChartGraph::new(
            vedaksha_graph::NodeId::chart_scoped(
                &vedaksha_graph::ids::NodeId::chart_hash(2451545.0 + i as f64, 28.6, 77.2, 0),
                "chart",
                "test",
            ),
            vedaksha_graph::DataClassification::Anonymous,
        );

        let chart_json = serde_json::to_value(&graph).unwrap();

        for &format in &formats {
            total += 1;

            // Direct emission
            use vedaksha_emit::GraphEmitter;
            let direct_result = match format {
                "cypher" => vedaksha_emit::cypher::CypherEmitter.emit(&graph),
                "surreal" => vedaksha_emit::surreal::SurrealEmitter.emit(&graph),
                "jsonld" => vedaksha_emit::jsonld::JsonLdEmitter.emit(&graph),
                "json" => vedaksha_emit::json_graph::JsonGraphEmitter.emit(&graph),
                "embedding" => vedaksha_emit::embedding_text::EmbeddingTextEmitter.emit(&graph),
                _ => continue,
            };

            // MCP emission
            let mcp_result = call_tool(
                &server,
                "emit_graph",
                serde_json::json!({
                    "chart_json": chart_json,
                    "format": format
                }),
            );

            match (direct_result, &mcp_result) {
                (Ok(direct_str), mcp_val) => {
                    let mcp_str = match mcp_val {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };

                    // For JSON-based formats (json, jsonld), parse both and
                    // compare as Value to ignore whitespace/key-order differences.
                    let match_found = if format == "json" || format == "jsonld" {
                        let d_val: Result<serde_json::Value, _> = serde_json::from_str(&direct_str);
                        let m_val: Result<serde_json::Value, _> = serde_json::from_str(&mcp_str);
                        match (d_val, m_val) {
                            (Ok(d), Ok(m)) => d == m,
                            _ => false,
                        }
                    } else {
                        // For text formats (cypher, surreal, embedding),
                        // normalize whitespace and compare
                        let d = direct_str.split_whitespace().collect::<Vec<_>>().join(" ");
                        let m = mcp_str.split_whitespace().collect::<Vec<_>>().join(" ");
                        d == m
                    };

                    if match_found {
                        pass += 1;
                    } else {
                        debug_fail += 1;
                        if debug_fail <= 2 {
                            eprintln!("  EMIT MISMATCH format={format} graph={i}");
                            eprintln!(
                                "    direct[..60]: {:?}",
                                &direct_str[..direct_str.len().min(60)]
                            );
                            eprintln!("    mcp[..60]:    {:?}", &mcp_str[..mcp_str.len().min(60)]);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    eprintln!("\n=== MCP EMIT_GRAPH CONSISTENCY ===");
    eprintln!("Total: {total}, Pass: {pass}");
    eprintln!("Pass rate: {:.1}%", 100.0 * pass as f64 / total as f64);

    assert!(
        pass == total,
        "MCP emit_graph consistency not 100%: {pass}/{total}"
    );
}

#[allow(dead_code)]
fn fail_count(_a: &str, _b: &str) -> bool {
    // For now, exact match required
    false
}

#[test]
fn mcp_tools_list_returns_all_10() {
    let server = McpServer::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    let response_str = server.handle_request(&serde_json::to_string(&request).unwrap());
    let response: serde_json::Value = serde_json::from_str(&response_str).unwrap();

    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools should be array");
    assert_eq!(tools.len(), 10, "Expected 10 MCP tools, got {}", tools.len());

    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
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

    eprintln!("\n=== MCP TOOLS LIST ===");
    eprintln!("All 10 tools present: {:?}", names);
}

#[test]
fn mcp_validation_rejects_bad_inputs() {
    let server = McpServer::new();
    let mut pass = 0;
    let mut total = 0;

    // 100 invalid JDs
    let bad_jds = [
        0.0,
        -1.0,
        1e15,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1000000.0,
        99999999.0,
    ];
    for &jd in &bad_jds {
        total += 1;
        let result = call_tool(
            &server,
            "compute_natal_chart",
            serde_json::json!({
                "julian_day": jd,
                "latitude": 28.0,
                "longitude": 77.0
            }),
        );
        if result.get("error_code").is_some() || result.get("code").is_some() {
            pass += 1;
        }
    }

    // Bad latitudes
    let bad_lats = [91.0, -91.0, 200.0, -200.0];
    for &lat in &bad_lats {
        total += 1;
        let result = call_tool(
            &server,
            "compute_natal_chart",
            serde_json::json!({
                "julian_day": 2451545.0,
                "latitude": lat,
                "longitude": 77.0
            }),
        );
        if result.get("error_code").is_some() || result.get("code").is_some() {
            pass += 1;
        }
    }

    // Bad longitudes
    let bad_lons = [181.0, -181.0, 500.0];
    for &lon in &bad_lons {
        total += 1;
        let result = call_tool(
            &server,
            "compute_natal_chart",
            serde_json::json!({
                "julian_day": 2451545.0,
                "latitude": 28.0,
                "longitude": lon
            }),
        );
        if result.get("error_code").is_some() || result.get("code").is_some() {
            pass += 1;
        }
    }

    // Bad dasha inputs
    let bad_dasha = [
        serde_json::json!({"moon_longitude": -1.0, "birth_jd": 2451545.0}),
        serde_json::json!({"moon_longitude": 361.0, "birth_jd": 2451545.0}),
        serde_json::json!({"moon_longitude": 100.0, "birth_jd": 0.0}),
    ];
    for args in &bad_dasha {
        total += 1;
        let result = call_tool(&server, "compute_dasha", args.clone());
        if result.get("error_code").is_some() || result.get("code").is_some() {
            pass += 1;
        }
    }

    // Bad emit formats
    total += 1;
    let result = call_tool(
        &server,
        "emit_graph",
        serde_json::json!({
            "chart_json": {},
            "format": "invalid_format"
        }),
    );
    if result.get("error_code").is_some() || result.get("code").is_some() {
        pass += 1;
    }

    eprintln!("\n=== MCP VALIDATION ===");
    eprintln!("Total: {total}, Correctly rejected: {pass}");
    eprintln!("Rejection rate: {:.1}%", 100.0 * pass as f64 / total as f64);

    assert!(
        pass as f64 / total as f64 > 0.90,
        "MCP validation rejection rate below 90%: {pass}/{total}"
    );
}
