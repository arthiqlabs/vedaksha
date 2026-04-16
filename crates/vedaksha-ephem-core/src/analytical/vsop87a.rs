// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

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

use super::coefficients;

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

/// Evaluate a single Poisson power series for one coordinate.
///
/// Returns `(position, velocity)` in AU and AU/millennium respectively.
///
/// Each `series` slice element `series[alpha]` is a list of `(A, B, C)` triples
/// representing terms `A * cos(B + C*t)`.
fn eval_series(series: &[&[(f64, f64, f64)]; 6], t: f64) -> (f64, f64) {
    let mut pos = 0.0_f64;
    let mut vel = 0.0_f64;

    for (alpha, terms) in series.iter().enumerate() {
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

        let mut sum_cos = 0.0_f64;
        let mut sum_sin_c = 0.0_f64; // sum of C_i * A_i * sin(B_i + C_i*t)

        for &(a, b, c) in terms.iter() {
            let angle = b + c * t;
            let cos_val = libm::cos(angle);
            let sin_val = libm::sin(angle);
            sum_cos += a * cos_val;
            sum_sin_c += a * c * sin_val;
        }

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
fn get_series(planet: Planet, coord: usize) -> [&'static [(f64, f64, f64)]; 6] {
    match (planet, coord) {
        // Mercury
        (Planet::Mercury, 0) => [
            coefficients::mercury::X0,
            coefficients::mercury::X1,
            coefficients::mercury::X2,
            coefficients::mercury::X3,
            coefficients::mercury::X4,
            coefficients::mercury::X5,
        ],
        (Planet::Mercury, 1) => [
            coefficients::mercury::Y0,
            coefficients::mercury::Y1,
            coefficients::mercury::Y2,
            coefficients::mercury::Y3,
            coefficients::mercury::Y4,
            coefficients::mercury::Y5,
        ],
        (Planet::Mercury, 2) => [
            coefficients::mercury::Z0,
            coefficients::mercury::Z1,
            coefficients::mercury::Z2,
            coefficients::mercury::Z3,
            coefficients::mercury::Z4,
            coefficients::mercury::Z5,
        ],
        // Venus
        (Planet::Venus, 0) => [
            coefficients::venus::X0,
            coefficients::venus::X1,
            coefficients::venus::X2,
            coefficients::venus::X3,
            coefficients::venus::X4,
            coefficients::venus::X5,
        ],
        (Planet::Venus, 1) => [
            coefficients::venus::Y0,
            coefficients::venus::Y1,
            coefficients::venus::Y2,
            coefficients::venus::Y3,
            coefficients::venus::Y4,
            coefficients::venus::Y5,
        ],
        (Planet::Venus, 2) => [
            coefficients::venus::Z0,
            coefficients::venus::Z1,
            coefficients::venus::Z2,
            coefficients::venus::Z3,
            coefficients::venus::Z4,
            coefficients::venus::Z5,
        ],
        // Earth
        (Planet::Earth, 0) => [
            coefficients::earth::X0,
            coefficients::earth::X1,
            coefficients::earth::X2,
            coefficients::earth::X3,
            coefficients::earth::X4,
            coefficients::earth::X5,
        ],
        (Planet::Earth, 1) => [
            coefficients::earth::Y0,
            coefficients::earth::Y1,
            coefficients::earth::Y2,
            coefficients::earth::Y3,
            coefficients::earth::Y4,
            coefficients::earth::Y5,
        ],
        (Planet::Earth, 2) => [
            coefficients::earth::Z0,
            coefficients::earth::Z1,
            coefficients::earth::Z2,
            coefficients::earth::Z3,
            coefficients::earth::Z4,
            coefficients::earth::Z5,
        ],
        // Mars
        (Planet::Mars, 0) => [
            coefficients::mars::X0,
            coefficients::mars::X1,
            coefficients::mars::X2,
            coefficients::mars::X3,
            coefficients::mars::X4,
            coefficients::mars::X5,
        ],
        (Planet::Mars, 1) => [
            coefficients::mars::Y0,
            coefficients::mars::Y1,
            coefficients::mars::Y2,
            coefficients::mars::Y3,
            coefficients::mars::Y4,
            coefficients::mars::Y5,
        ],
        (Planet::Mars, 2) => [
            coefficients::mars::Z0,
            coefficients::mars::Z1,
            coefficients::mars::Z2,
            coefficients::mars::Z3,
            coefficients::mars::Z4,
            coefficients::mars::Z5,
        ],
        // Jupiter
        (Planet::Jupiter, 0) => [
            coefficients::jupiter::X0,
            coefficients::jupiter::X1,
            coefficients::jupiter::X2,
            coefficients::jupiter::X3,
            coefficients::jupiter::X4,
            coefficients::jupiter::X5,
        ],
        (Planet::Jupiter, 1) => [
            coefficients::jupiter::Y0,
            coefficients::jupiter::Y1,
            coefficients::jupiter::Y2,
            coefficients::jupiter::Y3,
            coefficients::jupiter::Y4,
            coefficients::jupiter::Y5,
        ],
        (Planet::Jupiter, 2) => [
            coefficients::jupiter::Z0,
            coefficients::jupiter::Z1,
            coefficients::jupiter::Z2,
            coefficients::jupiter::Z3,
            coefficients::jupiter::Z4,
            coefficients::jupiter::Z5,
        ],
        // Saturn
        (Planet::Saturn, 0) => [
            coefficients::saturn::X0,
            coefficients::saturn::X1,
            coefficients::saturn::X2,
            coefficients::saturn::X3,
            coefficients::saturn::X4,
            coefficients::saturn::X5,
        ],
        (Planet::Saturn, 1) => [
            coefficients::saturn::Y0,
            coefficients::saturn::Y1,
            coefficients::saturn::Y2,
            coefficients::saturn::Y3,
            coefficients::saturn::Y4,
            coefficients::saturn::Y5,
        ],
        (Planet::Saturn, 2) => [
            coefficients::saturn::Z0,
            coefficients::saturn::Z1,
            coefficients::saturn::Z2,
            coefficients::saturn::Z3,
            coefficients::saturn::Z4,
            coefficients::saturn::Z5,
        ],
        // Uranus
        (Planet::Uranus, 0) => [
            coefficients::uranus::X0,
            coefficients::uranus::X1,
            coefficients::uranus::X2,
            coefficients::uranus::X3,
            coefficients::uranus::X4,
            coefficients::uranus::X5,
        ],
        (Planet::Uranus, 1) => [
            coefficients::uranus::Y0,
            coefficients::uranus::Y1,
            coefficients::uranus::Y2,
            coefficients::uranus::Y3,
            coefficients::uranus::Y4,
            coefficients::uranus::Y5,
        ],
        (Planet::Uranus, 2) => [
            coefficients::uranus::Z0,
            coefficients::uranus::Z1,
            coefficients::uranus::Z2,
            coefficients::uranus::Z3,
            coefficients::uranus::Z4,
            coefficients::uranus::Z5,
        ],
        // Neptune
        (Planet::Neptune, 0) => [
            coefficients::neptune::X0,
            coefficients::neptune::X1,
            coefficients::neptune::X2,
            coefficients::neptune::X3,
            coefficients::neptune::X4,
            coefficients::neptune::X5,
        ],
        (Planet::Neptune, 1) => [
            coefficients::neptune::Y0,
            coefficients::neptune::Y1,
            coefficients::neptune::Y2,
            coefficients::neptune::Y3,
            coefficients::neptune::Y4,
            coefficients::neptune::Y5,
        ],
        (Planet::Neptune, 2) => [
            coefficients::neptune::Z0,
            coefficients::neptune::Z1,
            coefficients::neptune::Z2,
            coefficients::neptune::Z3,
            coefficients::neptune::Z4,
            coefficients::neptune::Z5,
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

    let (px, vx) = eval_series(&get_series(planet, 0), t);
    let (py, vy) = eval_series(&get_series(planet, 1), t);
    let (pz, vz) = eval_series(&get_series(planet, 2), t);

    ([px, py, pz], [vx, vy, vz])
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
}
