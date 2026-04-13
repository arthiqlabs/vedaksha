// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized panchanga yoga (Sun–Moon combination) names.
//!
//! Index mapping (0-based, 0–26):
//!  0  Vishkambha     9  Ganda          18  Parigha
//!  1  Priti          10 Vriddhi        19  Shiva
//!  2  Ayushman       11 Dhruva         20  Siddha
//!  3  Saubhagya      12 Vyaghata       21  Sadhya
//!  4  Shobhana       13 Harshana       22  Shubha
//!  5  Atiganda       14 Vajra          23  Shukla
//!  6  Sukarma        15 Siddhi         24  Brahma
//!  7  Dhriti         16 Vyatipata      25  Indra
//!  8  Shoola         17 Variyan        26  Vaidhriti
//!
//! Sources: BPHS, Surya Siddhanta.

use crate::Language;

/// Number of panchanga yogas.
pub const PANCHANGA_YOGA_COUNT: usize = 27;

/// Get the localized name of a panchanga yoga.
///
/// # Panics
///
/// Panics in debug builds if `index >= PANCHANGA_YOGA_COUNT`.
#[must_use]
pub fn panchanga_yoga_name(index: usize, lang: Language) -> &'static str {
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
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarma",
    "Dhriti",
    "Shoola",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyan",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

static YOGAS_HI: &[&str] = &[
    "विष्कम्भ",
    "प्रीति",
    "आयुष्मान",
    "सौभाग्य",
    "शोभन",
    "अतिगण्ड",
    "सुकर्मा",
    "धृति",
    "शूल",
    "गण्ड",
    "वृद्धि",
    "ध्रुव",
    "व्याघात",
    "हर्षण",
    "वज्र",
    "सिद्धि",
    "व्यतीपात",
    "वरीयान",
    "परिघ",
    "शिव",
    "सिद्ध",
    "साध्य",
    "शुभ",
    "शुक्ल",
    "ब्रह्म",
    "इन्द्र",
    "वैधृति",
];

/// Sanskrit names in IAST transliteration.
static YOGAS_SA: &[&str] = &[
    "Viṣkambha",
    "Prīti",
    "Āyuṣmān",
    "Saubhāgya",
    "Śobhana",
    "Atigaṇḍa",
    "Sukarman",
    "Dhṛti",
    "Śūla",
    "Gaṇḍa",
    "Vṛddhi",
    "Dhruva",
    "Vyāghāta",
    "Harṣaṇa",
    "Vajra",
    "Siddhi",
    "Vyatīpāta",
    "Varīyān",
    "Parigha",
    "Śiva",
    "Siddha",
    "Sādhya",
    "Śubha",
    "Śukla",
    "Brahmā",
    "Indra",
    "Vaidhṛti",
];

static YOGAS_TA: &[&str] = &[
    "விஷ்கம்பம்",
    "பிரீதி",
    "ஆயுஷ்மான்",
    "சௌபாக்யம்",
    "சோபனம்",
    "அதிகண்டம்",
    "சுகர்மம்",
    "திருதி",
    "சூலம்",
    "கண்டம்",
    "விருத்தி",
    "துருவம்",
    "வியாகாதம்",
    "ஹர்ஷணம்",
    "வஜ்ரம்",
    "சித்தி",
    "வியதீபாதம்",
    "வரீயான்",
    "பரிகம்",
    "சிவம்",
    "சித்தம்",
    "சாத்தியம்",
    "சுபம்",
    "சுக்லம்",
    "பிரம்மம்",
    "இந்திரம்",
    "வைதிருதி",
];

static YOGAS_TE: &[&str] = &[
    "విష్కంభ",
    "ప్రీతి",
    "ఆయుష్మాన్",
    "సౌభాగ్య",
    "శోభన",
    "అతిగండ",
    "సుకర్మ",
    "ధృతి",
    "శూల",
    "గండ",
    "వృద్ధి",
    "ధ్రువ",
    "వ్యాఘాత",
    "హర్షణ",
    "వజ్ర",
    "సిద్ధి",
    "వ్యతీపాత",
    "వరీయాన్",
    "పరిఘ",
    "శివ",
    "సిద్ధ",
    "సాధ్య",
    "శుభ",
    "శుక్ల",
    "బ్రహ్మ",
    "ఇంద్ర",
    "వైధృతి",
];

static YOGAS_KN: &[&str] = &[
    "ವಿಷ್ಕಂಭ",
    "ಪ್ರೀತಿ",
    "ಆಯುಷ್ಮಾನ್",
    "ಸೌಭಾಗ್ಯ",
    "ಶೋಭನ",
    "ಅತಿಗಂಡ",
    "ಸುಕರ್ಮ",
    "ಧೃತಿ",
    "ಶೂಲ",
    "ಗಂಡ",
    "ವೃದ್ಧಿ",
    "ಧ್ರುವ",
    "ವ್ಯಾಘಾತ",
    "ಹರ್ಷಣ",
    "ವಜ್ರ",
    "ಸಿದ್ಧಿ",
    "ವ್ಯತೀಪಾತ",
    "ವರೀಯಾನ್",
    "ಪರಿಘ",
    "ಶಿವ",
    "ಸಿದ್ಧ",
    "ಸಾಧ್ಯ",
    "ಶುಭ",
    "ಶುಕ್ಲ",
    "ಬ್ರಹ್ಮ",
    "ಇಂದ್ರ",
    "ವೈಧೃತಿ",
];

static YOGAS_BN: &[&str] = &[
    "বিষ্কম্ভ",
    "প্রীতি",
    "আয়ুষ্মান",
    "সৌভাগ্য",
    "শোভন",
    "অতিগণ্ড",
    "সুকর্মা",
    "ধৃতি",
    "শূল",
    "গণ্ড",
    "বৃদ্ধি",
    "ধ্রুব",
    "ব্যাঘাত",
    "হর্ষণ",
    "বজ্র",
    "সিদ্ধি",
    "ব্যতীপাত",
    "বরীয়ান",
    "পরিঘ",
    "শিব",
    "সিদ্ধ",
    "সাধ্য",
    "শুভ",
    "শুক্ল",
    "ব্রহ্ম",
    "ইন্দ্র",
    "বৈধৃতি",
];
