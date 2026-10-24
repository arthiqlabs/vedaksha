// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Shared fixture gating for the integration tests.
//!
//! Several tests depend on files that are not in git: `data/de440s.bsp` (32 MB
//! binary kernel) and the generated Horizons oracle. Historically each test
//! coped by returning early when its fixture was missing, which meant the
//! suite reported a confident green while verifying nothing — `spk_reader_test`
//! printed "8 passed" in 0.00s with no kernel on disk.
//!
//! Skipping is still right for the fast per-push CI job, which has no kernel
//! and shouldn't spend 31 MB of download on every push. It is wrong for the
//! scheduled Full Validation run, whose entire purpose is to actually verify.
//! So the choice is explicit: set `VEDAKSHA_REQUIRE_FIXTURES=1` and a missing
//! fixture is a hard failure instead of a silent pass.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub mod tai_utc;

/// Workspace root, resolved from this crate's manifest directory.
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

/// The bundled DE440s SPK kernel. Fetch with `scripts/download_de440s.sh`.
pub fn bsp_path() -> PathBuf {
    workspace_root().join("data").join("de440s.bsp")
}

/// The generated JPL Horizons oracle fixture.
pub fn horizons_oracle_path() -> PathBuf {
    workspace_root()
        .join("tests")
        .join("oracle_jpl")
        .join("reference_positions.json")
}

/// Whether a missing fixture must fail rather than skip.
///
/// Set by the scheduled Full Validation workflow. Any value except `0` counts
/// as set, so `VEDAKSHA_REQUIRE_FIXTURES=1` reads naturally.
pub fn fixtures_required() -> bool {
    std::env::var("VEDAKSHA_REQUIRE_FIXTURES").is_ok_and(|v| v != "0")
}

/// Gate a fixture-dependent test on `path`.
///
/// Returns `true` to proceed. Returns `false` to skip (having said so on
/// stderr), or panics when `VEDAKSHA_REQUIRE_FIXTURES` is set — never returns
/// `false` quietly.
#[must_use]
pub fn require(path: &Path, what: &str, howto: &str) -> bool {
    if path.exists() {
        return true;
    }
    assert!(
        !fixtures_required(),
        "VEDAKSHA_REQUIRE_FIXTURES is set, but {what} is missing at {}.\n\
         This run was supposed to verify against it, not skip it.\n\
         Fix: {howto}",
        path.display()
    );
    eprintln!(
        "SKIPPED: {what} not found at {} — {howto}",
        path.display(),
        howto = howto
    );
    false
}

/// Gate on the DE440s kernel.
#[must_use]
pub fn require_bsp() -> bool {
    require(&bsp_path(), "de440s.bsp", "run scripts/download_de440s.sh")
}

/// Gate on the Horizons oracle fixture.
#[must_use]
pub fn require_horizons_oracle() -> bool {
    require(
        &horizons_oracle_path(),
        "Horizons oracle fixture",
        "run python3 scripts/generate_horizons_oracle.py",
    )
}

/// First Julian Day at which JPL Horizons treats the "UT" of its observer tables
/// as UTC rather than UT1: 1962-01-20, where its EOP file begins ("EOP coverage:
/// DATA-BASED 1962-JAN-20 TO ..." in every Horizons header). Its manual
/// (<https://ssd.jpl.nasa.gov/horizons/manual.html>): "prior to 1962, UT means
/// UT1. After 1962, UT means UTC", and for the future "the closest known
/// leap-second correction is used".
pub const HORIZONS_UTC_START_JD: f64 = 2_437_684.5;

/// `TAI - UTC` in seconds at a UTC Julian Day, from USNO's `tai-utc.dat`
/// (see `tai_utc.rs`). Before 1961 there is no such quantity and this is
/// `None`; after the last leap second (2017-01-01, 37 s) the last row stays in
/// force, as it does in Horizons.
pub fn tai_minus_utc(jd_utc: f64) -> Option<f64> {
    let &(_, a, mjd0, rate) = tai_utc::TAI_UTC
        .iter()
        .rev()
        .find(|&&(start, ..)| start <= jd_utc)?;
    Some(a + (jd_utc - 2_400_000.5 - mjd0) * rate)
}

/// The Terrestrial Time Julian Day of a Horizons oracle row, where Horizons'
/// time tag is UTC: `TT = UTC + 32.184 s + (TAI - UTC)`. `None` for rows before
/// [`HORIZONS_UTC_START_JD`], whose tag is UT1 and which go through the engine's
/// own UT1 to TT conversion instead.
///
/// This is what makes the comparison like for like. The engine is handed UT1 and
/// carries no UT1 - UTC (up to 0.9 s, 0.5 arcsec of Moon longitude), so
/// feeding it Horizons' UTC tag as if it were UT1 measured the sum of the
/// engine's error and that difference. Converting to TT here, with the leap-second
/// count, removes Delta T and UT1 - UTC from the comparison altogether, the way
/// `ecliptic_position_tt` was written to allow.
pub fn horizons_jd_tt(jd_ut: f64) -> Option<f64> {
    if jd_ut < HORIZONS_UTC_START_JD {
        return None;
    }
    let tai_utc = tai_minus_utc(jd_ut)?;
    Some(jd_ut + (32.184 + tai_utc) / 86_400.0)
}
