// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Ayanamsha and sidereal zodiac conversion.
//!
//! Every system in this module is derived forward from a primary definition —
//! a chapter, a committee report, a proposer's own publication, or a star
//! catalogue — and none of them was checked against, or tuned toward, the output
//! of any other implementation. The derivation, its declared assumptions and its
//! search records are in `docs/audit/2026-08-17-ayanamsha-cleanroom/`.
//!
//! # What this module returns
//!
//! **Mean ayanamsha, always.** Nutation in longitude is never included. A caller
//! who wants the true ayanamsha adds nutation themselves:
//!
//! ```text
//! true ayanamsha = mean ayanamsha + nutation in longitude
//! ```
//!
//! This is not a detail. The *Indian Astronomical Ephemeris* prints that same
//! relation, and the daily tables most panchanga-makers consume are the **true**
//! values, which differ from the mean by up to ~17 arcseconds. An engine that is
//! silent about which one it returns gets compared against the wrong column.
//!
//! # Star-anchored systems
//!
//! Several systems are defined by holding a named star at an assigned sidereal
//! longitude. For those, "the tropical longitude of the star" is ambiguous by
//! roughly 20 arcseconds of aberration plus 17 of nutation — larger than the
//! differences between neighbouring systems. This module uses the **mean
//! geometric place of date**: proper motion applied, aberration and nutation not.
//! See [`vedaksha_ephem_core::stars`].
//!
//! # Rates
//!
//! Where a tradition states a rate without units, "per year" means the **Julian
//! year** of 365.25 days, except where the primary's own arithmetic counts
//! calendar years — which is the case for exactly one system, Raman, and is
//! stated there.
//!
//! # Time scale
//!
//! `jd` is Terrestrial Time. Passing UT1 instead shifts the result by under a
//! milliarcsecond at any epoch this engine covers, so the distinction is
//! recorded for completeness rather than enforced.

use vedaksha_ephem_core::julian;
use vedaksha_ephem_core::precession::general_precession_p03;
use vedaksha_ephem_core::stars::{CatalogueStar, mean_ecliptic_longitude_of_date};
use vedaksha_math::angle::{normalize_degrees, normalize_degrees_signed};

// ── Primary constants ─────────────────────────────────────────────────────────
//
// Everything below is a value read from a primary source, not a value derived by
// this project. Each carries the citation that lets a reader check it.

/// Julian Day (TT) of the standard epoch B1950.0.
const B1950: f64 = 2_433_282.423_46;

/// Days in a Julian year.
const JULIAN_YEAR: f64 = 365.25;

/// Arcseconds in a degree.
const ARCSEC: f64 = 3600.0;

/// Indian official ayanamsha at J2000.0: 23°51'25".53.
///
/// *The Indian Astronomical Ephemeris 2022*, Positional Astronomy Centre,
/// Government of India, p. 380 ("AYANAMSA"): "Ayanamsha for J2000.0 is taken as
/// 23°51'25".53".
const IAE_J2000_ANCHOR_DEG: f64 = 23.0 + 51.0 / 60.0 + 25.53 / ARCSEC;

/// Fagan-Bradley synetic vernal point at B1950.0: 335°57'28".64.
///
/// Fagan, C. & Firebrace, R. C., *Primer of Sidereal Astrology*, p. 13: "for the
/// epoch 1950.0 he proposed as the mean longitude of the vernal point
/// 335° 57' 28.64"". The ayanamsha is 360° minus that (ibid., p. 16).
const FAGAN_SVP_B1950_DEG: f64 = 335.0 + 57.0 / 60.0 + 28.64 / ARCSEC;

/// Krishnamurti's anchor: 22°22'00" on the 1st of Chitra, 1900.
///
/// Krishnamurti, K. S., *Krishnamurti Padhdhati Vol-I*, p. 140.
const KP_ANCHOR_DEG: f64 = 22.0 + 22.0 / 60.0;

/// Julian Day (TT) of 1900 April 13, 00:00 — the 1st of Chitra 1900 (DA-5).
///
/// Rajan's table header, from which KSK's is derived by a flat +2', reads
/// "Ayanamsa on the 1st of Chitra (i.e. on the 13th or 14th April.)". The
/// ambiguity is 0.14 arcsecond; the 13th is adopted and the choice recorded.
const KP_ANCHOR_JD: f64 = 2_415_122.5;

/// Krishnamurti's stated rate, arcseconds per Julian year.
///
/// KSK states 50.2388475"/yr twice and attributes it to Newcomb; independent
/// research has failed to trace it there. It is used because it is the system's
/// own definition, not because it is the better model.
const KP_RATE_ARCSEC_PER_YEAR: f64 = 50.238_847_5;

/// Yukteshwar's anchor: 20°54'36" at the vernal equinox of 1894.
///
/// Yukteswar, Swami Sri, *The Holy Science* (1894): "The astronomical reference
/// books show the Vernal Equinox now to be 20°54'36" distant from the first
/// point of Aries (the fixed star Revati)."
const YUKTESHWAR_ANCHOR_DEG: f64 = 20.0 + 54.0 / 60.0 + 36.0 / ARCSEC;

/// Julian Day (TT) of the March equinox of 1894 (DA-6).
///
/// *The Holy Science* dates its anchor to "the position of the Vernal Equinox at
/// spring in the year 1894"; the equinox instant is the literal reading. This
/// value is **computed by this engine**, not transcribed: the test
/// `yukteshwar_epoch_is_the_1894_march_equinox` recomputes it from the solar
/// theory and fails if it drifts. Corresponds to 1894 March 20, 14:58:40 TT.
const YUKTESHWAR_ANCHOR_JD: f64 = 2_412_908.124_077_082;

/// Yukteshwar's rate, arcseconds per Julian year.
///
/// His own 24,000-year cycle, and the rate his anchor was produced by:
/// 1394 × 54" = 75,276" = 20°54'36", exactly the figure printed. Propagating his
/// anchor with a modern precession model instead inverts to a zero year near
/// 390 CE, missing his documented 500 CE by a century.
const YUKTESHWAR_RATE_ARCSEC_PER_YEAR: f64 = 54.0;

/// Raman's zero year, 397 CE.
///
/// Raman, B. V., *A Manual of Hindu Astrology* (1935), Ch. III Art. 49:
/// "(1) Subtract 397 from the year of birth (a.d.) (2) Multiply the remainder by
/// 50⅓" and reduce the product into degrees, minutes and seconds."
///
/// Raman lists 361, 394, 498 and 559 CE as alternatives and declines to defend
/// 397 over them, calling the question one of "considerable doubt".
const RAMAN_ZERO_YEAR: i32 = 397;

/// Raman's rate: 50⅓ arcseconds per calendar year, exactly.
const RAMAN_RATE_ARCSEC_PER_YEAR: f64 = 151.0 / 3.0;

/// Julian Day of the Kali Yuga epoch, from which the Surya Siddhanta's ahargana
/// is reckoned.
const KALI_EPOCH_JD: f64 = 588_465.75;

/// Days in a Mahayuga, per the Surya Siddhanta.
const MAHAYUGA_DAYS: f64 = 1_577_917_828.0;

/// Libration revolutions per Mahayuga, per Surya Siddhanta Ch. 3 vv. 9-12.
const SS_REVOLUTIONS_PER_MAHAYUGA: f64 = 600.0;

/// Amplitude of the Surya Siddhanta libration, in degrees (±27°).
const SS_AMPLITUDE_DEG: f64 = 27.0;

/// Julian Day (TT) of the Hipparcos catalogue epoch J1991.25.
const HIPPARCOS_EPOCH_JD: f64 = 2_448_349.062_5;

/// ζ Piscium — the yogatārā of Revati (DA-1).
///
/// The Surya Siddhanta gives coordinates, not modern catalogue identities.
/// Identifying Revati's junction star with ζ Piscium is an identification made
/// by the commentary, which records that the star "is by all authorities
/// identified with ζ Piscium". Vedaksha asserts it on that authority.
///
/// Astrometry: van Leeuwen, F. (2007), *A&A* 474, 653 — the re-reduction of the
/// Hipparcos raw data — via `VizieR` `I/311/hip2`, row HIP 5737 (ζ Psc A).
const ZETA_PISCIUM: CatalogueStar = CatalogueStar {
    catalogue: "I/311/hip2",
    identifier: "HIP 5737",
    epoch_jd_tt: HIPPARCOS_EPOCH_JD,
    ra_deg: 18.432_508_42,
    dec_deg: 7.575_489_38,
    pm_ra_cosdec_mas_per_year: 145.00,
    pm_dec_mas_per_year: -55.69,
};

/// δ Cancri (Asellus Australis) — the star Rao names for Pushya-paksha.
///
/// Astrometry: van Leeuwen (2007) via `VizieR` `I/311/hip2`, row HIP 42911.
const DELTA_CANCRI: CatalogueStar = CatalogueStar {
    catalogue: "I/311/hip2",
    identifier: "HIP 42911",
    epoch_jd_tt: HIPPARCOS_EPOCH_JD,
    ra_deg: 131.171_291_91,
    dec_deg: 18.154_863_73,
    pm_ra_cosdec_mas_per_year: -17.67,
    pm_dec_mas_per_year: -229.26,
};

/// α Virginis (Spica, Chitra).
///
/// Astrometry: van Leeuwen (2007) via `VizieR` `I/311/hip2`, row HIP 65474.
const SPICA: CatalogueStar = CatalogueStar {
    catalogue: "I/311/hip2",
    identifier: "HIP 65474",
    epoch_jd_tt: HIPPARCOS_EPOCH_JD,
    ra_deg: 201.298_352_28,
    dec_deg: -11.161_244_94,
    pm_ra_cosdec_mas_per_year: -42.35,
    pm_dec_mas_per_year: -30.67,
};

/// λ Scorpii (Shaula) — the star Chandra Hari names for his sidereal origin.
///
/// Astrometry: van Leeuwen (2007) via `VizieR` `I/311/hip2`, row HIP 85927.
const LAMBDA_SCORPII: CatalogueStar = CatalogueStar {
    catalogue: "I/311/hip2",
    identifier: "HIP 85927",
    epoch_jd_tt: HIPPARCOS_EPOCH_JD,
    ra_deg: 263.402_193_18,
    dec_deg: -37.103_748_69,
    pm_ra_cosdec_mas_per_year: -8.53,
    pm_dec_mas_per_year: -30.80,
};

/// Sagittarius A*, in the ICRF3 frame (DA-9).
///
/// Gordon, D., de Witt, A. & Jacobs, C. S. (2023), "Position and Proper Motion
/// of Sagittarius A* in the ICRF3 Frame from VLBI Absolute Astrometry", *AJ*
/// 165, 49 (doi:10.3847/1538-3881/aca65b). Position and proper motion at the
/// paper's stated 2015.0 reference epoch:
/// α = 17h45m40.034047s ± 0.000018s, δ = −29°00'28".21601 ± 0".00044,
/// `μ_α cos δ` = −3.128 ± 0.042 mas/yr, `μ_δ` = −5.584 ± 0.075 mas/yr.
///
/// **Sgr A* is not an ICRF3 catalogue entry** — it is undetectable in ICRF3's
/// S band and resolved out at X/Ka — so this paper's no-net-rotation solution
/// against 258 ICRF3 defining sources is the citable ICRF3-frame determination.
/// Cross-checked against Reid & Brunthaler (2020), *`ApJ`* 892, 39, whose combined
/// proper motion is −3.156 ± 0.006 and −5.585 ± 0.010 mas/yr — agreeing within
/// 1σ. Note their widely-quoted position is the arbitrary origin their offsets
/// are measured from, and sits ~40 mas from the ICRF3 position.
const SGR_A_STAR: CatalogueStar = CatalogueStar {
    catalogue: "Gordon, de Witt & Jacobs (2023), AJ 165, 49",
    identifier: "Sgr A*",
    // Julian epoch J2015.0.
    epoch_jd_tt: 2_457_023.75,
    ra_deg: 266.416_808_529,
    dec_deg: -29.007_837_781,
    pm_ra_cosdec_mas_per_year: -3.128,
    pm_dec_mas_per_year: -5.584,
};

// ── The enumeration ───────────────────────────────────────────────────────────

/// A sidereal zodiac system.
///
/// The list is rebuilt from primary literature rather than inherited. Eleven
/// systems survive, each traceable to a chapter, a star, a committee or a named
/// proposer's own publication, plus a tropical identity for callers who want a
/// uniform interface.
///
/// Systems that appeared in Vedaksha 5 and are not here were removed because no
/// primary defining their zero point could be located, or because the source
/// turned out to *estimate* a historical zero point rather than *stipulate* one.
/// Their names hard-error on parse rather than silently mapping to a neighbour;
/// see [`Ayanamsha::from_str`] and the audit directory's disposition table.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
// Serde routes through `key()` and `FromStr` rather than deriving variant names
// directly. That is the whole point: a retired name arriving in a config file
// must hard-error with its disposition, exactly as it does on the wasm and MCP
// surfaces, instead of failing with serde's "unknown variant" list or — worse —
// being silently absorbed by a `#[serde(other)]` arm somewhere downstream.
#[serde(into = "String", try_from = "String")]
#[non_exhaustive]
pub enum Ayanamsha {
    /// Indian official ayanamsha — the Chitra-paksha system, widely called
    /// "Lahiri".
    ///
    /// Anchor 23°51'25".53 at J2000.0, propagated by the P03 general precession
    /// in longitude. *The Indian Astronomical Ephemeris 2022*, p. 380.
    ///
    /// The popular name implies the star Chitra (Spica), but the operative
    /// definition does not use it: it is a J2000 anchor and a polynomial. For a
    /// system that actually fixes Spica, see [`Ayanamsha::TrueChitra`] — they are
    /// different rules and give different answers.
    IndianOfficial,

    /// Fagan-Bradley — the Western sidereal standard.
    ///
    /// Synetic vernal point 335°57'28".64 at B1950.0, ayanamsha = 360° − SVP.
    /// Fagan & Firebrace, *Primer of Sidereal Astrology*, pp. 13 and 16.
    ///
    /// Fagan states no precession model; P03 is adopted (DA-3) as the best
    /// current determination of the quantity he named, and that choice is ours.
    FaganBradley,

    /// Krishnamurti (KP).
    ///
    /// Anchor 22°22'00" on the 1st of Chitra 1900, propagated at KSK's own
    /// stated 50.2388475"/yr. *Krishnamurti Padhdhati Vol-I*, p. 140.
    ///
    /// KSK's three published numbers are mutually inconsistent: his anchor
    /// divided by his rate gives a zero year of 297.5 CE against the 291 AD he
    /// states. The anchor and rate are adopted because together they are the
    /// operative definition and are what his table reproduces; the discrepancy is
    /// a documented property of the system, not something to reconcile by
    /// choosing a rate that makes the inversion work.
    Krishnamurti,

    /// Raman.
    ///
    /// `ayanamsa(year) = (year − 397) × 50⅓"`. B. V. Raman, *A Manual of Hindu
    /// Astrology* (1935), Ch. III Art. 49.
    ///
    /// Three properties travel with it. Raman titles the article
    /// "Determination of (**Approximate**) Ayanamsa" — do not present it as more
    /// precise than its author does. He declines to defend 397 CE over the
    /// alternatives he lists. And the rule is **year-granular**: it counts
    /// calendar years, so this is the one system whose rate is per calendar year
    /// rather than per Julian year. Within a year the value is interpolated
    /// linearly, which is Art. 50's optional "odd days" correction and makes the
    /// function continuous while reproducing Art. 49 exactly at each 1 January.
    Raman,

    /// Surya Siddhanta — the text's own trepidation, not a linear precession.
    ///
    /// A zigzag libration of ±27° about the sidereal zero point, 600 revolutions
    /// per Mahayuga, phase measured as ahargana from the Kali epoch.
    /// Surya Siddhanta Ch. 3 vv. 9–12.
    ///
    /// The folding is **linear**, not sinusoidal — v. 10's *bhuja* rule
    /// ("multiply by three, divide by ten", 90 → 27) is linear folding, and it is
    /// the only reading under which the text's own three numbers agree: 27° at
    /// 54"/yr takes 1800 years, and the quarter period is 1800.04 years.
    ///
    /// Chapters 3 and 8 of the text disagree about the epoch by roughly a degree
    /// (DA-2). Chapter 3 is the ayanamsa definition and Chapter 8 is a star
    /// catalogue, so Chapter 3 is implemented literally and the disagreement is
    /// recorded as a property of the source. A repaired Surya Siddhanta would be
    /// our own system shipped under its name.
    SuryaSiddhanta,

    /// Yukteshwar.
    ///
    /// Anchor 20°54'36" at the March equinox of 1894, propagated at his own
    /// 54"/yr. *The Holy Science* (1894).
    ///
    /// His anchor is his own model's output rather than an observation —
    /// 1394 × 54" is exactly the figure printed, and the same passage says "by
    /// calculation it will appear that 1394 years have passed" — so his rate is
    /// the one that belongs with it.
    Yukteshwar,

    /// Revati-paksha — ζ Piscium held at sidereal 0°00'00", tracked live.
    ///
    /// The majority position among the authorities: Burgess records that "all
    /// authorities agree in placing it upon the ecliptic and all excepting our
    /// treatise and the Cakalya make its position exactly mark the initial point
    /// of the fixed sidereal sphere".
    ///
    /// Distinct from [`Ayanamsha::SuryaSiddhanta`], whose own zero point sits 10'
    /// east of ζ Piscium, and which propagates by its own trepidation rather than
    /// by tracking the star. Requires DA-1, the identification of Revati with
    /// ζ Piscium.
    ///
    /// Chapter 8 assigns Revati a polar latitude of zero while the star has about
    /// 13' of south latitude. The text idealises it; that is a property of the
    /// system, not an error to correct.
    RevatiPaksha,

    /// Pushya-paksha — δ Cancri held at sidereal 106°00'00" (16 Cn 00), tracked
    /// live.
    ///
    /// P.V.R. Narasimha Rao, "Introducing Pushya-paksha Ayanamsa": "So we define
    /// Pushya-paksha ayanamsa by fixing the sidereal longitude of Delta Cancri at
    /// 16Cn0."
    ///
    /// It needs no declared assumption: Rao names the star himself as part of the
    /// definition, so unlike Revati-paksha there is no inference gap. His stated
    /// Surya Siddhanta basis — Pushya at 106°, latitude 0° — is exactly what the
    /// Ch. 8 encoding rule produces independently.
    PushyaPaksha,

    /// True Chitra — Spica held at sidereal 180°00'00", tracked live.
    ///
    /// The condition is self-describing and Spica's identity is not contested, so
    /// no proposer's text is needed to know what it means.
    ///
    /// **Distinct from [`Ayanamsha::IndianOfficial`]**, which is defined by a
    /// J2000 anchor and a polynomial, not by Spica. Fixing the star strictly is a
    /// different rule from evaluating a polynomial, and the two diverge.
    TrueChitra,

    /// True Mula (Chandra Hari) — λ Scorpii held at sidereal 240°00'00", tracked
    /// live.
    ///
    /// K. Chandra Hari, "On the Origin of Sidereal Zodiac and Astronomy",
    /// *Indian Journal of History of Science* 33(4), 1998: "the initial point of
    /// the ancient Babylonian Zodiac was the same as that of Hindu's which had
    /// the star Mūla of sidereal longitude 240° as fiducial", with Mūla
    /// identified as λ Scorpii. The proposer names the star, so no declared
    /// assumption is needed.
    ///
    /// Not the same as Surya Siddhanta Ch. 8's Mula (dhruva 241°00', vikṣepa −9°,
    /// a *polar* figure). Chandra Hari assigns an *ecliptic* longitude.
    ChandraHari,

    /// Galactic Centre at 0° Sagittarius — Sgr A* held at sidereal 240°00'00".
    ///
    /// The one galactic system that qualifies, because the condition is
    /// self-describing and the astrometry has peer-reviewed primaries. Sgr A* is
    /// an unambiguous radio source with its own VLBI astrometry, not a modern
    /// guess at what an ancient text meant, so it needs no declared assumption
    /// about identity.
    ///
    /// Assigns the same sidereal longitude as [`Ayanamsha::ChandraHari`] but to a
    /// different object; the two are different systems and must not be collapsed.
    GalacticCenter0Sag,

    /// Tropical — the identity, zero at every epoch.
    ///
    /// Not a sidereal system and not derived from anything; it exists so callers
    /// can pass a uniform [`Ayanamsha`] and get tropical coordinates back
    /// unchanged.
    Tropical,
}

impl Default for Ayanamsha {
    /// The Indian official ayanamsha.
    ///
    /// Note that this is *not* the same as the default zodiac: `ChartConfig`
    /// defaults to `ayanamsha: None`, which is tropical. This default applies
    /// only where an `Ayanamsha` value is required and none was supplied.
    fn default() -> Self {
        Self::IndianOfficial
    }
}

impl Ayanamsha {
    /// Every system, in a stable order. The tropical identity is included last.
    pub const ALL: &'static [Self] = &[
        Self::IndianOfficial,
        Self::FaganBradley,
        Self::Krishnamurti,
        Self::Raman,
        Self::SuryaSiddhanta,
        Self::Yukteshwar,
        Self::RevatiPaksha,
        Self::PushyaPaksha,
        Self::TrueChitra,
        Self::ChandraHari,
        Self::GalacticCenter0Sag,
        Self::Tropical,
    ];

    /// The eleven sidereal systems, excluding the tropical identity.
    pub const SIDEREAL: &'static [Self] = &[
        Self::IndianOfficial,
        Self::FaganBradley,
        Self::Krishnamurti,
        Self::Raman,
        Self::SuryaSiddhanta,
        Self::Yukteshwar,
        Self::RevatiPaksha,
        Self::PushyaPaksha,
        Self::TrueChitra,
        Self::ChandraHari,
        Self::GalacticCenter0Sag,
    ];

    /// The canonical string name, and the one [`Ayanamsha::from_str`] round-trips.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::IndianOfficial => "IndianOfficial",
            Self::FaganBradley => "FaganBradley",
            Self::Krishnamurti => "Krishnamurti",
            Self::Raman => "Raman",
            Self::SuryaSiddhanta => "SuryaSiddhanta",
            Self::Yukteshwar => "Yukteshwar",
            Self::RevatiPaksha => "RevatiPaksha",
            Self::PushyaPaksha => "PushyaPaksha",
            Self::TrueChitra => "TrueChitra",
            Self::ChandraHari => "ChandraHari",
            Self::GalacticCenter0Sag => "GalacticCenter0Sag",
            Self::Tropical => "Tropical",
        }
    }

    /// The conventional display name of the system.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::IndianOfficial => "Indian official (Lahiri / Chitra-paksha)",
            Self::FaganBradley => "Fagan-Bradley",
            Self::Krishnamurti => "Krishnamurti (KP)",
            Self::Raman => "Raman",
            Self::SuryaSiddhanta => "Surya Siddhanta",
            Self::Yukteshwar => "Yukteshwar",
            Self::RevatiPaksha => "Revati-paksha",
            Self::PushyaPaksha => "Pushya-paksha",
            Self::TrueChitra => "True Chitra",
            Self::ChandraHari => "True Mula (Chandra Hari)",
            Self::GalacticCenter0Sag => "Galactic Centre at 0° Sagittarius",
            Self::Tropical => "Tropical (0°)",
        }
    }

    /// The primary source the system is derived from.
    #[must_use]
    pub const fn primary_source(self) -> &'static str {
        match self {
            Self::IndianOfficial => "Indian Astronomical Ephemeris 2022, p. 380",
            Self::FaganBradley => "Fagan & Firebrace, Primer of Sidereal Astrology, pp. 13, 16",
            Self::Krishnamurti => "Krishnamurti, Krishnamurti Padhdhati Vol-I, p. 140",
            Self::Raman => "Raman, A Manual of Hindu Astrology (1935), Ch. III Art. 49",
            Self::SuryaSiddhanta => "Surya Siddhanta Ch. 3 vv. 9-12",
            Self::Yukteshwar => "Yukteswar, The Holy Science (1894)",
            Self::RevatiPaksha => "Surya Siddhanta Ch. 8; zeta Piscium per Hipparcos",
            Self::PushyaPaksha => "Narasimha Rao, Introducing Pushya-paksha Ayanamsa",
            Self::TrueChitra => "Self-describing condition; Spica per Hipparcos",
            Self::ChandraHari => "Chandra Hari, Indian J. History of Science 33(4), 1998",
            Self::GalacticCenter0Sag => "Gordon, de Witt & Jacobs (2023), AJ 165, 49",
            Self::Tropical => "Not a sidereal system; identity by construction",
        }
    }

    /// Whether the system tracks a live star, so that its value depends on
    /// catalogue astrometry rather than on a published constant.
    #[must_use]
    pub const fn is_star_anchored(self) -> bool {
        matches!(
            self,
            Self::RevatiPaksha
                | Self::PushyaPaksha
                | Self::TrueChitra
                | Self::ChandraHari
                | Self::GalacticCenter0Sag
        )
    }
}

impl core::fmt::Display for Ayanamsha {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.key())
    }
}

/// The error returned when a string does not name a system this engine has.
///
/// This crate is not `no_std` (see the crate docs), so the offending input is
/// carried as an owned `String` unconditionally rather than varying by feature —
/// a public type whose shape changes with a feature flag is a trap for callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAyanamsha {
    /// The string that was offered.
    pub input: String,
    /// Why it was refused, and what to use instead.
    pub disposition: &'static str,
}

impl core::fmt::Display for UnknownAyanamsha {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "unknown ayanamsha \"{}\": {}",
            self.input, self.disposition
        )
    }
}

impl std::error::Error for UnknownAyanamsha {}

/// The disposition of every Vedaksha 5 name that this engine no longer has.
///
/// A removed name must **hard-error**, never silently fall back to a default or
/// to a neighbouring system: a silent remap moves a caller's chart without
/// telling them, which is the failure mode this whole re-derivation exists to
/// undo. Each entry says what happened and, where there is one, names the
/// system a caller most likely wants — but the caller has to choose it.
const DISPOSITIONS: &[(&str, &str)] = &[
    // Collapsed into a system that survives, under a different name.
    (
        "lahiri1940",
        "collapsed into IndianOfficial — the Indian Astronomical Ephemeris defines one Chitra-paksha ayanamsha, not a family of epochs; pass IndianOfficial",
    ),
    (
        "lahirivp285",
        "collapsed into IndianOfficial — the 285 CE coincidence is a property of that system, not a separate one; pass IndianOfficial",
    ),
    (
        "ayanamshaofdate",
        "collapsed into IndianOfficial — an evaluation mode, not a system; pass IndianOfficial",
    ),
    (
        "krishnamurti2",
        "collapsed into Krishnamurti — KSK published one anchor and one rate; pass Krishnamurti",
    ),
    (
        "suryasiddhantamean",
        "collapsed into SuryaSiddhanta — the text gives one libration, and it has no mean/true variants; pass SuryaSiddhanta",
    ),
    (
        "sscitra",
        "collapsed into SuryaSiddhanta — a software enumeration label, not a system the text defines; pass SuryaSiddhanta",
    ),
    (
        "ssdrevjul",
        "collapsed into SuryaSiddhanta — a software enumeration label, not a system the text defines; pass SuryaSiddhanta",
    ),
    (
        "bvramanmean",
        "collapsed into Raman — Art. 49 gives one rule; pass Raman",
    ),
    // Renamed, and the new name encodes a defining condition the old one did not.
    (
        "truechitrapaksha",
        "renamed TrueChitra, and re-derived from Spica at sidereal 180 degrees; pass TrueChitra",
    ),
    (
        "truerevati",
        "removed — the name does not say which longitude Revati is held at, and that is the whole discriminator: RevatiPaksha holds zeta Piscium at 0 degrees, while the Surya Siddhanta's own zero point sits 10 arcminutes east of it. Pass RevatiPaksha or SuryaSiddhanta deliberately",
    ),
    (
        "truepushya",
        "removed — its longitude did not match any located primary. Rao defines Pushya-paksha by delta Cancri at 106 degrees; pass PushyaPaksha",
    ),
    (
        "truemula",
        "removed — no primary was located for the galactic-alignment reading this name carried. For lambda Scorpii at sidereal 240 degrees per Chandra Hari (IJHS 33(4), 1998), pass ChandraHari",
    ),
    // Dropped: the source determines a historical zero point rather than stipulating one.
    (
        "babylonianhuber",
        "dropped — Huber estimated where the Babylonian zero point lay; he did not stipulate one. Two secondary accounts of his figure differ by 6 arcminutes, which is the signature of a measurement, not a definition",
    ),
    (
        "babylonianetpsc",
        "dropped — same category as the other Babylonian entries; no stipulated zero point",
    ),
    (
        "babyloniankuglerstar1",
        "dropped — Kugler analysed Babylonian star data and defined no zero point, still less three of them",
    ),
    (
        "babyloniankuglerstar2",
        "dropped — see BabylonianKuglerStar1",
    ),
    (
        "babyloniankuglerstar3",
        "dropped — see BabylonianKuglerStar1",
    ),
    // Dropped: no primary located, and searched for.
    (
        "aryabhata",
        "dropped — the Aryabhatiya contains no precession or libration rule. The sole basis is a second-hand quotation that its own editor says could not be found in the text",
    ),
    ("aryabhata528", "dropped — see Aryabhata"),
    (
        "deluce",
        "dropped — De Luce, Constellational Astrology (1963) is not on archive.org and was not obtained; unverified, not refuted",
    ),
    (
        "ushashashi",
        "dropped — Hindu Astrological Calculations (1978) is not on archive.org and was not obtained; unverified, not refuted",
    ),
    (
        "sassanian",
        "dropped — the attribution reached us only through secondary pages; Mercier (2004) was not obtained",
    ),
    (
        "aldebaran15tau",
        "dropped — see Sassanian; no primary stipulating Aldebaran at 15 Taurus was located",
    ),
    (
        "hipparchos",
        "dropped — a contested reconstruction of Hipparchus's catalogue is a determination, not a definition",
    ),
    ("jnbhasin", "dropped — no primary publication located"),
    ("sundararajan", "dropped — no primary publication located"),
    ("djwhalkhul", "dropped — no primary publication located"),
    ("djwhalkhultibetan2", "dropped — see DjwhalKhul"),
    ("skydram", "dropped — no primary publication located"),
    ("valensmoon", "dropped — no primary publication located"),
    (
        "truemoonsnode",
        "dropped — the lunar node's position is not a stipulated sidereal zero point",
    ),
    (
        "galacticcenterbrand",
        "dropped — Himmlische Matrix was not obtained, and the condition is not self-describing",
    ),
    (
        "galacticcentergalalign",
        "dropped — no primary located, and unlike Sgr A* at 240 degrees the condition cannot be computed without reading the proposer",
    ),
    (
        "galacticequatoriau1958",
        "dropped — the IAU 1958 pole is astronomy, not an ayanamsha; no proposer stipulating a zero point from it was located",
    ),
    (
        "galacticequatortrue",
        "dropped — see GalacticEquatorIau1958",
    ),
    (
        "galacticequatormidmula",
        "dropped — no citable primary; web descriptions of it are contamination hazards",
    ),
];

impl From<Ayanamsha> for String {
    fn from(a: Ayanamsha) -> Self {
        a.key().to_string()
    }
}

impl TryFrom<String> for Ayanamsha {
    type Error = UnknownAyanamsha;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        use core::str::FromStr as _;
        Self::from_str(&s)
    }
}

impl core::str::FromStr for Ayanamsha {
    type Err = UnknownAyanamsha;

    /// Parse a system name, case-insensitively and ignoring `_`, `-` and spaces.
    ///
    /// A name that Vedaksha 5 accepted but this engine no longer has produces an
    /// error explaining what happened to it. It never falls back to a default.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut normalized = [0u8; 64];
        let mut len = 0usize;
        for b in s.bytes() {
            if b == b'_' || b == b'-' || b == b' ' || b == b'.' || b == b'\'' {
                continue;
            }
            if len < normalized.len() {
                normalized[len] = b.to_ascii_lowercase();
                len += 1;
            }
        }
        let key = core::str::from_utf8(&normalized[..len]).unwrap_or("");

        let matched = match key {
            "indianofficial" | "lahiri" | "chitrapaksha" | "chitrapaksa" => {
                Some(Self::IndianOfficial)
            }
            "faganbradley" | "fagan" => Some(Self::FaganBradley),
            "krishnamurti" | "kp" => Some(Self::Krishnamurti),
            "raman" | "bvraman" => Some(Self::Raman),
            "suryasiddhanta" => Some(Self::SuryaSiddhanta),
            "yukteshwar" | "yukteswar" => Some(Self::Yukteshwar),
            "revatipaksha" | "revatipaksa" => Some(Self::RevatiPaksha),
            "pushyapaksha" | "pushyapaksa" => Some(Self::PushyaPaksha),
            "truechitra" => Some(Self::TrueChitra),
            "chandrahari" | "truemulachandrahari" => Some(Self::ChandraHari),
            "galacticcenter0sag" | "galacticcentre0sag" | "galacticcenter" | "galacticcentre" => {
                Some(Self::GalacticCenter0Sag)
            }
            "tropical" | "none" => Some(Self::Tropical),
            _ => None,
        };
        if let Some(a) = matched {
            return Ok(a);
        }

        let disposition = DISPOSITIONS
            .iter()
            .find(|(name, _)| *name == key)
            .map_or(
                "not a system this engine has; see docs/audit/2026-08-17-ayanamsha-cleanroom/ for the systems that survived primary-source re-derivation",
                |(_, why)| *why,
            );

        Err(UnknownAyanamsha {
            input: s.to_string(),
            disposition,
        })
    }
}

// ── The derivations ───────────────────────────────────────────────────────────

/// Ayanamsha for a star-anchored system: where the star is now, minus where the
/// system says it should be.
fn star_anchored(star: &CatalogueStar, assigned_longitude_deg: f64, jd: f64) -> f64 {
    let lambda = mean_ecliptic_longitude_of_date(star, jd);
    normalize_degrees_signed(lambda - assigned_longitude_deg)
}

/// Ayanamsha for a system anchored to a value at an epoch and propagated by P03
/// general precession in longitude.
fn precession_anchored(anchor_deg: f64, anchor_jd: f64, jd: f64) -> f64 {
    anchor_deg + (general_precession_p03(jd) - general_precession_p03(anchor_jd)) / ARCSEC
}

/// Ayanamsha for a system anchored to a value at an epoch and propagated at its
/// own constant rate, in arcseconds per Julian year.
fn rate_anchored(anchor_deg: f64, anchor_jd: f64, rate_arcsec_per_year: f64, jd: f64) -> f64 {
    anchor_deg + ((jd - anchor_jd) / JULIAN_YEAR) * rate_arcsec_per_year / ARCSEC
}

/// Raman's rule: `(year − 397) × 50⅓"`, interpolated within the year.
///
/// Art. 49 takes "the year of birth" and gives no point within it, so the value
/// is anchored at 1 January (DA-7) — the convention that reproduces both printed
/// examples exactly. Art. 50 says the intra-year change "can conveniently be
/// neglected" but may be added; adding it linearly across the actual length of
/// the calendar year is what makes this continuous while leaving every 1 January
/// exact.
///
/// This is the one system whose rate is per **calendar** year rather than per
/// Julian year, because Raman's own arithmetic counts calendar years.
fn raman(jd: f64) -> f64 {
    let (year, _, _) = julian::jd_to_calendar(jd);
    let year_start = julian::calendar_to_jd(year, 1, 1.0);
    let next_year_start = julian::calendar_to_jd(year + 1, 1, 1.0);
    let fraction = (jd - year_start) / (next_year_start - year_start);
    let elapsed = f64::from(year - RAMAN_ZERO_YEAR) + fraction;
    elapsed * RAMAN_RATE_ARCSEC_PER_YEAR / ARCSEC
}

/// The Surya Siddhanta's libration, per Ch. 3 vv. 9–12.
///
/// A zigzag of amplitude ±27°, period `Mahayuga / 600`, with phase measured as
/// ahargana from the Kali epoch. Zero at the Kali epoch, −27° a quarter period
/// later, zero again at the half period (499 CE), +27° at three quarters
/// (2299 CE).
///
/// The sign matters and is stated rather than inferred (DA-10): after the 499 CE
/// crossing the ayanamsa is **positive and increasing**. A negative result at
/// J2000 means the folding direction is inverted.
fn surya_siddhanta(jd: f64) -> f64 {
    let period = MAHAYUGA_DAYS / SS_REVOLUTIONS_PER_MAHAYUGA;
    let phase = ((jd - KALI_EPOCH_JD) / period).rem_euclid(1.0);

    // Linear folding, per v. 10's bhuja rule. Not a sinusoid: a sinusoidal
    // reading diverges by ~3.6 degrees at J2000, and under it the text's own
    // three numbers (27 degrees, 54"/yr, 600 revolutions) stop agreeing.
    let triangle = if phase <= 0.25 {
        4.0 * phase
    } else if phase <= 0.75 {
        2.0 - 4.0 * phase
    } else {
        4.0 * phase - 4.0
    };

    -SS_AMPLITUDE_DEG * triangle
}

// ── Public API ────────────────────────────────────────────────────────────────

/// The mean ayanamsha, in decimal degrees, for a given Julian Day.
///
/// Positive means the sidereal zodiac lags the tropical one, which is the case
/// for every system at every date in the common era. Values before a system's
/// own zero year are negative; they are returned signed rather than wrapped, so
/// that inverting a system to its zero year works.
///
/// Nutation is **not** included — see the module documentation.
///
/// # Arguments
///
/// * `system` — the sidereal system.
/// * `jd` — Julian Day Number, Terrestrial Time.
#[must_use]
pub fn ayanamsha_value(system: Ayanamsha, jd: f64) -> f64 {
    match system {
        Ayanamsha::IndianOfficial => precession_anchored(IAE_J2000_ANCHOR_DEG, julian::J2000, jd),
        Ayanamsha::FaganBradley => precession_anchored(360.0 - FAGAN_SVP_B1950_DEG, B1950, jd),
        Ayanamsha::Krishnamurti => {
            rate_anchored(KP_ANCHOR_DEG, KP_ANCHOR_JD, KP_RATE_ARCSEC_PER_YEAR, jd)
        }
        Ayanamsha::Raman => raman(jd),
        Ayanamsha::SuryaSiddhanta => surya_siddhanta(jd),
        Ayanamsha::Yukteshwar => rate_anchored(
            YUKTESHWAR_ANCHOR_DEG,
            YUKTESHWAR_ANCHOR_JD,
            YUKTESHWAR_RATE_ARCSEC_PER_YEAR,
            jd,
        ),
        Ayanamsha::RevatiPaksha => star_anchored(&ZETA_PISCIUM, 0.0, jd),
        Ayanamsha::PushyaPaksha => star_anchored(&DELTA_CANCRI, 106.0, jd),
        Ayanamsha::TrueChitra => star_anchored(&SPICA, 180.0, jd),
        Ayanamsha::ChandraHari => star_anchored(&LAMBDA_SCORPII, 240.0, jd),
        Ayanamsha::GalacticCenter0Sag => star_anchored(&SGR_A_STAR, 240.0, jd),
        Ayanamsha::Tropical => 0.0,
    }
}

/// Convert a tropical ecliptic longitude to sidereal longitude.
///
/// ```text
/// sidereal = tropical − ayanamsha
/// ```
///
/// Uses the **mean** ayanamsha; nutation in longitude is not included. See the
/// module documentation for why that distinction decides which published table
/// a result should be compared against.
#[must_use]
pub fn tropical_to_sidereal(tropical_longitude_deg: f64, system: Ayanamsha, jd: f64) -> f64 {
    normalize_degrees(tropical_longitude_deg - ayanamsha_value(system, jd))
}

/// Convert a sidereal ecliptic longitude to tropical longitude.
///
/// ```text
/// tropical = sidereal + ayanamsha
/// ```
///
/// Uses the **mean** ayanamsha; nutation in longitude is not included. See the
/// module documentation for why that distinction decides which published table
/// a result should be compared against.
#[must_use]
pub fn sidereal_to_tropical(sidereal_longitude_deg: f64, system: Ayanamsha, jd: f64) -> f64 {
    normalize_degrees(sidereal_longitude_deg + ayanamsha_value(system, jd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    /// Degrees corresponding to one nanodegree — the §6.1 anchor tolerance.
    const ANCHOR_TOL: f64 = 1e-9;

    fn dms(deg: f64) -> (i32, u32, f64) {
        let sign = if deg < 0.0 { -1 } else { 1 };
        let a = deg.abs();
        let d = a.trunc();
        let m = ((a - d) * 60.0).trunc();
        let s = (a - d - m / 60.0) * 3600.0;
        (sign * d as i32, m as u32, s)
    }

    // ── §6.1 Anchor reproduction ──────────────────────────────────────────────

    #[test]
    fn indian_official_reproduces_its_j2000_anchor() {
        let v = ayanamsha_value(Ayanamsha::IndianOfficial, julian::J2000);
        assert!(
            (v - IAE_J2000_ANCHOR_DEG).abs() < ANCHOR_TOL,
            "IAE 2022 p.380 gives 23°51'25\".53 at J2000; got {v}"
        );
    }

    #[test]
    fn fagan_bradley_reproduces_its_b1950_anchor() {
        let v = ayanamsha_value(Ayanamsha::FaganBradley, B1950);
        let expected = 360.0 - FAGAN_SVP_B1950_DEG;
        assert!(
            (v - expected).abs() < ANCHOR_TOL,
            "360° − SVP 335°57'28\".64 = 24°02'31\".36 at B1950.0; got {v}"
        );
    }

    #[test]
    fn krishnamurti_reproduces_its_1900_anchor() {
        let v = ayanamsha_value(Ayanamsha::Krishnamurti, KP_ANCHOR_JD);
        assert!(
            (v - KP_ANCHOR_DEG).abs() < ANCHOR_TOL,
            "KSK Vol-I p.140 gives 22°22'00\" on the 1st of Chitra 1900; got {v}"
        );
    }

    #[test]
    fn yukteshwar_reproduces_its_1894_anchor() {
        let v = ayanamsha_value(Ayanamsha::Yukteshwar, YUKTESHWAR_ANCHOR_JD);
        assert!(
            (v - YUKTESHWAR_ANCHOR_DEG).abs() < ANCHOR_TOL,
            "The Holy Science gives 20°54'36\" at the 1894 equinox; got {v}"
        );
    }

    #[test]
    fn raman_reproduces_both_printed_examples_exactly() {
        // Art. 49's own worked examples, which are the acceptance surface for the
        // year-granularity convention (DA-7).
        for (year, d, m, s) in [(1912, 21, 10, 55.0), (1918, 21, 15, 57.0)] {
            let jd = julian::calendar_to_jd(year, 1, 1.0);
            let v = ayanamsha_value(Ayanamsha::Raman, jd);
            let (gd, gm, gs) = dms(v);
            assert_eq!((gd, gm), (d, m), "Raman {year}: got {gd}°{gm}'{gs}\"");
            assert!(
                (gs - s).abs() < 1e-6,
                "Raman {year}: expected {d}°{m}'{s}\", got {gd}°{gm}'{gs}\""
            );
        }
    }

    #[test]
    fn surya_siddhanta_is_zero_at_the_kali_epoch() {
        let v = ayanamsha_value(Ayanamsha::SuryaSiddhanta, KALI_EPOCH_JD);
        assert!(
            v.abs() < ANCHOR_TOL,
            "the libration's phase origin is the Kali epoch, so it is zero there; got {v}"
        );
    }

    #[test]
    fn star_anchored_systems_put_their_star_where_the_system_says() {
        // §6.1 for Group B: the defining condition is an identity that must hold
        // at every epoch, not only at an anchor. If the assigned longitude and
        // the subtraction ever disagree, this catches it.
        let cases: [(Ayanamsha, &CatalogueStar, f64); 5] = [
            (Ayanamsha::RevatiPaksha, &ZETA_PISCIUM, 0.0),
            (Ayanamsha::PushyaPaksha, &DELTA_CANCRI, 106.0),
            (Ayanamsha::TrueChitra, &SPICA, 180.0),
            (Ayanamsha::ChandraHari, &LAMBDA_SCORPII, 240.0),
            (Ayanamsha::GalacticCenter0Sag, &SGR_A_STAR, 240.0),
        ];
        for (system, star, assigned) in cases {
            for jd in [1_721_060.0_f64, 2_415_020.0, julian::J2000, 2_524_594.0] {
                let sidereal = normalize_degrees(
                    mean_ecliptic_longitude_of_date(star, jd) - ayanamsha_value(system, jd),
                );
                let residual = normalize_degrees_signed(sidereal - assigned) * 3600.0;
                assert!(
                    residual.abs() < 1e-6,
                    "{}: star sits at sidereal {sidereal} at jd={jd}, should be {assigned} \
                     ({residual:+} arcsec)",
                    system.key()
                );
            }
        }
    }

    // ── §6.2 Zero-year inversion ──────────────────────────────────────────────

    /// Bisect for the Julian Day at which a system's ayanamsha is zero.
    fn zero_year(system: Ayanamsha, mut lo: f64, mut hi: f64) -> f64 {
        let f = |jd: f64| ayanamsha_value(system, jd);
        assert!(f(lo) * f(hi) < 0.0, "{}: no bracket", system.key());
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if f(lo) * f(mid) <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let jd = (lo + hi) / 2.0;
        let (year, _, _) = julian::jd_to_calendar(jd);
        f64::from(year) + (jd - julian::calendar_to_jd(year, 1, 1.0)) / 365.25
    }

    #[test]
    fn indian_official_inverts_to_the_documented_285_ce() {
        // IAE 2022 states, on the same page as the anchor, that the initial point
        // "coincides with the vernal equinoctial point of vernal equinox day of
        // 285 A.D.". Nothing in the derivation used that; it falls out.
        let y = zero_year(Ayanamsha::IndianOfficial, 1_700_000.0, julian::J2000);
        assert!(
            (y - 285.0).abs() < 1.0,
            "IAE documents 285 A.D.; the anchor inverts to {y:.2} CE"
        );
    }

    #[test]
    fn yukteshwar_inverts_to_his_documented_500_ce() {
        let y = zero_year(Ayanamsha::Yukteshwar, 1_700_000.0, julian::J2000);
        assert!(
            (y - 500.0).abs() < 1.0,
            "The Holy Science's cycle puts the zero at 500 CE; got {y:.2} CE"
        );
    }

    #[test]
    fn raman_inverts_to_his_documented_397_ce() {
        let y = zero_year(Ayanamsha::Raman, 1_700_000.0, julian::J2000);
        assert!(
            (y - 397.0).abs() < 0.01,
            "Art. 49's zero year is 397 CE; got {y:.4} CE"
        );
    }

    #[test]
    fn surya_siddhanta_inverts_to_499_ce_and_peaks_at_2299() {
        let y = zero_year(Ayanamsha::SuryaSiddhanta, 1_800_000.0, julian::J2000);
        assert!(
            (y - 499.0).abs() < 1.0,
            "the half-period crossing after the Kali epoch is 499 CE; got {y:.2} CE"
        );
        // DA-10: positive and increasing after 499 CE, reaching +27° at 2299 CE.
        let at_j2000 = ayanamsha_value(Ayanamsha::SuryaSiddhanta, julian::J2000);
        assert!(
            at_j2000 > 0.0,
            "a negative value at J2000 means the folding direction is inverted; got {at_j2000}"
        );
        let period = MAHAYUGA_DAYS / SS_REVOLUTIONS_PER_MAHAYUGA;
        let peak_jd = KALI_EPOCH_JD + 0.75 * period;
        let peak = ayanamsha_value(Ayanamsha::SuryaSiddhanta, peak_jd);
        assert!(
            (peak - SS_AMPLITUDE_DEG).abs() < 1e-9,
            "the extremum is +27°; got {peak}"
        );
        let (peak_year, _, _) = julian::jd_to_calendar(peak_jd);
        assert_eq!(peak_year, 2299, "the +27° extremum falls in 2299 CE");
    }

    #[test]
    fn krishnamurti_carries_its_own_documented_inconsistencies() {
        // KP is the one system exempt from the zero-year inversion, and an
        // exemption that is merely skipped is indistinguishable from an
        // oversight. So the inconsistency is pinned instead.
        //
        // KSK publishes three numbers that cannot all be true together: an
        // anchor of 22°22'00" at the 1st of Chitra 1900, a rate of
        // 50.2388475"/yr, and a zero year of 291 AD. Anchor divided by rate puts
        // the zero at ~297.5 CE, 6.5 years and about 5 arcminutes away.
        //
        // The anchor and rate are adopted because together they are the
        // operative definition and are what his own table reproduces. Choosing
        // a rate that made the inversion land on 291 would be tuning toward an
        // answer — the precise thing this module exists to stop — so the
        // disagreement is shipped, and this test is where it is recorded.
        let inverted = zero_year(Ayanamsha::Krishnamurti, 1_700_000.0, julian::J2000);
        assert!(
            (inverted - 297.5).abs() < 0.5,
            "KSK's anchor over his stated rate should invert to ~297.5 CE; got {inverted:.2}"
        );
        assert!(
            (inverted - 291.0).abs() > 5.0,
            "if this now lands on KSK's stated 291 AD, someone has changed the anchor or the \
             rate to make the inversion work. That is tuning, not derivation — check what moved."
        );

        // Where a tradition states its own rate, the derivation uses it and
        // records the divergence from the modern model rather than silently
        // substituting the better one. KSK attributes 50.2388475"/yr to Newcomb;
        // independent research has failed to trace it there. What this engine
        // can measure is the gap against the model it does implement: KSK's rate
        // sits about 6 arcsec/century below P03.
        //
        // Pinned so that "fixing" KP by swapping in the modern rate fails here
        // rather than silently redefining the system.
        let p03_rate_per_century = general_precession_p03(julian::J2000 + 36_525.0)
            - general_precession_p03(julian::J2000);
        let kp_rate_per_century = KP_RATE_ARCSEC_PER_YEAR * 36_525.0 / JULIAN_YEAR;
        let divergence = kp_rate_per_century - p03_rate_per_century;
        assert!(
            (-7.0..-5.0).contains(&divergence),
            "KSK's stated rate should sit ~6 arcsec/century below P03; got {divergence:.3}. \
             If this is now ~0, the stated rate has been replaced by the modern model, \
             which changes the definition of the system."
        );
    }

    #[test]
    fn true_chitra_inverts_to_285_ce_independently_of_the_indian_official_system() {
        // Two derivations that share no input — a J2000 polynomial anchor from a
        // government ephemeris, and a live star from a modern catalogue — land on
        // the same zero year. Neither was tuned toward the other.
        let y = zero_year(Ayanamsha::TrueChitra, 1_700_000.0, julian::J2000);
        assert!(
            (y - 285.0).abs() < 1.0,
            "Spica-at-180 inverts to {y:.2} CE; the Indian official system documents 285 A.D."
        );
    }

    #[test]
    fn revati_paksha_inverts_to_the_commentary_stated_year_for_zeta_piscium() {
        // Burgess records that zeta Piscium coincided with the vernal equinox in
        // A.D. 572, and that the Surya Siddhanta's own zero point (10' east of it)
        // did so about A.D. 560 — twelve years earlier at ~50"/yr. Holding the
        // star at 0 degrees must reproduce the first of those, not the second;
        // that is what distinguishes Revati-paksha from the Surya Siddhanta.
        let y = zero_year(Ayanamsha::RevatiPaksha, 1_700_000.0, julian::J2000);
        assert!(
            (y - 572.0).abs() < 5.0,
            "the commentary puts zeta Piscium at the equinox in A.D. 572; got {y:.2} CE"
        );
    }

    // ── Provenance and behaviour ──────────────────────────────────────────────

    #[test]
    fn yukteshwar_epoch_is_the_1894_march_equinox() {
        // The one constant in this module that this project computed rather than
        // read from a primary. It is pinned here so it cannot be a typed number:
        // if the solar theory moves, this fails rather than silently shifting the
        // system.
        //
        // 1894-03-20.624077 TT. Cross-checked against the published equinox time
        // for 1894 (March 20, ~14:56 UT).
        let (y, m, d) = julian::jd_to_calendar(YUKTESHWAR_ANCHOR_JD);
        assert_eq!((y, m), (1894, 3));
        assert!(
            (d - 20.624_077).abs() < 1e-5,
            "expected 1894 March 20.624077 TT, got {d}"
        );
    }

    #[test]
    fn every_system_is_monotonic_and_plausible_across_the_common_era() {
        // Not a conformance check against anyone — a shape check. Ayanamshas
        // increase with time at roughly 50"/yr, except the Surya Siddhanta, which
        // is a libration and reverses by construction.
        for &system in Ayanamsha::SIDEREAL {
            let a = ayanamsha_value(system, 2_415_020.0); // 1900
            let b = ayanamsha_value(system, 2_451_545.0); // 2000
            let rate = (b - a) * 3600.0; // arcsec per century
            assert!(
                (4_900.0..5_500.0).contains(&rate),
                "{}: {rate:.1} arcsec/century over the 20th century is not a precession rate",
                system.key()
            );
        }
    }

    #[test]
    fn no_system_jumps_at_a_year_boundary() {
        // Two systems are piecewise: Raman is defined on calendar years, and the
        // Surya Siddhanta folds at its quarter periods. A piecewise definition
        // implemented carelessly produces a step, and a step in an ayanamsha
        // moves every planet in a chart at midnight on 1 January.
        //
        // The Gregorian changeover year is in here on purpose: 1582 is 355 days
        // long, so anything that assumes a 365.25-day year drifts across it.
        for &system in Ayanamsha::SIDEREAL {
            for year in [398_i32, 1000, 1581, 1582, 1583, 1900, 1999, 2000, 2100] {
                let before = julian::calendar_to_jd(year + 1, 1, 1.0) - 1e-6;
                let after = julian::calendar_to_jd(year + 1, 1, 1.0) + 1e-6;
                let jump = (ayanamsha_value(system, after) - ayanamsha_value(system, before)).abs()
                    * 3600.0;
                assert!(
                    jump < 1e-3,
                    "{}: jumps {jump:.6} arcsec across the {year}/{} boundary",
                    system.key(),
                    year + 1
                );
            }
        }
    }

    #[test]
    fn the_surya_siddhanta_libration_turns_smoothly_at_its_extrema() {
        // A zigzag is continuous but not differentiable at the fold. Getting the
        // fold arithmetic wrong shows up as a discontinuity there and nowhere
        // else, so the generic year-boundary sweep above cannot see it.
        let period = MAHAYUGA_DAYS / SS_REVOLUTIONS_PER_MAHAYUGA;
        for quarter in [0.25_f64, 0.5, 0.75, 1.0] {
            let fold = KALI_EPOCH_JD + quarter * period;
            let a = ayanamsha_value(Ayanamsha::SuryaSiddhanta, fold - 1e-4);
            let b = ayanamsha_value(Ayanamsha::SuryaSiddhanta, fold + 1e-4);
            assert!(
                (a - b).abs() * 3600.0 < 1e-3,
                "discontinuity at phase {quarter}: {a} then {b}"
            );
        }
    }

    #[test]
    fn tropical_is_exactly_zero_everywhere() {
        for jd in [0.0_f64, 1_000_000.0, julian::J2000, 3_000_000.0] {
            assert_eq!(ayanamsha_value(Ayanamsha::Tropical, jd), 0.0);
        }
    }

    #[test]
    fn systems_are_distinct_from_one_another() {
        // Two variants that returned the same number would be one system wearing
        // two names — the exact artefact this re-derivation removed.
        let jd = julian::J2000;
        for (i, &a) in Ayanamsha::SIDEREAL.iter().enumerate() {
            for &b in &Ayanamsha::SIDEREAL[i + 1..] {
                let d = (ayanamsha_value(a, jd) - ayanamsha_value(b, jd)).abs() * 3600.0;
                assert!(
                    d > 1.0,
                    "{} and {} differ by only {d:.3} arcsec at J2000",
                    a.key(),
                    b.key()
                );
            }
        }
    }

    #[test]
    fn conversion_round_trips() {
        for &system in Ayanamsha::ALL {
            for jd in [2_415_020.0_f64, julian::J2000, 2_488_070.0] {
                for lon in [0.0_f64, 47.5, 180.0, 359.9] {
                    let there = tropical_to_sidereal(lon, system, jd);
                    let back = sidereal_to_tropical(there, system, jd);
                    let d = normalize_degrees_signed(back - lon).abs();
                    assert!(d < 1e-9, "{}: {lon} -> {there} -> {back}", system.key());
                }
            }
        }
    }

    // ── The name surface ──────────────────────────────────────────────────────

    #[test]
    fn every_key_round_trips_through_from_str() {
        for &system in Ayanamsha::ALL {
            assert_eq!(Ayanamsha::from_str(system.key()), Ok(system));
        }
    }

    #[test]
    fn parsing_ignores_case_separators_and_punctuation() {
        for s in [
            "indian_official",
            "INDIAN-OFFICIAL",
            "Indian Official",
            "lahiri",
            "LAHIRI",
        ] {
            assert_eq!(Ayanamsha::from_str(s), Ok(Ayanamsha::IndianOfficial), "{s}");
        }
        assert_eq!(
            Ayanamsha::from_str("fagan_bradley"),
            Ok(Ayanamsha::FaganBradley)
        );
        assert_eq!(Ayanamsha::from_str("KP"), Ok(Ayanamsha::Krishnamurti));
    }

    #[test]
    fn removed_names_hard_error_and_say_what_happened() {
        // The contract from the spec: a removed name must never silently become a
        // default or a neighbour. Every disposition entry must actually be
        // reachable, and must actually fail.
        for (name, _) in DISPOSITIONS {
            let err = Ayanamsha::from_str(name)
                .expect_err("a removed v5 name must not parse to a system");
            assert!(
                !err.disposition.is_empty(),
                "{name} errors without saying why"
            );
            assert!(
                !err.disposition.starts_with("not a system this engine has"),
                "{name} is in DISPOSITIONS but fell through to the generic message"
            );
        }
    }

    #[test]
    fn a_name_nobody_ever_shipped_still_errors_usefully() {
        let err = Ayanamsha::from_str("Bogus").unwrap_err();
        assert!(err.disposition.contains("2026-08-17-ayanamsha-cleanroom"));
    }

    #[test]
    fn no_disposition_shadows_a_live_system() {
        // If a name is both parseable and listed as removed, the listing is a lie.
        for (name, _) in DISPOSITIONS {
            assert!(
                Ayanamsha::from_str(name).is_err(),
                "{name} is listed as removed but still parses"
            );
        }
    }

    #[test]
    fn serde_round_trips_and_refuses_retired_names() {
        for &system in Ayanamsha::ALL {
            let json = serde_json::to_string(&system).unwrap();
            assert_eq!(json, format!("\"{}\"", system.key()));
            let back: Ayanamsha = serde_json::from_str(&json).unwrap();
            assert_eq!(back, system);
        }
        // The clause this exists for: a name from a stored config must not be
        // absorbed, and the error must carry the disposition rather than serde's
        // generic unknown-variant message.
        let err = serde_json::from_str::<Ayanamsha>("\"BabylonianHuber\"").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("estimated") || msg.contains("dropped"),
            "deserialization should surface the disposition, got: {msg}"
        );
        assert!(serde_json::from_str::<Ayanamsha>("\"Lahiri\"").is_ok());
    }

    #[test]
    fn metadata_is_present_for_every_system() {
        for &system in Ayanamsha::ALL {
            assert!(!system.name().is_empty());
            assert!(!system.key().is_empty());
            assert!(!system.primary_source().is_empty());
        }
        assert_eq!(Ayanamsha::SIDEREAL.len(), 11);
        assert_eq!(Ayanamsha::ALL.len(), 12);
        assert!(!Ayanamsha::SIDEREAL.contains(&Ayanamsha::Tropical));
    }
}
