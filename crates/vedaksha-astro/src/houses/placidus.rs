// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Placidus house system.
//!
//! The most widely used house system in Western astrology. It divides
//! the semi-arcs of the ecliptic into proportional segments using an
//! iterative method.
//!
//! For each intermediate cusp (11, 12, 2, 3), the algorithm finds the
//! ecliptic longitude `L` where the ratio of the hour angle to the
//! semi-arc equals a specific fraction (1/3 or 2/3).
//!
//! At polar latitudes (|φ| > 66.56°), the semi-arc method breaks down
//! because some ecliptic degrees never rise or set. In these cases we
//! fall back to Equal houses with `polar_fallback = true`.
//!
//! Source: Holden, *Elements of House Division*, pp. 47-52.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{CONVERGENCE_THRESHOLD, HouseCusps, HouseSystem, MAX_ITERATIONS, POLAR_LAT_THRESHOLD};

/// Compute right ascension of an ecliptic longitude.
///
/// `RA = atan2(sin(λ)*cos(ε), cos(λ))`
fn _ecliptic_ra(lon_deg: f64, eps_deg: f64) -> f64 {
    let lon = deg_to_rad(lon_deg);
    let eps = deg_to_rad(eps_deg);
    normalize_degrees(rad_to_deg(libm::atan2(
        libm::sin(lon) * libm::cos(eps),
        libm::cos(lon),
    )))
}

/// Compute declination of an ecliptic longitude.
///
/// `dec = asin(sin(λ)*sin(ε))`
fn ecliptic_dec(lon_deg: f64, eps_deg: f64) -> f64 {
    let lon = deg_to_rad(lon_deg);
    let eps = deg_to_rad(eps_deg);
    rad_to_deg(libm::asin(libm::sin(lon) * libm::sin(eps)))
}

/// Compute the diurnal semi-arc (in degrees) for a given declination
/// at a given latitude.
///
/// `SA = acos(-tan(φ)*tan(δ))`
///
/// Returns `None` if the body is circumpolar (never rises/sets).
fn semi_arc(lat_deg: f64, dec_deg: f64) -> Option<f64> {
    let lat = deg_to_rad(lat_deg);
    let dec = deg_to_rad(dec_deg);
    let arg = -libm::tan(lat) * libm::tan(dec);
    if !(-1.0..=1.0).contains(&arg) {
        return None;
    }
    Some(rad_to_deg(libm::acos(arg)))
}

/// Find a Placidus cusp by iterating on Right Ascension.
///
/// For cusps above the horizon (11, 12), we find the ecliptic longitude
/// whose RA satisfies: `RA(λ) = RAMC + f × SA(λ)`, where `f` is the
/// fractional distance from MC toward ASC through the diurnal semi-arc.
///
/// For cusps below the horizon (2, 3), the condition is:
/// `RA(λ) = RAIC - (1-f) × NSA(λ)`, where `NSA = 180° - SA` and `f`
/// is the fractional distance from ASC toward IC.
///
/// The algorithm iterates on RA (which changes linearly in the time
/// domain), converting to ecliptic longitude at each step to recompute
/// the declination and semi-arc.  Converges in 3-5 iterations.
///
/// * `fraction` — 1/3 or 2/3
/// * `above` — true for cusps 11, 12 (above horizon); false for 2, 3
///
/// Returns `None` if the cusp is circumpolar (never rises/sets).
fn find_cusp(ramc_deg: f64, lat_deg: f64, eps_deg: f64, fraction: f64, above: bool) -> Option<f64> {
    let eps = deg_to_rad(eps_deg);

    // Initial estimate: use MC's declination for initial SA
    let (_, mc) = super::compute_asc_mc(ramc_deg, lat_deg, eps_deg);
    let mc_dec = ecliptic_dec(mc, eps_deg);
    let initial_sa = semi_arc(lat_deg, mc_dec)?;

    // Start with estimated RA
    let mut ra = if above {
        // Above horizon: RA = RAMC + fraction * SA
        // (cusp 11 at 1/3 SA east of MC, cusp 12 at 2/3 SA)
        normalize_degrees(ramc_deg + fraction * initial_sa)
    } else {
        // Below horizon: RA = RAIC - (1 - fraction) * NSA
        // (cusp 2 at f=1/3 => 2/3 NSA west of IC, cusp 3 at f=2/3 => 1/3 NSA west)
        let raic = normalize_degrees(ramc_deg + 180.0);
        normalize_degrees(raic - (1.0 - fraction) * (180.0 - initial_sa))
    };

    for _ in 0..MAX_ITERATIONS {
        // Convert RA to ecliptic longitude
        // For a point on the ecliptic (lat=0): λ = atan2(sin(RA), cos(RA)*cos(ε))
        let ra_rad = deg_to_rad(ra);
        let lon_rad = libm::atan2(
            libm::sin(ra_rad),
            libm::cos(ra_rad) * libm::cos(eps),
        );
        let lon = normalize_degrees(rad_to_deg(lon_rad));

        // Get declination from this longitude
        let dec = ecliptic_dec(lon, eps_deg);

        // Get semi-arc
        let sa = match semi_arc(lat_deg, dec) {
            Some(sa) => sa,
            None => return None, // Circumpolar
        };

        // Compute target RA
        let new_ra = if above {
            normalize_degrees(ramc_deg + fraction * sa)
        } else {
            let raic = normalize_degrees(ramc_deg + 180.0);
            normalize_degrees(raic - (1.0 - fraction) * (180.0 - sa))
        };

        // Check convergence
        let mut diff = new_ra - ra;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }

        if libm::fabs(diff) < CONVERGENCE_THRESHOLD {
            // Convert final RA to ecliptic longitude
            let final_ra_rad = deg_to_rad(new_ra);
            let final_lon = libm::atan2(
                libm::sin(final_ra_rad),
                libm::cos(final_ra_rad) * libm::cos(eps),
            );
            return Some(normalize_degrees(rad_to_deg(final_lon)));
        }

        ra = new_ra;
    }

    // Return best estimate even if not fully converged
    let ra_rad = deg_to_rad(ra);
    let lon = libm::atan2(
        libm::sin(ra_rad),
        libm::cos(ra_rad) * libm::cos(eps),
    );
    Some(normalize_degrees(rad_to_deg(lon)))
}

/// Compute Placidus house cusps.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    // Polar fallback
    if libm::fabs(latitude) > POLAR_LAT_THRESHOLD {
        let mut result = super::equal::compute(ramc, latitude, obliquity);
        result.system = HouseSystem::Placidus;
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

    // Cusp 11 (index 10): fraction = 1/3, above horizon
    // Cusp 12 (index 11): fraction = 2/3, above horizon
    // Cusp 2  (index  1): fraction = 1/3, below horizon
    // Cusp 3  (index  2): fraction = 2/3, below horizon
    let c11 = find_cusp(ramc, latitude, obliquity, 1.0 / 3.0, true);
    let c12 = find_cusp(ramc, latitude, obliquity, 2.0 / 3.0, true);
    let c2 = find_cusp(ramc, latitude, obliquity, 1.0 / 3.0, false);
    let c3 = find_cusp(ramc, latitude, obliquity, 2.0 / 3.0, false);

    // If any cusp failed, fall back to equal
    if c11.is_none() || c12.is_none() || c2.is_none() || c3.is_none() {
        let mut result = super::equal::compute(ramc, latitude, obliquity);
        result.system = HouseSystem::Placidus;
        result.polar_fallback = true;
        return result;
    }

    cusps[10] = c11.unwrap();
    cusps[11] = c12.unwrap();
    cusps[1] = c2.unwrap();
    cusps[2] = c3.unwrap();

    // Cusps 5, 6, 8, 9 are opposite
    cusps[4] = normalize_degrees(cusps[10] + 180.0);
    cusps[5] = normalize_degrees(cusps[11] + 180.0);
    cusps[7] = normalize_degrees(cusps[1] + 180.0);
    cusps[8] = normalize_degrees(cusps[2] + 180.0);

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Placidus,
        polar_fallback: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equator_produces_cusps() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Placidus);
        assert!(!result.polar_fallback);
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }

    #[test]
    fn polar_fallback() {
        let result = compute(0.0, 70.0, 23.44);
        assert_eq!(result.system, HouseSystem::Placidus);
        assert!(result.polar_fallback, "should fallback at lat=70");
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
        // Cusps should all be valid
        for (i, &c) in result.cusps.iter().enumerate() {
            assert!((0.0..360.0).contains(&c), "cusp {i}: {c}");
        }
    }
}
