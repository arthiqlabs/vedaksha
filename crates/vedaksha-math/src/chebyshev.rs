// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Chebyshev polynomial evaluation for JPL ephemeris interpolation.
//!
//! Evaluates Chebyshev polynomials of the first kind using Clenshaw's
//! recurrence relation. Returns both value and first derivative (for velocity).
//!
//! JPL DE441 stores planetary positions as Chebyshev coefficients; this module
//! turns those coefficients into positions and velocities.
//!
//! Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 3;
//!         Clenshaw (1955), "A note on the summation of Chebyshev series", MTAC 9.

/// Evaluates a Chebyshev series and its derivative using Clenshaw's recurrence.
///
/// Given coefficients `c[0..n]` and a normalized argument `x` in `[-1, 1]`,
/// returns `(value, derivative)` where `value = sum(c[k] * T_k(x))` and
/// `derivative` is with respect to `x`.
///
/// # Panics
///
/// Panics if `coeffs` is empty.
#[must_use]
pub fn chebyshev_compute(coeffs: &[f64], x: f64) -> (f64, f64) {
    assert!(
        !coeffs.is_empty(),
        "chebyshev_compute: coefficients must not be empty"
    );
    let n = coeffs.len();
    if n == 1 {
        return (coeffs[0], 0.0);
    }
    let two_x = 2.0 * x;
    let mut b_k1 = 0.0;
    let mut b_k2 = 0.0;
    let mut db_k1 = 0.0;
    let mut db_k2 = 0.0;
    for k in (1..n).rev() {
        let b_k = two_x * b_k1 - b_k2 + coeffs[k];
        let db_k = 2.0 * b_k1 + two_x * db_k1 - db_k2;
        b_k2 = b_k1;
        b_k1 = b_k;
        db_k2 = db_k1;
        db_k1 = db_k;
    }
    let value = x * b_k1 - b_k2 + coeffs[0];
    let derivative = b_k1 + x * db_k1 - db_k2;
    (value, derivative)
}

/// Maps a time `t` in the interval `[t_start, t_end]` to the normalized
/// Chebyshev argument in `[-1, 1]`.
///
/// # Panics
///
/// Panics if `t_start == t_end` (zero-length interval).
#[must_use]
pub fn normalize_time(t: f64, t_start: f64, t_end: f64) -> f64 {
    assert!(
        (t_end - t_start).abs() > f64::EPSILON,
        "normalize_time: zero-length interval (t_start == t_end)"
    );
    2.0 * (t - t_start) / (t_end - t_start) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T_0(x) = 1 for all x; derivative = 0.
    #[test]
    fn compute_constant() {
        let coeffs = [1.0_f64];
        let (val, deriv) = chebyshev_compute(&coeffs, 0.5);
        assert!((val - 1.0).abs() < 1e-15, "value: {val}");
        assert!((deriv - 0.0).abs() < 1e-15, "deriv: {deriv}");
    }

    /// T_1(x) = x; derivative = 1.
    #[test]
    fn compute_linear() {
        // c = [0, 1] => 0*T_0 + 1*T_1 = x
        let coeffs = [0.0_f64, 1.0];
        let x = 0.7;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - x).abs() < 1e-15, "value: {val}");
        assert!((deriv - 1.0).abs() < 1e-15, "deriv: {deriv}");
    }

    /// T_2(x) = 2x^2 - 1; derivative = 4x.
    #[test]
    fn compute_quadratic() {
        // c = [0, 0, 1] => T_2(x) = 2x^2 - 1
        let coeffs = [0.0_f64, 0.0, 1.0];
        let x = 0.3;
        let expected_val = 2.0 * x * x - 1.0;
        let expected_deriv = 4.0 * x;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - expected_val).abs() < 1e-15, "value: {val}");
        assert!((deriv - expected_deriv).abs() < 1e-15, "deriv: {deriv}");
    }

    /// T_3(x) = 4x^3 - 3x; derivative = 12x^2 - 3.
    #[test]
    fn compute_cubic() {
        // c = [0, 0, 0, 1] => T_3(x)
        let coeffs = [0.0_f64, 0.0, 0.0, 1.0];
        let x = 0.5;
        let expected_val = 4.0 * x * x * x - 3.0 * x;
        let expected_deriv = 12.0 * x * x - 3.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - expected_val).abs() < 1e-15, "value: {val}");
        assert!((deriv - expected_deriv).abs() < 1e-15, "deriv: {deriv}");
    }

    /// 3*T_0 + 2*T_1 + 1*T_2 = 3 + 2x + (2x^2 - 1) = 2x^2 + 2x + 2.
    /// Derivative = 4x + 2.
    #[test]
    fn compute_mixed_coefficients() {
        let coeffs = [3.0_f64, 2.0, 1.0];
        let x = 0.4;
        let expected_val = 2.0 * x * x + 2.0 * x + 2.0;
        let expected_deriv = 4.0 * x + 2.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - expected_val).abs() < 1e-14, "value: {val}");
        assert!((deriv - expected_deriv).abs() < 1e-14, "deriv: {deriv}");
    }

    /// Evaluate at x = -1 boundary: T_n(-1) = (-1)^n.
    #[test]
    fn compute_at_boundary_neg1() {
        // T_2(-1) = 2*1 - 1 = 1; T_2'(-1) = 4*(-1) = -4
        let coeffs = [0.0_f64, 0.0, 1.0];
        let x = -1.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - 1.0).abs() < 1e-15, "value: {val}");
        assert!((deriv - (-4.0)).abs() < 1e-15, "deriv: {deriv}");
    }

    /// Evaluate at x = +1 boundary: T_n(1) = 1.
    #[test]
    fn compute_at_boundary_pos1() {
        // T_2(1) = 2*1 - 1 = 1; T_2'(1) = 4
        let coeffs = [0.0_f64, 0.0, 1.0];
        let x = 1.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - 1.0).abs() < 1e-15, "value: {val}");
        assert!((deriv - 4.0).abs() < 1e-15, "deriv: {deriv}");
    }

    /// Evaluate at x = 0: T_2(0) = -1; T_2'(0) = 0.
    #[test]
    fn compute_at_center() {
        let coeffs = [0.0_f64, 0.0, 1.0];
        let x = 0.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - (-1.0)).abs() < 1e-15, "value: {val}");
        assert!((deriv - 0.0).abs() < 1e-15, "deriv: {deriv}");
    }

    /// T_5(x) = 16x^5 - 20x^3 + 5x; derivative = 80x^4 - 60x^2 + 5.
    #[test]
    fn compute_degree5() {
        let coeffs = [0.0_f64, 0.0, 0.0, 0.0, 0.0, 1.0];
        let x: f64 = 0.6;
        let expected_val = 16.0 * x.powi(5) - 20.0 * x.powi(3) + 5.0 * x;
        let expected_deriv = 80.0 * x.powi(4) - 60.0 * x.powi(2) + 5.0;
        let (val, deriv) = chebyshev_compute(&coeffs, x);
        assert!((val - expected_val).abs() < 1e-15, "value: {val}");
        assert!((deriv - expected_deriv).abs() < 1e-15, "deriv: {deriv}");
    }

    /// normalize_time at center of interval maps to 0.
    #[test]
    fn normalize_time_center() {
        let result = normalize_time(1.5, 1.0, 2.0);
        assert!((result - 0.0).abs() < 1e-15, "result: {result}");
    }

    /// normalize_time at start of interval maps to -1.
    #[test]
    fn normalize_time_start() {
        let result = normalize_time(1.0, 1.0, 2.0);
        assert!((result - (-1.0)).abs() < 1e-15, "result: {result}");
    }

    /// normalize_time at end of interval maps to +1.
    #[test]
    fn normalize_time_end() {
        let result = normalize_time(2.0, 1.0, 2.0);
        assert!((result - 1.0).abs() < 1e-15, "result: {result}");
    }

    /// normalize_time at quarter-point maps to -0.5.
    #[test]
    fn normalize_time_quarter() {
        let result = normalize_time(1.25, 1.0, 2.0);
        assert!((result - (-0.5)).abs() < 1e-15, "result: {result}");
    }
}
