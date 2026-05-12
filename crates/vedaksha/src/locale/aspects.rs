// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized aspect type names.
//!
//! Index mapping (0-based):
//!  0  Conjunction (0°)
//!  1  Sextile (60°)
//!  2  Square (90°)
//!  3  Trine (120°)
//!  4  Opposition (180°)
//!  5  Semi-sextile (30°)
//!  6  Quincunx / Inconjunct (150°)
//!  7  Semi-square (45°)
//!  8  Sesquiquadrate (135°)
//!  9  Quintile (72°)
//! 10  Bi-quintile (144°)
//!
//! For Hindi/Sanskrit: Vedic terms are used where established.
//! Western-only aspects (semi-square, sesquiquadrate, quintile, bi-quintile)
//! use transliterated Sanskrit neologisms consistent with B.V. Raman's usage.

use crate::Language;

/// Number of aspect types.
pub const ASPECT_COUNT: usize = 11;

/// Get the localized name of an aspect type.
///
/// # Panics
///
/// Panics in debug builds if `index >= ASPECT_COUNT`.
#[must_use]
pub fn aspect_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => ASPECTS_EN[index],
        Language::Hindi => ASPECTS_HI[index],
        Language::Sanskrit => ASPECTS_SA[index],
        Language::Tamil => ASPECTS_TA[index],
        Language::Telugu => ASPECTS_TE[index],
        Language::Kannada => ASPECTS_KN[index],
        Language::Bengali => ASPECTS_BN[index],
    }
}

static ASPECTS_EN: &[&str] = &[
    "Conjunction",
    "Sextile",
    "Square",
    "Trine",
    "Opposition",
    "Semi-sextile",
    "Quincunx",
    "Semi-square",
    "Sesquiquadrate",
    "Quintile",
    "Bi-quintile",
];

static ASPECTS_HI: &[&str] = &[
    "युति",
    "षष्ठांश",
    "चतुर्थांश",
    "त्रिकोण",
    "विरोध",
    "अर्ध-षष्ठांश",
    "क्विन्कन्क्स",
    "अर्ध-चतुर्थांश",
    "सेस्कि-चतुर्थांश",
    "पंचमांश",
    "द्वि-पंचमांश",
];

/// Sanskrit names in Devanagari script.
static ASPECTS_SA: &[&str] = &[
    "युतिः",
    "सप्तमदृष्टिः",
    "पञ्चमदृष्टिः",
    "नवमदृष्टिः",
    "तृतीयदृष्टिः",
    "दशमदृष्टिः",
    "चतुर्थदृष्टिः",
    "अष्टमदृष्टिः",
    "द्वादशदृष्टिः",
    "षडष्टकम्",
    "केन्द्रम्",
];

static ASPECTS_TA: &[&str] = &[
    "சேர்க்கை",
    "ஷஷ்டி",
    "சதுரம்",
    "திரிகோணம்",
    "எதிர்ப்பு",
    "அர்த்த-ஷஷ்டி",
    "குயின்கன்க்ஸ்",
    "அர்த்த-சதுரம்",
    "செஸ்கிகுவாட்ரேட்",
    "குவிண்டைல்",
    "பை-குவிண்டைல்",
];

static ASPECTS_TE: &[&str] = &[
    "యుతి",
    "షష్ఠాంశ",
    "చతుర్థాంశ",
    "త్రికోణ",
    "విరోధ",
    "అర్థ-షష్ఠాంశ",
    "క్వింకన్క్స్",
    "అర్థ-చతుర్థాంశ",
    "సెస్కిక్వాడ్రేట్",
    "క్వింటైల్",
    "బై-క్వింటైల్",
];

static ASPECTS_KN: &[&str] = &[
    "ಯುತಿ",
    "ಷಷ್ಠಾಂಶ",
    "ಚತುರ್ಥಾಂಶ",
    "ತ್ರಿಕೋಣ",
    "ವಿರೋಧ",
    "ಅರ್ಧ-ಷಷ್ಠಾಂಶ",
    "ಕ್ವಿಂಕನ್ಕ್ಸ್",
    "ಅರ್ಧ-ಚತುರ್ಥಾಂಶ",
    "ಸೆಸ್ಕಿಕ್ವಾಡ್ರೇಟ್",
    "ಕ್ವಿಂಟೈಲ್",
    "ಬೈ-ಕ್ವಿಂಟೈಲ್",
];

static ASPECTS_BN: &[&str] = &[
    "যুতি",
    "ষষ্ঠাংশ",
    "চতুর্থাংশ",
    "ত্রিকোণ",
    "বিরোধ",
    "অর্ধ-ষষ্ঠাংশ",
    "কুইনকাংক্স",
    "অর্ধ-চতুর্থাংশ",
    "সেস্কিকুয়াড্রেট",
    "কুইনটাইল",
    "বাই-কুইনটাইল",
];
