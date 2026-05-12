// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Jupiter
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x0.bin")).expect("malformed jupiter/x0.bin")
});

pub static X1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x1.bin")).expect("malformed jupiter/x1.bin")
});

pub static X2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x2.bin")).expect("malformed jupiter/x2.bin")
});

pub static X3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x3.bin")).expect("malformed jupiter/x3.bin")
});

pub static X4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x4.bin")).expect("malformed jupiter/x4.bin")
});

pub static X5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/x5.bin")).expect("malformed jupiter/x5.bin")
});

pub static Y0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y0.bin")).expect("malformed jupiter/y0.bin")
});

pub static Y1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y1.bin")).expect("malformed jupiter/y1.bin")
});

pub static Y2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y2.bin")).expect("malformed jupiter/y2.bin")
});

pub static Y3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y3.bin")).expect("malformed jupiter/y3.bin")
});

pub static Y4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y4.bin")).expect("malformed jupiter/y4.bin")
});

pub static Y5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/y5.bin")).expect("malformed jupiter/y5.bin")
});

pub static Z0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z0.bin")).expect("malformed jupiter/z0.bin")
});

pub static Z1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z1.bin")).expect("malformed jupiter/z1.bin")
});

pub static Z2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z2.bin")).expect("malformed jupiter/z2.bin")
});

pub static Z3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z3.bin")).expect("malformed jupiter/z3.bin")
});

pub static Z4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z4.bin")).expect("malformed jupiter/z4.bin")
});

pub static Z5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("jupiter/z5.bin")).expect("malformed jupiter/z5.bin")
});
