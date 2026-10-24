// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Periods and antardashas of the rashi dashas.
//!
//! The chara, sthira and niryana shoola dashas all return twelve rashi
//! periods, each divided into twelve antardashas by the same rule. That rule
//! and the types that carry it live here. See [`super::rashi_strength`] for
//! the numbering and grading conventions used in these module docs.
//!
//! # The antardasha rule
//!
//! **Length.** BPHS, the chapter on working out antardashas (ch. 51 in the
//! recension this crate follows; another recension numbers it 53), sloka 5:
//! *krtva 'rkadha rashi-dasham rashers bhuktim kramad vadet*, the rashi's
//! dasha is divided into twelve and each antardasha is a twelfth. Grade (a).
//! The pratyantars repeat the division (same sloka), which this crate does
//! not compute. [`AntarLength::ProportionalToSthiraYears`] is the other
//! reading in circulation (each antar rashi's own 7, 8 or 9 years over 96);
//! the text gives that proportional rule only for the kalachakra dasha, so it
//! is a convention for these dashas, grade (c).
//!
//! **Where the twelve start.** Sloka 6, *adya-saptamayor madhye yo rashir
//! balavams tatah*: from the dasha rashi or the seventh from it, whichever is
//! stronger ([`AntarStart::StrongerOfRashiAndSeventh`], the default). Grade
//! (b): "stronger" is left to the strength section, so the comparison is a
//! [`Strength`](super::rashi_strength::Strength). The chapter before it
//! (ch. 50 in the same recension, slokas 90 and 91) instead starts the
//! antars at the rashi the dasha rashi's lord occupies
//! ([`AntarStart::LordOccupied`]); the two chapters are in conflict and
//! neither says which governs, so that is a parameter. The worked examples
//! of ch. 51 start every antar series at the dasha rashi itself
//! ([`AntarStart::RashiItself`]).
//!
//! **Order from the start.** Slokas 6 to 9 give a shape by the modality of
//! the dasha rashi and a direction by its parity:
//!
//! - a movable dasha rashi: the twelve in unbroken order (*anujjhita-
//!   margah*);
//! - a fixed dasha rashi: every sixth rashi (*shashtha-shashthadikah*), five
//!   signs on, which visits all twelve;
//! - a dual dasha rashi: the kendras from the start, then the kendras from
//!   its fifth, then those from its ninth (*kantakaj jneya lagna-pancha-
//!   bhagyatah*);
//! - in all three, forward for an odd dasha rashi and in reverse for an even
//!   one (*oje dasha-ashraye ganyah kramad utkramatah same*), "reverse"
//!   meaning the whole figure mirrored: the fifth and ninth are then counted
//!   backward as well.
//!
//! Grade (a) for the shapes and the direction; the sloka numbers 7 to 9
//! belong to the verses as the recension prints them, and the worked
//! examples of that chapter (an Aquarius and a Pisces dasha) reproduce
//! exactly as the module tests show. The same three shapes appear in the
//! critical edition of the Upadesha Sutras at 2.4.8 to 2.4.10 (*prathame
//! prak-pratyaktvam*, *dvitiye ravitah*, *prithak-kramena trtiye
//! catushtayadi*), where the commentary extends them to every rashi dasha
//! of the chara-paryaya kind; that is why this crate applies them to the
//! sthira and shoola dashas too (grade (c) for those two).
//!
//! [`AntarOrder::Consecutive`] is the other reading: the commentary on
//! 2.3.1 and 2.3.2 reports commentators who take every antardasha series
//! as an unbroken run of twelve, forward from an odd rashi and in reverse
//! from an even one, with no sixth-sixth or kendra grouping.

use serde::{Deserialize, Serialize};

use super::chara::{SIGN_NAMES, lord_sign_for};
use super::rashi_strength::{Modality, NatalPlacements, Strength, is_odd_sign, modality};

/// Dasha year length in days (Julian year), as every dasha here uses.
pub const DASHA_YEAR_DAYS: f64 = 365.25;

/// Sum of the sthira years of the twelve rashis (4 x (7 + 8 + 9)).
const STHIRA_TOTAL_YEARS: f64 = 96.0;

/// Which way a dasha walks the zodiac.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashaDirection {
    /// Aries, Taurus, Gemini, ...
    Zodiacal,
    /// Aries, Pisces, Aquarius, ...
    AntiZodiacal,
}

impl DashaDirection {
    /// +1 or -1 as a step in signs.
    #[must_use]
    pub const fn step(self) -> i8 {
        match self {
            Self::Zodiacal => 1,
            Self::AntiZodiacal => -1,
        }
    }

    /// The direction a zero-based sign's parity gives: forward from an odd
    /// sign, reverse from an even one.
    #[must_use]
    pub const fn by_parity(sign: u8) -> Self {
        if is_odd_sign(sign) {
            Self::Zodiacal
        } else {
            Self::AntiZodiacal
        }
    }
}

/// The sign `steps` places from `sign` in `direction` (zero-based, mod 12).
#[must_use]
pub fn walk(sign: u8, direction: DashaDirection, steps: u8) -> u8 {
    let moved = i16::from(sign) + i16::from(direction.step()) * i16::from(steps);
    moved.rem_euclid(12) as u8
}

/// The sthira years of a zodiac sign: 7 for a movable sign, 8 for a fixed
/// one, 9 for a dual one (Jaimini 2.3.3, *shashi-nanda-pavakah kramad abdah
/// sthira-dashayam*; BPHS 46.168). Grade (a).
#[must_use]
pub const fn sthira_years(sign: u8) -> f64 {
    match modality(sign) {
        Modality::Movable => 7.0,
        Modality::Fixed => 8.0,
        Modality::Dual => 9.0,
    }
}

/// One antardasha of a rashi dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RashiAntar {
    /// Zero-based sign index (0 = Aries).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: String,
    /// Start as Julian Day.
    pub start_jd: f64,
    /// End as Julian Day.
    pub end_jd: f64,
    /// Length in years.
    pub duration_years: f64,
}

/// One mahadasha of a rashi dasha, with its antardashas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RashiPeriod {
    /// Zero-based sign index (0 = Aries).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: String,
    /// Start as Julian Day.
    pub start_jd: f64,
    /// End as Julian Day.
    pub end_jd: f64,
    /// Length in years.
    pub duration_years: f64,
    /// The twelve antardashas, in order, tiling the period exactly.
    pub antar: Vec<RashiAntar>,
}

/// Where a rashi dasha's twelve antardashas start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AntarStart {
    /// The dasha rashi or the seventh from it, whichever is stronger (BPHS
    /// 51.6). Grade (b). The default.
    #[default]
    StrongerOfRashiAndSeventh,
    /// The rashi the dasha rashi's lord occupies (BPHS 50.90 and 50.91).
    /// Grade (b); in conflict with the default. A rashi with two lords uses
    /// the one `dasha::chara` resolves to for its duration.
    LordOccupied,
    /// The dasha rashi itself, as the worked examples of BPHS ch. 51 do.
    RashiItself,
}

/// The order of the twelve antardashas from their start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AntarOrder {
    /// Unbroken for a movable dasha rashi, every sixth for a fixed one,
    /// kendra groups for a dual one; direction by the dasha rashi's parity
    /// (BPHS 51.6 to 51.9). Grade (a). The default.
    #[default]
    ByModality,
    /// An unbroken run of twelve whatever the modality, direction by parity.
    /// A reading the critical edition's commentary on 2.3.1 and 2.3.2
    /// reports. Grade (c).
    Consecutive,
}

/// How long each antardasha is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AntarLength {
    /// A twelfth of the dasha each (BPHS 51.5). Grade (a). The default.
    #[default]
    EqualTwelfths,
    /// Each in proportion to its own rashi's sthira years over 96. A
    /// convention, grade (c); see the module doc.
    ProportionalToSthiraYears,
}

/// The start and order rules for antardashas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AntarRules {
    /// Where the twelve start.
    pub start: AntarStart,
    /// How they run from there.
    pub order: AntarOrder,
}

/// The sign an antardasha series starts at.
#[must_use]
pub fn antar_start_sign(
    dasha_sign: u8,
    rule: AntarStart,
    placements: &NatalPlacements,
    strength: &dyn Strength,
) -> u8 {
    let dasha_sign = dasha_sign % 12;
    match rule {
        AntarStart::RashiItself => dasha_sign,
        AntarStart::StrongerOfRashiAndSeventh => {
            strength.stronger_rashi(dasha_sign, (dasha_sign + 6) % 12)
        }
        AntarStart::LordOccupied => lord_sign_for(dasha_sign, placements.graha_signs()),
    }
}

/// The twelve rashis of an antardasha series, in order.
///
/// `dasha_sign` fixes the shape (its modality) and the direction (its
/// parity); `start_sign` is where the series begins.
#[must_use]
pub fn antar_sequence(dasha_sign: u8, start_sign: u8, order: AntarOrder) -> [u8; 12] {
    /// Offsets, in signs along the direction of travel, of the twelve
    /// antardashas of a dual rashi: the kendras from the start, from its
    /// fifth, from its ninth.
    const DUAL_OFFSETS: [u8; 12] = [0, 3, 6, 9, 4, 7, 10, 1, 8, 11, 2, 5];

    let direction = DashaDirection::by_parity(dasha_sign);
    let mut out = [0u8; 12];
    for (k, slot) in out.iter_mut().enumerate() {
        let k = k as u8;
        let offset = match (order, modality(dasha_sign)) {
            (AntarOrder::Consecutive, _) | (AntarOrder::ByModality, Modality::Movable) => k,
            (AntarOrder::ByModality, Modality::Fixed) => (k * 5) % 12,
            (AntarOrder::ByModality, Modality::Dual) => DUAL_OFFSETS[usize::from(k)],
        };
        *slot = walk(start_sign % 12, direction, offset);
    }
    out
}

/// Build the antardashas of one period.
///
/// The antars tile `[start_jd, end_jd]` exactly: the first starts at
/// `start_jd`, the last ends at `end_jd`, and each shares its bound with the
/// next. `maha_years` is the period's length in years.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_antars(
    dasha_sign: u8,
    start_jd: f64,
    end_jd: f64,
    maha_years: f64,
    rules: AntarRules,
    length: AntarLength,
    placements: &NatalPlacements,
    strength: &dyn Strength,
) -> Vec<RashiAntar> {
    let start_sign = antar_start_sign(dasha_sign, rules.start, placements, strength);
    let signs = antar_sequence(dasha_sign, start_sign, rules.order);
    let weight = |sign: u8| match length {
        AntarLength::EqualTwelfths => 1.0,
        AntarLength::ProportionalToSthiraYears => sthira_years(sign),
    };
    let total: f64 = signs.iter().map(|&s| weight(s)).sum();
    let span = end_jd - start_jd;

    let mut out = Vec::with_capacity(12);
    let mut cumulative = 0.0;
    let mut previous_end = start_jd;
    for (k, &sign) in signs.iter().enumerate() {
        let w = weight(sign);
        cumulative += w;
        let this_end = if k == 11 {
            end_jd
        } else {
            start_jd + span * (cumulative / total)
        };
        out.push(RashiAntar {
            sign_index: sign,
            sign_name: SIGN_NAMES[usize::from(sign)].to_string(),
            start_jd: previous_end,
            end_jd: this_end,
            duration_years: maha_years * w / total,
        });
        previous_end = this_end;
    }
    out
}

// Keep the constant honest: it is the sum over twelve signs.
const _: () = {
    let mut sum = 0.0;
    let mut s = 0u8;
    while s < 12 {
        sum += sthira_years(s);
        s += 1;
    }
    assert!(sum == STHIRA_TOTAL_YEARS);
};

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::chara::GrahaSigns;
    use crate::dasha::rashi_strength::JaiminiStrength;
    use crate::graha::Graha;

    /// A strength that always prefers the first argument, so a series starts
    /// at the dasha rashi itself.
    struct FirstWins;
    impl Strength for FirstWins {
        fn stronger_rashi(&self, a: u8, _b: u8) -> u8 {
            a
        }
        fn stronger_graha(&self, a: Graha, _b: Graha) -> Graha {
            a
        }
    }
    /// A strength that always prefers the second argument (the seventh).
    struct SecondWins;
    impl Strength for SecondWins {
        fn stronger_rashi(&self, _a: u8, b: u8) -> u8 {
            b
        }
        fn stronger_graha(&self, _a: Graha, b: Graha) -> Graha {
            b
        }
    }

    fn aries_chart() -> NatalPlacements {
        NatalPlacements::from_signs(
            0,
            GrahaSigns {
                sun: 0,
                moon: 0,
                mars: 0,
                mercury: 0,
                jupiter: 0,
                venus: 0,
                saturn: 0,
                rahu: 0,
            },
        )
    }

    // 1. BPHS ch. 51's own worked example, fixed rashi: "Aquarius is a fixed
    // and odd rasi. Therefore, starting from Aquarius there will be in
    // onwards the Antardasa of every sixth Rasi. Thus the first Antardasa
    // will be of Aquarius, then of Cancer, the sixth rasi from it, and later
    // of Sagittarius, the sixth rasi from Cancer, and so on", tabulated:
    // Aquarius Cancer Sagittarius Taurus Libra Pisces Leo Capricorn Gemini
    // Scorpio Aries Virgo. Zero-based: 10 3 8 1 6 11 4 9 2 7 0 5.
    #[test]
    fn fixed_odd_dasha_runs_every_sixth_rashi() {
        assert_eq!(
            antar_sequence(10, 10, AntarOrder::ByModality),
            [10, 3, 8, 1, 6, 11, 4, 9, 2, 7, 0, 5]
        );
    }

    // 2. The same chapter's dual, even example: Pisces. "In its Dasa the
    // first Antardasa will be of Pisces itself. Then in the reverse order
    // there will be Antardasas of its kendra rasis; namely, Sagittarius,
    // Virgo and Gemini. After that ... the kendra rasis from the 5th rasi
    // from it (Scorpio), namely Scorpio, Leo, Taurus and Aquarius. Lastly
    // ... the kendra rasis of the 9th from Pisces (Cancer) namely, Cancer,
    // Aries, Capricorn and Libra." Zero-based: 11 | 8 5 2 | 7 4 1 10 | 3 0 9 6.
    #[test]
    fn dual_even_dasha_runs_kendra_groups_in_reverse() {
        assert_eq!(
            antar_sequence(11, 11, AntarOrder::ByModality),
            [11, 8, 5, 2, 7, 4, 1, 10, 3, 0, 9, 6]
        );
    }

    // 3. A movable odd dasha (Aries): "in onwards order as Aries is
    // Moveable and odd sign".
    #[test]
    fn movable_odd_dasha_runs_forward() {
        assert_eq!(
            antar_sequence(0, 0, AntarOrder::ByModality),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        );
        // Movable even (Cancer): the same run, reversed.
        assert_eq!(
            antar_sequence(3, 3, AntarOrder::ByModality),
            [3, 2, 1, 0, 11, 10, 9, 8, 7, 6, 5, 4]
        );
    }

    // 4. Fixed even in reverse: from Taurus(1), sign - 5 each step, which is
    // "first, eighth, third, tenth" counted forward: 1, 8, 3, 10, 5, 0, 7,
    // 2, 9, 4, 11, 6 (zero-based 1, 8(=1+7), 3(=1+2), 10, 5, 0, 7, 2, 9, 4,
    // 11, 6).
    #[test]
    fn fixed_even_dasha_is_the_mirror() {
        assert_eq!(
            antar_sequence(1, 1, AntarOrder::ByModality),
            [1, 8, 3, 10, 5, 0, 7, 2, 9, 4, 11, 6]
        );
    }

    // 5. Every series, from every start, in both orders, names all twelve
    // rashis once and begins at its start.
    #[test]
    fn every_series_is_a_permutation_beginning_at_its_start() {
        for dasha in 0..12u8 {
            for start in [dasha, (dasha + 6) % 12] {
                for order in [AntarOrder::ByModality, AntarOrder::Consecutive] {
                    let seq = antar_sequence(dasha, start, order);
                    assert_eq!(seq[0], start);
                    let mut seen = [false; 12];
                    for s in seq {
                        assert!(
                            !seen[usize::from(s)],
                            "dasha {dasha} start {start}: {seq:?}"
                        );
                        seen[usize::from(s)] = true;
                    }
                }
            }
        }
    }

    // 6. The seventh as the start (the stronger of the two): Pisces with a
    // Virgo start (5), reverse. Kendras from Virgo in reverse: 5 2 11 8;
    // fifth from Virgo counted backward is Taurus(1): 1 10 7 4; ninth
    // backward is Capricorn(9): 9 6 3 0.
    #[test]
    fn series_may_start_at_the_seventh() {
        assert_eq!(
            antar_sequence(11, 5, AntarOrder::ByModality),
            [5, 2, 11, 8, 1, 10, 7, 4, 9, 6, 3, 0]
        );
    }

    // 7. The consecutive reading ignores modality: a fixed odd Aquarius is
    // an unbroken forward run.
    #[test]
    fn consecutive_reading_is_unbroken() {
        assert_eq!(
            antar_sequence(10, 10, AntarOrder::Consecutive),
            [10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    // 8. Start rules. FirstWins keeps the dasha rashi, SecondWins moves to
    // the seventh; LordOccupied goes to the sign the lord stands in. Aries
    // chart: Mars (Aries's lord) stands in Aries, so Aries itself; for
    // Taurus the lord Venus stands in Aries.
    #[test]
    fn antar_start_rules() {
        let p = aries_chart();
        assert_eq!(
            antar_start_sign(2, AntarStart::StrongerOfRashiAndSeventh, &p, &FirstWins),
            2
        );
        assert_eq!(
            antar_start_sign(2, AntarStart::StrongerOfRashiAndSeventh, &p, &SecondWins),
            8
        );
        assert_eq!(
            antar_start_sign(2, AntarStart::RashiItself, &p, &SecondWins),
            2
        );
        assert_eq!(
            antar_start_sign(0, AntarStart::LordOccupied, &p, &FirstWins),
            0
        );
        assert_eq!(
            antar_start_sign(1, AntarStart::LordOccupied, &p, &FirstWins),
            0
        );
    }

    // 9. Antars tile their dasha exactly; equal twelfths are each a twelfth.
    // An 8-year dasha from JD 2451545.0: 2922 days, twelfths of 243.5 days,
    // all exact in binary.
    #[test]
    fn equal_antars_tile_the_period() {
        let p = aries_chart();
        let start = 2_451_545.0;
        let end = start + 8.0 * DASHA_YEAR_DAYS;
        let antars = build_antars(
            10,
            start,
            end,
            8.0,
            AntarRules {
                start: AntarStart::RashiItself,
                order: AntarOrder::ByModality,
            },
            AntarLength::EqualTwelfths,
            &p,
            &FirstWins,
        );
        assert_eq!(antars.len(), 12);
        assert_eq!(antars[0].start_jd, start);
        assert_eq!(antars[11].end_jd, end);
        for (k, a) in antars.iter().enumerate() {
            assert_eq!(a.duration_years, 8.0 / 12.0);
            assert_eq!(a.end_jd - a.start_jd, 243.5, "antar {k}");
            if k > 0 {
                assert_eq!(a.start_jd, antars[k - 1].end_jd);
            }
        }
        assert_eq!(antars[0].sign_name, "Aquarius");
        assert_eq!(antars[1].sign_name, "Cancer");
    }

    // 10. Proportional lengths: weight = the antar rashi's own sthira years.
    // A 9-year Pisces dasha (Pisces is dual: 9), RashiItself, ByModality:
    // 11 8 5 2 7 4 1 10 3 0 9 6 with weights 9 9 9 9 8 8 8 8 7 7 7 7 (96).
    // The first antar is 9 x 9 / 96 = 0.84375 years; the fifth is
    // 9 x 8 / 96 = 0.75; the last is 9 x 7 / 96 = 0.65625.
    #[test]
    fn proportional_antars_follow_the_rashis_own_years() {
        let p = aries_chart();
        let start = 2_451_545.0;
        let end = start + 9.0 * DASHA_YEAR_DAYS;
        let antars = build_antars(
            11,
            start,
            end,
            9.0,
            AntarRules {
                start: AntarStart::RashiItself,
                order: AntarOrder::ByModality,
            },
            AntarLength::ProportionalToSthiraYears,
            &p,
            &FirstWins,
        );
        assert_eq!(antars[0].duration_years, 0.84375);
        assert_eq!(antars[4].duration_years, 0.75);
        assert_eq!(antars[11].duration_years, 0.65625);
        let total: f64 = antars.iter().map(|a| a.duration_years).sum();
        assert!((total - 9.0).abs() < 1e-12);
        assert_eq!(antars[11].end_jd, end);
        for w in antars.windows(2) {
            assert_eq!(w[0].end_jd, w[1].start_jd);
        }
    }

    // 11. With the real Jaimini comparison the seventh can win: a chart with
    // every graha in Libra(6) makes Libra the stronger of Aries and Libra,
    // so an Aries dasha starts at Libra.
    #[test]
    fn jaimini_strength_can_move_the_start_to_the_seventh() {
        let p = NatalPlacements::from_signs(
            0,
            GrahaSigns {
                sun: 6,
                moon: 6,
                mars: 6,
                mercury: 6,
                jupiter: 6,
                venus: 6,
                saturn: 6,
                rahu: 6,
            },
        );
        let s = JaiminiStrength::new(&p);
        assert_eq!(
            antar_start_sign(0, AntarStart::StrongerOfRashiAndSeventh, &p, &s),
            6
        );
        assert_eq!(
            antar_start_sign(6, AntarStart::StrongerOfRashiAndSeventh, &p, &s),
            6
        );
    }
}
