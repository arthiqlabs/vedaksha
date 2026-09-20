// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Cheshta (motion states) and graha yuddha (planetary war).
//!
//! # Sources
//!
//! BPHS, the chapter on graha bala. Two honesties up front:
//!
//! 1. BPHS defines the eight states QUALITATIVELY (retrograde, stationary,
//!    slower/faster than usual, entering the previous/next sign) and assigns
//!    each a Cheshta Bala strength — Vakra 60, Anuvakra 30, Vikala 15,
//!    Manda 30, Mandatara 15, Sama 7.5, Chara 45, Atichara 30 virupas. It
//!    states NO numeric speed fractions, so any speed-band implementation —
//!    this one included — is a conventional operationalization, not a verse.
//!    The bands below are this engine's stated reading, documented so a
//!    consumer can map them differently.
//! 2. Vakra vs Anuvakra ("retrograde" vs "retrograde entering the previous
//!    sign") is a SIGN-ENTRY distinction, not a speed one: instantaneous
//!    speed alone cannot tell them apart. This module resolves clearly
//!    retrograde motion as Vakra and the near-stationary retrograde sliver
//!    as Anuvakra. A caller that tracks sign boundaries can do better.
//!
//! This is distinct from [`crate::shadbala`] `cheshta_bala`/`cheshta_rasmi`,
//! which return a continuous 0–7 strength score: a consumer needs both the
//! category (here) and the score (there).

use crate::graha::Graha;

/// The eight classical motion states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cheshta {
    /// Clearly retrograde (speed < −1'/day).
    Vakra,
    /// Marginally retrograde (−1'/day ≤ speed < 0): the station
    /// neighbourhood. Sign-entry proper needs boundary tracking (module doc).
    Anuvakra,
    /// Stationary neighbourhood: exact zero up to +1'/day direct (one kala,
    /// the classical small-motion unit, used here as the conventional
    /// tolerance). Negative speeds route to Vakra/Anuvakra first, so this
    /// band is one-sided by construction.
    Vikala,
    /// Slow direct: below half the mean motion.
    Manda,
    /// Slower than Manda: below a third of the mean motion.
    Mandatara,
    /// Near the mean pace: 2/3–4/3 of mean motion.
    Sama,
    /// Fast: 4/3–2× mean motion.
    Chara,
    /// Excessive: above 2× mean motion.
    Atichara,
}

/// One arcminute in degrees — the stationary neighbourhood.
const KALA_DEG: f64 = 1.0 / 60.0;

/// Classical motion state from instantaneous vs mean daily motion.
///
/// `graha` is currently unused by the bands (they are pure speed ratios)
/// and is carried for the request's contract plus the sign-entry refinement
/// (Vakra/Anuvakra, Chara/Atichara) that needs boundary tracking — the
/// module doc's honesty #2. A non-positive `mean_speed_deg_per_day` is
/// caller error; the answer stays deterministic (the stationary checks fire
/// first, anything else reads `Sama`).
#[must_use]
pub fn cheshta(graha: Graha, speed_deg_per_day: f64, mean_speed_deg_per_day: f64) -> Cheshta {
    let _ = graha;
    let s = speed_deg_per_day;
    if s < -KALA_DEG {
        return Cheshta::Vakra;
    }
    if s < 0.0 {
        return Cheshta::Anuvakra;
    }
    if s <= KALA_DEG {
        return Cheshta::Vikala;
    }
    if mean_speed_deg_per_day <= 0.0 {
        return Cheshta::Sama;
    }
    let r = s / mean_speed_deg_per_day;
    if r < 1.0 / 3.0 {
        Cheshta::Mandatara
    } else if r < 2.0 / 3.0 {
        Cheshta::Manda
    } else if r <= 4.0 / 3.0 {
        Cheshta::Sama
    } else if r <= 2.0 {
        Cheshta::Chara
    } else {
        Cheshta::Atichara
    }
}

/// How a yuddha winner is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YuddhaRule {
    /// The northerly graha (greater latitude) wins.
    NorthernLatitude,
    /// The brighter graha wins, in the conventional order Venus, Jupiter,
    /// Mars, Mercury, Saturn. (Apparent magnitudes vary with phase and
    /// distance; this fixed order is the traditional reading.)
    Brighter,
}

/// Brightness rank (lower = brighter) for [`YuddhaRule::Brighter`].
fn brightness_rank(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Venus => Some(0),
        Graha::Jupiter => Some(1),
        Graha::Mars => Some(2),
        Graha::Mercury => Some(3),
        Graha::Saturn => Some(4),
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => None,
    }
}

/// Graha yuddha — planetary war — winner, or `None` where there is no war.
///
/// A war needs the two grahas within 1° of longitude (shortest arc). The
/// classical war is among the five star-planets; involving the Sun, Moon or
/// nodes is not a yuddha and yields `None` under EITHER rule, as does an
/// exact tie under the selected rule. Latitudes are ecliptic degrees, north
/// positive.
///
/// Source: BPHS, the chapter on graha bala.
#[must_use]
pub fn graha_yuddha(
    a: Graha,
    a_longitude_deg: f64,
    a_latitude_deg: f64,
    b: Graha,
    b_longitude_deg: f64,
    b_latitude_deg: f64,
    rule: YuddhaRule,
) -> Option<Graha> {
    if brightness_rank(a).is_none() || brightness_rank(b).is_none() {
        return None;
    }
    let mut sep = (a_longitude_deg - b_longitude_deg).rem_euclid(360.0);
    if sep > 180.0 {
        sep = 360.0 - sep;
    }
    if sep > 1.0 {
        return None;
    }
    match rule {
        YuddhaRule::NorthernLatitude => {
            if (a_latitude_deg - b_latitude_deg).abs() < 1e-12 {
                None
            } else if a_latitude_deg > b_latitude_deg {
                Some(a)
            } else {
                Some(b)
            }
        }
        YuddhaRule::Brighter => match (brightness_rank(a), brightness_rank(b)) {
            (Some(ra), Some(rb)) => match ra.cmp(&rb) {
                core::cmp::Ordering::Equal => None,
                core::cmp::Ordering::Less => Some(a),
                core::cmp::Ordering::Greater => Some(b),
            },
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrograde_and_station() {
        // Mean motion only shapes the DIRECT bands; these fire first.
        assert_eq!(cheshta(Graha::Mars, -0.5, 0.5), Cheshta::Vakra);
        assert_eq!(cheshta(Graha::Mars, -0.01, 0.5), Cheshta::Anuvakra);
        assert_eq!(cheshta(Graha::Mars, 0.0, 0.5), Cheshta::Vikala);
        assert_eq!(cheshta(Graha::Mars, 0.01, 0.5), Cheshta::Vikala);
    }

    #[test]
    fn direct_bands_scale_with_mean() {
        let m = 1.0;
        let g = Graha::Mars;
        assert_eq!(cheshta(g, 0.1, m), Cheshta::Mandatara);
        assert_eq!(cheshta(g, 0.5, m), Cheshta::Manda);
        assert_eq!(cheshta(g, 1.0, m), Cheshta::Sama);
        assert_eq!(cheshta(g, 1.5, m), Cheshta::Chara);
        assert_eq!(cheshta(g, 2.5, m), Cheshta::Atichara);
        // Bands are relative: doubling both speeds keeps the state.
        assert_eq!(cheshta(g, 3.0, 2.0), Cheshta::Chara);
        // The graha parameter is inert by design (see fn doc): the Sun at
        // its own mean pace reads Sama like anyone else.
        assert_eq!(cheshta(Graha::Sun, 1.0, 1.0), Cheshta::Sama);
    }

    #[test]
    fn no_war_beyond_one_degree() {
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                101.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }

    #[test]
    fn northerner_wins_latitude_war() {
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            Some(Graha::Saturn)
        );
        // Exact tie is no decision.
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                1.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }

    #[test]
    fn brighter_wins_brightness_war() {
        // Venus outranks Jupiter; luminaries/nodes are not combatants.
        assert_eq!(
            graha_yuddha(
                Graha::Jupiter,
                100.0,
                5.0,
                Graha::Venus,
                100.5,
                -1.0,
                YuddhaRule::Brighter
            ),
            Some(Graha::Venus)
        );
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Sun,
                100.5,
                0.0,
                YuddhaRule::Brighter
            ),
            None
        );
    }

    #[test]
    fn non_planets_are_no_war_under_either_rule() {
        // The combatant gate sits before the rule match: latitude rule
        // included, not just brightness.
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Sun,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
        assert_eq!(
            graha_yuddha(
                Graha::Rahu,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }
}
