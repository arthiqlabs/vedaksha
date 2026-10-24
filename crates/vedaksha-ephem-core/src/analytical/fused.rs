// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! Fused multiply-add, used only where the target has it in hardware.
//!
//! # What it changes, and what it does not
//!
//! `a.mul_add(b, c)` rounds once where `a * b + c` rounds twice, so a series
//! summed with it differs from the same series summed without it in the last
//! bit of each partial sum. Rust never contracts a separate multiply and add
//! into a fused one on its own (there is no fast-math), so the choice has to be
//! explicit, and this module is where it is made.
//!
//! **Output therefore differs by architecture**, by a few units in the last
//! place of each sum: a build that fuses and a build that does not return
//! slightly different bits for the same input. The bit-digest tests were
//! already per-architecture for the same kind of reason (`wide`'s vector
//! kernel is a different instruction sequence on NEON, AVX and scalar); this
//! adds one more axis to that. The difference is a few units in the last place
//! of each partial sum, far below every accuracy claim the crate makes.
//!
//! # Where it is fused
//!
//! | target | fused | why |
//! |---|---|---|
//! | `aarch64` | yes | FMA is part of base ARMv8 floating point; `fmadd` is one instruction |
//! | `x86_64` with `fma` enabled (`-C target-cpu=x86-64-v3`, the Docker and release builds) | yes | one `vfmadd` instruction |
//! | `x86_64` baseline | no | the baseline target has no FMA; `f64::mul_add` would call a software routine, many times slower than the two instructions it replaces |
//! | `wasm32` | no | no FMA in the instruction set (relaxed-SIMD fused ops are not enabled) |
//! | anything else | no | not measured |
//!
//! [`HARDWARE_FMA`] is a compile-time constant, so on a target where it is
//! false the `if` below disappears and [`mul_add`] is exactly `a * b + c`,
//! the arithmetic this crate has always done.

/// Whether this build fuses multiply-adds.
///
/// True exactly where `f64::mul_add` lowers to a single hardware instruction.
pub(crate) const HARDWARE_FMA: bool = cfg!(any(target_arch = "aarch64", target_feature = "fma"));

/// `a * b + c`, rounded once on targets that fuse ([`HARDWARE_FMA`]) and
/// rounded twice everywhere else.
#[inline(always)]
pub(crate) fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    if HARDWARE_FMA {
        a.mul_add(b, c)
    } else {
        a * b + c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fused and unfused forms differ in the last bit, so the cfg
    /// actually selects different arithmetic. Chosen so that `a * b` is not
    /// exactly representable: `(1 + 2^-27)^2 = 1 + 2^-26 + 2^-54`, whose last
    /// term the unfused product rounds away.
    #[test]
    fn fused_form_keeps_the_product_bits_the_unfused_form_rounds_away() {
        let a = 1.0 + 2.0_f64.powi(-27);
        let c = -(1.0 + 2.0_f64.powi(-26));
        let unfused = a * a + c;
        assert_eq!(
            unfused, 0.0,
            "the unfused product should round to 1 + 2^-26"
        );
        let got = mul_add(a, a, c);
        if HARDWARE_FMA {
            assert_eq!(got, 2.0_f64.powi(-54), "a fused build keeps the 2^-54 term");
        } else {
            assert_eq!(got, 0.0, "an unfused build must equal a * b + c");
        }
    }
}
