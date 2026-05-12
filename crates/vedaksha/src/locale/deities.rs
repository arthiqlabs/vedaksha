// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized nakshatra deity names.
//!
//! Index mapping (0-based, 0–26) follows nakshatra order:
//!  0  Ashwini Kumaras  9  Pitrs           18  Nirrti
//!  1  Yama             10 Bhaga           19  Apas
//!  2  Agni             11 Aryaman         20  Vishvedeva
//!  3  Brahma           12 Savitar         21  Vishnu
//!  4  Soma             13 Tvashtar        22  Vasu
//!  5  Rudra            14 Vayu            23  Varuna
//!  6  Aditi            15 Indragni        24  Ajaikapada
//!  7  Brihaspati       16 Mitra           25  Ahirbudhnya
//!  8  Sarpas           17 Indra           26  Pushan
//!
//! Sources: BPHS Ch. 3-6, B.V. Raman "A Manual of Hindu Astrology".

use crate::Language;

/// Number of nakshatra deities.
pub const DEITY_COUNT: usize = 27;

/// Get the localized name of a nakshatra deity.
///
/// # Panics
///
/// Panics in debug builds if `index >= DEITY_COUNT`.
#[must_use]
pub fn deity_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => DEITIES_EN[index],
        Language::Hindi => DEITIES_HI[index],
        Language::Sanskrit => DEITIES_SA[index],
        Language::Tamil => DEITIES_TA[index],
        Language::Telugu => DEITIES_TE[index],
        Language::Kannada => DEITIES_KN[index],
        Language::Bengali => DEITIES_BN[index],
    }
}

static DEITIES_EN: &[&str] = &[
    "Ashwini Kumaras",
    "Yama",
    "Agni",
    "Brahma",
    "Soma",
    "Rudra",
    "Aditi",
    "Brihaspati",
    "Sarpas",
    "Pitrs",
    "Bhaga",
    "Aryaman",
    "Savitar",
    "Tvashtar",
    "Vayu",
    "Indragni",
    "Mitra",
    "Indra",
    "Nirrti",
    "Apas",
    "Vishvedeva",
    "Vishnu",
    "Vasu",
    "Varuna",
    "Ajaikapada",
    "Ahirbudhnya",
    "Pushan",
];

static DEITIES_HI: &[&str] = &[
    "अश्विनी कुमार",
    "यम",
    "अग्नि",
    "ब्रह्मा",
    "सोम",
    "रुद्र",
    "अदिति",
    "बृहस्पति",
    "सर्प",
    "पितर",
    "भग",
    "अर्यमा",
    "सवितार",
    "त्वष्टा",
    "वायु",
    "इन्द्राग्नि",
    "मित्र",
    "इन्द्र",
    "निर्ऋति",
    "अपस्",
    "विश्वेदेव",
    "विष्णु",
    "वसु",
    "वरुण",
    "अजैकपाद",
    "अहिर्बुध्न्य",
    "पूषन",
];

/// Sanskrit names in Devanagari script (BPHS Ch. 3).
static DEITIES_SA: &[&str] = &[
    "अश्विनौ",
    "यमः",
    "अग्निः",
    "ब्रह्मा",
    "सोमः",
    "रुद्रः",
    "अदितिः",
    "बृहस्पतिः",
    "सर्पाः",
    "पितरः",
    "भगः",
    "अर्यमा",
    "सवितृ",
    "त्वष्टृ",
    "वायुः",
    "इन्द्राग्नी",
    "मित्रः",
    "इन्द्रः",
    "निरृतिः",
    "आपः",
    "विश्वेदेवाः",
    "विष्णुः",
    "वसवः",
    "वरुणः",
    "अजैकपात्",
    "अहिर्बुध्न्यः",
    "पूषन्",
];

static DEITIES_TA: &[&str] = &[
    "அஸ்வினி குமாரர்",
    "யமன்",
    "அக்னி",
    "பிரம்மா",
    "சோமன்",
    "ருத்ரன்",
    "அதிதி",
    "பிரகஸ்பதி",
    "சர்ப்பம்",
    "பித்ருக்கள்",
    "பகன்",
    "அர்யமன்",
    "சவிதா",
    "த்வஷ்டா",
    "வாயு",
    "இந்திராக்னி",
    "மித்ரன்",
    "இந்திரன்",
    "நிர்ருதி",
    "அபஸ்",
    "விஸ்வேதேவர்",
    "விஷ்ணு",
    "வசு",
    "வருணன்",
    "அஜைகபாதன்",
    "அஹிர்புத்னியன்",
    "பூஷன்",
];

static DEITIES_TE: &[&str] = &[
    "అశ్వినీ కుమారులు",
    "యముడు",
    "అగ్ని",
    "బ్రహ్మ",
    "సోముడు",
    "రుద్రుడు",
    "అదితి",
    "బృహస్పతి",
    "సర్పములు",
    "పితరులు",
    "భగుడు",
    "అర్యముడు",
    "సవిత",
    "త్వష్ట",
    "వాయువు",
    "ఇంద్రాగ్ని",
    "మిత్రుడు",
    "ఇంద్రుడు",
    "నిరృతి",
    "ఆపః",
    "విశ్వేదేవులు",
    "విష్ణువు",
    "వసువు",
    "వరుణుడు",
    "అజైకపాదుడు",
    "అహిర్బుధ్న్యుడు",
    "పూషణుడు",
];

static DEITIES_KN: &[&str] = &[
    "ಅಶ್ವಿನೀ ಕುಮಾರರು",
    "ಯಮ",
    "ಅಗ್ನಿ",
    "ಬ್ರಹ್ಮ",
    "ಸೋಮ",
    "ರುದ್ರ",
    "ಅದಿತಿ",
    "ಬೃಹಸ್ಪತಿ",
    "ಸರ್ಪಗಳು",
    "ಪಿತೃಗಳು",
    "ಭಗ",
    "ಅರ್ಯಮನ್",
    "ಸವಿತೃ",
    "ತ್ವಷ್ಟೃ",
    "ವಾಯು",
    "ಇಂದ್ರಾಗ್ನಿ",
    "ಮಿತ್ರ",
    "ಇಂದ್ರ",
    "ನಿರೃತಿ",
    "ಆಪಃ",
    "ವಿಶ್ವೇದೇವರು",
    "ವಿಷ್ಣು",
    "ವಸು",
    "ವರುಣ",
    "ಅಜೈಕಪಾದ",
    "ಅಹಿರ್ಬುಧ್ನ್ಯ",
    "ಪೂಷನ್",
];

static DEITIES_BN: &[&str] = &[
    "অশ্বিনী কুমার",
    "যম",
    "অগ্নি",
    "ব্রহ্মা",
    "সোম",
    "রুদ্র",
    "অদিতি",
    "বৃহস্পতি",
    "সর্প",
    "পিতৃগণ",
    "ভগ",
    "অর্যমা",
    "সবিতৃ",
    "ত্বষ্টা",
    "বায়ু",
    "ইন্দ্রাগ্নি",
    "মিত্র",
    "ইন্দ্র",
    "নির্ঋতি",
    "অপঃ",
    "বিশ্বেদেব",
    "বিষ্ণু",
    "বসু",
    "বরুণ",
    "অজৈকপাদ",
    "অহির্বুধ্ন্য",
    "পূষন",
];
