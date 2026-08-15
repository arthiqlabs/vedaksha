// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Muhurta search — finding auspicious times in Vedic electional astrology.
//!
//! Evaluates Moon nakshatra, tithi (lunar day), and weekday across a date range.
//! Source: BPHS (Brihat Parashara Hora Shastra); Muhurta Chintamani.

use crate::nakshatra::Nakshatra;

/// A tithi (lunar day) — one of 30 tithis in a lunar month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tithi {
    /// Tithi number (1-30). 1-15 = Shukla Paksha, 16-30 = Krishna Paksha.
    pub number: u8,
    /// Name of the tithi.
    pub name: &'static str,
}

/// Lunar fortnight — Shukla (waxing) or Krishna (waning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paksha {
    Shukla,
    Krishna,
}

impl Tithi {
    /// Which paksha (fortnight) this tithi belongs to.
    #[must_use]
    pub fn paksha(&self) -> Paksha {
        if self.number <= 15 {
            Paksha::Shukla
        } else {
            Paksha::Krishna
        }
    }

    /// Tithi lord (ruling planet). Source: Muhurtha Chintamani.
    #[must_use]
    pub fn lord(&self) -> &'static str {
        const LORDS: [&str; 15] = [
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Sun", "Moon",
            "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
        ];
        LORDS[((self.number - 1) % 15) as usize]
    }

    /// Remaining degrees before the next tithi begins.
    #[must_use]
    pub fn remaining_degrees(moon_lon: f64, sun_lon: f64) -> f64 {
        let diff = vedaksha_math::angle::normalize_degrees(moon_lon - sun_lon);
        12.0 - (diff % 12.0)
    }
}

/// Day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    /// Vara lord — ruling planet of the weekday.
    /// Source: standard Vedic weekday rulership.
    #[must_use]
    pub fn lord(&self) -> &'static str {
        match self {
            Self::Sunday => "Sun",
            Self::Monday => "Moon",
            Self::Tuesday => "Mars",
            Self::Wednesday => "Mercury",
            Self::Thursday => "Jupiter",
            Self::Friday => "Venus",
            Self::Saturday => "Saturn",
        }
    }

    /// Rahu Kalam slot — which 1/8th of daytime is inauspicious.
    /// Returns 1-8 where 1 is the first 1/8th after sunrise.
    /// Source: Kalaprakashika.
    #[must_use]
    pub fn rahu_kalam_slot(&self) -> u8 {
        match self {
            Self::Sunday => 8,
            Self::Monday => 2,
            Self::Tuesday => 7,
            Self::Wednesday => 5,
            Self::Thursday => 6,
            Self::Friday => 4,
            Self::Saturday => 3,
        }
    }

    /// Gulika Kalam slot — which 1/8th of daytime is ruled by Saturn's son.
    /// Source: Muhurtha Chintamani.
    #[must_use]
    pub fn gulika_kalam_slot(&self) -> u8 {
        match self {
            Self::Sunday => 7,
            Self::Monday => 6,
            Self::Tuesday => 5,
            Self::Wednesday => 4,
            Self::Thursday => 3,
            Self::Friday => 2,
            Self::Saturday => 1,
        }
    }
}

/// An inauspicious daytime window, as Julian Days (UT).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KalamWindow {
    /// Julian Day (UT) at which the window opens.
    pub start_jd: f64,
    /// Julian Day (UT) at which the window closes.
    pub end_jd: f64,
}

/// Rahu Kalam and Gulika Kalam as actual time windows for the vara containing
/// `jd_ut`, at the given observer.
///
/// Both are defined as one eighth of the daytime — sunrise to sunset divided
/// into eight equal parts — with the vara selecting which eighth. Returns
/// `(rahu, gulika)`, or `None` where the Sun does not both rise and set (polar
/// day and polar night), which is when the definition has no daytime to divide.
///
/// Source: Kalaprakashika (Rahu Kalam); Muhurtha Chintamani (Gulika Kalam).
#[must_use]
pub fn kalam_windows(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<(KalamWindow, KalamWindow)> {
    // ⚠️ `sun_rise_set` returns the FIRST rise and FIRST set inside its
    // 24-hour scan window, and they need not be in that order. Naively
    // pairing them can give a negative daytime and eight backwards windows.
    //
    // Concretely, with the `flat_sun` test fixture (RA/dec fixed at 0°),
    // scanning from day_start = JD 2459015.5 (2020-06-15 00:00 UT) at lon
    // 100°E finds the set at JD 2459015.741267 (≈05:47 UT) BEFORE the rise at
    // JD 2459016.235285 (≈17:39 UT) — measured directly via
    // `sun_rise_set(2459015.5, 0.0, 100.0, 0.0, flat_sun)`, not restated from
    // memory. (Honolulu does NOT reproduce this pattern for this fixture —
    // there the single-window pair comes out forward-ordered but anchored to
    // the *wrong* calendar day instead; see the derivation in
    // `kalam_windows_run_forwards_in_the_far_west` below.)
    //
    // So: find the sunrise that opens the vara's own day (the same backward
    // walk `vara_at` does), then take the sunset that FOLLOWS it by scanning
    // forward from the sunrise instant itself.
    let mut day_start = libm::floor(jd_ut - 0.5) + 0.5;
    let mut opening_sunrise = None;
    for _ in 0..4 {
        if let Some(r) = vedaksha_astro::riseset::sun_rise_set(
            day_start,
            lat_deg,
            lon_deg_east,
            elevation_m,
            equatorial,
        )
        .rise
        {
            if r <= jd_ut {
                opening_sunrise = Some(r);
                break;
            }
        }
        day_start -= 1.0;
    }
    let sunrise = opening_sunrise?;
    let sunset = vedaksha_astro::riseset::sun_rise_set(
        sunrise,
        lat_deg,
        lon_deg_east,
        elevation_m,
        equatorial,
    )
    .set?;

    let eighth = (sunset - sunrise) / 8.0;

    // Derived directly from `sunrise` (the walk-back result above) rather than
    // by calling `vara_at` again: `vara_at` re-runs the same backward walk
    // with `elevation_m` hardcoded to 0.0, which for a non-zero `elevation_m`
    // would pick a *different* sunrise than the one this window is anchored
    // to — the slot SELECTION and the window ANCHOR would then come from two
    // different horizons. `weekday_from_day_index` is the same private
    // helper `vara_at` applies to its own found rise, so this is the same
    // rule applied to a consistent sunrise, not a new one — and it avoids
    // repeating the walk-back a second time.
    let vara = weekday_from_day_index(sunrise + f64::from(tz_offset_minutes) / 1440.0);
    let window = |slot: u8| {
        // INVARIANT: `slot - 1` cannot underflow. Only `rahu_kalam_slot` and
        // `gulika_kalam_slot` ever feed this closure, and both are exhaustive
        // `match`es over all seven `Weekday` variants returning literals in
        // 1..=8 — there is no code path that can pass 0 here. Documented
        // rather than restructured into a checked/saturating form because a
        // panic on violation would be the correct signal that one of those
        // two match arms has been edited to return 0, not a case to paper
        // over silently.
        let start = sunrise + f64::from(slot - 1) * eighth;
        KalamWindow {
            start_jd: start,
            end_jd: start + eighth,
        }
    };

    Some((
        window(vara.rahu_kalam_slot()),
        window(vara.gulika_kalam_slot()),
    ))
}

/// Muhurta quality assessment for a given moment.
#[derive(Debug, Clone)]
pub struct MuhurtaAssessment {
    /// Julian Day.
    pub jd: f64,
    /// Moon's nakshatra.
    pub nakshatra: Nakshatra,
    /// Tithi (requires Sun and Moon longitude).
    pub tithi: Tithi,
    /// Day of the week.
    pub weekday: Weekday,
    /// Overall quality score (0.0 = inauspicious, 1.0 = highly auspicious).
    pub quality_score: f64,
    /// Specific factors contributing to the score.
    pub factors: Vec<String>,
    /// Julian Day at which the current tithi ends, if computed (requires the
    /// Moon/Sun daily motion — see [`compute_tithi_end`]). `None` from the
    /// position-only scan; populated for reported windows.
    pub tithi_end_jd: Option<f64>,
    /// Julian Day at which the current nakshatra ends, if computed (see
    /// [`compute_nakshatra_end`]). `None` from the position-only scan.
    pub nakshatra_end_jd: Option<f64>,
}

/// Compute the tithi from Sun and Moon sidereal longitudes.
///
/// `Tithi = floor((Moon_lon - Sun_lon) / 12) + 1`.
/// Source: BPHS.
#[must_use]
pub fn compute_tithi(moon_lon: f64, sun_lon: f64) -> Tithi {
    let diff = vedaksha_math::angle::normalize_degrees(moon_lon - sun_lon);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let number = (diff / 12.0).floor() as u8 + 1;
    let name = tithi_name(number);
    Tithi { number, name }
}

/// Get the **Universal Time** calendrical weekday for a Julian Day.
///
/// ⚠️ This is **not a vara.** A vara is the observer's day, reckoned from
/// local sunrise; the UT day boundary falls at a different local clock time at
/// every longitude, so this returns the wrong weekday for any instant between
/// that boundary and local midnight. Use [`vara_at`] for a vara.
///
/// Kept public because the UT weekday is a legitimate calendrical quantity —
/// it is only its use as a vara that is a defect.
///
/// Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 7.
#[must_use]
pub fn ut_weekday_from_jd(jd: f64) -> Weekday {
    weekday_from_day_index(jd)
}

/// Map a Julian Day to its weekday by the standard `(jd + 1.5) mod 7` index.
///
/// `rem_euclid` rather than `%` so a negative operand cannot produce a
/// negative remainder — unreachable for real Julian Days, but it makes the
/// fallback arm genuinely unreachable rather than incidentally so.
fn weekday_from_day_index(jd: f64) -> Weekday {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "libm::floor of a JD-scaled value is far inside i64, and \
                  rem_euclid(7) is in 0..=6, so neither cast can lose data"
    )]
    let day_index = (libm::floor(jd + 1.5) as i64).rem_euclid(7) as u8;
    match day_index {
        1 => Weekday::Monday,
        2 => Weekday::Tuesday,
        3 => Weekday::Wednesday,
        4 => Weekday::Thursday,
        5 => Weekday::Friday,
        6 => Weekday::Saturday,
        _ => Weekday::Sunday, // 0
    }
}

/// The **vara** — the Vedic weekday — for an observer, reckoned from local
/// sunrise to local sunrise.
///
/// The Vedic day begins at sunrise, so an instant between local midnight and
/// sunrise belongs to the *previous* vara. This starts from the sunrise that
/// opens the civil-UT day containing `jd_ut` and walks back, one civil UT day
/// at a time (up to four days), until a sunrise at or before `jd_ut` is
/// found, then takes the local civil weekday of that sunrise.
///
/// ⚠️ This is **not** guaranteed to return the most recent qualifying
/// sunrise. Each step scans a fixed one-day window starting at that day's
/// civil-UT midnight (`day_start`); if that window's rise falls within one
/// inter-rise gap *after* `day_start` and `jd_ut` itself lands just past
/// `day_start + 1`, the scan can settle on that rise while a later sunrise —
/// one that is still at or before `jd_ut`, in the following window — goes
/// unseen. For the real Sun this edge case spans roughly 22 seconds around
/// each day boundary; with the fixed-position `flat_sun` test fixture it
/// widens to roughly 4 minutes, from the sidereal/solar-day drift.
///
/// # Arguments
/// * `jd_ut` — the instant, Julian Day (UT)
/// * `lat_deg` — observer latitude, degrees
/// * `lon_deg_east` — observer longitude, degrees, east positive
/// * `tz_offset_minutes` — offset from UT of the observer's civil clock
/// * `equatorial` — the Sun's apparent `(right_ascension_deg, declination_deg)`
///   at a Julian Day (UT), or `None` where unavailable
///
/// Inside the polar day and polar night no sunrise exists and the
/// sunrise-to-sunrise vara is undefined; there this falls back to the local
/// civil weekday. The same fallback also fires if `equatorial` returns
/// `None` for every `day_start` tried — ephemeris unavailable, a different
/// situation from the polar one — and the two are indistinguishable in the
/// return value: a caller that needs to tell them apart must probe
/// `equatorial` itself. That fallback is a documented convention, not a
/// classical rule.
///
/// Source: Muhurta Chintamani; Kalaprakashika (the sunrise reckoning).
#[must_use]
pub fn vara_at(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    tz_offset_minutes: i32,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Weekday {
    let local_weekday =
        |jd: f64| weekday_from_day_index(jd + f64::from(tz_offset_minutes) / 1440.0);

    // 0h UT of the civil UT day containing `jd_ut`, then walk back until a
    // sunrise at or before the instant is found. Four days is ample: only the
    // polar cases fail, and they fail for every day.
    let mut day_start = libm::floor(jd_ut - 0.5) + 0.5;
    for _ in 0..4 {
        // NOTE: written as a nested `if let`, not a let-chain. The workspace
        // `rust-version` is 1.85 and let-chains did not stabilise until 1.88,
        // so a chain here would silently raise the MSRV. No other file in the
        // workspace uses one.
        if let Some(rise) =
            vedaksha_astro::riseset::sun_rise_set(day_start, lat_deg, lon_deg_east, 0.0, equatorial)
                .rise
        {
            if rise <= jd_ut {
                return local_weekday(rise);
            }
        }
        day_start -= 1.0;
    }
    local_weekday(jd_ut)
}

/// Assess muhurta quality for a given moment.
///
/// # Arguments
/// * `jd` — Julian Day (UT)
/// * `moon_sidereal_lon` — Moon's sidereal longitude in degrees
/// * `sun_sidereal_lon` — Sun's sidereal longitude in degrees
/// * `weekday` — the **vara**, which the caller must derive with [`vara_at`].
///   Taken rather than computed because a vara needs an observer and this
///   function has none.
#[must_use]
pub fn assess_muhurta(
    jd: f64,
    moon_sidereal_lon: f64,
    sun_sidereal_lon: f64,
    weekday: Weekday,
) -> MuhurtaAssessment {
    let nakshatra = Nakshatra::from_longitude(moon_sidereal_lon);
    let tithi = compute_tithi(moon_sidereal_lon, sun_sidereal_lon);

    let mut score = 0.5_f64; // neutral baseline
    let mut factors = Vec::new();

    // Auspicious nakshatras for general muhurta.
    // Source: Muhurta Chintamani; BPHS.
    let auspicious_nakshatras = [
        Nakshatra::Ashwini,
        Nakshatra::Rohini,
        Nakshatra::Mrigashira,
        Nakshatra::Punarvasu,
        Nakshatra::Pushya,
        Nakshatra::Hasta,
        Nakshatra::Swati,
        Nakshatra::Anuradha,
        Nakshatra::Shravana,
        Nakshatra::Dhanishta,
        Nakshatra::Revati,
    ];

    if auspicious_nakshatras.contains(&nakshatra) {
        score += 0.2;
        factors.push(format!("{} is an auspicious nakshatra", nakshatra.name()));
    }

    // Inauspicious nakshatras.
    let inauspicious = [
        Nakshatra::Bharani,
        Nakshatra::Ardra,
        Nakshatra::Ashlesha,
        Nakshatra::Jyeshtha,
        Nakshatra::Moola,
    ];
    if inauspicious.contains(&nakshatra) {
        score -= 0.2;
        factors.push(format!("{} is generally inauspicious", nakshatra.name()));
    }

    // Auspicious tithis (2, 3, 5, 7, 10, 11, 13 of each paksha).
    let tithi_in_paksha = if tithi.number <= 15 {
        tithi.number
    } else {
        tithi.number - 15
    };
    let auspicious_tithis = [2u8, 3, 5, 7, 10, 11, 13];
    if auspicious_tithis.contains(&tithi_in_paksha) {
        score += 0.15;
        factors.push(format!("{} is auspicious", tithi.name));
    }

    // Avoid Amavasya (30 / new moon).
    if tithi.number == 30 {
        score -= 0.3;
        factors.push("Amavasya (new moon) — avoid".into());
    }

    // Weekday considerations (simplified).
    // Source: Muhurta Chintamani.
    match weekday {
        Weekday::Monday | Weekday::Wednesday | Weekday::Thursday | Weekday::Friday => {
            score += 0.1;
            factors.push(format!("{weekday:?} is generally favorable"));
        }
        Weekday::Tuesday | Weekday::Saturday => {
            score -= 0.1;
            factors.push(format!("{weekday:?} — use caution"));
        }
        Weekday::Sunday => {}
    }

    score = score.clamp(0.0, 1.0);

    MuhurtaAssessment {
        jd,
        nakshatra,
        tithi,
        weekday,
        quality_score: score,
        factors,
        // Position-only scan does not compute boundary times; enriched later
        // for reported windows via compute_tithi_end / compute_nakshatra_end.
        tithi_end_jd: None,
        nakshatra_end_jd: None,
    }
}

/// Refine the Julian Day at which a monotonically-increasing angle (degrees)
/// reaches `target_deg`, by Newton iteration `jd ← jd + (target − angle)/rate`.
///
/// `angle_at(jd)` returns `(angle_deg ∈ [0, 360), rate_deg_per_day)` — the rate
/// is the body's daily motion, i.e. the analytic derivative. The lunar and
/// elongation angles are smooth and monotone, so this converges in a few steps.
fn refine_crossing(
    target_deg: f64,
    jd_init: f64,
    angle_at: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let mut jd = jd_init;
    for _ in 0..8 {
        let (angle, rate) = angle_at(jd)?;
        // Signed (target − angle), wrapped to (−180, 180].
        let mut f = (target_deg - angle).rem_euclid(360.0);
        if f > 180.0 {
            f -= 360.0;
        }
        if f.abs() < 1.0e-6 {
            return Some(jd); // < ~0.0036 arcsec ⇒ sub-second in time
        }
        if rate.abs() < 1.0e-9 {
            return None; // degenerate (no motion)
        }
        jd += f / rate;
    }
    Some(jd)
}

/// Julian Day at which the current **tithi** ends — when the Moon–Sun
/// elongation reaches the next multiple of 12° — refined against true
/// longitudes. The variable real duration of a tithi (≈20–27 h) comes
/// entirely from the varying lunar speed, which is why the daily motion is
/// required here rather than a mean-motion approximation.
///
/// `moon` / `sun` return `(longitude_deg, daily_motion_deg_per_day)`. Tropical
/// or sidereal input both work — the ayanamsha cancels in the elongation and
/// its rate.
///
/// # Errors / `None`
/// Returns `None` if a callback yields `None` or the elongation rate vanishes.
#[must_use]
pub fn compute_tithi_end(
    jd: f64,
    moon: &dyn Fn(f64) -> Option<(f64, f64)>,
    sun: &dyn Fn(f64) -> Option<(f64, f64)>,
) -> Option<f64> {
    let (m0, _) = moon(jd)?;
    let (s0, _) = sun(jd)?;
    let elong = (m0 - s0).rem_euclid(360.0);
    let next_boundary = ((elong / 12.0).floor() + 1.0) * 12.0;
    let angle_at = |t: f64| -> Option<(f64, f64)> {
        let (ml, ms) = moon(t)?;
        let (sl, ss) = sun(t)?;
        Some(((ml - sl).rem_euclid(360.0), ms - ss))
    };
    refine_crossing(next_boundary, jd, &angle_at)
}

/// Julian Day at which the current **nakshatra** ends — when the Moon's
/// sidereal longitude reaches the next multiple of 360/27° — refined against
/// the true lunar longitude.
///
/// `moon` returns `(sidereal_longitude_deg, daily_motion_deg_per_day)`.
///
/// # Errors / `None`
/// Returns `None` if the callback yields `None` or the lunar rate vanishes.
#[must_use]
pub fn compute_nakshatra_end(jd: f64, moon: &dyn Fn(f64) -> Option<(f64, f64)>) -> Option<f64> {
    const SPAN: f64 = 360.0 / 27.0;
    let (m0, _) = moon(jd)?;
    let m = m0.rem_euclid(360.0);
    let next_boundary = ((m / SPAN).floor() + 1.0) * SPAN;
    let angle_at = |t: f64| -> Option<(f64, f64)> {
        let (ml, ms) = moon(t)?;
        Some((ml.rem_euclid(360.0), ms))
    };
    refine_crossing(next_boundary, jd, &angle_at)
}

/// Search for auspicious muhurta windows in a date range.
///
/// Evaluates at 0.5-day intervals (roughly dawn and dusk).
///
/// # Arguments
/// * `start_jd` — start of search range
/// * `end_jd` — end of search range
/// * `lat_deg` / `lon_deg_east` / `tz_offset_minutes` — the observer, needed
///   to derive the vara (see [`vara_at`]) for each candidate instant.
/// * `get_moon_lon` — callback returning Moon sidereal longitude at JD
/// * `get_sun_lon` — callback returning Sun sidereal longitude at JD
/// * `equatorial` — the Sun's apparent `(right_ascension_deg, declination_deg)`
///   at a JD (UT), passed through to [`vara_at`]'s sunrise search.
/// * `min_quality` — minimum quality score (0.0-1.0) to include
///
/// # Performance
///
/// One [`vara_at`] call costs roughly 2 × (288 coarse scan steps + bisection)
/// `equatorial` evaluations, because each `sun_rise_set` scans a day at
/// 5-minute resolution and the walk-back always consumes two iterations.
/// Deriving the vara fresh at every 0.5-day step would put a one-year search
/// near 420,000 evaluations — a latency regression measured in minutes on a
/// tool that is otherwise fast. The vara changes at most once per civil day,
/// so this memoises on `libm::floor(jd - 0.5) + 0.5` (the same civil-day-start
/// `vara_at` computes internally) and recomputes only when it changes: one
/// `vara_at` call per civil day in the range, not one per step.
// The ninth argument is the observer's tz offset, needed alongside lat/lon to
// derive the vara per candidate instant (see the performance note above) —
// splitting it into a struct would obscure the callback wiring more than it
// helps. Same precedent as `vedaksha_astro::transits` and
// `vedaksha_ephem_core`.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn search_muhurta(
    start_jd: f64,
    end_jd: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    tz_offset_minutes: i32,
    get_moon_lon: &dyn Fn(f64) -> Option<f64>,
    get_sun_lon: &dyn Fn(f64) -> Option<f64>,
    equatorial: &dyn Fn(f64) -> Option<(f64, f64)>,
    min_quality: f64,
) -> Vec<MuhurtaAssessment> {
    let mut results = Vec::new();
    let mut jd = start_jd;
    let step = 0.5; // check every half day

    // (civil_day_start, vara) of the most recently computed vara. Recomputed
    // only when the civil day changes — see the performance note above.
    let mut memo: Option<(f64, Weekday)> = None;

    while jd <= end_jd {
        if let (Some(moon), Some(sun)) = (get_moon_lon(jd), get_sun_lon(jd)) {
            let day_start = libm::floor(jd - 0.5) + 0.5;
            // Exact equality is intentional, not a tolerance bug: both sides
            // come from the identical `libm::floor(jd - 0.5) + 0.5` formula
            // (this line and `vara_at`'s internal day-start walk), so they
            // are bit-identical whenever `jd` falls in the same civil day —
            // there is no accumulated float error to compare within.
            #[allow(clippy::float_cmp)]
            let same_day =
                matches!(memo, Some((cached_day_start, _)) if cached_day_start == day_start);
            let vara = match memo {
                Some((_, cached_vara)) if same_day => cached_vara,
                _ => {
                    let v = vara_at(jd, lat_deg, lon_deg_east, tz_offset_minutes, equatorial);
                    memo = Some((day_start, v));
                    v
                }
            };
            let assessment = assess_muhurta(jd, moon, sun, vara);
            if assessment.quality_score >= min_quality {
                results.push(assessment);
            }
        }
        jd += step;
    }

    results
}

fn tithi_name(number: u8) -> &'static str {
    match number {
        1 => "Shukla Pratipada",
        2 => "Shukla Dwitiya",
        3 => "Shukla Tritiya",
        4 => "Shukla Chaturthi",
        5 => "Shukla Panchami",
        6 => "Shukla Shashthi",
        7 => "Shukla Saptami",
        8 => "Shukla Ashtami",
        9 => "Shukla Navami",
        10 => "Shukla Dashami",
        11 => "Shukla Ekadashi",
        12 => "Shukla Dwadashi",
        13 => "Shukla Trayodashi",
        14 => "Shukla Chaturdashi",
        15 => "Purnima",
        16 => "Krishna Pratipada",
        17 => "Krishna Dwitiya",
        18 => "Krishna Tritiya",
        19 => "Krishna Chaturthi",
        20 => "Krishna Panchami",
        21 => "Krishna Shashthi",
        22 => "Krishna Saptami",
        23 => "Krishna Ashtami",
        24 => "Krishna Navami",
        25 => "Krishna Dashami",
        26 => "Krishna Ekadashi",
        27 => "Krishna Dwadashi",
        28 => "Krishna Trayodashi",
        29 => "Krishna Chaturdashi",
        30 => "Amavasya",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    const EPS: f64 = 1e-9;

    // --- compute_tithi ---

    #[test]
    fn tithi_moon_30_ahead_is_tritiya() {
        // diff = 30°, 30/12 = 2.5 → floor = 2 → number = 3 (Tritiya)
        let tithi = compute_tithi(30.0, 0.0);
        assert_eq!(tithi.number, 3, "Expected tithi 3 (Tritiya)");
        assert_eq!(tithi.name, "Shukla Tritiya");
    }

    #[test]
    fn tithi_moon_equals_sun_is_pratipada() {
        // diff = 0°, 0/12 = 0 → number = 1 (Pratipada)
        let tithi = compute_tithi(0.0, 0.0);
        assert_eq!(tithi.number, 1, "Expected tithi 1 (Pratipada)");
        assert_eq!(tithi.name, "Shukla Pratipada");
    }

    #[test]
    fn tithi_moon_180_ahead_is_in_krishna_paksha() {
        // diff = 180°, 180/12 = 15 → floor = 15 → number = 16 (Krishna Pratipada)
        let tithi = compute_tithi(180.0, 0.0);
        assert_eq!(tithi.number, 16, "Expected tithi 16 (Krishna Pratipada)");
    }

    #[test]
    fn tithi_purnima() {
        // diff = 180° - ε → tithi 15 (Purnima)
        // 14*12 = 168°, 15*12 = 180°: need 168 ≤ diff < 180
        let tithi = compute_tithi(170.0, 0.0);
        assert_eq!(tithi.number, 15, "Expected tithi 15 (Purnima)");
        assert_eq!(tithi.name, "Purnima");
    }

    #[test]
    fn tithi_amavasya() {
        // diff = 348°: 348/12 = 29 → floor=29 → number=30 (Amavasya)
        let tithi = compute_tithi(348.0, 0.0);
        assert_eq!(tithi.number, 30, "Expected tithi 30 (Amavasya)");
        assert_eq!(tithi.name, "Amavasya");
    }

    #[test]
    fn tithi_with_nonzero_sun() {
        // Moon at 50°, Sun at 20° → diff = 30° → tithi 3
        let tithi = compute_tithi(50.0, 20.0);
        assert_eq!(tithi.number, 3, "Expected tithi 3 (Tritiya)");
    }

    // --- ut_weekday_from_jd ---

    #[test]
    fn weekday_j2000_is_saturday() {
        // J2000.0 = JD 2451545.0 = Jan 1.5, 2000 = Saturday
        let wd = ut_weekday_from_jd(2_451_545.0);
        assert_eq!(wd, Weekday::Saturday, "J2000.0 should be Saturday");
    }

    #[test]
    fn weekday_advances_correctly() {
        // J2000.0 = Saturday; J2000.0 + 1 = Sunday
        let wd = ut_weekday_from_jd(2_451_546.0);
        assert_eq!(wd, Weekday::Sunday);
        let wd2 = ut_weekday_from_jd(2_451_547.0);
        assert_eq!(wd2, Weekday::Monday);
    }

    /// A Sun fixed on the equator, good enough to place a sunrise near 06:00
    /// local apparent time at any equatorial longitude.
    fn flat_sun(_jd: f64) -> Option<(f64, f64)> {
        Some((0.0, 0.0))
    }

    /// The defect KundaliMCP reported: `weekday_from_jd` returns the UT
    /// weekday, which flips at a different local clock time at every
    /// longitude. West of about −150° the UT day has already rolled over
    /// while it is still the previous evening locally.
    #[test]
    fn vara_does_not_follow_the_ut_day_boundary_in_the_far_west() {
        // 2020-06-14 20:00 Honolulu (UTC−10) == 2020-06-15 06:00Z, JD 2459015.75.
        let jd = 2_459_015.75;
        assert_eq!(
            ut_weekday_from_jd(jd),
            Weekday::Monday,
            "precondition: the UT weekday really is Monday here"
        );
        let vara = vara_at(jd, 21.3069, -157.8583, -600, &flat_sun);
        // Not because 20:00 "is still evening, before local midnight" — vara_at
        // does not reckon from midnight, and that framing would just be
        // re-describing the UT-day defect this test exists to catch. Derived
        // directly from `sun_rise_set`, the same call `vara_at` makes
        // internally: the first window it tries (day_start = JD 2459015.5 =
        // 2020-06-15 00:00Z) puts the fake sunrise at JD 2459015.952163512,
        // which is *after* `jd` and so is rejected; walking back one civil UT
        // day (day_start = JD 2459014.5 = 2020-06-14 00:00Z) puts it at JD
        // 2459014.954893945 = 2020-06-14 10:55:03 UT = 2020-06-14 00:55:03
        // local — a few minutes after local midnight, still on the same
        // Sunday. That sunrise is at or before `jd`, so its local civil
        // weekday (Sunday) is the vara returned.
        assert_eq!(
            vara,
            Weekday::Sunday,
            "the walked-back sunrise (2020-06-14 ~00:55 local) is at or before the query instant, so this is still Ravivara"
        );
    }

    /// The report asked specifically for a case east of +150° as well as one
    /// west of −150°: inside ±90° the UT and local days usually agree, which is
    /// exactly why this class of bug survives a green suite. In KundaliMCP's
    /// own harness 19 of 20 fixtures sat inside that band and the identical bug
    /// produced zero diffs across 928 snapshots.
    ///
    /// At one fixed UT instant the far east and the far west are ~22 hours
    /// apart on the civil clock, so they cannot share a vara. Asserted as a
    /// relation rather than two hard-coded weekdays: a hard-coded weekday here
    /// would be asserting the plan author's arithmetic, not the code's.
    #[test]
    fn far_east_and_far_west_cannot_share_a_vara_at_one_instant() {
        let jd = 2_459_015.75; // 2020-06-15 06:00Z
        let east = vara_at(jd, 0.0, 165.0, 660, &flat_sun); // UTC+11
        let west = vara_at(jd, 0.0, -165.0, -660, &flat_sun); // UTC−11
        assert_ne!(
            east, west,
            "observers 22 civil hours apart must not share a vara at one instant"
        );
    }

    /// A pair of point tests can pass by luck. Sweep the whole globe at one
    /// fixed UT instant: some longitudes must agree with the UT weekday and
    /// some must disagree. All-agree means the fix did nothing; all-disagree
    /// means it is not tracking the calendar at all.
    #[test]
    fn a_full_longitude_sweep_shows_the_vara_is_observer_dependent() {
        let jd = 2_459_015.75;
        let ut = ut_weekday_from_jd(jd);
        let (mut agreeing, mut differing) = (0_u32, 0_u32);
        let mut lon_i = -180_i32;
        while lon_i <= 180 {
            // Civil clock approximated by the nearest whole hour of solar time.
            // Integer arithmetic throughout — no float casts to placate clippy.
            let tz_minutes = (lon_i / 15) * 60;
            if vara_at(jd, 0.0, f64::from(lon_i), tz_minutes, &flat_sun) == ut {
                agreeing += 1;
            } else {
                differing += 1;
            }
            lon_i += 5;
        }
        assert!(
            differing > 0,
            "no longitude disagreed with the UT weekday — this sweep proves nothing"
        );
        assert!(
            agreeing > 0,
            "every longitude disagreed — the vara is not tracking the calendar"
        );
    }

    /// The vara turns at sunrise, not at local midnight: an instant between
    /// midnight and sunrise still belongs to the previous Vedic day.
    ///
    /// `flat_sun` fixes RA at 0°, not the Sun's real varying RA, so its
    /// "sunrise" does not land near 06:00 UT the way a real Sun's would: GMST
    /// at JD 2451544.5 (2000-01-01 00:00 UT) is already ~99.97° (~6.66h)
    /// ahead of RA=0, so the window's first altitude crossing is well past
    /// 06:00 UT. Rather than assert an unverified clock time, this derives
    /// the actual rise instant from `sun_rise_set` directly — the same call
    /// `vara_at` makes internally for this `day_start` — and brackets it by
    /// ±1h. Measured: rise ≈ day_start + 0.468 d ≈ 11:14 UT on 2000-01-01, a
    /// Saturday, so 10:14 UT is still Friday's vara and 12:14 UT is Saturday's.
    #[test]
    fn vara_turns_at_sunrise_not_at_local_midnight() {
        let day_start = 2_451_544.5; // 2000-01-01 00:00 UT, a Saturday.
        let rise = vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, 0.0, &flat_sun)
            .rise
            .expect("equatorial sun must rise");

        let one_hour = 1.0 / 24.0;
        assert_eq!(
            vara_at(rise - one_hour, 0.0, 0.0, 0, &flat_sun),
            Weekday::Friday,
            "before sunrise the vara is still the previous day's"
        );
        assert_eq!(
            vara_at(rise + one_hour, 0.0, 0.0, 0, &flat_sun),
            Weekday::Saturday,
            "after sunrise the vara has turned"
        );
    }

    /// Polar night has no sunrise to bound the instant, so the documented
    /// fallback (local civil weekday) must apply rather than a panic or a
    /// silent wrong answer.
    #[test]
    fn vara_falls_back_where_the_sun_does_not_rise() {
        let southern_sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        // Longyearbyen in midwinter.
        let v = vara_at(2_451_544.5, 78.22, 15.65, 60, &southern_sun);
        assert_eq!(
            v,
            Weekday::Saturday,
            "must fall back to the local civil weekday"
        );
    }

    // --- assess_muhurta ---

    #[test]
    fn auspicious_nakshatra_boosts_score() {
        // Rohini: lon ≈ 3*13.333 = 40°
        // Use Pushya: index 7 → lon ≈ 7*13.333 = 93.333°
        // JD 2451545.0 = J2000.0 = Saturday, per `weekday_j2000_is_saturday`
        // above — `assess_muhurta` no longer derives the vara itself, so it
        // is passed explicitly to keep this test's inputs identical to what
        // the old internal `ut_weekday_from_jd(jd)` call would have produced.
        let assessment = assess_muhurta(2_451_545.0, 94.0, 0.0, Weekday::Saturday);
        assert!(
            assessment.quality_score > 0.5,
            "Auspicious nakshatra should boost score above baseline"
        );
        assert!(
            assessment
                .factors
                .iter()
                .any(|f| f.contains("auspicious nakshatra")),
            "Expected an auspicious-nakshatra factor"
        );
    }

    #[test]
    fn amavasya_reduces_score() {
        // Amavasya: tithi 30, need diff ~348° → Moon lon = Sun lon + 348°
        // Sun at 0°, Moon at 348°
        // JD 2451545.0 = J2000.0 = Saturday (see comment above).
        let assessment = assess_muhurta(2_451_545.0, 348.0, 0.0, Weekday::Saturday);
        assert_eq!(assessment.tithi.number, 30, "Should be Amavasya");
        assert!(
            assessment.factors.iter().any(|f| f.contains("Amavasya")),
            "Expected an Amavasya factor"
        );
    }

    #[test]
    fn inauspicious_nakshatra_reduces_score() {
        // Ardra: index 5 → lon ≈ 5*13.333 = 66.666°
        // JD 2451545.0 = J2000.0 = Saturday (see comment above).
        let assessment = assess_muhurta(2_451_545.0, 67.0, 0.0, Weekday::Saturday);
        assert!(
            assessment
                .factors
                .iter()
                .any(|f| f.contains("inauspicious")),
            "Expected an inauspicious factor for Ardra"
        );
    }

    #[test]
    fn score_clamped_between_zero_and_one() {
        // Worst case: inauspicious nakshatra + Amavasya + Saturday
        // Baseline 0.5 - 0.2 (inauspicious) - 0.3 (Amavasya) - 0.1 (Saturday) = -0.1 → clamped to 0.0
        // Need: Ardra + Amavasya. Find a Saturday JD.
        // J2000.0 = Saturday. Moon = 348° (Amavasya region), need Ardra lon ~66-79°
        // These conflict — just test score is in [0, 1]
        // JD 2451545.0 = J2000.0 = Saturday (see comment above).
        let assessment = assess_muhurta(2_451_545.0, 67.0, 0.0, Weekday::Saturday);
        assert!(
            assessment.quality_score >= 0.0 && assessment.quality_score <= 1.0,
            "Score must be in [0, 1], got {}",
            assessment.quality_score
        );
    }

    // --- search_muhurta ---

    #[test]
    fn search_returns_results_above_threshold() {
        // Auspicious setup: Pushya nakshatra (~94°), tithi 3 (30° diff), Thursday (JD+4 from Saturday)
        // JD 2451549.0 = Wednesday (Sat+4)
        let moon_lon = 94.0_f64; // Pushya
        let sun_lon = 64.0_f64; // diff = 30° → tithi 3 (auspicious)
        let results = search_muhurta(
            2_451_545.0,
            2_451_546.0,
            0.0,
            0.0,
            0,
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            &flat_sun,
            0.5,
        );
        assert!(
            !results.is_empty(),
            "Should find at least one result above min_quality=0.5"
        );
    }

    #[test]
    fn search_high_threshold_returns_fewer_results() {
        let moon_lon = 94.0_f64;
        let sun_lon = 64.0_f64;
        let low_threshold = search_muhurta(
            2_451_545.0,
            2_451_555.0,
            0.0,
            0.0,
            0,
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            &flat_sun,
            0.0,
        );
        let high_threshold = search_muhurta(
            2_451_545.0,
            2_451_555.0,
            0.0,
            0.0,
            0,
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            &flat_sun,
            0.99,
        );
        assert!(
            high_threshold.len() <= low_threshold.len(),
            "Higher threshold should yield fewer or equal results"
        );
    }

    #[test]
    fn search_returns_empty_when_callback_returns_none() {
        let results = search_muhurta(
            2_451_545.0,
            2_451_550.0,
            0.0,
            0.0,
            0,
            &|_| None,
            &|_| None,
            &flat_sun,
            0.0,
        );
        assert!(
            results.is_empty(),
            "No assessments when callbacks return None"
        );
    }

    #[test]
    fn search_step_is_half_day() {
        // Over a 1-day range, expect exactly 3 samples: start, start+0.5, start+1.0
        let mut count = 0usize;
        let _ = search_muhurta(
            2_451_545.0,
            2_451_546.0,
            0.0,
            0.0,
            0,
            &|_| Some(94.0),
            &|_jd| Some(64.0),
            &flat_sun,
            0.0,
        )
        .len();
        // Verify count via the returned vec length (3 steps: 0, 0.5, 1.0)
        let results = search_muhurta(
            2_451_545.0,
            2_451_546.0,
            0.0,
            0.0,
            0,
            &|_| Some(94.0),
            &|_| Some(64.0),
            &flat_sun,
            0.0,
        );
        assert_eq!(
            results.len(),
            3,
            "Expected 3 samples over a 1-day range with 0.5-day step"
        );
        let _ = count;
        count = results.len();
        assert_eq!(count, 3);
    }

    /// Pins the memoisation actually firing: without it, `search_muhurta`
    /// would call `vara_at` (and so `equatorial`) once per 0.5-day STEP;
    /// with it, once per distinct CIVIL DAY. This counts `equatorial`
    /// invocations with a `Cell<u32>` rather than asserting a hand-derived
    /// number, because the exact per-`vara_at`-call cost (walk-back count ×
    /// scan resolution × bisection iterations) is an implementation detail
    /// of `vara_at`/`sun_rise_set` this test should not have to know. The
    /// reference count is instead measured directly: call `vara_at` once
    /// per expected trigger point (unmemoised) and sum, then assert the
    /// memoised `search_muhurta` run costs exactly that much — not more (no
    /// regression to per-step) and not less (memoisation isn't skipping a
    /// day it should have priced).
    #[test]
    fn search_muhurta_memoises_vara_per_civil_day_not_per_step() {
        use std::cell::Cell;

        let (lat, lon, tz) = (0.0, 0.0, 0);

        // `start` is chosen to already equal a civil-day-start
        // (`libm::floor(jd - 0.5) + 0.5 == jd`, since 2_451_545.5 - 0.5 =
        // 2_451_545.0 is an integer) so every one of the 5 civil-day
        // boundaries below is crossed at the SAME 0.0 offset into its day —
        // otherwise the first (partial) day would cost a different number of
        // `equatorial` calls than the other four and the two independently
        // measured totals below would not need to agree even under correct
        // memoisation.
        let start = 2_451_545.5;
        let end = start + 4.0; // 4 civil days later
        // With a 0.5-day step, start..=end is 9 samples (8 steps of 0.5 sum
        // exactly to 4.0 in f64, since 0.5 is a power-of-two fraction) but
        // only 5 distinct civil-day-starts: start, start+1.0, ..., start+4.0
        // — verified by the `results.len() == 9` assertion below and by
        // `libm::floor(jd - 0.5) + 0.5` being identical for `start + k` and
        // `start + k + 0.5` for each integer k in 0..4.
        let trigger_jds = [start, start + 1.0, start + 2.0, start + 3.0, start + 4.0];

        // Reference: sum of `equatorial` calls from 5 direct, unmemoised
        // `vara_at` calls — one at each expected memoisation trigger point.
        // This is exactly what a correctly memoising `search_muhurta` should
        // cost in total, however many `equatorial` evaluations one `vara_at`
        // call happens to take.
        let expected_calls = Cell::new(0u32);
        for &jd in &trigger_jds {
            let counted = |t: f64| {
                expected_calls.set(expected_calls.get() + 1);
                flat_sun(t)
            };
            let _ = vara_at(jd, lat, lon, tz, &counted);
        }
        let expected = expected_calls.get();
        assert!(
            expected > 0,
            "sanity: vara_at must call `equatorial` at least once"
        );

        let actual_calls = Cell::new(0u32);
        let counted = |t: f64| {
            actual_calls.set(actual_calls.get() + 1);
            flat_sun(t)
        };
        let results = search_muhurta(
            start,
            end,
            lat,
            lon,
            tz,
            &|_| Some(94.0),
            &|_| Some(64.0),
            &counted,
            0.0,
        );
        assert_eq!(
            results.len(),
            9,
            "9 samples at a 0.5-day step over a 4-day span"
        );
        assert_eq!(
            actual_calls.get(),
            expected,
            "memoisation must cost exactly 5 direct `vara_at` calls' worth of \
             `equatorial` invocations ({expected}) — one per distinct civil \
             day — not one per of the 9 steps"
        );
    }

    // --- paksha, lord, remaining_degrees ---

    #[test]
    fn tithi_paksha_shukla() {
        let t = compute_tithi(24.0, 0.0);
        assert_eq!(t.paksha(), Paksha::Shukla);
    }

    #[test]
    fn tithi_paksha_krishna() {
        let t = compute_tithi(200.0, 0.0);
        assert_eq!(t.paksha(), Paksha::Krishna);
    }

    #[test]
    fn tithi_has_paksha_name() {
        let t = compute_tithi(15.0, 0.0);
        assert!(t.name.starts_with("Shukla"));
    }

    #[test]
    fn purnima_name() {
        // diff=174° → floor(174/12)+1 = 15
        let t = compute_tithi(174.0, 0.0);
        assert_eq!(t.name, "Purnima");
    }

    #[test]
    fn amavasya_name() {
        // diff=354° → floor(354/12)+1 = 30
        let t = compute_tithi(354.0, 0.0);
        assert_eq!(t.name, "Amavasya");
    }

    #[test]
    fn tithi_lord_exists() {
        let t = compute_tithi(15.0, 0.0);
        assert!(!t.lord().is_empty());
    }

    #[test]
    fn tithi_remaining_degrees_positive() {
        let r = Tithi::remaining_degrees(45.0, 0.0);
        assert!(r > 0.0 && r <= 12.0);
    }

    // --- Weekday lord, Rahu Kalam, Gulika Kalam ---

    #[test]
    fn sunday_lord_is_sun() {
        assert_eq!(Weekday::Sunday.lord(), "Sun");
    }

    #[test]
    fn saturday_lord_is_saturn() {
        assert_eq!(Weekday::Saturday.lord(), "Saturn");
    }

    #[test]
    fn monday_rahu_kalam_is_slot_2() {
        assert_eq!(Weekday::Monday.rahu_kalam_slot(), 2);
    }

    #[test]
    fn saturday_gulika_kalam_is_slot_1() {
        assert_eq!(Weekday::Saturday.gulika_kalam_slot(), 1);
    }

    #[test]
    fn all_weekdays_have_lords() {
        let days = [
            Weekday::Sunday,
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
        ];
        for d in &days {
            assert!(!d.lord().is_empty());
        }
    }

    // --- tithi / nakshatra ending times ---

    #[test]
    fn tithi_end_linear_synthetic() {
        // Moon 13.176°/day, Sun 0.985°/day; both at 0° at j0 ⇒ elongation 0.
        let j0 = 2_451_545.0;
        let moon = |jd: f64| Some(((13.176 * (jd - j0)).rem_euclid(360.0), 13.176));
        let sun = |jd: f64| Some(((0.985 * (jd - j0)).rem_euclid(360.0), 0.985));
        let end = compute_tithi_end(j0, &moon, &sun).expect("tithi end");
        let expected = j0 + 12.0 / (13.176 - 0.985); // first 12° elongation boundary
        assert!((end - expected).abs() < 1e-9, "end {end} vs {expected}");
    }

    #[test]
    fn nakshatra_end_linear_synthetic() {
        let j0 = 2_451_545.0;
        let moon = |jd: f64| Some(((13.176 * (jd - j0)).rem_euclid(360.0), 13.176));
        let end = compute_nakshatra_end(j0, &moon).expect("nakshatra end");
        let expected = j0 + (360.0 / 27.0) / 13.176;
        assert!((end - expected).abs() < 1e-9, "end {end} vs {expected}");
    }

    // --- kalam_windows ---

    /// Rahu Kalam is the Nth eighth of the daytime, counted from sunrise. Its
    /// width must therefore be exactly one eighth of the day's length, and it
    /// must sit inside sunrise..sunset.
    #[test]
    fn rahu_kalam_is_one_eighth_of_the_daytime() {
        let (rahu, gulika) =
            kalam_windows(2_451_544.5 + 0.5, 0.0, 0.0, 0.0, 0, &flat_sun).expect("sun rises here");
        let rs = vedaksha_astro::riseset::sun_rise_set(2_451_544.5, 0.0, 0.0, 0.0, &flat_sun);
        let (sunrise, sunset) = (rs.rise.unwrap(), rs.set.unwrap());
        let eighth = (sunset - sunrise) / 8.0;

        assert!(
            ((rahu.end_jd - rahu.start_jd) - eighth).abs() < 1e-9,
            "rahu kalam must be exactly one eighth of the daytime"
        );
        assert!(
            rahu.start_jd >= sunrise - 1e-9 && rahu.end_jd <= sunset + 1e-9,
            "rahu kalam must lie within sunrise..sunset"
        );
        assert!(
            ((gulika.end_jd - gulika.start_jd) - eighth).abs() < 1e-9,
            "gulika kalam must be exactly one eighth of the daytime"
        );
    }

    /// The window must be placed by the vara's slot table. Saturday's gulika
    /// slot is 1, so on a Saturday gulika begins exactly at sunrise.
    #[test]
    fn saturday_gulika_starts_at_sunrise() {
        // 2000-01-01 12:00 UT is a Saturday at longitude 0.
        let (_, gulika) =
            kalam_windows(2_451_544.5 + 0.5, 0.0, 0.0, 0.0, 0, &flat_sun).expect("sun rises here");
        let rs = vedaksha_astro::riseset::sun_rise_set(2_451_544.5, 0.0, 0.0, 0.0, &flat_sun);
        assert!(
            (gulika.start_jd - rs.rise.unwrap()).abs() < 1e-9,
            "Saturday gulika is slot 1, so it starts at sunrise"
        );
    }

    /// `tz_offset_minutes` selects which weekday's slot table applies, and
    /// Rahu sits in a different eighth on different weekdays. Derived by
    /// direct computation while writing this test (not restated from the
    /// brief): at lon 165°E, `vara_at(2_459_015.75, 0.0, 165.0, 660,
    /// flat_sun)` (tz honoured) is Monday — rahu slot 2 — while
    /// `vara_at(2_459_015.75, 0.0, 165.0, 0, flat_sun)` (tz dropped) is
    /// Sunday — rahu slot 8. Slot 2 sits in the first quarter of the
    /// daytime and slot 8 is the last eighth, ending at sunset, so
    /// `kalam_windows` under the two tz values must place rahu's start more
    /// than 0.25 day (6 h) apart. Measured: `|2459015.1208594847 −
    /// 2459015.498298313| ≈ 0.377 d` (9.06 h) — the 0.25 d threshold sits
    /// comfortably below that measurement, not at a round number chosen to
    /// paper over noise.
    ///
    /// The tz comparison above constrains slot SELECTION only — it says
    /// nothing about whether step (b) (scanning forward from the
    /// walked-back sunrise for the sunset) actually ran. Added below: both
    /// windows must run forwards, and their width must equal one eighth of
    /// the REAL (positively-ordered) daytime. Derived independently via the
    /// same `sun_rise_set` primitive `kalam_windows` uses internally (not by
    /// calling `kalam_windows` again): the walk-back at lon 165°E settles on
    /// day_start = JD 2459014.5 (measured: `sun_rise_set(2459015.5, 0.0,
    /// 165.0, 0.0, flat_sun).rise` = JD 2459016.0552225793, which is AFTER
    /// `jd`, so it is rejected and the walk steps back one civil UT day),
    /// giving sunrise = JD 2459015.057953013 and — scanning forward from
    /// THAT sunrise — sunset = JD 2459015.5612047845: a positive 12.078 h
    /// daytime, one eighth of which is 0.0629064714 d (90.59 min).
    #[test]
    fn kalam_windows_uses_the_observers_own_weekday_not_the_ut_one() {
        let jd = 2_459_015.75;
        let (rahu_correct, gulika_correct) =
            kalam_windows(jd, 0.0, 165.0, 0.0, 660, &flat_sun).expect("sun rises here");
        let (rahu_wrong, _) =
            kalam_windows(jd, 0.0, 165.0, 0.0, 0, &flat_sun).expect("sun rises here");
        assert!(
            (rahu_correct.start_jd - rahu_wrong.start_jd).abs() > 0.25,
            "dropping tz_offset_minutes must select a materially different slot: honoured {} vs dropped {}",
            rahu_correct.start_jd,
            rahu_wrong.start_jd
        );

        let sunrise =
            vedaksha_astro::riseset::sun_rise_set(2_459_014.5, 0.0, 165.0, 0.0, &flat_sun)
                .rise
                .expect("walked-back sunrise exists");
        let sunset = vedaksha_astro::riseset::sun_rise_set(sunrise, 0.0, 165.0, 0.0, &flat_sun)
            .set
            .expect("forward-scanned sunset exists");
        let eighth = (sunset - sunrise) / 8.0;
        assert!(
            eighth > 0.0,
            "sanity: independently-derived daytime must be positive, got {} h",
            (sunset - sunrise) * 24.0
        );

        for w in [rahu_correct, gulika_correct] {
            assert!(
                w.end_jd > w.start_jd,
                "window ran backwards: {} .. {}",
                w.start_jd,
                w.end_jd
            );
            assert!(
                ((w.end_jd - w.start_jd) - eighth).abs() < 1e-9,
                "window width {} != one eighth of the daytime {eighth}",
                w.end_jd - w.start_jd
            );
        }
    }

    /// MUTATION PIN for step (b) specifically. This is a different mutation
    /// from the one `kalam_windows_run_forwards_in_the_far_west` guards
    /// against: that test catches reverting the WHOLE two-step algorithm
    /// (walk-back + forward-scan) to one naive single `sun_rise_set` call.
    /// This one catches keeping the walk-back but taking the sunset from
    /// that same walked-back `day_start` window instead of from the sunrise
    /// instant — the review found this survives all 41 existing tests and
    /// drives the daytime to about −11.85 h at lon 165°/100°/45°/−175°. 165°
    /// is already used above and −157.8583° is used by the far-west test
    /// below, so this uses 45°E to keep the longitudes distinct.
    ///
    /// The `expected` closure below reimplements the *correct* step (b) —
    /// scanning forward from the independently re-derived sunrise via the
    /// `sun_rise_set` primitive directly, not via `kalam_windows` — so it
    /// only agrees with `kalam_windows`'s actual output when `kalam_windows`
    /// also does the correct scan. Measured directly: at jd 2459015.75, lon
    /// 45°E, the walk-back settles on day_start = JD 2459014.5 and sunrise =
    /// JD 2459015.390376202; scanning forward from that sunrise gives sunset
    /// = JD 2459015.8936279733 (12.078 h daytime). Taking `.set` from
    /// `sun_rise_set(2459014.5, ...)` instead (the mutation) gives JD
    /// 2459014.896358407, which is BEFORE sunrise — a −11.856 h "daytime",
    /// so under the mutation `eighth` goes negative and every window's
    /// `end_jd` falls before its `start_jd`, failing the ordering assertion
    /// outright.
    ///
    /// PROVEN by actually applying this mutation to `kalam_windows` (taking
    /// `.set` from the `day_start`-anchored call while keeping the
    /// walk-back) and re-running: this test FAILED (`rahu ran backwards:
    /// 2459015.328623978 .. 2459015.266871754`); reverting made it pass
    /// again. See the fix-pass report for the full failure output.
    #[test]
    fn kalam_windows_step_b_scans_forward_from_sunrise_not_day_start() {
        let jd = 2_459_015.75;
        let lon = 45.0;
        let tz = 180; // lon / 15 * 60 — nearest-hour civil offset for this longitude.

        let sunrise = vedaksha_astro::riseset::sun_rise_set(2_459_014.5, 0.0, lon, 0.0, &flat_sun)
            .rise
            .expect("walked-back sunrise exists");
        let sunset = vedaksha_astro::riseset::sun_rise_set(sunrise, 0.0, lon, 0.0, &flat_sun)
            .set
            .expect("forward-scanned sunset exists");
        let eighth = (sunset - sunrise) / 8.0;
        assert!(
            eighth > 0.0,
            "sanity: independently-derived daytime must be positive, got {} h",
            (sunset - sunrise) * 24.0
        );

        let vara = weekday_from_day_index(sunrise + f64::from(tz) / 1440.0);
        let expected = |slot: u8| {
            let start = sunrise + f64::from(slot - 1) * eighth;
            (start, start + eighth)
        };
        let (rahu_start, rahu_end) = expected(vara.rahu_kalam_slot());
        let (gulika_start, gulika_end) = expected(vara.gulika_kalam_slot());

        let (rahu, gulika) =
            kalam_windows(jd, 0.0, lon, 0.0, tz, &flat_sun).expect("sun rises here");

        assert!(
            rahu.end_jd > rahu.start_jd,
            "rahu ran backwards: {} .. {}",
            rahu.start_jd,
            rahu.end_jd
        );
        assert!(
            gulika.end_jd > gulika.start_jd,
            "gulika ran backwards: {} .. {}",
            gulika.start_jd,
            gulika.end_jd
        );
        assert!(
            (rahu.start_jd - rahu_start).abs() < 1e-9,
            "rahu.start_jd {} != independently-derived {rahu_start}",
            rahu.start_jd
        );
        assert!(
            (rahu.end_jd - rahu_end).abs() < 1e-9,
            "rahu.end_jd {} != independently-derived {rahu_end}",
            rahu.end_jd
        );
        assert!(
            (gulika.start_jd - gulika_start).abs() < 1e-9,
            "gulika.start_jd {} != independently-derived {gulika_start}",
            gulika.start_jd
        );
        assert!(
            (gulika.end_jd - gulika_end).abs() < 1e-9,
            "gulika.end_jd {} != independently-derived {gulika_end}",
            gulika.end_jd
        );
    }

    /// REGRESSION GUARD. `sun_rise_set` reports the first rise and first set
    /// in its scan window and does not order them. This is the exact
    /// longitude class that hid the original vara bug (see
    /// `vara_does_not_follow_the_ut_day_boundary_in_the_far_west`), so it
    /// gets a test here too.
    ///
    /// MUTATION-TESTING NOTE: at this exact `(jd, lon)`, the Sun is *below*
    /// the horizon at `day_start` (measured: `geometric_altitude_deg(0.0,
    /// 0.0, 2_459_015.5, 21.3069, -157.8583)` = -14.77°), so a naive single
    /// `sun_rise_set(day_start, ...)` call does not itself yield a
    /// set-before-rise pair here — it yields a *forward-ordered* pair that
    /// belongs to the wrong day (see below), not a negative-width one. A
    /// longitude sweep at this `day_start` (checked by hand while writing
    /// this test) confirms the swap-order failure mode is real for this
    /// `equatorial` fixture — it appears at, e.g., lon 10°..180° and
    /// -180°..-175° — just not at Honolulu's exact longitude on this exact
    /// date. So the ordering (`end > start`) and width (45-135 min)
    /// assertions below, taken alone, would NOT catch a regression to the
    /// naive single-call implementation here: verified by actually reverting
    /// `kalam_windows` to a naive single call and re-running this test, which
    /// stayed green under the two assertions below. The `end_jd <
    /// next_days_rise` assertion was added specifically because of that
    /// finding — see its own comment.
    #[test]
    fn kalam_windows_run_forwards_in_the_far_west() {
        let jd = 2_459_015.75; // 2020-06-15 06:00Z == 2020-06-14 20:00 Honolulu
        let (rahu, gulika) =
            kalam_windows(jd, 21.3069, -157.8583, 0.0, -600, &flat_sun).expect("sun rises here");
        assert!(
            rahu.end_jd > rahu.start_jd,
            "rahu kalam ran backwards: {} .. {}",
            rahu.start_jd,
            rahu.end_jd
        );
        assert!(
            gulika.end_jd > gulika.start_jd,
            "gulika kalam ran backwards: {} .. {}",
            gulika.start_jd,
            gulika.end_jd
        );
        // "at this latitude and season" was a false premise: `flat_sun` fixes
        // the Sun's RA/dec at (0°, 0°), so this fixture has no seasons at
        // all, and daytime length is NOT latitude-independent either — it is
        // set by the fixed −50′ standard-altitude threshold interacting with
        // latitude (measured: 12.078 h at lat 0°, 12.086 h at lat 21.3069°,
        // 12.145 h at lat 51.5°, 12.101 h at lat −33.9°, all via
        // `sun_rise_set(2459015.5, lat, 0.0, 0.0, flat_sun)`). For this exact
        // call (jd 2459015.75, lat 21.3069°, lon −157.8583°) the measured
        // daytime is 12.086 h, so one eighth is ≈90.65 min; 45-135 min below
        // is kept as a generous sanity bound, not a tight prediction.
        let width_minutes = (rahu.end_jd - rahu.start_jd) * 1440.0;
        assert!(
            (45.0..135.0).contains(&width_minutes),
            "one eighth of the daytime = {width_minutes} min, which is not a plausible daytime"
        );

        // The two assertions above pass even for a naive single-call
        // implementation here (see the doc comment), because that naive call
        // happens to pick the *following* day's sunrise/sunset for this
        // longitude — `vara_at`'s walk-back rejects that same rise (it is
        // after `jd`) for exactly this reason; see
        // `vara_does_not_follow_the_ut_day_boundary_in_the_far_west`, which
        // derives it as JD 2459015.952163512. Independently re-derived here,
        // via the same `sun_rise_set` primitive `kalam_windows` uses
        // internally (not by calling `kalam_windows` again, so this is not
        // checking the function against itself): the vara's own daytime must
        // end strictly before that following sunrise. Rahu is slot 8 for
        // Sunday (the vara here), so `rahu.end_jd` is exactly this daytime's
        // sunset.
        let next_days_rise = vedaksha_astro::riseset::sun_rise_set(
            libm::floor(jd - 0.5) + 0.5,
            21.3069,
            -157.8583,
            0.0,
            &flat_sun,
        )
        .rise
        .expect("the following sunrise must exist here too");
        assert!(
            rahu.end_jd < next_days_rise,
            "rahu kalam (ending {}) must fall before the FOLLOWING sunrise ({next_days_rise}); a naive single-window sun_rise_set call wrongly selects that later cycle's day here",
            rahu.end_jd
        );
    }

    /// Polar night has no daytime to divide into eighths.
    #[test]
    fn no_kalam_windows_where_the_sun_does_not_rise() {
        let southern_sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        assert!(kalam_windows(2_451_544.5, 78.22, 15.65, 0.0, 60, &southern_sun).is_none());
    }

    /// Exercises `elevation_m` end-to-end, and is built to actually
    /// DISCRIMINATE between `kalam_windows` deriving its vara from its own
    /// elevation-aware sunrise vs. delegating to `vara_at` (which hardcodes
    /// elevation 0.0). An earlier version of this test called
    /// `kalam_windows` at `day_start + 0.5` — well after BOTH the
    /// elevation-adjusted and the sea-level sunrise on that day — so both
    /// derivations landed on the same day's sunrise and hence the same
    /// weekday; verified by actually reverting the `vara` line in
    /// `kalam_windows` to `vara_at(jd_ut, lat_deg, lon_deg_east,
    /// tz_offset_minutes, equatorial)` and re-running the old test, which
    /// still passed. See the fix-pass report for that failure-to-fail
    /// evidence.
    ///
    /// The two derivations disagree only when the query instant falls
    /// strictly BETWEEN the elevation-adjusted sunrise and the sea-level
    /// sunrise of the *same* civil-UT day. Walking through why, from
    /// `kalam_windows`'s own walk-back loop: inside that window the
    /// elevation-aware walk-back has already found a sunrise ≤ `jd_ut` on
    /// that day (the vara has turned), while the sea-level walk-back
    /// (`vara_at`) finds that same day's sea-level rise is still AFTER
    /// `jd_ut`, rejects it (its `if r <= jd_ut` guard fails), steps
    /// `day_start` back one civil day, and returns the PREVIOUS day's vara
    /// instead.
    ///
    /// All figures below derived directly via `sun_rise_set`/`vara_at`
    /// while writing this test, not restated from a plan: at lat 0°, lon 0°,
    /// `day_start` = JD 2_451_544.5 (2000-01-01 00:00 UT, a Saturday —
    /// J2000.0 = JD 2451545.0 = Saturday per `weekday_j2000_is_saturday`
    /// above).
    /// - `sun_rise_set(day_start, 0.0, 0.0, 0.0, flat_sun).rise` (sea level)
    ///   = JD 2451544.9687135713
    /// - `sun_rise_set(day_start, 0.0, 0.0, 15_000.0, flat_sun).rise`
    ///   (elevated) = JD 2451544.9587727264 — 14.3148 min EARLIER. 15 km is
    ///   deliberately far past any inhabited elevation: at the 2000 m used
    ///   elsewhere in this file the gap is only ~5.2 min, too tight to leave
    ///   comfortable margin at the query instant below; going higher widens
    ///   it, per the task's own suggestion.
    /// - the query instant is the gap's midpoint, JD 2451544.9637431488 —
    ///   ~7.16 min inside each edge.
    /// - `weekday_from_day_index(elevated_rise)` = Saturday (gulika slot 1,
    ///   starts exactly at sunrise).
    /// - `vara_at(mid, 0.0, 0.0, 0, flat_sun)` = Friday (gulika slot 2,
    ///   starts one eighth-daytime AFTER sunrise) — confirmed by direct
    ///   call, not hand-derived: `vara_at` rejects the same-day sea-level
    ///   rise (after `mid`) and falls back to the previous day's sea-level
    ///   rise, JD 2451543.9714440051 (Dec 31, 1999, a Friday).
    #[test]
    fn kalam_windows_selects_the_elevation_aware_vara_not_the_sea_level_one() {
        let day_start = 2_451_544.5; // 2000-01-01 00:00 UT, a Saturday.
        let elevation_m = 15_000.0;

        let sea_level_rise =
            vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, 0.0, &flat_sun)
                .rise
                .expect("sun rises at sea level too");
        let elevated_rise =
            vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, elevation_m, &flat_sun)
                .rise
                .expect("sun rises with elevation too");
        assert!(
            elevated_rise < sea_level_rise,
            "elevation must move the rise EARLIER: sea={sea_level_rise} elevated={elevated_rise}"
        );
        let gap_minutes = (sea_level_rise - elevated_rise) * 1440.0;
        assert!(
            gap_minutes > 10.0,
            "gap must be comfortably wide (>10 min) so the query instant below \
             sits clear of both edges, got {gap_minutes} min"
        );

        // Strictly inside the gap, equidistant from both edges.
        let jd_ut = (sea_level_rise + elevated_rise) / 2.0;
        assert!(
            elevated_rise < jd_ut && jd_ut < sea_level_rise,
            "query instant {jd_ut} must sit strictly inside the gap \
             ({elevated_rise}..{sea_level_rise})"
        );

        // The elevation-aware vara: what `kalam_windows` must use.
        // `weekday_from_day_index` is the same private helper both `vara_at`
        // and `kalam_windows` apply to whichever sunrise they land on — this
        // does not reimplement `kalam_windows`'s walk-back, it just names
        // the weekday of the sunrise instant already derived above.
        let elevation_vara = weekday_from_day_index(elevated_rise);
        // The sea-level vara: `vara_at` called DIRECTLY, the actual "wrong"
        // derivation a reverted `vara` line would substitute in.
        let sea_vara = vara_at(jd_ut, 0.0, 0.0, 0, &flat_sun);
        assert_ne!(
            elevation_vara, sea_vara,
            "the two derivations must disagree at this instant, or the \
             assertion below cannot discriminate between them: \
             elevation-aware sunrise {elevated_rise} -> {elevation_vara:?}; \
             vara_at({jd_ut}) (sea level) -> {sea_vara:?}"
        );
        // Not just different names — different gulika slots, so the two
        // derivations place the window at physically different instants.
        assert_ne!(
            elevation_vara.gulika_kalam_slot(),
            sea_vara.gulika_kalam_slot(),
            "sanity: {elevation_vara:?} and {sea_vara:?} must not share a \
             gulika slot, or this test cannot distinguish them positionally"
        );

        let (_, gulika) =
            kalam_windows(jd_ut, 0.0, 0.0, elevation_m, 0, &flat_sun).expect("sun rises here");

        // This is the fix's actual contract: THE SLOT `kalam_windows`
        // SELECTS must be the elevation-aware vara's slot, anchored at the
        // elevation-aware sunrise — not the sea-level vara's slot.
        assert!(
            (gulika.start_jd - elevated_rise).abs() < 1e-9,
            "kalam_windows must select the ELEVATION-AWARE vara's slot: \
             elevation-aware sunrise {elevated_rise} -> {elevation_vara:?} \
             (gulika slot {}), sea-level vara_at({jd_ut}) -> {sea_vara:?} \
             (gulika slot {}) — got gulika.start_jd = {}",
            elevation_vara.gulika_kalam_slot(),
            sea_vara.gulika_kalam_slot(),
            gulika.start_jd
        );
        assert!(
            (gulika.start_jd - sea_level_rise).abs() > 1e-4,
            "this would also pass by coincidence if elevation_m were ignored \
             entirely for the anchor — it must differ from the sea-level rise"
        );
    }

    #[test]
    fn tithi_nakshatra_end_real_ephemeris() {
        use vedaksha_astro::sidereal::{Ayanamsha, tropical_to_sidereal};
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates::apparent_position;

        let provider = AnalyticalProvider::new();
        let jd = 2_460_676.5;

        // Elongation is frame-agnostic, so tropical (lon, daily-motion) is fine.
        let moon_trop = |t: f64| {
            apparent_position(&provider, Body::Moon, t)
                .ok()
                .map(|p| (p.ecliptic.longitude.to_degrees(), p.longitude_speed))
        };
        let sun_trop = |t: f64| {
            apparent_position(&provider, Body::Sun, t)
                .ok()
                .map(|p| (p.ecliptic.longitude.to_degrees(), p.longitude_speed))
        };

        let t_end = compute_tithi_end(jd, &moon_trop, &sun_trop).expect("tithi end");
        // A tithi lasts ≈ 20–27 h, so the boundary is < ~1.13 days ahead.
        assert!(
            t_end > jd && t_end - jd < 1.2,
            "tithi end {} days away",
            t_end - jd
        );
        // The end instant must land exactly on a 12° elongation boundary.
        let (me, _) = moon_trop(t_end).unwrap();
        let (se, _) = sun_trop(t_end).unwrap();
        let elong = (me - se).rem_euclid(360.0);
        let off = (elong / 12.0 - (elong / 12.0).round()).abs() * 12.0;
        assert!(
            off < 1e-4,
            "elongation {elong}° not on a 12° boundary (off {off}°)"
        );

        // Nakshatra uses the sidereal Moon longitude.
        let moon_sid = |t: f64| {
            apparent_position(&provider, Body::Moon, t).ok().map(|p| {
                let trop = p.ecliptic.longitude.to_degrees();
                (
                    tropical_to_sidereal(trop, Ayanamsha::Lahiri, t),
                    p.longitude_speed,
                )
            })
        };
        let n_end = compute_nakshatra_end(jd, &moon_sid).expect("nakshatra end");
        assert!(
            n_end > jd && n_end - jd < 1.2,
            "naksh end {} days away",
            n_end - jd
        );
        let (ms, _) = moon_sid(n_end).unwrap();
        let span = 360.0 / 27.0;
        let off = (ms / span - (ms / span).round()).abs() * span;
        assert!(
            off < 1e-4,
            "sidereal Moon {ms}° not on a nakshatra boundary (off {off}°)"
        );
    }
}
