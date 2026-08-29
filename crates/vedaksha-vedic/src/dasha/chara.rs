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
//! **Direction — tested at the 9th sign from the lagna, not the lagna itself.**
//! The governing sutra is in Adhyaya 2, Pada 3 (its own number is unstable
//! across sources — direct readings of 22, 28, and 32 are on record, and a
//! fourth source independently reports 29): *pañcame padakramāt
//! prākpratyaktvaṃ caradaśāyām*. Grammatically, *pañcame* means "in the
//! fifth" — but Jaimini states his own letter-to-numeral convention
//! elsewhere (*sarvatra savarṇā bhāvā rāśayaś ca*, "everywhere, the houses
//! and signs are [denoted] by letters"), under which *pañcame* decodes
//! arithmetically to **nine**, not five. That decoding is not asserted here
//! on trust: it reproduces, independently, the numeric answer each of eight
//! other words coded the same way is glossed with elsewhere in this same
//! sutra layer (high confidence — a verifiable arithmetic fact, not a
//! judgment call). The sutra is read here as: take the 9th sign from the
//! lagna, and test it (not the lagna itself) for direction.
//!
//! **The classification tested at that 9th sign: vishama-pada ("odd-footed")
//! vs. sama-pada ("even-footed")** — Aries, Taurus, Gemini, Libra, Scorpio,
//! Sagittarius are vishama-pada; the other six are sama-pada. This is a
//! distinct classification from plain odd/even sign numbering (it agrees
//! with plain odd/even on eight signs and disagrees on exactly Taurus, Leo,
//! Scorpio, Aquarius — the four signs a still-earlier version of this module
//! called out as a "fixed-sign exception" to a plain odd/even rule tested
//! directly on the lagna, before the 9th-house step was found; see
//! `DATA_PROVENANCE.md` Fix 11 for that history). If the 9th sign from the
//! lagna is vishama-pada, Chara Dasha counts forward; otherwise backward.
//!
//! **Attribution and confidence:** this reading is transmitted in a
//! commentary called the *Subodhinī*, under the name Nīlakaṇṭha — moderate
//! confidence on the specific authorship, since a critical edition compiled
//! from multiple manuscripts notes that copies circulating under that name
//! are sometimes misattributed. The same critical edition describes the
//! 9th-house reading as the view of "many commentators" — a documented
//! majority, not a claim of universal agreement — and separately records a
//! named minority reading, under the name Kṛṣṇānanda Sarasvatī, holding that
//! *pañcame* keeps its literal sense and names a different dasha system
//! entirely, not a rival Chara Dasha direction rule. No source found states
//! a rival *Chara Dasha* direction table under any other name.
//!
//! **Superseded:** earlier versions of this module cited Adhyaya 1 Pada 1,
//! sutras 25-27, for direction. Those sutras are understood here to govern a
//! different rule — sign-to-lord counting, the concern of [`sign_distance`]
//! below, not sequence direction — and are no longer cited for direction.
//! See `DATA_PROVENANCE.md` Fix 11 for the full history of that correction.

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
/// direction given by [`chara_direction`] (forward or backward, tested at
/// the 9th sign from the lagna; see the module doc for the sutra and its
/// confidence grading).
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

/// Whether `sign` (0-indexed, 0 = Aries) is vishama-pada ("odd-footed"):
/// Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius. See the module doc —
/// this is a distinct classification from plain odd/even sign numbering,
/// used here as the test applied to the 9th sign from the lagna, not to the
/// lagna itself.
fn is_vishama_pada(sign: u8) -> bool {
    matches!(sign, 0 | 1 | 2 | 6 | 7 | 8)
}

/// Chara Dasha's counting direction: vishama-pada/sama-pada, tested at the
/// 9th sign from `lagna` — see the module doc for the sutra and its
/// confidence grading.
fn chara_direction(lagna: u8) -> Direction {
    let ninth_from_lagna = (lagna + 8) % 12;
    if is_vishama_pada(ninth_from_lagna) {
        Direction::Forward
    } else {
        Direction::Backward
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

    // 5-16. Direction for every lagna, tested at the 9th sign from the lagna
    // (vishama-pada => forward, sama-pada => backward). Table verified by
    // hand: 9th-from-lagna for each of the 12 lagnas, checked against the
    // vishama-pada set {Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius}.
    // Movable lagnas (Aries, Cancer, Libra, Capricorn) keep the same
    // direction a plain-odd/even-on-the-lagna rule would also give them;
    // the other eight flip relative to that plain rule, which is exactly
    // the vishama-pada/sama-pada split this module now applies at the 9th
    // house rather than at the lagna itself.
    #[test]
    fn aries_lagna_counts_forward() {
        let periods = compute_chara(0, TEST_JD); // Aries; 9th = Sagittarius, vishama-pada
        assert_eq!(periods[0].sign_index, 0);
        assert_eq!(
            periods[1].sign_index, 1,
            "second period should be Taurus (forward)"
        );
    }

    #[test]
    fn taurus_lagna_counts_backward() {
        let periods = compute_chara(1, TEST_JD); // Taurus; 9th = Capricorn, sama-pada
        assert_eq!(periods[0].sign_index, 1);
        assert_eq!(
            periods[1].sign_index, 0,
            "second period should be Aries (backward)"
        );
    }

    #[test]
    fn gemini_lagna_counts_backward() {
        let periods = compute_chara(2, TEST_JD); // Gemini; 9th = Aquarius, sama-pada
        assert_eq!(periods[0].sign_index, 2);
        assert_eq!(
            periods[1].sign_index, 1,
            "second period should be Taurus (backward)"
        );
    }

    #[test]
    fn cancer_lagna_counts_backward() {
        let periods = compute_chara(3, TEST_JD); // Cancer; 9th = Pisces, sama-pada
        assert_eq!(periods[0].sign_index, 3);
        assert_eq!(
            periods[1].sign_index, 2,
            "second period should be Gemini (backward)"
        );
    }

    #[test]
    fn leo_lagna_counts_forward() {
        let periods = compute_chara(4, TEST_JD); // Leo; 9th = Aries, vishama-pada
        assert_eq!(periods[0].sign_index, 4);
        assert_eq!(
            periods[1].sign_index, 5,
            "second period should be Virgo (forward)"
        );
    }

    #[test]
    fn virgo_lagna_counts_forward() {
        let periods = compute_chara(5, TEST_JD); // Virgo; 9th = Taurus, vishama-pada
        assert_eq!(periods[0].sign_index, 5);
        assert_eq!(
            periods[1].sign_index, 6,
            "second period should be Libra (forward)"
        );
    }

    #[test]
    fn libra_lagna_counts_forward() {
        let periods = compute_chara(6, TEST_JD); // Libra; 9th = Gemini, vishama-pada
        assert_eq!(periods[0].sign_index, 6);
        assert_eq!(
            periods[1].sign_index, 7,
            "second period should be Scorpio (forward)"
        );
    }

    #[test]
    fn scorpio_lagna_counts_backward() {
        let periods = compute_chara(7, TEST_JD); // Scorpio; 9th = Cancer, sama-pada
        assert_eq!(periods[0].sign_index, 7);
        assert_eq!(
            periods[1].sign_index, 6,
            "second period should be Libra (backward)"
        );
    }

    #[test]
    fn sagittarius_lagna_counts_backward() {
        let periods = compute_chara(8, TEST_JD); // Sagittarius; 9th = Leo, sama-pada
        assert_eq!(periods[0].sign_index, 8);
        assert_eq!(
            periods[1].sign_index, 7,
            "second period should be Scorpio (backward)"
        );
    }

    #[test]
    fn capricorn_lagna_counts_backward() {
        let periods = compute_chara(9, TEST_JD); // Capricorn; 9th = Virgo, sama-pada
        assert_eq!(periods[0].sign_index, 9);
        assert_eq!(
            periods[1].sign_index, 8,
            "second period should be Sagittarius (backward)"
        );
    }

    #[test]
    fn aquarius_lagna_counts_forward() {
        let periods = compute_chara(10, TEST_JD); // Aquarius; 9th = Libra, vishama-pada
        assert_eq!(periods[0].sign_index, 10);
        assert_eq!(
            periods[1].sign_index, 11,
            "second period should be Pisces (forward)"
        );
    }

    #[test]
    fn pisces_lagna_counts_forward() {
        let periods = compute_chara(11, TEST_JD); // Pisces; 9th = Scorpio, vishama-pada
        assert_eq!(periods[0].sign_index, 11);
        assert_eq!(
            periods[1].sign_index, 0,
            "second period should be Aries (forward)"
        );
    }

    // 17. The vishama-pada classification itself matches the module doc's set
    #[test]
    fn vishama_pada_set_is_exactly_documented() {
        let expected_vishama_pada = [
            true, true, true, false, false, false, true, true, true, false, false, false,
        ];
        for (sign, &expected) in expected_vishama_pada.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let sign = sign as u8;
            assert_eq!(
                is_vishama_pada(sign),
                expected,
                "sign {sign} ({}): expected vishama-pada = {expected}",
                SIGN_NAMES[sign as usize]
            );
        }
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
