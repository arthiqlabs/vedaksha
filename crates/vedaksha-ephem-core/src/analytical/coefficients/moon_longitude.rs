// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: ELP/MPP02 (Chapront & Francou 2003, A&A 404, 735;
//         IMCCE explanatory note `elpmpp02.pdf`, October 2002).
// Distribution: ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/
// Component: moon longitude
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded at first access.

use std::sync::LazyLock;

use super::loader::{ElpMainTerm, ElpPertTerm, parse_elp_main, parse_elp_pert};

pub static MAIN: LazyLock<Vec<ElpMainTerm>> = LazyLock::new(|| {
    parse_elp_main(include_bytes!("moon_longitude/main.bin"))
        .expect("malformed moon_longitude/main.bin")
});

pub static PERT_0: LazyLock<Vec<ElpPertTerm>> = LazyLock::new(|| {
    parse_elp_pert(include_bytes!("moon_longitude/pert_0.bin"))
        .expect("malformed moon_longitude/pert_0.bin")
});

pub static PERT_1: LazyLock<Vec<ElpPertTerm>> = LazyLock::new(|| {
    parse_elp_pert(include_bytes!("moon_longitude/pert_1.bin"))
        .expect("malformed moon_longitude/pert_1.bin")
});

pub static PERT_2: LazyLock<Vec<ElpPertTerm>> = LazyLock::new(|| {
    parse_elp_pert(include_bytes!("moon_longitude/pert_2.bin"))
        .expect("malformed moon_longitude/pert_2.bin")
});

pub static PERT_3: LazyLock<Vec<ElpPertTerm>> = LazyLock::new(|| {
    parse_elp_pert(include_bytes!("moon_longitude/pert_3.bin"))
        .expect("malformed moon_longitude/pert_3.bin")
});
