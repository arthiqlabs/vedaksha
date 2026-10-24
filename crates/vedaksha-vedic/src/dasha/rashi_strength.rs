// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Natal placements and Jaimini's "the stronger of two" comparisons.
//!
//! The sthira, niryana shoola and maheshwara dashas and the antardashas of
//! the rashi dashas all say "the stronger of" two rashis or two grahas
//! without a method in the rule itself. The method is Jaimini's own and is
//! given in the strength section that opens adhyaya 2, pada 3 ("now
//! strength"). This module implements the sources of strength that section
//! gives and the order it gives them in, behind the [`Strength`] trait, so a
//! caller who follows another school can supply a different comparison and
//! every dasha here will use it.
//!
//! # Numbering and grades
//!
//! Sutra numbers in this module and its siblings are those of the critical
//! edition of the Upadesha Sutras this crate was written against. They are
//! not stable across editions (in adhyaya 2, pada 1 another numbering runs
//! five lower), so each rule is also given by its incipit. Grades follow
//! `dasha::chara`'s usage: (a) the sutra or verse states the rule outright
//! and the reading is not in dispute; (b) stated, with a word or an extent
//! open to reading, so the crate takes the best-supported reading and says
//! so; (c) not stated for this system, carried from a sibling passage or a
//! commentary, i.e. a convention; (d) carried over without a worked example
//! to exercise it.
//!
//! # What the cascade implements
//!
//! For two **rashis** ([`JaiminiStrength::stronger_rashi`]), in order, the
//! first test that separates them decides:
//!
//! 1. *karaka-yogah prathamo bhanam* (2.3.6): a rashi holding grahas beats
//!    one holding none. The text states the same principle for grahas as
//!    *agrahat sagraho jyayan* (quoted at 2.3.9 and applied to the parents'
//!    karakas at 2.2.2). Grade (a).
//! 2. *samye bhuyasa* (2.3.7): with both occupied, the one with more
//!    grahas. Grade (a).
//! 3. *tatas tungadih* (2.3.8): then a rashi holding an exalted graha. The
//!    sutra's "adi" (a graha in a friend's house, a graha aspected by a
//!    benefic, per the commentary) is left open by the text's own wording
//!    and is **not** implemented. Grade (b).
//! 4. *nisargas tatah* (2.3.9): natural strength, dual over fixed over
//!    movable. Grade (a). (The BPHS statement of the same cascade at 46.161
//!    to 163 puts this before exaltation; the sutra's order is followed
//!    here.)
//! 5. *tad-abhave svamini itthambhavah* (2.3.10): failing all that, the
//!    same tests applied to the rashis' lords. A rashi with two lords
//!    (Scorpio: Mars and Ketu; Aquarius: Saturn and Rahu) is represented by
//!    its stronger lord. Grade (b): the sutra says only that the lord is
//!    judged "in this manner", and the lord's own tests are taken here to be
//!    the graha tests below.
//! 6. *svaminas trtiyah* (2.3.15): a rashi whose lord stands in it.
//!    Grade (a).
//! 7. *caturthatah purushe* (2.3.17): an odd (male) rashi over an even one.
//!    Grade (a).
//!
//! A total tie goes to the first argument. Not implemented: the "second"
//! strength (2.3.14, the conjunction or aspect of the lord, Jupiter and
//! Mercury on a rashi), because it needs the rashi aspects, and 2.3.12 (a
//! graha in the neighbouring male rashi), whose commentary reads two ways.
//!
//! For two **grahas** ([`JaiminiStrength::stronger_graha`]) the same tests
//! are read at the graha: companions (other grahas in its rashi) first, as
//! *agrahat sagraho jyayan* and *sagrahad adhikagrahah* have it; then
//! exaltation (2.3.29, *tungadi*); then the natural strength of the rashi it
//! stands in; then, for two grahas in one rashi, *agrayato 'tra visesat*
//! (2.3.11): the one further advanced in degrees (a node counts its degrees
//! from the other end of the rashi, as `karaka` does for Rahu, because it
//! moves backward). Grade (b) overall: the text applies "stronger" to
//! grahas in several places without listing the tests once.

use serde::{Deserialize, Serialize};

use crate::dasha::chara::GrahaSigns;
use crate::graha::Graha;

/// The nine grahas in weekday order, with the nodes after Saturn.
///
/// This is the cycle in which "the sixth graha from" a graha is counted
/// (BPHS 46.172, *tat-shashtha-khecharah*; Jaimini 2.1.53).
pub const GRAHA_CYCLE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// Position of `graha` in [`GRAHA_CYCLE`].
#[must_use]
pub const fn graha_index(graha: Graha) -> usize {
    match graha {
        Graha::Sun => 0,
        Graha::Moon => 1,
        Graha::Mars => 2,
        Graha::Mercury => 3,
        Graha::Jupiter => 4,
        Graha::Venus => 5,
        Graha::Saturn => 6,
        Graha::Rahu => 7,
        Graha::Ketu => 8,
    }
}

/// The graha `steps` places after `graha` in [`GRAHA_CYCLE`] (wrapping), so
/// `steps = 5` is "the sixth graha counting `graha` as the first".
#[must_use]
pub fn graha_after(graha: Graha, steps: usize) -> Graha {
    GRAHA_CYCLE[(graha_index(graha) + steps) % 9]
}

/// A rashi's nature by modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modality {
    /// Chara: Aries, Cancer, Libra, Capricorn.
    Movable,
    /// Sthira: Taurus, Leo, Scorpio, Aquarius.
    Fixed,
    /// Dwisvabhava: Gemini, Virgo, Sagittarius, Pisces.
    Dual,
}

/// Modality of a zero-based sign.
#[must_use]
pub const fn modality(sign: u8) -> Modality {
    match sign % 3 {
        0 => Modality::Movable,
        1 => Modality::Fixed,
        _ => Modality::Dual,
    }
}

/// Whether a zero-based sign is an odd (male, *oja*) sign: Aries, Gemini,
/// Leo, Libra, Sagittarius, Aquarius. Zero-based, so sign 0 is odd.
#[must_use]
pub const fn is_odd_sign(sign: u8) -> bool {
    sign % 2 == 0
}

/// The grahas that rule a zero-based sign.
///
/// Scorpio has Mars and Ketu, Aquarius Saturn and Rahu, as `dasha::chara`
/// applies them (BPHS 46.157).
#[must_use]
pub fn rashi_lords(sign: u8) -> &'static [Graha] {
    match sign % 12 {
        0 => &[Graha::Mars],
        1 | 6 => &[Graha::Venus],
        2 | 5 => &[Graha::Mercury],
        3 => &[Graha::Moon],
        4 => &[Graha::Sun],
        7 => &[Graha::Mars, Graha::Ketu],
        8 | 11 => &[Graha::Jupiter],
        9 => &[Graha::Saturn],
        _ => &[Graha::Saturn, Graha::Rahu],
    }
}

/// A graha's sign of exaltation, zero-based.
///
/// Rahu and Ketu use the pairing `dasha::chara` documents (Rahu in Taurus,
/// Ketu in Scorpio), which is itself contested across traditions.
#[must_use]
pub const fn exaltation_sign(graha: Graha) -> u8 {
    match graha {
        Graha::Sun => 0,
        Graha::Moon | Graha::Rahu => 1,
        Graha::Mars => 9,
        Graha::Mercury => 5,
        Graha::Jupiter => 3,
        Graha::Venus => 11,
        Graha::Saturn => 6,
        Graha::Ketu => 7,
    }
}

/// The lagna and the nine grahas, as the rashi dashas read them.
///
/// Signs are zero-based (0 = Aries). Degrees within the sign are optional:
/// they are used only to break a tie between two grahas in one rashi, and a
/// placement built from signs alone ([`NatalPlacements::from_signs`]) simply
/// skips that step.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NatalPlacements {
    lagna_sign: u8,
    signs: [u8; 9],
    degrees: [Option<f64>; 9],
}

impl NatalPlacements {
    /// From the natal signs `dasha::chara` already takes. Ketu is derived
    /// opposite Rahu. No degrees are known.
    #[must_use]
    pub fn from_signs(lagna_sign: u8, signs: GrahaSigns) -> Self {
        Self {
            lagna_sign: lagna_sign % 12,
            signs: [
                signs.sun % 12,
                signs.moon % 12,
                signs.mars % 12,
                signs.mercury % 12,
                signs.jupiter % 12,
                signs.venus % 12,
                signs.saturn % 12,
                signs.rahu % 12,
                signs.ketu() % 12,
            ],
            degrees: [None; 9],
        }
    }

    /// From sidereal longitudes in degrees. `grahas` is Sun, Moon, Mars,
    /// Mercury, Jupiter, Venus, Saturn, Rahu in that order; Ketu is derived
    /// 180 degrees from Rahu.
    #[must_use]
    pub fn from_longitudes(lagna_longitude: f64, grahas: [f64; 8]) -> Self {
        let sign_of = |lon: f64| (lon.rem_euclid(360.0) / 30.0).floor() as u8 % 12;
        let degrees_of = |lon: f64| lon.rem_euclid(30.0);
        let ketu = grahas[7] + 180.0;
        let mut signs = [0u8; 9];
        let mut degrees = [None; 9];
        for (i, lon) in grahas.iter().copied().chain([ketu]).enumerate() {
            signs[i] = sign_of(lon);
            degrees[i] = Some(degrees_of(lon));
        }
        Self {
            lagna_sign: sign_of(lagna_longitude),
            signs,
            degrees,
        }
    }

    /// Zero-based sign of the lagna.
    #[must_use]
    pub const fn lagna_sign(&self) -> u8 {
        self.lagna_sign
    }

    /// Zero-based sign a graha occupies.
    #[must_use]
    pub const fn sign_of(&self, graha: Graha) -> u8 {
        self.signs[graha_index(graha)]
    }

    /// Degrees a graha has traversed in its sign, if known.
    #[must_use]
    pub const fn degrees_of(&self, graha: Graha) -> Option<f64> {
        self.degrees[graha_index(graha)]
    }

    /// The natal signs in the form `dasha::chara` takes.
    #[must_use]
    pub fn graha_signs(&self) -> GrahaSigns {
        GrahaSigns {
            sun: self.signs[0],
            moon: self.signs[1],
            mars: self.signs[2],
            mercury: self.signs[3],
            jupiter: self.signs[4],
            venus: self.signs[5],
            saturn: self.signs[6],
            rahu: self.signs[7],
        }
    }

    /// The grahas standing in `sign`, in [`GRAHA_CYCLE`] order.
    #[must_use]
    pub fn occupants_of(&self, sign: u8) -> Vec<Graha> {
        GRAHA_CYCLE
            .iter()
            .copied()
            .filter(|&g| self.sign_of(g) == sign % 12)
            .collect()
    }

    /// How many grahas other than `graha` share its sign.
    #[must_use]
    pub fn companions_of(&self, graha: Graha) -> usize {
        GRAHA_CYCLE
            .iter()
            .filter(|&&g| g != graha && self.sign_of(g) == self.sign_of(graha))
            .count()
    }
}

/// A source of "the stronger of two". Implement this to replace the Jaimini
/// tests with another school's.
pub trait Strength {
    /// The stronger of two zero-based rashis. If `a == b`, returns `a`.
    fn stronger_rashi(&self, a: u8, b: u8) -> u8;

    /// The stronger of two grahas. If `a == b`, returns `a`.
    fn stronger_graha(&self, a: Graha, b: Graha) -> Graha;
}

/// A rashi's lord for a comparison: its only lord, or the stronger of its
/// two (Scorpio: Mars and Ketu; Aquarius: Saturn and Rahu) by `strength`,
/// the first-named lord on a tie.
#[must_use]
pub fn effective_lord(sign: u8, strength: &dyn Strength) -> Graha {
    match *rashi_lords(sign) {
        [first, second] => strength.stronger_graha(first, second),
        [only] => only,
        _ => Graha::Sun,
    }
}

/// Rank of a rashi's natural strength: dual over fixed over movable (2.3.9).
const fn natural_rank(sign: u8) -> u8 {
    match modality(sign) {
        Modality::Movable => 0,
        Modality::Fixed => 1,
        Modality::Dual => 2,
    }
}

/// Jaimini's strength tests over a chart. See the module doc for the order
/// and the grade of each test.
#[derive(Debug, Clone, Copy)]
pub struct JaiminiStrength<'a> {
    placements: &'a NatalPlacements,
}

impl<'a> JaiminiStrength<'a> {
    /// Judge strength over `placements`.
    #[must_use]
    pub const fn new(placements: &'a NatalPlacements) -> Self {
        Self { placements }
    }

    fn exalted_in(&self, sign: u8) -> bool {
        self.placements
            .occupants_of(sign)
            .iter()
            .any(|&g| self.placements.sign_of(g) == exaltation_sign(g))
    }

    fn lord_stands_in_own_rashi(&self, sign: u8) -> bool {
        rashi_lords(sign)
            .iter()
            .any(|&l| self.placements.sign_of(l) == sign)
    }

    /// Degrees "advanced" in the sign; a node's are counted from the far end.
    fn advanced_degrees(&self, graha: Graha) -> Option<f64> {
        let d = self.placements.degrees_of(graha)?;
        Some(match graha {
            Graha::Rahu | Graha::Ketu if d < 1e-9 => 0.0,
            Graha::Rahu | Graha::Ketu => 30.0 - d,
            _ => d,
        })
    }
}

impl Strength for JaiminiStrength<'_> {
    fn stronger_rashi(&self, a: u8, b: u8) -> u8 {
        let (a, b) = (a % 12, b % 12);
        if a == b {
            return a;
        }
        let pick = |a_wins: bool| if a_wins { a } else { b };

        // 2.3.6 and 2.3.7: grahas present, then how many; an empty rashi
        // has the fewest, so one comparison covers both.
        let (na, nb) = (
            self.placements.occupants_of(a).len(),
            self.placements.occupants_of(b).len(),
        );
        if na != nb {
            return pick(na > nb);
        }
        // 2.3.8: an exalted graha present.
        let (ea, eb) = (self.exalted_in(a), self.exalted_in(b));
        if ea != eb {
            return pick(ea);
        }
        // 2.3.9: natural strength.
        let (ra, rb) = (natural_rank(a), natural_rank(b));
        if ra != rb {
            return pick(ra > rb);
        }
        // 2.3.10: the lords, by the graha tests.
        let (la, lb) = (effective_lord(a, self), effective_lord(b, self));
        if let Some(winner) = self.graha_edge(la, lb) {
            return pick(winner == la);
        }
        // 2.3.15: the lord standing in its own rashi.
        let (oa, ob) = (
            self.lord_stands_in_own_rashi(a),
            self.lord_stands_in_own_rashi(b),
        );
        if oa != ob {
            return pick(oa);
        }
        // 2.3.17: odd over even.
        if is_odd_sign(a) != is_odd_sign(b) {
            return pick(is_odd_sign(a));
        }
        a
    }

    fn stronger_graha(&self, a: Graha, b: Graha) -> Graha {
        self.graha_edge(a, b).unwrap_or(a)
    }
}

impl JaiminiStrength<'_> {
    /// The stronger of two grahas, or `None` when no test separates them.
    fn graha_edge(&self, a: Graha, b: Graha) -> Option<Graha> {
        if a == b {
            return None;
        }
        let pick = |a_wins: bool| Some(if a_wins { a } else { b });
        let p = self.placements;

        // Companions: any at all, then how many; none is the fewest.
        let (ca, cb) = (p.companions_of(a), p.companions_of(b));
        if ca != cb {
            return pick(ca > cb);
        }
        // Exaltation.
        let (ea, eb) = (
            p.sign_of(a) == exaltation_sign(a),
            p.sign_of(b) == exaltation_sign(b),
        );
        if ea != eb {
            return pick(ea);
        }
        // Natural strength of the rashi stood in.
        let (ra, rb) = (natural_rank(p.sign_of(a)), natural_rank(p.sign_of(b)));
        if ra != rb {
            return pick(ra > rb);
        }
        // 2.3.11: in one rashi, the further advanced.
        if p.sign_of(a) == p.sign_of(b) {
            if let (Some(da), Some(db)) = (self.advanced_degrees(a), self.advanced_degrees(b)) {
                if (da - db).abs() > 1e-12 {
                    return pick(da > db);
                }
            }
        }
        None
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn signs(
        sun: u8,
        moon: u8,
        mars: u8,
        mercury: u8,
        jupiter: u8,
        venus: u8,
        saturn: u8,
        rahu: u8,
    ) -> GrahaSigns {
        GrahaSigns {
            sun,
            moon,
            mars,
            mercury,
            jupiter,
            venus,
            saturn,
            rahu,
        }
    }

    // Chart A. Sun and Moon in Aries(0); Mars, Mercury and Jupiter in
    // Cancer(3); Venus in Libra(6); Saturn in Capricorn(9); Rahu in
    // Gemini(2), so Ketu is in Sagittarius(8). Lagna Aries.
    //
    // Occupants by sign: 0:{Sun,Moon} 2:{Rahu} 3:{Mars,Mercury,Jupiter}
    // 6:{Venus} 8:{Ketu} 9:{Saturn}; all others empty.
    fn chart_a() -> NatalPlacements {
        NatalPlacements::from_signs(0, signs(0, 0, 3, 3, 3, 6, 9, 2))
    }

    // 1. Constants the rest of the module leans on.
    #[test]
    fn sign_nature_tables() {
        // Aries odd, Taurus even, ... Aquarius odd, Pisces even.
        let odd: Vec<u8> = (0..12).filter(|&s| is_odd_sign(s)).collect();
        assert_eq!(odd, vec![0, 2, 4, 6, 8, 10]);
        // Movable 0,3,6,9; fixed 1,4,7,10; dual 2,5,8,11.
        assert!(
            [0u8, 3, 6, 9]
                .iter()
                .all(|&s| modality(s) == Modality::Movable)
        );
        assert!(
            [1u8, 4, 7, 10]
                .iter()
                .all(|&s| modality(s) == Modality::Fixed)
        );
        assert!(
            [2u8, 5, 8, 11]
                .iter()
                .all(|&s| modality(s) == Modality::Dual)
        );
        // Scorpio and Aquarius carry two lords, every other sign one.
        for s in 0..12u8 {
            let n = rashi_lords(s).len();
            assert_eq!(n, if s == 7 || s == 10 { 2 } else { 1 }, "sign {s}");
        }
        assert_eq!(rashi_lords(7), &[Graha::Mars, Graha::Ketu]);
        assert_eq!(rashi_lords(10), &[Graha::Saturn, Graha::Rahu]);
    }

    // 2. "The sixth graha from" is counted with the starting graha as the
    // first, through the weekday cycle with the nodes after Saturn: from
    // Saturn that is Saturn, Rahu, Ketu, Sun, Moon, Mars; from Rahu it is
    // Rahu, Ketu, Sun, Moon, Mars, Mercury; from Ketu, Ketu, Sun, Moon,
    // Mars, Mercury, Jupiter.
    #[test]
    fn the_sixth_graha_from_the_malefics() {
        assert_eq!(graha_after(Graha::Saturn, 5), Graha::Mars);
        assert_eq!(graha_after(Graha::Rahu, 5), Graha::Mercury);
        assert_eq!(graha_after(Graha::Ketu, 5), Graha::Jupiter);
        for (i, g) in GRAHA_CYCLE.iter().enumerate() {
            assert_eq!(graha_index(*g), i);
        }
    }

    // 3. Placements from longitudes: sign = floor(lon / 30), degrees = lon
    // mod 30, Ketu opposite Rahu. 100 deg is Cancer 10; Rahu at 15 deg is
    // Aries 15 so Ketu is Libra 15.
    #[test]
    fn placements_from_longitudes() {
        let p = NatalPlacements::from_longitudes(
            355.0,
            [10.0, 100.0, 200.0, 250.0, 300.0, 330.0, 359.9, 15.0],
        );
        assert_eq!(p.lagna_sign(), 11);
        assert_eq!(p.sign_of(Graha::Moon), 3);
        assert_eq!(p.degrees_of(Graha::Moon), Some(10.0));
        assert_eq!(p.sign_of(Graha::Rahu), 0);
        assert_eq!(p.sign_of(Graha::Ketu), 6);
        assert_eq!(p.degrees_of(Graha::Ketu), Some(15.0));
        assert_eq!(p.sign_of(Graha::Saturn), 11);
        assert_eq!(p.graha_signs().ketu(), 6);
        // From signs alone nothing is known about degrees.
        assert_eq!(chart_a().degrees_of(Graha::Sun), None);
    }

    // 4. 2.3.6: a rashi with grahas beats one without; 2.3.7: more beats
    // fewer. Chart A: Cancer(3 grahas) > Aries(2) > Gemini(1) > Taurus(0).
    #[test]
    fn rashi_with_more_grahas_is_stronger() {
        let p = chart_a();
        let s = JaiminiStrength::new(&p);
        assert_eq!(s.stronger_rashi(0, 3), 3);
        assert_eq!(s.stronger_rashi(3, 0), 3);
        assert_eq!(s.stronger_rashi(0, 2), 0);
        assert_eq!(s.stronger_rashi(1, 2), 2, "occupied beats empty");
        assert_eq!(s.stronger_rashi(2, 1), 2);
    }

    // 5. 2.3.8: with the same number of grahas, an exalted graha decides.
    // Sun in Aries(0), Moon in Cancer(3), Mars in Leo(4), Mercury in
    // Libra(6), Jupiter in Scorpio(7), Venus in Gemini(2), Saturn in
    // Pisces(11), Rahu in Virgo(5) so Ketu is in Pisces(11). Aries holds one
    // graha (the Sun, exalted there) and Cancer holds one (the Moon, which
    // exalts in Taurus, not here). Both are movable, so only 2.3.8 can
    // separate them.
    #[test]
    fn exalted_graha_breaks_a_tie_in_numbers() {
        let p = NatalPlacements::from_signs(0, signs(0, 3, 4, 6, 7, 2, 11, 5));
        let s = JaiminiStrength::new(&p);
        assert_eq!(p.occupants_of(0), vec![Graha::Sun]);
        assert_eq!(p.occupants_of(3), vec![Graha::Moon]);
        assert_eq!(s.stronger_rashi(0, 3), 0);
        assert_eq!(s.stronger_rashi(3, 0), 0);
    }

    // 5b. 2.3.8 outranks the lords. Capricorn(9) holds Mars alone, exalted
    // there; Cancer(3) holds Mercury alone, not exalted there. Both are
    // movable. Without 2.3.8 the lords would decide the other way: Saturn,
    // Capricorn's lord, stands alone in Aries, while the Moon, Cancer's lord,
    // stands in Gemini with Jupiter and Venus. Mars's exaltation decides.
    #[test]
    fn exaltation_outranks_the_lords() {
        // sun 8, moon 2, mars 9, mercury 3, jupiter 2, venus 2, saturn 0,
        // rahu 5 (Ketu 11).
        let p = NatalPlacements::from_signs(0, signs(8, 2, 9, 3, 2, 2, 0, 5));
        let s = JaiminiStrength::new(&p);
        assert_eq!(p.occupants_of(9), vec![Graha::Mars]);
        assert_eq!(p.occupants_of(3), vec![Graha::Mercury]);
        assert_eq!(s.stronger_rashi(9, 3), 9);
        assert_eq!(s.stronger_rashi(3, 9), 9);
        // The lords alone would favour Cancer.
        assert_eq!(s.stronger_graha(Graha::Saturn, Graha::Moon), Graha::Moon);
    }

    // 5c. 2.3.15: with everything else tied, the rashi whose lord stands in
    // it, and this outranks 2.3.17 (odd over even).
    //
    // Taurus(1, even) holds Venus alone, its own lord at home; Leo(4, odd)
    // holds Mars alone, its lord the Sun standing alone in Aquarius(10).
    // Counts are 1 and 1; neither holds an exalted graha (Venus is not
    // exalted in Taurus, Mars not in Leo); both are fixed; the lords Venus
    // and the Sun each stand alone, unexalted, in a fixed rashi, so the graha
    // tests tie; and then Taurus, whose lord is at home, beats odd Leo.
    #[test]
    fn a_lord_at_home_decides_when_the_rest_ties() {
        // sun 10, moon 8, mars 4, mercury 2, jupiter 5, venus 1, saturn 11,
        // rahu 9 (Ketu 3).
        let p = NatalPlacements::from_signs(0, signs(10, 8, 4, 2, 5, 1, 11, 9));
        let s = JaiminiStrength::new(&p);
        assert_eq!(p.occupants_of(1), vec![Graha::Venus]);
        assert_eq!(p.occupants_of(4), vec![Graha::Mars]);
        assert_eq!(p.companions_of(Graha::Sun), 0);
        assert_eq!(s.stronger_rashi(4, 1), 1);
        assert_eq!(s.stronger_rashi(1, 4), 1);
    }

    // 6. 2.3.9: with nothing to separate them in numbers or exaltation,
    // natural strength: dual over fixed. In Chart A Taurus(1, fixed) and
    // Virgo(5, dual) are both empty.
    #[test]
    fn natural_strength_orders_empty_rashis() {
        let p = chart_a();
        let s = JaiminiStrength::new(&p);
        assert!(p.occupants_of(1).is_empty() && p.occupants_of(5).is_empty());
        assert_eq!(s.stronger_rashi(1, 5), 5, "dual over fixed");
        assert_eq!(s.stronger_rashi(5, 1), 5);
    }

    // 7. Chart C has empty rashis of each modality: everything in Aries and
    // Taurus. Empty: Gemini(dual) Cancer(movable) Leo(fixed) ...
    #[test]
    fn natural_strength_across_all_three_modalities() {
        let p = NatalPlacements::from_signs(0, signs(0, 0, 0, 0, 0, 0, 0, 1));
        // Rahu in Taurus(1), Ketu in Scorpio(7); all others in Aries.
        let s = JaiminiStrength::new(&p);
        // Cancer(3, movable, empty) vs Leo(4, fixed, empty) vs Gemini(2,
        // dual, empty).
        assert_eq!(s.stronger_rashi(3, 4), 4, "fixed over movable");
        assert_eq!(s.stronger_rashi(4, 2), 2, "dual over fixed");
        assert_eq!(s.stronger_rashi(3, 2), 2, "dual over movable");
    }

    // 8. 2.3.10: when presence, number, exaltation and nature all tie, the
    // lords are compared. Chart C again: Cancer(3) and Libra(6) are both
    // empty movable signs. Cancer's lord is the Moon (in Aries with six
    // companions), Libra's lord Venus (also in Aries, six companions too:
    // the Sun, Moon, Mars, Mercury, Jupiter, Saturn are all there with her).
    // Moon: companions = Sun, Mars, Mercury, Jupiter, Venus, Saturn = 6.
    // Venus: companions = Sun, Moon, Mars, Mercury, Jupiter, Saturn = 6.
    // Tied on companions; Moon and Venus are not exalted in Aries; same
    // rashi so natural strength ties; 2.3.11 needs degrees, absent here. The
    // graha comparison ties, the lord step falls through, neither lord
    // stands in its own rashi, and 2.3.17 decides: Libra(6) is odd and
    // Cancer(3) is even.
    #[test]
    fn lords_then_odd_over_even() {
        let p = NatalPlacements::from_signs(0, signs(0, 0, 0, 0, 0, 0, 0, 1));
        let s = JaiminiStrength::new(&p);
        assert_eq!(s.stronger_rashi(3, 6), 6);
        assert_eq!(s.stronger_rashi(6, 3), 6);
    }

    // 9. The lords step can decide. Chart D: Sun in Leo(4) alone and Moon in
    // Cancer(3) with Mars. Compare the empty movable signs Libra(6, lord
    // Venus) and Capricorn(9, lord Saturn) by their lords. Venus in Aries
    // with Jupiter and Mercury (2 companions); Saturn in Gemini alone.
    // Venus has companions, Saturn has none, so Libra is stronger even
    // though Capricorn is the later sign.
    #[test]
    fn the_stronger_lord_makes_the_stronger_rashi() {
        // sun 4, moon 3, mars 3, mercury 0, jupiter 0, venus 0, saturn 2,
        // rahu 11 (so Ketu is in Virgo(5)).
        let p = NatalPlacements::from_signs(0, signs(4, 3, 3, 0, 0, 0, 2, 11));
        let s = JaiminiStrength::new(&p);
        assert!(p.occupants_of(6).is_empty() && p.occupants_of(9).is_empty());
        assert_eq!(p.companions_of(Graha::Venus), 2);
        assert_eq!(p.companions_of(Graha::Saturn), 0);
        assert_eq!(s.stronger_rashi(6, 9), 6);
        assert_eq!(s.stronger_rashi(9, 6), 6);
    }

    // 10. Graha tests: companions first (agrahat sagraho jyayan).
    #[test]
    fn graha_with_companions_is_stronger() {
        let p = chart_a();
        let s = JaiminiStrength::new(&p);
        // Jupiter is in Cancer with Mars and Mercury; Venus is alone in
        // Libra; Saturn is alone in Capricorn.
        assert_eq!(
            s.stronger_graha(Graha::Venus, Graha::Jupiter),
            Graha::Jupiter
        );
        assert_eq!(
            s.stronger_graha(Graha::Jupiter, Graha::Venus),
            Graha::Jupiter
        );
        // Sun has one companion (Moon), Mars has two.
        assert_eq!(s.stronger_graha(Graha::Sun, Graha::Mars), Graha::Mars);
    }

    // 11. Alone and alone: exaltation, then the rashi's nature, then the
    // degrees. Chart E: Saturn alone in Libra(6) (exalted), Venus alone in
    // Virgo(5) (debilitated, dual), Mars alone in Taurus(1), Jupiter alone
    // in Aquarius(10), Sun alone in Capricorn(9), Moon alone in Scorpio(7),
    // Mercury alone in Sagittarius(8), Rahu alone in Leo(4) so Ketu is in
    // Aquarius(10) with Jupiter.
    #[test]
    fn alone_graha_exaltation_then_nature() {
        let p = NatalPlacements::from_signs(0, signs(9, 7, 1, 8, 10, 5, 6, 4));
        let s = JaiminiStrength::new(&p);
        // Saturn exalted, Venus not (Venus exalts in Pisces): Saturn wins
        // even though Venus stands in a dual rashi.
        assert_eq!(p.companions_of(Graha::Saturn), 0);
        assert_eq!(p.companions_of(Graha::Venus), 0);
        assert_eq!(s.stronger_graha(Graha::Venus, Graha::Saturn), Graha::Saturn);
        // Neither exalted: Mars (Taurus, fixed) against Mercury (Sagittarius,
        // dual). Mars has no companions, Mercury has none.
        assert_eq!(p.companions_of(Graha::Mars), 0);
        assert_eq!(p.companions_of(Graha::Mercury), 0);
        assert_eq!(
            s.stronger_graha(Graha::Mars, Graha::Mercury),
            Graha::Mercury
        );
        assert_eq!(
            s.stronger_graha(Graha::Mercury, Graha::Mars),
            Graha::Mercury
        );
    }

    // 12. 2.3.11: two grahas in one rashi, the further advanced wins; a node
    // counts its degrees from the far end. Mars at 10 and Mercury at 20 deg
    // of Gemini (neither exalted there, one companion each): Mercury. The
    // Moon at 10 deg of Taurus and Rahu at 20 deg of Taurus (both exalted
    // there under the crate's node pairing): Rahu counts as 30 - 20 = 10, so
    // they tie and the first argument stands. With Rahu at 5 deg it counts
    // as 25 and beats the Moon.
    #[test]
    fn degrees_decide_between_grahas_in_one_rashi() {
        // Sun 10 (Aries), Moon 40 (Taurus 10), Mars 70 (Gemini 10), Mercury 80
        // (Gemini 20), Jupiter 100, Venus 130, Saturn 160, Rahu 50 (Taurus 20).
        let p = NatalPlacements::from_longitudes(
            0.0,
            [10.0, 40.0, 70.0, 80.0, 100.0, 130.0, 160.0, 50.0],
        );
        let s = JaiminiStrength::new(&p);
        assert_eq!(
            s.stronger_graha(Graha::Mars, Graha::Mercury),
            Graha::Mercury
        );
        assert_eq!(
            s.stronger_graha(Graha::Mercury, Graha::Mars),
            Graha::Mercury
        );
        assert_eq!(s.stronger_graha(Graha::Moon, Graha::Rahu), Graha::Moon);
        assert_eq!(s.stronger_graha(Graha::Rahu, Graha::Moon), Graha::Rahu);
        let q = NatalPlacements::from_longitudes(
            0.0,
            [10.0, 40.0, 70.0, 80.0, 100.0, 130.0, 160.0, 35.0],
        );
        let t = JaiminiStrength::new(&q);
        assert_eq!(t.stronger_graha(Graha::Moon, Graha::Rahu), Graha::Rahu);
    }

    // 13. Equal arguments return themselves; a total tie goes to the first.
    #[test]
    fn identity_and_total_tie() {
        let p = chart_a();
        let s = JaiminiStrength::new(&p);
        assert_eq!(s.stronger_rashi(5, 5), 5);
        assert_eq!(s.stronger_graha(Graha::Sun, Graha::Sun), Graha::Sun);
        // Mars and Mercury share Cancer with Jupiter; neither is exalted
        // there and no degrees are known, so nothing separates them.
        assert_eq!(s.stronger_graha(Graha::Mars, Graha::Mercury), Graha::Mars);
        assert_eq!(
            s.stronger_graha(Graha::Mercury, Graha::Mars),
            Graha::Mercury
        );
    }
}
