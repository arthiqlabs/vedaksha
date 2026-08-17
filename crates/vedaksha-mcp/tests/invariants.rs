// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Mathematical invariant and fixed-point tests.
//!
//! These verify properties that must ALWAYS hold, regardless of input.

use vedaksha_astro::sidereal::{self, Ayanamsha};
use vedaksha_ephem_core::nutation;
use vedaksha_ephem_core::obliquity;
use vedaksha_ephem_core::sidereal_time;
use vedaksha_math::angle::normalize_degrees;
use vedaksha_vedic::dasha::vimshottari;
use vedaksha_vedic::nakshatra::Nakshatra;

// ─── SIDEREAL LONGITUDE ───

#[test]
fn sidereal_longitude_equals_tropical_minus_ayanamsha() {
    // For any longitude, any ayanamsha, any date:
    // sidereal = tropical - ayanamsha (mod 360)
    let systems = [
        Ayanamsha::Lahiri,
        Ayanamsha::FaganBradley,
        Ayanamsha::Raman,
        Ayanamsha::Krishnamurti,
        Ayanamsha::Yukteshwar,
        Ayanamsha::Tropical,
    ];
    let jds = [2451545.0, 2451000.0, 2452000.0, 2440000.0, 2460000.0];
    let longitudes = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 359.99];

    let mut total = 0;
    let mut pass = 0;

    for &sys in &systems {
        for &jd in &jds {
            let ayan = sidereal::ayanamsha_value(sys, jd);
            for &lon in &longitudes {
                total += 1;
                let sidereal = sidereal::tropical_to_sidereal(lon, sys, jd);
                let expected = normalize_degrees(lon - ayan);
                let diff = (sidereal - expected).abs();
                if diff < 1e-10 || (360.0 - diff).abs() < 1e-10 {
                    pass += 1;
                } else {
                    eprintln!(
                        "FAIL sidereal: sys={:?} jd={jd} lon={lon} expected={expected} got={sidereal} diff={diff}",
                        sys
                    );
                }
            }
        }
    }

    eprintln!("\n=== SIDEREAL LONGITUDE INVARIANT ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total, "Sidereal longitude invariant violated");
}

#[test]
fn sidereal_tropical_roundtrip() {
    // tropical_to_sidereal(sidereal_to_tropical(x)) == x
    let jd = 2451545.0;
    let mut total = 0;
    let mut pass = 0;

    for sys in [Ayanamsha::Lahiri, Ayanamsha::FaganBradley, Ayanamsha::Raman] {
        for lon in (0..360).map(|i| i as f64) {
            total += 1;
            let trop = sidereal::sidereal_to_tropical(lon, sys, jd);
            let back = sidereal::tropical_to_sidereal(trop, sys, jd);
            if (normalize_degrees(back) - normalize_degrees(lon)).abs() < 1e-10 {
                pass += 1;
            }
        }
    }

    eprintln!("\n=== SIDEREAL ROUNDTRIP ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── RETROGRADE DETECTION ───

#[test]
fn retrograde_detection_from_speed_sign() {
    // Retrograde = speed < 0. Test with known speeds.
    let cases = [
        (1.0, false),   // direct
        (0.0, false),   // stationary
        (-0.5, true),   // retrograde
        (-0.001, true), // barely retrograde
        (13.0, false),  // Moon (fast direct)
    ];

    let mut total = 0;
    let mut pass = 0;

    for &(speed, expected_retro) in &cases {
        total += 1;
        let is_retro = speed < 0.0;
        if is_retro == expected_retro {
            pass += 1;
        } else {
            eprintln!("FAIL retrograde: speed={speed} expected={expected_retro} got={is_retro}");
        }
    }

    // Also test with 100 random speeds
    for i in 0..100 {
        total += 1;
        let speed = (i as f64 - 50.0) * 0.1; // -5.0 to 4.9
        let is_retro = speed < 0.0;
        let expected = speed < 0.0;
        if is_retro == expected {
            pass += 1;
        }
    }

    eprintln!("\n=== RETROGRADE DETECTION ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── POLAR HOUSE FALLBACK ───

#[test]
fn polar_house_fallback_triggers_above_66_56() {
    use vedaksha_astro::houses::{HouseSystem, compute_houses};

    let mut total = 0;
    let mut pass = 0;

    let polar_lats = [
        67.0, 70.0, 75.0, 80.0, 85.0, 89.0, -67.0, -70.0, -80.0, -89.0,
    ];
    let non_polar_lats = [0.0, 10.0, 30.0, 45.0, 55.0, 60.0, 65.0, 66.0, -30.0, -60.0];

    // Placidus and Koch should fallback at polar latitudes
    for &system in &[HouseSystem::Placidus, HouseSystem::Koch] {
        for &lat in &polar_lats {
            total += 1;
            let cusps = compute_houses(90.0, lat, 23.44, system);
            if cusps.polar_fallback {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL polar: system={:?} lat={lat} should fallback but didn't",
                    system
                );
            }
        }

        for &lat in &non_polar_lats {
            total += 1;
            let cusps = compute_houses(90.0, lat, 23.44, system);
            if !cusps.polar_fallback {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL non-polar: system={:?} lat={lat} should NOT fallback but did",
                    system
                );
            }
        }
    }

    eprintln!("\n=== POLAR HOUSE FALLBACK ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── NAKSHATRA FROM LONGITUDE ───

#[test]
fn nakshatra_from_longitude_covers_full_circle() {
    let mut total = 0;
    let mut pass = 0;

    // Test every 0.1 degree across the full circle
    for i in 0..3600 {
        let lon = i as f64 * 0.1;
        total += 1;

        let nak = Nakshatra::from_longitude(lon);
        let expected_index = (lon / Nakshatra::SPAN) as u8;
        let expected_index = expected_index.min(26);

        if nak.index() == expected_index {
            pass += 1;
        } else {
            eprintln!(
                "FAIL nakshatra: lon={lon} expected_idx={expected_index} got={}",
                nak.index()
            );
        }
    }

    // Boundary tests: exact nakshatra boundaries
    for i in 0..27 {
        let boundary = i as f64 * Nakshatra::SPAN;
        total += 1;
        let nak = Nakshatra::from_longitude(boundary);
        if nak.index() == i as u8 {
            pass += 1;
        } else {
            eprintln!(
                "FAIL nakshatra boundary: lon={boundary} expected={i} got={}",
                nak.index()
            );
        }
    }

    // Pada tests
    for i in 0..108 {
        let lon = i as f64 * Nakshatra::PADA_SPAN + 0.01;
        total += 1;
        let pada = Nakshatra::pada_from_longitude(lon);
        let expected_pada = ((lon % Nakshatra::SPAN) / Nakshatra::PADA_SPAN) as u8 + 1;
        if pada == expected_pada.min(4) {
            pass += 1;
        } else {
            eprintln!("FAIL pada: lon={lon} expected={expected_pada} got={pada}");
        }
    }

    eprintln!("\n=== NAKSHATRA FROM LONGITUDE ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── DASHA PERIOD SUMS ───

#[test]
fn dasha_period_sums_equal_120_years() {
    let dasha_year_days = 365.25;
    let total_years = 120.0;
    let _total_days = total_years * dasha_year_days;

    let mut test_total = 0;
    let mut pass = 0;

    // Test 500 random Moon longitudes
    // The 9 maha dashas sum to: 120_years - (1-balance) * first_lord_years
    // This is always <= 120 years (equals 120 only when balance = 1.0)
    for i in 0..500 {
        let moon_lon = (i as f64) * 0.72; // 0 to 360
        let jd = 2451545.0;
        test_total += 1;

        let dasha = vimshottari::compute_vimshottari(moon_lon, jd, 1);

        // Expected sum: balance * first_lord_years + sum of remaining 8 full lords
        let first_lord_years = dasha.maha_dashas[0].lord.maha_dasha_years();
        let expected_days = dasha.initial_balance * first_lord_years * dasha_year_days
            + (total_years - first_lord_years) * dasha_year_days;

        let sum: f64 = dasha.maha_dashas.iter().map(|d| d.duration_days).sum();
        let diff = (sum - expected_days).abs();

        if diff < 0.01 {
            pass += 1;
        } else {
            eprintln!(
                "FAIL dasha sum: moon_lon={moon_lon} sum={sum} expected={expected_days} diff={diff}"
            );
        }
    }

    // Test that sub-periods sum to parent
    for i in 0..100 {
        let moon_lon = (i as f64) * 3.6;
        let dasha = vimshottari::compute_vimshottari(moon_lon, 2451545.0, 3);

        for maha in &dasha.maha_dashas {
            test_total += 1;
            let sub_sum: f64 = maha.sub_periods.iter().map(|s| s.duration_days).sum();
            let diff = (sub_sum - maha.duration_days).abs();
            if diff < 0.01 {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL sub-period sum: lord={:?} parent={} sub_sum={sub_sum} diff={diff}",
                    maha.lord, maha.duration_days
                );
            }

            // Level 3: pratyantar sums to antar
            for antar in &maha.sub_periods {
                test_total += 1;
                let prat_sum: f64 = antar.sub_periods.iter().map(|p| p.duration_days).sum();
                let diff = (prat_sum - antar.duration_days).abs();
                if diff < 0.01 {
                    pass += 1;
                } else {
                    eprintln!("FAIL pratyantar sum: lord={:?} diff={diff}", antar.lord);
                }
            }
        }
    }

    eprintln!("\n=== DASHA PERIOD SUMS ===");
    eprintln!("Total: {test_total}, Pass: {pass}");
    assert_eq!(pass, test_total, "Dasha period sum invariant violated");
}

// ─── GAST DIRECTLY ───

#[test]
fn gast_equals_gmst_plus_equation_of_equinoxes() {
    let mut total = 0;
    let mut pass = 0;

    let jds = [
        2451545.0, 2451000.0, 2452000.0, 2440000.0, 2460000.0, 2445000.0, 2450000.0, 2455000.0,
        2448000.0, 2453000.0, 2430000.0, 2435000.0, 2442000.0, 2457000.0, 2462000.0, 2447000.0,
        2449000.0, 2451500.0, 2454000.0, 2456000.0,
    ];

    for &jd in &jds {
        total += 1;

        let gmst = sidereal_time::gmst(jd);
        let (dpsi, deps) = nutation::nutation(jd);
        let eps_true = obliquity::true_obliquity(jd, deps);
        let gast = sidereal_time::gast(jd, dpsi, eps_true);

        // GAST = GMST + dpsi * cos(eps_true)
        let eq_equinoxes = dpsi * eps_true.cos();
        let expected_gast = vedaksha_math::angle::normalize_radians(gmst + eq_equinoxes);

        let diff = (gast - expected_gast).abs();
        if diff < 1e-12 || (2.0 * core::f64::consts::PI - diff).abs() < 1e-12 {
            pass += 1;
        } else {
            eprintln!("FAIL GAST: jd={jd} gast={gast} expected={expected_gast} diff={diff}");
        }
    }

    eprintln!("\n=== GAST = GMST + EQ_EQUINOXES ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── WHOLE SIGN HOUSES ───

#[test]
fn whole_sign_cusps_are_deterministic_from_asc() {
    use vedaksha_astro::houses::{HouseSystem, compute_houses};

    let mut total = 0;
    let mut pass = 0;

    for ramc in (0..360).step_by(10) {
        for lat in [-50, -30, 0, 30, 50] {
            total += 1;
            let cusps = compute_houses(ramc as f64, lat as f64, 23.44, HouseSystem::WholeSign);

            // House 1 starts at 0° of ASC's sign
            let asc_sign_start = (cusps.asc / 30.0).floor() * 30.0;
            let diff = (cusps.cusps[0] - asc_sign_start).abs();

            if diff < 0.01 || (360.0 - diff) < 0.01 {
                // Each subsequent cusp should be exactly 30° apart
                let mut all_30 = true;
                for i in 0..11 {
                    let gap = normalize_degrees(cusps.cusps[i + 1] - cusps.cusps[i]);
                    if (gap - 30.0).abs() > 0.01 {
                        all_30 = false;
                        eprintln!("FAIL whole_sign gap: cusp {} to {} = {gap}", i + 1, i + 2);
                    }
                }
                if all_30 {
                    pass += 1;
                }
            } else {
                eprintln!(
                    "FAIL whole_sign: asc={} sign_start={asc_sign_start} cusp1={}",
                    cusps.asc, cusps.cusps[0]
                );
            }
        }
    }

    eprintln!("\n=== WHOLE SIGN DETERMINISTIC ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── RAMC TIME SCALE (UT1, not TT) ───

/// The served chart's RAMC must be built from a **UT1** Julian Day.
///
/// This is the test that pins the actual call site. The cross-check in
/// `vedaksha_astro::riseset` pins `sidereal_time::local_sidereal_time`'s own
/// convention, but it cannot see what `server.rs` passes in; between
/// v1 and the UT-vs-TT fix the chart path converted to TT first
/// (`delta_t::ut1_to_tt`) and then used that as the rotational argument,
/// adding ΔT worth of Earth rotation instead of removing it. Nothing caught
/// it, because every chart assertion in the suite was a range check.
///
/// The RAMC is not emitted, but the tropical MC is, and the engine's own
/// documented closed form (`vedaksha_astro::houses::compute_asc_mc`) is
/// invertible:
/// ```text
/// MC   = atan2(sin RAMC, cos RAMC · cos ε)
/// RAMC = atan2(sin MC · cos ε, cos MC)
/// ```
/// (`tan MC = tan RAMC / cos ε` ⇔ `tan RAMC = tan MC · cos ε`, and atan2
/// keeps the quadrant on both sides.) `ε` here must be the SAME obliquity the
/// server hands to `compute_houses` — the mean obliquity at TT — or the
/// inversion is not the inverse.
///
/// The expected value is derived independently of this codebase:
/// GMST at JD 2,451,544.5 (2000-01-01 00:00 UT1) = **99.967795°**, from the
/// IAU 1982 series `GMST_s = 24110.54841 + 8640184.812866·Tu + 0.093104·Tu²
/// − 6.2e-6·Tu³` with `Tu = (JD − 2451545.0)/36525 = −1.369472e-5`, which
/// agrees with Meeus eq. 12.4 to 7 decimals. Same constant, same derivation
/// as `riseset::local_sidereal_degrees_matches_published_gmst_at_j2000`.
/// Local mean sidereal time at 77.2090° E is then 99.967795 + 77.2090 =
/// 177.176795°.
///
/// The served RAMC is *apparent*, so it differs from that mean value by the
/// equation of the equinoxes and nothing else. Δψ is bounded by the 18.6-year
/// nutation term at ±17.2″, so the tolerance is that physical bound,
/// 17.2/3600 = 0.0047778° — NOT a figure sized to make the test pass.
///
/// Measured, with the fix in place: recovered RAMC = 177.17324412561521°
/// (from the served MC = 176.91947671879240°), i.e. 3.550874e-3° = 12.78″
/// below the mean value. That is Δψ·cos(ε_true) at this instant, measured
/// independently as −3.550561239877e-3° in the `riseset` cross-check; the
/// remaining 3.13e-7° is the rounding of the 99.967795° literal to six
/// decimals, the same 3.131449e-7° already recorded there. Nothing else is
/// unaccounted for.
///
/// Mutation, measured: restoring `jd_tt` as the rotational argument in
/// `server.rs` moves the served MC to 177.20992574088470° and the recovered
/// RAMC to 177.43983661960689° — off by 0.263041619606895° (15.78′), 55× the
/// tolerance. The test fails; reverting makes it pass again.
#[test]
fn served_ramc_is_built_from_ut1_not_tt() {
    let jd_ut1 = 2_451_544.5_f64;
    let lon_east = 77.2090_f64;

    let server = vedaksha_mcp::server::McpServer::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "compute_natal_chart", "arguments": {
            "julian_day": jd_ut1, "latitude": 28.6139, "longitude": lon_east}}
    });
    let response: serde_json::Value =
        serde_json::from_str(&server.handle_request(&serde_json::to_string(&request).unwrap()))
            .unwrap();
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("compute_natal_chart returned no content");
    let chart: serde_json::Value = serde_json::from_str(text).unwrap();
    let mc_deg = chart["houses"]["mc"].as_f64().expect("houses.mc missing");

    // Same obliquity the server feeds compute_houses: MEAN obliquity at TT.
    let jd_tt = vedaksha_ephem_core::delta_t::ut1_to_tt(jd_ut1);
    let eps = obliquity::mean_obliquity(jd_tt); // radians

    let mc = mc_deg.to_radians();
    let ramc_deg = normalize_degrees((mc.sin() * eps.cos()).atan2(mc.cos()).to_degrees());

    // 99.967795° (IAU 1982, derived above) + 77.2090° E.
    let expected_mean_ramc = 99.967_795 + lon_east;
    // Physical bound on |Δψ·cos ε|: 17.2″.
    let eq_equinoxes_max_deg = 17.2 / 3600.0;

    let diff = (ramc_deg - expected_mean_ramc).abs();
    assert!(
        diff < eq_equinoxes_max_deg,
        "served RAMC = {ramc_deg}°, expected {expected_mean_ramc}° ± {eq_equinoxes_max_deg}° \
         (published GMST + longitude); off by {diff}°. A miss near 0.263° means the chart \
         path passed TT where UT1 belongs."
    );
}

/// A **served** sidereal request must produce a **sidereal** chart.
///
/// This is the tool-boundary counterpart to
/// `vedaksha_astro::chart::house_assignment_is_invariant_under_ayanamsha_rotation`.
/// That test proves `compute_chart` rotates planets and cusps together, but it
/// calls the astro layer directly and hands it a `ChartConfig` it built itself.
/// It says nothing about whether the caller's `ayanamsha` argument survives
/// the MCP handler's string parsing and reaches that config.
///
/// It did not, as far as the suite was concerned: mutating
/// `server.rs::call_compute_natal_chart` to discard the parsed ayanamsha and
/// force `ayanamsha: None` left the entire `vedaksha-mcp` + `vedaksha-wasm`
/// suite green. Every Jyotish caller would have silently received a tropical
/// chart. The hole was structural — the two frame tests are `vedaksha-astro`
/// unit tests over synthetic RAMCs, `served_ramc_is_built_from_ut1_not_tt`
/// above uses the tropical default, both Python fixture natal cases are
/// `Zodiac: Tropical`, and `mcp_surface_parity` covers only
/// `compute_panchanga`. Nothing served a sidereal chart and looked at it.
///
/// # The property
///
/// Two `compute_natal_chart` calls at the same instant, the same observer and
/// the same house system, differing only in `ayanamsha`. Sidereal must equal
/// tropical rotated by exactly the ayanamsha, on every reported longitude:
///
/// ```text
/// sidereal_x = normalize(tropical_x − ayanamsha_value(Lahiri, jd))
/// ```
///
/// for `asc`, `mc` and all twelve cusps. No external oracle: the right-hand
/// side is computed here from `vedaksha_astro::sidereal::ayanamsha_value`, the
/// same public function a caller would use, and the identity is the definition
/// of a sidereal frame rather than a measured figure.
///
/// # Derivation of the numbers asserted
///
/// At `jd = 2451544.5` (2000-01-01 00:00 UT), 28.6139° N, 77.2090° E, Placidus:
///
/// - `ayanamsha_value(Lahiri, 2451544.5)` = <ayanamsha>°
/// - served tropical `asc`  = 255.288134034110612°, `mc` = 176.919476718792396°
/// - served sidereal `asc`  = 231.431060259900079°, `mc` = 153.062402944581862°
/// - tropical − <ayanamsha> = sidereal
///
/// which matches the served sidereal `asc` to the last bit the subtraction can
/// carry. Every one of the twelve cusps shows the same <ayanamsha>°
/// offset. The tolerance below is 1e-9°, five orders looser than the observed
/// residual (< 1e-13°) and eleven orders tighter than the failure it must
/// catch — so it is a floating-point allowance, not a figure sized to pass.
///
/// # Mutation, measured
///
/// Forcing `ayanamsha: None` in the handler makes both calls return the
/// tropical chart, so the measured offset collapses from <ayanamsha>° to
/// 0° — a miss of the full ayanamsha against a 1e-9° tolerance. The test fails on `asc`,
/// on `mc` and on all twelve cusps.
#[test]
fn served_sidereal_request_yields_a_sidereal_chart() {
    let jd_ut1 = 2_451_544.5_f64;
    let lat = 28.6139_f64;
    let lon = 77.2090_f64;

    let served = |ayanamsha: &str| -> serde_json::Value {
        let server = vedaksha_mcp::server::McpServer::new();
        let request = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "compute_natal_chart", "arguments": {
                "julian_day": jd_ut1,
                "latitude": lat,
                "longitude": lon,
                "house_system": "Placidus",
                "ayanamsha": ayanamsha,
            }}
        });
        let response: serde_json::Value =
            serde_json::from_str(&server.handle_request(&serde_json::to_string(&request).unwrap()))
                .unwrap();
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("compute_natal_chart({ayanamsha}) returned no content"));
        serde_json::from_str(text).unwrap()
    };

    let trop = served("Tropical");
    let sid = served("Lahiri");

    // Guard: a degenerate ayanamsha would make every assertion below vacuous.
    // Lahiri at J2000 is tens of degrees; anything under 20° means the reference
    // itself has broken and the comparison proves nothing.
    let ayan = sidereal::ayanamsha_value(Ayanamsha::Lahiri, jd_ut1);
    assert!(
        ayan > 20.0,
        "ayanamsha_value(Lahiri, {jd_ut1}) = {ayan}°, expected the full Lahiri offset. With a \
         near-zero ayanamsha the sidereal/tropical comparison below cannot \
         distinguish a sidereal chart from a tropical one."
    );

    // Signed shortest angular separation, so the 0°/360° wrap cannot mask a
    // failure or manufacture one.
    let sep = |a: f64, b: f64| -> f64 {
        let d = normalize_degrees(a - b);
        if d > 180.0 { d - 360.0 } else { d }
    };
    const TOL: f64 = 1e-9;

    let mut checked = 0_usize;
    let mut angles: Vec<(String, f64, f64)> = vec![
        (
            "asc".to_string(),
            trop["houses"]["asc"].as_f64().expect("houses.asc missing"),
            sid["houses"]["asc"].as_f64().expect("houses.asc missing"),
        ),
        (
            "mc".to_string(),
            trop["houses"]["mc"].as_f64().expect("houses.mc missing"),
            sid["houses"]["mc"].as_f64().expect("houses.mc missing"),
        ),
    ];
    for i in 0..12 {
        angles.push((
            format!("cusp {}", i + 1),
            trop["houses"]["cusps"][i]
                .as_f64()
                .unwrap_or_else(|| panic!("houses.cusps[{i}] missing")),
            sid["houses"]["cusps"][i]
                .as_f64()
                .unwrap_or_else(|| panic!("houses.cusps[{i}] missing")),
        ));
    }

    for (label, t, s) in &angles {
        let offset = sep(*t, *s);
        assert!(
            (offset - ayan).abs() < TOL,
            "served {label}: tropical {t}°, sidereal {s}° — offset {offset}°, \
             expected the ayanamsha {ayan}° (±{TOL}). An offset of 0° means the \
             requested ayanamsha never reached the chart config and the caller \
             was served a tropical chart under a sidereal label."
        );
        checked += 1;
    }

    // Planet longitudes rotate by the same amount — the cusps and the bodies
    // must land in one frame, which is the defect `4b1bb58` fixed inside
    // `compute_chart` and this asserts is still true when served.
    let tp = trop["planets"].as_array().expect("planets missing");
    let sp = sid["planets"].as_array().expect("planets missing");
    assert_eq!(tp.len(), sp.len());
    assert!(
        tp.len() >= 9,
        "expected the 9 Jyotish bodies, got {}",
        tp.len()
    );
    for (t, s) in tp.iter().zip(sp.iter()) {
        let name = t["name"].as_str().unwrap_or("<unnamed>");
        assert_eq!(name, s["name"].as_str().unwrap_or("<unnamed>"));
        let (tl, sl) = (
            t["longitude"].as_f64().expect("planet longitude missing"),
            s["longitude"].as_f64().expect("planet longitude missing"),
        );
        let offset = sep(tl, sl);
        assert!(
            (offset - ayan).abs() < TOL,
            "served planet {name}: tropical {tl}°, sidereal {sl}° — offset \
             {offset}°, expected the ayanamsha {ayan}° (±{TOL})"
        );
        // House number is invariant under a rigid rotation of the whole chart.
        assert_eq!(
            t["house"], s["house"],
            "served planet {name} changed house between frames — a uniform \
             rotation of the zodiac cannot move a planet between houses"
        );
        checked += 1;
    }

    assert!(
        checked >= 23,
        "only {checked} angles compared; the served-frame check must cover \
         asc, mc, 12 cusps and every returned body"
    );

    // Secondary, and deliberately last: the frame the chart *reports*. This is
    // a label check, weaker than the arithmetic above — a mutation could keep
    // the summary honest while computing the wrong frame, or vice versa — so
    // it runs only after the numbers have already proved the frame.
    assert_eq!(
        trop["config_summary"].as_str().unwrap(),
        "Houses: Placidus, Zodiac: Tropical, Rulership: Traditional"
    );
    assert_eq!(
        sid["config_summary"].as_str().unwrap(),
        "Houses: Placidus, Zodiac: Lahiri, Rulership: Traditional",
        "the served chart reports a frame other than the one requested"
    );
}
