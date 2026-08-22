// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Oracle regression: `AnalyticalProvider` vs JPL Horizons (DE441).
//!
//! Shares the fixture with `oracle_comparison.rs`, but exercises the
//! zero-data VSOP87A + ELP/MPP02 path rather than the SPK reader. This is the
//! only place the analytical provider meets a fully independent reference —
//! `analytical_accuracy.rs` compares it against our own DE440s.
//!
//! Regenerate the fixture with `python3 scripts/generate_horizons_oracle.py`.
//!
//! # Scope — and what this test does NOT cover
//!
//! This file and `oracle_comparison.rs` have near-identical names, share one
//! fixture, and print near-identical reports. They are nonetheless **disjoint
//! instruments**: they read the same reference rows through two providers that
//! share no evaluation code.
//!
//! | | provider | positions come from | enters `analytical/`? |
//! |---|---|---|---|
//! | `analytical_oracle.rs` (this file) | `AnalyticalProvider` | VSOP87A + ELP/MPP02 series, zero data files | **yes** |
//! | `oracle_comparison.rs` | `SpkReader` | DE440s binary kernel, Chebyshev interpolation | **no** |
//!
//! So `oracle_comparison.rs` is blind to everything this test covers, and vice
//! versa. Changes to `analytical/elp_mpp02.rs`, `analytical/vsop87a.rs`,
//! `analytical/simd_trig.rs`, or a dependency they pull in (`wide`) can only be
//! seen here or in `analytical_accuracy.rs` — never there. Treating that test's
//! digest as a gate on this path is the specific mistake this note exists to
//! prevent; see the `wide` entry in `Cargo.toml` for the case where it happened.
//!
//! Two further limits of this test specifically:
//!
//! - **Pluto is not covered.** It has no analytical theory, so `body_from_name`
//!   drops it: 21,915 of the fixture's 24,350 rows are compared (9 bodies ×
//!   2,435).
//! - **It is an accuracy gate, not a bit-level one.** It asserts an arcsecond
//!   ceiling (`MEASURED_MAX_ARCSEC`), so any change smaller than that — a
//!   1-ULP shift in the lunar series, say — passes it silently. For bit-level
//!   movement use `analytical_bit_digest.rs`.

mod common;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::EphemerisProvider;

/// Fixture wrapper: `{ "_provenance": {...}, "rows": [...] }`.
#[derive(serde::Deserialize)]
struct OracleFixture {
    rows: Vec<OracleDataPoint>,
}

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    jd: f64,
    body: String,
    ref_longitude: f64,
}

/// JD of 2026-01-01 — boundary between measured and predicted ΔT. See the
/// same constant in `oracle_comparison.rs` for why the eras are budgeted
/// separately.
const MEASURED_DT_JD: f64 = 2_461_041.5;

/// Per-point ceiling over the measured-ΔT era for the analytical provider.
///
/// Necessarily looser than the SpkReader bound: VSOP87A is a truncated
/// analytical theory, not an interpolated numerical kernel. Observed max is
/// 24.2″ (Venus, 1958) across 13,815 comparisons.
///
/// Note this is materially worse than the 13.09″ that `analytical_accuracy.rs`
/// reports for the same provider — that test samples 10 dates, this one 2,435
/// per body. The sparse grid simply never lands near Venus's worst case. Trust
/// this number; it is the better-sampled one.
const MEASURED_MAX_ARCSEC: f64 = 30.0;

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

    if !common::require_horizons_oracle() {
        return;
    }

    let oracle_json = std::fs::read_to_string(&oracle_path).unwrap();
    let fixture: OracleFixture = serde_json::from_str(&oracle_json).unwrap();
    let data_points = fixture.rows;

    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();

    let mut total = 0;
    let mut within_1_degree = 0;
    let mut max_error_arcsec: f64 = 0.0;
    let mut max_error_body = String::new();
    let mut max_error_date = String::new();
    let mut sum_error: f64 = 0.0;

    // Measured-ΔT era, tracked separately — see MEASURED_DT_JD.
    // body -> (count, sum_arcsec, max_arcsec)
    let mut per_body: std::collections::BTreeMap<String, (u32, f64, f64)> =
        std::collections::BTreeMap::new();
    let mut m_total = 0;
    let mut m_sum: f64 = 0.0;
    let mut m_max: f64 = 0.0;
    let mut m_max_body = String::new();
    let mut m_max_date = String::new();

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

        if dp.jd < MEASURED_DT_JD {
            m_total += 1;
            m_sum += diff_arcsec;
            if diff_arcsec > m_max {
                m_max = diff_arcsec;
                m_max_body = dp.body.clone();
                m_max_date = dp.date.clone();
            }
            let e = per_body
                .entry(dp.body.clone())
                .or_insert((0u32, 0.0f64, 0.0f64));
            e.0 += 1;
            e.1 += diff_arcsec;
            e.2 = e.2.max(diff_arcsec);
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

    let m_mean = m_sum / m_total.max(1) as f64;
    eprintln!("\n--- Measured-ΔT era (before 2026) ---");
    eprintln!("Comparisons:        {m_total}");
    eprintln!("Mean error:         {m_mean:.3}\"");
    eprintln!("Max error:          {m_max:.3}\" ({m_max_body} at {m_max_date})");
    eprintln!(
        "\n{:<10} {:>7} {:>10} {:>10}",
        "Body", "n", "mean\"", "max\""
    );
    for (body, (n, sum, max)) in &per_body {
        eprintln!(
            "{:<10} {:>7} {:>10.3} {:>10.3}",
            body,
            n,
            sum / f64::from(*n),
            max
        );
    }

    // Assert where the residual is attributable to the provider rather than to
    // ΔT extrapolation. Beyond 2025 our ΔT and Horizons' diverge (~68 s by
    // 2099), which for the Moon alone is ~45″ — that is a time-scale
    // disagreement, not a VSOP87A/ELP-MPP02 error.
    assert!(
        m_max < MEASURED_MAX_ARCSEC,
        "measured-ΔT era max error {m_max:.3}″ ({m_max_body} at {m_max_date}) exceeds \
         {MEASURED_MAX_ARCSEC}″ — VSOP87A/ELP-MPP02 accuracy regressed"
    );
    assert!(
        within_1_degree == total,
        "Some analytical positions differ by more than 1° from reference"
    );

    eprintln!("\n All {} positions within 1 degree of reference.\n", total);
}
