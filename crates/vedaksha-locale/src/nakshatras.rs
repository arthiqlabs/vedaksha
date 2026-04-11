// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized nakshatra (lunar mansion) names.
//!
//! Index mapping (0-based, 0–26):
//!  0  Ashwini        9  Magha         18  Moola
//!  1  Bharani        10 Purva Phalguni 19  Purva Ashadha
//!  2  Krittika       11 Uttara Phalguni 20 Uttara Ashadha
//!  3  Rohini         12 Hasta          21 Shravana
//!  4  Mrigashira     13 Chitra         22 Dhanishta
//!  5  Ardra          14 Swati          23 Shatabhisha
//!  6  Punarvasu      15 Vishakha       24 Purva Bhadrapada
//!  7  Pushya         16 Anuradha       25 Uttara Bhadrapada
//!  8  Ashlesha       17 Jyeshtha       26 Revati
//!
//! Sources: BPHS Ch. 3-6, B.V. Raman "A Manual of Hindu Astrology".

use crate::Language;

/// Number of nakshatras.
pub const NAKSHATRA_COUNT: usize = 27;

/// Get the localized name of a nakshatra.
///
/// # Panics
///
/// Panics in debug builds if `index >= NAKSHATRA_COUNT`.
#[must_use]
pub fn nakshatra_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => NAKSHATRAS_EN[index],
        Language::Hindi => NAKSHATRAS_HI[index],
        Language::Sanskrit => NAKSHATRAS_SA[index],
        Language::Tamil => NAKSHATRAS_TA[index],
        Language::Telugu => NAKSHATRAS_TE[index],
        Language::Kannada => NAKSHATRAS_KN[index],
        Language::Bengali => NAKSHATRAS_BN[index],
    }
}

static NAKSHATRAS_EN: &[&str] = &[
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Moola",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

static NAKSHATRAS_HI: &[&str] = &[
    "अश्विनी",
    "भरणी",
    "कृत्तिका",
    "रोहिणी",
    "मृगशिरा",
    "आर्द्रा",
    "पुनर्वसु",
    "पुष्य",
    "आश्लेषा",
    "मघा",
    "पूर्व फाल्गुनी",
    "उत्तर फाल्गुनी",
    "हस्त",
    "चित्रा",
    "स्वाती",
    "विशाखा",
    "अनुराधा",
    "ज्येष्ठा",
    "मूल",
    "पूर्वाषाढ़ा",
    "उत्तराषाढ़ा",
    "श्रवण",
    "धनिष्ठा",
    "शतभिषा",
    "पूर्व भाद्रपद",
    "उत्तर भाद्रपद",
    "रेवती",
];

/// Sanskrit names in IAST transliteration (BPHS Ch. 3).
static NAKSHATRAS_SA: &[&str] = &[
    "Aśvinī",
    "Bharaṇī",
    "Kṛttikā",
    "Rohiṇī",
    "Mṛgaśirā",
    "Ārdrā",
    "Punarvasu",
    "Puṣya",
    "Āśleṣā",
    "Maghā",
    "Pūrva Phālgunī",
    "Uttara Phālgunī",
    "Hasta",
    "Citrā",
    "Svātī",
    "Viśākhā",
    "Anurādhā",
    "Jyeṣṭhā",
    "Mūla",
    "Pūrvāṣāḍhā",
    "Uttarāṣāḍhā",
    "Śravaṇa",
    "Dhaniṣṭhā",
    "Śatabhiṣā",
    "Pūrva Bhādrapadā",
    "Uttara Bhādrapadā",
    "Revatī",
];

static NAKSHATRAS_TA: &[&str] = &[
    "அஸ்வினி",
    "பரணி",
    "கார்த்திகை",
    "ரோகிணி",
    "மிருகசீரிடம்",
    "திருவாதிரை",
    "புனர்பூசம்",
    "பூசம்",
    "ஆயில்யம்",
    "மகம்",
    "பூரம்",
    "உத்திரம்",
    "அஸ்தம்",
    "சித்திரை",
    "சுவாதி",
    "விசாகம்",
    "அனுஷம்",
    "கேட்டை",
    "மூலம்",
    "பூராடம்",
    "உத்திராடம்",
    "திருவோணம்",
    "அவிட்டம்",
    "சதயம்",
    "பூரட்டாதி",
    "உத்திரட்டாதி",
    "ரேவதி",
];

static NAKSHATRAS_TE: &[&str] = &[
    "అశ్వని",
    "భరణి",
    "కృత్తిక",
    "రోహిణి",
    "మృగశిర",
    "ఆర్ద్ర",
    "పునర్వసు",
    "పుష్యమి",
    "ఆశ్లేష",
    "మఘ",
    "పూర్వ ఫల్గుణి",
    "ఉత్తర ఫల్గుణి",
    "హస్త",
    "చిత్త",
    "స్వాతి",
    "విశాఖ",
    "అనూరాధ",
    "జ్యేష్ఠ",
    "మూల",
    "పూర్వాషాఢ",
    "ఉత్తరాషాఢ",
    "శ్రవణం",
    "ధనిష్ఠ",
    "శతభిష",
    "పూర్వ భాద్రపద",
    "ఉత్తర భాద్రపద",
    "రేవతి",
];

static NAKSHATRAS_KN: &[&str] = &[
    "ಅಶ್ವಿನಿ",
    "ಭರಣಿ",
    "ಕೃತ್ತಿಕಾ",
    "ರೋಹಿಣಿ",
    "ಮೃಗಶಿರ",
    "ಆರ್ದ್ರ",
    "ಪುನರ್ವಸು",
    "ಪುಷ್ಯ",
    "ಆಶ್ಲೇಷ",
    "ಮಖ",
    "ಪೂರ್ವ ಫಲ್ಗುಣಿ",
    "ಉತ್ತರ ಫಲ್ಗುಣಿ",
    "ಹಸ್ತ",
    "ಚಿತ್ರ",
    "ಸ್ವಾತಿ",
    "ವಿಶಾಖ",
    "ಅನುರಾಧ",
    "ಜ್ಯೇಷ್ಠ",
    "ಮೂಲ",
    "ಪೂರ್ವಾಷಾಢ",
    "ಉತ್ತರಾಷಾಢ",
    "ಶ್ರವಣ",
    "ಧನಿಷ್ಠ",
    "ಶತಭಿಷ",
    "ಪೂರ್ವ ಭಾದ್ರಪದ",
    "ಉತ್ತರ ಭಾದ್ರಪದ",
    "ರೇವತಿ",
];

static NAKSHATRAS_BN: &[&str] = &[
    "অশ্বিনী",
    "ভরণী",
    "কৃত্তিকা",
    "রোহিণী",
    "মৃগশিরা",
    "আর্দ্রা",
    "পুনর্বসু",
    "পুষ্য",
    "আশ্লেষা",
    "মঘা",
    "পূর্ব ফাল্গুনী",
    "উত্তর ফাল্গুনী",
    "হস্ত",
    "চিত্রা",
    "স্বাতী",
    "বিশাখা",
    "অনুরাধা",
    "জ্যেষ্ঠা",
    "মূল",
    "পূর্বাষাঢ়া",
    "উত্তরাষাঢ়া",
    "শ্রবণ",
    "ধনিষ্ঠা",
    "শতভিষা",
    "পূর্ব ভাদ্রপদ",
    "উত্তর ভাদ্রপদ",
    "রেবতী",
];
