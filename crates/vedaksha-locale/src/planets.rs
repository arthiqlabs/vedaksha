// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized planet names for the 9 Vimshottari graha.
//!
//! Index mapping:
//! 0 = Sun, 1 = Moon, 2 = Mars, 3 = Mercury, 4 = Jupiter,
//! 5 = Venus, 6 = Saturn, 7 = Rahu, 8 = Ketu
//!
//! Sources: BPHS Ch. 3, B.V. Raman "A Manual of Hindu Astrology".

use crate::Language;

/// Number of planets in the lookup tables.
pub const PLANET_COUNT: usize = 9;

/// Get the localized name of a planet.
///
/// # Panics
///
/// Panics in debug builds if `index >= PLANET_COUNT`.
/// In release builds the behaviour is defined by the slice indexing rules.
#[must_use]
pub fn planet_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => PLANETS_EN[index],
        Language::Hindi => PLANETS_HI[index],
        Language::Sanskrit => PLANETS_SA[index],
        Language::Tamil => PLANETS_TA[index],
        Language::Telugu => PLANETS_TE[index],
        Language::Kannada => PLANETS_KN[index],
        Language::Bengali => PLANETS_BN[index],
    }
}

static PLANETS_EN: &[&str] = &[
    "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu",
];

static PLANETS_HI: &[&str] = &[
    "सूर्य",
    "चन्द्र",
    "मंगल",
    "बुध",
    "बृहस्पति",
    "शुक्र",
    "शनि",
    "राहु",
    "केतु",
];

/// Sanskrit names in IAST transliteration (BPHS Ch. 3).
static PLANETS_SA: &[&str] = &[
    "Sūrya",
    "Candra",
    "Maṅgala",
    "Budha",
    "Bṛhaspati",
    "Śukra",
    "Śani",
    "Rāhu",
    "Ketu",
];

static PLANETS_TA: &[&str] = &[
    "சூரியன்",
    "சந்திரன்",
    "செவ்வாய்",
    "புதன்",
    "குரு",
    "சுக்கிரன்",
    "சனி",
    "ராகு",
    "கேது",
];

static PLANETS_TE: &[&str] = &[
    "సూర్యుడు",
    "చంద్రుడు",
    "కుజుడు",
    "బుధుడు",
    "గురుడు",
    "శుక్రుడు",
    "శని",
    "రాహువు",
    "కేతువు",
];

static PLANETS_KN: &[&str] = &[
    "ಸೂರ್ಯ",
    "ಚಂದ್ರ",
    "ಮಂಗಳ",
    "ಬುಧ",
    "ಗುರು",
    "ಶುಕ್ರ",
    "ಶನಿ",
    "ರಾಹು",
    "ಕೇತು",
];

static PLANETS_BN: &[&str] = &[
    "সূর্য",
    "চন্দ্র",
    "মঙ্গল",
    "বুধ",
    "বৃহস্পতি",
    "শুক্র",
    "শনি",
    "রাহু",
    "কেতু",
];
