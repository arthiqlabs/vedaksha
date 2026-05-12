// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Uranus
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x0.bin")).expect("malformed uranus/x0.bin")
});

pub static X1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x1.bin")).expect("malformed uranus/x1.bin")
});

pub static X2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x2.bin")).expect("malformed uranus/x2.bin")
});

pub static X3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x3.bin")).expect("malformed uranus/x3.bin")
});

pub static X4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x4.bin")).expect("malformed uranus/x4.bin")
});

pub static X5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/x5.bin")).expect("malformed uranus/x5.bin")
});

pub static Y0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y0.bin")).expect("malformed uranus/y0.bin")
});

pub static Y1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y1.bin")).expect("malformed uranus/y1.bin")
});

pub static Y2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y2.bin")).expect("malformed uranus/y2.bin")
});

pub static Y3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y3.bin")).expect("malformed uranus/y3.bin")
});

pub static Y4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y4.bin")).expect("malformed uranus/y4.bin")
});

pub static Y5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/y5.bin")).expect("malformed uranus/y5.bin")
});

pub static Z0: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z0.bin")).expect("malformed uranus/z0.bin")
});

pub static Z1: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z1.bin")).expect("malformed uranus/z1.bin")
});

pub static Z2: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z2.bin")).expect("malformed uranus/z2.bin")
});

pub static Z3: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z3.bin")).expect("malformed uranus/z3.bin")
});

pub static Z4: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z4.bin")).expect("malformed uranus/z4.bin")
});

pub static Z5: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {
    parse_vsop87(include_bytes!("uranus/z5.bin")).expect("malformed uranus/z5.bin")
});
