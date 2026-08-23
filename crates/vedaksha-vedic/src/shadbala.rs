// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Shadbala (six-fold planetary strength).
//!
//! Computes the six components of planetary strength used in Vedic astrology
//! to assess how effectively a planet can deliver its results. Implements all
//! six components: Sthana Bala (positional), Dig Bala (directional), Kala Bala
//! (temporal), Cheshta Bala (motional), Naisargika Bala (natural), and Drik
//! Bala (aspectual).
//!
//! Source: BPHS Ch. 27.
//!
//! # Sub-component coverage
//!
//! Four of the six components are complete. Two are partial, and the gap is
//! named here rather than left for a caller to discover from a wrong number:
//!
//! * **Sthana Bala** is the sum of five sub-components. Four are implemented —
//!   Uchcha, Ojhayugma, Kendradi and Drekkana Bala. **Saptavargaja Bala, the
//!   planet's dignity across the seven vargas, is not**, because it requires a
//!   Moolatrikona degree table and a panchadha maitri (five-fold friendship)
//!   derivation that this crate does not yet carry from a primary source. A
//!   planet's Sthana Bala is therefore its true value less its Saptavargaja
//!   term, and is bounded by 165 virupas rather than the classical maximum.
//! * **Kala Bala** implements Nathonnatha and Paksha Bala. Tribhaga, the
//!   Varsha/Masa/Vara/Hora set, Ayana and Yuddha Bala are absent.
//!
//! Every other component — Dig, Cheshta, Naisargika and Drik Bala — is whole.

use crate::graha::{Graha, GrahaPosition};
use crate::varga::{VargaType, varga_sign};
use serde::Serialize;

/// Shadbala (six-fold strength) for a planet.
///
/// All component values are in virupas (shashtiamsas, 1/60 of a rupa).
#[derive(Debug, Clone, Serialize)]
pub struct Shadbala {
    /// The planet this strength applies to.
    pub planet: Graha,
    /// Positional strength: the sum of `uccha_bala`, `ojhayugma_bala`,
    /// `kendradi_bala` and `drekkana_bala`. Saptavargaja Bala is not
    /// included — see the module documentation.
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
    /// Sum of the six components above.
    pub total: f64,
    /// Uchcha Bala — exaltation sub-component of Sthana Bala (0–60
    /// virupas). Source: BPHS Ch.27.
    pub uccha_bala: f64,
    /// Ojhayugma Bala — odd/even rasi and navamsa sub-component of Sthana
    /// Bala (0–30 virupas). Source: BPHS Ch.27.
    pub ojhayugma_bala: f64,
    /// Kendradi Bala — angular/succedent/cadent sub-component of Sthana
    /// Bala (15–60 virupas). Source: BPHS Ch.27.
    pub kendradi_bala: f64,
    /// Drekkana Bala — decanate sub-component of Sthana Bala (0 or 15
    /// virupas). Source: BPHS Ch.27.
    pub drekkana_bala: f64,
    /// Benefit strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 vv.5-6.
    pub ishta_phala: f64,
    /// Affliction strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 v.6.
    pub kashta_phala: f64,
}

/// Additional temporal/motional parameters the six-fold computation needs
/// beyond a bare [`GrahaPosition`].
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

/// Uchcha Bala — degree-precise exaltation strength in virupas (0-60).
///
/// Defining rule (BPHS Ch. 27 Sl. 3-6): a planet at its exaltation degree has
/// 60 virupas, at its debilitation degree 0, and the value falls linearly with
/// the arc between:
///
/// ```text
/// uccha_bala = (180 - arc) / 3
/// arc        = min(|longitude - exaltation_longitude|,
///                  360 - |longitude - exaltation_longitude|)
/// ```
///
/// This is one of the five sub-components of Sthana Bala, not Sthana Bala
/// itself. Through v7.1.1 it was exported under the name `sthana_bala` and
/// took a `sign` argument it discarded; see [`sthana_bala_full`] for the
/// composite, and [`sthana_bala`] for the retained v7 spelling.
#[must_use]
pub fn uccha_bala(planet: Graha, longitude: f64) -> f64 {
    let exalt_lon = exaltation_longitude(planet);
    let raw_diff = (longitude - exalt_lon).abs();
    let arc = if raw_diff > 180.0 {
        360.0 - raw_diff
    } else {
        raw_diff
    };
    ((180.0 - arc) / 3.0).max(0.0)
}

/// Ojhayugma Bala — odd/even strength in virupas (0-30).
///
/// Defining rule (BPHS Ch. 27): the Moon and Venus are strengthened by even
/// (yugma) signs; the other five grahas by odd (oja) signs. The test is
/// applied twice — once to the rasi (D-1) and once to the navamsa (D-9) — and
/// each agreement is worth 15 virupas.
///
/// Sign indices are 0-based, so an *odd* sign in the classical 1-based
/// reckoning (Aries, Gemini, …) is an *even* index here.
///
/// Rahu and Ketu are outside the classical rule and score 0.
#[must_use]
pub fn ojhayugma_bala(planet: Graha, longitude: f64) -> f64 {
    if matches!(planet, Graha::Rahu | Graha::Ketu) {
        return 0.0;
    }
    // Classical "odd sign" == 0-based index even.
    let wants_even_sign = matches!(planet, Graha::Moon | Graha::Venus);

    let rasi = varga_sign(longitude, VargaType::Rashi);
    let navamsa = varga_sign(longitude, VargaType::Navamsha);

    let mut bala = 0.0;
    for sign in [rasi, navamsa] {
        let sign_is_classically_even = sign % 2 == 1;
        if sign_is_classically_even == wants_even_sign {
            bala += 15.0;
        }
    }
    bala
}

/// Kendradi Bala — quadrant strength in virupas (15, 30 or 60).
///
/// Defining rule (BPHS Ch. 27): a planet in a kendra (houses 1, 4, 7, 10)
/// scores 60 virupas; in a panapara (2, 5, 8, 11) 30; in an apoklima
/// (3, 6, 9, 12) 15. The rule depends only on the house, not on the planet.
///
/// A `bhava` outside 1-12 scores 0.
#[must_use]
pub fn kendradi_bala(bhava: u8) -> f64 {
    match bhava {
        1 | 4 | 7 | 10 => 60.0,
        2 | 5 | 8 | 11 => 30.0,
        3 | 6 | 9 | 12 => 15.0,
        _ => 0.0,
    }
}

/// Drekkana Bala — decanate strength in virupas (0 or 15).
///
/// Defining rule (BPHS Ch. 27): the male grahas (Sun, Mars, Jupiter) gain 15
/// virupas in the first drekkana of whatever sign they occupy, the neuter
/// grahas (Mercury, Saturn) in the second, and the female grahas (Moon,
/// Venus) in the third. There is no partial credit.
///
/// The drekkana here is the third of the *occupied sign* — degrees 0-10,
/// 10-20 and 20-30 within it — which is what the rule is stated over, not the
/// D-3 varga sign that [`crate::varga::varga_sign`] returns.
///
/// Rahu and Ketu are outside the classical rule and score 0.
#[must_use]
pub fn drekkana_bala(planet: Graha, longitude: f64) -> f64 {
    let wanted = match planet {
        Graha::Sun | Graha::Mars | Graha::Jupiter => 0u8,
        Graha::Mercury | Graha::Saturn => 1,
        Graha::Moon | Graha::Venus => 2,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    let within_sign = longitude.rem_euclid(360.0) % 30.0;
    // 0-10 deg -> 0, 10-20 -> 1, 20-30 -> 2. `min` guards the 30.0 boundary
    // that floating-point rounding can produce.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = ((within_sign / 10.0) as u8).min(2);
    if index == wanted { 15.0 } else { 0.0 }
}

/// Sthana Bala — positional strength in virupas (0-165).
///
/// Named `_full` for the same reason [`compute_shadbala_full`] is: the plain
/// spelling is taken by the v7 function this replaces.
///
/// The sum of the four implemented sub-components: [`uccha_bala`],
/// [`ojhayugma_bala`], [`kendradi_bala`] and [`drekkana_bala`].
///
/// **Saptavargaja Bala is not included.** BPHS Ch. 27 makes it the fifth
/// sub-component — the planet's dignity in each of the seven vargas, graded
/// by the five-fold friendship — and computing it needs a Moolatrikona degree
/// table and a panchadha maitri derivation this crate does not yet carry from
/// a primary source. Inventing either would be worse than the documented gap,
/// so the value returned here is the true Sthana Bala less that term.
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn sthana_bala_full(planet: Graha, longitude: f64, bhava: u8) -> f64 {
    uccha_bala(planet, longitude)
        + ojhayugma_bala(planet, longitude)
        + kendradi_bala(bhava)
        + drekkana_bala(planet, longitude)
}

/// Uchcha Bala under its historical name.
///
/// This is what `sthana_bala` returned through v7.1.1: the Uchcha
/// sub-component alone, with `sign` accepted and discarded. It is kept at that
/// exact behaviour so a v7 dependent keeps compiling and keeps getting the
/// number it got before, and is deprecated in favour of the two functions that
/// say what they are — [`uccha_bala`] for this value, [`sthana_bala_full`] for
/// the composite Sthana Bala.
#[must_use]
#[deprecated(
    since = "7.2.0",
    note = "returns Uchcha Bala only, not Sthana Bala: call uccha_bala for this value, or sthana_bala_full for the composite"
)]
pub fn sthana_bala(planet: Graha, _sign: u8, longitude: f64) -> f64 {
    uccha_bala(planet, longitude)
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

// ── Compute Shadbala ────────────────────────────────────────────────

/// Compute Shadbala from bare positions — Sthana, Dig and Naisargika Bala only.
///
/// Kala, Cheshta and Drik Bala need data a [`GrahaPosition`] does not carry, so
/// they are zero and `total` is the sum of the three that are computed, not of
/// six. That was true before v7.2.0 too, where the struct documented `total` as
/// a six-component sum regardless. Use [`compute_shadbala_full`].
///
/// `lagna_sign` is accepted for signature compatibility and not read.
#[must_use]
#[deprecated(
    since = "7.2.0",
    note = "computes three of the six components; use compute_shadbala_full"
)]
pub fn compute_shadbala(positions: &[GrahaPosition], _lagna_sign: u8) -> Vec<Shadbala> {
    positions
        .iter()
        .map(|pos| {
            let naisargika = naisargika_bala(pos.planet);
            let dig = dig_bala(pos.planet, pos.bhava);

            let uccha = uccha_bala(pos.planet, pos.longitude);
            let ojhayugma = ojhayugma_bala(pos.planet, pos.longitude);
            let kendradi = kendradi_bala(pos.bhava);
            let drekkana = drekkana_bala(pos.planet, pos.longitude);
            let sthana = uccha + ojhayugma + kendradi + drekkana;

            let (ishta, kashta) = ishta_kashta_phala(uccha, 0.0);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: 0.0,
                cheshta_bala: 0.0,
                naisargika_bala: naisargika,
                drik_bala: 0.0,
                total: naisargika + dig + sthana,
                uccha_bala: uccha,
                ojhayugma_bala: ojhayugma,
                kendradi_bala: kendradi,
                drekkana_bala: drekkana,
                ishta_phala: ishta,
                kashta_phala: kashta,
            }
        })
        .collect()
}

/// Compute Shadbala — all six components — for planets with extended data.
///
/// Sthana Bala omits Saptavargaja Bala and Kala Bala covers Nathonnatha and
/// Paksha only; see the module documentation for the full accounting.
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

            let uccha = uccha_bala(pos.planet, pos.longitude);
            let ojhayugma = ojhayugma_bala(pos.planet, pos.longitude);
            let kendradi = kendradi_bala(pos.bhava);
            let drekkana = drekkana_bala(pos.planet, pos.longitude);
            let sthana = uccha + ojhayugma + kendradi + drekkana;

            let kala = kala_bala(pos.planet, is_daytime, moon_phase_waxing);
            let cheshta = cheshta_bala(pos.planet, data.speed, data.average_speed);
            let drik = drik_bala(data.benefic_aspect_count, data.malefic_aspect_count);
            let total = naisargika + dig + sthana + kala + cheshta + drik;

            // BPHS Ch.28 v.5 builds the Rasmis from Uchcha Bala, not from the
            // Sthana Bala composite that contains it.
            let (ishta, kashta) = ishta_kashta_phala(uccha, cheshta);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: kala,
                cheshta_bala: cheshta,
                naisargika_bala: naisargika,
                drik_bala: drik,
                total,
                uccha_bala: uccha,
                ojhayugma_bala: ojhayugma,
                kendradi_bala: kendradi,
                drekkana_bala: drekkana,
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

    // ── Uccha Bala tests ────────────────────────────────────────────

    #[test]
    fn uccha_bala_exact_exaltation_is_60() {
        // Sun exalted at 10° Aries → 60 virupas
        let bala = uccha_bala(Graha::Sun, 10.0);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_exact_debilitation_is_0() {
        // Sun debilitated at 190° (10+180) → 0 virupas
        let bala = uccha_bala(Graha::Sun, 190.0);
        assert!((bala - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_midpoint_is_30() {
        // Sun at 100° → 90° from exaltation (10°) → (180-90)/3 = 30 virupas
        let bala = uccha_bala(Graha::Sun, 100.0);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_gradient_is_continuous() {
        // Jupiter at exact exaltation (95°) should be stronger than at 100°
        let at_exalt = uccha_bala(Graha::Jupiter, 95.0);
        let near_exalt = uccha_bala(Graha::Jupiter, 100.0);
        assert!(at_exalt > near_exalt);
    }

    // ── Ojhayugma Bala tests ────────────────────────────────────────

    #[test]
    fn ojhayugma_rewards_odd_signs_for_the_sun() {
        // 5° Aries: rasi index 0 (classically odd), navamsa of 5° Aries is
        // index 1 (classically even). The Sun wants odd, so it scores the
        // rasi half only.
        assert!((ojhayugma_bala(Graha::Sun, 5.0) - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ojhayugma_is_inverted_for_moon_and_venus() {
        // Same longitude, opposite preference: the Moon takes the navamsa
        // half where the Sun took the rasi half. The two must therefore sum
        // to the full 30 virupas at every longitude.
        for step in 0..360 {
            let lon = f64::from(step);
            let sun = ojhayugma_bala(Graha::Sun, lon);
            let moon = ojhayugma_bala(Graha::Moon, lon);
            assert!(
                (sun + moon - 30.0).abs() < f64::EPSILON,
                "at {lon}°: sun {sun} + moon {moon} != 30"
            );
        }
    }

    #[test]
    fn ojhayugma_is_zero_for_the_nodes() {
        assert!(ojhayugma_bala(Graha::Rahu, 5.0).abs() < f64::EPSILON);
        assert!(ojhayugma_bala(Graha::Ketu, 5.0).abs() < f64::EPSILON);
    }

    // ── Kendradi Bala tests ─────────────────────────────────────────

    #[test]
    fn kendradi_grades_the_three_house_classes() {
        for kendra in [1, 4, 7, 10] {
            assert!((kendradi_bala(kendra) - 60.0).abs() < f64::EPSILON);
        }
        for panapara in [2, 5, 8, 11] {
            assert!((kendradi_bala(panapara) - 30.0).abs() < f64::EPSILON);
        }
        for apoklima in [3, 6, 9, 12] {
            assert!((kendradi_bala(apoklima) - 15.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn kendradi_covers_every_house_and_rejects_others() {
        for bhava in 1..=12u8 {
            assert!(kendradi_bala(bhava) > 0.0, "house {bhava} scored nothing");
        }
        assert!(kendradi_bala(0).abs() < f64::EPSILON);
        assert!(kendradi_bala(13).abs() < f64::EPSILON);
    }

    // ── Drekkana Bala tests ─────────────────────────────────────────

    #[test]
    fn drekkana_rewards_each_sex_in_its_own_third() {
        // Third of the occupied sign, not the D-3 varga sign.
        let first = 5.0; // 5° Aries
        let second = 15.0;
        let third = 25.0;

        for male in [Graha::Sun, Graha::Mars, Graha::Jupiter] {
            assert!((drekkana_bala(male, first) - 15.0).abs() < f64::EPSILON);
            assert!(drekkana_bala(male, second).abs() < f64::EPSILON);
            assert!(drekkana_bala(male, third).abs() < f64::EPSILON);
        }
        for neuter in [Graha::Mercury, Graha::Saturn] {
            assert!(drekkana_bala(neuter, first).abs() < f64::EPSILON);
            assert!((drekkana_bala(neuter, second) - 15.0).abs() < f64::EPSILON);
        }
        for female in [Graha::Moon, Graha::Venus] {
            assert!(drekkana_bala(female, first).abs() < f64::EPSILON);
            assert!((drekkana_bala(female, third) - 15.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn drekkana_index_never_escapes_the_sign() {
        // Every longitude, including the sign boundaries and a negative one,
        // must land in exactly one of the three thirds — so across the three
        // sexes precisely 15 virupas is awarded at every degree.
        for step in 0..3600 {
            let lon = f64::from(step) / 10.0 - 180.0;
            let awarded = drekkana_bala(Graha::Sun, lon)
                + drekkana_bala(Graha::Mercury, lon)
                + drekkana_bala(Graha::Moon, lon);
            assert!(
                (awarded - 15.0).abs() < f64::EPSILON,
                "at {lon}°: {awarded} virupas awarded across the three thirds"
            );
        }
    }

    // ── Sthana Bala composite tests ─────────────────────────────────

    #[test]
    fn sthana_bala_is_the_sum_of_its_four_sub_components() {
        for planet in [Graha::Sun, Graha::Moon, Graha::Saturn, Graha::Rahu] {
            for bhava in 1..=12u8 {
                let lon = f64::from(bhava) * 27.3;
                let parts = uccha_bala(planet, lon)
                    + ojhayugma_bala(planet, lon)
                    + kendradi_bala(bhava)
                    + drekkana_bala(planet, lon);
                let composite = sthana_bala_full(planet, lon, bhava);
                assert!(
                    (composite - parts).abs() < f64::EPSILON,
                    "{planet:?} at {lon}° house {bhava}: {composite} != {parts}"
                );
            }
        }
    }

    #[test]
    fn sthana_bala_depends_on_the_house_it_used_to_ignore() {
        // The defect this replaced: sthana_bala returned Uccha Bala alone, so
        // two placements differing only in house scored identically.
        let angular = sthana_bala_full(Graha::Sun, 10.0, 1);
        let cadent = sthana_bala_full(Graha::Sun, 10.0, 3);
        assert!(
            (angular - cadent - 45.0).abs() < f64::EPSILON,
            "kendra {angular} vs apoklima {cadent}"
        );
    }

    #[test]
    fn sthana_bala_stays_within_its_documented_bound() {
        for step in 0..720 {
            let lon = f64::from(step) / 2.0;
            for bhava in 1..=12u8 {
                for planet in [Graha::Sun, Graha::Moon, Graha::Venus, Graha::Saturn] {
                    let bala = sthana_bala_full(planet, lon, bhava);
                    assert!(
                        (0.0..=165.0).contains(&bala),
                        "{planet:?} at {lon}° house {bhava}: {bala} outside 0..=165"
                    );
                }
            }
        }
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

    // ── Full Shadbala tests ─────────────────────────────────────────

    #[test]
    fn full_shadbala_includes_all_six_components() {
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        assert_eq!(results.len(), 1);
        let sb = &results[0];

        // Sthana for Jupiter at 105° (15° Cancer) in the 4th, worked by hand:
        //   uccha     — exaltation 95°, arc 10° -> (180-10)/3 = 56.667
        //   ojhayugma — rasi Cancer (index 3, classically even) and navamsa
        //               Scorpio (index 7, classically even); Jupiter wants
        //               odd, so neither half scores -> 0
        //   kendradi  — 4th house is a kendra -> 60
        //   drekkana  — 15° into the sign is the second third; Jupiter is
        //               male and wants the first -> 0
        let expected_uccha = (180.0 - 10.0) / 3.0;
        let expected_sthana = expected_uccha + 0.0 + 60.0 + 0.0;
        assert!(
            (sb.sthana_bala - expected_sthana).abs() < 0.01,
            "sthana {} != {expected_sthana}",
            sb.sthana_bala
        );
        assert!((sb.uccha_bala - expected_uccha).abs() < 0.01);
        assert!(sb.ojhayugma_bala.abs() < f64::EPSILON);
        assert!((sb.kendradi_bala - 60.0).abs() < f64::EPSILON);
        assert!(sb.drekkana_bala.abs() < f64::EPSILON);
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
            // Not f64::EPSILON: `sthana_bala` is itself a sum, so `total`
            // and this re-addition associate the same terms differently and
            // differ by an ulp of ~250, not of 1. Any real omission is at
            // least a whole virupa.
            assert!(
                (sb.total - sum).abs() < 1e-9,
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
    fn the_pipeline_and_the_standalone_sthana_bala_agree() {
        // `compute_shadbala_full` computes the four sub-components inline so
        // it can report each one, which means it does not go through
        // `sthana_bala`. The two must not be allowed to drift apart.
        for planet in [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Venus] {
            for bhava in 1..=12u8 {
                let sign = (bhava - 1) % 12;
                let data = [planet_data(planet, sign, bhava, 0.5, 0.5, 0, 0)];
                let sb = &compute_shadbala_full(&data, true, true)[0];
                let standalone = sthana_bala_full(planet, data[0].position.longitude, bhava);
                assert!(
                    (sb.sthana_bala - standalone).abs() < 1e-9,
                    "{planet:?} house {bhava}: pipeline {} vs standalone {standalone}",
                    sb.sthana_bala
                );
            }
        }
    }

    #[test]
    #[allow(deprecated)]
    fn the_v7_spellings_still_behave_as_they_did() {
        // These two exist only so a `vedaksha-vedic = "7"` dependent keeps
        // compiling across this minor release. `sthana_bala` must still be
        // Uchcha Bala with the sign discarded, and `compute_shadbala` must
        // still total the three components it can reach.
        for lon in [0.0, 10.0, 95.0, 190.0, 285.0, 359.9] {
            for sign in 0..12u8 {
                assert!(
                    (sthana_bala(Graha::Sun, sign, lon) - uccha_bala(Graha::Sun, lon)).abs()
                        < f64::EPSILON,
                    "sign {sign} changed the answer at {lon}°"
                );
            }
        }

        let positions = [pos(Graha::Sun, 0, 10)];
        let sb = &compute_shadbala(&positions, 0)[0];
        assert!(sb.kala_bala.abs() < f64::EPSILON);
        assert!(sb.cheshta_bala.abs() < f64::EPSILON);
        assert!(sb.drik_bala.abs() < f64::EPSILON);
        let three = sb.sthana_bala + sb.dig_bala + sb.naisargika_bala;
        assert!((sb.total - three).abs() < 1e-9, "{} != {three}", sb.total);
    }

    #[test]
    fn uccha_bala_is_a_part_of_sthana_bala_not_a_copy_of_it() {
        // Through v7.1.1 these two fields carried the same number, because
        // `sthana_bala` was Uccha Bala under another name. They must now
        // differ by exactly the other three sub-components.
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        let sb = &results[0];
        let rest = sb.ojhayugma_bala + sb.kendradi_bala + sb.drekkana_bala;
        assert!(
            rest > 0.0,
            "sub-components all zero — nothing is being tested"
        );
        assert!((sb.sthana_bala - sb.uccha_bala - rest).abs() < 1e-9);
        assert!(sb.uccha_bala < sb.sthana_bala);
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
        assert!((sb.total - sum).abs() < 1e-9);
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

        // Sthana for Saturn at 285° (15° Capricorn) in the 7th, by hand:
        //   uccha     — exaltation 200°, arc 85° -> (180-85)/3 = 31.667
        //   ojhayugma — rasi Capricorn (index 9) and navamsa Taurus (index 1)
        //               are both classically even; Saturn wants odd -> 0
        //   kendradi  — 7th house is a kendra -> 60
        //   drekkana  — second third of the sign, which is what the neuter
        //               grahas want -> 15
        let expected_uccha = (180.0 - 85.0) / 3.0;
        let expected_sthana = expected_uccha + 0.0 + 60.0 + 15.0;
        assert!(
            (sb.sthana_bala - expected_sthana).abs() < 0.01,
            "sthana {} != {expected_sthana}",
            sb.sthana_bala
        );
        assert!((sb.drekkana_bala - 15.0).abs() < f64::EPSILON);
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

        // BPHS Ch.28 v.5 builds the Rasmis from Uchcha Bala. Feeding it the
        // Sthana composite instead — which is what the code did while the two
        // were the same number — drives uchcha_rasmi to 12.4 and clamps ishta
        // to its 60 ceiling, so the error hides unless it is asserted here.
        //   uchcha_rasmi  = 31.667 * 7/60 = 3.6944
        //   cheshta_rasmi = 15     * 7/60 = 1.75
        //   ishta = 5 * (3.6944 + 1.75 - 2) = 17.222
        assert!(
            (sb.ishta_phala - 17.2222).abs() < 0.001,
            "ishta {} — is it being fed Sthana Bala rather than Uccha Bala?",
            sb.ishta_phala
        );
        assert!((sb.kashta_phala - 42.7778).abs() < 0.001);
    }
}
