// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Mars
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x0.bin")).expect("malformed mars/x0.bin"));

pub static X1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x1.bin")).expect("malformed mars/x1.bin"));

pub static X2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x2.bin")).expect("malformed mars/x2.bin"));

pub static X3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x3.bin")).expect("malformed mars/x3.bin"));

pub static X4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x4.bin")).expect("malformed mars/x4.bin"));

pub static X5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/x5.bin")).expect("malformed mars/x5.bin"));

pub static Y0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y0.bin")).expect("malformed mars/y0.bin"));

pub static Y1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y1.bin")).expect("malformed mars/y1.bin"));

pub static Y2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y2.bin")).expect("malformed mars/y2.bin"));

pub static Y3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y3.bin")).expect("malformed mars/y3.bin"));

pub static Y4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y4.bin")).expect("malformed mars/y4.bin"));

pub static Y5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/y5.bin")).expect("malformed mars/y5.bin"));

pub static Z0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z0.bin")).expect("malformed mars/z0.bin"));

pub static Z1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z1.bin")).expect("malformed mars/z1.bin"));

pub static Z2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z2.bin")).expect("malformed mars/z2.bin"));

pub static Z3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z3.bin")).expect("malformed mars/z3.bin"));

pub static Z4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z4.bin")).expect("malformed mars/z4.bin"));

pub static Z5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("mars/z5.bin")).expect("malformed mars/z5.bin"));
