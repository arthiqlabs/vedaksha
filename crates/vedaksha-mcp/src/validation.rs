// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Input validation for all MCP tool parameters.
//!
//! Every tool call goes through validation before computation.

use serde::{Deserialize, Serialize};

/// Valid Julian Day range: 1800–2400 CE.
pub const JD_MIN: f64 = 2_378_496.5; // ~1800 CE
/// Valid Julian Day range: 1800–2400 CE.
pub const JD_MAX: f64 = 2_597_641.5; // ~2400 CE

/// Maximum transit search span in days (100 years).
pub const MAX_TRANSIT_SEARCH_DAYS: f64 = 36_525.0;

/// Maximum `search_muhurta` search span in days.
///
/// `search_muhurta` is far more expensive per day of range than a transit
/// search: every 0.5-day step needs Sun/Moon sidereal longitudes, and each
/// distinct vara needs a `vara_with_validity` call (`vedaksha_vedic::muhurta`)
/// — two ~288-step, 5-minute-resolution `sun_rise_set` scans (walk-back
/// for the opening sunrise, walk-forward for the closing one), plus a
/// `compute_tithi_end`/`compute_nakshatra_end` refinement for every window
/// that passes `min_quality`. Measured directly in release mode (not
/// extrapolated): a real 30-day `search_muhurta` call at Chennai (lat 13.08,
/// lon 80.27) via the full MCP handler took ~15.9 s — about 530 ms per day
/// of range. [`MAX_TRANSIT_SEARCH_DAYS`] (100 years) applied to this tool
/// would be ~36,525 × 0.53 s ≈ 5.4 **hours**, which is a tool call that
/// would appear to hang rather than fail fast. 30 days keeps worst-case
/// latency at the ~16 s actually measured — slow for a synchronous call but
/// not indefinite — and still comfortably covers the practical use case
/// (electing an auspicious date within the next few weeks); a caller
/// wanting a longer horizon should issue several shorter searches rather
/// than one that risks timing out its own client.
pub const MAX_MUHURTA_SEARCH_DAYS: f64 = 30.0;

/// Structured MCP error response with a machine-readable code and
/// an optional hint for the calling agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub error_code: String,
    pub message: String,
    pub suggested_action: Option<String>,
}

impl McpError {
    /// Julian Day is outside the supported range.
    #[must_use]
    pub fn date_out_of_range(jd: f64) -> Self {
        Self {
            error_code: "DATE_OUT_OF_RANGE".into(),
            message: format!("Julian Day {jd} is outside valid range [{JD_MIN}, {JD_MAX}]"),
            suggested_action: Some("Provide a Julian Day between 1800 and 2400 CE.".into()),
        }
    }

    /// Geographic latitude is outside [-90, +90].
    #[must_use]
    pub fn invalid_latitude(lat: f64) -> Self {
        Self {
            error_code: "INVALID_LATITUDE".into(),
            message: format!("Latitude {lat} is outside valid range [-90, +90]"),
            suggested_action: Some("Provide latitude in degrees between -90 and +90.".into()),
        }
    }

    /// Geographic longitude is outside [-180, +180].
    #[must_use]
    pub fn invalid_longitude(lon: f64) -> Self {
        Self {
            error_code: "INVALID_LONGITUDE".into(),
            message: format!("Longitude {lon} is outside valid range [-180, +180]"),
            suggested_action: Some("Provide longitude in degrees between -180 and +180.".into()),
        }
    }

    /// Transit search window is larger than 100 years.
    #[must_use]
    pub fn search_range_too_large(days: f64) -> Self {
        Self {
            error_code: "SEARCH_RANGE_TOO_LARGE".into(),
            message: format!(
                "Transit search span {days} days exceeds maximum {MAX_TRANSIT_SEARCH_DAYS}"
            ),
            suggested_action: Some("Limit search range to 100 years (36525 days) or less.".into()),
        }
    }

    /// Muhurta search window is larger than [`MAX_MUHURTA_SEARCH_DAYS`]. A
    /// distinct error code from [`Self::search_range_too_large`] because the
    /// two tools have very different per-day costs and so very different
    /// limits — see [`MAX_MUHURTA_SEARCH_DAYS`] for the derivation.
    #[must_use]
    pub fn muhurta_search_range_too_large(days: f64) -> Self {
        Self {
            error_code: "MUHURTA_SEARCH_RANGE_TOO_LARGE".into(),
            message: format!(
                "Muhurta search span {days} days exceeds maximum {MAX_MUHURTA_SEARCH_DAYS}"
            ),
            suggested_action: Some(format!(
                "Limit the muhurta search range to {MAX_MUHURTA_SEARCH_DAYS} days or less, or \
                 issue several shorter searches."
            )),
        }
    }

    /// A named parameter has an invalid value.
    #[must_use]
    pub fn invalid_parameter(param: &str, detail: &str) -> Self {
        Self {
            error_code: "INVALID_PARAMETER".into(),
            message: format!("Invalid parameter '{param}': {detail}"),
            suggested_action: None,
        }
    }

    /// The underlying computation produced an error.
    #[must_use]
    pub fn computation_failed(detail: &str) -> Self {
        Self {
            error_code: "COMPUTATION_FAILED".into(),
            message: format!("Computation failed: {detail}"),
            suggested_action: Some("Check input parameters and try again.".into()),
        }
    }
}

/// Validate that a Julian Day is within the supported range.
///
/// # Errors
///
/// Returns [`McpError::date_out_of_range`] when `jd` is non-finite or
/// outside [`JD_MIN`]…[`JD_MAX`].
pub fn validate_jd(jd: f64) -> Result<(), McpError> {
    if !jd.is_finite() || !(JD_MIN..=JD_MAX).contains(&jd) {
        Err(McpError::date_out_of_range(jd))
    } else {
        Ok(())
    }
}

/// Validate that a geographic latitude is in the range [-90, +90].
///
/// # Errors
///
/// Returns [`McpError::invalid_latitude`] when `lat` is non-finite or
/// outside the valid range.
pub fn validate_latitude(lat: f64) -> Result<(), McpError> {
    if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
        Err(McpError::invalid_latitude(lat))
    } else {
        Ok(())
    }
}

/// Validate that a geographic longitude is in the range [-180, +180].
///
/// # Errors
///
/// Returns [`McpError::invalid_longitude`] when `lon` is non-finite or
/// outside the valid range.
pub fn validate_longitude(lon: f64) -> Result<(), McpError> {
    if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
        Err(McpError::invalid_longitude(lon))
    } else {
        Ok(())
    }
}

/// Validate a transit search window defined by two Julian Days.
///
/// Both endpoints are validated individually and the absolute span must not
/// exceed [`MAX_TRANSIT_SEARCH_DAYS`].
///
/// # Errors
///
/// Returns the first validation error encountered.
pub fn validate_search_span(start_jd: f64, end_jd: f64) -> Result<(), McpError> {
    validate_jd(start_jd)?;
    validate_jd(end_jd)?;
    let span = (end_jd - start_jd).abs();
    if span > MAX_TRANSIT_SEARCH_DAYS {
        Err(McpError::search_range_too_large(span))
    } else {
        Ok(())
    }
}

/// Validate a `search_muhurta` window defined by two Julian Days.
///
/// Both endpoints are validated individually and the absolute span must not
/// exceed [`MAX_MUHURTA_SEARCH_DAYS`] — a much tighter bound than
/// [`validate_search_span`]'s [`MAX_TRANSIT_SEARCH_DAYS`], because
/// `search_muhurta` costs roughly two orders of magnitude more per day of
/// range (see [`MAX_MUHURTA_SEARCH_DAYS`]'s derivation).
///
/// # Errors
///
/// Returns the first validation error encountered.
pub fn validate_muhurta_search_span(start_jd: f64, end_jd: f64) -> Result<(), McpError> {
    validate_jd(start_jd)?;
    validate_jd(end_jd)?;
    let span = (end_jd - start_jd).abs();
    if span > MAX_MUHURTA_SEARCH_DAYS {
        Err(McpError::muhurta_search_range_too_large(span))
    } else {
        Ok(())
    }
}

/// Validate a civil-clock offset from UT, in minutes.
///
/// Real-world UTC offsets range from −12:00 (e.g. Baker Island) to +14:00
/// (e.g. Kiribati's Line Islands), i.e. −720..=840 minutes. A wider value
/// would not correspond to any real observer's civil clock, so it is
/// rejected rather than silently accepted and fed into a weekday-naming
/// calculation that could never match a real calendar.
///
/// # Errors
///
/// Returns [`McpError::invalid_parameter`] when `minutes` is outside
/// −720..=840.
pub fn validate_tz_offset_minutes(minutes: i32) -> Result<(), McpError> {
    if (-720..=840).contains(&minutes) {
        Ok(())
    } else {
        Err(McpError::invalid_parameter(
            "tz_offset_minutes",
            "must be between -720 and 840 (UTC-12:00 to UTC+14:00)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_jd ──────────────────────────────────────────────────────────

    #[test]
    fn validate_jd_accepts_j2000() {
        // J2000.0 = JD 2 451 545.0
        assert!(validate_jd(2_451_545.0).is_ok());
    }

    #[test]
    fn validate_jd_accepts_boundary_values() {
        assert!(validate_jd(JD_MIN).is_ok());
        assert!(validate_jd(JD_MAX).is_ok());
    }

    #[test]
    fn validate_jd_rejects_below_min() {
        let err = validate_jd(JD_MIN - 1.0).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_jd_rejects_above_max() {
        let err = validate_jd(JD_MAX + 1.0).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_jd_rejects_nan() {
        let err = validate_jd(f64::NAN).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_jd_rejects_positive_infinity() {
        let err = validate_jd(f64::INFINITY).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_jd_rejects_negative_infinity() {
        let err = validate_jd(f64::NEG_INFINITY).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    // ── validate_latitude ────────────────────────────────────────────────────

    #[test]
    fn validate_latitude_accepts_equator() {
        assert!(validate_latitude(0.0).is_ok());
    }

    #[test]
    fn validate_latitude_accepts_poles() {
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(-90.0).is_ok());
    }

    #[test]
    fn validate_latitude_rejects_above_90() {
        let err = validate_latitude(91.0).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_latitude_rejects_below_minus_90() {
        let err = validate_latitude(-91.0).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_latitude_rejects_nan() {
        let err = validate_latitude(f64::NAN).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    // ── validate_longitude ───────────────────────────────────────────────────

    #[test]
    fn validate_longitude_accepts_prime_meridian() {
        assert!(validate_longitude(0.0).is_ok());
    }

    #[test]
    fn validate_longitude_accepts_boundaries() {
        assert!(validate_longitude(180.0).is_ok());
        assert!(validate_longitude(-180.0).is_ok());
    }

    #[test]
    fn validate_longitude_rejects_above_180() {
        let err = validate_longitude(181.0).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    #[test]
    fn validate_longitude_rejects_nan() {
        let err = validate_longitude(f64::NAN).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    // ── validate_search_span ─────────────────────────────────────────────────

    #[test]
    fn validate_search_span_rejects_200_year_window() {
        let start = 2_451_545.0; // J2000
        let end = start + 2.0 * MAX_TRANSIT_SEARCH_DAYS; // 200 years
        let err = validate_search_span(start, end).unwrap_err();
        assert_eq!(err.error_code, "SEARCH_RANGE_TOO_LARGE");
    }

    #[test]
    fn validate_search_span_accepts_one_year() {
        let start = 2_451_545.0;
        let end = start + 365.25;
        assert!(validate_search_span(start, end).is_ok());
    }

    // ── validate_muhurta_search_span ────────────────────────────────────────

    #[test]
    fn validate_muhurta_search_span_accepts_the_limit() {
        let start = 2_451_545.0;
        let end = start + MAX_MUHURTA_SEARCH_DAYS;
        assert!(validate_muhurta_search_span(start, end).is_ok());
    }

    #[test]
    fn validate_muhurta_search_span_rejects_just_over_the_limit() {
        let start = 2_451_545.0;
        let end = start + MAX_MUHURTA_SEARCH_DAYS + 1.0;
        let err = validate_muhurta_search_span(start, end).unwrap_err();
        assert_eq!(err.error_code, "MUHURTA_SEARCH_RANGE_TOO_LARGE");
    }

    #[test]
    fn validate_muhurta_search_span_rejects_a_span_that_validate_search_span_would_accept() {
        // Proves the two limits are actually different, not just named
        // differently: a 1-year window (well under MAX_TRANSIT_SEARCH_DAYS)
        // must still be rejected here.
        let start = 2_451_545.0;
        let end = start + 365.25;
        assert!(validate_search_span(start, end).is_ok());
        assert!(validate_muhurta_search_span(start, end).is_err());
    }

    // ── validate_tz_offset_minutes ──────────────────────────────────────────

    #[test]
    fn validate_tz_offset_minutes_accepts_zero() {
        assert!(validate_tz_offset_minutes(0).is_ok());
    }

    #[test]
    fn validate_tz_offset_minutes_accepts_the_boundaries() {
        assert!(validate_tz_offset_minutes(-720).is_ok());
        assert!(validate_tz_offset_minutes(840).is_ok());
    }

    #[test]
    fn validate_tz_offset_minutes_rejects_beyond_the_boundaries() {
        assert_eq!(
            validate_tz_offset_minutes(-721).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
        assert_eq!(
            validate_tz_offset_minutes(841).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }
}
