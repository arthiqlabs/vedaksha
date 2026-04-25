// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Angle normalization and conversion utilities.
//!
//! Provides functions to normalize angles to standard ranges and convert
//! between degrees, radians, DMS (degrees-minutes-seconds), and HMS
//! (hours-minutes-seconds) representations.
//!
//! Source: Standard trigonometric identities.

use core::f64::consts::PI;

/// Degrees-minutes-seconds representation of an angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dms {
    pub sign: i8,
    pub degrees: u32,
    pub minutes: u32,
    pub seconds: f64,
}

/// Hours-minutes-seconds representation of an angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hms {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: f64,
}

/// Normalize an angle in degrees to the range [0, 360).
///
/// Returns `0.0` for NaN or infinite inputs.
#[must_use]
pub fn normalize_degrees(angle: f64) -> f64 {
    if !angle.is_finite() {
        return 0.0;
    }
    let result = angle % 360.0;
    let result = if result < 0.0 { result + 360.0 } else { result };
    // Guard against floating-point modulo returning exactly 360.0
    if result >= 360.0 { 0.0 } else { result }
}

/// Normalize an angle in degrees to the range [-180, 180).
///
/// Returns `0.0` for NaN or infinite inputs.
#[must_use]
pub fn normalize_degrees_signed(angle: f64) -> f64 {
    if !angle.is_finite() {
        return 0.0;
    }
    let mut result = normalize_degrees(angle);
    if result >= 180.0 {
        result -= 360.0;
    }
    result
}

/// Normalize an angle in radians to the range [0, 2π).
///
/// Returns `0.0` for NaN or infinite inputs.
#[must_use]
pub fn normalize_radians(angle: f64) -> f64 {
    if !angle.is_finite() {
        return 0.0;
    }
    let two_pi = 2.0 * PI;
    let result = angle % two_pi;
    let result = if result < 0.0 {
        result + two_pi
    } else {
        result
    };
    // Guard against floating-point modulo returning exactly 2*pi
    if result >= two_pi { 0.0 } else { result }
}

/// Convert degrees to radians.
#[must_use]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * (PI / 180.0)
}

/// Convert radians to degrees.
#[must_use]
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * (180.0 / PI)
}

/// Convert decimal degrees to DMS (degrees-minutes-seconds).
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn deg_to_dms(decimal_degrees: f64) -> Dms {
    let sign = if decimal_degrees < 0.0 { -1i8 } else { 1i8 };
    let abs_deg = decimal_degrees.abs();
    let mut degrees = abs_deg as u32;
    let remainder = (abs_deg - f64::from(degrees)) * 60.0;
    let mut minutes = remainder as u32;
    let mut seconds = (remainder - f64::from(minutes)) * 60.0;
    // Carry propagation for floating-point accumulation edge cases
    if seconds >= 60.0 {
        seconds -= 60.0;
        minutes += 1;
    }
    if minutes >= 60 {
        minutes -= 60;
        degrees += 1;
    }
    Dms {
        sign,
        degrees,
        minutes,
        seconds,
    }
}

/// Convert DMS (degrees-minutes-seconds) to decimal degrees.
#[must_use]
pub fn dms_to_deg(dms: &Dms) -> f64 {
    let abs_val = f64::from(dms.degrees) + f64::from(dms.minutes) / 60.0 + dms.seconds / 3600.0;
    f64::from(dms.sign) * abs_val
}

/// Convert decimal degrees to HMS (hours-minutes-seconds).
///
/// Normalizes to [0, 360) first. Uses the convention that 1 hour = 15 degrees.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn deg_to_hms(decimal_degrees: f64) -> Hms {
    let normalized = normalize_degrees(decimal_degrees);
    let total_hours = normalized / 15.0;
    let mut hours = total_hours as u32;
    let remainder = (total_hours - f64::from(hours)) * 60.0;
    let mut minutes = remainder as u32;
    let mut seconds = (remainder - f64::from(minutes)) * 60.0;
    // Carry propagation for floating-point accumulation edge cases
    if seconds >= 60.0 {
        seconds -= 60.0;
        minutes += 1;
    }
    if minutes >= 60 {
        minutes -= 60;
        hours += 1;
    }
    Hms {
        hours,
        minutes,
        seconds,
    }
}

/// Convert HMS (hours-minutes-seconds) to decimal degrees.
#[must_use]
pub fn hms_to_deg(hms: &Hms) -> f64 {
    (f64::from(hms.hours) + f64::from(hms.minutes) / 60.0 + hms.seconds / 3600.0) * 15.0
}

/// Compute the shortest angular separation between two angles in degrees.
///
/// Both inputs are treated as degree values. The result is in [0, 180].
#[must_use]
pub fn angular_separation(a: f64, b: f64) -> f64 {
    let diff = normalize_degrees(a - b);
    if diff > 180.0 { 360.0 - diff } else { diff }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    // --- normalize_degrees ---

    #[test]
    fn normalize_degrees_zero() {
        assert!((normalize_degrees(0.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_positive() {
        assert!((normalize_degrees(45.0) - 45.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_360_becomes_0() {
        assert!((normalize_degrees(360.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_negative() {
        assert!((normalize_degrees(-90.0) - 270.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_large_positive() {
        assert!((normalize_degrees(720.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_large_negative() {
        assert!((normalize_degrees(-450.0) - 270.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_near_360() {
        assert!((normalize_degrees(359.9999) - 359.9999).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_nan() {
        assert!((normalize_degrees(f64::NAN) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_inf() {
        assert!((normalize_degrees(f64::INFINITY) - 0.0).abs() < EPS);
    }

    // --- normalize_degrees_signed ---

    #[test]
    fn normalize_degrees_signed_positive() {
        assert!((normalize_degrees_signed(90.0) - 90.0).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_signed_negative() {
        assert!((normalize_degrees_signed(-45.0) - (-45.0)).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_signed_270_becomes_neg90() {
        assert!((normalize_degrees_signed(270.0) - (-90.0)).abs() < EPS);
    }

    #[test]
    fn normalize_degrees_signed_180() {
        // 180 >= 180, so becomes -180
        assert!((normalize_degrees_signed(180.0) - (-180.0)).abs() < EPS);
    }

    // --- normalize_radians ---

    #[test]
    fn normalize_radians_zero() {
        assert!((normalize_radians(0.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_radians_pi() {
        assert!((normalize_radians(PI) - PI).abs() < EPS);
    }

    #[test]
    fn normalize_radians_two_pi() {
        assert!((normalize_radians(2.0 * PI) - 0.0).abs() < EPS);
    }

    #[test]
    fn normalize_radians_negative() {
        assert!((normalize_radians(-PI) - PI).abs() < EPS);
    }

    // --- deg_to_rad / rad_to_deg ---

    #[test]
    fn deg_rad_roundtrip() {
        let angle = 123.456_f64;
        assert!((rad_to_deg(deg_to_rad(angle)) - angle).abs() < EPS);
    }

    #[test]
    fn deg_rad_90_is_half_pi() {
        assert!((deg_to_rad(90.0) - PI / 2.0).abs() < EPS);
    }

    // --- DMS ---

    #[test]
    fn dms_positive() {
        let dms = deg_to_dms(45.5);
        assert_eq!(dms.sign, 1);
        assert_eq!(dms.degrees, 45);
        assert_eq!(dms.minutes, 30);
        assert!((dms.seconds - 0.0).abs() < 0.01);
    }

    #[test]
    fn dms_negative() {
        let dms = deg_to_dms(-10.25);
        assert_eq!(dms.sign, -1);
        assert_eq!(dms.degrees, 10);
        assert_eq!(dms.minutes, 15);
        assert!((dms.seconds - 0.0).abs() < 0.01);
    }

    #[test]
    fn dms_roundtrip() {
        let original = 37.123_456_f64;
        let dms = deg_to_dms(original);
        let back = dms_to_deg(&dms);
        assert!((back - original).abs() < 1e-10);
    }

    #[test]
    fn dms_zero() {
        let dms = deg_to_dms(0.0);
        assert_eq!(dms.sign, 1);
        assert_eq!(dms.degrees, 0);
        assert_eq!(dms.minutes, 0);
        assert!((dms.seconds - 0.0).abs() < 0.01);
    }

    // --- HMS ---

    #[test]
    fn hms_zero() {
        let hms = deg_to_hms(0.0);
        assert_eq!(hms.hours, 0);
        assert_eq!(hms.minutes, 0);
        assert!((hms.seconds - 0.0).abs() < 0.01);
    }

    #[test]
    fn hms_90deg() {
        let hms = deg_to_hms(90.0);
        assert_eq!(hms.hours, 6);
        assert_eq!(hms.minutes, 0);
        assert!((hms.seconds - 0.0).abs() < 0.01);
    }

    #[test]
    fn hms_roundtrip() {
        let original = 135.0_f64;
        let hms = deg_to_hms(original);
        let back = hms_to_deg(&hms);
        assert!((back - original).abs() < 1e-10);
    }

    // --- angular_separation ---

    #[test]
    fn separation_same() {
        assert!((angular_separation(45.0, 45.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn separation_opposite() {
        assert!((angular_separation(0.0, 180.0) - 180.0).abs() < EPS);
    }

    #[test]
    fn separation_wrap_around() {
        // 10 and 350 are 20 degrees apart going the short way
        assert!((angular_separation(10.0, 350.0) - 20.0).abs() < EPS);
    }

    #[test]
    fn separation_negative_input() {
        // -10 normalizes to 350; separation from 10 is 20
        assert!((angular_separation(-10.0, 10.0) - 20.0).abs() < EPS);
    }
}
