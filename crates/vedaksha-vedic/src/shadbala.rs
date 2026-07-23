// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Shadbala (six-fold planetary strength).
//!
//! Computes the six components of planetary strength used in Vedic astrology
//! to assess how effectively a planet can deliver its results. Implements all
//! six components: Sthana Bala (positional), Dig Bala (directional), Kala Bala
//! (temporal), Cheshta Bala (motional), Naisargika Bala (natural), and Drik
//! Bala (aspectual).
//!
//! Source: BPHS Ch. 27; B.V. Raman, *Graha and Bhava Balas*.

use crate::graha::{Graha, GrahaPosition};
use serde::Serialize;

/// Shadbala (six-fold strength) for a planet.
///
/// All component values are in virupas (shashtiamsas, 1/60 of a rupa).
#[derive(Debug, Clone, Serialize)]
pub struct Shadbala {
    /// The planet this strength applies to.
    pub planet: Graha,
    /// Positional strength (own/exalted/friend sign).
    pub sthana_bala: f64,
    /// Directional strength (planets strong in certain houses).
    pub dig_bala: f64,
    /// Temporal strength (day/night, paksha, hora).
    pub kala_bala: f64,
    /// Motional strength (retrograde, stationary, direct speed).
    pub cheshta_bala: f64,
    /// Natural strength (fixed per planet).
    pub naisargika_bala: f64,
    /// Aspectual strength (benefic/malefic aspects).
    pub drik_bala: f64,
    /// Total Shadbala (sum of all six components).
    pub total: f64,
    /// Exaltation strength component (0–60 virupas). Source: BPHS Ch.27.
    pub uccha_bala: f64,
    /// Benefit strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 vv.5-6.
    pub ishta_phala: f64,
    /// Affliction strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 v.6.
    pub kashta_phala: f64,
}

/// Additional temporal/motional parameters needed for full Shadbala computation.
#[derive(Debug, Clone, Copy)]
pub struct ShadbalaPlanetData {
    /// The planet position.
    pub position: GrahaPosition,
    /// Daily speed in degrees/day. Negative = retrograde.
    pub speed: f64,
    /// Average daily speed for this planet (used to classify motion).
    pub average_speed: f64,
    /// Number of benefic aspects received by this planet.
    pub benefic_aspect_count: u32,
    /// Number of malefic aspects received by this planet.
    pub malefic_aspect_count: u32,
}

// ── Naisargika Bala (natural strength) ──────────────────────────────

/// Natural strength — fixed values per planet (in virupas).
///
/// From brightest/strongest to weakest:
/// Sun (60), Moon (51.43), Venus (42.86), Jupiter (34.29),
/// Mercury (25.71), Mars (17.14), Saturn (8.57).
#[must_use]
pub fn naisargika_bala(planet: Graha) -> f64 {
    match planet {
        Graha::Sun => 60.0,
        Graha::Moon => 51.43,
        Graha::Venus => 42.86,
        Graha::Jupiter => 34.29,
        Graha::Mercury => 25.71,
        Graha::Mars => 17.14,
        Graha::Saturn => 8.57,
        // Rahu/Ketu not traditionally part of Shadbala
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

// ── Dig Bala (directional strength) ─────────────────────────────────

/// The house in which a planet has maximum directional strength.
///
/// - Sun, Mars -> 10th (south)
/// - Moon, Venus -> 4th (north)
/// - Mercury, Jupiter -> 1st (east)
/// - Saturn -> 7th (west)
#[must_use]
fn dig_bala_strong_house(planet: Graha) -> u8 {
    match planet {
        Graha::Sun | Graha::Mars => 10,
        Graha::Moon | Graha::Venus => 4,
        Graha::Mercury | Graha::Jupiter | Graha::Rahu | Graha::Ketu => 1,
        Graha::Saturn => 7,
    }
}

/// Directional strength in virupas (0-60).
///
/// Maximum (60) when planet is in its strong house.
/// Minimum (0) when planet is in the opposite house (6 houses away).
/// Linear interpolation in between.
#[must_use]
pub fn dig_bala(planet: Graha, bhava: u8) -> f64 {
    let strong = dig_bala_strong_house(planet);
    // Distance in houses (circular, 1-indexed, max 6)
    let raw_dist = bhava.abs_diff(strong);
    let dist = if raw_dist > 6 {
        12 - raw_dist
    } else {
        raw_dist
    };
    // 60 at distance 0, 0 at distance 6
    let bala = 60.0 - (f64::from(dist) * 10.0);
    bala.max(0.0)
}

// ── Sthana Bala (positional strength) ───────────────────────────────

/// Exaltation longitude for each planet in degrees (0-360).
///
/// Source: BPHS Ch. 3 Sl. 18.
fn exaltation_longitude(planet: Graha) -> f64 {
    match planet {
        Graha::Sun => 10.0,      // 10° Aries
        Graha::Moon => 33.0,     // 3° Taurus
        Graha::Mars => 298.0,    // 28° Capricorn
        Graha::Mercury => 165.0, // 15° Virgo
        Graha::Jupiter => 95.0,  // 5° Cancer
        Graha::Venus => 357.0,   // 27° Pisces
        Graha::Saturn => 200.0,  // 20° Libra
        Graha::Rahu => 50.0,     // 20° Taurus
        Graha::Ketu => 230.0,    // 20° Scorpio
    }
}

/// Degree-precise Uccha Bala (exaltation strength) in virupas (0-60).
///
/// Formula (BPHS Ch. 27 Sl. 3-6):
///   uccha_bala = (180 - arc) / 3
/// where arc = min(|longitude - exaltation_longitude|, 360 - |longitude - exaltation_longitude|).
///
/// Yields 60 virupas at exact exaltation, 0 at exact debilitation (180° away).
#[must_use]
pub fn sthana_bala(planet: Graha, _sign: u8, longitude: f64) -> f64 {
    let exalt_lon = exaltation_longitude(planet);
    let raw_diff = (longitude - exalt_lon).abs();
    let arc = if raw_diff > 180.0 {
        360.0 - raw_diff
    } else {
        raw_diff
    };
    ((180.0 - arc) / 3.0).max(0.0)
}

// ── Kala Bala (temporal strength) ───────────────────────────────────

/// Temporal strength in virupas.
///
/// Combines Nathonnatha Bala (day/night) and Paksha Bala (lunar phase).
///
/// - **Nathonnatha:** Sun, Jupiter, Venus strong during day; Moon, Mars, Saturn
///   strong at night; Mercury always strong. Max 30 virupas.
/// - **Paksha:** Benefics strong in Shukla Paksha (waxing); malefics strong
///   in Krishna Paksha (waning). Max 30 virupas.
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn kala_bala(planet: Graha, is_daytime: bool, moon_phase_waxing: bool) -> f64 {
    let mut bala = 0.0;

    // Nathonnatha Bala (day/night strength)
    let day_strong = matches!(planet, Graha::Sun | Graha::Jupiter | Graha::Venus);
    let night_strong = matches!(planet, Graha::Moon | Graha::Mars | Graha::Saturn);

    if planet == Graha::Mercury {
        bala += 30.0; // Mercury is always strong (twilight planet)
    } else if (is_daytime && day_strong) || (!is_daytime && night_strong) {
        bala += 30.0;
    }

    // Paksha Bala (lunar phase strength)
    let is_benefic = matches!(
        planet,
        Graha::Jupiter | Graha::Venus | Graha::Mercury | Graha::Moon
    );
    if (moon_phase_waxing && is_benefic) || (!moon_phase_waxing && !is_benefic) {
        bala += 30.0;
    }

    bala
}

// ── Cheshta Bala (motional strength) ────────────────────────────────

/// Motional strength in virupas based on planetary speed.
///
/// - Retrograde: 60 virupas (strongest — planet re-traverses degrees)
/// - Stationary (speed near 0): 30 virupas
/// - Direct slow (< 50% average): 15 virupas
/// - Direct normal: 30 virupas
/// - Direct fast (> 150% average): 45 virupas
///
/// Sun and Moon are never retrograde; for them this returns 30 (normal).
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn cheshta_bala(planet: Graha, speed: f64, average_speed: f64) -> f64 {
    // Sun and Moon have no retrograde motion — assign normal strength
    if matches!(planet, Graha::Sun | Graha::Moon) {
        return 30.0;
    }
    // Rahu/Ketu always move retrograde but are not part of traditional Shadbala
    if matches!(planet, Graha::Rahu | Graha::Ketu) {
        return 0.0;
    }

    if speed < 0.0 {
        60.0 // retrograde
    } else if speed.abs() < 0.01 {
        30.0 // stationary
    } else if speed < average_speed * 0.5 {
        15.0 // slow
    } else if speed > average_speed * 1.5 {
        45.0 // fast
    } else {
        30.0 // normal
    }
}

// ── Drik Bala (aspectual strength) ──────────────────────────────────

/// Aspectual strength in virupas.
///
/// Each benefic aspect adds 15 virupas; each malefic aspect subtracts 15.
/// Clamped to [-30, 60].
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn drik_bala(benefic_aspect_count: u32, malefic_aspect_count: u32) -> f64 {
    let benefic_strength = f64::from(benefic_aspect_count) * 15.0;
    let malefic_strength = f64::from(malefic_aspect_count) * 15.0;
    (benefic_strength - malefic_strength).clamp(-30.0, 60.0)
}

// ── Ishta/Kashta Phala (BPHS Ch.28 vv.5-6) ─────────────────────────

/// Uchcha Rasmi: linear transform of uccha_bala virupas to 0–7 scale.
///
/// Source: BPHS Ch.28 v.5.
#[must_use]
pub fn uccha_rasmi(uccha_bala_virupas: f64) -> f64 {
    uccha_bala_virupas * 7.0 / 60.0
}

/// Cheshta Rasmi: linear transform of cheshta_bala virupas to 0–7 scale.
///
/// Source: BPHS Ch.28 v.5.
#[must_use]
pub fn cheshta_rasmi(cheshta_bala_virupas: f64) -> f64 {
    cheshta_bala_virupas * 7.0 / 60.0
}

/// Ishta Phala (benefit) and Kashta Phala (affliction) from Rasmi values.
///
/// Formula: ishta = 5 × (uchcha_rasmi + cheshta_rasmi − 2), clipped to [0, 60].
/// kashta = 60 − ishta. Source: BPHS Ch.28 vv.5-6.
#[must_use]
pub fn ishta_kashta_phala(uccha_bala: f64, cheshta_bala: f64) -> (f64, f64) {
    let ur = uccha_rasmi(uccha_bala);
    let cr = cheshta_rasmi(cheshta_bala);
    let ishta = (5.0 * (ur + cr - 2.0)).clamp(0.0, 60.0);
    (ishta, 60.0 - ishta)
}

// ── Planet-sign helpers (reuse from yoga.rs logic) ──────────────────

#[allow(dead_code)]
fn own_signs(planet: Graha) -> &'static [u8] {
    match planet {
        Graha::Sun => &[4],
        Graha::Moon => &[3],
        Graha::Mars => &[0, 7],
        Graha::Mercury => &[2, 5],
        Graha::Jupiter => &[8, 11],
        Graha::Venus => &[1, 6],
        Graha::Saturn => &[9, 10],
        Graha::Rahu | Graha::Ketu => &[],
    }
}

#[allow(dead_code)]
fn exaltation_sign(planet: Graha) -> Option<u8> {
    match planet {
        Graha::Sun => Some(0),
        Graha::Moon => Some(1),
        Graha::Mars => Some(9),
        Graha::Mercury => Some(5),
        Graha::Jupiter => Some(3),
        Graha::Venus => Some(11),
        Graha::Saturn => Some(6),
        Graha::Rahu | Graha::Ketu => None,
    }
}

#[allow(dead_code)]
fn debilitation_sign(planet: Graha) -> Option<u8> {
    exaltation_sign(planet).map(|s| (s + 6) % 12)
}

#[allow(dead_code)]
fn is_in_own_sign(planet: Graha, sign: u8) -> bool {
    own_signs(planet).contains(&sign)
}

#[allow(dead_code)]
fn is_exalted(planet: Graha, sign: u8) -> bool {
    exaltation_sign(planet) == Some(sign)
}

#[allow(dead_code)]
fn is_debilitated(planet: Graha, sign: u8) -> bool {
    debilitation_sign(planet) == Some(sign)
}

/// Sign lord (ruler).
#[allow(dead_code)]
fn sign_lord(sign: u8) -> Graha {
    match sign {
        0 | 7 => Graha::Mars,
        1 | 6 => Graha::Venus,
        2 | 5 => Graha::Mercury,
        3 => Graha::Moon,
        8 | 11 => Graha::Jupiter,
        9 | 10 => Graha::Saturn,
        _ => Graha::Sun, // fallback, should not happen
    }
}

/// Simplified friendly sign check based on traditional friendships.
///
/// Uses the natural friendship table (naisargika maitri).
#[allow(dead_code)]
fn is_friendly_sign(planet: Graha, sign: u8) -> bool {
    let lord = sign_lord(sign);
    if lord == planet {
        return true; // own sign counts as friendly
    }
    matches!(
        (planet, lord),
        (Graha::Sun | Graha::Mars | Graha::Jupiter, Graha::Moon)
            | (Graha::Sun | Graha::Jupiter, Graha::Mars)
            | (Graha::Sun | Graha::Mars, Graha::Jupiter)
            | (
                Graha::Moon | Graha::Mars | Graha::Mercury | Graha::Jupiter,
                Graha::Sun,
            )
            | (Graha::Moon | Graha::Venus | Graha::Saturn, Graha::Mercury)
            | (Graha::Mercury | Graha::Saturn, Graha::Venus)
            | (Graha::Venus, Graha::Saturn)
    )
}

// ── Compute Shadbala ────────────────────────────────────────────────

/// Compute Shadbala for all given planet positions (basic — 3 components).
///
/// Uses Sthana Bala, Dig Bala, and Naisargika Bala. Kala, Cheshta, and Drik
/// Bala are set to 0.0. For full 6-component Shadbala, use
/// [`compute_shadbala_full`].
///
/// # Arguments
/// * `positions` — slice of planet positions with sign and bhava
/// * `_lagna_sign` — lagna sign (reserved for compatibility)
#[must_use]
pub fn compute_shadbala(positions: &[GrahaPosition], _lagna_sign: u8) -> Vec<Shadbala> {
    positions
        .iter()
        .map(|pos| {
            let naisargika = naisargika_bala(pos.planet);
            let dig = dig_bala(pos.planet, pos.bhava);
            let sthana = sthana_bala(pos.planet, pos.sign, pos.longitude);
            let total = naisargika + dig + sthana;
            let (ishta, kashta) = ishta_kashta_phala(sthana, 0.0);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: 0.0,
                cheshta_bala: 0.0,
                naisargika_bala: naisargika,
                drik_bala: 0.0,
                total,
                uccha_bala: sthana,
                ishta_phala: ishta,
                kashta_phala: kashta,
            }
        })
        .collect()
}

/// Compute full Shadbala (all 6 components) for planets with extended data.
///
/// # Arguments
/// * `planets` — slice of extended planet data (position, speed, aspects)
/// * `is_daytime` — whether the chart is for daytime (Sun above horizon)
/// * `moon_phase_waxing` — whether the Moon is in Shukla Paksha (waxing)
#[must_use]
pub fn compute_shadbala_full(
    planets: &[ShadbalaPlanetData],
    is_daytime: bool,
    moon_phase_waxing: bool,
) -> Vec<Shadbala> {
    planets
        .iter()
        .map(|data| {
            let pos = &data.position;
            let naisargika = naisargika_bala(pos.planet);
            let dig = dig_bala(pos.planet, pos.bhava);
            let sthana = sthana_bala(pos.planet, pos.sign, pos.longitude);
            let kala = kala_bala(pos.planet, is_daytime, moon_phase_waxing);
            let cheshta = cheshta_bala(pos.planet, data.speed, data.average_speed);
            let drik = drik_bala(data.benefic_aspect_count, data.malefic_aspect_count);
            let total = naisargika + dig + sthana + kala + cheshta + drik;

            let (ishta, kashta) = ishta_kashta_phala(sthana, cheshta);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: kala,
                cheshta_bala: cheshta,
                naisargika_bala: naisargika,
                drik_bala: drik,
                total,
                uccha_bala: sthana,
                ishta_phala: ishta,
                kashta_phala: kashta,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(planet: Graha, sign: u8, bhava: u8) -> GrahaPosition {
        GrahaPosition {
            planet,
            sign,
            longitude: f64::from(sign) * 30.0 + 15.0,
            bhava,
        }
    }

    fn planet_data(
        planet: Graha,
        sign: u8,
        bhava: u8,
        speed: f64,
        avg_speed: f64,
        benefic: u32,
        malefic: u32,
    ) -> ShadbalaPlanetData {
        ShadbalaPlanetData {
            position: pos(planet, sign, bhava),
            speed,
            average_speed: avg_speed,
            benefic_aspect_count: benefic,
            malefic_aspect_count: malefic,
        }
    }

    // ── Naisargika Bala tests ───────────────────────────────────────

    #[test]
    fn sun_naisargika_bala_is_60() {
        let bala = naisargika_bala(Graha::Sun);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_naisargika_bala_is_8_57() {
        let bala = naisargika_bala(Graha::Saturn);
        assert!((bala - 8.57).abs() < 0.01);
    }

    // ── Dig Bala tests ──────────────────────────────────────────────

    #[test]
    fn sun_dig_bala_maximum_in_10th() {
        let bala = dig_bala(Graha::Sun, 10);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_dig_bala_minimum_in_4th() {
        let bala = dig_bala(Graha::Sun, 4);
        assert!((bala - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn moon_dig_bala_maximum_in_4th() {
        let bala = dig_bala(Graha::Moon, 4);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_dig_bala_maximum_in_7th() {
        let bala = dig_bala(Graha::Saturn, 7);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dig_bala_intermediate_values() {
        let bala = dig_bala(Graha::Mercury, 4);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    // ── Sthana Bala (Uccha Bala) tests ───────────────────────────────

    #[test]
    fn sthana_bala_exact_exaltation_is_60() {
        // Sun exalted at 10° Aries → 60 virupas
        let bala = sthana_bala(Graha::Sun, 0, 10.0);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sthana_bala_exact_debilitation_is_0() {
        // Sun debilitated at 190° (10+180) → 0 virupas
        let bala = sthana_bala(Graha::Sun, 6, 190.0);
        assert!((bala - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sthana_bala_midpoint_is_30() {
        // Sun at 100° → 90° from exaltation (10°) → (180-90)/3 = 30 virupas
        let bala = sthana_bala(Graha::Sun, 3, 100.0);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sthana_bala_gradient_is_continuous() {
        // Jupiter at exact exaltation (95°) should be stronger than at 100°
        let at_exalt = sthana_bala(Graha::Jupiter, 3, 95.0);
        let near_exalt = sthana_bala(Graha::Jupiter, 3, 100.0);
        assert!(at_exalt > near_exalt);
    }

    // ── Kala Bala tests ─────────────────────────────────────────────

    #[test]
    fn sun_daytime_gets_nathonnatha() {
        let bala = kala_bala(Graha::Sun, true, false);
        // Daytime: 30. Sun is not benefic, Krishna paksha (non-benefic strong): +30
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_nighttime_no_nathonnatha() {
        let bala = kala_bala(Graha::Sun, false, false);
        // Night: 0. Krishna paksha, Sun is not benefic: +30
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mercury_always_gets_nathonnatha() {
        let day = kala_bala(Graha::Mercury, true, true);
        let night = kala_bala(Graha::Mercury, false, true);
        // Mercury: always 30 nathonnatha + benefic in shukla = +30 = 60
        assert!((day - 60.0).abs() < f64::EPSILON);
        assert!((night - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn moon_night_waxing() {
        let bala = kala_bala(Graha::Moon, false, true);
        // Night strong: 30. Benefic in waxing: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jupiter_day_waxing() {
        let bala = kala_bala(Graha::Jupiter, true, true);
        // Day strong: 30. Benefic in waxing: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_night_waning() {
        let bala = kala_bala(Graha::Saturn, false, false);
        // Night strong: 30. Non-benefic in waning: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    // ── Cheshta Bala tests ──────────────────────────────────────────

    #[test]
    fn retrograde_planet_gets_60() {
        let bala = cheshta_bala(Graha::Mars, -0.5, 0.5);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stationary_planet_gets_30() {
        let bala = cheshta_bala(Graha::Jupiter, 0.005, 0.08);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn slow_planet_gets_15() {
        let bala = cheshta_bala(Graha::Saturn, 0.01, 0.05);
        assert!((bala - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fast_planet_gets_45() {
        let bala = cheshta_bala(Graha::Venus, 2.0, 1.0);
        assert!((bala - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normal_speed_planet_gets_30() {
        let bala = cheshta_bala(Graha::Mercury, 1.0, 1.2);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_moon_always_30_cheshta() {
        let sun = cheshta_bala(Graha::Sun, 1.0, 1.0);
        let moon = cheshta_bala(Graha::Moon, 13.0, 13.0);
        assert!((sun - 30.0).abs() < f64::EPSILON);
        assert!((moon - 30.0).abs() < f64::EPSILON);
    }

    // ── Drik Bala tests ─────────────────────────────────────────────

    #[test]
    fn benefic_aspects_add_strength() {
        let bala = drik_bala(3, 0);
        assert!((bala - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn malefic_aspects_subtract_strength() {
        let bala = drik_bala(0, 2);
        assert!((bala - (-30.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn mixed_aspects() {
        let bala = drik_bala(2, 1);
        // 2*15 - 1*15 = 15
        assert!((bala - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drik_bala_clamped_high() {
        let bala = drik_bala(10, 0);
        // 10*15 = 150, clamped to 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drik_bala_clamped_low() {
        let bala = drik_bala(0, 10);
        // -150, clamped to -30
        assert!((bala - (-30.0)).abs() < f64::EPSILON);
    }

    // ── Legacy compute_shadbala tests ───────────────────────────────

    #[test]
    fn total_shadbala_is_sum_of_components() {
        let positions = [pos(Graha::Sun, 0, 10)];
        let results = compute_shadbala(&positions, 0);
        assert_eq!(results.len(), 1);
        let sb = &results[0];
        let expected = sb.sthana_bala
            + sb.dig_bala
            + sb.kala_bala
            + sb.cheshta_bala
            + sb.naisargika_bala
            + sb.drik_bala;
        assert!((sb.total - expected).abs() < f64::EPSILON);
    }

    // ── Full Shadbala tests ─────────────────────────────────────────

    #[test]
    fn full_shadbala_includes_all_six_components() {
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        assert_eq!(results.len(), 1);
        let sb = &results[0];

        // Sthana: Uccha Bala for Jupiter at 105° (sign 3, lon 3*30+15),
        // exaltation at 95°. arc=10, (180-10)/3 ≈ 56.67 virupas.
        let expected_sthana = (180.0 - 10.0) / 3.0;
        assert!((sb.sthana_bala - expected_sthana).abs() < 0.01);
        // Dig: house 4, strong house 1, dist 3 -> 60 - 30 = 30
        assert!((sb.dig_bala - 30.0).abs() < f64::EPSILON);
        // Naisargika: Jupiter = 34.29
        assert!((sb.naisargika_bala - 34.29).abs() < 0.01);
        // Kala: daytime + waxing benefic = 30 + 30 = 60
        assert!((sb.kala_bala - 60.0).abs() < f64::EPSILON);
        // Cheshta: retrograde = 60
        assert!((sb.cheshta_bala - 60.0).abs() < f64::EPSILON);
        // Drik: 2 benefic - 1 malefic = 15
        assert!((sb.drik_bala - 15.0).abs() < f64::EPSILON);

        // Total
        let expected = expected_sthana + 30.0 + 34.29 + 60.0 + 60.0 + 15.0;
        assert!((sb.total - expected).abs() < 0.01);
    }

    #[test]
    fn full_shadbala_total_equals_sum() {
        let data = [
            planet_data(Graha::Sun, 0, 10, 1.0, 1.0, 1, 0),
            planet_data(Graha::Saturn, 6, 7, 0.02, 0.05, 0, 2),
        ];
        let results = compute_shadbala_full(&data, true, false);
        for sb in &results {
            let sum = sb.sthana_bala
                + sb.dig_bala
                + sb.kala_bala
                + sb.cheshta_bala
                + sb.naisargika_bala
                + sb.drik_bala;
            assert!(
                (sb.total - sum).abs() < f64::EPSILON,
                "Total {} != sum {} for {:?}",
                sb.total,
                sum,
                sb.planet
            );
        }
    }

    // ── Ishta/Kashta Phala tests ─────────────────────────────────────────

    #[test]
    fn uccha_rasmi_at_max_virupas() {
        let r = uccha_rasmi(60.0);
        assert!((r - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cheshta_rasmi_at_max_virupas() {
        let r = cheshta_rasmi(60.0);
        assert!((r - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_rasmi_zero() {
        assert!((uccha_rasmi(0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ishta_phala_max_at_both_full() {
        // uccha=60 (ur=7), cheshta=60 (cr=7): ishta = 5*(7+7-2) = 60
        let (ishta, kashta) = ishta_kashta_phala(60.0, 60.0);
        assert!((ishta - 60.0).abs() < f64::EPSILON);
        assert!((kashta - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ishta_phala_clips_to_zero() {
        // uccha=0, cheshta=0: 5*(0+0-2) = -10 → clamp to 0
        let (ishta, kashta) = ishta_kashta_phala(0.0, 0.0);
        assert!((ishta - 0.0).abs() < f64::EPSILON);
        assert!((kashta - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn kashta_is_complement_of_ishta() {
        let (ishta, kashta) = ishta_kashta_phala(30.0, 30.0);
        assert!((ishta + kashta - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn full_shadbala_has_uccha_bala_matching_sthana() {
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        let sb = &results[0];
        assert!((sb.uccha_bala - sb.sthana_bala).abs() < f64::EPSILON);
    }

    #[test]
    fn full_shadbala_uccha_not_in_total() {
        // total must still equal sum of the original six components
        let data = [planet_data(Graha::Sun, 0, 10, 1.0, 1.0, 1, 0)];
        let results = compute_shadbala_full(&data, true, false);
        let sb = &results[0];
        let sum = sb.sthana_bala
            + sb.dig_bala
            + sb.kala_bala
            + sb.cheshta_bala
            + sb.naisargika_bala
            + sb.drik_bala;
        assert!((sb.total - sum).abs() < f64::EPSILON);
    }

    #[test]
    fn full_shadbala_saturn_nighttime() {
        let data = [planet_data(
            Graha::Saturn,
            9, // Capricorn (own sign)
            7, // 7th house (dig bala max)
            0.02,
            0.05,
            0,
            0,
        )];
        let results = compute_shadbala_full(&data, false, false);
        let sb = &results[0];

        // Sthana: Uccha Bala for Saturn at 285° (sign 9, lon 9*30+15),
        // exaltation at 200°. arc=85, (180-85)/3 ≈ 31.67 virupas.
        let expected_sthana = (180.0 - 85.0) / 3.0;
        assert!((sb.sthana_bala - expected_sthana).abs() < 0.01);
        // Dig: house 7 = max = 60
        assert!((sb.dig_bala - 60.0).abs() < f64::EPSILON);
        // Kala: night + waning non-benefic = 30 + 30 = 60
        assert!((sb.kala_bala - 60.0).abs() < f64::EPSILON);
        // Cheshta: slow (0.02 < 0.05*0.5=0.025) = 15
        assert!((sb.cheshta_bala - 15.0).abs() < f64::EPSILON);
        // Naisargika: 8.57
        assert!((sb.naisargika_bala - 8.57).abs() < 0.01);
        // Drik: 0 - 0 = 0
        assert!((sb.drik_bala - 0.0).abs() < f64::EPSILON);
    }
}
