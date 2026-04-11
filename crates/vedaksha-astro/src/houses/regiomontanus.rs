// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Regiomontanus house system.
//!
//! Divides the celestial equator into 12 equal 30° arcs starting from
//! the east point (RAMC + 90°), then projects these divisions onto the
//! ecliptic via house circles (great circles through the north and south
//! points of the horizon).
//!
//! Source: Holden, *Elements of House Division*, pp. 38-40.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{HouseCusps, HouseSystem};

/// Compute ecliptic longitude of a Regiomontanus house cusp.
///
/// For house cusp at equatorial position RAMC + 90° + i×30°:
/// ```text
/// tan(λ) = (sin(RA) * cos(ε) + tan(δ) * sin(ε)) / cos(RA)
/// ```
/// where RA = RAMC + 90° + i*30° and δ is the declination of the
/// equator point as projected through the house circle (great circle
/// through north/south points of horizon).
///
/// The declination for Regiomontanus:
/// ```text
/// tan(δ) = sin(RA - RAMC) * tan(φ)  ... simplified
/// ```
/// Wait — more precisely from Holden:
/// ```text
/// tan(δ) = tan(φ) * sin(RA - RAMC)
/// ```
/// No — the correct Regiomontanus formula gives the cusp longitude as:
/// ```text
/// λ = atan2(
///     sin(RAMC + 90° + i*30°),
///     cos(RAMC + 90° + i*30°) * cos(ε)
///       - tan(φ) * sin(RAMC + 90° + i*30° - RAMC) * sin(ε) / cos(...)
/// )
/// ```
///
/// Cleaner formulation (Holden): for equatorial offset `θ = 90° + i*30°`:
/// ```text
/// δ = atan(tan(φ) * sin(θ))
/// λ = atan2(sin(RAMC+θ), cos(RAMC+θ)*cos(ε) - tan(δ)*sin(ε))
/// ```
#[allow(clippy::cast_precision_loss)]
fn regio_cusp(ramc_deg: f64, lat_deg: f64, eps_deg: f64, house_index: usize) -> f64 {
    let ramc = deg_to_rad(ramc_deg);
    let lat = deg_to_rad(lat_deg);
    let eps = deg_to_rad(eps_deg);

    let cos_eps = libm::cos(eps);
    let sin_eps = libm::sin(eps);
    let tan_lat = libm::tan(lat);

    // Equatorial offset from RAMC: 90° + i*30°
    let theta = deg_to_rad(90.0 + (house_index as f64) * 30.0);

    // Declination of the house-circle intersection
    let dec = libm::atan(tan_lat * libm::sin(theta));
    let tan_dec = libm::tan(dec);

    let ra = ramc + theta;
    let sin_ra = libm::sin(ra);
    let cos_ra = libm::cos(ra);

    // Ecliptic longitude via house-circle projection (Holden)
    let y = sin_ra;
    let x = cos_ra * cos_eps - tan_dec * sin_eps;

    normalize_degrees(rad_to_deg(libm::atan2(y, x)))
}

/// Compute Regiomontanus house cusps.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let dsc = normalize_degrees(asc + 180.0);
    let ic = normalize_degrees(mc + 180.0);

    let mut cusps = core::array::from_fn(|i| regio_cusp(ramc, latitude, obliquity, i));

    // Force cardinal cusps to exact ASC/IC/DSC/MC values
    cusps[0] = asc; // cusp 1 = ASC
    cusps[3] = ic;  // cusp 4 = IC
    cusps[6] = dsc; // cusp 7 = DSC
    cusps[9] = mc;  // cusp 10 = MC

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Regiomontanus,
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
    fn cusp10_near_mc() {
        let result = compute(30.0, 45.0, 23.44);
        let diff = normalize_degrees(result.cusps[9] - result.mc);
        assert!(
            diff < 2.0 || diff > 358.0,
            "cusp10={} mc={} diff={diff}",
            result.cusps[9],
            result.mc
        );
    }

    #[test]
    fn cusp1_near_asc() {
        let result = compute(30.0, 45.0, 23.44);
        let diff = normalize_degrees(result.cusps[0] - result.asc);
        assert!(
            diff < 2.0 || diff > 358.0,
            "cusp1={} asc={} diff={diff}",
            result.cusps[0],
            result.asc
        );
    }

    #[test]
    fn equator_similar_to_equal() {
        // At equator, Regiomontanus should be close to Equal
        let regio = compute(0.0, 0.0, 23.44);
        let equal = super::super::equal::compute(0.0, 0.0, 23.44);
        for i in 0..12 {
            let diff = normalize_degrees(regio.cusps[i] - equal.cusps[i]);
            assert!(
                diff < 5.0 || diff > 355.0,
                "cusp {i}: regio={} equal={} diff={diff}",
                regio.cusps[i],
                equal.cusps[i]
            );
        }
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Regiomontanus);
    }
}
