// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Yogini Dasha — the 36-year 8-lord planetary period system.
//!
//! Yogini Dasha is a lesser-known but important dasha system from BPHS. It
//! cycles through 8 Yogini lords in a fixed 36-year cycle. The starting Yogini
//! is determined by the Moon's nakshatra index.
//!
//! **Formula:** Starting Yogini index = (`nakshatra_index` + 3) % 8
//!
//! **Dasha year length:** 365.25 days (Julian year, consistent with Vimshottari).
//!
//! Source: BPHS Ch. 56; B.V. Raman, "Hindu Predictive Astrology".

use serde::{Deserialize, Serialize};

use crate::nakshatra::Nakshatra;

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// Total years in the Yogini cycle.
pub const TOTAL_YOGINI_YEARS: f64 = 36.0;

/// The 8 Yogini lords with their ruling periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YoginiLord {
    /// Mangala (Mars) — 1 year.
    Mangala,
    /// Pingala (Sun) — 2 years.
    Pingala,
    /// Dhanya (Jupiter) — 3 years.
    Dhanya,
    /// Bhramari (Mars) — 4 years.
    Bhramari,
    /// Bhadrika (Mercury) — 5 years.
    Bhadrika,
    /// Ulka (Saturn) — 6 years.
    Ulka,
    /// Siddha (Venus) — 7 years.
    Siddha,
    /// Sankata (Rahu) — 8 years.
    Sankata,
}

impl YoginiLord {
    /// Duration of this Yogini's Maha period in years.
    #[must_use]
    pub const fn years(&self) -> f64 {
        match self {
            Self::Mangala => 1.0,
            Self::Pingala => 2.0,
            Self::Dhanya => 3.0,
            Self::Bhramari => 4.0,
            Self::Bhadrika => 5.0,
            Self::Ulka => 6.0,
            Self::Siddha => 7.0,
            Self::Sankata => 8.0,
        }
    }

    /// Conventional name of this Yogini lord.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Mangala => "Mangala",
            Self::Pingala => "Pingala",
            Self::Dhanya => "Dhanya",
            Self::Bhramari => "Bhramari",
            Self::Bhadrika => "Bhadrika",
            Self::Ulka => "Ulka",
            Self::Siddha => "Siddha",
            Self::Sankata => "Sankata",
        }
    }
}

/// Fixed Yogini sequence (indices 0–7).
pub const YOGINI_SEQUENCE: [YoginiLord; 8] = [
    YoginiLord::Mangala,
    YoginiLord::Pingala,
    YoginiLord::Dhanya,
    YoginiLord::Bhramari,
    YoginiLord::Bhadrika,
    YoginiLord::Ulka,
    YoginiLord::Siddha,
    YoginiLord::Sankata,
];

/// A single period in the Yogini Dasha hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoginiPeriod {
    /// The ruling Yogini of this period.
    pub lord: YoginiLord,
    /// Level: 1 = Maha, 2 = Antar, …, up to 5.
    pub level: u8,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in days.
    pub duration_days: f64,
    /// Sub-periods (next level down).
    pub sub_periods: Vec<YoginiPeriod>,
}

/// Complete Yogini Dasha tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoginiDasha {
    /// Moon's nakshatra at birth.
    pub moon_nakshatra: Nakshatra,
    /// Index into `YOGINI_SEQUENCE` for the first (current) Yogini at birth.
    pub starting_yogini_index: usize,
    /// Balance of the first Maha period (0.0 to 1.0).
    pub initial_balance: f64,
    /// All Maha periods (8 in sequence, first may be partial).
    pub maha_periods: Vec<YoginiPeriod>,
}

/// Compute sub-periods recursively.
fn compute_sub_periods(
    parent_idx: usize,
    start_jd: f64,
    parent_duration: f64,
    current_level: u8,
    max_level: u8,
) -> Vec<YoginiPeriod> {
    let mut periods = Vec::new();
    let mut current_jd = start_jd;

    for i in 0..8 {
        let idx = (parent_idx + i) % 8;
        let sub_lord = YOGINI_SEQUENCE[idx];
        let duration = parent_duration * sub_lord.years() / TOTAL_YOGINI_YEARS;
        let end_jd = current_jd + duration;

        let sub_periods = if current_level < max_level {
            compute_sub_periods(idx, current_jd, duration, current_level + 1, max_level)
        } else {
            Vec::new()
        };

        periods.push(YoginiPeriod {
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

/// Compute the complete Yogini Dasha tree.
///
/// # Arguments
/// * `moon_sidereal_longitude` — Moon's sidereal longitude in degrees at birth
/// * `birth_jd` — Julian Day of birth
/// * `levels` — how many levels deep (1 = Maha only, …, max 5)
///
/// # Formula
/// Starting Yogini index = (`nakshatra_index` + 3) % 8
///
/// Source: BPHS Ch. 56.
#[must_use]
pub fn compute_yogini(moon_sidereal_longitude: f64, birth_jd: f64, levels: u8) -> YoginiDasha {
    let levels = levels.clamp(1, 5);

    let nakshatra = Nakshatra::from_longitude(moon_sidereal_longitude);
    let nakshatra_index = nakshatra.index() as usize;

    // Determine starting Yogini: (nakshatra_index + 3) % 8
    let starting_yogini_index = (nakshatra_index + 3) % 8;
    let starting_lord = YOGINI_SEQUENCE[starting_yogini_index];

    // Compute balance of first Maha period based on position within nakshatra.
    let position_in_nak = moon_sidereal_longitude - nakshatra.start_longitude();
    let elapsed_fraction = position_in_nak / Nakshatra::SPAN;
    let initial_balance = 1.0 - elapsed_fraction;

    let mut maha_periods = Vec::with_capacity(8);
    let mut current_jd = birth_jd;

    let full_first_duration = starting_lord.years() * DASHA_YEAR_DAYS;

    for i in 0..8 {
        let idx = (starting_yogini_index + i) % 8;
        let maha_lord = YOGINI_SEQUENCE[idx];
        let full_duration_days = maha_lord.years() * DASHA_YEAR_DAYS;

        let duration = if i == 0 {
            full_first_duration * initial_balance
        } else {
            full_duration_days
        };

        let end_jd = current_jd + duration;

        let sub_periods = if levels > 1 {
            compute_sub_periods(idx, current_jd, duration, 2, levels)
        } else {
            Vec::new()
        };

        maha_periods.push(YoginiPeriod {
            lord: maha_lord,
            level: 1,
            start_jd: current_jd,
            end_jd,
            duration_days: duration,
            sub_periods,
        });

        current_jd = end_jd;
    }

    YoginiDasha {
        moon_nakshatra: nakshatra,
        starting_yogini_index,
        initial_balance,
        maha_periods,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;
    /// J2000.0
    const TEST_JD: f64 = 2_451_545.0;

    // 1. Total years in YOGINI_SEQUENCE sum to 36
    #[test]
    fn total_yogini_years_sum_to_36() {
        let total: f64 = YOGINI_SEQUENCE.iter().map(|l| l.years()).sum();
        assert!(
            (total - 36.0).abs() < EPS,
            "total yogini years: {total}, expected 36.0"
        );
    }

    // 2. Moon at 0° (Ashwini, index 0): starting lord = (0 + 3) % 8 = 3 = Bhramari
    #[test]
    fn moon_at_0_ashwini_starting_lord() {
        let result = compute_yogini(0.0, TEST_JD, 1);
        assert_eq!(result.moon_nakshatra, Nakshatra::Ashwini);
        // nakshatra_index=0 → starting = (0+3)%8 = 3 → Bhramari
        assert_eq!(result.starting_yogini_index, 3);
        assert_eq!(result.maha_periods[0].lord, YoginiLord::Bhramari);
        assert!(
            (result.initial_balance - 1.0).abs() < EPS,
            "balance at nakshatra start should be 1.0"
        );
    }

    // 3. Sub-periods sum to parent duration
    #[test]
    fn sub_periods_sum_to_parent() {
        let result = compute_yogini(0.0, TEST_JD, 2);
        for maha in &result.maha_periods {
            assert_eq!(maha.sub_periods.len(), 8);
            let sub_sum: f64 = maha.sub_periods.iter().map(|p| p.duration_days).sum();
            assert!(
                (sub_sum - maha.duration_days).abs() < 0.01,
                "lord {:?}: sub_sum={sub_sum}, parent={}",
                maha.lord,
                maha.duration_days
            );
        }
    }

    // 4. Levels are clamped to [1, 5]
    #[test]
    fn levels_clamped() {
        // level=0 should behave like level=1 (no sub-periods)
        let result0 = compute_yogini(0.0, TEST_JD, 0);
        assert!(result0.maha_periods[0].sub_periods.is_empty());

        // level=10 should behave like level=5
        let result10 = compute_yogini(0.0, TEST_JD, 10);
        let result5 = compute_yogini(0.0, TEST_JD, 5);
        assert_eq!(
            result10.maha_periods[0].sub_periods.len(),
            result5.maha_periods[0].sub_periods.len()
        );
    }
}
