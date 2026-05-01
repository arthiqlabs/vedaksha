// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Gochara — planetary transit interpretation against a natal reference point.
//!
//! Returns the geometric transit picture only. School-specific exemption
//! rules (Sun/Moon mutual, Jupiter/Mercury, Saturn/Rahu, etc.) are applied
//! by the caller via [`apply_vedha_exemptions`] or downstream.
//!
//! Source: BPHS Ch.29.

use serde::{Deserialize, Serialize};

use crate::yoga::YogaPlanet;

/// Selectable vedha pair table.
///
/// Different classical sources record slightly different vedha pairs. Pick
/// the table that matches the school the caller is interpreting against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VedhaTable {
    /// Table from BPHS Ch.29.
    Bphs29,
}

/// Reference point Gochara is read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GocharaReference {
    /// Read transits from the natal Moon's sign (most common).
    Chandra,
    /// Read transits from the natal Lagna (ascendant) sign.
    Lagna,
}

/// Classical favourable / neutral / unfavourable verdict from BPHS Ch.29.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GocharaEffect {
    /// Listed as favourable for this graha at this house from the reference.
    Favourable,
    /// Not listed as favourable; classical reading is unfavourable.
    Unfavourable,
}

/// Per-graha gochara result for a single transit moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahaGochara {
    /// The transiting graha.
    pub graha: YogaPlanet,
    /// Sign index (0–11) the graha currently occupies.
    pub transit_rashi: u8,
    /// Sign index (0–11) of the natal reference point.
    pub natal_reference_rashi: u8,
    /// House from the reference (1–12).
    pub house_from_natal: u8,
    /// Classical verdict per BPHS Ch.29.
    pub classical_effect: GocharaEffect,
    /// Other grahas currently in the corresponding vedha house, before any
    /// school-specific exemption filter is applied. Empty when the
    /// transiting graha is in an unfavourable house (vedha is only
    /// classically defined against favourable transits).
    pub vedha_candidates: Vec<YogaPlanet>,
    /// Optional Bhinna Ashtakavarga bindu count for this graha at this
    /// transit sign (0–8). Caller may attach it from a precomputed
    /// [`crate::ashtakavarga::AshtakavargaTable`].
    pub ashtakavarga_score: Option<u8>,
}

/// Sidereal sign index (0–11) for each of the seven Gochara grahas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitPositions {
    pub sun: u8,
    pub moon: u8,
    pub mars: u8,
    pub mercury: u8,
    pub jupiter: u8,
    pub venus: u8,
    pub saturn: u8,
}

// BPHS Ch.29 — favourable house numbers (1-indexed) per graha.
// Anything not listed here is unfavourable.
const FAVOURABLE_HOUSES: [(YogaPlanet, &[u8]); 7] = [
    (YogaPlanet::Sun, &[3, 6, 10, 11]),
    (YogaPlanet::Moon, &[1, 3, 6, 7, 10, 11]),
    (YogaPlanet::Mars, &[3, 6, 11]),
    (YogaPlanet::Mercury, &[2, 4, 6, 8, 10, 11]),
    (YogaPlanet::Jupiter, &[2, 5, 7, 9, 11]),
    (YogaPlanet::Venus, &[1, 2, 3, 4, 5, 8, 9, 11, 12]),
    (YogaPlanet::Saturn, &[3, 6, 11]),
];

// BPHS Ch.29 — vedha (obstruction) pair table.
// (favourable_house_for_graha, vedha_house) — when another graha occupies
// the vedha house, the favourable transit is obstructed.
const BPHS29_VEDHA: [(YogaPlanet, &[(u8, u8)]); 7] = [
    (YogaPlanet::Sun, &[(3, 9), (6, 12), (10, 4), (11, 5)]),
    (
        YogaPlanet::Moon,
        &[(1, 5), (3, 9), (6, 12), (7, 2), (10, 4), (11, 8)],
    ),
    (YogaPlanet::Mars, &[(3, 12), (6, 9), (11, 5)]),
    (
        YogaPlanet::Mercury,
        &[(2, 5), (4, 3), (6, 9), (8, 1), (10, 8), (11, 12)],
    ),
    (
        YogaPlanet::Jupiter,
        &[(2, 12), (5, 4), (7, 3), (9, 10), (11, 8)],
    ),
    (
        YogaPlanet::Venus,
        &[
            (1, 8),
            (2, 7),
            (3, 1),
            (4, 10),
            (5, 9),
            (8, 5),
            (9, 11),
            (11, 6),
            (12, 3),
        ],
    ),
    (YogaPlanet::Saturn, &[(3, 12), (6, 9), (11, 5)]),
];

#[inline]
fn house_from(reference_sign: u8, transit_sign: u8) -> u8 {
    let h = (i16::from(transit_sign) - i16::from(reference_sign)).rem_euclid(12) + 1;
    // h is in [1, 12] by construction.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let h = h as u8;
    h
}

#[inline]
fn favourable_houses_for(graha: YogaPlanet) -> &'static [u8] {
    for (g, list) in FAVOURABLE_HOUSES {
        if g == graha {
            return list;
        }
    }
    &[]
}

#[inline]
fn vedha_pairs_for(graha: YogaPlanet, table: VedhaTable) -> &'static [(u8, u8)] {
    match table {
        VedhaTable::Bphs29 => {
            for (g, pairs) in BPHS29_VEDHA {
                if g == graha {
                    return pairs;
                }
            }
            &[]
        }
    }
}

#[inline]
fn lookup_vedha_house(graha: YogaPlanet, transit_house: u8, table: VedhaTable) -> Option<u8> {
    for &(fav, vedha) in vedha_pairs_for(graha, table) {
        if fav == transit_house {
            return Some(vedha);
        }
    }
    None
}

/// All seven grahas in canonical Gochara order.
const GOCHARA_GRAHAS: [YogaPlanet; 7] = [
    YogaPlanet::Sun,
    YogaPlanet::Moon,
    YogaPlanet::Mars,
    YogaPlanet::Mercury,
    YogaPlanet::Jupiter,
    YogaPlanet::Venus,
    YogaPlanet::Saturn,
];

#[inline]
fn transit_sign_of(graha: YogaPlanet, t: TransitPositions) -> Option<u8> {
    match graha {
        YogaPlanet::Sun => Some(t.sun),
        YogaPlanet::Moon => Some(t.moon),
        YogaPlanet::Mars => Some(t.mars),
        YogaPlanet::Mercury => Some(t.mercury),
        YogaPlanet::Jupiter => Some(t.jupiter),
        YogaPlanet::Venus => Some(t.venus),
        YogaPlanet::Saturn => Some(t.saturn),
        YogaPlanet::Rahu | YogaPlanet::Ketu => None,
    }
}

/// Compute Gochara (transit interpretation) for all seven grahas against a
/// natal reference sign.
///
/// Returns the geometric picture only — classical favourable/unfavourable
/// verdicts and raw vedha candidates. School-specific exemption rules are
/// the caller's responsibility (see [`apply_vedha_exemptions`]).
///
/// Rahu and Ketu are never returned — BPHS Ch.29 Gochara is silent on the
/// nodes.
///
/// Source: BPHS Ch.29.
#[must_use]
pub fn compute_gochara(
    transits: &TransitPositions,
    natal_reference_sign: u8,
    vedha_table: VedhaTable,
) -> Vec<GrahaGochara> {
    let mut out = Vec::with_capacity(7);

    for graha in GOCHARA_GRAHAS {
        let Some(transit_sign) = transit_sign_of(graha, *transits) else {
            continue;
        };

        let house = house_from(natal_reference_sign, transit_sign);
        let favourable = favourable_houses_for(graha).contains(&house);
        let classical_effect = if favourable {
            GocharaEffect::Favourable
        } else {
            GocharaEffect::Unfavourable
        };

        // Vedha is classically defined only against favourable transits.
        // For unfavourable transits we leave the candidate list empty.
        let mut vedha_candidates: Vec<YogaPlanet> = Vec::new();
        if favourable {
            if let Some(vedha_house) = lookup_vedha_house(graha, house, vedha_table) {
                let target_sign_i =
                    (i16::from(natal_reference_sign) + i16::from(vedha_house) - 1).rem_euclid(12);
                // target_sign_i is in [0, 11] by construction.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let target_sign = target_sign_i as u8;
                for other in GOCHARA_GRAHAS {
                    if other == graha {
                        continue;
                    }
                    if let Some(other_sign) = transit_sign_of(other, *transits) {
                        if other_sign == target_sign {
                            vedha_candidates.push(other);
                        }
                    }
                }
            }
        }

        out.push(GrahaGochara {
            graha,
            transit_rashi: transit_sign,
            natal_reference_rashi: natal_reference_sign,
            house_from_natal: house,
            classical_effect,
            vedha_candidates,
            ashtakavarga_score: None,
        });
    }

    out
}

/// Classical school exemption profile applied on top of the raw vedha
/// candidate list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchoolProfile {
    /// No exemptions — return raw geometry.
    Geometry,
    /// Parashari convention: Sun and Moon do not vedha each other; Jupiter
    /// and Mercury do not vedha each other; Saturn and Rahu do not vedha
    /// each other (Rahu is not iterated as a graha here, included for
    /// completeness when callers extend the model).
    Parashari,
}

/// Strip school-specific exemption pairs from a `vedha_candidates` list.
///
/// Mutates a single [`GrahaGochara`] entry in place. Apply per-entry across
/// the result of [`compute_gochara`] when ready to interpret.
pub fn apply_vedha_exemptions(entry: &mut GrahaGochara, school: SchoolProfile) {
    match school {
        SchoolProfile::Geometry => {}
        SchoolProfile::Parashari => {
            entry.vedha_candidates.retain(|other| {
                !matches!(
                    (entry.graha, *other),
                    (YogaPlanet::Sun, YogaPlanet::Moon)
                        | (YogaPlanet::Moon, YogaPlanet::Sun)
                        | (YogaPlanet::Jupiter, YogaPlanet::Mercury)
                        | (YogaPlanet::Mercury, YogaPlanet::Jupiter)
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transits() -> TransitPositions {
        // Arbitrary positions; chosen to exercise vedha lookup.
        TransitPositions {
            sun: 0,     // Aries
            moon: 4,    // Leo
            mars: 2,    // Gemini
            mercury: 6, // Libra
            jupiter: 8, // Sagittarius
            venus: 10,  // Aquarius
            saturn: 6,  // Libra
        }
    }

    #[test]
    fn house_from_wraps_correctly() {
        assert_eq!(house_from(0, 0), 1);
        assert_eq!(house_from(0, 11), 12);
        assert_eq!(house_from(11, 0), 2);
        assert_eq!(house_from(5, 5), 1);
    }

    #[test]
    fn returns_seven_entries_excluding_nodes() {
        let g = compute_gochara(&sample_transits(), 0, VedhaTable::Bphs29);
        assert_eq!(g.len(), 7);
        for entry in &g {
            assert!(matches!(
                entry.graha,
                YogaPlanet::Sun
                    | YogaPlanet::Moon
                    | YogaPlanet::Mars
                    | YogaPlanet::Mercury
                    | YogaPlanet::Jupiter
                    | YogaPlanet::Venus
                    | YogaPlanet::Saturn
            ));
        }
    }

    #[test]
    fn sun_in_third_house_is_favourable() {
        // Reference at sign 0, Sun at sign 2 → 3rd house from natal.
        let mut t = sample_transits();
        t.sun = 2;
        let g = compute_gochara(&t, 0, VedhaTable::Bphs29);
        let sun = g.iter().find(|e| e.graha == YogaPlanet::Sun).unwrap();
        assert_eq!(sun.house_from_natal, 3);
        assert_eq!(sun.classical_effect, GocharaEffect::Favourable);
    }

    #[test]
    fn sun_in_first_house_is_unfavourable() {
        // Sun at the reference sign → 1st house, not in [3,6,10,11].
        let mut t = sample_transits();
        t.sun = 0;
        let g = compute_gochara(&t, 0, VedhaTable::Bphs29);
        let sun = g.iter().find(|e| e.graha == YogaPlanet::Sun).unwrap();
        assert_eq!(sun.house_from_natal, 1);
        assert_eq!(sun.classical_effect, GocharaEffect::Unfavourable);
        assert!(sun.vedha_candidates.is_empty());
    }

    #[test]
    fn vedha_only_attached_to_favourable_transits() {
        let g = compute_gochara(&sample_transits(), 0, VedhaTable::Bphs29);
        for entry in &g {
            if entry.classical_effect == GocharaEffect::Unfavourable {
                assert!(
                    entry.vedha_candidates.is_empty(),
                    "unfavourable transit must not carry vedha candidates"
                );
            }
        }
    }

    #[test]
    fn sun_third_house_with_planet_in_ninth_records_vedha() {
        // BPHS Ch.29: Sun favourable 3 ↔ vedha 9.
        // Reference sign 0, Sun in 3rd (sign 2), Saturn in 9th (sign 8).
        let mut t = sample_transits();
        t.sun = 2;
        t.saturn = 8;
        // Make sure no other planet sits in sign 8.
        t.moon = 1;
        t.mars = 3;
        t.mercury = 4;
        t.jupiter = 5;
        t.venus = 10;
        let g = compute_gochara(&t, 0, VedhaTable::Bphs29);
        let sun = g.iter().find(|e| e.graha == YogaPlanet::Sun).unwrap();
        assert_eq!(sun.classical_effect, GocharaEffect::Favourable);
        assert_eq!(sun.vedha_candidates, vec![YogaPlanet::Saturn]);
    }

    #[test]
    fn parashari_exemption_drops_sun_moon_mutual_vedha() {
        // Construct: Moon transiting 1st house from natal (favourable, vedha 5),
        // Sun transiting 5th house from natal — geometric vedha pair.
        // Reference sign 0, Moon in sign 0 (1st), Sun in sign 4 (5th).
        let t = TransitPositions {
            sun: 4,
            moon: 0,
            mars: 1,
            mercury: 2,
            jupiter: 3,
            venus: 5,
            saturn: 6,
        };
        let mut g = compute_gochara(&t, 0, VedhaTable::Bphs29);
        let moon = g.iter().find(|e| e.graha == YogaPlanet::Moon).unwrap();
        assert_eq!(moon.house_from_natal, 1);
        assert!(moon.vedha_candidates.contains(&YogaPlanet::Sun));

        // Parashari profile must strip Sun from Moon's vedha candidates.
        for entry in g.iter_mut() {
            apply_vedha_exemptions(entry, SchoolProfile::Parashari);
        }
        let moon = g.iter().find(|e| e.graha == YogaPlanet::Moon).unwrap();
        assert!(!moon.vedha_candidates.contains(&YogaPlanet::Sun));
    }

    #[test]
    fn vedha_candidate_count_never_exceeds_six() {
        // Mathematical invariant: at most 6 other grahas can be in any one
        // sign at the same time.
        let g = compute_gochara(&sample_transits(), 7, VedhaTable::Bphs29);
        for entry in &g {
            assert!(entry.vedha_candidates.len() <= 6);
        }
    }

    #[test]
    fn favourable_house_table_is_well_formed() {
        for (_, list) in FAVOURABLE_HOUSES {
            for &h in list {
                assert!((1..=12).contains(&h));
            }
        }
    }

    #[test]
    fn vedha_table_is_well_formed() {
        for (_, pairs) in BPHS29_VEDHA {
            for &(fav, vedha) in pairs {
                assert!((1..=12).contains(&fav));
                assert!((1..=12).contains(&vedha));
                assert_ne!(fav, vedha, "favourable house cannot be its own vedha");
            }
        }
    }

    #[test]
    fn vedha_pairs_only_reference_favourable_houses() {
        // Every vedha entry's `fav` must appear in the favourable list for
        // that graha — vedha is defined only against favourable transits.
        for (graha, pairs) in BPHS29_VEDHA {
            let favs = favourable_houses_for(graha);
            for &(fav, _) in pairs {
                assert!(
                    favs.contains(&fav),
                    "{graha:?}: vedha for non-favourable house {fav}"
                );
            }
        }
    }

    #[test]
    fn lagna_reference_works_same_geometry() {
        // Reference enum is selected by caller; compute_gochara takes the
        // sign index directly. Verify that switching reference sign
        // shifts the house numbering by the expected amount.
        let t = sample_transits();
        let from_aries = compute_gochara(&t, 0, VedhaTable::Bphs29);
        let from_taurus = compute_gochara(&t, 1, VedhaTable::Bphs29);
        for (a, b) in from_aries.iter().zip(from_taurus.iter()) {
            assert_eq!(a.graha, b.graha);
            // Moving reference forward by 1 sign decrements the house from
            // natal by 1 (mod 12).
            let expected = ((a.house_from_natal as i16 - 2).rem_euclid(12) + 1) as u8;
            assert_eq!(b.house_from_natal, expected);
        }
    }
}
