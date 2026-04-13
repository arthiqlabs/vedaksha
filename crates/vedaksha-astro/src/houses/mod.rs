// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! House system computation for Western astrology.
//!
//! Implements 10 house systems following Holden's *A History of Horoscopic
//! Astrology* and Ralph William Holden's *The Elements of House Division*.
//!
//! Each system divides the ecliptic into 12 houses starting from the
//! Ascendant (ASC). Systems differ in how intermediate cusps are derived.

mod alcabitius;
mod campanus;
mod equal;
mod koch;
mod morinus;
mod placidus;
mod porphyry;
mod regiomontanus;
mod sripathi;
mod whole_sign;

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

/// The 12 house cusps in degrees [0, 360).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HouseCusps {
    /// Cusps 1-12. `cusps[0]` = ASC (cusp 1), `cusps[9]` = MC (cusp 10)
    pub cusps: [f64; 12],
    /// Ascendant in degrees
    pub asc: f64,
    /// Midheaven (MC) in degrees
    pub mc: f64,
    /// The house system used
    pub system: HouseSystem,
    /// Warning if polar fallback was applied
    pub polar_fallback: bool,
}

/// Supported house systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HouseSystem {
    Placidus,
    Koch,
    Equal,
    WholeSign,
    Campanus,
    Regiomontanus,
    Porphyry,
    Morinus,
    Alcabitius,
    Sripathi,
}

/// Compute house cusps for a given location and time.
///
/// # Arguments
/// * `ramc` — Right Ascension of MC (RAMC) in degrees [0, 360)
/// * `latitude` — geographic latitude in degrees [-90, +90]
/// * `obliquity` — obliquity of the ecliptic in degrees
/// * `system` — which house system to use
#[must_use]
pub fn compute_houses(ramc: f64, latitude: f64, obliquity: f64, system: HouseSystem) -> HouseCusps {
    match system {
        HouseSystem::Placidus => placidus::compute(ramc, latitude, obliquity),
        HouseSystem::Koch => koch::compute(ramc, latitude, obliquity),
        HouseSystem::Equal => equal::compute(ramc, latitude, obliquity),
        HouseSystem::WholeSign => whole_sign::compute(ramc, latitude, obliquity),
        HouseSystem::Campanus => campanus::compute(ramc, latitude, obliquity),
        HouseSystem::Regiomontanus => regiomontanus::compute(ramc, latitude, obliquity),
        HouseSystem::Porphyry => porphyry::compute(ramc, latitude, obliquity),
        HouseSystem::Morinus => morinus::compute(ramc, latitude, obliquity),
        HouseSystem::Alcabitius => alcabitius::compute(ramc, latitude, obliquity),
        HouseSystem::Sripathi => sripathi::compute(ramc, latitude, obliquity),
    }
}

/// Compute ASC and MC from RAMC, latitude, and obliquity (all in degrees).
///
/// Returns `(asc, mc)` in degrees [0, 360).
///
/// # Formulae
///
/// **ASC** (Meeus Ch. 13 / Holden):
/// ```text
/// ASC = atan2(cos(RAMC), -(sin(RAMC)*cos(ε) + tan(φ)*sin(ε)))
/// ```
///
/// **MC**:
/// ```text
/// MC = atan(tan(RAMC) / cos(ε))
/// ```
/// with quadrant adjustment to match RAMC quadrant.
fn compute_asc_mc(ramc_deg: f64, lat_deg: f64, eps_deg: f64) -> (f64, f64) {
    let ramc = deg_to_rad(ramc_deg);
    let lat = deg_to_rad(lat_deg);
    let eps = deg_to_rad(eps_deg);

    let sin_ramc = libm::sin(ramc);
    let cos_ramc = libm::cos(ramc);
    let cos_eps = libm::cos(eps);
    let sin_eps = libm::sin(eps);
    let tan_lat = libm::tan(lat);

    // ASC
    let y = cos_ramc;
    let x = -(sin_ramc * cos_eps + tan_lat * sin_eps);
    let asc = normalize_degrees(rad_to_deg(libm::atan2(y, x)));

    // MC — use atan2 for correct quadrant:
    // MC = atan2(sin(RAMC), cos(RAMC)*cos(ε))
    // This is equivalent to atan(tan(RAMC)/cos(ε)) but handles all quadrants.
    let mc = normalize_degrees(rad_to_deg(libm::atan2(sin_ramc, cos_ramc * cos_eps)));

    (asc, mc)
}

/// Polar latitude threshold for Placidus/Koch fallback (degrees).
const POLAR_LAT_THRESHOLD: f64 = 66.56;

/// Maximum iterations for semi-arc methods.
const MAX_ITERATIONS: usize = 50;

/// Convergence threshold in degrees for iterative methods.
const CONVERGENCE_THRESHOLD: f64 = 1e-8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asc_mc_equator_vernal() {
        // RAMC=0, lat=0, eps=23.44
        // At equator with RAMC=0: MC=0 (vernal equinox on meridian)
        // ASC = atan2(cos(0), -(sin(0)*cos(e) + tan(0)*sin(e)))
        //     = atan2(1, 0) = 90° (east point of ecliptic)
        let (asc, mc) = compute_asc_mc(0.0, 0.0, 23.44);
        assert!((asc - 90.0).abs() < 1.0, "ASC={asc}");
        assert!(mc.abs() < 1.0 || (360.0 - mc).abs() < 1.0, "MC={mc}");
    }

    #[test]
    fn asc_mc_equator_90() {
        // RAMC=90, lat=0, eps=23.44
        let (asc, mc) = compute_asc_mc(90.0, 0.0, 23.44);
        // MC should be near 90° adjusted for obliquity
        assert!(mc > 80.0 && mc < 100.0, "MC={mc}");
        // ASC should be roughly 90° ahead
        assert!(asc > 170.0 && asc < 200.0, "ASC={asc}");
    }

    #[test]
    fn all_systems_produce_12_cusps() {
        let systems = [
            HouseSystem::Placidus,
            HouseSystem::Koch,
            HouseSystem::Equal,
            HouseSystem::WholeSign,
            HouseSystem::Campanus,
            HouseSystem::Regiomontanus,
            HouseSystem::Porphyry,
            HouseSystem::Morinus,
            HouseSystem::Alcabitius,
            HouseSystem::Sripathi,
        ];
        for sys in &systems {
            let result = compute_houses(30.0, 45.0, 23.44, *sys);
            assert_eq!(result.cusps.len(), 12, "system {:?}", sys);
            for (i, &cusp) in result.cusps.iter().enumerate() {
                assert!(
                    (0.0..360.0).contains(&cusp),
                    "system {:?}, cusp {i}: {cusp}",
                    sys
                );
            }
        }
    }

    #[test]
    fn mc_via_atan2_quadrant() {
        // RAMC in third quadrant (200 deg) should produce MC in Q3 range
        let (_, mc) = compute_asc_mc(200.0, 45.0, 23.44);
        assert!(
            mc >= 180.0 && mc < 360.0,
            "MC for RAMC=200 should be in [180, 360), got {mc}"
        );
    }
}
