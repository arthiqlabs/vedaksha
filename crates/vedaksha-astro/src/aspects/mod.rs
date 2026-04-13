// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Aspect calculation and pattern detection.
//!
//! Source: Standard astrological definitions. Orb tables from William Lilly.

pub mod patterns;

/// Type of aspect between two celestial bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AspectType {
    // Major
    /// 0°
    Conjunction,
    /// 60°
    Sextile,
    /// 90°
    Square,
    /// 120°
    Trine,
    /// 180°
    Opposition,
    // Minor
    /// 30°
    SemiSextile,
    /// 150°
    Quincunx,
    /// 45°
    SemiSquare,
    /// 135°
    Sesquiquadrate,
    /// 72°
    Quintile,
    /// 144°
    BiQuintile,
}

impl AspectType {
    /// The exact angle of this aspect in degrees.
    #[must_use]
    pub const fn angle(&self) -> f64 {
        match self {
            Self::Conjunction => 0.0,
            Self::Sextile => 60.0,
            Self::Square => 90.0,
            Self::Trine => 120.0,
            Self::Opposition => 180.0,
            Self::SemiSextile => 30.0,
            Self::Quincunx => 150.0,
            Self::SemiSquare => 45.0,
            Self::Sesquiquadrate => 135.0,
            Self::Quintile => 72.0,
            Self::BiQuintile => 144.0,
        }
    }

    /// Whether this is a major (Ptolemaic) aspect.
    #[must_use]
    pub const fn is_major(&self) -> bool {
        matches!(
            self,
            Self::Conjunction | Self::Sextile | Self::Square | Self::Trine | Self::Opposition
        )
    }

    /// Default orb for this aspect in degrees.
    ///
    /// Source: Traditional orbs from William Lilly.
    #[must_use]
    pub const fn default_orb(&self) -> f64 {
        match self {
            Self::Sextile => 6.0,
            Self::Square => 7.0,
            Self::Conjunction | Self::Trine | Self::Opposition => 8.0,
            Self::SemiSextile
            | Self::Quincunx
            | Self::SemiSquare
            | Self::Sesquiquadrate
            | Self::Quintile
            | Self::BiQuintile => 2.0,
        }
    }

    /// All aspect types.
    pub const ALL: &'static [Self] = &[
        Self::Conjunction,
        Self::Sextile,
        Self::Square,
        Self::Trine,
        Self::Opposition,
        Self::SemiSextile,
        Self::Quincunx,
        Self::SemiSquare,
        Self::Sesquiquadrate,
        Self::Quintile,
        Self::BiQuintile,
    ];

    /// Major aspects only.
    pub const MAJOR: &'static [Self] = &[
        Self::Conjunction,
        Self::Sextile,
        Self::Square,
        Self::Trine,
        Self::Opposition,
    ];
}

/// Whether a body is applying to or separating from an aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AspectMotion {
    /// Bodies are moving closer to exact aspect.
    Applying,
    /// Bodies are moving away from exact aspect.
    Separating,
}

/// A detected aspect between two bodies.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Aspect {
    /// Index of the first body.
    pub body1_index: usize,
    /// Index of the second body.
    pub body2_index: usize,
    /// The type of aspect.
    pub aspect_type: AspectType,
    /// The exact orb (difference from exact aspect angle) in degrees.
    pub orb: f64,
    /// Whether applying or separating.
    pub motion: AspectMotion,
    /// Aspect strength: 1.0 at exact, 0.0 at orb boundary (inverse orb ratio).
    pub strength: f64,
}

/// A body's position and speed for aspect calculation.
#[derive(Debug, Clone, Copy)]
pub struct BodyPosition {
    /// Ecliptic longitude in degrees \[0, 360).
    pub longitude: f64,
    /// Daily speed in degrees/day (positive = direct, negative = retrograde).
    pub speed: f64,
}

/// Find all aspects between a set of bodies.
///
/// Checks all pairs. Uses the provided aspect types and their default orbs.
///
/// # Arguments
/// * `positions` — array of body positions
/// * `aspect_types` — which aspects to check for (e.g., `AspectType::MAJOR`)
/// * `orb_factor` — multiplier for default orbs (1.0 = standard, 0.5 = tight)
#[must_use]
pub fn find_aspects(
    positions: &[BodyPosition],
    aspect_types: &[AspectType],
    orb_factor: f64,
) -> Vec<Aspect> {
    let mut aspects = Vec::new();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let separation = vedaksha_math::angle::angular_separation(
                positions[i].longitude,
                positions[j].longitude,
            );

            for &aspect_type in aspect_types {
                let target_angle = aspect_type.angle();
                let max_orb = aspect_type.default_orb() * orb_factor;
                let orb = (separation - target_angle).abs();

                if orb <= max_orb {
                    let relative_speed = positions[i].speed - positions[j].speed;
                    let motion = determine_motion(
                        positions[i].longitude,
                        positions[j].longitude,
                        relative_speed,
                        target_angle,
                    );

                    let strength = 1.0 - orb / max_orb;

                    aspects.push(Aspect {
                        body1_index: i,
                        body2_index: j,
                        aspect_type,
                        orb,
                        motion,
                        strength,
                    });
                }
            }
        }
    }

    aspects
}

/// Determine if an aspect is applying or separating.
fn determine_motion(lon1: f64, lon2: f64, relative_speed: f64, target_angle: f64) -> AspectMotion {
    let diff = vedaksha_math::angle::normalize_degrees(lon1 - lon2);

    // Simulate one tiny step forward to see if orb shrinks or grows.
    let future_diff =
        vedaksha_math::angle::normalize_degrees((lon1 + relative_speed * 0.01) - lon2);

    let current_orb = orb_from_diff(diff, target_angle);
    let future_orb = orb_from_diff(future_diff, target_angle);

    if future_orb < current_orb {
        AspectMotion::Applying
    } else {
        AspectMotion::Separating
    }
}

/// Compute how far `diff` (a normalized [0,360) angle) is from `target_angle`.
fn orb_from_diff(diff: f64, target_angle: f64) -> f64 {
    let candidates = [
        (diff - target_angle).abs(),
        (diff - target_angle + 360.0).abs(),
        (diff - target_angle - 360.0).abs(),
    ];
    candidates.iter().copied().fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn pos(longitude: f64, speed: f64) -> BodyPosition {
        BodyPosition { longitude, speed }
    }

    // --- AspectType helpers ---

    #[test]
    fn conjunction_angle_is_zero() {
        assert!((AspectType::Conjunction.angle() - 0.0).abs() < EPS);
    }

    #[test]
    fn opposition_angle_is_180() {
        assert!((AspectType::Opposition.angle() - 180.0).abs() < EPS);
    }

    #[test]
    fn is_major_for_trine() {
        assert!(AspectType::Trine.is_major());
    }

    #[test]
    fn is_not_major_for_quincunx() {
        assert!(!AspectType::Quincunx.is_major());
    }

    // --- find_aspects: basic detection ---

    #[test]
    fn detects_sextile_at_60_degrees() {
        let positions = [pos(0.0, 1.0), pos(60.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Sextile),
            "Expected a sextile"
        );
    }

    #[test]
    fn detects_square_at_90_degrees() {
        let positions = [pos(0.0, 1.0), pos(90.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Square),
            "Expected a square"
        );
    }

    #[test]
    fn detects_trine_at_120_degrees() {
        let positions = [pos(0.0, 1.0), pos(120.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Expected a trine"
        );
    }

    #[test]
    fn detects_conjunction_wrap_around() {
        // 359° and 1° are 2° apart — well within conjunction orb of 8°
        let positions = [pos(359.0, 1.0), pos(1.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Conjunction),
            "Expected a conjunction across 0°"
        );
    }

    #[test]
    fn orb_boundary_just_within_trine() {
        // 0° and 127° → separation = 127°, orb from 120° = 7° ≤ 8° (trine orb)
        let positions = [pos(0.0, 1.0), pos(127.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Expected trine within orb"
        );
    }

    #[test]
    fn no_trine_outside_orb() {
        // 0° and 129° → separation = 129°, orb = 9° > 8° (trine orb)
        let positions = [pos(0.0, 1.0), pos(129.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        assert!(
            !aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Should not find trine outside orb"
        );
    }

    #[test]
    fn applying_when_faster_body_approaches() {
        // Body at 118° moving at 2°/day, body at 120° stationary.
        // Faster body is catching up to trine → applying.
        let positions = [pos(118.0, 2.0), pos(0.0, 0.0)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        let trine = aspects.iter().find(|a| a.aspect_type == AspectType::Trine);
        assert!(trine.is_some(), "Expected a trine");
        assert_eq!(
            trine.unwrap().motion,
            AspectMotion::Applying,
            "Should be applying"
        );
    }

    #[test]
    fn separating_when_bodies_move_apart() {
        // Body at 122° moving at 2°/day, body at 0° stationary.
        // Faster body already past trine and moving further.
        let positions = [pos(122.0, 2.0), pos(0.0, 0.0)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        let trine = aspects.iter().find(|a| a.aspect_type == AspectType::Trine);
        assert!(trine.is_some(), "Expected a trine");
        assert_eq!(
            trine.unwrap().motion,
            AspectMotion::Separating,
            "Should be separating"
        );
    }

    #[test]
    fn exact_aspect_has_strength_one() {
        let positions = [pos(0.0, 1.0), pos(120.0, 0.0)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 1.0);
        let trine = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Trine)
            .expect("Expected a trine");
        assert!(
            (trine.strength - 1.0).abs() < EPS,
            "Exact aspect should have strength 1.0, got {}",
            trine.strength
        );
    }

    #[test]
    fn orb_factor_restricts_detection() {
        // With tight orb_factor=0.1, only very close aspects detected.
        // 0° and 127° (orb=7°) should not be found with trine orb * 0.1 = 0.8°
        let positions = [pos(0.0, 1.0), pos(127.0, 0.5)];
        let aspects = find_aspects(&positions, AspectType::MAJOR, 0.1);
        assert!(
            !aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Should not find trine with very tight orb factor"
        );
    }
}
