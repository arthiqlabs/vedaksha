// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: Earth
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{Vsop87Term, parse_vsop87};

pub static X0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x0.bin")).expect("malformed earth/x0.bin"));

pub static X1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x1.bin")).expect("malformed earth/x1.bin"));

pub static X2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x2.bin")).expect("malformed earth/x2.bin"));

pub static X3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x3.bin")).expect("malformed earth/x3.bin"));

pub static X4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x4.bin")).expect("malformed earth/x4.bin"));

pub static X5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/x5.bin")).expect("malformed earth/x5.bin"));

pub static Y0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y0.bin")).expect("malformed earth/y0.bin"));

pub static Y1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y1.bin")).expect("malformed earth/y1.bin"));

pub static Y2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y2.bin")).expect("malformed earth/y2.bin"));

pub static Y3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y3.bin")).expect("malformed earth/y3.bin"));

pub static Y4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y4.bin")).expect("malformed earth/y4.bin"));

pub static Y5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/y5.bin")).expect("malformed earth/y5.bin"));

pub static Z0: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z0.bin")).expect("malformed earth/z0.bin"));

pub static Z1: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z1.bin")).expect("malformed earth/z1.bin"));

pub static Z2: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z2.bin")).expect("malformed earth/z2.bin"));

pub static Z3: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z3.bin")).expect("malformed earth/z3.bin"));

pub static Z4: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z4.bin")).expect("malformed earth/z4.bin"));

pub static Z5: LazyLock<Vec<Vsop87Term>> =
    LazyLock::new(|| parse_vsop87(include_bytes!("earth/z5.bin")).expect("malformed earth/z5.bin"));
