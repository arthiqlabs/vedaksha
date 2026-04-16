// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Alcabitius (Alchabitius) house system.
//!
//! Divides the diurnal and nocturnal semi-arcs of the Ascendant into
//! three equal parts on the celestial equator, then projects to ecliptic.
//!
//! The semi-arc of the ASC is the time (in equatorial degrees) it takes
//! from rising to culmination (diurnal semi-arc) or from culmination to
//! setting (nocturnal semi-arc).
//!
//! Source: Holden, *Elements of House Division*, pp. 41-42.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{HouseCusps, HouseSystem};

/// Compute the diurnal semi-arc (DSA) from declination and latitude.
///
/// `DSA = acos(-tan(φ) * tan(δ))` in degrees.
/// Returns 90° if the argument is out of range (circumpolar).
fn diurnal_semi_arc(lat_deg: f64, dec_deg: f64) -> f64 {
    let lat = deg_to_rad(lat_deg);
    let dec = deg_to_rad(dec_deg);
    let arg = -libm::tan(lat) * libm::tan(dec);
    if !(-1.0..=1.0).contains(&arg) {
        return 90.0; // circumpolar fallback
    }
    rad_to_deg(libm::acos(arg))
}

/// Compute ecliptic longitude from an equatorial RA.
///
/// Finds the ecliptic longitude of the point on the ecliptic that has
/// the given RA. Uses atan2 for correct quadrant handling.
///
/// `λ = atan2(sin(RA), cos(RA) * cos(ε))`
fn equator_to_ecliptic(ra_deg: f64, eps_deg: f64) -> f64 {
    let ra = deg_to_rad(ra_deg);
    let eps = deg_to_rad(eps_deg);
    normalize_degrees(rad_to_deg(libm::atan2(
        libm::sin(ra),
        libm::cos(ra) * libm::cos(eps),
    )))
}

/// Compute Alcabitius house cusps.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);

    // Compute RA and declination of the Ascendant
    let asc_rad = deg_to_rad(asc);
    let eps_rad = deg_to_rad(obliquity);
    // RA of ASC: atan2(sin(λ)*cos(ε), cos(λ))
    let ra_asc = normalize_degrees(rad_to_deg(libm::atan2(
        libm::sin(asc_rad) * libm::cos(eps_rad),
        libm::cos(asc_rad),
    )));
    // Declination of ASC: asin(sin(λ)*sin(ε))
    let dec_asc = rad_to_deg(libm::asin(libm::sin(asc_rad) * libm::sin(eps_rad)));

    // Diurnal semi-arc of the ASC
    let dsa = diurnal_semi_arc(latitude, dec_asc);
    let nsa = 180.0 - dsa; // nocturnal semi-arc

    let mut cusps = [0.0_f64; 12];
    cusps[0] = asc;
    cusps[9] = mc;

    let dsc = normalize_degrees(asc + 180.0);
    let ic = normalize_degrees(mc + 180.0);
    cusps[6] = dsc;
    cusps[3] = ic;

    // Cusps 11, 12 are 1/3 and 2/3 of the diurnal semi-arc
    // from MC toward ASC on the equator.
    // RA of cusp 11 = RAMC + DSA/3
    // RA of cusp 12 = RAMC + 2*DSA/3
    let ra_11 = normalize_degrees(ramc + dsa / 3.0);
    let ra_12 = normalize_degrees(ramc + 2.0 * dsa / 3.0);

    cusps[10] = equator_to_ecliptic(ra_11, obliquity);
    cusps[11] = equator_to_ecliptic(ra_12, obliquity);

    // Cusps 2, 3 are 1/3 and 2/3 of the nocturnal semi-arc
    // from ASC toward IC on the equator.
    // RA of ASC ≈ RAMC + DSA (since ASC rises DSA degrees after MC)
    // RA of cusp 2 = ra_asc + NSA/3
    // RA of cusp 3 = ra_asc + 2*NSA/3
    let ra_cusp2 = normalize_degrees(ra_asc + nsa / 3.0);
    let ra_cusp3 = normalize_degrees(ra_asc + 2.0 * nsa / 3.0);

    cusps[1] = equator_to_ecliptic(ra_cusp2, obliquity);
    cusps[2] = equator_to_ecliptic(ra_cusp3, obliquity);

    // Cusps 5, 6 = opposite of cusps 11, 12
    cusps[4] = normalize_degrees(cusps[10] + 180.0);
    cusps[5] = normalize_degrees(cusps[11] + 180.0);

    // Cusps 8, 9 = opposite of cusps 2, 3
    cusps[7] = normalize_degrees(cusps[1] + 180.0);
    cusps[8] = normalize_degrees(cusps[2] + 180.0);

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Alcabitius,
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
    fn cusp1_equals_asc() {
        let result = compute(60.0, 40.0, 23.44);
        assert!(
            (result.cusps[0] - result.asc).abs() < 1e-10,
            "cusp1={} asc={}",
            result.cusps[0],
            result.asc
        );
    }

    #[test]
    fn cusp10_equals_mc() {
        let result = compute(60.0, 40.0, 23.44);
        assert!(
            (result.cusps[9] - result.mc).abs() < 1e-10,
            "cusp10={} mc={}",
            result.cusps[9],
            result.mc
        );
    }

    #[test]
    fn opposite_cusps() {
        let result = compute(100.0, 30.0, 23.44);
        // cusp 7 should be opposite cusp 1
        let diff = normalize_degrees(result.cusps[6] - result.cusps[0]);
        assert!((diff - 180.0).abs() < 1e-6, "cusp7-cusp1 diff={diff}");
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Alcabitius);
    }
}
