// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Kala mana — lunisolar calendar: masa (with adhika/kshaya), samvats,
//! the 60-year samvatsara, ayana and ritu.
//!
//! # Sources
//!
//! - Surya Siddhanta (mean longitudes at the Kali epoch; the sankranti as
//!   the solar-month boundary); Reingold & Dershowitz, *Calendrical
//!   Calculations*, for the algorithmic form of the lunation/sankranti
//!   reckoning.
//! - A lunation is named by the sankranti it contains: a lunation with NO
//!   sankranti is **adhika** (taking the following month's name), one
//!   containing TWO sankrantis yields a **kshaya** month (the second
//!   sankranti's month is the skipped one) at the following boundary. A
//!   result without adhika detection is wrong for any year carrying one.
//!
//! # Contract
//!
//! Locale-free canonical values: masa as index (0 = Chaitra, .., 11 =
//! Phalguna) with adhika/kshaya flags; years as numbers; ayana/ritu as
//! enums/indices. The caller owns localization. Positions come from the
//! caller's `moon_and_sun` callback (tropical or sidereal — the ayanamsha
//! cancels in the elongation); the Sun's sidereal longitude is derived
//! inside via `ayanamsha`.

use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::julian;

use crate::muhurta::{MoonAndSun, refine_crossing};

/// Which new/full-moon boundary opens the month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasaScheme {
    /// Months open at new moon (amavasyanta).
    Amanta,
    /// Months open at full moon (purnimanta).
    Purnimanta,
}

/// A lunar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Masa {
    /// Month index: 0 = Chaitra, 1 = Vaisakha, .., 11 = Phalguna.
    pub index: u8,
    /// The lunation held no sankranti; the month repeats the following name.
    pub adhika: bool,
    /// The lunation held two sankrantis; this (second) month is skipped.
    pub kshaya: bool,
}

/// Elongation target for the scheme's boundary: 0° (new moon) or 180°.
fn boundary_target(scheme: MasaScheme) -> f64 {
    match scheme {
        MasaScheme::Amanta => 0.0,
        MasaScheme::Purnimanta => 180.0,
    }
}

/// Elongation (Moon − Sun, degrees in [0, 360)) and its rate at `jd`.
fn elongation_at(
    jd: f64,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<(f64, f64)> {
    let ((ml, ms), (sl, ss)) = moon_and_sun(jd)?;
    Some(((ml - sl).rem_euclid(360.0), ms - ss))
}

/// Most recent scheme boundary at or before `jd` (new or full moon).
fn boundary_before(
    jd: f64,
    scheme: MasaScheme,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let target = boundary_target(scheme);
    // Step back in 1-day strides (elongation moves ~12°/day, so no boundary
    // is skipped) until the elongation wraps past the target.
    let mut t1 = jd;
    let mut e1 = elongation_at(t1, moon_and_sun)?.0;
    for _ in 0..40 {
        let t0 = t1 - 1.0;
        let e0 = elongation_at(t0, moon_and_sun)?.0;
        if crossed(e0, e1, target) {
            let angle_at = |t: f64| elongation_at(t, moon_and_sun);
            return refine_crossing(target, t0, &angle_at);
        }
        t1 = t0;
        e1 = e0;
    }
    None
}

/// Next scheme boundary strictly after `jd`.
fn boundary_after(
    jd: f64,
    scheme: MasaScheme,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let target = boundary_target(scheme);
    let mut t0 = jd;
    let mut e0 = elongation_at(t0, moon_and_sun)?.0;
    for _ in 0..40 {
        let t1 = t0 + 1.0;
        let e1 = elongation_at(t1, moon_and_sun)?.0;
        if crossed(e0, e1, target) {
            // Refine from the bracket's opening step: refining from `jd`
            // itself would converge back to a boundary AT `jd` (within the
            // 1e-6° stop tolerance) instead of the next one.
            let angle_at = |t: f64| elongation_at(t, moon_and_sun);
            return refine_crossing(target, t0, &angle_at);
        }
        t0 = t1;
        e0 = e1;
    }
    None
}

/// Whether the monotone angle moved from `e0` to `e1` across `target`
/// (all in [0, 360)) within one scan step.
///
/// The 180° detector must ignore the 0° branch-cut wrap: stepping from
/// 348° to 12° crosses 0°, not 180°. A genuine step moves ~12° (1-day
/// stride), a wrap ~348°, so a step wider than 180° is a wrap — a crossing
/// only for target 0° (additionally confined to the wrap neighbourhood).
fn crossed(e0: f64, e1: f64, target: f64) -> bool {
    if target == 0.0 {
        // Wrap crossing: e1 < e0 across the 0° branch cut (elongation rate
        // is positive, so a decrease means a wrap, not retrogression).
        e1 < e0 && (e0 > 300.0 || e1 < 60.0)
    } else {
        (e1 - e0).abs() < 180.0 && ((e0 < target && e1 >= target) || (e0 >= target && e1 < target))
    }
}

/// Sidereal solar longitude and rate at `jd` (tropical callback minus the
/// true ayanamsha).
fn sidereal_sun_at(
    jd: f64,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<(f64, f64)> {
    let (_, (sl, ss)) = moon_and_sun(jd)?;
    let aya = true_ayanamsha_value(ayanamsha, jd);
    Some(((sl - aya).rem_euclid(360.0), ss))
}

/// Rashi index (0–11) of a longitude.
fn rashi_of(lon_deg: f64) -> u8 {
    floor_to_u8(lon_deg.rem_euclid(360.0) / 30.0) % 12
}

/// Non-negative float known to fit its target; the casts below are exact
/// for the JD-scale magnitudes this module handles (see each call site).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn floor_to_u8(x: f64) -> u8 {
    x.floor() as u8
}

/// As [`floor_to_u8`], for year counts.
#[allow(clippy::cast_possible_truncation)]
fn floor_to_i32(x: f64) -> i32 {
    x.floor() as i32
}

/// Sankranti rashis (0–11) entered inside `[t0, t1]`, in order.
fn sankrantis_in(
    t0: f64,
    t1: f64,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
    ayanamsha: Ayanamsha,
) -> Option<Vec<u8>> {
    // 2-day scan: the Sun moves ~2°/step, so no 30° boundary is skipped.
    let mut out = Vec::new();
    let mut t = t0;
    let mut prev = sidereal_sun_at(t, moon_and_sun, ayanamsha)?.0;
    while t < t1 {
        let t_next = (t + 2.0).min(t1);
        let cur = sidereal_sun_at(t_next, moon_and_sun, ayanamsha)?.0;
        // Rashis entered between prev and cur.
        let mut k = rashi_of(prev) + 1;
        loop {
            let edge = f64::from(k % 12) * 30.0;
            let crossed_edge = if cur >= prev {
                prev < edge && edge <= cur
            } else {
                // Wrapped past 0°: edges in (prev, 360) and [0, cur].
                edge > prev || edge <= cur
            };
            if !crossed_edge {
                break;
            }
            out.push(k % 12);
            k += 1;
            if out.len() > 3 {
                return Some(out);
            }
        }
        prev = cur;
        t = t_next;
    }
    Some(out)
}

/// Lunar month containing `jd`.
///
/// Boundaries are new moons (amanta) or full moons (purnimanta). The month
/// index is the contained sankranti's rashi (0 = Chaitra for a Mesha
/// sankranti, .., 11 = Phalguna for Mina) — the sankranti-naming rule is the
/// consumer-agreed contract (feature request 2026-09-19); the engine pins
/// its MECHANICS (boundaries, counts, adhika/kshaya) against stub oracles,
/// not the doctrine against the sky. No sankranti → adhika with the
/// NEXT lunation's name; two → kshaya of the second sankranti's month.
/// Returns `None` where the callback is unavailable.
#[must_use]
pub fn lunar_month(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<Masa> {
    let t0 = boundary_before(jd, scheme, moon_and_sun)?;
    let t1 = boundary_after(jd, scheme, moon_and_sun)?;
    let ks = sankrantis_in(t0, t1, moon_and_sun, ayanamsha)?;
    match ks.len() {
        1 => Some(Masa {
            index: ks[0],
            adhika: false,
            kshaya: false,
        }),
        0 => {
            // Adhika: named by the following lunation's sankranti.
            let t2 = boundary_after(t1 + 1.0 / 86400.0, scheme, moon_and_sun)?;
            let next = sankrantis_in(t1, t2, moon_and_sun, ayanamsha)?;
            let index = next.first().copied().unwrap_or_else(|| {
                // A second consecutive empty lunation is not expected;
                // fall back to the Sun's own rashi.
                rashi_of(sidereal_sun_at(jd, moon_and_sun, ayanamsha).map_or(0.0, |(lon, _)| lon))
            });
            Some(Masa {
                index,
                adhika: true,
                kshaya: false,
            })
        }
        _ => Some(Masa {
            index: ks[1],
            adhika: false,
            kshaya: true,
        }),
    }
}

/// Kali-epoch constant: 3102 BCE February 18 (astronomical year −3101),
/// the Surya Siddhanta/Aryabhatiya mean-longitude epoch.
fn kali_epoch_jd() -> f64 {
    julian::calendar_to_jd(-3101, 2, 18.0)
}

/// Kali samvat — elapsed sidereal solar years since the Kali epoch + 1, on
/// the mean year (365.25636 d). Lunisolar boundary traditions (Mesha
/// sankranti vs Chaitra shukla) differ by weeks, not years; the count is
/// the same for any date not within those weeks — documented, not resolved.
#[must_use]
pub fn kali_samvat(jd: f64) -> i32 {
    const MEAN_SIDEREAL_YEAR: f64 = 365.25636;
    floor_to_i32((jd - kali_epoch_jd()) / MEAN_SIDEREAL_YEAR) + 1
}

/// Mean-Jupiter samvatsara index, 0 = Prabhava .. 59: signs traversed by
/// Jupiter's MEAN longitude since the Kali epoch (whose construction sets
/// all mean longitudes ~0) modulo 60, on the mean sidereal period
/// (4332.59 d). True-longitude ingress traditions differ within a year;
/// the cycle index agrees except near a Mesha ingress.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn samvatsara(jd: f64) -> u8 {
    const JUPITER_PERIOD_D: f64 = 4332.59;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn signs_since_epoch(jd: f64) -> i64 {
        ((jd - kali_epoch_jd()) / JUPITER_PERIOD_D * 12.0).floor() as i64
    }
    signs_since_epoch(jd).rem_euclid(60) as u8
}

/// New moon opening Chaitra (index 0, non-adhika) of the CE year containing
/// `jd`, for the Vikrama/Shaka year turnover.
fn chaitra_new_moon(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let (year, _, _) = julian::jd_to_calendar(jd);
    // Scan new moons from Jan 1; the first non-adhika Chaitra opens the year.
    let mut t = julian::calendar_to_jd(year, 1, 1.0);
    for _ in 0..14 {
        let t1 = boundary_after(t + 1.0 / 86400.0, MasaScheme::Amanta, moon_and_sun)?;
        let masa = lunar_month(t1 + 1.0, MasaScheme::Amanta, ayanamsha, moon_and_sun)?;
        if masa.index == 0 && !masa.adhika {
            // For purnimanta the year turns a fortnight earlier, at the
            // full moon opening Chaitra krishna: the full moon preceding
            // this Chaitra new moon.
            if scheme == MasaScheme::Purnimanta {
                return boundary_before(t1 - 1.0, MasaScheme::Purnimanta, moon_and_sun);
            }
            return Some(t1);
        }
        t = t1;
    }
    None
}

/// Vikrama samvat — CE year + 57 once Chaitra has opened, + 56 before.
/// `scheme` selects the amanta/purnimanta year turnover.
#[must_use]
pub fn vikrama_samvat(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<i32> {
    let (year, _, _) = julian::jd_to_calendar(jd);
    let chaitra = chaitra_new_moon(jd, scheme, ayanamsha, moon_and_sun)?;
    Some(if jd >= chaitra { year + 57 } else { year + 56 })
}

/// Shaka samvat — 135 behind Vikrama (epoch 78 CE), same turnover.
#[must_use]
pub fn shaka_samvat(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<i32> {
    vikrama_samvat(jd, scheme, ayanamsha, moon_and_sun).map(|v| v - 135)
}

/// Solstice-to-solstice half of the sidereal solar year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ayana {
    /// Makara → Mithuna end: sidereal Sun in [270°, 90°).
    Uttarayana,
    /// Karka → Dhanus end: sidereal Sun in [90°, 270°).
    Dakshinayana,
}

/// Ayana at `jd` from the sidereal Sun.
#[must_use]
pub fn ayana(
    jd: f64,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<Ayana> {
    let (lon, _) = sidereal_sun_at(jd, moon_and_sun, ayanamsha)?;
    if lon >= 270.0 || lon < 90.0 {
        Some(Ayana::Uttarayana)
    } else {
        Some(Ayana::Dakshinayana)
    }
}

/// Ritu index at `jd`, 0 = Shishira (Makara+Kumbha), 1 = Vasanta
/// (Mina+Mesha), 2 = Grishma, 3 = Varsha, 4 = Sharad, 5 = Hemanta —
/// six 60° spans of sidereal solar longitude from Makara 0°.
#[must_use]
pub fn ritu(
    jd: f64,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<u8> {
    let (lon, _) = sidereal_sun_at(jd, moon_and_sun, ayanamsha)?;
    Some(floor_to_u8((lon - 270.0).rem_euclid(360.0) / 60.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat-rate stub cosmos: elongation 12°/day from new moon at t0,
    /// tropical Sun sun0 + 1°/day. NOTE the engine works in sidereal Sun
    /// (tropical minus ~24.2° true ayanamsha here), so hand-placed spans
    /// must be computed sidereally: sun0 = 245 puts the sidereal span at
    /// (220.8, 250.8], holding exactly edge 240 (k 8).
    fn stub(t0: f64, elong_rate: f64, sun0: f64) -> impl Fn(f64) -> Option<MoonAndSun> + Sync {
        move |t: f64| {
            let elong = ((t - t0) * elong_rate).rem_euclid(360.0);
            let sun = sun0 + (t - t0);
            Some(((elong + sun, 13.0), (sun, 1.0)))
        }
    }

    #[test]
    fn nominal_lunation_named_by_its_sankranti() {
        let t0 = 2_460_000.0;
        let f = stub(t0, 12.0, 245.0);
        // Lunation [t0, t0+30]; sidereal sun (220.8, 250.8] holds edge 240.
        let m = lunar_month(t0 + 10.0, MasaScheme::Amanta, Ayanamsha::IndianOfficial, &f).unwrap();
        assert_eq!(
            m,
            Masa {
                index: 8,
                adhika: false,
                kshaya: false
            }
        );
    }

    #[test]
    fn empty_lunation_is_adhika_with_next_name() {
        // Fast moon (13°/day -> 27.7 d), sidereal sun from 240.8°: span
        // (240.8, 268.5] holds no multiple of 30. Next lunation holds 270.
        let t0 = 2_460_000.0;
        let f = stub(t0, 13.0, 265.0);
        let m = lunar_month(t0 + 10.0, MasaScheme::Amanta, Ayanamsha::IndianOfficial, &f).unwrap();
        assert!(m.adhika && !m.kshaya);
        assert_eq!(m.index, 9);
    }

    #[test]
    fn double_sankranti_lunation_is_kshaya() {
        // Slow moon (11°/day -> 32.7 d), sidereal sun from 207.8°:
        // (207.8, 240.5] holds 210 and 240 -> kshaya of the second (k 8).
        let t0 = 2_460_000.0;
        let f = stub(t0, 11.0, 232.0);
        let m = lunar_month(t0 + 10.0, MasaScheme::Amanta, Ayanamsha::IndianOfficial, &f).unwrap();
        assert!(m.kshaya && !m.adhika);
        assert_eq!(m.index, 8);
    }

    #[test]
    fn purnimanta_bounds_differ_from_amanta() {
        // jd = t0+40 sits in amanta [t0+30, t0+60] — sidereal sun
        // (250.8, 280.8] holds edge 270 -> index 9 — but in purnimanta
        // [t0+15, t0+45], holding 240 -> index 8. The schemes genuinely
        // disagree here; a shared-boundary implementation fails this.
        let t0 = 2_460_000.0;
        let f = stub(t0, 12.0, 245.0);
        let a = lunar_month(t0 + 40.0, MasaScheme::Amanta, Ayanamsha::IndianOfficial, &f).unwrap();
        let p = lunar_month(
            t0 + 40.0,
            MasaScheme::Purnimanta,
            Ayanamsha::IndianOfficial,
            &f,
        )
        .unwrap();
        assert_eq!(a.index, 9);
        assert_eq!(p.index, 8);
        assert!(!p.adhika && !p.kshaya);
        // Just past a full moon (elong 240°): the forward scan meets the
        // 0° wrap (the intervening new moon) BEFORE the next full moon.
        // The wrap is not a 180-crossing; mistaking it returns a PAST
        // boundary and a false adhika. Guards the `crossed` wrap rule.
        let q = lunar_month(
            t0 + 20.0,
            MasaScheme::Purnimanta,
            Ayanamsha::IndianOfficial,
            &f,
        )
        .unwrap();
        assert_eq!(q.index, 8);
        assert!(!q.adhika && !q.kshaya);
    }

    #[test]
    fn kali_counts_mean_years_from_epoch() {
        let e = kali_epoch_jd();
        assert_eq!(kali_samvat(e), 1);
        // A day into year 2 (exact-boundary fp would be a coin flip).
        assert_eq!(kali_samvat(e + 366.0), 2);
        // 2024 CE runs inside Kali year 5126 (begun April 2024): check the
        // year, not the boundary week (documented convention).
        assert_eq!(kali_samvat(julian::calendar_to_jd(2024, 6, 1.0)), 5126);
    }

    #[test]
    fn samvatsara_cycles_sixty_from_epoch() {
        assert_eq!(samvatsara(kali_epoch_jd()), 0);
        // A day into the second mean Jupiter-sign: index 1.
        assert_eq!(samvatsara(kali_epoch_jd() + 4332.59 / 12.0 + 1.0), 1);
        // Past sixty signs: wrapped to 0.
        assert_eq!(samvatsara(kali_epoch_jd() + 4332.59 * 5.0 + 10.0), 0);
    }

    #[test]
    fn ayana_and_ritu_from_flat_sun() {
        // Tropical 100° - ~23.85° ayanamsha ~= 76° sidereal (Mithuna):
        // Uttarayana, Grishma (index 2).
        let f = |t: f64| Some((((50.0 + t) % 360.0, 13.0), (100.0, 1.0)));
        let jd = 2_451_545.0;
        assert_eq!(
            ayana(jd, Ayanamsha::IndianOfficial, &f),
            Some(Ayana::Uttarayana)
        );
        assert_eq!(ritu(jd, Ayanamsha::IndianOfficial, &f), Some(2));
    }

    #[test]
    fn vikrama_and_shaka_turn_at_chaitra() {
        // Real-ish cosmos not needed: stub with one new moon a year is
        // overkill; check the relation on a fixed stub cosmos where
        // Chaitra opens mid-March 2024.
        let t0 = julian::calendar_to_jd(2024, 1, 1.0);
        // Elongation 0 at Apr 8 2024 (new moon), sun racing to force
        // index-0 months: use the nominal stub shifted so a boundary
        // lands Apr 8 and the sankranti inside is k 0.
        let nm = julian::calendar_to_jd(2024, 4, 8.0);
        let f = move |t: f64| {
            let elong = ((t - nm) * 12.18).rem_euclid(360.0);
            // Sun sidereal ~358° at nm, moving 1°/day: crosses 0 (k 0)
            // two days later, inside the lunation.
            let sun_sid = 358.0 + (t - nm);
            // Tropical = sidereal + fixed ayanamsha 23.85.
            Some(((elong + sun_sid, 13.2), (sun_sid + 23.85, 1.0)))
        };
        let _ = t0;
        let v_before = vikrama_samvat(
            julian::calendar_to_jd(2024, 3, 1.0),
            MasaScheme::Amanta,
            Ayanamsha::IndianOfficial,
            &f,
        );
        let v_after = vikrama_samvat(
            julian::calendar_to_jd(2024, 5, 1.0),
            MasaScheme::Amanta,
            Ayanamsha::IndianOfficial,
            &f,
        );
        assert_eq!(v_before, Some(2080));
        assert_eq!(v_after, Some(2081));
        assert_eq!(
            shaka_samvat(
                julian::calendar_to_jd(2024, 5, 1.0),
                MasaScheme::Amanta,
                Ayanamsha::IndianOfficial,
                &f
            ),
            Some(1946)
        );
    }

    #[test]
    fn unavailable_ephemeris_is_none() {
        let no: fn(f64) -> Option<MoonAndSun> = |_| None;
        assert_eq!(
            lunar_month(
                2_460_000.0,
                MasaScheme::Amanta,
                Ayanamsha::IndianOfficial,
                &no
            ),
            None
        );
    }
}
