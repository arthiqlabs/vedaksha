// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
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
//! **Starting lord:** Determined by `ASHTOTTARI_LORDS_BY_NAK` — the Ardradi
//! 27-element lookup table from BPHS Ch. 35, vv. 17–20. The sequence begins
//! from Ardra (nakshatra 5) with Sun as its lord. **Not** `nakshatra_index % 8`.
//!
//! **The first maha's sub-periods.** A maha always has its full length, and
//! its sub-periods divide that full length in their fixed shares.
//! The birth falls part of the way through the first maha, by the share of the
//! nakshatra the Moon has traversed, so that maha's **notional start** is the
//! birth less that share of its full length, and its sub-periods run from the
//! notional start. The maha itself still opens at the birth, with
//! `initial_balance` of its length. Its sub-periods are all kept: those with
//! `end_jd <= birth_jd` ended before birth, the one running at birth has
//! `start_jd < birth_jd < end_jd` and carries its true start, and so on down
//! every level. So the first maha's `sub_periods` sum to its full length, not to
//! its `duration_days`; every later maha's sub-periods tile it exactly. Until
//! v10.1.0 the first maha's balance alone was divided, from the birth, which
//! named the wrong running sub-period whenever the elapsed share exceeded the
//! maha lord's own.
//!
//! Source: BPHS Ch. 35 vv. 17–20 (Ardradi Ashtottari variant).

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
///
/// Source: BPHS Ch. 35 vv. 17–20 (Ardradi Ashtottari variant).
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

/// Ardradi nakshatra-to-starting-lord mapping (27 nakshatras → 8 lords).
///
/// The Ashtottari system begins its cycle from Ardra (nakshatra 5) with Sun
/// as the ruling lord. Each group of nakshatras maps to one lord in the order:
/// Sun, Moon, Mars, Mercury, Saturn, Jupiter, Rahu, Venus (then wraps back).
///
/// Source: BPHS, Ch. 35, vv. 17–20 (Ardradi Ashtottari variant).
pub const ASHTOTTARI_LORDS_BY_NAK: [AshtottariLord; 27] = [
    // 0  Ashwini         1  Bharani
    AshtottariLord::Rahu,
    AshtottariLord::Rahu,
    // 2  Krittika        3  Rohini          4  Mrigashira
    AshtottariLord::Venus,
    AshtottariLord::Venus,
    AshtottariLord::Venus,
    // 5  Ardra           6  Punarvasu       7  Pushya          8  Ashlesha
    AshtottariLord::Sun,
    AshtottariLord::Sun,
    AshtottariLord::Sun,
    AshtottariLord::Sun,
    // 9  Magha           10 PurvaPhalguni   11 UttaraPhalguni
    AshtottariLord::Moon,
    AshtottariLord::Moon,
    AshtottariLord::Moon,
    // 12 Hasta           13 Chitra          14 Swati           15 Vishakha
    AshtottariLord::Mars,
    AshtottariLord::Mars,
    AshtottariLord::Mars,
    AshtottariLord::Mars,
    // 16 Anuradha        17 Jyeshtha
    AshtottariLord::Mercury,
    AshtottariLord::Mercury,
    // 18 Moola           19 PurvaAshadha    20 UttaraAshadha
    AshtottariLord::Mercury,
    AshtottariLord::Saturn,
    AshtottariLord::Saturn,
    // 21 Shravana        22 Dhanishta       23 Shatabhisha
    AshtottariLord::Saturn,
    AshtottariLord::Jupiter,
    AshtottariLord::Jupiter,
    // 24 PurvaBhadrapada 25 UttaraBhadrapada 26 Revati
    AshtottariLord::Jupiter,
    AshtottariLord::Rahu,
    AshtottariLord::Rahu,
];

/// Return the position of a lord in [`ASHTOTTARI_SEQUENCE`] (0-7).
fn lord_sequence_index(lord: AshtottariLord) -> usize {
    match lord {
        AshtottariLord::Sun => 0,
        AshtottariLord::Moon => 1,
        AshtottariLord::Mars => 2,
        AshtottariLord::Mercury => 3,
        AshtottariLord::Saturn => 4,
        AshtottariLord::Jupiter => 5,
        AshtottariLord::Rahu => 6,
        AshtottariLord::Venus => 7,
    }
}

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
    let mut periods = Vec::with_capacity(8);
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

    // Starting lord: Ardradi lookup (BPHS Ch. 35, vv. 17–20).
    // NOT nakshatra_index % 8 — that gives Ashwini→Sun, which is wrong.
    // Ardradi means "starting from Ardra"; Ardra (index 5) → Sun.
    let starting_lord = ASHTOTTARI_LORDS_BY_NAK[nakshatra_index];
    let starting_idx = lord_sequence_index(starting_lord);

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

        // The sub-periods divide the maha's full length from its notional
        // start, which for the first maha is the birth less the elapsed share.
        let (sub_start, sub_span) = if i == 0 {
            (
                current_jd - full_duration_days * elapsed_fraction,
                full_duration_days,
            )
        } else {
            (current_jd, duration)
        };
        let sub_periods = if levels > 1 {
            compute_sub_periods(idx, sub_start, sub_span, 2, levels)
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

    // 6. Ardradi lord lookup: Ashwini (index 0) → Rahu (not Sun).
    //    Source: BPHS Ch. 35, vv. 17–20.
    #[test]
    fn moon_at_ashwini_starting_lord_is_rahu() {
        let result = compute_ashtottari(0.0, TEST_JD, 1);
        assert_eq!(
            result.starting_lord,
            AshtottariLord::Rahu,
            "Ashwini maps to Rahu in the Ardradi variant (BPHS Ch. 35)"
        );
        assert!(
            (result.initial_balance - 1.0).abs() < EPS,
            "balance at nakshatra start should be 1.0"
        );
    }

    // 7. Ardra (index 5) → Sun (the anchor of the Ardradi sequence).
    //    Source: BPHS Ch. 35 vv. 17–20.
    #[test]
    fn ardra_starting_lord_is_sun() {
        // Ardra starts at 5 * 13.333...° = 66.666...°
        let ardra_start = 5.0 * crate::nakshatra::Nakshatra::SPAN;
        let result = compute_ashtottari(ardra_start, TEST_JD, 1);
        assert_eq!(
            result.starting_lord,
            AshtottariLord::Sun,
            "Ardra (index 5) maps to Sun in Ardradi Ashtottari"
        );
    }

    // 8. Krittika (index 2) → Venus.
    //    Source: BPHS Ch. 35 vv. 17–20.
    #[test]
    fn krittika_starting_lord_is_venus() {
        let krittika_start = 2.0 * crate::nakshatra::Nakshatra::SPAN;
        let result = compute_ashtottari(krittika_start, TEST_JD, 1);
        assert_eq!(
            result.starting_lord,
            AshtottariLord::Venus,
            "Krittika (index 2) maps to Venus in Ardradi Ashtottari"
        );
    }

    // 9. Shravana (index 21) → Saturn.
    //    Source: BPHS Ch. 35 vv. 17–20.
    #[test]
    fn shravana_starting_lord_is_saturn() {
        let shravana_start = 21.0 * crate::nakshatra::Nakshatra::SPAN;
        let result = compute_ashtottari(shravana_start, TEST_JD, 1);
        assert_eq!(
            result.starting_lord,
            AshtottariLord::Saturn,
            "Shravana (index 21) maps to Saturn in Ardradi Ashtottari"
        );
    }

    // Rule test (handoff 2026-10-31): the first maha's sub-periods divide its
    // FULL length from its notional start (birth less the elapsed share), so
    // for an elapsed share `e` the sub-period running at birth is the first
    // whose cumulative share exceeds `e`, and every sub-period has its full
    // share. Checked at two levels against shares worked from the years table
    // alone. The elapsed shares chosen put the running antar past the maha
    // lord's own for most cases, which the pre-v10.1.0 builder got wrong.
    #[test]
    fn first_maha_sub_periods_run_from_the_notional_start() {
        // The index of the first entry whose cumulative share exceeds `e`.
        fn running(shares: &[f64], e: f64) -> (usize, f64) {
            let mut cum = 0.0;
            for (k, s) in shares.iter().enumerate() {
                if cum + s > e {
                    return (k, (e - cum) / s);
                }
                cum += s;
            }
            unreachable!("shares sum to one")
        }
        let mut moved = 0;
        for nak_index in [0_u8, 4, 13, 26] {
            for e in [0.05, 0.3, 0.5, 0.77, 0.95] {
                let start_lon = f64::from(nak_index) * Nakshatra::SPAN;
                let lon = start_lon + e * Nakshatra::SPAN;
                let tree = compute_ashtottari(lon, TEST_JD, 3);
                let maha = &tree.periods[0];
                let first = lord_sequence_index(tree.starting_lord);
                let shares: Vec<f64> = (0..8)
                    .map(|i| ASHTOTTARI_SEQUENCE[(first + i) % 8].years() / TOTAL_ASHTOTTARI_YEARS)
                    .collect();
                let full = ASHTOTTARI_SEQUENCE[first].years() * DASHA_YEAR_DAYS;
                let notional = TEST_JD - full * e;
                let subs = &maha.sub_periods;
                assert_eq!(subs.len(), 8);
                assert!((subs[0].start_jd - notional).abs() < 1e-6);
                assert!((subs[8 - 1].end_jd - maha.end_jd).abs() < 1e-6);
                for (i, sub) in subs.iter().enumerate() {
                    assert_eq!(sub.lord, ASHTOTTARI_SEQUENCE[(first + i) % 8]);
                    let want = full * shares[i];
                    assert!(
                        (sub.duration_days - want).abs() < 1e-9 * full,
                        "nak {nak_index} e {e} sub {i}: {} vs full share {want}",
                        sub.duration_days
                    );
                }
                let (k, e2) = running(&shares, e);
                if k > 0 {
                    moved += 1;
                }
                let at_birth = subs
                    .iter()
                    .position(|p| p.start_jd <= TEST_JD && TEST_JD < p.end_jd)
                    .expect("one sub-period holds the birth");
                assert_eq!(at_birth, k, "nak {nak_index} e {e}: running antar");
                // One level down, the same rule inside the running antar.
                let antar = &subs[k];
                let a_first = (first + k) % 8;
                let a_shares: Vec<f64> = (0..8)
                    .map(|i| {
                        ASHTOTTARI_SEQUENCE[(a_first + i) % 8].years() / TOTAL_ASHTOTTARI_YEARS
                    })
                    .collect();
                let (k2, _) = running(&a_shares, e2);
                let at_birth2 = antar
                    .sub_periods
                    .iter()
                    .position(|p| p.start_jd <= TEST_JD && TEST_JD < p.end_jd)
                    .expect("one pratyantar holds the birth");
                assert_eq!(at_birth2, k2, "nak {nak_index} e {e}: running pratyantar");
                for (i, p) in antar.sub_periods.iter().enumerate() {
                    let want = antar.duration_days * a_shares[i];
                    assert!((p.duration_days - want).abs() < 1e-9 * full);
                }
            }
        }
        // The probe can fail: most cases put the birth past the first antar.
        assert!(moved >= 10, "only {moved} cases exercise a later antar");
    }
}
