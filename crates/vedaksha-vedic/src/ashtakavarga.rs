// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Bhinna Ashtakavarga and Sarvashtakavarga.
//!
//! Source: BPHS Ch.66 vv.13-68.
//! Named 'bhinna' to distinguish from Shodhita (rectified) Ashtakavarga.
//! Trikona/Ekadhipatya Shodhana and Pinda Sadhana are deferred.

use serde::{Deserialize, Serialize};

use crate::yoga::YogaPlanet;

/// Bhinna Ashtakavarga table for one planet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtakavargaTable {
    /// The planet this table belongs to.
    pub planet: YogaPlanet,
    /// Bindu count per zodiac sign (index 0 = Aries … 11 = Pisces).
    pub bindus: [u8; 12],
    /// Sum of all 12 bindus.
    pub total: u8,
}

/// Input positions for Bhinna Ashtakavarga computation.
#[derive(Debug, Clone, Deserialize)]
pub struct BhinnaAshtakavargaInput {
    /// Sign index of Sun (0–11).
    pub sun: u8,
    /// Sign index of Moon (0–11).
    pub moon: u8,
    /// Sign index of Mars (0–11).
    pub mars: u8,
    /// Sign index of Mercury (0–11).
    pub mercury: u8,
    /// Sign index of Jupiter (0–11).
    pub jupiter: u8,
    /// Sign index of Venus (0–11).
    pub venus: u8,
    /// Sign index of Saturn (0–11).
    pub saturn: u8,
    /// Sign index of Lagna / Ascendant (0–11).
    pub lagna: u8,
}

// BPHS Ch.66 vv.13-68.
// Row i = target planet (0=Sun, 1=Moon, 2=Mars, 3=Mercury, 4=Jupiter, 5=Venus, 6=Saturn).
// Col j = contributor  (0=Sun, 1=Moon, 2=Mars, 3=Mercury, 4=Jupiter, 5=Venus, 6=Saturn, 7=Lagna).
// Bit k (0-indexed) is set when house (k+1) counted from contributor gives a bindu to the target.
// house_from_contributor = (target_sign - contributor_sign).rem_euclid(12) + 1.
// bit index = house - 1.
const BINDU_MASKS: [[u16; 8]; 7] = [
    [1995, 1572, 1995, 3892, 1328, 2144, 1995, 3628], // Sun    total=48
    [1764, 1637, 1846, 1757, 3785, 1884, 1076, 1572], // Moon   total=49
    [1588, 1060, 1739, 1076, 3616, 3232, 1993, 1573], // Mars   total=39
    [3376, 1706, 1995, 3893, 3232, 1439, 1995, 1707], // Mercury total=54
    [1999, 1362, 1739, 1851, 1743, 1842, 2100, 1915], // Jupiter total=56
    [3200, 3487, 3372, 1332, 1936, 1951, 1948, 1439], // Venus  total=52
    [1739, 1060, 3636, 4000, 3120, 3104, 1076, 1581], // Saturn total=39
];

/// Returns Bhinna Ashtakavarga for all 7 planets.
///
/// Source: BPHS Ch.66 vv.13-68.
#[must_use]
pub fn bhinna_ashtakavarga(input: &BhinnaAshtakavargaInput) -> [AshtakavargaTable; 7] {
    let contributor_signs: [u8; 8] = [
        input.sun, input.moon, input.mars, input.mercury,
        input.jupiter, input.venus, input.saturn, input.lagna,
    ];
    let target_planets = [
        YogaPlanet::Sun, YogaPlanet::Moon, YogaPlanet::Mars,
        YogaPlanet::Mercury, YogaPlanet::Jupiter, YogaPlanet::Venus, YogaPlanet::Saturn,
    ];

    core::array::from_fn(|pi| {
        let masks = BINDU_MASKS[pi];
        let mut bindus = [0u8; 12];
        for sign in 0u8..12 {
            let mut count = 0u8;
            for (ci, &contrib_sign) in contributor_signs.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let bit = (i16::from(sign) - i16::from(contrib_sign)).rem_euclid(12) as u8;
                if (masks[ci] >> bit) & 1 == 1 {
                    count += 1;
                }
            }
            bindus[sign as usize] = count;
        }
        let total: u8 = bindus.iter().sum();
        AshtakavargaTable {
            planet: target_planets[pi],
            bindus,
            total,
        }
    })
}

/// Sarvashtakavarga = per-sign sum of all 7 planet tables.
///
/// Lagna's Ashtakavarga is excluded per BPHS Ch.66.
#[must_use]
pub fn sarvashtakavarga(tables: &[AshtakavargaTable; 7]) -> [u8; 12] {
    let mut sarva = [0u8; 12];
    for table in tables {
        for (i, &b) in table.bindus.iter().enumerate() {
            sarva[i] = sarva[i].saturating_add(b);
        }
    }
    sarva
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_different_signs() -> BhinnaAshtakavargaInput {
        BhinnaAshtakavargaInput {
            sun: 3, moon: 7, mars: 1,
            mercury: 10, jupiter: 5,
            venus: 8, saturn: 11, lagna: 0,
        }
    }

    // Canonical total-bindu checksums from BPHS Ch.66 — the primary correctness gate.

    #[test]
    fn sun_total_bindus_is_48() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[0].total, 48, "Sun total bindus must be 48");
    }

    #[test]
    fn moon_total_bindus_is_49() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[1].total, 49, "Moon total bindus must be 49");
    }

    #[test]
    fn mars_total_bindus_is_39() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[2].total, 39, "Mars total bindus must be 39");
    }

    #[test]
    fn mercury_total_bindus_is_54() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[3].total, 54, "Mercury total bindus must be 54");
    }

    #[test]
    fn jupiter_total_bindus_is_56() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[4].total, 56, "Jupiter total bindus must be 56");
    }

    #[test]
    fn venus_total_bindus_is_52() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[5].total, 52, "Venus total bindus must be 52");
    }

    #[test]
    fn saturn_total_bindus_is_39() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[6].total, 39, "Saturn total bindus must be 39");
    }

    #[test]
    fn total_matches_bindu_sum() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        for table in &tables {
            let sum: u8 = table.bindus.iter().sum();
            assert_eq!(table.total, sum, "total must equal sum of bindus for {:?}", table.planet);
        }
    }

    #[test]
    fn planet_order_is_sun_moon_mars_mercury_jupiter_venus_saturn() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        assert_eq!(tables[0].planet, YogaPlanet::Sun);
        assert_eq!(tables[1].planet, YogaPlanet::Moon);
        assert_eq!(tables[2].planet, YogaPlanet::Mars);
        assert_eq!(tables[3].planet, YogaPlanet::Mercury);
        assert_eq!(tables[4].planet, YogaPlanet::Jupiter);
        assert_eq!(tables[5].planet, YogaPlanet::Venus);
        assert_eq!(tables[6].planet, YogaPlanet::Saturn);
    }

    #[test]
    fn sarvashtakavarga_sum_matches_all_planet_totals() {
        let tables = bhinna_ashtakavarga(&all_different_signs());
        let sarva = sarvashtakavarga(&tables);
        let sarva_total: u32 = sarva.iter().map(|&b| u32::from(b)).sum();
        let planet_total: u32 = tables.iter().map(|t| u32::from(t.total)).sum();
        assert_eq!(sarva_total, planet_total);
    }

    #[test]
    fn sarvashtakavarga_grand_total_is_337() {
        // 48+49+39+54+56+52+39 = 337
        let tables = bhinna_ashtakavarga(&all_different_signs());
        let sarva = sarvashtakavarga(&tables);
        let total: u32 = sarva.iter().map(|&b| u32::from(b)).sum();
        assert_eq!(total, 337);
    }

    #[test]
    fn all_contributors_in_same_sign_canonical_totals_unchanged() {
        // Totals are position-independent invariants — same regardless of input signs.
        let input = BhinnaAshtakavargaInput {
            sun: 0, moon: 0, mars: 0,
            mercury: 0, jupiter: 0, venus: 0, saturn: 0, lagna: 0,
        };
        let tables = bhinna_ashtakavarga(&input);
        assert_eq!(tables[0].total, 48);
        assert_eq!(tables[1].total, 49);
        assert_eq!(tables[2].total, 39);
        assert_eq!(tables[3].total, 54);
        assert_eq!(tables[4].total, 56);
        assert_eq!(tables[5].total, 52);
        assert_eq!(tables[6].total, 39);
    }
}
