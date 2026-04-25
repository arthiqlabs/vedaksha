// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Synastry — aspects between two natal charts.
//!
//! Computes all aspects between planets in chart A and planets in chart B.

use crate::aspects::{AspectType, BodyPosition};

/// A synastry aspect between two charts.
#[derive(Debug, Clone)]
pub struct SynastryAspect {
    /// Planet from chart A
    pub chart_a_body: usize,
    /// Planet from chart B
    pub chart_b_body: usize,
    /// Aspect type
    pub aspect_type: AspectType,
    /// Orb in degrees
    pub orb: f64,
    /// Aspect strength (1.0 at exact, 0.0 at orb boundary)
    pub strength: f64,
}

/// Find all synastry aspects between two charts.
///
/// # Arguments
/// * `chart_a` — positions of planets in chart A
/// * `chart_b` — positions of planets in chart B
/// * `aspect_types` — which aspects to check
/// * `orb_factor` — multiplier for default orbs (1.0 = standard)
#[must_use]
pub fn find_synastry_aspects(
    chart_a: &[BodyPosition],
    chart_b: &[BodyPosition],
    aspect_types: &[AspectType],
    orb_factor: f64,
) -> Vec<SynastryAspect> {
    let mut aspects = Vec::new();

    for (i, pos_a) in chart_a.iter().enumerate() {
        for (j, pos_b) in chart_b.iter().enumerate() {
            let separation =
                vedaksha_math::angle::angular_separation(pos_a.longitude, pos_b.longitude);

            for &aspect_type in aspect_types {
                let target = aspect_type.angle();
                let max_orb = aspect_type.default_orb() * orb_factor;
                let orb = (separation - target).abs();

                if orb <= max_orb {
                    let strength = 1.0 - orb / max_orb;
                    aspects.push(SynastryAspect {
                        chart_a_body: i,
                        chart_b_body: j,
                        aspect_type,
                        orb,
                        strength,
                    });
                }
            }
        }
    }

    aspects
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn pos(longitude: f64) -> BodyPosition {
        BodyPosition {
            longitude,
            speed: 1.0,
        }
    }

    #[test]
    fn conjunction_detected_between_charts() {
        // Chart A: Sun at 30°, Chart B: Sun at 32° — 2° orb, within conjunction orb of 8°
        let chart_a = [pos(30.0)];
        let chart_b = [pos(32.0)];
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        assert!(
            aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Conjunction),
            "Expected a conjunction synastry aspect"
        );
    }

    #[test]
    fn trine_detected_between_charts() {
        // Chart A: Sun at 0°, Chart B: Moon at 120° — exact trine
        let chart_a = [pos(0.0)];
        let chart_b = [pos(120.0)];
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        assert!(
            aspects.iter().any(|a| a.aspect_type == AspectType::Trine),
            "Expected a trine synastry aspect"
        );
    }

    #[test]
    fn no_aspects_when_far_apart() {
        // Chart A: planet at 0°, Chart B: planet at 55° — no major aspect
        // Nearest major aspect: sextile at 60° (orb=5°, max=6°) — actually within orb!
        // Use 52° instead: sextile orb = 8°, actual orb = 8°, boundary excluded
        let chart_a = [pos(0.0)];
        let chart_b = [pos(75.0)];
        // 75° — nearest: sextile at 60° (orb=15°, max=6°), square at 90° (orb=15°, max=7°) → none
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        assert!(
            aspects.is_empty(),
            "Expected no aspects, found: {aspects:?}"
        );
    }

    #[test]
    fn multiple_aspects_detected_between_charts() {
        // Chart A has 2 planets, Chart B has 2 planets — cross-chart aspects
        let chart_a = [pos(0.0), pos(90.0)];
        let chart_b = [pos(120.0), pos(0.5)];
        // 0° vs 120° → trine; 0° vs 0.5° → conjunction; 90° vs 0.5° → square (within 7° orb)
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        assert!(
            aspects.len() >= 2,
            "Expected multiple synastry aspects, got {}",
            aspects.len()
        );
    }

    #[test]
    fn exact_aspect_has_strength_one() {
        let chart_a = [pos(0.0)];
        let chart_b = [pos(0.0)];
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        let conj = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Conjunction)
            .expect("Expected a conjunction");
        assert!(
            (conj.strength - 1.0).abs() < EPS,
            "Exact aspect should have strength 1.0, got {}",
            conj.strength
        );
    }

    #[test]
    fn body_indices_are_correct() {
        // Chart A planet 1 (index 1) vs Chart B planet 0 (index 0)
        let chart_a = [pos(10.0), pos(120.0)];
        let chart_b = [pos(0.0)];
        // 120° vs 0° → exact trine
        let aspects = find_synastry_aspects(&chart_a, &chart_b, AspectType::MAJOR, 1.0);
        let trine = aspects
            .iter()
            .find(|a| a.aspect_type == AspectType::Trine)
            .expect("Expected a trine");
        assert_eq!(trine.chart_a_body, 1);
        assert_eq!(trine.chart_b_body, 0);
    }
}
