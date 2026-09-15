// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Planetary combustion detection.
//!
//! # Sources
//!
//! - **Principle** — BPHS Ch.7 vv.28-29: a graha's strength is lost at
//!   conjunction with the Sun and whole at opposition. Those verses give no
//!   per-graha degrees.
//! - **Orbs** — Surya Siddhanta IX.6-8 for the planets (Jupiter 11°,
//!   Saturn 15°, Mars 17°, Venus 10° or 8° when retrograde, Mercury 14° or 12°
//!   when retrograde) and X.1 for the Moon (12°). Only Venus and Mercury carry
//!   a retrograde figure; Mars has one orb.
//!
//! The Surya Siddhanta states these as *kalamsha* — degrees of time, an arc of
//! the equator — not ecliptic separation. This module applies them to the
//! shortest ecliptic arc between the graha and the Sun, which is the ordinary
//! Jyotish reading of the rule; it is not a heliacal-visibility computation.
//!
//! # Changed in v9.2.0
//!
//! Through v9.1.x Saturn's orb was 16° and a retrograde Mars used 8°. Neither
//! figure is in the cited verses. A superior planet retrogrades near
//! opposition, so the Mars change cannot alter a real chart. For Saturn two
//! bands move, because `DeeplyCombust` is a third of the orb: between 15° and
//! 16° from the Sun now reads `None` where it read `Combust`, and between 5°
//! and 5⅓° now reads `Combust` where it read `DeeplyCombust`.

use crate::graha::Graha;

/// Combustion state of a planet relative to the Sun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombustionState {
    /// Planet is not combust, or planet is Sun / Rahu / Ketu.
    None,
    /// Separation from Sun is less than the combustion orb (see the module
    /// doc for the orb sources).
    Combust,
    /// Separation from Sun is less than one-third of the combustion orb.
    /// (Modern convention — not stated literally in BPHS Ch.7.)
    DeeplyCombust,
}

/// Combustion orb in degrees — the separation from the Sun below which
/// [`combustion_state`] reports [`CombustionState::Combust`]. Returns `None` for
/// the Sun, Rahu and Ketu, which are never combust.
///
/// [`combustion_state`] reads this function and nothing else, so a caller that
/// grades combustion more finely than the three states, or sizes a window from
/// the orb, stays consistent with the engine. `DeeplyCombust` begins at a third
/// of this value.
///
/// Source: Surya Siddhanta IX.6-8 (planets) and X.1 (Moon).
#[must_use]
pub fn combustion_orb_deg(planet: Graha, is_retrograde: bool) -> Option<f64> {
    match planet {
        Graha::Moon => Some(12.0),
        Graha::Mars => Some(17.0),
        Graha::Mercury => Some(if is_retrograde { 12.0 } else { 14.0 }),
        Graha::Jupiter => Some(11.0),
        Graha::Venus => Some(if is_retrograde { 8.0 } else { 10.0 }),
        Graha::Saturn => Some(15.0),
        Graha::Sun | Graha::Rahu | Graha::Ketu => None,
    }
}

/// Shortest arc between two longitudes (0–180°).
fn angular_separation(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs() % 360.0;
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// Returns the combustion state of `planet` relative to the Sun.
///
/// Principle: BPHS Ch.7 vv.28-29. Orbs: Surya Siddhanta IX.6-8 and X.1.
/// Deep combustion threshold (orb/3) is a modern convention.
#[must_use]
pub fn combustion_state(
    planet: Graha,
    planet_lon: f64,
    sun_lon: f64,
    is_retrograde: bool,
) -> CombustionState {
    let Some(threshold) = combustion_orb_deg(planet, is_retrograde) else {
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

    /// The published orb must be the boundary `combustion_state` actually
    /// uses, for every graha in both motions: just inside the orb is combust,
    /// the orb itself is not, and deep combustion ends exactly at a third of it.
    #[test]
    fn published_orb_is_the_boundary_combustion_state_uses() {
        let grahas = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
        ];
        let mut with_orb = 0;
        for g in grahas {
            for retro in [false, true] {
                let Some(orb) = combustion_orb_deg(g, retro) else {
                    for sep in [0.0, 1.0, 5.0] {
                        assert_eq!(combustion_state(g, sep, 0.0, retro), CombustionState::None);
                    }
                    continue;
                };
                with_orb += 1;
                let at = |sep: f64| combustion_state(g, sep, 0.0, retro);
                assert_eq!(at(orb - 1e-9), CombustionState::Combust, "{g:?} r={retro}");
                assert_eq!(at(orb), CombustionState::None, "{g:?} r={retro}");
                assert_eq!(
                    at(orb / 3.0 - 1e-9),
                    CombustionState::DeeplyCombust,
                    "{g:?} r={retro}"
                );
                assert_eq!(at(orb / 3.0), CombustionState::Combust, "{g:?} r={retro}");
            }
        }
        assert_eq!(with_orb, 12, "six combustible grahas × two motions");
    }

    #[test]
    fn sun_is_never_combust() {
        assert_eq!(
            combustion_state(Graha::Sun, 0.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn rahu_ketu_never_combust() {
        assert_eq!(
            combustion_state(Graha::Rahu, 5.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(Graha::Ketu, 5.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn moon_orb_boundary_is_combust() {
        assert_eq!(
            combustion_state(Graha::Moon, 11.9, 0.0, false),
            CombustionState::Combust
        );
    }

    #[test]
    fn moon_exactly_at_orb_is_not_combust() {
        assert_eq!(
            combustion_state(Graha::Moon, 12.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn moon_deep_combustion_below_4_degrees() {
        assert_eq!(
            combustion_state(Graha::Moon, 3.9, 0.0, false),
            CombustionState::DeeplyCombust
        );
    }

    #[test]
    fn moon_at_deep_threshold_is_combust_not_deep() {
        assert_eq!(
            combustion_state(Graha::Moon, 4.0, 0.0, false),
            CombustionState::Combust
        );
    }

    #[test]
    fn mars_direct_orb_17() {
        assert_eq!(
            combustion_state(Graha::Mars, 16.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Mars, 17.0, 0.0, false),
            CombustionState::None
        );
    }

    /// Surya Siddhanta IX.6-8 gives Mars one orb. Through v9.1.x retrograde
    /// Mars used 8°, so 12° from the Sun read `None`.
    #[test]
    fn mars_retrograde_orb_is_still_17() {
        assert_eq!(
            combustion_state(Graha::Mars, 12.0, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Mars, 16.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Mars, 17.0, 0.0, true),
            CombustionState::None
        );
    }

    #[test]
    fn mercury_direct_orb_14() {
        assert_eq!(
            combustion_state(Graha::Mercury, 13.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Mercury, 14.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn mercury_retrograde_orb_12() {
        assert_eq!(
            combustion_state(Graha::Mercury, 11.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Mercury, 12.0, 0.0, true),
            CombustionState::None
        );
    }

    #[test]
    fn jupiter_orb_11_same_direct_retrograde() {
        assert_eq!(
            combustion_state(Graha::Jupiter, 10.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Jupiter, 11.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(Graha::Jupiter, 10.9, 0.0, true),
            CombustionState::Combust
        );
    }

    #[test]
    fn venus_direct_orb_10() {
        assert_eq!(
            combustion_state(Graha::Venus, 9.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Venus, 10.0, 0.0, false),
            CombustionState::None
        );
    }

    #[test]
    fn venus_retrograde_orb_8() {
        assert_eq!(
            combustion_state(Graha::Venus, 7.9, 0.0, true),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Venus, 8.0, 0.0, true),
            CombustionState::None
        );
    }

    /// Surya Siddhanta IX.6: 15°. Through v9.1.x this was 16°, so 15.5° from
    /// the Sun read `Combust`.
    #[test]
    fn saturn_orb_15_same_direct_retrograde() {
        assert_eq!(
            combustion_state(Graha::Saturn, 14.9, 0.0, false),
            CombustionState::Combust
        );
        assert_eq!(
            combustion_state(Graha::Saturn, 15.0, 0.0, false),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(Graha::Saturn, 15.5, 0.0, true),
            CombustionState::None
        );
        assert_eq!(
            combustion_state(Graha::Saturn, 14.9, 0.0, true),
            CombustionState::Combust
        );
        // Deep combustion at a third of the orb: 5°, was 5⅓°.
        assert_eq!(
            combustion_state(Graha::Saturn, 4.9, 0.0, false),
            CombustionState::DeeplyCombust
        );
        assert_eq!(
            combustion_state(Graha::Saturn, 5.2, 0.0, false),
            CombustionState::Combust
        );
    }

    #[test]
    fn angular_separation_wraps_across_360() {
        // Planet at 355°, Sun at 5° → sep = 10° (< Moon's 12°) → Combust
        assert_eq!(
            combustion_state(Graha::Moon, 355.0, 5.0, false),
            CombustionState::Combust
        );
    }
}
