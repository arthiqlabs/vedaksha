// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Upagrahas — the vara-portion bodies (Gulika, Mandi, Kala, Mrityu,
//! Ardhaprahara, Yamaghantaka) and the solar-offset bodies.
//!
//! # Sources
//!
//! BPHS, chapter 3 (the *graha-guna-svarupa* chapter), verses 66 to 70. The
//! Sanskrit was read in three printed recensions; two number the portions
//! 66 to 69 and the lagna of the portion 70 (the numbering used here), the
//! third runs the portion verses to 70 and puts the lagna at 71. Grades:
//! (a) read at the verse, editions agree; (b) read at the verse in one
//! recension only; (c) a choice the verse leaves open.
//!
//! * **The eight portions (3.66-67, grade a).** `ravivaradi-shanyantam
//!   gulikadi nirupyate; divasan ashtadha bhaktva varesad ganayet kramat;
//!   ashtamamsho nirishah syat`: divide the day (sunrise to sunset) into
//!   eight equal parts, give the first seven to the seven grahas in the
//!   weekday order (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn)
//!   starting from the lord of the vara, and the eighth has no lord. `Ratrim
//!   apy ashtadha krtva varesat panchamaditah`: the night (sunset to the next
//!   sunrise) is divided the same way, counting from the lord of the fifth
//!   weekday from the vara. So by day Sunday gives the Sun the first portion
//!   and Saturn the seventh; by night Sunday starts at Jupiter, the lord of
//!   the fifth day (Thursday), so Saturn's is the third.
//! * **Whose portion is which (3.68-69, grade a).** `Shanyamsho gulikah
//!   proktah`: Saturn's portion is Gulika; `gurvamsho yamaghantakah`:
//!   Jupiter's is Yamaghantaka; `bhaumamsho mrtyur adishtah`: Mars's is
//!   Mrityu; `ravyamsho kalasamjnakah`: the Sun's is Kala; `saumyamsho
//!   'rdhapraharakah`: Mercury's is Ardhaprahara. The Moon's and Venus's
//!   portions are named by no upagraha. The closing pada differs: one
//!   recension reads `svasvadeshodbhavah sphutah` (each arises for one's own
//!   place), the other two `sphuta-karma-pradeshakah` (indicators of the
//!   action each name suggests). Either way the portions depend on the
//!   place's own day and night, which is why the bounds are the caller's, for
//!   the place and date of birth.
//! * **The point (3.70, grade b for Gulika, nothing for the rest).** One
//!   recension reads `gulikarambha-samayat lagnam samsadhayet`: the lagna is
//!   computed from the instant the portion *begins*, and that lagna is the
//!   upagraha. The other reads `gulikeshta-vashat lagnam sphutam yat
//!   svasvadeshajam`, the lagna for Gulika's own ishta, which its commentary
//!   takes as the beginning and calls the end of the portion mathematically
//!   improper. The verse states this for Gulika only. It names no instant for
//!   Kala, Mrityu, Ardhaprahara or Yamaghantaka, and no verse consulted names
//!   the middle or the end of any portion; the functions for those four take the same
//!   [`UpagrahaPoint`] as Gulika and carry no default. [`UpagrahaPoint`] has no
//!   middle-of-the-portion variant because no source consulted reads one.
//!   The end of Saturn's portion for Mandi, which the earlier documentation of
//!   this module called the common reading, is a later convention: BPHS's
//!   yoga chapters use the name Mandi where this chapter says Gulika, and the
//!   commentaries gloss the one as the other. [`mandi`] therefore returns
//!   the same portion as [`gulika`] and takes the point as a parameter.
//! * **Dhuma through Upaketu** are fixed offsets from the (sidereal) Sun's
//!   longitude: Dhuma = Sun + 133°20'; Vyatipata = 360° − Dhuma;
//!   Parivesha = Vyatipata + 180°; Indrachapa = 360° − Parivesha;
//!   Upaketu = Indrachapa + 16°40'.
//!
//! The longitude of a vara-portion upagraha is the rashi rising at the
//! instant these functions return — leaving the ascendant to
//! [`vedaksha_astro::houses::compute_houses`], which the caller owns.
//!
//! # Contract
//!
//! Locale-free canonical values: Julian Day (UT) instants and sidereal
//! longitudes in degrees. The caller owns localization and the ascendant.

use crate::muhurta::Weekday;

/// Which end of the portion the upagraha instant is.
///
/// BPHS 3.70 states the start, for Gulika. It names no instant for the
/// other portions; see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpagrahaPoint {
    /// Opening instant of the portion: the reading BPHS 3.70 states for
    /// Gulika.
    PortionStart,
    /// Closing instant of the portion: a later convention, not a BPHS verse
    /// consulted.
    PortionEnd,
}

/// Index of a vara or portion lord in the weekday order: Sun 0, Moon 1,
/// Mars 2, Mercury 3, Jupiter 4, Venus 5, Saturn 6.
const fn weekday_index(vara: Weekday) -> u8 {
    match vara {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6,
    }
}

// Weekday-order index of the lord of each portion's upagraha (BPHS 3.68-69):
// the Sun's portion is Kala, Mars's Mrityu, Mercury's Ardhaprahara and
// Jupiter's Yamaghantaka. Saturn's, Gulika's, is `saturn_portion`.
const SUN: u8 = 0;
const MARS: u8 = 2;
const MERCURY: u8 = 3;
const JUPITER: u8 = 4;

/// The portion (1-based, from sunrise or sunset) ruled by the lord with
/// weekday-order index `lord` (BPHS 3.66-67).
///
/// The seven grahas take the first seven portions in the weekday order,
/// starting from the lord of the vara by day and from the lord of the fifth
/// weekday from the vara by night; the eighth portion has no lord, so the
/// result is always 1 to 7.
fn portion_of(lord: u8, vara: Weekday, is_night_birth: bool) -> f64 {
    let start = if is_night_birth {
        (weekday_index(vara) + 4) % 7
    } else {
        weekday_index(vara)
    };
    f64::from((lord + 7 - start) % 7 + 1)
}

/// Saturn's portion (1-based, from sunrise or sunset) for the vara.
///
/// Day births count from the weekday lord; night births from the 5th
/// weekday on (see the module doc). The day column reproduces
/// [`crate::muhurta::Weekday::gulika_kalam_slot`]. Kept as an explicit table
/// so [`portion_of`] has an independent check.
fn saturn_portion(vara: Weekday, is_night_birth: bool) -> f64 {
    if is_night_birth {
        f64::from(match vara {
            Weekday::Sunday => 3,
            Weekday::Monday => 2,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 7,
            Weekday::Thursday => 6,
            Weekday::Friday => 5,
            Weekday::Saturday => 4,
        })
    } else {
        f64::from(vara.gulika_kalam_slot())
    }
}

/// Instant (JD UT) of the `portion`-th eighth of the day or night — the
/// upagraha rising moment.
///
/// `is_night_birth` selects the sunset-to-next-sunrise division over the
/// sunrise-to-sunset one; `point` selects the portion's opening or closing
/// instant. The rashi rising at the returned instant (via
/// [`vedaksha_astro::houses::compute_houses`]) is the upagraha's longitude.
fn portion_instant(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    portion: f64,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    let (open, span) = if is_night_birth {
        (jd_sunset, jd_next_sunrise - jd_sunset)
    } else {
        (jd_sunrise, jd_sunset - jd_sunrise)
    };
    let eighth = span / 8.0;
    open + match point {
        UpagrahaPoint::PortionStart => (portion - 1.0) * eighth,
        UpagrahaPoint::PortionEnd => portion * eighth,
    }
}

/// Gulika's rising instant (JD UT): Saturn's portion of the day or night.
///
/// `jd_sunrise`, `jd_sunset` and `jd_next_sunrise` bound the day and the
/// night of the vara; `is_night_birth` selects which is divided; `point`
/// selects the portion's opening or closing instant. BPHS 3.70 reads
/// [`UpagrahaPoint::PortionStart`].
#[must_use]
pub fn gulika(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    portion_instant(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        saturn_portion(vara, is_night_birth),
        is_night_birth,
        point,
    )
}

/// Mandi's rising instant (JD UT) — same shape as [`gulika`], the same
/// Saturn portion (BPHS's yoga chapters use the name Mandi for the body this
/// chapter calls Gulika). A caller who follows the later convention of the
/// end of the portion for Mandi passes [`UpagrahaPoint::PortionEnd`].
#[must_use]
pub fn mandi(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    gulika(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        vara,
        is_night_birth,
        point,
    )
}

/// Kala's rising instant (JD UT): the Sun's portion of the day or night
/// (BPHS 3.68-69, `ravyamsho kalasamjnakah`). Arguments as [`gulika`].
///
/// The verse names the instant for Gulika only; `point` is the caller's
/// choice. See the module documentation.
#[must_use]
pub fn kala(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    portion_instant(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        portion_of(SUN, vara, is_night_birth),
        is_night_birth,
        point,
    )
}

/// Mrityu's rising instant (JD UT): Mars's portion of the day or night
/// (BPHS 3.68-69, `bhaumamsho mrtyur adishtah`). Arguments as [`gulika`].
///
/// The verse names the instant for Gulika only; `point` is the caller's
/// choice. See the module documentation.
#[must_use]
pub fn mrityu(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    portion_instant(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        portion_of(MARS, vara, is_night_birth),
        is_night_birth,
        point,
    )
}

/// Ardhaprahara's rising instant (JD UT): Mercury's portion of the day or
/// night (BPHS 3.68-69, `saumyamsho 'rdhapraharakah`). Arguments as
/// [`gulika`].
///
/// The verse names the instant for Gulika only; `point` is the caller's
/// choice. See the module documentation.
#[must_use]
pub fn ardhaprahara(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    portion_instant(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        portion_of(MERCURY, vara, is_night_birth),
        is_night_birth,
        point,
    )
}

/// Yamaghantaka's rising instant (JD UT): Jupiter's portion of the day or
/// night (BPHS 3.68-69, `gurvamsho yamaghantakah`). Arguments as [`gulika`].
///
/// The verse names the instant for Gulika only; `point` is the caller's
/// choice. See the module documentation.
#[must_use]
pub fn yamaghantaka(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    portion_instant(
        jd_sunrise,
        jd_sunset,
        jd_next_sunrise,
        portion_of(JUPITER, vara, is_night_birth),
        is_night_birth,
        point,
    )
}

/// Normalize to [0, 360).
fn norm(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}

/// Dhuma — Sun + 133°20' (4 signs 13°20'). Input is the sidereal Sun.
#[must_use]
pub fn dhuma(surya_longitude_deg: f64) -> f64 {
    norm(surya_longitude_deg + 133.0 + 20.0 / 60.0)
}

/// Vyatipata — 360° − Dhuma (the mirror of Dhuma about 0°).
///
/// Note the mirror is of the ABSOLUTE longitude: Vyatipata equals
/// Sun + 226°40' only for a Sun at Mesha 0°, not in general.
#[must_use]
pub fn vyatipata(surya_longitude_deg: f64) -> f64 {
    norm(360.0 - dhuma(surya_longitude_deg))
}

/// Parivesha — Vyatipata + 180°.
#[must_use]
pub fn parivesha(surya_longitude_deg: f64) -> f64 {
    norm(360.0 - dhuma(surya_longitude_deg) + 180.0)
}

/// Indrachapa — 360° − Parivesha (again a mirror of an absolute).
#[must_use]
pub fn indrachapa(surya_longitude_deg: f64) -> f64 {
    norm(360.0 - parivesha(surya_longitude_deg))
}

/// Upaketu — Indrachapa + 16°40'.
#[must_use]
pub fn upaketu(surya_longitude_deg: f64) -> f64 {
    norm(indrachapa(surya_longitude_deg) + 16.0 + 40.0 / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUNRISE: f64 = 0.0;
    const SUNSET: f64 = 0.5;
    const NEXT: f64 = 1.0;

    fn g(vara: Weekday, night: bool, point: UpagrahaPoint) -> f64 {
        gulika(SUNRISE, SUNSET, NEXT, vara, night, point)
    }

    #[test]
    fn gulika_day_portions_match_kalam_slots() {
        // Sunday 7th eighth of a 0.5-day day, start: 6/16 = 0.375.
        assert!((g(Weekday::Sunday, false, UpagrahaPoint::PortionStart) - 0.375).abs() < 1e-12);
        // Saturday 1st: sunrise itself.
        assert!((g(Weekday::Saturday, false, UpagrahaPoint::PortionStart) - 0.0).abs() < 1e-12);
        // End of Sunday's portion: 7/16 = 0.4375.
        assert!((g(Weekday::Sunday, false, UpagrahaPoint::PortionEnd) - 0.4375).abs() < 1e-12);
    }

    #[test]
    fn gulika_night_portions() {
        // Sunday night 3rd eighth from sunset: 0.5 + 2/16 = 0.625.
        assert!((g(Weekday::Sunday, true, UpagrahaPoint::PortionStart) - 0.625).abs() < 1e-12);
        // Tuesday night 1st: sunset itself.
        assert!((g(Weekday::Tuesday, true, UpagrahaPoint::PortionStart) - 0.5).abs() < 1e-12);
        // Wednesday night 7th, end: 0.5 + 7/16 = 0.9375.
        assert!((g(Weekday::Wednesday, true, UpagrahaPoint::PortionEnd) - 0.9375).abs() < 1e-12);
    }

    #[test]
    fn mandi_shares_gulikas_portion() {
        for night in [false, true] {
            for point in [UpagrahaPoint::PortionStart, UpagrahaPoint::PortionEnd] {
                assert_eq!(
                    mandi(SUNRISE, SUNSET, NEXT, Weekday::Friday, night, point),
                    gulika(SUNRISE, SUNSET, NEXT, Weekday::Friday, night, point)
                );
            }
        }
    }

    // ---- Kala, Mrityu, Ardhaprahara, Yamaghantaka (BPHS 3.66-69) ----------
    //
    // The expected portion of each is worked by hand from the verses: the
    // seven grahas take the first seven portions in the weekday order from the
    // vara lord by day, from the fifth weekday's lord by night; the Sun's is
    // Kala, Mars's Mrityu, Mercury's Ardhaprahara, Jupiter's Yamaghantaka.
    // With sunrise 0, sunset 0.5 and the next sunrise 1.0, portion `p` of the
    // day starts at (p - 1) / 16 and of the night at 0.5 + (p - 1) / 16.

    const DAY_START: f64 = SUNRISE;
    const NIGHT_START: f64 = SUNSET;
    const ALL_VARAS: [Weekday; 7] = [
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ];

    type PortionFn = fn(f64, f64, f64, Weekday, bool, UpagrahaPoint) -> f64;

    /// The portion (1 to 7) of one upagraha, read back from the instant its
    /// function returns at the start of the portion.
    fn start_portion(f: PortionFn, vara: Weekday, night: bool) -> f64 {
        let t = f(
            SUNRISE,
            SUNSET,
            NEXT,
            vara,
            night,
            UpagrahaPoint::PortionStart,
        );
        let open = if night { NIGHT_START } else { DAY_START };
        (t - open) * 16.0 + 1.0
    }

    /// Hand-derived (Kala, Mrityu, Ardhaprahara, Yamaghantaka) portions by
    /// day for Sunday to Saturday, counting the seven grahas in the weekday
    /// order from the vara lord.
    ///
    /// * Sunday: Sun 1, Moon 2, Mars 3, Mercury 4, Jupiter 5, Venus 6, Saturn 7.
    /// * Monday: Moon 1, Mars 2, Mercury 3, Jupiter 4, Venus 5, Saturn 6, Sun 7.
    /// * Tuesday: Mars 1, Mercury 2, Jupiter 3, Venus 4, Saturn 5, Sun 6, Moon 7.
    /// * Wednesday: Mercury 1, Jupiter 2, Venus 3, Saturn 4, Sun 5, Moon 6, Mars 7.
    /// * Thursday: Jupiter 1, Venus 2, Saturn 3, Sun 4, Moon 5, Mars 6, Mercury 7.
    /// * Friday: Venus 1, Saturn 2, Sun 3, Moon 4, Mars 5, Mercury 6, Jupiter 7.
    /// * Saturday: Saturn 1, Sun 2, Moon 3, Mars 4, Mercury 5, Jupiter 6, Venus 7.
    const DAY: [(f64, f64, f64, f64); 7] = [
        (1.0, 3.0, 4.0, 5.0), // Sunday
        (7.0, 2.0, 3.0, 4.0), // Monday
        (6.0, 1.0, 2.0, 3.0), // Tuesday
        (5.0, 7.0, 1.0, 2.0), // Wednesday
        (4.0, 6.0, 7.0, 1.0), // Thursday
        (3.0, 5.0, 6.0, 7.0), // Friday
        (2.0, 4.0, 5.0, 6.0), // Saturday
    ];

    /// By night each vara counts from the lord of the fifth weekday from it.
    ///
    /// * Sunday (from Thursday): Jupiter 1, Venus 2, Saturn 3, Sun 4, Moon 5,
    ///   Mars 6, Mercury 7.
    /// * Monday (from Friday): Venus 1, Saturn 2, Sun 3, Moon 4, Mars 5,
    ///   Mercury 6, Jupiter 7.
    /// * Tuesday (from Saturday): Saturn 1, Sun 2, Moon 3, Mars 4, Mercury 5,
    ///   Jupiter 6, Venus 7.
    /// * Wednesday (from Sunday): Sun 1, Moon 2, Mars 3, Mercury 4, Jupiter 5,
    ///   Venus 6, Saturn 7.
    /// * Thursday (from Monday): Moon 1, Mars 2, Mercury 3, Jupiter 4, Venus 5,
    ///   Saturn 6, Sun 7.
    /// * Friday (from Tuesday): Mars 1, Mercury 2, Jupiter 3, Venus 4, Saturn 5,
    ///   Sun 6, Moon 7.
    /// * Saturday (from Wednesday): Mercury 1, Jupiter 2, Venus 3, Saturn 4,
    ///   Sun 5, Moon 6, Mars 7.
    const NIGHT: [(f64, f64, f64, f64); 7] = [
        (4.0, 6.0, 7.0, 1.0), // Sunday
        (3.0, 5.0, 6.0, 7.0), // Monday
        (2.0, 4.0, 5.0, 6.0), // Tuesday
        (1.0, 3.0, 4.0, 5.0), // Wednesday
        (7.0, 2.0, 3.0, 4.0), // Thursday
        (6.0, 1.0, 2.0, 3.0), // Friday
        (5.0, 7.0, 1.0, 2.0), // Saturday
    ];

    #[test]
    fn kala_mrityu_ardhaprahara_yamaghantaka_follow_the_portion_table() {
        for (i, vara) in ALL_VARAS.into_iter().enumerate() {
            for (night, table) in [(false, &DAY), (true, &NIGHT)] {
                let (k, m, a, y) = table[i];
                assert_eq!(start_portion(kala, vara, night), k, "kala {vara:?} {night}");
                assert_eq!(
                    start_portion(mrityu, vara, night),
                    m,
                    "mrityu {vara:?} {night}"
                );
                assert_eq!(
                    start_portion(ardhaprahara, vara, night),
                    a,
                    "ardhaprahara {vara:?} {night}"
                );
                assert_eq!(
                    start_portion(yamaghantaka, vara, night),
                    y,
                    "yamaghantaka {vara:?} {night}"
                );
            }
        }
    }

    /// The general formula reproduces the explicit Saturn table the module
    /// already carried, for all fourteen (vara, period) pairs.
    #[test]
    fn portion_formula_reproduces_the_saturn_table() {
        for vara in ALL_VARAS {
            for night in [false, true] {
                assert_eq!(
                    portion_of(6, vara, night),
                    saturn_portion(vara, night),
                    "{vara:?} {night}"
                );
            }
        }
    }

    /// 3.66-67: the seven grahas take seven different portions and the eighth
    /// has no lord. The five upagrahas' portions are therefore distinct and
    /// none is the eighth.
    #[test]
    fn the_five_portions_are_distinct_and_never_the_eighth() {
        for vara in ALL_VARAS {
            for night in [false, true] {
                let g = start_portion(gulika, vara, night);
                let all = [
                    g,
                    start_portion(kala, vara, night),
                    start_portion(mrityu, vara, night),
                    start_portion(ardhaprahara, vara, night),
                    start_portion(yamaghantaka, vara, night),
                ];
                for (i, p) in all.iter().enumerate() {
                    assert!((1.0..=7.0).contains(p), "{vara:?} {night}: {p}");
                    assert!(
                        all[i + 1..].iter().all(|q| q != p),
                        "{vara:?} {night}: portions collide: {all:?}"
                    );
                }
            }
        }
    }

    /// Start and end of the same portion are one eighth apart, and the end is
    /// the next portion's start.
    #[test]
    fn point_selects_the_portion_edge() {
        let eighth_of_day = 0.5 / 8.0;
        for f in [kala, mrityu, ardhaprahara, yamaghantaka] {
            let s = f(
                SUNRISE,
                SUNSET,
                NEXT,
                Weekday::Tuesday,
                false,
                UpagrahaPoint::PortionStart,
            );
            let e = f(
                SUNRISE,
                SUNSET,
                NEXT,
                Weekday::Tuesday,
                false,
                UpagrahaPoint::PortionEnd,
            );
            assert!((e - s - eighth_of_day).abs() < 1e-12);
        }
        // Night eighths are of sunset-to-next-sunrise: with a night of 0.4 d
        // (sunset 0.5, next sunrise 0.9) one eighth is 0.05.
        let s = kala(
            0.0,
            0.5,
            0.9,
            Weekday::Wednesday,
            true,
            UpagrahaPoint::PortionStart,
        );
        // Wednesday night starts at the Sun: Kala is the first portion.
        assert!((s - 0.5).abs() < 1e-12);
        let e = kala(
            0.0,
            0.5,
            0.9,
            Weekday::Wednesday,
            true,
            UpagrahaPoint::PortionEnd,
        );
        assert!((e - 0.55).abs() < 1e-12);
    }

    #[test]
    fn solar_chain_at_aries_zero() {
        // Sun 0 -> Dhuma 133°20', Vyatipata 226°40', Parivesha 46°40',
        // Indrachapa 313°20', Upaketu 330°.
        assert!((dhuma(0.0) - 133.333_333_3).abs() < 1e-6);
        assert!((vyatipata(0.0) - 226.666_666_7).abs() < 1e-6);
        assert!((parivesha(0.0) - 46.666_666_7).abs() < 1e-6);
        assert!((indrachapa(0.0) - 313.333_333_3).abs() < 1e-6);
        assert!((upaketu(0.0) - 330.0).abs() < 1e-6);
    }

    #[test]
    fn solar_chain_relations_hold_for_any_sun() {
        // The chain is definitional: mirrors are of absolute longitudes,
        // so each step must equal its stated combination of the previous.
        let sun = 217.44;
        assert!((vyatipata(sun) - norm(360.0 - dhuma(sun))).abs() < 1e-9);
        assert!((parivesha(sun) - norm(vyatipata(sun) + 180.0)).abs() < 1e-9);
        assert!((indrachapa(sun) - norm(360.0 - parivesha(sun))).abs() < 1e-9);
        assert!((upaketu(sun) - norm(indrachapa(sun) + 16.0 + 40.0 / 60.0)).abs() < 1e-9);
        // And the chain genuinely moves: no two consecutive links coincide.
        assert!((dhuma(sun) - vyatipata(sun)).abs() > 1.0);
        assert!((vyatipata(sun) - parivesha(sun)).abs() > 1.0);
    }
}
