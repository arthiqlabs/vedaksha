// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Polynomial interpolation methods.
//!
//! Provides Hermite interpolation (for position+velocity pairs) and
//! Lagrange interpolation (for arbitrary-order polynomial fitting).
//!
//! Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 3;
//! Hildebrand, "Introduction to Numerical Analysis", Ch. 2.

#[cfg(not(feature = "std"))]
use alloc::vec;

/// Standard Lagrange basis polynomial interpolation.
///
/// Given `n` data points `(xs[i], ys[i])`, evaluates the unique polynomial of
/// degree ≤ n−1 that passes through all points at parameter `t`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths or are empty.
#[must_use]
pub fn lagrange_interpolate(xs: &[f64], ys: &[f64], t: f64) -> f64 {
    assert_eq!(
        xs.len(),
        ys.len(),
        "lagrange: x and y must have same length"
    );
    assert!(
        !xs.is_empty(),
        "lagrange: must have at least one data point"
    );
    let n = xs.len();
    let mut result = 0.0;
    for i in 0..n {
        let mut basis = 1.0;
        for j in 0..n {
            if i != j {
                basis *= (t - xs[j]) / (xs[i] - xs[j]);
            }
        }
        result += ys[i] * basis;
    }
    result
}

/// Hermite interpolation returning `(position, velocity)`.
///
/// Given `n` knots with positions `ys[i]` and derivatives `dys[i]` at `xs[i]`,
/// evaluates using the divided-difference formulation with duplicated knots.
///
/// # Panics
///
/// Panics if `xs`, `ys`, and `dys` do not all have the same length, or are empty.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::needless_range_loop)]
pub fn hermite_interpolate(xs: &[f64], ys: &[f64], dys: &[f64], t: f64) -> (f64, f64) {
    assert_eq!(xs.len(), ys.len(), "hermite: x and y must have same length");
    assert_eq!(
        xs.len(),
        dys.len(),
        "hermite: x and dy must have same length"
    );
    assert!(!xs.is_empty(), "hermite: must have at least one knot point");
    let n = xs.len();
    if n == 1 {
        let dt = t - xs[0];
        return (ys[0] + dys[0] * dt, dys[0]);
    }
    // Build 2n-point divided difference table
    let m = 2 * n;
    let mut z = vec![0.0_f64; m];
    let mut q = vec![vec![0.0_f64; m]; m];
    for i in 0..n {
        z[2 * i] = xs[i];
        z[2 * i + 1] = xs[i];
        q[2 * i][0] = ys[i];
        q[2 * i + 1][0] = ys[i];
        q[2 * i + 1][1] = dys[i];
        if i > 0 {
            q[2 * i][1] = (q[2 * i][0] - q[2 * i - 1][0]) / (z[2 * i] - z[2 * i - 1]);
        }
    }
    for j in 2..m {
        for i in j..m {
            if (z[i] - z[i - j]).abs() < f64::EPSILON {
                q[i][j] = 0.0;
            } else {
                q[i][j] = (q[i][j - 1] - q[i - 1][j - 1]) / (z[i] - z[i - j]);
            }
        }
    }
    // Newton form for position
    let mut pos = q[0][0];
    let mut product = 1.0_f64;
    for i in 1..m {
        product *= t - z[i - 1];
        pos += q[i][i] * product;
    }
    // Derivative of Newton form
    let mut vel = 0.0_f64;
    #[allow(clippy::needless_range_loop)]
    for i in 1..m {
        let mut sum = 0.0_f64;
        for k in 0..i {
            let mut prod = 1.0_f64;
            for j in 0..i {
                if j != k {
                    prod *= t - z[j];
                }
            }
            sum += prod;
        }
        vel += q[i][i] * sum;
    }
    (pos, vel)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn lagrange_linear() {
        // y = 2x + 1; two points: (0, 1), (1, 3)
        let xs = [0.0, 1.0];
        let ys = [1.0, 3.0];
        let result = lagrange_interpolate(&xs, &ys, 0.5);
        assert!((result - 2.0).abs() < EPS, "expected 2.0, got {result}");
    }

    #[test]
    fn lagrange_quadratic() {
        // y = x^2; three points: (0, 0), (1, 1), (2, 4)
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0, 4.0];
        let result = lagrange_interpolate(&xs, &ys, 0.5);
        assert!((result - 0.25).abs() < EPS, "expected 0.25, got {result}");
    }

    #[test]
    fn lagrange_exact_at_knots() {
        // y = x^3; four points
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys: Vec<f64> = xs.iter().map(|&v| v * v * v).collect();
        for (&xi, &yi) in xs.iter().zip(ys.iter()) {
            let result = lagrange_interpolate(&xs, &ys, xi);
            assert!(
                (result - yi).abs() < EPS,
                "at x={xi}: expected {yi}, got {result}"
            );
        }
    }

    #[test]
    fn lagrange_cubic() {
        // y = x^3; four points, evaluate at t=2.5 → 15.625
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys: Vec<f64> = xs.iter().map(|&v| v * v * v).collect();
        let result = lagrange_interpolate(&xs, &ys, 2.5);
        assert!(
            (result - 15.625).abs() < 1e-10,
            "expected 15.625, got {result}"
        );
    }

    #[test]
    fn hermite_linear() {
        // f(x) = 3x + 1; two knots at x=0 and x=1
        // f(0)=1, f'(0)=3, f(1)=4, f'(1)=3
        let xs = [0.0, 1.0];
        let ys = [1.0, 4.0];
        let dys = [3.0, 3.0];
        let (pos, vel) = hermite_interpolate(&xs, &ys, &dys, 0.5);
        assert!((pos - 2.5).abs() < EPS, "expected pos=2.5, got {pos}");
        assert!((vel - 3.0).abs() < EPS, "expected vel=3.0, got {vel}");
    }

    #[test]
    fn hermite_quadratic() {
        // f(x) = x^2; two knots at x=0 and x=2
        // f(0)=0, f'(0)=0, f(2)=4, f'(2)=4
        // evaluate at t=1 → f(1)=1, f'(1)=2
        let xs = [0.0, 2.0];
        let ys = [0.0, 4.0];
        let dys = [0.0, 4.0];
        let (pos, vel) = hermite_interpolate(&xs, &ys, &dys, 1.0);
        assert!((pos - 1.0).abs() < EPS, "expected pos=1.0, got {pos}");
        assert!((vel - 2.0).abs() < EPS, "expected vel=2.0, got {vel}");
    }

    #[test]
    fn hermite_exact_at_knots() {
        // f(x) = x^3; three knots at x=0, 1, 2
        let xs = [0.0, 1.0, 2.0];
        let ys: Vec<f64> = xs.iter().map(|&v| v * v * v).collect();
        let dys: Vec<f64> = xs.iter().map(|&v| 3.0 * v * v).collect();
        for (&xi, &yi) in xs.iter().zip(ys.iter()) {
            let (pos, _) = hermite_interpolate(&xs, &ys, &dys, xi);
            assert!(
                (pos - yi).abs() < 1e-10,
                "at x={xi}: expected {yi}, got {pos}"
            );
        }
    }

    #[test]
    fn hermite_cubic() {
        // f(x) = x^3; two knots at x=0 and x=1
        // f(0)=0, f'(0)=0, f(1)=1, f'(1)=3
        // evaluate at t=0.5 → f(0.5)=0.125, f'(0.5)=0.75
        let xs = [0.0, 1.0];
        let ys = [0.0, 1.0];
        let dys = [0.0, 3.0];
        let (pos, vel) = hermite_interpolate(&xs, &ys, &dys, 0.5);
        assert!((pos - 0.125).abs() < EPS, "expected pos=0.125, got {pos}");
        assert!((vel - 0.75).abs() < EPS, "expected vel=0.75, got {vel}");
    }

    #[test]
    fn hermite_single_knot() {
        // Single knot: linear extrapolation from (2.0, 5.0) with slope 3.0
        // At t=4.0: pos = 5.0 + 3.0*(4.0-2.0) = 11.0, vel = 3.0
        let xs = [2.0];
        let ys = [5.0];
        let dys = [3.0];
        let (pos, vel) = hermite_interpolate(&xs, &ys, &dys, 4.0);
        assert!((pos - 11.0).abs() < EPS, "expected pos=11.0, got {pos}");
        assert!((vel - 3.0).abs() < EPS, "expected vel=3.0, got {vel}");
    }
}
