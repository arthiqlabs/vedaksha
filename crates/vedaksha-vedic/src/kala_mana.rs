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
//! - A new-moon-ending (**amanta**) lunation is named by the sankranti it
//!   contains: a lunation with NO sankranti is **adhika** (taking the
//!   following month's name), one containing TWO sankrantis yields a
//!   **kshaya** month (the second sankranti's month is the skipped one) at
//!   the following boundary. A result without adhika detection is wrong for
//!   any year carrying one.
//! - A full-moon-ending (**purnimanta**) lunation takes the name of the
//!   amanta month whose light half (shukla paksha) closes it: *Report of the
//!   Calendar Reform Committee* (Government of India, 1955), chapter V,
//!   section 5.7, "Siddhantic rules for the lunar calendar", which states the
//!   purnimanta month "begins from the moment of full-moon just a fort-night
//!   before the initial new-moon of an amanta month, and it also takes the
//!   name of that month". A shukla day therefore keeps its amanta name and a
//!   krishna day takes the name of the following amanta month.
//!
//! # Adhika and kshaya under the purnimanta reckoning
//!
//! The rule above is stated for the month's name, and the name of an amanta
//! month includes whether it is adhika or kshaya, so a purnimanta lunation
//! inherits the whole [`Masa`] (index, `adhika`, `kshaya`) of the amanta
//! month that closes it. Consequences: the purnimanta adhika month runs from
//! the full moon a fortnight before the adhika amanta month's new moon to the
//! full moon inside it, and the krishna half of the adhika amanta month falls
//! in the purnimanta month that follows, which is the ordinary (nija) month
//! of the same name. **This is the crate's reading, graded as such**: the
//! source states the naming rule only for the plain case and has no sentence
//! on adhika or kshaya under the full-moon reckoning. Its month table
//! (Table 20) labels the krishna half of an adhika amanta month "mala" in the
//! full-moon-ending column as well, which marks the religious period (new
//! moon to new moon) rather than a full-moon-to-full-moon lunation; a
//! lunation carries one label, so that table's half-month labelling is not
//! reproduced and is not used as an oracle. A caller that needs the
//! observance period of an adhika month should use [`MasaScheme::Amanta`].
//!
//! # Lunation bounds and instants
//!
//! [`lunar_month_span`] returns a lunation with its bounds. Boundaries are
//! found on a fixed grid of whole Julian days, so a given new or full moon
//! comes out as the same `f64` whichever instant the search starts from;
//! consecutive spans therefore tile exactly (`end_jd` of one is `start_jd`
//! of the next, bit for bit), and a boundary instant belongs to the lunation
//! it opens. The refined instant is the root finder's, run to the resolution of
//! a Julian Day (about 0.1 ms; until v9.5.0 it stopped within 1e-6 degree, about
//! 8 ms of time).

//! # Contract
//!
//! Locale-free canonical values: masa as index (0 = Chaitra, .., 11 =
//! Phalguna) with adhika/kshaya flags; years as numbers; ayana/ritu as
//! enums/indices. The caller owns localization. Positions come from the
//! caller's `moon_and_sun` callback (tropical or sidereal — the ayanamsha
//! cancels in the elongation); the Sun's sidereal longitude is derived
//! inside via `ayanamsha`.
//!
//! A calendar asks for many instants of one span, and each [`lunar_month_span`]
//! reads the pair about a hundred times. Take the callback from one
//! [`crate::walked::WalkedSky`] over the span and it makes 18 to 37 times fewer
//! provider calls (measured over a year of every-fourth-day spans and four
//! months of both schemes) and the same spans to 0.1 ms.

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

/// Cells of one day scanned before a boundary search gives up (a lunation is
/// at most about 29.9 days, so 45 is ample in either direction).
const MAX_SCAN_CELLS: u32 = 45;

/// A margin below the shortest real lunation (about 29.27 days). Searches for
/// the boundary after next start this far past a known boundary: nothing lies
/// in between, so the answer is unchanged and the scan is shorter.
const LUNATION_FLOOR_DAYS: f64 = 25.0;

/// As [`LUNATION_FLOOR_DAYS`], for the gap between a full moon and the next
/// new moon (or the reverse), about 14.3 to 15.4 days.
const HALF_LUNATION_FLOOR_DAYS: f64 = 10.0;

/// Most recent scheme boundary at or before `jd` (new or full moon).
///
/// The scan walks whole-Julian-day cells `[k, k + 1]` (elongation moves
/// ~12 degrees a day, so no boundary is skipped) and refines the first
/// crossing from the cell's opening instant. Because the cells are fixed in
/// absolute time and not relative to `jd`, the refined instant of a given
/// boundary is the same `f64` for every `jd` that finds it. The scan starts
/// one cell above `jd`'s own: a refined root may sit a few milliseconds off
/// its cell, and an instant just below a boundary must still see it.
fn boundary_before(
    jd: f64,
    scheme: MasaScheme,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let target = boundary_target(scheme);
    let angle_at = |t: f64| elongation_at(t, moon_and_sun);
    let mut hi = jd.floor() + 2.0;
    let mut e_hi = elongation_at(hi, moon_and_sun)?.0;
    for _ in 0..MAX_SCAN_CELLS {
        let lo = hi - 1.0;
        let e_lo = elongation_at(lo, moon_and_sun)?.0;
        if crossed(e_lo, e_hi, target) {
            let b = refine_crossing(target, lo, &angle_at)?;
            if b <= jd {
                return Some(b);
            }
        }
        hi = lo;
        e_hi = e_lo;
    }
    None
}

/// Next scheme boundary strictly after `jd`, on the same grid as
/// [`boundary_before`] (so `boundary_after(b)` for a returned boundary `b`
/// is the following one, and `boundary_before(b)` is `b` itself).
fn boundary_after(
    jd: f64,
    scheme: MasaScheme,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let target = boundary_target(scheme);
    let angle_at = |t: f64| elongation_at(t, moon_and_sun);
    let mut lo = jd.floor() - 1.0;
    let mut e_lo = elongation_at(lo, moon_and_sun)?.0;
    for _ in 0..MAX_SCAN_CELLS {
        let hi = lo + 1.0;
        let e_hi = elongation_at(hi, moon_and_sun)?.0;
        if crossed(e_lo, e_hi, target) {
            let b = refine_crossing(target, lo, &angle_at)?;
            if b > jd {
                return Some(b);
            }
        }
        lo = hi;
        e_lo = e_hi;
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

/// A [`Masa`] with the lunation it holds for.
///
/// **Convention: half-open, `[start_jd, end_jd)`.** `start_jd` belongs to the
/// lunation it opens and `end_jd` to the next one, so
/// `start_jd <= jd < end_jd` and [`lunar_month`] returns `masa` at every
/// instant of the span, including `start_jd` itself. The bounds are the
/// scheme's own boundaries (new moons for amanta, full moons for
/// purnimanta). They are canonical: the same lunation yields bit-identical
/// bounds from any `jd` inside it, so the `end_jd` of one span equals the
/// `start_jd` of the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MasaSpan {
    /// The month the lunation holds.
    pub masa: Masa,
    /// Julian day of the boundary opening the lunation (inclusive).
    pub start_jd: f64,
    /// Julian day of the boundary closing it (exclusive).
    pub end_jd: f64,
}

/// The [`Masa`] of the amanta lunation `[t0, t1)` (`jd` lies in it, or in
/// the purnimanta span that this lunation closes).
fn amanta_masa(
    jd: f64,
    t0: f64,
    t1: f64,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<Masa> {
    let ks = sankrantis_in(t0, t1, moon_and_sun, ayanamsha)?;
    match ks.len() {
        1 => Some(Masa {
            index: ks[0],
            adhika: false,
            kshaya: false,
        }),
        0 => {
            // Adhika: named by the following lunation's sankranti.
            let t2 = boundary_after(t1 + LUNATION_FLOOR_DAYS, MasaScheme::Amanta, moon_and_sun)?;
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

/// Lunar month containing `jd`, with the bounds of its lunation.
///
/// **Amanta** (boundaries at new moons): the month index is the contained
/// sankranti's rashi (0 = Chaitra for a Mesha sankranti, .., 11 = Phalguna
/// for Mina). No sankranti: adhika, with the NEXT lunation's name; two:
/// kshaya of the second sankranti's month. The engine pins its MECHANICS
/// (boundaries, counts, adhika/kshaya) against stub oracles, not the
/// doctrine against the sky.
///
/// **Purnimanta** (boundaries at full moons): the lunation from full moon to
/// full moon takes the whole [`Masa`] of the amanta month whose shukla
/// paksha closes it, so a shukla day carries its amanta name and a krishna
/// day the name of the amanta month that follows. See the module doc for the
/// source and for how adhika and kshaya carry over.
///
/// The bounds convention is on [`MasaSpan`]. Returns `None` where the
/// callback is unavailable.
#[must_use]
pub fn lunar_month_span(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<MasaSpan> {
    let start_jd = boundary_before(jd, scheme, moon_and_sun)?;
    let end_jd = boundary_after(jd.max(start_jd + LUNATION_FLOOR_DAYS), scheme, moon_and_sun)?;
    let masa = match scheme {
        MasaScheme::Amanta => amanta_masa(jd, start_jd, end_jd, ayanamsha, moon_and_sun)?,
        MasaScheme::Purnimanta => {
            // The new moon inside the full-moon span opens the amanta month
            // whose shukla half ends at `end_jd`.
            let new_moon = boundary_after(
                start_jd + HALF_LUNATION_FLOOR_DAYS,
                MasaScheme::Amanta,
                moon_and_sun,
            )?;
            if new_moon >= end_jd {
                return None; // not a Moon-Sun cycle: nothing to name
            }
            let next_new_moon = boundary_after(
                new_moon + LUNATION_FLOOR_DAYS,
                MasaScheme::Amanta,
                moon_and_sun,
            )?;
            amanta_masa(jd, new_moon, next_new_moon, ayanamsha, moon_and_sun)?
        }
    };
    Some(MasaSpan {
        masa,
        start_jd,
        end_jd,
    })
}

/// Lunar month containing `jd`: the [`Masa`] of [`lunar_month_span`], whose
/// docs give the naming rules for both schemes. A caller asking for many
/// instants should use [`lunar_month_span`] and reuse the answer across
/// `[start_jd, end_jd)`.
#[must_use]
pub fn lunar_month(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<Masa> {
    lunar_month_span(jd, scheme, ayanamsha, moon_and_sun).map(|s| s.masa)
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

/// Julian day at which the Vikrama/Shaka year numbered from calendar year
/// `year` turns over: the new moon opening Chaitra (index 0, not adhika) of
/// that CE year for [`MasaScheme::Amanta`], or the full moon a fortnight
/// before it for [`MasaScheme::Purnimanta`] (the full-moon span that Chaitra
/// shukla closes). [`vikrama_samvat`] and [`shaka_samvat`] are `year + 57`
/// (resp. `year - 78`) at and after this instant and one less before it.
///
/// A caller dating many instants of one year should ask once here and
/// compare, instead of calling [`vikrama_samvat`] for each.
///
/// The search does not scan lunations from 1 January. A Chaitra lunation
/// contains a Mesha sankranti, and the first one after 1 January is found
/// directly, so the lunation holding it is tried first and the walk (from 35
/// days before it) is only the fallback; the answer is the one a walk from
/// 1 January gives, because no lunation opening earlier can contain it.
#[must_use]
pub fn year_turnover(
    year: i32,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let jan1 = julian::calendar_to_jd(year, 1, 1.0);
    let sun_at = |t: f64| sidereal_sun_at(t, moon_and_sun, ayanamsha);
    // First Mesha sankranti (sidereal Sun at 0 degrees) after 1 January: a
    // straight-line estimate refined by the root finder.
    let (lon, rate) = sun_at(jan1)?;
    if rate <= 0.0 {
        return None;
    }
    let estimate = jan1 + (360.0 - lon).rem_euclid(360.0) / rate;
    let mesha = refine_crossing(0.0, estimate, &sun_at)?;
    // The Chaitra lunation contains this sankranti, so it opens at the new
    // moon at or before it. Take that first; the walk below is the general
    // search and covers a candidate that is not Chaitra or not after 1
    // January.
    let turn = |t1: f64| -> Option<Option<f64>> {
        let masa = lunar_month(t1 + 1.0, MasaScheme::Amanta, ayanamsha, moon_and_sun)?;
        if masa.index != 0 || masa.adhika {
            return Some(None);
        }
        // For purnimanta the year turns a fortnight earlier, at the full
        // moon opening Chaitra krishna: the full moon preceding this Chaitra
        // new moon.
        if scheme == MasaScheme::Purnimanta {
            return boundary_before(t1 - 1.0, MasaScheme::Purnimanta, moon_and_sun).map(Some);
        }
        Some(Some(t1))
    };
    let opening = boundary_before(mesha, MasaScheme::Amanta, moon_and_sun)?;
    if opening > jan1 + 1.0 / 86400.0 {
        if let Some(found) = turn(opening)? {
            return Some(found);
        }
    }
    // Walk new moons from 35 days before the sankranti (a lunation is under
    // 30 days long): the first non-adhika Chaitra opens the year.
    let mut t = (mesha - 35.0).max(jan1);
    for _ in 0..14 {
        let t1 = boundary_after(t, MasaScheme::Amanta, moon_and_sun)?;
        if let Some(found) = turn(t1)? {
            return Some(found);
        }
        t = t1;
    }
    None
}

/// Vikrama samvat — CE year + 57 once Chaitra has opened, + 56 before.
/// `scheme` selects the amanta/purnimanta year turnover
/// ([`year_turnover`]).
#[must_use]
pub fn vikrama_samvat(
    jd: f64,
    scheme: MasaScheme,
    ayanamsha: Ayanamsha,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<i32> {
    let (year, _, _) = julian::jd_to_calendar(jd);
    let turnover = year_turnover(year, scheme, ayanamsha, moon_and_sun)?;
    Some(if jd >= turnover { year + 57 } else { year + 56 })
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
        // Stub: amanta lunations [t0 + 30k, t0 + 30k + 30), full moons at
        // t0 + 15 + 30k. Sidereal sun = 220.8 + (t - t0), so the amanta
        // lunation [t0, t0+30) holds edge 240 (index 8), [t0+30, t0+60)
        // holds 270 (index 9), [t0+60, t0+90) holds 300 (index 10).
        //
        // Purnimanta span [t0+15, t0+45) holds the new moon t0+30, so the
        // amanta month opening there (index 9) closes it with its shukla
        // half: the whole span is 9, for a krishna day (t0+20, amanta 8)
        // and a shukla day (t0+40, amanta 9) alike. A shared-boundary
        // implementation, or the sankranti-in-span rule (which gives 8 for
        // both, edge 240 lying inside [t0+15, t0+45)), fails this.
        let t0 = 2_460_000.0;
        let f = stub(t0, 12.0, 245.0);
        let ay = Ayanamsha::IndianOfficial;
        let am = |jd: f64| lunar_month(jd, MasaScheme::Amanta, ay, &f).unwrap();
        let pu = |jd: f64| lunar_month(jd, MasaScheme::Purnimanta, ay, &f).unwrap();
        assert_eq!(am(t0 + 40.0).index, 9);
        assert_eq!(pu(t0 + 40.0).index, 9);
        assert_eq!(am(t0 + 20.0).index, 8);
        assert_eq!(pu(t0 + 20.0).index, 9);
        // Next span [t0+45, t0+75) holds new moon t0+60: index 10, both for
        // its krishna day t0+50 (amanta 9) and shukla day t0+70 (amanta 10).
        assert_eq!(am(t0 + 50.0).index, 9);
        assert_eq!(pu(t0 + 50.0).index, 10);
        assert_eq!(pu(t0 + 70.0).index, 10);
        for jd in [t0 + 20.0, t0 + 40.0, t0 + 50.0, t0 + 70.0] {
            let m = pu(jd);
            assert!(!m.adhika && !m.kshaya);
        }
        // The span itself: [t0+15, t0+45) to within the refinement stop.
        let sp = lunar_month_span(t0 + 20.0, MasaScheme::Purnimanta, ay, &f).unwrap();
        assert!((sp.start_jd - (t0 + 15.0)).abs() < 1e-3, "{sp:?}");
        assert!((sp.end_jd - (t0 + 45.0)).abs() < 1e-3, "{sp:?}");
        // Just past a full moon (elong 240 degrees): the forward scan meets
        // the 0 degree wrap (the intervening new moon) BEFORE the next full
        // moon. The wrap is not a 180-crossing; mistaking it returns a PAST
        // boundary and wrong bounds. Guards the `crossed` wrap rule.
        let q = lunar_month_span(t0 + 20.0, MasaScheme::Purnimanta, ay, &f).unwrap();
        assert!(q.start_jd <= t0 + 20.0 && t0 + 20.0 < q.end_jd);
        assert!(q.end_jd - q.start_jd > 29.9 && q.end_jd - q.start_jd < 30.1);
    }

    #[test]
    fn adhika_and_kshaya_carry_over_to_purnimanta() {
        let ay = Ayanamsha::IndianOfficial;
        let t0 = 2_460_000.0;
        // Fast moon (13 deg/day, lunation 27.69 d), sidereal sun from
        // 240.8: amanta [t0, t0+27.69) is empty -> adhika index 9; the next
        // amanta [t0+27.69, t0+55.38) spans sun (268.5, 296.2] -> edge 270,
        // index 9, nija. Full moons at t0 + 13.85 + 27.69k.
        let f = stub(t0, 13.0, 265.0);
        let pu = |jd: f64| lunar_month(jd, MasaScheme::Purnimanta, ay, &f).unwrap();
        // Purnimanta span [t0-13.85, t0+13.85) holds new moon t0, which
        // opens the ADHIKA amanta month: the span is adhika 9.
        let m = pu(t0 + 10.0);
        assert!(m.adhika && !m.kshaya, "{m:?}");
        assert_eq!(m.index, 9);
        // The span [t0+13.85, t0+41.54) holds new moon t0+27.69, opening
        // the nija amanta 9: ordinary 9, including the krishna half of the
        // adhika amanta month (t0+20), which lies in it.
        let m = pu(t0 + 20.0);
        assert!(!m.adhika && !m.kshaya, "{m:?}");
        assert_eq!(m.index, 9);

        // Slow moon (11 deg/day, lunation 32.73 d), sidereal sun from
        // 207.8: amanta [t0, t0+32.73) holds edges 210 and 240 -> kshaya,
        // index 8; the next holds 270 -> index 9. Full moons t0 + 16.36 +
        // 32.73k, so span [t0-16.36, t0+16.36) opens at new moon t0.
        let f = stub(t0, 11.0, 232.0);
        let pu = |jd: f64| lunar_month(jd, MasaScheme::Purnimanta, ay, &f).unwrap();
        let m = pu(t0 + 5.0);
        assert!(m.kshaya && !m.adhika, "{m:?}");
        assert_eq!(m.index, 8);
        let m = pu(t0 + 20.0);
        assert!(!m.kshaya && !m.adhika, "{m:?}");
        assert_eq!(m.index, 9);
    }

    #[test]
    fn span_of_stub_lunation_is_half_open_and_tiles() {
        let ay = Ayanamsha::IndianOfficial;
        let t0 = 2_460_000.0;
        let f = stub(t0, 12.0, 245.0);
        for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
            let s = lunar_month_span(t0 + 40.0, scheme, ay, &f).unwrap();
            assert!(s.start_jd <= t0 + 40.0 && t0 + 40.0 < s.end_jd);
            // The opening instant belongs to the lunation it opens, the
            // closing instant to the next.
            let at_start = lunar_month_span(s.start_jd, scheme, ay, &f).unwrap();
            assert_eq!(at_start, s);
            let at_end = lunar_month_span(s.end_jd, scheme, ay, &f).unwrap();
            assert_eq!(at_end.start_jd, s.end_jd);
            assert!((at_end.end_jd - at_end.start_jd - 30.0).abs() < 1e-3);
            // Every jd of the span gives the identical record, bit for bit.
            for k in 0..30 {
                let jd = s.start_jd + 0.999 * f64::from(k) + 0.0123;
                if jd < s.end_jd {
                    assert_eq!(lunar_month_span(jd, scheme, ay, &f).unwrap(), s);
                }
            }
        }
    }

    // ---- A low-precision sky for property tests -------------------------

    /// Degrees to radians.
    fn rad(d: f64) -> f64 {
        d.to_radians()
    }

    /// Tropical Sun longitude of date: Meeus, *Astronomical Algorithms*
    /// ch. 25 (low accuracy), geometric longitude plus the equation of
    /// centre. Good to a fraction of a degree, which dates a lunation to
    /// within an hour: ample for properties that do not sit on a boundary.
    fn sun_lon(jd: f64) -> f64 {
        let t = (jd - 2_451_545.0) / 36525.0;
        let l0 = 280.46646 + 36000.76983 * t;
        let m = rad(357.52911 + 35999.05029 * t);
        let c = (1.914602 - 0.004817 * t) * m.sin()
            + 0.019993 * (2.0 * m).sin()
            + 0.000289 * (3.0 * m).sin();
        (l0 + c).rem_euclid(360.0)
    }

    /// Tropical Moon longitude of date: the thirteen largest periodic terms
    /// of Meeus ch. 47 table 47.A (longitude), on the mean elements of the
    /// same chapter. Good to about 0.3 degrees.
    fn moon_lon(jd: f64) -> f64 {
        let t = (jd - 2_451_545.0) / 36525.0;
        let lp = 218.3164477 + 481_267.88123421 * t;
        let d = rad(297.8501921 + 445_267.1114034 * t);
        let m = rad(357.5291092 + 35999.0502909 * t);
        let mp = rad(134.9633964 + 477_198.8675055 * t);
        let f = rad(93.2720950 + 483_202.0175233 * t);
        let sum = 6.288774 * mp.sin()
            + 1.274027 * (2.0 * d - mp).sin()
            + 0.658314 * (2.0 * d).sin()
            + 0.213618 * (2.0 * mp).sin()
            - 0.185116 * m.sin()
            - 0.114332 * (2.0 * f).sin()
            + 0.058793 * (2.0 * d - 2.0 * mp).sin()
            + 0.057066 * (2.0 * d - m - mp).sin()
            + 0.053322 * (2.0 * d + mp).sin()
            + 0.045758 * (2.0 * d - m).sin()
            - 0.040923 * (m - mp).sin()
            - 0.034720 * d.sin()
            - 0.030383 * (m + mp).sin();
        (lp + sum).rem_euclid(360.0)
    }

    /// Longitude and daily motion (central difference over +-0.01 d).
    fn with_rate(lon: fn(f64) -> f64, jd: f64) -> (f64, f64) {
        let h = 0.01;
        let mut dl = lon(jd + h) - lon(jd - h);
        if dl > 180.0 {
            dl -= 360.0;
        } else if dl < -180.0 {
            dl += 360.0;
        }
        (lon(jd), dl / (2.0 * h))
    }

    fn sky(jd: f64) -> Option<MoonAndSun> {
        Some((with_rate(moon_lon, jd), with_rate(sun_lon, jd)))
    }

    /// Tropical elongation of `sky` at `jd`, degrees in [0, 360).
    fn sky_elongation(jd: f64) -> f64 {
        (moon_lon(jd) - sun_lon(jd)).rem_euclid(360.0)
    }

    const AYA: Ayanamsha = Ayanamsha::IndianOfficial;

    #[test]
    fn spans_tile_and_hold_one_masa_on_the_sky() {
        for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
            let mut jd = julian::calendar_to_jd(2022, 1, 1.0) + 0.31;
            let end = julian::calendar_to_jd(2026, 1, 1.0);
            let mut n = 0;
            while jd < end {
                let s = lunar_month_span(jd, scheme, AYA, &sky).unwrap();
                assert!(s.start_jd <= jd && jd < s.end_jd, "{scheme:?} {jd} {s:?}");
                let len = s.end_jd - s.start_jd;
                assert!((29.2..=29.9).contains(&len), "{scheme:?} {jd} len {len}");
                // The same Masa at the opening instant, a hair before the
                // close, and the middle.
                for probe in [s.start_jd, 0.5 * (s.start_jd + s.end_jd), s.end_jd - 1.0e-7] {
                    let m = lunar_month(probe, scheme, AYA, &sky).unwrap();
                    assert_eq!(m, s.masa, "{scheme:?} probe {probe} span {s:?}");
                }
                // Tiling: the instant the span closes opens the next one,
                // bit for bit, and the next one is a different lunation.
                let nx = lunar_month_span(s.end_jd, scheme, AYA, &sky).unwrap();
                assert_eq!(nx.start_jd.to_bits(), s.end_jd.to_bits());
                assert!(nx.end_jd > nx.start_jd);
                // And from any jd of the span the bounds are the same bits.
                let again = lunar_month_span(s.start_jd + 0.37 * len, scheme, AYA, &sky).unwrap();
                assert_eq!(again, s);
                jd += 2.9;
                n += 1;
            }
            assert!(n > 400);
        }
    }

    #[test]
    fn purnimanta_name_is_the_amanta_month_that_closes_the_span() {
        // Defining rule, checked day by day over 2000-2030 at twelve
        // different times of day: a shukla day (elongation below 180) keeps
        // its amanta Masa; a krishna day takes the Masa of the amanta month
        // that follows, found independently as the amanta Masa 16 days on
        // (a krishna day is at most ~15 days before the next new moon and
        // the next lunation is at least 29.2 long, so +16 d lies in it).
        let mut shukla = 0;
        let mut krishna = 0;
        let mut adhika_seen = 0;
        let mut p_adhika_seen = 0;
        let mut p_kshaya_seen = 0;
        let mut jd = julian::calendar_to_jd(2000, 1, 1.0) + 0.07;
        let end = julian::calendar_to_jd(2030, 1, 1.0);
        while jd < end {
            let a = lunar_month(jd, MasaScheme::Amanta, AYA, &sky).unwrap();
            let p = lunar_month(jd, MasaScheme::Purnimanta, AYA, &sky).unwrap();
            if sky_elongation(jd) < 180.0 {
                shukla += 1;
                assert_eq!(p, a, "shukla day {jd}");
            } else {
                krishna += 1;
                let next = lunar_month(jd + 16.0, MasaScheme::Amanta, AYA, &sky).unwrap();
                assert_eq!(p, next, "krishna day {jd}");
                // Away from adhika and kshaya, the name is the next index.
                if !a.adhika && !p.kshaya && !p.adhika {
                    assert_eq!(p.index, (a.index + 1) % 12, "krishna day {jd}");
                }
            }
            adhika_seen += i32::from(a.adhika);
            p_adhika_seen += i32::from(p.adhika);
            p_kshaya_seen += i32::from(p.kshaya);
            jd += 1.0 + 0.0791;
        }
        // The sample could have failed to exercise the rule; it did not.
        assert!(shukla > 4000 && krishna > 4000, "{shukla} {krishna}");
        assert!(adhika_seen > 250, "adhika amanta days: {adhika_seen}");
        assert!(
            p_adhika_seen > 300,
            "adhika purnimanta days: {p_adhika_seen}"
        );
        let _ = p_kshaya_seen; // kshaya is rare in a 30-year window; not asserted
    }

    #[test]
    fn adhika_shravana_2023_under_both_schemes() {
        // Sky facts, not table values: the Sun entered Karka (sidereal
        // Cancer) about 16 July 2023 and Simha about 17 August; the new
        // moons fell on 17 July and 16 August, so the lunation between them
        // holds no sankranti: adhika, named for the sankranti that follows
        // (Simha, index 4 = Shravana). Its full moon is 1 August; the
        // preceding full moon is 3 July.
        let at = |m: u32, d: f64| julian::calendar_to_jd(2023, m, d);
        // Shukla half (17 Jul new moon .. 1 Aug full moon).
        let a = lunar_month(at(7, 25.0), MasaScheme::Amanta, AYA, &sky).unwrap();
        assert!(a.adhika && a.index == 4, "{a:?}");
        let p = lunar_month(at(7, 25.0), MasaScheme::Purnimanta, AYA, &sky).unwrap();
        assert!(p.adhika && p.index == 4, "{p:?}");
        // Krishna half before it (3 Jul .. 17 Jul): amanta nija Ashadha (3),
        // purnimanta the closing amanta month, adhika Shravana.
        let a = lunar_month(at(7, 10.0), MasaScheme::Amanta, AYA, &sky).unwrap();
        assert!(!a.adhika && a.index == 3, "{a:?}");
        let p = lunar_month(at(7, 10.0), MasaScheme::Purnimanta, AYA, &sky).unwrap();
        assert!(p.adhika && p.index == 4, "{p:?}");
        // Krishna half of the adhika amanta month (1 Aug .. 16 Aug): the
        // purnimanta span is the nija month, Shravana, not adhika.
        let a = lunar_month(at(8, 8.0), MasaScheme::Amanta, AYA, &sky).unwrap();
        assert!(a.adhika && a.index == 4, "{a:?}");
        let p = lunar_month(at(8, 8.0), MasaScheme::Purnimanta, AYA, &sky).unwrap();
        assert!(!p.adhika && p.index == 4, "{p:?}");
        // The nija month's own shukla half agrees.
        let p = lunar_month(at(8, 25.0), MasaScheme::Purnimanta, AYA, &sky).unwrap();
        assert!(!p.adhika && p.index == 4, "{p:?}");
    }

    #[test]
    fn purnimanta_is_never_a_month_early_where_the_old_rule_was() {
        // The 1990 dates the defect was reported on (shukla days). The
        // sankranti-in-span rule gave amanta - 1 here; the closing rule
        // gives the amanta name.
        for (m, d) in [(9, 2.0), (9, 20.0)] {
            let jd = julian::calendar_to_jd(1990, m, d);
            let a = lunar_month(jd, MasaScheme::Amanta, AYA, &sky).unwrap();
            let p = lunar_month(jd, MasaScheme::Purnimanta, AYA, &sky).unwrap();
            assert!(sky_elongation(jd) < 180.0, "1990-{m}-{d} is shukla");
            assert_eq!(p, a, "1990-{m}-{d}");
        }
    }

    // ---- Reference: the 9.4.1 code, kept to prove nothing else moved ----

    mod legacy {
        use super::*;

        pub(super) fn boundary_before(
            jd: f64,
            scheme: MasaScheme,
            f: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
        ) -> Option<f64> {
            let target = boundary_target(scheme);
            let mut t1 = jd;
            let mut e1 = elongation_at(t1, f)?.0;
            for _ in 0..40 {
                let t0 = t1 - 1.0;
                let e0 = elongation_at(t0, f)?.0;
                if crossed(e0, e1, target) {
                    let angle_at = |t: f64| elongation_at(t, f);
                    return refine_crossing(target, t0, &angle_at);
                }
                t1 = t0;
                e1 = e0;
            }
            None
        }

        pub(super) fn boundary_after(
            jd: f64,
            scheme: MasaScheme,
            f: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
        ) -> Option<f64> {
            let target = boundary_target(scheme);
            let mut t0 = jd;
            let mut e0 = elongation_at(t0, f)?.0;
            for _ in 0..40 {
                let t1 = t0 + 1.0;
                let e1 = elongation_at(t1, f)?.0;
                if crossed(e0, e1, target) {
                    let angle_at = |t: f64| elongation_at(t, f);
                    return refine_crossing(target, t0, &angle_at);
                }
                t0 = t1;
                e0 = e1;
            }
            None
        }

        /// 9.4.1 `lunar_month`, amanta or purnimanta by sankranti-in-span.
        pub(super) fn lunar_month(
            jd: f64,
            scheme: MasaScheme,
            ay: Ayanamsha,
            f: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
        ) -> Option<Masa> {
            let t0 = boundary_before(jd, scheme, f)?;
            let t1 = boundary_after(jd, scheme, f)?;
            let ks = sankrantis_in(t0, t1, f, ay)?;
            match ks.len() {
                1 => Some(Masa {
                    index: ks[0],
                    adhika: false,
                    kshaya: false,
                }),
                0 => {
                    let t2 = boundary_after(t1 + 1.0 / 86400.0, scheme, f)?;
                    let next = sankrantis_in(t1, t2, f, ay)?;
                    let index = next.first().copied().unwrap_or_else(|| {
                        rashi_of(sidereal_sun_at(jd, f, ay).map_or(0.0, |(lon, _)| lon))
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

        /// 9.4.1 `chaitra_new_moon`: scan from 1 January.
        pub(super) fn chaitra_new_moon(
            jd: f64,
            scheme: MasaScheme,
            ay: Ayanamsha,
            f: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
        ) -> Option<f64> {
            let (year, _, _) = julian::jd_to_calendar(jd);
            let mut t = julian::calendar_to_jd(year, 1, 1.0);
            for _ in 0..14 {
                let t1 = boundary_after(t + 1.0 / 86400.0, MasaScheme::Amanta, f)?;
                let masa = lunar_month(t1 + 1.0, MasaScheme::Amanta, ay, f)?;
                if masa.index == 0 && !masa.adhika {
                    if scheme == MasaScheme::Purnimanta {
                        return boundary_before(t1 - 1.0, MasaScheme::Purnimanta, f);
                    }
                    return Some(t1);
                }
                t = t1;
            }
            None
        }
    }

    /// Counts calls of [`sky`].
    struct Counted(std::sync::atomic::AtomicU64);

    impl Counted {
        fn new() -> Self {
            Self(std::sync::atomic::AtomicU64::new(0))
        }
        fn eval(&self, jd: f64) -> Option<MoonAndSun> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            sky(jd)
        }
        fn take(&self) -> u64 {
            self.0.swap(0, std::sync::atomic::Ordering::Relaxed)
        }
    }

    #[test]
    fn amanta_answers_match_9_4_1() {
        // Every instant of a 12-year sweep (period chosen incommensurate
        // with the lunation and the day) names the same Masa as the 9.4.1
        // code. Adhika years are in the window (see the purnimanta test).
        let mut jd = julian::calendar_to_jd(2005, 1, 1.0) + 0.013;
        let end = julian::calendar_to_jd(2017, 1, 1.0);
        let mut n = 0;
        while jd < end {
            let old = legacy::lunar_month(jd, MasaScheme::Amanta, AYA, &sky).unwrap();
            let new = lunar_month(jd, MasaScheme::Amanta, AYA, &sky).unwrap();
            assert_eq!(old, new, "jd {jd}");
            jd += 0.9137;
            n += 1;
        }
        assert!(n > 4700);
    }

    /// Turnover instants and year numbers of the new search equal the
    /// 9.4.1 scan's, for several ayanamshas (including the tropical zero
    /// and the trepidation system, which move the Mesha sankranti by weeks),
    /// both schemes, and instants all through the year and in the weeks
    /// around each turnover.
    #[test]
    fn year_numbers_match_9_4_1() {
        use std::collections::HashMap;
        let ayanamshas = [
            Ayanamsha::IndianOfficial,
            Ayanamsha::Raman,
            Ayanamsha::Tropical,
            Ayanamsha::SuryaSiddhanta,
            Ayanamsha::GalacticCenter0Sag,
        ];
        let mut compared = 0;
        let mut flips = 0;
        for ay in ayanamshas {
            for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
                let mut old_turn: HashMap<i32, f64> = HashMap::new();
                for year in 1990..2040 {
                    let jan1 = julian::calendar_to_jd(year, 1, 1.0);
                    let old = legacy::chaitra_new_moon(jan1 + 100.0, scheme, ay, &sky).unwrap();
                    let new = year_turnover(year, scheme, ay, &sky).unwrap();
                    assert!(
                        (old - new).abs() < 1.0e-6,
                        "{ay:?} {scheme:?} {year}: old {old} new {new}"
                    );
                    old_turn.insert(year, old);
                }
                let old_vikrama = |jd: f64| {
                    let (y, _, _) = julian::jd_to_calendar(jd);
                    if jd >= old_turn[&y] { y + 57 } else { y + 56 }
                };
                for year in 1991..2039 {
                    let t = old_turn[&year];
                    let mut probes: Vec<f64> = Vec::new();
                    for dt in [
                        -40.0,
                        -21.0,
                        -14.0,
                        -10.0,
                        -7.0,
                        -3.0,
                        -1.0,
                        -1.0 / 24.0,
                        -1.0 / 1440.0,
                        1.0 / 1440.0,
                        1.0 / 24.0,
                        1.0,
                        3.0,
                        7.0,
                        10.0,
                        14.0,
                        21.0,
                        40.0,
                    ] {
                        probes.push(t + dt);
                    }
                    let jan1 = julian::calendar_to_jd(year, 1, 1.0);
                    let mut x = jan1 + 0.5;
                    while x < jan1 + 366.0 {
                        probes.push(x);
                        x += 13.7;
                    }
                    for jd in probes {
                        let (y, _, _) = julian::jd_to_calendar(jd);
                        if !old_turn.contains_key(&y) {
                            continue;
                        }
                        let want = old_vikrama(jd);
                        let got = vikrama_samvat(jd, scheme, ay, &sky).unwrap();
                        assert_eq!(got, want, "{ay:?} {scheme:?} jd {jd}");
                        assert_eq!(
                            shaka_samvat(jd, scheme, ay, &sky).unwrap(),
                            want - 135,
                            "{ay:?} {scheme:?} jd {jd}"
                        );
                        compared += 1;
                        flips += i32::from(jd >= t && jd < t + 1.0);
                    }
                }
            }
        }
        // Both sides of every turnover were probed, and many instants.
        assert!(compared > 10_000, "{compared}");
        assert!(flips > 400, "{flips}");
    }

    #[test]
    fn turnover_search_costs_a_fraction_of_the_scan() {
        let c = Counted::new();
        let f = |t: f64| c.eval(t);
        let mut old_total = 0;
        let mut new_total = 0;
        let mut n = 0;
        for year in 2000..2030 {
            let jd = julian::calendar_to_jd(year, 8, 1.0);
            let _ = legacy::chaitra_new_moon(jd, MasaScheme::Amanta, AYA, &f).unwrap();
            old_total += c.take();
            let _ = vikrama_samvat(jd, MasaScheme::Amanta, AYA, &f).unwrap();
            new_total += c.take();
            n += 1;
        }
        eprintln!(
            "vikrama_samvat callbacks per call, amanta, 30 years: 9.4.1 scan {} -> {}",
            old_total / n,
            new_total / n
        );
        assert!(new_total * 2 < old_total, "{new_total} vs {old_total}");
    }

    #[test]
    fn span_cache_costs_one_search_per_lunation() {
        let c = Counted::new();
        let f = |t: f64| c.eval(t);
        let start = julian::calendar_to_jd(2024, 3, 1.0) + 0.2;
        for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
            // 9.4.1 way: one lunar_month per day for 62 days (legacy code,
            // legacy boundary search).
            for d in 0..62 {
                let _ = legacy::lunar_month(start + f64::from(d), scheme, AYA, &f).unwrap();
            }
            let per_day = c.take();
            // Now: one lunar_month per day, new boundary search.
            for d in 0..62 {
                let _ = lunar_month(start + f64::from(d), scheme, AYA, &f).unwrap();
            }
            let per_day_new = c.take();
            // Span cache: ask again only when the day leaves the span.
            let mut span: Option<MasaSpan> = None;
            let mut asks = 0;
            let mut cached = Vec::new();
            for d in 0..62 {
                let jd = start + f64::from(d);
                let s = match span {
                    Some(s) if s.start_jd <= jd && jd < s.end_jd => s,
                    _ => {
                        asks += 1;
                        lunar_month_span(jd, scheme, AYA, &f).unwrap()
                    }
                };
                span = Some(s);
                cached.push(s.masa);
            }
            let cache_cost = c.take();
            // The cached answers are the per-day answers.
            for (d, m) in cached.iter().enumerate() {
                let jd = start + f64::from(u32::try_from(d).unwrap());
                assert_eq!(*m, lunar_month(jd, scheme, AYA, &f).unwrap());
            }
            let _ = c.take();
            eprintln!(
                "62 days, {scheme:?}: per-day 9.4.1 {per_day} callbacks, per-day now {per_day_new}, \
                 span cache {cache_cost} callbacks ({asks} asks)"
            );
            assert!(asks <= 4, "62 days touch at most 3 lunations (+1 slack)");
        }
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
