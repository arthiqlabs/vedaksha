// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Kala khanda — day-division windows (hora, choghadiya, yamaganda, abhijit).
//!
//! # Sources
//!
//! - Muhurta Chintamani; Kalaprakashika — the classical ghati tables this
//!   family segments (same family as [`crate::muhurta::rahu_kalam_slot`]).
//! - Hora lords run in the Chaldean order (Saturn, Jupiter, Mars, Sun,
//!   Venus, Mercury, Moon), the day's first hora ruled by the weekday lord,
//!   the sequence continuing unbroken through the night.
//! - Choghadiya names cycle Udveg, Char, Labh, Amrit, Kaal, Shubh, Rog. The
//!   day starts from the weekday lord's name and steps forward one per part;
//!   the night starts from the name of the weekday four days on and steps
//!   BACKWARD two per part (which is why the night order is not the day
//!   order shifted).
//! - Yamaganda is Jupiter's eighth: the day, counted from sunrise in the
//!   fixed Sun-led planetary order, whose portion Jupiter rules — Sunday
//!   5th, Monday 4th, Tuesday 3rd, Wednesday 2nd, Thursday 1st, Friday 7th,
//!   Saturday 6th.
//! - Abhijit is the 8th of the 15 daytime muhurtas; its midpoint is local
//!   apparent noon by construction (7.5/15 of the day). Whether Wednesday
//!   keeps it is a tradition variant, exposed as a parameter.
//!
//! # Contract
//!
//! All times are Julian Days (UT), locale-free. Day parts divide
//! sunrise-to-sunset, night parts sunset-to-next-sunrise. These are total
//! functions of their arithmetic inputs: where the Sun neither rises nor
//! sets (polar day/night) there is no daytime to divide, so a polar caller
//! must resolve that before calling (cf. [`crate::muhurta::kalam_windows`]).

use crate::muhurta::Weekday;

/// Graha index for hora lords: 0 Sun, 1 Moon, 2 Mars, 3 Mercury, 4 Jupiter,
/// 5 Venus, 6 Saturn (weekday-lord order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HoraGraha {
    Sun = 0,
    Moon = 1,
    Mars = 2,
    Mercury = 3,
    Jupiter = 4,
    Venus = 5,
    Saturn = 6,
}

/// Chaldean order, slowest to fastest: Saturn, Jupiter, Mars, Sun, Venus,
/// Mercury, Moon — as [`HoraGraha`] indices.
const CHALDEAN: [u8; 7] = [6, 4, 2, 0, 5, 3, 1];

/// Position of a weekday's lord inside [`CHALDEAN`].
fn chaldean_pos(vara: Weekday) -> usize {
    match vara {
        Weekday::Sunday => 3,
        Weekday::Monday => 6,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 5,
        Weekday::Thursday => 1,
        Weekday::Friday => 4,
        Weekday::Saturday => 0,
    }
}

/// 24 horas — 12 day parts from sunrise, 12 night parts from sunset — as
/// `(start_jd, end_jd, graha index)`, with the index in [`HoraGraha`]
/// numbering. The lord sequence never restarts: the night's first hora is
/// the 13th hora from sunrise.
#[must_use]
pub fn hora(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
) -> Vec<(f64, f64, u8)> {
    let day = jd_sunset - jd_sunrise;
    let night = jd_next_sunrise - jd_sunset;
    let (dh, nh) = (day / 12.0, night / 12.0);
    let start = chaldean_pos(vara);
    let mut out = Vec::with_capacity(24);
    for i in 0..12 {
        let f = i as f64;
        out.push((
            jd_sunrise + f * dh,
            jd_sunrise + (f + 1.0) * dh,
            CHALDEAN[(start + i) % 7],
        ));
    }
    for j in 0..12 {
        let f = j as f64;
        out.push((
            jd_sunset + f * nh,
            jd_sunset + (f + 1.0) * nh,
            CHALDEAN[(start + 12 + j) % 7],
        ));
    }
    out
}

/// Choghadiya kind index: 0 Udveg, 1 Char, 2 Labh, 3 Amrit, 4 Kaal,
/// 5 Shubh, 6 Rog — the canonical cycle order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChoghadiyaKind {
    Udveg = 0,
    Char = 1,
    Labh = 2,
    Amrit = 3,
    Kaal = 4,
    Shubh = 5,
    Rog = 6,
}

/// The weekday's FIRST daytime kind, as a cycle index.
fn day_start(vara: Weekday) -> usize {
    match vara {
        Weekday::Sunday => 0,    // Udveg
        Weekday::Monday => 3,    // Amrit
        Weekday::Tuesday => 6,   // Rog
        Weekday::Wednesday => 2, // Labh
        Weekday::Thursday => 5,  // Shubh
        Weekday::Friday => 1,    // Char
        Weekday::Saturday => 4,  // Kaal
    }
}

/// 16 choghadiyas — 8 day parts, 8 night parts — as
/// `(start_jd, end_jd, kind index)` in [`ChoghadiyaKind`] numbering.
#[must_use]
pub fn choghadiya(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
) -> Vec<(f64, f64, u8)> {
    let day = jd_sunset - jd_sunrise;
    let night = jd_next_sunrise - jd_sunset;
    let (dd, nd) = (day / 8.0, night / 8.0);
    // Night starts from the weekday four days on (counting the vara as one,
    // the fifth): Sunday -> Thursday's Shubh, etc. Explicit map, not
    // `vara as usize`: the discriminants are declaration order today, but
    // nothing pins that.
    let night_start = day_start(match vara {
        Weekday::Sunday => Weekday::Thursday,
        Weekday::Monday => Weekday::Friday,
        Weekday::Tuesday => Weekday::Saturday,
        Weekday::Wednesday => Weekday::Sunday,
        Weekday::Thursday => Weekday::Monday,
        Weekday::Friday => Weekday::Tuesday,
        Weekday::Saturday => Weekday::Wednesday,
    });
    let mut out = Vec::with_capacity(16);
    let ds = day_start(vara);
    for i in 0..8 {
        let f = i as f64;
        out.push((
            jd_sunrise + f * dd,
            jd_sunrise + (f + 1.0) * dd,
            ((ds + i) % 7) as u8,
        ));
    }
    for j in 0..8 {
        let f = j as f64;
        out.push((
            jd_sunset + f * nd,
            jd_sunset + (f + 1.0) * nd,
            ((night_start + 7 * 8 - 2 * j) % 7) as u8,
        ));
    }
    out
}

/// Yamaganda — Jupiter's eighth of the daytime — as `(start_jd, end_jd)`.
///
/// Slots (1-based, from sunrise): Sunday 5, Monday 4, Tuesday 3, Wednesday
/// 2, Thursday 1, Friday 7, Saturday 6.
#[must_use]
pub fn yamaganda(jd_sunrise: f64, jd_sunset: f64, vara: Weekday) -> (f64, f64) {
    let slot: f64 = match vara {
        Weekday::Sunday => 5.0,
        Weekday::Monday => 4.0,
        Weekday::Tuesday => 3.0,
        Weekday::Wednesday => 2.0,
        Weekday::Thursday => 1.0,
        Weekday::Friday => 7.0,
        Weekday::Saturday => 6.0,
    };
    let eighth = (jd_sunset - jd_sunrise) / 8.0;
    (
        jd_sunrise + (slot - 1.0) * eighth,
        jd_sunrise + slot * eighth,
    )
}

/// Abhijit muhurta — the 8th of the 15 equal daytime muhurtas — as
/// `(start_jd, end_jd)`, or `None` where it does not occur.
///
/// The only non-occurrence in the tradition is Wednesday under the schools
/// that omit it there; `omit_wednesday` selects that school. Passing `false`
/// returns the window on Wednesday like any other day — the omission is
/// never silent either way.
#[must_use]
pub fn abhijit_muhurta(
    jd_sunrise: f64,
    jd_sunset: f64,
    vara: Weekday,
    omit_wednesday: bool,
) -> Option<(f64, f64)> {
    if omit_wednesday && vara == Weekday::Wednesday {
        return None;
    }
    let muh = (jd_sunset - jd_sunrise) / 15.0;
    Some((jd_sunrise + 7.0 * muh, jd_sunrise + 8.0 * muh))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUNRISE: f64 = 0.0;
    const SUNSET: f64 = 0.5;
    const NEXT: f64 = 1.0;

    #[test]
    fn hora_lords_follow_chaldean_order_from_weekday_lord() {
        let h = hora(SUNRISE, SUNSET, NEXT, Weekday::Sunday);
        assert_eq!(h.len(), 24);
        // Day hora 1 = Sun (0); then Venus (5), Mercury (3), Moon (1) ...
        assert_eq!(h[0].2, 0);
        assert_eq!(h[1].2, 5);
        assert_eq!(h[2].2, 3);
        // Night hora 1 = 13th from sunrise: (3 + 12) % 7 = 1 -> Jupiter (4).
        assert_eq!(h[12].2, 4);
        // Windows tile sunrise->sunset->next sunrise without gaps.
        assert_eq!(h[0].0, SUNRISE);
        assert_eq!(h[11].1, SUNSET);
        assert_eq!(h[12].0, SUNSET);
        assert_eq!(h[23].1, NEXT);
        let dh = 0.5 / 12.0;
        assert!((h[0].1 - dh).abs() < 1e-12);
    }

    #[test]
    fn hora_rolls_into_next_weekday_lord() {
        // Every vara opens with its own lord (0 Sun .. 6 Saturn): the full
        // weekday-lord table, pinned as values, not via CHALDEAN.
        let expected = [
            (Weekday::Sunday, 0),
            (Weekday::Monday, 1),
            (Weekday::Tuesday, 2),
            (Weekday::Wednesday, 3),
            (Weekday::Thursday, 4),
            (Weekday::Friday, 5),
            (Weekday::Saturday, 6),
        ];
        for (vara, lord) in expected {
            assert_eq!(hora(SUNRISE, SUNSET, NEXT, vara)[0].2, lord);
        }
        // 24 horas advance the Chaldean dial by 24 % 7 = 3, which is exactly
        // the weekday-lord succession (Sun -> Moon -> Mars ...): Sunday's
        // dial position 3 + 3 lands on Monday's lord position 6.
        assert_eq!((3 + 24) % 7, 6);
    }

    #[test]
    fn choghadiya_sunday_sequences() {
        let c = choghadiya(SUNRISE, SUNSET, NEXT, Weekday::Sunday);
        assert_eq!(c.len(), 16);
        let day: Vec<u8> = c[..8].iter().map(|w| w.2).collect();
        assert_eq!(day, vec![0, 1, 2, 3, 4, 5, 6, 0]);
        let night: Vec<u8> = c[8..].iter().map(|w| w.2).collect();
        assert_eq!(night, vec![5, 3, 1, 6, 4, 2, 0, 5]);
    }

    #[test]
    fn choghadiya_wednesday_night_uses_backward_two_step() {
        // Wednesday night starts Udveg and steps back two per part:
        // Udveg, Shubh, Amrit, Char, Rog, Kaal, Labh, Udveg. A forward
        // step would give Labh second — this test tells them apart.
        let c = choghadiya(SUNRISE, SUNSET, NEXT, Weekday::Wednesday);
        let day: Vec<u8> = c[..8].iter().map(|w| w.2).collect();
        assert_eq!(day, vec![2, 3, 4, 5, 6, 0, 1, 2]);
        let night: Vec<u8> = c[8..].iter().map(|w| w.2).collect();
        assert_eq!(night, vec![0, 5, 3, 1, 6, 4, 2, 0]);
    }

    #[test]
    fn yamaganda_slots() {
        // Sunday 5th eighth of a 0.5-day day: (0.25, 0.3125).
        assert_eq!(yamaganda(SUNRISE, SUNSET, Weekday::Sunday), (0.25, 0.3125));
        // Thursday 1st: the day's opening eighth.
        assert_eq!(yamaganda(SUNRISE, SUNSET, Weekday::Thursday), (0.0, 0.0625));
        // Saturday 6th.
        assert_eq!(
            yamaganda(SUNRISE, SUNSET, Weekday::Saturday),
            (0.3125, 0.375)
        );
    }

    #[test]
    fn abhijit_is_the_8th_of_15() {
        // Day 0.5 -> muhurta 1/30; 8th = (7/30, 8/30), centred on noon.
        let (s, e) = abhijit_muhurta(SUNRISE, SUNSET, Weekday::Sunday, true).unwrap();
        assert!((s - 7.0 / 30.0).abs() < 1e-12);
        assert!((e - 8.0 / 30.0).abs() < 1e-12);
        assert!(((s + e) / 2.0 - 0.25).abs() < 1e-12);
    }

    #[test]
    fn abhijit_wednesday_omission_is_explicit() {
        assert_eq!(
            abhijit_muhurta(SUNRISE, SUNSET, Weekday::Wednesday, true),
            None
        );
        assert!(abhijit_muhurta(SUNRISE, SUNSET, Weekday::Wednesday, false).is_some());
    }
}
