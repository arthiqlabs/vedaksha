// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-locale
//!
//! Localization engine for the Vedākṣa platform.
//!
//! Provides localized names for all astrological terms in 7 Tier 1 languages:
//! English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Bengali.
//!
//! # Implementation Note
//!
//! This crate uses static lookup tables rather than Mozilla Fluent (.ftl files).
//! Static tables are `no_std`-compatible, zero-allocation at lookup time, and
//! sufficient for our fixed astrological vocabulary. See `DATA_PROVENANCE.md`
//! for details.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod aspects;
pub mod dashas;
pub mod deities;
pub mod dignities;
pub mod houses;
pub mod karanas;
pub mod nakshatras;
pub mod panchanga_yogas;
pub mod planets;
pub mod signs;
pub mod yogas;

/// Supported Tier 1 languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Hindi,
    Sanskrit,
    Tamil,
    Telugu,
    Kannada,
    Bengali,
}

impl Language {
    /// ISO 639-1/639-2 code for this language.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Hindi => "hi",
            Self::Sanskrit => "sa",
            Self::Tamil => "ta",
            Self::Telugu => "te",
            Self::Kannada => "kn",
            Self::Bengali => "bn",
        }
    }

    /// Native name of this language.
    #[must_use]
    pub const fn native_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Hindi => "हिन्दी",
            Self::Sanskrit => "संस्कृतम्",
            Self::Tamil => "தமிழ்",
            Self::Telugu => "తెలుగు",
            Self::Kannada => "ಕನ್ನಡ",
            Self::Bengali => "বাংলা",
        }
    }

    /// All supported languages, in canonical order.
    pub const ALL: &'static [Self] = &[
        Self::English,
        Self::Hindi,
        Self::Sanskrit,
        Self::Tamil,
        Self::Telugu,
        Self::Kannada,
        Self::Bengali,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspects::ASPECT_COUNT;
    use deities::DEITY_COUNT;
    use karanas::KARANA_COUNT;
    use nakshatras::NAKSHATRA_COUNT;
    use panchanga_yogas::PANCHANGA_YOGA_COUNT;
    use planets::PLANET_COUNT;
    use signs::SIGN_COUNT;

    // ── Language metadata ────────────────────────────────────────────────────

    #[test]
    fn language_code_english() {
        assert_eq!(Language::English.code(), "en");
    }

    #[test]
    fn language_native_name_hindi() {
        assert_eq!(Language::Hindi.native_name(), "हिन्दी");
    }

    #[test]
    fn language_all_contains_all_seven() {
        assert_eq!(Language::ALL.len(), 7);
        assert!(Language::ALL.contains(&Language::English));
        assert!(Language::ALL.contains(&Language::Hindi));
        assert!(Language::ALL.contains(&Language::Sanskrit));
        assert!(Language::ALL.contains(&Language::Tamil));
        assert!(Language::ALL.contains(&Language::Telugu));
        assert!(Language::ALL.contains(&Language::Kannada));
        assert!(Language::ALL.contains(&Language::Bengali));
    }

    // ── Planet names ─────────────────────────────────────────────────────────

    #[test]
    fn planet_name_english_sun() {
        assert_eq!(planets::planet_name(0, Language::English), "Sun");
    }

    #[test]
    fn planet_name_hindi_sun() {
        assert_eq!(planets::planet_name(0, Language::Hindi), "सूर्य");
    }

    #[test]
    fn planet_count_is_nine() {
        assert_eq!(PLANET_COUNT, 9);
    }

    #[test]
    fn all_languages_all_planets_non_empty() {
        for lang in Language::ALL {
            for i in 0..PLANET_COUNT {
                let name = planets::planet_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty planet name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Sign names ───────────────────────────────────────────────────────────

    #[test]
    fn sign_name_sanskrit_aries() {
        assert_eq!(signs::sign_name(0, Language::Sanskrit), "मेषः");
    }

    #[test]
    fn sign_count_is_twelve() {
        assert_eq!(SIGN_COUNT, 12);
    }

    #[test]
    fn all_languages_all_signs_non_empty() {
        for lang in Language::ALL {
            for i in 0..SIGN_COUNT {
                let name = signs::sign_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty sign name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Nakshatra names ──────────────────────────────────────────────────────

    #[test]
    fn nakshatra_name_tamil_ashwini_is_tamil_script() {
        let name = nakshatras::nakshatra_name(0, Language::Tamil);
        // Tamil script starts in Unicode range U+0B80–U+0BFF
        assert!(
            name.chars().any(|c| ('\u{0B80}'..='\u{0BFF}').contains(&c)),
            "Expected Tamil script, got: {name}"
        );
    }

    #[test]
    fn nakshatra_count_is_twenty_seven() {
        assert_eq!(NAKSHATRA_COUNT, 27);
    }

    #[test]
    fn all_languages_all_nakshatras_non_empty() {
        for lang in Language::ALL {
            for i in 0..NAKSHATRA_COUNT {
                let name = nakshatras::nakshatra_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty nakshatra name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Aspect names ─────────────────────────────────────────────────────────

    #[test]
    fn aspect_count_is_eleven() {
        assert_eq!(ASPECT_COUNT, 11);
    }

    #[test]
    fn aspect_names_english_all_non_empty() {
        for i in 0..ASPECT_COUNT {
            let name = aspects::aspect_name(i, Language::English);
            assert!(!name.is_empty(), "Empty aspect name at index {i}");
        }
    }

    #[test]
    fn aspect_name_english_conjunction() {
        assert_eq!(aspects::aspect_name(0, Language::English), "Conjunction");
    }

    // ── Dasha lord names ─────────────────────────────────────────────────────

    #[test]
    fn dasha_lord_ketu_is_first() {
        // Index 0 in Vimshottari = Ketu
        assert_eq!(dashas::dasha_lord_name(0, Language::English), "Ketu");
    }

    #[test]
    fn dasha_lord_mercury_is_last() {
        // Index 8 in Vimshottari = Mercury
        assert_eq!(dashas::dasha_lord_name(8, Language::English), "Mercury");
    }

    #[test]
    fn all_dasha_lords_all_languages_non_empty() {
        for lang in Language::ALL {
            for i in 0..dashas::DASHA_LORD_COUNT {
                let name = dashas::dasha_lord_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty dasha lord name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Deity names ─────────────────────────────────────────────────────────

    #[test]
    fn deity_count_is_twenty_seven() {
        assert_eq!(DEITY_COUNT, 27);
    }

    #[test]
    fn deity_name_english_first() {
        assert_eq!(deities::deity_name(0, Language::English), "Ashwini Kumaras");
    }

    #[test]
    fn deity_name_hindi_first() {
        assert_eq!(deities::deity_name(0, Language::Hindi), "अश्विनी कुमार");
    }

    #[test]
    fn all_languages_all_deities_non_empty() {
        for lang in Language::ALL {
            for i in 0..DEITY_COUNT {
                let name = deities::deity_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty deity name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Karana names ────────────────────────────────────────────────────────

    #[test]
    fn karana_count_is_eleven() {
        assert_eq!(KARANA_COUNT, 11);
    }

    #[test]
    fn karana_name_english_first() {
        assert_eq!(karanas::karana_name(0, Language::English), "Bava");
    }

    #[test]
    fn karana_name_sanskrit_vishti() {
        assert_eq!(karanas::karana_name(6, Language::Sanskrit), "विष्टिः");
    }

    // ── Devanagari spot-checks (one per module) ──

    #[test]
    fn sanskrit_planet_sun_devanagari() {
        assert_eq!(planets::planet_name(0, Language::Sanskrit), "सूर्यः");
    }

    #[test]
    fn sanskrit_nakshatra_ashwini_devanagari() {
        assert_eq!(nakshatras::nakshatra_name(0, Language::Sanskrit), "अश्विनी");
    }

    #[test]
    fn sanskrit_deity_yama_devanagari() {
        assert_eq!(deities::deity_name(1, Language::Sanskrit), "यमः");
    }

    #[test]
    fn sanskrit_house_first_devanagari() {
        assert_eq!(houses::house_name(1, Language::Sanskrit), "प्रथमभावः");
    }

    #[test]
    fn sanskrit_dignity_ucca_devanagari() {
        assert_eq!(dignities::dignity_name(0, Language::Sanskrit), "उच्चम्");
    }

    #[test]
    fn sanskrit_panchanga_yoga_vishkambha_devanagari() {
        assert_eq!(
            panchanga_yogas::panchanga_yoga_name(0, Language::Sanskrit),
            "विष्कम्भः"
        );
    }

    #[test]
    fn sanskrit_yoga_gajakesari_devanagari() {
        assert_eq!(yogas::yoga_name(0, Language::Sanskrit), "गजकेसरी");
    }

    #[test]
    fn sanskrit_aspect_yuti_devanagari() {
        assert_eq!(aspects::aspect_name(0, Language::Sanskrit), "युतिः");
    }

    #[test]
    fn sanskrit_native_name_devanagari() {
        assert_eq!(Language::Sanskrit.native_name(), "संस्कृतम्");
    }

    #[test]
    fn all_languages_all_karanas_non_empty() {
        for lang in Language::ALL {
            for i in 0..KARANA_COUNT {
                let name = karanas::karana_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty karana name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }

    // ── Panchanga yoga names ────────────────────────────────────────────────

    #[test]
    fn panchanga_yoga_count_is_twenty_seven() {
        assert_eq!(PANCHANGA_YOGA_COUNT, 27);
    }

    #[test]
    fn panchanga_yoga_name_english_first() {
        assert_eq!(
            panchanga_yogas::panchanga_yoga_name(0, Language::English),
            "Vishkambha"
        );
    }

    #[test]
    fn panchanga_yoga_name_hindi_last() {
        assert_eq!(
            panchanga_yogas::panchanga_yoga_name(26, Language::Hindi),
            "वैधृति"
        );
    }

    #[test]
    fn all_languages_all_panchanga_yogas_non_empty() {
        for lang in Language::ALL {
            for i in 0..PANCHANGA_YOGA_COUNT {
                let name = panchanga_yogas::panchanga_yoga_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty panchanga yoga name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }
}
