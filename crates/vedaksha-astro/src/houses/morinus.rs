// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Morinus house system.
//!
//! Equal divisions of the celestial equator projected onto the ecliptic.
//! Each cusp is the ecliptic longitude corresponding to RAMC + i×30°.
//! This system is latitude-independent (geographic latitude does not
//! affect the cusps, though it still affects the ASC).
//!
//! Source: Holden, *Elements of House Division*, pp. 43-44.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{HouseCusps, HouseSystem};

/// Convert a right ascension on the equator to ecliptic longitude.
///
/// Projects the equator point (RA, dec=0) perpendicular to the ecliptic
/// to find the corresponding ecliptic longitude.
///
/// `λ = atan2(sin(RA) * cos(ε), cos(RA))`
fn ra_to_longitude(ra_deg: f64, eps_deg: f64) -> f64 {
    let ra = deg_to_rad(ra_deg);
    let eps = deg_to_rad(eps_deg);
    normalize_degrees(rad_to_deg(libm::atan2(
        libm::sin(ra) * libm::cos(eps),
        libm::cos(ra),
    )))
}

/// Compute Morinus house cusps.
///
/// Cusp i corresponds to ecliptic longitude of RAMC + (i+1)×30° on the
/// equator projected to ecliptic. Note: Morinus cusp 1 starts at
/// RAMC + 30° (the equatorial point 30° after the MC).
///
/// However the standard convention places cusp 10 = MC, so we use
/// cusp index offset such that cusps[9] ≈ MC.
#[allow(clippy::cast_precision_loss)]
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);

    // Morinus cusps: cusp_i = ecliptic longitude of (RAMC + (i-10)*30°)
    // so that cusp 10 (index 9) = longitude of RAMC = MC.
    // Equivalently, cusp 1 (index 0) = longitude of RAMC - 9*30° = RAMC - 270°
    // = RAMC + 90°.
    let cusps = core::array::from_fn(|i| {
        // cusp number is i+1, offset from MC (cusp 10) is (i+1 - 10) = i - 9
        let equator_ra = normalize_degrees(ramc + ((i as f64) - 9.0) * 30.0);
        ra_to_longitude(equator_ra, obliquity)
    });

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Morinus,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cusp10_near_mc() {
        // Morinus cusp 10 is the equatorial projection of RAMC onto the
        // ecliptic, which differs slightly from the MC (ecliptic point at
        // same RA as RAMC). The difference depends on obliquity.
        let result = compute(30.0, 45.0, 23.44);
        let diff = normalize_degrees(result.cusps[9] - result.mc);
        assert!(
            diff < 6.0 || diff > 354.0,
            "cusp10={} mc={}",
            result.cusps[9],
            result.mc
        );
    }

    #[test]
    fn latitude_independent_cusps() {
        let r1 = compute(60.0, 0.0, 23.44);
        let r2 = compute(60.0, 50.0, 23.44);
        // Cusps should be the same (latitude only affects ASC)
        for i in 0..12 {
            assert!(
                (r1.cusps[i] - r2.cusps[i]).abs() < 1e-10,
                "cusp {i}: {} vs {}",
                r1.cusps[i],
                r2.cusps[i]
            );
        }
    }

    #[test]
    fn all_cusps_in_range() {
        let result = compute(200.0, 30.0, 23.44);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Morinus);
    }
}
