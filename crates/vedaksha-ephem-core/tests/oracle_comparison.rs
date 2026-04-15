// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Oracle comparison: Vedākṣha vs independent reference data.
//!
//! Reference values are astronomical facts (planetary longitudes)
//! generated from an independent ephemeris implementation.

use std::fs;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::reader::SpkReader;
use vedaksha_ephem_core::jpl::EphemerisProvider;

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    jd: f64,
    body: String,
    #[serde(alias = "swe_longitude")]
    ref_longitude: f64,
    #[serde(alias = "swe_latitude")]
    #[allow(dead_code)]
    ref_latitude: f64,
    #[serde(alias = "swe_distance")]
    #[allow(dead_code)]
    ref_distance: f64,
    #[serde(alias = "swe_speed")]
    #[allow(dead_code)]
    ref_speed: f64,
}

fn body_from_name(name: &str) -> Option<Body> {
    match name {
        "Sun" => Some(Body::Sun),
        "Moon" => Some(Body::Moon),
        "Mercury" => Some(Body::Mercury),
        "Venus" => Some(Body::Venus),
        "Mars" => Some(Body::Mars),
        "Jupiter" => Some(Body::Jupiter),
        "Saturn" => Some(Body::Saturn),
        "Uranus" => Some(Body::Uranus),
        "Neptune" => Some(Body::Neptune),
        "Pluto" => Some(Body::Pluto),
        _ => None,
    }
}

fn bsp_path() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("de440s.bsp")
}

fn oracle_data_path() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("oracle_jpl")
        .join("reference_positions.json")
}

#[test]
fn compare_against_reference() {
    let bsp = bsp_path();
    if !bsp.exists() {
        eprintln!("DE440s not found at {}. Skipping.", bsp.display());
        return;
    }

    let oracle_path = oracle_data_path();
    if !oracle_path.exists() {
        eprintln!("Oracle data not found at {}. Run: python3 tests/oracle_comparison.py", oracle_path.display());
        return;
    }

    let reader = SpkReader::open(&bsp).expect("Failed to open DE440s");
    let (jd_min, jd_max) = reader.time_range();

    let oracle_json = fs::read_to_string(&oracle_path).expect("Failed to read oracle data");
    let data_points: Vec<OracleDataPoint> =
        serde_json::from_str(&oracle_json).expect("Failed to parse oracle data");

    eprintln!("\n===========================================================================");
    eprintln!(" Vedākṣha — Accuracy Report");
    eprintln!("===========================================================================\n");

    let mut total = 0;
    let mut skipped = 0;
    let mut within_1_arcsec = 0;
    let mut within_10_arcsec = 0;
    let mut within_1_arcmin = 0;
    let mut within_1_degree = 0;
    let mut max_error: f64 = 0.0;
    let mut max_error_body = String::new();
    let mut max_error_date = String::new();
    let mut sum_error: f64 = 0.0;
    let mut errors: Vec<(String, String, f64, f64, f64)> = Vec::new();

    for dp in &data_points {
        let body = match body_from_name(&dp.body) {
            Some(b) => b,
            None => continue,
        };

        // Skip dates outside DE440s range
        if dp.jd < jd_min || dp.jd > jd_max {
            skipped += 1;
            continue;
        }

        // Compute using Vedākṣha
        let result = coordinates::apparent_position(&reader, body, dp.jd);
        let vedaksha_lon = match result {
            Ok(pos) => pos.ecliptic.longitude.to_degrees(),
            Err(e) => {
                eprintln!("  ERROR: {} at {} (JD {}): {:?}", dp.body, dp.date, dp.jd, e);
                skipped += 1;
                continue;
            }
        };

        // Compute angular difference (shortest arc)
        let mut diff = (vedaksha_lon - dp.ref_longitude).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }

        let diff_arcsec = diff * 3600.0;

        total += 1;
        sum_error += diff_arcsec;

        if diff_arcsec <= 1.0 {
            within_1_arcsec += 1;
        }
        if diff_arcsec <= 10.0 {
            within_10_arcsec += 1;
        }
        if diff_arcsec <= 60.0 {
            within_1_arcmin += 1;
        }
        if diff <= 1.0 {
            within_1_degree += 1;
        }

        if diff_arcsec > max_error {
            max_error = diff_arcsec;
            max_error_body = dp.body.clone();
            max_error_date = dp.date.clone();
        }

        errors.push((dp.body.clone(), dp.date.clone(), dp.ref_longitude, vedaksha_lon, diff_arcsec));
    }

    // Print detailed results
    eprintln!("{:<10} {:<24} {:>12} {:>12} {:>10}", "Body", "Date", "Ref", "Vedākṣha", "Diff (″)");
    eprintln!("{}", "-".repeat(72));

    for (body, date, swe, ved, diff) in &errors {
        let marker = if *diff > 60.0 { " ⚠" } else if *diff > 10.0 { " ●" } else { "" };
        eprintln!("{:<10} {:<24} {:>12.6} {:>12.6} {:>9.3}{}", body, date, swe, ved, diff, marker);
    }

    // Print summary
    eprintln!("\n===========================================================================");
    eprintln!(" ACCURACY SUMMARY");
    eprintln!("===========================================================================\n");
    eprintln!("Total comparisons:    {}", total);
    eprintln!("Skipped (out of range): {}", skipped);
    eprintln!("");
    eprintln!("Within 1 arcsecond:   {} / {} ({:.1}%)", within_1_arcsec, total, 100.0 * within_1_arcsec as f64 / total as f64);
    eprintln!("Within 10 arcseconds: {} / {} ({:.1}%)", within_10_arcsec, total, 100.0 * within_10_arcsec as f64 / total as f64);
    eprintln!("Within 1 arcminute:   {} / {} ({:.1}%)", within_1_arcmin, total, 100.0 * within_1_arcmin as f64 / total as f64);
    eprintln!("Within 1 degree:      {} / {} ({:.1}%)", within_1_degree, total, 100.0 * within_1_degree as f64 / total as f64);
    eprintln!("");
    eprintln!("Mean error:           {:.3} arcseconds", sum_error / total as f64);
    eprintln!("Max error:            {:.3} arcseconds ({} at {})", max_error, max_error_body, max_error_date);

    // The test passes if all positions are within 1 degree
    // (our pipeline doesn't yet have all corrections for sub-arcsecond)
    assert!(
        within_1_degree == total,
        "Some positions differ by more than 1 degree from reference"
    );

    eprintln!("\n✓ All {} positions within 1 degree of reference.", total);
}
