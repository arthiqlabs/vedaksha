// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Planetary combustion detection.
//!
//! Source: BPHS Ch.7 vv.28-29.

use crate::yoga::YogaPlanet;

/// Combustion state of a planet relative to the Sun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombustionState {
    /// Planet is not combust, or planet is Sun / Rahu / Ketu.
    None,
    /// Separation from Sun is less than the combustion orb. Source: BPHS Ch.7 vv.28-29.
    Combust,
    /// Separation from Sun is less than one-third of the combustion orb.
    /// (Modern convention — not stated literally in BPHS Ch.7.)
    DeeplyCombust,
}

/// Combustion orb in degrees. Returns `None` for Sun, Rahu, Ketu (never combust).
///
/// Source: BPHS Ch.7 vv.28-29.
fn orb(planet: YogaPlanet, is_retrograde: bool) -> Option<f64> {
    match planet {
        YogaPlanet::Moon => Some(12.0),
        YogaPlanet::Mars => Some(if is_retrograde { 8.0 } else { 17.0 }),
        YogaPlanet::Mercury => Some(if is_retrograde { 12.0 } else { 14.0 }),
        YogaPlanet::Jupiter => Some(11.0),
        YogaPlanet::Venus => Some(if is_retrograde { 8.0 } else { 10.0 }),
        YogaPlanet::Saturn => Some(16.0),
        YogaPlanet::Sun | YogaPlanet::Rahu | YogaPlanet::Ketu => None,
    }
}

/// Shortest arc between two longitudes (0–180°).
fn angular_separation(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs() % 360.0;
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// Returns the combustion state of `planet` relative to the Sun.
///
/// Source: BPHS Ch.7 vv.28-29.
/// Deep combustion threshold (orb/3) is a modern convention.
#[must_use]
pub fn combustion_state(
    planet: YogaPlanet,
    planet_lon: f64,
    sun_lon: f64,
    is_retrograde: bool,
) -> CombustionState {
    let Some(threshold) = orb(planet, is_retrograde) else {
        return CombustionState::None;
    };
    let sep = angular_separation(planet_lon, sun_lon);
    if sep < threshold / 3.0 {
        CombustionState::DeeplyCombust
    } else if sep < threshold {
        CombustionState::Combust
    } else {
        CombustionState::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_is_never_combust() {
        assert_eq!(
            combustion_state(YogaPlanet::Sun, 0.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn rahu_ketu_never_combust() {
        assert_eq!(
            combustion_state(YogaPlanet::Rahu, 5.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(YogaPlanet::Ketu, 5.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn moon_orb_boundary_is_combust() {
        assert_eq!(
            combustion_state(YogaPlanet::Moon, 11.9, 0.0, false),
            CombustionState::Combust
        );
    }

    #[test]
    fn moon_exactly_at_orb_is_not_combust() {
        assert_eq!(
            combustion_state(YogaPlanet::Moon, 12.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn moon_deep_combustion_below_4_degrees() {
        assert_eq!(
            combustion_state(YogaPlanet::Moon, 3.9, 0.0, false),
            CombustionState::DeeplyCombust
        );
    }

    #[test]
    fn moon_at_deep_threshold_is_combust_not_deep() {
        assert_eq!(
            combustion_state(YogaPlanet::Moon, 4.0, 0.0, false),
            CombustionState::Combust
        );
    }

    #[test]
    fn mars_direct_orb_17() {
        assert_eq!(
            combustion_state(YogaPlanet::Mars, 16.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Mars, 17.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn mars_retrograde_orb_8() {
        assert_eq!(
            combustion_state(YogaPlanet::Mars, 7.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Mars, 8.0, 0.0, true),
            CombustionState::None
        );
    }

    #[test]
    fn mercury_direct_orb_14() {
        assert_eq!(
            combustion_state(YogaPlanet::Mercury, 13.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Mercury, 14.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn mercury_retrograde_orb_12() {
        assert_eq!(
            combustion_state(YogaPlanet::Mercury, 11.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Mercury, 12.0, 0.0, true),
            CombustionState::None
        );
    }

    #[test]
    fn jupiter_orb_11_same_direct_retrograde() {
        assert_eq!(
            combustion_state(YogaPlanet::Jupiter, 10.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Jupiter, 11.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(YogaPlanet::Jupiter, 10.9, 0.0, true),
            CombustionState::Combust
        );
    }

    #[test]
    fn venus_direct_orb_10() {
        assert_eq!(
            combustion_state(YogaPlanet::Venus, 9.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Venus, 10.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn venus_retrograde_orb_8() {
        assert_eq!(
            combustion_state(YogaPlanet::Venus, 7.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Venus, 8.0, 0.0, true),
            CombustionState::None
        );
    }

    #[test]
    fn saturn_orb_16_same_direct_retrograde() {
        assert_eq!(
            combustion_state(YogaPlanet::Saturn, 15.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(YogaPlanet::Saturn, 16.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(YogaPlanet::Saturn, 15.9, 0.0, true),
            CombustionState::Combust
        );
    }

    #[test]
    fn angular_separation_wraps_across_360() {
        // Planet at 355°, Sun at 5° → sep = 10° (< Moon's 12°) → Combust
        assert_eq!(
            combustion_state(YogaPlanet::Moon, 355.0, 5.0, false),
            CombustionState::Combust
        );
    }
}
