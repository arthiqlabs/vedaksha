// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_shadbala` — the complete six-fold strength of the seven grahas for
//! a moment and a place, BPHS chapter 27, with Ishta and Kashta Phala from
//! chapter 28.
//!
//! The tool casts the chart itself (the routine `compute_natal_chart` uses) and
//! reads the birth moment from the engine's own sunrise and sunset for the
//! place; see `vedaksha_vedic::shadbala_chart` for where each input comes from
//! and `vedaksha_vedic::shadbala` for every rule.
//!
//! Through the release before this one the tool took the grahas as caller
//! supplied rows (sign, longitude, bhava, speed, two aspect counts) and two
//! booleans, and served a partial answer: no Saptavargaja, Kala Bala as two
//! 0-or-30 steps, Cheshta Bala from a speed grade, Dig Bala a step per house,
//! Drik Bala fifteen virupas per aspect. It now serves every part the verses
//! give, each in its own field, so the input is the moment and the place.

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Input parameters for the `compute_shadbala` tool.
///
/// `deny_unknown_fields` so that a call written for the earlier shape (a
/// `planets` array) is refused by name instead of being read as a chart with
/// no moment.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeShadbalaInput {
    /// Julian Day (UT1) of the birth.
    pub julian_day: f64,
    /// Geographic latitude, degrees, north positive.
    pub latitude: f64,
    /// Geographic longitude, degrees, east positive.
    pub longitude: f64,
    /// Ayanamsha of the sidereal zodiac the strengths are read in.
    pub ayanamsha: String,
    /// House system whose cusps Dig Bala is measured from and whose house
    /// placements give Kendradi Bala. Defaults to Placidus.
    pub house_system: Option<String>,
    /// Observer elevation above sea level, metres. Defaults to 0.
    #[serde(default)]
    pub elevation_m: f64,
    /// Offset of the observer's civil clock from UT, minutes. Defaults to 0.
    #[serde(default)]
    pub tz_offset_minutes: i32,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_shadbala",
        description: "Compute the complete six-fold Shadbala of the seven grahas (Sun to \
            Saturn) for a birth moment and place, per BPHS Ch.27, with Ishta and Kashta Phala \
            per Ch.28, in virupas. Pass the moment and the place; the tool casts the chart \
            itself and reads sunrise, sunset and the next sunrise from the engine. Every part \
            is in the output, each in its own field: Sthana (uccha, saptavargaja, ojhayugma, \
            kendradi, drekkana), Dig (continuous, the arc from the weak cusp over 3, 27.7), \
            Kala (nathonnata, paksha, tribhaga, abda, masa, vara, hora, ayana and yuddha, \
            which is signed), Cheshta (the cheshta kendra for Mars to Saturn, the Sun's ayana \
            and the Moon's paksha bala, 27.18 and 27.24-25), Naisargika and Drik (a quarter \
            of the benefic sphuta drishti added, a quarter of the malefic taken away, the \
            aspect of Mercury and Jupiter added whole, 27.19; it can be negative or pass 60). \
            The Sun's ayana and the Moon's paksha bala are counted once in kala and once in \
            cheshta, each undoubled. The rules are the verses' defaults; the benefic and \
            malefic grahas are those of BPHS 3.11, with Rahu (the mean node) and Ketu counting \
            only as malefic company for Mercury. Each graha also carries its motion state \
            (vakra, anuvakra, ... atichara, 27.21-23) against the mean geocentric motion; it \
            is reported beside the strengths and is not part of any total. The Sun must both \
            rise and set at the place: in the polar day or night there is no Hindu day to \
            divide into thirds and horas, and the call fails by name.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "julian_day": {
                    "type": "number",
                    "description": "Julian Day number in UT1 (Universal Time) — not TT, not \
                        TDB, i.e. the same scale as compute_natal_chart's julian_day."
                },
                "latitude": {
                    "type": "number", "minimum": -90, "maximum": 90,
                    "description": "Geographic latitude in degrees, north positive. The \
                        Hindu day is reckoned from this place's sunrise."
                },
                "longitude": {
                    "type": "number", "minimum": -180, "maximum": 180,
                    "description": "Geographic longitude in degrees, east positive."
                },
                "ayanamsha": {
                    "type": "string",
                    "description": crate::tools::ayanamsha_sidereal_schema_description(),
                    "enum": crate::tools::ayanamsha_sidereal_schema_enum()
                },
                "house_system": {
                    "type": "string",
                    "description": "House system whose first, fourth, seventh and tenth \
                        cusps Dig Bala is measured from, and whose house placements give \
                        Kendradi Bala: Placidus, Koch, Equal, WholeSign, etc. The same \
                        names compute_natal_chart takes.",
                    "default": "Placidus"
                },
                "elevation_m": {
                    "type": "number", "default": 0,
                    "minimum": -500, "maximum": 9000,
                    "description": "Observer elevation in metres above sea level \
                        [-500, 9000] (default 0). Lowers the horizon by the dip and so \
                        moves sunrise and sunset."
                },
                "tz_offset_minutes": {
                    "type": "integer", "default": 0,
                    "minimum": -720, "maximum": 840,
                    "description": "Offset of the observer's civil clock from UT, in \
                        minutes, in [-720, 840] (UTC-12:00 to UTC+14:00). Names the \
                        calendar date of the sunrise that begins the Hindu day, from which \
                        the lords of the year, month and day (abda, masa, vara) are \
                        reckoned."
                }
            },
            "required": ["julian_day", "latitude", "longitude", "ayanamsha"]
        }),
        annotations: super::ToolAnnotations::READ_ONLY,
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "julian_day": {
                    "type": "number",
                    "description": "The Julian Day (UT1) the strengths were computed for, echoed back."
                },
                "true_ayanamsha_value": {
                    "type": "number",
                    "description": "True ayanamsha (mean plus nutation in longitude) applied, in degrees."
                },
                "house_system": {
                    "type": "string",
                    "description": "The house system requested. When `polar_fallback` is \
                        true it was undefined at this latitude, and the cusps and house \
                        placements come from the fallback the natal chart uses."
                },
                "polar_fallback": {
                    "type": "boolean",
                    "description": "True when the requested house system was undefined at \
                        this latitude and a fallback was used, so Dig Bala and Kendradi \
                        Bala are read from the fallback's cusps and houses."
                },
                "moment": {
                    "type": "object",
                    "description": "The birth moment Kala Bala was reckoned from. Every instant is a Julian Day (UT1).",
                    "properties": {
                        "jd": { "type": "number", "description": "The birth." },
                        "jd_sunrise": { "type": "number", "description": "The sunrise that begins the Hindu day holding the birth: the last at or before it." },
                        "jd_sunset": { "type": "number", "description": "The sunset of that day." },
                        "jd_next_sunrise": { "type": "number", "description": "The sunrise that ends the Hindu day." },
                        "hindu_day_jdn": { "type": "integer", "description": "Julian Day Number of the local calendar date of jd_sunrise, at tz_offset_minutes." },
                        "apparent_hours_from_midnight": { "type": "number", "description": "Local apparent solar time, hours since midnight, 0 to 24: the hour angle of the true Sun plus 12 hours." }
                    },
                    "required": [
                        "jd", "jd_sunrise", "jd_sunset", "jd_next_sunrise",
                        "hindu_day_jdn", "apparent_hours_from_midnight"
                    ]
                },
                "cusps": {
                    "type": "object",
                    "description": "The four angular cusps Dig Bala is measured from, sidereal degrees.",
                    "properties": {
                        "first": { "type": "number" },
                        "fourth": { "type": "number" },
                        "seventh": { "type": "number" },
                        "tenth": { "type": "number" }
                    },
                    "required": ["first", "fourth", "seventh", "tenth"]
                },
                "planets": {
                    "type": "array",
                    "description": "The seven grahas, Sun to Saturn, in the order Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn. All strengths are in virupas.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "planet": { "type": "string", "description": "Graha assessed." },
                            "sthana": {
                                "type": "object",
                                "description": "Positional strength by part.",
                                "properties": {
                                    "uccha": { "type": "number", "description": "Exaltation strength, 0-60 (27.1)." },
                                    "saptavargaja": { "type": "number", "description": "Dignity in the seven vargas D1, D2, D3, D7, D9, D12 and D30 (27.2-27.4)." },
                                    "ojhayugma": { "type": "number", "description": "Odd/even sign and navamsa strength, 0-30." },
                                    "kendradi": { "type": "number", "description": "Angularity strength from the house, 15, 30 or 60." },
                                    "drekkana": { "type": "number", "description": "Decanate strength, 0 or 15 (27.6)." },
                                    "total": { "type": "number", "description": "Sum of the five." }
                                },
                                "required": ["uccha", "saptavargaja", "ojhayugma", "kendradi", "drekkana", "total"]
                            },
                            "dig_bala": { "type": "number", "description": "Directional strength, 0-60, continuous in the longitude (27.7)." },
                            "kala": {
                                "type": "object",
                                "description": "Temporal strength by part.",
                                "properties": {
                                    "nathonnata": { "type": "number", "description": "Day/night strength from the time from local midnight, 0-60 (27.8-27.9)." },
                                    "paksha": { "type": "number", "description": "Strength from the Moon-Sun elongation, 0-60 (27.10-27.11), undoubled for the Moon." },
                                    "tribhaga": { "type": "number", "description": "60 to Jupiter and to the lord of the third of the day or night (27.12)." },
                                    "abda": { "type": "number", "description": "15 to the lord of the year (27.13)." },
                                    "masa": { "type": "number", "description": "30 to the lord of the month." },
                                    "vara": { "type": "number", "description": "45 to the lord of the day." },
                                    "hora": { "type": "number", "description": "60 to the lord of the hora." },
                                    "ayana": { "type": "number", "description": "Strength from the sayana longitude, 0-60 (27.15-27.17), undoubled." },
                                    "yuddha": { "type": "number", "description": "Signed planetary-war adjustment (27.20): the difference of the two strengths to the victor, from the vanquished." },
                                    "total": { "type": "number", "description": "Sum of the nine." }
                                },
                                "required": [
                                    "nathonnata", "paksha", "tribhaga", "abda", "masa",
                                    "vara", "hora", "ayana", "yuddha", "total"
                                ]
                            },
                            "cheshta_bala": { "type": "number", "description": "Motional strength: the Sun's ayana bala, the Moon's paksha bala, the folded cheshta kendra over 3 for the others (27.18, 27.24, 27.25)." },
                            "naisargika_bala": { "type": "number", "description": "Natural strength, fixed per graha (27.14)." },
                            "drik_bala": { "type": "number", "description": "Aspectual strength from the sphuta drishti, signed (27.19)." },
                            "total": { "type": "number", "description": "Sum of the six: sthana.total + dig_bala + kala.total + cheshta_bala + naisargika_bala + drik_bala." },
                            "ishta_phala": { "type": "number", "description": "Benefic potential, 0-60 (28.5-28.6)." },
                            "kashta_phala": { "type": "number", "description": "Malefic potential, 0-60." },
                            "motion": {
                                "type": "object",
                                "description": "The graha's motion state (27.21-27.23). Reported beside the strengths and not part of any total.",
                                "properties": {
                                    "state": { "type": "string", "description": "Vakra, Anuvakra, Vikala, Manda, Mandatara, Sama, Chara or Atichara." },
                                    "virupas": { "type": "number", "description": "The cheshta bala that state carries in the verse." },
                                    "speed_deg_per_day": { "type": "number", "description": "Speed in longitude, negative when retrograde." },
                                    "mean_speed_deg_per_day": { "type": "number", "description": "Mean geocentric motion at the birth, the pace the state is judged against." }
                                },
                                "required": ["state", "virupas", "speed_deg_per_day", "mean_speed_deg_per_day"]
                            }
                        },
                        "required": [
                            "planet", "sthana", "dig_bala", "kala", "cheshta_bala",
                            "naisargika_bala", "drik_bala", "total", "ishta_phala",
                            "kashta_phala", "motion"
                        ]
                    }
                }
            },
            "required": [
                "julian_day", "true_ayanamsha_value", "house_system", "polar_fallback",
                "moment", "cusps", "planets"
            ]
        })),
        structured_key: None,
    }
}

/// Why `Tropical` is refused. The `vedaksha-wasm` `compute_shadbala` export
/// carries the same words; its parity test compares them.
pub const TROPICAL_REFUSED: &str = "Shadbala is defined on sidereal positions (the signs, \
    houses and vargas it reads are those of the sidereal zodiac), so a tropical ayanamsha \
    gives no Shadbala; pass one of the sidereal systems, for example IndianOfficial";

/// Validate a [`ComputeShadbalaInput`].
///
/// The ayanamsha and house-system names are resolved by the handler, with the
/// engine's own parsers, so this surface cannot drift from the systems the
/// engine has.
///
/// # Errors
/// Returns [`McpError`] for a Julian Day, latitude, longitude, elevation or
/// clock offset out of range, and for the `Tropical` ayanamsha: Shadbala is
/// defined on sidereal positions, so a chart without the ayanamsha rotation
/// has none to give, and a number computed from it would read as one.
pub fn validate(input: &ComputeShadbalaInput) -> Result<(), McpError> {
    if matches!(
        input
            .ayanamsha
            .parse::<vedaksha_astro::sidereal::Ayanamsha>(),
        Ok(vedaksha_astro::sidereal::Ayanamsha::Tropical)
    ) {
        return Err(McpError::invalid_parameter("ayanamsha", TROPICAL_REFUSED));
    }
    validation::validate_jd(input.julian_day)?;
    validation::validate_latitude(input.latitude)?;
    validation::validate_longitude(input.longitude)?;
    validation::validate_elevation_m(input.elevation_m)?;
    validation::validate_tz_offset_minutes(input.tz_offset_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> ComputeShadbalaInput {
        ComputeShadbalaInput {
            julian_day: 2_461_338.0,
            latitude: 13.08,
            longitude: 80.27,
            ayanamsha: "IndianOfficial".to_string(),
            house_system: None,
            elevation_m: 0.0,
            tz_offset_minutes: 330,
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        assert!(validate(&valid()).is_ok());
    }

    #[test]
    fn validate_names_the_field_it_rejects() {
        let cases: [(fn(&mut ComputeShadbalaInput), &str); 5] = [
            (|i| i.julian_day = 1.0, "DATE_OUT_OF_RANGE"),
            (|i| i.latitude = 91.0, "INVALID_LATITUDE"),
            (|i| i.longitude = -181.0, "INVALID_LONGITUDE"),
            (|i| i.elevation_m = 20_000.0, "INVALID_PARAMETER"),
            (|i| i.tz_offset_minutes = 900, "INVALID_PARAMETER"),
        ];
        for (break_it, code) in cases {
            let mut input = valid();
            break_it(&mut input);
            assert_eq!(validate(&input).unwrap_err().error_code, code);
        }
    }

    #[test]
    fn validate_refuses_the_tropical_ayanamsha_under_every_spelling() {
        for name in ["Tropical", "tropical", "TROPICAL", "None", "none"] {
            let mut input = valid();
            input.ayanamsha = name.to_string();
            let err = validate(&input).unwrap_err();
            assert_eq!(err.error_code, "INVALID_PARAMETER", "{name}");
            assert!(err.message.contains("ayanamsha"), "{name}: {}", err.message);
            assert!(err.message.contains("sidereal"), "{name}: {}", err.message);
        }
        // Every sidereal system is still accepted, and an unknown name is left
        // to the handler's own parser (and its own message).
        for a in vedaksha_astro::sidereal::Ayanamsha::SIDEREAL {
            let mut input = valid();
            input.ayanamsha = a.key().to_string();
            assert!(validate(&input).is_ok(), "{}", a.key());
        }
        let mut unknown = valid();
        unknown.ayanamsha = "NoSuchSystem".to_string();
        assert!(validate(&unknown).is_ok());
    }

    #[test]
    fn the_schema_offers_only_the_sidereal_systems() {
        let def = definition();
        let offered = def.input_schema["properties"]["ayanamsha"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(
            offered.len(),
            vedaksha_astro::sidereal::Ayanamsha::SIDEREAL.len()
        );
        assert!(!offered.iter().any(|v| v == "Tropical"));
        let text = def.input_schema["properties"]["ayanamsha"]["description"]
            .as_str()
            .unwrap();
        assert!(!text.contains("Pass Tropical"), "{text}");
        assert!(text.contains("Tropical is refused"), "{text}");
    }

    #[test]
    fn the_earlier_planets_input_is_refused_by_name() {
        let old = serde_json::json!({
            "planets": [{ "planet": "Sun", "sign": 2, "longitude": 60.6, "bhava": 10,
                          "speed": 0.9, "average_speed": 1.0 }],
            "is_daytime": true, "moon_phase_waxing": true
        });
        // serde names the first unknown field it meets, in key order.
        let err = serde_json::from_value::<ComputeShadbalaInput>(old).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("unknown field"), "{text}");
        assert!(
            text.contains("is_daytime") || text.contains("planets"),
            "{text}"
        );
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_shadbala");
        let required: Vec<&str> = def.input_schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(
            required,
            ["julian_day", "latitude", "longitude", "ayanamsha"]
        );
        // The description no longer says any part is omitted.
        assert!(!def.description.contains("omits"));
    }
}
