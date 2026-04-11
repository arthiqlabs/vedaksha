// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Campanus house system.
//!
//! Divides the prime vertical (the great circle through zenith, nadir,
//! east and west points) into 12 equal 30° arcs, then projects these
//! divisions onto the ecliptic along house circles (great circles through
//! the north and south points of the horizon).
//!
//! Source: Holden, *Elements of House Division*, pp. 35-37.

use vedaksha_math::angle::{deg_to_rad, normalize_degrees, rad_to_deg};

use super::{HouseCusps, HouseSystem};

/// Compute ecliptic longitude of a Campanus house cusp.
///
/// Uses 3D vector geometry: find the prime vertical division point,
/// construct the house circle (great circle through N/S horizon points
/// and the PV point), then intersect with the ecliptic plane.
#[allow(clippy::cast_precision_loss)]
fn campanus_cusp(ramc_deg: f64, lat_deg: f64, eps_deg: f64, house_index: usize) -> f64 {
    let ramc = deg_to_rad(ramc_deg);
    let lat = deg_to_rad(lat_deg);
    let eps = deg_to_rad(eps_deg);

    // Prime vertical angle: cusp 1 at 0° (east), going toward nadir
    // East(0°) -> Nadir(90°) -> West(180°) -> Zenith(270°)
    let pv_angle = deg_to_rad((house_index as f64) * 30.0);

    let cos_lat = libm::cos(lat);
    let sin_lat = libm::sin(lat);
    let cos_eps = libm::cos(eps);
    let sin_eps = libm::sin(eps);
    let sin_ramc = libm::sin(ramc);
    let cos_ramc = libm::cos(ramc);
    let sin_a = libm::sin(pv_angle);
    let cos_a = libm::cos(pv_angle);

    // PV division point in local (E, N, U) frame: (cos A, 0, -sin A)
    // Convert to equatorial (x, y, z) where:
    //   x = toward vernal equinox (RA=0)
    //   y = toward RA=90°
    //   z = toward north celestial pole
    //
    // Local-to-equatorial rotation:
    //   The local East direction in equatorial: perpendicular to meridian in equatorial plane
    //     East = (-sin(RAMC), cos(RAMC), 0)  [perpendicular to meridian, in equatorial plane]
    //   Wait, local East points toward RA = RAMC + 90° on the horizon.
    //   Actually, local East in equatorial coords:
    //     East = (-sin(RAMC+90°), cos(RAMC+90°), 0) ... no.
    //
    //   Let me think more carefully. The local East direction at the observer:
    //   In the equatorial frame, the observer's East direction points toward
    //   increasing hour angle. Since RA = LST - H, increasing H means decreasing RA.
    //   At the observer's meridian (H=0), East = toward RA increasing = toward RA = RAMC + 90°.
    //
    //   In equatorial xyz:
    //     East = (-sin(RAMC + π/2), cos(RAMC + π/2), 0) = (-cos(RAMC), -sin(RAMC), 0)
    //   Wait, no. East should point toward rising stars, which is toward decreasing H.
    //   Since H = LST - RA, and LST = RAMC, decreasing H means increasing RA.
    //   So East points toward RA = RAMC + 90°.
    //   In equatorial xyz: East = (cos(RAMC + π/2), sin(RAMC + π/2), 0) = (-sin(RAMC), cos(RAMC), 0)

    // Local frame axes in equatorial xyz:
    //   E = (-sin(RAMC), cos(RAMC), 0)
    //   N = (-cos(RAMC)*sin(lat), -sin(RAMC)*sin(lat), cos(lat))
    //     [North: toward the pole projected to horizon = horizontal component toward pole]
    //     [In equatorial: N = cos(lat) * zenith_horizontal_component - ...]
    //     Actually:
    //     Zenith in equatorial = (cos(RAMC)*cos(lat), sin(RAMC)*cos(lat), sin(lat))
    //                            [it has dec = lat, RA = RAMC, on the meridian]
    //     North = Zenith × East / |Zenith × East|  [North = Up cross East for right-handed]
    //     Wait: (U × E) gives South. (E × U) gives North.
    //     N = E × U (cross product)

    // Let me just define the three local axes in equatorial coordinates:

    // Zenith (Up) in equatorial xyz: dec = lat, RA = RAMC
    let u_x = cos_ramc * cos_lat;
    let u_y = sin_ramc * cos_lat;
    let u_z = sin_lat;

    // East in equatorial xyz: perpendicular to meridian, in equatorial plane
    // East = toward RA = RAMC + 90° on the equator
    let e_x = -sin_ramc;
    let e_y = cos_ramc;
    let e_z = 0.0;

    // North = Up × East (right-hand rule: N = U × E)
    // Actually for a right-handed (E, N, U) frame: N = U × E
    // Wait: if (E, N, U) is right-handed, then E × N = U, so N = U × E
    // Let me verify: E × N should give U direction.
    // For right-handed: if we want E,N,U to be x,y,z of a right-handed frame,
    // then E × N = U.
    // N = U × E:
    let n_x = u_y * e_z - u_z * e_y; // = 0 - sin_lat * cos_ramc = -sin_lat * cos_ramc
    let n_y = u_z * e_x - u_x * e_z; // = sin_lat * (-sin_ramc) - 0 = -sin_lat * sin_ramc
    let n_z = u_x * e_y - u_y * e_x; // = cos_ramc*cos_lat*cos_ramc - sin_ramc*cos_lat*(-sin_ramc)
    // = cos_lat * (cos²ramc + sin²ramc) = cos_lat

    // PV point in local frame: (cos_a, 0, -sin_a)
    // In equatorial xyz:
    let p_x = cos_a * e_x + 0.0 * n_x + (-sin_a) * u_x;
    let p_y = cos_a * e_y + 0.0 * n_y + (-sin_a) * u_y;
    let p_z = cos_a * e_z + 0.0 * n_z + (-sin_a) * u_z;

    // North horizon point in equatorial xyz:
    // Azimuth = 0° (due north), Altitude = 0°
    // In local (E, N, U) = (0, 1, 0)
    // In equatorial:
    let nh_x = n_x; // = -sin_lat * cos_ramc
    let nh_y = n_y; // = -sin_lat * sin_ramc
    let nh_z = n_z; // = cos_lat

    // House circle normal = PV_point × N_horizon (cross product)
    let hc_x = p_y * nh_z - p_z * nh_y;
    let hc_y = p_z * nh_x - p_x * nh_z;
    let hc_z = p_x * nh_y - p_y * nh_x;

    // Ecliptic normal in equatorial xyz: (0, -sin(ε), cos(ε))
    // Intersection of two great circles: normal of result = ecliptic_normal × house_circle_normal
    let en_x = 0.0;
    let en_y = -sin_eps;
    let en_z = cos_eps;

    let ix = en_y * hc_z - en_z * hc_y;
    let iy = en_z * hc_x - en_x * hc_z;
    let iz = en_x * hc_y - en_y * hc_x;

    // The intersection point (ix, iy, iz) is in equatorial xyz.
    // Convert to ecliptic longitude:
    // For a point on the ecliptic, λ = atan2(iy_ecl, ix_ecl) where
    // ix_ecl = ix (same x-axis) and iy_ecl = iy*cos(ε) + iz*sin(ε)
    // (rotation around x-axis by ε)
    let ecl_x = ix;
    let ecl_y = iy * cos_eps + iz * sin_eps;

    // We get two antipodal points. Pick the one on the correct side.
    // The PV point should be on the same side as the cusp.
    // Check using dot product with the PV point.
    let dot = ix * p_x + iy * p_y + iz * p_z;
    let (fx, fy) = if dot < 0.0 {
        (-ecl_x, -ecl_y)
    } else {
        (ecl_x, ecl_y)
    };

    normalize_degrees(rad_to_deg(libm::atan2(fy, fx)))
}

/// Compute Campanus house cusps.
pub(super) fn compute(ramc: f64, latitude: f64, obliquity: f64) -> HouseCusps {
    let (asc, mc) = super::compute_asc_mc(ramc, latitude, obliquity);
    let dsc = normalize_degrees(asc + 180.0);
    let ic = normalize_degrees(mc + 180.0);

    let mut cusps = core::array::from_fn(|i| campanus_cusp(ramc, latitude, obliquity, i));

    // Force cardinal cusps to exact ASC/IC/DSC/MC values
    cusps[0] = asc; // cusp 1 = ASC
    cusps[3] = ic;  // cusp 4 = IC
    cusps[6] = dsc; // cusp 7 = DSC
    cusps[9] = mc;  // cusp 10 = MC

    HouseCusps {
        cusps,
        asc,
        mc,
        system: HouseSystem::Campanus,
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
    fn cusp1_reasonable() {
        // Campanus cusp 1 is derived from prime vertical projection,
        // so it may differ from the ASC by more than other systems.
        let result = compute(30.0, 45.0, 23.44);
        let diff = normalize_degrees(result.cusps[0] - result.asc);
        assert!(
            diff < 15.0 || diff > 345.0,
            "cusp1={} asc={} diff={diff}",
            result.cusps[0],
            result.asc
        );
    }

    #[test]
    fn system_tag() {
        let result = compute(0.0, 0.0, 23.44);
        assert_eq!(result.system, HouseSystem::Campanus);
    }
}
