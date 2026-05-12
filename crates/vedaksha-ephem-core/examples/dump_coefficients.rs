// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Diagnostic dumper for VDKBLOB1 coefficient blobs.
//!
//! Usage:
//!     cargo run -p vedaksha-ephem-core --example dump_coefficients -- <path/to/blob.bin>
//!
//! The blob's record type is inferred from `record_size_bytes` in the
//! 24-byte header (24 = Vsop87Term, 72 = ElpMainTerm, 68 = ElpPertTerm).
//! Each record is printed as one line of `key=value` pairs.

use std::env;
use std::fs;
use std::process::ExitCode;

use vedaksha_ephem_core::analytical::coefficients::loader::{
    ELP_MAIN_RECORD_SIZE, ELP_PERT_RECORD_SIZE, VSOP87_RECORD_SIZE, parse_elp_main, parse_elp_pert,
    parse_header, parse_vsop87,
};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: dump_coefficients <path/to/blob.bin>");
        return ExitCode::from(2);
    };

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };

    let header = match parse_header(&bytes) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error: {path}: {e}");
            return ExitCode::from(1);
        }
    };

    println!(
        "# {path}: version={} record_size={} record_count={}",
        header.version, header.record_size, header.record_count
    );

    match header.record_size {
        s if s == VSOP87_RECORD_SIZE => {
            let recs = parse_vsop87(&bytes).expect("header validated above");
            for (i, t) in recs.iter().enumerate() {
                println!(
                    "rec={i} amplitude={:.15e} phase={:.15e} frequency={:.15e}",
                    t.amplitude, t.phase, t.frequency
                );
            }
        }
        s if s == ELP_MAIN_RECORD_SIZE => {
            let recs = parse_elp_main(&bytes).expect("header validated above");
            for (i, t) in recs.iter().enumerate() {
                println!(
                    "rec={i} i1={} i2={} i3={} i4={} amp={:.5} b1={:.2} b2={:.2} b3={:.2} b4={:.2} b5={:.2} b6={:.2}",
                    t.i1, t.i2, t.i3, t.i4, t.amp, t.b1, t.b2, t.b3, t.b4, t.b5, t.b6
                );
            }
        }
        s if s == ELP_PERT_RECORD_SIZE => {
            let recs = parse_elp_pert(&bytes).expect("header validated above");
            for (i, t) in recs.iter().enumerate() {
                println!(
                    "rec={i} s={:.13e} c={:.13e} i1={} i2={} i3={} i4={} i5={} i6={} i7={} i8={} i9={} i10={} i11={} i12={} i13={}",
                    t.s,
                    t.c,
                    t.i1,
                    t.i2,
                    t.i3,
                    t.i4,
                    t.i5,
                    t.i6,
                    t.i7,
                    t.i8,
                    t.i9,
                    t.i10,
                    t.i11,
                    t.i12,
                    t.i13
                );
            }
        }
        other => {
            eprintln!("error: {path}: unknown record size {other}");
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}
