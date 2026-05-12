// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Saturn
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x0.bin")).expect("malformed saturn/x0.bin")
});

pub static X1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x1.bin")).expect("malformed saturn/x1.bin")
});

pub static X2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x2.bin")).expect("malformed saturn/x2.bin")
});

pub static X3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x3.bin")).expect("malformed saturn/x3.bin")
});

pub static X4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x4.bin")).expect("malformed saturn/x4.bin")
});

pub static X5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/x5.bin")).expect("malformed saturn/x5.bin")
});

pub static Y0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y0.bin")).expect("malformed saturn/y0.bin")
});

pub static Y1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y1.bin")).expect("malformed saturn/y1.bin")
});

pub static Y2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y2.bin")).expect("malformed saturn/y2.bin")
});

pub static Y3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y3.bin")).expect("malformed saturn/y3.bin")
});

pub static Y4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y4.bin")).expect("malformed saturn/y4.bin")
});

pub static Y5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/y5.bin")).expect("malformed saturn/y5.bin")
});

pub static Z0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z0.bin")).expect("malformed saturn/z0.bin")
});

pub static Z1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z1.bin")).expect("malformed saturn/z1.bin")
});

pub static Z2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z2.bin")).expect("malformed saturn/z2.bin")
});

pub static Z3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z3.bin")).expect("malformed saturn/z3.bin")
});

pub static Z4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z4.bin")).expect("malformed saturn/z4.bin")
});

pub static Z5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("saturn/z5.bin")).expect("malformed saturn/z5.bin")
});
