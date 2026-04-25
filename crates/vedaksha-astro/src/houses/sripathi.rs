// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Sripathi house system.
//!
//! Uses the midpoints of Porphyry house cusps. Specifically, each
//! Sripathi cusp is the midpoint between two consecutive Porphyry cusps.
//! This shifts each house boundary to the middle of the corresponding
//! Porphyry house, creating a system popular in Indian astrology.
//!
//! Source: B.V. Raman, *A Manual of Hindu Astrology*.

use vedaksha_math::angle::normalize_degrees;

use super::{HouseCusps, HouseSystem};

/// Compute the midpoint of two ecliptic longitudes, taking the shorter arc.
fn midpoint(a: f64, b: f64) -> f64 {
    let diff = normalize_degrees(b - a);
    normalize_degrees(a + diff / 2.0)
}

/// Compute Sripathi house cusps (midpoints of Porphyry cusps).
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let porphyry = super::porphyry::compute(ramc, latitude, obliquity);

    // Each Sripathi cusp i is the midpoint of Porphyry cusps i and i+1.
    let cusps = core::array::from_fn(|i| {
        let next = (i + 1) % 12;
        midpoint(porphyry.cusps[i], porphyry.cusps[next])
    });

    HouseCusps {
        cusps,
        asc: porphyry.asc,
        mc: porphyry.mc,
        system: HouseSystem::Sripathi,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_cusps_in_range() {
        let result = compute(30.0, 45.0, 23.44);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn cusps_between_porphyry() {
        let porphyry = super::super::porphyry::compute(30.0, 45.0, 23.44);
        let sripathi = compute(30.0, 45.0, 23.44);

        // Each Sripathi cusp should be between two consecutive Porphyry cusps
        for i in 0..12 {
            let next = (i + 1) % 12;
            let arc = normalize_degrees(porphyry.cusps[next] - porphyry.cusps[i]);
            let to_sri = normalize_degrees(sripathi.cusps[i] - porphyry.cusps[i]);
            assert!(
                to_sri < arc + 1e-6,
                "sripathi cusp {i} not between porphyry cusps: to_sri={to_sri} arc={arc}"
            );
        }
    }

    #[test]
    fn midpoint_fn_basic() {
        assert!((midpoint(10.0, 20.0) - 15.0).abs() < 1e-10);
        // Across 0°
        let m = midpoint(350.0, 10.0);
        assert!((m - 0.0).abs() < 1e-10 || (m - 360.0).abs() < 1e-10);
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Sripathi);
    }
}
