// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Analytical ephemeris models.
//!
//! This module provides planetary and lunar positions computed from analytical
//! series expansions, requiring no external data files at runtime:
//!
//! - **VSOP87A** (Bretagnon & Francou 1988) — heliocentric rectangular
//!   ecliptic coordinates (J2000.0) for Mercury through Neptune
//! - **ELP/MPP02** (Chapront 2002) — geocentric lunar coordinates
//!
//! The [`AnalyticalProvider`] struct implements [`EphemerisProvider`] to wire
//! these evaluators into the unified trait interface.

pub mod coefficients;
pub mod elp_mpp02;
mod simd_trig;
pub mod vsop87a;

use crate::bodies::Body;
use crate::error::ComputeError;
use crate::jpl::{AU_KM, EphemerisProvider, Position, StateVector, Velocity};

use self::vsop87a::Planet;

/// J2000.0 obliquity in radians: 84381.406 arcsec.
/// Used to derive COS_EPS and SIN_EPS; kept for documentation.
#[allow(dead_code)]
const OBLIQUITY_J2000: f64 = 84_381.406 * core::f64::consts::PI / (180.0 * 3600.0);

/// Earth-Moon mass ratio (DE440/441 value).
const EMRAT: f64 = 81.300_568_94;

/// Julian millennia to days conversion factor.
const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// Minimum supported Julian Day (~-2000 CE).
const JD_MIN: f64 = 990_575.0;

/// Maximum supported Julian Day (~+3000 CE).
const JD_MAX: f64 = 2_816_788.0;

/// Precomputed cosine of J2000 obliquity.
const COS_EPS: f64 = 0.917_482_062_074_920_2; // libm::cos(OBLIQUITY_J2000)

/// Precomputed sine of J2000 obliquity.
const SIN_EPS: f64 = 0.397_777_155_931_913_7; // libm::sin(OBLIQUITY_J2000)

/// Rotate a vector from ecliptic (J2000) to equatorial (ICRS approximation).
#[inline]
fn ecliptic_to_equatorial(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (x, y * COS_EPS - z * SIN_EPS, y * SIN_EPS + z * COS_EPS)
}

/// Convert VSOP87A planet enum from Body enum.
///
/// `EarthMoonBarycenter` maps to VSOP87A's `ear` series, which is the **Earth's
/// centre** and not the barycentre. Callers must lift it — see
/// [`earth_to_emb`]. Returning it raw is what put the observer 4,671 km out
/// until 2026-08-20.
fn body_to_vsop_planet(body: Body) -> Option<Planet> {
    match body {
        Body::Mercury => Some(Planet::Mercury),
        Body::Venus => Some(Planet::Venus),
        Body::EarthMoonBarycenter => Some(Planet::Earth),
        Body::Mars => Some(Planet::Mars),
        Body::Jupiter => Some(Planet::Jupiter),
        Body::Saturn => Some(Planet::Saturn),
        Body::Uranus => Some(Planet::Uranus),
        Body::Neptune => Some(Planet::Neptune),
        _ => None,
    }
}

/// Compute a VSOP87A planet state in barycentric equatorial (ICRS) frame.
///
/// Heliocentric ecliptic → equatorial rotation. Heliocentric ≈ barycentric
/// (Sun ≈ SSB; error < 0.5 arcsec for inner planets, acceptable budget).
fn vsop_state(planet: Planet, jd: f64) -> StateVector {
    let (pos_ecl, vel_ecl) = vsop87a::vsop87a_heliocentric(planet, jd);

    let (px, py, pz) = ecliptic_to_equatorial(pos_ecl[0], pos_ecl[1], pos_ecl[2]);

    // Velocity: AU/millennium → AU/day
    let vx_day = vel_ecl[0] / DAYS_PER_MILLENNIUM;
    let vy_day = vel_ecl[1] / DAYS_PER_MILLENNIUM;
    let vz_day = vel_ecl[2] / DAYS_PER_MILLENNIUM;
    let (vx, vy, vz) = ecliptic_to_equatorial(vx_day, vy_day, vz_day);

    StateVector {
        position: Position {
            x: px,
            y: py,
            z: pz,
        },
        velocity: Velocity {
            x: vx,
            y: vy,
            z: vz,
        },
    }
}

/// Compute Moon position relative to EMB in equatorial (ICRS) frame.
///
/// The coordinates pipeline (`coordinates.rs`) expects `compute_state(Body::Moon)`
/// to return the Moon's position **relative to EMB** (Earth-Moon Barycenter),
/// matching the SPK convention (center=3). The conversion from geocentric is:
///
///   Moon_rel_EMB = Moon_geocentric * EMRAT / (1 + EMRAT)
///
/// Steps:
/// 1. Moon geocentric from ELP/MPP02 (J2000 ecliptic rectangular, km)
/// 2. Convert km → AU
/// 3. Scale by EMRAT/(1+EMRAT) to get Moon relative to EMB
/// 4. Rotate J2000 ecliptic → equatorial
fn moon_state(jd: f64) -> StateVector {
    // Moon geocentric in J2000 ecliptic rectangular (km, km/day)
    let moon = elp_mpp02::elp_geocentric(jd);

    // Convert km → AU
    let mg_x = moon.x / AU_KM;
    let mg_y = moon.y / AU_KM;
    let mg_z = moon.z / AU_KM;
    let mg_vx = moon.vx / AU_KM;
    let mg_vy = moon.vy / AU_KM;
    let mg_vz = moon.vz / AU_KM;

    // Convert geocentric → relative to EMB:
    // Moon_rel_EMB = Moon_geocentric * EMRAT / (1 + EMRAT)
    let emb_factor = EMRAT / (1.0 + EMRAT);

    let rel_x = mg_x * emb_factor;
    let rel_y = mg_y * emb_factor;
    let rel_z = mg_z * emb_factor;

    let rel_vx = mg_vx * emb_factor;
    let rel_vy = mg_vy * emb_factor;
    let rel_vz = mg_vz * emb_factor;

    // J2000 ecliptic �� equatorial
    let (px, py, pz) = ecliptic_to_equatorial(rel_x, rel_y, rel_z);
    let (vx, vy, vz) = ecliptic_to_equatorial(rel_vx, rel_vy, rel_vz);

    StateVector {
        position: Position {
            x: px,
            y: py,
            z: pz,
        },
        velocity: Velocity {
            x: vx,
            y: vy,
            z: vz,
        },
    }
}

/// Analytical ephemeris provider backed by VSOP87A and ELP/MPP02.
///
/// This provider requires no external data files. It covers the range
/// ~-2000 CE to ~+3000 CE with sub-arcsecond accuracy for most bodies.
///
/// # Approximations
///
/// - **Sun ≈ SSB**: The Sun's position is returned as (0,0,0) in the
///   barycentric frame. The true Sun-SSB offset is < 0.01 AU, well within
///   the 0.5 arcsecond error budget for astrological purposes.
/// - **Heliocentric ≈ barycentric**: Planetary positions from VSOP87A are
///   heliocentric; the Sun ≈ SSB approximation makes these effectively
///   barycentric.
#[derive(Debug, Clone, Copy)]
pub struct AnalyticalProvider;

impl AnalyticalProvider {
    /// Create a new analytical ephemeris provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyticalProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Lift VSOP87A's Earth-centre state to the Earth-Moon barycentre.
///
/// VSOP87A's `ear` series gives the Earth's centre. Every consumer of this
/// trait — `coordinates::earth_state` above all — expects `EarthMoonBarycenter`
/// to mean the barycentre, which is what the SPK path genuinely stores. The
/// offset is `Moon_rel_EMB / EMRAT`, about 4,671 km:
///
/// ```text
/// EMB = Earth + r/(1+EMRAT) = Earth + Moon_rel_EMB/EMRAT
/// ```
///
/// `earth_state` divides by the same `EMRAT`, so the two cancel exactly and the
/// observer returns to the VSOP87A Earth this series actually defines.
fn earth_to_emb(earth: StateVector, moon_rel_emb: StateVector) -> StateVector {
    let f = 1.0 / EMRAT;
    StateVector {
        position: Position {
            x: earth.position.x + moon_rel_emb.position.x * f,
            y: earth.position.y + moon_rel_emb.position.y * f,
            z: earth.position.z + moon_rel_emb.position.z * f,
        },
        velocity: Velocity {
            x: earth.velocity.x + moon_rel_emb.velocity.x * f,
            y: earth.velocity.y + moon_rel_emb.velocity.y * f,
            z: earth.velocity.z + moon_rel_emb.velocity.z * f,
        },
    }
}

impl EphemerisProvider for AnalyticalProvider {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        // Range check
        if jd < JD_MIN || jd > JD_MAX {
            return Err(ComputeError::DateOutOfRange {
                jd,
                min: JD_MIN,
                max: JD_MAX,
            });
        }

        match body {
            // Planets: Mercury through Neptune (and EMB)
            Body::Mercury
            | Body::Venus
            | Body::EarthMoonBarycenter
            | Body::Mars
            | Body::Jupiter
            | Body::Saturn
            | Body::Uranus
            | Body::Neptune => {
                let planet = body_to_vsop_planet(body)
                    .expect("body_to_vsop_planet should succeed for planet bodies");
                let state = vsop_state(planet, jd);
                if body == Body::EarthMoonBarycenter {
                    return Ok(earth_to_emb(state, moon_state(jd)));
                }
                Ok(state)
            }

            // Sun ≈ SSB origin
            Body::Sun => Ok(StateVector {
                position: Position {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                velocity: Velocity {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            }),

            // Moon: geocentric ELP + Earth position from VSOP87A
            Body::Moon => Ok(moon_state(jd)),

            // Lunar nodes have no state vector. They are directions, and
            // `coordinates` resolves them from `nodes::node_longitude`
            // before it ever reaches a provider. Returning a synthetic
            // unit vector here is what let the geocentric pipeline treat a
            // node as a body one AU away; the error reached 75°.
            Body::MeanNode | Body::TrueNode | Body::TrueNodeOsculating => {
                Err(ComputeError::BodyNotAvailable {
                    body_id: body.naif_id(),
                })
            }

            // Pluto not available in analytical theory
            Body::Pluto => Err(ComputeError::BodyNotAvailable {
                body_id: body.naif_id(),
            }),
        }
    }

    fn time_range(&self) -> (f64, f64) {
        (JD_MIN, JD_MAX)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const J2000: f64 = 2_451_545.0;

    fn provider() -> AnalyticalProvider {
        AnalyticalProvider::new()
    }

    fn distance(sv: &StateVector) -> f64 {
        let p = &sv.position;
        libm::sqrt(p.x * p.x + p.y * p.y + p.z * p.z)
    }

    #[test]
    fn time_range_covers_modern_era() {
        let (min, max) = provider().time_range();
        // 2000 CE ≈ JD 2451545
        assert!(min < 2_451_545.0, "min JD should be before J2000");
        assert!(max > 2_451_545.0, "max JD should be after J2000");
        // Range should cover at least 1000 CE to 2500 CE
        assert!(min < 2_086_300.0, "should cover back to ~1000 CE");
        assert!(max > 2_634_166.0, "should cover forward to ~2500 CE");
    }

    #[test]
    fn pluto_returns_body_not_available() {
        let result = provider().compute_state(Body::Pluto, J2000);
        assert!(result.is_err());
        match result.unwrap_err() {
            ComputeError::BodyNotAvailable { body_id } => {
                assert_eq!(body_id, Body::Pluto.naif_id());
            }
            other => panic!("Expected BodyNotAvailable, got {:?}", other),
        }
    }

    #[test]
    fn date_out_of_range_returns_error() {
        // Way before valid range
        let result = provider().compute_state(Body::Mars, 0.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ComputeError::DateOutOfRange { .. } => {}
            other => panic!("Expected DateOutOfRange, got {:?}", other),
        }

        // Way after valid range
        let result = provider().compute_state(Body::Mars, 5_000_000.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ComputeError::DateOutOfRange { .. } => {}
            other => panic!("Expected DateOutOfRange, got {:?}", other),
        }
    }

    #[test]
    fn sun_near_origin() {
        let sv = provider().compute_state(Body::Sun, J2000).unwrap();
        let r = distance(&sv);
        assert!(
            r < 1e-10,
            "Sun should be at origin (SSB approximation), distance = {r}"
        );
    }

    #[test]
    fn mars_at_reasonable_distance() {
        let sv = provider().compute_state(Body::Mars, J2000).unwrap();
        let r = distance(&sv);
        // Mars heliocentric distance ranges ~1.38-1.67 AU
        assert!(
            r > 1.0 && r < 2.0,
            "Mars distance should be 1-2 AU, got {r:.4} AU"
        );
    }

    #[test]
    fn moon_at_reasonable_distance() {
        // AnalyticalProvider returns Moon relative to EMB (not barycentric),
        // matching the SPK convention used by coordinates.rs.
        // Moon geocentric distance ~384400 km ≈ 0.00257 AU.
        // Moon_rel_EMB = Moon_geocentric * EMRAT/(1+EMRAT) ≈ 0.00254 AU.
        let sv = provider().compute_state(Body::Moon, J2000).unwrap();
        let r = distance(&sv);
        assert!(
            r > 0.001 && r < 0.005,
            "Moon_rel_EMB distance should be ~0.0025 AU, got {r:.6} AU"
        );
    }

    #[test]
    fn nodes_are_not_state_vector_bodies() {
        // A node is a direction, not a place. This provider used to answer
        // with a synthetic unit vector one AU from the barycentre, which the
        // geocentric pipeline then treated as a body and mangled by up to
        // 75°. `coordinates` resolves nodes from `nodes::node_longitude`
        // before any provider is consulted; end-to-end coverage lives in
        // `tests/node_frame.rs`.
        for body in [Body::MeanNode, Body::TrueNode, Body::TrueNodeOsculating] {
            assert!(
                matches!(
                    provider().compute_state(body, J2000),
                    Err(ComputeError::BodyNotAvailable { .. })
                ),
                "{body:?} must not produce a state vector"
            );
        }
    }

    #[test]
    fn all_supported_bodies_return_ok() {
        let bodies = [
            Body::Sun,
            Body::Moon,
            Body::Mercury,
            Body::Venus,
            Body::EarthMoonBarycenter,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
        ];
        for body in bodies {
            let result = provider().compute_state(body, J2000);
            assert!(
                result.is_ok(),
                "{:?} should return Ok, got {:?}",
                body,
                result.err()
            );
            let sv = result.unwrap();
            assert!(
                sv.position.x.is_finite() && sv.position.y.is_finite() && sv.position.z.is_finite(),
                "{:?} position has non-finite values",
                body
            );
            assert!(
                sv.velocity.x.is_finite() && sv.velocity.y.is_finite() && sv.velocity.z.is_finite(),
                "{:?} velocity has non-finite values",
                body
            );
        }
    }
}
