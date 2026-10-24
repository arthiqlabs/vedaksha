// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Niryana Shoola Dasha (Jaimini) — the rashi dasha read for the time of death.
//!
//! *Niryana* is departure, death; *shoola* is the trident, the trikona
//! (trine) groups the dasha is built from. The numbering and grading
//! conventions are those of [`super::rashi_strength`]; the antardasha rule is
//! that of [`super::rashi_antar`].
//!
//! # What the texts give
//!
//! The Jaimini passages that define the dasha:
//!
//! - **2.4.15**, *sthula-adarsha-vaishamya-ashrayo manduka-trikutah*: the
//!   *manduka* (frog) or *trikuta* dasha takes its years from the sthira
//!   dasha, starts from the stronger of the lagna (*sthula*, the first) and
//!   the seventh (*adarsha*, the facing), goes by trikona groups in a
//!   four-fold round, and has its direction from odd and even
//!   (*vaishamya-ashraya*). The critical edition's commentary spells the
//!   shape: the lagna, fifth and ninth, then the second, sixth and tenth,
//!   then the third, seventh and eleventh, then the fourth, eighth and
//!   twelfth. Grade (a) for years, anchor and groups; (b) for the direction,
//!   which the commentary derives from "odd and even" without spelling the
//!   even case, so an even anchor mirrors the whole figure (the fifth and
//!   ninth are then counted backward).
//! - **2.4.16**, *niryana-labha-adih shoola-dasha phale*: the shoola dasha is
//!   this same figure ("like the manduka-trikuta", in the four-fold round of
//!   trikona groups) made for the result of death, and begins, per the
//!   commentary, from the *niryana* place (the sixth, eighth and so on of the
//!   Rudra-shoola passages) or its seventh (*labha*), whichever is stronger.
//!   That anchor differs from 2.3.18's, so the texts disagree about where
//!   the dasha of death starts; 2.3.18 names the native and is the default,
//!   and 2.4.16's anchor is [`ShoolaAnchor::Explicit`] with the rashi the
//!   caller reads it as.
//! - **2.3.18**, *pitri-labha-prathama-prani-adih shoola-dasha niryane*: for
//!   death, the shoola dasha begins from the stronger (*prani*) of the first
//!   and the seventh. This is the native's anchor, and the default here.
//!   Grade (a) for the words; (b) for the comparison, which is a
//!   [`Strength`](super::rashi_strength::Strength). The commentary records
//!   a second reading, that this names a trikona's dasha *within* the
//!   sthira or chara dasha run from that anchor rather than a dasha of its
//!   own; the first is implemented.
//! - **2.3.19 to 2.3.23** give the same dasha for relatives, each from the
//!   stronger of the *n*th rashi from the lagna and the *n*th from the
//!   seventh: the ninth for the father, the fourth for the mother, the third
//!   for a brother, the fifth for a sister or a son, the eleventh for an
//!   elder brother ([`ShoolaRelative`]). Grade (b): the commentary spells the
//!   "from the lagna and from the seventh" pair for the father, mother and
//!   brother, and not for the other two, which the same pattern implies.
//! - **2.2.1 to 2.2.11** instead name the parents' karakas (the stronger of
//!   Surya and Shukra for the father, of Chandra and Mangala for the mother,
//!   Guru for a wife, the chara karaka for others) and have the death in the
//!   trikona dashas of the rashi that karaka occupies. That is
//!   [`ShoolaAnchor::Explicit`] with the karaka's rashi, which the caller
//!   picks (the stronger-of-two choice is [`Strength::stronger_graha`]).
//!
//! BPHS's own *shoola dasha* (ch. 46 in the recension this crate follows,
//! slokas 181 and 182, *nirnayasya vicharartham kaishchic chhoola-dasha
//! smrita*) is a different figure: from the stronger of the eighth from the
//! lagna and the eighth from the seventh (the second and eighth rashis, in
//! the recension's own English), forward from an odd rashi and in reverse
//! from an even one, the twelve rashis in unbroken order, years as in sthira
//! ([`ShoolaAnchor::SecondOrEighth`] with [`ShoolaOrder::Consecutive`]). Grade
//! (a) for BPHS's statement. It is not the default because the dasha named
//! for *niryana* in the sutras is the one above, and no text says the two
//! are one.
//!
//! **Years.** Sthira years, 7, 8 or 9 by modality (2.3.3; the manduka and
//! shoola passages draw on them, and BPHS 46.182 says so in words). There is
//! no lord-distance count in these passages. Grade (a). Twelve periods make
//! 96 years; the dasha does not repeat.
//!
//! **Antardashas.** The defaults of [`super::rashi_antar`]; grade (c), since
//! the passages above are silent on the antars of this dasha and the BPHS
//! antar chapter names the chara, sthira and trikona dashas only.
//!
//! # The marana (killing) rashi
//!
//! The sutras say which rashis of a dasha bring death: 2.1.34 and 2.1.35,
//! *tatra-rksha-visheshah, papa-madhye papa-kone ripu-rogayoh pape va*. The
//! commentary reads three marks, each of which makes a rashi's dasha one of
//! death: the rashi lies between two malefic-occupied rashis (*papa-
//! madhya*); it is a trikona (first, fifth or ninth) from the lagna or the
//! atmakaraka's rashi and a malefic stands in it (*papa-kone*); it is the
//! twelfth or eighth (*ripu*, *roga*) from the lagna or the atmakaraka and a
//! malefic stands in it. [`cruel_rashi_marks`] reports which marks a rashi
//! has and [`marana_rashi`] picks, among rashis a caller names (the three of
//! a trikona group, or the four of a sthira *khanda*), the marked one, the
//! stronger if several. Grade (b): the sutra words are terse and the marks
//! are the commentary's; the tie rule is not in the text. The malefics are
//! those the commentary lists, Surya, Mangala, Shani and Rahu
//! ([`MARANA_MALEFICS`]), with Guru, Ketu, Shukra and Budha benefic; the
//! caller may pass another set.
//!
//! Not implemented: the elders' verse quoted in the commentary (a shoola
//! wholly malefic kills in its first rashi, a mixed one in the middle, a
//! wholly benefic one in the last), which is commentary and not sutra;
//! 2.1.36 and 2.1.37 (the lords of the twelfth and eighth under the
//! waning-Moon or Shukra aspect alone, and the ninth-part aspected by the
//! lord of the tenth or eighth), which need aspects.

use serde::{Deserialize, Serialize};

use super::chara::SIGN_NAMES;
use super::rashi_antar::{
    AntarLength, AntarRules, DASHA_YEAR_DAYS, DashaDirection, RashiPeriod, build_antars,
    sthira_years, walk,
};
use super::rashi_strength::{NatalPlacements, Strength};
use crate::graha::Graha;

/// A relative whose death the shoola dasha is read for (2.3.19 to 2.3.23).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShoolaRelative {
    /// The father: the ninth (2.3.19).
    Father,
    /// The mother: the fourth (2.3.20).
    Mother,
    /// A brother: the third (2.3.21).
    Brother,
    /// A sister or a son: the fifth (2.3.22).
    SisterOrSon,
    /// An elder brother: the eleventh (2.3.23).
    ElderBrother,
}

impl ShoolaRelative {
    /// The house counted from the lagna and from the seventh.
    #[must_use]
    pub const fn house(self) -> u8 {
        match self {
            Self::Father => 9,
            Self::Mother => 4,
            Self::Brother => 3,
            Self::SisterOrSon => 5,
            Self::ElderBrother => 11,
        }
    }
}

/// Where the shoola dasha starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShoolaAnchor {
    /// The stronger of the lagna and the seventh (2.3.18). The native's
    /// death. The default.
    #[default]
    LagnaOrSeventh,
    /// The stronger of the nth rashi from the lagna and the nth from the
    /// seventh, for a relative (2.3.19 to 2.3.23).
    Relative(ShoolaRelative),
    /// The stronger of the eighth from the lagna and the eighth from the
    /// seventh, i.e. the eighth and second rashis (BPHS 46.181).
    SecondOrEighth,
    /// A rashi the caller names, zero-based: a karaka's rashi (2.2.1 to
    /// 2.2.11), the maraka rashi of 2.4.16, or any other reading.
    Explicit(u8),
}

/// How the twelve periods follow one another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShoolaOrder {
    /// Trikona groups: anchor, fifth, ninth, then the second, sixth and
    /// tenth, and so on (2.4.15). The default.
    #[default]
    TrikonaGroups,
    /// Unbroken, one rashi after another (BPHS 46.182).
    Consecutive,
}

/// Everything [`compute_niryana_shoola`] leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ShoolaOptions {
    /// Where the dasha starts.
    pub anchor: ShoolaAnchor,
    /// The order of the twelve periods.
    pub order: ShoolaOrder,
    /// Start and order of each period's antardashas.
    pub antar: AntarRules,
    /// Length of each antardasha.
    pub antar_length: AntarLength,
}

/// A computed niryana shoola dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoolaDasha {
    /// Zero-based sign the first period is of.
    pub anchor_sign: u8,
    /// The direction the dasha walks, from the anchor's parity.
    pub direction: DashaDirection,
    /// The twelve periods, in order, with their antardashas.
    pub periods: Vec<RashiPeriod>,
}

/// The offsets, in signs along the direction of travel, of the twelve
/// periods of the trikona-groups order.
const TRIKONA_OFFSETS: [u8; 12] = [0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11];

/// The sign a shoola anchor names.
fn anchor_sign(anchor: ShoolaAnchor, placements: &NatalPlacements, strength: &dyn Strength) -> u8 {
    let lagna = placements.lagna_sign();
    let seventh = walk(lagna, DashaDirection::Zodiacal, 6);
    let stronger_of_houses = |from_lagna: u8, from_seventh: u8| {
        strength.stronger_rashi(
            walk(lagna, DashaDirection::Zodiacal, from_lagna),
            walk(seventh, DashaDirection::Zodiacal, from_seventh),
        )
    };
    match anchor {
        ShoolaAnchor::LagnaOrSeventh => strength.stronger_rashi(lagna, seventh),
        ShoolaAnchor::Relative(r) => stronger_of_houses(r.house() - 1, r.house() - 1),
        ShoolaAnchor::SecondOrEighth => stronger_of_houses(7, 7),
        ShoolaAnchor::Explicit(sign) => sign % 12,
    }
}

/// Compute the niryana shoola dasha: twelve periods of 7, 8 or 9 years from
/// the anchor, each with twelve antardashas.
#[must_use]
pub fn compute_niryana_shoola(
    placements: &NatalPlacements,
    birth_jd: f64,
    options: &ShoolaOptions,
    strength: &dyn Strength,
) -> ShoolaDasha {
    let anchor = anchor_sign(options.anchor, placements, strength);
    let direction = DashaDirection::by_parity(anchor);
    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;
    for k in 0u8..12 {
        let offset = match options.order {
            ShoolaOrder::TrikonaGroups => TRIKONA_OFFSETS[usize::from(k)],
            ShoolaOrder::Consecutive => k,
        };
        let sign = walk(anchor, direction, offset);
        let years = sthira_years(sign);
        let end_jd = current_jd + years * DASHA_YEAR_DAYS;
        periods.push(RashiPeriod {
            sign_index: sign,
            sign_name: SIGN_NAMES[usize::from(sign)].to_string(),
            start_jd: current_jd,
            end_jd,
            duration_years: years,
            antar: build_antars(
                sign,
                current_jd,
                end_jd,
                years,
                options.antar,
                options.antar_length,
                placements,
                strength,
            ),
        });
        current_jd = end_jd;
    }
    ShoolaDasha {
        anchor_sign: anchor,
        direction,
        periods,
    }
}

/// The three rashis of the trikona that starts at `rashi`: itself, its
/// fifth and its ninth.
#[must_use]
pub fn trikona_of(rashi: u8) -> [u8; 3] {
    [
        rashi % 12,
        walk(rashi, DashaDirection::Zodiacal, 4),
        walk(rashi, DashaDirection::Zodiacal, 8),
    ]
}

/// The malefics of 2.1.34 and 2.1.35's commentary: Surya, Mangala, Shani
/// and Rahu.
pub const MARANA_MALEFICS: [Graha; 4] = [Graha::Sun, Graha::Mars, Graha::Saturn, Graha::Rahu];

/// Which of the sutra's marks a rashi carries (2.1.34 and 2.1.35).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CruelRashiMarks {
    /// *papa-madhya*: the rashi before it and the rashi after it each hold a
    /// malefic.
    pub between_malefics: bool,
    /// *papa-kone*: a trikona (first, fifth, ninth) from the lagna or the
    /// atmakaraka's rashi, holding a malefic.
    pub malefic_trikona: bool,
    /// *ripu-rogayoh pape*: the twelfth or eighth from the lagna or the
    /// atmakaraka's rashi, holding a malefic.
    pub malefic_twelfth_or_eighth: bool,
}

impl CruelRashiMarks {
    /// Whether any mark is present.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.between_malefics || self.malefic_trikona || self.malefic_twelfth_or_eighth
    }
}

/// The marks of 2.1.34 and 2.1.35 that `rashi` carries.
#[must_use]
pub fn cruel_rashi_marks(
    rashi: u8,
    placements: &NatalPlacements,
    atmakaraka: Graha,
    malefics: &[Graha],
) -> CruelRashiMarks {
    let rashi = rashi % 12;
    let holds_malefic = |sign: u8| {
        placements
            .occupants_of(sign)
            .iter()
            .any(|g| malefics.contains(g))
    };
    let lagna = placements.lagna_sign();
    let ak_sign = placements.sign_of(atmakaraka);
    // Is `rashi` the `n`th (1-based) from the lagna or from the karaka?
    let nth_from_either = |ns: &[u8]| {
        [lagna, ak_sign].iter().any(|&base| {
            ns.iter()
                .any(|&n| walk(base, DashaDirection::Zodiacal, n - 1) == rashi)
        })
    };
    CruelRashiMarks {
        between_malefics: holds_malefic(walk(rashi, DashaDirection::AntiZodiacal, 1))
            && holds_malefic(walk(rashi, DashaDirection::Zodiacal, 1)),
        malefic_trikona: holds_malefic(rashi) && nth_from_either(&[1, 5, 9]),
        malefic_twelfth_or_eighth: holds_malefic(rashi) && nth_from_either(&[12, 8]),
    }
}

/// Of the rashis in `candidates`, the one whose dasha brings death under
/// 2.1.34 and 2.1.35: the one carrying a mark ([`cruel_rashi_marks`]), the
/// stronger by `strength` if several do, `None` if none does.
///
/// For the trikona of a shoola anchor pass [`trikona_of`]; for a sthira
/// *khanda* (a chara, a sthira and a dual rashi in a row) pass its three.
#[must_use]
pub fn marana_rashi(
    candidates: &[u8],
    placements: &NatalPlacements,
    atmakaraka: Graha,
    malefics: &[Graha],
    strength: &dyn Strength,
) -> Option<u8> {
    candidates
        .iter()
        .copied()
        .filter(|&r| cruel_rashi_marks(r, placements, atmakaraka, malefics).any())
        .reduce(|acc, r| strength.stronger_rashi(acc, r))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::chara::GrahaSigns;
    use crate::dasha::rashi_antar::AntarStart;
    use crate::dasha::rashi_strength::JaiminiStrength;

    const TEST_JD: f64 = 2_451_545.0;

    fn chart(lagna: u8, s: [u8; 8]) -> NatalPlacements {
        NatalPlacements::from_signs(
            lagna,
            GrahaSigns {
                sun: s[0],
                moon: s[1],
                mars: s[2],
                mercury: s[3],
                jupiter: s[4],
                venus: s[5],
                saturn: s[6],
                rahu: s[7],
            },
        )
    }

    fn run(p: &NatalPlacements, options: &ShoolaOptions) -> ShoolaDasha {
        compute_niryana_shoola(p, TEST_JD, options, &JaiminiStrength::new(p))
    }

    fn signs_of(d: &ShoolaDasha) -> Vec<u8> {
        d.periods.iter().map(|p| p.sign_index).collect()
    }

    fn explicit(anchor: u8, order: ShoolaOrder) -> ShoolaOptions {
        ShoolaOptions {
            anchor: ShoolaAnchor::Explicit(anchor),
            order,
            ..ShoolaOptions::default()
        }
    }

    // 1. 2.4.15: trikona groups with sthira years. From Aries (odd, so
    // forward): Aries, Leo, Sagittarius | Taurus, Virgo, Capricorn |
    // Gemini, Libra, Aquarius | Cancer, Scorpio, Pisces, with years by
    // modality 7 8 9 | 8 9 7 | 9 7 8 | 7 8 9 = 96.
    #[test]
    fn trikona_groups_from_an_odd_anchor() {
        let p = chart(0, [0; 8]);
        let d = run(&p, &explicit(0, ShoolaOrder::TrikonaGroups));
        assert_eq!(signs_of(&d), vec![0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11]);
        let years: Vec<f64> = d.periods.iter().map(|x| x.duration_years).collect();
        assert_eq!(
            years,
            vec![7.0, 8.0, 9.0, 8.0, 9.0, 7.0, 9.0, 7.0, 8.0, 7.0, 8.0, 9.0]
        );
        assert_eq!(years.iter().sum::<f64>(), 96.0);
        assert_eq!(d.direction, DashaDirection::Zodiacal);
    }

    // 2. An even anchor mirrors the figure. From Taurus(1), reverse:
    // 1, 1-4=9, 1-8=5 | 0, 8, 4 | 11, 7, 3 | 10, 6, 2.
    #[test]
    fn trikona_groups_from_an_even_anchor_mirror() {
        let p = chart(0, [0; 8]);
        let d = run(&p, &explicit(1, ShoolaOrder::TrikonaGroups));
        assert_eq!(signs_of(&d), vec![1, 9, 5, 0, 8, 4, 11, 7, 3, 10, 6, 2]);
        assert_eq!(d.direction, DashaDirection::AntiZodiacal);
    }

    // 3. BPHS 46.182: unbroken, forward from odd, reverse from even.
    #[test]
    fn consecutive_order_is_the_bphs_shoola() {
        let p = chart(0, [0; 8]);
        let odd = run(&p, &explicit(2, ShoolaOrder::Consecutive));
        assert_eq!(signs_of(&odd), vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 1]);
        // The recension's own worked example runs from Pisces(11) in
        // reverse: Pisces 9, Aquarius 8, Capricorn 7, Sagittarius 9, ...
        let even = run(&p, &explicit(11, ShoolaOrder::Consecutive));
        assert_eq!(signs_of(&even), vec![11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        let years: Vec<f64> = even.periods.iter().map(|x| x.duration_years).collect();
        assert_eq!(
            years,
            vec![9.0, 8.0, 7.0, 9.0, 8.0, 7.0, 9.0, 8.0, 7.0, 9.0, 8.0, 7.0]
        );
    }

    // 4. Anchors. Lagna Aries(0). Sun and Moon in Sagittarius(8); Mars,
    // Mercury and Jupiter in Gemini(2); Venus in Pisces(11); Saturn and
    // Rahu in Leo(4), so Ketu is in Aquarius(10).
    //
    // Stronger of lagna Aries and seventh Libra: both empty movable signs
    // with no exalted graha; the lords Mars (two companions) and Venus (none)
    // decide: Mars, so Aries(0).
    // Father, the ninth from lagna (Sagittarius, two grahas) against the
    // ninth from the seventh (Gemini, three grahas): Gemini(2).
    // Second or eighth: the eighth from lagna is Scorpio(7), empty, the
    // eighth from the seventh is Taurus(1), empty; both fixed; lords
    // Mars/Ketu (Scorpio) and Venus (Taurus). Scorpio's effective lord is
    // the stronger of Mars (two companions) and Ketu (Aquarius, with no one
    // else): Mars. Mars (two companions) against Venus (none): Scorpio(7).
    #[test]
    fn anchors_pick_the_stronger_of_the_pair() {
        let p = chart(0, [8, 8, 2, 2, 2, 11, 4, 4]);
        let anchor = |a: ShoolaAnchor| {
            run(
                &p,
                &ShoolaOptions {
                    anchor: a,
                    ..ShoolaOptions::default()
                },
            )
            .anchor_sign
        };
        assert_eq!(anchor(ShoolaAnchor::LagnaOrSeventh), 0);
        assert_eq!(anchor(ShoolaAnchor::Relative(ShoolaRelative::Father)), 2);
        assert_eq!(anchor(ShoolaAnchor::SecondOrEighth), 7);
        assert_eq!(anchor(ShoolaAnchor::Explicit(15)), 3, "reduced mod 12");
    }

    // 5. The house each relative reads from (2.3.19 to 2.3.23).
    #[test]
    fn relatives_use_their_houses() {
        assert_eq!(ShoolaRelative::Father.house(), 9);
        assert_eq!(ShoolaRelative::Mother.house(), 4);
        assert_eq!(ShoolaRelative::Brother.house(), 3);
        assert_eq!(ShoolaRelative::SisterOrSon.house(), 5);
        assert_eq!(ShoolaRelative::ElderBrother.house(), 11);
        // For a mother with every graha in Aries: the fourth from the lagna
        // Aries is Cancer, the fourth from the seventh Libra is Capricorn.
        // Both are empty movable signs; the Moon and Saturn are lords, each
        // in Aries with six companions and no exaltation there; the
        // comparison is a total tie on the graha tests, neither lord stands
        // in its rashi, and Capricorn(9, even) against Cancer(3, even) is
        // even/even, so the first argument, Cancer, stands.
        let p = chart(0, [0; 8]);
        let d = run(
            &p,
            &ShoolaOptions {
                anchor: ShoolaAnchor::Relative(ShoolaRelative::Mother),
                ..ShoolaOptions::default()
            },
        );
        assert_eq!(d.anchor_sign, 3);
    }

    // 6. Antars are the shared ones and tile each period.
    #[test]
    fn antars_tile_each_period() {
        let p = chart(0, [8, 8, 2, 2, 2, 11, 4, 4]);
        let d = run(
            &p,
            &ShoolaOptions {
                antar: AntarRules {
                    start: AntarStart::RashiItself,
                    ..AntarRules::default()
                },
                ..ShoolaOptions::default()
            },
        );
        assert_eq!(d.periods[0].start_jd, TEST_JD);
        for (k, x) in d.periods.iter().enumerate() {
            assert_eq!(x.antar.len(), 12);
            assert_eq!(x.antar[0].start_jd, x.start_jd);
            assert_eq!(x.antar[11].end_jd, x.end_jd);
            assert_eq!(x.antar[0].sign_index, x.sign_index);
            if k > 0 {
                assert_eq!(x.start_jd, d.periods[k - 1].end_jd);
            }
        }
    }

    // 7. The marks of 2.1.34 and 2.1.35. Lagna Aries(0); atmakaraka the
    // Moon in Cancer(3). Sun in Leo(4), Mars in Capricorn(9), Saturn in
    // Sagittarius(8), Rahu in Aquarius(10) (Ketu in Leo), Mercury in
    // Taurus, Jupiter in Gemini, Venus in Aries.
    //
    // Counted from the lagna: trikonas Aries, Leo, Sagittarius; twelfth
    // Pisces, eighth Scorpio. From the karaka (Cancer): trikonas Cancer,
    // Scorpio, Pisces; twelfth Gemini, eighth Aquarius.
    //
    // Leo: Sun stands in it and it is the lagna's fifth: malefic trikona.
    // Sagittarius: Saturn stands in it and it is the lagna's ninth: malefic
    // trikona. Capricorn: Saturn is in the rashi before it and Rahu in the
    // rashi after it: between malefics (Mars stands in it but it is none of
    // the trikonas or twelfth/eighth). Aquarius: Rahu stands in it and it is
    // the karaka's eighth: malefic eighth. Aries: a trikona but only Venus,
    // not a malefic, stands there. Pisces: the lagna's twelfth but empty.
    #[test]
    fn the_three_marks() {
        let p = chart(0, [4, 3, 9, 1, 2, 0, 8, 10]);
        let marks = |r| cruel_rashi_marks(r, &p, Graha::Moon, &MARANA_MALEFICS);
        let m = marks(4);
        assert!(m.malefic_trikona && !m.between_malefics && !m.malefic_twelfth_or_eighth);
        let m = marks(8);
        assert!(m.malefic_trikona && !m.between_malefics && !m.malefic_twelfth_or_eighth);
        let m = marks(9);
        assert!(m.between_malefics && !m.malefic_trikona && !m.malefic_twelfth_or_eighth);
        let m = marks(10);
        assert!(m.malefic_twelfth_or_eighth && !m.malefic_trikona && !m.between_malefics);
        assert!(!marks(0).any(), "Venus is no malefic");
        assert!(!marks(11).any(), "an empty twelfth is not marked");
    }

    // 8. Picking the marana rashi. Same chart. Of [Leo, Capricorn,
    // Aquarius] all three are marked. Leo holds Sun and Ketu (two grahas),
    // Capricorn and Aquarius one each, so Leo is the strongest. Of
    // [Capricorn, Aquarius] both hold one graha; Mars is exalted in
    // Capricorn, Rahu is exalted in Taurus and not in Aquarius, so
    // Capricorn. Of [Aries, Pisces] neither is marked: None.
    #[test]
    fn marana_rashi_picks_the_stronger_marked_rashi() {
        let p = chart(0, [4, 3, 9, 1, 2, 0, 8, 10]);
        let s = JaiminiStrength::new(&p);
        let pick = |c: &[u8]| marana_rashi(c, &p, Graha::Moon, &MARANA_MALEFICS, &s);
        assert_eq!(pick(&[4, 9, 10]), Some(4));
        assert_eq!(
            pick(&[10, 9, 4]),
            Some(4),
            "order of the candidates is immaterial"
        );
        assert_eq!(pick(&[9, 10]), Some(9));
        assert_eq!(pick(&[10, 9]), Some(9));
        assert_eq!(pick(&[0, 11]), None);
        assert_eq!(pick(&[]), None);
        // A trikona group handed over whole.
        assert_eq!(trikona_of(4), [4, 8, 0]);
        assert_eq!(pick(&trikona_of(4)), Some(4));
    }

    // 9. A caller-supplied malefic set changes the marks: with the Moon
    // counted a malefic, the Moon's own rashi Cancer(3) is the karaka's
    // first and holds a malefic.
    #[test]
    fn the_malefic_set_is_a_parameter() {
        let p = chart(0, [4, 3, 9, 1, 2, 0, 8, 10]);
        let with_moon = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Saturn,
            Graha::Rahu,
        ];
        assert!(!cruel_rashi_marks(3, &p, Graha::Moon, &MARANA_MALEFICS).any());
        assert!(cruel_rashi_marks(3, &p, Graha::Moon, &with_moon).malefic_trikona);
    }
}
