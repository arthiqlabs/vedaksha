// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Whole-Sign house system.
//!
//! The simplest house system: house 1 starts at 0° of the ASC's zodiac
//! sign. Each house occupies exactly one 30° sign.
//!
//! Source: Hellenistic tradition; Holden, *Elements of House Division*.

use vedaksha_math::angle::normalize_degrees;

use super::{HouseCusps, HouseSystem};

/// Compute Whole-Sign house cusps.
#[allow(clippy::cast_precision_loss)]
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let sign_start = (asc / 30.0).floor() * 30.0;
    let cusps = core::array::from_fn(|i| normalize_degrees(sign_start + (i as f64) * 30.0));
    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::WholeSign,
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
    fn cusp1_at_sign_boundary() {
        let result = compute(45.0, 30.0, 23.44);
        assert!(
            (result.cusps[0] % 30.0).abs() < 1e-10 || (30.0 - result.cusps[0] % 30.0).abs() < 1e-10,
            "cusp1={} not at sign boundary",
            result.cusps[0]
        );
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::WholeSign);
        assert!(!result.polar_fallback);
    }
}
