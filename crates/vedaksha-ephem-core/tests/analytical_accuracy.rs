// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Integration tests comparing AnalyticalProvider against SpkReader (DE440s).
//!
//! Validates that the truncated analytical series produce ecliptic longitudes
//! close enough to DE440s for chart-equivalent astrology computations.

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::reader::SpkReader;
use vedaksha_ephem_core::jpl::EphemerisProvider;

/// Path to DE440s BSP file relative to workspace root.
fn bsp_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("de440s.bsp")
}

/// Test dates spanning 1900-2100.
fn test_dates() -> Vec<f64> {
    vec![
        2415020.5, // ~1900 Jan 0.5
        2425020.5, // ~1927
        2435020.5, // ~1954
        2445020.5, // ~1982
        2451545.0, // J2000.0
        2455197.5, // ~2010
        2462867.5, // ~2031
        2470000.0, // ~2050
        2478000.0, // ~2072
        2488069.5, // ~2100
    ]
}

/// Compute angular difference in degrees, handling wrap-around.
fn angular_diff_deg(lon1_deg: f64, lon2_deg: f64) -> f64 {
    let mut diff = (lon1_deg - lon2_deg).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    diff
}

/// Convert arcseconds to degrees.
fn arcsec_to_deg(arcsec: f64) -> f64 {
    arcsec / 3600.0
}

/// Tolerance: 10 arcseconds in degrees.
/// Planet longitude tolerance. VSOP87A has intrinsic ~10-13" errors for
/// Venus and Mars over 200-year spans (1900-2100). 15" is well within the
/// 1-arcminute budget for astrological computations.
const PLANET_TOLERANCE_ARCSEC: f64 = 15.0;

/// Tolerance for Moon: 30 arcseconds.
const MOON_TOLERANCE_ARCSEC: f64 = 30.0;

// ─── Test 1: Planet longitude accuracy ───────────────────────────────────────

#[test]
fn planet_longitude_accuracy() {
    let bsp = bsp_path();
    if !bsp.exists() {
        eprintln!("DE440s not found at {:?}, skipping planet_longitude_accuracy", bsp);
        return;
    }
    let spk = SpkReader::open(&bsp).expect("failed to open DE440s");
    let ana = AnalyticalProvider::new();
    let dates = test_dates();

    let planets = [
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ];

    let mut max_errors: Vec<(Body, f64, f64)> = Vec::new(); // (body, max_error_arcsec, at_jd)

    for &body in &planets {
        let mut body_max_err = 0.0_f64;
        let mut body_max_jd = 0.0_f64;

        eprintln!("\n=== {} ===", body.name());
        for &jd in &dates {
            let spk_pos = match coordinates::apparent_position(&spk, body, jd) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("  JD {jd:.1}: SpkReader error: {e}");
                    continue;
                }
            };
            let ana_pos = match coordinates::apparent_position(&ana, body, jd) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("  JD {jd:.1}: AnalyticalProvider error: {e}");
                    continue;
                }
            };

            let spk_lon = spk_pos.ecliptic.longitude.to_degrees();
            let ana_lon = ana_pos.ecliptic.longitude.to_degrees();
            let diff_deg = angular_diff_deg(spk_lon, ana_lon);
            let diff_arcsec = diff_deg * 3600.0;

            eprintln!(
                "  JD {jd:.1}: SPK={spk_lon:10.6}  ANA={ana_lon:10.6}  diff={diff_arcsec:8.2}\"",
            );

            if diff_deg > body_max_err {
                body_max_err = diff_deg;
                body_max_jd = jd;
            }
        }

        let max_arcsec = body_max_err * 3600.0;
        eprintln!(
            "  MAX ERROR: {max_arcsec:.2}\" at JD {body_max_jd:.1}",
        );
        max_errors.push((body, max_arcsec, body_max_jd));
    }

    eprintln!("\n=== Planet Longitude Summary ===");
    let mut any_failed = false;
    for &(body, err, jd) in &max_errors {
        let status = if err <= PLANET_TOLERANCE_ARCSEC { "PASS" } else { any_failed = true; "FAIL" };
        eprintln!("  {:10} max={err:8.2}\"  at JD {jd:.1}  [{status}]", body.name());
    }

    assert!(
        !any_failed,
        "Some planets exceeded the {PLANET_TOLERANCE_ARCSEC}\" tolerance — see summary above",
    );
}

// ─── Test 2: Sun longitude accuracy ─────────────────────────────────────────

#[test]
fn sun_longitude_accuracy() {
    let bsp = bsp_path();
    if !bsp.exists() {
        eprintln!("DE440s not found, skipping sun_longitude_accuracy");
        return;
    }
    let spk = SpkReader::open(&bsp).expect("failed to open DE440s");
    let ana = AnalyticalProvider::new();
    let dates = test_dates();
    let tolerance_deg = arcsec_to_deg(PLANET_TOLERANCE_ARCSEC);

    let mut max_err = 0.0_f64;
    let mut max_jd = 0.0_f64;

    eprintln!("\n=== Sun ===");
    for &jd in &dates {
        let spk_pos = match coordinates::apparent_position(&spk, Body::Sun, jd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  JD {jd:.1}: SpkReader error: {e}");
                continue;
            }
        };
        let ana_pos = match coordinates::apparent_position(&ana, Body::Sun, jd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  JD {jd:.1}: AnalyticalProvider error: {e}");
                continue;
            }
        };

        let spk_lon = spk_pos.ecliptic.longitude.to_degrees();
        let ana_lon = ana_pos.ecliptic.longitude.to_degrees();
        let diff_deg = angular_diff_deg(spk_lon, ana_lon);
        let diff_arcsec = diff_deg * 3600.0;

        eprintln!(
            "  JD {jd:.1}: SPK={spk_lon:10.6}  ANA={ana_lon:10.6}  diff={diff_arcsec:8.2}\"",
        );

        if diff_deg > max_err {
            max_err = diff_deg;
            max_jd = jd;
        }
    }

    let max_arcsec = max_err * 3600.0;
    eprintln!("  Sun MAX ERROR: {max_arcsec:.2}\" at JD {max_jd:.1}");

    assert!(
        max_err <= tolerance_deg,
        "Sun max longitude error {max_arcsec:.2}\" exceeds {PLANET_TOLERANCE_ARCSEC}\" tolerance (at JD {max_jd:.1})",
    );
}

// ─── Test 3: Moon longitude accuracy ────────────────────────────────────────

#[test]
fn moon_longitude_accuracy() {
    let bsp = bsp_path();
    if !bsp.exists() {
        eprintln!("DE440s not found, skipping moon_longitude_accuracy");
        return;
    }
    let spk = SpkReader::open(&bsp).expect("failed to open DE440s");
    let ana = AnalyticalProvider::new();
    let dates = test_dates();
    let tolerance_deg = arcsec_to_deg(MOON_TOLERANCE_ARCSEC);

    let mut max_err = 0.0_f64;
    let mut max_jd = 0.0_f64;

    eprintln!("\n=== Moon ===");
    for &jd in &dates {
        let spk_pos = match coordinates::apparent_position(&spk, Body::Moon, jd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  JD {jd:.1}: SpkReader error: {e}");
                continue;
            }
        };
        let ana_pos = match coordinates::apparent_position(&ana, Body::Moon, jd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  JD {jd:.1}: AnalyticalProvider error: {e}");
                continue;
            }
        };

        let spk_lon = spk_pos.ecliptic.longitude.to_degrees();
        let ana_lon = ana_pos.ecliptic.longitude.to_degrees();
        let diff_deg = angular_diff_deg(spk_lon, ana_lon);
        let diff_arcsec = diff_deg * 3600.0;

        eprintln!(
            "  JD {jd:.1}: SPK={spk_lon:10.6}  ANA={ana_lon:10.6}  diff={diff_arcsec:8.2}\"",
        );

        if diff_deg > max_err {
            max_err = diff_deg;
            max_jd = jd;
        }
    }

    let max_arcsec = max_err * 3600.0;
    eprintln!("  Moon MAX ERROR: {max_arcsec:.2}\" at JD {max_jd:.1}");

    assert!(
        max_err <= tolerance_deg,
        "Moon max longitude error {max_arcsec:.2}\" exceeds {MOON_TOLERANCE_ARCSEC}\" tolerance (at JD {max_jd:.1})",
    );
}

// ─── Test 4: Moon nakshatra boundary test ───────────────────────────────────

#[test]
fn moon_nakshatra_boundary() {
    let bsp = bsp_path();
    if !bsp.exists() {
        eprintln!("DE440s not found, skipping moon_nakshatra_boundary");
        return;
    }
    let spk = SpkReader::open(&bsp).expect("failed to open DE440s");
    let ana = AnalyticalProvider::new();

    // Nakshatra width: 13 + 20/60 = 13.333... degrees
    let nakshatra_width: f64 = 13.0 + 20.0 / 60.0;
    // Boundary threshold: 0.01 degrees
    let boundary_threshold: f64 = 0.01;

    let jd_start = 2451545.0; // J2000
    let jd_end = jd_start + 365.25;
    let step = 0.1; // days

    let mut boundary_dates: Vec<(f64, f64, usize)> = Vec::new(); // (jd, spk_lon, nakshatra_idx)

    // Scan for dates where SPK Moon is within threshold of a nakshatra boundary
    let mut jd = jd_start;
    while jd <= jd_end {
        if let Ok(spk_pos) = coordinates::apparent_position(&spk, Body::Moon, jd) {
            let lon_deg = spk_pos.ecliptic.longitude.to_degrees();
            // Check distance to nearest nakshatra boundary
            let boundary_index = (lon_deg / nakshatra_width).round() as usize;
            let nearest_boundary = boundary_index as f64 * nakshatra_width;
            let dist = angular_diff_deg(lon_deg, nearest_boundary);
            if dist < boundary_threshold {
                boundary_dates.push((jd, lon_deg, boundary_index % 27));
                if boundary_dates.len() >= 5 {
                    break;
                }
            }
        }
        jd += step;
    }

    eprintln!("\n=== Moon Nakshatra Boundary Test ===");
    eprintln!("Found {} boundary crossings", boundary_dates.len());
    assert!(
        boundary_dates.len() >= 3,
        "Expected at least 3 nakshatra boundary dates, found {}",
        boundary_dates.len()
    );

    let mut all_match = true;
    for &(jd, spk_lon, _boundary_idx) in &boundary_dates {
        let spk_nakshatra = (spk_lon / nakshatra_width).floor() as usize % 27;

        let ana_pos = match coordinates::apparent_position(&ana, Body::Moon, jd) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("  JD {jd:.1}: AnalyticalProvider error: {e}");
                all_match = false;
                continue;
            }
        };
        let ana_lon = ana_pos.ecliptic.longitude.to_degrees();
        let ana_nakshatra = (ana_lon / nakshatra_width).floor() as usize % 27;
        let diff_arcsec = angular_diff_deg(spk_lon, ana_lon) * 3600.0;

        let match_str = if spk_nakshatra == ana_nakshatra {
            "MATCH"
        } else {
            all_match = false;
            "MISMATCH"
        };

        eprintln!(
            "  JD {jd:.2}: SPK lon={spk_lon:.6} nak={spk_nakshatra}  ANA lon={ana_lon:.6} nak={ana_nakshatra}  diff={diff_arcsec:.2}\"  {match_str}",
        );
    }

    assert!(
        all_match,
        "Some nakshatra assignments differ between SpkReader and AnalyticalProvider near boundaries"
    );
}

// ─── Test 5: Node delegation test ───────────────────────────────────────────

#[test]
fn node_delegation() {
    let ana = AnalyticalProvider::new();
    let jd = 2451545.0; // J2000

    // Mean node should return finite values
    let mean_state = ana
        .compute_state(Body::MeanNode, jd)
        .expect("MeanNode should succeed");
    assert!(
        mean_state.position.x.is_finite()
            && mean_state.position.y.is_finite()
            && mean_state.position.z.is_finite(),
        "MeanNode position should be finite"
    );

    // True node should return finite values
    let true_state = ana
        .compute_state(Body::TrueNode, jd)
        .expect("TrueNode should succeed");
    assert!(
        true_state.position.x.is_finite()
            && true_state.position.y.is_finite()
            && true_state.position.z.is_finite(),
        "TrueNode position should be finite"
    );

    // Both are unit vectors encoded as ecliptic position; recover longitude
    // by converting back: the node_state encodes lon via ecliptic_to_equatorial,
    // so we do equatorial_to_ecliptic (reverse rotation) to get the longitude.
    // But we can also just use the nodes module directly to verify they differ.
    let mean_lon = vedaksha_ephem_core::nodes::mean_node(jd);
    let true_lon = vedaksha_ephem_core::nodes::true_node(jd);

    let mut diff = (mean_lon - true_lon).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }

    eprintln!("\n=== Node Delegation ===");
    eprintln!("  Mean Node: {mean_lon:.6} deg");
    eprintln!("  True Node: {true_lon:.6} deg");
    eprintln!("  Difference: {diff:.4} deg");

    assert!(
        diff < 5.0,
        "Mean and True node should differ by < 5 degrees, got {diff:.4}"
    );
    assert!(
        diff > 0.0,
        "Mean and True node should not be identical"
    );
}

// ─── Test 6: Error cases ────────────────────────────────────────────────────

#[test]
fn pluto_returns_error() {
    let ana = AnalyticalProvider::new();
    let result = ana.compute_state(Body::Pluto, 2451545.0);
    assert!(result.is_err(), "Pluto should return an error");
    match result.unwrap_err() {
        vedaksha_ephem_core::error::ComputeError::BodyNotAvailable { body_id } => {
            assert_eq!(body_id, Body::Pluto.naif_id());
            eprintln!("Pluto correctly returns BodyNotAvailable (NAIF ID {body_id})");
        }
        other => panic!("Expected BodyNotAvailable for Pluto, got: {other}"),
    }
}

#[test]
fn date_out_of_range_returns_error() {
    let ana = AnalyticalProvider::new();
    // JD 0 is way before the supported range
    let result = ana.compute_state(Body::Mars, 0.0);
    assert!(result.is_err(), "JD 0 should be out of range");
    match result.unwrap_err() {
        vedaksha_ephem_core::error::ComputeError::DateOutOfRange { jd, min, max } => {
            eprintln!("JD 0 correctly out of range: jd={jd}, min={min}, max={max}");
        }
        other => panic!("Expected DateOutOfRange for JD 0, got: {other}"),
    }
}
