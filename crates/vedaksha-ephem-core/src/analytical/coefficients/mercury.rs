// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Mercury
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x0.bin")).expect("malformed mercury/x0.bin")
});

pub static X1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x1.bin")).expect("malformed mercury/x1.bin")
});

pub static X2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x2.bin")).expect("malformed mercury/x2.bin")
});

pub static X3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x3.bin")).expect("malformed mercury/x3.bin")
});

pub static X4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x4.bin")).expect("malformed mercury/x4.bin")
});

pub static X5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/x5.bin")).expect("malformed mercury/x5.bin")
});

pub static Y0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y0.bin")).expect("malformed mercury/y0.bin")
});

pub static Y1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y1.bin")).expect("malformed mercury/y1.bin")
});

pub static Y2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y2.bin")).expect("malformed mercury/y2.bin")
});

pub static Y3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y3.bin")).expect("malformed mercury/y3.bin")
});

pub static Y4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y4.bin")).expect("malformed mercury/y4.bin")
});

pub static Y5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/y5.bin")).expect("malformed mercury/y5.bin")
});

pub static Z0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z0.bin")).expect("malformed mercury/z0.bin")
});

pub static Z1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z1.bin")).expect("malformed mercury/z1.bin")
});

pub static Z2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z2.bin")).expect("malformed mercury/z2.bin")
});

pub static Z3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z3.bin")).expect("malformed mercury/z3.bin")
});

pub static Z4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z4.bin")).expect("malformed mercury/z4.bin")
});

pub static Z5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("mercury/z5.bin")).expect("malformed mercury/z5.bin")
});
