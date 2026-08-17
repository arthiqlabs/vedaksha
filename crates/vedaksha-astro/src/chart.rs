// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Chart orchestrator — top-level entry point that assembles a complete chart.
//!
//! Given pre-computed planet positions, geographic/temporal parameters, and a
//! configuration, [`compute_chart`] wires together houses, aspects, and
//! dignities into a single [`ComputedChart`] result.
//!
//! This module does **not** depend on any ephemeris provider. The caller is
//! responsible for obtaining planet positions (e.g. via an `EphemerisProvider`
//! or MCP server) and passing them in.

use crate::aspects::{Aspect, AspectType, BodyPosition, find_aspects};
use crate::dignity::{DignityPlanet, RulershipScheme, dignity_of, sign_of};
use crate::houses::{HouseCusps, HouseSystem, compute_houses};
use serde::{Deserialize, Serialize};
use vedaksha_math::angle::normalize_degrees;

/// Configuration for chart computation.
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// House system to use.
    pub house_system: HouseSystem,
    /// If `Some`, apply sidereal zodiac with the given ayanamsha.
    /// If `None`, use the tropical zodiac.
    pub ayanamsha: Option<crate::sidereal::Ayanamsha>,
    /// Which rulership scheme for essential dignities.
    pub rulership_scheme: RulershipScheme,
    /// Which aspect types to detect.
    pub aspect_types: Vec<AspectType>,
    /// Multiplier for default orbs (1.0 = standard).
    pub orb_factor: f64,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            house_system: HouseSystem::Placidus,
            ayanamsha: None, // Tropical
            rulership_scheme: RulershipScheme::Traditional,
            aspect_types: AspectType::MAJOR.to_vec(),
            orb_factor: 1.0,
        }
    }
}

/// A planet's position in the chart with all computed attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPlanet {
    /// Display name of the planet (e.g. "Sun", "Moon").
    pub name: String,
    /// Ecliptic longitude in degrees (sidereal if ayanamsha applied).
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Heliocentric distance in AU.
    pub distance: f64,
    /// Daily speed in degrees/day.
    pub speed: f64,
    /// Whether the planet is retrograde (speed < 0).
    pub retrograde: bool,
    /// Name of the zodiac sign the planet occupies.
    pub sign: String,
    /// 0-based sign index (0 = Aries .. 11 = Pisces).
    pub sign_index: u8,
    /// House number (1-12) the planet falls in.
    pub house: u8,
    /// Essential dignity state description.
    pub dignity: String,
}

/// Complete computed chart result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedChart {
    /// All planets with their computed positions, signs, houses, and dignities.
    pub planets: Vec<ChartPlanet>,
    /// The 12 house cusps (plus ASC and MC).
    pub houses: HouseCusps,
    /// All detected aspects between planets.
    pub aspects: Vec<Aspect>,
    /// Human-readable summary of the configuration used.
    pub config_summary: String,
}

/// Compute a full chart from planet positions.
///
/// # Frame
///
/// **Everything this function returns is in ONE frame**, selected by
/// `config.ayanamsha`: tropical when it is `None`, sidereal on that ayanamsha
/// when it is `Some`. That covers planet longitudes *and* the house cusps,
/// `asc` and `mc` alike.
///
/// The cusps come out of [`compute_houses`] tropically — they must, since the
/// RAMC/obliquity transform that produces them is defined against the equinox
/// of date — so on a sidereal chart they are rotated by the same ayanamsha the
/// planets get, before any house assignment or reporting. Cusps, `asc` and `mc`
/// are ecliptic longitudes, so subtracting the ayanamsha is the right operation
/// on them; the ayanamsha must **not** be subtracted from `ramc`, which is an
/// equatorial quantity feeding an obliquity transform.
///
/// Rotating planets without rotating the cusps is a frame mix, and it was the
/// live behaviour before this was fixed: with Lahiri at ~24.2° the planets were
/// displaced by 24.2/30 = 0.807 of a house relative to the cusps, so nearly
/// every graha on a sidereal chart landed in the wrong house, and the reported
/// `asc` was in a different frame from the chart's own planets. House
/// assignment is invariant under a uniform rotation of the whole zodiac — see
/// `house_assignment_is_invariant_under_ayanamsha_rotation`, which is the test
/// that pins this.
///
/// Whole-Sign is the one exception: its cusps are anchored to sign boundaries,
/// which are themselves frame dependent, so it is re-anchored on the rotated
/// ascendant rather than rotated rigidly. See the comment on step 2 and
/// `whole_sign_bhavas_follow_the_sidereal_sign_of_the_ascendant`.
///
/// # Arguments
///
/// * `planet_data` — Vec of `(name, longitude, latitude, distance, speed)`.
///   Longitudes are tropical ecliptic degrees.
/// * `ramc`        — Right Ascension of MC in degrees (equatorial, tropical
///   frame — never ayanamsha-corrected).
/// * `geo_latitude` — Geographic latitude in degrees.
/// * `obliquity`   — Obliquity of the ecliptic in degrees.
/// * `jd`          — Julian Day (for ayanamsha computation).
/// * `config`      — Chart configuration.
#[must_use]
pub fn compute_chart(
    planet_data: &[(String, f64, f64, f64, f64)],
    ramc: f64,
    geo_latitude: f64,
    obliquity: f64,
    jd: f64,
    config: &ChartConfig,
) -> ComputedChart {
    // 1. Apply ayanamsha if sidereal
    let ayanamsha_offset = config
        .ayanamsha
        .map_or(0.0, |a| crate::sidereal::ayanamsha_value(a, jd));

    // 2. Compute houses — tropically, as the RAMC/obliquity transform
    //    requires — then rotate the cusps into the chart's frame so they share
    //    it with the planet longitudes computed in step 3. Skipping this
    //    rotation is a frame mix: see the `# Frame` note above.
    let mut houses = compute_houses(ramc, geo_latitude, obliquity, config.house_system);
    if ayanamsha_offset != 0.0 {
        for cusp in &mut houses.cusps {
            *cusp = normalize_degrees(*cusp - ayanamsha_offset);
        }
        houses.asc = normalize_degrees(houses.asc - ayanamsha_offset);
        houses.mc = normalize_degrees(houses.mc - ayanamsha_offset);

        // Whole-Sign is the one system that is NOT covariant under a rigid
        // rotation of the zodiac: its cusps are anchored to sign boundaries,
        // and sign boundaries move when the frame does. Every other system
        // here derives its cusps from the ascendant and the RAMC alone, so
        // rotating them is exact — but rotating Whole-Sign leaves cusp 1 at
        // `sign_start_tropical − ayanamsha`, which is not a sign boundary in
        // the sidereal frame, and the chart would then report an `asc` in one
        // sign and a 1st house starting in another. Re-anchor on the rotated
        // ascendant, applying the module's own rule (`whole_sign::compute`:
        // cusp 1 = floor(asc/30)·30, twelve 30° houses) in the chart's frame.
        // This keeps bhava = rashi, which is what the Vedic surface means by
        // whole-sign and what `vedaksha_mcp::tools::compute_bhavas` documents.
        if houses.system == HouseSystem::WholeSign {
            let sign_start = libm::floor(houses.asc / 30.0) * 30.0;
            for (i, cusp) in houses.cusps.iter_mut().enumerate() {
                // `i` is 0..12, exactly representable in f64; no precision loss.
                let steps = f64::from(u8::try_from(i).unwrap_or(0));
                *cusp = normalize_degrees(sign_start + steps * 30.0);
            }
        }
    }
    let houses = houses;

    // 3. Build planet positions with sign, house, dignity
    let mut planets = Vec::new();
    let mut body_positions = Vec::new();

    for (name, lon, lat, dist, speed) in planet_data {
        let sidereal_lon = vedaksha_math::angle::normalize_degrees(*lon - ayanamsha_offset);
        let sign = sign_of(sidereal_lon);
        let house = determine_house(sidereal_lon, &houses);

        // Map planet name to DignityPlanet for dignity lookup
        let dignity = match name_to_dignity_planet(name) {
            Some(dp) => format!("{:?}", dignity_of(dp, sign, config.rulership_scheme)),
            None => "Peregrine".to_string(),
        };

        planets.push(ChartPlanet {
            name: name.clone(),
            longitude: sidereal_lon,
            latitude: *lat,
            distance: *dist,
            speed: *speed,
            retrograde: *speed < 0.0,
            sign: sign.name().to_string(),
            sign_index: sign as u8,
            house,
            dignity,
        });

        body_positions.push(BodyPosition {
            longitude: sidereal_lon,
            speed: *speed,
        });
    }

    // 4. Find aspects
    let aspects = find_aspects(&body_positions, &config.aspect_types, config.orb_factor);

    let config_summary = format!(
        "Houses: {:?}, Zodiac: {}, Rulership: {:?}",
        config.house_system,
        config
            .ayanamsha
            .map_or_else(|| "Tropical".to_string(), |a| format!("{a:?}")),
        config.rulership_scheme,
    );

    ComputedChart {
        planets,
        houses,
        aspects,
        config_summary,
    }
}

/// Determine which house (1-12) a planet is in based on house cusps.
#[allow(clippy::cast_possible_truncation)]
fn determine_house(longitude: f64, houses: &HouseCusps) -> u8 {
    for i in 0..12 {
        let cusp = houses.cusps[i];
        let next_cusp = houses.cusps[(i + 1) % 12];

        if next_cusp > cusp {
            // Normal case: cusp range does not cross 0°/360°
            if longitude >= cusp && longitude < next_cusp {
                return (i as u8) + 1;
            }
        } else {
            // Wraps around 0°/360°
            if longitude >= cusp || longitude < next_cusp {
                return (i as u8) + 1;
            }
        }
    }
    1 // fallback
}

/// Map a planet name string to its [`DignityPlanet`] variant.
fn name_to_dignity_planet(name: &str) -> Option<DignityPlanet> {
    match name.to_lowercase().as_str() {
        "sun" => Some(DignityPlanet::Sun),
        "moon" => Some(DignityPlanet::Moon),
        "mars" => Some(DignityPlanet::Mars),
        "mercury" => Some(DignityPlanet::Mercury),
        "jupiter" => Some(DignityPlanet::Jupiter),
        "venus" => Some(DignityPlanet::Venus),
        "saturn" => Some(DignityPlanet::Saturn),
        "uranus" => Some(DignityPlanet::Uranus),
        "neptune" => Some(DignityPlanet::Neptune),
        "pluto" => Some(DignityPlanet::Pluto),
        _ => None,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a planet tuple.
    fn planet(name: &str, lon: f64, speed: f64) -> (String, f64, f64, f64, f64) {
        (name.to_string(), lon, 0.0, 1.0, speed)
    }

    /// Standard test parameters: RAMC=30, lat=45, obliquity=23.44, JD=J2000.
    const RAMC: f64 = 30.0;
    const LAT: f64 = 45.0;
    const OBL: f64 = 23.44;
    const JD: f64 = 2_451_545.0;

    #[test]
    fn compute_chart_returns_planets_and_houses() {
        let data = vec![planet("Sun", 130.0, 1.0), planet("Moon", 45.0, 13.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets.len(), 2);
        assert_eq!(chart.houses.cusps.len(), 12);
        assert!(!chart.config_summary.is_empty());
    }

    #[test]
    fn planets_have_correct_sign_assignments() {
        // 130° = Leo (sign index 4)
        let data = vec![planet("Sun", 130.0, 1.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].sign, "Leo");
        assert_eq!(chart.planets[0].sign_index, 4);
    }

    #[test]
    fn retrograde_flag_from_negative_speed() {
        let data = vec![planet("Mars", 200.0, -0.3), planet("Venus", 100.0, 1.2)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert!(chart.planets[0].retrograde, "Mars should be retrograde");
        assert!(!chart.planets[1].retrograde, "Venus should be direct");
    }

    #[test]
    fn aspects_detected_between_planets() {
        // Sun at 0° and Moon at 120° should form a trine
        let data = vec![planet("Sun", 0.0, 1.0), planet("Moon", 120.0, 13.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert!(
            chart
                .aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Trine),
            "Expected a trine between Sun and Moon"
        );
    }

    #[test]
    fn sidereal_mode_shifts_longitudes() {
        let data = vec![planet("Sun", 130.0, 1.0)];
        let mut config = ChartConfig::default();
        config.ayanamsha = Some(crate::sidereal::Ayanamsha::Lahiri);

        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &config);

        // Lahiri ayanamsha at J2000 is ~23.856°, so 130 − 23.856 ≈ 106.144°
        let expected = vedaksha_math::angle::normalize_degrees(
            130.0 - crate::sidereal::ayanamsha_value(crate::sidereal::Ayanamsha::Lahiri, JD),
        );
        assert!(
            (chart.planets[0].longitude - expected).abs() < 0.01,
            "Sidereal longitude mismatch: got {}, expected {}",
            chart.planets[0].longitude,
            expected
        );
    }

    /// House assignment must be **invariant under a uniform rotation of the
    /// zodiac**.
    ///
    /// Switching from tropical to sidereal rotates the whole frame by one
    /// constant, the ayanamsha. A rotation cannot change which house a planet
    /// occupies — it moves the planet and the cusp that bounds it by the same
    /// amount. So for the same instant, place and house system,
    /// `ayanamsha: None` and `ayanamsha: Some(Lahiri)` must return identical
    /// house numbers for every planet, while every reported longitude — planet,
    /// cusp, `asc`, `mc` alike — differs by exactly the ayanamsha.
    ///
    /// The pre-fix code rotated the planets and left the cusps tropical, so
    /// planets slid 24.2/30 = 0.807 of a house against a fixed grid and the
    /// invariance broke for most of them. This property is exact, needs no
    /// external oracle, and the bug cannot satisfy it.
    ///
    /// Whole-Sign is deliberately excluded here and tested separately: its
    /// cusps are anchored to sign boundaries, which are themselves frame
    /// dependent, so it is the one system that is genuinely NOT covariant
    /// under this rotation. See
    /// `whole_sign_bhavas_follow_the_sidereal_sign_of_the_ascendant`.
    ///
    /// The latitude grid runs from the equator to ±85°, crossing
    /// `houses::POLAR_LAT_THRESHOLD` (66.56°) in both hemispheres so the
    /// Placidus/Koch fallback to Equal houses is covered rather than merely
    /// assumed rotation-safe. It is: Equal cusps are `asc + k·30°`, which
    /// commutes with a rigid rotation. Measured, 0 mismatches across all
    /// 9 systems at every polar latitude in the grid.
    #[test]
    fn house_assignment_is_invariant_under_ayanamsha_rotation() {
        use crate::sidereal::{Ayanamsha, ayanamsha_value};

        // Every house system whose cusps are a function of the ascendant and
        // the RAMC alone — i.e. every one except Whole-Sign.
        let systems = [
            HouseSystem::Placidus,
            HouseSystem::Koch,
            HouseSystem::Equal,
            HouseSystem::Campanus,
            HouseSystem::Regiomontanus,
            HouseSystem::Porphyry,
            HouseSystem::Morinus,
            HouseSystem::Alcabitius,
            HouseSystem::Sripathi,
        ];
        // (ramc, latitude, jd) — different epochs so the ayanamsha differs,
        // and latitudes from the equator through the polar circle to 85°.
        //
        // The last six straddle `houses::POLAR_LAT_THRESHOLD` = 66.56°, above
        // which Placidus and Koch abandon the semi-arc method and fall back to
        // Equal houses with `polar_fallback = true`. That fallback path was
        // uncovered here while the grid stopped at 60°: the substituted system
        // is a *different* function of the ascendant, so nothing in this test
        // had shown it is rotation-covariant. It is — Equal cusps are
        // asc + k·30°, which commutes with a rigid rotation — and the
        // `polar_fallback_exercised` assertion below pins that the path is
        // genuinely entered rather than merely available.
        //
        // Both hemispheres and both sides of the threshold are present so a
        // sign error in the `libm::fabs(latitude) > POLAR_LAT_THRESHOLD` guard
        // cannot hide. 66.0° is deliberately *below* the 66.56° threshold, not
        // at the 66.5° Arctic circle, so it is a genuine no-fallback control.
        let charts = [
            (30.0, 0.0, 2_433_282.5),       // 1950-01-01 UT, equator
            (177.17, 28.6139, JD),          // 2000-01-01 UT, Delhi
            (264.78, 40.7128, 2_461_100.0), // 2026-02-28 UT, New York
            (95.0, 60.0, 2_461_100.0),      // high but sub-polar
            (95.0, 66.0, 2_461_100.0),      // just below the 66.56° threshold
            (95.0, 67.0, 2_461_100.0),      // just above it — fallback engages
            (211.4, 75.0, 2_433_282.5),     // deep Arctic
            (348.9, 85.0, JD),              // near the north pole
            (211.4, -67.0, JD),             // southern mirror, just past
            (348.9, -85.0, 2_461_100.0),    // near the south pole
        ];
        // Set when a chart in the grid actually took the Placidus/Koch polar
        // substitution. Asserted non-zero at the end: without it, raising
        // POLAR_LAT_THRESHOLD past 85° would silently un-cover the path again
        // while this test stayed green.
        let mut polar_fallback_exercised = 0_usize;
        // 24 planets at 15° spacing, so every house is populated and the
        // 0°/360° wrap is crossed.
        let data: Vec<_> = (0..24)
            .map(|i| planet("Sun", f64::from(i) * 15.0, 1.0))
            .collect();

        for system in systems {
            for (ramc, lat, jd) in charts {
                let ayan = ayanamsha_value(Ayanamsha::Lahiri, jd);
                let mut trop_cfg = ChartConfig {
                    house_system: system,
                    ..ChartConfig::default()
                };
                trop_cfg.ayanamsha = None;
                let mut sid_cfg = trop_cfg.clone();
                sid_cfg.ayanamsha = Some(Ayanamsha::Lahiri);

                let trop = compute_chart(&data, ramc, lat, OBL, jd, &trop_cfg);
                let sid = compute_chart(&data, ramc, lat, OBL, jd, &sid_cfg);

                // The fallback decision is a function of latitude alone, so it
                // must be identical in both frames. If it were not, the two
                // charts would be comparing different house systems and every
                // assertion below would be meaningless.
                assert_eq!(
                    trop.houses.polar_fallback, sid.houses.polar_fallback,
                    "{system:?} @ramc {ramc} lat {lat} jd {jd}: polar_fallback \
                     differs between frames ({} vs {}) — the two charts are not \
                     the same house system and cannot be compared",
                    trop.houses.polar_fallback, sid.houses.polar_fallback
                );
                if trop.houses.polar_fallback {
                    polar_fallback_exercised += 1;
                }

                // Longitudes rotate by exactly the ayanamsha.
                for (t, s) in trop.planets.iter().zip(sid.planets.iter()) {
                    let expected = normalize_degrees(t.longitude - ayan);
                    assert!(
                        libm::fabs(s.longitude - expected) < 1e-9,
                        "{system:?} @ramc {ramc} lat {lat} jd {jd}: planet longitude \
                         {} should rotate to {expected}, got {}",
                        t.longitude,
                        s.longitude
                    );
                }
                for i in 0..12 {
                    let expected = normalize_degrees(trop.houses.cusps[i] - ayan);
                    assert!(
                        libm::fabs(sid.houses.cusps[i] - expected) < 1e-9,
                        "{system:?} @ramc {ramc} lat {lat} jd {jd}: cusp {} should \
                         rotate to {expected}, got {}. A cusp that did NOT move is \
                         the frame mix this test exists to catch.",
                        i + 1,
                        sid.houses.cusps[i]
                    );
                }
                for (label, t, s) in [
                    ("asc", trop.houses.asc, sid.houses.asc),
                    ("mc", trop.houses.mc, sid.houses.mc),
                ] {
                    let expected = normalize_degrees(t - ayan);
                    assert!(
                        libm::fabs(s - expected) < 1e-9,
                        "{system:?} @ramc {ramc} lat {lat} jd {jd}: {label} should \
                         rotate to {expected}, got {s}"
                    );
                }

                // The property itself: identical house numbers.
                for (t, s) in trop.planets.iter().zip(sid.planets.iter()) {
                    assert_eq!(
                        t.house, s.house,
                        "{system:?} @ramc {ramc} lat {lat} jd {jd}: planet at \
                         tropical {}° is in house {} tropically but house {} \
                         sidereally — a uniform rotation of the zodiac cannot \
                         move a planet between houses",
                        t.longitude, t.house, s.house
                    );
                }
            }
        }

        // Derived, not observed-and-copied. Of the 10 charts in the grid, 5
        // sit above the 66.56° threshold — 67, 75, 85, −67, −85; the sixth
        // added latitude, 66.0, is deliberately below it. Exactly 2 of the 9
        // systems substitute Equal up there, Placidus and Koch; the other
        // seven are closed-form in the RAMC and the ascendant and never set
        // the flag. 5 × 2 = 10.
        //
        // The equality (rather than `> 0`) is what pins the reviewer's
        // finding: a count above 10 would mean a system started falling back
        // that previously did not, and a count below 10 that the grid or the
        // threshold moved.
        assert_eq!(
            polar_fallback_exercised, 10,
            "expected the Placidus/Koch polar substitution on 5 polar charts × \
             2 systems = 10 charts; saw {polar_fallback_exercised}. A count of 0 \
             means the grid no longer reaches past POLAR_LAT_THRESHOLD and the \
             fallback path is uncovered again."
        );
    }

    /// Whole-Sign on a sidereal chart must anchor on the **sidereal** sign of
    /// the ascendant — bhava = rashi, the meaning
    /// `vedaksha_mcp::tools::compute_bhavas` documents ("the entire sign
    /// containing the ascendant is the 1st bhava").
    ///
    /// This is the one house system that is not covariant under the rotation
    /// checked in `house_assignment_is_invariant_under_ayanamsha_rotation`,
    /// because its cusps are sign boundaries and sign boundaries are frame
    /// dependent. Rotating the tropical cusps rigidly would leave cusp 1 at
    /// `floor(asc_tropical/30)·30 − ayanamsha`, which is not a sign boundary in
    /// the sidereal frame — the chart would report `asc` in one sign and start
    /// the 1st house in another. Leaving the cusps tropical (the pre-fix
    /// behaviour) is worse still. Both are caught here.
    ///
    /// Exact, no oracle needed: the reported `asc` is the tropical `asc`
    /// rotated by the ayanamsha, cusp 1 is a multiple of 30°, cusps are 30°
    /// apart, and a planet's house is fixed by sign arithmetic alone —
    /// `house = 1 + (sign(planet) − sign(asc)) mod 12`.
    ///
    /// The frame assertion is load-bearing and was added after measuring that
    /// without it this test stays GREEN on the pre-fix code: whole-sign cusps
    /// sit on tropical sign boundaries, so sign arithmetic against a tropical
    /// cusp 1 reproduces `determine_house` even when the planets are sidereal.
    /// Only comparing `asc` against the tropical chart exposes the mix.
    #[test]
    fn whole_sign_bhavas_follow_the_sidereal_sign_of_the_ascendant() {
        use crate::sidereal::{Ayanamsha, ayanamsha_value};

        let charts = [
            (30.0, 0.0, 2_433_282.5),
            (177.17, 28.6139, JD),
            (264.78, 40.7128, 2_461_100.0),
            (95.0, 60.0, 2_461_100.0),
        ];
        let data: Vec<_> = (0..24)
            .map(|i| planet("Sun", f64::from(i) * 15.0, 1.0))
            .collect();

        let cfg = ChartConfig {
            house_system: HouseSystem::WholeSign,
            ayanamsha: Some(Ayanamsha::Lahiri),
            ..ChartConfig::default()
        };

        let mut trop_cfg = cfg.clone();
        trop_cfg.ayanamsha = None;

        for (ramc, lat, jd) in charts {
            let chart = compute_chart(&data, ramc, lat, OBL, jd, &cfg);
            let asc = chart.houses.asc;

            // The reported asc must be in the SAME frame as the planets: the
            // tropical asc rotated by the ayanamsha. Without this, a chart that
            // left asc/cusps tropical while rotating the planets would satisfy
            // every other assertion below.
            let trop = compute_chart(&data, ramc, lat, OBL, jd, &trop_cfg);
            let ayan = ayanamsha_value(Ayanamsha::Lahiri, jd);
            let expected_asc = normalize_degrees(trop.houses.asc - ayan);
            assert!(
                libm::fabs(asc - expected_asc) < 1e-9,
                "@ramc {ramc} lat {lat} jd {jd}: sidereal asc should be the \
                 tropical asc {} rotated by the ayanamsha {ayan} = {expected_asc}, \
                 got {asc}. An asc that did not move is a frame mix against the \
                 chart's own sidereal planets.",
                trop.houses.asc
            );

            // Cusp 1 is the start of the sidereal sign holding the sidereal asc.
            let expected_cusp1 = libm::floor(asc / 30.0) * 30.0;
            assert!(
                libm::fabs(chart.houses.cusps[0] - expected_cusp1) < 1e-9,
                "@ramc {ramc} lat {lat} jd {jd}: sidereal asc {asc}° sits in the \
                 sign starting at {expected_cusp1}°, but cusp 1 is {}°",
                chart.houses.cusps[0]
            );

            // Twelve 30° houses.
            for i in 0..12 {
                let gap =
                    normalize_degrees(chart.houses.cusps[(i + 1) % 12] - chart.houses.cusps[i]);
                assert!(
                    libm::fabs(gap - 30.0) < 1e-9,
                    "@ramc {ramc} lat {lat} jd {jd}: cusp {} → {} gap is {gap}°, not 30°",
                    i + 1,
                    (i + 1) % 12 + 1
                );
            }

            // House number is pure sign arithmetic against the asc's sign.
            let asc_sign = chart.houses.cusps[0] / 30.0;
            for p in &chart.planets {
                let planet_sign = libm::floor(p.longitude / 30.0);
                // Both operands are in 0..12 and integral in f64; the modulo
                // keeps the result in 0..12, so the u8 conversion is exact and
                // cannot truncate.
                let offset = (planet_sign - asc_sign + 12.0) % 12.0;
                let expected_house = (offset as u8) + 1;
                assert_eq!(
                    p.house, expected_house,
                    "@ramc {ramc} lat {lat} jd {jd}: planet at sidereal {}° \
                     (sign {planet_sign}) with asc sign {asc_sign} belongs in \
                     bhava {expected_house}, got {}",
                    p.longitude, p.house
                );
            }
        }
    }

    #[test]
    fn dignity_computed_for_known_planet() {
        // Sun at 130° (Leo) should have Domicile dignity
        let data = vec![planet("Sun", 130.0, 1.0)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].dignity, "Domicile");
    }

    #[test]
    fn unknown_body_gets_peregrine() {
        let data = vec![planet("Chiron", 100.0, 0.05)];
        let chart = compute_chart(&data, RAMC, LAT, OBL, JD, &ChartConfig::default());

        assert_eq!(chart.planets[0].dignity, "Peregrine");
    }

    #[test]
    fn determine_house_basic() {
        // Create simple equal house cusps at 0, 30, 60, ...
        let cusps: [f64; 12] = core::array::from_fn(|i| (i as f64) * 30.0);
        let houses = HouseCusps {
            cusps,
            asc: 0.0,
            mc: 270.0,
            system: HouseSystem::Equal,
            polar_fallback: false,
        };

        assert_eq!(determine_house(15.0, &houses), 1);
        assert_eq!(determine_house(45.0, &houses), 2);
        assert_eq!(determine_house(350.0, &houses), 12);
    }
}
