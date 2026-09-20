// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Upagrahas — Saturn's portion (Gulika/Mandi) and the solar-offset bodies.
//!
//! # Sources
//!
//! BPHS, the chapter on upagrahas.
//!
//! - **Gulika/Mandi**: divide the day (sunrise to sunset) or the night
//!   (sunset to next sunrise) into eight equal parts, counting planetary
//!   portions from the weekday lord in the fixed Sun-led order. The
//!   Saturn-ruled portion is Gulika's: by day Sunday 7th, Monday 6th,
//!   Tuesday 5th, Wednesday 4th, Thursday 3rd, Friday 2nd, Saturday 1st
//!   (the same eighth [`crate::muhurta::Weekday::gulika_kalam_slot`]
//!   selects). By night the count starts from the 5th weekday, giving
//!   Sunday 3rd, Monday 2nd, Tuesday 1st, Wednesday 7th, Thursday 6th,
//!   Friday 5th, Saturday 4th. The longitude is the rashi rising at that
//!   portion — the instant this module returns, leaving the ascendant to
//!   [`vedaksha_astro::houses::compute_houses`], which the caller owns.
//! - **Whether the start or the end of Saturn's portion** is taken differs
//!   between traditions (Gulika is commonly read at the start, Mandi toward
//!   the end): [`UpagrahaPoint`] exposes it. The two functions are otherwise
//!   the same shape because in the Parashari allocation they ARE the same
//!   portion.
//! - **Dhuma through Upaketu** are fixed offsets from the (sidereal) Sun's
//!   longitude: Dhuma = Sun + 133°20'; Vyatipata = 360° − Dhuma;
//!   Parivesha = Vyatipata + 180°; Indrachapa = 360° − Parivesha;
//!   Upaketu = Indrachapa + 16°40'.
//!
//! # Contract
//!
//! Locale-free canonical values: Julian Day (UT) instants and sidereal
//! longitudes in degrees. The caller owns localization and the ascendant.

use crate::muhurta::Weekday;

/// Which end of Saturn's portion the upagraha instant is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpagrahaPoint {
    /// Opening instant of Saturn's portion (the common Gulika reading).
    PortionStart,
    /// Closing instant of Saturn's portion (the common Mandi reading).
    PortionEnd,
}

/// Saturn's portion (1-based, from sunrise or sunset) for the vara.
///
/// Day births count from the weekday lord; night births from the 5th
/// weekday on (see the module doc). The day column reproduces
/// [`crate::muhurta::Weekday::gulika_kalam_slot`].
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

/// Instant (JD UT) of Saturn's portion — the upagraha rising moment.
///
/// `is_night_birth` selects the sunset-to-next-sunrise division over the
/// sunrise-to-sunset one; `point` selects the portion's opening or closing
/// instant. The rashi rising at the returned instant (via
/// [`vedaksha_astro::houses::compute_houses`]) is the upagraha's longitude.
fn portion_instant(
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Weekday,
    is_night_birth: bool,
    point: UpagrahaPoint,
) -> f64 {
    let (open, span) = if is_night_birth {
        (jd_sunset, jd_next_sunrise - jd_sunset)
    } else {
        (jd_sunrise, jd_sunset - jd_sunrise)
    };
    let eighth = span / 8.0;
    let portion = saturn_portion(vara, is_night_birth);
    open + match point {
        UpagrahaPoint::PortionStart => (portion - 1.0) * eighth,
        UpagrahaPoint::PortionEnd => portion * eighth,
    }
}

/// Gulika's rising instant (JD UT). See [`portion_instant`] for the
/// arguments; Gulika is conventionally read at [`UpagrahaPoint::PortionStart`].
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
        vara,
        is_night_birth,
        point,
    )
}

/// Mandi's rising instant (JD UT) — same shape as [`gulika`], the same
/// Saturn portion; Mandi is conventionally read at
/// [`UpagrahaPoint::PortionEnd`].
#[must_use]
pub fn mandi(
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
        vara,
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
