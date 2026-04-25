// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Astrological pattern detection.
//!
//! Detects Grand Trine, T-Square, Yod, Grand Cross, and Stellium patterns
//! from a set of pre-computed aspects and body positions.
//!
//! Source: Standard astrological pattern definitions.

use super::{Aspect, AspectType, BodyPosition};

/// An astrological pattern formed by multiple aspects.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// The type of pattern.
    pub pattern_type: PatternType,
    /// Indices of the bodies forming this pattern.
    pub body_indices: Vec<usize>,
}

/// Type of multi-body astrological pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// Three bodies in mutual trine (120° apart from each other).
    GrandTrine,
    /// Two bodies in opposition with a third squaring both.
    TSquare,
    /// Two bodies in sextile with a third quincunx to both.
    Yod,
    /// Four bodies forming a cross of oppositions and squares.
    GrandCross,
    /// Three or more bodies within conjunction orb of each other.
    Stellium,
}

/// Detect patterns from a list of aspects and body positions.
///
/// Returns all detected patterns. A single chart may contain multiple
/// instances of the same pattern type.
#[must_use]
pub fn detect_patterns(aspects: &[Aspect], positions: &[BodyPosition]) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    detect_grand_trines(aspects, &mut patterns);
    detect_t_squares(aspects, &mut patterns);
    detect_yods(aspects, &mut patterns);
    detect_grand_crosses(aspects, &mut patterns);
    detect_stelliums(positions, &mut patterns);

    patterns
}

// ---------------------------------------------------------------------------
// Helper: check whether two bodies form a given aspect type in the aspect list
// ---------------------------------------------------------------------------

fn has_aspect(aspects: &[Aspect], i: usize, j: usize, kind: AspectType) -> bool {
    aspects.iter().any(|a| {
        a.aspect_type == kind
            && ((a.body1_index == i && a.body2_index == j)
                || (a.body1_index == j && a.body2_index == i))
    })
}

// ---------------------------------------------------------------------------
// Grand Trine: A-B trine, B-C trine, A-C trine
// ---------------------------------------------------------------------------

fn detect_grand_trines(aspects: &[Aspect], out: &mut Vec<Pattern>) {
    // Collect all unique body indices involved in at least one trine.
    let trine_bodies: Vec<usize> = {
        let mut v: Vec<usize> = aspects
            .iter()
            .filter(|a| a.aspect_type == AspectType::Trine)
            .flat_map(|a| [a.body1_index, a.body2_index])
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    let n = trine_bodies.len();
    for ai in 0..n {
        for bi in (ai + 1)..n {
            for ci in (bi + 1)..n {
                let a = trine_bodies[ai];
                let b = trine_bodies[bi];
                let c = trine_bodies[ci];

                if has_aspect(aspects, a, b, AspectType::Trine)
                    && has_aspect(aspects, b, c, AspectType::Trine)
                    && has_aspect(aspects, a, c, AspectType::Trine)
                {
                    out.push(Pattern {
                        pattern_type: PatternType::GrandTrine,
                        body_indices: vec![a, b, c],
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// T-Square: A-B opposition, C squares both A and B
// ---------------------------------------------------------------------------

fn detect_t_squares(aspects: &[Aspect], out: &mut Vec<Pattern>) {
    let oppositions: Vec<(usize, usize)> = aspects
        .iter()
        .filter(|a| a.aspect_type == AspectType::Opposition)
        .map(|a| (a.body1_index, a.body2_index))
        .collect();

    // Collect all body indices.
    let all_bodies: Vec<usize> = {
        let mut v: Vec<usize> = aspects
            .iter()
            .flat_map(|a| [a.body1_index, a.body2_index])
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    for (a, b) in &oppositions {
        for &c in &all_bodies {
            if c == *a || c == *b {
                continue;
            }
            if has_aspect(aspects, c, *a, AspectType::Square)
                && has_aspect(aspects, c, *b, AspectType::Square)
            {
                out.push(Pattern {
                    pattern_type: PatternType::TSquare,
                    body_indices: vec![*a, *b, c],
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Yod (Finger of God): A-B sextile, C quincunx to both A and B
// ---------------------------------------------------------------------------

fn detect_yods(aspects: &[Aspect], out: &mut Vec<Pattern>) {
    let sextiles: Vec<(usize, usize)> = aspects
        .iter()
        .filter(|a| a.aspect_type == AspectType::Sextile)
        .map(|a| (a.body1_index, a.body2_index))
        .collect();

    let all_bodies: Vec<usize> = {
        let mut v: Vec<usize> = aspects
            .iter()
            .flat_map(|a| [a.body1_index, a.body2_index])
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    for (a, b) in &sextiles {
        for &c in &all_bodies {
            if c == *a || c == *b {
                continue;
            }
            if has_aspect(aspects, c, *a, AspectType::Quincunx)
                && has_aspect(aspects, c, *b, AspectType::Quincunx)
            {
                out.push(Pattern {
                    pattern_type: PatternType::Yod,
                    body_indices: vec![*a, *b, c],
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Grand Cross: four bodies forming two pairs of oppositions with squares
// ---------------------------------------------------------------------------

fn detect_grand_crosses(aspects: &[Aspect], out: &mut Vec<Pattern>) {
    // Collect opposition pairs.
    let oppositions: Vec<(usize, usize)> = aspects
        .iter()
        .filter(|a| a.aspect_type == AspectType::Opposition)
        .map(|a| (a.body1_index, a.body2_index))
        .collect();

    let opp_count = oppositions.len();
    for i in 0..opp_count {
        for j in (i + 1)..opp_count {
            let (body_a, body_b) = oppositions[i];
            let (body_c, body_d) = oppositions[j];

            // All four bodies must be distinct.
            let mut indices = [body_a, body_b, body_c, body_d];
            indices.sort_unstable();
            if indices[0] == indices[1] || indices[1] == indices[2] || indices[2] == indices[3] {
                continue;
            }

            // The two pairs must form squares with each other.
            // In a Grand Cross: body_a squares body_c, body_a squares body_d, etc.
            if has_aspect(aspects, body_a, body_c, AspectType::Square)
                && has_aspect(aspects, body_a, body_d, AspectType::Square)
                && has_aspect(aspects, body_b, body_c, AspectType::Square)
                && has_aspect(aspects, body_b, body_d, AspectType::Square)
            {
                out.push(Pattern {
                    pattern_type: PatternType::GrandCross,
                    body_indices: vec![body_a, body_b, body_c, body_d],
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Stellium: three or more bodies all within conjunction orb of each other
// ---------------------------------------------------------------------------

fn detect_stelliums(positions: &[BodyPosition], out: &mut Vec<Pattern>) {
    let orb = AspectType::Conjunction.default_orb();
    let n = positions.len();

    // Build a graph of which pairs are within conjunction orb.
    // Then find all maximal cliques of size >= 3.
    let mut adjacent = vec![vec![false; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let sep = vedaksha_math::angle::angular_separation(
                positions[i].longitude,
                positions[j].longitude,
            );
            if sep <= orb {
                adjacent[i][j] = true;
                adjacent[j][i] = true;
            }
        }
    }

    // Enumerate all subsets of size >= 3 that form cliques.
    // For typical chart sizes (10-20 bodies) this is fine.
    for size in 3..=n {
        find_cliques_of_size(&adjacent, n, size, out);
    }
}

/// Find all cliques of exactly `size` in the adjacency matrix and emit them as
/// Stellium patterns. Only add if not already a sub-clique of a larger one
/// (we use a simple deduplcation: skip if we have already emitted a superset).
fn find_cliques_of_size(adjacent: &[Vec<bool>], n: usize, size: usize, out: &mut Vec<Pattern>) {
    // Generate combinations of `size` indices.
    let mut combo: Vec<usize> = (0..size).collect();

    loop {
        // Check if combo is a clique.
        let is_clique = combo.windows(2).all(|_| true) && {
            let mut ok = true;
            'outer: for ai in 0..combo.len() {
                for bi in (ai + 1)..combo.len() {
                    if !adjacent[combo[ai]][combo[bi]] {
                        ok = false;
                        break 'outer;
                    }
                }
            }
            ok
        };

        if is_clique {
            // Only add if not already subsumed by an existing larger stellium.
            let already_covered = out.iter().any(|p| {
                p.pattern_type == PatternType::Stellium
                    && combo.iter().all(|idx| p.body_indices.contains(idx))
            });
            if !already_covered {
                out.push(Pattern {
                    pattern_type: PatternType::Stellium,
                    body_indices: combo.clone(),
                });
            }
        }

        // Advance to next combination.
        if !next_combination(&mut combo, n) {
            break;
        }
    }
}

/// Advance a sorted combination to the next one in lexicographic order.
/// Returns `false` when we have exhausted all combinations.
fn next_combination(combo: &mut [usize], n: usize) -> bool {
    let k = combo.len();
    // Find rightmost element that can be incremented.
    let mut i = k;
    loop {
        if i == 0 {
            return false;
        }
        i -= 1;
        if combo[i] < n - k + i {
            combo[i] += 1;
            for j in (i + 1)..k {
                combo[j] = combo[j - 1] + 1;
            }
            return true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspects::{BodyPosition, find_aspects};

    fn pos(longitude: f64) -> BodyPosition {
        BodyPosition {
            longitude,
            speed: 1.0,
        }
    }

    // --- Grand Trine ---

    #[test]
    fn detects_grand_trine() {
        // Three bodies exactly 120° apart.
        let positions = [pos(0.0), pos(120.0), pos(240.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::GrandTrine),
            "Expected Grand Trine"
        );
    }

    // --- Stellium ---

    #[test]
    fn detects_stellium_three_bodies() {
        // Three bodies very close together.
        let positions = [pos(10.0), pos(12.0), pos(14.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::Stellium),
            "Expected Stellium"
        );
    }

    #[test]
    fn stellium_contains_correct_indices() {
        let positions = [pos(10.0), pos(12.0), pos(14.0), pos(200.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        let stellium = patterns
            .iter()
            .find(|p| p.pattern_type == PatternType::Stellium)
            .expect("Expected a Stellium");
        // Body at index 3 (200°) should NOT be in the stellium.
        assert!(
            !stellium.body_indices.contains(&3),
            "Body 3 should not be in the stellium"
        );
        // Bodies 0,1,2 should all be in the stellium.
        assert!(stellium.body_indices.contains(&0));
        assert!(stellium.body_indices.contains(&1));
        assert!(stellium.body_indices.contains(&2));
    }

    #[test]
    fn no_stellium_when_bodies_spread_out() {
        let positions = [pos(0.0), pos(60.0), pos(120.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            !patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::Stellium),
            "Should not find stellium when bodies are spread"
        );
    }

    // --- T-Square ---

    #[test]
    fn detects_t_square() {
        // A at 0°, B at 180° (opposition), C at 90° (squares both).
        let positions = [pos(0.0), pos(180.0), pos(90.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::TSquare),
            "Expected T-Square"
        );
    }

    // --- Yod ---

    #[test]
    fn detects_yod() {
        // A at 0°, B at 60° (sextile), C at 150° (quincunx to A=150°, quincunx to B=90°).
        // C must be quincunx (150°) to both A and B.
        // sep(C=210°, A=0°) = 150° ✓, sep(C=210°, B=60°) = 150° ✓
        let positions = [pos(0.0), pos(60.0), pos(210.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            patterns.iter().any(|p| p.pattern_type == PatternType::Yod),
            "Expected Yod"
        );
    }

    // --- Grand Cross ---

    #[test]
    fn detects_grand_cross() {
        // Four bodies: 0°, 90°, 180°, 270° forming a perfect cross.
        let positions = [pos(0.0), pos(90.0), pos(180.0), pos(270.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::GrandCross),
            "Expected Grand Cross"
        );
    }

    // --- no false positives ---

    #[test]
    fn no_grand_trine_for_random_positions() {
        let positions = [pos(0.0), pos(45.0), pos(200.0)];
        let aspects = find_aspects(&positions, AspectType::ALL, 1.0);
        let patterns = detect_patterns(&aspects, &positions);
        assert!(
            !patterns
                .iter()
                .any(|p| p.pattern_type == PatternType::GrandTrine),
            "Should not find Grand Trine for random positions"
        );
    }
}
