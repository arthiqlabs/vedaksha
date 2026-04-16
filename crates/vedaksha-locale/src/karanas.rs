// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized karana (half-tithi) names.
//!
//! Index mapping (0-based, 0–10):
//!  0  Bava         4  Garaja       8  Shakuni
//!  1  Balava       5  Vanija       9  Chatushpada
//!  2  Kaulava      6  Vishti      10  Naga
//!  3  Taitila      7  Kimstughna
//!
//! The first 7 (Bava–Vishti) are "chara" (movable) karanas that repeat.
//! The last 4 (Kimstughna–Naga) are "sthira" (fixed) karanas.
//!
//! Sources: BPHS, Surya Siddhanta.

use crate::Language;

/// Number of unique karanas.
pub const KARANA_COUNT: usize = 11;

/// Get the localized name of a karana.
///
/// # Panics
///
/// Panics in debug builds if `index >= KARANA_COUNT`.
#[must_use]
pub fn karana_name(index: usize, lang: Language) -> &'static str {
    match lang {
        Language::English => KARANAS_EN[index],
        Language::Hindi => KARANAS_HI[index],
        Language::Sanskrit => KARANAS_SA[index],
        Language::Tamil => KARANAS_TA[index],
        Language::Telugu => KARANAS_TE[index],
        Language::Kannada => KARANAS_KN[index],
        Language::Bengali => KARANAS_BN[index],
    }
}

static KARANAS_EN: &[&str] = &[
    "Bava",
    "Balava",
    "Kaulava",
    "Taitila",
    "Garaja",
    "Vanija",
    "Vishti",
    "Kimstughna",
    "Shakuni",
    "Chatushpada",
    "Naga",
];

static KARANAS_HI: &[&str] = &[
    "बव",
    "बालव",
    "कौलव",
    "तैतिल",
    "गरज",
    "वणिज",
    "विष्टि",
    "किंस्तुघ्न",
    "शकुनि",
    "चतुष्पद",
    "नाग",
];

/// Sanskrit names in Devanagari script.
static KARANAS_SA: &[&str] = &[
    "बवः",
    "बालवः",
    "कौलवः",
    "तैतिलः",
    "गरः",
    "वणिज्",
    "विष्टिः",
    "शकुनिः",
    "चतुष्पात्",
    "नागः",
    "किंस्तुघ्नः",
];

static KARANAS_TA: &[&str] = &[
    "பவம்",
    "பாலவம்",
    "கௌலவம்",
    "தைதிலம்",
    "கரசம்",
    "வணிசம்",
    "விஷ்டி",
    "கிம்ஸ்துக்னம்",
    "சகுனி",
    "சதுஷ்பதம்",
    "நாகம்",
];

static KARANAS_TE: &[&str] = &[
    "బవ",
    "బాలవ",
    "కౌలవ",
    "తైతిల",
    "గరజ",
    "వణిజ",
    "విష్టి",
    "కింస్తుఘ్న",
    "శకుని",
    "చతుష్పద",
    "నాగ",
];

static KARANAS_KN: &[&str] = &[
    "ಬವ",
    "ಬಾಲವ",
    "ಕೌಲವ",
    "ತೈತಿಲ",
    "ಗರಜ",
    "ವಣಿಜ",
    "ವಿಷ್ಟಿ",
    "ಕಿಂಸ್ತುಘ್ನ",
    "ಶಕುನಿ",
    "ಚತುಷ್ಪದ",
    "ನಾಗ",
];

static KARANAS_BN: &[&str] = &[
    "বব",
    "বালব",
    "কৌলব",
    "তৈতিল",
    "গরজ",
    "বণিজ",
    "বিষ্টি",
    "কিংস্তুঘ্ন",
    "শকুনি",
    "চতুষ্পদ",
    "নাগ",
];
