// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

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
