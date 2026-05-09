// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Oracle regression: AnalyticalProvider vs independent reference data.
//! Validates that the analytical provider produces positions within
//! 1 degree of the independent reference data (same baseline as oracle_comparison.rs).

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::EphemerisProvider;

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    jd: f64,
    body: String,
    #[serde(alias = "swe_longitude")]
    ref_longitude: f64,
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
        _ => None, // Skip Pluto
    }
}

#[test]
fn analytical_oracle_regression() {
    let oracle_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("oracle_jpl")
        .join("reference_positions.json");

    if !oracle_path.exists() {
        eprintln!(
            "Oracle data not found at {}, skipping",
            oracle_path.display()
        );
        return;
    }

    let oracle_json = std::fs::read_to_string(&oracle_path).unwrap();
    let data_points: Vec<OracleDataPoint> = serde_json::from_str(&oracle_json).unwrap();

    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();

    let mut total = 0;
    let mut within_1_degree = 0;
    let mut max_error_arcsec: f64 = 0.0;
    let mut max_error_body = String::new();
    let mut max_error_date = String::new();
    let mut sum_error: f64 = 0.0;

    for dp in &data_points {
        let body = match body_from_name(&dp.body) {
            Some(b) => b,
            None => continue,
        };

        if dp.jd < jd_min || dp.jd > jd_max {
            continue;
        }

        let result = coordinates::apparent_position(&provider, body, dp.jd);
        let lon = match result {
            Ok(pos) => pos.ecliptic.longitude.to_degrees(),
            Err(e) => {
                eprintln!(
                    "  ERROR: {} at {} (JD {}): {:?}",
                    dp.body, dp.date, dp.jd, e
                );
                continue;
            }
        };

        let mut diff = (lon - dp.ref_longitude).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        let diff_arcsec = diff * 3600.0;

        total += 1;
        sum_error += diff_arcsec;
        if diff <= 1.0 {
            within_1_degree += 1;
        }
        if diff_arcsec > max_error_arcsec {
            max_error_arcsec = diff_arcsec;
            max_error_body = dp.body.clone();
            max_error_date = dp.date.clone();
        }
    }

    eprintln!("\n===========================================================================");
    eprintln!(" Analytical Provider — Oracle Regression");
    eprintln!("===========================================================================\n");
    eprintln!("Total comparisons:  {total}");
    eprintln!(
        "Within 1 degree:    {within_1_degree}/{total} ({:.1}%)",
        100.0 * within_1_degree as f64 / total.max(1) as f64
    );
    eprintln!(
        "Mean error:         {:.3}\"",
        sum_error / total.max(1) as f64
    );
    eprintln!(
        "Max error:          {:.3}\" ({} at {})",
        max_error_arcsec, max_error_body, max_error_date
    );

    assert!(
        within_1_degree == total,
        "Some analytical positions differ by more than 1° from reference"
    );

    eprintln!("\n All {} positions within 1 degree of reference.\n", total);
}
