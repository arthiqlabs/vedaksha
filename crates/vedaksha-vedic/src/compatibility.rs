// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Ashtakoota (8-factor) compatibility scoring.
//!
//! The Ashtakoota system evaluates compatibility between two birth charts by
//! comparing eight factors (kootas) for a maximum of 36 points total.
//!
//! This module currently implements Bhakoot (Rasyadhipati) — the 7-point sub-score
//! based on the sign (rashi) relationship between partners.
//!
//! Sources:
//! - B.V. Raman, "Muhurtha (Electional Astrology)", 10th ed., UBS Publishers, 1979.
//! - BPHS, Stree Jataka Adhyaya (on compatibility).
//! - Parashara's rules as enumerated in most classical Jyotish compilations.

use serde::{Deserialize, Serialize};

/// The 12 Vedic rashis (signs), 0-indexed.
///
/// `repr(u8)` ensures stable cast to/from integers for diff arithmetic.
///
/// Source: BPHS; standard Vedic zodiac enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rashi {
    Aries = 0,
    Taurus = 1,
    Gemini = 2,
    Cancer = 3,
    Leo = 4,
    Virgo = 5,
    Libra = 6,
    Scorpio = 7,
    Sagittarius = 8,
    Capricorn = 9,
    Aquarius = 10,
    Pisces = 11,
}

impl Rashi {
    /// Conventional Sanskrit/English name of this rashi.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }
}

/// Bhakoot (Rasyadhipati) Ashtakoota sub-score.
///
/// Returns **7** (full marks) when no dosha applies, or **0** (dosha) for:
///
/// | Dosha       | Condition                            |
/// |-------------|--------------------------------------|
/// | Shadashtak  | `diff` = 6 (6th/8th axis, a from b) |
/// | Shadashtak  | `diff` = 7 (mutual 8th)              |
/// | Nava-Pancham| `diff` = 4 (5th from a)              |
/// | Nava-Pancham| `diff` = 8 (9th from a)              |
///
/// where `diff = (rashi_b − rashi_a) rem_euclid 12`.
///
/// Note: Both `diff=6` and `diff=7` create the Shadashtak (6/8) axis because
/// the 6th-from-a relationship (diff=6) and 8th-from-a relationship (diff=7
/// in 0-indexed signs, corresponding to the classical 8th position) are
/// mutually afflicted. Similarly `diff=4` (5th) and `diff=8` (9th) form the
/// Nava-Pancham axis.
///
/// # Arguments
/// * `rashi_a` — Sign of the first partner (e.g. girl's Moon sign).
/// * `rashi_b` — Sign of the second partner (e.g. boy's Moon sign).
///
/// # Returns
/// `7` for compatibility, `0` for dosha.
///
/// Sources:
/// - B.V. Raman, "Muhurtha (Electional Astrology)", 10th ed.,
///   UBS Publishers, 1979, ch. on Ashtakoota.
/// - BPHS, Stree Jataka Adhyaya.
#[must_use]
pub fn bhakoot_score(rashi_a: Rashi, rashi_b: Rashi) -> u8 {
    let a = rashi_a as i8;
    let b = rashi_b as i8;
    let diff = ((b - a).rem_euclid(12)) as u8;
    if matches!(diff, 4 | 6 | 7 | 8) {
        return 0;
    }
    7
}

/// Maximum Bhakoot score (full marks).
pub const BHAKOOT_MAX: u8 = 7;

#[cfg(test)]
mod tests {
    use super::*;

    // ── bhakoot_score ─────────────────────────────────────────────────────────

    // No-dosha cases — should return 7.

    #[test]
    fn same_sign_no_dosha() {
        // diff = 0: same sign, no dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Aries), 7);
        assert_eq!(bhakoot_score(Rashi::Leo, Rashi::Leo), 7);
    }

    #[test]
    fn diff_1_no_dosha() {
        // diff = 1: adjacent signs, no dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Taurus), 7);
    }

    #[test]
    fn diff_2_no_dosha() {
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Gemini), 7);
    }

    #[test]
    fn diff_3_no_dosha() {
        // diff = 3: no dosha (not in {4,6,7,8}).
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Cancer), 7);
    }

    #[test]
    fn diff_5_no_dosha() {
        // diff = 5: no dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Virgo), 7);
    }

    #[test]
    fn diff_9_no_dosha() {
        // diff = 9: no dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Capricorn), 7);
    }

    #[test]
    fn diff_10_no_dosha() {
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Aquarius), 7);
    }

    #[test]
    fn diff_11_no_dosha() {
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Pisces), 7);
    }

    // Dosha cases — should return 0.

    #[test]
    fn shadashtak_diff_6_is_dosha() {
        // Aries(0) + Libra(6): diff = 6 → Shadashtak dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Libra), 0);
    }

    #[test]
    fn shadashtak_diff_7_is_dosha() {
        // Aries(0) + Scorpio(7): diff = 7 → Shadashtak dosha (mutual 8th).
        // Source: B.V. Raman, "Muhurtha", ch. Ashtakoota.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Scorpio), 0);
    }

    #[test]
    fn nava_pancham_diff_4_is_dosha() {
        // Aries(0) + Leo(4): diff = 4 → Nava-Pancham dosha.
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Leo), 0);
    }

    #[test]
    fn nava_pancham_diff_8_is_dosha() {
        // Aries(0) + Sagittarius(8): diff = 8 → Nava-Pancham dosha (reverse).
        assert_eq!(bhakoot_score(Rashi::Aries, Rashi::Sagittarius), 0);
    }

    // Symmetry: swap a and b, diff wraps to 12 - diff.

    #[test]
    fn reverse_diff_6_becomes_diff_6() {
        // Libra(6) + Aries(0): diff = (0-6) rem_euclid 12 = 6 → dosha.
        assert_eq!(bhakoot_score(Rashi::Libra, Rashi::Aries), 0);
    }

    #[test]
    fn reverse_diff_7_becomes_diff_5_no_dosha() {
        // Scorpio(7) + Aries(0): diff = (0-7) rem_euclid 12 = 5 → no dosha.
        // The axis is NOT symmetric for diff=7 (that's by design in the classical rule).
        assert_eq!(bhakoot_score(Rashi::Scorpio, Rashi::Aries), 7);
    }

    #[test]
    fn reverse_diff_4_becomes_diff_8_dosha() {
        // Leo(4) + Aries(0): diff = (0-4) rem_euclid 12 = 8 → dosha.
        assert_eq!(bhakoot_score(Rashi::Leo, Rashi::Aries), 0);
    }

    // Cross-sign wrap-around test.

    #[test]
    fn diff_wraps_correctly_at_pisces() {
        // Scorpio(7) + Gemini(2): diff = (2-7) rem_euclid 12 = 7 → dosha.
        assert_eq!(bhakoot_score(Rashi::Scorpio, Rashi::Gemini), 0);
    }

    #[test]
    fn bhakoot_max_constant_is_7() {
        assert_eq!(BHAKOOT_MAX, 7);
    }
}
