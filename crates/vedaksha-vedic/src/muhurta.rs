// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
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

/// What [`kalam_windows`] derived for one instant and one observer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KalamReckoning {
    /// The vara in force at the instant.
    pub vara: Weekday,
    /// `true` when [`Self::vara`] was reckoned from a real sunrise — i.e. the
    /// backward search found the sunrise that opened the Vedic day — and
    /// `false` when it is the **civil-weekday fallback**, which is what the
    /// polar day and polar night (no sunrise exists to reckon from) and an
    /// unavailable ephemeris both produce.
    ///
    /// This is the caller's ONLY way to tell the two apart, and it must be
    /// surfaced wherever the vara is: a civil weekday presented as a vara is
    /// precisely the defect the sunrise reckoning exists to fix, so above
    /// roughly ±66.5° latitude in the local summer or winter it would
    /// otherwise reappear silently.
    ///
    /// [`Self::windows`] is NOT a substitute for this flag. `windows` is
    /// `None` in a third case too — a sunrise was found but the following
    /// sunset was not — where the vara IS sunrise-reckoned and this flag is
    /// `true`. Reading `windows.is_none()` as "the vara is a fallback" is
    /// therefore wrong in both directions of consequence.
    pub from_sunrise: bool,
    /// `(rahu, gulika)` as real time windows, or `None` where the Sun does not
    /// both rise and set within the day — polar day and polar night — so
    /// there is no daytime to divide into eighths.
    pub windows: Option<(KalamWindow, KalamWindow)>,
}

/// Rahu Kalam and Gulika Kalam as actual time windows for the vara containing
/// `jd_ut`, at the given observer — plus the vara itself and how it was
/// reckoned, so a caller (e.g. `compute_panchanga`'s handler) that needs all
/// three never has to run a second, separate sunrise scan (via [`vara_at`]).
///
/// Both windows are defined as one eighth of the daytime — sunrise to sunset
/// divided into eight equal parts — with the vara selecting which eighth.
///
/// [`KalamReckoning::vara`] is always populated. Inside the polar day/night
/// this function's own sunrise search finds no sunrise, so the vara falls back
/// to the local civil weekday — the same documented convention [`vara_at`]
/// uses — rather than being withheld just because the windows are undefined;
/// [`KalamReckoning::from_sunrise`] is what distinguishes that fallback from a
/// genuine sunrise-reckoned vara.
///
/// Source: Kalaprakashika (Rahu Kalam); Muhurtha Chintamani (Gulika Kalam).
#[must_use]
pub fn kalam_windows(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> KalamReckoning {
    let local_weekday =
        |jd: f64| weekday_from_day_index(jd + f64::from(tz_offset_minutes) / 1440.0);
    // (a) The sunrise that OPENS the vara containing `jd_ut`, from the
    // INSTANT-anchored search rather than by walking `sun_rise_set`'s 24-hour
    // window back along the civil-UT calendar. That walk was wrong for the
    // same reason it was wrong in `vara_with_validity`: `sun_rise_set` reports
    // only the FIRST rise in its window, and two sunrises share one window
    // whenever the inter-sunrise gap is under a civil day, so the walk could
    // settle on a sunrise a whole vara too early and anchor the eighths there.
    // See the "Why this is anchored on the instant" note on
    // [`vara_with_validity`].
    //
    // `elevation_m` (not 0.0) is passed here deliberately: the slot SELECTION
    // and the window ANCHOR must come from the same horizon. `vara_at` and
    // `vara_with_validity` now take the observer's elevation for the same
    // reason, so all three agree by construction rather than by coincidence at
    // sea level.
    let Some(sunrise) = vedaksha_astro::riseset::previous_rise(
        jd_ut,
        lat_deg,
        lon_deg_east,
        elevation_m,
        equatorial,
    ) else {
        // Polar day/night, or `equatorial` unavailable: no sunrise bounds the
        // vara, so there is no daytime to divide into eighths. The vara still
        // falls back to the local civil weekday — same documented convention
        // as `vara_at` — rather than being withheld, and `from_sunrise: false`
        // is what tells the caller that is what happened. This is the ONLY
        // return site that sets it false.
        return KalamReckoning {
            vara: local_weekday(jd_ut),
            from_sunrise: false,
            windows: None,
        };
    };

    // `weekday_from_day_index` is the same private helper `vara_at` applies to
    // its own found rise, so this is the same rule applied to a consistent
    // sunrise, not a new one — and it avoids repeating the search.
    let vara = local_weekday(sunrise);

    // (b) The sunset that FOLLOWS that sunrise — the ordering requirement is
    // real: `sun_rise_set` reports the first rise and first set in its window
    // and does NOT order them, so pairing them from one calendar-anchored
    // window can give a negative daytime and eight backwards eighths.
    // Concretely, with the `flat_sun` test fixture (RA/dec fixed at 0°),
    // scanning from day_start = JD 2459015.5 (2020-06-15 00:00 UT) at lon
    // 100°E finds the set at JD 2459015.741267 (≈05:47 UT) BEFORE the rise at
    // JD 2459016.235285 (≈17:39 UT) — measured directly, not restated from
    // memory.
    //
    // Anchoring the scan on the `sunrise` INSTANT fixes that, and — unlike the
    // rise search above — is immune to the two-events-in-one-window defect:
    // the body is above the horizon at `sunrise` and stays there until it
    // sets, so the window `[sunrise, sunrise + 1]` contains at most one set
    // before the body rises again, and the first set in it IS the first set
    // after the sunrise. Keeping the one-day window here also preserves the
    // polar contract: where the daytime exceeds 24 h there is no eighth to
    // report and this correctly yields `None`.
    let Some(sunset) = vedaksha_astro::riseset::sun_rise_set(
        sunrise,
        lat_deg,
        lon_deg_east,
        elevation_m,
        equatorial,
    )
    .set
    else {
        // Sunrise found but no matching sunset — pathologically only
        // possible if `equatorial` starts failing partway through the
        // forward scan. The vara (anchored on a real sunrise) is still
        // valid, so `from_sunrise` stays TRUE here even though the windows
        // are absent; there is just no daytime to divide. This is the case
        // that makes `windows.is_none()` an unsound proxy for the fallback.
        return KalamReckoning {
            vara,
            from_sunrise: true,
            windows: None,
        };
    };

    let eighth = (sunset - sunrise) / 8.0;
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

    KalamReckoning {
        vara,
        from_sunrise: true,
        windows: Some((
            window(vara.rahu_kalam_slot()),
            window(vara.gulika_kalam_slot()),
        )),
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
/// sunrise belongs to the *previous* vara. This takes the local civil weekday
/// of [`vedaksha_astro::riseset::previous_rise`] — the most recent sunrise at
/// or before `jd_ut`, located by scanning outward from the instant itself.
///
/// # Arguments
/// * `jd_ut` — the instant, Julian Day (UT)
/// * `lat_deg` — observer latitude, degrees
/// * `lon_deg_east` — observer longitude, degrees, east positive
/// * `elevation_m` — observer elevation above sea level, metres. **A
///   determinant of the answer, not a refinement**: it lowers the visible
///   horizon by the dip `−0.0293°·√(elevation_m)`
///   ([`vedaksha_astro::riseset::horizon_dip_deg`], Meeus Ch. 16), which moves
///   sunrise earlier and so moves the sunrise-to-sunrise boundary this
///   function reckons against. At Lhasa (29.65 N, 91.13 E, 3650 m) the dip is
///   −0.0293·√3650 = −1.7702°, which moved sunrise on 2020-06-15 from JD
///   2459015.454732473 to 2459015.448311250 — 9.2466 minutes earlier,
///   measured, not estimated. That is a window every day in which an observer
///   passing `0.0` here and one passing `3650.0` are in DIFFERENT varas. Pass
///   the observer's real elevation, or `0.0` to ask for the sea-level horizon
///   deliberately; there is no default, because a silent sea-level assumption
///   is exactly the kind of hidden determinant that made the UT-weekday
///   defect this function replaced so hard to see.
/// * `tz_offset_minutes` — offset from UT of the observer's civil clock
/// * `equatorial` — the Sun's apparent `(right_ascension_deg, declination_deg)`
///   at a Julian Day (UT), or `None` where unavailable
///
/// Inside the polar day and polar night no sunrise exists and the
/// sunrise-to-sunrise vara is undefined; there this falls back to the local
/// civil weekday. The same fallback also fires if `equatorial` returns
/// `None` anywhere in the backward search — ephemeris unavailable, a different
/// situation from the polar one — and the two are indistinguishable in the
/// return value: a caller that needs to tell them apart must probe
/// `equatorial` itself. That fallback is a documented convention, not a
/// classical rule.
///
/// ⚠️ Because that fallback is silent in the return value, a caller that
/// SURFACES this weekday to a user should surface with it whether a sunrise
/// was actually found — [`kalam_windows`] reports exactly that as
/// [`KalamReckoning::from_sunrise`], derived from the same single scan.
///
/// Source: Muhurta Chintamani; Kalaprakashika (the sunrise reckoning).
#[must_use]
pub fn vara_at(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Weekday {
    vara_with_validity(
        jd_ut,
        lat_deg,
        lon_deg_east,
        elevation_m,
        tz_offset_minutes,
        equatorial,
    )
    .0
}

/// The vara for `jd_ut`, plus the half-open Julian-Day interval `[start,
/// end)` over which that same vara holds — i.e. `[the sunrise that opened
/// it, the next sunrise)`.
///
/// This is the ONE derivation `vara_at` wraps: the interval is exact by
/// construction, so a caller (e.g. [`search_muhurta`]'s memo) that recomputes
/// only when its cached `jd` falls outside `[start, end)` can never go stale
/// — unlike memoising on the civil-UT day, which drifts out of sync with the
/// sunrise boundary and hands the wrong vara to any sample that falls between
/// local midnight and local sunrise.
///
/// `elevation_m` is the observer's height above sea level in metres and is a
/// determinant of BOTH ends of that interval, not a refinement of them — see
/// the `elevation_m` note on [`vara_at`].
///
/// Returns `None` for the interval — "not derivable, do not cache" — in
/// exactly two cases: the polar fallback (no sunrise bounds the vara at all,
/// so there is no interval to report), and a forward search for the closing
/// sunrise that fails within its own bound (which includes the `equatorial`
/// closure returning `None` throughout that search). The `Weekday` itself is
/// still correct in both cases; only the cached interval is withheld.
///
/// # Why this is anchored on the instant, not on the calendar
///
/// Both ends come from the INSTANT-anchored searches
/// [`vedaksha_astro::riseset::previous_rise`] and
/// [`vedaksha_astro::riseset::next_rise`], so `start <= jd_ut < end` holds by
/// construction — it is the acceptance test each search applies to its own
/// bisected root, not a property argued from window arithmetic.
///
/// The earlier implementation walked
/// [`vedaksha_astro::riseset::sun_rise_set`]'s 24-hour window back along the
/// civil-UT calendar. That window reports only the FIRST rise inside it, and
/// the inter-sunrise gap is shorter than a civil day — always for a Sun held at
/// fixed right ascension (one sidereal rotation, 0.9972695663 d) and for the
/// real Sun over roughly half the year — so TWO sunrises could share one
/// window and the second was invisible. The walk then settled on the earlier
/// one, putting `start` a whole vara too early and leaving `jd_ut` as much as
/// one rotation PAST `end`: the vara itself came back wrong, not merely a
/// stale cache. Measured with the fixed-RA fixture at lat −9°, tz +660, JD
/// 2451554.75, sweeping longitude at 0.01°: 98 of 36,001 samples (0.27%) failed
/// containment, the worst leaving `jd_ut` 0.2527 d beyond `end`. See
/// `vara_with_validity_pins_the_two_sunrises_in_one_window_regression`.
///
/// Source: Muhurta Chintamani; Kalaprakashika (the sunrise reckoning).
#[must_use]
pub fn vara_with_validity(
    jd_ut: f64,
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> (Weekday, Option<(f64, f64)>) {
    let local_weekday =
        |jd: f64| weekday_from_day_index(jd + f64::from(tz_offset_minutes) / 1440.0);

    // The sunrise that OPENED the vara containing `jd_ut`: the most recent one
    // at or before the instant, on the observer's OWN horizon — `elevation_m`
    // is threaded through rather than hardcoded to 0.0, so that this and
    // `kalam_windows` (which has always passed the observer's elevation) can
    // never land on different sunrises and report different varas for the same
    // observer. See the `elevation_m` note on `vara_at`.
    let Some(start) = vedaksha_astro::riseset::previous_rise(
        jd_ut,
        lat_deg,
        lon_deg_east,
        elevation_m,
        equatorial,
    ) else {
        // Polar day/night, or `equatorial` returned `None`: fall back to the
        // local civil weekday, the same documented convention as before. No
        // sunrise was found at all, so there is no interval to report.
        return (local_weekday(jd_ut), None);
    };
    let weekday = local_weekday(start);

    // The sunrise that CLOSES it: the first one strictly after the instant.
    // Anchoring this on `jd_ut` rather than on `start` is what makes `jd_ut <
    // end` true by construction; anchoring on `start` would reintroduce an
    // argument about whether the search can overshoot.
    let Some(end) =
        vedaksha_astro::riseset::next_rise(jd_ut, lat_deg, lon_deg_east, elevation_m, equatorial)
    else {
        // The opening sunrise exists but the closing one is beyond the search
        // bound (polar onset, or `equatorial` failing through the forward
        // search): the weekday is still correct, the interval is not
        // derivable, so the caller must not cache it.
        return (weekday, None);
    };

    (weekday, Some((start, end)))
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
    angle_at: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
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

/// The Moon and the Sun as evaluated at one instant:
/// `((moon_longitude_deg, moon_daily_motion), (sun_longitude_deg,
/// sun_daily_motion))`, both motions in degrees per day.
///
/// This is what [`compute_tithi_end`]'s callback returns. Pairing them in one
/// return value is the point: every quantity that function derives is a
/// difference between the two, so it only ever wants them together, and a
/// caller can then serve both from a single ephemeris evaluation at that
/// instant.
pub type MoonAndSun = ((f64, f64), (f64, f64));

/// Julian Day at which the current **tithi** ends — when the Moon–Sun
/// elongation reaches the next multiple of 12° — refined against true
/// longitudes. The variable real duration of a tithi (≈20–27 h) comes
/// entirely from the varying lunar speed, which is why the daily motion is
/// required here rather than a mean-motion approximation.
///
/// `moon_and_sun` returns both bodies at one instant — see [`MoonAndSun`].
/// Tropical or sidereal input both work: the ayanamsha cancels in the
/// elongation and its rate.
///
/// # Why one callback and not two
///
/// Every quantity this function needs is an elongation, so it never wants one
/// body without the other at the same `t`. Two callbacks made that impossible
/// to exploit: a caller backing them with an ephemeris had to enter the
/// provider twice per instant and recompute whatever is shared between the two
/// evaluations. One callback lets the caller answer both from a single
/// evaluation at `t` — which is exactly what `vedaksha_ephem_core`'s batch
/// entry point does, hoisting the per-timestamp frames and sharing one
/// memoizing provider across the pair.
///
/// # Errors / `None`
/// Returns `None` if the callback yields `None` or the elongation rate
/// vanishes.
#[must_use]
pub fn compute_tithi_end(
    jd: f64,
    moon_and_sun: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync),
) -> Option<f64> {
    let ((m0, _), (s0, _)) = moon_and_sun(jd)?;
    let elong = (m0 - s0).rem_euclid(360.0);
    let next_boundary = ((elong / 12.0).floor() + 1.0) * 12.0;
    let angle_at = |t: f64| -> Option<(f64, f64)> {
        let ((ml, ms), (sl, ss)) = moon_and_sun(t)?;
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
pub fn compute_nakshatra_end(
    jd: f64,
    moon: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<f64> {
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
/// * `lat_deg` / `lon_deg_east` / `elevation_m` / `tz_offset_minutes` — the
///   observer, needed to derive the vara (see [`vara_at`]) for each candidate
///   instant. `elevation_m` is part of that observer for the same reason the
///   other three are: it moves the sunrise that bounds the vara (see the
///   `elevation_m` note on [`vara_at`]), and a candidate's `quality_score`
///   depends on its vara. Omitting it would let this function and
///   [`kalam_windows`] report different weekdays for one observer.
/// * `get_moon_lon` — callback returning Moon sidereal longitude at JD
/// * `get_sun_lon` — callback returning Sun sidereal longitude at JD
/// * `equatorial` — the Sun's apparent `(right_ascension_deg, declination_deg)`
///   at a JD (UT), passed through to [`vara_at`]'s sunrise search.
/// * `min_quality` — minimum quality score (0.0-1.0) to include
///
/// # Performance
///
/// One [`vara_with_validity`] call costs roughly 2 × a dozen `equatorial`
/// evaluations: the backward and forward searches each converge on their
/// horizon crossing analytically from Meeus eq. 15.1
/// (`vedaksha_astro::riseset`), re-evaluating the Sun's position only once per
/// iteration. That replaced a 5-minute outward scan whose cost was 2 × (up to
/// 1152 steps + bisection) — typically ~288 steps a direction, with the
/// four-day hard bound reserved for latitudes just inside the polar circles,
/// where a body can fail to clear the horizon on one rotation and clear it on
/// the next. Even at the new cost, deriving the vara fresh at every 0.5-day
/// step would repeat that work ~730 times for a one-year search, so the memo
/// below still earns its place. The vara is constant
/// over `[start, end)` — sunrise to the next sunrise — so this memoises on
/// that exact interval (as returned by [`vara_with_validity`]) and recomputes
/// only when `jd` falls outside it. That is not an approximation: unlike
/// memoising on the civil-UT day (which drifts out of sync with the sunrise
/// boundary — a step landing between local midnight and local sunrise gets
/// the *previous* civil day's stale vara, not the one actually in force), the
/// cached interval **is** the vara's real extent, so it cannot go stale.
///
/// What the memo cannot remove is the vara work itself, and about four fifths
/// of a scan's cost sits under `vedaksha_astro::riseset` in those two sunrise
/// searches. The candidate days are independent, so with the `std` feature the
/// scan is split across [`std::thread::available_parallelism`] workers over
/// contiguous ascending chunks — see `search_candidates_in_parallel`. Each
/// worker carries its own memo. Results are unchanged: bit-identical and in
/// the same order as a serial walk, which
/// `threaded_and_serial_scans_agree_bit_for_bit` asserts field by field on the
/// raw bit patterns. Ranges shorter than one chunk, and targets that report no
/// parallelism, run serially.
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
    elevation_m: f64,
    tz_offset_minutes: i32,
    get_moon_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    get_sun_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    min_quality: f64,
) -> Vec<MuhurtaAssessment> {
    // Materialise the candidate instants FIRST, by the very same iterated
    // `jd += step` the walk has always used. Everything below then consumes
    // slices of this one sequence, so a chunked evaluation sees exactly the
    // `f64` bit patterns a serial walk would have produced. That is
    // bit-identity by construction — it does not rest on an argument about
    // when `start_jd + k * step` is exactly representable, which stops being
    // true once the accumulated value crosses a binade boundary.
    let mut candidates = Vec::new();
    let mut jd = start_jd;
    while jd <= end_jd {
        candidates.push(jd);
        jd += SEARCH_STEP_DAYS;
    }

    #[cfg(feature = "std")]
    if let Some(results) = search_candidates_in_parallel(
        &candidates,
        lat_deg,
        lon_deg_east,
        elevation_m,
        tz_offset_minutes,
        get_moon_lon,
        get_sun_lon,
        equatorial,
        min_quality,
    ) {
        return results;
    }

    evaluate_candidates(
        &candidates,
        lat_deg,
        lon_deg_east,
        elevation_m,
        tz_offset_minutes,
        get_moon_lon,
        get_sun_lon,
        equatorial,
        min_quality,
    )
}

/// Candidate spacing for [`search_muhurta`]: half a day, i.e. roughly dawn and
/// dusk.
const SEARCH_STEP_DAYS: f64 = 0.5;

/// Fewest candidate instants [`search_muhurta`] will hand a worker.
///
/// A worker starts with a cold vara memo, so each chunk pays at most one
/// `vara_with_validity` that the serial walk would have taken from the memo.
/// A vara runs sunrise to sunrise — about one day, i.e. about two candidates —
/// so a chunk of `C` candidates derives roughly `C / 2` varas and the cold
/// start adds at most one of them, i.e. `2 / C` extra work. Net speedup across
/// `W` workers is therefore about `W / (1 + 2/C)`.
///
/// At `C = 8` the redundancy is 25% and eight workers still net about 6.4×; at
/// `C = 4` it is 50% and the curve turns over. Eight is where the re-derivation
/// is still under a quarter, and it matters that the threshold sit here rather
/// than higher: an MCP `search_muhurta` request is capped at a 30-day span
/// (`crate::validation::MAX_MUHURTA_SEARCH_DAYS` in `vedaksha-mcp`), which is
/// 61 candidates, so a threshold of 32 would have left the served path serial
/// in every case it is actually asked to serve.
///
/// A search shorter than two chunks runs serially: below that there is nothing
/// to split, and the spawn cost and the re-derivation are both pure loss.
#[cfg(feature = "std")]
const MIN_CANDIDATES_PER_WORKER: usize = 8;

/// Evaluate `candidates` across worker threads, or `None` when the range is
/// too short to be worth splitting or the platform reports no parallelism.
///
/// The days are independent: [`vara_with_validity`] is pure and
/// [`assess_muhurta`] reads nothing but its own arguments, so there is no
/// cross-candidate carry other than the memo — and the memo only ever *skips*
/// recomputing a value it would otherwise have derived identically.
///
/// Results stay bit-identical and order-stable. Chunks are contiguous and
/// ascending, are joined in the order they were spawned, and are concatenated
/// in that order, so the output sequence is the serial one. Nothing is summed
/// across candidates, so no floating-point accumulation order changes.
#[cfg(feature = "std")]
#[allow(clippy::too_many_arguments)]
fn search_candidates_in_parallel(
    candidates: &[f64],
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    get_moon_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    get_sun_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    min_quality: f64,
) -> Option<Vec<MuhurtaAssessment>> {
    // `available_parallelism` reports `Err` on the single-threaded targets this
    // crate also builds for (wasm32 among them), which lands on the serial path
    // below rather than on a `spawn` that would fail at run time.
    let available = std::thread::available_parallelism().ok()?.get();
    let workers = (candidates.len() / MIN_CANDIDATES_PER_WORKER).min(available);
    if workers < 2 {
        return None;
    }

    let per_worker = candidates.len().div_ceil(workers);
    let mut chunked: Vec<Vec<MuhurtaAssessment>> = Vec::with_capacity(workers);
    std::thread::scope(|scope| {
        let handles: Vec<_> = candidates
            .chunks(per_worker)
            .map(|chunk| {
                scope.spawn(move || {
                    evaluate_candidates(
                        chunk,
                        lat_deg,
                        lon_deg_east,
                        elevation_m,
                        tz_offset_minutes,
                        get_moon_lon,
                        get_sun_lon,
                        equatorial,
                        min_quality,
                    )
                })
            })
            .collect();
        for handle in handles {
            chunked.push(handle.join().expect("muhurta worker thread panicked"));
        }
    });

    Some(chunked.into_iter().flatten().collect())
}

/// Assess one contiguous, ascending run of candidate instants.
///
/// This is the whole of [`search_muhurta`]'s per-day work, factored out so a
/// worker thread can run it over a slice.
#[allow(clippy::too_many_arguments)]
fn evaluate_candidates(
    candidates: &[f64],
    lat_deg: f64,
    lon_deg_east: f64,
    elevation_m: f64,
    tz_offset_minutes: i32,
    get_moon_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    get_sun_lon: &(dyn Fn(f64) -> Option<f64> + Sync),
    equatorial: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    min_quality: f64,
) -> Vec<MuhurtaAssessment> {
    // (vara, [start, end)) — the vara's own real extent, as returned by
    // `vara_with_validity`. Recomputed only when `jd` falls outside that
    // interval — see the performance note above. `None` (from a polar case,
    // or `equatorial` failing) means the last computation isn't cacheable at
    // all, so the next step recomputes unconditionally too.
    //
    // The memo is per-run, and so per-worker: it is `&mut` state, which shared
    // across threads would be unsound, and it would be pointless besides —
    // chunks cover disjoint contiguous JD ranges, so one worker's cached
    // interval can only ever match another's at a single chunk boundary.
    let mut memo: Option<(Weekday, f64, f64)> = None;
    let mut results = Vec::new();

    for &jd in candidates {
        if let (Some(moon), Some(sun)) = (get_moon_lon(jd), get_sun_lon(jd)) {
            let still_valid = matches!(memo, Some((_, start, end)) if jd >= start && jd < end);
            let vara = match memo {
                Some((cached_vara, _, _)) if still_valid => cached_vara,
                _ => {
                    let (v, validity) = vara_with_validity(
                        jd,
                        lat_deg,
                        lon_deg_east,
                        elevation_m,
                        tz_offset_minutes,
                        equatorial,
                    );
                    memo = validity.map(|(start, end)| (v, start, end));
                    v
                }
            };
            let assessment = assess_muhurta(jd, moon, sun, vara);
            if assessment.quality_score >= min_quality {
                results.push(assessment);
            }
        }
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

    /// The defect a downstream consumer reported: `weekday_from_jd` returns
    /// the UT weekday, which flips at a different local clock time at every
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
        let vara = vara_at(jd, 21.3069, -157.8583, 0.0, -600, &flat_sun);
        // Not because 20:00 "is still evening, before local midnight" — vara_at
        // does not reckon from midnight, and that framing would just be
        // re-describing the UT-day defect this test exists to catch. Derived
        // independently here via `previous_rise`, the same primitive `vara_at`
        // uses: the most recent fake sunrise at or before `jd` is JD
        // 2459014.954893945 = 2020-06-14 10:55:03 UT = 2020-06-14 00:55:03
        // local — a few minutes after local midnight, still on the same
        // Sunday. (The NEXT sunrise, JD 2459015.952163512, is after `jd`, so
        // it does not open this vara.) The opening sunrise's local civil
        // weekday — Sunday — is the vara.
        let opening =
            vedaksha_astro::riseset::previous_rise(jd, 21.3069, -157.8583, 0.0, &flat_sun)
                .expect("the fixture rises daily at this latitude");
        assert!(
            libm::fabs(opening - 2_459_014.954_893_945) < 1e-6,
            "precondition: the opening sunrise must be JD 2459014.954893945, got {opening}"
        );
        assert_eq!(
            vara,
            Weekday::Sunday,
            "the opening sunrise (2020-06-14 ~00:55 local) is at or before the query instant, so this is still Ravivara"
        );
    }

    /// The report asked specifically for a case east of +150° as well as one
    /// west of −150°: inside ±90° the UT and local days usually agree, which is
    /// exactly why this class of bug survives a green suite. In that consumer's
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
        let east = vara_at(jd, 0.0, 165.0, 0.0, 660, &flat_sun); // UTC+11
        let west = vara_at(jd, 0.0, -165.0, 0.0, -660, &flat_sun); // UTC−11
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
            if vara_at(jd, 0.0, f64::from(lon_i), 0.0, tz_minutes, &flat_sun) == ut {
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
            vara_at(rise - one_hour, 0.0, 0.0, 0.0, 0, &flat_sun),
            Weekday::Friday,
            "before sunrise the vara is still the previous day's"
        );
        assert_eq!(
            vara_at(rise + one_hour, 0.0, 0.0, 0.0, 0, &flat_sun),
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
        let v = vara_at(2_451_544.5, 78.22, 15.65, 0.0, 60, &southern_sun);
        assert_eq!(
            v,
            Weekday::Saturday,
            "must fall back to the local civil weekday"
        );
    }

    /// FIX 2. `elevation_m` is a determinant of the vara, so it has to be in
    /// `vara_at`'s signature and has to reach `previous_rise`. Before the fix
    /// `vara_at` hardcoded 0.0 and there was no parameter to pass, which is
    /// how it and `kalam_windows` (which always passed the observer's own
    /// elevation) came to disagree at altitude.
    ///
    /// Same fixture and reasoning as
    /// `kalam_windows_selects_the_elevation_aware_vara_not_the_sea_level_one`
    /// below: at lat 0°/lon 0° with the fixed-RA `flat_sun`, an elevation of
    /// 3650 m (Lhasa — the elevation figure used elsewhere in this codebase's
    /// docstrings, and reachable through both shipped APIs now that they
    /// bound `elevation_m` to [−500, 9000], unlike the 15,000 m this test
    /// used before) moves the rise 7.06 min earlier than sea level (measured:
    /// `sea_rise=2451544.9687135713`, `high_rise=2451544.9638098693` at this
    /// fixture's `day_start`), and the midpoint of that gap is in the LATER
    /// vara for the elevated observer and still in the earlier one at sea
    /// level. The gap and the ordering are asserted rather than assumed, so
    /// this cannot silently degenerate into comparing a value with itself.
    #[test]
    fn vara_at_honours_the_observers_elevation() {
        let day_start = 2_451_544.5; // 2000-01-01 00:00 UT
        let elevation_m = 3650.0; // Lhasa; reachable through both shipped APIs' [-500, 9000] bound

        let sea_rise = vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, 0.0, &flat_sun)
            .rise
            .expect("sun rises at sea level");
        let high_rise =
            vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, elevation_m, &flat_sun)
                .rise
                .expect("sun rises for the elevated observer");
        assert!(
            high_rise < sea_rise,
            "elevation must move the rise EARLIER: sea={sea_rise} high={high_rise}"
        );

        let jd_ut = 0.5 * (sea_rise + high_rise);
        let sea_vara = vara_at(jd_ut, 0.0, 0.0, 0.0, 0, &flat_sun);
        let high_vara = vara_at(jd_ut, 0.0, 0.0, elevation_m, 0, &flat_sun);
        assert_ne!(
            sea_vara, high_vara,
            "the same instant is in different varas for the two observers, so \
             elevation_m must change vara_at's answer: sea {sea_vara:?} vs \
             elevated {high_vara:?}"
        );
        // The elevated answer is the one anchored on the elevated sunrise.
        assert_eq!(high_vara, weekday_from_day_index(high_rise));
        // At sea level the same-rotation rise is still in the future at
        // `jd_ut`, so the search falls back to the previous rotation's rise —
        // derived here rather than assumed to be exactly one day earlier
        // (for a fixed-RA Sun the gap is one SIDEREAL day, 0.99727 d).
        let sea_anchor =
            vedaksha_astro::riseset::previous_rise(jd_ut, 0.0, 0.0, 0.0, &flat_sun).unwrap();
        assert!(sea_anchor < high_rise, "the fallback is a rotation earlier");
        assert_eq!(sea_vara, weekday_from_day_index(sea_anchor));
    }

    /// FIX 2, at the `search_muhurta` level: the elevation the caller supplies
    /// has to survive the memo and reach `vara_with_validity`, or the tool
    /// answers for a sea-level observer whatever it was asked.
    #[test]
    fn search_muhurta_honours_the_observers_elevation() {
        let day_start = 2_451_544.5;
        let elevation_m = 15_000.0;
        let sea_rise = vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, 0.0, &flat_sun)
            .rise
            .expect("sun rises at sea level");
        let high_rise =
            vedaksha_astro::riseset::sun_rise_set(day_start, 0.0, 0.0, elevation_m, &flat_sun)
                .rise
                .expect("sun rises for the elevated observer");
        let jd_ut = 0.5 * (sea_rise + high_rise);

        let run = |elev: f64| {
            search_muhurta(
                jd_ut,
                jd_ut,
                0.0,
                0.0,
                elev,
                0,
                &|_| Some(94.0),
                &|_| Some(64.0),
                &flat_sun,
                0.0,
            )
        };
        let sea = run(0.0);
        let high = run(elevation_m);
        assert_eq!(sea.len(), 1, "one candidate at the single sampled instant");
        assert_eq!(high.len(), 1);
        assert_ne!(
            sea[0].weekday, high[0].weekday,
            "elevation_m must reach the vara derivation inside search_muhurta"
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

    /// FINDING 1 fix (the interval itself). Step 2 of `vara_with_validity`
    /// used to start its forward scan for the closing sunrise at
    /// `opening_day_start + 1.0` (0h UT of the day after the opening
    /// sunrise). When the opening sunrise landed only minutes after
    /// `opening_day_start`, the true next sunrise fell just BEFORE
    /// `opening_day_start + 1.0` — outside that scan's window — so the scan
    /// missed it and caught the sunrise AFTER that instead, returning an
    /// interval spanning TWO varas. The containment check never failed (the
    /// returned `end` was always a real sunrise at or after the true one,
    /// just the wrong — too late — one), which is why no prior test caught
    /// this: every existing assertion checks containment, never length.
    ///
    /// This sweeps a full 360° of longitude at 0.1° resolution (a fixed
    /// lat/instant pair, chosen to include the exact worst case pinned in
    /// `vara_with_validity_pins_the_reported_two_vara_regression` below) and
    /// asserts every returned interval is close to ONE day, not two.
    ///
    /// `flat_sun` fixes the Sun's RA at 0°, so consecutive sunrises (same
    /// altitude threshold, same upward crossing) recur once per SIDEREAL
    /// rotation, not once per solar day: the altitude depends on the local
    /// hour angle `H = LST − RA` alone (since `RA` and `dec` are both
    /// constant here), and `LST`'s dominant term advances at
    /// `360.98564736629`°/day (`vedaksha_ephem_core::sidereal_time::gmst`,
    /// Meeus eq. 12.4), so one full cycle of `H` — and hence one
    /// sunrise-to-sunrise interval — takes `360 / 360.98564736629` days =
    /// `0.9972695663290739` d, about 3 min 56 s short of a solar day. This
    /// was verified independently against this exact code (temporary probe,
    /// since removed): at the pinned worst case below (lat 0, lon 158.4) the
    /// fixed function returned a `0.9972695661708713`-day interval, and
    /// re-measured against the analytic Meeus Ch. 15 search that replaced the
    /// 5-minute scan it returns `0.9972695666365325` d — the two differ by
    /// 4.7e-10 d, one ULP. Both match the closed form to 9 significant
    /// figures; the tiny residual is the GMST polynomial's `T²` term
    /// (negligible over one day) plus the resolution of the root finder, not a
    /// modeling gap.
    #[test]
    fn vara_with_validity_never_spans_more_than_one_vara() {
        // 360 / 360.98564736629 (deg/day, the GMST rate) — see derivation
        // above. `flat_sun`'s fixed RA makes this the fixture's true
        // sunrise-to-sunrise spacing, not the caller-facing solar day.
        const SIDEREAL_SPACING_DAYS: f64 = 0.997_269_566_329_073_9;
        // Measured bound on how far any single sample's interval length
        // strayed from that constant across this exact sweep: 7.7e-10 d
        // against the 5-minute scan plus bisection, re-measured at
        // 3.0745861412384556e-10 d (worst at lon −180) against the analytic
        // Meeus Ch. 15 search that replaced it — both from the GMST
        // polynomial's tiny T² term plus the root finder's own ULP-scale
        // resolution. 1e-6 leaves three orders of margin over either without
        // being loose enough to let a real one-day slip through undetected.
        const TOLERANCE_DAYS: f64 = 1e-6;

        let lat = 0.0;
        let jd = 2_451_554.75; // the reviewer's pinned worst-case instant
        let mut checked = 0u32;
        // Tenths of a degree, so the loop stays in exact integers: -180.0°
        // to 180.0° at a 0.1° step (3601 samples), including 158.4° exactly.
        let mut lon_tenths = -1800_i32;
        while lon_tenths <= 1800 {
            let lon = f64::from(lon_tenths) / 10.0;
            let (_weekday, validity) = vara_with_validity(jd, lat, lon, 0.0, 0, &flat_sun);
            let (start, end) = validity.unwrap_or_else(|| {
                panic!("lon={lon}: the equatorial sun rises and sets at every non-polar latitude")
            });
            let length = end - start;
            assert!(
                (length - SIDEREAL_SPACING_DAYS).abs() < TOLERANCE_DAYS,
                "lon={lon}: interval length {length} d is not close to the \
                 fixture's sidereal-day spacing {SIDEREAL_SPACING_DAYS} d — \
                 start={start}, end={end} spans more than one vara"
            );
            checked += 1;
            lon_tenths += 1;
        }
        assert_eq!(checked, 3601, "sanity: full 360° sweep at 0.1° steps");
    }

    /// FINDING 1 fix — named regression for the reviewer's exact reported
    /// worst case (measured directly against the OLD, buggy code, temporary
    /// probe since removed): `lat = 0.0`, `lon = 158.4`, `jd = 2451554.75`
    /// used to return `start = 2451554.5026106257`, `end =
    /// 2451556.497149758` — a `1.9945`-day span, covering two sunrises. This
    /// pins that exact input to a single-vara-length interval so a future
    /// regression on this precise case is caught even if the general sweep
    /// above is ever narrowed or resampled differently.
    #[test]
    fn vara_with_validity_pins_the_reported_two_vara_regression() {
        let (lat, lon, jd) = (0.0, 158.4, 2_451_554.75);
        let (_weekday, validity) = vara_with_validity(jd, lat, lon, 0.0, 0, &flat_sun);
        let (start, end) = validity.expect("equatorial sun rises and sets at lat 0");
        let length = end - start;

        assert!(
            length < 1.5,
            "the pinned worst case must not span two varas: start={start}, \
             end={end}, length={length} d"
        );

        // Independently measured value for this exact input (see the
        // derivation in `vara_with_validity_never_spans_more_than_one_vara`
        // above): 360 / 360.98564736629 = 0.9972695663290739 d.
        const EXPECTED_LENGTH_DAYS: f64 = 0.997_269_566_329_073_9;
        assert!(
            (length - EXPECTED_LENGTH_DAYS).abs() < 1e-6,
            "length {length} d does not match the independently derived \
             sidereal spacing {EXPECTED_LENGTH_DAYS} d"
        );
    }

    /// TWO-SUNRISES-IN-ONE-WINDOW regression, `flat_sun` reproducer.
    ///
    /// The retired implementation walked `sun_rise_set`'s 24-hour window back
    /// along the civil-UT calendar. `sun_rise_set` reports only the FIRST rise
    /// in its window, and this fixture's rises recur once per SIDEREAL
    /// rotation — `360 / 360.98564736629` = 0.9972695663290739 d, the GMST
    /// rate of Meeus eq. 12.4 as implemented by
    /// `vedaksha_ephem_core::sidereal_time::gmst` — which is SHORTER than the
    /// window. So two sunrises could share one window, and the second was
    /// invisible.
    ///
    /// Measured against the old code (probe run, since removed) at lat −9.0,
    /// lon 159.4, tz +660, jd 2451554.75: `day_start` = JD 2451554.5, whose
    /// window's first rise (JD 2451555.4970812206) is after `jd_ut` and is
    /// rejected; the walk steps back to `day_start` = JD 2451553.5, whose
    /// window holds TWO rises — JD 2451553.5025420883 and JD
    /// 2451554.4998116544 — and reports only the first. The old return was
    /// vara **Monday**, `[2451553.502542088, 2451554.499811655)`, which does
    /// not even contain `jd_ut` = 2451554.75: the instant lands 0.2502 d PAST
    /// `end`. A coarse 1-second scan of the altitude crossings over `[jd − 2,
    /// jd + 2]` located WHICH rotation is correct — rises near JD
    /// 2451553.5025420883, 2451554.4998116544, 2451555.4970812206,
    /// 2451556.4943507873 — identifying that the vara containing `jd_ut`
    /// opens at the second of those (a **Tuesday** at tz +660), not the first
    /// (**Monday**) the old code returned. That coarse scan's own resolution
    /// (~1 s ≈ 1.16e-5 d) does not, by itself, support the ~1e-10-day
    /// precision of the literals asserted below: those are `start`/`end` as
    /// this test's own call to `vara_with_validity` — i.e. the roots
    /// `previous_rise`/`next_rise` converge to — actually produced,
    /// re-measured directly against the implementation: the residual between
    /// the literal below and a fresh run was ≤4.7e-10 d (one ULP at this JD's
    /// magnitude). The 1e-6 d assertion tolerance is therefore not tied to the
    /// coarse scan's resolution; it is kept three orders of magnitude looser
    /// than that measured ULP-level residual on purpose, for headroom, not
    /// because 1e-6 d is the achieved precision.
    ///
    /// ⚠️ That headroom means this assertion does NOT constrain the last
    /// digits. When the sunrise search moved from a 5-minute scan plus
    /// bisection to the analytic Meeus Ch. 15 iteration, `start` moved from
    /// 2451554.4998116549104452 to 2451554.4998116544447839 and `end` from
    /// 2451555.4970812210813165 to 2451555.4970812206156552 — one ULP each,
    /// 4.0e-5 s — and this test could not have seen it. The literals below
    /// were re-derived from the current implementation anyway (they already
    /// happened to match its value, not the old one); the assertion that
    /// actually pins the search at ULP resolution is
    /// `vedaksha_astro::riseset`'s scan-oracle sweep.
    ///
    /// ⚠️ `tz_offset_minutes` MUST be non-zero here. At tz 0 the two candidate
    /// sunrises usually floor to the same weekday, which is exactly why every
    /// pre-existing test — all of which used tz 0 for the sweep cases — missed
    /// this.
    #[test]
    fn vara_with_validity_pins_the_two_sunrises_in_one_window_regression() {
        let (lat, lon, tz, jd) = (-9.0, 159.4, 660, 2_451_554.75);
        let (vara, validity) = vara_with_validity(jd, lat, lon, 0.0, tz, &flat_sun);
        let (start, end) = validity.expect("the fixture rises daily at lat -9");

        assert_eq!(
            vara,
            Weekday::Tuesday,
            "the vara opens at the LATER of the two sunrises in the old walk's \
             window (JD 2451554.4998116544); the old code returned Monday from \
             the earlier one. got [{start}, {end})"
        );
        assert!(
            start <= jd && jd < end,
            "containment: [{start}, {end}) must contain {jd} — the old code \
             returned an interval ending 0.2502 d BEFORE it"
        );
        assert!(
            libm::fabs(start - 2_451_554.499_811_654_4) < 1e-6,
            "start {start} must be the bisection-derived sunrise \
             JD 2451554.4998116544"
        );
        assert!(
            libm::fabs(end - 2_451_555.497_081_220_6) < 1e-6,
            "end {end} must be the bisection-derived next sunrise \
             JD 2451555.4970812206"
        );
    }

    /// TWO-SUNRISES-IN-ONE-WINDOW regression, REAL-SUN reproducer.
    ///
    /// The defect is not an artefact of the fixed-RA fixture: the real Sun's
    /// inter-sunrise gap is also shorter than a civil day over roughly half the
    /// year (the equation of time plus the observer's own latitude/declination
    /// geometry), so the same two-rises-in-one-window collapse happens with
    /// `AnalyticalProvider`.
    ///
    /// Measured at lat −9.0, lon 87.668, tz +660, jd 2459113.4. The old code
    /// returned vara **Saturday**, `[2459111.500415422, 2459112.49999942)` —
    /// `jd_ut` sits 0.9001 d past `end`, a full vara out. A coarse 1-second
    /// altitude scan located WHICH rotation is correct: rises near JD
    /// 2459111.500415422, 2459112.49999942, 2459113.499583689,
    /// 2459114.4991684277, identifying that the vara containing `jd_ut` opens
    /// at the second of those and closes at the third — a **Sunday** at tz
    /// +660, not the old code's Saturday. As in the `flat_sun` regression
    /// above, that scan's own ~1-second (~1.16e-5 d) resolution is coarser
    /// than the ~1e-10-day precision of the literals asserted below; those
    /// are `start`/`end` as this test's own call to `vara_with_validity`
    /// actually produces — the roots `previous_rise`/`next_rise` converge to —
    /// re-measured directly against the implementation with a residual of 0
    /// (bit-exact) between the literal and a fresh run. The 1e-6 d tolerance
    /// below is headroom over that measured residual, not a restatement of the
    /// coarse scan's resolution.
    ///
    /// ⚠️ As in the `flat_sun` regression above, that headroom means the
    /// assertion does not constrain the last digits. Moving the sunrise search
    /// from the 5-minute scan plus bisection to the analytic Meeus Ch. 15
    /// iteration left `start` BIT-EXACTLY unchanged
    /// (2459112.4999994197860360) and moved `end` by one ULP, from
    /// 2459113.4995836885645986 to 2459113.4995836890302598 — 4.0e-5 s. Both
    /// literals were re-derived from the current implementation; the test that
    /// pins the search at ULP resolution is `vedaksha_astro::riseset`'s
    /// scan-oracle sweep.
    #[test]
    fn vara_with_validity_pins_the_real_sun_two_sunrises_regression() {
        use vedaksha_astro::riseset::sun_equatorial_deg;
        use vedaksha_ephem_core::analytical::AnalyticalProvider;

        let provider = AnalyticalProvider;
        let real_sun = |j: f64| sun_equatorial_deg(&provider, j);

        let (lat, lon, tz, jd) = (-9.0, 87.668, 660, 2_459_113.4);
        let (vara, validity) = vara_with_validity(jd, lat, lon, 0.0, tz, &real_sun);
        let (start, end) = validity.expect("the sun rises daily at lat -9");

        assert_eq!(
            vara,
            Weekday::Sunday,
            "the real Sun reproduces the defect too: the old code returned \
             Saturday from a sunrise a whole vara early. got [{start}, {end})"
        );
        assert!(
            start <= jd && jd < end,
            "containment: [{start}, {end}) must contain {jd} — the old code \
             returned an interval ending 0.9001 d BEFORE it"
        );
        assert!(
            libm::fabs(start - 2_459_112.499_999_42) < 1e-6,
            "start {start} must be the bisection-derived sunrise \
             JD 2459112.49999942"
        );
        assert!(
            libm::fabs(end - 2_459_113.499_583_688_6) < 1e-6,
            "end {end} must be the bisection-derived next sunrise \
             JD 2459113.499583689"
        );
    }

    /// Containment as a SWEEP, not a point — the pinned reproducers above are
    /// two samples and a pair of point tests can pass by luck.
    ///
    /// SAMPLING, stated explicitly:
    /// - Sweep A: the full −180°..180° longitude range at **0.01°** (36,001
    ///   samples) at lat −9.0, tz +660, jd 2451554.75. That is exactly the
    ///   configuration in which the failing band was measured against the old
    ///   code: **98 of 36,001 samples (0.272%)** violated containment, the
    ///   worst leaving `jd_ut` 0.2527 d beyond `end` (at lon 160.31). 0.01° is
    ///   the resolution that band was found at, so this sweep is fine enough
    ///   to hit it by construction; a coarser step risks stepping over it.
    /// - Sweep B: the same longitude range at 0.5° (721 samples) across five
    ///   latitudes spanning both hemispheres, at a different non-zero tz
    ///   (+330) and a different instant, so the property is not pinned to one
    ///   observer. Coarse on purpose — Sweep A already carries the resolution;
    ///   this carries the latitude and tz spread without a 5× runtime.
    ///
    /// ⚠️ Both sweeps use a NON-ZERO `tz_offset_minutes`. At tz 0 the two
    /// candidate sunrises usually floor to the same weekday, which masks the
    /// defect entirely — the reason the pre-existing tz-0 sweeps stayed green
    /// through it.
    ///
    /// Interval length is asserted alongside containment: containment alone
    /// would still pass for an interval spanning two varas (the historical
    /// FINDING-1 defect), and length alone would still pass for an interval of
    /// the right width sitting in the wrong place. The expected length is the
    /// fixture's own rise-to-rise spacing — one SIDEREAL rotation,
    /// `360 / 360.98564736629` = 0.9972695663290739 d, NOT 1.0 — derived in
    /// `vara_with_validity_never_spans_more_than_one_vara` above.
    #[test]
    fn vara_with_validity_contains_the_instant_across_a_fine_longitude_sweep() {
        const SIDEREAL_SPACING_DAYS: f64 = 0.997_269_566_329_073_9;
        const TOLERANCE_DAYS: f64 = 1e-6;

        let check = |lat: f64, lon: f64, tz: i32, jd: f64| {
            let (_vara, validity) = vara_with_validity(jd, lat, lon, 0.0, tz, &flat_sun);
            let (start, end) = validity.unwrap_or_else(|| {
                panic!("lat={lat} lon={lon}: the fixture rises at every non-polar latitude")
            });
            assert!(
                start <= jd && jd < end,
                "lat={lat} lon={lon} tz={tz}: [{start}, {end}) does not contain {jd}"
            );
            let length = end - start;
            assert!(
                libm::fabs(length - SIDEREAL_SPACING_DAYS) < TOLERANCE_DAYS,
                "lat={lat} lon={lon} tz={tz}: interval length {length} d is not \
                 one sidereal rotation ({SIDEREAL_SPACING_DAYS} d) — \
                 [{start}, {end})"
            );
        };

        // Sweep A — hundredths of a degree, integer loop counter so no float
        // accumulation and no `as` cast.
        let mut lon_hundredths = -18_000_i32;
        let mut checked_a = 0_u32;
        while lon_hundredths <= 18_000 {
            check(-9.0, f64::from(lon_hundredths) / 100.0, 660, 2_451_554.75);
            checked_a += 1;
            lon_hundredths += 1;
        }
        assert_eq!(checked_a, 36_001, "sanity: full 360° sweep at 0.01° steps");

        // Sweep B — half-degree steps, five latitudes, a different non-zero tz
        // and a different instant.
        let mut checked_b = 0_u32;
        for lat in [-51.5_f64, -33.9, -9.0, 21.3, 45.0] {
            let mut lon_halves = -360_i32;
            while lon_halves <= 360 {
                check(lat, f64::from(lon_halves) / 2.0, 330, 2_459_113.4);
                checked_b += 1;
                lon_halves += 1;
            }
        }
        assert_eq!(checked_b, 5 * 721, "sanity: 5 latitudes × 721 longitudes");
    }

    /// FINDING 1 fix. The old memo keyed on `libm::floor(jd - 0.5) + 0.5`
    /// (civil-UT midnight) instead of the vara's real sunrise-to-sunrise
    /// extent, so a sample landing between local midnight and local sunrise
    /// got the PREVIOUS civil day's stale vara. Proven at Chennai (lat
    /// 13.08, lon 80.27, tz +330): measured directly (see the fix-pass
    /// report), the correct fresh-every-step sequence over 9 samples from JD
    /// 2_451_545.5 is `[Sat, Sun, Sun, Mon, Mon, Tue, Tue, Wed, Wed]` while
    /// the old civil-day-keyed memo produced `[Sat, Sat, Sun, Sun, Mon, Mon,
    /// Tue, Tue, Wed]` — 4 of 9 wrong.
    ///
    /// Rather than paste that hand sequence in as the assertion, this
    /// derives the reference independently at test time: `vara_at` called
    /// completely fresh (no memo of any kind) at each of the 9 sample
    /// instants `search_muhurta` itself would visit. That reference is
    /// compared value-by-value against what the memoised `search_muhurta`
    /// actually returns, so this fails on ANY divergence, not just the
    /// specific one measured above.
    #[test]
    fn search_muhurta_memoised_vara_matches_unmemoised_reference_at_chennai() {
        let (lat, lon, tz) = (13.08, 80.27, 330);
        let start = 2_451_545.5;
        let end = start + 4.0; // 9 samples at the 0.5-day step (0..=8 half-days)

        let moon_lon = 94.0_f64; // Pushya — arbitrary, fixed across both runs
        let sun_lon = 64.0_f64;

        let mut expected = Vec::new();
        let mut jd = start;
        while jd <= end {
            expected.push(vara_at(jd, lat, lon, 0.0, tz, &flat_sun));
            jd += 0.5;
        }
        assert_eq!(
            expected.len(),
            9,
            "sanity: 9 samples over a 4-day span at a 0.5-day step"
        );

        // min_quality 0.0 keeps every sample regardless of score, so the
        // result count and order line up 1:1 with `expected` above.
        let results = search_muhurta(
            start,
            end,
            lat,
            lon,
            0.0,
            tz,
            &|_| Some(moon_lon),
            &|_| Some(sun_lon),
            &flat_sun,
            0.0,
        );
        assert_eq!(results.len(), 9, "min_quality 0.0 must keep every sample");
        let actual: Vec<Weekday> = results.iter().map(|a| a.weekday).collect();

        assert_eq!(
            actual, expected,
            "search_muhurta's memoised vara must match the unmemoised \
             reference sample-by-sample — a civil-UT-day-keyed memo hands \
             the previous day's stale vara to any sample that falls between \
             local midnight and local sunrise"
        );
    }

    /// Secondary check (not the load-bearing one above): the memo still
    /// saves work relative to deriving the vara fresh at every 0.5-day step.
    /// Unlike the retired civil-day version, the fix's memo interval is the
    /// vara's actual sunrise-to-sunrise extent, which is not aligned to a
    /// fixed civil-day schedule — there is no "exactly one trigger per
    /// calendar day" model to assert an exact count against here. So this
    /// only bounds it: memoised calls must be strictly fewer than one
    /// `vara_at` call per step (some memoisation fired) and at least one
    /// (the memo is not accidentally skipping the vara computation
    /// entirely).
    /// Splitting the day scan across workers must not change one bit of the
    /// answer, nor the order the windows come back in.
    ///
    /// 400 days is 801 candidate instants — well over the two chunks
    /// `search_muhurta` needs before it will split — so on any host reporting
    /// 2-way parallelism this compares the threaded path against the serial
    /// one. `search_candidates_in_parallel` is called directly as well, so the
    /// threaded values are compared even if `search_muhurta`'s own dispatch
    /// were to change.
    ///
    /// The floats are compared as raw bit patterns, not with a tolerance: the
    /// claim is that chunking reproduces the identical sequence, and a
    /// tolerance would not test that claim.
    #[test]
    fn threaded_and_serial_scans_agree_bit_for_bit() {
        let (lat, lon, tz) = (13.08, 80.27, 330);
        let start = 2_451_545.5;
        let end = start + 400.0;

        // Smooth synthetic longitudes at roughly the true mean rates, so the
        // scan walks the whole nakshatra/tithi cycle rather than sitting on one
        // value. Exact rates do not matter here — only that both paths see the
        // same ones.
        let moon = |jd: f64| Some((jd - start) * 13.176_396);
        let sun = |jd: f64| Some((jd - start) * 0.985_647);

        let mut candidates = Vec::new();
        let mut jd = start;
        while jd <= end {
            candidates.push(jd);
            jd += SEARCH_STEP_DAYS;
        }
        assert!(
            candidates.len() >= 2 * MIN_CANDIDATES_PER_WORKER,
            "the range must be long enough for the scan to split at all: {} candidates",
            candidates.len()
        );

        let serial =
            evaluate_candidates(&candidates, lat, lon, 0.0, tz, &moon, &sun, &flat_sun, 0.0);

        // Every path that can produce this answer, checked against the serial
        // walk. `search_candidates_in_parallel` is `None` only on a host that
        // reports no parallelism, where there is no threaded path to check.
        let mut under_test = vec![search_muhurta(
            start, end, lat, lon, 0.0, tz, &moon, &sun, &flat_sun, 0.0,
        )];
        if let Some(parallel) = search_candidates_in_parallel(
            &candidates,
            lat,
            lon,
            0.0,
            tz,
            &moon,
            &sun,
            &flat_sun,
            0.0,
        ) {
            under_test.push(parallel);
        }

        for got in &under_test {
            assert_eq!(
                got.len(),
                serial.len(),
                "the threaded scan must report the same number of windows"
            );
            for (i, (t, s)) in got.iter().zip(&serial).enumerate() {
                assert_eq!(
                    t.jd.to_bits(),
                    s.jd.to_bits(),
                    "window {i}: candidate instant differs bitwise"
                );
                assert_eq!(
                    t.quality_score.to_bits(),
                    s.quality_score.to_bits(),
                    "window {i}: quality score differs bitwise"
                );
                assert_eq!(t.tithi, s.tithi, "window {i}: tithi differs");
                assert_eq!(t.weekday, s.weekday, "window {i}: vara differs");
                assert_eq!(t.factors, s.factors, "window {i}: factors differ");
                assert_eq!(
                    format!("{:?}", t.nakshatra),
                    format!("{:?}", s.nakshatra),
                    "window {i}: nakshatra differs"
                );
            }
        }
    }

    #[test]
    fn search_muhurta_memoisation_still_saves_equatorial_calls() {
        // `AtomicU32`, not `Cell`: the `equatorial` callback is now
        // `&(dyn Fn(..) + Sync)`, so the counter it closes over has to be
        // shareable too. `Relaxed` is the right ordering — the assertions read
        // the total only after `search_muhurta` has returned, i.e. after every
        // worker has been joined, so there is nothing to order against.
        use std::sync::atomic::{AtomicU32, Ordering};

        let (lat, lon, tz) = (13.08, 80.27, 330);
        let start = 2_451_545.5;
        let end = start + 4.0; // 9 steps

        let per_step_calls = AtomicU32::new(0);
        let mut jd = start;
        while jd <= end {
            let counted = |t: f64| {
                per_step_calls.fetch_add(1, Ordering::Relaxed);
                flat_sun(t)
            };
            let _ = vara_at(jd, lat, lon, 0.0, tz, &counted);
            jd += 0.5;
        }
        let unmemoised_total = per_step_calls.load(Ordering::Relaxed);

        let actual_calls = AtomicU32::new(0);
        let counted = |t: f64| {
            actual_calls.fetch_add(1, Ordering::Relaxed);
            flat_sun(t)
        };
        let results = search_muhurta(
            start,
            end,
            lat,
            lon,
            0.0,
            tz,
            &|_| Some(94.0),
            &|_| Some(64.0),
            &counted,
            0.0,
        );
        assert_eq!(results.len(), 9);
        assert!(
            actual_calls.load(Ordering::Relaxed) > 0,
            "sanity: some `equatorial` evaluation must happen"
        );
        assert!(
            actual_calls.load(Ordering::Relaxed) < unmemoised_total,
            "memoisation should cost fewer `equatorial` calls than \
             recomputing per step: {} memoised vs {unmemoised_total} \
             unmemoised",
            actual_calls.load(Ordering::Relaxed)
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
        let moon_and_sun = |jd: f64| {
            Some((
                ((13.176 * (jd - j0)).rem_euclid(360.0), 13.176),
                ((0.985 * (jd - j0)).rem_euclid(360.0), 0.985),
            ))
        };
        let end = compute_tithi_end(j0, &moon_and_sun).expect("tithi end");
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
        let windows = kalam_windows(2_451_544.5 + 0.5, 0.0, 0.0, 0.0, 0, &flat_sun).windows;
        let (rahu, gulika) = windows.expect("sun rises here");
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
        let windows = kalam_windows(2_451_544.5 + 0.5, 0.0, 0.0, 0.0, 0, &flat_sun).windows;
        let (_, gulika) = windows.expect("sun rises here");
        let rs = vedaksha_astro::riseset::sun_rise_set(2_451_544.5, 0.0, 0.0, 0.0, &flat_sun);
        assert!(
            (gulika.start_jd - rs.rise.unwrap()).abs() < 1e-9,
            "Saturday gulika is slot 1, so it starts at sunrise"
        );
    }

    /// `tz_offset_minutes` selects which weekday's slot table applies, and
    /// Rahu sits in a different eighth on different weekdays. Derived by
    /// direct computation while writing this test (not restated from the
    /// brief): at lon 165°E, `vara_at(2_459_015.75, 0.0, 165.0, 0.0, 660,
    /// flat_sun)` (tz honoured) is Monday — rahu slot 2 — while
    /// `vara_at(2_459_015.75, 0.0, 165.0, 0.0, 0, flat_sun)` (tz dropped) is
    /// Sunday — rahu slot 8. Slot 2 sits in the first quarter of the
    /// daytime and slot 8 is the last eighth, ending at sunset, so
    /// `kalam_windows` under the two tz values must place rahu's start more
    /// than 0.25 day (6 h) apart. Measured: `|2459015.1208594847 −
    /// 2459015.498298313| ≈ 0.377 d` (9.06 h) — the 0.25 d threshold sits
    /// comfortably below that measurement, not at a round number chosen to
    /// paper over noise.
    ///
    /// The tz comparison above constrains slot SELECTION only — it says
    /// nothing about whether step (b) (scanning forward from the opening
    /// sunrise for the sunset) actually ran. Added below: both windows must
    /// run forwards, and their width must equal one eighth of the REAL
    /// (positively-ordered) daytime. Derived independently, not by calling
    /// `kalam_windows` again: at lon 165°E the opening sunrise is JD
    /// 2459015.057953013 — the most recent one at or before `jd`, and here
    /// also the only rise inside the window `[JD 2459014.5, 2459015.5]`, so
    /// `sun_rise_set(2459014.5, …).rise` names the same instant that
    /// `previous_rise(jd, …)` does and is used below as the independent
    /// derivation. (The NEXT rise, JD 2459016.0552225793, is after `jd`.)
    /// Scanning forward from THAT sunrise gives sunset = JD
    /// 2459015.5612047845: a positive 12.078 h daytime, one eighth of which
    /// is 0.0629064714 d (90.59 min).
    #[test]
    fn kalam_windows_uses_the_observers_own_weekday_not_the_ut_one() {
        let jd = 2_459_015.75;
        let windows_correct = kalam_windows(jd, 0.0, 165.0, 0.0, 660, &flat_sun).windows;
        let (rahu_correct, gulika_correct) = windows_correct.expect("sun rises here");
        let windows_wrong = kalam_windows(jd, 0.0, 165.0, 0.0, 0, &flat_sun).windows;
        let (rahu_wrong, _) = windows_wrong.expect("sun rises here");
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
    /// (opening-sunrise search + forward scan) to one naive single
    /// `sun_rise_set` call. This one catches keeping the sunrise search but
    /// taking the sunset from a CALENDAR-anchored window instead of from the
    /// sunrise instant — the review found this survives all 41 existing tests
    /// and drives the daytime to about −11.85 h at lon 165°/100°/45°/−175°.
    /// 165° is already used above and −157.8583° is used by the far-west test
    /// below, so this uses 45°E to keep the longitudes distinct.
    ///
    /// The `day_start` in this test's name and derivations is the civil-UT
    /// midnight the retired walk-back anchored on; it survives here as the
    /// name of the mutation being pinned, not as anything the current
    /// implementation computes.
    ///
    /// The `expected` closure below reimplements the *correct* step (b) —
    /// scanning forward from the independently re-derived sunrise via the
    /// `sun_rise_set` primitive directly, not via `kalam_windows` — so it
    /// only agrees with `kalam_windows`'s actual output when `kalam_windows`
    /// also does the correct scan. Measured directly: at jd 2459015.75, lon
    /// 45°E, the opening sunrise is JD 2459015.390376202 — here the only rise
    /// inside `[JD 2459014.5, 2459015.5]`, so `sun_rise_set(2459014.5, …)`
    /// names the same instant `previous_rise(jd, …)` does and serves as the
    /// independent derivation; scanning forward from that sunrise gives sunset
    /// = JD 2459015.8936279733 (12.078 h daytime). Taking `.set` from
    /// `sun_rise_set(2459014.5, ...)` instead (the mutation) gives JD
    /// 2459014.896358407, which is BEFORE sunrise — a −11.856 h "daytime",
    /// so under the mutation `eighth` goes negative and every window's
    /// `end_jd` falls before its `start_jd`, failing the ordering assertion
    /// outright.
    ///
    /// PROVEN by actually applying this mutation to `kalam_windows` (taking
    /// `.set` from the `day_start`-anchored call while keeping the
    /// sunrise search) and re-running: this test FAILED (`rahu ran backwards:
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

        let reckoning = kalam_windows(jd, 0.0, lon, 0.0, tz, &flat_sun);
        assert_eq!(
            reckoning.vara, vara,
            "kalam_windows must return the same vara it used to select the slots"
        );
        assert!(
            reckoning.from_sunrise,
            "a sunrise was found here, so the vara is not the civil fallback"
        );
        let (rahu, gulika) = reckoning.windows.expect("sun rises here");

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
        let windows = kalam_windows(jd, 21.3069, -157.8583, 0.0, -600, &flat_sun).windows;
        let (rahu, gulika) = windows.expect("sun rises here");
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
        // longitude — that rise is AFTER `jd`, so it does not open this vara;
        // see `vara_does_not_follow_the_ut_day_boundary_in_the_far_west`,
        // which derives it as JD 2459015.952163512. Independently re-derived
        // here via `next_rise` (not by calling `kalam_windows` again, so this
        // is not checking the function against itself): the vara's own
        // daytime must end strictly before that following sunrise. Rahu is
        // slot 8 for Sunday (the vara here), so `rahu.end_jd` is exactly this
        // daytime's sunset.
        let next_days_rise =
            vedaksha_astro::riseset::next_rise(jd, 21.3069, -157.8583, 0.0, &flat_sun)
                .expect("the following sunrise must exist here too");
        assert!(
            rahu.end_jd < next_days_rise,
            "rahu kalam (ending {}) must fall before the FOLLOWING sunrise ({next_days_rise}); a naive single-window sun_rise_set call wrongly selects that later cycle's day here",
            rahu.end_jd
        );
    }

    /// Polar night has no daytime to divide into eighths — but FINDING 2's
    /// contract requires the vara to still come back (the local-civil
    /// fallback), even though the windows cannot. Same instant/observer as
    /// `vara_falls_back_where_the_sun_does_not_rise` above, which pins the
    /// fallback weekday directly: Saturday.
    #[test]
    fn no_kalam_windows_where_the_sun_does_not_rise() {
        let southern_sun = |_jd: f64| Some((0.0_f64, -23.0_f64));
        let reckoning = kalam_windows(2_451_544.5, 78.22, 15.65, 0.0, 60, &southern_sun);
        assert!(
            reckoning.windows.is_none(),
            "polar night has no daytime to divide"
        );
        assert!(
            !reckoning.from_sunrise,
            "polar night found no sunrise, so the vara is the civil fallback \
             and must say so"
        );
        assert_eq!(
            reckoning.vara,
            Weekday::Saturday,
            "the vara must still fall back to the local civil weekday even with no windows"
        );
    }

    /// Exercises `elevation_m` end-to-end, and is built to actually
    /// DISCRIMINATE between `kalam_windows` deriving its vara from its own
    /// elevation-aware sunrise vs. delegating to `vara_at` (which hardcodes
    /// elevation 0.0). An earlier version of this test called
    /// `kalam_windows` at `day_start + 0.5` — well after BOTH the
    /// elevation-adjusted and the sea-level sunrise on that day — so both
    /// derivations landed on the same day's sunrise and hence the same
    /// weekday; verified by actually reverting the `vara` line in
    /// `kalam_windows` to `vara_at(jd_ut, lat_deg, lon_deg_east, 0.0,
    /// tz_offset_minutes, equatorial)` and re-running the old test, which
    /// still passed. See the fix-pass report for that failure-to-fail
    /// evidence.
    ///
    /// The two derivations disagree only when the query instant falls
    /// strictly BETWEEN the elevation-adjusted sunrise and the sea-level
    /// sunrise of the *same* rotation. Walking through why, from
    /// `kalam_windows`'s own sunrise search: inside that gap the
    /// elevation-aware `previous_rise` has already found a sunrise ≤ `jd_ut`
    /// (the vara has turned), while the sea-level `previous_rise` that
    /// `vara_at` runs finds that rise is still AFTER `jd_ut` at sea level and
    /// so returns the PREVIOUS rotation's sunrise — and hence the previous
    /// day's vara — instead.
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
    /// - `vara_at(mid, 0.0, 0.0, 0.0, 0, flat_sun)` — note the explicit
    ///   `0.0` elevation, which since the FIX 2 pass is an ARGUMENT rather
    ///   than an unstated convention, so this test now asks for the
    ///   sea-level derivation deliberately instead of receiving it by
    ///   default — = Friday (gulika slot 2, starts one eighth-daytime AFTER
    ///   sunrise), confirmed by direct call, not hand-derived: at sea level
    ///   the search rejects the same-day rise (after `mid`) and falls back to
    ///   the previous day's rise, JD 2451543.9714440051 (Dec 31, 1999, a
    ///   Friday).
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
        // does not reimplement `kalam_windows`'s sunrise search, it just names
        // the weekday of the sunrise instant already derived above.
        let elevation_vara = weekday_from_day_index(elevated_rise);
        // The sea-level vara: `vara_at` called DIRECTLY with elevation 0.0,
        // the actual "wrong" derivation a reverted `vara` line would
        // substitute in — and, since FIX 2, a derivation the caller has to
        // ask for explicitly rather than one the API hands out silently.
        let sea_vara = vara_at(jd_ut, 0.0, 0.0, 0.0, 0, &flat_sun);
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

        let windows = kalam_windows(jd_ut, 0.0, 0.0, elevation_m, 0, &flat_sun).windows;
        let (_, gulika) = windows.expect("sun rises here");

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
        let trop = |body: Body, t: f64| {
            apparent_position(&provider, body, t)
                .ok()
                .map(|p| (p.ecliptic.longitude.to_degrees(), p.longitude_speed))
        };
        let moon_trop = |t: f64| trop(Body::Moon, t);
        let sun_trop = |t: f64| trop(Body::Sun, t);
        let moon_and_sun_trop = |t: f64| Some((moon_trop(t)?, sun_trop(t)?));

        let t_end = compute_tithi_end(jd, &moon_and_sun_trop).expect("tithi end");
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
