// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Essential dignities from Ptolemy's table plus modern rulerships.
//!
//! Implements the classical system of essential dignities — domicile,
//! exaltation, detriment, and fall — for both the traditional seven planets
//! and the three modern planets (Uranus, Neptune, Pluto).
//!
//! Sources:
//! - Ptolemy, *Tetrabiblos*, Book I.
//! - William Lilly, *Christian Astrology* (1647).
//! - Modern rulerships: 20th-century consensus (Uranus/Aquarius,
//!   Neptune/Pisces, Pluto/Scorpio).

/// Zodiac sign (0-indexed for array lookup).
///
/// Variants are numbered 0–11 in the standard ecliptic order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sign {
    /// ♈ 0°–30°
    Aries = 0,
    /// ♉ 30°–60°
    Taurus,
    /// ♊ 60°–90°
    Gemini,
    /// ♋ 90°–120°
    Cancer,
    /// ♌ 120°–150°
    Leo,
    /// ♍ 150°–180°
    Virgo,
    /// ♎ 180°–210°
    Libra,
    /// ♏ 210°–240°
    Scorpio,
    /// ♐ 240°–270°
    Sagittarius,
    /// ♑ 270°–300°
    Capricorn,
    /// ♒ 300°–330°
    Aquarius,
    /// ♓ 330°–360°
    Pisces,
}

impl Sign {
    /// Convert a 0-based index (0–11) to a `Sign`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `idx > 11`. In release builds the value is
    /// masked to `idx % 12` to keep array bounds safe.
    #[must_use]
    pub fn from_index(idx: u8) -> Self {
        match idx % 12 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    /// The conventional English name of the sign.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }

    /// The sign directly opposite on the zodiac wheel (sign + 6 mod 12).
    #[must_use]
    pub fn opposite(self) -> Self {
        Self::from_index((self as u8 + 6) % 12)
    }
}

// ─── Planets ──────────────────────────────────────────────────────────────────

/// A planet recognised for dignity purposes: the traditional seven plus the
/// three modern outer planets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DignityPlanet {
    /// ☉ The Sun.
    Sun,
    /// ☽ The Moon.
    Moon,
    /// ☿ Mercury.
    Mercury,
    /// ♀ Venus.
    Venus,
    /// ♂ Mars.
    Mars,
    /// ♃ Jupiter.
    Jupiter,
    /// ♄ Saturn.
    Saturn,
    /// ♅ Uranus (modern outer planet).
    Uranus,
    /// ♆ Neptune (modern outer planet).
    Neptune,
    /// ♇ Pluto (modern outer planet).
    Pluto,
}

// ─── Dignity State ────────────────────────────────────────────────────────────

/// The essential dignity state of a planet placed in a given sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DignityState {
    /// Planet rules this sign — strongest positive essential dignity.
    Domicile,
    /// Planet is exalted in this sign — second-strongest positive dignity.
    Exaltation,
    /// Planet is in its detriment (placed in the sign opposite its domicile).
    Detriment,
    /// Planet is in its fall (placed in the sign opposite its exaltation).
    Fall,
    /// No special dignity — planet is peregrine.
    Peregrine,
}

// ─── Rulership Scheme ─────────────────────────────────────────────────────────

/// Which rulership system to apply when determining domicile and detriment.
///
/// - `Traditional` uses only the seven classical planets and assigns no modern
///   rulers.
/// - `Modern` replaces Mars→Scorpio, Saturn→Aquarius, Jupiter→Pisces with
///   Pluto, Uranus, and Neptune respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulershipScheme {
    /// Seven-planet classical system (Ptolemy, Lilly).
    Traditional,
    /// Includes Uranus, Neptune, and Pluto as co-rulers / primary modern rulers.
    Modern,
}

// ─── Lookup Tables ────────────────────────────────────────────────────────────

/// Traditional domicile rulers indexed by `Sign as u8`.
///
/// Source: Ptolemy, *Tetrabiblos* I.17.
const TRADITIONAL_RULERS: [DignityPlanet; 12] = [
    DignityPlanet::Mars,    // Aries
    DignityPlanet::Venus,   // Taurus
    DignityPlanet::Mercury, // Gemini
    DignityPlanet::Moon,    // Cancer
    DignityPlanet::Sun,     // Leo
    DignityPlanet::Mercury, // Virgo
    DignityPlanet::Venus,   // Libra
    DignityPlanet::Mars,    // Scorpio
    DignityPlanet::Jupiter, // Sagittarius
    DignityPlanet::Saturn,  // Capricorn
    DignityPlanet::Saturn,  // Aquarius
    DignityPlanet::Jupiter, // Pisces
];

/// Modern domicile rulers indexed by `Sign as u8`.
///
/// Identical to traditional except Scorpio → Pluto, Aquarius → Uranus,
/// Pisces → Neptune.
const MODERN_RULERS: [DignityPlanet; 12] = [
    DignityPlanet::Mars,    // Aries
    DignityPlanet::Venus,   // Taurus
    DignityPlanet::Mercury, // Gemini
    DignityPlanet::Moon,    // Cancer
    DignityPlanet::Sun,     // Leo
    DignityPlanet::Mercury, // Virgo
    DignityPlanet::Venus,   // Libra
    DignityPlanet::Pluto,   // Scorpio
    DignityPlanet::Jupiter, // Sagittarius
    DignityPlanet::Saturn,  // Capricorn
    DignityPlanet::Uranus,  // Aquarius
    DignityPlanet::Neptune, // Pisces
];

/// Exaltation placements — `(sign_index, planet)`.
///
/// Only the seven classical planets have traditional exaltations.
/// Source: Ptolemy, *Tetrabiblos* I.19.
const EXALTATIONS: [(u8, DignityPlanet); 7] = [
    (0, DignityPlanet::Sun),     // Aries
    (1, DignityPlanet::Moon),    // Taurus
    (5, DignityPlanet::Mercury), // Virgo
    (11, DignityPlanet::Venus),  // Pisces
    (9, DignityPlanet::Mars),    // Capricorn
    (3, DignityPlanet::Jupiter), // Cancer
    (6, DignityPlanet::Saturn),  // Libra
];

// ─── Public API ───────────────────────────────────────────────────────────────

/// Determine which zodiac sign a given ecliptic longitude falls in.
///
/// The longitude is first normalised to \[0°, 360°) and then divided into
/// 30° segments.
///
/// # Arguments
///
/// * `longitude_deg` — Ecliptic longitude in degrees (any value, including
///   negative).
#[must_use]
pub fn sign_of(longitude_deg: f64) -> Sign {
    let normalized = vedaksha_math::angle::normalize_degrees(longitude_deg);
    // Each sign spans exactly 30°. Cast is safe: value is 0.0–359.999…
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let sign_idx = (normalized / 30.0) as u8;
    Sign::from_index(sign_idx)
}

/// Get the domicile ruler of a sign under the specified rulership scheme.
///
/// # Arguments
///
/// * `sign`   — The zodiac sign to query.
/// * `scheme` — Whether to use traditional or modern rulers.
#[must_use]
pub fn domicile_ruler(sign: Sign, scheme: RulershipScheme) -> DignityPlanet {
    let idx = sign as usize;
    match scheme {
        RulershipScheme::Traditional => TRADITIONAL_RULERS[idx],
        RulershipScheme::Modern => MODERN_RULERS[idx],
    }
}

/// Get the planet exalted in a sign, if any.
///
/// Returns `None` for signs that have no classical exaltation (e.g. the outer
/// planets Uranus, Neptune, and Pluto carry no traditional exaltation).
///
/// # Arguments
///
/// * `sign` — The zodiac sign to query.
#[must_use]
pub fn exaltation_ruler(sign: Sign) -> Option<DignityPlanet> {
    let idx = sign as u8;
    EXALTATIONS.iter().find(|(s, _)| *s == idx).map(|(_, p)| *p)
}

/// Determine the essential dignity state of a planet in a given sign.
///
/// Checks, in order: domicile → exaltation → detriment → fall → peregrine.
///
/// # Arguments
///
/// * `planet` — The planet to assess.
/// * `sign`   — The sign the planet occupies.
/// * `scheme` — Rulership scheme to apply.
#[must_use]
pub fn dignity_of(planet: DignityPlanet, sign: Sign, scheme: RulershipScheme) -> DignityState {
    // Domicile
    if domicile_ruler(sign, scheme) == planet {
        return DignityState::Domicile;
    }

    // Exaltation
    if exaltation_ruler(sign) == Some(planet) {
        return DignityState::Exaltation;
    }

    // Detriment — opposite sign of domicile
    let opposite = sign.opposite();
    if domicile_ruler(opposite, scheme) == planet {
        return DignityState::Detriment;
    }

    // Fall — opposite sign of exaltation
    if let Some(exalt_planet) = exaltation_ruler(opposite) {
        if exalt_planet == planet {
            return DignityState::Fall;
        }
    }

    DignityState::Peregrine
}

/// Convenience wrapper: determine dignity directly from an ecliptic longitude.
///
/// Calls [`sign_of`] then [`dignity_of`].
///
/// # Arguments
///
/// * `planet`        — The planet to assess.
/// * `longitude_deg` — Ecliptic longitude in degrees (any value).
/// * `scheme`        — Rulership scheme to apply.
#[must_use]
pub fn dignity_at_longitude(
    planet: DignityPlanet,
    longitude_deg: f64,
    scheme: RulershipScheme,
) -> DignityState {
    let sign = sign_of(longitude_deg);
    dignity_of(planet, sign, scheme)
}

// ─── Accidental Dignities ────────────────────────────────────────────────────

/// Accidental dignity conditions.
///
/// Unlike essential dignities (based on sign placement), accidental dignities
/// reflect a planet's condition due to its position relative to the Sun,
/// its motion, and its house placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccidentalDignity {
    /// Planet within 0°17' of Sun — extremely powerful.
    Cazimi,
    /// Planet within 8°30' of Sun — weakened (combust).
    Combust,
    /// Planet within 17° of Sun — partially weakened.
    UnderSunBeams,
    /// Planet is retrograde — weakened.
    Retrograde,
    /// Planet is direct and fast (speed > 1°/day) — strengthened.
    DirectAndFast,
    /// Planet in angular house (1, 4, 7, 10) — strengthened.
    Angular,
    /// Planet in succedent house (2, 5, 8, 11) — moderate.
    Succedent,
    /// Planet in cadent house (3, 6, 9, 12) — weakened.
    Cadent,
}

/// Determine accidental dignities for a planet.
///
/// Returns all applicable accidental dignities. A planet may have multiple
/// conditions (e.g. both `Combust` and `Retrograde`).
///
/// # Arguments
///
/// * `planet_longitude` — planet's longitude in degrees.
/// * `sun_longitude`    — Sun's longitude in degrees.
/// * `speed`            — planet's daily speed in degrees.
/// * `house`            — house number (1–12).
#[must_use]
pub fn accidental_dignities(
    planet_longitude: f64,
    sun_longitude: f64,
    speed: f64,
    house: u8,
) -> Vec<AccidentalDignity> {
    let mut dignities = Vec::new();

    let sun_separation = vedaksha_math::angle::angular_separation(planet_longitude, sun_longitude);

    // Cazimi: within 17 arcminutes (0.2833°) of Sun center
    if sun_separation < 17.0 / 60.0 {
        dignities.push(AccidentalDignity::Cazimi);
    } else if sun_separation < 8.5 {
        // Combust: within 8°30' of Sun
        dignities.push(AccidentalDignity::Combust);
    } else if sun_separation < 17.0 {
        // Under Sun's beams: within 17°
        dignities.push(AccidentalDignity::UnderSunBeams);
    }

    // Retrograde check
    if speed < 0.0 {
        dignities.push(AccidentalDignity::Retrograde);
    } else if speed > 1.0 {
        dignities.push(AccidentalDignity::DirectAndFast);
    }

    // House position
    match house {
        1 | 4 | 7 | 10 => dignities.push(AccidentalDignity::Angular),
        2 | 5 | 8 | 11 => dignities.push(AccidentalDignity::Succedent),
        3 | 6 | 9 | 12 => dignities.push(AccidentalDignity::Cadent),
        _ => {}
    }

    dignities
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── sign_of ───────────────────────────────────────────────────────────────

    #[test]
    fn sign_of_0_is_aries() {
        assert_eq!(sign_of(0.0), Sign::Aries);
    }

    #[test]
    fn sign_of_30_is_taurus() {
        assert_eq!(sign_of(30.0), Sign::Taurus);
    }

    #[test]
    fn sign_of_359_is_pisces() {
        assert_eq!(sign_of(359.0), Sign::Pisces);
    }

    #[test]
    fn sign_of_negative_wraps_correctly() {
        // −10° normalises to 350° → Pisces
        assert_eq!(sign_of(-10.0), Sign::Pisces);
        // −360° normalises to 0° → Aries
        assert_eq!(sign_of(-360.0), Sign::Aries);
    }

    #[test]
    fn sign_of_boundary_values() {
        assert_eq!(sign_of(60.0), Sign::Gemini);
        assert_eq!(sign_of(90.0), Sign::Cancer);
        assert_eq!(sign_of(180.0), Sign::Libra);
        assert_eq!(sign_of(270.0), Sign::Capricorn);
    }

    // ── Sign helpers ──────────────────────────────────────────────────────────

    #[test]
    fn opposite_aries_is_libra() {
        assert_eq!(Sign::Aries.opposite(), Sign::Libra);
    }

    #[test]
    fn opposite_cancer_is_capricorn() {
        assert_eq!(Sign::Cancer.opposite(), Sign::Capricorn);
    }

    #[test]
    fn opposite_is_symmetric() {
        for idx in 0u8..12 {
            let sign = Sign::from_index(idx);
            assert_eq!(
                sign.opposite().opposite(),
                sign,
                "{} opposite not symmetric",
                sign.name()
            );
        }
    }

    // ── domicile_ruler ────────────────────────────────────────────────────────

    #[test]
    fn sun_rules_leo_in_both_schemes() {
        assert_eq!(
            domicile_ruler(Sign::Leo, RulershipScheme::Traditional),
            DignityPlanet::Sun
        );
        assert_eq!(
            domicile_ruler(Sign::Leo, RulershipScheme::Modern),
            DignityPlanet::Sun
        );
    }

    #[test]
    fn moon_rules_cancer_traditional() {
        assert_eq!(
            domicile_ruler(Sign::Cancer, RulershipScheme::Traditional),
            DignityPlanet::Moon
        );
    }

    #[test]
    fn scorpio_ruler_mars_traditional_pluto_modern() {
        assert_eq!(
            domicile_ruler(Sign::Scorpio, RulershipScheme::Traditional),
            DignityPlanet::Mars
        );
        assert_eq!(
            domicile_ruler(Sign::Scorpio, RulershipScheme::Modern),
            DignityPlanet::Pluto
        );
    }

    #[test]
    fn aquarius_ruler_saturn_traditional_uranus_modern() {
        assert_eq!(
            domicile_ruler(Sign::Aquarius, RulershipScheme::Traditional),
            DignityPlanet::Saturn
        );
        assert_eq!(
            domicile_ruler(Sign::Aquarius, RulershipScheme::Modern),
            DignityPlanet::Uranus
        );
    }

    #[test]
    fn pisces_ruler_jupiter_traditional_neptune_modern() {
        assert_eq!(
            domicile_ruler(Sign::Pisces, RulershipScheme::Traditional),
            DignityPlanet::Jupiter
        );
        assert_eq!(
            domicile_ruler(Sign::Pisces, RulershipScheme::Modern),
            DignityPlanet::Neptune
        );
    }

    // ── exaltation_ruler ──────────────────────────────────────────────────────

    #[test]
    fn sun_exalted_in_aries() {
        assert_eq!(exaltation_ruler(Sign::Aries), Some(DignityPlanet::Sun));
    }

    #[test]
    fn saturn_exalted_in_libra() {
        assert_eq!(exaltation_ruler(Sign::Libra), Some(DignityPlanet::Saturn));
    }

    #[test]
    fn mars_exalted_in_capricorn() {
        assert_eq!(exaltation_ruler(Sign::Capricorn), Some(DignityPlanet::Mars));
    }

    #[test]
    fn no_exaltation_in_aries_for_moon() {
        // Moon is exalted in Taurus, not Aries
        assert_ne!(exaltation_ruler(Sign::Aries), Some(DignityPlanet::Moon));
    }

    #[test]
    fn gemini_has_no_classical_exaltation() {
        assert_eq!(exaltation_ruler(Sign::Gemini), None);
    }

    // ── dignity_of ────────────────────────────────────────────────────────────

    #[test]
    fn sun_domicile_in_leo() {
        assert_eq!(
            dignity_of(DignityPlanet::Sun, Sign::Leo, RulershipScheme::Traditional),
            DignityState::Domicile
        );
    }

    #[test]
    fn sun_detriment_in_aquarius() {
        assert_eq!(
            dignity_of(
                DignityPlanet::Sun,
                Sign::Aquarius,
                RulershipScheme::Traditional
            ),
            DignityState::Detriment
        );
    }

    #[test]
    fn sun_exaltation_in_aries() {
        assert_eq!(
            dignity_of(
                DignityPlanet::Sun,
                Sign::Aries,
                RulershipScheme::Traditional
            ),
            DignityState::Exaltation
        );
    }

    #[test]
    fn sun_fall_in_libra() {
        // Sun exalted in Aries → fall in Libra (opposite)
        assert_eq!(
            dignity_of(
                DignityPlanet::Sun,
                Sign::Libra,
                RulershipScheme::Traditional
            ),
            DignityState::Fall
        );
    }

    #[test]
    fn mars_exaltation_in_capricorn() {
        assert_eq!(
            dignity_of(
                DignityPlanet::Mars,
                Sign::Capricorn,
                RulershipScheme::Traditional
            ),
            DignityState::Exaltation
        );
    }

    #[test]
    fn mars_fall_in_cancer() {
        // Mars exalted in Capricorn → fall in Cancer
        assert_eq!(
            dignity_of(
                DignityPlanet::Mars,
                Sign::Cancer,
                RulershipScheme::Traditional
            ),
            DignityState::Fall
        );
    }

    #[test]
    fn moon_domicile_in_cancer_traditional() {
        assert_eq!(
            dignity_of(
                DignityPlanet::Moon,
                Sign::Cancer,
                RulershipScheme::Traditional
            ),
            DignityState::Domicile
        );
    }

    #[test]
    fn sun_peregrine_in_gemini() {
        // Sun has no rulership, exaltation, detriment, or fall in Gemini
        assert_eq!(
            dignity_of(
                DignityPlanet::Sun,
                Sign::Gemini,
                RulershipScheme::Traditional
            ),
            DignityState::Peregrine
        );
    }

    #[test]
    fn pluto_domicile_in_scorpio_modern_only() {
        assert_eq!(
            dignity_of(DignityPlanet::Pluto, Sign::Scorpio, RulershipScheme::Modern),
            DignityState::Domicile
        );
        // Pluto has no dignity in traditional scheme
        assert_eq!(
            dignity_of(
                DignityPlanet::Pluto,
                Sign::Scorpio,
                RulershipScheme::Traditional
            ),
            DignityState::Peregrine
        );
    }

    #[test]
    fn uranus_domicile_in_aquarius_modern_only() {
        assert_eq!(
            dignity_of(
                DignityPlanet::Uranus,
                Sign::Aquarius,
                RulershipScheme::Modern
            ),
            DignityState::Domicile
        );
        assert_eq!(
            dignity_of(
                DignityPlanet::Uranus,
                Sign::Aquarius,
                RulershipScheme::Traditional
            ),
            DignityState::Peregrine
        );
    }

    // ── dignity_at_longitude ──────────────────────────────────────────────────

    #[test]
    fn dignity_at_longitude_sun_in_leo_by_longitude() {
        // 130° = 10° Leo → Sun domicile
        assert_eq!(
            dignity_at_longitude(DignityPlanet::Sun, 130.0, RulershipScheme::Traditional),
            DignityState::Domicile
        );
    }

    #[test]
    fn dignity_at_longitude_wraps_negative() {
        // −10° → 350° → Pisces; Jupiter rules Pisces traditionally
        assert_eq!(
            dignity_at_longitude(DignityPlanet::Jupiter, -10.0, RulershipScheme::Traditional),
            DignityState::Domicile
        );
    }

    // ── accidental_dignities ─────────────────────────────────────────────────

    #[test]
    fn cazimi_within_17_arcminutes_of_sun() {
        // Planet 0.1° from Sun → within 0.2833° → Cazimi
        let result = accidental_dignities(100.1, 100.0, 1.0, 1);
        assert!(
            result.contains(&AccidentalDignity::Cazimi),
            "Expected Cazimi, got {result:?}"
        );
    }

    #[test]
    fn combust_within_8_5_degrees_of_sun() {
        // Planet 5° from Sun → Combust
        let result = accidental_dignities(105.0, 100.0, 1.0, 1);
        assert!(
            result.contains(&AccidentalDignity::Combust),
            "Expected Combust, got {result:?}"
        );
    }

    #[test]
    fn under_sun_beams_within_17_degrees() {
        // Planet 15° from Sun → Under Sun's beams
        let result = accidental_dignities(115.0, 100.0, 1.0, 1);
        assert!(
            result.contains(&AccidentalDignity::UnderSunBeams),
            "Expected UnderSunBeams, got {result:?}"
        );
    }

    #[test]
    fn retrograde_with_negative_speed() {
        let result = accidental_dignities(200.0, 100.0, -0.5, 1);
        assert!(
            result.contains(&AccidentalDignity::Retrograde),
            "Expected Retrograde, got {result:?}"
        );
    }

    #[test]
    fn direct_and_fast_with_high_speed() {
        let result = accidental_dignities(200.0, 100.0, 1.5, 1);
        assert!(
            result.contains(&AccidentalDignity::DirectAndFast),
            "Expected DirectAndFast, got {result:?}"
        );
    }

    #[test]
    fn angular_house() {
        let result = accidental_dignities(200.0, 100.0, 1.0, 1);
        assert!(
            result.contains(&AccidentalDignity::Angular),
            "Expected Angular for house 1, got {result:?}"
        );

        let result10 = accidental_dignities(200.0, 100.0, 1.0, 10);
        assert!(
            result10.contains(&AccidentalDignity::Angular),
            "Expected Angular for house 10, got {result10:?}"
        );
    }

    #[test]
    fn cadent_house() {
        let result = accidental_dignities(200.0, 100.0, 1.0, 3);
        assert!(
            result.contains(&AccidentalDignity::Cadent),
            "Expected Cadent for house 3, got {result:?}"
        );

        let result12 = accidental_dignities(200.0, 100.0, 1.0, 12);
        assert!(
            result12.contains(&AccidentalDignity::Cadent),
            "Expected Cadent for house 12, got {result12:?}"
        );
    }

    #[test]
    fn succedent_house() {
        let result = accidental_dignities(200.0, 100.0, 1.0, 2);
        assert!(
            result.contains(&AccidentalDignity::Succedent),
            "Expected Succedent for house 2, got {result:?}"
        );
    }

    #[test]
    fn no_sun_condition_beyond_17_degrees() {
        let result = accidental_dignities(200.0, 100.0, 1.0, 1);
        assert!(
            !result.contains(&AccidentalDignity::Cazimi),
            "Should not be Cazimi at 100° separation"
        );
        assert!(
            !result.contains(&AccidentalDignity::Combust),
            "Should not be Combust at 100° separation"
        );
        assert!(
            !result.contains(&AccidentalDignity::UnderSunBeams),
            "Should not be UnderSunBeams at 100° separation"
        );
    }
}
