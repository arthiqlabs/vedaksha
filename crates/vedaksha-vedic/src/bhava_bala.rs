// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Bhava bala — the strength of each of the twelve bhavas.
//!
//! The strength of a bhava is the sum of five terms, each in virupas (60
//! virupas make one rupa), kept separate on the result:
//!
//! 1. [`BhavaBala::adhipati`] — the shadbala pinda of the lord of the rashi
//!    holding the bhava madhya;
//! 2. [`BhavaBala::dig`] — the directional strength of the bhava;
//! 3. [`BhavaBala::drishti`] — the net aspect on the bhava madhya (signed);
//! 4. [`BhavaBala::occupants`] — one rupa added or taken away for the grahas
//!    standing in the bhava;
//! 5. [`BhavaBala::udaya`] — a quarter rupa for the bhava's rashi by the time
//!    of birth.
//!
//! # Source
//!
//! BPHS chapter 27 (the *spashta bala* chapter), verses 26 to 31. The Sanskrit
//! was read at the verses in two recensions that agree on the dig rule, the
//! aspect rule, the occupants and the classes of birth (only the first states
//! the amount in 27.31). The verse numbers below are those of the recension
//! that numbers this the 27th chapter and puts the aspect chapter before it
//! (chapter 26); in the other recension (the Uttarakhanda) the same rules sit
//! in the 6th chapter of that part, in a different order. Chapter 28 (ishta
//! and kashta) is not bhava bala, and 27.32-33 are the shadbala requirements,
//! not bhava bala.
//! Grades follow the convention of [`crate::drishti`]: (a) read at the verse,
//! editions agree; (b) read at the verse in one recension only; (c) a choice
//! the verse leaves open, made and stated here.
//!
//! * **27.26-28, the dig of a bhava (grade a).** The text gives a continuous
//!   rule. From the bhava's longitude take the "powerless" point of its
//!   rashi, and divide by 3 the arc between them, taken the short way
//!   (the verse: if the remainder is over six signs, take it from twelve):
//!   * Gemini, Virgo, Libra, Aquarius and the first half of Sagittarius
//!     (`kanya-yugma-tula-kumbha-capadyardha`): the 7th bhava;
//!   * Aries, Taurus, Leo, the first half of Capricorn and the second half of
//!     Sagittarius (`go-aja-simha-mrgadyardha-capantyardha`): the 4th;
//!   * Cancer and Scorpio (`karka-vrishchikato lagnam`): the lagna;
//!   * the second half of Capricorn and Pisces (`mrgantyardha-jhasha`): the
//!     10th.
//!
//!   The result runs from 0 at the powerless point to 60 at the point
//!   opposite it. A second, "stepped" reading is in circulation that gives
//!   only 0, 10, 20, 30 and 40 virupas. No verse or classical commentary
//!   consulted states it; the verse says divide the arc by 3, and the stepped
//!   values come from a modern worked example. It is not implemented.
//! * **27.28-29, the aspect (grade a for the rule, c for one detail).** To the
//!   dig is added a quarter of the aspect ([`crate::drishti::sputa_drishti_graha`],
//!   BPHS 26.6-26.12) of each benefic on the bhava madhya (`sad-drishti-pada-yuk`)
//!   and a quarter of each malefic's is taken away (`papa-drishti-pada-
//!   vivarjitam`); then the aspect of Mercury and of Jupiter is added (`jnejya-
//!   drishti-yutam`). The verse does not say whether Mercury's and Jupiter's
//!   quarter is also taken in the first sum; read as written, they are in both,
//!   so a benefic Jupiter contributes 1.25 times his aspect and a malefic
//!   Mercury 0.75 times (grade c; no classical source consulted reads the
//!   second clause as replacing the first). The classification of the other
//!   grahas, and of Mercury, as benefic or malefic is not in these verses:
//!   it is the caller's `shubha` reading. The verse does not make Mercury
//!   benefic "whatever his company": it adds his aspect in full without
//!   condition, and leaves his quarter to the caller's classification. The
//!   nodes cast no aspect here: the aspect values of chapter 26 and the
//!   benefic and malefic quarters of these verses are given for the seven
//!   grahas, Sun to Saturn, and none for Rahu or Ketu.
//! * **27.29, the lord (grade a).** The bhava's strength is joined with that of
//!   its own lord (`svasvami-bala`): the pinda of the lord of the rashi
//!   holding the madhya. The lords are those of BPHS chapter 4's description
//!   of each rashi (Aries and Scorpio Mars, Taurus and Libra Venus, Gemini
//!   and Virgo Mercury, Cancer Moon, Leo Sun, Sagittarius and Pisces Jupiter,
//!   Capricorn and Aquarius Saturn); the nodes are not lords here.
//! * **27.30, the occupants (grade a for the amounts and for what "joined"
//!   means, c for the count).** A bhava joined with Mercury or Jupiter has one
//!   rupa added, one joined with Saturn, Mars or the Sun has one rupa taken
//!   away (`ekena`, the singular rupa). A graha is joined with a bhava when it
//!   stands in it, and BPHS's bhava rule says where a bhava ends: a graha
//!   between two sandhis belongs to the bhava they enclose (`graha
//!   sandhi-dvayantastho jneyas tad-bhavagah sada`), a sandhi being half the
//!   sum of two adjacent bhavas (`purvaparayuter ardham sandhih syad bhavayor
//!   dvayoh`, in the other recension). So the bhava runs from the midpoint of
//!   the previous madhya and its own to the midpoint of its own and the next.
//!   The singular amount is read as once per class and not once per graha: a
//!   bhava holding both Mercury and Jupiter gets one rupa. The verse leaves
//!   that count open.
//! * **27.31, the time of birth (grade a for the classes, b for the
//!   amount).** By day a bhava in a shirshodaya rashi, at twilight one in an
//!   ubhayodaya rashi, by night one in a prishthodaya rashi, is given a
//!   quarter (`padamitam balam`), taken as a quarter rupa, 15 virupas, as the
//!   other recension is silent on the amount. The classes are BPHS chapter
//!   4's own, each rashi's description naming how it rises: shirshodaya are
//!   Gemini, Leo, Virgo, Libra, Scorpio and Aquarius; prishthodaya are Aries,
//!   Taurus, Cancer, Sagittarius and Capricorn; **ubhayodaya is Pisces
//!   alone**. Glossing *ubhayodaya* as "a dual rashi" does not match chapter 4:
//!   the dual (dvisvabhava) signs are a different classification, and the
//!   twilight bonus here goes to Pisces only.
//!
//! The ranking [`BhavaBala::krama`] and [`BhavaBala::rupa`] are conveniences
//! and not in the verses.
//!
//! # Contract
//!
//! Locale-free: the twelve bhava madhya longitudes, the
//! seven grahas' longitudes and shadbala pindas, the benefic reading and the
//! period of birth all come from the caller, so the strength is computed from
//! the same shadbala and the same bhavas the caller shows. Longitudes are
//! sidereal degrees; the madhyas run in zodiacal order from the first bhava.

use serde::Serialize;

use crate::drishti::{SputaSpecialRule, VedicPlanet, sputa_drishti_graha};
use crate::graha::Graha;

/// One rupa, in virupas (27.30 adds or takes away this much).
pub const RUPA_VIRUPA: f64 = 60.0;

/// The quarter rupa of 27.31, in virupas.
pub const UDAYA_VIRUPA: f64 = 15.0;

/// The seven grahas bhava bala is defined over.
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The time of birth that selects the rashi class of 27.31.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BirthPeriod {
    /// Birth by day: shirshodaya rashis earn the quarter rupa.
    Day,
    /// Birth at twilight: the ubhayodaya rashi (Pisces) earns it.
    Twilight,
    /// Birth by night: prishthodaya rashis earn it.
    Night,
}

/// The strength of one bhava, its parts kept apart. All in virupas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BhavaBala {
    /// The bhava, 1 to 12.
    pub bhava: u8,
    /// Bhavadhipati bala: the shadbala pinda of the lord of the rashi
    /// holding the madhya (27.29).
    pub adhipati: f64,
    /// Bhava dig bala, 0 to 60 (27.26-28).
    pub dig: f64,
    /// Net aspect on the madhya, signed (27.28-29).
    pub drishti: f64,
    /// Plus or minus a rupa for the occupants (27.30): +60, 0 or -60, or 0
    /// when a benefic and a malefic share the bhava.
    pub occupants: f64,
    /// The quarter rupa by the time of birth, 0 or 15 (27.31).
    pub udaya: f64,
    /// The sum of the five parts above.
    pub virupa: f64,
    /// `virupa / 60`.
    pub rupa: f64,
    /// Rank of `virupa` among the twelve, 1 the strongest. Equal strengths
    /// rank the lower bhava first.
    pub krama: u8,
}

/// What [`compute_bhava_bala`] reads. Every field is the caller's.
pub struct BhavaBalaInput<'a> {
    /// Shadbala pinda of the seven grahas, in virupas.
    pub pinda: &'a [(Graha, f64)],
    /// Sidereal madhya longitudes of the twelve bhavas, the first bhava
    /// first, in zodiacal order. The 1st, 4th, 7th and 10th are the points
    /// the dig rule subtracts.
    pub madhya: &'a [f64; 12],
    /// Sidereal longitudes of the seven grahas. Rahu and Ketu may be present
    /// and are ignored.
    pub positions: &'a [(Graha, f64)],
    /// The chart's benefic reading of a graha: `true` for a benefic. The
    /// verses do not classify; this is where the caller's natural and
    /// temporal reading enters. Asked for each of the seven.
    pub shubha: &'a dyn Fn(Graha) -> bool,
    /// The time of birth, for 27.31.
    pub period: BirthPeriod,
    /// Which recension's special aspects the aspect term uses; see
    /// [`crate::drishti`]. [`SputaSpecialRule::PerSignRemainder`] is the
    /// default of that module.
    pub special_aspects: SputaSpecialRule,
}

/// Zero-based sign (0 = Aries) of a longitude.
fn sign_index(longitude_deg: f64) -> u8 {
    ((longitude_deg.rem_euclid(360.0) / 30.0).floor() as u8).min(11)
}

/// The lord of a rashi, BPHS chapter 4 (each rashi's description names it).
fn sign_lord(sign: u8) -> Graha {
    match sign {
        0 | 7 => Graha::Mars,
        1 | 6 => Graha::Venus,
        2 | 5 => Graha::Mercury,
        3 => Graha::Moon,
        4 => Graha::Sun,
        8 | 11 => Graha::Jupiter,
        9 | 10 => Graha::Saturn,
        _ => unreachable!("sign_index is clamped to 0..=11"),
    }
}

/// The value a caller supplied for `graha`, or a panic naming what is
/// missing (a caller bug, in every profile).
fn value_for(list: &[(Graha, f64)], graha: Graha, what: &str) -> f64 {
    list.iter()
        .find(|&&(g, _)| g == graha)
        .map(|&(_, v)| v)
        .unwrap_or_else(|| panic!("bhava bala needs a {what} for {graha:?}"))
}

/// Bhava dig bala of 27.26-28: the short arc from the rashi group's powerless
/// point to the bhava's longitude, over three.
fn dig_virupa(longitude_deg: f64, madhya: &[f64; 12]) -> f64 {
    let lon = longitude_deg.rem_euclid(360.0);
    let sign = sign_index(lon);
    let first_half = lon - 30.0 * f64::from(sign) < 15.0;
    // Index into `madhya`: 6 the 7th bhava, 3 the 4th, 0 the lagna, 9 the 10th.
    let point = match sign {
        2 | 5 | 6 | 10 => 6,
        0 | 1 | 4 => 3,
        3 | 7 => 0,
        8 => {
            if first_half {
                6
            } else {
                3
            }
        }
        9 => {
            if first_half {
                3
            } else {
                9
            }
        }
        11 => 9,
        _ => unreachable!("sign_index is clamped to 0..=11"),
    };
    let apart = (lon - madhya[point]).rem_euclid(360.0);
    let arc = if apart > 180.0 { 360.0 - apart } else { apart };
    arc / 3.0
}

/// The net aspect on `madhya_deg` of 27.28-29.
fn drishti_net(
    madhya_deg: f64,
    positions: &[(Graha, f64)],
    shubha: &dyn Fn(Graha) -> bool,
    rule: SputaSpecialRule,
) -> f64 {
    let mut net = 0.0;
    for g in SEVEN {
        let cast = sputa_drishti_graha(
            VedicPlanet::from(g),
            value_for(positions, g, "position"),
            madhya_deg,
            rule,
        );
        // A quarter of a benefic's aspect added, of a malefic's taken away.
        net += if shubha(g) { cast / 4.0 } else { -cast / 4.0 };
        // Then the aspect of Mercury and Jupiter, in full.
        if matches!(g, Graha::Mercury | Graha::Jupiter) {
            net += cast;
        }
    }
    net
}

/// Whether `longitude_deg` stands in the arc from `start_deg` up to, not
/// including, `end_deg`, going forward round the zodiac.
fn within(longitude_deg: f64, start_deg: f64, end_deg: f64) -> bool {
    let span = (end_deg - start_deg).rem_euclid(360.0);
    (longitude_deg - start_deg).rem_euclid(360.0) < span
}

/// The sandhi between bhava `i - 1` and bhava `i`: half the sum of the two
/// madhyas, the short way round (the madhyas run forward, so the arc between
/// them is under 360 degrees).
fn sandhi_before(madhya: &[f64; 12], i: usize) -> f64 {
    let prev = madhya[(i + 11) % 12];
    let arc = (madhya[i] - prev).rem_euclid(360.0);
    prev + arc / 2.0
}

/// The occupant adjustment of 27.30 for bhava `i`.
fn occupants_virupa(i: usize, madhya: &[f64; 12], positions: &[(Graha, f64)]) -> f64 {
    let start = sandhi_before(madhya, i);
    let end = sandhi_before(madhya, (i + 1) % 12);
    let holds = |gs: &[Graha]| {
        gs.iter()
            .any(|&g| within(value_for(positions, g, "position"), start, end))
    };
    let mut adjust = 0.0;
    if holds(&[Graha::Mercury, Graha::Jupiter]) {
        adjust += RUPA_VIRUPA;
    }
    if holds(&[Graha::Saturn, Graha::Mars, Graha::Sun]) {
        adjust -= RUPA_VIRUPA;
    }
    adjust
}

/// The quarter rupa of 27.31 for a bhava whose madhya is in `sign`.
fn udaya_virupa(sign: u8, period: BirthPeriod) -> f64 {
    let earns = match period {
        // Shirshodaya: Gemini, Leo, Virgo, Libra, Scorpio, Aquarius.
        BirthPeriod::Day => matches!(sign, 2 | 4 | 5 | 6 | 7 | 10),
        // Ubhayodaya: Pisces.
        BirthPeriod::Twilight => sign == 11,
        // Prishthodaya: Aries, Taurus, Cancer, Sagittarius, Capricorn.
        BirthPeriod::Night => matches!(sign, 0 | 1 | 3 | 8 | 9),
    };
    if earns { UDAYA_VIRUPA } else { 0.0 }
}

/// Rank 1 to 12 of each strength, the strongest first; equal strengths rank
/// the lower index first, so ranks are never shared.
fn krama_ranks(virupa: &[f64; 12]) -> [u8; 12] {
    let mut order: [usize; 12] = std::array::from_fn(|i| i);
    order.sort_by(|&a, &b| virupa[b].total_cmp(&virupa[a]).then(a.cmp(&b)));
    let mut ranks = [0_u8; 12];
    for (rank, &i) in order.iter().enumerate() {
        ranks[i] = (rank + 1) as u8;
    }
    ranks
}

/// Bhava bala of the twelve bhavas, each with its parts apart.
///
/// See the module documentation for the rule and the verse behind each part.
///
/// # Panics
///
/// Panics when `input.pinda` or `input.positions` lacks any of the seven
/// grahas (Sun to Saturn): the strength of a bhava needs the lord's pinda, and
/// the aspects and occupants need every position. A silent zero would return a
/// confident wrong strength.
#[must_use]
pub fn compute_bhava_bala(input: &BhavaBalaInput<'_>) -> [BhavaBala; 12] {
    let madhya = input.madhya;
    let mut out: [BhavaBala; 12] = std::array::from_fn(|i| {
        let lon = madhya[i];
        let adhipati = value_for(input.pinda, sign_lord(sign_index(lon)), "shadbala pinda");
        let dig = dig_virupa(lon, madhya);
        let drishti = drishti_net(lon, input.positions, input.shubha, input.special_aspects);
        let occupants = occupants_virupa(i, madhya, input.positions);
        let udaya = udaya_virupa(sign_index(lon), input.period);
        let virupa = adhipati + dig + drishti + occupants + udaya;
        BhavaBala {
            bhava: (i + 1) as u8,
            adhipati,
            dig,
            drishti,
            occupants,
            udaya,
            virupa,
            rupa: virupa / 60.0,
            krama: 0,
        }
    });
    let ranks = krama_ranks(&std::array::from_fn(|i| out[i].virupa));
    for (b, rank) in out.iter_mut().zip(ranks) {
        b.krama = rank;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Equal houses from a lagna at Aries 15 degrees: madhyas 15, 45, ...
    fn equal_madhya() -> [f64; 12] {
        std::array::from_fn(|i| 15.0 + 30.0 * i as f64)
    }

    /// 27.26-28: the zero and the maximum of the dig of a bhava. With the
    /// lagna at 15 degrees the 4th, 7th and 10th madhyas are 105, 195 and
    /// 285. Each expected value is the short arc to the group's point / 3.
    #[test]
    fn dig_hand_vectors_by_rashi_group() {
        let m = equal_madhya();
        // Gemini 15 (75), group "7th": |75 - 195| = 120 -> 40.
        assert!(close(dig_virupa(75.0, &m), 40.0));
        // Virgo 15 (165): |165 - 195| = 30 -> 10.
        assert!(close(dig_virupa(165.0, &m), 10.0));
        // Libra 15 (195): on the 7th itself -> 0, the powerless point.
        assert!(close(dig_virupa(195.0, &m), 0.0));
        // Aquarius 15 (315): the arc to 195 is 120 -> 40.
        assert!(close(dig_virupa(315.0, &m), 40.0));
        // Aries 15 (15), group "4th": 15 - 105 = -90 -> 90 -> 30.
        assert!(close(dig_virupa(15.0, &m), 30.0));
        // Taurus 15 (45): 60 -> 20. Leo 15 (135): 30 -> 10.
        assert!(close(dig_virupa(45.0, &m), 20.0));
        assert!(close(dig_virupa(135.0, &m), 10.0));
        // Cancer 15 (105), group "lagna": 105 - 15 = 90 -> 30.
        assert!(close(dig_virupa(105.0, &m), 30.0));
        // Scorpio 15 (225): 210 apart, the short way 150 -> 50.
        assert!(close(dig_virupa(225.0, &m), 50.0));
        // Pisces 15 (345), group "10th": 345 - 285 = 60 -> 20.
        assert!(close(dig_virupa(345.0, &m), 20.0));
    }

    /// The two halves of Sagittarius and Capricorn split between groups.
    #[test]
    fn dig_splits_sagittarius_and_capricorn_at_fifteen_degrees() {
        let m = equal_madhya();
        // Sagittarius 5 (245): first half, "7th": 245 - 195 = 50 -> 50/3.
        assert!(close(dig_virupa(245.0, &m), 50.0 / 3.0));
        // Sagittarius 20 (260): second half, "4th": 260 - 105 = 155 -> 155/3.
        assert!(close(dig_virupa(260.0, &m), 155.0 / 3.0));
        // Sagittarius 15 exactly (255) is the second half: 255 - 105 = 150 -> 50.
        assert!(close(dig_virupa(255.0, &m), 50.0));
        // Capricorn 5 (275): first half, "4th": 275 - 105 = 170 -> 170/3.
        assert!(close(dig_virupa(275.0, &m), 170.0 / 3.0));
        // Capricorn 20 (290): second half, "10th": 290 - 285 = 5 -> 5/3.
        assert!(close(dig_virupa(290.0, &m), 5.0 / 3.0));
    }

    /// The dig never leaves 0..=60 and reaches 60 exactly opposite the
    /// powerless point: a Gemini longitude of 75 with the 7th madhya at 255
    /// is 180 degrees from it.
    #[test]
    fn dig_is_bounded_and_peaks_opposite_the_point() {
        let m = equal_madhya();
        let mut x = 0.0;
        while x < 360.0 {
            let v = dig_virupa(x, &m);
            assert!((0.0..=60.0).contains(&v), "x={x}: {v}");
            x += 0.5;
        }
        let mut peak = equal_madhya();
        peak[6] = 255.0;
        assert!(close(dig_virupa(75.0, &peak), 60.0));
    }

    /// 27.29 and chapter 4: the lords of the twelve rashis.
    #[test]
    fn sign_lords_are_the_classical_seven() {
        let want = [
            Graha::Mars,
            Graha::Venus,
            Graha::Mercury,
            Graha::Moon,
            Graha::Sun,
            Graha::Mercury,
            Graha::Venus,
            Graha::Mars,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Saturn,
            Graha::Jupiter,
        ];
        for (sign, lord) in want.into_iter().enumerate() {
            assert_eq!(sign_lord(sign as u8), lord, "sign {sign}");
        }
    }

    /// Positions that cast nothing on a madhya at `m`: each stands on `m`
    /// itself, a separation of 0 (nothing within a sign).
    fn silent_positions(m: f64) -> Vec<(Graha, f64)> {
        SEVEN.iter().map(|&g| (g, m)).collect()
    }

    /// 27.28-29, one active graha at a time, worked by hand.
    ///
    /// The madhya is at 0 and the active graha stands at 300, so x = 60
    /// (two signs, no degrees past): the generic aspect is 0 + 15 = 15, and
    /// Saturn's special `c = 2` is 60 - 0 = 60, Mars's is 1.5 * 0 + 15 = 15,
    /// Jupiter's generic 15 (his specials start at c = 3).
    #[test]
    fn drishti_net_hand_vectors() {
        let benefic = |_: Graha| true;
        let malefic = |_: Graha| false;
        let net = |g: Graha, shubha: &dyn Fn(Graha) -> bool| {
            let mut pos = silent_positions(0.0);
            pos.iter_mut().find(|(x, _)| *x == g).unwrap().1 = 300.0;
            drishti_net(0.0, &pos, shubha, SputaSpecialRule::PerSignRemainder)
        };
        // Moon, benefic: +15/4. Malefic: -15/4.
        assert!(close(net(Graha::Moon, &benefic), 3.75));
        assert!(close(net(Graha::Moon, &malefic), -3.75));
        // Venus follows the same quarter rule.
        assert!(close(net(Graha::Venus, &benefic), 3.75));
        // Mercury, benefic: 15/4 + the full 15 = 18.75; malefic: -3.75 + 15.
        assert!(close(net(Graha::Mercury, &benefic), 18.75));
        assert!(close(net(Graha::Mercury, &malefic), 11.25));
        // Jupiter, benefic: 15/4 + 15 = 18.75 (his generic 15 at c = 2).
        assert!(close(net(Graha::Jupiter, &benefic), 18.75));
        // Saturn, malefic: his special 60 -> -60/4 = -15; he gets no full add.
        assert!(close(net(Graha::Saturn, &malefic), -15.0));
        // Mars, malefic: 15 -> -3.75.
        assert!(close(net(Graha::Mars, &malefic), -3.75));
        // A graha standing on the madhya, or 30 behind it, casts nothing.
        let pos = silent_positions(0.0);
        assert!(close(
            drishti_net(0.0, &pos, &benefic, SputaSpecialRule::PerSignRemainder),
            0.0
        ));
    }

    /// 27.28: the sum over several grahas is the sum of the single casts.
    #[test]
    fn drishti_net_is_additive() {
        let benefic = |g: Graha| matches!(g, Graha::Moon | Graha::Venus);
        let mut pos = silent_positions(0.0);
        for (g, lon) in [
            (Graha::Moon, 300.0),   // x = 60: 15, benefic: +3.75
            (Graha::Saturn, 300.0), // x = 60: special 60, malefic: -15
            (Graha::Venus, 195.0),  // x = 165: over five signs, 2 * 15 = 30, benefic: +7.5
        ] {
            pos.iter_mut().find(|(x, _)| *x == g).unwrap().1 = lon;
        }
        assert!(close(
            drishti_net(0.0, &pos, &benefic, SputaSpecialRule::PerSignRemainder),
            3.75 - 15.0 + 7.5
        ));
    }

    /// 27.30. Equal houses: the sandhis are at 0, 30, 60, ... so bhava 1
    /// spans [0, 30).
    #[test]
    fn occupants_hand_vectors() {
        let m = equal_madhya();
        let at = |placed: &[(Graha, f64)]| -> Vec<(Graha, f64)> {
            // Everyone else far from the bhavas under test: in bhava 12.
            let mut pos: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 350.0)).collect();
            for &(g, lon) in placed {
                pos.iter_mut().find(|(x, _)| *x == g).unwrap().1 = lon;
            }
            pos
        };
        // Jupiter alone at 20: bhava 1 gets +60.
        assert!(close(
            occupants_virupa(0, &m, &at(&[(Graha::Jupiter, 20.0)])),
            60.0
        ));
        // Jupiter and Mercury together: once per class, still +60.
        assert!(close(
            occupants_virupa(
                0,
                &m,
                &at(&[(Graha::Jupiter, 20.0), (Graha::Mercury, 10.0)])
            ),
            60.0
        ));
        // Jupiter and Mars together: +60 - 60 = 0.
        assert!(close(
            occupants_virupa(0, &m, &at(&[(Graha::Jupiter, 20.0), (Graha::Mars, 25.0)])),
            0.0
        ));
        // Saturn, Mars and the Sun all in bhava 3 [60, 90): one rupa off, once.
        assert!(close(
            occupants_virupa(
                2,
                &m,
                &at(&[
                    (Graha::Saturn, 61.0),
                    (Graha::Mars, 75.0),
                    (Graha::Sun, 89.0)
                ])
            ),
            -60.0
        ));
        // The Moon and Venus change nothing.
        assert!(close(
            occupants_virupa(0, &m, &at(&[(Graha::Moon, 20.0), (Graha::Venus, 21.0)])),
            0.0
        ));
    }

    /// The bhava is sandhi to sandhi, not sign to sign: with unequal
    /// madhyas the boundary is the midpoint of the two neighbouring madhyas.
    #[test]
    fn occupants_use_the_sandhi_midpoint_of_unequal_bhavas() {
        // Madhyas 10, 50, 100, 130, ...: the sandhi between bhava 1 and 2 is
        // at 30, between 2 and 3 at 75, between 12 and 1 at the midpoint of
        // the last madhya and 10.
        let m = [
            10.0, 50.0, 100.0, 130.0, 170.0, 200.0, 230.0, 270.0, 300.0, 320.0, 340.0, 355.0,
        ];
        let only = |g: Graha, lon: f64| -> Vec<(Graha, f64)> {
            SEVEN
                .iter()
                .map(|&x| (x, if x == g { lon } else { 200.0 }))
                .collect()
        };
        let jup = |lon| occupants_virupa(1, &m, &only(Graha::Jupiter, lon));
        // Bhava 2 spans [30, 75).
        assert!(close(jup(29.9), 0.0));
        assert!(close(jup(30.0), 60.0));
        assert!(close(jup(74.9), 60.0));
        assert!(close(jup(75.0), 0.0));
        // Bhava 1 wraps: from the midpoint of 355 and 10 = 2.5 (355 + 7.5 = 362.5)
        // to 30.
        let jup1 = |lon| occupants_virupa(0, &m, &only(Graha::Jupiter, lon));
        assert!(close(jup1(2.4), 0.0));
        assert!(close(jup1(2.5), 60.0));
        assert!(close(jup1(29.9), 60.0));
        assert!(close(jup1(359.0), 0.0));
    }

    /// 27.31 and chapter 4: the classes, and the ubhayodaya point.
    #[test]
    fn udaya_classes_by_period() {
        let shirsha = [2_u8, 4, 5, 6, 7, 10];
        let prishtha = [0_u8, 1, 3, 8, 9];
        for s in 0..12_u8 {
            let day = udaya_virupa(s, BirthPeriod::Day);
            let night = udaya_virupa(s, BirthPeriod::Night);
            let twi = udaya_virupa(s, BirthPeriod::Twilight);
            assert!(
                close(day, if shirsha.contains(&s) { 15.0 } else { 0.0 }),
                "{s}"
            );
            assert!(
                close(night, if prishtha.contains(&s) { 15.0 } else { 0.0 }),
                "{s}"
            );
            // Pisces alone, not the dual signs.
            assert!(close(twi, if s == 11 { 15.0 } else { 0.0 }), "{s}");
        }
        // Every rashi is in exactly one class.
        for s in 0..12_u8 {
            let n = usize::from(shirsha.contains(&s))
                + usize::from(prishtha.contains(&s))
                + usize::from(s == 11);
            assert_eq!(n, 1, "sign {s}");
        }
    }

    fn pinda_table() -> Vec<(Graha, f64)> {
        vec![
            (Graha::Sun, 400.0),
            (Graha::Moon, 410.0),
            (Graha::Mars, 420.0),
            (Graha::Mercury, 430.0),
            (Graha::Jupiter, 440.0),
            (Graha::Venus, 450.0),
            (Graha::Saturn, 460.0),
        ]
    }

    /// Whole computation on a chart small enough to check by hand: equal
    /// houses from Aries 15, all seven grahas on the 12th madhya (345),
    /// everyone malefic, a night birth.
    ///
    /// Bhava 1 (madhya 15, Aries): lord Mars -> adhipati 420; dig: Aries is
    /// the "4th" group, 15 - 105 = -90 -> 30; udaya: Aries is prishthodaya,
    /// night -> 15. The grahas at 345 are x = 30 from 15 (c = 1, d = 0): no
    /// aspect from any. Bhava 12 holds all seven (sandhi 330 to 360):
    /// Jupiter/Mercury +60, Saturn/Mars/Sun -60 -> 0.
    #[test]
    fn compute_assembles_the_parts_for_a_hand_checked_chart() {
        let m = equal_madhya();
        let positions: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 345.0)).collect();
        let pinda = pinda_table();
        let all_malefic = |_: Graha| false;
        let out = compute_bhava_bala(&BhavaBalaInput {
            pinda: &pinda,
            madhya: &m,
            positions: &positions,
            shubha: &all_malefic,
            period: BirthPeriod::Night,
            special_aspects: SputaSpecialRule::PerSignRemainder,
        });
        let b1 = out[0];
        assert_eq!(b1.bhava, 1);
        assert!(close(b1.adhipati, 420.0));
        assert!(close(b1.dig, 30.0));
        assert!(close(b1.udaya, 15.0));
        assert!(close(b1.occupants, 0.0));
        // x from 345 to 15 is 30: c = 1, d = 0 -> every cast is 0 (Saturn's
        // c = 1 is 2 * 0 = 0 as well).
        assert!(close(b1.drishti, 0.0));
        assert!(close(b1.virupa, 420.0 + 30.0 + 15.0));
        assert!(close(b1.rupa, b1.virupa / 60.0));
        // Bhava 12: Pisces, lord Jupiter, "10th" group, 345 - 285 = 60 -> 20,
        // night: Pisces is not prishthodaya -> 0; occupants +60 - 60 = 0;
        // grahas on the madhya itself cast nothing.
        let b12 = out[11];
        assert!(close(b12.adhipati, 440.0));
        assert!(close(b12.dig, 20.0));
        assert!(close(b12.udaya, 0.0));
        assert!(close(b12.occupants, 0.0));
        assert!(close(b12.drishti, 0.0));
        assert!(close(b12.virupa, 460.0));
        // Every row is the sum of its parts.
        for b in out {
            assert!(
                close(
                    b.virupa,
                    b.adhipati + b.dig + b.drishti + b.occupants + b.udaya
                ),
                "bhava {}",
                b.bhava
            );
        }
    }

    /// The same chart with the Moon moved to Sagittarius 15 (255) and read as
    /// a benefic: onto the 1st madhya (15) her separation is 120 degrees
    /// (c = 4, d = 0), the generic aspect 150 - 120 = 30, a quarter added:
    /// bhava 1's net aspect is +7.5 and nothing else about bhava 1 moves.
    #[test]
    fn a_benefic_aspect_adds_a_quarter_to_the_bhava_it_falls_on() {
        let m = equal_madhya();
        let mut positions: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 345.0)).collect();
        positions
            .iter_mut()
            .find(|(g, _)| *g == Graha::Moon)
            .unwrap()
            .1 = 255.0;
        let pinda = pinda_table();
        let moon_benefic = |g: Graha| g == Graha::Moon;
        let out = compute_bhava_bala(&BhavaBalaInput {
            pinda: &pinda,
            madhya: &m,
            positions: &positions,
            shubha: &moon_benefic,
            period: BirthPeriod::Night,
            special_aspects: SputaSpecialRule::PerSignRemainder,
        });
        assert!(close(out[0].drishti, 7.5));
        assert!(close(out[0].virupa, 420.0 + 30.0 + 7.5 + 15.0));
    }

    /// The ranking is a permutation of 1..=12 ordered by strength.
    #[test]
    fn krama_is_a_permutation_ordered_by_strength() {
        let m = equal_madhya();
        let positions: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 345.0)).collect();
        let pinda = pinda_table();
        let shubha = |g: Graha| matches!(g, Graha::Jupiter | Graha::Venus | Graha::Moon);
        let out = compute_bhava_bala(&BhavaBalaInput {
            pinda: &pinda,
            madhya: &m,
            positions: &positions,
            shubha: &shubha,
            period: BirthPeriod::Day,
            special_aspects: SputaSpecialRule::PerSignRemainder,
        });
        let mut ranks: Vec<u8> = out.iter().map(|b| b.krama).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, (1..=12).collect::<Vec<u8>>());
        for a in out {
            for b in out {
                if a.krama < b.krama {
                    assert!(
                        a.virupa >= b.virupa,
                        "bhava {} (rank {}) vs bhava {} (rank {})",
                        a.bhava,
                        a.krama,
                        b.bhava,
                        b.krama
                    );
                }
            }
        }
    }

    /// Ties are broken by the lower bhava, never shared: hand-ranked.
    #[test]
    fn krama_ranks_hand_vector_with_ties() {
        // Strengths for bhavas 1..=12: 300 twice (bhavas 2 and 9), the top
        // 500 (bhava 5), 100 four times (bhavas 1, 6, 7, 12), others distinct.
        let v = [
            100.0, 300.0, 250.0, 200.0, 500.0, 100.0, 100.0, 150.0, 300.0, 400.0, 120.0, 100.0,
        ];
        // Order strongest first: b5 500, b10 400, b2 300, b9 300, b3 250,
        // b4 200, b8 150, b11 120, b1 100, b6 100, b7 100, b12 100.
        let want = [9, 3, 5, 6, 1, 10, 11, 7, 4, 2, 8, 12];
        assert_eq!(krama_ranks(&v), want);
    }

    /// Rahu and Ketu in `positions` change nothing.
    #[test]
    fn nodes_in_positions_are_ignored() {
        let m = equal_madhya();
        let mut positions: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 345.0)).collect();
        let pinda = pinda_table();
        let shubha = |_: Graha| true;
        let input = |positions: &[(Graha, f64)]| {
            compute_bhava_bala(&BhavaBalaInput {
                pinda: &pinda,
                madhya: &m,
                positions,
                shubha: &shubha,
                period: BirthPeriod::Day,
                special_aspects: SputaSpecialRule::default(),
            })
        };
        let base = input(&positions);
        positions.push((Graha::Rahu, 100.0));
        positions.push((Graha::Ketu, 280.0));
        assert_eq!(input(&positions), base);
    }

    #[test]
    #[should_panic(expected = "needs a position for Venus")]
    fn missing_position_panics() {
        let m = equal_madhya();
        let positions: Vec<(Graha, f64)> = SEVEN
            .iter()
            .filter(|&&g| g != Graha::Venus)
            .map(|&g| (g, 345.0))
            .collect();
        let pinda = pinda_table();
        let shubha = |_: Graha| true;
        let _ = compute_bhava_bala(&BhavaBalaInput {
            pinda: &pinda,
            madhya: &m,
            positions: &positions,
            shubha: &shubha,
            period: BirthPeriod::Day,
            special_aspects: SputaSpecialRule::default(),
        });
    }

    #[test]
    #[should_panic(expected = "needs a shadbala pinda for Saturn")]
    fn missing_pinda_panics() {
        let m = equal_madhya();
        let positions: Vec<(Graha, f64)> = SEVEN.iter().map(|&g| (g, 345.0)).collect();
        let pinda: Vec<(Graha, f64)> = pinda_table()
            .into_iter()
            .filter(|&(g, _)| g != Graha::Saturn)
            .collect();
        let shubha = |_: Graha| true;
        let _ = compute_bhava_bala(&BhavaBalaInput {
            pinda: &pinda,
            madhya: &m,
            positions: &positions,
            shubha: &shubha,
            period: BirthPeriod::Day,
            special_aspects: SputaSpecialRule::default(),
        });
    }
}
