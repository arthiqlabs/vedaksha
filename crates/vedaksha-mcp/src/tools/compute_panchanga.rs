// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_panchanga` — the five limbs of the Vedic almanac for an instant.

use serde::Deserialize;

use crate::validation::McpError;

#[derive(Debug, Clone, Deserialize)]
pub struct ComputePanchangaInput {
    /// Julian Day (UT). Determines the vara (weekday).
    pub jd: f64,
    /// Sidereal longitude of the Sun, degrees [0, 360).
    pub sun: f64,
    /// Sidereal longitude of the Moon, degrees [0, 360).
    pub moon: f64,
    /// Observer latitude in degrees [−90, +90]. Required for the vara.
    pub latitude: f64,
    /// Observer longitude in degrees [−180, +180], east positive.
    pub longitude: f64,
    /// Observer elevation above sea level in metres. Defaults to 0.
    #[serde(default)]
    pub elevation_m: f64,
    /// Offset of the observer's civil clock from UT, in minutes. Defaults to
    /// 0. Validated against [`crate::validation::validate_tz_offset_minutes`]
    /// (−720..=840) — the same bound `search_muhurta` enforces.
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_panchanga",
        description: "Compute the panchanga — the five limbs of the Vedic almanac — for an \
            instant: tithi (lunar day, with paksha and lord), vara (weekday reckoned from local \
            sunrise, with its lord and the Rahu and Gulika Kalam windows as Julian Days), \
            nakshatra (with pada), yoga (one of the 27 nithya yogas, with degrees remaining), \
            and karana (half-tithi). Takes sidereal longitudes; all returned instants are \
            Julian Days (UT). vara.from_sunrise reports HOW the weekday was reckoned: true \
            means it was taken from an actual local sunrise (the Vedic definition); false means \
            no sunrise exists to reckon from — the polar day or polar night, above about ±66.5° \
            latitude — and the value is the observer's local CIVIL weekday as a documented \
            fallback, which is a different quantity. Check it before presenting the vara at high \
            latitude. vara.rahu_kalam being null is NOT the same signal: the Kalam windows can \
            also be null while from_sunrise is true.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "jd":   { "type": "number", "description": "Julian Day (UT), used for the vara" },
                "sun":  { "type": "number", "description": "Sidereal longitude of Sun [0, 360)" },
                "moon": { "type": "number", "description": "Sidereal longitude of Moon [0, 360)" },
                "latitude": {
                    "type": "number", "minimum": -90, "maximum": 90,
                    "description": "Observer latitude in degrees. Required — the vara is \
                                    reckoned from local sunrise, so it depends on the observer."
                },
                "longitude": {
                    "type": "number", "minimum": -180, "maximum": 180,
                    "description": "Observer longitude in degrees, east positive. Required."
                },
                "elevation_m": {
                    "type": "number", "default": 0,
                    "minimum": -500, "maximum": 9000,
                    "description": "Observer elevation in metres above sea level [-500, 9000]; \
                                    lowers the horizon by the dip and so moves sunrise — at \
                                    3650 m (Lhasa) 9.2 minutes earlier, enough to change \
                                    the vara in that window."
                },
                "tz_offset_minutes": {
                    "type": "integer", "default": 0,
                    "minimum": -720, "maximum": 840,
                    "description": "Offset of the observer's civil clock from UT, in minutes, \
                                    in [-720, 840] (UTC-12:00 to UTC+14:00). Used only to name \
                                    the vara's weekday."
                }
            },
            "required": ["jd", "sun", "moon", "latitude", "longitude"]
        }),
    }
}

fn validate_lon(name: &'static str, value: f64) -> Result<(), McpError> {
    if !value.is_finite() || !(0.0..360.0).contains(&value) {
        return Err(McpError::invalid_parameter(
            name,
            "must be a finite number in [0, 360)",
        ));
    }
    Ok(())
}

/// Validate a [`ComputePanchangaInput`].
///
/// # Errors
/// Returns [`McpError`] when `jd` is non-finite or a longitude is outside [0, 360).
pub fn validate(input: &ComputePanchangaInput) -> Result<(), McpError> {
    if !input.jd.is_finite() {
        return Err(McpError::invalid_parameter("jd", "must be a finite number"));
    }
    validate_lon("sun", input.sun)?;
    validate_lon("moon", input.moon)?;
    if !input.latitude.is_finite() || !(-90.0..=90.0).contains(&input.latitude) {
        return Err(McpError::invalid_parameter(
            "latitude",
            "must be a finite number in [-90, 90]",
        ));
    }
    if !input.longitude.is_finite() || !(-180.0..=180.0).contains(&input.longitude) {
        return Err(McpError::invalid_parameter(
            "longitude",
            "must be a finite number in [-180, 180]",
        ));
    }
    crate::validation::validate_elevation_m(input.elevation_m)?;
    crate::validation::validate_tz_offset_minutes(input.tz_offset_minutes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ComputePanchangaInput {
        ComputePanchangaInput {
            jd: 2_451_545.0,
            sun: 280.0,
            moon: 220.0,
            latitude: 13.08,
            longitude: 80.27,
            elevation_m: 0.0,
            tz_offset_minutes: 0,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn validate_rejects_non_finite_jd() {
        let mut input = valid_input();
        input.jd = f64::INFINITY;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_longitude_at_360() {
        let mut input = valid_input();
        input.moon = 360.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_panchanga");
        let required = def.input_schema["required"].as_array().unwrap();
        let names: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"jd"));
        assert!(names.contains(&"sun"));
        assert!(names.contains(&"moon"));
        assert!(names.contains(&"latitude"));
        assert!(names.contains(&"longitude"));
    }

    #[test]
    fn validate_rejects_out_of_range_latitude() {
        let mut input = valid_input();
        input.latitude = 91.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_out_of_range_longitude() {
        let mut input = valid_input();
        input.longitude = 200.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_non_finite_elevation() {
        let mut input = valid_input();
        input.elevation_m = f64::NAN;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    /// FIX: `tz_offset_minutes` was previously unvalidated here — the schema
    /// declared no bounds and `validate` never called
    /// `validate_tz_offset_minutes`, so `999_999_999` (or `841`/`-721`)
    /// passed straight through to weekday naming. Mirrors
    /// `search_muhurta`'s equivalent boundary test.
    #[test]
    fn validate_rejects_tz_offset_minutes_out_of_range() {
        let mut input = valid_input();
        input.tz_offset_minutes = 841;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );

        let mut input = valid_input();
        input.tz_offset_minutes = -721;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );

        let mut input = valid_input();
        input.tz_offset_minutes = 999_999_999;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_accepts_tz_offset_minutes_at_the_boundaries() {
        let mut input = valid_input();
        input.tz_offset_minutes = -720;
        assert!(validate(&input).is_ok());
        input.tz_offset_minutes = 840;
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_schema_declares_tz_offset_minutes_bounds() {
        let def = definition();
        let props = &def.input_schema["properties"];
        assert_eq!(props["tz_offset_minutes"]["minimum"], -720);
        assert_eq!(props["tz_offset_minutes"]["maximum"], 840);
    }
}
