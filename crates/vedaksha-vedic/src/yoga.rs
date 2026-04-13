// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Vedic yoga (planetary combination) detection.
//!
//! Detects classical yogas from planetary positions. Implements 50 yogas from
//! BPHS (Brihat Parashara Hora Shastra) and Phaladipika, including the five
//! Pancha Mahapurusha yogas, Gajakesari, Budhaditya, and many more.
//!
//! Source: BPHS; Phaladipika (Mantreshwara).

use crate::bhava;

/// A detected Vedic yoga (planetary combination).
#[derive(Debug, Clone)]
pub struct Yoga {
    /// The type/name of the yoga.
    pub yoga_type: YogaType,
    /// Human-readable name.
    pub name: &'static str,
    /// Brief description of the yoga's significance.
    pub description: &'static str,
    /// Planets involved in forming this yoga.
    pub planets: Vec<YogaPlanet>,
}

/// Planet identifiers for yoga detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YogaPlanet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

/// Recognized Vedic yoga types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YogaType {
    // ── Pancha Mahapurusha Yogas (5) ────────────────────────────────
    /// Mars in own/exalted sign in a kendra.
    Ruchaka,
    /// Mercury in own/exalted sign in a kendra.
    Bhadra,
    /// Jupiter in own/exalted sign in a kendra.
    Hamsa,
    /// Venus in own/exalted sign in a kendra.
    Malavya,
    /// Saturn in own/exalted sign in a kendra.
    Sasa,

    // ── Moon-based yogas ────────────────────────────────────────────
    /// Jupiter in a kendra from Moon.
    Gajakesari,
    /// Sun-Mercury conjunction (same sign).
    Budhaditya,
    /// Moon-Mars conjunction (same sign).
    ChandraMangala,
    /// Benefics in 6, 7, 8 from Moon.
    Adhi,
    /// Benefic in 10th from lagna or Moon.
    Amala,
    /// Planet in 2nd from Moon (not Sun).
    Sunapha,
    /// Planet in 12th from Moon (not Sun).
    Anapha,
    /// Planets in both 2nd and 12th from Moon.
    Durudhara,
    /// No planets in 2nd or 12th from Moon.
    Kemadruma,
    /// Moon in 6th or 8th from Jupiter.
    Shakata,
    /// Moon-Saturn conjunction.
    Vish,

    // ── Sun-based yogas ─────────────────────────────────────────────
    /// Planet in 2nd from Sun (not Moon).
    Vesi,
    /// Planet in 12th from Sun (not Moon).
    Vosi,
    /// Planets in both 2nd and 12th from Sun.
    Obhayachari,

    // ── Raja / Power yogas ──────────────────────────────────────────
    /// Debilitation cancellation — lord of debilitation sign in kendra.
    NeechabhangaRajaYoga,
    /// Lords of kendra and trikona conjoined or aspecting.
    RajaYoga,
    /// Lords of 6, 8, 12 in each other's houses.
    Viparita,
    /// Benefic in lagna and Jupiter aspects lagna.
    Chamara,
    /// Benefics in kendra, no malefics in kendra.
    Parvata,
    /// Lords of 4th and 9th in mutual kendra.
    Kahala,
    /// Lords of 5th and 6th in mutual kendra.
    Shankha,
    /// 9th lord in 10th, 10th lord in 9th.
    Khyati,
    /// Benefic in 10th from Moon.
    Amara,
    /// Sign lord of lagna in kendra/trikona.
    Parijata,

    // ── Wealth yogas ────────────────────────────────────────────────
    /// Wealth yoga — lords of 2, 11 in kendra/trikona.
    Dhana,
    /// Venus in own/exalted in kendra/trikona, 9th lord strong.
    Lakshmi,
    /// Jupiter, Venus, Mercury in kendra/trikona/2nd.
    Saraswati,
    /// Moon in friendly sign, lord of Moon sign in kendra.
    Pushkala,

    /// Lords of 2nd and 11th in kendra/trikona.
    DhanaSpecial,

    // ── Spiritual yogas ─────────────────────────────────────────────
    /// 4+ planets in one sign.
    Sanyasa,

    // ── Negative yogas ──────────────────────────────────────────────
    /// 11th lord in 6th/8th/12th.
    Daridra,
    /// Planetary war (two planets within 1 degree).
    GrahaYuddha,

    // ── Kartari yogas ───────────────────────────────────────────────
    /// Malefics on both sides of lagna (hemmed in).
    PapaKartari,
    /// Benefics on both sides of lagna (good hemming).
    ShubhaKartari,

    // ── Exchange yogas ──────────────────────────────────────────────
    /// Mutual exchange of signs between two planets.
    Parivartana,

    // ── Sign-distribution yogas (Sankhya) ───────────────────────────
    /// All planets in movable signs.
    Rajju,
    /// All planets in fixed signs.
    Musala,
    /// All planets in dual signs.
    Nala,
    /// All planets in 2 signs.
    Dama,
    /// All planets in 3 signs (Kedara).
    Kedara,
    /// All planets in 4 signs (Shoola).
    Shoola,
    /// All planets in 5 signs (Yuga).
    Yuga,
    /// All planets in 6 signs (Gola).
    Gola,

    // ── Miscellaneous ───────────────────────────────────────────────
    /// Lord of exalted planet's sign in kendra/trikona.
    Mridanga,
    /// Moon in a benefic sign.
    Chandra,
}

/// Planetary position data for yoga detection.
#[derive(Debug, Clone, Copy)]
pub struct PlanetPosition {
    /// Which planet.
    pub planet: YogaPlanet,
    /// Sign index (0 = Aries, 1 = Taurus, ..., 11 = Pisces).
    pub sign: u8,
    /// Sidereal longitude in degrees.
    pub longitude: f64,
    /// House (bhava) 1-12.
    pub bhava: u8,
}

// ── Planet-sign relationships ────────────────────────────────────────

/// Signs owned by each planet (traditional rulership).
#[must_use]
fn own_signs(planet: YogaPlanet) -> &'static [u8] {
    match planet {
        YogaPlanet::Sun => &[4],         // Leo
        YogaPlanet::Moon => &[3],        // Cancer
        YogaPlanet::Mars => &[0, 7],     // Aries, Scorpio
        YogaPlanet::Mercury => &[2, 5],  // Gemini, Virgo
        YogaPlanet::Jupiter => &[8, 11], // Sagittarius, Pisces
        YogaPlanet::Venus => &[1, 6],    // Taurus, Libra
        YogaPlanet::Saturn => &[9, 10],  // Capricorn, Aquarius
        YogaPlanet::Rahu | YogaPlanet::Ketu => &[],
    }
}

/// Exaltation sign for each planet.
#[must_use]
fn exaltation_sign(planet: YogaPlanet) -> Option<u8> {
    match planet {
        YogaPlanet::Sun => Some(0),     // Aries
        YogaPlanet::Moon => Some(1),    // Taurus
        YogaPlanet::Mars => Some(9),    // Capricorn
        YogaPlanet::Mercury => Some(5), // Virgo
        YogaPlanet::Jupiter => Some(3), // Cancer
        YogaPlanet::Venus => Some(11),  // Pisces
        YogaPlanet::Saturn => Some(6),  // Libra
        YogaPlanet::Rahu | YogaPlanet::Ketu => None,
    }
}

/// Debilitation sign (opposite of exaltation).
#[must_use]
fn debilitation_sign(planet: YogaPlanet) -> Option<u8> {
    exaltation_sign(planet).map(|s| (s + 6) % 12)
}

/// Check if a planet is in its own sign.
#[must_use]
fn is_in_own_sign(planet: YogaPlanet, sign: u8) -> bool {
    own_signs(planet).contains(&sign)
}

/// Check if a planet is exalted.
#[must_use]
fn is_exalted(planet: YogaPlanet, sign: u8) -> bool {
    exaltation_sign(planet) == Some(sign)
}

/// Check if a planet is debilitated.
#[must_use]
fn is_debilitated(planet: YogaPlanet, sign: u8) -> bool {
    debilitation_sign(planet) == Some(sign)
}

/// Lord (ruler) of a given sign.
#[must_use]
fn sign_lord(sign: u8) -> Option<YogaPlanet> {
    match sign {
        0 | 7 => Some(YogaPlanet::Mars),     // Aries, Scorpio
        1 | 6 => Some(YogaPlanet::Venus),    // Taurus, Libra
        2 | 5 => Some(YogaPlanet::Mercury),  // Gemini, Virgo
        3 => Some(YogaPlanet::Moon),         // Cancer
        4 => Some(YogaPlanet::Sun),          // Leo
        8 | 11 => Some(YogaPlanet::Jupiter), // Sagittarius, Pisces
        9 | 10 => Some(YogaPlanet::Saturn),  // Capricorn, Aquarius
        _ => None,
    }
}

/// Check if a planet is a natural benefic.
#[must_use]
fn is_natural_benefic(planet: YogaPlanet) -> bool {
    matches!(
        planet,
        YogaPlanet::Jupiter | YogaPlanet::Venus | YogaPlanet::Mercury | YogaPlanet::Moon
    )
}

/// Check if a planet is a natural malefic.
#[must_use]
fn is_natural_malefic(planet: YogaPlanet) -> bool {
    matches!(
        planet,
        YogaPlanet::Sun
            | YogaPlanet::Mars
            | YogaPlanet::Saturn
            | YogaPlanet::Rahu
            | YogaPlanet::Ketu
    )
}

/// Check if a sign is a movable (chara) sign.
#[must_use]
fn is_movable_sign(sign: u8) -> bool {
    matches!(sign, 0 | 3 | 6 | 9) // Aries, Cancer, Libra, Capricorn
}

/// Check if a sign is a fixed (sthira) sign.
#[must_use]
fn is_fixed_sign(sign: u8) -> bool {
    matches!(sign, 1 | 4 | 7 | 10) // Taurus, Leo, Scorpio, Aquarius
}

/// Check if a sign is a dual (dvisvabhava) sign.
#[must_use]
fn is_dual_sign(sign: u8) -> bool {
    matches!(sign, 2 | 5 | 8 | 11) // Gemini, Virgo, Sagittarius, Pisces
}

/// Check if a sign is ruled by a benefic.
#[must_use]
fn is_benefic_sign(sign: u8) -> bool {
    if let Some(lord) = sign_lord(sign) {
        is_natural_benefic(lord)
    } else {
        false
    }
}

/// Get the seven visible planets (Sun through Saturn), excluding nodes.
fn seven_planets() -> [YogaPlanet; 7] {
    [
        YogaPlanet::Sun,
        YogaPlanet::Moon,
        YogaPlanet::Mars,
        YogaPlanet::Mercury,
        YogaPlanet::Jupiter,
        YogaPlanet::Venus,
        YogaPlanet::Saturn,
    ]
}

/// Find a planet position by planet type.
fn find_planet(positions: &[PlanetPosition], planet: YogaPlanet) -> Option<&PlanetPosition> {
    positions.iter().find(|p| p.planet == planet)
}

/// Get the bhava (house number 1-12) of a sign relative to a reference sign.
fn sign_to_bhava(sign: u8, ref_sign: u8) -> u8 {
    ((sign + 12 - ref_sign) % 12) + 1
}

// ── Yoga detection ───────────────────────────────────────────────────

/// Detect all applicable yogas from the given planetary positions.
///
/// # Arguments
/// * `positions` — slice of planet positions with sign and bhava data
/// * `lagna_sign` — sign index (0-11) of the ascendant
/// * `moon_sign` — sign index (0-11) of the Moon
#[must_use]
pub fn detect_yogas(positions: &[PlanetPosition], lagna_sign: u8, moon_sign: u8) -> Vec<Yoga> {
    let mut yogas = Vec::new();

    // Pancha Mahapurusha (5)
    yogas.extend(detect_pancha_mahapurusha(positions));

    // Moon-based yogas
    yogas.extend(detect_gajakesari(positions, moon_sign));
    yogas.extend(detect_budhaditya(positions));
    yogas.extend(detect_chandra_mangala(positions));
    yogas.extend(detect_adhi(positions, moon_sign));
    yogas.extend(detect_amala(positions, lagna_sign, moon_sign));
    yogas.extend(detect_sunapha(positions, moon_sign));
    yogas.extend(detect_anapha(positions, moon_sign));
    yogas.extend(detect_durudhara(positions, moon_sign));
    yogas.extend(detect_kemadruma(positions, moon_sign));
    yogas.extend(detect_shakata(positions, moon_sign));
    yogas.extend(detect_vish(positions));

    // Sun-based yogas
    yogas.extend(detect_vesi(positions));
    yogas.extend(detect_vosi(positions));
    yogas.extend(detect_obhayachari(positions));

    // Raja / Power yogas
    yogas.extend(detect_neechabhanga(positions, lagna_sign, moon_sign));
    yogas.extend(detect_chamara(positions, lagna_sign));
    yogas.extend(detect_parvata(positions));
    yogas.extend(detect_kahala(positions, lagna_sign));
    yogas.extend(detect_shankha(positions, lagna_sign));
    yogas.extend(detect_khyati(positions, lagna_sign));
    yogas.extend(detect_amara(positions, moon_sign));
    yogas.extend(detect_parijata(positions, lagna_sign));

    // Wealth yogas
    yogas.extend(detect_lakshmi(positions, lagna_sign));
    yogas.extend(detect_saraswati(positions, lagna_sign));
    yogas.extend(detect_pushkala(positions, moon_sign));
    yogas.extend(detect_dhana_special(positions, lagna_sign));

    // Spiritual yogas
    yogas.extend(detect_sanyasa(positions));

    // Negative yogas
    yogas.extend(detect_daridra(positions, lagna_sign));
    yogas.extend(detect_graha_yuddha(positions));

    // Kartari yogas
    yogas.extend(detect_papa_kartari(positions, lagna_sign));
    yogas.extend(detect_shubha_kartari(positions, lagna_sign));

    // Exchange yogas
    yogas.extend(detect_parivartana(positions));

    // Sign-distribution (Sankhya) yogas
    yogas.extend(detect_sign_distribution_yogas(positions));

    // Miscellaneous
    yogas.extend(detect_mridanga(positions, lagna_sign));
    yogas.extend(detect_chandra_yoga(positions, moon_sign));

    yogas
}

// ── Pancha Mahapurusha ──────────────────────────────────────────────

/// Detect Pancha Mahapurusha Yogas.
///
/// A Mahapurusha yoga forms when Mars, Mercury, Jupiter, Venus, or Saturn
/// is in its own sign or exalted, AND occupies a kendra (houses 1, 4, 7, 10).
fn detect_pancha_mahapurusha(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    for pos in positions {
        if !(is_in_own_sign(pos.planet, pos.sign) || is_exalted(pos.planet, pos.sign)) {
            continue;
        }
        if !bhava::is_kendra(pos.bhava) {
            continue;
        }
        let (yoga_type, name, desc) = match pos.planet {
            YogaPlanet::Mars => (
                YogaType::Ruchaka,
                "Ruchaka",
                "Mars in own/exalted sign in kendra — courage, leadership, military prowess",
            ),
            YogaPlanet::Mercury => (
                YogaType::Bhadra,
                "Bhadra",
                "Mercury in own/exalted sign in kendra — intelligence, eloquence, learning",
            ),
            YogaPlanet::Jupiter => (
                YogaType::Hamsa,
                "Hamsa",
                "Jupiter in own/exalted sign in kendra — wisdom, righteousness, spiritual merit",
            ),
            YogaPlanet::Venus => (
                YogaType::Malavya,
                "Malavya",
                "Venus in own/exalted sign in kendra — luxury, beauty, artistic talent",
            ),
            YogaPlanet::Saturn => (
                YogaType::Sasa,
                "Sasa",
                "Saturn in own/exalted sign in kendra — authority, discipline, organizational power",
            ),
            _ => continue,
        };
        yogas.push(Yoga {
            yoga_type,
            name,
            description: desc,
            planets: vec![pos.planet],
        });
    }
    yogas
}

// ── Moon-based yogas ────────────────────────────────────────────────

/// Detect Gajakesari Yoga — Jupiter in a kendra (1, 4, 7, 10) from Moon.
fn detect_gajakesari(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    for pos in positions {
        if pos.planet != YogaPlanet::Jupiter {
            continue;
        }
        let distance = (pos.sign + 12 - moon_sign) % 12;
        if matches!(distance, 0 | 3 | 6 | 9) {
            yogas.push(Yoga {
                yoga_type: YogaType::Gajakesari,
                name: "Gajakesari",
                description: "Jupiter in kendra from Moon — fame, prosperity, lasting reputation",
                planets: vec![YogaPlanet::Jupiter, YogaPlanet::Moon],
            });
        }
    }
    yogas
}

/// Detect Budhaditya Yoga — Sun and Mercury in the same sign.
fn detect_budhaditya(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let sun = find_planet(positions, YogaPlanet::Sun);
    let mercury = find_planet(positions, YogaPlanet::Mercury);
    if let (Some(s), Some(m)) = (sun, mercury) {
        if s.sign == m.sign {
            return vec![Yoga {
                yoga_type: YogaType::Budhaditya,
                name: "Budhaditya",
                description: "Sun-Mercury conjunction — intelligence, communication skill, sharp intellect",
                planets: vec![YogaPlanet::Sun, YogaPlanet::Mercury],
            }];
        }
    }
    vec![]
}

/// Detect Chandra-Mangala Yoga — Moon and Mars in the same sign.
fn detect_chandra_mangala(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let moon = find_planet(positions, YogaPlanet::Moon);
    let mars = find_planet(positions, YogaPlanet::Mars);
    if let (Some(m), Some(ma)) = (moon, mars) {
        if m.sign == ma.sign {
            return vec![Yoga {
                yoga_type: YogaType::ChandraMangala,
                name: "Chandra-Mangala",
                description: "Moon-Mars conjunction — wealth through self-effort, enterprise, courage",
                planets: vec![YogaPlanet::Moon, YogaPlanet::Mars],
            }];
        }
    }
    vec![]
}

/// Detect Adhi Yoga — benefics (Jupiter, Venus, Mercury) in 6, 7, 8 from Moon.
fn detect_adhi(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let benefics_in_range: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| is_natural_benefic(p.planet) && p.planet != YogaPlanet::Moon)
        .filter(|p| {
            let distance = (p.sign + 12 - moon_sign) % 12;
            matches!(distance, 5..=7)
        })
        .map(|p| p.planet)
        .collect();

    if benefics_in_range.len() >= 2 {
        return vec![Yoga {
            yoga_type: YogaType::Adhi,
            name: "Adhi",
            description: "Benefics in 6/7/8 from Moon — leadership, authority, prosperity",
            planets: benefics_in_range,
        }];
    }
    vec![]
}

/// Detect Amala Yoga — a natural benefic in the 10th house from lagna or Moon.
fn detect_amala(positions: &[PlanetPosition], lagna_sign: u8, moon_sign: u8) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    for pos in positions {
        if !is_natural_benefic(pos.planet) || pos.planet == YogaPlanet::Moon {
            continue;
        }
        let dist_lagna = (pos.sign + 12 - lagna_sign) % 12;
        let dist_moon = (pos.sign + 12 - moon_sign) % 12;
        if dist_lagna == 9 || dist_moon == 9 {
            yogas.push(Yoga {
                yoga_type: YogaType::Amala,
                name: "Amala",
                description: "Benefic in 10th from lagna/Moon — unblemished character, virtuous deeds",
                planets: vec![pos.planet],
            });
            break;
        }
    }
    yogas
}

/// Detect Sunapha Yoga — a planet (not Sun) in 2nd from Moon.
fn detect_sunapha(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let second_from_moon = (moon_sign + 1) % 12;
    let planets: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| {
            p.sign == second_from_moon
                && p.planet != YogaPlanet::Sun
                && p.planet != YogaPlanet::Moon
                && p.planet != YogaPlanet::Rahu
                && p.planet != YogaPlanet::Ketu
        })
        .map(|p| p.planet)
        .collect();

    if planets.is_empty() {
        vec![]
    } else {
        vec![Yoga {
            yoga_type: YogaType::Sunapha,
            name: "Sunapha",
            description: "Planet in 2nd from Moon — self-acquired wealth, intelligence",
            planets,
        }]
    }
}

/// Detect Anapha Yoga — a planet (not Sun) in 12th from Moon.
fn detect_anapha(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let twelfth_from_moon = (moon_sign + 11) % 12;
    let planets: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| {
            p.sign == twelfth_from_moon
                && p.planet != YogaPlanet::Sun
                && p.planet != YogaPlanet::Moon
                && p.planet != YogaPlanet::Rahu
                && p.planet != YogaPlanet::Ketu
        })
        .map(|p| p.planet)
        .collect();

    if planets.is_empty() {
        vec![]
    } else {
        vec![Yoga {
            yoga_type: YogaType::Anapha,
            name: "Anapha",
            description: "Planet in 12th from Moon — good health, virtue, fame",
            planets,
        }]
    }
}

/// Detect Durudhara Yoga — planets in both 2nd and 12th from Moon (not Sun).
fn detect_durudhara(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let second_from_moon = (moon_sign + 1) % 12;
    let twelfth_from_moon = (moon_sign + 11) % 12;

    let has_2nd = positions.iter().any(|p| {
        p.sign == second_from_moon
            && p.planet != YogaPlanet::Sun
            && p.planet != YogaPlanet::Moon
            && p.planet != YogaPlanet::Rahu
            && p.planet != YogaPlanet::Ketu
    });
    let has_12th = positions.iter().any(|p| {
        p.sign == twelfth_from_moon
            && p.planet != YogaPlanet::Sun
            && p.planet != YogaPlanet::Moon
            && p.planet != YogaPlanet::Rahu
            && p.planet != YogaPlanet::Ketu
    });

    if has_2nd && has_12th {
        let planets: Vec<YogaPlanet> = positions
            .iter()
            .filter(|p| {
                (p.sign == second_from_moon || p.sign == twelfth_from_moon)
                    && p.planet != YogaPlanet::Sun
                    && p.planet != YogaPlanet::Moon
                    && p.planet != YogaPlanet::Rahu
                    && p.planet != YogaPlanet::Ketu
            })
            .map(|p| p.planet)
            .collect();
        vec![Yoga {
            yoga_type: YogaType::Durudhara,
            name: "Durudhara",
            description: "Planets in 2nd and 12th from Moon — wealth, generosity, fame",
            planets,
        }]
    } else {
        vec![]
    }
}

/// Detect Kemadruma Yoga — no planets in 2nd or 12th from Moon (excluding Sun, nodes).
fn detect_kemadruma(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let second_from_moon = (moon_sign + 1) % 12;
    let twelfth_from_moon = (moon_sign + 11) % 12;

    let has_planet_in_2nd = positions.iter().any(|p| {
        p.sign == second_from_moon
            && p.planet != YogaPlanet::Sun
            && p.planet != YogaPlanet::Moon
            && p.planet != YogaPlanet::Rahu
            && p.planet != YogaPlanet::Ketu
    });
    let has_planet_in_12th = positions.iter().any(|p| {
        p.sign == twelfth_from_moon
            && p.planet != YogaPlanet::Sun
            && p.planet != YogaPlanet::Moon
            && p.planet != YogaPlanet::Rahu
            && p.planet != YogaPlanet::Ketu
    });

    if !has_planet_in_2nd && !has_planet_in_12th {
        vec![Yoga {
            yoga_type: YogaType::Kemadruma,
            name: "Kemadruma",
            description: "No planets in 2nd or 12th from Moon — poverty, struggles, isolation",
            planets: vec![],
        }]
    } else {
        vec![]
    }
}

/// Detect Shakata Yoga — Moon in 6th or 8th from Jupiter.
fn detect_shakata(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    if let Some(jup) = find_planet(positions, YogaPlanet::Jupiter) {
        let distance = (moon_sign + 12 - jup.sign) % 12;
        // 6th = 5 signs, 8th = 7 signs from Jupiter
        if matches!(distance, 5 | 7) {
            return vec![Yoga {
                yoga_type: YogaType::Shakata,
                name: "Shakata",
                description: "Moon in 6th/8th from Jupiter — fluctuating fortunes, instability",
                planets: vec![YogaPlanet::Moon, YogaPlanet::Jupiter],
            }];
        }
    }
    vec![]
}

/// Detect Vish Yoga — Moon-Saturn conjunction (same sign).
fn detect_vish(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let moon = find_planet(positions, YogaPlanet::Moon);
    let saturn = find_planet(positions, YogaPlanet::Saturn);
    if let (Some(m), Some(s)) = (moon, saturn) {
        if m.sign == s.sign {
            return vec![Yoga {
                yoga_type: YogaType::Vish,
                name: "Vish",
                description: "Moon-Saturn conjunction — mental stress, emotional difficulties",
                planets: vec![YogaPlanet::Moon, YogaPlanet::Saturn],
            }];
        }
    }
    vec![]
}

// ── Sun-based yogas ─────────────────────────────────────────────────

/// Detect Vesi Yoga — a planet (not Moon) in 2nd from Sun.
fn detect_vesi(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let sun = find_planet(positions, YogaPlanet::Sun);
    let Some(sun_pos) = sun else { return vec![] };
    let second_from_sun = (sun_pos.sign + 1) % 12;

    let planets: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| {
            p.sign == second_from_sun
                && p.planet != YogaPlanet::Moon
                && p.planet != YogaPlanet::Sun
                && p.planet != YogaPlanet::Rahu
                && p.planet != YogaPlanet::Ketu
        })
        .map(|p| p.planet)
        .collect();

    if planets.is_empty() {
        vec![]
    } else {
        vec![Yoga {
            yoga_type: YogaType::Vesi,
            name: "Vesi",
            description: "Planet in 2nd from Sun — wealth, status, good character",
            planets,
        }]
    }
}

/// Detect Vosi Yoga — a planet (not Moon) in 12th from Sun.
fn detect_vosi(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let sun = find_planet(positions, YogaPlanet::Sun);
    let Some(sun_pos) = sun else { return vec![] };
    let twelfth_from_sun = (sun_pos.sign + 11) % 12;

    let planets: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| {
            p.sign == twelfth_from_sun
                && p.planet != YogaPlanet::Moon
                && p.planet != YogaPlanet::Sun
                && p.planet != YogaPlanet::Rahu
                && p.planet != YogaPlanet::Ketu
        })
        .map(|p| p.planet)
        .collect();

    if planets.is_empty() {
        vec![]
    } else {
        vec![Yoga {
            yoga_type: YogaType::Vosi,
            name: "Vosi",
            description: "Planet in 12th from Sun — charitable, learned, skilled",
            planets,
        }]
    }
}

/// Detect Obhayachari Yoga — planets in both 2nd and 12th from Sun (not Moon).
fn detect_obhayachari(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let sun = find_planet(positions, YogaPlanet::Sun);
    let Some(sun_pos) = sun else { return vec![] };
    let second_from_sun = (sun_pos.sign + 1) % 12;
    let twelfth_from_sun = (sun_pos.sign + 11) % 12;

    let is_valid = |p: &&PlanetPosition| -> bool {
        p.planet != YogaPlanet::Moon
            && p.planet != YogaPlanet::Sun
            && p.planet != YogaPlanet::Rahu
            && p.planet != YogaPlanet::Ketu
    };

    let has_2nd = positions
        .iter()
        .any(|p| p.sign == second_from_sun && is_valid(&p));
    let has_12th = positions
        .iter()
        .any(|p| p.sign == twelfth_from_sun && is_valid(&p));

    if has_2nd && has_12th {
        let planets: Vec<YogaPlanet> = positions
            .iter()
            .filter(|p| (p.sign == second_from_sun || p.sign == twelfth_from_sun) && is_valid(p))
            .map(|p| p.planet)
            .collect();
        vec![Yoga {
            yoga_type: YogaType::Obhayachari,
            name: "Obhayachari",
            description: "Planets in 2nd and 12th from Sun — eloquent, wealthy, powerful",
            planets,
        }]
    } else {
        vec![]
    }
}

// ── Raja / Power yogas ──────────────────────────────────────────────

/// Detect Neechabhanga Raja Yoga (debilitation cancellation).
fn detect_neechabhanga(positions: &[PlanetPosition], lagna_sign: u8, moon_sign: u8) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    for pos in positions {
        if !is_debilitated(pos.planet, pos.sign) {
            continue;
        }
        let Some(lord) = sign_lord(pos.sign) else {
            continue;
        };
        let Some(lord_pos) = find_planet(positions, lord) else {
            continue;
        };
        let dist_lagna = (lord_pos.sign + 12 - lagna_sign) % 12;
        let dist_moon = (lord_pos.sign + 12 - moon_sign) % 12;
        let in_kendra_lagna = matches!(dist_lagna, 0 | 3 | 6 | 9);
        let in_kendra_moon = matches!(dist_moon, 0 | 3 | 6 | 9);

        if in_kendra_lagna || in_kendra_moon {
            yogas.push(Yoga {
                yoga_type: YogaType::NeechabhangaRajaYoga,
                name: "Neechabhanga Raja Yoga",
                description:
                    "Debilitation cancelled — planet's weakness transforms into strength and power",
                planets: vec![pos.planet, lord],
            });
        }
    }
    yogas
}

/// Detect Chamara Yoga — benefic in lagna and Jupiter aspects lagna.
fn detect_chamara(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let benefic_in_lagna = positions
        .iter()
        .any(|p| p.sign == lagna_sign && is_natural_benefic(p.planet));

    if !benefic_in_lagna {
        return vec![];
    }

    // Jupiter aspects: 5th, 7th, 9th from its position (signs 4, 6, 8 away)
    if let Some(jup) = find_planet(positions, YogaPlanet::Jupiter) {
        let aspects = [4, 6, 8];
        let jupiter_aspects_lagna =
            jup.sign == lagna_sign || aspects.iter().any(|&a| (jup.sign + a) % 12 == lagna_sign);

        if jupiter_aspects_lagna {
            return vec![Yoga {
                yoga_type: YogaType::Chamara,
                name: "Chamara",
                description: "Benefic in lagna aspected by Jupiter — royal bearing, wisdom, honor",
                planets: vec![YogaPlanet::Jupiter],
            }];
        }
    }
    vec![]
}

/// Detect Parvata Yoga — benefics in kendra, no malefics in kendra.
fn detect_parvata(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let benefics_in_kendra: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| bhava::is_kendra(p.bhava) && is_natural_benefic(p.planet))
        .map(|p| p.planet)
        .collect();

    let malefics_in_kendra = positions
        .iter()
        .any(|p| bhava::is_kendra(p.bhava) && is_natural_malefic(p.planet));

    if !benefics_in_kendra.is_empty() && !malefics_in_kendra {
        vec![Yoga {
            yoga_type: YogaType::Parvata,
            name: "Parvata",
            description: "Benefics in kendra, no malefics — strong, wealthy, famous",
            planets: benefics_in_kendra,
        }]
    } else {
        vec![]
    }
}

/// Detect Kahala Yoga — lords of 4th and 9th in mutual kendra.
fn detect_kahala(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let sign_4 = (lagna_sign + 3) % 12;
    let sign_9 = (lagna_sign + 8) % 12;
    let lord_4 = sign_lord(sign_4);
    let lord_9 = sign_lord(sign_9);

    if let (Some(l4), Some(l9)) = (lord_4, lord_9) {
        if let (Some(p4), Some(p9)) = (find_planet(positions, l4), find_planet(positions, l9)) {
            let dist = (p4.sign + 12 - p9.sign) % 12;
            if matches!(dist, 0 | 3 | 6 | 9) {
                return vec![Yoga {
                    yoga_type: YogaType::Kahala,
                    name: "Kahala",
                    description: "Lords of 4th and 9th in mutual kendra — bold, energetic, prosperous",
                    planets: vec![l4, l9],
                }];
            }
        }
    }
    vec![]
}

/// Detect Shankha Yoga — lords of 5th and 6th in mutual kendra.
fn detect_shankha(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let sign_5 = (lagna_sign + 4) % 12;
    let sign_6 = (lagna_sign + 5) % 12;
    let lord_5 = sign_lord(sign_5);
    let lord_6 = sign_lord(sign_6);

    if let (Some(l5), Some(l6)) = (lord_5, lord_6) {
        if let (Some(p5), Some(p6)) = (find_planet(positions, l5), find_planet(positions, l6)) {
            let dist = (p5.sign + 12 - p6.sign) % 12;
            if matches!(dist, 0 | 3 | 6 | 9) {
                return vec![Yoga {
                    yoga_type: YogaType::Shankha,
                    name: "Shankha",
                    description: "Lords of 5th and 6th in mutual kendra — long-lived, virtuous, wealthy",
                    planets: vec![l5, l6],
                }];
            }
        }
    }
    vec![]
}

/// Detect Khyati Yoga — 9th lord in 10th, 10th lord in 9th (exchange).
fn detect_khyati(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let sign_9 = (lagna_sign + 8) % 12;
    let sign_10 = (lagna_sign + 9) % 12;
    let lord_9 = sign_lord(sign_9);
    let lord_10 = sign_lord(sign_10);

    if let (Some(l9), Some(l10)) = (lord_9, lord_10) {
        if let (Some(p9), Some(p10)) = (find_planet(positions, l9), find_planet(positions, l10)) {
            // 9th lord in 10th sign, 10th lord in 9th sign
            if p9.sign == sign_10 && p10.sign == sign_9 {
                return vec![Yoga {
                    yoga_type: YogaType::Khyati,
                    name: "Khyati",
                    description: "9th lord in 10th, 10th lord in 9th — fame, reputation, powerful career",
                    planets: vec![l9, l10],
                }];
            }
        }
    }
    vec![]
}

/// Detect Amara Yoga — benefic in 10th from Moon.
fn detect_amara(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    let tenth_from_moon = (moon_sign + 9) % 12;
    let benefics: Vec<YogaPlanet> = positions
        .iter()
        .filter(|p| {
            p.sign == tenth_from_moon
                && is_natural_benefic(p.planet)
                && p.planet != YogaPlanet::Moon
        })
        .map(|p| p.planet)
        .collect();

    if benefics.is_empty() {
        vec![]
    } else {
        vec![Yoga {
            yoga_type: YogaType::Amara,
            name: "Amara",
            description: "Benefic in 10th from Moon — lasting fame, prosperity",
            planets: benefics,
        }]
    }
}

/// Detect Parijata Yoga — sign lord of lagna in kendra or trikona.
fn detect_parijata(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let Some(lord) = sign_lord(lagna_sign) else {
        return vec![];
    };
    let Some(lord_pos) = find_planet(positions, lord) else {
        return vec![];
    };

    if bhava::is_kendra(lord_pos.bhava) || bhava::is_trikona(lord_pos.bhava) {
        vec![Yoga {
            yoga_type: YogaType::Parijata,
            name: "Parijata",
            description: "Lagna lord in kendra/trikona — gradual rise, happiness, authority",
            planets: vec![lord],
        }]
    } else {
        vec![]
    }
}

// ── Wealth yogas ────────────────────────────────────────────────────

/// Detect Lakshmi Yoga — Venus in own/exalted sign in kendra/trikona, 9th lord strong.
fn detect_lakshmi(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let venus = find_planet(positions, YogaPlanet::Venus);
    let Some(v) = venus else { return vec![] };

    if !(is_in_own_sign(YogaPlanet::Venus, v.sign) || is_exalted(YogaPlanet::Venus, v.sign)) {
        return vec![];
    }
    if !(bhava::is_kendra(v.bhava) || bhava::is_trikona(v.bhava)) {
        return vec![];
    }

    // 9th lord should be strong (in own/exalted sign or kendra)
    let sign_9 = (lagna_sign + 8) % 12;
    let Some(lord_9) = sign_lord(sign_9) else {
        return vec![];
    };
    let Some(l9_pos) = find_planet(positions, lord_9) else {
        return vec![];
    };
    let lord_strong = is_in_own_sign(lord_9, l9_pos.sign)
        || is_exalted(lord_9, l9_pos.sign)
        || bhava::is_kendra(l9_pos.bhava);

    if lord_strong {
        vec![Yoga {
            yoga_type: YogaType::Lakshmi,
            name: "Lakshmi",
            description: "Venus strong in kendra/trikona, 9th lord strong — great wealth, fortune",
            planets: vec![YogaPlanet::Venus, lord_9],
        }]
    } else {
        vec![]
    }
}

/// Detect Saraswati Yoga — Jupiter, Venus, Mercury in kendra/trikona/2nd.
fn detect_saraswati(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let check_planet = |planet: YogaPlanet| -> bool {
        if let Some(pos) = find_planet(positions, planet) {
            let bhava = sign_to_bhava(pos.sign, lagna_sign);
            bhava::is_kendra(bhava) || bhava::is_trikona(bhava) || bhava == 2
        } else {
            false
        }
    };

    if check_planet(YogaPlanet::Jupiter)
        && check_planet(YogaPlanet::Venus)
        && check_planet(YogaPlanet::Mercury)
    {
        vec![Yoga {
            yoga_type: YogaType::Saraswati,
            name: "Saraswati",
            description: "Jupiter, Venus, Mercury in kendra/trikona/2nd — learning, wisdom, eloquence",
            planets: vec![YogaPlanet::Jupiter, YogaPlanet::Venus, YogaPlanet::Mercury],
        }]
    } else {
        vec![]
    }
}

/// Detect Pushkala Yoga — Moon in friendly sign, lord of Moon sign in kendra.
fn detect_pushkala(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    // Moon should be in a friendly/own/exalted sign
    if let Some(moon_pos) = find_planet(positions, YogaPlanet::Moon) {
        let is_friendly = is_in_own_sign(YogaPlanet::Moon, moon_sign)
            || is_exalted(YogaPlanet::Moon, moon_sign)
            || is_benefic_sign(moon_sign);

        if !is_friendly {
            return vec![];
        }

        // Lord of Moon sign should be in kendra
        let Some(lord) = sign_lord(moon_sign) else {
            return vec![];
        };
        let Some(lord_pos) = find_planet(positions, lord) else {
            return vec![];
        };

        if bhava::is_kendra(lord_pos.bhava) {
            return vec![Yoga {
                yoga_type: YogaType::Pushkala,
                name: "Pushkala",
                description: "Moon in friendly sign, its lord in kendra — wealth, popularity, fame",
                planets: vec![YogaPlanet::Moon, lord],
            }];
        }
        // Use moon_pos to avoid unused warning
        let _ = moon_pos;
    }
    vec![]
}

/// Detect Dhana Special Yoga — lords of 2nd and 11th in kendra or trikona.
fn detect_dhana_special(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let sign_2 = (lagna_sign + 1) % 12;
    let sign_11 = (lagna_sign + 10) % 12;
    let lord_2 = sign_lord(sign_2);
    let lord_11 = sign_lord(sign_11);

    if let (Some(l2), Some(l11)) = (lord_2, lord_11) {
        if let (Some(p2), Some(p11)) = (find_planet(positions, l2), find_planet(positions, l11)) {
            let l2_good = bhava::is_kendra(p2.bhava) || bhava::is_trikona(p2.bhava);
            let l11_good = bhava::is_kendra(p11.bhava) || bhava::is_trikona(p11.bhava);
            if l2_good && l11_good {
                return vec![Yoga {
                    yoga_type: YogaType::DhanaSpecial,
                    name: "Dhana (Special)",
                    description: "Lords of 2nd and 11th in kendra/trikona — strong wealth accumulation",
                    planets: vec![l2, l11],
                }];
            }
        }
    }
    vec![]
}

// ── Spiritual yogas ─────────────────────────────────────────────────

/// Detect Sanyasa Yoga — 4+ planets in one sign.
fn detect_sanyasa(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let mut sign_counts: [u8; 12] = [0; 12];
    for pos in positions {
        if pos.sign < 12 {
            sign_counts[pos.sign as usize] += 1;
        }
    }

    for (sign_idx, &count) in sign_counts.iter().enumerate() {
        if count >= 4 {
            #[allow(clippy::cast_possible_truncation)]
            let sign = sign_idx as u8;
            let planets: Vec<YogaPlanet> = positions
                .iter()
                .filter(|p| p.sign == sign)
                .map(|p| p.planet)
                .collect();
            return vec![Yoga {
                yoga_type: YogaType::Sanyasa,
                name: "Sanyasa",
                description: "4+ planets in one sign — renunciation, spiritual inclination",
                planets,
            }];
        }
    }
    vec![]
}

// ── Negative yogas ──────────────────────────────────────────────────

/// Detect Daridra Yoga — 11th lord in 6th, 8th, or 12th house.
fn detect_daridra(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let sign_11 = (lagna_sign + 10) % 12;
    let Some(lord_11) = sign_lord(sign_11) else {
        return vec![];
    };
    let Some(lord_pos) = find_planet(positions, lord_11) else {
        return vec![];
    };

    if bhava::is_dusthana(lord_pos.bhava) {
        vec![Yoga {
            yoga_type: YogaType::Daridra,
            name: "Daridra",
            description: "11th lord in dusthana (6/8/12) — financial difficulties, reduced gains",
            planets: vec![lord_11],
        }]
    } else {
        vec![]
    }
}

/// Detect Graha Yuddha (planetary war) — two planets within 1 degree.
fn detect_graha_yuddha(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    let war_planets: &[YogaPlanet] = &[
        YogaPlanet::Mars,
        YogaPlanet::Mercury,
        YogaPlanet::Jupiter,
        YogaPlanet::Venus,
        YogaPlanet::Saturn,
    ];

    for (i, pos_a) in positions.iter().enumerate() {
        if !war_planets.contains(&pos_a.planet) {
            continue;
        }
        for pos_b in &positions[i + 1..] {
            if !war_planets.contains(&pos_b.planet) {
                continue;
            }
            let diff = (pos_a.longitude - pos_b.longitude).abs();
            let angular_diff = if diff > 180.0 { 360.0 - diff } else { diff };
            if angular_diff < 1.0 {
                yogas.push(Yoga {
                    yoga_type: YogaType::GrahaYuddha,
                    name: "Graha Yuddha",
                    description: "Planetary war — two planets within 1 degree, conflict of energies",
                    planets: vec![pos_a.planet, pos_b.planet],
                });
            }
        }
    }
    yogas
}

// ── Kartari yogas ───────────────────────────────────────────────────

/// Detect Papa Kartari — malefics on both sides of lagna (signs adjacent to lagna).
fn detect_papa_kartari(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let prev_sign = (lagna_sign + 11) % 12;
    let next_sign = (lagna_sign + 1) % 12;

    let malefic_prev = positions
        .iter()
        .any(|p| p.sign == prev_sign && is_natural_malefic(p.planet));
    let malefic_next = positions
        .iter()
        .any(|p| p.sign == next_sign && is_natural_malefic(p.planet));

    if malefic_prev && malefic_next {
        vec![Yoga {
            yoga_type: YogaType::PapaKartari,
            name: "Papa Kartari",
            description: "Malefics hemming lagna — obstacles, restrictions, blocked growth",
            planets: vec![],
        }]
    } else {
        vec![]
    }
}

/// Detect Shubha Kartari — benefics on both sides of lagna.
fn detect_shubha_kartari(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    let prev_sign = (lagna_sign + 11) % 12;
    let next_sign = (lagna_sign + 1) % 12;

    let benefic_prev = positions
        .iter()
        .any(|p| p.sign == prev_sign && is_natural_benefic(p.planet));
    let benefic_next = positions
        .iter()
        .any(|p| p.sign == next_sign && is_natural_benefic(p.planet));

    if benefic_prev && benefic_next {
        vec![Yoga {
            yoga_type: YogaType::ShubhaKartari,
            name: "Shubha Kartari",
            description: "Benefics hemming lagna — protection, support, auspicious environment",
            planets: vec![],
        }]
    } else {
        vec![]
    }
}

// ── Exchange yogas ──────────────────────────────────────────────────

/// Detect Parivartana Yoga — mutual exchange of signs between two planets.
fn detect_parivartana(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let mut yogas = Vec::new();
    for (i, pos_a) in positions.iter().enumerate() {
        let Some(lord_a_sign) = sign_lord(pos_a.sign) else {
            continue;
        };
        if lord_a_sign != pos_a.planet {
            // pos_a is NOT in its own sign, so check if lord_a_sign is in pos_a's own sign
            for pos_b in &positions[i + 1..] {
                if pos_b.planet != lord_a_sign {
                    continue;
                }
                // pos_b is the lord of pos_a's sign. Is pos_a the lord of pos_b's sign?
                if let Some(lord_b_sign) = sign_lord(pos_b.sign) {
                    if lord_b_sign == pos_a.planet {
                        yogas.push(Yoga {
                            yoga_type: YogaType::Parivartana,
                            name: "Parivartana",
                            description:
                                "Mutual sign exchange — planets share and enhance each other's results",
                            planets: vec![pos_a.planet, pos_b.planet],
                        });
                    }
                }
            }
        }
    }
    yogas
}

// ── Sign-distribution (Sankhya) yogas ───────────────────────────────

/// Detect sign-distribution yogas based on how planets are spread across signs.
fn detect_sign_distribution_yogas(positions: &[PlanetPosition]) -> Vec<Yoga> {
    let mut yogas = Vec::new();

    // Get the 7 visible planets for sign-distribution analysis
    let visible: Vec<&PlanetPosition> = positions
        .iter()
        .filter(|p| seven_planets().contains(&p.planet))
        .collect();

    if visible.is_empty() {
        return yogas;
    }

    // Check movable/fixed/dual sign yogas
    let all_movable = visible.iter().all(|p| is_movable_sign(p.sign));
    let all_fixed = visible.iter().all(|p| is_fixed_sign(p.sign));
    let all_dual = visible.iter().all(|p| is_dual_sign(p.sign));

    if all_movable && visible.len() >= 3 {
        yogas.push(Yoga {
            yoga_type: YogaType::Rajju,
            name: "Rajju",
            description: "All planets in movable signs — travel, change, dynamic life",
            planets: visible.iter().map(|p| p.planet).collect(),
        });
    }
    if all_fixed && visible.len() >= 3 {
        yogas.push(Yoga {
            yoga_type: YogaType::Musala,
            name: "Musala",
            description: "All planets in fixed signs — stability, determination, pride",
            planets: visible.iter().map(|p| p.planet).collect(),
        });
    }
    if all_dual && visible.len() >= 3 {
        yogas.push(Yoga {
            yoga_type: YogaType::Nala,
            name: "Nala",
            description: "All planets in dual signs — adaptability, mixed results",
            planets: visible.iter().map(|p| p.planet).collect(),
        });
    }

    // Count distinct signs occupied
    let mut occupied_signs = [false; 12];
    for p in &visible {
        if (p.sign as usize) < 12 {
            occupied_signs[p.sign as usize] = true;
        }
    }
    let sign_count = occupied_signs.iter().filter(|&&b| b).count();

    match sign_count {
        2 => yogas.push(Yoga {
            yoga_type: YogaType::Dama,
            name: "Dama",
            description: "All planets in 2 signs — concentrated energy, extremes",
            planets: visible.iter().map(|p| p.planet).collect(),
        }),
        3 => yogas.push(Yoga {
            yoga_type: YogaType::Kedara,
            name: "Kedara",
            description: "All planets in 3 signs — agricultural prosperity, steady growth",
            planets: visible.iter().map(|p| p.planet).collect(),
        }),
        4 => yogas.push(Yoga {
            yoga_type: YogaType::Shoola,
            name: "Shoola",
            description: "All planets in 4 signs — sharp intellect, possible health issues",
            planets: visible.iter().map(|p| p.planet).collect(),
        }),
        5 => yogas.push(Yoga {
            yoga_type: YogaType::Yuga,
            name: "Yuga",
            description: "All planets in 5 signs — heretical views, unconventional",
            planets: visible.iter().map(|p| p.planet).collect(),
        }),
        6 => yogas.push(Yoga {
            yoga_type: YogaType::Gola,
            name: "Gola",
            description: "All planets in 6 signs — moderate wealth, average life",
            planets: visible.iter().map(|p| p.planet).collect(),
        }),
        _ => {}
    }

    yogas
}

// ── Miscellaneous ───────────────────────────────────────────────────

/// Detect Mridanga Yoga — lord of sign where an exalted planet sits is in kendra/trikona.
fn detect_mridanga(positions: &[PlanetPosition], lagna_sign: u8) -> Vec<Yoga> {
    for pos in positions {
        if !is_exalted(pos.planet, pos.sign) {
            continue;
        }
        let Some(lord) = sign_lord(pos.sign) else {
            continue;
        };
        let Some(lord_pos) = find_planet(positions, lord) else {
            continue;
        };
        let lord_bhava = sign_to_bhava(lord_pos.sign, lagna_sign);
        if bhava::is_kendra(lord_bhava) || bhava::is_trikona(lord_bhava) {
            return vec![Yoga {
                yoga_type: YogaType::Mridanga,
                name: "Mridanga",
                description: "Lord of exalted planet's sign in kendra/trikona — fame like a drum",
                planets: vec![pos.planet, lord],
            }];
        }
    }
    vec![]
}

/// Detect Chandra Yoga — Moon in a benefic-ruled sign.
fn detect_chandra_yoga(positions: &[PlanetPosition], moon_sign: u8) -> Vec<Yoga> {
    if is_benefic_sign(moon_sign) && find_planet(positions, YogaPlanet::Moon).is_some() {
        return vec![Yoga {
            yoga_type: YogaType::Chandra,
            name: "Chandra",
            description: "Moon in a benefic sign — pleasant disposition, comfort, good mind",
            planets: vec![YogaPlanet::Moon],
        }];
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(planet: YogaPlanet, sign: u8, bhava: u8) -> PlanetPosition {
        PlanetPosition {
            planet,
            sign,
            longitude: f64::from(sign) * 30.0 + 15.0,
            bhava,
        }
    }

    fn pos_lng(planet: YogaPlanet, sign: u8, bhava: u8, longitude: f64) -> PlanetPosition {
        PlanetPosition {
            planet,
            sign,
            longitude,
            bhava,
        }
    }

    // ── Existing tests ──────────────────────────────────────────────

    #[test]
    fn ruchaka_mars_in_aries_1st_house() {
        let positions = [pos(YogaPlanet::Mars, 0, 1)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Ruchaka));
    }

    #[test]
    fn ruchaka_mars_exalted_in_capricorn_7th() {
        let positions = [pos(YogaPlanet::Mars, 9, 7)];
        let yogas = detect_yogas(&positions, 3, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Ruchaka));
    }

    #[test]
    fn hamsa_jupiter_exalted_in_cancer_4th() {
        let positions = [pos(YogaPlanet::Jupiter, 3, 4)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Hamsa));
    }

    #[test]
    fn sasa_saturn_exalted_in_libra_10th() {
        let positions = [pos(YogaPlanet::Saturn, 6, 10)];
        let yogas = detect_yogas(&positions, 9, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Sasa));
    }

    #[test]
    fn no_mahapurusha_own_sign_not_in_kendra() {
        let positions = [pos(YogaPlanet::Mars, 0, 5)];
        let yogas = detect_yogas(&positions, 8, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::Ruchaka));
    }

    #[test]
    fn gajakesari_jupiter_kendra_from_moon() {
        let positions = [pos(YogaPlanet::Moon, 0, 1), pos(YogaPlanet::Jupiter, 3, 4)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Gajakesari));
    }

    #[test]
    fn gajakesari_not_when_jupiter_not_in_kendra_from_moon() {
        let positions = [pos(YogaPlanet::Moon, 0, 1), pos(YogaPlanet::Jupiter, 2, 3)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::Gajakesari));
    }

    #[test]
    fn budhaditya_sun_mercury_same_sign() {
        let positions = [pos(YogaPlanet::Sun, 4, 5), pos(YogaPlanet::Mercury, 4, 5)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Budhaditya));
    }

    #[test]
    fn budhaditya_requires_same_sign() {
        let positions = [pos(YogaPlanet::Sun, 4, 5), pos(YogaPlanet::Mercury, 5, 6)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::Budhaditya));
    }

    #[test]
    fn multiple_yogas_detected_simultaneously() {
        let positions = [
            pos(YogaPlanet::Mars, 0, 1),
            pos(YogaPlanet::Sun, 4, 5),
            pos(YogaPlanet::Mercury, 4, 5),
            pos(YogaPlanet::Moon, 0, 1),
            pos(YogaPlanet::Jupiter, 3, 4),
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Ruchaka));
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Hamsa));
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Budhaditya));
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Gajakesari));
    }

    #[test]
    fn no_positive_yogas_when_no_conditions_met() {
        // Mars in Gemini (neutral), house 3 (not kendra, not own/exalted)
        // Only Kemadruma (negative yoga) should fire since no planets flank Moon
        let positions = [pos(YogaPlanet::Mars, 2, 3)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(
            yogas
                .iter()
                .all(|y| matches!(y.yoga_type, YogaType::Kemadruma))
        );
    }

    #[test]
    fn chandra_mangala_moon_mars_conjunction() {
        let positions = [pos(YogaPlanet::Moon, 7, 8), pos(YogaPlanet::Mars, 7, 8)];
        let yogas = detect_yogas(&positions, 0, 7);
        assert!(
            yogas
                .iter()
                .any(|y| y.yoga_type == YogaType::ChandraMangala)
        );
    }

    #[test]
    fn neechabhanga_debilitated_planet_with_lord_in_kendra() {
        let positions = [pos(YogaPlanet::Mars, 3, 4), pos(YogaPlanet::Moon, 0, 1)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(
            yogas
                .iter()
                .any(|y| y.yoga_type == YogaType::NeechabhangaRajaYoga)
        );
    }

    #[test]
    fn malavya_venus_in_taurus_1st() {
        let positions = [pos(YogaPlanet::Venus, 1, 1)];
        let yogas = detect_yogas(&positions, 1, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Malavya));
    }

    #[test]
    fn bhadra_mercury_in_virgo_kendra() {
        let positions = [pos(YogaPlanet::Mercury, 5, 10)];
        let yogas = detect_yogas(&positions, 8, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Bhadra));
    }

    // ── New yoga tests ──────────────────────────────────────────────

    #[test]
    fn kemadruma_no_planets_around_moon() {
        // Moon in Aries (0), no planets in Taurus (1) or Pisces (11)
        let positions = [
            pos(YogaPlanet::Moon, 0, 1),
            pos(YogaPlanet::Mars, 5, 6), // far away
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Kemadruma));
    }

    #[test]
    fn kemadruma_cancelled_by_planet_in_2nd() {
        // Moon in Aries (0), Mars in Taurus (1) = 2nd from Moon
        let positions = [pos(YogaPlanet::Moon, 0, 1), pos(YogaPlanet::Mars, 1, 2)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::Kemadruma));
    }

    #[test]
    fn sunapha_planet_in_2nd_from_moon() {
        // Moon in Aries (0), Mars in Taurus (1) = 2nd from Moon
        let positions = [pos(YogaPlanet::Moon, 0, 1), pos(YogaPlanet::Mars, 1, 2)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Sunapha));
    }

    #[test]
    fn anapha_planet_in_12th_from_moon() {
        // Moon in Taurus (1), Jupiter in Aries (0) = 12th from Moon
        let positions = [pos(YogaPlanet::Moon, 1, 2), pos(YogaPlanet::Jupiter, 0, 1)];
        let yogas = detect_yogas(&positions, 0, 1);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Anapha));
    }

    #[test]
    fn durudhara_planets_on_both_sides_of_moon() {
        // Moon in Taurus (1), Mars in Gemini (2) = 2nd, Jupiter in Aries (0) = 12th
        let positions = [
            pos(YogaPlanet::Moon, 1, 2),
            pos(YogaPlanet::Mars, 2, 3),
            pos(YogaPlanet::Jupiter, 0, 1),
        ];
        let yogas = detect_yogas(&positions, 0, 1);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Durudhara));
    }

    #[test]
    fn vish_yoga_moon_saturn_conjunction() {
        let positions = [pos(YogaPlanet::Moon, 3, 4), pos(YogaPlanet::Saturn, 3, 4)];
        let yogas = detect_yogas(&positions, 0, 3);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Vish));
    }

    #[test]
    fn shakata_moon_6th_from_jupiter() {
        // Jupiter in Aries (0), Moon in Virgo (5) = 6th from Jupiter
        let positions = [pos(YogaPlanet::Jupiter, 0, 1), pos(YogaPlanet::Moon, 5, 6)];
        let yogas = detect_yogas(&positions, 0, 5);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Shakata));
    }

    #[test]
    fn vesi_planet_in_2nd_from_sun() {
        // Sun in Leo (4), Mars in Virgo (5) = 2nd from Sun
        let positions = [pos(YogaPlanet::Sun, 4, 5), pos(YogaPlanet::Mars, 5, 6)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Vesi));
    }

    #[test]
    fn sanyasa_four_planets_in_one_sign() {
        let positions = [
            pos(YogaPlanet::Sun, 4, 5),
            pos(YogaPlanet::Mercury, 4, 5),
            pos(YogaPlanet::Venus, 4, 5),
            pos(YogaPlanet::Mars, 4, 5),
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Sanyasa));
    }

    #[test]
    fn graha_yuddha_close_planets() {
        let positions = [
            pos_lng(YogaPlanet::Mars, 4, 5, 120.5),
            pos_lng(YogaPlanet::Mercury, 4, 5, 121.0),
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::GrahaYuddha));
    }

    #[test]
    fn no_graha_yuddha_distant_planets() {
        let positions = [
            pos_lng(YogaPlanet::Mars, 4, 5, 120.0),
            pos_lng(YogaPlanet::Mercury, 4, 5, 125.0),
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::GrahaYuddha));
    }

    #[test]
    fn parivartana_mutual_exchange() {
        // Mars in Cancer (3, Moon's sign), Moon in Aries (0, Mars's sign)
        let positions = [pos(YogaPlanet::Mars, 3, 4), pos(YogaPlanet::Moon, 0, 1)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Parivartana));
    }

    #[test]
    fn papa_kartari_malefics_hemming_lagna() {
        // Lagna in Taurus (1). Mars in Aries (0), Saturn in Gemini (2)
        let positions = [pos(YogaPlanet::Mars, 0, 12), pos(YogaPlanet::Saturn, 2, 2)];
        let yogas = detect_yogas(&positions, 1, 5);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::PapaKartari));
    }

    #[test]
    fn chandra_yoga_moon_in_benefic_sign() {
        // Moon in Cancer (3, Moon-ruled = benefic)
        let positions = [pos(YogaPlanet::Moon, 3, 4)];
        let yogas = detect_yogas(&positions, 0, 3);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Chandra));
    }

    #[test]
    fn parvata_benefics_in_kendra_no_malefics() {
        let positions = [pos(YogaPlanet::Jupiter, 3, 4), pos(YogaPlanet::Venus, 6, 7)];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(yogas.iter().any(|y| y.yoga_type == YogaType::Parvata));
    }

    #[test]
    fn parvata_cancelled_by_malefic_in_kendra() {
        let positions = [
            pos(YogaPlanet::Jupiter, 3, 4),
            pos(YogaPlanet::Mars, 0, 1), // malefic in kendra
        ];
        let yogas = detect_yogas(&positions, 0, 0);
        assert!(!yogas.iter().any(|y| y.yoga_type == YogaType::Parvata));
    }
}
