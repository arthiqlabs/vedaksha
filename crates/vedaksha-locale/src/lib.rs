// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
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
pub mod dignities;
pub mod houses;
pub mod nakshatras;
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
            Self::Sanskrit => "Saṃskṛtam",
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
    use nakshatras::NAKSHATRA_COUNT;
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
        assert_eq!(signs::sign_name(0, Language::Sanskrit), "Meṣa");
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
}
