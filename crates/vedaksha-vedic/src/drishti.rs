// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedic planetary aspects (drishti).
//!
//! In Vedic astrology, all planets aspect the 7th house from their position
//! (180° opposition). Mars, Jupiter, and Saturn have additional special aspects.
//!
//! Aspects are sign-based (whole-sign), not degree-based. A planet in sign X
//! aspects the entire sign that is N houses away.
//!
//! All planets also have graded aspects on houses 3,10 (Quarter/25%),
//! 4,8 (ThreeQuarter/75%), and 5,9 (Half/50%). Mars, Jupiter, and Saturn
//! override their special houses to Full (100%).
//!
//! Source: BPHS Ch. 26 Sl. 2-7.

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
}
