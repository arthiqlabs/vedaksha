// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Chebyshev interpolation of smooth vector-valued functions of time.
//!
//! JPL's SPK kernels store each body as a sequence of Chebyshev polynomials,
//! one per short interval, and evaluate the polynomial at the requested time
//! (see [`crate::jpl`]). This module does the same for a function the caller
//! supplies, fitting the polynomial at run time instead of reading it from a
//! file: the analytical Moon and planets (see
//! [`crate::analytical::ChebyshevProvider`]) and an apparent-position scan
//! ([`crate::walk::BodyWalk`]) both build on it.
//!
//! # The fit
//!
//! A segment over `[start, end]` is fitted by interpolation at the `n`
//! Chebyshev nodes of the first kind,
//!
//! ```text
//! x_k = cos(π (k + ½) / n),     k = 0 … n − 1,     time_k = mid + half · x_k
//! ```
//!
//! which gives the degree `n − 1` polynomial `p(x) = Σ_j a_j T_j(x)` through the
//! `n` samples, with
//!
//! ```text
//! a_0 = (1/n) Σ_k f(x_k),     a_j = (2/n) Σ_k f(x_k) cos(j π (k + ½) / n)   (j ≥ 1).
//! ```
//!
//! That is exact for any polynomial of degree below `n`. Evaluation is
//! Clenshaw's recurrence; the derivative is evaluated from the derivative's own
//! Chebyshev coefficients, which follow from the recurrence
//! `d_{j-1} = d_{j+1} + 2 j a_j`.
//!
//! # The error
//!
//! For a function with `n` continuous derivatives the interpolation error on
//! the segment is bounded, at every point of it, by
//!
//! ```text
//! |f(x) − p(x)|  ≤  max|f⁽ⁿ⁾| · half^n / (2^(n−1) · n!)
//! ```
//!
//! (the standard remainder of polynomial interpolation, whose node polynomial
//! `Π (x − x_k)` is smallest, `half^n / 2^(n−1)`, at exactly these nodes).
//! For a sinusoid of angular frequency `ω` that is `(ω · half)^n / (2^(n−1) n!)`
//! times the amplitude, which falls off factorially in `n`: that is what makes
//! a dozen nodes enough for a lunar day or two at the precision of a `f64`.
//! `tests::sinusoid_error_is_inside_the_remainder_bound` checks the
//! implementation against that bound, and the callers state and test the error
//! of the specific fits they build.

/// A Chebyshev polynomial fit of a `C`-component function over one time
/// interval.
#[derive(Debug, Clone)]
pub struct Chebyshev<const C: usize> {
    mid: f64,
    half: f64,
    /// `coeffs[j]` is the coefficient of `T_j`, per component.
    coeffs: Vec<[f64; C]>,
    /// Chebyshev coefficients of the derivative with respect to the scaled
    /// variable `x`, one fewer than `coeffs`.
    deriv: Vec<[f64; C]>,
}

impl<const C: usize> Chebyshev<C> {
    /// Fit `f` over `[start, end]` with `n` nodes (degree `n − 1`).
    ///
    /// `f` is called exactly `n` times, in order of decreasing time.
    ///
    /// # Panics
    ///
    /// If `n < 2` or `end <= start`.
    pub fn fit(start: f64, end: f64, n: usize, mut f: impl FnMut(f64) -> [f64; C]) -> Self {
        match Self::try_fit(start, end, n, |t| Ok::<_, core::convert::Infallible>(f(t))) {
            Ok(fit) => fit,
            Err(never) => match never {},
        }
    }

    /// [`Self::fit`] for a function that can fail; the first error is
    /// returned and the remaining samples are not taken.
    ///
    /// # Errors
    ///
    /// Whatever `f` returns.
    ///
    /// # Panics
    ///
    /// If `n < 2` or `end <= start`.
    pub fn try_fit<E>(
        start: f64,
        end: f64,
        n: usize,
        mut f: impl FnMut(f64) -> Result<[f64; C], E>,
    ) -> Result<Self, E> {
        assert!(n >= 2, "a Chebyshev fit needs at least two nodes");
        assert!(end > start, "a Chebyshev segment needs end > start");
        let mid = 0.5 * (start + end);
        let half = 0.5 * (end - start);

        let nf = n as f64;
        let mut samples: Vec<[f64; C]> = Vec::with_capacity(n);
        // `rounding[k]` is how far the node time `mid + half·x_k` lies above the
        // double it was rounded to (an exact quantity, from two-sum).
        let mut rounding: Vec<f64> = Vec::with_capacity(n);
        let mut node_x: Vec<f64> = Vec::with_capacity(n);
        for k in 0..n {
            let x = libm::cos(core::f64::consts::PI * (k as f64 + 0.5) / nf);
            let offset = half * x;
            let time = mid + offset;
            let virtual_part = time - mid;
            rounding.push((mid - (time - virtual_part)) + (offset - virtual_part));
            node_x.push(x);
            samples.push(f(time)?);
        }

        let mut coeffs = dct_coefficients(&samples);
        let mut deriv = derivative_coefficients(&coeffs);

        // A Julian date near 2.45e6 is a multiple of 4.7e-10 day, so the sample
        // at `mid + half·x_k` was in fact taken up to 2.3e-10 day away from it.
        // For a body moving at v that is a position error of v · 2.3e-10 day:
        // 0.9 m for Mercury, 2 cm for the Moon, a floor no number of nodes
        // lowers. Move each sample to the node it stands for, to first order,
        // with the derivative of the fit just made (accurate to ~1e-6 relative,
        // so the correction's own error is of order 1e-16 of the sample).
        for ((sample, &x), &e) in samples.iter_mut().zip(&node_x).zip(&rounding) {
            let slope = clenshaw(&deriv, x);
            for (v, d) in sample.iter_mut().zip(slope) {
                *v += d / half * e;
            }
        }
        coeffs = dct_coefficients(&samples);
        deriv = derivative_coefficients(&coeffs);

        Ok(Self {
            mid,
            half,
            coeffs,
            deriv,
        })
    }

    /// First time covered.
    #[must_use]
    pub fn start(&self) -> f64 {
        self.mid - self.half
    }

    /// Last time covered.
    #[must_use]
    pub fn end(&self) -> f64 {
        self.mid + self.half
    }

    /// Number of nodes the fit was built from.
    #[must_use]
    pub fn nodes(&self) -> usize {
        self.coeffs.len()
    }

    /// The fitted function at `time`. Outside `[start, end]` the polynomial
    /// still evaluates, but it is an extrapolation and diverges quickly.
    #[must_use]
    pub fn value(&self, time: f64) -> [f64; C] {
        clenshaw(&self.coeffs, (time - self.mid) / self.half)
    }

    /// The fitted function's derivative with respect to time.
    #[must_use]
    pub fn derivative(&self, time: f64) -> [f64; C] {
        let mut d = clenshaw(&self.deriv, (time - self.mid) / self.half);
        for v in &mut d {
            *v /= self.half;
        }
        d
    }
}

/// Chebyshev coefficients of the interpolant through `samples` taken at the
/// nodes `x_k = cos(π (k + ½) / n)`.
fn dct_coefficients<const C: usize>(samples: &[[f64; C]]) -> Vec<[f64; C]> {
    let n = samples.len();
    let nf = n as f64;
    let mut coeffs = vec![[0.0_f64; C]; n];
    for (j, coeff) in coeffs.iter_mut().enumerate() {
        let jf = j as f64;
        let mut acc = [0.0_f64; C];
        for (k, sample) in samples.iter().enumerate() {
            let t_j = libm::cos(core::f64::consts::PI * jf * (k as f64 + 0.5) / nf);
            for (a, &s) in acc.iter_mut().zip(sample) {
                *a += s * t_j;
            }
        }
        let scale = if j == 0 { 1.0 / nf } else { 2.0 / nf };
        for (c, a) in coeff.iter_mut().zip(acc) {
            *c = a * scale;
        }
    }
    coeffs
}

/// `Σ_j a_j T_j(x)` by Clenshaw's recurrence, per component.
fn clenshaw<const C: usize>(a: &[[f64; C]], x: f64) -> [f64; C] {
    let mut b1 = [0.0_f64; C];
    let mut b2 = [0.0_f64; C];
    for coeff in a.iter().skip(1).rev() {
        for c in 0..C {
            let b0 = coeff[c] + 2.0 * x * b1[c] - b2[c];
            b2[c] = b1[c];
            b1[c] = b0;
        }
    }
    let mut out = [0.0_f64; C];
    for c in 0..C {
        out[c] = a[0][c] + x * b1[c] - b2[c];
    }
    out
}

/// Chebyshev coefficients of the derivative with respect to `x`, in the same
/// convention as the input (the `j = 0` coefficient is the plain one).
fn derivative_coefficients<const C: usize>(a: &[[f64; C]]) -> Vec<[f64; C]> {
    let n = a.len();
    // Work in the convention where the j = 0 coefficient is doubled, so the
    // three-term recurrence is uniform, then halve it back.
    let mut d = vec![[0.0_f64; C]; n + 1];
    for j in (1..n).rev() {
        for c in 0..C {
            d[j - 1][c] = d[j + 1][c] + 2.0 * (j as f64) * a[j][c];
        }
    }
    d.truncate(n.max(2) - 1);
    for c in 0..C {
        d[0][c] *= 0.5;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_polynomial_below_the_degree_is_recovered_exactly() {
        // 1 + 2t + 3t^2 - 0.5 t^3 over [-2, 5], four nodes (degree 3).
        let f = |t: f64| [1.0 + 2.0 * t + 3.0 * t * t - 0.5 * t * t * t, 7.0 - t];
        let fit = Chebyshev::<2>::fit(-2.0, 5.0, 4, f);
        for k in 0..=70 {
            let t = -2.0 + 0.1 * f64::from(k);
            let (got, want) = (fit.value(t), f(t));
            assert!(
                (got[0] - want[0]).abs() < 1e-12,
                "t = {t}: {got:?} vs {want:?}"
            );
            assert!((got[1] - want[1]).abs() < 1e-12);
        }
    }

    #[test]
    fn the_derivative_of_a_polynomial_is_recovered_exactly() {
        let f = |t: f64| [1.0 + 2.0 * t + 3.0 * t * t - 0.5 * t * t * t];
        let fit = Chebyshev::<1>::fit(-2.0, 5.0, 4, f);
        for k in 0..=70 {
            let t = -2.0 + 0.1 * f64::from(k);
            let want = 2.0 + 6.0 * t - 1.5 * t * t;
            let got = fit.derivative(t)[0];
            assert!((got - want).abs() < 1e-11, "t = {t}: {got} vs {want}");
        }
    }

    /// The interpolation remainder bound from the module doc, which is a
    /// property of the nodes and not of this implementation: if the nodes or the
    /// coefficients were wrong the error would exceed it by orders of magnitude.
    #[test]
    fn sinusoid_error_is_inside_the_remainder_bound() {
        let (omega, amp) = (2.3_f64, 0.75_f64);
        let (start, end) = (10.0, 13.0);
        let half = 0.5 * (end - start);
        for n in [6_usize, 9, 12] {
            let fit = Chebyshev::<1>::fit(start, end, n, |t| [amp * libm::sin(omega * t + 0.4)]);
            let factorial: f64 = (1..=n).map(|k| k as f64).product();
            let bound =
                amp * (omega * half).powi(n as i32) / (2.0_f64.powi(n as i32 - 1) * factorial);
            let mut worst = 0.0_f64;
            for k in 0..=3000 {
                let t = start + (end - start) * f64::from(k) / 3000.0;
                worst = worst.max((fit.value(t)[0] - amp * libm::sin(omega * t + 0.4)).abs());
            }
            // Allow a little for the rounding floor once the bound is below it.
            assert!(
                worst <= bound * 1.01 + 2e-15,
                "n = {n}: error {worst:e} vs bound {bound:e}"
            );
            // And the bound is not vacuous: the error is within a small factor of it
            // while it is above the rounding floor.
            if bound > 1e-12 {
                assert!(
                    worst > bound * 1e-3,
                    "n = {n}: error {worst:e} suspiciously far inside {bound:e}"
                );
            }
        }
    }

    #[test]
    fn the_derivative_of_a_sinusoid_converges() {
        let (omega, amp) = (2.3_f64, 0.75_f64);
        // The derivative converges more slowly than the value, by a factor of
        // about n^2 / half; twenty nodes put the value at 1e-14 and this at 1e-11.
        let fit = Chebyshev::<1>::fit(10.0, 13.0, 20, |t| [amp * libm::sin(omega * t)]);
        for k in 0..=300 {
            let t = 10.0 + 0.01 * f64::from(k);
            let want = amp * omega * libm::cos(omega * t);
            assert!((fit.derivative(t)[0] - want).abs() < 1e-9, "t = {t}");
        }
    }

    #[test]
    fn samples_are_taken_at_the_requested_node_count_and_inside_the_interval() {
        let mut calls = Vec::new();
        let _ = Chebyshev::<1>::fit(5.0, 9.0, 7, |t| {
            calls.push(t);
            [t]
        });
        assert_eq!(calls.len(), 7);
        assert!(calls.iter().all(|&t| t > 5.0 && t < 9.0));
        assert!(
            calls.windows(2).all(|w| w[0] > w[1]),
            "nodes are in decreasing time"
        );
    }

    /// A Julian date is a multiple of 4.7e-10 day, so a node time is rounded;
    /// without the correction the fit of a linear function of time is off by its
    /// slope times that rounding. Here the slope is 1e5 per day, so the
    /// uncorrected error would be ~2e-5 and the corrected one is ~1e-10.
    #[test]
    fn rounding_of_the_node_times_is_corrected() {
        let base = 2_451_545.0;
        let f = |t: f64| [1.0e5 * (t - base - 0.3)];
        let fit = Chebyshev::<1>::fit(base, base + 3.0, 6, f);
        let mut worst = 0.0_f64;
        for k in 0..=300 {
            let t = base + 0.01 * f64::from(k);
            worst = worst.max((fit.value(t)[0] - f(t)[0]).abs());
        }
        assert!(
            worst < 1e-8,
            "linear function off by {worst:e}; the node-time rounding is back"
        );
    }

    #[test]
    fn the_first_error_stops_the_fit() {
        let mut taken = 0;
        let r = Chebyshev::<1>::try_fit(0.0, 1.0, 8, |t| {
            taken += 1;
            if taken == 3 { Err("boom") } else { Ok([t]) }
        });
        assert_eq!(r.unwrap_err(), "boom");
        assert_eq!(taken, 3);
    }

    #[test]
    #[should_panic(expected = "at least two nodes")]
    fn one_node_is_refused() {
        let _ = Chebyshev::<1>::fit(0.0, 1.0, 1, |t| [t]);
    }
}
