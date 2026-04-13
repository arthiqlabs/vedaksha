// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized zodiac sign (rāśi) names.
//!
//! Index mapping (0-based):
//! 0=Aries, 1=Taurus, 2=Gemini, 3=Cancer, 4=Leo, 5=Virgo,
//! 6=Libra, 7=Scorpio, 8=Sagittarius, 9=Capricorn, 10=Aquarius, 11=Pisces
//!
//! Sources: BPHS Ch. 4, B.V. Raman "A Manual of Hindu Astrology".

use crate::Language;

/// Number of zodiac signs.
pub const SIGN_COUNT: usize = 12;

/// Get the localized name of a zodiac sign (rāśi).
///
/// # Panics
///
/// Panics in debug builds if `index >= SIGN_COUNT`.
#[must_use]
pub fn sign_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => SIGNS_EN[index],
        Language::Hindi => SIGNS_HI[index],
        Language::Sanskrit => SIGNS_SA[index],
        Language::Tamil => SIGNS_TA[index],
        Language::Telugu => SIGNS_TE[index],
        Language::Kannada => SIGNS_KN[index],
        Language::Bengali => SIGNS_BN[index],
    }
}

static SIGNS_EN: &[&str] = &[
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

static SIGNS_HI: &[&str] = &[
    "मेष",
    "वृषभ",
    "मिथुन",
    "कर्क",
    "सिंह",
    "कन्या",
    "तुला",
    "वृश्चिक",
    "धनु",
    "मकर",
    "कुम्भ",
    "मीन",
];

/// Sanskrit names in IAST transliteration.
static SIGNS_SA: &[&str] = &[
    "Meṣa",
    "Vṛṣabha",
    "Mithuna",
    "Karka",
    "Siṃha",
    "Kanyā",
    "Tulā",
    "Vṛścika",
    "Dhanus",
    "Makara",
    "Kumbha",
    "Mīna",
];

static SIGNS_TA: &[&str] = &[
    "மேஷம்",
    "ரிஷபம்",
    "மிதுனம்",
    "கடகம்",
    "சிம்மம்",
    "கன்னி",
    "துலாம்",
    "விருச்சிகம்",
    "தனுசு",
    "மகரம்",
    "கும்பம்",
    "மீனம்",
];

static SIGNS_TE: &[&str] = &[
    "మేషం",
    "వృషభం",
    "మిథునం",
    "కర్కాటకం",
    "సింహం",
    "కన్య",
    "తుల",
    "వృశ్చికం",
    "ధనుస్సు",
    "మకరం",
    "కుంభం",
    "మీనం",
];

static SIGNS_KN: &[&str] = &[
    "ಮೇಷ",
    "ವೃಷಭ",
    "ಮಿಥುನ",
    "ಕರ್ಕಾಟಕ",
    "ಸಿಂಹ",
    "ಕನ್ಯಾ",
    "ತುಲಾ",
    "ವೃಶ್ಚಿಕ",
    "ಧನುಸ್ಸು",
    "ಮಕರ",
    "ಕುಂಭ",
    "ಮೀನ",
];

static SIGNS_BN: &[&str] = &[
    "মেষ",
    "বৃষ",
    "মিথুন",
    "কর্কট",
    "সিংহ",
    "কন্যা",
    "তুলা",
    "বৃশ্চিক",
    "ধনু",
    "মকর",
    "কুম্ভ",
    "মীন",
];
