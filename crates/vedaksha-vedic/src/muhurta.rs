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
/// * `jd` — Julian Day
/// * `moon_sidereal_lon` — Moon's sidereal longitude in degrees
/// * `sun_sidereal_lon` — Sun's sidereal longitude in degrees
#[must_use]
pub fn assess_muhurta(jd: f64, moon_sidereal_lon: f64, sun_sidereal_lon: f64) -> MuhurtaAssessment {
    let nakshatra = Nakshatra::from_longitude(moon_sidereal_lon);
    let tithi = compute_tithi(moon_sidereal_lon, sun_sidereal_lon);
    let weekday = ut_weekday_from_jd(jd);

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
/// * `get_moon_lon` — callback returning Moon sidereal longitude at JD
/// * `get_sun_lon` — callback returning Sun sidereal longitude at JD
/// * `min_quality` — minimum quality score (0.0-1.0) to include
#[must_use]
pub fn search_muhurta(
    start_jd: f64,
    end_jd: f64,
    get_moon_lon: &dyn Fn(f64) -> Option<f64>,
    get_sun_lon: &dyn Fn(f64) -> Option<f64>,
    min_quality: f64,
) -> Vec<MuhurtaAssessment> {
    let mut results = Vec::new();
    let mut jd = start_jd;
    let step = 0.5; // check every half day

    while jd <= end_jd {
        if let (Some(moon), Some(sun)) = (get_moon_lon(jd), get_sun_lon(jd)) {
            let assessment = assess_muhurta(jd, moon, sun);
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
        let assessment = assess_muhurta(2_451_545.0, 94.0, 0.0);
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
        let assessment = assess_muhurta(2_451_545.0, 348.0, 0.0);
        assert_eq!(assessment.tithi.number, 30, "Should be Amavasya");
        assert!(
            assessment.factors.iter().any(|f| f.contains("Amavasya")),
            "Expected an Amavasya factor"
        );
    }

    #[test]
    fn inauspicious_nakshatra_reduces_score() {
        // Ardra: index 5 → lon ≈ 5*13.333 = 66.666°
        let assessment = assess_muhurta(2_451_545.0, 67.0, 0.0);
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
        let assessment = assess_muhurta(2_451_545.0, 67.0, 0.0);
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
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
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
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            0.0,
        );
        let high_threshold = search_muhurta(
            2_451_545.0,
            2_451_555.0,
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            0.99,
        );
        assert!(
            high_threshold.len() <= low_threshold.len(),
            "Higher threshold should yield fewer or equal results"
        );
    }

    #[test]
    fn search_returns_empty_when_callback_returns_none() {
        let results = search_muhurta(2_451_545.0, 2_451_550.0, &|_| None, &|_| None, 0.0);
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
            &|_| Some(94.0),
            &|_jd| Some(64.0),
            0.0,
        )
        .len();
        // Verify count via the returned vec length (3 steps: 0, 0.5, 1.0)
        let results = search_muhurta(
            2_451_545.0,
            2_451_546.0,
            &|_| Some(94.0),
            &|_| Some(64.0),
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
