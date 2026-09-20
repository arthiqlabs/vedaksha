// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Arudha padas (Jaimini).
//!
//! # Sources
//!
//! - **Rule** — Jaimini Sutra I.1 (arudha): count from the bhava to its
//!   lord's rashi, then that many again from the lord's rashi. The same rule
//!   gives graha arudhas (count from the graha's rashi to its dispositor's
//!   rashi).
//! - **Exception** — BPHS, the chapter on padas, Ch.29 vv.4-5: the bhava
//!   itself or its 7th may not be its pada; in the same-house case the 10th
//!   therefrom is the pada, in the 7th-house case the 4th from the original
//!   bhava. Counting ten (inclusively) from the landed 7th house reaches the
//!   same 4th-from-bhava house, so the BPHS wording is arithmetically one
//!   rule: ten from the landed pada.
//!
//! # The variant
//!
//! Some Jaimini commentaries are read as counting the ten from the starting
//! bhava in both exception cases instead. The two readings coincide whenever
//! the preliminary pada lands on the bhava itself (landed == bhava, so both
//! bases are the same house) and differ ONLY when it lands on the 7th:
//! ten-from-landed reaches the 4th from the bhava, ten-from-bhava the 10th.
//! [`ArudhaException`] exposes that choice rather than hard-coding one.

/// Which house the exception count of ten starts from.
///
/// See the module doc: the readings agree on the same-house landing and
/// differ only on the 7th-house landing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArudhaException {
    /// Count ten (inclusively) from the preliminary, landed pada. This is
    /// the BPHS Ch.29 vv.4-5 rule: "10th therefrom" in the same-house case,
    /// "4th from the original bhava" in the 7th-house case — the same house
    /// either way it is stated.
    TenthFromLanded,
    /// Count ten (inclusively) from the starting bhava in both exception
    /// cases (the Jaimini-commentary reading).
    TenthFromBhava,
}

/// Arudha pada of a bhava — the rashi index (0 = Mesha, .., 11 = Mina).
///
/// Counts inclusively from `bhava_rashi` to `adhipati_rashi` (the bhava
/// lord's rashi), then that many again from the lord's rashi. If the
/// preliminary pada lands on the bhava itself or its 7th, counts ten more
/// from the base [`ArudhaException`] selects.
///
/// The same function gives graha arudhas: pass the graha's rashi as
/// `bhava_rashi` and its dispositor's rashi as `adhipati_rashi`. Which
/// planet lords/disposits which rashi is the caller's doctrine (co-lordships
/// of Vrshchika and Kumbha differ between schools), so this function takes
/// rashi indices, not grahas.
///
/// Inputs are normalized with `% 12`; all counting is 1-based inclusive
/// (a lord in its own bhava counts 1, not 0).
#[must_use]
pub fn arudha_pada(bhava_rashi: u8, adhipati_rashi: u8, exception: ArudhaException) -> u8 {
    let bhava = bhava_rashi % 12;
    let lord = adhipati_rashi % 12;
    // Inclusive distance bhava -> lord, in 1..=12.
    let steps = (lord + 12 - bhava) % 12 + 1;
    let landed = (lord + steps - 1) % 12;
    let seventh = (bhava + 6) % 12;
    if landed == bhava || landed == seventh {
        let base = match exception {
            ArudhaException::TenthFromLanded => landed,
            ArudhaException::TenthFromBhava => bhava,
        };
        // Tenth house inclusively = nine steps forward.
        (base + 9) % 12
    } else {
        landed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_twice_from_lord() {
        // Mesha bhava (0), lord Mars in Vrshchika (7): 8 count, 8 again
        // from 7 -> (7 + 8 - 1) % 12 = 2 (Mithuna).
        assert_eq!(arudha_pada(0, 7, ArudhaException::TenthFromLanded), 2);
        assert_eq!(arudha_pada(0, 7, ArudhaException::TenthFromBhava), 2);
    }

    #[test]
    fn lord_in_own_bhava_counts_one_then_excepts() {
        // Steps = 1, landed = bhava -> exception: 10th from there = 9.
        // Both variants agree here (landed == bhava).
        assert_eq!(arudha_pada(0, 0, ArudhaException::TenthFromLanded), 9);
        assert_eq!(arudha_pada(0, 0, ArudhaException::TenthFromBhava), 9);
    }

    #[test]
    fn seventh_landing_distinguishes_the_variants() {
        // Bhava Mesha (0), lord in Karka (3): steps 4, landed (3+3)%12 = 6,
        // the 7th from the bhava. Tenth-from-landed -> (6+9)%12 = 3 (the
        // 4th from the bhava, per BPHS); tenth-from-bhava -> 9.
        assert_eq!(arudha_pada(0, 3, ArudhaException::TenthFromLanded), 3);
        assert_eq!(arudha_pada(0, 3, ArudhaException::TenthFromBhava), 9);
    }

    #[test]
    fn graha_arudha_uses_the_same_rule() {
        // Shukra in Tula (6), dispositor Shukra in Vrshabha (1): steps 8,
        // landed (1+7)%12 = 8 (Vrshchika) — no exception.
        assert_eq!(arudha_pada(6, 1, ArudhaException::TenthFromLanded), 8);
    }

    #[test]
    fn inputs_normalize_mod_12() {
        assert_eq!(
            arudha_pada(12, 7, ArudhaException::TenthFromLanded),
            arudha_pada(0, 7, ArudhaException::TenthFromLanded)
        );
    }
}
