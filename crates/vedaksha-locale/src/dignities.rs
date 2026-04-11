// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized planetary dignity names.
//!
//! Index mapping (0-based):
//!  0  Domicile   — planet in its own sign (svakṣetra)
//!  1  Exaltation — planet in its sign of exaltation (ucca)
//!  2  Detriment  — planet opposite its domicile (nīcabhañga opposite)
//!  3  Fall       — planet in its sign of debilitation (nīca)
//!  4  Peregrine  — planet in a neutral/unaffiliated sign
//!
//! Sources: BPHS Ch. 3; Ptolemy "Tetrabiblos" I.17-20.

use crate::Language;

/// Number of dignity categories.
pub const DIGNITY_COUNT: usize = 5;

/// Get the localized name of a dignity category.
///
/// # Panics
///
/// Panics in debug builds if `index >= DIGNITY_COUNT`.
#[must_use]
pub fn dignity_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => DIGNITIES_EN[index],
        Language::Hindi => DIGNITIES_HI[index],
        Language::Sanskrit => DIGNITIES_SA[index],
        Language::Tamil => DIGNITIES_TA[index],
        Language::Telugu => DIGNITIES_TE[index],
        Language::Kannada => DIGNITIES_KN[index],
        Language::Bengali => DIGNITIES_BN[index],
    }
}

static DIGNITIES_EN: &[&str] = &["Domicile", "Exaltation", "Detriment", "Fall", "Peregrine"];

static DIGNITIES_HI: &[&str] = &["स्वक्षेत्र", "उच्च", "शत्रुक्षेत्र", "नीच", "सामान्य"];

/// Sanskrit names in IAST transliteration.
static DIGNITIES_SA: &[&str] = &["Svakṣetra", "Ucca", "Śatrukṣetra", "Nīca", "Sāmānya"];

static DIGNITIES_TA: &[&str] = &["சொந்த வீடு", "உச்சம்", "பகை வீடு", "நீசம்", "சாமான்யம்"];

static DIGNITIES_TE: &[&str] = &["స్వక్షేత్రం", "ఉచ్చం", "శత్రుక్షేత్రం", "నీచం", "సాధారణం"];

static DIGNITIES_KN: &[&str] = &["ಸ್ವಕ್ಷೇತ್ರ", "ಉಚ್ಚ", "ಶತ್ರುಕ್ಷೇತ್ರ", "ನೀಚ", "ಸಾಮಾನ್ಯ"];

static DIGNITIES_BN: &[&str] = &["স্বক্ষেত্র", "উচ্চ", "শত্রুক্ষেত্র", "নীচ", "সাধারণ"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn dignity_count_is_five() {
        assert_eq!(DIGNITY_COUNT, 5);
    }

    #[test]
    fn domicile_english() {
        assert_eq!(dignity_name(0, Language::English), "Domicile");
    }

    #[test]
    fn exaltation_hindi() {
        assert_eq!(dignity_name(1, Language::Hindi), "उच्च");
    }

    #[test]
    fn fall_sanskrit() {
        assert_eq!(dignity_name(3, Language::Sanskrit), "Nīca");
    }

    #[test]
    fn all_languages_all_dignities_non_empty() {
        for lang in Language::ALL {
            for i in 0..DIGNITY_COUNT {
                let name = dignity_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty dignity name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }
}
