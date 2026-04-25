// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Equal house system.
//!
//! Each house spans exactly 30°, starting from the Ascendant.
//! The MC floats and is not necessarily at cusp 10.
//!
//! Source: Holden, *Elements of House Division*.

use vedaksha_math::angle::normalize_degrees;

use super::{HouseCusps, HouseSystem};

/// Compute Equal house cusps.
#[allow(clippy::cast_precision_loss)]
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let cusps = core::array::from_fn(|i| normalize_degrees(asc + (i as f64) * 30.0));
    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Equal,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cusps_30_apart() {
        let result = compute(0.0, 0.0, 23.44);
        for i in 0..12 {
            let next = (i + 1) % 12;
            let diff = normalize_degrees(result.cusps[next] - result.cusps[i]);
            assert!(
                (diff - 30.0).abs() < 1e-10,
                "cusps {i}->{next}: diff={diff}"
            );
        }
    }

    #[test]
    fn cusp1_equals_asc() {
        let result = compute(45.0, 30.0, 23.44);
        assert!(
            (result.cusps[0] - result.asc).abs() < 1e-10,
            "cusp1={} asc={}",
            result.cusps[0],
            result.asc
        );
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Equal);
        assert!(!result.polar_fallback);
    }
}
