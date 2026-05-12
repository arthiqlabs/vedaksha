// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized house (bhāva) names.
//!
//! Index mapping (0-based): 0 = First House (Lagna) … 11 = Twelfth House.
//!
//! Sources: BPHS Ch. 11; B.V. Raman "A Manual of Hindu Astrology" Ch. 4.

use crate::Language;

/// Number of houses in a chart.
pub const HOUSE_COUNT: usize = 12;

/// Get the localized name of a house (bhāva).
///
/// `number` is 1-based (1–12); internally mapped to 0-based index.
///
/// # Panics
///
/// Panics in debug builds if `number` is 0 or > 12.
#[must_use]
pub fn house_name(number: u8, lang: Language) -> &'static str {
    let index = (number.wrapping_sub(1)) as usize;
    match lang {
        Language::English => HOUSES_EN[index],
        Language::Hindi => HOUSES_HI[index],
        Language::Sanskrit => HOUSES_SA[index],
        Language::Tamil => HOUSES_TA[index],
        Language::Telugu => HOUSES_TE[index],
        Language::Kannada => HOUSES_KN[index],
        Language::Bengali => HOUSES_BN[index],
    }
}

static HOUSES_EN: &[&str] = &[
    "First House",
    "Second House",
    "Third House",
    "Fourth House",
    "Fifth House",
    "Sixth House",
    "Seventh House",
    "Eighth House",
    "Ninth House",
    "Tenth House",
    "Eleventh House",
    "Twelfth House",
];

static HOUSES_HI: &[&str] = &[
    "प्रथम भाव",
    "द्वितीय भाव",
    "तृतीय भाव",
    "चतुर्थ भाव",
    "पंचम भाव",
    "षष्ठ भाव",
    "सप्तम भाव",
    "अष्टम भाव",
    "नवम भाव",
    "दशम भाव",
    "एकादश भाव",
    "द्वादश भाव",
];

/// Sanskrit names in Devanagari script.
static HOUSES_SA: &[&str] = &[
    "प्रथमभावः",
    "द्वितीयभावः",
    "तृतीयभावः",
    "चतुर्थभावः",
    "पञ्चमभावः",
    "षष्ठभावः",
    "सप्तमभावः",
    "अष्टमभावः",
    "नवमभावः",
    "दशमभावः",
    "एकादशभावः",
    "द्वादशभावः",
];

static HOUSES_TA: &[&str] = &[
    "முதல் பாவம்",
    "இரண்டாம் பாவம்",
    "மூன்றாம் பாவம்",
    "நான்காம் பாவம்",
    "ஐந்தாம் பாவம்",
    "ஆறாம் பாவம்",
    "ஏழாம் பாவம்",
    "எட்டாம் பாவம்",
    "ஒன்பதாம் பாவம்",
    "பத்தாம் பாவம்",
    "பதினொன்றாம் பாவம்",
    "பன்னிரண்டாம் பாவம்",
];

static HOUSES_TE: &[&str] = &[
    "మొదటి భావం",
    "రెండవ భావం",
    "మూడవ భావం",
    "నాల్గవ భావం",
    "ఐదవ భావం",
    "ఆరవ భావం",
    "ఏడవ భావం",
    "ఎనిమిదవ భావం",
    "తొమ్మిదవ భావం",
    "పదవ భావం",
    "పదకొండవ భావం",
    "పన్నెండవ భావం",
];

static HOUSES_KN: &[&str] = &[
    "ಮೊದಲನೇ ಭಾವ",
    "ಎರಡನೇ ಭಾವ",
    "ಮೂರನೇ ಭಾವ",
    "ನಾಲ್ಕನೇ ಭಾವ",
    "ಐದನೇ ಭಾವ",
    "ಆರನೇ ಭಾವ",
    "ಏಳನೇ ಭಾವ",
    "ಎಂಟನೇ ಭಾವ",
    "ಒಂಬತ್ತನೇ ಭಾವ",
    "ಹತ್ತನೇ ಭಾವ",
    "ಹನ್ನೊಂದನೇ ಭಾವ",
    "ಹನ್ನೆರಡನೇ ಭಾವ",
];

static HOUSES_BN: &[&str] = &[
    "প্রথম ভাব",
    "দ্বিতীয় ভাব",
    "তৃতীয় ভাব",
    "চতুর্থ ভাব",
    "পঞ্চম ভাব",
    "ষষ্ঠ ভাব",
    "সপ্তম ভাব",
    "অষ্টম ভাব",
    "নবম ভাব",
    "দশম ভাব",
    "একাদশ ভাব",
    "দ্বাদশ ভাব",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn house_count_is_twelve() {
        assert_eq!(HOUSE_COUNT, 12);
    }

    #[test]
    fn first_house_english() {
        assert_eq!(house_name(1, Language::English), "First House");
    }

    #[test]
    fn tenth_house_hindi() {
        assert_eq!(house_name(10, Language::Hindi), "दशम भाव");
    }

    #[test]
    fn all_languages_all_houses_non_empty() {
        for lang in Language::ALL {
            for n in 1u8..=12 {
                let name = house_name(n, *lang);
                assert!(
                    !name.is_empty(),
                    "Empty house name: lang={:?}, house={n}",
                    lang
                );
            }
        }
    }
}
