// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Porphyry house system.
//!
//! Trisects each quadrant between the angles (ASC, MC, DSC, IC).
//! Simple proportional division — no projection mathematics needed.
//!
//! Source: Holden, *Elements of House Division*, pp. 29-30.

use vedaksha_math::angle::normalize_degrees;

use super::{HouseCusps, HouseSystem};

/// Compute Porphyry house cusps by trisecting each quadrant.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let dsc = normalize_degrees(asc + 180.0);
    let ic = normalize_degrees(mc + 180.0);

    // Arc from ASC to IC (going forward through zodiac)
    let arc_asc_ic = normalize_degrees(ic - asc);
    // Arc from IC to DSC
    let arc_ic_dsc = normalize_degrees(dsc - ic);
    // Arc from DSC to MC
    let arc_dsc_mc = normalize_degrees(mc - dsc);
    // Arc from MC to ASC (next)
    let arc_mc_asc = normalize_degrees(asc - mc);

    let mut cusps = [0.0_f64; 12];
    cusps[0] = asc; // cusp 1 = ASC
    cusps[3] = ic; // cusp 4 = IC
    cusps[6] = dsc; // cusp 7 = DSC
    cusps[9] = mc; // cusp 10 = MC

    // Trisect quadrant ASC -> IC => cusps 2, 3
    cusps[1] = normalize_degrees(asc + arc_asc_ic / 3.0);
    cusps[2] = normalize_degrees(asc + 2.0 * arc_asc_ic / 3.0);

    // Trisect quadrant IC -> DSC => cusps 5, 6
    cusps[4] = normalize_degrees(ic + arc_ic_dsc / 3.0);
    cusps[5] = normalize_degrees(ic + 2.0 * arc_ic_dsc / 3.0);

    // Trisect quadrant DSC -> MC => cusps 8, 9
    cusps[7] = normalize_degrees(dsc + arc_dsc_mc / 3.0);
    cusps[8] = normalize_degrees(dsc + 2.0 * arc_dsc_mc / 3.0);

    // Trisect quadrant MC -> ASC => cusps 11, 12
    cusps[10] = normalize_degrees(mc + arc_mc_asc / 3.0);
    cusps[11] = normalize_degrees(mc + 2.0 * arc_mc_asc / 3.0);

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Porphyry,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_placed_correctly() {
        let result = compute(30.0, 45.0, 23.44);
        let dsc = normalize_degrees(result.asc + 180.0);
        let ic = normalize_degrees(result.mc + 180.0);
        assert!((result.cusps[0] - result.asc).abs() < 1e-10);
        assert!((result.cusps[6] - dsc).abs() < 1e-10);
        assert!((result.cusps[9] - result.mc).abs() < 1e-10);
        assert!((result.cusps[3] - ic).abs() < 1e-10);
    }

    #[test]
    fn all_cusps_in_range() {
        let result = compute(120.0, 52.0, 23.44);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Porphyry);
    }
}
