// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Localized Vimshottari dasha lord names.
//!
//! The 9 Vimshottari mahādasha lords in their canonical order (BPHS Ch. 46):
//! 0 = Ketu, 1 = Venus, 2 = Sun, 3 = Moon, 4 = Mars,
//! 5 = Rahu, 6 = Jupiter, 7 = Saturn, 8 = Mercury
//!
//! This module delegates to `planets::planet_name` using the mapping from
//! dasha-lord position to planet index.

use crate::{Language, planets};

/// Number of dasha lords in the Vimshottari system.
pub const DASHA_LORD_COUNT: usize = 9;

/// Mapping from Vimshottari dasha order to `planets` module index.
///
/// Dasha order (BPHS): Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury
/// Planet indices:       8,    5,     0,   1,    2,    7,    4,       6,       3
const DASHA_TO_PLANET: &[usize] = &[8, 5, 0, 1, 2, 7, 4, 6, 3];

/// Get the localized name of a Vimshottari dasha lord.
///
/// `index` is the position in the Vimshottari sequence (0 = Ketu … 8 = Mercury).
///
/// # Panics
///
/// Panics in debug builds if `index >= DASHA_LORD_COUNT`.
#[must_use]
pub fn dasha_lord_name(index: usize, lang: Language) -> &'static str {
    planets::planet_name(DASHA_TO_PLANET[index], lang)
}
