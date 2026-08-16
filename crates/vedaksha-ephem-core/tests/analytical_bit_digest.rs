// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Bit-exact dump of the analytical provider's per-row output.
//!
//! This exists because `oracle_comparison.rs`'s digest is a **blind instrument**
//! for anything in `analytical/`. That test drives a `SpkReader` over the DE440s
//! binary kernel and never calls `analytical/elp_mpp02.rs`; an FMA change that
//! rewrote 90.67% of ELP's output bits left its digest byte-identical. Any
//! change to the lunar/planetary theory — or to a dependency those modules use,
//! such as `wide` via `analytical/simd_trig.rs` — has to be gated here instead.
//!
//! It prints, it does not assert. Accuracy is `analytical_oracle.rs`'s job; this
//! is a before/after fingerprint you diff across two commits. Output goes to
//! **stdout**, which the libtest harness captures and discards for a passing
//! test, so a routine `--include-ignored` run stays quiet. You only see the rows
//! when you ask for them with `--nocapture`.
//!
//! Re-run (from the workspace root, on both commits being compared):
//!
//! ```text
//! cargo test -p vedaksha-ephem-core --release --test analytical_bit_digest \
//!     -- --ignored --nocapture \
//!   | grep '^ROW ' | tr -s ' ' | LC_ALL=C sort | shasum -a 256
//! ```
//!
//! Restrict to the lunar theory — the only consumer of `simd_trig`/`wide` — by
//! inserting `| grep ' Moon '` before the `tr`.

mod common;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::EphemerisProvider;

#[derive(serde::Deserialize)]
struct OracleFixture {
    rows: Vec<OracleDataPoint>,
}

#[derive(serde::Deserialize)]
struct OracleDataPoint {
    date: String,
    jd: f64,
    body: String,
}

/// Same mapping as `analytical_oracle.rs`: Pluto has no analytical theory, so
/// the fixture's 24,350 rows reduce to 9 bodies' worth.
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
        _ => None,
    }
}

#[test]
#[ignore = "diagnostic dump, not an assertion — see the module comment"]
fn analytical_bit_digest() {
    let oracle_path = common::horizons_oracle_path();
    assert!(
        oracle_path.exists(),
        "fixture missing at {} — this dump has nothing to fingerprint",
        oracle_path.display()
    );

    let fixture: OracleFixture =
        serde_json::from_str(&std::fs::read_to_string(&oracle_path).unwrap()).unwrap();

    let provider = AnalyticalProvider;
    let (jd_min, jd_max) = provider.time_range();

    let mut emitted = 0u32;
    let mut skipped_body = 0u32;
    let mut skipped_range = 0u32;

    for dp in &fixture.rows {
        let Some(body) = body_from_name(&dp.body) else {
            skipped_body += 1;
            continue;
        };
        if dp.jd < jd_min || dp.jd > jd_max {
            skipped_range += 1;
            continue;
        }

        // An error is emitted as a row rather than skipped: a body that starts
        // failing must change the digest, not quietly shrink it.
        match coordinates::apparent_position(&provider, body, dp.jd) {
            Ok(p) => println!(
                "ROW {} {} {:.6} {:016x} {:016x} {:016x} {:016x}",
                dp.date,
                dp.body,
                dp.jd,
                p.ecliptic.longitude.to_bits(),
                p.ecliptic.latitude.to_bits(),
                p.ecliptic.distance.to_bits(),
                p.longitude_speed.to_bits(),
            ),
            Err(e) => println!("ROW {} {} {:.6} ERR {e:?}", dp.date, dp.body, dp.jd),
        }
        emitted += 1;
    }

    eprintln!(
        "analytical_bit_digest: {emitted} rows emitted \
         ({skipped_body} skipped as body-without-theory, \
         {skipped_range} skipped as out-of-range, \
         {} in fixture)",
        fixture.rows.len()
    );
}
