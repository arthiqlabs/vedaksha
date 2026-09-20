// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Vishesha (special) lagnas — BPHS, the chapter on special ascendants.
//!
//! Hora, Ghati and Bhava lagnas start at the Sun's sidereal longitude at
//! sunrise and advance uniformly: one rashi per 2.5 ghatis (Hora), per
//! 1 ghati (Ghati), per 5 ghatis (Bhava). One civil day is 60 ghatis, so
//! the rates are 12°, 30° and 6° per ghati respectively.
//!
//! # Contract
//!
//! The Sun's tropical longitude comes from the caller's `tropical_sun`
//! callback (degrees, any frame the caller can supply consistently — only
//! the sunrise value is read); `ayanamsha` converts it to sidereal at
//! sunrise. Returns `None` where the callback is unavailable. All longitudes
//! are sidereal degrees in [0, 360), locale-free.

use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};

use crate::graha::Graha;

/// One civil day is 60 ghatis.
const GHATIS_PER_DAY: f64 = 60.0;

/// Sidereal Sun at sunrise, or `None` where the callback is unavailable.
fn sidereal_sun_at_rise(
    jd_sunrise: f64,
    tropical_sun: &(dyn Fn(f64) -> Option<f64> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<f64> {
    let trop = tropical_sun(jd_sunrise)?;
    Some((trop - true_ayanamsha_value(ayanamsha, jd_sunrise)).rem_euclid(360.0))
}

/// Uniform-advance special lagna: `ghatis_per_rashi` per 30°.
fn advancing_lagna(
    jd_sunrise: f64,
    jd_birth: f64,
    tropical_sun: &(dyn Fn(f64) -> Option<f64> + Sync),
    ayanamsha: Ayanamsha,
    ghatis_per_rashi: f64,
) -> Option<f64> {
    let rise = sidereal_sun_at_rise(jd_sunrise, tropical_sun, ayanamsha)?;
    let elapsed_ghatis = (jd_birth - jd_sunrise) * GHATIS_PER_DAY;
    Some((rise + elapsed_ghatis * 30.0 / ghatis_per_rashi).rem_euclid(360.0))
}

/// Hora Lagna — one rashi per 2.5 ghatis from the sunrise Sun.
#[must_use]
pub fn hora_lagna(
    jd_sunrise: f64,
    jd_birth: f64,
    tropical_sun: &(dyn Fn(f64) -> Option<f64> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<f64> {
    advancing_lagna(jd_sunrise, jd_birth, tropical_sun, ayanamsha, 2.5)
}

/// Ghati Lagna — one rashi per ghati from the sunrise Sun.
#[must_use]
pub fn ghati_lagna(
    jd_sunrise: f64,
    jd_birth: f64,
    tropical_sun: &(dyn Fn(f64) -> Option<f64> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<f64> {
    advancing_lagna(jd_sunrise, jd_birth, tropical_sun, ayanamsha, 1.0)
}

/// Bhava Lagna — one rashi per 5 ghatis from the sunrise Sun.
#[must_use]
pub fn bhava_lagna(
    jd_sunrise: f64,
    jd_birth: f64,
    tropical_sun: &(dyn Fn(f64) -> Option<f64> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<f64> {
    advancing_lagna(jd_sunrise, jd_birth, tropical_sun, ayanamsha, 5.0)
}

/// One pala is 24 s: 3600 palas to the civil day.
const PALAS_PER_DAY: f64 = 3600.0;

/// Pranapada Lagna — sidereal longitude in [0, 360).
///
/// Elapsed palas since sunrise divided by 15 give signs (the fraction gives
/// degrees: 15 palas are one sign), added to the Sun's longitude — plus
/// 240° when the Sun stands in a fixed rashi (Vrshabha, Simha, Vrshchika,
/// Kumbha) or 120° in a dual one (Mithuna, Kanya, Dhanus, Mina); nothing
/// added in a movable rashi (Mesha, Karka, Tula, Makara).
///
/// Source: BPHS, the chapter on special ascendants.
#[must_use]
pub fn pranapada(surya_longitude_deg: f64, jd_sunrise: f64, jd_birth: f64) -> f64 {
    let elapsed_palas = (jd_birth - jd_sunrise) * PALAS_PER_DAY;
    let arc = elapsed_palas / 15.0 * 30.0;
    let rashi = (surya_longitude_deg.rem_euclid(360.0) / 30.0).floor() as u8 % 12;
    let quality = match rashi {
        0 | 3 | 6 | 9 => 0.0,    // movable
        1 | 4 | 7 | 10 => 240.0, // fixed
        _ => 120.0,              // dual
    };
    (surya_longitude_deg + quality + arc).rem_euclid(360.0)
}

/// Sri Lagna — Janma Lagna advanced by the Moon's progress through its
/// birth nakshatra, the nakshatra reckoned as twelve signs: each completed
/// pada contributes three signs, the partial pada proportionally.
///
/// Source: BPHS, the chapter on special ascendants.
#[must_use]
pub fn sri_lagna(moon_sidereal_lon_deg: f64, janma_lagna_lon_deg: f64) -> f64 {
    const SPAN: f64 = 360.0 / 27.0;
    let frac = moon_sidereal_lon_deg.rem_euclid(360.0).rem_euclid(SPAN) / SPAN;
    (janma_lagna_lon_deg + frac * 360.0).rem_euclid(360.0)
}

/// 0-based rashi index ⇔ 1-based odd rashi (Mesha 0 = 1st = odd): even
/// index means odd rashi.
fn is_odd_rashi(rashi: u8) -> bool {
    rashi.is_multiple_of(2)
}

/// Kala (strength units) of a 9th lord for [`indu_lagna`]: Sun 30, Moon 16,
/// Mars 6, Mercury 8, Jupiter 10, Venus 12, Saturn 1. The nodes own no sign
/// and must not occur here: this panics (a caller bug, in every profile —
/// a silent 0 would return a confident wrong rashi).
fn kala(lord: Graha) -> u32 {
    match lord {
        Graha::Sun => 30,
        Graha::Moon => 16,
        Graha::Mars => 6,
        Graha::Mercury => 8,
        Graha::Jupiter => 10,
        Graha::Venus => 12,
        Graha::Saturn => 1,
        Graha::Rahu | Graha::Ketu => panic!("nodes own no sign; not 9th lords"),
    }
}

/// Indu Lagna — the rashi index (0 = Mesha) reached by counting, inclusively
/// from the Moon's rashi, the remainder of the two 9th lords' kalas summed
/// and divided by 12 (a zero remainder counts 12).
///
/// The lords are parameters because sign lordship (co-lordships of
/// Vrshchika and Kumbha) is doctrine, not computation: pass the 9th lord
/// from Janma Lagna and from the Moon under the caller's school.
///
/// Source: BPHS, the chapter on special ascendants.
///
/// # Panics
///
/// Panics when either lord is Rahu or Ketu (via [`kala`]) — the nodes own
/// no sign, so there is no 9th-lordship to compute.
#[must_use]
pub fn indu_lagna(moon_rashi: u8, ninth_lord_from_lagna: Graha, ninth_lord_from_moon: Graha) -> u8 {
    let moon = moon_rashi % 12;
    let total = kala(ninth_lord_from_lagna) + kala(ninth_lord_from_moon);
    let mut rem = total % 12;
    if rem == 0 {
        rem = 12;
    }
    (moon + ((rem - 1) % 12) as u8) % 12
}

/// Varnada Lagna — the rashi index (0 = Mesha).
///
/// Each of Janma Lagna and Hora Lagna is counted — inclusively from Mesha
/// forward for an odd rashi (1st, 3rd, ...), inclusively from Mina backward
/// for an even one — under its OWN parity. `is_odd_count` carries the Janma
/// Lagna's parity (true = odd) and MUST agree with `lagna_rashi`
/// (`lagna_rashi % 2 == 0` ⇔ odd — Mesha is 0-based 0 = 1st); it exists
/// because the request's contract names the counting direction explicitly,
/// and disagreement is a caller bug, caught by assertion in every profile. The Hora
/// Lagna's parity is read from `hora_lagna_rashi` the same way. The two
/// counts are added when the parities agree, differenced when they differ;
/// the result is counted from Mesha forward if odd, from Mina backward if
/// even (wrapping past 12).
///
/// Source: BPHS, the chapter on special ascendants.
///
/// # Panics
///
/// Panics when `is_odd_count` disagrees with `lagna_rashi`'s own parity —
/// a caller bug, not a computable case.
#[must_use]
pub fn varnada_lagna(lagna_rashi: u8, hora_lagna_rashi: u8, is_odd_count: bool) -> u8 {
    let lagna = lagna_rashi % 12;
    assert_eq!(
        is_odd_count,
        is_odd_rashi(lagna),
        "is_odd_count must agree with lagna_rashi parity"
    );
    let hora = hora_lagna_rashi % 12;
    // 0-based Mesha = 1st = odd.
    let lagna_odd = is_odd_count;
    let hora_odd = is_odd_rashi(hora);
    // Inclusive count Mesha -> rashi, or Mina -> rashi backward.
    let count = |rashi: u8, odd: bool| -> u32 {
        if odd {
            u32::from(rashi) + 1
        } else {
            u32::from((11 + 12 - rashi) % 12) + 1
        }
    };
    let n1 = count(lagna, lagna_odd);
    let n2 = count(hora, hora_odd);
    let total: u32 = if lagna_odd == hora_odd {
        n1 + n2
    } else {
        n1.abs_diff(n2)
    };
    // A zero difference counts a full twelve.
    let steps = if total == 0 { 12 } else { total };
    if steps % 2 == 1 {
        // Odd: forward from Mesha.
        ((steps - 1) % 12) as u8
    } else {
        // Even: backward from Mina.
        ((11 + 12 - ((steps - 1) % 12)) % 12) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed Sun: tropical 100°, IndianOfficial ayanamsha at J2000 ≈ 23.85°.
    // Tests pin RATES and the sunrise anchor, not the ayanamsha value.
    fn flat_sun(_jd: f64) -> Option<f64> {
        Some(100.0)
    }

    fn rise(ayanamsha: Ayanamsha) -> f64 {
        sidereal_sun_at_rise(2_451_545.0, &flat_sun, ayanamsha).unwrap()
    }

    #[test]
    fn at_sunrise_all_three_equal_the_sun() {
        let jd = 2_451_545.0;
        let a = Ayanamsha::IndianOfficial;
        assert_eq!(hora_lagna(jd, jd, &flat_sun, a).unwrap(), rise(a));
        assert_eq!(ghati_lagna(jd, jd, &flat_sun, a).unwrap(), rise(a));
        assert_eq!(bhava_lagna(jd, jd, &flat_sun, a).unwrap(), rise(a));
    }

    #[test]
    fn rates_per_ghati() {
        // 2.5 ghatis after sunrise: Hora +30°, Ghati +75°, Bhava +15°.
        // Tolerance 1e-6°: JD differences near J2000 resolve only to ~3e-7°.
        let jd = 2_451_545.0;
        let birth = jd + 2.5 / 60.0;
        let a = Ayanamsha::IndianOfficial;
        let r = rise(a);
        assert!((hora_lagna(jd, birth, &flat_sun, a).unwrap() - (r + 30.0)).abs() < 1e-6);
        assert!(
            (ghati_lagna(jd, birth, &flat_sun, a).unwrap() - (r + 75.0).rem_euclid(360.0)).abs()
                < 1e-6
        );
        assert!((bhava_lagna(jd, birth, &flat_sun, a).unwrap() - (r + 15.0)).abs() < 1e-6);
    }

    #[test]
    fn unavailable_ephemeris_is_none() {
        let no: fn(f64) -> Option<f64> = |_| None;
        let a = Ayanamsha::IndianOfficial;
        assert_eq!(hora_lagna(2_451_545.0, 2_451_545.1, &no, a), None);
    }

    #[test]
    fn pranapada_anchor_depends_on_sun_sign_quality() {
        let jd = 2_451_545.0;
        // Movable Sun (Mesha 10°): pranapada IS the Sun at sunrise.
        assert!((pranapada(10.0, jd, jd) - 10.0).abs() < 1e-9);
        // Fixed Sun (Vrshabha 45°): +240°.
        assert!((pranapada(45.0, jd, jd) - 285.0).abs() < 1e-9);
        // Dual Sun (Mithuna 70°): +120° = 190°.
        assert!((pranapada(70.0, jd, jd) - 190.0).abs() < 1e-9);
        // 15 palas later the arc advances one full sign (30°). Tolerance
        // 1e-4°: JD differences near J2000 resolve only to ~2e-6°.
        assert!((pranapada(10.0, jd, jd + 15.0 / 3600.0) - 40.0).abs() < 1e-4);
    }

    #[test]
    fn sri_lagna_advances_twelve_signs_per_nakshatra() {
        const SPAN: f64 = 360.0 / 27.0;
        // Moon at the nakshatra's opening: Sri IS the Janma Lagna.
        assert!((sri_lagna(0.0, 123.0) - 123.0).abs() < 1e-9);
        // One pada in: +3 signs.
        assert!((sri_lagna(SPAN / 4.0, 0.0) - 90.0).abs() < 1e-9);
        // Half the nakshatra: opposite the lagna.
        assert!((sri_lagna(SPAN / 2.0, 10.0) - 190.0).abs() < 1e-9);
    }

    #[test]
    fn indu_lagna_counts_kala_remainder_from_the_moon() {
        // 9th from Mesha = Dhanus (Jupiter, 10); 9th from Karka = Mina
        // (Jupiter, 10): total 20, remainder 8, count 8 from Karka (3).
        assert_eq!(indu_lagna(3, Graha::Jupiter, Graha::Jupiter), 10);
        // Sun (30) + Saturn (1) = 31, remainder 7, from Simha (4) -> 10.
        assert_eq!(indu_lagna(4, Graha::Sun, Graha::Saturn), 10);
        // Zero remainder counts twelve: Venus (12) + Venus (12) = 24.
        assert_eq!(indu_lagna(0, Graha::Venus, Graha::Venus), 11);
    }

    #[test]
    #[should_panic(expected = "nodes own no sign")]
    fn indu_lagna_nodes_panic() {
        let _ = indu_lagna(0, Graha::Rahu, Graha::Jupiter);
    }

    #[test]
    fn varnada_same_parity_adds() {
        // Lagna Mesha (odd, count 1), HL Simha (odd, count 5): 1 + 5 = 6,
        // even, backward from Mina -> Kanya (6).
        assert_eq!(varnada_lagna(0, 4, true), 6);
    }

    #[test]
    fn varnada_mixed_parity_differences() {
        // Lagna Mesha (odd, 1), HL Vrshabha (even: 12 - 1 = 11):
        // |1 - 11| = 10, even, backward from Mina -> Mithuna (2).
        assert_eq!(varnada_lagna(0, 1, true), 2);
    }

    #[test]
    #[should_panic(expected = "is_odd_count must agree")]
    fn varnada_contradictory_parity_panics() {
        // Mesha (0-based 0) is odd-counted; declaring otherwise is caller bug.
        let _ = varnada_lagna(0, 4, false);
    }
}
