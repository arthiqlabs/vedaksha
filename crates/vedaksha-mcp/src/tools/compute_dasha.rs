// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `compute_dasha` — Vedic dasha period computation tool.
//!
//! Supports eight classical systems: three Moon-longitude based
//! (Vimshottari, Ashtottari, Yogini), two Lagna-sign based (Chara, Narayana)
//! and three Jaimini rashi dashas read from the lagna and the grahas'
//! longitudes (Sthira, Niryana Shoola, Maheshwara).

use serde::Deserialize;

use crate::validation::{self, McpError};

/// Maximum number of dasha levels supported (Moon-based systems).
const MAX_LEVELS: u8 = 5;

/// The most Yogini periods one response may carry. One round has
/// 8 + 8^2 + ... + 8^levels periods (584 at the default three levels, 37,448
/// at five), so this admits ten rounds at four levels and one at five.
const MAX_YOGINI_PERIODS: u64 = 50_000;

/// Dasha system selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashaSystem {
    Vimshottari,
    Ashtottari,
    Yogini,
    Chara,
    Narayana,
    Sthira,
    NiryanaShoola,
    Maheshwara,
}

impl DashaSystem {
    /// Parse a system name, ignoring case and any `_`, `-` or space, so
    /// `NiryanaShoola`, `niryana_shoola` and `niryana-shoola` all name the
    /// same system.
    ///
    /// # Errors
    /// Returns [`McpError::invalid_parameter`] if `s` does not match one of
    /// `Vimshottari`, `Ashtottari`, `Yogini`, `Chara`, `Narayana`, `Sthira`,
    /// `NiryanaShoola` or `Maheshwara`.
    pub fn parse(s: &str) -> Result<Self, McpError> {
        let normalized: String = s
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match normalized.as_str() {
            "vimshottari" => Ok(Self::Vimshottari),
            "ashtottari" => Ok(Self::Ashtottari),
            "yogini" => Ok(Self::Yogini),
            "chara" => Ok(Self::Chara),
            "narayana" => Ok(Self::Narayana),
            "sthira" => Ok(Self::Sthira),
            "niryanashoola" => Ok(Self::NiryanaShoola),
            "maheshwara" => Ok(Self::Maheshwara),
            _ => Err(McpError::invalid_parameter(
                "system",
                "must be one of: Vimshottari, Ashtottari, Yogini, Chara, Narayana, Sthira, \
                 NiryanaShoola, Maheshwara",
            )),
        }
    }

    /// Whether this system is one of the three Jaimini rashi dashas that read
    /// the lagna and the grahas' longitudes (the atmakaraka and the tie-breaks
    /// of the strength tests need degrees, not just signs).
    #[must_use]
    pub fn is_longitude_based(self) -> bool {
        matches!(self, Self::Sthira | Self::NiryanaShoola | Self::Maheshwara)
    }

    /// Whether this system uses the natal Moon's sidereal longitude
    /// (`true`) or the lagna sign (`false`) as its anchor.
    #[must_use]
    pub fn is_moon_based(self) -> bool {
        matches!(self, Self::Vimshottari | Self::Ashtottari | Self::Yogini)
    }
}

/// Natal sign positions (0 = Aries .. 11 = Pisces) of the seven classical
/// grahas plus Rahu, as served by `compute_natal_chart`'s `sign_index`
/// field. Required, with every field, when `system` is `Chara` or
/// `Narayana` — their dasha durations are chart-dependent (see
/// [`crate::server`]'s `compute_dasha` dispatch and
/// `vedaksha_vedic::dasha::chara`'s module doc for the rule).
///
/// Ketu is deliberately NOT a field here: it is derived internally as
/// `(rahu + 6) % 12`, since the two lunar nodes are always exactly
/// opposite by definition and accepting both invites an
/// internally-inconsistent chart.
///
/// Every field is `Option<i32>`, and both halves of that are deliberate.
/// `Option`, because although all eight are required whenever `graha_signs`
/// is, a caller who omits one should be told *which* one by [`validate`]
/// rather than get a generic deserialization failure. `i32` rather than
/// `u8`, because `u8` rejects a negative inside `serde` before [`validate`]
/// runs, yielding `invalid value: integer -1, expected u8` with no field
/// name — and `-1` is a plausible mistake, being exactly what an off-by-one
/// from a 1-indexed source produces. Widening the parse lets every range
/// error reach [`validate`] and come back naming the offending graha.
///
/// `deny_unknown_fields` is load-bearing rather than tidiness: `ketu` is
/// deliberately absent because it is derived from `rahu`, and silently
/// ignoring a supplied `ketu` would accept an internally inconsistent chart
/// while appearing to honour it. Rejecting it says so.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrahaSignsInput {
    /// Sun's natal sign, 0–11 (0 = Aries).
    pub sun: Option<i32>,
    /// Moon's natal sign, 0–11 (0 = Aries).
    pub moon: Option<i32>,
    /// Mars's natal sign, 0–11 (0 = Aries).
    pub mars: Option<i32>,
    /// Mercury's natal sign, 0–11 (0 = Aries).
    pub mercury: Option<i32>,
    /// Jupiter's natal sign, 0–11 (0 = Aries).
    pub jupiter: Option<i32>,
    /// Venus's natal sign, 0–11 (0 = Aries).
    pub venus: Option<i32>,
    /// Saturn's natal sign, 0–11 (0 = Aries).
    pub saturn: Option<i32>,
    /// Rahu's (north lunar node's) natal sign, 0–11 (0 = Aries).
    pub rahu: Option<i32>,
}

impl GrahaSignsInput {
    /// Convert to the library's [`vedaksha_vedic::dasha::chara::GrahaSigns`].
    ///
    /// # Panics
    /// Panics if any field is `None` or outside `[0, 11]`. Only call after
    /// [`validate`] has confirmed both — the same "validated above" contract
    /// `moon_longitude` and `lagna_sign` already rely on below.
    #[must_use]
    pub fn into_graha_signs(self) -> vedaksha_vedic::dasha::chara::GrahaSigns {
        // `validate` bounds every field to [0, 11], so each cast is exact.
        let sign = |v: Option<i32>| -> u8 {
            u8::try_from(v.expect("validated above")).expect("validated above")
        };
        vedaksha_vedic::dasha::chara::GrahaSigns {
            sun: sign(self.sun),
            moon: sign(self.moon),
            mars: sign(self.mars),
            mercury: sign(self.mercury),
            jupiter: sign(self.jupiter),
            venus: sign(self.venus),
            saturn: sign(self.saturn),
            rahu: sign(self.rahu),
        }
    }
}

/// Sidereal longitudes (degrees, [0, 360)) of the seven classical grahas plus
/// Rahu, for the systems that need degrees: Sthira, Niryana Shoola and
/// Maheshwara. Ketu is derived 180 degrees from Rahu, as `graha_signs` derives
/// it, and is deliberately not a field.
///
/// Every field is optional so [`validate`] can name the one that is missing;
/// `deny_unknown_fields` so a supplied `ketu` is refused rather than ignored.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrahaLongitudesInput {
    /// Sun's sidereal longitude.
    pub sun: Option<f64>,
    /// Moon's sidereal longitude.
    pub moon: Option<f64>,
    /// Mars's sidereal longitude.
    pub mars: Option<f64>,
    /// Mercury's sidereal longitude.
    pub mercury: Option<f64>,
    /// Jupiter's sidereal longitude.
    pub jupiter: Option<f64>,
    /// Venus's sidereal longitude.
    pub venus: Option<f64>,
    /// Saturn's sidereal longitude.
    pub saturn: Option<f64>,
    /// Rahu's (north lunar node's) sidereal longitude.
    pub rahu: Option<f64>,
}

impl GrahaLongitudesInput {
    /// The eight longitudes in the order `NatalPlacements::from_longitudes`
    /// takes: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu.
    ///
    /// # Panics
    /// Panics if a field is `None`. Only call after [`validate`].
    #[must_use]
    pub fn into_array(self) -> [f64; 8] {
        let get = |v: Option<f64>| v.expect("validated above");
        [
            get(self.sun),
            get(self.moon),
            get(self.mars),
            get(self.mercury),
            get(self.jupiter),
            get(self.venus),
            get(self.saturn),
            get(self.rahu),
        ]
    }

    fn named(self) -> [(&'static str, Option<f64>); 8] {
        [
            ("graha_longitudes.sun", self.sun),
            ("graha_longitudes.moon", self.moon),
            ("graha_longitudes.mars", self.mars),
            ("graha_longitudes.mercury", self.mercury),
            ("graha_longitudes.jupiter", self.jupiter),
            ("graha_longitudes.venus", self.venus),
            ("graha_longitudes.saturn", self.saturn),
            ("graha_longitudes.rahu", self.rahu),
        ]
    }
}

/// Input parameters for the `compute_dasha` tool.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ComputeDashaInput {
    /// Dasha system. Defaults to `"Vimshottari"`.
    pub system: Option<String>,
    /// Birth Julian Day used as the dasha epoch, in **UT1** (Universal
    /// Time) — not TT, not TDB, i.e. the same scale as
    /// `compute_natal_chart`'s `julian_day`.
    ///
    /// This epoch is only added to, never converted, so every returned
    /// `start_jd`/`end_jd` is on the same UT1 scale as the input.
    pub birth_jd: f64,
    /// Natal Moon sidereal longitude in degrees \[0, 360).
    /// Required for Vimshottari, Ashtottari, Yogini.
    pub moon_longitude: Option<f64>,
    /// Lagna (ascendant) sign 0–11 (0 = Aries), the same 0-indexed
    /// convention `compute_natal_chart` serves as `sign_index`.
    /// Required for Chara, Narayana.
    pub lagna_sign: Option<u8>,
    /// Natal sign positions of the seven classical grahas plus Rahu.
    /// Required for Chara, Narayana; irrelevant otherwise.
    pub graha_signs: Option<GrahaSignsInput>,
    /// Number of nested dasha levels (1–5). Defaults to 3.
    /// Ignored by the sign-based systems, which return a single level (with
    /// their antardashas where they have them).
    pub levels: Option<u8>,
    /// Sidereal longitudes of the seven classical grahas plus Rahu. Required,
    /// with `lagna_sign`, for Sthira, Niryana Shoola and Maheshwara.
    pub graha_longitudes: Option<GrahaLongitudesInput>,
    /// Which chara karakas name the atmakaraka for Sthira and Maheshwara:
    /// `"7"` (Sun to Saturn, the default) or `"8"` (adds Rahu), as
    /// `compute_karakas` takes it.
    pub karaka_scheme: Option<String>,
    /// Yogini only: how many 36-year rounds of the cycle to produce, 1 to
    /// [`MAX_YOGINI_ROUNDS`](vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS).
    /// Defaults to 1. Exclusive with `until`.
    pub rounds: Option<u8>,
    /// Yogini only: a horizon, Julian Day (UT1). Whole rounds are produced
    /// until the tree reaches it (at most the maximum number of rounds).
    /// Exclusive with `rounds`.
    pub until: Option<f64>,
    /// Chara only: whether each period carries its twelve antardashas.
    /// Defaults to false, which leaves the response exactly as it was.
    pub antardashas: Option<bool>,
}

/// Tool metadata for MCP tool-listing.
#[must_use]
pub fn definition() -> super::ToolDefinition {
    super::ToolDefinition {
        name: "compute_dasha",
        description: "Compute Vedic dasha (planetary period) sequences. Supports eight \
            classical systems: Vimshottari, Ashtottari, Yogini (Moon-longitude based, \
            require `moon_longitude`); Chara, Narayana (sign based, require BOTH \
            `lagna_sign` AND `graha_signs`, since each sign's period length is counted \
            to the sign its lord occupies in this chart); Sthira, NiryanaShoola \
            (`niryana_shoola` is accepted too) and Maheshwara (Jaimini rashi dashas, \
            require `lagna_sign` AND `graha_longitudes`, since the atmakaraka and the \
            tie-breaks of the strength tests read degrees). `graha_signs` takes the \
            eight `sign_index` values compute_natal_chart already returns, \
            `graha_longitudes` the eight sidereal longitudes; Ketu is derived from Rahu \
            and must not be supplied. Yogini takes `rounds` (1 to 10 whole 36-year \
            rounds) or `until` (a horizon Julian Day); Chara takes `antardashas` \
            (twelve per period, BPHS 51.5-51.9). Sthira, NiryanaShoola and Maheshwara \
            use the defaults of the vedic crate's dasha functions (see their module \
            docs) and always carry their antardashas. Sthira and Maheshwara start from \
            the Brahma graha; a chart whose marks leave none returns `no_brahma: true` \
            with a message, not an error. Returns a JSON dasha tree with start/end \
            Julian Days.",
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "system": {
                    "type": "string",
                    "description": "Dasha system selector",
                    "enum": [
                        "Vimshottari", "Ashtottari", "Yogini", "Chara", "Narayana",
                        "Sthira", "NiryanaShoola", "Maheshwara"
                    ],
                    "default": "Vimshottari"
                },
                "birth_jd": {
                    "type": "number",
                    "description": "Birth Julian Day used as the dasha epoch, in \
                        UT1 (Universal Time) — not TT, not TDB, i.e. the same scale as \
                        compute_natal_chart's julian_day. This epoch is only added to, \
                        never converted: every returned start_jd/end_jd is on the same \
                        UT1 scale as the input."
                },
                "moon_longitude": {
                    "type": "number",
                    "description": "Natal Moon sidereal longitude in degrees [0, 360). Required for Vimshottari, Ashtottari, Yogini.",
                    "minimum": 0,
                    "maximum": 360
                },
                "lagna_sign": {
                    "type": "integer",
                    "description": "Lagna (ascendant) sign 0–11 (0 = Aries). Required for \
                        Chara, Narayana, Sthira, NiryanaShoola, Maheshwara. This is the same 0-indexed convention \
                        compute_natal_chart, compute_bhavas and compute_vargas all serve as \
                        `sign_index`, so an ascendant read from any of them passes straight \
                        through. Through v8.1.0 this one parameter was 1-indexed while every \
                        other tool was 0-indexed; that mismatch is resolved here.",
                    "minimum": 0,
                    "maximum": 11
                },
                "graha_signs": {
                    "type": "object",
                    "description": "Natal sign positions of the seven classical grahas plus \
                        Rahu, 0-11 each (0 = Aries), as served by compute_natal_chart's \
                        sign_index. Required for Chara, Narayana, whose period lengths are \
                        chart-dependent — each sign's duration is the count from that sign to \
                        the sign its lord actually occupies. Sign indices are used rather than \
                        longitudes: they chain directly from compute_natal_chart's sign_index \
                        with no conversion, carry no tropical-vs-sidereal ambiguity, and avoid \
                        the boundary question of a planet at exactly 30.0 degrees. Ketu is not \
                        a field: it is derived as (rahu + 6) mod 12, since the two lunar nodes \
                        are always exactly opposite.",
                    "properties": {
                        "sun":     { "type": "integer", "description": "Sun's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "moon":    { "type": "integer", "description": "Moon's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "mars":    { "type": "integer", "description": "Mars's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "mercury": { "type": "integer", "description": "Mercury's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "jupiter": { "type": "integer", "description": "Jupiter's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "venus":   { "type": "integer", "description": "Venus's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "saturn":  { "type": "integer", "description": "Saturn's natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 },
                        "rahu":    { "type": "integer", "description": "Rahu's (north lunar node's) natal sign, 0-11 (0 = Aries).", "minimum": 0, "maximum": 11 }
                    },
                    "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu"]
                },
                "graha_longitudes": {
                    "type": "object",
                    "description": "Sidereal longitudes in degrees [0, 360) of the seven \
                        classical grahas plus Rahu, as compute_natal_chart serves them \
                        (Rahu from its MeanNode). Required, with lagna_sign, for Sthira, \
                        NiryanaShoola and Maheshwara, whose Brahma graha, atmakaraka and \
                        strength tie-breaks read degrees within the sign. Ketu is not a \
                        field: it is derived 180 degrees from Rahu. Do not also pass \
                        graha_signs for these systems.",
                    "properties": {
                        "sun":     { "type": "number", "description": "Sun's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "moon":    { "type": "number", "description": "Moon's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "mars":    { "type": "number", "description": "Mars's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "mercury": { "type": "number", "description": "Mercury's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "jupiter": { "type": "number", "description": "Jupiter's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "venus":   { "type": "number", "description": "Venus's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "saturn":  { "type": "number", "description": "Saturn's sidereal longitude.", "minimum": 0, "maximum": 360 },
                        "rahu":    { "type": "number", "description": "Rahu's (north lunar node's) sidereal longitude.", "minimum": 0, "maximum": 360 }
                    },
                    "required": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu"]
                },
                "karaka_scheme": {
                    "type": "string",
                    "description": "Which chara karakas name the atmakaraka for Sthira and \
                        Maheshwara: \"7\" (Sun to Saturn) or \"8\" (adds Rahu), as \
                        compute_karakas takes it. Applies to those two systems only.",
                    "enum": ["7", "8"],
                    "default": "7"
                },
                "rounds": {
                    "type": "integer",
                    "description": "Yogini only: how many whole 36-year rounds of the cycle \
                        to produce. Round 1 is the round from birth, whose first maha is the \
                        birth yogini's unelapsed share; each later round runs all eight \
                        yoginis whole. Every maha and sub-period carries its `round`. \
                        Defaults to 1. Exclusive with `until`. One response carries at most \
                        50,000 periods: ten rounds at four levels, one at five.",
                    "minimum": 1,
                    "maximum": vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS,
                    "default": 1
                },
                "until": {
                    "type": "number",
                    "description": "Yogini only: a horizon as a Julian Day (UT1). Whole rounds \
                        are produced until the tree reaches it, at least one and at most the \
                        maximum of `rounds`; compare the last maha's end_jd with it to see \
                        whether it was cut. Exclusive with `rounds`."
                },
                "antardashas": {
                    "type": "boolean",
                    "description": "Chara only: when true each of the twelve periods carries \
                        its twelve antardashas in `antar` (BPHS 51.5-51.9: the stronger of \
                        the dasha rashi and its seventh starts, by modality, a twelfth of the \
                        period each). Defaults to false. Sthira, NiryanaShoola and Maheshwara \
                        always carry theirs.",
                    "default": false
                },
                "levels": {
                    "type": "integer",
                    "description": "Number of nested dasha levels (1–5). Ignored by the \
                        sign-based systems (Chara, Narayana, Sthira, NiryanaShoola, \
                        Maheshwara).",
                    "minimum": 1,
                    "maximum": 5,
                    "default": 3
                }
            },
            "required": ["birth_jd"]
        }),
        annotations: super::ToolAnnotations::READ_ONLY,
        // The five systems return FOUR different envelopes, so this is a
        // `oneOf` rather than one object. Through v8.1.0 it declared the
        // Vimshottari shape unconditionally, with moon_nakshatra /
        // initial_balance / maha_dashas all "required" -- false for
        // Ashtottari (which serves `periods` and `starting_lord`), false for
        // Yogini (`maha_periods`, `starting_yogini_index`), and false for
        // Chara/Narayana, which return no nakshatra at all. Nothing caught it
        // because `structured_key` is None, so no structuredContent is
        // emitted and the output-schema tests had nothing to check against.
        output_schema: Some(serde_json::json!({
            "type": "object",
            "description": "Envelope depends on `system`; see the four variants below. \
                Every period carries start_jd/end_jd on the same UT1 scale as birth_jd.",
            "oneOf": [
                {
                    "title": "Vimshottari",
                    "type": "object",
                    "properties": {
                        "moon_nakshatra": { "type": "string" },
                        "initial_balance": { "type": "number" },
                        "maha_dashas": { "type": "array", "items": { "type": "object" } }
                    },
                    "required": ["moon_nakshatra", "initial_balance", "maha_dashas"]
                },
                {
                    "title": "Ashtottari",
                    "type": "object",
                    "properties": {
                        "moon_nakshatra": { "type": "string" },
                        "starting_lord": { "type": "string" },
                        "initial_balance": { "type": "number" },
                        "periods": { "type": "array", "items": { "type": "object" } }
                    },
                    "required": ["moon_nakshatra", "starting_lord", "initial_balance", "periods"]
                },
                {
                    "title": "Yogini",
                    "type": "object",
                    "properties": {
                        "moon_nakshatra": { "type": "string" },
                        "starting_yogini_index": { "type": "integer" },
                        "initial_balance": { "type": "number" },
                        "maha_periods": { "type": "array", "items": { "type": "object" } }
                    },
                    "required": [
                        "moon_nakshatra", "starting_yogini_index",
                        "initial_balance", "maha_periods"
                    ]
                },
                {
                    "title": "Chara / Narayana",
                    "type": "object",
                    "description": "Sign-based systems. Twelve single-level periods, one \
                        per rashi, in dasha order. No nakshatra or balance: these systems \
                        are anchored on the lagna and the grahas' natal signs, not the Moon. \
                        With `antardashas` true, Chara's periods also carry `antar`.",
                    "properties": {
                        "lagna_sign": {
                            "type": "integer",
                            "description": "Lagna the sequence starts from, 0-11 (0 = Aries)."
                        },
                        "periods": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "sign_index": { "type": "integer" },
                                    "sign_name": { "type": "string" },
                                    "start_jd": { "type": "number" },
                                    "end_jd": { "type": "number" },
                                    "duration_years": {
                                        "type": "number",
                                        "description": "Counted from this sign to the sign \
                                            its lord occupies in THIS chart, so it varies \
                                            between charts."
                                    },
                                    "antar": {
                                        "type": "array",
                                        "description": "Chara with `antardashas` true only: \
                                            the twelve antardashas, tiling the period.",
                                        "items": { "type": "object" }
                                    }
                                },
                                "required": [
                                    "sign_index", "sign_name",
                                    "start_jd", "end_jd", "duration_years"
                                ]
                            }
                        }
                    },
                    "required": ["lagna_sign", "periods"]
                },
                {
                    "title": "Sthira",
                    "type": "object",
                    "description": "Twelve periods of 7, 8 or 9 years by modality from the \
                        Brahma graha's rashi, each with twelve antardashas. When the chart \
                        has no Brahma graha, `no_brahma` is true, `message` says so, and \
                        `anchor_sign`, `brahma` and `direction` are null and `periods` is \
                        empty: the text names no other start.",
                    "properties": {
                        "no_brahma": { "type": "boolean" },
                        "message": { "type": "string" },
                        "anchor_sign": { "type": ["integer", "null"] },
                        "brahma": { "type": ["string", "null"] },
                        "direction": { "type": ["string", "null"] },
                        "periods": { "type": "array", "items": { "type": "object" } }
                    },
                    "required": ["no_brahma", "anchor_sign", "brahma", "direction", "periods"]
                },
                {
                    "title": "Niryana Shoola",
                    "type": "object",
                    "description": "Twelve periods of 7, 8 or 9 years in trikona groups from \
                        the stronger of the lagna and the seventh, each with twelve \
                        antardashas.",
                    "properties": {
                        "anchor_sign": { "type": "integer" },
                        "direction": { "type": "string" },
                        "periods": { "type": "array", "items": { "type": "object" } }
                    },
                    "required": ["anchor_sign", "direction", "periods"]
                },
                {
                    "title": "Maheshwara",
                    "type": "object",
                    "description": "The Maheshwara graha, the span of life to the end of its \
                        rashi's sthira period, and the antardashas of that period that can \
                        carry death. When the sthira dasha has no Brahma anchor, \
                        `no_brahma` is true with a `message`, `maheshwara` and `rashi` are \
                        still given, and the rest is absent.",
                    "properties": {
                        "no_brahma": { "type": "boolean" },
                        "message": { "type": "string" },
                        "maheshwara": { "type": "string" },
                        "rashi": { "type": "integer" },
                        "eighth_lord": { "type": "string" },
                        "eighth_lord_rashi": { "type": "integer" },
                        "death_trikona": { "type": "array", "items": { "type": "integer" } },
                        "life_span_years": { "type": "number" },
                        "period": { "type": "object" },
                        "death_antars": { "type": "array", "items": { "type": "integer" } }
                    },
                    "required": ["no_brahma", "maheshwara", "rashi"]
                }
            ]
        })),
        structured_key: None,
    }
}

/// Resolve the system selector and validate every input field.
///
/// # Errors
///
/// Returns the first [`McpError`] encountered.
pub fn validate(input: &ComputeDashaInput) -> Result<DashaSystem, McpError> {
    let system = match &input.system {
        Some(s) => DashaSystem::parse(s)?,
        None => DashaSystem::Vimshottari,
    };

    validation::validate_jd(input.birth_jd)?;
    validate_extensions(input, system)?;

    if system.is_longitude_based() {
        let sign = input.lagna_sign.ok_or_else(|| {
            McpError::invalid_parameter(
                "lagna_sign",
                "required for Sthira, NiryanaShoola and Maheshwara",
            )
        })?;
        if !(0..=11).contains(&sign) {
            return Err(McpError::invalid_parameter(
                "lagna_sign",
                "must be an integer in [0, 11]",
            ));
        }
        let longitudes = input.graha_longitudes.ok_or_else(|| {
            McpError::invalid_parameter(
                "graha_longitudes",
                "required for Sthira, NiryanaShoola and Maheshwara",
            )
        })?;
        for (name, value) in longitudes.named() {
            let value = value.ok_or_else(|| {
                McpError::invalid_parameter(
                    name,
                    "required when system is Sthira, NiryanaShoola or Maheshwara",
                )
            })?;
            if !value.is_finite() || !(0.0..360.0).contains(&value) {
                return Err(McpError::invalid_parameter(
                    name,
                    "must be a finite number in [0, 360)",
                ));
            }
        }
        // The two ways of giving the grahas must not both be given: signs and
        // longitudes could disagree, and one of them would be ignored.
        if input.graha_signs.is_some() {
            return Err(McpError::invalid_parameter(
                "graha_signs",
                "not used by Sthira, NiryanaShoola or Maheshwara; pass graha_longitudes",
            ));
        }
    } else if system.is_moon_based() {
        let moon = input.moon_longitude.ok_or_else(|| {
            McpError::invalid_parameter(
                "moon_longitude",
                "required for Vimshottari, Ashtottari, and Yogini",
            )
        })?;
        if !moon.is_finite() || !(0.0..360.0).contains(&moon) {
            return Err(McpError::invalid_parameter(
                "moon_longitude",
                "must be a finite number in [0, 360)",
            ));
        }
    } else {
        let sign = input.lagna_sign.ok_or_else(|| {
            McpError::invalid_parameter("lagna_sign", "required for Chara and Narayana")
        })?;
        if !(0..=11).contains(&sign) {
            return Err(McpError::invalid_parameter(
                "lagna_sign",
                "must be an integer in [0, 11]",
            ));
        }

        let graha_signs = input.graha_signs.ok_or_else(|| {
            McpError::invalid_parameter("graha_signs", "required for Chara and Narayana")
        })?;
        for (name, value) in [
            ("graha_signs.sun", graha_signs.sun),
            ("graha_signs.moon", graha_signs.moon),
            ("graha_signs.mars", graha_signs.mars),
            ("graha_signs.mercury", graha_signs.mercury),
            ("graha_signs.jupiter", graha_signs.jupiter),
            ("graha_signs.venus", graha_signs.venus),
            ("graha_signs.saturn", graha_signs.saturn),
            ("graha_signs.rahu", graha_signs.rahu),
        ] {
            let value = value.ok_or_else(|| {
                McpError::invalid_parameter(name, "required when system is Chara or Narayana")
            })?;
            // Both ends checked. A negative value is the likelier caller
            // mistake of the two -- it is what an off-by-one from a 1-indexed
            // source produces -- and reaching here at all is why the field is
            // parsed as `i32` rather than `u8`.
            if !(0..=11).contains(&value) {
                return Err(McpError::invalid_parameter(
                    name,
                    "must be an integer in [0, 11]",
                ));
            }
        }
    }

    if let Some(levels) = input.levels
        && (levels == 0 || levels > MAX_LEVELS)
    {
        return Err(McpError::invalid_parameter(
            "levels",
            &format!("must be between 1 and {MAX_LEVELS}"),
        ));
    }

    Ok(system)
}

/// The options that change which answer a system gives, checked against the
/// system they belong to. An option that moved the answer and was silently
/// ignored would read as the answer the caller asked for, so each is refused
/// where it does not apply (data the caller merely supplies, such as
/// `graha_signs` to a Moon-based system, is ignored, as it always was).
fn validate_extensions(input: &ComputeDashaInput, system: DashaSystem) -> Result<(), McpError> {
    let only = |param: &str, systems: &str| {
        McpError::invalid_parameter(param, &format!("applies to {systems} only"))
    };
    if (input.rounds.is_some() || input.until.is_some()) && system != DashaSystem::Yogini {
        let param = if input.rounds.is_some() {
            "rounds"
        } else {
            "until"
        };
        return Err(only(param, "Yogini"));
    }
    if input.antardashas == Some(true) && system != DashaSystem::Chara {
        return Err(McpError::invalid_parameter(
            "antardashas",
            "applies to Chara only; Sthira, NiryanaShoola and Maheshwara always carry \
             theirs, and the other systems have levels",
        ));
    }
    if input.karaka_scheme.is_some()
        && !matches!(system, DashaSystem::Sthira | DashaSystem::Maheshwara)
    {
        return Err(only("karaka_scheme", "Sthira and Maheshwara"));
    }
    if let Some(scheme) = &input.karaka_scheme
        && !matches!(scheme.as_str(), "7" | "8")
    {
        return Err(McpError::invalid_parameter(
            "karaka_scheme",
            "must be \"7\" or \"8\"",
        ));
    }
    if input.rounds.is_some() && input.until.is_some() {
        return Err(McpError::invalid_parameter(
            "rounds",
            "exclusive with until: give a number of rounds or a horizon, not both",
        ));
    }
    if let Some(rounds) = input.rounds {
        let max = vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS;
        if rounds == 0 || rounds > max {
            return Err(McpError::invalid_parameter(
                "rounds",
                &format!("must be between 1 and {max}"),
            ));
        }
    }
    if let Some(until) = input.until
        && !until.is_finite()
    {
        return Err(McpError::invalid_parameter(
            "until",
            "must be a finite Julian Day",
        ));
    }
    if system == DashaSystem::Yogini {
        let levels = u32::from(input.levels.unwrap_or(3).clamp(1, MAX_LEVELS));
        let per_round: u64 = (1..=levels).map(|l| 8_u64.pow(l)).sum();
        let max = u64::from(vedaksha_vedic::dasha::yogini::MAX_YOGINI_ROUNDS);
        // `until` stops at the first round ending at or past it. The rounds
        // are counted by the engine itself at one level (at most
        // `MAX_YOGINI_ROUNDS` rounds of eight periods), so the count is exact
        // and an `until` however far ahead cannot overflow it.
        let rounds = match (input.rounds, input.until) {
            (Some(r), _) => u64::from(r),
            (None, Some(until)) => input
                .moon_longitude
                .filter(|m| (0.0..360.0).contains(m))
                .and_then(|moon| {
                    vedaksha_vedic::dasha::yogini::compute_yogini_until(
                        moon,
                        input.birth_jd,
                        1,
                        until,
                    )
                    .maha_periods
                    .last()
                    .map(|p| u64::from(p.round))
                })
                .unwrap_or(1)
                .min(max),
            (None, None) => 1,
        };
        if per_round * rounds > MAX_YOGINI_PERIODS {
            return Err(McpError::invalid_parameter(
                if input.rounds.is_some() {
                    "rounds"
                } else {
                    "until"
                },
                &format!(
                    "{rounds} rounds at {levels} levels is {} periods, over the \
                     {MAX_YOGINI_PERIODS} one response may carry: ask for fewer levels or \
                     rounds",
                    per_round * rounds
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moon_input() -> ComputeDashaInput {
        ComputeDashaInput {
            system: None,
            birth_jd: 2_451_545.0, // J2000
            moon_longitude: Some(123.45),
            ..ComputeDashaInput::default()
        }
    }

    fn valid_graha_signs() -> GrahaSignsInput {
        GrahaSignsInput {
            sun: Some(9),
            moon: Some(2),
            mars: Some(1),
            mercury: Some(10),
            jupiter: Some(10),
            venus: Some(10),
            saturn: Some(4),
            rahu: Some(2),
        }
    }

    fn lagna_input() -> ComputeDashaInput {
        ComputeDashaInput {
            system: Some("Chara".to_string()),
            birth_jd: 2_451_545.0,
            moon_longitude: None,
            lagna_sign: Some(0),
            graha_signs: Some(valid_graha_signs()),
            ..ComputeDashaInput::default()
        }
    }

    #[test]
    fn validate_accepts_default_vimshottari() {
        let s = validate(&moon_input()).unwrap();
        assert_eq!(s, DashaSystem::Vimshottari);
    }

    #[test]
    fn validate_accepts_explicit_ashtottari() {
        let mut input = moon_input();
        input.system = Some("Ashtottari".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Ashtottari);
    }

    #[test]
    fn validate_accepts_yogini_case_insensitive() {
        let mut input = moon_input();
        input.system = Some("yogini".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Yogini);
    }

    #[test]
    fn validate_accepts_chara_with_lagna() {
        assert_eq!(validate(&lagna_input()).unwrap(), DashaSystem::Chara);
    }

    #[test]
    fn validate_accepts_narayana_with_lagna() {
        let mut input = lagna_input();
        input.system = Some("Narayana".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Narayana);
    }

    #[test]
    fn validate_rejects_unknown_system() {
        let mut input = moon_input();
        input.system = Some("DoesNotExist".to_string());
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_moon_based_without_moon_longitude() {
        let mut input = moon_input();
        input.moon_longitude = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_lagna_based_without_lagna_sign() {
        let mut input = lagna_input();
        input.lagna_sign = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_accepts_narayana_with_graha_signs() {
        let mut input = lagna_input();
        input.system = Some("Narayana".to_string());
        assert_eq!(validate(&input).unwrap(), DashaSystem::Narayana);
    }

    #[test]
    fn validate_rejects_chara_without_graha_signs() {
        let mut input = lagna_input();
        input.graha_signs = None;
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
    }

    #[test]
    fn validate_rejects_chara_missing_rahu() {
        let mut input = lagna_input();
        let mut signs = valid_graha_signs();
        signs.rahu = None;
        input.graha_signs = Some(signs);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(
            err.message.contains("rahu"),
            "error should name the missing field, got: {}",
            err.message
        );
    }

    #[test]
    fn validate_rejects_chara_graha_sign_out_of_range() {
        let mut input = lagna_input();
        let mut signs = valid_graha_signs();
        signs.mars = Some(12);
        input.graha_signs = Some(signs);
        let err = validate(&input).unwrap_err();
        assert_eq!(err.error_code, "INVALID_PARAMETER");
        assert!(
            err.message.contains("mars"),
            "error should name the offending field, got: {}",
            err.message
        );
    }

    #[test]
    fn validate_moon_based_ignores_missing_graha_signs() {
        // graha_signs is irrelevant for Moon-based systems.
        assert!(validate(&moon_input()).is_ok());
    }

    #[test]
    fn validate_rejects_moon_longitude_below_zero() {
        let mut input = moon_input();
        input.moon_longitude = Some(-1.0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_moon_longitude_equal_to_360() {
        let mut input = moon_input();
        input.moon_longitude = Some(360.0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_moon_longitude_nan() {
        let mut input = moon_input();
        input.moon_longitude = Some(f64::NAN);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_accepts_lagna_sign_zero_as_aries() {
        // 0 is Aries and VALID as of v9. It was rejected through v8.1.0,
        // when this parameter alone was 1-indexed.
        let mut input = lagna_input();
        input.lagna_sign = Some(0);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn validate_rejects_lagna_sign_above_eleven() {
        let mut input = lagna_input();
        input.lagna_sign = Some(12);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_lagna_sign_far_above_range() {
        let mut input = lagna_input();
        input.lagna_sign = Some(13);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_birth_jd_out_of_range() {
        let mut input = moon_input();
        input.birth_jd = 0.0;
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "DATE_OUT_OF_RANGE"
        );
    }

    #[test]
    fn validate_rejects_levels_zero() {
        let mut input = moon_input();
        input.levels = Some(0);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_rejects_levels_above_max() {
        let mut input = moon_input();
        input.levels = Some(6);
        assert_eq!(
            validate(&input).unwrap_err().error_code,
            "INVALID_PARAMETER"
        );
    }

    #[test]
    fn validate_accepts_max_levels() {
        let mut input = moon_input();
        input.levels = Some(5);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, "compute_dasha");
        let required: Vec<&str> = def.input_schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"birth_jd"));
        // moon_longitude / lagna_sign are conditional on `system` and not in
        // the JSON-Schema `required` list — agents discover the requirement
        // through the field descriptions.
        let systems = def.input_schema["properties"]["system"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(systems.len(), 8);
        // Every system the schema lists is one the parser takes.
        for name in systems {
            assert!(DashaSystem::parse(name.as_str().unwrap()).is_ok(), "{name}");
        }
    }

    fn valid_graha_longitudes() -> GrahaLongitudesInput {
        GrahaLongitudesInput {
            sun: Some(29.0),
            moon: Some(95.0),
            mars: Some(100.0),
            mercury: Some(38.0),
            jupiter: Some(252.0),
            venus: Some(63.0),
            saturn: Some(124.0),
            rahu: Some(350.0),
        }
    }

    fn jaimini_input(system: &str) -> ComputeDashaInput {
        ComputeDashaInput {
            system: Some(system.to_string()),
            birth_jd: 2_451_545.0,
            lagna_sign: Some(0),
            graha_longitudes: Some(valid_graha_longitudes()),
            ..ComputeDashaInput::default()
        }
    }

    #[test]
    fn the_new_systems_parse_under_either_spelling() {
        for (spelling, system) in [
            ("Sthira", DashaSystem::Sthira),
            ("sthira", DashaSystem::Sthira),
            ("NiryanaShoola", DashaSystem::NiryanaShoola),
            ("niryana_shoola", DashaSystem::NiryanaShoola),
            ("Niryana-Shoola", DashaSystem::NiryanaShoola),
            ("MAHESHWARA", DashaSystem::Maheshwara),
        ] {
            assert_eq!(DashaSystem::parse(spelling).unwrap(), system, "{spelling}");
        }
        assert!(DashaSystem::parse("Shoola").is_err());
        assert!(!DashaSystem::Chara.is_longitude_based());
        assert!(DashaSystem::Sthira.is_longitude_based());
    }

    #[test]
    fn validate_asks_the_jaimini_dashas_for_the_lagna_and_the_longitudes() {
        for system in ["Sthira", "NiryanaShoola", "Maheshwara"] {
            assert!(validate(&jaimini_input(system)).is_ok(), "{system}");

            let mut no_lagna = jaimini_input(system);
            no_lagna.lagna_sign = None;
            assert!(
                validate(&no_lagna)
                    .unwrap_err()
                    .message
                    .contains("lagna_sign")
            );

            let mut none = jaimini_input(system);
            none.graha_longitudes = None;
            assert!(
                validate(&none)
                    .unwrap_err()
                    .message
                    .contains("graha_longitudes")
            );

            let mut nan = jaimini_input(system);
            nan.graha_longitudes.as_mut().unwrap().venus = Some(f64::NAN);
            assert!(validate(&nan).unwrap_err().message.contains("venus"));

            // The two ways of naming the grahas may not both be given.
            let mut both = jaimini_input(system);
            both.graha_signs = Some(valid_graha_signs());
            assert!(
                validate(&both)
                    .unwrap_err()
                    .message
                    .contains("graha_longitudes")
            );
        }
    }

    // Ten rounds at four levels is 46,800 periods and passes; one more level
    // is 374,480 and does not. At five levels one round (37,448) passes and
    // two (74,896) do not; `until` 40 years out counts as up to three rounds.
    #[test]
    fn validate_caps_the_yogini_response_size() {
        let yogini = |levels: u8, rounds: Option<u8>, until: Option<f64>| ComputeDashaInput {
            system: Some("Yogini".to_string()),
            levels: Some(levels),
            rounds,
            until,
            ..moon_input()
        };
        assert!(validate(&yogini(4, Some(10), None)).is_ok());
        assert!(validate(&yogini(5, Some(10), None)).is_err());
        assert!(validate(&yogini(5, Some(1), None)).is_ok());
        assert!(validate(&yogini(5, Some(2), None)).is_err());
        assert!(validate(&yogini(5, None, None)).is_ok());
        let forty_years = 2_451_545.0 + 40.0 * 365.25;
        assert!(validate(&yogini(4, None, Some(forty_years))).is_ok());
        assert!(
            validate(&yogini(5, None, Some(forty_years)))
                .unwrap_err()
                .message
                .contains("periods")
        );
        // An `until` inside the first round asks for one round, which five
        // levels may carry.
        assert!(validate(&yogini(5, None, Some(2_451_546.0))).is_ok());
        // An `until` however far ahead is counted as the engine's last round,
        // without overflow: ten rounds fit at four levels and not at five.
        assert!(validate(&yogini(4, None, Some(1e24))).is_ok());
        assert!(validate(&yogini(5, None, Some(1e24))).is_err());
    }

    #[test]
    fn validate_refuses_an_option_on_a_system_it_would_not_move() {
        let mut rounds = moon_input();
        rounds.rounds = Some(2);
        assert!(validate(&rounds).unwrap_err().message.contains("Yogini"));
        let mut until = moon_input();
        until.until = Some(2_460_000.0);
        assert!(validate(&until).unwrap_err().message.contains("Yogini"));
        let mut yogini = moon_input();
        yogini.system = Some("Yogini".to_string());
        yogini.rounds = Some(2);
        assert!(validate(&yogini).is_ok());
        yogini.until = Some(2_460_000.0);
        assert!(validate(&yogini).unwrap_err().message.contains("until"));
        let mut antardashas = lagna_input();
        antardashas.antardashas = Some(true);
        assert!(validate(&antardashas).is_ok());
        antardashas.system = Some("Narayana".to_string());
        assert!(
            validate(&antardashas)
                .unwrap_err()
                .message
                .contains("Chara")
        );
        // `false` is the default and says nothing.
        antardashas.antardashas = Some(false);
        assert!(validate(&antardashas).is_ok());
    }
}
