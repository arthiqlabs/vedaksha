// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! One-shot capture of pre-rederivation lunar outputs as a numerical
//! regression oracle. Run before quarantining the contaminated
//! implementation. The binary it depends on is itself contaminated; this
//! file is deleted in the same commit that quarantines the rest of the
//! contaminated lunar code, so the legacy-oracle fixture is the only
//! artifact that crosses the clean-room firewall.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use vedaksha_ephem_core::analytical::elp_mpp02::{elp_geocentric, MoonRectangular};

const N_ROWS: usize = 10_000;
const JD_START: f64 = 625_673.5;   // approx -3000-01-01 TT
const JD_END:   f64 = 2_816_787.5; // approx +3000-01-01 TT

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../tests/fixtures/lunar_legacy_oracle.bin");
    let f = File::create(&path).expect("create fixture");
    let mut w = BufWriter::new(f);

    let step = (JD_END - JD_START) / (N_ROWS as f64 - 1.0);
    for i in 0..N_ROWS {
        let jd = JD_START + (i as f64) * step;
        let MoonRectangular { x, y, z, vx, vy, vz } = elp_geocentric(jd);
        for v in [jd, x, y, z, vx, vy, vz] {
            w.write_all(&v.to_le_bytes()).unwrap();
        }
    }
    w.flush().unwrap();
    eprintln!("wrote {} rows to {}", N_ROWS, path.display());
}
