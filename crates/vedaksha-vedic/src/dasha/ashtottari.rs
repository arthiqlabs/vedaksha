// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Ashtottari Dasha — the 108-year planetary period system.
//!
//! The Ashtottari cycle divides 108 years among 8 planets (excluding Ketu) based
//! on the Moon's nakshatra at birth. Each Maha Dasha is subdivided into Antar,
//! Pratyantar, Sookshma, and Prana periods (up to 5 levels).
//!
//! **Dasha year length:** 365.25 days (standard Julian year per BPHS convention).
//!
//! **Lord sequence:** Sun(6), Moon(15), Mars(8), Mercury(17), Saturn(10),
//! Jupiter(19), Rahu(12), Venus(21) = 108 years total.
//!
//! **Starting lord:** `nakshatra_index % 8` maps to the sequence.
//!
//! Source: BPHS Ch. 46 Sl. 2-11.

use serde::{Deserialize, Serialize};

use crate::nakshatra::Nakshatra;

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// Total years in the Ashtottari cycle.
pub const TOTAL_ASHTOTTARI_YEARS: f64 = 108.0;

/// The 8 Ashtottari lords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AshtottariLord {
    /// Sun — 6 years.
    Sun,
    /// Moon — 15 years.
    Moon,
    /// Mars — 8 years.
    Mars,
    /// Mercury — 17 years.
    Mercury,
    /// Saturn — 10 years.
    Saturn,
    /// Jupiter — 19 years.
    Jupiter,
    /// Rahu — 12 years.
    Rahu,
    /// Venus — 21 years.
    Venus,
}

impl AshtottariLord {
    /// Duration of this lord's Maha Dasha period in years.
    #[must_use]
    pub const fn years(&self) -> f64 {
        match self {
            Self::Sun => 6.0,
            Self::Moon => 15.0,
            Self::Mars => 8.0,
            Self::Mercury => 17.0,
            Self::Saturn => 10.0,
            Self::Jupiter => 19.0,
            Self::Rahu => 12.0,
            Self::Venus => 21.0,
        }
    }

    /// Conventional name of this lord.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mars => "Mars",
            Self::Mercury => "Mercury",
            Self::Saturn => "Saturn",
            Self::Jupiter => "Jupiter",
            Self::Rahu => "Rahu",
            Self::Venus => "Venus",
        }
    }
}

/// Fixed Ashtottari sequence (indices 0-7).
pub const ASHTOTTARI_SEQUENCE: [AshtottariLord; 8] = [
    AshtottariLord::Sun,
    AshtottariLord::Moon,
    AshtottariLord::Mars,
    AshtottariLord::Mercury,
    AshtottariLord::Saturn,
    AshtottariLord::Jupiter,
    AshtottariLord::Rahu,
    AshtottariLord::Venus,
];

/// A period in the Ashtottari Dasha hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtottariPeriod {
    /// The ruling planet of this period.
    pub lord: AshtottariLord,
    /// Level: 1=Maha, 2=Antar, 3=Pratyantar, 4=Sookshma, 5=Prana.
    pub level: u8,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in days.
    pub duration_days: f64,
    /// Sub-periods (next level down).
    pub sub_periods: Vec<AshtottariPeriod>,
}

/// Complete Ashtottari Dasha tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtottariDasha {
    /// Moon's nakshatra at birth (as string name).
    pub moon_nakshatra: String,
    /// The starting lord of the first Maha Dasha.
    pub starting_lord: AshtottariLord,
    /// Balance of the first Maha Dasha (0.0 to 1.0).
    pub initial_balance: f64,
    /// All Maha Dasha periods (8 in sequence, first may be partial).
    pub periods: Vec<AshtottariPeriod>,
}

/// Compute sub-periods recursively.
fn compute_sub_periods(
    parent_idx: usize,
    start_jd: f64,
    parent_duration: f64,
    current_level: u8,
    max_level: u8,
) -> Vec<AshtottariPeriod> {
    let mut periods = Vec::new();
    let mut current_jd = start_jd;

    for i in 0..8 {
        let idx = (parent_idx + i) % 8;
        let sub_lord = ASHTOTTARI_SEQUENCE[idx];
        let duration = parent_duration * sub_lord.years() / TOTAL_ASHTOTTARI_YEARS;
        let end_jd = current_jd + duration;

        let sub_periods = if current_level < max_level {
            compute_sub_periods(idx, current_jd, duration, current_level + 1, max_level)
        } else {
            Vec::new()
        };

        periods.push(AshtottariPeriod {
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

/// Compute the complete Ashtottari Dasha tree.
///
/// # Arguments
/// * `moon_sidereal_longitude` — Moon's sidereal longitude in degrees at birth
/// * `birth_jd` — Julian Day of birth
/// * `levels` — how many levels deep (1=Maha only, 2=Maha+Antar, ..., max 5)
///
/// Source: BPHS Ch. 46 Sl. 2-11.
#[must_use]
pub fn compute_ashtottari(
    moon_sidereal_longitude: f64,
    birth_jd: f64,
    levels: u8,
) -> AshtottariDasha {
    let levels = levels.clamp(1, 5);

    let nakshatra = Nakshatra::from_longitude(moon_sidereal_longitude);
    let nakshatra_index = nakshatra.index() as usize;

    // Starting lord: nakshatra_index % 8
    let starting_idx = nakshatra_index % 8;
    let starting_lord = ASHTOTTARI_SEQUENCE[starting_idx];

    // Compute balance of first Maha Dasha based on position within nakshatra.
    let position_in_nak = moon_sidereal_longitude - nakshatra.start_longitude();
    let elapsed_fraction = position_in_nak / Nakshatra::SPAN;
    let initial_balance = 1.0 - elapsed_fraction;

    let mut periods = Vec::with_capacity(8);
    let mut current_jd = birth_jd;

    for i in 0..8 {
        let idx = (starting_idx + i) % 8;
        let maha_lord = ASHTOTTARI_SEQUENCE[idx];
        let full_duration_days = maha_lord.years() * DASHA_YEAR_DAYS;

        let duration = if i == 0 {
            full_duration_days * initial_balance
        } else {
            full_duration_days
        };

        let end_jd = current_jd + duration;

        let sub_periods = if levels > 1 {
            compute_sub_periods(idx, current_jd, duration, 2, levels)
        } else {
            Vec::new()
        };

        periods.push(AshtottariPeriod {
            lord: maha_lord,
            level: 1,
            start_jd: current_jd,
            end_jd,
            duration_days: duration,
            sub_periods,
        });

        current_jd = end_jd;
    }

    AshtottariDasha {
        moon_nakshatra: format!("{:?}", nakshatra),
        starting_lord,
        initial_balance,
        periods,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;
    /// J2000.0
    const TEST_JD: f64 = 2_451_545.0;

    // 1. Total years in ASHTOTTARI_SEQUENCE sum to 108
    #[test]
    fn ashtottari_total_is_108_years() {
        let total: f64 = ASHTOTTARI_SEQUENCE.iter().map(|l| l.years()).sum();
        assert!(
            (total - 108.0).abs() < EPS,
            "total ashtottari years: {total}, expected 108.0"
        );
    }

    // 2. Compute produces exactly 8 maha periods
    #[test]
    fn ashtottari_has_8_maha_periods() {
        let result = compute_ashtottari(0.0, TEST_JD, 1);
        assert_eq!(
            result.periods.len(),
            8,
            "expected 8 Ashtottari maha periods, got {}",
            result.periods.len()
        );
    }

    // 3. Sub-periods sum to parent duration
    #[test]
    fn ashtottari_sub_periods_sum_to_parent() {
        let result = compute_ashtottari(0.0, TEST_JD, 2);
        for maha in &result.periods {
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

    // 4. Full cycle with balance=1.0 covers 108 years
    #[test]
    fn full_cycle_covers_108_years() {
        let result = compute_ashtottari(0.0, TEST_JD, 1);
        let total_days: f64 = result.periods.iter().map(|m| m.duration_days).sum();
        let expected = 108.0 * 365.25;
        assert!(
            (total_days - expected).abs() < 0.01,
            "total_days={total_days}, expected={expected}"
        );
    }

    // 5. Periods are contiguous
    #[test]
    fn periods_are_contiguous() {
        let result = compute_ashtottari(5.0, TEST_JD, 2);
        for i in 1..result.periods.len() {
            let gap = (result.periods[i].start_jd - result.periods[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between periods {}-{}: {gap}", i - 1, i);
        }
    }

    // 6. Moon at 0° (Ashwini, index 0): starting lord = index 0 % 8 = 0 = Sun
    #[test]
    fn moon_at_0_starting_lord_is_sun() {
        let result = compute_ashtottari(0.0, TEST_JD, 1);
        assert_eq!(result.starting_lord, AshtottariLord::Sun);
        assert!(
            (result.initial_balance - 1.0).abs() < EPS,
            "balance at nakshatra start should be 1.0"
        );
    }
}
