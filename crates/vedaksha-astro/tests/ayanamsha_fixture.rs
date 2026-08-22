// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The engine must agree with an independent derivation of the same primaries.
//!
//! `scripts/generate_ayanamsha.py` derives every ayanamsha again, in Python,
//! from the anchors recorded in
//! `docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json`. It shares
//! no code with this crate — it reads the same published polynomials and the
//! same catalogue rows and does the arithmetic itself.
//!
//! This is what keeps the derivation executable. Sections 6.1–6.3 of the spec
//! test self-consistency, and a reverse-engineered anchor reproduces itself
//! perfectly, so they cannot on their own establish that a value was derived.
//! What they *can* establish is that no constant entered by being typed into
//! one side: the Rust hardcodes its constants while Python reads the manifest,
//! so a divergence between them fails here.
//!
//! **They are not fully independent, and the earlier wording ("share nothing")
//! overstated it.** Both read the same primary values from
//! `derivation-inputs.json`, so a single transcription error made identically
//! into both would pass. What closes that for the catalogue rows is the
//! committed VizieR responses; for the polynomials it is the ERFA pins; for the
//! book anchors nothing closes it, and the audit dir says so.
//!
//! Regenerate with `python3 scripts/generate_ayanamsha.py`; the same script's
//! `--verify` mode is wired into full validation.

use std::str::FromStr;
use vedaksha_astro::sidereal::{Ayanamsha, ayanamsha_value};

const FIXTURE: &str = include_str!("fixtures/ayanamsha.json");

#[test]
fn engine_reproduces_the_independent_python_derivation() {
    let fixture: serde_json::Value =
        serde_json::from_str(FIXTURE).expect("fixture is not valid JSON");

    let tolerance = fixture["tolerance_deg"]
        .as_f64()
        .expect("fixture declares no tolerance");
    let rows = fixture["rows"].as_array().expect("fixture has no rows");
    assert!(!rows.is_empty(), "an empty fixture would pass forever");

    let mut compared = 0usize;
    let mut worst = (0.0_f64, String::new());

    for row in rows {
        let jd = row["jd_tt"].as_f64().expect("row has no jd_tt");
        let label = row["label"].as_str().unwrap_or("?");
        let values = row["ayanamsha_deg"]
            .as_object()
            .expect("row has no ayanamsha_deg map");

        for (key, want) in values {
            let system = Ayanamsha::from_str(key)
                .unwrap_or_else(|e| panic!("fixture names a system the engine lacks: {e}"));
            let want = want.as_f64().expect("non-numeric fixture value");
            let got = ayanamsha_value(system, jd);
            let delta = (got - want).abs();
            if delta > worst.0 {
                worst = (delta, format!("{key} at {label}"));
            }
            assert!(
                delta <= tolerance,
                "{key} at {label} (jd={jd}): engine {got:.12}, Python {want:.12}, \
                 differ by {:.3e} deg ({:.6} arcsec) — tolerance is {tolerance:.0e} deg",
                delta,
                delta * 3600.0
            );
            compared += 1;
        }
    }

    // Guard against the fixture silently shrinking to nothing meaningful.
    assert_eq!(
        compared,
        rows.len() * Ayanamsha::ALL.len(),
        "the fixture does not cover every system at every epoch"
    );
    println!(
        "{compared} comparisons, worst disagreement {:.3e} deg at {}",
        worst.0, worst.1
    );
}

#[test]
fn the_fixture_covers_every_system_the_engine_ships() {
    // If a system is added to the engine and not to the fixture, the cross-check
    // silently stops covering it. This fails instead.
    let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let listed: Vec<&str> = fixture["systems"]
        .as_array()
        .expect("fixture has no systems list")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    for system in Ayanamsha::ALL {
        assert!(
            listed.contains(&system.key()),
            "{} is in the engine but not in the fixture — regenerate it with \
             `python3 scripts/generate_ayanamsha.py`",
            system.key()
        );
    }
    assert_eq!(
        listed.len(),
        Ayanamsha::ALL.len(),
        "the fixture lists {} systems, the engine has {}",
        listed.len(),
        Ayanamsha::ALL.len()
    );
}
