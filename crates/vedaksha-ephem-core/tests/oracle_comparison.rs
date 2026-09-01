// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Oracle comparison: `SpkReader` (DE440s) vs JPL Horizons (DE441).
//!
//! Reference values are astronomical facts — apparent geocentric ecliptic
//! longitudes fetched from NASA/JPL Horizons, a public-domain US Government
//! work. Horizons serves DE441, so this is a genuinely independent kernel
//! from the DE440s that `SpkReader` reads: the comparison measures our
//! Chebyshev evaluation and apparent-place pipeline against JPL's own.
//!
//! Regenerate the fixture with `python3 scripts/generate_horizons_oracle.py`;
//! that script documents the frame and time-scale contract both sides must
//! honour.
//!
//! # Scope — and what this test does NOT cover
//!
//! This file and `analytical_oracle.rs` have near-identical names, share one
//! fixture, and print near-identical reports. They are nonetheless **disjoint
//! instruments**: they read the same reference rows through two providers that
//! share no evaluation code.
//!
//! | | provider | positions come from | enters `analytical/`? |
//! |---|---|---|---|
//! | `oracle_comparison.rs` (this file) | `SpkReader` | DE440s binary kernel, Chebyshev interpolation | **no** |
//! | `analytical_oracle.rs` | `AnalyticalProvider` | VSOP87A + ELP/MPP02 series, zero data files | **yes** |
//!
//! What this file covers is the SPK reader, the Chebyshev evaluation and the
//! shared apparent-place pipeline (light-time, aberration, precession,
//! nutation). What it does **not** cover is the entire `analytical/` module:
//! nothing here constructs an `AnalyticalProvider`, so VSOP87A, ELP/MPP02,
//! `analytical/simd_trig.rs` and the `wide` dependency behind it are all
//! invisible to it.
//!
//! That matters because this test's per-row digest (`6dea5ddbd3c61b08…`) has
//! twice been used as a gate on changes it cannot see. Both times it was
//! demonstrated, not assumed:
//!
//! - an FMA change that rewrote **90.67%** of ELP/MPP02's output bits left this
//!   digest byte-identical;
//! - the `wide` 0.7.33 → 1.6.1 bump (`ee85cdf`) moved 6 Moon rows in
//!   `analytical_bit_digest.rs` while this digest did not move at all.
//!
//! If a change touches `analytical/`, this digest is the wrong instrument. Use
//! `analytical_oracle.rs` for accuracy and `analytical_bit_digest.rs` for bits.

mod common;

use std::fs;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_ephem_core::jpl::reader::SpkReader;

/// Fixture wrapper: `{ "_provenance": {...}, "rows": [...] }`.
#[derive(serde::Deserialize)]
struct OracleFixture {
    rows: Vec<OracleDataPoint>,
}

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    jd: f64,
    body: String,
    ref_longitude: f64,
    #[allow(dead_code)]
    ref_latitude: f64,
    #[allow(dead_code)]
    ref_distance: f64,
    #[allow(dead_code)]
    ref_speed: f64,
}

/// JD of 2026-01-01 — the boundary between ΔT that is *measured* and ΔT that
/// is *predicted* (`delta_t.rs` carries IERS values through 2025, then
/// extrapolates).
///
/// This split matters. At or before this epoch the residual against Horizons
/// reflects our ephemeris and apparent-place pipeline, and is sub-arcsecond.
/// After it, our Espenak & Meeus ΔT extrapolation and Horizons' own ΔT
/// prediction diverge — by ~68 s at 2099 — and that time offset shows up as a
/// longitude error proportional to each body's angular rate. The Moon, at
/// ~0.64″/s, picks up ~45″ of it; Pluto, at ~0.0003″/s, essentially none.
/// Confirmed empirically: at 2099-02-06 the Sun, Moon, Mercury, Venus and Mars
/// — rates spanning 0.03–0.64″/s — all imply the same 66–71 s offset.
///
/// So a future-date residual is a ΔT-prediction disagreement, not an ephemeris
/// error, and the two get separate budgets rather than one meaningless bound.
const MEASURED_DT_JD: f64 = 2_461_041.5;

/// Per-point ceiling over the measured-ΔT era. Observed max is 1.184″ (a
/// single Uranus point in 1900); 2.0″ leaves headroom without going slack.
const MEASURED_MAX_ARCSEC: f64 = 2.0;

/// Mean ceiling over the measured-ΔT era. Observed mean is 0.103″.
const MEASURED_MEAN_ARCSEC: f64 = 0.20;

/// Per-point ceiling over the predicted-ΔT era, sized by the ΔT divergence
/// rather than by our ephemeris. Observed max is 44.9″ (Moon, 2099).
const PREDICTED_MAX_ARCSEC: f64 = 60.0;

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
    if !common::require_bsp() || !common::require_horizons_oracle() {
        return;
    }
    let bsp = bsp_path();
    let oracle_path = oracle_data_path();

    let reader = SpkReader::open(&bsp).expect("Failed to open DE440s");
    let (jd_min, jd_max) = reader.time_range();

    let oracle_json = fs::read_to_string(&oracle_path).expect("Failed to read oracle data");
    let fixture: OracleFixture =
        serde_json::from_str(&oracle_json).expect("Failed to parse oracle data");
    let data_points = fixture.rows;

    eprintln!("\n===========================================================================");
    eprintln!(" Vedaksha — Accuracy Report");
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

    // Measured-ΔT era statistics, tracked separately — see MEASURED_DT_JD.
    let mut m_total = 0;
    let mut m_sum: f64 = 0.0;
    let mut m_max: f64 = 0.0;
    let mut m_max_body = String::new();
    let mut m_max_date = String::new();

    // Measured-ΔT era, per body — body -> (count, sum_arcsec, max_arcsec).
    // Mirrors analytical_oracle.rs's per_body table so the two accuracy paths
    // publish comparable per-body figures in the same shape.
    let mut per_body: std::collections::BTreeMap<String, (u32, f64, f64)> =
        std::collections::BTreeMap::new();

    // Predicted-ΔT era: tracked only to report and to bound the ΔT divergence.
    let mut p_max: f64 = 0.0;
    let mut p_max_body = String::new();
    let mut p_max_date = String::new();

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

        // Compute using Vedaksha
        let result = coordinates::apparent_position(&reader, body, dp.jd);
        let vedaksha_lon = match result {
            Ok(pos) => pos.ecliptic.longitude.to_degrees(),
            Err(e) => {
                eprintln!(
                    "  ERROR: {} at {} (JD {}): {:?}",
                    dp.body, dp.date, dp.jd, e
                );
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
        } else if diff_arcsec > p_max {
            p_max = diff_arcsec;
            p_max_body = dp.body.clone();
            p_max_date = dp.date.clone();
        }

        errors.push((
            dp.body.clone(),
            dp.date.clone(),
            dp.ref_longitude,
            vedaksha_lon,
            diff_arcsec,
        ));
    }

    // Print detailed results
    eprintln!(
        "{:<10} {:<24} {:>12} {:>12} {:>10}",
        "Body", "Date", "Ref", "Vedaksha", "Diff (″)"
    );
    eprintln!("{}", "-".repeat(72));

    for (body, date, swe, ved, diff) in &errors {
        let marker = if *diff > 60.0 {
            " ⚠"
        } else if *diff > 10.0 {
            " ●"
        } else {
            ""
        };
        eprintln!(
            "{:<10} {:<24} {:>12.6} {:>12.6} {:>9.3}{}",
            body, date, swe, ved, diff, marker
        );
    }

    // Print summary
    eprintln!("\n===========================================================================");
    eprintln!(" ACCURACY SUMMARY");
    eprintln!("===========================================================================\n");
    eprintln!("Total comparisons:    {}", total);
    eprintln!("Skipped (out of range): {}", skipped);
    eprintln!("");
    eprintln!(
        "Within 1 arcsecond:   {} / {} ({:.1}%)",
        within_1_arcsec,
        total,
        100.0 * within_1_arcsec as f64 / total as f64
    );
    eprintln!(
        "Within 10 arcseconds: {} / {} ({:.1}%)",
        within_10_arcsec,
        total,
        100.0 * within_10_arcsec as f64 / total as f64
    );
    eprintln!(
        "Within 1 arcminute:   {} / {} ({:.1}%)",
        within_1_arcmin,
        total,
        100.0 * within_1_arcmin as f64 / total as f64
    );
    eprintln!(
        "Within 1 degree:      {} / {} ({:.1}%)",
        within_1_degree,
        total,
        100.0 * within_1_degree as f64 / total as f64
    );
    eprintln!("");
    eprintln!(
        "Mean error:           {:.3} arcseconds",
        sum_error / total as f64
    );
    eprintln!(
        "Max error:            {:.3} arcseconds ({} at {})",
        max_error, max_error_body, max_error_date
    );

    let m_mean = m_sum / m_total as f64;
    eprintln!("\n--- Measured-ΔT era (JD < {MEASURED_DT_JD}, i.e. before 2026) ---");
    eprintln!("Comparisons:          {m_total}");
    eprintln!("Mean error:           {m_mean:.3} arcseconds");
    eprintln!("Max error:            {m_max:.3} arcseconds ({m_max_body} at {m_max_date})");
    eprintln!("\nPER-BODY (measured-ΔT era)");
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
    eprintln!("\n--- Predicted-ΔT era (2026+; residual is ΔT divergence, not ephemeris) ---");
    eprintln!("Max error:            {p_max:.3} arcseconds ({p_max_body} at {p_max_date})");

    // Accuracy is asserted where it is actually attributable to us: the era
    // where ΔT is measured rather than predicted. These bounds are what the
    // published sub-arcsecond claim rests on, so they are tight enough to fail
    // on a real regression.
    assert!(
        m_max < MEASURED_MAX_ARCSEC,
        "measured-ΔT era max error {m_max:.3}″ ({m_max_body} at {m_max_date}) exceeds \
         {MEASURED_MAX_ARCSEC}″ — the sub-arcsecond claim no longer holds"
    );
    assert!(
        m_mean < MEASURED_MEAN_ARCSEC,
        "measured-ΔT era mean error {m_mean:.3}″ exceeds {MEASURED_MEAN_ARCSEC}″"
    );

    // Future dates are bounded only against ΔT-prediction divergence. A blowout
    // here means either our ΔT model or the ephemeris moved — worth a look, but
    // it is not the accuracy claim.
    assert!(
        p_max < PREDICTED_MAX_ARCSEC,
        "predicted-ΔT era max error {p_max:.3}″ ({p_max_body} at {p_max_date}) exceeds \
         {PREDICTED_MAX_ARCSEC}″ — check delta_t.rs against Horizons' ΔT"
    );

    // Nothing anywhere in range should drift past an arcminute.
    assert!(
        within_1_arcmin == total,
        "{} of {total} positions differ from reference by more than 1 arcminute",
        total - within_1_arcmin
    );

    eprintln!(
        "\n✓ {m_total} measured-ΔT comparisons: mean {m_mean:.3}″, max {m_max:.3}″ \
         — sub-arcsecond agreement with JPL Horizons DE441."
    );
}
