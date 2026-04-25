// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
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
//! **Starting sign:** For odd lagnas, the sequence runs forward (Aries→…);
//! for even lagnas, the sequence also runs forward in the simplified version
//! used here (full Jaimini treatment involves Chara/Sthira/Dwiswabhava
//! distinctions not yet implemented).
//!
//! Source: Jaimini Sutras 1.2; B.V. Raman, "Studies in Jaimini Astrology".

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
/// `lagna_sign` and proceeding sign by sign through all 12 signs.
///
/// Source: Jaimini Sutras 1.2.
#[must_use]
pub fn compute_chara(lagna_sign: u8, birth_jd: f64) -> Vec<CharaPeriod> {
    let lagna_sign = lagna_sign % 12;
    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;

    for i in 0u8..12 {
        let sign = (lagna_sign + i) % 12;
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
}
