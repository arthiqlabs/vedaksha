// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Koch house system.
//!
//! Also known as the "Birthplace" system. It uses the geographic latitude
//! and the MC to determine house cusps by computing the positions the MC
//! degree occupied at fractional intervals of its semi-arc before
//! culmination.
//!
//! Like Placidus, Koch breaks down at extreme latitudes and falls back
//! to Equal houses when |φ| > 66.56°.
//!
//! Source: Holden, *Elements of House Division*, pp. 55-58.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{HouseCusps, HouseSystem, POLAR_LAT_THRESHOLD};

/// Compute declination of an ecliptic longitude.
fn ecliptic_dec(lon_deg: f64, eps_deg: f64) -> f64 {
    let lon = deg_to_rad(lon_deg);
    let eps = deg_to_rad(eps_deg);
    rad_to_deg(libm::asin(libm::sin(lon) * libm::sin(eps)))
}

/// Koch cusp computation.
///
/// The Koch system works by finding what RAMC value would place the MC
/// at a specific fraction of its semi-arc, then computing the ASC for
/// that RAMC.
///
/// For cusp at fraction `f` of the semi-arc:
/// 1. Compute declination of MC: `dec_mc = asin(sin(MC)*sin(ε))`
/// 2. Compute semi-arc of MC: `SA_mc = acos(-tan(φ)*tan(dec_mc))`
/// 3. Compute the RAMC offset: `RAMC' = RAMC - f * SA_mc`
/// 4. The Koch cusp = ASC computed with RAMC'
///
/// For cusps below the horizon, use the nocturnal semi-arc and add
/// the offset.
fn koch_cusp(
    ramc_deg: f64,
    lat_deg: f64,
    eps_deg: f64,
    mc_deg: f64,
    fraction: f64,
    above: bool,
) -> Option<f64> {
    let dec_mc = ecliptic_dec(mc_deg, eps_deg);
    let lat = deg_to_rad(lat_deg);
    let dec = deg_to_rad(dec_mc);

    let arg = -libm::tan(lat) * libm::tan(dec);
    if !(-1.0..=1.0).contains(&arg) {
        return None; // circumpolar MC
    }

    let sa_mc = rad_to_deg(libm::acos(arg));

    let ramc_offset = if above {
        // Above horizon: go back from RAMC by fraction of diurnal SA
        normalize_degrees(ramc_deg - fraction * sa_mc)
    } else {
        // Below horizon: go forward from RAMC by fraction of diurnal SA
        normalize_degrees(ramc_deg + fraction * sa_mc)
    };

    // Compute the ASC for this modified RAMC
    let (asc, _) = super::compute_asc_mc(ramc_offset, lat_deg, eps_deg);
    Some(asc)
}

/// Compute Koch house cusps.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    // Polar fallback
    if libm::fabs(latitude) > POLAR_LAT_THRESHOLD {
        let mut result = super::equal::compute(ramc, latitude, obliquity);
        result.system = HouseSystem::Koch;
        result.polar_fallback = true;
        return result;
    }

    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let dsc = normalize_degrees(asc + 180.0);
    let ic = normalize_degrees(mc + 180.0);

    let mut cusps = [0.0_f64; 12];
    cusps[0] = asc;
    cusps[6] = dsc;
    cusps[9] = mc;
    cusps[3] = ic;

    // Koch fractions: f = fraction of MC's semi-arc offset from culmination
    // Cusp 11: 2/3 of diurnal SA backward from RAMC
    // Cusp 12: 1/3 of diurnal SA backward from RAMC
    // Cusp 2:  1/3 of nocturnal SA forward from RAMC
    // Cusp 3:  2/3 of nocturnal SA forward from RAMC
    let c11 = koch_cusp(ramc, latitude, obliquity, mc, 2.0 / 3.0, true);
    let c12 = koch_cusp(ramc, latitude, obliquity, mc, 1.0 / 3.0, true);
    let c2 = koch_cusp(ramc, latitude, obliquity, mc, 1.0 / 3.0, false);
    let c3 = koch_cusp(ramc, latitude, obliquity, mc, 2.0 / 3.0, false);

    if c11.is_none() || c12.is_none() || c2.is_none() || c3.is_none() {
        let mut result = super::equal::compute(ramc, latitude, obliquity);
        result.system = HouseSystem::Koch;
        result.polar_fallback = true;
        return result;
    }

    cusps[10] = c11.unwrap();
    cusps[11] = c12.unwrap();
    cusps[1] = c2.unwrap();
    cusps[2] = c3.unwrap();

    // Opposite cusps
    cusps[4] = normalize_degrees(cusps[10] + 180.0);
    cusps[5] = normalize_degrees(cusps[11] + 180.0);
    cusps[7] = normalize_degrees(cusps[1] + 180.0);
    cusps[8] = normalize_degrees(cusps[2] + 180.0);

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Koch,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equator_produces_cusps() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Koch);
        assert!(!result.polar_fallback);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn polar_fallback() {
        let result = compute(0.0, 70.0, 23.44);
        assert_eq!(result.system, HouseSystem::Koch);
        assert!(result.polar_fallback);
    }

    #[test]
    fn cusp1_equals_asc() {
        let result = compute(30.0, 45.0, 23.44);
        if !result.polar_fallback {
            assert!(
                (result.cusps[0] - result.asc).abs() < 1e-6,
                "cusp1={} asc={}",
                result.cusps[0],
                result.asc
            );
        }
    }

    #[test]
    fn cusp10_equals_mc() {
        let result = compute(30.0, 45.0, 23.44);
        if !result.polar_fallback {
            assert!(
                (result.cusps[9] - result.mc).abs() < 1e-6,
                "cusp10={} mc={}",
                result.cusps[9],
                result.mc
            );
        }
    }

    #[test]
    fn opposite_cusps() {
        let result = compute(90.0, 40.0, 23.44);
        if !result.polar_fallback {
            for i in 0..6 {
                let diff = normalize_degrees(result.cusps[i + 6] - result.cusps[i]);
                assert!(
                    (diff - 180.0).abs() < 0.01,
                    "cusps {i} and {}: diff={diff}",
                    i + 6
                );
            }
        }
    }

    #[test]
    fn mid_latitude() {
        let result = compute(120.0, 51.5, 23.44);
        assert!(!result.polar_fallback);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Koch);
    }
}
