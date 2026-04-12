// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Panchanga Yoga and Karana — the two remaining limbs of the panchanga.
//!
//! **Panchanga Yoga** (astronomical yoga): one of 27 yogas determined by the
//! sum of the sidereal longitudes of the Sun and Moon, each spanning 360°/27
//! = 13°20'.
//!
//! **Karana**: half of a tithi. There are 60 karanas in a lunar month — 4 fixed
//! and 7 rotating (cycling 8 times through the 56 non-fixed positions).
//!
//! Source: BPHS Ch. 3; Surya Siddhanta; Muhurtha Chintamani.

use vedaksha_math::angle::normalize_degrees;

/// Span of each panchanga yoga in degrees (360 / 27 = 13°20').
const YOGA_SPAN: f64 = 360.0 / 27.0;

/// Span of each karana in degrees (6°).
const KARANA_SPAN: f64 = 6.0;

/// The 27 Panchanga Yoga names in traditional order.
///
/// Source: BPHS Ch. 3; Surya Siddhanta.
const YOGA_NAMES: [&str; 27] = [
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

/// The 7 rotating karana names, cycling through indices 1–56.
///
/// Source: Surya Siddhanta; Muhurtha Chintamani.
const ROTATING_KARANA_NAMES: [&str; 7] = [
    "Bava", "Balava", "Kaulava", "Taitila", "Garaja", "Vanija", "Vishti",
];

/// The 4 fixed karana names, at indices 0, 57, 58, 59.
///
/// Source: Surya Siddhanta; Muhurtha Chintamani.
const FIXED_KARANA_NAMES: [&str; 4] = ["Kimstughna", "Shakuni", "Chatushpada", "Naga"];

// ---------------------------------------------------------------------------
// Panchanga Yoga
// ---------------------------------------------------------------------------

/// One of the 27 astronomical yogas of the panchanga.
///
/// Determined by `floor(normalize(sun_lon + moon_lon) / (360/27))`.
///
/// Source: BPHS Ch. 3; Surya Siddhanta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanchangaYoga {
    /// 0-based index (0 = Vishkambha … 26 = Vaidhriti).
    pub index: u8,
    /// Sanskrit transliterated name.
    pub name: &'static str,
    /// Degrees remaining until the next yoga boundary.
    pub remaining_degrees: f64,
}

/// Compute the Panchanga Yoga from sidereal longitudes of Sun and Moon.
///
/// Formula: `yoga_index = floor(normalize(sun + moon) / (360/27))`
///
/// Source: BPHS Ch. 3; Surya Siddhanta.
#[must_use]
pub fn compute_panchanga_yoga(sun_sidereal_lon: f64, moon_sidereal_lon: f64) -> PanchangaYoga {
    let sum = normalize_degrees(sun_sidereal_lon + moon_sidereal_lon);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = (sum / YOGA_SPAN).floor() as u8 % 27;
    let boundary = f64::from(index + 1) * YOGA_SPAN;
    let remaining = normalize_degrees(boundary - sum);
    PanchangaYoga {
        index,
        name: YOGA_NAMES[index as usize],
        remaining_degrees: remaining,
    }
}

// ---------------------------------------------------------------------------
// Karana
// ---------------------------------------------------------------------------

/// One of the 60 karanas (half-tithis) of the panchanga.
///
/// 4 fixed karanas appear once each (indices 0, 57, 58, 59).
/// 7 rotating karanas cycle through indices 1–56.
///
/// Source: Surya Siddhanta; Muhurtha Chintamani.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Karana {
    /// 0-based karana index (0–59).
    pub index: u8,
    /// Sanskrit transliterated name.
    pub name: &'static str,
    /// Whether this is a fixed (non-rotating) karana.
    pub is_fixed: bool,
}

/// Compute the Karana from sidereal longitudes of Moon and Sun.
///
/// Formula: `karana_index = floor(normalize(moon - sun) / 6.0)`
///
/// Source: Surya Siddhanta; Muhurtha Chintamani.
#[must_use]
pub fn compute_karana(moon_lon: f64, sun_lon: f64) -> Karana {
    let diff = normalize_degrees(moon_lon - sun_lon);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = (diff / KARANA_SPAN).floor() as u8 % 60;

    let (name, is_fixed) = match index {
        0 => (FIXED_KARANA_NAMES[0], true),
        57 => (FIXED_KARANA_NAMES[1], true),
        58 => (FIXED_KARANA_NAMES[2], true),
        59 => (FIXED_KARANA_NAMES[3], true),
        i => {
            // Rotating karanas occupy indices 1–56; cycle of 7.
            let rotating_pos = (i - 1) as usize % 7;
            (ROTATING_KARANA_NAMES[rotating_pos], false)
        }
    };

    Karana {
        index,
        name,
        is_fixed,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- Panchanga Yoga ---

    #[test]
    fn yoga_at_zero_is_vishkambha() {
        let y = compute_panchanga_yoga(0.0, 0.0);
        assert_eq!(y.index, 0);
        assert_eq!(y.name, "Vishkambha");
    }

    #[test]
    fn yoga_wraps_at_360() {
        // Sun=200, Moon=200 → sum=400 → normalize=40 → 40/13.333=3 → Saubhagya
        let y = compute_panchanga_yoga(200.0, 200.0);
        assert_eq!(y.index, 3);
        assert_eq!(y.name, "Saubhagya");
    }

    #[test]
    fn yoga_remaining_positive() {
        // At sum=0 the remaining should be one full yoga span (~13.333°).
        let y = compute_panchanga_yoga(0.0, 0.0);
        assert!(y.remaining_degrees > 0.0);
        assert!((y.remaining_degrees - YOGA_SPAN).abs() < EPS);
    }

    #[test]
    fn all_27_yogas_reachable() {
        let mut seen = [false; 27];
        for i in 0u8..27 {
            // Place the sum right in the middle of each yoga span.
            let mid = f64::from(i) * YOGA_SPAN + YOGA_SPAN / 2.0;
            let y = compute_panchanga_yoga(mid, 0.0);
            seen[y.index as usize] = true;
            assert_eq!(y.name, YOGA_NAMES[i as usize]);
        }
        assert!(seen.iter().all(|&s| s), "not all 27 yogas were reached");
    }

    // --- Karana ---

    #[test]
    fn karana_at_zero_is_kimstughna() {
        let k = compute_karana(0.0, 0.0);
        assert_eq!(k.index, 0);
        assert_eq!(k.name, "Kimstughna");
        assert!(k.is_fixed);
    }

    #[test]
    fn karana_rotating_cycle() {
        // diff=6° → index 1 → first rotating = Bava
        let k = compute_karana(6.0, 0.0);
        assert_eq!(k.index, 1);
        assert_eq!(k.name, "Bava");
        assert!(!k.is_fixed);
    }

    #[test]
    fn karana_vishti_is_seventh_rotating() {
        // Vishti is the 7th rotating karana. Index 7 → rotating_pos = (7-1)%7 = 6 → Vishti.
        let k = compute_karana(42.0, 0.0); // 42/6 = 7
        assert_eq!(k.index, 7);
        assert_eq!(k.name, "Vishti");
        assert!(!k.is_fixed);
    }

    #[test]
    fn karana_shakuni_is_fixed() {
        // diff=342° → 342/6 = 57 → Shakuni (fixed)
        let k = compute_karana(342.0, 0.0);
        assert_eq!(k.index, 57);
        assert_eq!(k.name, "Shakuni");
        assert!(k.is_fixed);
    }
}
