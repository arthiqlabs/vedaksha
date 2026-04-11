// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedic houses (bhava).
//!
//! In Vedic astrology, the most common house system is Whole Sign (Rashi-based),
//! where house 1 is the entire sign containing the ascendant (lagna). Each
//! subsequent sign corresponds to the next house in sequence.
//!
//! Source: BPHS (Brihat Parashara Hora Shastra); Parashara's bhava system.

/// Vedic bhava (house) assignment for a chart.
#[derive(Debug, Clone)]
pub struct BhavaChart {
    /// Sign index (0-11) for each house (1-12).
    /// `house_signs[0]` = sign of 1st house (lagna), `house_signs[1]` = 2nd house, etc.
    pub house_signs: [u8; 12],
    /// The ascendant (lagna) sign index (0-11).
    pub lagna_sign: u8,
}

/// Compute Vedic bhavas (whole-sign houses) from an ascendant longitude.
///
/// In the Vedic whole-sign system, house 1 is the sign containing the ascendant,
/// house 2 is the next sign, and so on around the zodiac.
///
/// # Arguments
/// * `asc_sidereal_deg` — sidereal longitude of the ascendant in degrees (any value,
///   will be normalised to `[0°, 360°)`)
#[must_use]
pub fn compute_bhavas(asc_sidereal_deg: f64) -> BhavaChart {
    let normalized = vedaksha_math::angle::normalize_degrees(asc_sidereal_deg);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lagna_sign = (normalized / 30.0) as u8;
    #[allow(clippy::cast_possible_truncation)]
    let house_signs = core::array::from_fn(|i| (lagna_sign + i as u8) % 12);
    BhavaChart {
        house_signs,
        lagna_sign,
    }
}

/// Determine which bhava (house 1–12) a planet occupies given a chart.
///
/// # Arguments
/// * `planet_sign` — the sign index (0-11) the planet occupies
/// * `chart` — the bhava chart computed from the ascendant
#[must_use]
pub fn planet_bhava(planet_sign: u8, chart: &BhavaChart) -> u8 {
    let diff = (planet_sign + 12 - chart.lagna_sign) % 12;
    diff + 1 // houses are 1-indexed
}

/// Check if a bhava is a kendra (angular house: 1, 4, 7, 10).
///
/// Kendras are the most powerful houses, associated with cardinal directions.
#[must_use]
pub fn is_kendra(bhava: u8) -> bool {
    matches!(bhava, 1 | 4 | 7 | 10)
}

/// Check if a bhava is a trikona (trinal house: 1, 5, 9).
///
/// Trikonas are houses of dharma and fortune, considered highly auspicious.
#[must_use]
pub fn is_trikona(bhava: u8) -> bool {
    matches!(bhava, 1 | 5 | 9)
}

/// Check if a bhava is a dusthana (malefic house: 6, 8, 12).
///
/// Dusthanas are houses of difficulty, disease, obstacles, and loss.
#[must_use]
pub fn is_dusthana(bhava: u8) -> bool {
    matches!(bhava, 6 | 8 | 12)
}

/// Check if a bhava is an upachaya (growth house: 3, 6, 10, 11).
///
/// Upachayas improve over time; malefic planets placed here can give strength.
#[must_use]
pub fn is_upachaya(bhava: u8) -> bool {
    matches!(bhava, 3 | 6 | 10 | 11)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asc_at_0_degrees_aries_gives_correct_houses() {
        let chart = compute_bhavas(0.0);
        assert_eq!(chart.lagna_sign, 0, "Lagna should be Aries (0)");
        assert_eq!(chart.house_signs[0], 0, "House 1 = Aries (0)");
        assert_eq!(chart.house_signs[1], 1, "House 2 = Taurus (1)");
        assert_eq!(chart.house_signs[6], 6, "House 7 = Libra (6)");
        assert_eq!(chart.house_signs[11], 11, "House 12 = Pisces (11)");
    }

    #[test]
    fn asc_at_120_degrees_leo_gives_lagna_leo() {
        let chart = compute_bhavas(120.0);
        assert_eq!(chart.lagna_sign, 4, "Lagna should be Leo (4)");
        assert_eq!(chart.house_signs[0], 4, "House 1 = Leo (4)");
        assert_eq!(chart.house_signs[1], 5, "House 2 = Virgo (5)");
        // Wrap: house 9 (index 8): (4 + 8) % 12 = 0 (Aries)
        assert_eq!(chart.house_signs[8], 0, "House 9 = Aries (0) (wraps)");
    }

    #[test]
    fn planet_bhava_in_lagna_sign_is_house_1() {
        let chart = compute_bhavas(60.0); // Lagna = Gemini (2)
        assert_eq!(planet_bhava(chart.lagna_sign, &chart), 1);
    }

    #[test]
    fn planet_bhava_six_signs_away_is_house_7() {
        let chart = compute_bhavas(0.0); // Lagna = Aries (0)
        // 6 signs from Aries = Libra (6)
        assert_eq!(planet_bhava(6, &chart), 7);
    }

    #[test]
    fn planet_bhava_wraps_correctly_past_pisces() {
        let chart = compute_bhavas(300.0); // Lagna = Capricorn (10)
        // Planet in Aries (0): (0 + 12 - 10) % 12 = 2, bhava = 3
        assert_eq!(planet_bhava(0, &chart), 3);
    }

    #[test]
    fn is_kendra_correct() {
        assert!(is_kendra(1));
        assert!(is_kendra(4));
        assert!(is_kendra(7));
        assert!(is_kendra(10));
        assert!(!is_kendra(2));
        assert!(!is_kendra(5));
        assert!(!is_kendra(9));
        assert!(!is_kendra(11));
    }

    #[test]
    fn is_trikona_correct() {
        assert!(is_trikona(1));
        assert!(is_trikona(5));
        assert!(is_trikona(9));
        assert!(!is_trikona(2));
        assert!(!is_trikona(4));
        assert!(!is_trikona(6));
    }

    #[test]
    fn is_dusthana_correct() {
        assert!(is_dusthana(6));
        assert!(is_dusthana(8));
        assert!(is_dusthana(12));
        assert!(!is_dusthana(1));
        assert!(!is_dusthana(5));
        assert!(!is_dusthana(7));
    }

    #[test]
    fn is_upachaya_correct() {
        assert!(is_upachaya(3));
        assert!(is_upachaya(6));
        assert!(is_upachaya(10));
        assert!(is_upachaya(11));
        assert!(!is_upachaya(1));
        assert!(!is_upachaya(5));
        assert!(!is_upachaya(9));
    }

    #[test]
    fn asc_negative_degrees_normalizes_correctly() {
        // -30° normalises to 330° → Capricorn (11)
        let chart = compute_bhavas(-30.0);
        assert_eq!(chart.lagna_sign, 11, "Lagna should be Pisces (11) for -30°");
    }
}
