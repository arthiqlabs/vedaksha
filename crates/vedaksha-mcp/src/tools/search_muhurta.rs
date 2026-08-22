// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `search_muhurta` — search for auspicious time windows (muhurta) within a
//! given period.

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Input parameters for the `search_muhurta` tool.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchMuhurtaInput {
    /// Start of the search window as a Julian Day in **UT1** (Universal
    /// Time) — not TT, not TDB. The engine converts to TT internally for the
    /// dynamical terms and uses UT1 directly for sunrise, which is what each
    /// candidate's vara is reckoned from. Every returned `jd`, `tithi_end_jd`
    /// and `nakshatra_end_jd` is on the same UT1 scale.
    pub start_jd: f64,
    /// End of the search window as a Julian Day in **UT1** (Universal Time) —
    /// not TT, not TDB.
    pub end_jd: f64,
    /// Geographic latitude in degrees \[-90, +90\].
    pub latitude: f64,
    /// Geographic longitude in degrees \[-180, +180\], east positive.
    pub longitude: f64,
    /// Observer elevation above sea level in metres. Defaults to 0.
    ///
    /// A determinant of each candidate's vara, not a refinement of it: the
    /// horizon dip moves sunrise, and the vara is reckoned from sunrise. Same
    /// parameter and same default as `compute_panchanga` — the two tools
    /// must be given the same observer to be expected to agree.
    #[serde(default)]
    pub elevation_m: Option<f64>,
    /// Minimum quality score (0.0–1.0) for a muhurta to be included.
    /// Defaults to 0.5 when absent.
    pub min_quality: Option<f64>,
    /// Offset of the observer's civil clock from UT, in minutes. Names the
    /// vara (weekday) reported for each candidate — it does not affect
    /// which sunrise bounds the vara, only what that vara is called.
    /// Defaults to 0 (UT) when absent.
    #[serde(default)]
    pub tz_offset_minutes: Option<i32>,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "search_muhurta",
        description: "Search for auspicious time windows (muhurta) within a given period for a \
            geographic location. Returns ranked muhurta candidates with quality scores based on \
            tithi, nakshatra, yoga, karana, and planetary positions. The search window is capped \
            at 30 days (see MUHURTA_SEARCH_RANGE_TOO_LARGE) — the per-candidate vara and \
            tithi/nakshatra refinement make this tool far more expensive per day of range than \
            a transit search, so a wider span would make a single call take minutes.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "start_jd": {
                    "type": "number",
                    "description": "Start of the search window as a Julian Day in \
                                    UT1 (Universal Time) — not TT, not TDB. The engine \
                                    converts to TT internally for the dynamical terms and \
                                    uses UT1 directly for sunrise, which each candidate's \
                                    vara is reckoned from; every returned jd, tithi_end_jd \
                                    and nakshatra_end_jd is on this same UT1 scale. The \
                                    span from start_jd to end_jd must not exceed 30 days."
                },
                "end_jd": {
                    "type": "number",
                    "description": "End of the search window as a Julian Day in UT1 \
                                    (Universal Time) — not TT, not TDB. The span from \
                                    start_jd to end_jd must not exceed 30 days."
                },
                "latitude": {
                    "type": "number",
                    "description": "Geographic latitude in degrees [-90, +90]"
                },
                "longitude": {
                    "type": "number",
                    "description": "Geographic longitude in degrees [-180, +180], east positive"
                },
                "elevation_m": {
                    "type": "number",
                    "description": "Observer elevation in metres above sea level [-500, 9000] \
                                    (default 0). Lowers the horizon by the dip and so moves \
                                    sunrise, which is what each candidate's vara is reckoned \
                                    from — at 3650 m (Lhasa) 9.2 minutes earlier. Pass the \
                                    same value as compute_panchanga for the same observer, or \
                                    the two tools can report different weekdays for one \
                                    instant.",
                    "minimum": -500,
                    "maximum": 9000,
                    "default": 0
                },
                "min_quality": {
                    "type": "number",
                    "description": "Minimum quality score [0.0, 1.0] for muhurta inclusion (default 0.5)",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.5
                },
                "tz_offset_minutes": {
                    "type": "integer",
                    "description": "Offset of the observer's civil clock from UT, in minutes. \
                                    Names the vara (weekday) reported for each candidate — it \
                                    does not change which sunrise bounds the vara, only what \
                                    that vara is called. Default 0 (UT).",
                    "minimum": -720,
                    "maximum": 840,
                    "default": 0
                }
            },
            "required": ["start_jd", "end_jd", "latitude", "longitude"]
        }),
    }
}

/// Validate all input fields before computation.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &SearchMuhurtaInput) -> Result<(), McpError> {
    validation::validate_muhurta_search_span(input.start_jd, input.end_jd)?;
    validation::validate_latitude(input.latitude)?;
    validation::validate_longitude(input.longitude)?;

    if let Some(q) = input.min_quality
        && (!q.is_finite() || !(0.0..=1.0).contains(&q))
    {
        return Err(McpError::invalid_parameter(
            "min_quality",
            "min_quality must be a finite number in [0.0, 1.0]",
        ));
    }

    if let Some(tz) = input.tz_offset_minutes {
        validation::validate_tz_offset_minutes(tz)?;
    }

    if let Some(elevation_m) = input.elevation_m {
        validation::validate_elevation_m(elevation_m)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::JD_MIN;

    fn valid_input() -> SearchMuhurtaInput {
        SearchMuhurtaInput {
            start_jd: 2_451_545.0,
            end_jd: 2_451_545.0 + 30.0,
            latitude: 13.08,
            longitude: 80.27,
            elevation_m: None,
            min_quality: None,
            tz_offset_minutes: None,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_invalid_latitude() {
        let mut input = valid_input();
        input.latitude = 91.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LATITUDE");
    }

    #[test]
    fn validate_rejects_invalid_longitude() {
        let mut input = valid_input();
        input.longitude = -200.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_LONGITUDE");
    }

    #[test]
    fn validate_rejects_jd_range_too_large() {
        let mut input = valid_input();
        input.end_jd = input.start_jd + 40_000.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "MUHURTA_SEARCH_RANGE_TOO_LARGE");
    }

    /// FINDING 3. `search_muhurta`'s cap is the tighter
    /// `MAX_MUHURTA_SEARCH_DAYS` (30), not the 100-year
    /// `MAX_TRANSIT_SEARCH_DAYS` `compute_transit`-style tools use — a span
    /// well under the transit cap must still be rejected here.
    #[test]
    fn validate_rejects_a_span_within_the_transit_cap_but_over_the_muhurta_cap() {
        let mut input = valid_input();
        input.end_jd = input.start_jd + 90.0; // < 100 years, > 30 days
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "MUHURTA_SEARCH_RANGE_TOO_LARGE");
    }

    #[test]
    fn validate_accepts_the_muhurta_cap_exactly() {
        let mut input = valid_input();
        input.end_jd = input.start_jd + crate::validation::MAX_MUHURTA_SEARCH_DAYS;
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_start_jd_below_min() {
        let mut input = valid_input();
        input.start_jd = JD_MIN - 1.0;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "DATE_OUT_OF_RANGE");
    }

    #[test]
    fn validate_rejects_min_quality_out_of_range() {
        let mut input = valid_input();
        input.min_quality = Some(1.5);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_min_quality_zero() {
        let mut input = valid_input();
        input.min_quality = Some(0.0);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"start_jd"));
        assert!(names.contains(&"end_jd"));
        assert!(names.contains(&"latitude"));
        assert!(names.contains(&"longitude"));
        // tz_offset_minutes is optional (defaults to 0/UT), not required.
        assert!(!names.contains(&"tz_offset_minutes"));
        // elevation_m likewise defaults to 0 (sea level).
        assert!(!names.contains(&"elevation_m"));
    }

    // --- elevation_m ---

    /// `elevation_m` must be DECLARED in the schema, not merely accepted by
    /// the deserialiser. An agent picks its arguments from the schema, so an
    /// undeclared field is an argument no caller will ever pass — which is
    /// how `search_muhurta` came to derive its vara at sea level while
    /// `compute_panchanga` was elevation-aware.
    #[test]
    fn definition_schema_declares_elevation_m() {
        let def = definition();
        let props = &def.input_schema["properties"];
        assert!(
            props["elevation_m"].is_object(),
            "schema must declare elevation_m"
        );
        assert_eq!(props["elevation_m"]["default"], 0);
        assert_eq!(props["elevation_m"]["minimum"], -500);
        assert_eq!(props["elevation_m"]["maximum"], 9000);
    }

    /// The declared bounds and the executed bounds must be the same bounds —
    /// a schema `minimum`/`maximum` is advisory to the caller, and only
    /// `validate` actually refuses.
    #[test]
    fn validate_accepts_elevation_m_at_the_boundaries() {
        let mut input = valid_input();
        input.elevation_m = Some(crate::validation::ELEVATION_MIN_M);
        assert!(validate(&input).is_ok());
        input.elevation_m = Some(crate::validation::ELEVATION_MAX_M);
        assert!(validate(&input).is_ok());
        input.elevation_m = Some(3650.0); // Lhasa
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_elevation_m_out_of_range() {
        for bad in [
            crate::validation::ELEVATION_MIN_M - 1.0,
            crate::validation::ELEVATION_MAX_M + 1.0,
            1e9,
            f64::NAN,
            f64::INFINITY,
        ] {
            let mut input = valid_input();
            input.elevation_m = Some(bad);
            let err = validate(&input).unwrap_err();
            assert_eq!(err.error_code, "INVALID_PARAMETER", "accepted {bad}");
        }
    }

    #[test]
    fn validate_accepts_missing_elevation_m() {
        assert!(validate(&valid_input()).is_ok());
    }

    /// The field must actually deserialise from the wire under its schema
    /// name — a `#[serde(rename)]` slip or a typo in the schema would leave
    /// the value silently at `None` and the tool silently at sea level.
    #[test]
    fn elevation_m_deserialises_from_the_schema_name() {
        let input: SearchMuhurtaInput = serde_json::from_value(serde_json::json!({
            "start_jd": 2_451_545.0, "end_jd": 2_451_546.0,
            "latitude": 29.65, "longitude": 91.13, "elevation_m": 3650.0
        }))
        .expect("deserialises");
        assert_eq!(input.elevation_m, Some(3650.0));
    }

    #[test]
    fn definition_schema_declares_tz_offset_minutes() {
        let def = definition();
        let props = &def.input_schema["properties"];
        assert!(
            props["tz_offset_minutes"].is_object(),
            "schema must declare tz_offset_minutes"
        );
        assert_eq!(props["tz_offset_minutes"]["default"], 0);
    }

    // --- tz_offset_minutes ---

    #[test]
    fn validate_accepts_missing_tz_offset_minutes() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_accepts_tz_offset_minutes_at_the_boundaries() {
        let mut input = valid_input();
        input.tz_offset_minutes = Some(-720);
        assert!(validate(&input).is_ok());
        input.tz_offset_minutes = Some(840);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_tz_offset_minutes_out_of_range() {
        let mut input = valid_input();
        input.tz_offset_minutes = Some(-721);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");

        let mut input = valid_input();
        input.tz_offset_minutes = Some(841);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }
}
