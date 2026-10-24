// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
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
//! **Source and grading.** BPHS, the dasha chapter (ch. 46 in the recension
//! this crate follows throughout, as `dasha::chara` does; another recension
//! counts two extra chapters earlier and numbers it 48, with the same sloka
//! numbers), slokas 195-200:
//!
//! - 195-196: the eight yoginis, in order Mangala, Pingala, Dhanya, Bhramari,
//!   Bhadrika, Ulka, Siddha, Sankata. Grade (a): the verse itself.
//! - 197-198: each yogini's graha, one per yogini, in the same order: Chandra
//!   (Moon) is born from Mangala, Surya (Sun) from Pingala, Guru (Jupiter)
//!   from Dhanya, Kuja (Mars) from Bhramari, Budha (Mercury) from Bhadrika,
//!   Shanaishchara (Saturn) from Ulka, Bhargava (Venus) from Siddha, and
//!   *tamas* (Rahu) from Sankata. Grade (a). The lord docs below follow this
//!   verse. (An earlier doc on [`YoginiLord::Mangala`] read "Mars", which is
//!   Bhramari's graha, not Mangala's; only the comment was affected.)
//! - 199: "add three to the birth nakshatra and divide by eight (*vasu*); the
//!   remainder one, two, ... names Mangala, Pingala, ..." — with Ashwini
//!   counted as one, and a remainder of nothing taken as the eighth. This is
//!   the formula above. Grade (a).
//! - 200: the maha periods run one year, two years, ... eight years in that
//!   order, and the balance at birth is worked from the nakshatra's elapsed
//!   and unelapsed parts (*bhukta* and *bhogya*). Grade (a).
//!
//! **After the first round.** The verses read fix the eight yoginis, their
//! years, the start rule and the balance; they do not say in words what
//! follows the eighth period. [`compute_yogini_rounds`] and
//! [`compute_yogini_until`] continue the same eight in the same order, each
//! later round whole (1 + 2 + ... + 8 = 36 years), beginning again with the
//! birth yogini. That is the conventional extension and the reading the
//! remainder arithmetic of sloka 199 itself implies (it wraps modulo eight);
//! grade (c), a convention rather than a stated rule. [`compute_yogini`]
//! keeps its single round, unchanged.
//!
//! **Sub-periods.** The sloka text read (195-200) gives no sub-period rule
//! for this system. The crate's sub-periods (each period split among the
//! eight yoginis from its own yogini, in proportion to their years over 36,
//! recursively) are a convention shared with the other nakshatra dashas in
//! this crate, and apply identically in every round; grade (c).

use serde::{Deserialize, Serialize};

use crate::nakshatra::Nakshatra;

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// Total years in the Yogini cycle.
pub const TOTAL_YOGINI_YEARS: f64 = 36.0;

/// The most rounds of the Yogini cycle one call will produce.
///
/// [`compute_yogini_rounds`] clamps its `rounds` argument to
/// `1..=MAX_YOGINI_ROUNDS`, and [`compute_yogini_until`] stops here however
/// far its horizon lies. Ten rounds is 360 years, three times the 120-year
/// span the other nakshatra dashas of this crate treat as a full life, and at
/// five levels the whole tree is still only a few hundred thousand periods.
/// A truncated horizon is visible: the last maha's `end_jd` is where the
/// tree stops.
pub const MAX_YOGINI_ROUNDS: u8 = 10;

/// The 8 Yogini lords with their ruling periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YoginiLord {
    /// Mangala (Chandra, the Moon) — 1 year.
    Mangala,
    /// Pingala (Surya, the Sun) — 2 years.
    Pingala,
    /// Dhanya (Guru, Jupiter) — 3 years.
    Dhanya,
    /// Bhramari (Kuja, Mars) — 4 years.
    Bhramari,
    /// Bhadrika (Budha, Mercury) — 5 years.
    Bhadrika,
    /// Ulka (Shani, Saturn) — 6 years.
    Ulka,
    /// Siddha (Shukra, Venus) — 7 years.
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
    /// Which 36-year round of the Yogini cycle this period belongs to,
    /// 1-based. Round 1 is the round from birth, whose first maha is the
    /// birth yogini's unelapsed share; every later round runs all eight
    /// yoginis whole. A sub-period carries the round of the maha it sits in.
    /// Serialised data from before this field existed reads back as round 1.
    #[serde(default = "first_round")]
    pub round: u8,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in days.
    pub duration_days: f64,
    /// Sub-periods (next level down).
    pub sub_periods: Vec<YoginiPeriod>,
}

/// Serde default for [`YoginiPeriod::round`]: output written before the
/// field existed held the first round only.
const fn first_round() -> u8 {
    1
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
    /// All Maha periods: eight per round, in sequence, the first of round 1
    /// partial. [`compute_yogini`] returns one round (8 periods);
    /// [`compute_yogini_rounds`] and [`compute_yogini_until`] return more.
    pub maha_periods: Vec<YoginiPeriod>,
}

/// Compute sub-periods recursively.
fn compute_sub_periods(
    parent_idx: usize,
    start_jd: f64,
    parent_duration: f64,
    current_level: u8,
    max_level: u8,
    round: u8,
) -> Vec<YoginiPeriod> {
    let mut periods = Vec::with_capacity(8);
    let mut current_jd = start_jd;

    for i in 0..8 {
        let idx = (parent_idx + i) % 8;
        let sub_lord = YOGINI_SEQUENCE[idx];
        let duration = parent_duration * sub_lord.years() / TOTAL_YOGINI_YEARS;
        let end_jd = current_jd + duration;

        let sub_periods = if current_level < max_level {
            compute_sub_periods(
                idx,
                current_jd,
                duration,
                current_level + 1,
                max_level,
                round,
            )
        } else {
            Vec::new()
        };

        periods.push(YoginiPeriod {
            lord: sub_lord,
            level: current_level,
            round,
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
/// Returns the first round only: eight maha periods, the birth yogini's
/// unelapsed share and then the next seven whole, ending before the native's
/// 36th year. For later rounds see [`compute_yogini_rounds`] and
/// [`compute_yogini_until`].
///
/// Source: BPHS dasha chapter, slokas 195-200 (see the module doc).
#[must_use]
pub fn compute_yogini(moon_sidereal_longitude: f64, birth_jd: f64, levels: u8) -> YoginiDasha {
    compute_yogini_rounds(moon_sidereal_longitude, birth_jd, levels, 1)
}

/// Compute the Yogini Dasha tree for `rounds` successive 36-year rounds.
///
/// Round 1 is exactly what [`compute_yogini`] returns. Each later round runs
/// all eight yoginis whole (1 + 2 + … + 8 = 36 years), continuing the fixed
/// [`YOGINI_SEQUENCE`] order from the yogini after the last one of the round
/// before, so its first maha is the birth yogini again, in full, opening at
/// the previous round's last `end_jd`. Sub-periods follow the same rule in
/// every round. The module doc grades the continuation as a convention.
///
/// `rounds` is clamped to `1..=`[`MAX_YOGINI_ROUNDS`], as `levels` is clamped
/// to `1..=5`. The result's `maha_periods` holds `8 × rounds` entries, each
/// carrying its [`YoginiPeriod::round`].
#[must_use]
pub fn compute_yogini_rounds(
    moon_sidereal_longitude: f64,
    birth_jd: f64,
    levels: u8,
    rounds: u8,
) -> YoginiDasha {
    let rounds = rounds.clamp(1, MAX_YOGINI_ROUNDS);
    build_yogini(
        moon_sidereal_longitude,
        birth_jd,
        levels,
        |round, _round_end| round >= rounds,
    )
}

/// Compute the Yogini Dasha tree out to a horizon.
///
/// Whole rounds are produced until the tree reaches `until_jd`: the last
/// round's final `end_jd` is the first one at or beyond it. At least one
/// round is always returned, so a horizon inside the first 36 years gives
/// [`compute_yogini`]'s output exactly. A horizon exactly on a round's end
/// stops there; one a hair past it takes the next round whole. At most
/// [`MAX_YOGINI_ROUNDS`] rounds are produced however far the horizon is
/// (compare the last maha's `end_jd` with `until_jd` to see whether it was
/// cut), and a NaN horizon gives one round.
#[must_use]
pub fn compute_yogini_until(
    moon_sidereal_longitude: f64,
    birth_jd: f64,
    levels: u8,
    until_jd: f64,
) -> YoginiDasha {
    build_yogini(
        moon_sidereal_longitude,
        birth_jd,
        levels,
        |round, round_end| round >= MAX_YOGINI_ROUNDS || round_end >= until_jd || until_jd.is_nan(),
    )
}

/// Build the tree round by round; `done(round, round_end_jd)` is asked after
/// each finished round whether to stop.
fn build_yogini(
    moon_sidereal_longitude: f64,
    birth_jd: f64,
    levels: u8,
    mut done: impl FnMut(u8, f64) -> bool,
) -> YoginiDasha {
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

    let mut round: u8 = 1;
    loop {
        for i in 0..8 {
            let idx = (starting_yogini_index + i) % 8;
            let maha_lord = YOGINI_SEQUENCE[idx];
            let full_duration_days = maha_lord.years() * DASHA_YEAR_DAYS;

            // Only round 1 opens with the unelapsed share; every later round
            // starts again with the birth yogini, whole.
            let duration = if round == 1 && i == 0 {
                full_first_duration * initial_balance
            } else {
                full_duration_days
            };

            let end_jd = current_jd + duration;

            let sub_periods = if levels > 1 {
                compute_sub_periods(idx, current_jd, duration, 2, levels, round)
            } else {
                Vec::new()
            };

            maha_periods.push(YoginiPeriod {
                lord: maha_lord,
                level: 1,
                round,
                start_jd: current_jd,
                end_jd,
                duration_days: duration,
                sub_periods,
            });

            current_jd = end_jd;
        }

        if done(round, current_jd) {
            break;
        }
        round += 1;
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

    // ── Rounds beyond the first ──────────────────────────────────────────
    //
    // Hand-derived vectors. Ashwini (index 0) gives starting yogini
    // (0 + 3) % 8 = 3 = Bhramari. A Moon at 0 degrees has balance 1.0, so
    // round 1 is Bhramari 4, Bhadrika 5, Ulka 6, Siddha 7, Sankata 8,
    // Mangala 1, Pingala 2, Dhanya 3 years. In days (x 365.25): 1461,
    // 1826.25, 2191.5, 2556.75, 2922, 365.25, 730.5, 1095.75. From J2000.0
    // (2451545.0) the cumulative bounds are 2453006, 2454832.25, 2457023.75,
    // 2459580.5, 2462502.5, 2462867.75, 2463598.25, 2464694. Every figure is
    // a multiple of 0.25 and so exact in f64 at this magnitude, which is why
    // the comparisons below are `==`.

    const ASHWINI_ROUND_ONE: [(YoginiLord, f64); 8] = [
        (YoginiLord::Bhramari, 2_453_006.0),
        (YoginiLord::Bhadrika, 2_454_832.25),
        (YoginiLord::Ulka, 2_457_023.75),
        (YoginiLord::Siddha, 2_459_580.5),
        (YoginiLord::Sankata, 2_462_502.5),
        (YoginiLord::Mangala, 2_462_867.75),
        (YoginiLord::Pingala, 2_463_598.25),
        (YoginiLord::Dhanya, 2_464_694.0),
    ];

    // 5. Round one is the hand-derived vector, whichever entry point asks.
    #[test]
    fn round_one_matches_the_hand_derived_vector() {
        for tree in [
            compute_yogini(0.0, TEST_JD, 1),
            compute_yogini_rounds(0.0, TEST_JD, 1, 1),
            compute_yogini_rounds(0.0, TEST_JD, 1, 3),
            compute_yogini_until(0.0, TEST_JD, 1, TEST_JD + 100.0),
        ] {
            let mut start = TEST_JD;
            for (maha, &(lord, end)) in tree.maha_periods.iter().zip(&ASHWINI_ROUND_ONE) {
                assert_eq!(maha.lord, lord);
                assert_eq!(maha.round, 1);
                assert_eq!(maha.start_jd, start);
                assert_eq!(maha.end_jd, end);
                start = end;
            }
        }
    }

    // 6. A Moon half way through Ashwini has balance exactly 0.5 (the
    // half-span is an exact halving), so the first maha is 2 years = 730.5
    // days and round 1 is 34 years long. Round 2 therefore opens at
    // 2451545 + 730.5 + (5+6+7+8+1+2+3 = 32 years = 11688 days)
    // = 2463963.5 with the birth yogini in full, and ends 13149 days
    // (36 years) later at 2477112.5.
    #[test]
    fn round_two_opens_where_round_one_ends_with_the_birth_yogini_in_full() {
        let tree = compute_yogini_rounds(0.5 * Nakshatra::SPAN, TEST_JD, 1, 2);
        assert_eq!(tree.initial_balance, 0.5);
        assert_eq!(tree.maha_periods.len(), 16);
        let (r1, r2) = tree.maha_periods.split_at(8);
        assert_eq!(r1[0].duration_days, 730.5);
        assert_eq!(r1[7].end_jd, 2_463_963.5);
        assert_eq!(r2[0].start_jd, r1[7].end_jd);
        assert_eq!(r2[0].lord, YoginiLord::Bhramari);
        assert_eq!(
            r2[0].duration_days, 1461.0,
            "birth yogini whole, not the balance"
        );
        assert_eq!(r2[7].end_jd, 2_477_112.5);
        assert!(r1.iter().all(|p| p.round == 1));
        assert!(r2.iter().all(|p| p.round == 2));
    }

    // 7. For every nakshatra and a spread of positions inside it: every
    // maha of every later round carries its own yogini's whole years, the
    // lords run YOGINI_SEQUENCE in order from the birth yogini and wrap, the
    // tree is contiguous, and each later round is 36 x 365.25 days.
    #[test]
    fn later_rounds_are_the_eight_yoginis_whole_in_sequence_order() {
        for nak in 0..27 {
            for frac in [0.0, 0.25, 0.5, 0.9999] {
                let lon = (f64::from(nak) + frac) * Nakshatra::SPAN;
                let tree = compute_yogini_rounds(lon, TEST_JD, 1, 4);
                assert_eq!(tree.maha_periods.len(), 32);
                for (i, maha) in tree.maha_periods.iter().enumerate() {
                    let expect = YOGINI_SEQUENCE[(tree.starting_yogini_index + i) % 8];
                    assert_eq!(maha.lord, expect, "nak {nak} frac {frac} maha {i}");
                    assert_eq!(usize::from(maha.round), i / 8 + 1);
                    if i >= 8 {
                        assert_eq!(maha.duration_days, expect.years() * 365.25);
                    }
                    if i > 0 {
                        assert_eq!(maha.start_jd, tree.maha_periods[i - 1].end_jd);
                    }
                }
                for r in 1..4 {
                    let open = tree.maha_periods[8 * r].start_jd;
                    let close = tree.maha_periods[8 * r + 7].end_jd;
                    assert!(
                        (close - open - 36.0 * 365.25).abs() < 1e-6,
                        "nak {nak} frac {frac} round {}: {}",
                        r + 1,
                        close - open
                    );
                }
            }
        }
    }

    // 8. At every level of every round, a period's sub-periods tile it:
    // the first starts at its start, the last ends at its end (to float
    // rounding), adjacent ones share their bound, the first sub-period is
    // the period's own yogini, and each carries its parent's round.
    #[test]
    fn sub_periods_tile_their_parent_in_every_round() {
        fn check(p: &YoginiPeriod, levels: u8) {
            if p.level == levels {
                assert!(p.sub_periods.is_empty());
                return;
            }
            assert_eq!(p.sub_periods.len(), 8);
            assert_eq!(p.sub_periods[0].start_jd, p.start_jd);
            assert_eq!(
                p.sub_periods[0].lord, p.lord,
                "first sub is the period's own"
            );
            assert!((p.sub_periods[7].end_jd - p.end_jd).abs() < 1e-6);
            for w in p.sub_periods.windows(2) {
                assert_eq!(w[0].end_jd, w[1].start_jd);
            }
            for sub in &p.sub_periods {
                assert_eq!(sub.round, p.round);
                assert_eq!(sub.level, p.level + 1);
                // Proportional to the yoginis' years over 36.
                let want = p.duration_days * sub.lord.years() / TOTAL_YOGINI_YEARS;
                assert!((sub.duration_days - want).abs() < 1e-9 * p.duration_days.max(1.0));
                check(sub, levels);
            }
        }
        let tree = compute_yogini_rounds(100.0, TEST_JD, 3, 3);
        assert_eq!(tree.maha_periods.len(), 24);
        for maha in &tree.maha_periods {
            check(maha, 3);
        }
    }

    // 9. `rounds` is clamped to 1..=MAX_YOGINI_ROUNDS, like `levels`.
    #[test]
    fn rounds_are_clamped() {
        assert_eq!(
            compute_yogini_rounds(0.0, TEST_JD, 1, 0).maha_periods.len(),
            8
        );
        assert_eq!(
            compute_yogini_rounds(0.0, TEST_JD, 1, u8::MAX)
                .maha_periods
                .len(),
            8 * usize::from(MAX_YOGINI_ROUNDS)
        );
    }

    // 10. `until_jd` stops at the first whole round that reaches the
    // horizon. Round 1 of the Ashwini vector ends at 2464694.0 and each
    // later round is 13149 days, so round 2 ends 2477843.0 and round 3
    // 2490992.0.
    #[test]
    fn until_stops_at_the_first_round_that_reaches_the_horizon() {
        let len = |until: f64| {
            compute_yogini_until(0.0, TEST_JD, 1, until)
                .maha_periods
                .len()
        };
        assert_eq!(len(TEST_JD - 1000.0), 8, "before birth: one round");
        assert_eq!(len(TEST_JD + 1.0), 8);
        assert_eq!(len(2_464_694.0), 8, "exactly the round end stops there");
        assert_eq!(len(2_464_694.5), 16);
        assert_eq!(len(2_477_843.0), 16);
        assert_eq!(len(2_477_843.5), 24);
        assert_eq!(len(f64::NAN), 8);
        assert_eq!(len(f64::INFINITY), 8 * usize::from(MAX_YOGINI_ROUNDS));
        // A 100-year horizon needs three rounds, and the tree reaches it.
        let tree = compute_yogini_until(0.0, TEST_JD, 1, TEST_JD + 100.0 * 365.25);
        assert_eq!(tree.maha_periods.len(), 24);
        assert!(tree.maha_periods[23].end_jd >= TEST_JD + 100.0 * 365.25);
        assert!(tree.maha_periods[15].end_jd < TEST_JD + 100.0 * 365.25);
    }

    // 11. Round one is the previous release's output, bit for bit, at every
    // level. `legacy_tree` transcribes the pre-rounds algorithm (one round,
    // no `round` field) so the comparison has an independent baseline; it
    // returns `Legacy` nodes.
    struct Legacy {
        lord: YoginiLord,
        level: u8,
        start: f64,
        end: f64,
        duration: f64,
        subs: Vec<Legacy>,
    }

    fn legacy_subs(parent_idx: usize, start: f64, dur: f64, level: u8, max: u8) -> Vec<Legacy> {
        let mut out = Vec::new();
        let mut cur = start;
        for i in 0..8 {
            let idx = (parent_idx + i) % 8;
            let lord = YOGINI_SEQUENCE[idx];
            let d = dur * lord.years() / TOTAL_YOGINI_YEARS;
            let subs = if level < max {
                legacy_subs(idx, cur, d, level + 1, max)
            } else {
                Vec::new()
            };
            out.push(Legacy {
                lord,
                level,
                start: cur,
                end: cur + d,
                duration: d,
                subs,
            });
            cur += d;
        }
        out
    }

    fn legacy_tree(lon: f64, birth: f64, levels: u8) -> Vec<Legacy> {
        let levels = levels.clamp(1, 5);
        let nak = Nakshatra::from_longitude(lon);
        let start_idx = (nak.index() as usize + 3) % 8;
        let balance = 1.0 - (lon - nak.start_longitude()) / Nakshatra::SPAN;
        let mut out = Vec::new();
        let mut cur = birth;
        for i in 0..8 {
            let idx = (start_idx + i) % 8;
            let lord = YOGINI_SEQUENCE[idx];
            let d = if i == 0 {
                YOGINI_SEQUENCE[start_idx].years() * DASHA_YEAR_DAYS * balance
            } else {
                lord.years() * DASHA_YEAR_DAYS
            };
            let subs = if levels > 1 {
                legacy_subs(idx, cur, d, 2, levels)
            } else {
                Vec::new()
            };
            out.push(Legacy {
                lord,
                level: 1,
                start: cur,
                end: cur + d,
                duration: d,
                subs,
            });
            cur += d;
        }
        out
    }

    fn same_as_legacy(new: &YoginiPeriod, old: &Legacy) {
        assert_eq!(new.lord, old.lord);
        assert_eq!(new.level, old.level);
        assert_eq!(new.round, 1);
        assert_eq!(new.start_jd.to_bits(), old.start.to_bits());
        assert_eq!(new.end_jd.to_bits(), old.end.to_bits());
        assert_eq!(new.duration_days.to_bits(), old.duration.to_bits());
        assert_eq!(new.sub_periods.len(), old.subs.len());
        for (x, y) in new.sub_periods.iter().zip(&old.subs) {
            same_as_legacy(x, y);
        }
    }

    #[test]
    fn round_one_is_bit_identical_to_the_single_round_algorithm() {
        for (lon, birth, levels) in [
            (0.0, TEST_JD, 1),
            (213.7, 2_460_000.123_456, 4),
            (359.9, 2_415_020.75, 3),
            (87.123_456, 2_488_069.5, 5),
        ] {
            let old = legacy_tree(lon, birth, levels);
            for tree in [
                compute_yogini(lon, birth, levels),
                compute_yogini_rounds(lon, birth, levels, 3),
                compute_yogini_until(lon, birth, levels, birth + 100.0 * 365.25),
            ] {
                for (a, b) in tree.maha_periods.iter().zip(&old) {
                    same_as_legacy(a, b);
                }
            }
        }
    }

    // 12. The serde default for output written before `round` existed.
    #[test]
    fn serde_default_round_is_the_first() {
        assert_eq!(first_round(), 1);
    }
}
