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
use crate::jpl::{AU_KM, EMRAT, EphemerisProvider, Position, StateVector, Velocity};

use self::vsop87a::Planet;

/// Precomputed cosine of the J2000 obliquity, **84381.448 arcsec**, the
/// IAU 1976 value.
///
/// That is deliberately *not* the IAU 2006 value of 84381.406 arcsec that
/// [`crate::obliquity::mean_obliquity`] evaluates. VSOP87A and ELP/MPP02 are
/// referred to the dynamical equinox and ecliptic of J2000 as their authors
/// defined it, and 84381.448 is the obliquity that frame is built on; rotating
/// their output with the IAU 2006 value would introduce a 0.042 arcsec frame
/// error against the very theory being evaluated.
///
/// Through v7.1.1 a module-level `OBLIQUITY_J2000` constant carried 84381.406,
/// was dead code behind an `#[allow]`, and the comment beside this literal and
/// [`SIN_EPS`] claimed to be its cosine and sine. Neither was — both encode
/// 84381.448. The values were right and the documentation was wrong, so the
/// documentation moved.
///
/// The literal itself was also 5.74e-12 (26083 ulps) above the true cosine,
/// which left `COS_EPS^2 + SIN_EPS^2` a part in 1e11 from unity, so
/// [`ecliptic_to_equatorial`] was not quite a rotation. Corrected in v7.2.0;
/// the change is bounded by 1.2e-6 arcsec and moved the analytical bit digest.
const COS_EPS: f64 = 0.917_482_062_069_181_8;

/// Precomputed sine of the same obliquity. This one was already exact.
const SIN_EPS: f64 = 0.397_777_155_931_913_7;

/// Julian millennia to days conversion factor.
const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// Minimum supported Julian Day (~-2000 CE).
const JD_MIN: f64 = 990_575.0;

/// Maximum supported Julian Day (~+3000 CE).
const JD_MAX: f64 = 2_816_788.0;

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
/// trait — [`EphemerisProvider::earth_state`]'s default body above all —
/// expects `EarthMoonBarycenter` to mean the barycentre, which is what the SPK
/// path genuinely stores. The offset is `Moon_rel_EMB / EMRAT`, about 4,671 km:
///
/// ```text
/// EMB = Earth + r/(1+EMRAT) = Earth + Moon_rel_EMB/EMRAT
/// ```
///
/// [`EphemerisProvider::earth_state`]'s default body divides by the same
/// `EMRAT`, so the two cancel and the observer returns to the VSOP87A Earth this
/// series actually defines. That round trip is why [`AnalyticalProvider`]
/// overrides `earth_state`; see the override's own comment.
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

        // The node arms and the Pluto arm return the same error and are kept
        // apart on purpose: a node has no state vector by nature, Pluto merely
        // has no analytical theory here, and the comment on each says which.
        // Merging them would leave one comment attached to both meanings.
        #[allow(clippy::match_same_arms)]
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
            Body::MeanNode
            | Body::TrueNode
            | Body::TrueNodeOsculating
            | Body::MeanSouthNode
            | Body::TrueSouthNode
            | Body::TrueSouthNodeOsculating => Err(ComputeError::BodyNotAvailable {
                body_id: body.naif_id(),
            }),

            // Pluto not available in analytical theory
            Body::Pluto => Err(ComputeError::BodyNotAvailable {
                body_id: body.naif_id(),
            }),
        }
    }

    /// Earth's own barycentric state — VSOP87A's `ear` series, directly.
    ///
    /// The trait's default body computes `EMB − Moon_rel_EMB/EMRAT`. On this
    /// provider both of those terms come from series *this file* composes:
    /// [`earth_to_emb`] builds `EarthMoonBarycenter` as
    /// `vsop_state(Earth) + moon_state/EMRAT`, and the default body then
    /// subtracts `moon_state/EMRAT` straight back off. The two Moon terms
    /// cancel algebraically, and the answer is `vsop_state(Earth)` — which is
    /// what this override returns.
    ///
    /// # What it costs, and what it buys
    ///
    /// `moon_state` is a full ELP/MPP02 evaluation: 35,758 trig terms, ~260 µs
    /// measured. The default body pulls it **twice** — once inside
    /// `compute_state(EarthMoonBarycenter, ·)` and once for
    /// `compute_state(Moon, ·)` — to produce a number that VSOP87A's 2,202-term
    /// Earth series (~11 µs) already determines. `ecliptic_position(Sun, jd)`,
    /// the workhorse of `search_transits` and `search_muhurta`'s solar scan,
    /// pays exactly that and nothing else of consequence.
    ///
    /// # Determinism: bounded in absolute terms, not in per-component ULP
    ///
    /// `(a + b) − b ≠ a` in binary floating point in general: the addition
    /// rounds once and the subtraction rounds again. Here `a` is Earth's
    /// position and `b` the EMRAT-scaled Moon term, with `|b/a| ≈ 3.1e-5` at
    /// the ~1 AU vector-magnitude scale that governs the addition's rounding.
    /// That fixes the addition's rounding error at `|δ| ≤ ulp(1 AU)/2 ≈
    /// 1.66e-5 m` (0.03 **mm**) — an *absolute* bound that holds everywhere in
    /// the supported range, ten orders of magnitude inside VSOP87A's own
    /// 0.239″ mean residual against Horizons. This path removes the two
    /// roundings rather than introducing one, so where it differs at all it
    /// is the *more* accurate of the two.
    ///
    /// That bound is **absolute, not relative**: a ULP is measured against a
    /// value's own magnitude, and Earth's individual x/y/z components each
    /// cross zero over time, where `ulp(component)` shrinks toward zero while
    /// the ~1.66e-5 m absolute error stays fixed. Measured directly: a sweep
    /// bisected onto a `position.z` zero crossing, and a one-second-step
    /// sweep across the 2025 March equinox — both ordinary, in-range dates,
    /// not edge cases — find the discarded round trip reaching **up to 1,237
    /// ULP** of the affected component. So "≤1 ULP" is not a property of this
    /// construction in general, only of JDs that happen to stay away from a
    /// crossing; `earth_state_matches_the_emb_minus_moon_construction` below
    /// asserts the absolute bound, not a per-component ULP count, for exactly
    /// this reason.
    ///
    /// **Measured, not assumed: the fixture digest moved nothing.** Over the
    /// 21,915-row Horizons fixture, `analytical_bit_digest`'s ROW dump is
    /// **byte-for-byte identical** either side of this change and
    /// `EXPECTED_DIGEST` did not need re-pinning — a fact about this specific
    /// fixture's rows, not a proof that the arithmetic is always
    /// bit-identical; see that test's own doc comment for the measured
    /// production rate at which the underlying perturbation reaches a
    /// printed digest bit.
    ///
    /// The two tests below pin both halves:
    /// `earth_state_is_bit_identical_to_vsop87a_earth` pins the identity this
    /// override asserts, and `earth_state_matches_the_emb_minus_moon_construction`
    /// pins its agreement with the construction it replaces.
    ///
    /// This override is specific to `AnalyticalProvider`. On the SPK path EMB
    /// and Moon are independent kernel segments, nothing cancels, and the
    /// default body is the only correct construction — which is why it stays
    /// the default.
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        if jd < JD_MIN || jd > JD_MAX {
            return Err(ComputeError::DateOutOfRange {
                jd,
                min: JD_MIN,
                max: JD_MAX,
            });
        }
        Ok(vsop_state(Planet::Earth, jd))
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

    /// The two hardcoded rotation constants must be the cosine and sine of the
    /// obliquity they claim to come from, and together must form a rotation.
    ///
    /// Nothing checked either through v7.1.1. `OBLIQUITY_J2000` named a
    /// different obliquity from the one the constants encode, and `COS_EPS`
    /// was 26083 ulps off, so `ecliptic_to_equatorial` was very slightly
    /// non-orthogonal.
    /// The obliquity [`COS_EPS`] and [`SIN_EPS`] must reproduce. Defined here
    /// rather than beside them because nothing outside this check needs it —
    /// through v7.1.1 it sat at module level as dead code behind an `#[allow]`,
    /// carrying a different value from the one the constants encode.
    const OBLIQUITY_J2000: f64 = 84_381.448 * core::f64::consts::PI / (180.0 * 3600.0);

    #[test]
    fn cos_eps_and_sin_eps_are_the_obliquity_they_claim() {
        // One ulp of tolerance: the literals are transcribed, and libm's
        // rounding need not match whatever produced them to the last bit.
        let ulp = f64::EPSILON;
        assert!(
            (COS_EPS - libm::cos(OBLIQUITY_J2000)).abs() <= ulp,
            "COS_EPS {COS_EPS} vs cos(OBLIQUITY_J2000) {}",
            libm::cos(OBLIQUITY_J2000)
        );
        assert!(
            (SIN_EPS - libm::sin(OBLIQUITY_J2000)).abs() <= ulp,
            "SIN_EPS {SIN_EPS} vs sin(OBLIQUITY_J2000) {}",
            libm::sin(OBLIQUITY_J2000)
        );
    }

    /// `ecliptic_to_equatorial` must preserve length — it is a rotation about
    /// the x axis and nothing else.
    #[test]
    fn ecliptic_to_equatorial_preserves_length() {
        for (x, y, z) in [
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 1.0),
            (0.3, -0.7, 0.65),
            (-5.2, 1.1, -0.4),
        ] {
            let before = libm::sqrt(x * x + y * y + z * z);
            let (rx, ry, rz) = ecliptic_to_equatorial(x, y, z);
            let after = libm::sqrt(rx * rx + ry * ry + rz * rz);
            assert!(
                (after / before - 1.0).abs() < 1e-15,
                "length changed by {} for ({x}, {y}, {z})",
                after / before - 1.0
            );
        }
    }

    /// JDs spanning the provider's supported range, used by the `earth_state`
    /// invariant tests below. `JD_MIN`/`JD_MAX` are included because the
    /// override carries its own range check and the boundaries are where an
    /// off-by-one in that check would show.
    const EARTH_STATE_JDS: [f64; 8] = [
        JD_MIN,
        1_356_173.0, // ~ -1000 CE
        2_086_302.5, // ~ 1000 CE
        J2000,       // 2000-01-01.5
        2_460_676.5, // 2025-01-01, the benches' epoch
        2_469_807.5, // 2050-01-01
        2_816_787.0, // one day inside the upper edge
        JD_MAX,
    ];

    /// **The invariant this override exists to establish.**
    ///
    /// `AnalyticalProvider::earth_state` must be bit-for-bit
    /// `vsop_state(Planet::Earth, jd)` — not "close to", not "within a ULP".
    /// It is a direct return, and this pins it as one. `analytical_bit_digest`
    /// is a looser instrument for this: it fingerprints the whole apparent
    /// pipeline, so a future change that reintroduced an EMRAT round trip here
    /// would move it, but this test says *what* moved.
    #[test]
    fn earth_state_is_bit_identical_to_vsop87a_earth() {
        let p = provider();
        for jd in EARTH_STATE_JDS {
            let got = p.earth_state(jd).expect("in-range jd");
            let want = vsop_state(Planet::Earth, jd);
            for (label, a, b) in [
                ("position.x", got.position.x, want.position.x),
                ("position.y", got.position.y, want.position.y),
                ("position.z", got.position.z, want.position.z),
                ("velocity.x", got.velocity.x, want.velocity.x),
                ("velocity.y", got.velocity.y, want.velocity.y),
                ("velocity.z", got.velocity.z, want.velocity.z),
            ] {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "earth_state({jd}).{label} = {a:?} ({:016x}) is not bit-identical to \
                     vsop_state(Earth, {jd}).{label} = {b:?} ({:016x})",
                    a.to_bits(),
                    b.to_bits(),
                );
            }
        }
    }

    /// The override is an *algebraic* shortcut, so it must still agree with the
    /// construction it replaces — to the absolute-distance bound asserted
    /// below, not to a fixed ULP count (see "What this does NOT bound" below
    /// for why a per-component ULP assertion was removed from this test).
    ///
    /// This is the test that would catch the shortcut being wrong rather than
    /// merely different: if `earth_to_emb` ever stopped being built from the
    /// same VSOP87A Earth term (or the EMRAT divisors diverged), the two paths
    /// would separate by kilometres, not fractions of a millimetre. The bound
    /// is expressed in metres so the failure message is readable: the
    /// tolerance below is 1 mm, still seven orders of magnitude below the
    /// 56.8 km a wrong EMRAT divisor costs and eight below the 4,671 km an
    /// un-lifted EMB costs.
    ///
    /// # What was actually measured (2026-08-29), on THIS sweep's 800 JDs
    ///
    /// Over the 800 JDs this sweeps — the eight anchors above, 426 spread
    /// evenly across the whole supported range, and 366 daily steps through
    /// 2025 — **every one of the 2,400 position components is bit-identical**
    /// between the two paths, and exactly **2 of the 2,400 velocity components
    /// differ, by exactly 1 ULP** (both `velocity.z`, at jd 2816787 and jd
    /// 2021848.22). Maximum positional separation: 0 m. That is a true report
    /// of these particular JDs, not a general property of the construction —
    /// see the next section.
    ///
    /// # What this does NOT bound: per-component ULP near a zero crossing
    ///
    /// The underlying rounding argument only supports an *absolute* bound.
    /// Writing `a` for Earth's component and `b` for the EMRAT-scaled Moon
    /// term (`|b/a| ≈ 3.1e-5` at the ~1 AU scale that governs the addition):
    /// `fl(a + b) = a + b + δ` with `|δ| ≤ ulp(1 AU)/2 ≈ 1.66e-5 m`, always.
    /// That does *not* imply the subtraction's result is within a small
    /// number of ULPs of `a`: ULP is relative to `a`'s own magnitude, and
    /// Earth's x/y/z components individually pass through zero, where
    /// `ulp(a)` shrinks toward zero while `δ` does not shrink with it. A
    /// sweep bisected onto a `position.z` zero crossing, and a
    /// one-second-step sweep across the 2025 March equinox — both ordinary,
    /// in-range dates — measure the discarded round trip reaching **up to
    /// 1,237 ULP**. This test's fixed JD list simply doesn't land near a
    /// crossing, which is exactly why an earlier version of this test
    /// asserted `ulps <= 1` here: that assertion passed today by accident of
    /// which JDs were sampled, and would have started failing on a
    /// non-regression the moment someone extended or randomized the JD list.
    /// It has been removed; the `d < TOLERANCE_M` absolute-distance assertion
    /// below is the real, general correctness guard, because it does not
    /// depend on how close any component happens to be to zero.
    ///
    /// `analytical_bit_digest` agrees at the other end of the pipeline: over
    /// the 21,915-row Horizons fixture the ROW dump is **byte-for-byte
    /// unchanged** by this override (sha256 of the dumps either side of the
    /// change: `e1a1784dc35970a2af1316f7049bb8064bec1ce3afecc8f0da3859cf0807f5af`
    /// both times), so `EXPECTED_DIGEST` did not need re-pinning — measured
    /// for this specific fixture, not proven in general (production rate:
    /// ~2.0e-4 differing Sun rows per row evaluated; see that test's doc
    /// comment).
    #[test]
    fn earth_state_matches_the_emb_minus_moon_construction() {
        const AU_M: f64 = AU_KM * 1000.0;
        const TOLERANCE_M: f64 = 1e-3;

        // The anchors, a coarse sweep of the full supported range, and a year
        // of daily steps around the contemporary epoch (where the tie, if one
        // existed, would matter most).
        let mut jds: Vec<f64> = EARTH_STATE_JDS.to_vec();
        let step = (JD_MAX - JD_MIN) / 425.0;
        for i in 0..426 {
            jds.push(JD_MIN + f64::from(i) * step);
        }
        for day in 0..366 {
            jds.push(2_460_676.5 + f64::from(day));
        }

        let p = provider();
        let mut components = 0usize;
        let mut non_exact = 0usize;
        let mut max_m = 0.0_f64;

        for &jd in &jds {
            let direct = p.earth_state(jd).expect("in-range jd");

            // The trait default's construction, spelled out here so the test
            // does not depend on which provider supplies the default body.
            let emb = p
                .compute_state(Body::EarthMoonBarycenter, jd)
                .expect("in-range jd");
            let moon = p.compute_state(Body::Moon, jd).expect("in-range jd");
            let f = 1.0 / EMRAT;
            let via_emb = [
                emb.position.x - moon.position.x * f,
                emb.position.y - moon.position.y * f,
                emb.position.z - moon.position.z * f,
                emb.velocity.x - moon.velocity.x * f,
                emb.velocity.y - moon.velocity.y * f,
                emb.velocity.z - moon.velocity.z * f,
            ];
            let got = [
                direct.position.x,
                direct.position.y,
                direct.position.z,
                direct.velocity.x,
                direct.velocity.y,
                direct.velocity.z,
            ];

            // No per-component ULP assertion here on purpose. Per-component
            // ULP is not bounded near a zero crossing — see
            // docs/audit/2026-08-29-perf-investigation.md #1 and the
            // "Determinism" doc comment on `earth_state` above: measured up
            // to 1,237 ULP at ordinary, in-range JDs (a sweep bisected onto a
            // `position.z` zero crossing, and a one-second-step sweep across
            // the 2025 March equinox). This sweep's fixed JD list happens not
            // to land near a crossing, so it currently sees only exact
            // matches or 1-ULP velocity ties, but that is a property of
            // these particular JDs, not of the construction — an assertion
            // like `ulps <= 1` would be a latent trap for whoever later
            // extends or randomizes the JD list, failing on a phantom
            // regression rather than a real one. The `d < TOLERANCE_M` check
            // below, which bounds absolute distance rather than a
            // per-component ULP count, is the real, general correctness
            // guard.
            for (i, (a, b)) in got.iter().zip(via_emb.iter()).enumerate() {
                components += 1;
                if a.to_bits() != b.to_bits() {
                    non_exact += 1;
                    let ulps = a.to_bits().abs_diff(b.to_bits());
                    let kind = if i < 3 { "position" } else { "velocity" };
                    println!(
                        "  jd {jd}: {kind}[{}] differs by {ulps} ULP ({a:e} vs {b:e})",
                        i % 3
                    );
                }
            }

            let d = libm::sqrt(
                (got[0] - via_emb[0]) * (got[0] - via_emb[0])
                    + (got[1] - via_emb[1]) * (got[1] - via_emb[1])
                    + (got[2] - via_emb[2]) * (got[2] - via_emb[2]),
            ) * AU_M;
            max_m = max_m.max(d);

            assert!(
                d < TOLERANCE_M,
                "at jd {jd}, earth_state and EMB−Moon/EMRAT disagree by {d} m \
                 (tolerance {TOLERANCE_M} m); the two are supposed to cancel to at most \
                 ~1 ULP (3.3e-5 m), so this is a real divergence, not rounding"
            );
        }

        println!(
            "earth_state vs EMB−Moon/EMRAT over {} JDs: {non_exact}/{components} components \
             differ, max positional separation {max_m:e} m",
            jds.len()
        );
    }

    /// The override must reject out-of-range dates the same way
    /// `compute_state` does. Without its own check it would silently answer for
    /// any `jd`, because it no longer routes through `compute_state`.
    #[test]
    fn earth_state_rejects_out_of_range_dates() {
        let p = provider();
        for jd in [0.0, JD_MIN - 1.0, JD_MAX + 1.0, 5_000_000.0] {
            match p.earth_state(jd) {
                Err(ComputeError::DateOutOfRange { .. }) => {}
                other => panic!("earth_state({jd}) should be DateOutOfRange, got {other:?}"),
            }
        }
    }
}
