// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized yoga names.
//!
//! Index mapping (0-based):
//!  0  Ruchaka Yoga     — Mahapurusha (Mars in own/exaltation sign in kendra)
//!  1  Bhadra Yoga      — Mahapurusha (Mercury in own/exaltation sign in kendra)
//!  2  Hamsa Yoga       — Mahapurusha (Jupiter in own/exaltation sign in kendra)
//!  3  Malavya Yoga     — Mahapurusha (Venus in own/exaltation sign in kendra)
//!  4  Shasha Yoga      — Mahapurusha (Saturn in own/exaltation sign in kendra)
//!  5  Gajakesari Yoga  — Jupiter in kendra from Moon
//!  6  Budhaditya Yoga  — Sun and Mercury conjunct
//!  7  Kemadruma Yoga   — No planets in 2nd/12th from Moon
//!
//! Sources: BPHS Ch. 35-37; B.V. Raman, "Three Hundred Important Combinations".

use crate::Language;

/// Number of yoga entries in the lookup tables.
pub const YOGA_COUNT: usize = 8;

/// Get the localized name of a yoga.
///
/// # Panics
///
/// Panics in debug builds if `index >= YOGA_COUNT`.
#[must_use]
pub fn yoga_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => YOGAS_EN[index],
        Language::Hindi => YOGAS_HI[index],
        Language::Sanskrit => YOGAS_SA[index],
        Language::Tamil => YOGAS_TA[index],
        Language::Telugu => YOGAS_TE[index],
        Language::Kannada => YOGAS_KN[index],
        Language::Bengali => YOGAS_BN[index],
    }
}

static YOGAS_EN: &[&str] = &[
    "Ruchaka Yoga",
    "Bhadra Yoga",
    "Hamsa Yoga",
    "Malavya Yoga",
    "Shasha Yoga",
    "Gajakesari Yoga",
    "Budhaditya Yoga",
    "Kemadruma Yoga",
];

static YOGAS_HI: &[&str] = &[
    "रुचक योग",
    "भद्र योग",
    "हंस योग",
    "मालव्य योग",
    "शश योग",
    "गजकेसरी योग",
    "बुधादित्य योग",
    "केमद्रुम योग",
];

/// Sanskrit names in IAST transliteration.
static YOGAS_SA: &[&str] = &[
    "Rucaka Yoga",
    "Bhadra Yoga",
    "Haṃsa Yoga",
    "Mālavya Yoga",
    "Śaśa Yoga",
    "Gajakesarī Yoga",
    "Budhāditya Yoga",
    "Kemadrama Yoga",
];

static YOGAS_TA: &[&str] = &[
    "ருசக யோகம்",
    "பத்ர யோகம்",
    "ஹம்ச யோகம்",
    "மாலவ்ய யோகம்",
    "சசா யோகம்",
    "கஜகேசரி யோகம்",
    "புதாதித்ய யோகம்",
    "கேமத்ரும யோகம்",
];

static YOGAS_TE: &[&str] = &[
    "రుచక యోగం",
    "భద్ర యోగం",
    "హంస యోగం",
    "మాలవ్య యోగం",
    "శశ యోగం",
    "గజకేసరి యోగం",
    "బుధాదిత్య యోగం",
    "కేమద్రుమ యోగం",
];

static YOGAS_KN: &[&str] = &[
    "ರುಚಕ ಯೋಗ",
    "ಭದ್ರ ಯೋಗ",
    "ಹಂಸ ಯೋಗ",
    "ಮಾಲವ್ಯ ಯೋಗ",
    "ಶಶ ಯೋಗ",
    "ಗಜಕೇಸರಿ ಯೋಗ",
    "ಬುಧಾದಿತ್ಯ ಯೋಗ",
    "ಕೇಮದ್ರುಮ ಯೋಗ",
];

static YOGAS_BN: &[&str] = &[
    "রুচক যোগ",
    "ভদ্র যোগ",
    "হংস যোগ",
    "মালব্য যোগ",
    "শশ যোগ",
    "গজকেসরী যোগ",
    "বুধাদিত্য যোগ",
    "কেমদ্রুম যোগ",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn yoga_count_is_eight() {
        assert_eq!(YOGA_COUNT, 8);
    }

    #[test]
    fn ruchaka_yoga_english() {
        assert_eq!(yoga_name(0, Language::English), "Ruchaka Yoga");
    }

    #[test]
    fn gajakesari_yoga_hindi() {
        assert_eq!(yoga_name(5, Language::Hindi), "गजकेसरी योग");
    }

    #[test]
    fn all_languages_all_yogas_non_empty() {
        for lang in Language::ALL {
            for i in 0..YOGA_COUNT {
                let name = yoga_name(i, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty yoga name: lang={:?}, index={i}",
                    lang
                );
            }
        }
    }
}
