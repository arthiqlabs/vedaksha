// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedic planetary aspects (drishti).
//!
//! In Vedic astrology, all planets aspect the 7th house from their position
//! (180° opposition). Mars, Jupiter, and Saturn have additional special aspects.
//!
//! [`aspect_strength`] and [`find_vedic_aspects`] are sign-based (whole-sign),
//! not degree-based. A planet in sign X aspects the entire sign that is N
//! houses away.
//!
//! All planets also have graded aspects on houses 3,10 (Quarter/25%),
//! 4,8 (ThreeQuarter/75%), and 5,9 (Half/50%). Mars, Jupiter, and Saturn
//! override their special houses to Full (100%).
//!
//! Source: BPHS Ch. 26 Sl. 2-7.
//!
//! # Sputa (degree) drishti
//!
//! [`sputa_drishti`] and [`sputa_drishti_graha`] give the aspect, in virupas
//! (0 to 60, 60 being a full rupa), of one longitude on another, so the
//! aspect on a bhava madhya or on a graha at a given degree is a number
//! rather than a sign count. The rules are BPHS's own *sphuta drishti*
//! verses (chapter 26).
//!
//! **Numbering and grades.** Verse numbers are those of the recension that
//! carries this chapter as chapter 26 and the shadbala chapter as chapter 27;
//! the bhava bala and vimshopaka citations in this crate follow the same
//! numbering. A second family of manuscripts, the Uttarabhaga recension with
//! the Subodhini commentary, carries the generic verses as its chapter 2,
//! verses 3 to 6, and differs on the special aspects (see below). Grades:
//! (a) read at the verse in two or more editions that agree; (b) read at the
//! verse in one recension only; (c) a choice between readings the verse or
//! the editions leave open.
//!
//! * **The generic rule, BPHS 26.6 to 26.8 (grade a).** Let `x` be the
//!   longitude of the aspected point less that of the aspecting graha, in
//!   `[0, 360)`. The verses cascade on the size of `x`. Over six signs
//!   (`x` over 180 degrees): take `x` from ten signs (300 degrees) and halve
//!   it. Over five signs: drop the whole signs and double the degrees. Over
//!   four: take the degrees from five signs' worth (150 degrees minus `x`).
//!   Over three: take `x` from four signs, halve it and add 30. Over two:
//!   drop the signs and add 15 to the degrees. Over one: drop the signs and
//!   halve the degrees. At one sign or less the verses give nothing, and the
//!   same edition's worked example treats a difference over ten signs as nil
//!   (the ten-sign subtraction would go negative); this module does both.
//!   Every branch meets its neighbours, so the curve is continuous: 0 at 30
//!   degrees, 15 at 60, 45 at 90, 30 at 120, 0 at 150, 60 at 180 (the 7th,
//!   full), 0 at 300.
//! * **The special aspects of Saturn, Mars and Jupiter, BPHS 26.9 to 26.12
//!   (grade a for the structure, c for two details).** For named values of
//!   `c`, the whole signs in `x`, with `d` the degrees past them, these
//!   replace the generic value. Saturn: `c = 1` gives `2d`; `c = 9` gives
//!   twice the degrees still to run, `2(30 - d)`; `c = 2` gives `60 - d/2`;
//!   `c = 8` gives `d + 30`. Mars: `c = 3` or `7` gives `60 - d`; `c = 2`
//!   gives `1.5d + 15`; `c = 6` gives 60. Jupiter: `c = 3` or `7` gives
//!   `45 + d/2`; `c = 4` or `8` gives `60 - 2d`. Two details are choices
//!   (grade c). Jupiter's `c = 4`/`8` verse reads "less the doubled degrees"
//!   (`dvigunair lavair unah`); some printed renderings drop the doubling,
//!   but only with the doubling does the curve reach 0 at 150 degrees where
//!   the generic curve starts from 0. And Jupiter's `c = 3`/`7` verse names
//!   half of the *elapsed* degrees (`bhoga`): only that reading meets the
//!   neighbouring values at 90, 120, 210 and 240 degrees. With these
//!   readings Saturn's and Mars's curves are continuous and no value
//!   exceeds 60. Jupiter's is continuous except at 270 degrees, where the
//!   `c = 8` value has fallen to 0 and the generic `c = 9` value starts at
//!   15: that step is in the verses as read.
//! * **The other recension, [`SputaSpecialRule::AddedSteps`] (grade b).** The
//!   Uttarabhaga recension (its chapter 2, verses 6 and 7) has no special
//!   curves. It adds 45, 30 and 15 virupas to the generic value for Saturn
//!   in the 3rd or 10th sign from himself, Jupiter in the 5th or 9th and
//!   Mars in the 4th or 8th ("in order", `sharavedah kharamashcha tithayo
//!   yojitah kramat`), the places being signs counted from the aspecting
//!   graha. That sum can pass 60 (Saturn, aspected point in the 3rd sign
//!   and 20 degrees past the aspector's degree: 35 + 45 = 80). It is kept as
//!   the recension reads. The recensions disagree and no source consulted
//!   settles which is BPHS's, so both are offered; the default is the one
//!   whose special aspects join the generic curve and stay within a rupa.
//! * **The nodes.** The verses name the seven grahas. For Rahu and Ketu
//!   [`sputa_drishti_graha`] returns the generic value under either rule.

/// Vedic aspect strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectStrength {
    /// Full aspect (100% strength)
    Full,
    /// Three-quarter aspect (75%)
    ThreeQuarter,
    /// Half aspect (50%)
    Half,
    /// Quarter aspect (25%)
    Quarter,
    /// No aspect
    None,
}

/// A Vedic planet for drishti purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VedicPlanet {
    /// Sun (Surya)
    Sun,
    /// Moon (Chandra)
    Moon,
    /// Mars (Mangala)
    Mars,
    /// Mercury (Budha)
    Mercury,
    /// Jupiter (Guru)
    Jupiter,
    /// Venus (Shukra)
    Venus,
    /// Saturn (Shani)
    Saturn,
    /// Rahu (north lunar node)
    Rahu,
    /// Ketu (south lunar node)
    Ketu,
}

/// A detected Vedic aspect.
#[derive(Debug, Clone)]
pub struct VedicAspect {
    /// The planet casting the aspect
    pub aspecting_planet: VedicPlanet,
    /// Sign index (0-11) of the aspecting planet
    pub aspecting_sign: u8,
    /// Sign index (0-11) being aspected
    pub aspected_sign: u8,
    /// Strength of the aspect
    pub strength: AspectStrength,
    /// Number of houses away (1-12)
    pub houses_away: u8,
}

/// Get the aspect strength of a planet on a sign that is `houses_away` from it.
///
/// All planets have a full aspect on the 7th house from their position.
/// Mars additionally aspects the 4th and 8th, Jupiter the 5th and 9th,
/// and Saturn the 3rd and 10th.
///
/// Source: BPHS Ch. 26.
#[must_use]
pub fn aspect_strength(planet: VedicPlanet, houses_away: u8) -> AspectStrength {
    // Standard 7th house aspect for all planets (BPHS Ch. 26 Sl. 2)
    if houses_away == 7 {
        return AspectStrength::Full;
    }

    // Graded aspects per BPHS Ch. 26 Sl. 2-7:
    // Houses 3,10 → Quarter (25%); Saturn overrides to Full
    // Houses 4,8  → ThreeQuarter (75%); Mars overrides to Full
    // Houses 5,9  → Half (50%); Jupiter overrides to Full
    match houses_away {
        3 | 10 => match planet {
            VedicPlanet::Saturn => AspectStrength::Full,
            _ => AspectStrength::Quarter,
        },
        4 | 8 => match planet {
            VedicPlanet::Mars => AspectStrength::Full,
            _ => AspectStrength::ThreeQuarter,
        },
        5 | 9 => match planet {
            VedicPlanet::Jupiter => AspectStrength::Full,
            _ => AspectStrength::Half,
        },
        _ => AspectStrength::None,
    }
}

/// Find all Vedic aspects from a set of planet positions.
///
/// Takes planet positions as `(VedicPlanet, sign_index 0-11)` pairs and
/// returns all non-zero aspects cast by each planet.
#[must_use]
pub fn find_vedic_aspects(planets: &[(VedicPlanet, u8)]) -> Vec<VedicAspect> {
    let mut aspects = Vec::new();
    for &(planet, sign) in planets {
        for houses in 1_u8..=12 {
            let strength = aspect_strength(planet, houses);
            if strength != AspectStrength::None {
                aspects.push(VedicAspect {
                    aspecting_planet: planet,
                    aspecting_sign: sign,
                    // Houses are counted inclusively in Jyotish: the sign a
                    // graha occupies is the 1st, so the 7th from Aries is
                    // Libra. Hence `houses - 1`, matching the 1-based house
                    // numbers `aspect_strength` grades.
                    aspected_sign: (sign + houses - 1) % 12,
                    strength,
                    houses_away: houses,
                });
            }
        }
    }
    aspects
}

/// Which recension's rule governs the special aspects of Saturn, Jupiter and
/// Mars in [`sputa_drishti_graha`]. See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SputaSpecialRule {
    /// BPHS 26.9 to 26.12: for named whole-sign remainders the special curve
    /// replaces the generic value, and no value passes 60. The default.
    #[default]
    PerSignRemainder,
    /// The Uttarabhaga recension: 45 (Saturn, 3rd and 10th sign), 30 (Jupiter,
    /// 5th and 9th) and 15 (Mars, 4th and 8th) are added to the generic value.
    AddedSteps,
}

impl From<crate::graha::Graha> for VedicPlanet {
    fn from(g: crate::graha::Graha) -> Self {
        use crate::graha::Graha;
        match g {
            Graha::Sun => Self::Sun,
            Graha::Moon => Self::Moon,
            Graha::Mars => Self::Mars,
            Graha::Mercury => Self::Mercury,
            Graha::Jupiter => Self::Jupiter,
            Graha::Venus => Self::Venus,
            Graha::Saturn => Self::Saturn,
            Graha::Rahu => Self::Rahu,
            Graha::Ketu => Self::Ketu,
        }
    }
}

/// Whole signs and degrees past them in the separation `x` of the aspected
/// point from the aspecting graha: `(c, d)` with `c` in `0..=11`, `d` in
/// `[0, 30)`.
fn split_separation(drashta_longitude_deg: f64, drishya_longitude_deg: f64) -> (u8, f64) {
    let mut x = (drishya_longitude_deg - drashta_longitude_deg).rem_euclid(360.0);
    // `rem_euclid` can return exactly 360.0 for a tiny negative difference.
    if x >= 360.0 {
        x -= 360.0;
    }
    let c = ((x / 30.0).floor() as u8).min(11);
    (c, x - 30.0 * f64::from(c))
}

/// The generic cascade of BPHS 26.6 to 26.8 for `c` whole signs and `d`
/// degrees past them. Each arm is the verse's own arithmetic written for the
/// sign band it governs (the verses test "over N signs" from the top down, so
/// a separation of `c` signs and `d` degrees falls in exactly one band).
fn generic_virupa(c: u8, d: f64) -> f64 {
    match c {
        // Over one sign: the degrees past the first sign, halved.
        1 => d / 2.0,
        // Over two: the degrees past two signs, plus 15.
        2 => d + 15.0,
        // Over three: four signs less x is 30 - d; halved, plus 30.
        3 => (30.0 - d) / 2.0 + 30.0,
        // Over four: five signs less x is 30 - d.
        4 => 30.0 - d,
        // Over five: the degrees past five signs, doubled.
        5 => 2.0 * d,
        // Over six: ten signs (300 degrees) less x, halved. With
        // x = 30c + d this is 150 - 15c - d/2, for c = 6 to 9; at ten signs
        // and beyond the commentary reads nil.
        6..=9 => 150.0 - 15.0 * f64::from(c) - d / 2.0,
        // One sign or less, and ten signs or more: no aspect.
        _ => 0.0,
    }
}

/// The special curves of BPHS 26.9 to 26.12, or `None` where the generic
/// value stands.
fn special_virupa(planet: VedicPlanet, c: u8, d: f64) -> Option<f64> {
    match (planet, c) {
        // 26.9-10: Saturn.
        (VedicPlanet::Saturn, 1) => Some(2.0 * d),
        (VedicPlanet::Saturn, 9) => Some(2.0 * (30.0 - d)),
        (VedicPlanet::Saturn, 2) => Some(60.0 - d / 2.0),
        (VedicPlanet::Saturn, 8) => Some(d + 30.0),
        // 26.11: Mars.
        (VedicPlanet::Mars, 3 | 7) => Some(60.0 - d),
        (VedicPlanet::Mars, 2) => Some(1.5 * d + 15.0),
        (VedicPlanet::Mars, 6) => Some(60.0),
        // 26.12: Jupiter.
        (VedicPlanet::Jupiter, 3 | 7) => Some(45.0 + d / 2.0),
        (VedicPlanet::Jupiter, 4 | 8) => Some(60.0 - 2.0 * d),
        _ => None,
    }
}

/// The aspect, in virupas (0 to 60), that the point at `drashta_longitude_deg`
/// casts on the point at `drishya_longitude_deg`, by the generic sphuta
/// drishti rule of BPHS 26.6 to 26.8.
///
/// Both longitudes are in degrees in any consistent frame (sidereal for
/// Jyotish) and are normalised here. The aspected point may be a graha or a
/// bhava madhya. The special aspects of Saturn, Mars and Jupiter are in
/// [`sputa_drishti_graha`]. See the module documentation for the verses.
///
/// The value is 0 for a separation of 30 degrees or less and of 300 degrees
/// or more, 60 at exactly 180 degrees.
#[must_use]
pub fn sputa_drishti(drashta_longitude_deg: f64, drishya_longitude_deg: f64) -> f64 {
    let (c, d) = split_separation(drashta_longitude_deg, drishya_longitude_deg);
    generic_virupa(c, d)
}

/// The aspect, in virupas, that `planet` at `drashta_longitude_deg` casts on
/// `drishya_longitude_deg`: [`sputa_drishti`] with the special aspects of
/// Saturn, Mars and Jupiter applied under `rule`.
///
/// For every other graha, and for Rahu and Ketu, this is the generic value
/// under either rule. See the module documentation for the verses and for the
/// two recensions `rule` chooses between.
#[must_use]
pub fn sputa_drishti_graha(
    planet: VedicPlanet,
    drashta_longitude_deg: f64,
    drishya_longitude_deg: f64,
    rule: SputaSpecialRule,
) -> f64 {
    let (c, d) = split_separation(drashta_longitude_deg, drishya_longitude_deg);
    match rule {
        SputaSpecialRule::PerSignRemainder => {
            special_virupa(planet, c, d).unwrap_or_else(|| generic_virupa(c, d))
        }
        SputaSpecialRule::AddedSteps => {
            // Places are signs counted inclusively from the aspecting graha:
            // the sign it stands in is the 1st.
            let sign_of = |lon: f64| ((lon.rem_euclid(360.0) / 30.0).floor() as i32).rem_euclid(12);
            let house = (sign_of(drishya_longitude_deg) - sign_of(drashta_longitude_deg))
                .rem_euclid(12)
                + 1;
            let step = match (planet, house) {
                (VedicPlanet::Saturn, 3 | 10) => 45.0,
                (VedicPlanet::Jupiter, 5 | 9) => 30.0,
                (VedicPlanet::Mars, 4 | 8) => 15.0,
                _ => 0.0,
            };
            generic_virupa(c, d) + step
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_planets_aspect_7th_with_full_strength() {
        let planets = [
            VedicPlanet::Sun,
            VedicPlanet::Moon,
            VedicPlanet::Mars,
            VedicPlanet::Mercury,
            VedicPlanet::Jupiter,
            VedicPlanet::Venus,
            VedicPlanet::Saturn,
            VedicPlanet::Rahu,
            VedicPlanet::Ketu,
        ];
        for planet in planets {
            assert_eq!(
                aspect_strength(planet, 7),
                AspectStrength::Full,
                "{planet:?} should have Full strength on 7th house"
            );
        }
    }

    #[test]
    fn mars_special_aspects() {
        assert_eq!(aspect_strength(VedicPlanet::Mars, 4), AspectStrength::Full);
        assert_eq!(aspect_strength(VedicPlanet::Mars, 7), AspectStrength::Full);
        assert_eq!(aspect_strength(VedicPlanet::Mars, 8), AspectStrength::Full);
        // Graded aspects on non-special houses
        assert_eq!(
            aspect_strength(VedicPlanet::Mars, 3),
            AspectStrength::Quarter
        );
        assert_eq!(aspect_strength(VedicPlanet::Mars, 5), AspectStrength::Half);
        // No aspect on unaspected houses
        assert_eq!(aspect_strength(VedicPlanet::Mars, 1), AspectStrength::None);
    }

    #[test]
    fn jupiter_special_aspects() {
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 5),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 7),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 9),
            AspectStrength::Full
        );
        // Graded aspects on non-special houses
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 4),
            AspectStrength::ThreeQuarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 8),
            AspectStrength::ThreeQuarter
        );
    }

    #[test]
    fn saturn_special_aspects() {
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 3),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 7),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 10),
            AspectStrength::Full
        );
        // Graded aspects on non-special houses
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 4),
            AspectStrength::ThreeQuarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 9),
            AspectStrength::Half
        );
    }

    #[test]
    fn sun_graded_aspects() {
        // Sun has graded aspects per BPHS Ch. 26
        assert_eq!(aspect_strength(VedicPlanet::Sun, 7), AspectStrength::Full);
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 3),
            AspectStrength::Quarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 10),
            AspectStrength::Quarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 4),
            AspectStrength::ThreeQuarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 8),
            AspectStrength::ThreeQuarter
        );
        assert_eq!(aspect_strength(VedicPlanet::Sun, 5), AspectStrength::Half);
        assert_eq!(aspect_strength(VedicPlanet::Sun, 9), AspectStrength::Half);
        // No aspect on remaining houses
        assert_eq!(aspect_strength(VedicPlanet::Sun, 1), AspectStrength::None);
        assert_eq!(aspect_strength(VedicPlanet::Sun, 2), AspectStrength::None);
        assert_eq!(aspect_strength(VedicPlanet::Sun, 6), AspectStrength::None);
        assert_eq!(aspect_strength(VedicPlanet::Sun, 11), AspectStrength::None);
        assert_eq!(aspect_strength(VedicPlanet::Sun, 12), AspectStrength::None);
    }

    #[test]
    fn all_planets_full_on_7th() {
        let planets = [
            VedicPlanet::Sun,
            VedicPlanet::Moon,
            VedicPlanet::Mars,
            VedicPlanet::Mercury,
            VedicPlanet::Jupiter,
            VedicPlanet::Venus,
            VedicPlanet::Saturn,
        ];
        for planet in planets {
            assert_eq!(
                aspect_strength(planet, 7),
                AspectStrength::Full,
                "{planet:?} should have Full strength on 7th house"
            );
        }
    }

    #[test]
    fn mars_full_on_4th_and_8th() {
        assert_eq!(aspect_strength(VedicPlanet::Mars, 4), AspectStrength::Full);
        assert_eq!(aspect_strength(VedicPlanet::Mars, 8), AspectStrength::Full);
    }

    #[test]
    fn jupiter_full_on_5th_and_9th() {
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 5),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Jupiter, 9),
            AspectStrength::Full
        );
    }

    #[test]
    fn saturn_full_on_3rd_and_10th() {
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 3),
            AspectStrength::Full
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Saturn, 10),
            AspectStrength::Full
        );
    }

    #[test]
    fn generic_planet_quarter_on_3rd_10th() {
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 3),
            AspectStrength::Quarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 10),
            AspectStrength::Quarter
        );
    }

    #[test]
    fn generic_planet_threequarter_on_4th_8th() {
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 4),
            AspectStrength::ThreeQuarter
        );
        assert_eq!(
            aspect_strength(VedicPlanet::Sun, 8),
            AspectStrength::ThreeQuarter
        );
    }

    #[test]
    fn generic_planet_half_on_5th_9th() {
        assert_eq!(aspect_strength(VedicPlanet::Venus, 9), AspectStrength::Half);
        assert_eq!(aspect_strength(VedicPlanet::Venus, 5), AspectStrength::Half);
    }

    #[test]
    fn find_vedic_aspects_mars_produces_seven_aspects() {
        // Mars now casts 7 aspects: 3(Q),4(F),5(H),7(F),8(F),9(H),10(Q)
        let planets = [(VedicPlanet::Mars, 0_u8)];
        let aspects = find_vedic_aspects(&planets);
        assert_eq!(
            aspects.len(),
            7,
            "Mars should cast exactly 7 non-None aspects"
        );
        let houses: Vec<u8> = aspects.iter().map(|a| a.houses_away).collect();
        assert!(houses.contains(&4));
        assert!(houses.contains(&7));
        assert!(houses.contains(&8));
    }

    #[test]
    fn find_vedic_aspects_jupiter_produces_seven_aspects() {
        let planets = [(VedicPlanet::Jupiter, 2_u8)];
        let aspects = find_vedic_aspects(&planets);
        assert_eq!(
            aspects.len(),
            7,
            "Jupiter should cast exactly 7 non-None aspects"
        );
    }

    #[test]
    fn find_vedic_aspects_sun_produces_seven_aspects() {
        // Sun now casts 7 aspects: 3(Q),4(TQ),5(H),7(F),8(TQ),9(H),10(Q)
        let planets = [(VedicPlanet::Sun, 3_u8)];
        let aspects = find_vedic_aspects(&planets);
        assert_eq!(
            aspects.len(),
            7,
            "Sun should cast exactly 7 non-None aspects"
        );
    }

    #[test]
    fn find_vedic_aspects_aspected_sign_wraps_correctly() {
        // Houses count inclusively, so from Aquarius (10) the 4th is Taurus
        // (10→11→0→1), the 7th is Leo (4) and the 8th is Virgo (5).
        let planets = [(VedicPlanet::Mars, 10_u8)];
        let aspects = find_vedic_aspects(&planets);
        let aspected: Vec<u8> = aspects.iter().map(|a| a.aspected_sign).collect();
        assert!(aspected.contains(&1), "4th house aspect should be Taurus");
        assert!(aspected.contains(&4), "7th house aspect should be Leo");
        assert!(aspected.contains(&5), "8th house aspect should be Virgo");
    }

    /// The 7th-from-self aspect is the one every graha casts, so it pins the
    /// counting convention: from Aries the 7th is Libra, not Scorpio. This is
    /// the assertion whose absence let an off-by-one live in `aspected_sign`
    /// while every `aspect_strength` test passed.
    #[test]
    fn seventh_aspect_lands_on_the_opposite_sign() {
        for (sign, expected) in [(0_u8, 6_u8), (3, 9), (6, 0), (11, 5)] {
            let aspects = find_vedic_aspects(&[(VedicPlanet::Sun, sign)]);
            let seventh = aspects
                .iter()
                .find(|a| a.houses_away == 7)
                .expect("every graha aspects the 7th");
            assert_eq!(
                seventh.aspected_sign, expected,
                "7th from sign {sign} should be {expected}"
            );
            assert_eq!(seventh.strength, AspectStrength::Full);
        }
    }

    #[test]
    fn special_aspects_land_on_the_classical_signs() {
        // From Aries (0): Saturn 3rd = Gemini (2), 10th = Capricorn (9);
        // Jupiter 5th = Leo (4), 9th = Sagittarius (8);
        // Mars 4th = Cancer (3), 8th = Scorpio (7).
        let full_targets = |planet: VedicPlanet| -> Vec<u8> {
            let mut v: Vec<u8> = find_vedic_aspects(&[(planet, 0)])
                .iter()
                .filter(|a| a.strength == AspectStrength::Full)
                .map(|a| a.aspected_sign)
                .collect();
            v.sort_unstable();
            v
        };
        assert_eq!(full_targets(VedicPlanet::Saturn), vec![2, 6, 9]);
        assert_eq!(full_targets(VedicPlanet::Jupiter), vec![4, 6, 8]);
        assert_eq!(full_targets(VedicPlanet::Mars), vec![3, 6, 7]);
    }

    #[test]
    fn a_graha_never_aspects_its_own_sign() {
        for sign in 0_u8..12 {
            for planet in [
                VedicPlanet::Sun,
                VedicPlanet::Mars,
                VedicPlanet::Jupiter,
                VedicPlanet::Saturn,
            ] {
                assert!(
                    find_vedic_aspects(&[(planet, sign)])
                        .iter()
                        .all(|a| a.aspected_sign != sign),
                    "{planet:?} in sign {sign} must not aspect itself"
                );
            }
        }
    }

    // ---- sputa (degree) drishti ------------------------------------------
    //
    // Every expected value below is worked by hand from the verse the test
    // names (BPHS 26.6-26.12), not read from a table and not produced by the
    // code under test. `x` is the aspected longitude less the aspecting one.

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// 26.6-26.8, one vector per branch of the cascade, each mid-branch so a
    /// wrong band edge cannot hide.
    #[test]
    fn generic_cascade_hand_vectors() {
        // x = 20: one sign or less, the verses give nothing.
        assert!(close(sputa_drishti(0.0, 20.0), 0.0));
        // x = 45: over one sign; drop the sign, halve the 15 degrees = 7.5.
        assert!(close(sputa_drishti(0.0, 45.0), 7.5));
        // x = 75: over two; drop the two signs, 15 degrees + 15 = 30.
        assert!(close(sputa_drishti(0.0, 75.0), 30.0));
        // x = 100: over three; four signs (120) - 100 = 20, halved 10, + 30 = 40.
        assert!(close(sputa_drishti(0.0, 100.0), 40.0));
        // x = 135: over four; five signs (150) - 135 = 15.
        assert!(close(sputa_drishti(0.0, 135.0), 15.0));
        // x = 165: over five; drop the five signs, 15 degrees doubled = 30.
        assert!(close(sputa_drishti(0.0, 165.0), 30.0));
        // x = 180: the 7th, a full rupa.
        assert!(close(sputa_drishti(0.0, 180.0), 60.0));
        // x = 200: over six; ten signs (300) - 200 = 100, halved = 50.
        assert!(close(sputa_drishti(0.0, 200.0), 50.0));
        // x = 250: 300 - 250 = 50, halved = 25.
        assert!(close(sputa_drishti(0.0, 250.0), 25.0));
        // x = 299: 300 - 299 = 1, halved = 0.5.
        assert!(close(sputa_drishti(0.0, 299.0), 0.5));
        // x = 300 and beyond ten signs: nil.
        assert!(close(sputa_drishti(0.0, 300.0), 0.0));
        assert!(close(sputa_drishti(0.0, 330.0), 0.0));
    }

    /// The curve depends only on the difference, and both longitudes are
    /// normalised: 350 -> 50 is a difference of 60, as is 10 -> 70, and a
    /// longitude of 360 + 70 is the same point as 70.
    #[test]
    fn sputa_drishti_depends_only_on_the_difference() {
        let want = 15.0; // x = 60: over two signs, 0 degrees + 15.
        assert!(close(sputa_drishti(10.0, 70.0), want));
        assert!(close(sputa_drishti(350.0, 50.0), want));
        assert!(close(sputa_drishti(10.0, 430.0), want));
        assert!(close(sputa_drishti(-10.0, 50.0), want));
    }

    /// Between the branch edges the curve must be continuous: every verse
    /// of the cascade meets its neighbour. A wrong constant in any one arm
    /// opens a gap at that arm's edge.
    #[test]
    fn generic_curve_is_continuous_and_bounded() {
        let eps = 1e-7;
        for k in 1..=10 {
            let edge = 30.0 * f64::from(k);
            let below = sputa_drishti(0.0, edge - eps);
            let above = sputa_drishti(0.0, edge + eps);
            assert!(
                (below - above).abs() < 1e-5,
                "gap at {edge} degrees: {below} vs {above}"
            );
        }
        let mut x = 0.0;
        while x < 360.0 {
            let v = sputa_drishti(0.0, x);
            assert!((0.0..=60.0).contains(&v), "x={x}: {v} out of 0..=60");
            x += 0.25;
        }
    }

    /// 26.2-26.12 agree that every graha aspects the 7th fully, whatever the
    /// recension: at x = 180 both rules give 60 for every graha.
    #[test]
    fn every_graha_casts_a_full_rupa_on_the_exact_opposite() {
        for g in [
            VedicPlanet::Sun,
            VedicPlanet::Moon,
            VedicPlanet::Mars,
            VedicPlanet::Mercury,
            VedicPlanet::Jupiter,
            VedicPlanet::Venus,
            VedicPlanet::Saturn,
        ] {
            for rule in [
                SputaSpecialRule::PerSignRemainder,
                SputaSpecialRule::AddedSteps,
            ] {
                assert!(
                    close(sputa_drishti_graha(g, 12.0, 192.0, rule), 60.0),
                    "{g:?} {rule:?}"
                );
            }
        }
    }

    /// 26.9-26.10, Saturn. c = whole signs in x, d = degrees past them.
    #[test]
    fn saturn_special_aspects_hand_vectors() {
        let r = SputaSpecialRule::PerSignRemainder;
        let s = |x: f64| sputa_drishti_graha(VedicPlanet::Saturn, 0.0, x, r);
        // c = 1, d = 15: the elapsed degrees doubled = 30 (generic would be 7.5).
        assert!(close(s(45.0), 30.0));
        // c = 2, d = 15: 60 less half the degrees = 52.5 (generic 30).
        assert!(close(s(75.0), 52.5));
        // c = 8, d = 15: the degrees + 30 = 45 (generic 22.5).
        assert!(close(s(255.0), 45.0));
        // c = 9, d = 15: the degrees still to run (15) doubled = 30 (generic 7.5).
        assert!(close(s(285.0), 30.0));
        // The 3rd and the 10th peak at exactly 2 and 9 signs.
        assert!(close(s(60.0), 60.0));
        assert!(close(s(270.0), 60.0));
        // A band the verses do not name is the generic value: c = 4, d = 10.
        assert!(close(s(130.0), 20.0));
    }

    /// 26.11, Mars.
    #[test]
    fn mars_special_aspects_hand_vectors() {
        let r = SputaSpecialRule::PerSignRemainder;
        let m = |x: f64| sputa_drishti_graha(VedicPlanet::Mars, 0.0, x, r);
        // c = 3, d = 15: 60 less the degrees = 45 (generic 37.5).
        assert!(close(m(105.0), 45.0));
        // c = 7, d = 15: the same = 45 (generic 37.5).
        assert!(close(m(225.0), 45.0));
        // c = 2, d = 15: one and a half times the degrees + 15 = 37.5 (generic 30).
        assert!(close(m(75.0), 37.5));
        // c = 6: a full rupa for the whole sign.
        assert!(close(m(180.0), 60.0));
        assert!(close(m(195.0), 60.0));
        assert!(close(m(209.5), 60.0));
        // Outside the named bands: generic. c = 4, d = 10 -> 20.
        assert!(close(m(130.0), 20.0));
    }

    /// 26.12, Jupiter.
    #[test]
    fn jupiter_special_aspects_hand_vectors() {
        let r = SputaSpecialRule::PerSignRemainder;
        let j = |x: f64| sputa_drishti_graha(VedicPlanet::Jupiter, 0.0, x, r);
        // c = 3, d = 15: half the elapsed degrees + 45 = 52.5 (generic 37.5).
        assert!(close(j(105.0), 52.5));
        // c = 7, d = 15: the same = 52.5 (generic 37.5).
        assert!(close(j(225.0), 52.5));
        // c = 4, d = 15: 60 less the DOUBLED degrees = 30 (generic 15).
        assert!(close(j(135.0), 30.0));
        // c = 8, d = 15: the same = 30 (generic 22.5).
        assert!(close(j(255.0), 30.0));
        // The 5th and 9th peak at exactly 4 and 8 signs.
        assert!(close(j(120.0), 60.0));
        assert!(close(j(240.0), 60.0));
        // Not named: c = 1, d = 15 -> generic 7.5.
        assert!(close(j(45.0), 7.5));
    }

    /// With the special readings Saturn and Mars are continuous everywhere
    /// and no special value passes 60; Jupiter is continuous everywhere but
    /// 270 degrees (module doc). A wrong constant in any special arm opens a
    /// gap at that arm's edge.
    #[test]
    fn special_curves_are_continuous_and_bounded() {
        let r = SputaSpecialRule::PerSignRemainder;
        let eps = 1e-7;
        for (planet, skip) in [
            (VedicPlanet::Saturn, None),
            (VedicPlanet::Mars, None),
            (VedicPlanet::Jupiter, Some(9)),
        ] {
            for k in 1..=10_u32 {
                if Some(k) == skip {
                    continue;
                }
                let edge = 30.0 * f64::from(k);
                let below = sputa_drishti_graha(planet, 0.0, edge - eps, r);
                let above = sputa_drishti_graha(planet, 0.0, edge + eps, r);
                assert!(
                    (below - above).abs() < 1e-5,
                    "{planet:?}: gap at {edge}: {below} vs {above}"
                );
            }
            let mut x = 0.0;
            while x < 360.0 {
                let v = sputa_drishti_graha(planet, 0.0, x, r);
                assert!((0.0..=60.0).contains(&v), "{planet:?} x={x}: {v}");
                x += 0.25;
            }
        }
    }

    /// The other recension: 45, 30 and 15 added by sign counted from the
    /// aspecting graha, whatever the degrees.
    #[test]
    fn added_steps_hand_vectors() {
        let r = SputaSpecialRule::AddedSteps;
        // Saturn at Aries 10, aspected point at Gemini 8 (3rd sign): x = 58,
        // over one sign: 28/2 = 14; + 45 = 59.
        assert!(close(
            sputa_drishti_graha(VedicPlanet::Saturn, 10.0, 68.0, r),
            59.0
        ));
        // The same pair under the other rule: Saturn c = 1, d = 28 -> 56.
        assert!(close(
            sputa_drishti_graha(
                VedicPlanet::Saturn,
                10.0,
                68.0,
                SputaSpecialRule::PerSignRemainder
            ),
            56.0
        ));
        // Mars at Aries 10, aspected at Cancer 20 (4th sign): x = 100,
        // over three: (120-100)/2 + 30 = 40; + 15 = 55.
        assert!(close(
            sputa_drishti_graha(VedicPlanet::Mars, 10.0, 110.0, r),
            55.0
        ));
        // Jupiter at Aries 10, aspected at Sagittarius 5 (9th sign): x = 235,
        // over six: (300-235)/2 = 32.5; + 30 = 62.5, past a rupa as read.
        assert!(close(
            sputa_drishti_graha(VedicPlanet::Jupiter, 10.0, 245.0, r),
            62.5
        ));
        // Saturn in the 4th sign has no step: x = 100 -> generic 40.
        assert!(close(
            sputa_drishti_graha(VedicPlanet::Saturn, 10.0, 110.0, r),
            40.0
        ));
        // Sun, Moon, Mercury, Venus and the nodes never take a step.
        for g in [
            VedicPlanet::Sun,
            VedicPlanet::Moon,
            VedicPlanet::Mercury,
            VedicPlanet::Venus,
            VedicPlanet::Rahu,
            VedicPlanet::Ketu,
        ] {
            assert!(close(sputa_drishti_graha(g, 10.0, 68.0, r), 14.0), "{g:?}");
            assert!(close(
                sputa_drishti_graha(g, 10.0, 68.0, SputaSpecialRule::PerSignRemainder),
                14.0
            ));
        }
    }

    #[test]
    fn graha_converts_to_vedic_planet() {
        use crate::graha::Graha;
        assert_eq!(VedicPlanet::from(Graha::Sun), VedicPlanet::Sun);
        assert_eq!(VedicPlanet::from(Graha::Saturn), VedicPlanet::Saturn);
        assert_eq!(VedicPlanet::from(Graha::Ketu), VedicPlanet::Ketu);
    }
}
