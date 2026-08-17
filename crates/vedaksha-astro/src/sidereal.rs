// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Ayanamsha and sidereal zodiac conversion.
//!
//! # QUARANTINED — values removed pending primary-source re-derivation
//!
//! Every ayanamsha constant previously in this module has been removed. They are
//! being re-derived from primary definitions behind a two-agent firewall; see
//! `docs/audit/2026-08-<date>-ayanamsha-cleanroom/`.
//!
//! The removal is deliberate and this module is intentionally non-functional in
//! this commit: [`ayanamsha_value`] panics. The values were removed *before* the
//! re-derivation branch was cut so that the implementation agent's working tree
//! could not contain them. A derivation that can see its target is not a
//! derivation.
//!
//! The enum below is retained only so the workspace continues to compile. It is
//! **not** authority for which systems Vedaksha will expose: the surviving list is
//! rebuilt from primary literature, not inherited from this taxonomy.

/// Ayanamsha system selector.
///
/// Each variant represents a distinct tradition for defining the relationship
/// between the tropical and sidereal zodiacs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ayanamsha {
    /// Lahiri (Chitrapaksha) — Indian government standard.
    ///
    /// Based on the star Spica (Chitra) at 0° Libra.
    /// Source: Indian Calendar Reform Committee (1955).
    Lahiri,

    /// Raman ayanamsha.
    ///
    Raman,

    /// Krishnamurti — K. S. Krishnamurti Paddhati (KP system).
    Krishnamurti,

    /// Fagan–Bradley — Western sidereal astrology standard.
    ///
    /// Source: Cyril Fagan & Donald Bradley, *Primer of Sidereal Astrology* (1967).
    FaganBradley,

    /// Yukteshwar — Sri Yukteshwar's system from *The Holy Science* (1894).
    Yukteshwar,

    /// JN Bhasin — J. N. Bhasin's ayanamsha.
    JnBhasin,

    /// Djwhal Khul — Tibetan / Alice Bailey esoteric system.
    DjwhalKhul,

    /// Sassanian / Aldebaran at 15° Taurus.
    Aldebaran15Tau,

    /// Hipparchos — based on Hipparchus's original star catalogue.
    Hipparchos,

    /// Galactic Center at 0° Sagittarius (Mula nakshatra).
    GalacticCenter0Sag,

    /// True Chitrapaksha — Spica placed exactly at 180° ecliptic longitude.
    TrueChitrapaksha,

    /// Tropical — identity (0° ayanamsha).
    ///
    /// Included for convenience so callers can pass a uniform `Ayanamsha`
    /// value and get tropical coordinates back unchanged.
    Tropical,

    // ── Additional systems ────────────────────────────────────────────────────
    /// De Luce — Robert De Luce's Western sidereal system.
    DeLuce,

    /// Raman Mean Ayanamsha — alternative Raman computation, same anchor.
    BvRamanMean,

    /// Usha-Shashi — Usha and Shashi ayanamsha.
    UshaShashi,

    /// Krishnamurti 2 — second KP reference value.
    Krishnamurti2,

    /// Surya Siddhanta — classical Indian astronomical text.
    SuryaSiddhanta,

    /// Surya Siddhanta (Mean) — mean-sun variant of SS ayanamsha.
    SuryaSiddhantaMean,

    /// Aryabhata — based on Aryabhata's Aryabhatiya (499 CE).
    Aryabhata,

    /// Aryabhata (528 CE) — later Aryabhata reference.
    Aryabhata528,

    /// SS Drev-Jul — Surya Siddhanta with Drev-Jul correction.
    SsDrevJul,

    /// SS Citra — Surya Siddhanta Citra-paksha variant.
    SsCitra,

    /// True Pushya — Pushya nakshatra at exact 93° ecliptic.
    TruePushya,

    /// True Revati — Revati star placed at 0° Aries.
    TrueRevati,

    /// True Mula — Mula nakshatra at galactic center alignment.
    TrueMula,

    /// Sundara Rajan — V. Sundara Rajan's ayanamsha.
    SundaraRajan,

    /// Babylonian (Huber) — Peter Huber's Babylonian star-catalog reconstruction.
    BabylonianHuber,

    /// Babylonian (ETPSC) — Babylonian ayanamsha per ETPSC standard.
    BabylonianEtpsc,

    /// Babylonian (Kugler Star 1) — Kugler's first Babylonian star reference.
    BabylonianKuglerStar1,

    /// Babylonian (Kugler Star 2) — Kugler's second Babylonian star reference.
    BabylonianKuglerStar2,

    /// Babylonian (Kugler Star 3) — Kugler's third Babylonian star reference.
    BabylonianKuglerStar3,

    /// Sassanian — Persian/Sassanid astrological tradition.
    Sassanian,

    /// Galactic Center Brand — Brand's galactic center definition.
    GalacticCenterBrand,

    /// Galactic Center Galactic Alignment — precise GC alignment system.
    GalacticCenterGalAlign,

    /// Galactic Equator IAU 1958 — IAU 1958 galactic equator pole.
    GalacticEquatorIau1958,

    /// Galactic Equator True — true galactic equator crossing.
    GalacticEquatorTrue,

    /// Galactic Equator Mid-Mula — galactic equator at mid-Mula nakshatra.
    GalacticEquatorMidMula,

    /// Skydram — Skydram astrological system.
    Skydram,

    /// True Moon's Node — uses the mean lunar node for reference.
    TrueMoonsNode,

    /// Lahiri 1940 — early Lahiri reference value (pre-reform).
    Lahiri1940,

    /// Lahiri VP285 — Lahiri ayanamsha per Vishnu Purana 285 reference.
    LahiriVp285,

    /// Valensmoon — Valen's lunar-referenced ayanamsha.
    ValensMoon,

    /// Ayanamsha Of Date — computed from current date via Newcomb precession.
    AyanamshaOfDate,

    /// Djwhal Khul Tibetan 2 — alternate Alice Bailey esoteric reference.
    DjwhalKhulTibetan2,
}

impl Ayanamsha {
    /// Returns the conventional name of the ayanamsha system.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Lahiri => "Lahiri (Chitrapaksha)",
            Self::Raman => "Raman",
            Self::Krishnamurti => "Krishnamurti (KP)",
            Self::FaganBradley => "Fagan-Bradley",
            Self::Yukteshwar => "Yukteshwar",
            Self::JnBhasin => "JN Bhasin",
            Self::DjwhalKhul => "Djwhal Khul (Tibetan)",
            Self::Aldebaran15Tau => "Aldebaran at 15° Taurus (Sassanian)",
            Self::Hipparchos => "Hipparchos",
            Self::GalacticCenter0Sag => "Galactic Center at 0° Sagittarius",
            Self::TrueChitrapaksha => "True Chitrapaksha",
            Self::Tropical => "Tropical (0°)",
            Self::DeLuce => "De Luce",
            Self::BvRamanMean => "B. V. Raman Mean",
            Self::UshaShashi => "Usha-Shashi",
            Self::Krishnamurti2 => "Krishnamurti 2",
            Self::SuryaSiddhanta => "Surya Siddhanta",
            Self::SuryaSiddhantaMean => "Surya Siddhanta (Mean)",
            Self::Aryabhata => "Aryabhata",
            Self::Aryabhata528 => "Aryabhata (528 CE)",
            Self::SsDrevJul => "SS Drev-Jul",
            Self::SsCitra => "SS Citra",
            Self::TruePushya => "True Pushya",
            Self::TrueRevati => "True Revati",
            Self::TrueMula => "True Mula",
            Self::SundaraRajan => "Sundara Rajan",
            Self::BabylonianHuber => "Babylonian (Huber)",
            Self::BabylonianEtpsc => "Babylonian (ETPSC)",
            Self::BabylonianKuglerStar1 => "Babylonian (Kugler Star 1)",
            Self::BabylonianKuglerStar2 => "Babylonian (Kugler Star 2)",
            Self::BabylonianKuglerStar3 => "Babylonian (Kugler Star 3)",
            Self::Sassanian => "Sassanian",
            Self::GalacticCenterBrand => "Galactic Center (Brand)",
            Self::GalacticCenterGalAlign => "Galactic Center (Galactic Alignment)",
            Self::GalacticEquatorIau1958 => "Galactic Equator IAU 1958",
            Self::GalacticEquatorTrue => "Galactic Equator (True)",
            Self::GalacticEquatorMidMula => "Galactic Equator Mid-Mula",
            Self::Skydram => "Skydram",
            Self::TrueMoonsNode => "True Moon's Node",
            Self::Lahiri1940 => "Lahiri 1940",
            Self::LahiriVp285 => "Lahiri VP285",
            Self::ValensMoon => "Valensmoon",
            Self::AyanamshaOfDate => "Ayanamsha Of Date",
            Self::DjwhalKhulTibetan2 => "Djwhal Khul Tibetan 2",
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Compute the ayanamsha value in decimal degrees for a given Julian Day.
///
/// # Panics
///
/// Always. The constants this function returned were removed pending
/// primary-source re-derivation; see the module docs and
/// `docs/audit/2026-08-<date>-ayanamsha-cleanroom/`.
#[must_use]
pub fn ayanamsha_value(_system: Ayanamsha, _jd: f64) -> f64 {
    unimplemented!(
        "ayanamsha constants are quarantined pending primary-source re-derivation; \
         see docs/audit/2026-08-<date>-ayanamsha-cleanroom/"
    )
}

/// Convert a tropical ecliptic longitude to sidereal longitude.
///
/// ```text
/// sidereal = tropical − ayanamsha
/// ```
///
/// # Panics
///
/// Always, while [`ayanamsha_value`] is quarantined.
#[must_use]
pub fn tropical_to_sidereal(tropical_longitude_deg: f64, system: Ayanamsha, jd: f64) -> f64 {
    let ayan = ayanamsha_value(system, jd);
    vedaksha_math::angle::normalize_degrees(tropical_longitude_deg - ayan)
}

/// Convert a sidereal ecliptic longitude to tropical longitude.
///
/// ```text
/// tropical = sidereal + ayanamsha
/// ```
///
/// # Panics
///
/// Always, while [`ayanamsha_value`] is quarantined.
#[must_use]
pub fn sidereal_to_tropical(sidereal_longitude_deg: f64, system: Ayanamsha, jd: f64) -> f64 {
    let ayan = ayanamsha_value(system, jd);
    vedaksha_math::angle::normalize_degrees(sidereal_longitude_deg + ayan)
}
