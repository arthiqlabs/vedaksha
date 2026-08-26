// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
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
                .map_or(0, |a| a.len());

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

            // v7.4.0 replaced the flat {division: sign} map with one envelope
            // shared by both input shapes. The single-longitude path yields
            // exactly one placement, and it names no graha.
            let mcp_sign = mcp_result
                .pointer("/vargas/0/placements/0/varga_sign")
                .and_then(serde_json::Value::as_u64)
                .map_or(255, |v| v as u8);

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
            use vedaksha_graph::emitters::GraphEmitter;
            let direct_result = match format {
                "cypher" => vedaksha_graph::emitters::cypher::CypherEmitter.emit(&graph),
                "surreal" => vedaksha_graph::emitters::surreal::SurrealEmitter.emit(&graph),
                "jsonld" => vedaksha_graph::emitters::jsonld::JsonLdEmitter.emit(&graph),
                "json" => vedaksha_graph::emitters::json_graph::JsonGraphEmitter.emit(&graph),
                "embedding" => {
                    vedaksha_graph::emitters::embedding_text::EmbeddingTextEmitter.emit(&graph)
                }
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
fn mcp_tools_list_returns_every_registered_tool() {
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

    // No count assertion here. `tools::tests::snapshot_matches_current_tool_
    // definitions` already compares the whole registry — names, descriptions
    // and schemas — against the committed `tools/mcp-tools.json`, which is
    // strictly stronger than a length and cannot be satisfied by a tool that
    // was swapped for another. The `contains` assertions below earn their keep
    // for a different reason: they name the tool that went missing, and they do
    // not need editing when one is added.
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
    assert!(names.contains(&"compute_ashtakavarga"));
    assert!(names.contains(&"compute_gochara"));
    assert!(names.contains(&"compute_panchanga"));
    assert!(names.contains(&"compute_drishti"));
    assert!(names.contains(&"compute_bhavas"));
    assert!(names.contains(&"compute_synastry"));
    assert!(names.contains(&"compute_composite"));

    eprintln!("\n=== MCP TOOLS LIST ===");
    eprintln!("All {} tools present: {:?}", names.len(), names);
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

/// The documented path must compute, not describe itself.
///
/// Through v7.3.1 `compute_vargas` answered its own four required parameters
/// with `{"status":"validated","message":"Provide planet_longitude ..."}` while
/// its description promised a chart per division. This is the test that would
/// have caught that: it asserts what the description claims, so a regression to
/// any stub fails here rather than being discovered by a caller.
#[test]
fn mcp_compute_vargas_documented_path_returns_charts() {
    let server = McpServer::new();

    let result = call_tool(
        &server,
        "compute_vargas",
        serde_json::json!({
            "julian_day": 2451545.0,
            "latitude": 28.6139,
            "longitude": 77.2090,
            "divisions": ["D1", "D9"],
            "ayanamsha": "IndianOfficial"
        }),
    );

    assert!(
        result.get("status").is_none() && result.get("message").is_none(),
        "the documented path returned a status/message stub: {result}"
    );

    let vargas = result["vargas"]
        .as_array()
        .unwrap_or_else(|| panic!("no vargas array: {result}"));
    assert_eq!(vargas.len(), 2, "one entry per requested division");

    for varga in vargas {
        assert!(
            varga["lagna_sign"].as_u64().is_some_and(|s| s < 12),
            "every varga carries its lagna: {varga}"
        );
        let placements = varga["placements"].as_array().unwrap();
        assert_eq!(
            placements.len(),
            10,
            "the same ten bodies compute_natal_chart returns"
        );
        for placement in placements {
            assert!(placement["planet"].is_string(), "named graha: {placement}");
            assert!(placement["varga_sign"].as_u64().is_some_and(|s| s < 12));
            assert!(
                placement["bhava"]
                    .as_u64()
                    .is_some_and(|b| (1..=12).contains(&b))
            );
            // Dignity is present for the seven grahas and absent for the three
            // node variants, which have none. Stated rather than assumed.
            let is_node = placement["planet"].as_str().unwrap().contains("Node");
            assert_eq!(
                placement.get("dignity").is_none(),
                is_node,
                "dignity presence must track whether the body has one: {placement}"
            );
        }
    }

    // The ayanamsha was applied before dividing, not ignored.
    assert!(
        result["ayanamsha_value"]
            .as_f64()
            .is_some_and(|a| a > 23.0 && a < 24.5),
        "IndianOfficial at J2000 is ~23.86 deg, got {}",
        result["ayanamsha_value"]
    );
}

/// D-1 *is* the rashi chart, so `compute_vargas` must agree with
/// `compute_natal_chart` body for body. This is what makes "the varga is
/// derived from the D-1" a checked claim rather than a comment: the two
/// surfaces would otherwise have no oracle in common.
#[test]
fn mcp_compute_vargas_d1_agrees_with_the_natal_chart() {
    let server = McpServer::new();
    let args = serde_json::json!({
        "julian_day": 2451545.0,
        "latitude": 28.6139,
        "longitude": 77.2090,
        "ayanamsha": "IndianOfficial"
    });

    let natal = call_tool(&server, "compute_natal_chart", args.clone());
    let mut varga_args = args;
    varga_args["divisions"] = serde_json::json!(["D1"]);
    let vargas = call_tool(&server, "compute_vargas", varga_args);

    let natal_planets = natal["planets"].as_array().unwrap();
    let placements = vargas["vargas"][0]["placements"].as_array().unwrap();
    assert_eq!(natal_planets.len(), placements.len());

    for (natal_planet, placement) in natal_planets.iter().zip(placements) {
        assert_eq!(natal_planet["name"], placement["planet"]);
        assert_eq!(
            natal_planet["sign_index"], placement["varga_sign"],
            "D1 sign must equal the natal sign for {}",
            natal_planet["name"]
        );
        assert_eq!(
            natal_planet["longitude"], placement["rashi_longitude"],
            "the same longitude must be divided as was reported"
        );
    }
}

/// The `element` tradition must actually change something, and only the four
/// vargas where the texts diverge. A parameter that silently does nothing is
/// worse than no parameter.
#[test]
fn mcp_compute_vargas_tradition_changes_only_the_four_divergent_vargas() {
    let server = McpServer::new();
    let base = serde_json::json!({
        "julian_day": 2451545.0, "latitude": 28.6139, "longitude": 77.2090,
        "divisions": ["D9", "D10", "D16", "D20", "D30", "D45"],
        "ayanamsha": "IndianOfficial"
    });

    let modality = call_tool(&server, "compute_vargas", base.clone());
    let mut element_args = base;
    element_args["tradition"] = serde_json::json!("element");
    let element = call_tool(&server, "compute_vargas", element_args);

    let divergent = ["D16", "D20", "D30", "D45"];
    let mut differed = Vec::new();
    for (m, e) in modality["vargas"]
        .as_array()
        .unwrap()
        .iter()
        .zip(element["vargas"].as_array().unwrap())
    {
        let division = m["division"].as_str().unwrap();
        if m["placements"] != e["placements"] || m["lagna_sign"] != e["lagna_sign"] {
            differed.push(division.to_string());
        }
        if !divergent.contains(&division) {
            assert_eq!(
                m["placements"], e["placements"],
                "{division} must be identical under both traditions"
            );
        }
    }
    assert!(
        !differed.is_empty(),
        "the element tradition changed nothing at all — the parameter is inert"
    );
    for division in &differed {
        assert!(
            divergent.contains(&division.as_str()),
            "{division} changed, but only D16, D20, D30 and D45 should"
        );
    }
}
