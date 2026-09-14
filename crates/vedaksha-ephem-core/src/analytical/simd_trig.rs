// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Vectorized `sin`/`cos` for four `f64` lanes at once.
//!
//! The ELP/MPP02 lunar series evaluates tens of thousands of `sin`/`cos` per
//! position. This computes both for four phase angles simultaneously with
//! [`wide`]'s `f64x4::sin_cos` algorithm (on AVX targets wide's own function,
//! elsewhere a bit-identical port — see [`sincos_f64x4`]). The accompanying test pins its accuracy
//! against scalar `libm::sincos` across the full domain the lunar phases
//! occupy, so the vectorized path is a measured, bounded-error substitute
//! rather than an assumed-equivalent one.
//!
//! # Validated domain (corrected 2026-08-29)
//!
//! Through v7.5.0 this comment and [`tests::matches_libm_across_domain`]
//! validated only `|x| ≤ 3000`, on the assumption that "lunar phases stay
//! well inside this". That assumption was never checked against what
//! `elp_mpp02.rs` actually feeds this kernel. It does not hold: computing
//! the real argument polynomials and real per-term integer multipliers
//! (`docs/audit/2026-08-29-perf-investigation.md` §3b), the maximum `|phase|`
//! reaching [`sincos_f64x4`] across `AnalyticalProvider`'s full supported JD
//! range (`JD_MIN`/`JD_MAX` in `analytical/mod.rs`, ~-2000 CE to ~+3000 CE)
//! is **3,303,561 rad**, at the −2000 CE edge — over 1,000× beyond the
//! previously validated bound. A present-day (2025) chart alone already
//! reaches ~21,000 rad, 7× beyond it.
//!
//! `elp_mpp02.rs`'s test `sincos_matches_libm_at_real_elp_phase_domain`
//! measures `sincos_f64x4` against scalar `libm::sincos` at every real
//! term-phase value the kernel is actually called with across that full
//! range, plus a dense sweep of the entire reachable interval. **Measured
//! 2026-08-29 (aarch64, both `--release` LTO/codegen-units=1 profile and
//! debug profile — identical): max sin error 1.11e-16, max cos error
//! 2.22e-16 — 1-2 ULP**, indistinguishable from the error at small
//! arguments and many orders of magnitude below the project's ~0.169″
//! theory floor. `wide::f64x4::sin_cos`'s large-argument reduction does not
//! degrade at this domain; the kernel was accidentally under-documented,
//! not actually broken. The validated domain below is widened to match
//! reality rather than being narrowed to match the doc.
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
/// [`tests::matches_libm_across_domain`] over `|x| ≤ 3,400,000` — covering
/// every phase `elp_mpp02.rs`'s main and perturbation series actually reach
/// across `AnalyticalProvider`'s full supported JD range (measured maximum
/// 3,303,561 rad; see the module doc above) — to within 1-2 ULP.
///
/// Without AVX this runs [`const_lanes::sin_cos`], a copy of `wide`'s own
/// algorithm that differs only in how its constants are built — see there for
/// why, and [`tests::const_lane_port_is_bit_identical_to_wide`] for the proof
/// that the bits do not change.
#[inline]
#[must_use]
pub fn sincos_f64x4(x: f64x4) -> (f64x4, f64x4) {
    #[cfg(target_feature = "avx")]
    {
        x.sin_cos()
    }
    #[cfg(not(target_feature = "avx"))]
    {
        const_lanes::sin_cos(x)
    }
}

/// `wide` 1.6.1's `f64x4::sin_cos`, with every lane constant a Rust `const`.
///
/// # Why this exists
///
/// Without AVX, `wide::f64x4` is a pair of `f64x2`. Its `sin_cos` builds nine
/// of its constants at run time through `splat` (`f64x4::from(0.5)`,
/// `i64x4::from(1)`, the masks inside `is_finite` and `flip_signs`). `splat`
/// is a `[x; 4]` repeat, which LLVM rewrites to
/// `llvm.experimental.memset.pattern` and then does not fold back into a
/// constant, even once everything is inlined. On Apple targets that lowers to a
/// `memset_pattern16` libcall to fill a stack slot that is read straight back.
///
/// Measured 2026-09-14, aarch64 (M5 Pro), shipped fat-LTO profile: nine such
/// calls per `sin_cos`, about 40% of `sample`'s busy samples on the
/// `moon_scan_365d` bench, and ~15.6 ns per four-lane call against ~7.5 ns
/// for this version in an isolated loop. A downstream consumer's scan workload
/// surfaced it first. The fill appears under no-LTO `codegen-units = 16` too,
/// so it is not a profile artefact.
///
/// A `const` is evaluated by the compiler, so the constants this port owns emit
/// no repeat loop. (`wide`'s own `f64x2::abs` still builds its mask at run time
/// on targets without NEON or simd128 — a speed cost there, not a bit change.)
/// End to end, same machine, `main` and this change benchmarked interleaved
/// (load ~2): `elp_mpp02_moon` 221 → 149 µs, `moon_scan_365d` 79.3 → 54.4 ms,
/// `apparent_position_full_chart` 4.98 → 2.94 ms. The shares in the module doc
/// above were measured before this and no longer describe the kernel.
///
/// # Why the output bits cannot change
///
/// Every floating-point operation below is the same `wide` method, called on
/// the same operands in the same order as upstream: `abs`, `round_ties_even`,
/// `round_int`, `mul_add`, `mul_neg_add`, `*`, `select`. The constants have the
/// same values. What is replaced is bit manipulation only — `is_finite` and
/// `flip_signs`, re-expressed over the same masks — and a bitwise operation
/// has exactly one correct result. [`tests::const_lane_port_is_bit_identical_to_wide`]
/// holds the port to upstream bit for bit, so a future `wide` that changes its
/// algorithm fails that test instead of silently forking the two — in this
/// repository's CI, against its lockfile. A downstream build that resolves a
/// newer `wide` (the requirement is a caret) runs no such test: its AVX targets
/// would follow upstream while the rest keep this port.
///
/// AVX targets keep `wide`'s own `sin_cos`, unchanged. Their `splat` is the
/// same repeat, but `memset_pattern16` exists only on Apple platforms, the
/// intrinsic lowers differently elsewhere, and no AVX build has been profiled
/// — so there is no measurement to justify touching the bits a production x86
/// consumer already has. The test below still compiles this module there.
#[cfg(any(not(target_feature = "avx"), test))]
mod const_lanes {
    use wide::bytemuck::cast;
    use wide::{f64x4, i64x4};

    const fn f4(c: f64) -> f64x4 {
        f64x4::new([c, c, c, c])
    }

    const fn i4(c: i64) -> i64x4 {
        i64x4::new([c, c, c, c])
    }

    // Polynomial and reduction constants, verbatim from `wide`.
    const P0SIN: f64x4 = f4(-1.666_666_666_666_663_072_95E-1);
    const P1SIN: f64x4 = f4(8.333_333_333_322_118_588_78E-3);
    const P2SIN: f64x4 = f4(-1.984_126_982_958_953_859_96E-4);
    const P3SIN: f64x4 = f4(2.755_731_362_138_572_452_13E-6);
    const P4SIN: f64x4 = f4(-2.505_074_776_285_780_728_66E-8);
    const P5SIN: f64x4 = f4(1.589_623_015_765_465_680_60E-10);

    const P0COS: f64x4 = f4(4.166_666_666_666_659_292_18E-2);
    const P1COS: f64x4 = f4(-1.388_888_888_887_305_641_16E-3);
    const P2COS: f64x4 = f4(2.480_158_728_885_170_453_48E-5);
    const P3COS: f64x4 = f4(-2.755_731_417_929_673_881_12E-7);
    const P4COS: f64x4 = f4(2.087_570_084_197_473_167_78E-9);
    const P5COS: f64x4 = f4(-1.135_853_652_138_768_173_00E-11);

    const DP1: f64x4 = f4(7.853_981_554_508_209_228_515_625E-1 * 2.);
    const DP2: f64x4 = f4(7.946_627_356_147_928_367_14E-9 * 2.);
    const DP3: f64x4 = f4(3.061_616_997_868_382_943_07E-17 * 2.);

    const TWO_OVER_PI: f64x4 = f4(2.0 / core::f64::consts::PI);

    // The values upstream builds with `splat` at run time.
    const HALF: f64x4 = f4(0.5);
    const ONE: f64x4 = f4(1.0);
    const ZERO: f64x4 = f4(0.0);
    const NAN: f64x4 = f4(f64::NAN);
    const SIGN_BIT: f64x4 = f4(-0.0);
    const I0: i64x4 = i4(0);
    const I1: i64x4 = i4(1);
    const I2: i64x4 = i4(2);
    const OVERFLOW_Q: i64x4 = i4(0x0080_0000_0000_0000);
    #[allow(clippy::cast_possible_wrap)] // a bit mask, not a number
    const SHIFTED_EXP_MASK: i64x4 = i4(0xFFE0_0000_0000_0000_u64 as i64);

    /// `wide`'s `is_finite`: all-ones where the exponent is not all ones.
    #[inline(always)]
    fn is_finite(v: f64x4) -> f64x4 {
        let u: i64x4 = cast(v);
        let shifted: i64x4 = u << 1_u32;
        let finite: i64x4 = !(shifted & SHIFTED_EXP_MASK).simd_eq(SHIFTED_EXP_MASK);
        cast(finite)
    }

    /// `wide`'s `polynomial_5!` macro, same nesting.
    #[inline(always)]
    fn polynomial_5(x: f64x4, c: [f64x4; 6]) -> f64x4 {
        let x2 = x * x;
        let x4 = x2 * x2;
        c[3].mul_add(x, c[2])
            .mul_add(x2, c[5].mul_add(x, c[4]).mul_add(x4, c[1].mul_add(x, c[0])))
    }

    #[inline(always)]
    pub(super) fn sin_cos(v: f64x4) -> (f64x4, f64x4) {
        let xa = v.abs();

        let y = (xa * TWO_OVER_PI).round_ties_even();
        let q = y.round_int();

        let x = y.mul_neg_add(DP3, y.mul_neg_add(DP2, y.mul_neg_add(DP1, xa)));

        let x2 = x * x;
        let mut s = polynomial_5(x2, [P0SIN, P1SIN, P2SIN, P3SIN, P4SIN, P5SIN]);
        let mut c = polynomial_5(x2, [P0COS, P1COS, P2COS, P3COS, P4COS, P5COS]);
        s = (x * x2).mul_add(s, x);
        c = (x2 * x2).mul_add(c, x2.mul_neg_add(HALF, ONE));

        let swap = !((q & I1).simd_eq(I0));

        let mut overflow: f64x4 = cast(q.simd_gt(OVERFLOW_Q));
        overflow &= is_finite(xa);
        s = overflow.select(ZERO, s);
        c = overflow.select(ONE, c);

        let mut sin1 = cast::<_, f64x4>(swap).select(c, s);
        let sign_sin: i64x4 = (q << 62_u32) ^ cast::<_, i64x4>(v);
        sin1 ^= cast::<_, f64x4>(sign_sin) & SIGN_BIT;

        let mut cos1 = cast::<_, f64x4>(swap).select(s, c);
        let sign_cos: i64x4 = ((q + I1) & I2) << 62_u32;
        cos1 ^= cast::<_, f64x4>(sign_cos);

        let finite = is_finite(v);
        (finite.select(sin1, NAN), finite.select(cos1, NAN))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vectorized kernel must track scalar `libm::sincos` across the full
    /// domain the lunar phases occupy, including large arguments and quadrant
    /// boundaries.
    ///
    /// Domain widened 2026-08-29 from `|x| ≤ 3000` to `|x| ≤ 3,400,000`: the
    /// original bound was an unverified assumption ("lunar phases stay well
    /// inside this"), and `docs/audit/2026-08-29-perf-investigation.md` §3b
    /// found `elp_mpp02.rs` actually feeds this kernel phases up to
    /// 3,303,561 rad at the edge of `AnalyticalProvider`'s supported JD
    /// range. `elp_mpp02::tests::sincos_matches_libm_at_real_elp_phase_domain`
    /// checks the exact real per-term phases (computed from the real
    /// argument polynomials, not guessed); this test's coarse sweep covers
    /// the same interval at lower density as a second, independent check
    /// that doesn't depend on the coefficient tables.
    #[test]
    fn matches_libm_across_domain() {
        let mut max_sin_err = 0.0_f64;
        let mut max_cos_err = 0.0_f64;

        let mut samples: Vec<f64> = Vec::new();
        let mut x = -3_400_000.0_f64;
        while x <= 3_400_000.0 {
            samples.push(x);
            x += 33.700_37; // fine, irrational-ish step to vary the reduction
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

    /// [`const_lanes::sin_cos`] must return exactly what `wide`'s own
    /// `f64x4::sin_cos` returns, bit for bit, on every lane. Compiled on every
    /// target, so AVX builds hold the port to upstream too.
    ///
    /// Covers the lunar phase domain densely, both sides of every quadrant
    /// boundary out to ±500π, both sides of the `q > 2^55` overflow select, the
    /// IEEE specials whose handling is pure bit manipulation (±0, subnormal,
    /// ±∞, quiet and signalling NaN), and 2M arbitrary bit patterns.
    #[test]
    fn const_lane_port_is_bit_identical_to_wide() {
        let mut samples: Vec<f64> = Vec::new();
        let mut x = -3_400_000.0_f64;
        while x <= 3_400_000.0 {
            samples.push(x);
            x += 1.370_31;
        }
        for k in -2000..=2000 {
            let b = f64::from(k) * core::f64::consts::FRAC_PI_4;
            samples.extend([b, b.next_up(), b.next_down()]);
        }
        samples.extend([
            0.0,
            -0.0,
            5e-324,
            -5e-324,
            1e17,
            9.1e18,
            -9.3e18,
            4e19,
            1e300,
            -1e300,
            f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
            f64::from_bits(0x7FF0_0000_0000_0001),
            f64::from_bits(0xFFF4_0000_0000_0000),
        ]);
        // `q = round(|x|·2/π)` crosses the `q > 2^55` overflow select here.
        let q_edge = (1_u64 << 55) as f64 * core::f64::consts::FRAC_PI_2;
        for k in -64_i32..=64 {
            let x = q_edge * (1.0 + f64::from(k) * f64::EPSILON * 4.0);
            samples.extend([x, -x]);
        }
        // Arbitrary bit patterns, deterministic (xorshift64).
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        for _ in 0..2_000_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            samples.push(f64::from_bits(state));
        }

        let bits = |v: f64x4| v.to_array().map(f64::to_bits);
        let mut differing = 0_usize;
        let mut first = None;
        for chunk in samples.chunks(4) {
            let mut lane = [0.0_f64; 4];
            lane[..chunk.len()].copy_from_slice(chunk);
            let v = f64x4::from(lane);
            let (ws, wc) = v.sin_cos();
            let (ps, pc) = const_lanes::sin_cos(v);
            if bits(ws) != bits(ps) || bits(wc) != bits(pc) {
                differing += 1;
                first.get_or_insert(lane);
            }
        }
        assert!(samples.len() > 6_900_000, "sample grid shrank");
        assert_eq!(
            differing, 0,
            "{differing} chunks differ from wide's sin_cos; first: {first:?}"
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
