// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Narayana Dasha — Jaimini sign-based directional period system.
//!
//! Narayana Dasha assigns periods to signs based on the lagna. The direction
//! of sign progression depends on whether the lagna is an odd or even sign:
//! - Odd-sign lagnas (Aries, Gemini, Leo, Libra, Sagittarius, Aquarius):
//!   signs progress forward (1st, 2nd, 3rd, ...).
//! - Even-sign lagnas (Taurus, Cancer, Virgo, Scorpio, Capricorn, Pisces):
//!   signs progress backward (12th, 11th, 10th, ...).
//!
//! The duration for each sign equals the number of signs from that sign to
//! its lord's sign (1-12 years). Standard Parashari lordships are used.
//!
//! Source: Jaimini Sutras Ch. 2.

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

/// A single Narayana Dasha sign period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarayanaPeriod {
    /// Zero-based sign index (0 = Aries, 11 = Pisces).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: String,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in years.
    pub duration_years: f64,
}

/// Complete Narayana Dasha sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarayanaDasha {
    /// Zero-based sign index of the lagna.
    pub lagna_sign: u8,
    /// All 12 sign periods in dasha order.
    pub periods: Vec<NarayanaPeriod>,
}

/// Return whether a sign index is odd-sign (Aries=0 is odd-sign, Taurus=1 is even-sign, etc.).
///
/// Odd signs (1st, 3rd, 5th, 7th, 9th, 11th of the zodiac):
/// Aries(0), Gemini(2), Leo(4), Libra(6), Sagittarius(8), Aquarius(10).
fn is_odd_sign(sign: u8) -> bool {
    sign % 2 == 0
}

/// Return the sign occupied by the traditional (Parashari) ruler of `sign`.
///
/// Standard Parashari lordships used for simplicity per Jaimini Sutras Ch. 2.
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
        // 7 (Scorpio → Mars → Aries) and fallback
        _ => 0,
    }
}

/// Forward sign distance from `from` to `to` (1-12).
///
/// A sign-to-itself returns 12, not 0, per Jaimini convention.
fn sign_distance(from: u8, to: u8) -> u8 {
    #[allow(clippy::cast_possible_truncation)]
    let dist = ((i16::from(to) - i16::from(from)).rem_euclid(12)) as u8;
    if dist == 0 { 12 } else { dist }
}

/// Compute the Narayana Dasha sequence for all 12 signs.
///
/// # Arguments
/// * `lagna_sign` — Zero-based sign index of the Lagna (Ascendant), 0-11.
/// * `birth_jd`   — Julian Day of birth.
///
/// # Returns
///
/// A [`NarayanaDasha`] with 12 [`NarayanaPeriod`] entries. For odd-sign lagnas,
/// signs progress forward from lagna; for even-sign lagnas, signs progress
/// backward from lagna.
///
/// Source: Jaimini Sutras Ch. 2.
#[must_use]
pub fn compute_narayana(lagna_sign: u8, birth_jd: f64) -> NarayanaDasha {
    let lagna_sign = lagna_sign % 12;
    let odd = is_odd_sign(lagna_sign);

    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;

    for i in 0u8..12 {
        let sign = if odd {
            (lagna_sign + i) % 12
        } else {
            // Backward: lagna, lagna-1, lagna-2, ...
            (lagna_sign + 12 - i) % 12
        };

        let lord_sign = sign_lord_sign(sign);
        let duration_years = f64::from(sign_distance(sign, lord_sign));
        let duration_days = duration_years * DASHA_YEAR_DAYS;

        periods.push(NarayanaPeriod {
            sign_index: sign,
            sign_name: SIGN_NAMES[sign as usize].to_string(),
            start_jd: current_jd,
            end_jd: current_jd + duration_days,
            duration_years,
        });

        current_jd += duration_days;
    }

    NarayanaDasha {
        lagna_sign,
        periods,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JD: f64 = 2_451_545.0; // J2000.0

    // 1. Narayana dasha produces exactly 12 periods
    #[test]
    fn narayana_has_12_periods() {
        let result = compute_narayana(0, TEST_JD);
        assert_eq!(
            result.periods.len(),
            12,
            "expected 12 Narayana periods, got {}",
            result.periods.len()
        );
    }

    // 2. Odd sign (Aries=0) goes forward: first period is Aries
    #[test]
    fn narayana_odd_sign_goes_forward() {
        let result = compute_narayana(0, TEST_JD);
        assert_eq!(
            result.periods[0].sign_index, 0,
            "first period should be Aries"
        );
        assert_eq!(result.periods[0].sign_name, "Aries");
        assert_eq!(
            result.periods[1].sign_index, 1,
            "second period should be Taurus"
        );
        assert_eq!(
            result.periods[2].sign_index, 2,
            "third period should be Gemini"
        );
    }

    // 3. Even sign (Taurus=1) goes backward: first=Taurus, second=Aries
    #[test]
    fn narayana_even_sign_goes_backward() {
        let result = compute_narayana(1, TEST_JD);
        assert_eq!(
            result.periods[0].sign_index, 1,
            "first period should be Taurus"
        );
        assert_eq!(result.periods[0].sign_name, "Taurus");
        assert_eq!(
            result.periods[1].sign_index, 0,
            "second period should be Aries"
        );
        assert_eq!(
            result.periods[2].sign_index, 11,
            "third period should be Pisces"
        );
    }

    // 4. Periods are contiguous
    #[test]
    fn periods_are_contiguous() {
        let result = compute_narayana(0, TEST_JD);
        for i in 1..result.periods.len() {
            let gap = (result.periods[i].start_jd - result.periods[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between periods {}-{}: {gap}", i - 1, i);
        }
    }

    // 5. All 12 signs appear exactly once
    #[test]
    fn all_12_signs_appear() {
        let result = compute_narayana(0, TEST_JD);
        let mut seen = [false; 12];
        for p in &result.periods {
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

    // 6. Sign distances are in reasonable range (1-12)
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
}
