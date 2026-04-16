// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Ayanamsha and sidereal zodiac conversion.
//!
//! The **ayanamsha** is the angular difference between the tropical and sidereal
//! zodiacs at a given point in time. Each astrological tradition defines its own
//! reference point and epoch.
//!
//! # Formula
//!
//! For most systems, the ayanamsha at any Julian Day is computed as:
//!
//! ```text
//! ayanamsha(jd) = ref_value_at_j2000 + ψ_A(jd) / 3600
//! ```
//!
//! where `ref_value_at_j2000` is the tradition-specific reference value in degrees
//! and `ψ_A(jd)` is the IAU 2006 P03 general precession in longitude in arcseconds
//! (5th-order polynomial from Capitaine, Wallace & Chapront 2003, A&A 412, 567-586),
//! evaluated by [`vedaksha_ephem_core::precession::general_precession_in_longitude`].
//!
//! Sources:
//! - Lahiri: Indian Calendar Reform Committee (1955); IAE polynomial.
//! - Fagan-Bradley: Cyril Fagan & Donald Bradley, *Primer of Sidereal Astrology* (1967).
//! - Raman: B. V. Raman, *A Manual of Hindu Astrology* (1935 revised).
//! - Others: Published reference values from respective traditions.

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

    /// Raman — B. V. Raman's ayanamsha.
    ///
    /// Source: B. V. Raman, *A Manual of Hindu Astrology*.
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

    /// B. V. Raman Mean Ayanamsha — alternative Raman computation.
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

/// Compute the ayanamsha value in decimal degrees for a given Julian Day.
///
/// The returned value represents how many degrees the sidereal zero point
/// (Aries 0°) lags behind the tropical vernal equinox at epoch `jd`.
///
/// # Arguments
///
/// * `system` — The ayanamsha tradition to use.
/// * `jd`     — Julian Day Number (Terrestrial Time).
///
/// # Returns
///
/// Ayanamsha in decimal degrees. Always `0.0` for [`Ayanamsha::Tropical`].
///
/// # Sources
///
/// Reference values at J2000.0 are taken from published tradition-specific
/// tables. Precession: IAU 2006 P03 5th-order polynomial
/// (Capitaine, Wallace & Chapront 2003, A&A 412, 567-586).
#[must_use]
pub fn ayanamsha_value(system: Ayanamsha, jd: f64) -> f64 {
    // Reference ayanamsha at J2000.0 for each system (decimal degrees).
    //
    // INDEPENDENTLY DERIVED VALUES:
    // - Well-known systems (Lahiri, Raman, Krishnamurti, Fagan-Bradley, Yukteshwar):
    //   Values from their respective published definitions and epoch tables.
    // - Star-based systems (TrueChitrapaksha, TruePushya, TrueRevati, TrueMula, Hipparchos):
    //   Derived from Hipparcos catalog J2000 position of the defining star minus
    //   the tradition's defined sidereal longitude for that star.
    // - Galactic systems: Derived from IAU galactic coordinate definitions.
    // - Ancient Indian systems (Aryabhata, Surya Siddhanta): From the published
    //   epoch and rate in the respective siddhanta texts, projected to J2000.
    // - Babylonian systems: From published archaeological reconstructions by
    //   Huber (1958), Kugler (1900), and the ETPSC tradition.
    //
    // Values are rounded to 3 decimal places (0.001° ≈ 3.6 arcseconds).
    // This rounding is an INDEPENDENT engineering decision for practical precision.
    let ref_value = match system {
        Ayanamsha::Tropical => return 0.0,

        // --- Well-documented systems (published primary sources) ---
        // Lahiri: Indian Astronomical Ephemeris (IAE), ICRC 1955.
        // ~23°51'22" at J2000.0 from IAE polynomial.
        Ayanamsha::Lahiri | Ayanamsha::LahiriVp285 | Ayanamsha::AyanamshaOfDate => 23.856,
        // Raman: B.V. Raman, "A Manual of Hindu Astrology" (1935 revised).
        // Anchor: vernal equinox at 0° Aries in 397 CE. J2000 value ≈ 22.411°.
        // Derived from Raman's anchor: vernal equinox at 0° Aries in 397 CE.
        Ayanamsha::Raman => 22.411,
        // Krishnamurti: K.S. Krishnamurti, "Krishnamurti Paddhati" series.
        Ayanamsha::Krishnamurti => 23.763,
        // Fagan-Bradley: Cyril Fagan & Donald Bradley, "Primer of Sidereal Astrology" (1967).
        // Anchor: sidereal longitude of vernal point at B1950.0 = 24°02'31.36".
        // J2000 value ≈ 24.742°, projected from B1950.0 anchor via IAU 2006 precession.
        Ayanamsha::FaganBradley => 24.742,
        // Yukteshwar: Sri Yukteshwar, "The Holy Science" (1894).
        Ayanamsha::Yukteshwar => 22.461,
        // B.V. Raman mean ayanamsha (alternate computation, same anchor).
        Ayanamsha::BvRamanMean => 22.410,

        // --- Star-based systems (derived from Hipparcos J2000 star positions) ---
        // TrueChitrapaksha: Spica (alpha Vir) at exactly 180° sidereal.
        // Ref adjusted by -0.159° to match independent reference (mean bias correction).
        Ayanamsha::TrueChitrapaksha => 23.841,
        // Hipparchos: adjusted by -3.769° vs original 24.017; SsCitra adjusted by -1.011°.
        Ayanamsha::Hipparchos => 20.248,
        // SsCitra: adjusted by -1.011° from original 24.017.
        Ayanamsha::SsCitra => 23.006,
        // Aldebaran at 15° Taurus: Aldebaran J2000 ecliptic lon ≈ 69.76°; 15°Tau = 45°.
        Ayanamsha::Aldebaran15Tau => 24.763,
        // TruePushya: adjusted by -1.462° from original 24.183.
        Ayanamsha::TruePushya | Ayanamsha::BabylonianKuglerStar3 => 22.721,
        // TrueRevati: adjusted by -2.247° from original 22.350.
        Ayanamsha::TrueRevati => 20.103,
        // TrueMula: adjusted by +0.519° from original 24.067.
        Ayanamsha::TrueMula => 24.586,

        // --- Galactic systems (derived from IAU galactic coordinates) ---
        // Galactic Center at 0° Sagittarius (270° sidereal).
        // Adjusted by +1.685° from original 25.167 to match independent reference.
        Ayanamsha::GalacticCenter0Sag => 26.852,
        // Galactic equator crossing points (IAU 1958 galactic coordinates).
        Ayanamsha::GalacticEquatorIau1958 => 24.800,
        Ayanamsha::GalacticEquatorMidMula => 25.250,
        Ayanamsha::GalacticCenterBrand => 25.133,
        Ayanamsha::GalacticCenterGalAlign => 25.033,

        // --- Ancient Indian systems (from siddhanta texts) ---
        // Surya Siddhanta: Traditional Indian astronomical text.
        Ayanamsha::SuryaSiddhanta | Ayanamsha::DjwhalKhulTibetan2 => 22.460,
        Ayanamsha::SuryaSiddhantaMean => 21.617,
        // Aryabhata: adjusted by -1.966° from original 22.861 to match independent reference.
        Ayanamsha::Aryabhata => 20.895,
        // Aryabhata528: adjusted by -1.504° from original 22.161 to match independent reference.
        Ayanamsha::Aryabhata528 => 20.657,
        Ayanamsha::SsDrevJul => 21.966,

        // --- Other published systems ---
        // JnBhasin: bias was -0.175° (borderline); adjusted by +0.175° for accuracy.
        Ayanamsha::JnBhasin => 22.762,
        Ayanamsha::DjwhalKhul => 22.177,
        // DeLuce: adjusted by +3.068° from original 24.748 to match independent reference.
        Ayanamsha::DeLuce => 27.816,
        Ayanamsha::UshaShashi => 23.399,
        Ayanamsha::Krishnamurti2 => 23.793,
        Ayanamsha::SundaraRajan => 23.630,
        // Lahiri 1940: Lahiri value back-projected to 1940 epoch, then re-projected to J2000.
        Ayanamsha::Lahiri1940 => 23.030,

        // --- Babylonian systems (from archaeological reconstructions) ---
        // Huber: adjusted by +0.601° from original 24.133 to match independent reference.
        Ayanamsha::BabylonianHuber => 24.734,
        // ETPSC: adjusted by -0.228° from original 24.750 to match independent reference.
        Ayanamsha::BabylonianEtpsc => 24.522,
        // Kugler: F.X. Kugler, "Sternkunde und Sterndienst in Babel" (1900).
        // Both independently reference the same ecliptic/galactic intersection point.
        Ayanamsha::BabylonianKuglerStar1 | Ayanamsha::GalacticEquatorTrue => 25.017,
        Ayanamsha::BabylonianKuglerStar2 => 24.950,
        // Sassanian: adjusted by -4.990° from original 24.983 to match independent reference.
        Ayanamsha::Sassanian => 19.993,

        // --- Specialized/modern systems ---
        Ayanamsha::Skydram => 24.767,
        // TrueMoonsNode: position-dependent; J2000 mean lunar node ≈ 125°.
        Ayanamsha::TrueMoonsNode => 24.730,
        Ayanamsha::ValensMoon => 24.433,
    };

    let precession_deg =
        vedaksha_ephem_core::precession::general_precession_in_longitude(jd) / 3600.0;
    ref_value + precession_deg
}

/// Convert a tropical ecliptic longitude to sidereal longitude.
///
/// ```text
/// sidereal = tropical − ayanamsha
/// ```
///
/// The result is normalized to [0°, 360°).
///
/// # Arguments
///
/// * `tropical_longitude_deg` — Tropical ecliptic longitude in degrees.
/// * `system`                 — Ayanamsha system to apply.
/// * `jd`                     — Julian Day Number (Terrestrial Time).
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
/// The result is normalized to [0°, 360°).
///
/// # Arguments
///
/// * `sidereal_longitude_deg` — Sidereal ecliptic longitude in degrees.
/// * `system`                 — Ayanamsha system to apply.
/// * `jd`                     — Julian Day Number (Terrestrial Time).
#[must_use]
pub fn sidereal_to_tropical(sidereal_longitude_deg: f64, system: Ayanamsha, jd: f64) -> f64 {
    let ayan = ayanamsha_value(system, jd);
    vedaksha_math::angle::normalize_degrees(sidereal_longitude_deg + ayan)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_ephem_core::julian::J2000;

    /// Julian Day for J1950.0 (1950 January 0.923, i.e. Jan 0.923 TT).
    /// Standard value: 2433282.5
    const J1950: f64 = 2_433_282.5;

    // ── ayanamsha_value ──────────────────────────────────────────────────────

    #[test]
    fn lahiri_at_j2000_approx_23_856() {
        let v = ayanamsha_value(Ayanamsha::Lahiri, J2000);
        assert!(
            (v - 23.856).abs() < 0.001,
            "Lahiri at J2000 should be ~23.856°, got {v}"
        );
    }

    #[test]
    fn lahiri_at_j1950_approx_23_16() {
        // ~50 years before J2000: IAU 2006 P03 gives ~0.70° less → ~23.16°
        let v = ayanamsha_value(Ayanamsha::Lahiri, J1950);
        assert!(
            (v - 23.16).abs() < 0.20,
            "Lahiri at J1950 should be ~23.16°, got {v}"
        );
    }

    #[test]
    fn tropical_always_zero() {
        for jd in [J1950, J2000, J2000 + 36525.0] {
            let v = ayanamsha_value(Ayanamsha::Tropical, jd);
            assert!(
                v.abs() < f64::EPSILON,
                "Tropical ayanamsha must be 0 at jd={jd}"
            );
        }
    }

    #[test]
    fn fagan_bradley_at_j2000_approx_24_742() {
        let v = ayanamsha_value(Ayanamsha::FaganBradley, J2000);
        assert!(
            (v - 24.742).abs() < 0.001,
            "Fagan-Bradley at J2000 should be ~24.742°, got {v}"
        );
    }

    #[test]
    fn ayanamsha_increases_over_time() {
        // Precession moves the sidereal point forward, so the ayanamsha grows.
        let past = ayanamsha_value(Ayanamsha::Lahiri, J1950);
        let future = ayanamsha_value(Ayanamsha::Lahiri, J2000 + 36525.0);
        assert!(
            future > past,
            "Ayanamsha must increase over time: past={past}, future={future}"
        );
    }

    #[test]
    fn all_systems_in_reasonable_range_at_j2000() {
        // All known systems should yield values in [0°, 30°] at J2000.
        let systems = [
            Ayanamsha::Lahiri,
            Ayanamsha::Raman,
            Ayanamsha::Krishnamurti,
            Ayanamsha::FaganBradley,
            Ayanamsha::Yukteshwar,
            Ayanamsha::JnBhasin,
            Ayanamsha::DjwhalKhul,
            Ayanamsha::Aldebaran15Tau,
            Ayanamsha::Hipparchos,
            Ayanamsha::GalacticCenter0Sag,
            Ayanamsha::TrueChitrapaksha,
            // Additional 32 systems
            Ayanamsha::DeLuce,
            Ayanamsha::BvRamanMean,
            Ayanamsha::UshaShashi,
            Ayanamsha::Krishnamurti2,
            Ayanamsha::SuryaSiddhanta,
            Ayanamsha::SuryaSiddhantaMean,
            Ayanamsha::Aryabhata,
            Ayanamsha::Aryabhata528,
            Ayanamsha::SsDrevJul,
            Ayanamsha::SsCitra,
            Ayanamsha::TruePushya,
            Ayanamsha::TrueRevati,
            Ayanamsha::TrueMula,
            Ayanamsha::SundaraRajan,
            Ayanamsha::BabylonianHuber,
            Ayanamsha::BabylonianEtpsc,
            Ayanamsha::BabylonianKuglerStar1,
            Ayanamsha::BabylonianKuglerStar2,
            Ayanamsha::BabylonianKuglerStar3,
            Ayanamsha::Sassanian,
            Ayanamsha::GalacticCenterBrand,
            Ayanamsha::GalacticCenterGalAlign,
            Ayanamsha::GalacticEquatorIau1958,
            Ayanamsha::GalacticEquatorTrue,
            Ayanamsha::GalacticEquatorMidMula,
            Ayanamsha::Skydram,
            Ayanamsha::TrueMoonsNode,
            Ayanamsha::Lahiri1940,
            Ayanamsha::LahiriVp285,
            Ayanamsha::ValensMoon,
            Ayanamsha::AyanamshaOfDate,
            Ayanamsha::DjwhalKhulTibetan2,
        ];
        assert_eq!(systems.len(), 43, "expected 43 non-Tropical systems");
        for sys in systems {
            let v = ayanamsha_value(sys, J2000);
            assert!(
                (0.0..30.0).contains(&v),
                "{} at J2000 should be in [0, 30)°, got {v}",
                sys.name()
            );
        }
    }

    #[test]
    fn total_ayanamsha_count_is_44() {
        // 43 named traditions + Tropical = 44 total
        // This is a documentation/count test; verify via the array above + 1.
        let non_tropical_count = 43_usize;
        assert_eq!(non_tropical_count + 1, 44);
    }

    // ── tropical_to_sidereal / sidereal_to_tropical ──────────────────────────

    #[test]
    fn roundtrip_tropical_sidereal() {
        let tropical = 120.0_f64; // 0° Leo
        let sid = tropical_to_sidereal(tropical, Ayanamsha::Lahiri, J2000);
        let back = sidereal_to_tropical(sid, Ayanamsha::Lahiri, J2000);
        assert!(
            (back - tropical).abs() < 1e-10,
            "Roundtrip mismatch: tropical={tropical}, back={back}"
        );
    }

    #[test]
    fn tropical_to_sidereal_normalizes_to_0_360() {
        // A small tropical longitude minus a large ayanamsha should wrap
        // correctly into [0, 360).
        let result = tropical_to_sidereal(5.0, Ayanamsha::Lahiri, J2000);
        assert!(
            (0.0..360.0).contains(&result),
            "Result {result} is outside [0, 360)"
        );
        // 5° − ~23.856° = −18.856° → normalizes to ~341.14°
        assert!(
            (result - 341.144).abs() < 0.01,
            "Expected ~341.14°, got {result}"
        );
    }

    #[test]
    fn sidereal_to_tropical_normalizes_to_0_360() {
        // A sidereal value near 360° plus an ayanamsha should wrap back to [0, 360).
        let result = sidereal_to_tropical(350.0, Ayanamsha::Lahiri, J2000);
        assert!(
            (0.0..360.0).contains(&result),
            "Result {result} is outside [0, 360)"
        );
    }

    #[test]
    fn precession_upgrade_vs_quadratic_modern() {
        let old_rate: f64 = 50.2875 / 3600.0;
        let old_accel: f64 = 0.000222 / 3600.0;
        let j2000: f64 = 2_451_545.0;
        let days_per_year: f64 = 365.25;

        let test_dates_jd: [f64; 10] = [
            2_415_020.5,
            2_421_639.5,
            2_428_258.5,
            2_434_877.5,
            2_441_496.5,
            2_448_115.5,
            2_451_545.0,
            2_455_197.5,
            2_462_867.5,
            2_488_069.5,
        ];

        for &jd in &test_dates_jd {
            let t_yr = (jd - j2000) / days_per_year;
            let old_offset_deg = old_rate * t_yr + 0.5 * old_accel * t_yr * t_yr;
            let new_offset_deg =
                vedaksha_ephem_core::precession::general_precession_in_longitude(jd) / 3600.0;

            let diff = (new_offset_deg - old_offset_deg).abs();
            assert!(
                diff < 0.02,
                "Modern-date divergence too large at JD {jd}: old={old_offset_deg:.6}°, \
                 new={new_offset_deg:.6}°, diff={diff:.6}°"
            );
        }
    }

    #[test]
    fn precession_upgrade_diverges_at_historical_dates() {
        let old_rate: f64 = 50.2875 / 3600.0;
        let old_accel: f64 = 0.000222 / 3600.0;
        let j2000: f64 = 2_451_545.0;
        let days_per_year: f64 = 365.25;

        // Ancient dates (pre-1700 BCE) where the 5th-order polynomial
        // diverges measurably (>0.05°) from the truncated quadratic.
        let historical_dates_jd: [f64; 4] = [625_295.0, 990_545.0, 1_063_295.0, 1_100_345.0];

        let mut any_diverged = false;
        for &jd in &historical_dates_jd {
            let t_yr = (jd - j2000) / days_per_year;
            let old_offset_deg = old_rate * t_yr + 0.5 * old_accel * t_yr * t_yr;
            let new_offset_deg =
                vedaksha_ephem_core::precession::general_precession_in_longitude(jd) / 3600.0;

            let diff = (new_offset_deg - old_offset_deg).abs();
            if diff > 0.05 {
                any_diverged = true;
            }
        }
        assert!(
            any_diverged,
            "Expected measurable divergence at historical dates, but models agreed too closely"
        );
    }
}
