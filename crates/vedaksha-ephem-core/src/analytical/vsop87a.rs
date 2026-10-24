// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! VSOP87A Poisson series evaluator.
//!
//! Computes heliocentric ecliptic rectangular coordinates (J2000.0 frame) for
//! the eight major planets of the solar system using the VSOP87A analytical
//! theory (Bretagnon & Francou 1988).
//!
//! # Coordinate system
//! - Origin: Sun
//! - Plane: ecliptic of J2000.0
//! - Units: AU (position), AU/millennium (velocity)
//! - Frame: rectangular (X, Y, Z)
//!
//! # Series formula
//! ```text
//! coord(t) = sum_{alpha=0}^{5} t^alpha * sum_i [A_i * cos(B_i + C_i * t)]
//! ```
//! where `t` = Julian millennia from J2000.0 = (JD − 2 451 545.0) / 365 250.0.

use core::cell::RefCell;
use std::sync::LazyLock;

use wide::f64x4;

use super::coefficients;
use super::coefficients::loader::Vsop87Term;
use super::simd_trig::sincos_f64x4;

/// The eight major planets supported by VSOP87A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

// ─── Evaluation by angle addition over shared frequencies ──────────────────────
//
// Every term is `A · cos(B + C·t)`. The straightforward evaluation forms the
// angle `B + C·t` and calls `sincos` on it, once per term. But
//
//     cos(B + C·t) = cos B · cos C·t − sin B · sin C·t
//     sin(B + C·t) = sin B · cos C·t + cos B · sin C·t
//
// and `B` is a constant of the term, so `cos B` and `sin B` can be taken once,
// when the tables are built. What is left to compute per evaluation is the sine
// and cosine of `C·t`, and the frequencies `C` repeat: the X, Y and Z series
// and their six powers of `t` draw on the same few hundred frequencies per
// planet (Earth: 624 distinct `C` among 2,202 terms; Saturn: 1,870 among
// 7,512). So one `sincos` per *distinct frequency*, and each term costs two
// multiply-adds for its position and two for its velocity.
//
// Measured over the loaded tables by `tests::vsop_tables_share_their_frequencies`.
//
// **Accuracy.** The old route rounded `C·t`, added `B` and rounded again, so
// its angle was good to the `ulp` of the sum (1.5e-11 rad at |C·t| ≈ 1e5, the
// edge of the supported range). This route rounds `C·t` once and never forms
// the sum: its angle error is at most the old route's. The two differ by that
// rounding times the term's amplitude; `tests::tables_match_the_direct_angle_route`
// bounds it.

/// One power-of-`t` group of one coordinate, as table indices and coefficients.
struct Group {
    /// Index into [`PlanetTables::freqs`] of each term's frequency.
    freq: Vec<u32>,
    /// Per term `[A cos B, −A sin B, A C sin B, A C cos B]`: the position term
    /// is `p·cos(Ct) + q·sin(Ct)` and the velocity term `r·cos(Ct) + s·sin(Ct)`.
    coef: Vec<[f64; 4]>,
}

/// One planet's tables: its distinct frequencies and its 18 groups
/// (X, Y, Z times the six powers of `t`).
struct PlanetTables {
    freqs: Vec<f64>,
    groups: [[Group; 6]; 3],
}

impl PlanetTables {
    fn build(planet: Planet) -> Self {
        let series: [[&'static [Vsop87Term]; 6]; 3] = [
            get_series(planet, 0),
            get_series(planet, 1),
            get_series(planet, 2),
        ];
        // Distinct frequencies, in a fixed order (sorted by bit pattern).
        let mut bits: Vec<u64> = series
            .iter()
            .flatten()
            .flat_map(|terms| terms.iter().map(|t| t.frequency.to_bits()))
            .collect();
        bits.sort_unstable();
        bits.dedup();
        let freqs: Vec<f64> = bits.iter().map(|&b| f64::from_bits(b)).collect();

        let group = |terms: &[Vsop87Term]| -> Group {
            let mut freq = Vec::with_capacity(terms.len());
            let mut coef = Vec::with_capacity(terms.len());
            // Within a group the order is free (it is summed), and ordering by
            // frequency makes the reads of the shared sine/cosine table walk it
            // forward instead of jumping across it.
            let mut ordered: Vec<(u32, [f64; 4])> = terms
                .iter()
                .map(|t| {
                    let k = bits
                        .binary_search(&t.frequency.to_bits())
                        .expect("every frequency is in the table")
                        as u32;
                    let (sin_b, cos_b) = libm::sincos(t.phase);
                    let ac = t.amplitude * t.frequency;
                    (
                        k,
                        [
                            t.amplitude * cos_b,
                            -(t.amplitude * sin_b),
                            ac * sin_b,
                            ac * cos_b,
                        ],
                    )
                })
                .collect();
            ordered.sort_by_key(|&(k, _)| k);
            for (k, c) in ordered {
                freq.push(k);
                coef.push(c);
            }
            Group { freq, coef }
        };
        let groups = series.map(|coord| coord.map(group));
        Self { freqs, groups }
    }
}

static TABLES: [LazyLock<PlanetTables>; 8] = [
    LazyLock::new(|| PlanetTables::build(Planet::Mercury)),
    LazyLock::new(|| PlanetTables::build(Planet::Venus)),
    LazyLock::new(|| PlanetTables::build(Planet::Earth)),
    LazyLock::new(|| PlanetTables::build(Planet::Mars)),
    LazyLock::new(|| PlanetTables::build(Planet::Jupiter)),
    LazyLock::new(|| PlanetTables::build(Planet::Saturn)),
    LazyLock::new(|| PlanetTables::build(Planet::Uranus)),
    LazyLock::new(|| PlanetTables::build(Planet::Neptune)),
];

thread_local! {
    /// `(sin C_k·t, cos C_k·t)` for the planet being evaluated.
    static TRIG: RefCell<Vec<[f64; 2]>> = const { RefCell::new(Vec::new()) };
}

/// `(Σ p·cos + q·sin,  Σ r·cos + s·sin)` over one group, with four
/// independent accumulators.
fn group_sums(group: &Group, trig: &[[f64; 2]]) -> (f64, f64) {
    let mut pos = [0.0_f64; 4];
    let mut vel = [0.0_f64; 4];
    let (idx, idx_rest) = group.freq.as_chunks::<4>();
    let (coef, coef_rest) = group.coef.as_chunks::<4>();
    for (ic, cc) in idx.iter().zip(coef) {
        for (lane, (&k, c)) in ic.iter().zip(cc).enumerate() {
            let [sin_ct, cos_ct] = trig[k as usize];
            pos[lane] += c[0] * cos_ct + c[1] * sin_ct;
            vel[lane] += c[2] * cos_ct + c[3] * sin_ct;
        }
    }
    let mut p = (pos[0] + pos[1]) + (pos[2] + pos[3]);
    let mut v = (vel[0] + vel[1]) + (vel[2] + vel[3]);
    for (&k, c) in idx_rest.iter().zip(coef_rest) {
        let [sin_ct, cos_ct] = trig[k as usize];
        p += c[0] * cos_ct + c[1] * sin_ct;
        v += c[2] * cos_ct + c[3] * sin_ct;
    }
    (p, v)
}

/// One coordinate: `Σ_α t^α · Σ_i A cos(B + C t)` and its time derivative.
///
/// Returns `(position, velocity)` in AU and AU/millennium.
fn eval_coordinate(groups: &[Group; 6], trig: &[[f64; 2]], t: f64) -> (f64, f64) {
    let mut pos = 0.0_f64;
    let mut vel = 0.0_f64;

    for (alpha, group) in groups.iter().enumerate() {
        let alpha_f = alpha as f64;

        // t^alpha — precomputed
        let t_pow = if alpha == 0 {
            1.0
        } else {
            t.powi(alpha as i32)
        };

        // t^(alpha-1) — used only for velocity; undefined at t=0 when alpha==1
        // (alpha==0 term has no t^(alpha-1) contribution to velocity)
        let t_pow_prev = if alpha == 0 {
            0.0 // derivative of t^0 is 0
        } else if alpha == 1 {
            1.0 // t^0
        } else {
            // alpha >= 2: if t == 0 the whole alpha*t^(alpha-1) block is 0
            t.powi((alpha - 1) as i32)
        };

        // sum_cos = Σ A cos(B + C t);  sum_sin_c = Σ A·C·sin(B + C t)
        let (sum_cos, sum_sin_c) = group_sums(group, trig);

        // Position contribution: t^alpha * sum_cos
        pos += t_pow * sum_cos;

        // Velocity contribution: d/dt [t^alpha * sum_cos]
        //   = alpha * t^(alpha-1) * sum_cos  +  t^alpha * (-sum_sin_c)
        if alpha > 0 {
            vel += alpha_f * t_pow_prev * sum_cos;
        }
        vel -= t_pow * sum_sin_c;
    }

    (pos, vel)
}

/// Return the six power-series arrays for `planet` and `coord` (0=X, 1=Y, 2=Z).
fn get_series(planet: Planet, coord: usize) -> [&'static [Vsop87Term]; 6] {
    match (planet, coord) {
        // Mercury
        (Planet::Mercury, 0) => [
            coefficients::mercury::X0.as_slice(),
            coefficients::mercury::X1.as_slice(),
            coefficients::mercury::X2.as_slice(),
            coefficients::mercury::X3.as_slice(),
            coefficients::mercury::X4.as_slice(),
            coefficients::mercury::X5.as_slice(),
        ],
        (Planet::Mercury, 1) => [
            coefficients::mercury::Y0.as_slice(),
            coefficients::mercury::Y1.as_slice(),
            coefficients::mercury::Y2.as_slice(),
            coefficients::mercury::Y3.as_slice(),
            coefficients::mercury::Y4.as_slice(),
            coefficients::mercury::Y5.as_slice(),
        ],
        (Planet::Mercury, 2) => [
            coefficients::mercury::Z0.as_slice(),
            coefficients::mercury::Z1.as_slice(),
            coefficients::mercury::Z2.as_slice(),
            coefficients::mercury::Z3.as_slice(),
            coefficients::mercury::Z4.as_slice(),
            coefficients::mercury::Z5.as_slice(),
        ],
        // Venus
        (Planet::Venus, 0) => [
            coefficients::venus::X0.as_slice(),
            coefficients::venus::X1.as_slice(),
            coefficients::venus::X2.as_slice(),
            coefficients::venus::X3.as_slice(),
            coefficients::venus::X4.as_slice(),
            coefficients::venus::X5.as_slice(),
        ],
        (Planet::Venus, 1) => [
            coefficients::venus::Y0.as_slice(),
            coefficients::venus::Y1.as_slice(),
            coefficients::venus::Y2.as_slice(),
            coefficients::venus::Y3.as_slice(),
            coefficients::venus::Y4.as_slice(),
            coefficients::venus::Y5.as_slice(),
        ],
        (Planet::Venus, 2) => [
            coefficients::venus::Z0.as_slice(),
            coefficients::venus::Z1.as_slice(),
            coefficients::venus::Z2.as_slice(),
            coefficients::venus::Z3.as_slice(),
            coefficients::venus::Z4.as_slice(),
            coefficients::venus::Z5.as_slice(),
        ],
        // Earth
        (Planet::Earth, 0) => [
            coefficients::earth::X0.as_slice(),
            coefficients::earth::X1.as_slice(),
            coefficients::earth::X2.as_slice(),
            coefficients::earth::X3.as_slice(),
            coefficients::earth::X4.as_slice(),
            coefficients::earth::X5.as_slice(),
        ],
        (Planet::Earth, 1) => [
            coefficients::earth::Y0.as_slice(),
            coefficients::earth::Y1.as_slice(),
            coefficients::earth::Y2.as_slice(),
            coefficients::earth::Y3.as_slice(),
            coefficients::earth::Y4.as_slice(),
            coefficients::earth::Y5.as_slice(),
        ],
        (Planet::Earth, 2) => [
            coefficients::earth::Z0.as_slice(),
            coefficients::earth::Z1.as_slice(),
            coefficients::earth::Z2.as_slice(),
            coefficients::earth::Z3.as_slice(),
            coefficients::earth::Z4.as_slice(),
            coefficients::earth::Z5.as_slice(),
        ],
        // Mars
        (Planet::Mars, 0) => [
            coefficients::mars::X0.as_slice(),
            coefficients::mars::X1.as_slice(),
            coefficients::mars::X2.as_slice(),
            coefficients::mars::X3.as_slice(),
            coefficients::mars::X4.as_slice(),
            coefficients::mars::X5.as_slice(),
        ],
        (Planet::Mars, 1) => [
            coefficients::mars::Y0.as_slice(),
            coefficients::mars::Y1.as_slice(),
            coefficients::mars::Y2.as_slice(),
            coefficients::mars::Y3.as_slice(),
            coefficients::mars::Y4.as_slice(),
            coefficients::mars::Y5.as_slice(),
        ],
        (Planet::Mars, 2) => [
            coefficients::mars::Z0.as_slice(),
            coefficients::mars::Z1.as_slice(),
            coefficients::mars::Z2.as_slice(),
            coefficients::mars::Z3.as_slice(),
            coefficients::mars::Z4.as_slice(),
            coefficients::mars::Z5.as_slice(),
        ],
        // Jupiter
        (Planet::Jupiter, 0) => [
            coefficients::jupiter::X0.as_slice(),
            coefficients::jupiter::X1.as_slice(),
            coefficients::jupiter::X2.as_slice(),
            coefficients::jupiter::X3.as_slice(),
            coefficients::jupiter::X4.as_slice(),
            coefficients::jupiter::X5.as_slice(),
        ],
        (Planet::Jupiter, 1) => [
            coefficients::jupiter::Y0.as_slice(),
            coefficients::jupiter::Y1.as_slice(),
            coefficients::jupiter::Y2.as_slice(),
            coefficients::jupiter::Y3.as_slice(),
            coefficients::jupiter::Y4.as_slice(),
            coefficients::jupiter::Y5.as_slice(),
        ],
        (Planet::Jupiter, 2) => [
            coefficients::jupiter::Z0.as_slice(),
            coefficients::jupiter::Z1.as_slice(),
            coefficients::jupiter::Z2.as_slice(),
            coefficients::jupiter::Z3.as_slice(),
            coefficients::jupiter::Z4.as_slice(),
            coefficients::jupiter::Z5.as_slice(),
        ],
        // Saturn
        (Planet::Saturn, 0) => [
            coefficients::saturn::X0.as_slice(),
            coefficients::saturn::X1.as_slice(),
            coefficients::saturn::X2.as_slice(),
            coefficients::saturn::X3.as_slice(),
            coefficients::saturn::X4.as_slice(),
            coefficients::saturn::X5.as_slice(),
        ],
        (Planet::Saturn, 1) => [
            coefficients::saturn::Y0.as_slice(),
            coefficients::saturn::Y1.as_slice(),
            coefficients::saturn::Y2.as_slice(),
            coefficients::saturn::Y3.as_slice(),
            coefficients::saturn::Y4.as_slice(),
            coefficients::saturn::Y5.as_slice(),
        ],
        (Planet::Saturn, 2) => [
            coefficients::saturn::Z0.as_slice(),
            coefficients::saturn::Z1.as_slice(),
            coefficients::saturn::Z2.as_slice(),
            coefficients::saturn::Z3.as_slice(),
            coefficients::saturn::Z4.as_slice(),
            coefficients::saturn::Z5.as_slice(),
        ],
        // Uranus
        (Planet::Uranus, 0) => [
            coefficients::uranus::X0.as_slice(),
            coefficients::uranus::X1.as_slice(),
            coefficients::uranus::X2.as_slice(),
            coefficients::uranus::X3.as_slice(),
            coefficients::uranus::X4.as_slice(),
            coefficients::uranus::X5.as_slice(),
        ],
        (Planet::Uranus, 1) => [
            coefficients::uranus::Y0.as_slice(),
            coefficients::uranus::Y1.as_slice(),
            coefficients::uranus::Y2.as_slice(),
            coefficients::uranus::Y3.as_slice(),
            coefficients::uranus::Y4.as_slice(),
            coefficients::uranus::Y5.as_slice(),
        ],
        (Planet::Uranus, 2) => [
            coefficients::uranus::Z0.as_slice(),
            coefficients::uranus::Z1.as_slice(),
            coefficients::uranus::Z2.as_slice(),
            coefficients::uranus::Z3.as_slice(),
            coefficients::uranus::Z4.as_slice(),
            coefficients::uranus::Z5.as_slice(),
        ],
        // Neptune
        (Planet::Neptune, 0) => [
            coefficients::neptune::X0.as_slice(),
            coefficients::neptune::X1.as_slice(),
            coefficients::neptune::X2.as_slice(),
            coefficients::neptune::X3.as_slice(),
            coefficients::neptune::X4.as_slice(),
            coefficients::neptune::X5.as_slice(),
        ],
        (Planet::Neptune, 1) => [
            coefficients::neptune::Y0.as_slice(),
            coefficients::neptune::Y1.as_slice(),
            coefficients::neptune::Y2.as_slice(),
            coefficients::neptune::Y3.as_slice(),
            coefficients::neptune::Y4.as_slice(),
            coefficients::neptune::Y5.as_slice(),
        ],
        (Planet::Neptune, 2) => [
            coefficients::neptune::Z0.as_slice(),
            coefficients::neptune::Z1.as_slice(),
            coefficients::neptune::Z2.as_slice(),
            coefficients::neptune::Z3.as_slice(),
            coefficients::neptune::Z4.as_slice(),
            coefficients::neptune::Z5.as_slice(),
        ],
        _ => unreachable!("coord index must be 0, 1, or 2"),
    }
}

/// Compute heliocentric ecliptic rectangular coordinates for `planet` at `jd`.
///
/// # Arguments
/// - `planet`: one of the eight major planets
/// - `jd`: Julian Date (TDB)
///
/// # Returns
/// A tuple `(position_au, velocity_au_per_millennium)` where each element is
/// `[X, Y, Z]` in the J2000.0 ecliptic frame.
///
/// Position is in AU; velocity is in AU per Julian millennium (365 250 days).
pub fn vsop87a_heliocentric(planet: Planet, jd: f64) -> ([f64; 3], [f64; 3]) {
    // Julian millennia from J2000.0
    let t = (jd - 2_451_545.0) / 365_250.0;

    let tables = &*TABLES[planet as usize];
    TRIG.with_borrow_mut(|scratch| {
        // sin and cos of C·t for every distinct frequency of this planet.
        let n = tables.freqs.len();
        if scratch.len() < n {
            scratch.resize(n, [0.0, 0.0]);
        }
        let trig = &mut scratch[..n];
        for (freqs, out) in tables.freqs.chunks(4).zip(trig.chunks_mut(4)) {
            let mut angles = [0.0_f64; 4];
            for (angle, &c) in angles.iter_mut().zip(freqs) {
                *angle = c * t;
            }
            let (sin4, cos4) = sincos_f64x4(f64x4::from(angles));
            let (sin4, cos4) = (sin4.to_array(), cos4.to_array());
            for (k, slot) in out.iter_mut().enumerate() {
                *slot = [sin4[k], cos4[k]];
            }
        }

        let trig = &*trig;
        let (px, vx) = eval_coordinate(&tables.groups[0], trig, t);
        let (py, vy) = eval_coordinate(&tables.groups[1], trig, t);
        let (pz, vz) = eval_coordinate(&tables.groups[2], trig, t);

        ([px, py, pz], [vx, vy, vz])
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 epoch
    const J2000: f64 = 2_451_545.0;

    fn distance(pos: [f64; 3]) -> f64 {
        libm::sqrt(pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2])
    }

    #[test]
    fn earth_at_j2000_is_roughly_one_au() {
        let (pos, _vel) = vsop87a_heliocentric(Planet::Earth, J2000);
        let r = distance(pos);
        // Earth's distance from Sun ranges 0.983–1.017 AU; at J2000 it's ~0.9833
        assert!(
            r > 0.97 && r < 1.03,
            "Earth heliocentric distance at J2000 = {r:.6} AU, expected ~1 AU"
        );
    }

    #[test]
    fn jupiter_is_farther_than_earth() {
        let (earth_pos, _) = vsop87a_heliocentric(Planet::Earth, J2000);
        let (jup_pos, _) = vsop87a_heliocentric(Planet::Jupiter, J2000);
        let r_earth = distance(earth_pos);
        let r_jupiter = distance(jup_pos);
        assert!(
            r_jupiter > r_earth,
            "Jupiter ({r_jupiter:.3} AU) should be farther than Earth ({r_earth:.3} AU)"
        );
    }

    #[test]
    fn all_planets_finite_nonzero_at_j2000() {
        let planets = [
            Planet::Mercury,
            Planet::Venus,
            Planet::Earth,
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
            Planet::Uranus,
            Planet::Neptune,
        ];

        for planet in planets {
            let (pos, vel) = vsop87a_heliocentric(planet, J2000);
            for (i, &v) in pos.iter().enumerate() {
                assert!(v.is_finite(), "{planet:?} pos[{i}] is not finite");
            }
            for (i, &v) in vel.iter().enumerate() {
                assert!(v.is_finite(), "{planet:?} vel[{i}] is not finite");
            }
            let r = distance(pos);
            assert!(r > 0.0, "{planet:?} has zero heliocentric distance");
        }
    }

    #[test]
    fn known_planet_distances_at_j2000() {
        // Approximate semi-major axes in AU for a rough sanity check
        let expected: &[(Planet, f64, f64)] = &[
            (Planet::Mercury, 0.30, 0.50),
            (Planet::Venus, 0.70, 0.74),
            (Planet::Earth, 0.97, 1.03),
            (Planet::Mars, 1.38, 1.67),
            (Planet::Jupiter, 4.90, 5.50),
            (Planet::Saturn, 9.00, 9.60),
            (Planet::Uranus, 18.20, 20.20),
            (Planet::Neptune, 29.80, 30.40),
        ];

        for &(planet, lo, hi) in expected {
            let (pos, _) = vsop87a_heliocentric(planet, J2000);
            let r = distance(pos);
            assert!(
                r >= lo && r <= hi,
                "{planet:?}: distance {r:.4} AU not in expected range [{lo}, {hi}]"
            );
        }
    }

    #[test]
    fn velocity_is_finite_and_nonzero_for_earth() {
        let (_pos, vel) = vsop87a_heliocentric(Planet::Earth, J2000);
        let speed = libm::sqrt(vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]);
        assert!(speed.is_finite(), "Earth speed is not finite");
        assert!(speed > 0.0, "Earth speed is zero");
    }

    #[test]
    fn eval_at_t_zero_is_stable() {
        // t = 0 is the J2000 epoch; no division-by-zero should occur
        let (pos, vel) = vsop87a_heliocentric(Planet::Earth, J2000);
        for &v in pos.iter().chain(vel.iter()) {
            assert!(v.is_finite(), "value at t=0 is not finite: {v}");
        }
    }

    const ALL_PLANETS: [Planet; 8] = [
        Planet::Mercury,
        Planet::Venus,
        Planet::Earth,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
        Planet::Uranus,
        Planet::Neptune,
    ];

    /// The sharing the evaluation is built on, re-measured from the tables.
    #[test]
    fn vsop_tables_share_their_frequencies() {
        for planet in ALL_PLANETS {
            let tables = &*TABLES[planet as usize];
            let terms: usize = tables.groups.iter().flatten().map(|g| g.freq.len()).sum();
            let ratio = tables.freqs.len() as f64 / terms as f64;
            println!(
                "{planet:?}: {terms} terms, {} distinct frequencies ({:.0}%)",
                tables.freqs.len(),
                100.0 * ratio
            );
            // Measured 25% to 36%. At 50% the saving would be gone.
            assert!(
                ratio < 0.45,
                "{planet:?}: {ratio} of terms have a distinct frequency"
            );
        }
    }

    fn two_sum(a: f64, b: f64) -> (f64, f64) {
        let s = a + b;
        let bb = s - a;
        (s, (a - (s - bb)) + (b - bb))
    }

    /// `(sin, cos)` of `B + C·t` with the sum carried in double-double and
    /// reduced modulo 2π in extended precision. `f64::mul_add` here is the exact
    /// error of a product wherever it runs, not a speed choice.
    fn exact_pair(b: f64, c: f64, t: f64) -> (f64, f64) {
        const TAU_LO: f64 = 2.449_293_598_294_706_4e-16;
        let hi = c * t;
        let lo = c.mul_add(t, -hi);
        let (s, e) = two_sum(b, hi);
        let (hi, lo) = two_sum(s, e + lo);
        let n = (hi / core::f64::consts::TAU).round();
        let p = n * core::f64::consts::TAU;
        let p_err = n.mul_add(core::f64::consts::TAU, -p);
        let d = hi - p;
        let small = lo - p_err - n * TAU_LO;
        let (sn, cs) = libm::sincos(d);
        (sn + small * cs, cs - small * sn)
    }

    /// The route this file used to take: one `sincos` of the rounded angle per
    /// term. Kept here as a reference, with the exact-phase reference beside it.
    fn reference_position(planet: Planet, jd: f64, exact: bool) -> [f64; 3] {
        let t = (jd - J2000) / 365_250.0;
        let mut out = [0.0; 3];
        for (coord, slot) in out.iter_mut().enumerate() {
            for (alpha, terms) in get_series(planet, coord).iter().enumerate() {
                let mut sum = 0.0;
                for term in terms.iter() {
                    let cos = if exact {
                        exact_pair(term.phase, term.frequency, t).1
                    } else {
                        libm::cos(term.phase + term.frequency * t)
                    };
                    sum += term.amplitude * cos;
                }
                *slot += if alpha == 0 {
                    1.0
                } else {
                    t.powi(alpha as i32)
                } * sum;
            }
        }
        out
    }

    /// Positions from the shared-frequency route against the exact-phase
    /// reference, with the old per-term-angle route measured beside it.
    ///
    /// Both approximate the exact value to the rounding of an angle that at the
    /// range edge is of order 1e5 radians (`ulp` 1.5e-11) times an amplitude of
    /// a fraction of an AU: millimetres to metres. The new route rounds `C·t`
    /// once where the old rounded it and then the sum, so it must not be worse,
    /// and the test asserts both that and an absolute bound on the new route.
    #[test]
    fn tables_match_the_exact_phase_reference_no_worse_than_the_old_route() {
        use crate::analytical::{JD_MAX, JD_MIN};
        let au_m = crate::jpl::AU_KM * 1000.0;
        let mut worst_new = 0.0_f64;
        let mut worst_old = 0.0_f64;
        for planet in ALL_PLANETS {
            let (mut planet_new, mut planet_old) = (0.0_f64, 0.0_f64);
            for k in 0..=24 {
                let jd = JD_MIN + (JD_MAX - JD_MIN) * f64::from(k) / 24.0;
                let want = reference_position(planet, jd, true);
                let old = reference_position(planet, jd, false);
                let (got, _) = vsop87a_heliocentric(planet, jd);
                for ((g, w), o) in got.iter().zip(&want).zip(&old) {
                    planet_new = planet_new.max((g - w).abs() * au_m);
                    planet_old = planet_old.max((o - w).abs() * au_m);
                }
            }
            println!(
                "{planet:?}: new route {planet_new:.2e} m, old route {planet_old:.2e} m from the exact-phase value"
            );
            worst_new = worst_new.max(planet_new);
            worst_old = worst_old.max(planet_old);
        }
        // Summation order differs too, which is a few units in the last place of a
        // 1 AU sum (about 3e-5 m): allow it on top of "no worse".
        assert!(
            worst_new <= worst_old + 1e-3,
            "new {worst_new:e} m is worse than old {worst_old:e} m"
        );
        assert!(
            worst_new < 5.0,
            "new route off by {worst_new:e} m from the exact phase"
        );
    }

    /// The velocity coefficients (`A·C·sin B`, `A·C·cos B`) against a central
    /// difference of the positions: a sign or factor error in either shows up
    /// at order one.
    #[test]
    fn velocity_is_the_derivative_of_position() {
        // 0.073 day: the central difference's truncation error is (omega h)^2 / 6,
        // 4.5e-6 for Mercury's 88-day period, inside the 1e-4 allowed below.
        let h = 2.0e-7; // millennia
        for planet in ALL_PLANETS {
            for jd in [J2000, J2000 + 20_000.0, J2000 - 150_000.0] {
                let t0 = (jd - J2000) / 365_250.0;
                let at = |t: f64| vsop87a_heliocentric(planet, J2000 + t * 365_250.0).0;
                let (lo, hi) = (at(t0 - h), at(t0 + h));
                let (_, vel) = vsop87a_heliocentric(planet, jd);
                for c in 0..3 {
                    let numeric = (hi[c] - lo[c]) / (2.0 * h);
                    let tolerance = 1e-4 * vel[c].abs().max(1e-3);
                    assert!(
                        (vel[c] - numeric).abs() < tolerance,
                        "{planet:?} jd {jd} component {c}: velocity {} vs central difference {numeric}",
                        vel[c]
                    );
                }
            }
        }
    }

    /// `simd_trig::sincos_f64x4` was validated against ELP/MPP02's real phase
    /// domain (up to 3,303,561 rad, see `elp_mpp02.rs`'s
    /// `sincos_matches_libm_at_real_elp_phase_domain` and `simd_trig.rs`'s
    /// module doc) but never against VSOP87A's own argument distribution.
    /// `docs/audit/2026-08-29-perf-investigation.md` §3/§3b requires this be
    /// measured before vectorizing `eval_series`: the argument here is inside
    /// the already-validated bound, but that has to be checked, not assumed.
    ///
    /// This computes the ACTUAL `angle = B + C*t` value every term of every
    /// planet's every coordinate's every power-of-t group would feed
    /// `sincos_f64x4`, across `AnalyticalProvider`'s full supported JD range
    /// (`JD_MIN`/`JD_MAX` in `analytical/mod.rs`, ~-2000 CE to ~+3000 CE),
    /// sampled at epochs spanning that range — not a synthetic domain check
    /// unrelated to VSOP87A's real coefficients. It then measures
    /// `sincos_f64x4` vs. scalar `libm::sincos` at every one of those real
    /// arguments, plus a dense sweep of the full reachable interval, mirroring
    /// exactly the method `elp_mpp02.rs`'s own real-phase-domain test uses.
    #[test]
    fn sincos_matches_libm_at_real_vsop87a_phase_domain() {
        use crate::analytical::{JD_MAX, JD_MIN};

        const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

        let t_min = (JD_MIN - J2000) / DAYS_PER_MILLENNIUM;
        let t_max = (JD_MAX - J2000) / DAYS_PER_MILLENNIUM;
        // Epochs spanning the full supported range, in Julian millennia
        // (VSOP87A's own time unit): both extremes plus interior points,
        // including the present day (t ~ 0.0253).
        let epochs = [t_min, -3.0, -2.0, -1.0, -0.5, 0.0, 0.0253, 0.5, t_max];

        let planets = [
            Planet::Mercury,
            Planet::Venus,
            Planet::Earth,
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
            Planet::Uranus,
            Planet::Neptune,
        ];

        let mut max_abs_phase = 0.0_f64;
        let mut max_abs_phase_t = 0.0_f64;
        let mut real_phases: Vec<f64> = Vec::new();

        for &t in &epochs {
            for &planet in &planets {
                for coord in 0..3 {
                    for series in get_series(planet, coord) {
                        for term in series.iter() {
                            let angle = term.phase + term.frequency * t;
                            real_phases.push(angle);
                            if angle.abs() > max_abs_phase {
                                max_abs_phase = angle.abs();
                                max_abs_phase_t = t;
                            }
                        }
                    }
                }
            }
        }

        println!(
            "max |angle| fed to sincos_f64x4 across the supported JD range \
             [{JD_MIN}, {JD_MAX}]: {max_abs_phase:e} rad at t={max_abs_phase_t} \
             millennia ({} real term-angle samples)",
            real_phases.len()
        );
        // Cross-check against the investigation's independently-derived
        // figure over the full JD range: 1,356,541 rad. Our epoch grid uses
        // the exact t_min/t_max rather than the investigation's own sample
        // points, so allow a wide band -- this is a sanity check that we are
        // in the same regime, not a precise reproduction.
        assert!(
            max_abs_phase > 1_300_000.0,
            "expected max |angle| > 1.3e6 rad near the JD range edges, got {max_abs_phase:e}"
        );

        // Dense sweep across the full reachable interval (irrational step, as
        // in simd_trig::tests::matches_libm_across_domain and
        // elp_mpp02::tests::sincos_matches_libm_at_real_elp_phase_domain) to
        // catch reduction-boundary cases the discrete real term angles don't
        // happen to land on.
        let mut x = -max_abs_phase;
        while x <= max_abs_phase {
            real_phases.push(x);
            x += 137.035_999; // fine, irrational-ish step to vary the reduction
        }

        let mut max_sin_err = 0.0_f64;
        let mut max_cos_err = 0.0_f64;
        for chunk in real_phases.chunks(4) {
            let mut lane = [0.0_f64; 4];
            lane[..chunk.len()].copy_from_slice(chunk);
            let (s, c) = sincos_f64x4(f64x4::from(lane));
            let (s, c) = (s.to_array(), c.to_array());
            for i in 0..chunk.len() {
                let (ls, lc) = libm::sincos(lane[i]);
                max_sin_err = max_sin_err.max((s[i] - ls).abs());
                max_cos_err = max_cos_err.max((c[i] - lc).abs());
            }
        }

        println!(
            "max sin err = {max_sin_err:e}, max cos err = {max_cos_err:e} \
             over the real VSOP87A angle domain (|x| up to {max_abs_phase:e} rad)"
        );

        // Same 1e-12 bound simd_trig::tests::matches_libm_across_domain and
        // elp_mpp02::tests::sincos_matches_libm_at_real_elp_phase_domain use,
        // so all three tests hold the kernel to one documented standard. If
        // this fails, STOP -- do not vectorize eval_series; the kernel is not
        // safe at VSOP87A's real argument range and this is a correctness
        // question for simd_trig.rs, not something to work around here.
        assert!(
            max_sin_err < 1e-12,
            "max sin abs error {max_sin_err:e} over the real production VSOP87A \
             angle domain exceeds 1e-12 -- the vector sincos kernel is not safe \
             at this range; see docs/audit/2026-08-29-perf-investigation.md §3"
        );
        assert!(
            max_cos_err < 1e-12,
            "max cos abs error {max_cos_err:e} over the real production VSOP87A \
             angle domain exceeds 1e-12 -- the vector sincos kernel is not safe \
             at this range; see docs/audit/2026-08-29-perf-investigation.md §3"
        );
    }
}
