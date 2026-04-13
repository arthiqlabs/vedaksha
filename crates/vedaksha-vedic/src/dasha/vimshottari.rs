// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vimshottari Dasha — the 120-year planetary period system.
//!
//! The Vimshottari cycle divides 120 years among 9 planets based on the Moon's
//! nakshatra at birth. Each Maha Dasha is subdivided into Antar Dasha, Pratyantar,
//! Sookshma, and Prana periods (up to 5 levels).
//!
//! **Dasha year length:** 365.25 days (standard Julian year per BPHS convention).
//!
//! **Boundary convention:** Moon at exactly 0° sidereal is in Ashwini (not Revati).
//! The nakshatra boundary uses `>=` lower / `<` upper, matching `Nakshatra::from_longitude`.
//!
//! Source: BPHS Ch. 46-47; B.V. Raman, "Hindu Predictive Astrology" Ch. 18.

use serde::{Deserialize, Serialize};

use crate::nakshatra::{DashaLord, Nakshatra};

/// Dasha year length in days (Julian year).
///
/// Note: BPHS specifies the 120-year cycle without leap-year adjustments.
/// Using 365.25 (Julian year) is the standard convention in classical Jyotish
/// software and aligns with the tropical/sidereal year approximation used
/// throughout BPHS calculations.
const DASHA_YEAR_DAYS: f64 = 365.25;

/// Total years in the Vimshottari cycle.
const TOTAL_DASHA_YEARS: f64 = 120.0;

/// Fixed Vimshottari dasha sequence.
const DASHA_SEQUENCE: [DashaLord; 9] = [
    DashaLord::Ketu,
    DashaLord::Venus,
    DashaLord::Sun,
    DashaLord::Moon,
    DashaLord::Mars,
    DashaLord::Rahu,
    DashaLord::Jupiter,
    DashaLord::Saturn,
    DashaLord::Mercury,
];

/// A period in the Vimshottari dasha hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaPeriod {
    /// The ruling planet of this period.
    pub lord: DashaLord,
    /// Level: 1=Maha, 2=Antar, 3=Pratyantar, 4=Sookshma, 5=Prana.
    pub level: u8,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in days.
    pub duration_days: f64,
    /// Sub-periods (next level down).
    pub sub_periods: Vec<DashaPeriod>,
}

/// Complete Vimshottari dasha tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariDasha {
    /// The birth Moon's nakshatra.
    pub moon_nakshatra: Nakshatra,
    /// Balance of first Maha Dasha (0.0 to 1.0).
    pub initial_balance: f64,
    /// All Maha Dasha periods (9 in sequence, first may be partial).
    pub maha_dashas: Vec<DashaPeriod>,
}

/// Find the index of a `DashaLord` in `DASHA_SEQUENCE`.
fn dasha_sequence_index(lord: DashaLord) -> usize {
    DASHA_SEQUENCE
        .iter()
        .position(|&l| l == lord)
        .expect("all DashaLord variants are in DASHA_SEQUENCE")
}

/// Compute sub-periods recursively.
fn compute_sub_periods(
    parent_lord: DashaLord,
    start_jd: f64,
    parent_duration: f64,
    current_level: u8,
    max_level: u8,
) -> Vec<DashaPeriod> {
    let parent_idx = dasha_sequence_index(parent_lord);
    let mut periods = Vec::new();
    let mut current_jd = start_jd;

    // Sub-periods start from the parent lord, then follow the sequence.
    for i in 0..9 {
        let idx = (parent_idx + i) % 9;
        let sub_lord = DASHA_SEQUENCE[idx];
        let duration = parent_duration * sub_lord.maha_dasha_years() / TOTAL_DASHA_YEARS;
        let end_jd = current_jd + duration;

        let sub_periods = if current_level < max_level {
            compute_sub_periods(sub_lord, current_jd, duration, current_level + 1, max_level)
        } else {
            Vec::new()
        };

        periods.push(DashaPeriod {
            lord: sub_lord,
            level: current_level,
            start_jd: current_jd,
            end_jd,
            duration_days: duration,
            sub_periods,
        });

        current_jd = end_jd;
    }

    periods
}

/// Compute the complete Vimshottari dasha tree.
///
/// # Arguments
/// * `moon_sidereal_longitude` — Moon's sidereal longitude in degrees at birth
/// * `birth_jd` — Julian Day of birth
/// * `levels` — how many levels deep (1=Maha only, 2=Maha+Antar, ..., max 5)
///
/// Source: BPHS Ch. 46-47; B.V. Raman Ch. 18.
#[must_use]
pub fn compute_vimshottari(
    moon_sidereal_longitude: f64,
    birth_jd: f64,
    levels: u8,
) -> VimshottariDasha {
    let levels = levels.clamp(1, 5);

    let nakshatra = Nakshatra::from_longitude(moon_sidereal_longitude);
    let lord = nakshatra.dasha_lord();

    // Compute balance of first Maha Dasha.
    let position_in_nak = moon_sidereal_longitude - nakshatra.start_longitude();
    let elapsed_fraction = position_in_nak / Nakshatra::SPAN;
    let initial_balance = 1.0 - elapsed_fraction;

    // Build dasha sequence starting from current lord.
    let sequence_start = dasha_sequence_index(lord);

    let mut maha_dashas = Vec::with_capacity(9);
    let mut current_jd = birth_jd;

    // One full 120-year cycle: 9 Maha Dashas starting from current lord.
    for i in 0..9 {
        let idx = (sequence_start + i) % 9;
        let maha_lord = DASHA_SEQUENCE[idx];
        let full_duration_days = maha_lord.maha_dasha_years() * DASHA_YEAR_DAYS;

        let duration = if i == 0 {
            full_duration_days * initial_balance
        } else {
            full_duration_days
        };

        let end_jd = current_jd + duration;

        let sub_periods = if levels > 1 {
            compute_sub_periods(maha_lord, current_jd, duration, 2, levels)
        } else {
            Vec::new()
        };

        maha_dashas.push(DashaPeriod {
            lord: maha_lord,
            level: 1,
            start_jd: current_jd,
            end_jd,
            duration_days: duration,
            sub_periods,
        });

        current_jd = end_jd;
    }

    VimshottariDasha {
        moon_nakshatra: nakshatra,
        initial_balance,
        maha_dashas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;
    /// Arbitrary birth Julian Day for tests.
    const TEST_JD: f64 = 2_451_545.0; // J2000.0

    // 1. Total years sum to 120
    #[test]
    fn total_dasha_years_sum_to_120() {
        let total: f64 = DASHA_SEQUENCE.iter().map(|l| l.maha_dasha_years()).sum();
        assert!(
            (total - 120.0).abs() < EPS,
            "total dasha years: {total}, expected 120.0"
        );
    }

    // 2. Moon at 0° (Ashwini): starting lord = Ketu, balance = 1.0
    #[test]
    fn moon_at_0_ashwini_full_ketu() {
        let result = compute_vimshottari(0.0, TEST_JD, 1);
        assert_eq!(result.moon_nakshatra, Nakshatra::Ashwini);
        assert!((result.initial_balance - 1.0).abs() < EPS);
        assert_eq!(result.maha_dashas[0].lord, DashaLord::Ketu);
    }

    // 3. Moon at 6.667° (middle of Ashwini): balance ≈ 0.5
    #[test]
    fn moon_at_middle_ashwini_half_balance() {
        let mid = Nakshatra::SPAN / 2.0; // 6.6666...
        let result = compute_vimshottari(mid, TEST_JD, 1);
        assert_eq!(result.moon_nakshatra, Nakshatra::Ashwini);
        assert!(
            (result.initial_balance - 0.5).abs() < EPS,
            "balance: {}, expected ~0.5",
            result.initial_balance
        );
    }

    // 4. Moon at 13.333° (end of Ashwini / start of Bharani): lord = Venus
    #[test]
    fn moon_at_nakshatra_boundary_bharani() {
        let result = compute_vimshottari(Nakshatra::SPAN, TEST_JD, 1);
        assert_eq!(result.moon_nakshatra, Nakshatra::Bharani);
        assert_eq!(result.maha_dashas[0].lord, DashaLord::Venus);
        // Balance should be 1.0 (just entered Bharani)
        assert!(
            (result.initial_balance - 1.0).abs() < EPS,
            "balance: {}",
            result.initial_balance
        );
    }

    // 5. Maha Dasha sequence: first 3 lords starting from Ketu
    #[test]
    fn maha_dasha_sequence_from_ketu() {
        let result = compute_vimshottari(0.0, TEST_JD, 1);
        assert_eq!(result.maha_dashas[0].lord, DashaLord::Ketu);
        assert_eq!(result.maha_dashas[1].lord, DashaLord::Venus);
        assert_eq!(result.maha_dashas[2].lord, DashaLord::Sun);
    }

    // 6. Duration of full Ketu Maha Dasha: 7 * 365.25 = 2556.75 days
    #[test]
    fn full_ketu_maha_dasha_duration() {
        let result = compute_vimshottari(0.0, TEST_JD, 1);
        let expected = 7.0 * 365.25;
        assert!(
            (result.maha_dashas[0].duration_days - expected).abs() < EPS,
            "got {}, expected {}",
            result.maha_dashas[0].duration_days,
            expected
        );
    }

    // 7. Antar Dasha sub-periods sum to parent duration
    #[test]
    fn antar_dashas_sum_to_parent() {
        let result = compute_vimshottari(0.0, TEST_JD, 2);
        for maha in &result.maha_dashas {
            assert_eq!(maha.sub_periods.len(), 9);
            let sub_sum: f64 = maha.sub_periods.iter().map(|p| p.duration_days).sum();
            assert!(
                (sub_sum - maha.duration_days).abs() < 0.01,
                "lord {:?}: sub_sum={sub_sum}, parent={}",
                maha.lord,
                maha.duration_days
            );
        }
    }

    // 8. Antar Dasha starts from parent lord: Ketu Maha -> first Antar = Ketu
    #[test]
    fn antar_dasha_starts_from_parent_lord() {
        let result = compute_vimshottari(0.0, TEST_JD, 2);
        // Ketu Maha -> first Antar = Ketu-Ketu
        assert_eq!(result.maha_dashas[0].sub_periods[0].lord, DashaLord::Ketu);
        // Venus Maha -> first Antar = Venus-Venus
        assert_eq!(result.maha_dashas[1].sub_periods[0].lord, DashaLord::Venus);
    }

    // 9. Level 3 (Pratyantar): each antar has 9 sub-periods
    #[test]
    fn pratyantar_level_3_has_9_sub_periods() {
        let result = compute_vimshottari(0.0, TEST_JD, 3);
        let first_antar = &result.maha_dashas[0].sub_periods[0];
        assert_eq!(first_antar.sub_periods.len(), 9);
        assert_eq!(first_antar.sub_periods[0].level, 3);
        // Pratyantar sub-periods should sum to antar duration
        let sub_sum: f64 = first_antar
            .sub_periods
            .iter()
            .map(|p| p.duration_days)
            .sum();
        assert!(
            (sub_sum - first_antar.duration_days).abs() < 0.001,
            "pratyantar sum={sub_sum}, antar={}",
            first_antar.duration_days
        );
    }

    // 10. Full 120-year cycle coverage (with balance=1.0)
    #[test]
    fn full_cycle_covers_120_years() {
        let result = compute_vimshottari(0.0, TEST_JD, 1);
        let total_days: f64 = result.maha_dashas.iter().map(|m| m.duration_days).sum();
        let expected = 120.0 * 365.25;
        assert!(
            (total_days - expected).abs() < 0.01,
            "total_days={total_days}, expected={expected}"
        );
    }

    // 11. Partial first dasha: total < 120 years
    #[test]
    fn partial_first_dasha_less_than_120_years() {
        // Moon at mid-Ashwini: balance = 0.5, so first Ketu is 3.5 years
        let mid = Nakshatra::SPAN / 2.0;
        let result = compute_vimshottari(mid, TEST_JD, 1);
        let total_days: f64 = result.maha_dashas.iter().map(|m| m.duration_days).sum();
        // Should be 120 - 3.5 = 116.5 years
        let expected = (120.0 - 3.5) * 365.25;
        assert!(
            (total_days - expected).abs() < 0.01,
            "total_days={total_days}, expected={expected}"
        );
    }

    // 12. Maha Dasha periods are contiguous (no gaps)
    #[test]
    fn maha_dashas_are_contiguous() {
        let result = compute_vimshottari(5.0, TEST_JD, 2);
        for i in 1..result.maha_dashas.len() {
            let gap = (result.maha_dashas[i].start_jd - result.maha_dashas[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between maha dashas {}-{}: {gap}", i - 1, i);
        }
    }
}
