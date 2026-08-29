// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Chara (Jaimini) Dasha — sign-based variable-period system.
//!
//! Chara Dasha is one of the most important conditional dashas from Jaimini
//! astrology. Unlike planetary dashas, Chara assigns periods to signs (rashis).
//! Each sign gets a period of years equal to the sign-distance from itself to
//! the sign occupied by its lord.
//!
//! **Sign distance:** Forward count from sign to lord's sign (1–12); a sign
//! whose lord is in the same sign gets 12 years.
//!
//! **Starting sign and direction:** per Jaimini Sutras, Adhyaya 1 Pada 1,
//! sutras 1.1.25-27:
//! - Sutra 1.1.25 (*Pracheevruttirvishamabheshu*): odd signs (1-indexed;
//!   0-indexed even values here — Aries, Gemini, Leo, Libra, Sagittarius,
//!   Aquarius) count forward.
//! - Sutra 1.1.26 (*Paravrutyottareshu*): even signs (0-indexed odd values —
//!   Taurus, Cancer, Virgo, Scorpio, Capricorn, Pisces) count backward.
//! - Sutra 1.1.27 (*Nakwachit*, "this does not apply in some places"): a
//!   terse, two-word exception with no explicit list in the sutra text
//!   itself. The commentarial reconstruction used here (corroborated by two
//!   independently-styled sources) is that the exception covers exactly the
//!   four fixed (Sthira) signs — Taurus, Scorpio, Leo, Aquarius — where the
//!   plain odd/even direction is inverted: Taurus and Scorpio (even) count
//!   forward; Leo and Aquarius (odd) count backward.
//!
//! **Confidence:** the base odd/even rule (sutras 25-26) is high confidence —
//! direct sutra text, corroborated by essentially every secondary source
//! found, and internally consistent with the identical odd/even logic
//! applied elsewhere in the same Pada (sutras 1.1.32, 1.1.34). The fixed-sign
//! exception mapping (sutra 27) is moderate confidence — the sutra itself
//! gives no explicit list, and the four-sign mapping is a commentarial
//! reconstruction, though two independently-styled sources ("Sthira" framing
//! vs. "Vishama Pada"/"Sama Pada" framing) converge on the identical four
//! signs.
//!
//! **A different, popular secondary-source mnemonic exists** ("movable signs
//! forward, fixed signs backward, dual signs forward-then-backward") and is
//! *not* implemented here: it is internally inconsistent with sutras 25-26
//! for movable-even signs (e.g. Cancer — movable but even — which the sutra
//! text places in the backward group with no stated exception for movable
//! signs). This module implements the sutra-literal + reconstructed-exception
//! reading instead.
//!
//! No confirmed classical sub-rule exists for dual (Dwiswabhava) signs beyond
//! the plain odd/even rule; dual signs are not an exception category in this
//! reading — only the four fixed signs are.
//!
//! Source: Jaimini Sutras 1.1 (sutras 25-28 cover direction and duration).

use serde::{Deserialize, Serialize};

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// English names of the 12 signs, zero-indexed from Aries.
const SIGN_NAMES: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// A single Chara Dasha sign period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharaPeriod {
    /// Zero-based sign index (0 = Aries, 11 = Pisces).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: &'static str,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in years.
    pub duration_years: f64,
}

/// Compute the Chara Dasha sequence for all 12 signs.
///
/// # Arguments
/// * `lagna_sign` — Zero-based sign index of the Lagna (Ascendant), 0–11.
/// * `birth_jd`   — Julian Day of birth.
///
/// # Returns
///
/// A `Vec` of 12 [`CharaPeriod`] entries in the dasha order, starting from
/// `lagna_sign` and proceeding sign by sign through all 12 signs, in the
/// direction given by [`chara_direction`] (forward or backward per Jaimini
/// Sutras 1.1.25-27; see the module doc for the confidence split).
///
/// Source: Jaimini Sutras 1.1 (sutras 25-28).
#[must_use]
pub fn compute_chara(lagna_sign: u8, birth_jd: f64) -> Vec<CharaPeriod> {
    let lagna_sign = lagna_sign % 12;
    let direction = chara_direction(lagna_sign);
    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;

    for i in 0u8..12 {
        let sign = match direction {
            Direction::Forward => (lagna_sign + i) % 12,
            Direction::Backward => (lagna_sign + 12 - i) % 12,
        };
        let lord_sign = sign_lord_sign(sign);
        let duration_years = f64::from(sign_distance(sign, lord_sign));
        let duration_days = duration_years * DASHA_YEAR_DAYS;

        periods.push(CharaPeriod {
            sign_index: sign,
            sign_name: SIGN_NAMES[sign as usize],
            start_jd: current_jd,
            end_jd: current_jd + duration_days,
            duration_years,
        });

        current_jd += duration_days;
    }

    periods
}

/// Chara Dasha's counting direction for a given lagna sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

/// Whether `sign` (0-indexed, 0 = Aries) is an odd sign in the classical
/// 1-indexed numbering (1 = Aries = odd, 2 = Taurus = even, ...) — i.e. an
/// even 0-indexed value.
fn is_odd_sign(sign: u8) -> bool {
    sign.is_multiple_of(2)
}

/// Chara Dasha's counting direction, per Jaimini Sutras 1.1.25-27.
///
/// The base rule (sutras 25-26, high confidence): odd signs count forward,
/// even signs count backward. Sutra 27 states a terse, unlisted exception
/// ("this does not apply in some places"); the commentarial reconstruction
/// (moderate confidence, corroborated by two independently-styled sources)
/// is that the four fixed (Sthira) signs — Taurus, Scorpio, Leo, Aquarius —
/// invert the plain odd/even direction. A different, popular secondary-source
/// mnemonic ("movable forward, fixed backward, dual forward-then-backward")
/// is not implemented here: it is internally inconsistent with sutras 25-26
/// for movable-even signs (e.g. Cancer), which the sutra text places in the
/// backward group with no stated exception for movable signs.
fn chara_direction(sign: u8) -> Direction {
    match sign {
        1 | 7 => Direction::Forward,   // Taurus, Scorpio: fixed exception
        4 | 10 => Direction::Backward, // Leo, Aquarius: fixed exception
        _ if is_odd_sign(sign) => Direction::Forward,
        _ => Direction::Backward,
    }
}

/// Return the sign occupied by the traditional (Parashari) ruler of `sign`.
///
/// Each planet's primary domicile sign is used. Mars and Saturn each rule two
/// signs; we assign Mars's primary sign to Aries (day house), Scorpio (night
/// house) and Jupiter to Sagittarius/Pisces accordingly.
///
/// Index: 0 = Aries … 11 = Pisces.
fn sign_lord_sign(sign: u8) -> u8 {
    match sign {
        0 => 7,  // Aries → Mars → Scorpio
        1 => 6,  // Taurus → Venus → Libra
        2 => 5,  // Gemini → Mercury → Virgo
        3 => 3,  // Cancer → Moon → Cancer (self)
        4 => 4,  // Leo → Sun → Leo (self)
        5 => 2,  // Virgo → Mercury → Gemini
        6 => 1,  // Libra → Venus → Taurus
        8 => 11, // Sagittarius → Jupiter → Pisces
        9 => 10, // Capricorn → Saturn → Aquarius
        10 => 9, // Aquarius → Saturn → Capricorn
        11 => 8, // Pisces → Jupiter → Sagittarius
        // 7 (Scorpio → Mars → Aries) and fallback both return 0
        _ => 0,
    }
}

/// Forward sign distance from `from` to `to` (1–12).
///
/// A sign-to-itself returns 12, not 0, per Jaimini convention.
fn sign_distance(from: u8, to: u8) -> u8 {
    #[allow(clippy::cast_possible_truncation)]
    let dist = ((i16::from(to) - i16::from(from)).rem_euclid(12)) as u8;
    if dist == 0 { 12 } else { dist }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JD: f64 = 2_451_545.0; // J2000.0

    // 1. Chara dasha produces exactly 12 sign periods
    #[test]
    fn chara_dasha_produces_12_periods() {
        let periods = compute_chara(0, TEST_JD);
        assert_eq!(
            periods.len(),
            12,
            "expected 12 Chara periods, got {}",
            periods.len()
        );
    }

    // 2. Periods are contiguous (end of one = start of next)
    #[test]
    fn periods_are_contiguous() {
        let periods = compute_chara(0, TEST_JD);
        for i in 1..periods.len() {
            let gap = (periods[i].start_jd - periods[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between period {}-{}: {gap}", i - 1, i);
        }
    }

    // 3. All 12 signs appear exactly once
    #[test]
    fn all_12_signs_appear() {
        let periods = compute_chara(0, TEST_JD);
        let mut seen = [false; 12];
        for p in &periods {
            assert!(
                (p.sign_index as usize) < 12,
                "sign_index out of range: {}",
                p.sign_index
            );
            assert!(
                !seen[p.sign_index as usize],
                "sign {} appears more than once",
                p.sign_index
            );
            seen[p.sign_index as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "not all 12 signs appeared");
    }

    // 4. Sign distances are in reasonable range (1–12)
    #[test]
    fn sign_distances_are_reasonable() {
        for sign in 0u8..12 {
            let lord = sign_lord_sign(sign);
            let dist = sign_distance(sign, lord);
            assert!(
                (1..=12).contains(&dist),
                "sign {sign}: distance to lord sign {lord} = {dist}, expected 1..=12"
            );
        }
    }

    // 5. Odd, non-fixed lagna (Gemini=2) counts forward
    #[test]
    fn odd_non_fixed_lagna_counts_forward() {
        let periods = compute_chara(2, TEST_JD); // Gemini
        assert_eq!(
            periods[0].sign_index, 2,
            "first period should be Gemini itself"
        );
        assert_eq!(
            periods[1].sign_index, 3,
            "second period should be Cancer (forward)"
        );
    }

    // 6. Even, non-fixed lagna (Cancer=3) counts backward
    #[test]
    fn even_non_fixed_lagna_counts_backward() {
        let periods = compute_chara(3, TEST_JD); // Cancer
        assert_eq!(
            periods[0].sign_index, 3,
            "first period should be Cancer itself"
        );
        assert_eq!(
            periods[1].sign_index, 2,
            "second period should be Gemini (backward)"
        );
    }

    // 7. Fixed exception: Taurus (even) counts forward, not backward
    #[test]
    fn fixed_sign_exception_taurus_counts_forward() {
        let periods = compute_chara(1, TEST_JD); // Taurus
        assert_eq!(
            periods[0].sign_index, 1,
            "first period should be Taurus itself"
        );
        assert_eq!(
            periods[1].sign_index, 2,
            "second period should be Gemini (forward, exception overrides even-backward)"
        );
    }

    // 8. Fixed exception: Scorpio (even) counts forward, not backward
    #[test]
    fn fixed_sign_exception_scorpio_counts_forward() {
        let periods = compute_chara(7, TEST_JD); // Scorpio
        assert_eq!(
            periods[0].sign_index, 7,
            "first period should be Scorpio itself"
        );
        assert_eq!(
            periods[1].sign_index, 8,
            "second period should be Sagittarius (forward, exception overrides even-backward)"
        );
    }

    // 9. Fixed exception: Leo (odd) counts backward, not forward
    #[test]
    fn fixed_sign_exception_leo_counts_backward() {
        let periods = compute_chara(4, TEST_JD); // Leo
        assert_eq!(
            periods[0].sign_index, 4,
            "first period should be Leo itself"
        );
        assert_eq!(
            periods[1].sign_index, 3,
            "second period should be Cancer (backward, exception overrides odd-forward)"
        );
    }

    // 10. Fixed exception: Aquarius (odd) counts backward, not forward
    #[test]
    fn fixed_sign_exception_aquarius_counts_backward() {
        let periods = compute_chara(10, TEST_JD); // Aquarius
        assert_eq!(
            periods[0].sign_index, 10,
            "first period should be Aquarius itself"
        );
        assert_eq!(
            periods[1].sign_index, 9,
            "second period should be Capricorn (backward, exception overrides odd-forward)"
        );
    }

    // 11. Aries (odd, non-fixed) still counts forward — regression guard for the un-exceptional case
    #[test]
    fn aries_lagna_still_counts_forward() {
        let periods = compute_chara(0, TEST_JD); // Aries
        assert_eq!(periods[0].sign_index, 0);
        assert_eq!(
            periods[1].sign_index, 1,
            "second period should be Taurus (forward)"
        );
    }

    // 12. All 12 signs still appear exactly once regardless of direction (backward case)
    #[test]
    fn all_12_signs_appear_going_backward() {
        let periods = compute_chara(3, TEST_JD); // Cancer, backward
        let mut seen = [false; 12];
        for p in &periods {
            assert!(
                !seen[p.sign_index as usize],
                "sign {} appears more than once",
                p.sign_index
            );
            seen[p.sign_index as usize] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "not all 12 signs appeared going backward"
        );
    }
}
