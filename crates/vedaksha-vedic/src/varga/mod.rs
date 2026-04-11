// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Divisional charts (vargas) for Vedic astrology.
//!
//! Vargas divide each zodiac sign into smaller equal (or unequal) parts and
//! map a planet's sidereal longitude to a new sign in that divisional chart.
//! The Shodasha Vargas (16 divisional charts) from D-1 through D-60 are
//! implemented here.
//!
//! Source: BPHS Ch. 6-7; B.V. Raman — _A Manual of Hindu Astrology_.

use vedaksha_math::angle::normalize_degrees;

// ──────────────────────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────────────────────

/// Varga (divisional chart) type — represents one of the 16 standard Shodasha
/// Vargas used in Parashari Jyotish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VargaType {
    /// D-1: The birth chart itself (identity / personality).
    Rashi,
    /// D-2: Hora — wealth.
    Hora,
    /// D-3: Drekkana — siblings and courage.
    Drekkana,
    /// D-4: Chaturthamsha — property and fortune.
    Chaturthamsha,
    /// D-7: Saptamsha — children and progeny.
    Saptamsha,
    /// D-9: Navamsha — spouse, dharma (most important varga after D-1).
    Navamsha,
    /// D-10: Dashamsha — career and profession.
    Dashamsha,
    /// D-12: Dwadashamsha — parents.
    Dwadashamsha,
    /// D-16: Shodashamsha — vehicles and happiness.
    Shodashamsha,
    /// D-20: Vimshamsha — spiritual progress.
    Vimshamsha,
    /// D-24: `ChaturVimshamsha` — education and learning.
    ChaturVimshamsha,
    /// D-27: Saptavimshamsha — strength and weakness.
    Saptavimshamsha,
    /// D-30: Trimshamsha — misfortune and evil.
    Trimshamsha,
    /// D-40: Khavedamsha — auspicious and inauspicious effects.
    Khavedamsha,
    /// D-45: Akshavedamsha — general indications.
    Akshavedamsha,
    /// D-60: Shashtiamsha — past karma (most granular varga).
    Shashtiamsha,
}

impl VargaType {
    /// The divisional number (e.g., [`VargaType::Navamsha`] returns `9`).
    #[must_use]
    pub const fn division(&self) -> u32 {
        match self {
            Self::Rashi => 1,
            Self::Hora => 2,
            Self::Drekkana => 3,
            Self::Chaturthamsha => 4,
            Self::Saptamsha => 7,
            Self::Navamsha => 9,
            Self::Dashamsha => 10,
            Self::Dwadashamsha => 12,
            Self::Shodashamsha => 16,
            Self::Vimshamsha => 20,
            Self::ChaturVimshamsha => 24,
            Self::Saptavimshamsha => 27,
            Self::Trimshamsha => 30,
            Self::Khavedamsha => 40,
            Self::Akshavedamsha => 45,
            Self::Shashtiamsha => 60,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the varga sign for a given sidereal longitude.
///
/// Takes a sidereal longitude in degrees and returns the sign index (0–11,
/// where 0 = Aries and 11 = Pisces) that the planet occupies in the specified
/// divisional chart.
///
/// The longitude is first normalised to [0, 360) before computation.
///
/// Source: BPHS Ch. 6-7.
#[must_use]
pub fn varga_sign(longitude_deg: f64, varga: VargaType) -> u8 {
    let lon = normalize_degrees(longitude_deg);
    match varga {
        VargaType::Rashi => rashi(lon),
        VargaType::Hora => hora(lon),
        VargaType::Drekkana => drekkana(lon),
        VargaType::Chaturthamsha => chaturthamsha(lon),
        VargaType::Saptamsha => saptamsha(lon),
        VargaType::Navamsha => navamsha(lon),
        VargaType::Dashamsha => dashamsha(lon),
        VargaType::Dwadashamsha => dwadashamsha(lon),
        VargaType::Shodashamsha => shodashamsha(lon),
        VargaType::Vimshamsha => vimshamsha(lon),
        VargaType::ChaturVimshamsha => chaturvimshamsha(lon),
        VargaType::Saptavimshamsha => saptavimshamsha(lon),
        VargaType::Trimshamsha => trimshamsha(lon),
        VargaType::Khavedamsha => khavedamsha(lon),
        VargaType::Akshavedamsha => akshavedamsha(lon),
        VargaType::Shashtiamsha => shashtiamsha(lon),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// General formula for vargas with a uniform division size.
///
/// `division` — number of equal parts per sign (e.g. 9 for Navamsha).
/// `start_sign` — the sign (0–11) from which counting begins.
#[inline]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn general_varga(lon: f64, division: u32, start_sign: u8) -> u8 {
    let position_in_sign = lon % 30.0;
    let part = (position_in_sign * f64::from(division) / 30.0) as u8;
    (start_sign + part) % 12
}

/// Return the natal sign (0–11) for a normalised longitude.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sign_of(lon: f64) -> u8 {
    (lon / 30.0) as u8 % 12
}

// ──────────────────────────────────────────────────────────────────────────────
// Individual varga implementations
// ──────────────────────────────────────────────────────────────────────────────

/// D-1 Rashi — the birth chart. `sign = floor(lon / 30)`.
fn rashi(lon: f64) -> u8 {
    sign_of(lon)
}

/// D-2 Hora — wealth.
///
/// Each sign is split into two 15° halves.
/// - Odd signs  (Aries, Gemini, …): first half → Leo (4), second → Cancer (3).
/// - Even signs (Taurus, Cancer, …): first half → Cancer (3), second → Leo (4).
///
/// "Odd/even" here follows 1-based sign numbering (Aries = 1 = odd).
fn hora(lon: f64) -> u8 {
    let sign = sign_of(lon); // 0 = Aries
    let pos = lon % 30.0;
    let first_half = pos < 15.0;

    // sign is 0-based: even index (0,2,4,…) = odd sign (Aries=1, Gemini=3, …)
    let odd_sign = sign % 2 == 0;

    // Leo when both match (odd+first or even+second); Cancer otherwise.
    if odd_sign == first_half { 4 } else { 3 }
}

/// D-3 Drekkana — siblings.
///
/// Three 10° divisions per sign:
/// - First decanate  (0–10°): same sign.
/// - Second decanate (10–20°): 5th sign from natal sign.
/// - Third decanate  (20–30°): 9th sign from natal sign.
fn drekkana(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let pos = lon % 30.0;
    // pos is in [0.0, 30.0), so (pos / 10.0) is in [0.0, 3.0) — safe to cast
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let decanate = (pos / 10.0) as u8; // 0, 1, or 2
    let offset = match decanate {
        0 => 0,
        1 => 4, // 5th sign (0-based +4)
        _ => 8, // 9th sign (0-based +8)
    };
    (sign + offset) % 12
}

/// D-4 Chaturthamsha — property/fortune.
///
/// Four 7°30′ divisions. Count from sign, then 4th, 7th, 10th sign.
/// Offsets: 0, 3, 6, 9.
fn chaturthamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    general_varga(lon, 4, sign)
}

/// D-7 Saptamsha — children.
///
/// Seven ~4°17′ divisions.
/// - Odd signs: count from the sign itself.
/// - Even signs: count from the 7th sign.
fn saptamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let odd_sign = sign % 2 == 0; // 0-based even index = odd sign
    let start = if odd_sign { sign } else { (sign + 6) % 12 };
    general_varga(lon, 7, start)
}

/// D-9 Navamsha — spouse and dharma (most important varga after D-1).
///
/// Nine 3°20′ divisions per sign.
/// - Movable signs (Aries=0, Cancer=3, Libra=6, Capricorn=9): count from sign.
/// - Fixed signs   (Taurus=1, Leo=4, Scorpio=7, Aquarius=10): count from 9th.
/// - Dual signs    (Gemini=2, Virgo=5, Sagittarius=8, Pisces=11): count from 5th.
fn navamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let start_sign = match sign % 3 {
        0 => sign,            // movable: from same sign
        1 => (sign + 8) % 12, // fixed: from 9th sign (0-based +8)
        _ => (sign + 4) % 12, // dual:  from 5th sign (0-based +4)
    };
    general_varga(lon, 9, start_sign)
}

/// D-10 Dashamsha — career.
///
/// Ten 3° divisions per sign.
/// - Odd signs: count from sign.
/// - Even signs: count from 9th sign.
fn dashamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let odd_sign = sign % 2 == 0;
    let start = if odd_sign { sign } else { (sign + 8) % 12 };
    general_varga(lon, 10, start)
}

/// D-12 Dwadashamsha — parents.
///
/// Twelve 2°30′ divisions. Count from the sign itself through all 12 signs.
fn dwadashamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    general_varga(lon, 12, sign)
}

/// D-16 Shodashamsha — vehicles and happiness.
///
/// Sixteen 1°52′30″ divisions.
/// - Movable signs: count from Aries (0).
/// - Fixed signs:   count from Leo (4).
/// - Dual signs:    count from Sagittarius (8).
fn shodashamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let start = match sign % 3 {
        0 => 0, // Aries
        1 => 4, // Leo
        _ => 8, // Sagittarius
    };
    general_varga(lon, 16, start)
}

/// D-20 Vimshamsha — spiritual progress.
///
/// Twenty 1°30′ divisions.
/// - Movable signs: count from Aries (0).
/// - Fixed signs:   count from Sagittarius (8).
/// - Dual signs:    count from Leo (4).
fn vimshamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let start = match sign % 3 {
        0 => 0, // Aries
        1 => 8, // Sagittarius
        _ => 4, // Leo
    };
    general_varga(lon, 20, start)
}

/// D-24 `ChaturVimshamsha` — education and learning.
///
/// Twenty-four 1°15′ divisions.
/// - Odd signs:  count from Leo (4).
/// - Even signs: count from Cancer (3).
fn chaturvimshamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let odd_sign = sign % 2 == 0;
    let start = if odd_sign { 4 } else { 3 }; // Leo or Cancer
    general_varga(lon, 24, start)
}

/// D-27 Saptavimshamsha — strength and weakness.
///
/// Twenty-seven 1°6′40″ divisions.
/// - Fire signs  (Aries=0, Leo=4, Sagittarius=8):    count from Aries (0).
/// - Earth signs (Taurus=1, Virgo=5, Capricorn=9):   count from Cancer (3).
/// - Air signs   (Gemini=2, Libra=6, Aquarius=10):   count from Libra (6).
/// - Water signs (Cancer=3, Scorpio=7, Pisces=11):   count from Capricorn (9).
fn saptavimshamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let start = match sign % 4 {
        0 => 0, // Fire  → Aries
        1 => 3, // Earth → Cancer
        2 => 6, // Air   → Libra
        _ => 9, // Water → Capricorn
    };
    general_varga(lon, 27, start)
}

/// D-30 Trimshamsha — misfortune.
///
/// Five **unequal** divisions per sign, each ruled by a planet:
///
/// | Odd sign  | Range       | Even sign | Range       | Sign result |
/// |-----------|-------------|-----------|-------------|-------------|
/// | Mars      | 0–5°        | Venus     | 0–5°        |             |
/// | Saturn    | 5–10°       | Mercury   | 5–12°       |             |
/// | Jupiter   | 10–18°      | Jupiter   | 12–20°      |             |
/// | Mercury   | 18–25°      | Saturn    | 20–25°      |             |
/// | Venus     | 25–30°      | Mars      | 25–30°      |             |
///
/// The sign result follows the Parashari rule where each trimsha maps to the
/// sign owned by its ruling planet (using the natural sign lordship):
/// - Mars    → Aries (0) for odd / Scorpio (7) for even signs
/// - Saturn  → Aquarius (10) for odd / Capricorn (9) for even signs
/// - Jupiter → Sagittarius (8) for odd / Pisces (11) for even signs
/// - Mercury → Gemini (2) for odd / Virgo (5) for even signs
/// - Venus   → Taurus (1) for odd / Libra (6) for even signs
fn trimshamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let pos = lon % 30.0;
    let odd_sign = sign % 2 == 0;

    if odd_sign {
        // Odd signs: 0-5 Mars, 5-10 Saturn, 10-18 Jupiter, 18-25 Mercury, 25-30 Venus
        if pos < 5.0 {
            0 // Aries (Mars)
        } else if pos < 10.0 {
            10 // Aquarius (Saturn)
        } else if pos < 18.0 {
            8 // Sagittarius (Jupiter)
        } else if pos < 25.0 {
            2 // Gemini (Mercury)
        } else {
            1 // Taurus (Venus)
        }
    } else {
        // Even signs: 0-5 Venus, 5-12 Mercury, 12-20 Jupiter, 20-25 Saturn, 25-30 Mars
        if pos < 5.0 {
            6 // Libra (Venus)
        } else if pos < 12.0 {
            5 // Virgo (Mercury)
        } else if pos < 20.0 {
            11 // Pisces (Jupiter)
        } else if pos < 25.0 {
            9 // Capricorn (Saturn)
        } else {
            7 // Scorpio (Mars)
        }
    }
}

/// D-40 Khavedamsha — auspicious and inauspicious effects.
///
/// Forty 0°45′ divisions.
/// - Odd signs:  count from Aries (0).
/// - Even signs: count from Libra (6).
fn khavedamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let odd_sign = sign % 2 == 0;
    let start = if odd_sign { 0 } else { 6 };
    general_varga(lon, 40, start)
}

/// D-45 Akshavedamsha — general indications.
///
/// Forty-five 0°40′ divisions.
/// - Movable signs: count from Aries (0).
/// - Fixed signs:   count from Leo (4).
/// - Dual signs:    count from Sagittarius (8).
fn akshavedamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    let start = match sign % 3 {
        0 => 0, // Aries
        1 => 4, // Leo
        _ => 8, // Sagittarius
    };
    general_varga(lon, 45, start)
}

/// D-60 Shashtiamsha — past karma (most granular standard varga).
///
/// Sixty 0°30′ divisions. Each division maps sequentially through all 12 signs,
/// cycling 5 times (5 × 12 = 60). Counting starts from the natal sign itself.
fn shashtiamsha(lon: f64) -> u8 {
    let sign = sign_of(lon);
    general_varga(lon, 60, sign)
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── VargaType::division() ────────────────────────────────────────────────

    #[test]
    fn varga_type_divisions_match_d_number() {
        assert_eq!(VargaType::Rashi.division(), 1);
        assert_eq!(VargaType::Hora.division(), 2);
        assert_eq!(VargaType::Drekkana.division(), 3);
        assert_eq!(VargaType::Chaturthamsha.division(), 4);
        assert_eq!(VargaType::Saptamsha.division(), 7);
        assert_eq!(VargaType::Navamsha.division(), 9);
        assert_eq!(VargaType::Dashamsha.division(), 10);
        assert_eq!(VargaType::Dwadashamsha.division(), 12);
        assert_eq!(VargaType::Shodashamsha.division(), 16);
        assert_eq!(VargaType::Vimshamsha.division(), 20);
        assert_eq!(VargaType::ChaturVimshamsha.division(), 24);
        assert_eq!(VargaType::Saptavimshamsha.division(), 27);
        assert_eq!(VargaType::Trimshamsha.division(), 30);
        assert_eq!(VargaType::Khavedamsha.division(), 40);
        assert_eq!(VargaType::Akshavedamsha.division(), 45);
        assert_eq!(VargaType::Shashtiamsha.division(), 60);
    }

    // ── D-1 Rashi ────────────────────────────────────────────────────────────

    #[test]
    fn rashi_0_deg_is_aries() {
        assert_eq!(varga_sign(0.0, VargaType::Rashi), 0);
    }

    #[test]
    fn rashi_30_deg_is_taurus() {
        assert_eq!(varga_sign(30.0, VargaType::Rashi), 1);
    }

    #[test]
    fn rashi_359_deg_is_pisces() {
        assert_eq!(varga_sign(359.0, VargaType::Rashi), 11);
    }

    // ── D-2 Hora ─────────────────────────────────────────────────────────────

    #[test]
    fn hora_odd_sign_first_half_is_leo() {
        // Aries (odd sign), 0–15° → Leo (4)
        assert_eq!(varga_sign(5.0, VargaType::Hora), 4);
    }

    #[test]
    fn hora_odd_sign_second_half_is_cancer() {
        // Aries (odd sign), 15–30° → Cancer (3)
        assert_eq!(varga_sign(20.0, VargaType::Hora), 3);
    }

    #[test]
    fn hora_even_sign_first_half_is_cancer() {
        // Taurus (even sign), 0–15° → Cancer (3)
        assert_eq!(varga_sign(35.0, VargaType::Hora), 3);
    }

    // ── D-3 Drekkana ─────────────────────────────────────────────────────────

    #[test]
    fn drekkana_first_decanate_is_same_sign() {
        // 0° Aries, first 10° → Aries (0)
        assert_eq!(varga_sign(0.0, VargaType::Drekkana), 0);
    }

    #[test]
    fn drekkana_second_decanate_is_5th_sign() {
        // 10° Aries → 5th from Aries = Leo (4)
        assert_eq!(varga_sign(10.5, VargaType::Drekkana), 4);
    }

    #[test]
    fn drekkana_third_decanate_is_9th_sign() {
        // 20° Aries → 9th from Aries = Sagittarius (8)
        assert_eq!(varga_sign(20.5, VargaType::Drekkana), 8);
    }

    // ── D-9 Navamsha ─────────────────────────────────────────────────────────

    #[test]
    fn navamsha_0_deg_aries_is_aries() {
        // Aries is movable (sign%3==0): first navamsha starts from Aries itself
        assert_eq!(varga_sign(0.0, VargaType::Navamsha), 0);
    }

    #[test]
    fn navamsha_second_part_of_aries_is_taurus() {
        // 3.34° Aries → second navamsha of movable Aries = Taurus (1)
        assert_eq!(varga_sign(3.34, VargaType::Navamsha), 1);
    }

    #[test]
    fn navamsha_taurus_fixed_starts_from_capricorn() {
        // Taurus is fixed (sign%3==1), start from 9th sign = Capricorn (9)
        // 30.0° = start of Taurus, first navamsha → Capricorn
        assert_eq!(varga_sign(30.0, VargaType::Navamsha), 9);
    }

    // ── D-10 Dashamsha ───────────────────────────────────────────────────────

    #[test]
    fn dashamsha_odd_sign_starts_from_sign() {
        // 0° Aries (odd sign) → Aries (0)
        assert_eq!(varga_sign(0.0, VargaType::Dashamsha), 0);
    }

    #[test]
    fn dashamsha_even_sign_starts_from_9th() {
        // 30° = Taurus (even sign), start from 9th = Capricorn (9)
        assert_eq!(varga_sign(30.0, VargaType::Dashamsha), 9);
    }

    // ── D-12 Dwadashamsha ────────────────────────────────────────────────────

    #[test]
    fn dwadashamsha_0_deg_aries_is_aries() {
        assert_eq!(varga_sign(0.0, VargaType::Dwadashamsha), 0);
    }

    #[test]
    fn dwadashamsha_2_5_deg_is_taurus() {
        // 2.5° Aries → second dwadashamsha → Taurus (1)
        assert_eq!(varga_sign(2.5, VargaType::Dwadashamsha), 1);
    }

    #[test]
    fn dwadashamsha_27_5_deg_is_pisces() {
        // 27.5° Aries → 12th dwadashamsha → Pisces (11)
        assert_eq!(varga_sign(27.5, VargaType::Dwadashamsha), 11);
    }

    // ── D-60 Shashtiamsha ────────────────────────────────────────────────────

    #[test]
    fn shashtiamsha_0_deg_aries_is_aries() {
        // First shashtiamsha of Aries → Aries (0)
        assert_eq!(varga_sign(0.0, VargaType::Shashtiamsha), 0);
    }

    #[test]
    fn shashtiamsha_0_5_deg_aries_is_taurus() {
        // 0.5° Aries → second shashtiamsha → Taurus (1)
        assert_eq!(varga_sign(0.5, VargaType::Shashtiamsha), 1);
    }
}
