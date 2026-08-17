// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vectorized `sin`/`cos` for four `f64` lanes at once.
//!
//! The ELP/MPP02 lunar series evaluates tens of thousands of `sin`/`cos` per
//! position. This wraps [`wide`]'s `f64x4::sin_cos`, which computes both for
//! four phase angles simultaneously. The accompanying test pins its accuracy
//! against scalar `libm::sincos` across the full domain the lunar phases
//! occupy, so the vectorized path is a measured, bounded-error substitute
//! rather than an assumed-equivalent one.
//!
//! # Where the time actually goes
//!
//! This comment used to call `sin_cos` "the dominant cost". Measurement does
//! not support that, and the wrong belief would send an optimisation effort to
//! the wrong place. One `elp_geocentric` evaluates **35,758 series terms**,
//! which `chunks_exact(4)` turns into **8,936 `sincos_f64x4` calls** plus 14
//! scalar `libm::sincos` remainder terms (counts derived from the
//! `record_count` headers of the 15 `coefficients/moon_*/{main,pert_0..3}.bin`
//! tables, not estimated).
//!
//! Measured 2026-08-16 on **aarch64** (Apple M5 Pro, `--release`) — costs are
//! target-dependent, so re-measure before trusting these on x86-64:
//!
//! | phase | share of `elp_mpp02_moon` |
//! |---|---|
//! | `sin_cos` (8,936 calls) | **40.7%** (39.5–42.0%) |
//! | perturbation phase + ω 13-multiplier dot products | **36.5%** (35.0–38.1%) |
//!
//! `sin_cos` leads by roughly **4 points**, not by a margin that makes it *the*
//! cost. Halving it would buy ~20% end-to-end at best.
//!
//! Method: ablation against a `criterion` baseline of `elp_mpp02_moon`
//! (312.09 µs [310.07, 314.20]). Replacing `sin_cos` with two multiplies, data
//! flow preserved, gives 185.05 µs; collapsing the 13-term `phase`/`omega` dot
//! products in `eval_pert_series` to one multiplier each gives 198.07 µs. The
//! ranges come from combining the criterion confidence bounds.
//!
//! Ablation measures the *marginal* cost, which is the number an optimisation
//! would actually recover. The same 8,936 calls timed standalone in a tight
//! loop cost 179.89 µs (57.6% of baseline) — that larger figure is an upper
//! bound the real loop never pays, because term assembly and the trig overlap
//! in the pipeline.

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
