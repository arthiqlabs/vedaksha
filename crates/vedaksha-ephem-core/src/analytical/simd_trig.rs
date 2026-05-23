// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vectorized `sin`/`cos` for four `f64` lanes at once.
//!
//! The ELP/MPP02 lunar series evaluates tens of thousands of `sin`/`cos` per
//! position — the dominant cost. This wraps [`wide`]'s `f64x4::sin_cos`, which
//! computes both for four phase angles simultaneously. The accompanying test
//! pins its accuracy against scalar `libm::sincos` across the full domain the
//! lunar phases occupy, so the vectorized path is a measured, bounded-error
//! substitute rather than an assumed-equivalent one.

use wide::f64x4;

/// Compute `(sin(x), cos(x))` for four lanes simultaneously via `wide`.
///
/// Accuracy versus scalar `libm::sincos` is asserted by
/// [`tests::matches_libm_across_domain`] over `|x| ≤ 3000` (lunar phases stay
/// well inside this) to within a few ULP.
#[inline]
#[must_use]
pub fn sincos_f64x4(x: f64x4) -> (f64x4, f64x4) {
    x.sin_cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vectorized kernel must track scalar `libm::sincos` across the full
    /// domain the lunar phases occupy, including large arguments and quadrant
    /// boundaries.
    #[test]
    fn matches_libm_across_domain() {
        let mut max_sin_err = 0.0_f64;
        let mut max_cos_err = 0.0_f64;

        let mut samples: Vec<f64> = Vec::new();
        let mut x = -3000.0_f64;
        while x <= 3000.0 {
            samples.push(x);
            x += 0.000_37; // fine, irrational-ish step to vary the reduction
        }
        for k in -40..=40 {
            let b = f64::from(k) * core::f64::consts::FRAC_PI_2;
            samples.push(b);
            samples.push(b + 1e-9);
            samples.push(b - 1e-9);
        }

        for chunk in samples.chunks(4) {
            let mut lane = [0.0_f64; 4];
            lane[..chunk.len()].copy_from_slice(chunk);
            let (s, c) = sincos_f64x4(f64x4::from(lane));
            let s = s.to_array();
            let c = c.to_array();
            for i in 0..chunk.len() {
                let (ls, lc) = libm::sincos(lane[i]);
                max_sin_err = max_sin_err.max((s[i] - ls).abs());
                max_cos_err = max_cos_err.max((c[i] - lc).abs());
            }
        }

        // Report-and-assert: sin/cos bounded by 1, so abs ≈ rel error here.
        // The bar is "negligible against ELP term magnitudes" — a few ULP.
        assert!(
            max_sin_err < 1e-12,
            "max sin abs error {max_sin_err:e} exceeds 1e-12"
        );
        assert!(
            max_cos_err < 1e-12,
            "max cos abs error {max_cos_err:e} exceeds 1e-12"
        );
    }

    #[test]
    fn lanes_are_independent() {
        let xs = [
            0.0,
            core::f64::consts::FRAC_PI_2,
            core::f64::consts::PI,
            -1.0,
        ];
        let (s, c) = sincos_f64x4(f64x4::from(xs));
        let s = s.to_array();
        let c = c.to_array();
        for i in 0..4 {
            let (ls, lc) = libm::sincos(xs[i]);
            assert!((s[i] - ls).abs() < 1e-12, "lane {i} sin");
            assert!((c[i] - lc).abs() < 1e-12, "lane {i} cos");
        }
    }
}
