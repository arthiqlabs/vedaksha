// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Julian Day number utilities.
//!
//! Algorithms from Meeus, *Astronomical Algorithms*, 2nd ed., Chapter 7.

/// Julian Day number of the J2000.0 epoch (2000 January 1.5 TT).
pub const J2000: f64 = 2_451_545.0;

/// Number of Julian days in a Julian century.
const DAYS_PER_CENTURY: f64 = 36_525.0;

/// Returns the number of Julian centuries elapsed since J2000.0.
///
/// # Arguments
/// * `jd` — Julian Day number (TT or TDB)
#[must_use]
pub fn centuries_from_j2000(jd: f64) -> f64 {
    (jd - J2000) / DAYS_PER_CENTURY
}

/// Converts a proleptic Gregorian (or Julian) calendar date to a Julian Day number.
///
/// Uses Meeus eq. 7.1. For dates before 1582 October 15 the proleptic Julian
/// calendar is assumed; Gregorian thereafter.
///
/// # Arguments
/// * `year`  — astronomical year (negative for BCE, 0 = 1 BCE)
/// * `month` — month number 1–12
/// * `day`   — day with fractional hours (e.g. 4.81 for 4th at 19h 26m 24s)
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
pub fn calendar_to_jd(year: i32, month: u32, day: f64) -> f64 {
    let (yr, mo) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    // Determine whether the date falls in the Gregorian or Julian calendar.
    // The Gregorian calendar started on 1582 October 15.
    let b = if yr > 1582 || (yr == 1582 && mo > 10) || (yr == 1582 && mo == 10 && day >= 15.0) {
        let a_val = (f64::from(yr) / 100.0).floor() as i64;
        2 - a_val + (a_val / 4)
    } else {
        0_i64
    };

    (365.25_f64 * (f64::from(yr) + 4716.0)).floor()
        + (30.6001_f64 * (f64::from(mo) + 1.0)).floor()
        + day
        + b as f64
        - 1524.5
}

/// Converts a Julian Day number to a proleptic Gregorian calendar date.
///
/// Returns `(year, month, day)` where `day` carries the fractional part
/// (time of day). Inverse of [`calendar_to_jd`].
///
/// # Arguments
/// * `jd` — Julian Day number
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn jd_to_calendar(jd: f64) -> (i32, u32, f64) {
    let jd_shifted = jd + 0.5;
    let z = jd_shifted.floor() as i64;
    let frac = jd_shifted - jd_shifted.floor();

    let a = if z < 2_299_161 {
        z
    } else {
        let alpha = ((z as f64 - 1_867_216.25) / 36_524.25).floor() as i64;
        z + 1 + alpha - alpha / 4
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as f64 + frac;

    let month_raw = if e < 14 { e - 1 } else { e - 13 };
    // month_raw is always 1–12 by construction from Meeus algorithm
    let month = month_raw as u32;

    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for floating-point JD comparisons (< 1 second).
    const EPS: f64 = 1e-5;

    #[test]
    fn j2000_epoch() {
        let jd = calendar_to_jd(2000, 1, 1.5);
        assert!(
            (jd - J2000).abs() < EPS,
            "J2000 epoch: expected {J2000}, got {jd}"
        );
    }

    /// Meeus example 7a: 1957 October 4.81 → JD 2_436_116.31
    #[test]
    fn meeus_example_7a() {
        let jd = calendar_to_jd(1957, 10, 4.81);
        assert!(
            (jd - 2_436_116.31).abs() < EPS,
            "Meeus 7a: expected 2436116.31, got {jd}"
        );
    }

    /// Meeus example 7b: 333 January 27.5 → JD 1_842_713.0
    #[test]
    fn meeus_example_7b() {
        let jd = calendar_to_jd(333, 1, 27.5);
        assert!(
            (jd - 1_842_713.0).abs() < EPS,
            "Meeus 7b: expected 1842713.0, got {jd}"
        );
    }

    #[test]
    fn jd_roundtrip() {
        let orig_year = 2024_i32;
        let orig_month = 6_u32;
        let orig_day = 21.75_f64;
        let jd = calendar_to_jd(orig_year, orig_month, orig_day);
        let (year, month, day) = jd_to_calendar(jd);
        assert_eq!(year, orig_year);
        assert_eq!(month, orig_month);
        assert!((day - orig_day).abs() < EPS, "day roundtrip: {day}");
    }

    #[test]
    fn centuries_at_j2000() {
        let t = centuries_from_j2000(J2000);
        assert!(t.abs() < f64::EPSILON, "expected 0.0, got {t}");
    }

    #[test]
    fn centuries_one_century_later() {
        let jd = J2000 + 36_525.0;
        let t = centuries_from_j2000(jd);
        assert!((t - 1.0).abs() < f64::EPSILON, "expected 1.0, got {t}");
    }
}
