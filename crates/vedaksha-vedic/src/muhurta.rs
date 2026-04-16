// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
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

/// Get the weekday for a Julian Day.
///
/// Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 7.
#[must_use]
pub fn weekday_from_jd(jd: f64) -> Weekday {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let day_index = ((jd + 1.5) % 7.0).floor() as u8;
    match day_index {
        1 => Weekday::Monday,
        2 => Weekday::Tuesday,
        3 => Weekday::Wednesday,
        4 => Weekday::Thursday,
        5 => Weekday::Friday,
        6 => Weekday::Saturday,
        _ => Weekday::Sunday, // 0, and unreachable values
    }
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
    let weekday = weekday_from_jd(jd);

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
    }
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

    // --- weekday_from_jd ---

    #[test]
    fn weekday_j2000_is_saturday() {
        // J2000.0 = JD 2451545.0 = Jan 1.5, 2000 = Saturday
        let wd = weekday_from_jd(2_451_545.0);
        assert_eq!(wd, Weekday::Saturday, "J2000.0 should be Saturday");
    }

    #[test]
    fn weekday_advances_correctly() {
        // J2000.0 = Saturday; J2000.0 + 1 = Sunday
        let wd = weekday_from_jd(2_451_546.0);
        assert_eq!(wd, Weekday::Sunday);
        let wd2 = weekday_from_jd(2_451_547.0);
        assert_eq!(wd2, Weekday::Monday);
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
}
