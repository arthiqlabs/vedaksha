// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Analytical ephemeris models.
//!
//! This module provides planetary and lunar positions computed from analytical
//! series expansions, requiring no external data files at runtime:
//!
//! - **VSOP87A** (Bretagnon & Francou 1988) — heliocentric rectangular
//!   ecliptic coordinates (J2000.0) for Mercury through Neptune, rotated to
//!   the equatorial frame by the rotation the theory's authors publish
//! - **ELP/MPP02** (Chapront 2002) — geocentric lunar coordinates
//!
//! The [`AnalyticalProvider`] struct implements [`EphemerisProvider`] to wire
//! these evaluators into the unified trait interface.

pub mod chebyshev_provider;
pub mod coefficients;
pub mod elp_mpp02;
mod phase_graph;
mod simd_trig;
pub mod vsop87a;

use crate::bodies::Body;
use crate::error::ComputeError;
use crate::jpl::{AU_KM, EMRAT, EphemerisProvider, Position, StateVector, Velocity};

pub use self::chebyshev_provider::{ChebyshevConfig, ChebyshevProvider, SegmentSpec};
use self::vsop87a::Planet;

/// The rotation from VSOP87A's frame to the equatorial frame, **as the theory's
/// authors publish it**, applied to every VSOP87A vector (position and
/// velocity) in [`vsop_state`].
///
/// VSOP87A's rectangular coordinates are, in the authors' words, "given in the
/// inertial frame defined by the dynamical equinox and ecliptic J2000". Their
/// documentation (`vsop87.doc`, section "REFERENCE SYSTEM", Bretagnon and
/// Francou, distributed with the series at
/// `https://ftp.imcce.fr/pub/ephem/planets/vsop87/`) goes on: "The rectangular
/// coordinates of VSOP87A and VSOP87E defined in dynamical ecliptic frame J2000
/// can be connected to the equatorial frame FK5 J2000 with the following
/// rotation", and prints this matrix to twelve decimals:
///
/// ```text
///   X        +1.000000000000  +0.000000440360  -0.000000190919   X
///   Y     =  -0.000000479966  +0.917482137087  -0.397776982902   Y
///   Z FK5     0.000000000000  +0.397776982902  +0.917482137087   Z VSOP87A
/// ```
///
/// Read off the matrix, it is `Rz(phi) · Rx(eps)`: the ordinary ecliptic to
/// equator rotation about x by `eps = 84381.4091` arcsec, followed by a
/// rotation about the equatorial z axis by `phi = -0.0990` arcsec, an equinox
/// offset. The authors state the matrix, not those two angles;
/// `tests::vsop87a_matrix_is_rz_phi_rx_eps` pins that the matrix reproduces
/// from them to its own rounding. The VSOP2013 distribution documents its
/// ICRF rotation in the same `Rz(phi) · Rx(eps)` form, with different
/// constants (`eps = 84381.41136`, `phi = -0.05188` arcsec), because that
/// theory was fitted to a different numerical integration; those constants are
/// not VSOP87A's and are not used here.
///
/// # What this replaced, and what that was worth
///
/// Before this change VSOP87A was rotated with `Rx(84381.448 arcsec)` alone,
/// the same [`ecliptic_to_equatorial`] the Moon still uses, on the belief that
/// 84381.448 is the obliquity VSOP87A's frame is built on. The authors'
/// matrix differs from that by 38.9 mas about x (84381.448 against 84381.4091)
/// and 99.0 mas about z (the equinox offset, which was absent altogether).
/// Against DE440s over 1900 to 2100 (30-day grid, geometric heliocentric
/// vectors, no light time or aberration), the mean angular residual went:
///
/// | body | before | after |
/// |---|---|---|
/// | Mercury | 0.102″ | 0.021″ |
/// | Venus | 0.090″ | 0.014″ |
/// | Earth | 0.088″ | 0.019″ |
/// | Mars | 0.075″ | 0.032″ |
/// | Jupiter | 0.236″ | 0.170″ |
/// | Saturn | 0.217″ | 0.141″ |
///
/// What remains is not a frame error. Uranus and Neptune barely moved (0.605″
/// to 0.611″ and 1.095″ to 1.029″), and a constant rotation fitted per body
/// leaves them at 0.61″ and 0.90″. Evaluating the complete untruncated VSOP87A
/// series gives the same residuals as the committed tables to four decimals
/// for the outer planets and to within 0.005″ in the mean for the inner ones,
/// so the committed tables' truncation is not the cause either. The theory was
/// fitted to the JPL integration DE200 (`vsop87.doc`, "PRECISION"), not to
/// DE440, and that is what the residual measures.
///
/// The target of this matrix is FK5, not the ICRS the rest of the engine
/// assumes. The two are different realisations of the equator and equinox, and
/// no published rotation between them is applied here: nothing read for this
/// change gives one for a theory fitted to DE200. Measured, after this
/// rotation the Venus and Earth vectors still sit about 16 mas of rotation
/// from DE440s (a fit, so evidence and not a correction).
const VSOP87A_TO_EQUATORIAL: [[f64; 3]; 3] = [
    [1.000_000_000_000, 0.000_000_440_360, -0.000_000_190_919],
    [-0.000_000_479_966, 0.917_482_137_087, -0.397_776_982_902],
    [0.0, 0.397_776_982_902, 0.917_482_137_087],
];

/// Precomputed cosine of the J2000 obliquity, **84381.448 arcsec**, the
/// IAU 1976 value. It serves the Moon only, through
/// [`ecliptic_to_equatorial`]; VSOP87A has its own rotation,
/// `VSOP87A_TO_EQUATORIAL`.
///
/// That is deliberately *not* the IAU 2006 value of 84381.406 arcsec that
/// [`crate::obliquity::mean_obliquity`] evaluates, and whether it is the
/// right rotation for ELP/MPP02 is not settled here. Measured against DE440s
/// over 1900 to 2100 (30-day grid), the geocentric lunar vector this rotation
/// produces sits 0.030″ mean from the SPK path in angle, and the rotation that
/// best fits the difference is (37.6, 16.4, 14.0) mas about (x, y, z), whose
/// x part is the 38.9 mas between 84381.448 and 84381.4091. That fit is
/// evidence, not a correction; ELP/MPP02's own published frame relation has
/// not been checked against it.
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

/// Rotate a vector from ecliptic (J2000) to equatorial (ICRS approximation)
/// about x by the IAU 1976 obliquity. Used for the Moon only; VSOP87A goes
/// through [`vsop87a_to_equatorial`].
#[inline]
fn ecliptic_to_equatorial(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (x, y * COS_EPS - z * SIN_EPS, y * SIN_EPS + z * COS_EPS)
}

/// Rotate a VSOP87A vector from the theory's dynamical ecliptic and equinox of
/// J2000 to the equatorial frame, by the authors' published matrix
/// (`VSOP87A_TO_EQUATORIAL`).
#[inline]
fn vsop87a_to_equatorial(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let m = &VSOP87A_TO_EQUATORIAL;
    (
        m[0][0] * x + m[0][1] * y + m[0][2] * z,
        m[1][0] * x + m[1][1] * y + m[1][2] * z,
        m[2][0] * x + m[2][1] * y + m[2][2] * z,
    )
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

/// Compute a VSOP87A planet state in barycentric equatorial frame.
///
/// Heliocentric ecliptic → equatorial rotation, by the matrix the VSOP87
/// authors publish (`VSOP87A_TO_EQUATORIAL`), whose target is FK5 J2000 and
/// which is treated as ICRS. Heliocentric ≈ barycentric (Sun ≈ SSB; error
/// < 0.5 arcsec for inner planets, acceptable budget).
fn vsop_state(planet: Planet, jd: f64) -> StateVector {
    let (pos_ecl, vel_ecl) = vsop87a::vsop87a_heliocentric(planet, jd);

    let (px, py, pz) = vsop87a_to_equatorial(pos_ecl[0], pos_ecl[1], pos_ecl[2]);

    // Velocity: AU/millennium → AU/day
    let vx_day = vel_ecl[0] / DAYS_PER_MILLENNIUM;
    let vy_day = vel_ecl[1] / DAYS_PER_MILLENNIUM;
    let vz_day = vel_ecl[2] / DAYS_PER_MILLENNIUM;
    let (vx, vy, vz) = vsop87a_to_equatorial(vx_day, vy_day, vz_day);

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
/// `compute_state(Body::Moon)` returns the Moon's position **relative to
/// EMB** (Earth-Moon Barycenter), matching the SPK convention (center=3).
/// [`retarded_geocentric`](crate::coordinates)'s Moon branch reaches this
/// same convention through the position-only [`EphemerisProvider::moon_position`]
/// trait method instead, not through `compute_state` -- see
/// `moon_position_rel_emb` below, which performs the identical conversion.
/// The conversion from geocentric is:
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

/// Compute Moon **position only** (no velocity) relative to EMB in
/// equatorial (ICRS) frame.
///
/// Position-only sibling of [`moon_state`]; performs exactly the same steps
/// on the position half — [`elp_mpp02::elp_geocentric_position`] instead of
/// [`elp_mpp02::elp_geocentric`], then the identical km→AU conversion, EMRAT
/// scaling and `ecliptic_to_equatorial` call — so its result is bit-for-bit
/// identical to `moon_state(jd).position`. See
/// [`crate::jpl::EphemerisProvider::moon_position`] for why this exists.
fn moon_position_rel_emb(jd: f64) -> Position {
    // Moon geocentric position in J2000 ecliptic rectangular (km).
    let (mx, my, mz) = elp_mpp02::elp_geocentric_position(jd);

    // Convert km → AU
    let mg_x = mx / AU_KM;
    let mg_y = my / AU_KM;
    let mg_z = mz / AU_KM;

    // Convert geocentric → relative to EMB:
    // Moon_rel_EMB = Moon_geocentric * EMRAT / (1 + EMRAT)
    let emb_factor = EMRAT / (1.0 + EMRAT);

    let rel_x = mg_x * emb_factor;
    let rel_y = mg_y * emb_factor;
    let rel_z = mg_z * emb_factor;

    // J2000 ecliptic -> equatorial
    let (px, py, pz) = ecliptic_to_equatorial(rel_x, rel_y, rel_z);

    Position {
        x: px,
        y: py,
        z: pz,
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
    /// `moon_state` is a full ELP/MPP02 evaluation: 35,758 series terms, 56 µs
    /// measured at v10.0.0 (188 µs at v9.4.1, before the series was built by angle
    /// addition; criterion medians of `elp_mpp02_moon`). The default body pulls
    /// it **twice** — once inside `compute_state(EarthMoonBarycenter, ·)` and once
    /// for `compute_state(Moon, ·)` — to produce a number that VSOP87A's
    /// 2,202-term Earth series (2.9 µs at v10.0.0, 6.1 µs at v9.4.1) already
    /// determines. `ecliptic_position(Sun, jd)`,
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
    /// the supported range, ten orders of magnitude inside the analytical
    /// path's own mean residual against Horizons (0.160″ at v10.0.0). This path removes the two
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

    /// Moon position only, via [`moon_position_rel_emb`] — skips the
    /// velocity half of [`moon_state`]'s ELP/MPP02 evaluation entirely
    /// rather than computing and discarding it.
    ///
    /// # Bit-identity
    ///
    /// [`moon_position_rel_emb`] performs the exact same operations, in the
    /// same order, as `moon_state`'s position half, differing only in
    /// calling [`elp_mpp02::elp_geocentric_position`] (proven bit-identical
    /// to `elp_geocentric`'s position fields by that module's own tests)
    /// instead of [`elp_mpp02::elp_geocentric`]. So this override's result is
    /// bit-for-bit identical to `compute_state(Body::Moon, jd)?.position` —
    /// see `moon_position_is_bit_identical_to_compute_state_moon` below.
    fn moon_position(&self, jd: f64) -> Result<Position, ComputeError> {
        if jd < JD_MIN || jd > JD_MAX {
            return Err(ComputeError::DateOutOfRange {
                jd,
                min: JD_MIN,
                max: JD_MAX,
            });
        }
        Ok(moon_position_rel_emb(jd))
    }

    /// The Moon's distance relative to the EMB from the ELP/MPP02 distance
    /// series alone ([`elp_mpp02::elp_geocentric_distance`]), with the same
    /// km→AU and EMRAT scaling [`moon_position_rel_emb`] applies to each
    /// component. The longitude and latitude series are not evaluated.
    ///
    /// `None` outside the provider's range, where [`Self::moon_position`]
    /// errors, as the trait's contract requires.
    fn moon_distance(&self, jd: f64) -> Option<f64> {
        if jd < JD_MIN || jd > JD_MAX {
            return None;
        }
        let emb_factor = EMRAT / (1.0 + EMRAT);
        Some(elp_mpp02::elp_geocentric_distance(jd) / AU_KM * emb_factor)
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

    /// The matrix the VSOP87 authors print in `vsop87.doc`, typed a second time
    /// here in the document's own row order and signs, so that an edit to
    /// `VSOP87A_TO_EQUATORIAL` alone is caught.
    const PUBLISHED_ROWS: [[f64; 3]; 3] = [
        [1.000000000000, 0.000000440360, -0.000000190919],
        [-0.000000479966, 0.917482137087, -0.397776982902],
        [0.000000000000, 0.397776982902, 0.917482137087],
    ];

    /// The constant is the published matrix, entry for entry, and its first
    /// column is where the VSOP87A x axis lands: `(1, -4.79966e-7, 0)`, a right
    /// ascension of -0.0990 arcsec in the equatorial frame.
    #[test]
    fn vsop87a_matrix_is_the_published_matrix() {
        for (i, (got_row, want_row)) in VSOP87A_TO_EQUATORIAL
            .iter()
            .zip(&PUBLISHED_ROWS)
            .enumerate()
        {
            for (j, (got, want)) in got_row.iter().zip(want_row).enumerate() {
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "VSOP87A_TO_EQUATORIAL[{i}][{j}] = {got} is not the published {want}"
                );
            }
        }
        let (x, y, z) = vsop87a_to_equatorial(1.0, 0.0, 0.0);
        assert_eq!((x, y, z), (1.0, -0.000000479966, 0.0));
    }

    /// The published matrix is a rotation, to the precision it is printed at.
    ///
    /// Each entry is rounded to twelve decimals, so each is wrong by at most
    /// 5e-13, and a row product or a determinant mixes three entries of size
    /// up to 1: the defect cannot honestly be expected below about 1.5e-12.
    /// Measured 2026-10-24: largest `|M·Mᵀ − I|` entry 5.8e-13. The bound is
    /// 2e-12, three times the measurement; a transcription slip in the ninth
    /// decimal (a 1e-9 error) is four hundred times over it.
    #[test]
    fn vsop87a_matrix_is_a_rotation_to_its_printed_precision() {
        let m = &VSOP87A_TO_EQUATORIAL;
        let mut worst = 0.0_f64;
        for (i, row_i) in m.iter().enumerate() {
            for (j, row_j) in m.iter().enumerate() {
                let dot: f64 = row_i.iter().zip(row_j).map(|(a, b)| a * b).sum();
                let want = if i == j { 1.0 } else { 0.0 };
                worst = worst.max((dot - want).abs());
            }
        }
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        println!(
            "VSOP87A matrix: worst |M Mt - I| = {worst:e}, det - 1 = {:e}",
            det - 1.0
        );
        assert!(worst < 2e-12, "M Mt differs from I by {worst:e}");
        assert!((det - 1.0).abs() < 2e-12, "det is {det}, not +1");
    }

    /// The matrix is `Rz(phi) · Rx(eps)` with `eps = 84381.4091` arcsec and
    /// `phi = -0.0990` arcsec, to its own rounding.
    ///
    /// Working. `Rz(phi) · Rx(eps)` is
    ///
    /// ```text
    /// [ cos phi   -sin phi cos eps    sin phi sin eps ]
    /// [ sin phi    cos phi cos eps   -cos phi sin eps ]
    /// [ 0          sin eps            cos eps         ]
    /// ```
    ///
    /// Its bottom row is `(0, sin eps, cos eps)`, so the printed `0.397776982902`
    /// and `0.917482137087` fix `eps = atan2(0.397776982902, 0.917482137087)`,
    /// which is 84381.4091 arcsec (23 deg 26 min 21.4091 sec). Its (2,1)
    /// entry is `sin phi`, so the printed `-0.000000479966` fixes
    /// `phi = -0.0990` arcsec. The (1,2) and (1,3) entries then follow as
    /// `-sin phi cos eps = +4.4036e-7` and `sin phi sin eps = -1.9092e-7`,
    /// matching the printed `0.000000440360` and `-0.000000190919`. The
    /// tolerance is 1e-12, twice the 5e-13 a twelve-decimal print can hide;
    /// measured largest difference 4.6e-13.
    #[test]
    fn vsop87a_matrix_is_rz_phi_rx_eps() {
        const ARCSEC: f64 = core::f64::consts::PI / (180.0 * 3600.0);
        let eps = 84_381.4091 * ARCSEC;
        let phi = -0.0990 * ARCSEC;
        let (se, ce) = (libm::sin(eps), libm::cos(eps));
        let (sp, cp) = (libm::sin(phi), libm::cos(phi));
        let want = [
            [cp, -sp * ce, sp * se],
            [sp, cp * ce, -cp * se],
            [0.0, se, ce],
        ];
        let worst = VSOP87A_TO_EQUATORIAL
            .iter()
            .flatten()
            .zip(want.iter().flatten())
            .map(|(got, want)| (got - want).abs())
            .fold(0.0_f64, f64::max);
        println!("VSOP87A matrix vs Rz(phi) Rx(eps): worst entry difference {worst:e}");
        assert!(worst < 1e-12, "largest entry difference {worst:e}");
    }

    /// `vsop_state` rotates VSOP87A with the published matrix and not with the
    /// Moon's obliquity-only rotation.
    ///
    /// The two differ by 38.9 mas about x and 99.0 mas about z, a rotation of
    /// 106 mas whose axis lies at ecliptic latitude -59 degrees. The planets
    /// stay within 8 degrees of the ecliptic plane, so they are at least 50
    /// degrees from the axis, and at 0.3 AU or more the two rotations move a
    /// vector apart by at least 1e-7 AU: eight orders over the 1e-15 AU
    /// agreement required with the matrix, so a revert to
    /// `ecliptic_to_equatorial` cannot pass.
    #[test]
    fn vsop_state_uses_the_published_rotation() {
        let planets = [Planet::Mercury, Planet::Venus, Planet::Earth, Planet::Mars];
        for planet in planets {
            for jd in [2_415_020.5, J2000, 2_460_676.5] {
                let (raw, raw_v) = vsop87a::vsop87a_heliocentric(planet, jd);
                let got = vsop_state(planet, jd);
                let m = &PUBLISHED_ROWS;
                let rot = |v: [f64; 3]| -> [f64; 3] {
                    core::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
                };
                let want = rot(raw);
                let want_v = rot([
                    raw_v[0] / DAYS_PER_MILLENNIUM,
                    raw_v[1] / DAYS_PER_MILLENNIUM,
                    raw_v[2] / DAYS_PER_MILLENNIUM,
                ]);
                let got_p = [got.position.x, got.position.y, got.position.z];
                let got_v = [got.velocity.x, got.velocity.y, got.velocity.z];
                for (k, (((gp, wp), gv), wv)) in
                    got_p.iter().zip(&want).zip(&got_v).zip(&want_v).enumerate()
                {
                    assert!(
                        (gp - wp).abs() < 1e-15,
                        "{planet:?} jd {jd} position[{k}]: {gp} vs published-matrix {wp}"
                    );
                    assert!(
                        (gv - wv).abs() < 1e-17,
                        "{planet:?} jd {jd} velocity[{k}]: {gv} vs published-matrix {wv}"
                    );
                }
                // And it is a different answer from the obliquity-only rotation.
                let (ox, oy, oz) = ecliptic_to_equatorial(raw[0], raw[1], raw[2]);
                let apart = libm::sqrt(
                    (got_p[0] - ox).powi(2) + (got_p[1] - oy).powi(2) + (got_p[2] - oz).powi(2),
                );
                assert!(
                    apart > 5e-8,
                    "{planet:?} jd {jd}: rotations agree to {apart:e} AU, they should differ \
                     by the 0.1 arcsec equinox offset"
                );
            }
        }
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
                 ~1.66e-5 m absolute (the addition's rounding error, bounded by \
                 ulp(1 AU)/2 — not a per-component ULP bound, see this module's doc), \
                 so this is a real divergence, not rounding"
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

    /// `moon_distance` against the length of `moon_position`'s vector, over
    /// the provider's whole range, and `None` outside it.
    ///
    /// The light-time iteration's bit-exactness proof assumes the trait's
    /// `MOON_DISTANCE_REL_TOL`, so this measures the real error and requires
    /// it to sit far inside that bound.
    #[test]
    fn moon_distance_matches_the_position_length() {
        let p = provider();
        let step = (JD_MAX - JD_MIN) / 2_000.0;
        let mut worst = 0.0_f64;
        for i in 0..=2_000 {
            let jd = JD_MIN + f64::from(i) * step;
            let pos = p.moon_position(jd).expect("in-range jd");
            let length = libm::sqrt(pos.x * pos.x + pos.y * pos.y + pos.z * pos.z);
            let d = p.moon_distance(jd).expect("in-range jd");
            worst = worst.max((d - length).abs() / length);
        }
        println!("moon_distance worst relative error vs |moon_position|: {worst:e}");
        assert!(
            worst < crate::jpl::MOON_DISTANCE_REL_TOL / 100.0,
            "moon_distance is {worst:e} off |moon_position| relative; the contract allows {:e}",
            crate::jpl::MOON_DISTANCE_REL_TOL
        );
        assert!(p.moon_distance(JD_MIN - 1.0).is_none());
        assert!(p.moon_distance(JD_MAX + 1.0).is_none());
    }

    /// **The invariant `AnalyticalProvider::moon_position` exists to
    /// establish.**
    ///
    /// It must be bit-for-bit `compute_state(Body::Moon, jd)?.position` —
    /// not "close to". Skipping the velocity half of the underlying ELP/MPP02
    /// evaluation must change no computed position value.
    #[test]
    fn moon_position_is_bit_identical_to_compute_state_moon() {
        let p = provider();
        // Anchors spanning the supported range, plus a spread of contemporary
        // and historical dates.
        let jds: Vec<f64> = {
            let mut v = EARTH_STATE_JDS.to_vec();
            let step = (JD_MAX - JD_MIN) / 200.0;
            for i in 0..=200 {
                v.push(JD_MIN + f64::from(i) * step);
            }
            v
        };
        for jd in jds {
            let got = p.moon_position(jd).expect("in-range jd");
            let want = p
                .compute_state(Body::Moon, jd)
                .expect("in-range jd")
                .position;
            for (label, a, b) in [
                ("x", got.x, want.x),
                ("y", got.y, want.y),
                ("z", got.z, want.z),
            ] {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "moon_position({jd}).{label} = {a:?} ({:016x}) is not bit-identical to \
                     compute_state(Moon, {jd}).position.{label} = {b:?} ({:016x})",
                    a.to_bits(),
                    b.to_bits(),
                );
            }
        }
    }

    /// The override must reject out-of-range dates the same way
    /// `compute_state` does.
    #[test]
    fn moon_position_rejects_out_of_range_dates() {
        let p = provider();
        for jd in [0.0, JD_MIN - 1.0, JD_MAX + 1.0, 5_000_000.0] {
            match p.moon_position(jd) {
                Err(ComputeError::DateOutOfRange { .. }) => {}
                other => panic!("moon_position({jd}) should be DateOutOfRange, got {other:?}"),
            }
        }
    }
}
