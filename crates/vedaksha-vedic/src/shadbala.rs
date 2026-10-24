// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Shadbala (six-fold planetary strength).
//!
//! Computes the six components of planetary strength used in Vedic astrology
//! to assess how effectively a planet can deliver its results: Sthana Bala
//! (positional), Dig Bala (directional), Kala Bala (temporal), Cheshta Bala
//! (motional), Naisargika Bala (natural) and Drik Bala (aspectual).
//!
//! # Source
//!
//! BPHS chapter 27 (the Spashtabala adhyaya), read in the Sanskrit, with the
//! moolatrikona arcs and the five-fold friendship from chapter 3 (slokas 49 to
//! 58). Chapter numbers follow the recension in which shadbala is chapter 27; a
//! longer recension that counts two extra earlier chapters numbers it 29, and
//! the sloka numbers agree. The year and month lords, which the strength verse
//! names but does not reckon, are reckoned by the Surya Siddhanta, chapter I,
//! slokas 51 and 52.
//!
//! Each rule below is graded the way [`crate::dasha::chara`] grades its sutras:
//! **(a)** stated at the verse and read there; **(b)** stated at the verse with
//! one parameter the verse leaves open, which is exposed; **(d)** this crate's
//! convention where the verse is silent.
//!
//! # Two forms
//!
//! * **The complete form**, [`compute_shadbala_complete`], takes the birth
//!   moment, the ayanamsha and the seven grahas and returns a
//!   [`CompleteShadbala`] per graha with every part below. It is the form that
//!   follows the verse.
//! * **The coarse form**, [`compute_shadbala_full`] with [`ShadbalaPlanetData`]
//!   and [`Shadbala`], takes only the two booleans `is_daytime` and
//!   `moon_phase_waxing` and a speed. It cannot carry the inputs the verse
//!   needs (a time of day in ghatis, the Sun and Moon longitudes, the
//!   other six grahas for the friendships, a mean longitude), so it keeps the
//!   0-or-30 steps it always had, and it is what the MCP and WASM surfaces
//!   serve. Its coverage is listed at the end of this section.
//!
//! # What the complete form implements
//!
//! | Part | Rule | Grade |
//! |---|---|---|
//! | Uchcha | 27.1: arc from the debilitation point, folded, over 3 | (a) |
//! | Saptavargaja | 27.2 to 27.4: 45, 30, 20, 15, 10, 4, 2 for moolatrikona, own sign, adhimitra, mitra, sama, shatru, adhishatru, in each of D1, D2, D3, D7, D9, D12, D30 | (a); the ladder is a parameter ([`SaptavargajaLadder`]) |
//! | Moolatrikona arcs | 3.51 to 3.54 | (a) |
//! | Panchadha maitri | 3.55 to 3.58: natural friendship derived from the moolatrikona sign and exaltation sign, temporary friendship from the D1 positions, compounded | (a) |
//! | Moolatrikona outside D1 | 27.4 says the values recur in the other six vargas; a varga position has no degree arc, so the varga *sign* is tested against the moolatrikona sign | (b), [`MoolatrikonaScope`], default every varga |
//! | Ojhayugma, Kendradi | 27.4 (second half), 27.5 | (a) |
//! | Drekkana | 27.6: male, neuter, female in the first, middle, last drekkana | (a), [`DrekkanaOrder`] |
//! | Nathonnata | 27.8 and 27.9: continuous in the time from local midnight | (a) |
//! | Paksha | 27.10 and 27.11: continuous in the Moon-Sun elongation; benefics Moon, Mercury, Venus, Jupiter | (a); the benefic set is an argument |
//! | Tribhaga | 27.12 | (a) |
//! | Abda, masa, vara, hora | 27.13: 15, 30, 45, 60 to the lords | (a) for the weights; lords reckoned by SS I.51 and I.52 (a); hora division (d), [`HoraKind`] |
//! | Ayana | 27.15 to 27.17: the khanda method on the sayana longitude | (a); the declination reading is (b), [`AyanaReading`] |
//! | Yuddha | 27.20: the difference of the two shadbalas goes to the victor from the vanquished | (a); the winner rule is the caller's [`YuddhaRule`] |
//! | Cheshta | 27.24 and 27.25: the cheshta kendra, folded, over 3 for Mars to Saturn; 27.18 for the Sun and Moon | (a) |
//! | Naisargika | 27.14 | (a), unchanged |
//!
//! **The pairing the totals use (27.18).** The Sun's cheshta bala is his ayana
//! bala and the Moon's is her paksha bala. Both are therefore counted twice in
//! a graha's total, once in kala bala and once in cheshta bala, each
//! *undoubled* in its own field. A convention that doubles the Sun's ayana and
//! the Moon's paksha inside kala bala and gives them no cheshta reaches the
//! same total with different parts; this module does not.
//!
//! # What neither form covers
//!
//! * **Dig Bala** is the bhava-step grade ([`dig_bala`]), 10 virupas per house
//!   from the weak house. BPHS 27.7 makes it continuous in the degrees from
//!   the weak angle's cusp, which needs the four cusps; not implemented.
//! * **Drik Bala** is [`drik_bala`], fifteen virupas per aspect received. BPHS
//!   27.19 takes a quarter of the drishti pinda; not implemented.
//! * **Bhava bala** (27.26 to 27.31) is a separate computation.
//!
//! # Coverage of the coarse form
//!
//! Sthana Bala is Uchcha, Ojhayugma, Kendradi and Drekkana; Saptavargaja is
//! absent. Kala Bala is Nathonnata and Paksha as 0-or-30 steps; Tribhaga, the
//! abda-masa-vara-hora set, Ayana and Yuddha are absent. Cheshta Bala is the
//! speed grade of [`cheshta_bala`], which is not a verse rule. Dig, Naisargika
//! and Drik Bala are as above.

use crate::cheshta::{YuddhaRule, graha_yuddha};
use crate::graha::{Graha, GrahaPosition};
use crate::kala_khanda::hora;
use crate::mean_motion::mean_longitude;
use crate::muhurta::Weekday;
use crate::varga::{VargaType, varga_sign};
use serde::Serialize;

/// Shadbala (six-fold strength) for a planet.
///
/// All component values are in virupas (shashtiamsas, 1/60 of a rupa).
#[derive(Debug, Clone, Serialize)]
pub struct Shadbala {
    /// The planet this strength applies to.
    pub planet: Graha,
    /// Positional strength: the sum of `uccha_bala`, `ojhayugma_bala`,
    /// `kendradi_bala` and `drekkana_bala`. Saptavargaja Bala is not
    /// included — see the module documentation.
    pub sthana_bala: f64,
    /// Directional strength (planets strong in certain houses).
    pub dig_bala: f64,
    /// Temporal strength: the day/night and paksha steps of [`kala_bala`] (the
    /// coarse form; see the module documentation for what it omits).
    pub kala_bala: f64,
    /// Motional strength: the speed grade of [`cheshta_bala`] (the coarse
    /// form).
    pub cheshta_bala: f64,
    /// Natural strength (fixed per planet).
    pub naisargika_bala: f64,
    /// Aspectual strength (benefic/malefic aspects).
    pub drik_bala: f64,
    /// Sum of the six components above.
    pub total: f64,
    /// Uchcha Bala — exaltation sub-component of Sthana Bala (0–60
    /// virupas). Source: BPHS Ch.27.
    pub uccha_bala: f64,
    /// Ojhayugma Bala — odd/even rasi and navamsa sub-component of Sthana
    /// Bala (0–30 virupas). Source: BPHS Ch.27.
    pub ojhayugma_bala: f64,
    /// Kendradi Bala — angular/succedent/cadent sub-component of Sthana
    /// Bala (15–60 virupas). Source: BPHS Ch.27.
    pub kendradi_bala: f64,
    /// Drekkana Bala — decanate sub-component of Sthana Bala (0 or 15
    /// virupas). Source: BPHS Ch.27.
    pub drekkana_bala: f64,
    /// Benefit strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 vv.5-6.
    pub ishta_phala: f64,
    /// Affliction strength per Rasmi scale (0–60 virupas). Source: BPHS Ch.28 v.6.
    pub kashta_phala: f64,
}

/// Additional temporal/motional parameters the six-fold computation needs
/// beyond a bare [`GrahaPosition`].
#[derive(Debug, Clone, Copy)]
pub struct ShadbalaPlanetData {
    /// The planet position.
    pub position: GrahaPosition,
    /// Daily speed in degrees/day. Negative = retrograde.
    pub speed: f64,
    /// Average daily speed for this planet (used to classify motion).
    pub average_speed: f64,
    /// Number of benefic aspects received by this planet.
    pub benefic_aspect_count: u32,
    /// Number of malefic aspects received by this planet.
    pub malefic_aspect_count: u32,
}

// ── Naisargika Bala (natural strength) ──────────────────────────────

/// Natural strength — fixed values per planet (in virupas).
///
/// From brightest/strongest to weakest:
/// Sun (60), Moon (51.43), Venus (42.86), Jupiter (34.29),
/// Mercury (25.71), Mars (17.14), Saturn (8.57).
#[must_use]
pub fn naisargika_bala(planet: Graha) -> f64 {
    match planet {
        Graha::Sun => 60.0,
        Graha::Moon => 51.43,
        Graha::Venus => 42.86,
        Graha::Jupiter => 34.29,
        Graha::Mercury => 25.71,
        Graha::Mars => 17.14,
        Graha::Saturn => 8.57,
        // Rahu/Ketu not traditionally part of Shadbala
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

// ── Dig Bala (directional strength) ─────────────────────────────────

/// The house in which a planet has maximum directional strength.
///
/// - Sun, Mars -> 10th (south)
/// - Moon, Venus -> 4th (north)
/// - Mercury, Jupiter -> 1st (east)
/// - Saturn -> 7th (west)
#[must_use]
fn dig_bala_strong_house(planet: Graha) -> u8 {
    match planet {
        Graha::Sun | Graha::Mars => 10,
        Graha::Moon | Graha::Venus => 4,
        Graha::Mercury | Graha::Jupiter | Graha::Rahu | Graha::Ketu => 1,
        Graha::Saturn => 7,
    }
}

/// Directional strength in virupas (0-60).
///
/// Maximum (60) when planet is in its strong house.
/// Minimum (0) when planet is in the opposite house (6 houses away).
/// Linear interpolation in between.
#[must_use]
pub fn dig_bala(planet: Graha, bhava: u8) -> f64 {
    let strong = dig_bala_strong_house(planet);
    // Distance in houses (circular, 1-indexed, max 6)
    let raw_dist = bhava.abs_diff(strong);
    let dist = if raw_dist > 6 {
        12 - raw_dist
    } else {
        raw_dist
    };
    // 60 at distance 0, 0 at distance 6
    let bala = 60.0 - (f64::from(dist) * 10.0);
    bala.max(0.0)
}

// ── Sthana Bala (positional strength) ───────────────────────────────

/// Exaltation longitude for each planet in degrees (0-360).
///
/// Source: BPHS Ch. 3 Sl. 18.
fn exaltation_longitude(planet: Graha) -> f64 {
    match planet {
        Graha::Sun => 10.0,      // 10° Aries
        Graha::Moon => 33.0,     // 3° Taurus
        Graha::Mars => 298.0,    // 28° Capricorn
        Graha::Mercury => 165.0, // 15° Virgo
        Graha::Jupiter => 95.0,  // 5° Cancer
        Graha::Venus => 357.0,   // 27° Pisces
        Graha::Saturn => 200.0,  // 20° Libra
        Graha::Rahu => 50.0,     // 20° Taurus
        Graha::Ketu => 230.0,    // 20° Scorpio
    }
}

/// Uchcha Bala — degree-precise exaltation strength in virupas (0-60).
///
/// Defining rule (BPHS Ch. 27 Sl. 3-6): a planet at its exaltation degree has
/// 60 virupas, at its debilitation degree 0, and the value falls linearly with
/// the arc between:
///
/// ```text
/// uccha_bala = (180 - arc) / 3
/// arc        = min(|longitude - exaltation_longitude|,
///                  360 - |longitude - exaltation_longitude|)
/// ```
///
/// This is one of the five sub-components of Sthana Bala, not Sthana Bala
/// itself. Through v7.1.1 it was exported under the name `sthana_bala` and
/// took a `sign` argument it discarded; see [`sthana_bala_full`] for the
/// composite, and [`sthana_bala`] for the retained v7 spelling.
#[must_use]
pub fn uccha_bala(planet: Graha, longitude: f64) -> f64 {
    let exalt_lon = exaltation_longitude(planet);
    let raw_diff = (longitude - exalt_lon).abs();
    let arc = if raw_diff > 180.0 {
        360.0 - raw_diff
    } else {
        raw_diff
    };
    ((180.0 - arc) / 3.0).max(0.0)
}

/// Ojhayugma Bala — odd/even strength in virupas (0-30).
///
/// Defining rule (BPHS Ch. 27): the Moon and Venus are strengthened by even
/// (yugma) signs; the other five grahas by odd (oja) signs. The test is
/// applied twice — once to the rasi (D-1) and once to the navamsa (D-9) — and
/// each agreement is worth 15 virupas.
///
/// Sign indices are 0-based, so an *odd* sign in the classical 1-based
/// reckoning (Aries, Gemini, …) is an *even* index here.
///
/// Rahu and Ketu are outside the classical rule and score 0.
#[must_use]
pub fn ojhayugma_bala(planet: Graha, longitude: f64) -> f64 {
    if matches!(planet, Graha::Rahu | Graha::Ketu) {
        return 0.0;
    }
    // Classical "odd sign" == 0-based index even.
    let wants_even_sign = matches!(planet, Graha::Moon | Graha::Venus);

    let rasi = varga_sign(longitude, VargaType::Rashi);
    let navamsa = varga_sign(longitude, VargaType::Navamsha);

    let mut bala = 0.0;
    for sign in [rasi, navamsa] {
        let sign_is_classically_even = sign % 2 == 1;
        if sign_is_classically_even == wants_even_sign {
            bala += 15.0;
        }
    }
    bala
}

/// Kendradi Bala — quadrant strength in virupas (15, 30 or 60).
///
/// Defining rule (BPHS Ch. 27): a planet in a kendra (houses 1, 4, 7, 10)
/// scores 60 virupas; in a panapara (2, 5, 8, 11) 30; in an apoklima
/// (3, 6, 9, 12) 15. The rule depends only on the house, not on the planet.
///
/// A `bhava` outside 1-12 scores 0.
#[must_use]
pub fn kendradi_bala(bhava: u8) -> f64 {
    match bhava {
        1 | 4 | 7 | 10 => 60.0,
        2 | 5 | 8 | 11 => 30.0,
        3 | 6 | 9 | 12 => 15.0,
        _ => 0.0,
    }
}

/// Which drekkana each sex is strengthened in (BPHS 27.6).
///
/// The verse (grade (a)) reads, in transliteration, *adya-madhya-avasaneshu
/// drekkaneshu sthitah kramat; pum-napumsaka-yosha-akhya dadyur anghrimitam
/// balam*: "stationed in the first, middle and last drekkana respectively,
/// those called male, neuter and female give a quarter [of a rupa]". The words
/// run male, neuter, female, against first, middle, last, which is
/// [`DrekkanaOrder::MaleNeuterFemale`], the default.
///
/// Some English renderings of the verse put the female graha before the neuter
/// one, which gives the female the second drekkana and the neuter the third.
/// That is [`DrekkanaOrder::MaleFemaleNeuter`]; it is carried so a consumer who
/// follows such a rendering can say so, and it is not the order of the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrekkanaOrder {
    /// Male in the first, neuter in the second, female in the third: the order
    /// of the words of BPHS 27.6. Sun, Mars, Jupiter are male; Mercury, Saturn
    /// neuter; Moon, Venus female.
    #[default]
    MaleNeuterFemale,
    /// Male in the first, female in the second, neuter in the third.
    MaleFemaleNeuter,
}

/// Drekkana Bala — decanate strength in virupas (0 or 15).
///
/// Defining rule (BPHS Ch. 27 Sl. 6, grade (a)): the male grahas (Sun, Mars,
/// Jupiter) gain 15 virupas in the first drekkana of whatever sign they occupy,
/// the neuter grahas (Mercury, Saturn) in the second, and the female grahas
/// (Moon, Venus) in the third. There is no partial credit. This is the order of
/// the words of the verse; see [`DrekkanaOrder`] and [`drekkana_bala_ordered`]
/// for the other order some renderings give.
///
/// The drekkana here is the third of the *occupied sign* — degrees 0-10,
/// 10-20 and 20-30 within it — which is what the rule is stated over, not the
/// D-3 varga sign that [`crate::varga::varga_sign`] returns.
///
/// Rahu and Ketu are outside the classical rule and score 0.
#[must_use]
pub fn drekkana_bala(planet: Graha, longitude: f64) -> f64 {
    drekkana_bala_ordered(planet, longitude, DrekkanaOrder::MaleNeuterFemale)
}

/// [`drekkana_bala`] with the drekkana order chosen.
#[must_use]
pub fn drekkana_bala_ordered(planet: Graha, longitude: f64, order: DrekkanaOrder) -> f64 {
    let (neuter_third, female_third) = match order {
        DrekkanaOrder::MaleNeuterFemale => (1u8, 2u8),
        DrekkanaOrder::MaleFemaleNeuter => (2u8, 1u8),
    };
    let wanted = match planet {
        Graha::Sun | Graha::Mars | Graha::Jupiter => 0u8,
        Graha::Mercury | Graha::Saturn => neuter_third,
        Graha::Moon | Graha::Venus => female_third,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    let within_sign = longitude.rem_euclid(360.0) % 30.0;
    // 0-10 deg -> 0, 10-20 -> 1, 20-30 -> 2. `min` guards the 30.0 boundary
    // that floating-point rounding can produce.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = ((within_sign / 10.0) as u8).min(2);
    if index == wanted { 15.0 } else { 0.0 }
}

/// Sthana Bala — positional strength in virupas (0-165).
///
/// Named `_full` for the same reason [`compute_shadbala_full`] is: the plain
/// spelling is taken by the v7 function this replaces.
///
/// The sum of the four implemented sub-components: [`uccha_bala`],
/// [`ojhayugma_bala`], [`kendradi_bala`] and [`drekkana_bala`].
///
/// **Saptavargaja Bala is not included.** BPHS Ch. 27 makes it the fifth
/// sub-component — the planet's dignity in each of the seven vargas, graded
/// by the five-fold friendship — and computing it needs a Moolatrikona degree
/// table and a panchadha maitri derivation this crate does not yet carry from
/// a primary source. Inventing either would be worse than the documented gap,
/// so the value returned here is the true Sthana Bala less that term.
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn sthana_bala_full(planet: Graha, longitude: f64, bhava: u8) -> f64 {
    uccha_bala(planet, longitude)
        + ojhayugma_bala(planet, longitude)
        + kendradi_bala(bhava)
        + drekkana_bala(planet, longitude)
}

/// Uchcha Bala under its historical name.
///
/// This is what `sthana_bala` returned through v7.1.1: the Uchcha
/// sub-component alone, with `sign` accepted and discarded. It is kept at that
/// exact behaviour so a v7 dependent keeps compiling and keeps getting the
/// number it got before, and is deprecated in favour of the two functions that
/// say what they are — [`uccha_bala`] for this value, [`sthana_bala_full`] for
/// the composite Sthana Bala.
#[must_use]
#[deprecated(
    since = "7.2.0",
    note = "returns Uchcha Bala only, not Sthana Bala: call uccha_bala for this value, or sthana_bala_full for the composite"
)]
pub fn sthana_bala(planet: Graha, _sign: u8, longitude: f64) -> f64 {
    uccha_bala(planet, longitude)
}

// ── Kala Bala (temporal strength) ───────────────────────────────────

/// Temporal strength in virupas, coarse form: two of Kala Bala's parts as
/// steps.
///
/// **Not the verse's rule.** BPHS 27.8 to 27.11 make Nathonnata and Paksha
/// Bala continuous, up to 60 virupas each; this keeps the 0-or-30 steps the
/// boolean inputs allow, and omits Tribhaga, Abda, Masa, Vara, Hora, Ayana
/// and Yuddha. It is what the MCP and WASM surfaces serve. For the verse use
/// [`nathonnata_bala`], [`paksha_bala`] and the rest, or
/// [`compute_shadbala_complete`].
///
/// - **Nathonnatha:** Sun, Jupiter, Venus strong during day; Moon, Mars, Saturn
///   strong at night; Mercury always strong. 30 virupas.
/// - **Paksha:** Benefics (Jupiter, Venus, Mercury, Moon) strong in Shukla
///   Paksha (waxing); the rest strong in Krishna Paksha (waning). 30 virupas.
#[must_use]
pub fn kala_bala(planet: Graha, is_daytime: bool, moon_phase_waxing: bool) -> f64 {
    let mut bala = 0.0;

    // Nathonnatha Bala (day/night strength)
    let day_strong = matches!(planet, Graha::Sun | Graha::Jupiter | Graha::Venus);
    let night_strong = matches!(planet, Graha::Moon | Graha::Mars | Graha::Saturn);

    if planet == Graha::Mercury {
        bala += 30.0; // Mercury is always strong (twilight planet)
    } else if (is_daytime && day_strong) || (!is_daytime && night_strong) {
        bala += 30.0;
    }

    // Paksha Bala (lunar phase strength)
    let is_benefic = matches!(
        planet,
        Graha::Jupiter | Graha::Venus | Graha::Mercury | Graha::Moon
    );
    if (moon_phase_waxing && is_benefic) || (!moon_phase_waxing && !is_benefic) {
        bala += 30.0;
    }

    bala
}

// ── Cheshta Bala (motional strength) ────────────────────────────────

/// Motional strength in virupas based on planetary speed, coarse form.
///
/// **Not a verse rule.** BPHS 27.24 and 27.25 take Cheshta Bala from the
/// cheshta kendra (see [`cheshta_kendra`], [`cheshta_bala_at`]); 27.18 gives
/// the Sun the ayana bala and the Moon the paksha bala. This speed grade is a
/// stand-in for callers who have a speed and no mean longitudes, and it is
/// what the MCP and WASM surfaces serve. 27.21 to 27.23 do grade eight motion
/// states by strength; [`crate::cheshta::Cheshta::virupas`] carries those.
///
/// - Retrograde: 60 virupas (strongest — planet re-traverses degrees)
/// - Stationary (speed near 0): 30 virupas
/// - Direct slow (< 50% average): 15 virupas
/// - Direct normal: 30 virupas
/// - Direct fast (> 150% average): 45 virupas
///
/// Sun and Moon are never retrograde; for them this returns 30 (normal).
#[must_use]
pub fn cheshta_bala(planet: Graha, speed: f64, average_speed: f64) -> f64 {
    // Sun and Moon have no retrograde motion — assign normal strength
    if matches!(planet, Graha::Sun | Graha::Moon) {
        return 30.0;
    }
    // Rahu/Ketu always move retrograde but are not part of traditional Shadbala
    if matches!(planet, Graha::Rahu | Graha::Ketu) {
        return 0.0;
    }

    if speed < 0.0 {
        60.0 // retrograde
    } else if speed.abs() < 0.01 {
        30.0 // stationary
    } else if speed < average_speed * 0.5 {
        15.0 // slow
    } else if speed > average_speed * 1.5 {
        45.0 // fast
    } else {
        30.0 // normal
    }
}

// ── Drik Bala (aspectual strength) ──────────────────────────────────

/// Aspectual strength in virupas.
///
/// Each benefic aspect adds 15 virupas; each malefic aspect subtracts 15.
/// Clamped to [-30, 60].
///
/// Source: BPHS Ch. 27.
#[must_use]
pub fn drik_bala(benefic_aspect_count: u32, malefic_aspect_count: u32) -> f64 {
    let benefic_strength = f64::from(benefic_aspect_count) * 15.0;
    let malefic_strength = f64::from(malefic_aspect_count) * 15.0;
    (benefic_strength - malefic_strength).clamp(-30.0, 60.0)
}

// ── Ishta/Kashta Phala (BPHS Ch.28 vv.5-6) ─────────────────────────

/// Uchcha Rasmi: linear transform of uccha_bala virupas to 0–7 scale.
///
/// Source: BPHS Ch.28 v.5.
#[must_use]
pub fn uccha_rasmi(uccha_bala_virupas: f64) -> f64 {
    uccha_bala_virupas * 7.0 / 60.0
}

/// Cheshta Rasmi: linear transform of cheshta_bala virupas to 0–7 scale.
///
/// Source: BPHS Ch.28 v.5.
#[must_use]
pub fn cheshta_rasmi(cheshta_bala_virupas: f64) -> f64 {
    cheshta_bala_virupas * 7.0 / 60.0
}

/// Ishta Phala (benefit) and Kashta Phala (affliction) from Rasmi values.
///
/// Formula: ishta = 5 × (uchcha_rasmi + cheshta_rasmi − 2), clipped to [0, 60].
/// kashta = 60 − ishta. Source: BPHS Ch.28 vv.5-6.
#[must_use]
pub fn ishta_kashta_phala(uccha_bala: f64, cheshta_bala: f64) -> (f64, f64) {
    let ur = uccha_rasmi(uccha_bala);
    let cr = cheshta_rasmi(cheshta_bala);
    let ishta = (5.0 * (ur + cr - 2.0)).clamp(0.0, 60.0);
    (ishta, 60.0 - ishta)
}

// ── Compute Shadbala ────────────────────────────────────────────────

/// Compute Shadbala from bare positions — Sthana, Dig and Naisargika Bala only.
///
/// Kala, Cheshta and Drik Bala need data a [`GrahaPosition`] does not carry, so
/// they are zero and `total` is the sum of the three that are computed, not of
/// six. That was true before v7.2.0 too, where the struct documented `total` as
/// a six-component sum regardless. Use [`compute_shadbala_full`].
///
/// `lagna_sign` is accepted for signature compatibility and not read.
#[must_use]
#[deprecated(
    since = "7.2.0",
    note = "computes three of the six components; use compute_shadbala_full"
)]
pub fn compute_shadbala(positions: &[GrahaPosition], _lagna_sign: u8) -> Vec<Shadbala> {
    positions
        .iter()
        .map(|pos| {
            let naisargika = naisargika_bala(pos.planet);
            let dig = dig_bala(pos.planet, pos.bhava);

            let uccha = uccha_bala(pos.planet, pos.longitude);
            let ojhayugma = ojhayugma_bala(pos.planet, pos.longitude);
            let kendradi = kendradi_bala(pos.bhava);
            let drekkana = drekkana_bala(pos.planet, pos.longitude);
            let sthana = uccha + ojhayugma + kendradi + drekkana;

            let (ishta, kashta) = ishta_kashta_phala(uccha, 0.0);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: 0.0,
                cheshta_bala: 0.0,
                naisargika_bala: naisargika,
                drik_bala: 0.0,
                total: naisargika + dig + sthana,
                uccha_bala: uccha,
                ojhayugma_bala: ojhayugma,
                kendradi_bala: kendradi,
                drekkana_bala: drekkana,
                ishta_phala: ishta,
                kashta_phala: kashta,
            }
        })
        .collect()
}

/// Compute Shadbala — all six components — for planets with extended data,
/// coarse form.
///
/// Sthana Bala omits Saptavargaja Bala, Kala Bala covers Nathonnatha and
/// Paksha only as steps, and Cheshta Bala is the speed grade; see the module
/// documentation for the full accounting. This is the form the MCP and WASM
/// surfaces serve and its answers do not change; the form that follows the
/// verse is [`compute_shadbala_complete`].
///
/// # Arguments
/// * `planets` — slice of extended planet data (position, speed, aspects)
/// * `is_daytime` — whether the chart is for daytime (Sun above horizon)
/// * `moon_phase_waxing` — whether the Moon is in Shukla Paksha (waxing)
#[must_use]
pub fn compute_shadbala_full(
    planets: &[ShadbalaPlanetData],
    is_daytime: bool,
    moon_phase_waxing: bool,
) -> Vec<Shadbala> {
    planets
        .iter()
        .map(|data| {
            let pos = &data.position;
            let naisargika = naisargika_bala(pos.planet);
            let dig = dig_bala(pos.planet, pos.bhava);

            let uccha = uccha_bala(pos.planet, pos.longitude);
            let ojhayugma = ojhayugma_bala(pos.planet, pos.longitude);
            let kendradi = kendradi_bala(pos.bhava);
            let drekkana = drekkana_bala(pos.planet, pos.longitude);
            let sthana = uccha + ojhayugma + kendradi + drekkana;

            let kala = kala_bala(pos.planet, is_daytime, moon_phase_waxing);
            let cheshta = cheshta_bala(pos.planet, data.speed, data.average_speed);
            let drik = drik_bala(data.benefic_aspect_count, data.malefic_aspect_count);
            let total = naisargika + dig + sthana + kala + cheshta + drik;

            // BPHS Ch.28 v.5 builds the Rasmis from Uchcha Bala, not from the
            // Sthana Bala composite that contains it.
            let (ishta, kashta) = ishta_kashta_phala(uccha, cheshta);
            Shadbala {
                planet: pos.planet,
                sthana_bala: sthana,
                dig_bala: dig,
                kala_bala: kala,
                cheshta_bala: cheshta,
                naisargika_bala: naisargika,
                drik_bala: drik,
                total,
                uccha_bala: uccha,
                ojhayugma_bala: ojhayugma,
                kendradi_bala: kendradi,
                drekkana_bala: drekkana,
                ishta_phala: ishta,
                kashta_phala: kashta,
            }
        })
        .collect()
}

// ── Sign lords, moolatrikona and the five-fold friendship ───────────
//
// BPHS chapter 3, slokas 49 to 58. The seven classical grahas only; Rahu and
// Ketu have no moolatrikona, no sign and no place in the friendship.

/// The lord of a sign (0 = Aries … 11 = Pisces): the seven classical grahas'
/// own signs (Sun Leo; Moon Cancer; Mars Aries and Scorpio; Mercury Gemini and
/// Virgo; Jupiter Sagittarius and Pisces; Venus Taurus and Libra; Saturn
/// Capricorn and Aquarius). A sign of 12 or more is read modulo 12.
#[must_use]
pub fn sign_lord(sign: u8) -> Graha {
    match sign % 12 {
        0 | 7 => Graha::Mars,
        1 | 6 => Graha::Venus,
        2 | 5 => Graha::Mercury,
        3 => Graha::Moon,
        4 => Graha::Sun,
        8 | 11 => Graha::Jupiter,
        // 9 and 10.
        _ => Graha::Saturn,
    }
}

/// The sign (0-11) a longitude falls in.
fn sign_of(longitude: f64) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let sign = (longitude.rem_euclid(360.0) / 30.0) as u8;
    sign.min(11)
}

/// The seven grahas of Shadbala, in the order Sun to Saturn.
const CLASSICAL: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

fn is_classical(planet: Graha) -> bool {
    !matches!(planet, Graha::Rahu | Graha::Ketu)
}

/// A graha's moolatrikona: the sign and the degree arc within it (BPHS 3.51
/// to 3.54, grade (a)). The arc is half-open, `from_deg <= degree < to_deg`;
/// the rest of the sign is the graha's own sign, except for the Moon, whose
/// first 3 degrees of Vrishabha are her exaltation, and Mercury, whose first
/// 15 degrees of Kanya are his.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoolatrikonaArc {
    /// The sign (0 = Aries).
    pub sign: u8,
    /// Degree within the sign where the moolatrikona begins.
    pub from_deg: f64,
    /// Degree within the sign where it ends (exclusive).
    pub to_deg: f64,
}

/// The moolatrikona of a graha, or `None` for Rahu and Ketu.
///
/// | Graha | Sign | Arc |
/// |---|---|---|
/// | Sun | Simha | 0 to 20 |
/// | Moon | Vrishabha | 3 to 30 (0 to 3 is uchcha) |
/// | Mars | Mesha | 0 to 12 |
/// | Mercury | Kanya | 15 to 20 (0 to 15 is uchcha) |
/// | Jupiter | Dhanu | 0 to 10 |
/// | Venus | Tula | 0 to 15 |
/// | Saturn | Kumbha | 0 to 20 |
#[must_use]
pub fn moolatrikona_arc(planet: Graha) -> Option<MoolatrikonaArc> {
    let (sign, from_deg, to_deg) = match planet {
        Graha::Sun => (4, 0.0, 20.0),
        Graha::Moon => (1, 3.0, 30.0),
        Graha::Mars => (0, 0.0, 12.0),
        Graha::Mercury => (5, 15.0, 20.0),
        Graha::Jupiter => (8, 0.0, 10.0),
        Graha::Venus => (6, 0.0, 15.0),
        Graha::Saturn => (10, 0.0, 20.0),
        Graha::Rahu | Graha::Ketu => return None,
    };
    Some(MoolatrikonaArc {
        sign,
        from_deg,
        to_deg,
    })
}

/// Whether a D1 longitude lies in the graha's moolatrikona arc.
#[must_use]
pub fn in_moolatrikona(planet: Graha, longitude: f64) -> bool {
    let Some(arc) = moolatrikona_arc(planet) else {
        return false;
    };
    let within = longitude.rem_euclid(30.0);
    sign_of(longitude) == arc.sign && within >= arc.from_deg && within < arc.to_deg
}

/// Friend, neutral or enemy: the three-fold relation of one graha to another,
/// natural ([`natural_relation`]) or temporary ([`temporary_relation`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Friendship {
    /// A friend.
    Friend,
    /// Neither (natural relation only).
    Neutral,
    /// An enemy.
    Enemy,
}

/// The five-fold relation (panchadha maitri) of BPHS 3.57.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maitri {
    /// Extreme friend: friend by nature and friend for the moment.
    AdhiMitra,
    /// Friend: neutral by nature, friend for the moment.
    Mitra,
    /// Neutral: friend by one measure and enemy by the other.
    Sama,
    /// Enemy: neutral by nature, enemy for the moment.
    Shatru,
    /// Extreme enemy: enemy by nature and enemy for the moment.
    AdhiShatru,
}

/// Whether `sign` is, for `planet`, one of the signs whose lords are its
/// natural friends (BPHS 3.55, grade (a)): the signs 2, 4, 5, 8, 9 and 12
/// from the graha's moolatrikona sign, and the sign of its own exaltation.
fn is_natural_friend_sign(planet: Graha, sign: u8) -> bool {
    let Some(arc) = moolatrikona_arc(planet) else {
        return false;
    };
    let offset = (sign + 12 - arc.sign) % 12;
    let exaltation_sign = sign_of(exaltation_longitude(planet));
    matches!(offset, 1 | 3 | 4 | 7 | 8 | 11) || sign == exaltation_sign
}

/// How `of` regards `to` by nature (naisargika maitri), derived from BPHS 3.55:
/// from `of`'s moolatrikona sign, the lords of the 2nd, 4th, 5th, 8th, 9th and
/// 12th signs and the lord of `of`'s exaltation sign are its friends, the
/// lords of the other signs are its enemies, and a graha that owns signs of
/// both kinds is neutral. Grade (a).
///
/// The relation is not symmetric (the Moon counts Mars neutral; Mars counts
/// the Moon a friend). `None` for the same graha, or for Rahu or Ketu.
#[must_use]
pub fn natural_relation(of: Graha, to: Graha) -> Option<Friendship> {
    if of == to || !is_classical(of) || !is_classical(to) {
        return None;
    }
    let mut owns_friend_sign = false;
    let mut owns_other_sign = false;
    for sign in 0..12u8 {
        if sign_lord(sign) == to {
            if is_natural_friend_sign(of, sign) {
                owns_friend_sign = true;
            } else {
                owns_other_sign = true;
            }
        }
    }
    Some(match (owns_friend_sign, owns_other_sign) {
        (true, false) => Friendship::Friend,
        (false, true) => Friendship::Enemy,
        // Both kinds of sign (neutral); every classical graha owns a sign,
        // so (false, false) cannot occur.
        (true, true) | (false, false) => Friendship::Neutral,
    })
}

/// The temporary relation (tatkalika maitri, BPHS 3.56, grade (a)): grahas in
/// the 2nd, 3rd, 4th, 10th, 11th or 12th sign from one another are friends for
/// the moment; any other placement, including the same sign, makes them
/// enemies. Signs are 0-11, from the D1 chart. Symmetric.
#[must_use]
pub fn temporary_relation(of_sign: u8, to_sign: u8) -> Friendship {
    let offset = (to_sign % 12 + 12 - of_sign % 12) % 12;
    if matches!(offset, 1 | 2 | 3 | 9 | 10 | 11) {
        Friendship::Friend
    } else {
        Friendship::Enemy
    }
}

/// The five-fold relation of `of` to `to` (BPHS 3.57, grade (a)): friend by
/// nature and for the moment is adhimitra; neutral by nature and a friend for
/// the moment, mitra; a friend by one measure and an enemy by the other, sama;
/// neutral by nature and an enemy for the moment, shatru; an enemy by both,
/// adhishatru. `None` where [`natural_relation`] is.
#[must_use]
pub fn panchadha_maitri(of: Graha, to: Graha, of_sign: u8, to_sign: u8) -> Option<Maitri> {
    let natural = natural_relation(of, to)?;
    let temporary = temporary_relation(of_sign, to_sign);
    Some(match (natural, temporary) {
        (Friendship::Friend, Friendship::Friend) => Maitri::AdhiMitra,
        (Friendship::Neutral, Friendship::Friend) => Maitri::Mitra,
        (Friendship::Neutral, Friendship::Enemy) => Maitri::Shatru,
        (Friendship::Enemy, Friendship::Enemy) => Maitri::AdhiShatru,
        // A friend by one measure and an enemy by the other. The temporary
        // relation is never neutral; that arm only completes the match.
        (Friendship::Friend, Friendship::Enemy)
        | (Friendship::Enemy, Friendship::Friend)
        | (_, Friendship::Neutral) => Maitri::Sama,
    })
}

// ── Saptavargaja Bala (BPHS 27.2 to 27.4) ───────────────────────────

/// The seven vargas of Saptavargaja Bala, in the order the verse lists them:
/// rasi, hora, drekkana, saptamsha, navamsha, dvadashamsha, trimshamsha.
pub const SAPTAVARGA: [VargaType; 7] = [
    VargaType::Rashi,
    VargaType::Hora,
    VargaType::Drekkana,
    VargaType::Saptamsha,
    VargaType::Navamsha,
    VargaType::Dwadashamsha,
    VargaType::Trimshamsha,
];

/// The virupas Saptavargaja Bala awards per varga, by the dignity of the sign
/// the graha is in. BPHS 27.2 and 27.3 give 45, 30, 20, 15, 10, 4 and 2
/// ([`SaptavargajaLadder::BPHS`], the default); the ladder is a field so a
/// consumer following another authority's scale can say so.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaptavargajaLadder {
    /// In the graha's moolatrikona.
    pub moolatrikona: f64,
    /// In the graha's own sign.
    pub own: f64,
    /// In the sign of an adhimitra.
    pub adhimitra: f64,
    /// In the sign of a mitra.
    pub mitra: f64,
    /// In the sign of a sama.
    pub sama: f64,
    /// In the sign of a shatru.
    pub shatru: f64,
    /// In the sign of an adhishatru.
    pub adhishatru: f64,
}

impl SaptavargajaLadder {
    /// BPHS 27.2 and 27.3: 45, 30, 20, 15, 10, 4, 2.
    pub const BPHS: Self = Self {
        moolatrikona: 45.0,
        own: 30.0,
        adhimitra: 20.0,
        mitra: 15.0,
        sama: 10.0,
        shatru: 4.0,
        adhishatru: 2.0,
    };

    fn for_maitri(self, maitri: Maitri) -> f64 {
        match maitri {
            Maitri::AdhiMitra => self.adhimitra,
            Maitri::Mitra => self.mitra,
            Maitri::Sama => self.sama,
            Maitri::Shatru => self.shatru,
            Maitri::AdhiShatru => self.adhishatru,
        }
    }
}

impl Default for SaptavargajaLadder {
    fn default() -> Self {
        Self::BPHS
    }
}

/// Where the moolatrikona grade (45) can be awarded in Saptavargaja Bala.
///
/// BPHS 27.4 says the values recur in the other six vargas ("evam" before the
/// list hora, drekkana, saptamsha, navamsha, dvadashamsha, trimshamsha). The
/// moolatrikona of 3.51 to 3.54 is a degree arc of the rasi, and a position in
/// a divisional chart is a sign without such an arc, so the verse leaves open
/// how the grade is tested there. Grade (b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoolatrikonaScope {
    /// D1 by the degree arc; in each other varga, by the varga *sign*: a graha
    /// whose varga sign is its moolatrikona sign gets the moolatrikona grade.
    /// This follows "evam": the full ladder is available in every varga. The
    /// default.
    #[default]
    AllVargas,
    /// D1 by the degree arc only; in the other six vargas a graha in its own
    /// sign gets the own-sign grade and the moolatrikona grade is not awarded.
    RashiOnly,
}

/// Saptavargaja Bala: the graha's dignity in the sign it holds in each of the
/// seven vargas, in virupas (BPHS 27.2 to 27.4, grade (a); the moolatrikona
/// outside D1 is grade (b), see [`MoolatrikonaScope`]).
///
/// Per varga: the moolatrikona grade if the graha is in its moolatrikona, the
/// own-sign grade if the varga sign is one it owns, otherwise the grade of the
/// five-fold relation ([`panchadha_maitri`]) to the lord of the varga sign.
/// The relation's temporary half is read from the D1 positions of the two
/// grahas, never from the varga charts. The Moon in the first 3 degrees of
/// Vrishabha (her exaltation) and Mercury in the first 15 of Kanya are not in
/// the moolatrikona arc; the Moon is graded as in the sign of Venus, Mercury
/// as in his own.
///
/// `positions` must hold the D1 position of the graha and of every sign lord
/// the seven vargas can name, which for a complete chart is all seven grahas.
/// `None` if one is missing. Rahu and Ketu score 0.
#[must_use]
pub fn saptavargaja_bala(
    planet: Graha,
    positions: &[GrahaPosition],
    scope: MoolatrikonaScope,
    ladder: SaptavargajaLadder,
) -> Option<f64> {
    let Some(arc) = moolatrikona_arc(planet) else {
        return Some(0.0);
    };
    let sign_of_graha = |g: Graha| {
        positions
            .iter()
            .find(|p| p.planet == g)
            .map(|p| sign_of(p.longitude))
    };
    let longitude = positions.iter().find(|p| p.planet == planet)?.longitude;
    let own_sign = sign_of(longitude);

    let mut total = 0.0;
    for (index, varga) in SAPTAVARGA.iter().enumerate() {
        let varga_sign = varga_sign(longitude, *varga);
        let lord = sign_lord(varga_sign);
        let in_trikona = if index == 0 {
            in_moolatrikona(planet, longitude)
        } else {
            scope == MoolatrikonaScope::AllVargas && varga_sign == arc.sign
        };
        total += if in_trikona {
            ladder.moolatrikona
        } else if lord == planet {
            ladder.own
        } else {
            let maitri = panchadha_maitri(planet, lord, own_sign, sign_of_graha(lord)?)?;
            ladder.for_maitri(maitri)
        };
    }
    Some(total)
}

// ── Nathonnata and Paksha Bala (BPHS 27.8 to 27.11) ─────────────────

/// Nathonnata Bala in virupas (BPHS 27.8 and 27.9, grade (a)).
///
/// The time from local midnight to the birth is *unnata* in ghatis (0 to 30,
/// the nearer midnight being the one measured from, so a birth after noon is
/// measured to the midnight that follows); 30 less that is *nata*. The Moon,
/// Mars and Saturn get twice the nata, 60 at midnight and 0 at noon; the Sun,
/// Jupiter and Venus get 60 less that, twice the unnata, 60 at noon and 0 at
/// midnight; Mercury always gets 60. Equivalently, with `d` the degrees of
/// time from midnight, folded to 0 to 180 (15 degrees an hour), the day grahas
/// get `d / 3` and the night grahas `(180 - d) / 3`.
///
/// `apparent_hours_from_midnight` is **local apparent solar time**, the hour
/// angle of the true Sun plus 12 hours, not clock time or mean time; the
/// verse's "ishta" is the time by the Sun. Rahu and Ketu score 0.
#[must_use]
pub fn nathonnata_bala(planet: Graha, apparent_hours_from_midnight: f64) -> f64 {
    let degrees = (apparent_hours_from_midnight * 15.0).rem_euclid(360.0);
    let folded = if degrees > 180.0 {
        360.0 - degrees
    } else {
        degrees
    };
    match planet {
        Graha::Sun | Graha::Jupiter | Graha::Venus => folded / 3.0,
        Graha::Moon | Graha::Mars | Graha::Saturn => (180.0 - folded) / 3.0,
        Graha::Mercury => 60.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

/// Local apparent solar time in hours since midnight, from the hour angle of
/// the true Sun in degrees (0 at local noon, positive westward).
#[must_use]
pub fn apparent_hours_from_sun_hour_angle(hour_angle_deg: f64) -> f64 {
    (hour_angle_deg / 15.0 + 12.0).rem_euclid(24.0)
}

/// The graha set BPHS 27.11 names for the full paksha bala: the Moon,
/// Mercury, Venus and Jupiter ("indu-jna-shukra-ijya"), the rest taking the
/// complement. Grade (a) for the verse; whether Mercury stays in it when he
/// keeps malefic company, or the Moon after the full moon, is doctrine the
/// verse does not decide, so [`paksha_bala`] takes the set as an argument.
pub const BPHS_PAKSHA_BENEFICS: [Graha; 4] =
    [Graha::Moon, Graha::Mercury, Graha::Venus, Graha::Jupiter];

/// The paksha bala of the benefic group, in virupas (BPHS 27.10): the Moon's
/// elongation from the Sun, folded to 0 to 180 degrees, over 3. 0 at the new
/// moon, 60 at the full moon. The malefic group has 60 less this. Any two
/// longitudes in one frame will do.
#[must_use]
pub fn paksha_pool(moon_longitude_deg: f64, sun_longitude_deg: f64) -> f64 {
    let elongation = (moon_longitude_deg - sun_longitude_deg).rem_euclid(360.0);
    let folded = if elongation > 180.0 {
        360.0 - elongation
    } else {
        elongation
    };
    folded / 3.0
}

/// Paksha Bala of one graha in virupas (BPHS 27.10 and 27.11, grade (a)): the
/// [`paksha_pool`] if the graha is in `benefics`, 60 less it otherwise. The
/// Moon's is **undoubled**; BPHS 27.18 makes it her cheshta bala as well (see
/// the module documentation). Rahu and Ketu score 0.
#[must_use]
pub fn paksha_bala(
    planet: Graha,
    moon_longitude_deg: f64,
    sun_longitude_deg: f64,
    benefics: &[Graha],
) -> f64 {
    if !is_classical(planet) {
        return 0.0;
    }
    let pool = paksha_pool(moon_longitude_deg, sun_longitude_deg);
    if benefics.contains(&planet) {
        pool
    } else {
        60.0 - pool
    }
}

// ── Tribhaga Bala (BPHS 27.12) ──────────────────────────────────────

/// The lord of the third of the day or night holding `jd` (BPHS 27.12,
/// grade (a)): by day Mercury, the Sun, Saturn in the first, second and last
/// third of sunrise to sunset; by night the Moon, Venus, Mars in the thirds of
/// sunset to the next sunrise. `None` if `jd` is outside sunrise to next
/// sunrise, or the three instants are not in order. Each third is half-open at
/// its start.
#[must_use]
pub fn tribhaga_lord(
    jd: f64,
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
) -> Option<Graha> {
    if !(jd_sunrise < jd_sunset && jd_sunset < jd_next_sunrise) {
        return None;
    }
    if !(jd_sunrise..jd_next_sunrise).contains(&jd) {
        return None;
    }
    let (start, end, lords) = if jd < jd_sunset {
        (
            jd_sunrise,
            jd_sunset,
            [Graha::Mercury, Graha::Sun, Graha::Saturn],
        )
    } else {
        (
            jd_sunset,
            jd_next_sunrise,
            [Graha::Moon, Graha::Venus, Graha::Mars],
        )
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let third = (((jd - start) / (end - start)) * 3.0) as usize;
    Some(lords[third.min(2)])
}

/// Tribhaga Bala of one graha: 60 to Jupiter always and to the lord of the
/// third holding the birth ([`tribhaga_lord`]), 0 to the rest (BPHS 27.12).
#[must_use]
pub fn tribhaga_bala(planet: Graha, third_lord: Graha) -> f64 {
    if planet == Graha::Jupiter || planet == third_lord {
        60.0
    } else {
        0.0
    }
}

// ── Abda, masa, vara and hora Bala (BPHS 27.13) ─────────────────────

/// The Julian Day Number of the Kali Yuga epoch of the Surya Siddhanta:
/// midnight, at the Hindu prime meridian, beginning Friday 18 February 3102 BCE
/// (Julian calendar).
pub const KALI_EPOCH_JDN: i64 = 588_466;

/// The ahargana (civil days elapsed since the end of creation, counted by the
/// Surya Siddhanta) at the Kali Yuga epoch.
///
/// Chapter I of the Surya Siddhanta puts 1,972,944,000 years of the kalpa
/// behind the beginning of the Kali Yuga (six manvantaras with their
/// twilights, twenty-seven mahayugas, and the first three yugas of the
/// twenty-eighth), of which creation took 17,064,000 (verse 24): 1,955,880,000
/// years from the end of creation. At 1,577,917,828 civil days to each
/// 4,320,000-year mahayuga that is `1_955_880_000 / 4_320_000 = 452.75`
/// mahayugas, and `452.75 * 1_577_917_828 = 714_402_296_627` days, a whole
/// number.
pub const AHARGANA_AT_KALI_EPOCH: i64 = 714_402_296_627;

/// The ahargana at the beginning (midnight) of the civil day with Julian Day
/// Number `jdn`: the days elapsed before it.
#[must_use]
pub fn ahargana_from_jdn(jdn: i64) -> i64 {
    AHARGANA_AT_KALI_EPOCH + (jdn - KALI_EPOCH_JDN)
}

/// The Julian Day Number labelling the Hindu civil day that begins at the
/// sunrise `jd_sunrise` (UT): the local calendar date of that sunrise, at
/// `utc_offset_hours` from UT. A birth before sunrise belongs to the Hindu day
/// that began at the previous sunrise.
#[must_use]
pub fn hindu_day_jdn(jd_sunrise: f64, utc_offset_hours: f64) -> i64 {
    (jd_sunrise + 0.5 + utc_offset_hours / 24.0).floor() as i64
}

/// The graha ruling the seven-day cycle position `r` as the Surya Siddhanta
/// numbers it: 1 Sun, 2 Moon, 3 Mars, 4 Mercury, 5 Jupiter, 6 Venus, and 0
/// (the seventh) Saturn.
fn lord_of_remainder(r: i64) -> Graha {
    match r.rem_euclid(7) {
        1 => Graha::Sun,
        2 => Graha::Moon,
        3 => Graha::Mars,
        4 => Graha::Mercury,
        5 => Graha::Jupiter,
        6 => Graha::Venus,
        _ => Graha::Saturn,
    }
}

/// The lord of the day (vara, Surya Siddhanta I.51): the ahargana plus one,
/// divided by seven, the remainder naming the lord. It is the lord of the
/// weekday, so the ahargana of a Sunday begins a count at 1.
#[must_use]
pub fn vara_lord(ahargana: i64) -> Graha {
    lord_of_remainder(ahargana + 1)
}

/// The lord of the month (masa, Surya Siddhanta I.52): the ahargana divided by
/// 30, the whole quotient doubled, one added, divided by seven, the remainder
/// naming the lord. The "month" is a block of 30 days, not a lunar or solar
/// month. The lord of a month is the lord of its first day, and the lords of
/// successive months run two places on in the weekday order.
#[must_use]
pub fn masa_lord(ahargana: i64) -> Graha {
    lord_of_remainder(2 * ahargana.div_euclid(30) + 1)
}

/// The lord of the year (abda, Surya Siddhanta I.52): the ahargana divided by
/// 360, the whole quotient trebled, one added, divided by seven, the remainder
/// naming the lord. The "year" is a block of 360 days. The lords of successive
/// years run three places on in the weekday order.
#[must_use]
pub fn abda_lord(ahargana: i64) -> Graha {
    lord_of_remainder(3 * ahargana.div_euclid(360) + 1)
}

/// How the day is divided into horas for [`hora_lord`]. BPHS 27.13 names the
/// hora lord and is silent on the division, so this is grade (d).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HoraKind {
    /// Twenty-four equal parts of the day from one sunrise to the next, an
    /// hour each (two and a half ghatis). The Surya Siddhanta (XII.78 and 79)
    /// counts the horas as the twenty-fourths of the day, the first ruled by
    /// the lord of the day, in the order Saturn, Jupiter, Mars, Sun, Venus,
    /// Mercury, Moon. The default.
    #[default]
    Equal,
    /// Twelve parts of the day (sunrise to sunset) and twelve of the night
    /// (sunset to the next sunrise), as [`crate::kala_khanda::hora`].
    Unequal,
}

fn weekday_of_lord(lord: Graha) -> Weekday {
    match lord {
        Graha::Moon => Weekday::Monday,
        Graha::Mars => Weekday::Tuesday,
        Graha::Mercury => Weekday::Wednesday,
        Graha::Jupiter => Weekday::Thursday,
        Graha::Venus => Weekday::Friday,
        Graha::Saturn => Weekday::Saturday,
        Graha::Sun | Graha::Rahu | Graha::Ketu => Weekday::Sunday,
    }
}

fn graha_of_hora_index(index: u8) -> Graha {
    match index {
        0 => Graha::Sun,
        1 => Graha::Moon,
        2 => Graha::Mars,
        3 => Graha::Mercury,
        4 => Graha::Jupiter,
        5 => Graha::Venus,
        _ => Graha::Saturn,
    }
}

/// The hora lord at `jd` (BPHS 27.13). The first hora of the Hindu day is
/// ruled by its lord `vara`, and the lords follow in the order Saturn,
/// Jupiter, Mars, Sun, Venus, Mercury, Moon without a break through the
/// night. `None` if `jd` is outside sunrise to the next sunrise.
#[must_use]
pub fn hora_lord(
    kind: HoraKind,
    jd: f64,
    jd_sunrise: f64,
    jd_sunset: f64,
    jd_next_sunrise: f64,
    vara: Graha,
) -> Option<Graha> {
    if !(jd_sunrise < jd_sunset && jd_sunset < jd_next_sunrise) {
        return None;
    }
    // Equal horas are the unequal division of a day whose "sunset" is the
    // midpoint: twelve and twelve equal parts.
    let sunset = match kind {
        HoraKind::Equal => 0.5 * (jd_sunrise + jd_next_sunrise),
        HoraKind::Unequal => jd_sunset,
    };
    hora(jd_sunrise, sunset, jd_next_sunrise, weekday_of_lord(vara))
        .into_iter()
        .find(|&(start, end, _)| start <= jd && jd < end)
        .map(|(_, _, index)| graha_of_hora_index(index))
}

/// One of the four weights of BPHS 27.13: 15 to the lord of the year, 30 of the
/// month, 45 of the day, 60 of the hora. A graha that is several of these
/// lords gets each weight.
fn lord_weight(planet: Graha, lord: Graha, weight: f64) -> f64 {
    if planet == lord { weight } else { 0.0 }
}

// ── Ayana Bala (BPHS 27.15 to 27.17) ────────────────────────────────

/// The three khandas of BPHS 27.15 ("pancha-abdhayah surah suryah": 45, 33,
/// 12), the piecewise-linear steps of the declination over the three signs from
/// the nearer equinoctial point to the solstice. Their sum, 90, stands for the
/// greatest declination.
const AYANA_KHANDAS: [f64; 3] = [45.0, 33.0, 12.0];

/// The greatest declination the khanda table stands for, in degrees: the 24
/// degrees of the Surya Siddhanta (II.28), whose sine it gives as 1397. Used
/// only to put a measured declination on the khanda scale.
pub const AYANA_GREATEST_DECLINATION_DEG: f64 = 24.0;

/// Where Ayana Bala gets the declination from. BPHS 27.15 and 27.16 derive it
/// from the longitude by the khanda table; the table is a coarse stand-in for
/// the graha's measured declination, which also carries its latitude. Grade
/// (b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AyanaReading {
    /// The verse as written: the declination from the sayana longitude by the
    /// khanda table ([`ayana_bala_khanda`]). It is the declination of the point
    /// of the ecliptic, with no latitude. The default.
    #[default]
    KhandaOfLongitude,
    /// The graha's measured declination, put on the khanda scale and run
    /// through the verse's last step ([`ayana_bala_declination`]). For the Moon
    /// and the inner planets this moves the result by up to some virupas.
    Declination,
}

/// The ayana bala from a declination magnitude on the khanda scale (0 to 90) and
/// which side of the equator the graha stands on. `north` is the sign of the
/// declination.
///
/// Three signs (90) are joined with the scale value when the graha is on its
/// own favoured side and taken from it otherwise, then divided by 3 (BPHS
/// 27.16 and 27.17): the Sun, Mars, Jupiter and Venus favour the north, the
/// Moon and Saturn the south, and Mercury is always joined. Rahu and Ketu
/// score 0.
fn ayana_from_scale(planet: Graha, north: bool, scale: f64) -> f64 {
    let sign = match planet {
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => {
            if north {
                1.0
            } else {
                -1.0
            }
        }
        Graha::Moon | Graha::Saturn => {
            if north {
                -1.0
            } else {
                1.0
            }
        }
        Graha::Mercury => 1.0,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    ((90.0 + sign * scale) / 3.0).clamp(0.0, 60.0)
}

/// Ayana Bala from the sayana (tropical) longitude by the khanda table of BPHS
/// 27.15 to 27.17, in virupas, 0 to 60. Grade (a).
///
/// Take the *bhuja*, the arc from the nearer equinoctial point (0 or 180
/// degrees), 0 to 90. The completed signs of it contribute the khandas 45, 33,
/// 12 in turn, and the degrees beyond them contribute that many thirtieths of
/// the next khanda. The total, 0 to 90, is the declination on the khanda scale
/// (it follows the true declination of the ecliptic point to within 0.7
/// degrees). The longitude is north of the equator in the six signs from Aries
/// and south in the six from Libra.
///
/// `sayana_longitude_deg` is the sidereal longitude plus the ayanamsha.
#[must_use]
pub fn ayana_bala_khanda(planet: Graha, sayana_longitude_deg: f64) -> f64 {
    let lon = sayana_longitude_deg.rem_euclid(360.0);
    let north = lon < 180.0;
    let bhuja = if lon < 90.0 {
        lon
    } else if lon < 180.0 {
        180.0 - lon
    } else if lon < 270.0 {
        lon - 180.0
    } else {
        360.0 - lon
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let completed = ((bhuja / 30.0) as usize).min(3);
    let scale = if completed >= 3 {
        AYANA_KHANDAS.iter().sum::<f64>()
    } else {
        let before: f64 = AYANA_KHANDAS[..completed].iter().sum();
        before + (bhuja - 30.0 * completed as f64) * AYANA_KHANDAS[completed] / 30.0
    };
    ayana_from_scale(planet, north, scale)
}

/// Ayana Bala from a measured declination, in virupas, 0 to 60: the verse's
/// last step ([`ayana_bala_khanda`]) with the declination in degrees in place
/// of the khanda total, `(24 + k) / 48 * 60` for a graha on its favoured side
/// at declination `k` from the equator, `(24 - k) / 48 * 60` on the other,
/// clamped to 0 and 60 (the Moon can stand beyond 24 degrees). Mercury takes
/// `(24 + |k|) / 48 * 60`. Grade (b).
#[must_use]
pub fn ayana_bala_declination(planet: Graha, declination_deg: f64) -> f64 {
    let scale = declination_deg.abs() / AYANA_GREATEST_DECLINATION_DEG * 90.0;
    ayana_from_scale(planet, declination_deg >= 0.0, scale)
}

// ── Yuddha Bala (BPHS 27.20) ────────────────────────────────────────

/// A graha entering the planetary-war reckoning of [`yuddha_adjustments`].
#[derive(Debug, Clone, Copy)]
pub struct WarCandidate {
    /// Which graha.
    pub graha: Graha,
    /// Ecliptic longitude in degrees (any one frame).
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees, north positive.
    pub latitude_deg: f64,
    /// Its shadbala in virupas before any war adjustment.
    pub shadbala_before_war: f64,
}

/// Yuddha Bala (BPHS 27.20, grade (a)): where two of the five star grahas
/// (Mars, Mercury, Jupiter, Venus, Saturn) are at war, the difference of their
/// two shadbalas is added to the victor's and taken from the vanquished's.
/// Returns one signed adjustment per candidate, in order.
///
/// A war is [`crate::cheshta::graha_yuddha`]'s: within one degree of longitude,
/// the winner chosen by `rule`; the verse does not say who wins, so the winner
/// rule is the caller's. The difference is of the shadbalas *before* any war
/// adjustment, so the victor gains exactly what the vanquished loses and the
/// adjustments sum to zero. Where three grahas are within a degree of one
/// another each pair is a war, and a graha's adjustment is the sum over its
/// pairs. The Sun, the Moon and the nodes are never at war.
#[must_use]
pub fn yuddha_adjustments(candidates: &[WarCandidate], rule: YuddhaRule) -> Vec<f64> {
    let mut adjustments = vec![0.0; candidates.len()];
    for (i, a) in candidates.iter().enumerate() {
        for (j, b) in candidates.iter().enumerate().skip(i + 1) {
            let winner = graha_yuddha(
                a.graha,
                a.longitude_deg,
                a.latitude_deg,
                b.graha,
                b.longitude_deg,
                b.latitude_deg,
                rule,
            );
            let Some(winner) = winner else { continue };
            let difference = (a.shadbala_before_war - b.shadbala_before_war).abs();
            if winner == a.graha {
                adjustments[i] += difference;
                adjustments[j] -= difference;
            } else {
                adjustments[j] += difference;
                adjustments[i] -= difference;
            }
        }
    }
    adjustments
}

// ── Cheshta Bala from the cheshta kendra (BPHS 27.18, 27.24, 27.25) ─

/// The point halfway along the shorter arc from `a` to `b`, degrees. The verse
/// averages a graha's mean and true longitudes ("half the sum"); for two
/// longitudes that straddle 0 degrees a plain half-sum is 180 degrees away, so
/// the average is taken along the short arc.
fn arc_midpoint(a: f64, b: f64) -> f64 {
    let d = (b - a).rem_euclid(360.0);
    let signed = if d > 180.0 { d - 360.0 } else { d };
    (a + signed / 2.0).rem_euclid(360.0)
}

/// Fold an angle to 0 to 180 (more than six signs is taken from the circle).
fn fold180(angle: f64) -> f64 {
    let a = angle.rem_euclid(360.0);
    if a > 180.0 { 360.0 - a } else { a }
}

/// The cheshta kendra of Mars, Mercury, Jupiter, Venus or Saturn, degrees, 0
/// to 180 (BPHS 27.24, grade (a)): the graha's *chala-ucca* (sighrocca) less
/// the mean of its mean and true longitudes, folded.
///
/// - Mars, Jupiter, Saturn: the sighrocca is the mean Sun; the mean and true
///   longitudes are the graha's own.
/// - Mercury, Venus: the "mean" is the mean Sun and the sighrocca is the
///   graha's heliocentric mean longitude.
///
/// `mean_sun_deg` and `mean_planet_deg` come from
/// [`crate::mean_motion::mean_longitude`]. All three longitudes must be in one
/// frame; the mean ones are tropical, so a sidereal `true_planet_deg` needs the
/// ayanamsha added first. `None` for the Sun, Moon and nodes, whose cheshta
/// bala is BPHS 27.18, not the kendra.
#[must_use]
pub fn cheshta_kendra(
    planet: Graha,
    mean_sun_deg: f64,
    mean_planet_deg: f64,
    true_planet_deg: f64,
) -> Option<f64> {
    let kendra = match planet {
        Graha::Mars | Graha::Jupiter | Graha::Saturn => {
            mean_sun_deg - arc_midpoint(mean_planet_deg, true_planet_deg)
        }
        Graha::Mercury | Graha::Venus => {
            mean_planet_deg - arc_midpoint(mean_sun_deg, true_planet_deg)
        }
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => return None,
    };
    Some(fold180(kendra))
}

/// Cheshta Bala from the cheshta kendra, in virupas, 0 to 60: the folded kendra
/// over 3 (BPHS 27.25). 60 at opposition-like phases (the graha retrograde
/// and near the Earth), 0 at conjunction with the Sun.
#[must_use]
pub fn cheshta_bala_from_kendra(kendra_deg: f64) -> f64 {
    kendra_deg / 3.0
}

/// Cheshta Bala of Mars, Mercury, Jupiter, Venus or Saturn at `jd_tt`, from
/// the graha's true tropical longitude (BPHS 27.24 and 27.25). The mean
/// longitudes are read from [`crate::mean_motion::mean_longitude`]. `None` for
/// the Sun, Moon and nodes.
#[must_use]
pub fn cheshta_bala_at(planet: Graha, jd_tt: f64, true_tropical_longitude_deg: f64) -> Option<f64> {
    let mean_sun = mean_longitude(Graha::Sun, jd_tt).0;
    let mean_planet = mean_longitude(planet, jd_tt).0;
    cheshta_kendra(planet, mean_sun, mean_planet, true_tropical_longitude_deg)
        .map(cheshta_bala_from_kendra)
}

// ── The complete form ───────────────────────────────────────────────

/// The birth moment as Kala Bala needs it.
#[derive(Debug, Clone, Copy)]
pub struct KalaMoment {
    /// The birth, Julian Day (UT).
    pub jd: f64,
    /// The sunrise that begins the Hindu day holding the birth: the last
    /// sunrise at or before `jd`. Julian Day (UT).
    pub jd_sunrise: f64,
    /// The sunset of that day.
    pub jd_sunset: f64,
    /// The sunrise that ends it.
    pub jd_next_sunrise: f64,
    /// The Julian Day Number labelling that Hindu day ([`hindu_day_jdn`]).
    pub hindu_day_jdn: i64,
    /// Local apparent solar time of the birth, hours since midnight in 0 to 24
    /// ([`apparent_hours_from_sun_hour_angle`]).
    pub apparent_hours_from_midnight: f64,
}

/// What the complete form needs beyond the grahas.
#[derive(Debug, Clone, Copy)]
pub struct ShadbalaContext {
    /// The instant as Julian Day (TT), for the mean longitudes.
    pub jd_tt: f64,
    /// The ayanamsha in degrees that turned tropical longitudes into the
    /// sidereal ones in [`CompletePlanetData`].
    pub ayanamsha_deg: f64,
    /// The birth moment.
    pub moment: KalaMoment,
}

/// One graha's chart data for the complete form.
#[derive(Debug, Clone, Copy)]
pub struct CompletePlanetData {
    /// The sidereal placement; `longitude` is the apparent geocentric one.
    pub position: GrahaPosition,
    /// Ecliptic latitude in degrees, north positive: needed only to name the
    /// winner of a planetary war under [`YuddhaRule::NorthernLatitude`].
    pub latitude_deg: f64,
    /// Equatorial declination in degrees, north positive: needed only under
    /// [`AyanaReading::Declination`].
    pub declination_deg: Option<f64>,
    /// Number of benefic aspects received (Drik Bala).
    pub benefic_aspect_count: u32,
    /// Number of malefic aspects received (Drik Bala).
    pub malefic_aspect_count: u32,
}

/// The doctrine and convention choices of the complete form, each documented
/// where its type is. [`ShadbalaOptions::default`] is the verse.
#[derive(Debug, Clone, Copy)]
pub struct ShadbalaOptions<'a> {
    /// The grahas that take the paksha bala of the benefic group
    /// ([`BPHS_PAKSHA_BENEFICS`] by default).
    pub paksha_benefics: &'a [Graha],
    /// How the horas divide the day.
    pub hora: HoraKind,
    /// Where the moolatrikona grade is awarded.
    pub moolatrikona: MoolatrikonaScope,
    /// The Saptavargaja grades.
    pub ladder: SaptavargajaLadder,
    /// Where Ayana Bala's declination comes from.
    pub ayana: AyanaReading,
    /// The drekkana order.
    pub drekkana: DrekkanaOrder,
    /// How the winner of a planetary war is decided.
    pub yuddha_rule: YuddhaRule,
}

impl Default for ShadbalaOptions<'static> {
    fn default() -> Self {
        Self {
            paksha_benefics: &BPHS_PAKSHA_BENEFICS,
            hora: HoraKind::default(),
            moolatrikona: MoolatrikonaScope::default(),
            ladder: SaptavargajaLadder::default(),
            ayana: AyanaReading::default(),
            drekkana: DrekkanaOrder::default(),
            yuddha_rule: YuddhaRule::NorthernLatitude,
        }
    }
}

/// Why the complete form could not be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadbalaError {
    /// One of the seven classical grahas is absent from the input.
    MissingGraha(Graha),
    /// [`AyanaReading::Declination`] was chosen and this graha has no
    /// declination.
    MissingDeclination(Graha),
    /// The sunrise, sunset and next sunrise are not in order, or the birth is
    /// not between the two sunrises, or a time is not finite.
    BadMoment,
}

impl core::fmt::Display for ShadbalaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingGraha(g) => write!(f, "{g:?} is missing from the shadbala input"),
            Self::MissingDeclination(g) => write!(
                f,
                "{g:?} has no declination, which AyanaReading::Declination needs"
            ),
            Self::BadMoment => write!(
                f,
                "the birth must lie between two sunrises with the sunset between them"
            ),
        }
    }
}

impl std::error::Error for ShadbalaError {}

/// Sthana Bala by part, virupas.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SthanaParts {
    /// Uchcha Bala, 0 to 60 (27.1).
    pub uccha: f64,
    /// Saptavargaja Bala (27.2 to 27.4).
    pub saptavargaja: f64,
    /// Ojhayugma Bala, 0 to 30.
    pub ojhayugma: f64,
    /// Kendradi Bala, 15 to 60.
    pub kendradi: f64,
    /// Drekkana Bala, 0 or 15.
    pub drekkana: f64,
    /// Their sum.
    pub total: f64,
}

/// Kala Bala by part, virupas. Paksha is the graha's own, undoubled; yuddha is
/// signed.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct KalaParts {
    /// Nathonnata Bala, 0 to 60 (27.8 and 27.9).
    pub nathonnata: f64,
    /// Paksha Bala, 0 to 60 (27.10 and 27.11).
    pub paksha: f64,
    /// Tribhaga Bala, 0 or 60 (27.12).
    pub tribhaga: f64,
    /// Abda Bala, 0 or 15: the lord of the year (27.13).
    pub abda: f64,
    /// Masa Bala, 0 or 30: the lord of the month.
    pub masa: f64,
    /// Vara Bala, 0 or 45: the lord of the day.
    pub vara: f64,
    /// Hora Bala, 0 or 60: the lord of the hora.
    pub hora: f64,
    /// Ayana Bala, 0 to 60 (27.15 to 27.17), undoubled.
    pub ayana: f64,
    /// Yuddha Bala, signed (27.20).
    pub yuddha: f64,
    /// Their sum.
    pub total: f64,
}

/// The complete Shadbala of one graha: every part the verse gives, each in its
/// own field, in virupas.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CompleteShadbala {
    /// The graha.
    pub planet: Graha,
    /// Positional strength by part.
    pub sthana: SthanaParts,
    /// Directional strength ([`dig_bala`]).
    pub dig_bala: f64,
    /// Temporal strength by part.
    pub kala: KalaParts,
    /// Motional strength: the Sun's ayana bala, the Moon's paksha bala, the
    /// kendra over 3 for the others (27.18, 27.24, 27.25).
    pub cheshta_bala: f64,
    /// Natural strength.
    pub naisargika_bala: f64,
    /// Aspectual strength ([`drik_bala`]).
    pub drik_bala: f64,
    /// Sum of the six: `sthana.total + dig_bala + kala.total + cheshta_bala +
    /// naisargika_bala + drik_bala`.
    pub total: f64,
    /// Benefit strength per Rasmi scale (BPHS 28.5 and 28.6).
    pub ishta_phala: f64,
    /// Affliction strength per Rasmi scale.
    pub kashta_phala: f64,
}

fn validate_moment(m: &KalaMoment) -> Result<(), ShadbalaError> {
    let finite = [
        m.jd,
        m.jd_sunrise,
        m.jd_sunset,
        m.jd_next_sunrise,
        m.apparent_hours_from_midnight,
    ]
    .iter()
    .all(|x| x.is_finite());
    let ordered = m.jd_sunrise < m.jd_sunset && m.jd_sunset < m.jd_next_sunrise;
    let inside = m.jd_sunrise <= m.jd && m.jd < m.jd_next_sunrise;
    if finite && ordered && inside {
        Ok(())
    } else {
        Err(ShadbalaError::BadMoment)
    }
}

/// Compute Shadbala by every part of BPHS chapter 27 (see the module
/// documentation for what each part is, its verse and its grade, and for the
/// two components still outside it, Dig and Drik).
///
/// `planets` must hold all seven classical grahas; Rahu and Ketu, if present,
/// are ignored. The result has one entry per classical graha supplied, in
/// input order.
///
/// The kala total pairs the parts as BPHS 27.18 does: the Sun's ayana bala and
/// the Moon's paksha bala stay in kala bala undoubled, and are each the graha's
/// cheshta bala as well.
///
/// # Errors
/// [`ShadbalaError::MissingGraha`] if a classical graha is absent,
/// [`ShadbalaError::MissingDeclination`] under [`AyanaReading::Declination`]
/// for a graha without one, [`ShadbalaError::BadMoment`] for a malformed
/// birth moment.
pub fn compute_shadbala_complete(
    planets: &[CompletePlanetData],
    context: &ShadbalaContext,
    options: &ShadbalaOptions<'_>,
) -> Result<Vec<CompleteShadbala>, ShadbalaError> {
    let moment = &context.moment;
    validate_moment(moment)?;

    let find = |g: Graha| {
        planets
            .iter()
            .find(|p| p.position.planet == g)
            .ok_or(ShadbalaError::MissingGraha(g))
    };
    let mut positions = Vec::with_capacity(7);
    for g in CLASSICAL {
        positions.push(find(g)?.position);
    }
    let sun_lon = find(Graha::Sun)?.position.longitude;
    let moon_lon = find(Graha::Moon)?.position.longitude;

    let ahargana = ahargana_from_jdn(moment.hindu_day_jdn);
    let vara = vara_lord(ahargana);
    let abda = abda_lord(ahargana);
    let masa = masa_lord(ahargana);
    let hora_ruler = hora_lord(
        options.hora,
        moment.jd,
        moment.jd_sunrise,
        moment.jd_sunset,
        moment.jd_next_sunrise,
        vara,
    )
    .ok_or(ShadbalaError::BadMoment)?;
    let third = tribhaga_lord(
        moment.jd,
        moment.jd_sunrise,
        moment.jd_sunset,
        moment.jd_next_sunrise,
    )
    .ok_or(ShadbalaError::BadMoment)?;
    let mut results = Vec::with_capacity(7);
    let mut candidates = Vec::with_capacity(7);
    for data in planets {
        let planet = data.position.planet;
        if !is_classical(planet)
            || results
                .iter()
                .any(|r: &CompleteShadbala| r.planet == planet)
        {
            continue;
        }
        let lon = data.position.longitude;

        let uccha = uccha_bala(planet, lon);
        let saptavargaja =
            saptavargaja_bala(planet, &positions, options.moolatrikona, options.ladder)
                .ok_or(ShadbalaError::MissingGraha(planet))?;
        let ojhayugma = ojhayugma_bala(planet, lon);
        let kendradi = kendradi_bala(data.position.bhava);
        let drekkana = drekkana_bala_ordered(planet, lon, options.drekkana);
        let sthana = SthanaParts {
            uccha,
            saptavargaja,
            ojhayugma,
            kendradi,
            drekkana,
            total: uccha + saptavargaja + ojhayugma + kendradi + drekkana,
        };

        let paksha = paksha_bala(planet, moon_lon, sun_lon, options.paksha_benefics);
        let ayana = match options.ayana {
            AyanaReading::KhandaOfLongitude => {
                ayana_bala_khanda(planet, lon + context.ayanamsha_deg)
            }
            AyanaReading::Declination => {
                let declination = data
                    .declination_deg
                    .ok_or(ShadbalaError::MissingDeclination(planet))?;
                ayana_bala_declination(planet, declination)
            }
        };
        let mut kala = KalaParts {
            nathonnata: nathonnata_bala(planet, moment.apparent_hours_from_midnight),
            paksha,
            tribhaga: tribhaga_bala(planet, third),
            abda: lord_weight(planet, abda, 15.0),
            masa: lord_weight(planet, masa, 30.0),
            vara: lord_weight(planet, vara, 45.0),
            hora: lord_weight(planet, hora_ruler, 60.0),
            ayana,
            yuddha: 0.0,
            total: 0.0,
        };
        kala.total = kala.nathonnata
            + kala.paksha
            + kala.tribhaga
            + kala.abda
            + kala.masa
            + kala.vara
            + kala.hora
            + kala.ayana;

        let cheshta = match planet {
            Graha::Sun => ayana,
            Graha::Moon => paksha,
            _ => cheshta_bala_at(planet, context.jd_tt, lon + context.ayanamsha_deg).unwrap_or(0.0),
        };
        let dig = dig_bala(planet, data.position.bhava);
        let naisargika = naisargika_bala(planet);
        let drik = drik_bala(data.benefic_aspect_count, data.malefic_aspect_count);
        let total = sthana.total + dig + kala.total + cheshta + naisargika + drik;

        candidates.push(WarCandidate {
            graha: planet,
            longitude_deg: lon,
            latitude_deg: data.latitude_deg,
            shadbala_before_war: total,
        });
        let (ishta, kashta) = ishta_kashta_phala(uccha, cheshta);
        results.push(CompleteShadbala {
            planet,
            sthana,
            dig_bala: dig,
            kala,
            cheshta_bala: cheshta,
            naisargika_bala: naisargika,
            drik_bala: drik,
            total,
            ishta_phala: ishta,
            kashta_phala: kashta,
        });
    }

    // Yuddha last: it moves a graha's total by the difference of the totals
    // the war found them at.
    let war = yuddha_adjustments(&candidates, options.yuddha_rule);
    for (result, adjustment) in results.iter_mut().zip(war) {
        result.kala.yuddha = adjustment;
        result.kala.total += adjustment;
        result.total += adjustment;
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(planet: Graha, sign: u8, bhava: u8) -> GrahaPosition {
        GrahaPosition {
            planet,
            sign,
            longitude: f64::from(sign) * 30.0 + 15.0,
            bhava,
        }
    }

    fn planet_data(
        planet: Graha,
        sign: u8,
        bhava: u8,
        speed: f64,
        avg_speed: f64,
        benefic: u32,
        malefic: u32,
    ) -> ShadbalaPlanetData {
        ShadbalaPlanetData {
            position: pos(planet, sign, bhava),
            speed,
            average_speed: avg_speed,
            benefic_aspect_count: benefic,
            malefic_aspect_count: malefic,
        }
    }

    // ── Naisargika Bala tests ───────────────────────────────────────

    #[test]
    fn sun_naisargika_bala_is_60() {
        let bala = naisargika_bala(Graha::Sun);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_naisargika_bala_is_8_57() {
        let bala = naisargika_bala(Graha::Saturn);
        assert!((bala - 8.57).abs() < 0.01);
    }

    // ── Dig Bala tests ──────────────────────────────────────────────

    #[test]
    fn sun_dig_bala_maximum_in_10th() {
        let bala = dig_bala(Graha::Sun, 10);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_dig_bala_minimum_in_4th() {
        let bala = dig_bala(Graha::Sun, 4);
        assert!((bala - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn moon_dig_bala_maximum_in_4th() {
        let bala = dig_bala(Graha::Moon, 4);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_dig_bala_maximum_in_7th() {
        let bala = dig_bala(Graha::Saturn, 7);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dig_bala_intermediate_values() {
        let bala = dig_bala(Graha::Mercury, 4);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    // ── Uccha Bala tests ────────────────────────────────────────────

    #[test]
    fn uccha_bala_exact_exaltation_is_60() {
        // Sun exalted at 10° Aries → 60 virupas
        let bala = uccha_bala(Graha::Sun, 10.0);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_exact_debilitation_is_0() {
        // Sun debilitated at 190° (10+180) → 0 virupas
        let bala = uccha_bala(Graha::Sun, 190.0);
        assert!((bala - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_midpoint_is_30() {
        // Sun at 100° → 90° from exaltation (10°) → (180-90)/3 = 30 virupas
        let bala = uccha_bala(Graha::Sun, 100.0);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_bala_gradient_is_continuous() {
        // Jupiter at exact exaltation (95°) should be stronger than at 100°
        let at_exalt = uccha_bala(Graha::Jupiter, 95.0);
        let near_exalt = uccha_bala(Graha::Jupiter, 100.0);
        assert!(at_exalt > near_exalt);
    }

    // ── Ojhayugma Bala tests ────────────────────────────────────────

    #[test]
    fn ojhayugma_rewards_odd_signs_for_the_sun() {
        // 5° Aries: rasi index 0 (classically odd), navamsa of 5° Aries is
        // index 1 (classically even). The Sun wants odd, so it scores the
        // rasi half only.
        assert!((ojhayugma_bala(Graha::Sun, 5.0) - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ojhayugma_is_inverted_for_moon_and_venus() {
        // Same longitude, opposite preference: the Moon takes the navamsa
        // half where the Sun took the rasi half. The two must therefore sum
        // to the full 30 virupas at every longitude.
        for step in 0..360 {
            let lon = f64::from(step);
            let sun = ojhayugma_bala(Graha::Sun, lon);
            let moon = ojhayugma_bala(Graha::Moon, lon);
            assert!(
                (sun + moon - 30.0).abs() < f64::EPSILON,
                "at {lon}°: sun {sun} + moon {moon} != 30"
            );
        }
    }

    #[test]
    fn ojhayugma_is_zero_for_the_nodes() {
        assert!(ojhayugma_bala(Graha::Rahu, 5.0).abs() < f64::EPSILON);
        assert!(ojhayugma_bala(Graha::Ketu, 5.0).abs() < f64::EPSILON);
    }

    // ── Kendradi Bala tests ─────────────────────────────────────────

    #[test]
    fn kendradi_grades_the_three_house_classes() {
        for kendra in [1, 4, 7, 10] {
            assert!((kendradi_bala(kendra) - 60.0).abs() < f64::EPSILON);
        }
        for panapara in [2, 5, 8, 11] {
            assert!((kendradi_bala(panapara) - 30.0).abs() < f64::EPSILON);
        }
        for apoklima in [3, 6, 9, 12] {
            assert!((kendradi_bala(apoklima) - 15.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn kendradi_covers_every_house_and_rejects_others() {
        for bhava in 1..=12u8 {
            assert!(kendradi_bala(bhava) > 0.0, "house {bhava} scored nothing");
        }
        assert!(kendradi_bala(0).abs() < f64::EPSILON);
        assert!(kendradi_bala(13).abs() < f64::EPSILON);
    }

    // ── Drekkana Bala tests ─────────────────────────────────────────

    #[test]
    fn drekkana_rewards_each_sex_in_its_own_third() {
        // Third of the occupied sign, not the D-3 varga sign.
        let first = 5.0; // 5° Aries
        let second = 15.0;
        let third = 25.0;

        for male in [Graha::Sun, Graha::Mars, Graha::Jupiter] {
            assert!((drekkana_bala(male, first) - 15.0).abs() < f64::EPSILON);
            assert!(drekkana_bala(male, second).abs() < f64::EPSILON);
            assert!(drekkana_bala(male, third).abs() < f64::EPSILON);
        }
        for neuter in [Graha::Mercury, Graha::Saturn] {
            assert!(drekkana_bala(neuter, first).abs() < f64::EPSILON);
            assert!((drekkana_bala(neuter, second) - 15.0).abs() < f64::EPSILON);
        }
        for female in [Graha::Moon, Graha::Venus] {
            assert!(drekkana_bala(female, first).abs() < f64::EPSILON);
            assert!((drekkana_bala(female, third) - 15.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn drekkana_index_never_escapes_the_sign() {
        // Every longitude, including the sign boundaries and a negative one,
        // must land in exactly one of the three thirds — so across the three
        // sexes precisely 15 virupas is awarded at every degree.
        for step in 0..3600 {
            let lon = f64::from(step) / 10.0 - 180.0;
            let awarded = drekkana_bala(Graha::Sun, lon)
                + drekkana_bala(Graha::Mercury, lon)
                + drekkana_bala(Graha::Moon, lon);
            assert!(
                (awarded - 15.0).abs() < f64::EPSILON,
                "at {lon}°: {awarded} virupas awarded across the three thirds"
            );
        }
    }

    // ── Sthana Bala composite tests ─────────────────────────────────

    #[test]
    fn sthana_bala_is_the_sum_of_its_four_sub_components() {
        for planet in [Graha::Sun, Graha::Moon, Graha::Saturn, Graha::Rahu] {
            for bhava in 1..=12u8 {
                let lon = f64::from(bhava) * 27.3;
                let parts = uccha_bala(planet, lon)
                    + ojhayugma_bala(planet, lon)
                    + kendradi_bala(bhava)
                    + drekkana_bala(planet, lon);
                let composite = sthana_bala_full(planet, lon, bhava);
                assert!(
                    (composite - parts).abs() < f64::EPSILON,
                    "{planet:?} at {lon}° house {bhava}: {composite} != {parts}"
                );
            }
        }
    }

    #[test]
    fn sthana_bala_depends_on_the_house_it_used_to_ignore() {
        // The defect this replaced: sthana_bala returned Uccha Bala alone, so
        // two placements differing only in house scored identically.
        let angular = sthana_bala_full(Graha::Sun, 10.0, 1);
        let cadent = sthana_bala_full(Graha::Sun, 10.0, 3);
        assert!(
            (angular - cadent - 45.0).abs() < f64::EPSILON,
            "kendra {angular} vs apoklima {cadent}"
        );
    }

    #[test]
    fn sthana_bala_stays_within_its_documented_bound() {
        for step in 0..720 {
            let lon = f64::from(step) / 2.0;
            for bhava in 1..=12u8 {
                for planet in [Graha::Sun, Graha::Moon, Graha::Venus, Graha::Saturn] {
                    let bala = sthana_bala_full(planet, lon, bhava);
                    assert!(
                        (0.0..=165.0).contains(&bala),
                        "{planet:?} at {lon}° house {bhava}: {bala} outside 0..=165"
                    );
                }
            }
        }
    }

    // ── Kala Bala tests ─────────────────────────────────────────────

    #[test]
    fn sun_daytime_gets_nathonnatha() {
        let bala = kala_bala(Graha::Sun, true, false);
        // Daytime: 30. Sun is not benefic, Krishna paksha (non-benefic strong): +30
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_nighttime_no_nathonnatha() {
        let bala = kala_bala(Graha::Sun, false, false);
        // Night: 0. Krishna paksha, Sun is not benefic: +30
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mercury_always_gets_nathonnatha() {
        let day = kala_bala(Graha::Mercury, true, true);
        let night = kala_bala(Graha::Mercury, false, true);
        // Mercury: always 30 nathonnatha + benefic in shukla = +30 = 60
        assert!((day - 60.0).abs() < f64::EPSILON);
        assert!((night - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn moon_night_waxing() {
        let bala = kala_bala(Graha::Moon, false, true);
        // Night strong: 30. Benefic in waxing: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jupiter_day_waxing() {
        let bala = kala_bala(Graha::Jupiter, true, true);
        // Day strong: 30. Benefic in waxing: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn saturn_night_waning() {
        let bala = kala_bala(Graha::Saturn, false, false);
        // Night strong: 30. Non-benefic in waning: 30. Total = 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    // ── Cheshta Bala tests ──────────────────────────────────────────

    #[test]
    fn retrograde_planet_gets_60() {
        let bala = cheshta_bala(Graha::Mars, -0.5, 0.5);
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stationary_planet_gets_30() {
        let bala = cheshta_bala(Graha::Jupiter, 0.005, 0.08);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn slow_planet_gets_15() {
        let bala = cheshta_bala(Graha::Saturn, 0.01, 0.05);
        assert!((bala - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fast_planet_gets_45() {
        let bala = cheshta_bala(Graha::Venus, 2.0, 1.0);
        assert!((bala - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normal_speed_planet_gets_30() {
        let bala = cheshta_bala(Graha::Mercury, 1.0, 1.2);
        assert!((bala - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sun_moon_always_30_cheshta() {
        let sun = cheshta_bala(Graha::Sun, 1.0, 1.0);
        let moon = cheshta_bala(Graha::Moon, 13.0, 13.0);
        assert!((sun - 30.0).abs() < f64::EPSILON);
        assert!((moon - 30.0).abs() < f64::EPSILON);
    }

    // ── Drik Bala tests ─────────────────────────────────────────────

    #[test]
    fn benefic_aspects_add_strength() {
        let bala = drik_bala(3, 0);
        assert!((bala - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn malefic_aspects_subtract_strength() {
        let bala = drik_bala(0, 2);
        assert!((bala - (-30.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn mixed_aspects() {
        let bala = drik_bala(2, 1);
        // 2*15 - 1*15 = 15
        assert!((bala - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drik_bala_clamped_high() {
        let bala = drik_bala(10, 0);
        // 10*15 = 150, clamped to 60
        assert!((bala - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drik_bala_clamped_low() {
        let bala = drik_bala(0, 10);
        // -150, clamped to -30
        assert!((bala - (-30.0)).abs() < f64::EPSILON);
    }

    // ── Full Shadbala tests ─────────────────────────────────────────

    #[test]
    fn full_shadbala_includes_all_six_components() {
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        assert_eq!(results.len(), 1);
        let sb = &results[0];

        // Sthana for Jupiter at 105° (15° Cancer) in the 4th, worked by hand:
        //   uccha     — exaltation 95°, arc 10° -> (180-10)/3 = 56.667
        //   ojhayugma — rasi Cancer (index 3, classically even) and navamsa
        //               Scorpio (index 7, classically even); Jupiter wants
        //               odd, so neither half scores -> 0
        //   kendradi  — 4th house is a kendra -> 60
        //   drekkana  — 15° into the sign is the second third; Jupiter is
        //               male and wants the first -> 0
        let expected_uccha = (180.0 - 10.0) / 3.0;
        let expected_sthana = expected_uccha + 0.0 + 60.0 + 0.0;
        assert!(
            (sb.sthana_bala - expected_sthana).abs() < 0.01,
            "sthana {} != {expected_sthana}",
            sb.sthana_bala
        );
        assert!((sb.uccha_bala - expected_uccha).abs() < 0.01);
        assert!(sb.ojhayugma_bala.abs() < f64::EPSILON);
        assert!((sb.kendradi_bala - 60.0).abs() < f64::EPSILON);
        assert!(sb.drekkana_bala.abs() < f64::EPSILON);
        // Dig: house 4, strong house 1, dist 3 -> 60 - 30 = 30
        assert!((sb.dig_bala - 30.0).abs() < f64::EPSILON);
        // Naisargika: Jupiter = 34.29
        assert!((sb.naisargika_bala - 34.29).abs() < 0.01);
        // Kala: daytime + waxing benefic = 30 + 30 = 60
        assert!((sb.kala_bala - 60.0).abs() < f64::EPSILON);
        // Cheshta: retrograde = 60
        assert!((sb.cheshta_bala - 60.0).abs() < f64::EPSILON);
        // Drik: 2 benefic - 1 malefic = 15
        assert!((sb.drik_bala - 15.0).abs() < f64::EPSILON);

        // Total
        let expected = expected_sthana + 30.0 + 34.29 + 60.0 + 60.0 + 15.0;
        assert!((sb.total - expected).abs() < 0.01);
    }

    #[test]
    fn full_shadbala_total_equals_sum() {
        let data = [
            planet_data(Graha::Sun, 0, 10, 1.0, 1.0, 1, 0),
            planet_data(Graha::Saturn, 6, 7, 0.02, 0.05, 0, 2),
        ];
        let results = compute_shadbala_full(&data, true, false);
        for sb in &results {
            let sum = sb.sthana_bala
                + sb.dig_bala
                + sb.kala_bala
                + sb.cheshta_bala
                + sb.naisargika_bala
                + sb.drik_bala;
            // Not f64::EPSILON: `sthana_bala` is itself a sum, so `total`
            // and this re-addition associate the same terms differently and
            // differ by an ulp of ~250, not of 1. Any real omission is at
            // least a whole virupa.
            assert!(
                (sb.total - sum).abs() < 1e-9,
                "Total {} != sum {} for {:?}",
                sb.total,
                sum,
                sb.planet
            );
        }
    }

    // ── Ishta/Kashta Phala tests ─────────────────────────────────────────

    #[test]
    fn uccha_rasmi_at_max_virupas() {
        let r = uccha_rasmi(60.0);
        assert!((r - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cheshta_rasmi_at_max_virupas() {
        let r = cheshta_rasmi(60.0);
        assert!((r - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uccha_rasmi_zero() {
        assert!((uccha_rasmi(0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ishta_phala_max_at_both_full() {
        // uccha=60 (ur=7), cheshta=60 (cr=7): ishta = 5*(7+7-2) = 60
        let (ishta, kashta) = ishta_kashta_phala(60.0, 60.0);
        assert!((ishta - 60.0).abs() < f64::EPSILON);
        assert!((kashta - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ishta_phala_clips_to_zero() {
        // uccha=0, cheshta=0: 5*(0+0-2) = -10 → clamp to 0
        let (ishta, kashta) = ishta_kashta_phala(0.0, 0.0);
        assert!((ishta - 0.0).abs() < f64::EPSILON);
        assert!((kashta - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn kashta_is_complement_of_ishta() {
        let (ishta, kashta) = ishta_kashta_phala(30.0, 30.0);
        assert!((ishta + kashta - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_pipeline_and_the_standalone_sthana_bala_agree() {
        // `compute_shadbala_full` computes the four sub-components inline so
        // it can report each one, which means it does not go through
        // `sthana_bala`. The two must not be allowed to drift apart.
        for planet in [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Venus] {
            for bhava in 1..=12u8 {
                let sign = (bhava - 1) % 12;
                let data = [planet_data(planet, sign, bhava, 0.5, 0.5, 0, 0)];
                let sb = &compute_shadbala_full(&data, true, true)[0];
                let standalone = sthana_bala_full(planet, data[0].position.longitude, bhava);
                assert!(
                    (sb.sthana_bala - standalone).abs() < 1e-9,
                    "{planet:?} house {bhava}: pipeline {} vs standalone {standalone}",
                    sb.sthana_bala
                );
            }
        }
    }

    #[test]
    #[allow(deprecated)]
    fn the_v7_spellings_still_behave_as_they_did() {
        // These two exist only so a `vedaksha-vedic = "7"` dependent keeps
        // compiling across this minor release. `sthana_bala` must still be
        // Uchcha Bala with the sign discarded, and `compute_shadbala` must
        // still total the three components it can reach.
        for lon in [0.0, 10.0, 95.0, 190.0, 285.0, 359.9] {
            for sign in 0..12u8 {
                assert!(
                    (sthana_bala(Graha::Sun, sign, lon) - uccha_bala(Graha::Sun, lon)).abs()
                        < f64::EPSILON,
                    "sign {sign} changed the answer at {lon}°"
                );
            }
        }

        let positions = [pos(Graha::Sun, 0, 10)];
        let sb = &compute_shadbala(&positions, 0)[0];
        assert!(sb.kala_bala.abs() < f64::EPSILON);
        assert!(sb.cheshta_bala.abs() < f64::EPSILON);
        assert!(sb.drik_bala.abs() < f64::EPSILON);
        let three = sb.sthana_bala + sb.dig_bala + sb.naisargika_bala;
        assert!((sb.total - three).abs() < 1e-9, "{} != {three}", sb.total);
    }

    #[test]
    fn uccha_bala_is_a_part_of_sthana_bala_not_a_copy_of_it() {
        // Through v7.1.1 these two fields carried the same number, because
        // `sthana_bala` was Uccha Bala under another name. They must now
        // differ by exactly the other three sub-components.
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let results = compute_shadbala_full(&data, true, true);
        let sb = &results[0];
        let rest = sb.ojhayugma_bala + sb.kendradi_bala + sb.drekkana_bala;
        assert!(
            rest > 0.0,
            "sub-components all zero — nothing is being tested"
        );
        assert!((sb.sthana_bala - sb.uccha_bala - rest).abs() < 1e-9);
        assert!(sb.uccha_bala < sb.sthana_bala);
    }

    #[test]
    fn full_shadbala_uccha_not_in_total() {
        // total must still equal sum of the original six components
        let data = [planet_data(Graha::Sun, 0, 10, 1.0, 1.0, 1, 0)];
        let results = compute_shadbala_full(&data, true, false);
        let sb = &results[0];
        let sum = sb.sthana_bala
            + sb.dig_bala
            + sb.kala_bala
            + sb.cheshta_bala
            + sb.naisargika_bala
            + sb.drik_bala;
        assert!((sb.total - sum).abs() < 1e-9);
    }

    #[test]
    fn full_shadbala_saturn_nighttime() {
        let data = [planet_data(
            Graha::Saturn,
            9, // Capricorn (own sign)
            7, // 7th house (dig bala max)
            0.02,
            0.05,
            0,
            0,
        )];
        let results = compute_shadbala_full(&data, false, false);
        let sb = &results[0];

        // Sthana for Saturn at 285° (15° Capricorn) in the 7th, by hand:
        //   uccha     — exaltation 200°, arc 85° -> (180-85)/3 = 31.667
        //   ojhayugma — rasi Capricorn (index 9) and navamsa Taurus (index 1)
        //               are both classically even; Saturn wants odd -> 0
        //   kendradi  — 7th house is a kendra -> 60
        //   drekkana  — second third of the sign, which is what the neuter
        //               grahas want -> 15
        let expected_uccha = (180.0 - 85.0) / 3.0;
        let expected_sthana = expected_uccha + 0.0 + 60.0 + 15.0;
        assert!(
            (sb.sthana_bala - expected_sthana).abs() < 0.01,
            "sthana {} != {expected_sthana}",
            sb.sthana_bala
        );
        assert!((sb.drekkana_bala - 15.0).abs() < f64::EPSILON);
        // Dig: house 7 = max = 60
        assert!((sb.dig_bala - 60.0).abs() < f64::EPSILON);
        // Kala: night + waning non-benefic = 30 + 30 = 60
        assert!((sb.kala_bala - 60.0).abs() < f64::EPSILON);
        // Cheshta: slow (0.02 < 0.05*0.5=0.025) = 15
        assert!((sb.cheshta_bala - 15.0).abs() < f64::EPSILON);
        // Naisargika: 8.57
        assert!((sb.naisargika_bala - 8.57).abs() < 0.01);
        // Drik: 0 - 0 = 0
        assert!((sb.drik_bala - 0.0).abs() < f64::EPSILON);

        // BPHS Ch.28 v.5 builds the Rasmis from Uchcha Bala. Feeding it the
        // Sthana composite instead — which is what the code did while the two
        // were the same number — drives uchcha_rasmi to 12.4 and clamps ishta
        // to its 60 ceiling, so the error hides unless it is asserted here.
        //   uchcha_rasmi  = 31.667 * 7/60 = 3.6944
        //   cheshta_rasmi = 15     * 7/60 = 1.75
        //   ishta = 5 * (3.6944 + 1.75 - 2) = 17.222
        assert!(
            (sb.ishta_phala - 17.2222).abs() < 0.001,
            "ishta {} — is it being fed Sthana Bala rather than Uccha Bala?",
            sb.ishta_phala
        );
        assert!((sb.kashta_phala - 42.7778).abs() < 0.001);
    }
    // ═══════════════════════════════════════════════════════════════
    // The complete form: BPHS chapter 27 by part.
    // ═══════════════════════════════════════════════════════════════

    fn gp(planet: Graha, longitude: f64, bhava: u8) -> GrahaPosition {
        GrahaPosition {
            planet,
            sign: sign_of(longitude),
            longitude,
            bhava,
        }
    }

    /// Sun 6 Leo, Moon 2 Taurus, Mars 5.5 Aries, Mercury 2 Gemini, Jupiter 5
    /// Sagittarius, Venus 10 Cancer, Saturn 10 Libra: the chart the
    /// hand-derived Saptavargaja vectors below are worked on. In the order
    /// Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn.
    fn friendship_chart() -> [GrahaPosition; 7] {
        [
            gp(Graha::Sun, 126.0, 10),
            gp(Graha::Moon, 32.0, 2),
            gp(Graha::Mars, 5.5, 1),
            gp(Graha::Mercury, 62.0, 12),
            gp(Graha::Jupiter, 245.0, 9),
            gp(Graha::Venus, 100.0, 4),
            gp(Graha::Saturn, 190.0, 7),
        ]
    }

    // ── Drekkana order (BPHS 27.6) ──────────────────────────────────

    #[test]
    fn drekkana_order_is_the_order_of_the_words_of_the_verse() {
        // "adya-madhya-avasaneshu ... pum-napumsaka-yosha": male, neuter,
        // female against first, middle, last.
        let (first, middle, last) = (5.0, 15.0, 25.0);
        for (planet, third) in [
            (Graha::Sun, first),
            (Graha::Mars, first),
            (Graha::Jupiter, first),
            (Graha::Mercury, middle),
            (Graha::Saturn, middle),
            (Graha::Moon, last),
            (Graha::Venus, last),
        ] {
            assert!((drekkana_bala(planet, third) - 15.0).abs() < f64::EPSILON);
            for other in [first, middle, last] {
                if other != third {
                    assert!(drekkana_bala(planet, other).abs() < f64::EPSILON);
                }
            }
        }
    }

    #[test]
    fn the_other_drekkana_order_swaps_only_neuter_and_female() {
        let (first, middle, last) = (5.0, 15.0, 25.0);
        let swapped = DrekkanaOrder::MaleFemaleNeuter;
        // Females take the middle drekkana, neuters the last.
        for female in [Graha::Moon, Graha::Venus] {
            assert!((drekkana_bala_ordered(female, middle, swapped) - 15.0).abs() < f64::EPSILON);
            assert!(drekkana_bala_ordered(female, last, swapped).abs() < f64::EPSILON);
        }
        for neuter in [Graha::Mercury, Graha::Saturn] {
            assert!((drekkana_bala_ordered(neuter, last, swapped) - 15.0).abs() < f64::EPSILON);
            assert!(drekkana_bala_ordered(neuter, middle, swapped).abs() < f64::EPSILON);
        }
        // Males are in the first under either order.
        for male in [Graha::Sun, Graha::Mars, Graha::Jupiter] {
            assert!((drekkana_bala_ordered(male, first, swapped) - 15.0).abs() < f64::EPSILON);
        }
        // The default is the verse's.
        assert_eq!(DrekkanaOrder::default(), DrekkanaOrder::MaleNeuterFemale);
    }

    // ── Moolatrikona and sign lords (BPHS 3.49 to 3.54) ─────────────

    #[test]
    fn sign_lords_are_the_classical_rulerships() {
        let lords = [
            Graha::Mars,
            Graha::Venus,
            Graha::Mercury,
            Graha::Moon,
            Graha::Sun,
            Graha::Mercury,
            Graha::Venus,
            Graha::Mars,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Saturn,
            Graha::Jupiter,
        ];
        for (sign, lord) in lords.iter().enumerate() {
            assert_eq!(sign_lord(sign as u8), *lord, "sign {sign}");
        }
        // Each classical graha owns one sign (Sun, Moon) or two.
        for g in CLASSICAL {
            let n = (0..12u8).filter(|s| sign_lord(*s) == g).count();
            assert_eq!(
                n,
                if matches!(g, Graha::Sun | Graha::Moon) {
                    1
                } else {
                    2
                }
            );
        }
    }

    #[test]
    fn moolatrikona_arcs_are_the_degrees_of_3_51_to_3_54() {
        // (graha, sign, from, to) as the verses give them.
        let table = [
            (Graha::Sun, 4u8, 0.0, 20.0),
            (Graha::Moon, 1, 3.0, 30.0),
            (Graha::Mars, 0, 0.0, 12.0),
            (Graha::Mercury, 5, 15.0, 20.0),
            (Graha::Jupiter, 8, 0.0, 10.0),
            (Graha::Venus, 6, 0.0, 15.0),
            (Graha::Saturn, 10, 0.0, 20.0),
        ];
        for (g, sign, from, to) in table {
            let arc = moolatrikona_arc(g).expect("a classical graha");
            assert_eq!((arc.sign, arc.from_deg, arc.to_deg), (sign, from, to));
            let base = f64::from(sign) * 30.0;
            // Inside the arc at both ends, outside just beyond each.
            assert!(in_moolatrikona(g, base + from), "{g:?} at its arc start");
            assert!(in_moolatrikona(g, base + to - 1e-6), "{g:?} at its arc end");
            assert!(
                !in_moolatrikona(g, base + to),
                "{g:?} at its arc end (open)"
            );
            if from > 0.0 {
                assert!(!in_moolatrikona(g, base + from - 1e-6));
            }
            // Never in another sign.
            assert!(!in_moolatrikona(g, base + 30.0 + from + 1.0));
        }
        assert!(moolatrikona_arc(Graha::Rahu).is_none());
        assert!(!in_moolatrikona(Graha::Ketu, 100.0));
    }

    // ── The five-fold friendship (BPHS 3.55 to 3.58) ─────────────────

    #[test]
    fn natural_friendship_follows_the_signs_of_3_55() {
        use Friendship::{Enemy, Friend, Neutral};
        // Worked from the rule, not from a table.
        //
        // Sun: moolatrikona Simha (index 4). The 2nd, 4th, 5th, 8th, 9th and
        // 12th signs from it are Kanya, Vrishchika, Dhanu, Meena, Mesha, Karka
        // and its exaltation sign is Mesha. Friend-signs: Kanya, Vrishchika,
        // Dhanu, Meena, Mesha, Karka.
        //   Moon: owns Karka (friend sign) only          -> friend
        //   Mars: Mesha, Vrishchika (both friend signs)  -> friend
        //   Jupiter: Dhanu, Meena (both friend signs)    -> friend
        //   Mercury: Mithuna (not), Kanya (friend sign)  -> neutral
        //   Venus: Vrishabha (not), Tula (not)           -> enemy
        //   Saturn: Makara (not), Kumbha (not)           -> enemy
        let sun = [
            (Graha::Moon, Friend),
            (Graha::Mars, Friend),
            (Graha::Jupiter, Friend),
            (Graha::Mercury, Neutral),
            (Graha::Venus, Enemy),
            (Graha::Saturn, Enemy),
        ];
        for (to, want) in sun {
            assert_eq!(
                natural_relation(Graha::Sun, to),
                Some(want),
                "Sun -> {to:?}"
            );
        }
        // Moon: moolatrikona Vrishabha (1). 2nd Mithuna, 4th Simha, 5th Kanya,
        // 8th Dhanu, 9th Makara, 12th Mesha, exaltation Vrishabha.
        //   Sun: Simha (friend)                      -> friend
        //   Mercury: Mithuna, Kanya (both)           -> friend
        //   Mars: Mesha (friend), Vrishchika (not)   -> neutral
        //   Jupiter: Dhanu (friend), Meena (not)     -> neutral
        //   Venus: Vrishabha (friend), Tula (not)    -> neutral
        //   Saturn: Makara (friend), Kumbha (not)    -> neutral
        let moon = [
            (Graha::Sun, Friend),
            (Graha::Mercury, Friend),
            (Graha::Mars, Neutral),
            (Graha::Jupiter, Neutral),
            (Graha::Venus, Neutral),
            (Graha::Saturn, Neutral),
        ];
        for (to, want) in moon {
            assert_eq!(
                natural_relation(Graha::Moon, to),
                Some(want),
                "Moon -> {to:?}"
            );
        }
        // Mercury: moolatrikona Kanya (5). 2nd Tula, 4th Dhanu, 5th Makara,
        // 8th Mesha, 9th Vrishabha, 12th Simha, exaltation Kanya.
        // Friend-signs: Tula, Dhanu, Makara, Mesha, Vrishabha, Simha, Kanya.
        //   Venus: Vrishabha, Tula (both friend signs)     -> friend
        //   Sun: Simha (friend sign)                       -> friend
        //   Moon: Karka (not)                              -> enemy
        //   Mars: Mesha (friend), Vrishchika (not)         -> neutral
        //   Jupiter: Dhanu (friend), Meena (not)           -> neutral
        //   Saturn: Makara (friend), Kumbha (not)          -> neutral
        let mercury = [
            (Graha::Venus, Friend),
            (Graha::Sun, Friend),
            (Graha::Moon, Enemy),
            (Graha::Mars, Neutral),
            (Graha::Jupiter, Neutral),
            (Graha::Saturn, Neutral),
        ];
        for (to, want) in mercury {
            assert_eq!(
                natural_relation(Graha::Mercury, to),
                Some(want),
                "Mercury -> {to:?}"
            );
        }
    }

    #[test]
    fn natural_friendship_is_not_symmetric_and_has_no_self_or_node() {
        // Mars counts the Moon a friend; the Moon counts Mars neutral.
        assert_eq!(
            natural_relation(Graha::Mars, Graha::Moon),
            Some(Friendship::Friend)
        );
        assert_eq!(
            natural_relation(Graha::Moon, Graha::Mars),
            Some(Friendship::Neutral)
        );
        // The Moon has no natural enemy: every graha owns a friend sign or
        // is neutral toward her.
        for to in CLASSICAL {
            if to != Graha::Moon {
                assert_ne!(
                    natural_relation(Graha::Moon, to),
                    Some(Friendship::Enemy),
                    "Moon -> {to:?}"
                );
            }
        }
        for g in CLASSICAL {
            assert_eq!(natural_relation(g, g), None);
            assert_eq!(natural_relation(g, Graha::Rahu), None);
            assert_eq!(natural_relation(Graha::Ketu, g), None);
            // Every graha has at least one natural friend.
            assert!(
                CLASSICAL
                    .iter()
                    .any(|&o| natural_relation(g, o) == Some(Friendship::Friend)),
                "{g:?} has no friend"
            );
        }
    }

    #[test]
    fn temporary_friendship_is_the_2_3_4_10_11_12_rule() {
        // From sign 3 (Karka): friends are the signs 1, 2, 3, 9, 10, 11 on.
        for to in 0..12u8 {
            let offset = (to + 12 - 3) % 12;
            let want = if matches!(offset, 1 | 2 | 3 | 9 | 10 | 11) {
                Friendship::Friend
            } else {
                Friendship::Enemy
            };
            assert_eq!(temporary_relation(3, to), want, "offset {offset}");
        }
        // The same sign is an enemy (BPHS: placed elsewhere are friends).
        assert_eq!(temporary_relation(7, 7), Friendship::Enemy);
        // Symmetric: the 2nd from one is the 12th from the other.
        for a in 0..12u8 {
            for b in 0..12u8 {
                assert_eq!(temporary_relation(a, b), temporary_relation(b, a));
            }
        }
    }

    #[test]
    fn panchadha_maitri_compounds_as_3_57_says() {
        use Maitri::{AdhiMitra, AdhiShatru, Mitra, Sama, Shatru};
        // Natural Friend (Sun -> Moon), Neutral (Sun -> Mercury), Enemy
        // (Sun -> Venus); temporary Friend when the two signs are 2, 3, 4,
        // 10, 11 or 12 apart, Enemy otherwise (Sun in Simha, 4).
        let sun = 4;
        // Moon in Karka (offset 11): friend + friend.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Moon, sun, 3),
            Some(AdhiMitra)
        );
        // Moon in the same sign: friend + enemy.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Moon, sun, 4),
            Some(Sama)
        );
        // Mercury in Kanya (offset 1): neutral + friend.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Mercury, sun, 5),
            Some(Mitra)
        );
        // Mercury in Simha: neutral + enemy.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Mercury, sun, 4),
            Some(Shatru)
        );
        // Venus in Kanya (offset 1): enemy + friend.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Venus, sun, 5),
            Some(Sama)
        );
        // Venus in Dhanu (offset 4): enemy + enemy.
        assert_eq!(
            panchadha_maitri(Graha::Sun, Graha::Venus, sun, 8),
            Some(AdhiShatru)
        );
        assert_eq!(panchadha_maitri(Graha::Sun, Graha::Sun, 4, 4), None);
    }

    // ── Saptavargaja Bala (BPHS 27.2 to 27.4) ────────────────────────

    #[test]
    fn saptavargaja_sun_worked_by_hand() {
        let chart = friendship_chart();
        let (scope_all, scope_rashi) = (MoolatrikonaScope::AllVargas, MoolatrikonaScope::RashiOnly);
        let ladder = SaptavargajaLadder::BPHS;
        // Sun at 6 deg Simha (126 deg). The seven varga signs, worked from
        // the varga rules, then confirmed against the module:
        //   D1  Simha (4), in the 0-20 arc                    moolatrikona
        //   D2  odd sign, first half            -> Simha (4)
        //   D3  first decanate                  -> Simha (4)
        //   D7  odd sign: count from Simha; floor(6*7/30) = 1 -> Kanya (5)
        //   D9  fixed sign: from the 9th (Mesha); floor(6*9/30) = 1 -> Vrishabha (1)
        //   D12 from Simha; floor(6*12/30) = 2   -> Tula (6)
        //   D30 odd sign, 5-10 degrees: Saturn   -> Kumbha (10)
        let signs = [4u8, 4, 4, 5, 1, 6, 10];
        for (varga, want) in SAPTAVARGA.iter().zip(signs) {
            assert_eq!(varga_sign(126.0, *varga), want, "{varga:?}");
        }
        // Lords and relations to the Sun (Simha, 4); the other grahas' D1
        // signs are Mercury Mithuna (2), Venus Karka (3), Saturn Tula (6):
        //   Kanya -> Mercury: neutral by nature; Mithuna is 11th from Simha
        //            (offset 10), a temporary friend      -> mitra     15
        //   Vrishabha -> Venus: enemy by nature; Karka is 12th (offset 11),
        //            a temporary friend                   -> sama      10
        //   Tula -> Venus: the same                        -> sama      10
        //   Kumbha -> Saturn: enemy by nature; Tula is 3rd (offset 2),
        //            a temporary friend                   -> sama      10
        // D1 45; D2 and D3 are Simha, the moolatrikona sign: 45 each where
        // the sign is tested, 30 (own) where only D1 gets the grade.
        let all = saptavargaja_bala(Graha::Sun, &chart, scope_all, ladder).expect("complete chart");
        assert!(
            (all - (45.0 + 45.0 + 45.0 + 15.0 + 10.0 + 10.0 + 10.0)).abs() < 1e-12,
            "{all}"
        );
        let rashi =
            saptavargaja_bala(Graha::Sun, &chart, scope_rashi, ladder).expect("complete chart");
        assert!(
            (rashi - (45.0 + 30.0 + 30.0 + 15.0 + 10.0 + 10.0 + 10.0)).abs() < 1e-12,
            "{rashi}"
        );
    }

    #[test]
    fn saptavargaja_moon_worked_by_hand() {
        let chart = friendship_chart();
        let ladder = SaptavargajaLadder::BPHS;
        // Moon at 2 deg Vrishabha (32 deg): inside the 0-3 exaltation, so
        // not in the 3-30 moolatrikona arc; graded as in a sign of Venus.
        //   D1  Vrishabha (1)                         lord Venus
        //   D2  even sign, first half     -> Karka (3)  lord Moon
        //   D3  first decanate            -> Vrishabha (1)
        //   D7  even sign: from the 7th (Vrishchika); floor(2*7/30)=0 -> Vrishchika (7), Mars
        //   D9  fixed sign: from the 9th (Makara); floor(2*9/30)=0 -> Makara (9), Saturn
        //   D12 from Vrishabha; floor(2*12/30)=0   -> Vrishabha (1)
        //   D30 even sign, 0-5: Venus     -> Tula (6), Venus
        let signs = [1u8, 3, 1, 7, 9, 1, 6];
        for (varga, want) in SAPTAVARGA.iter().zip(signs) {
            assert_eq!(varga_sign(32.0, *varga), want, "{varga:?}");
        }
        // Relations (Moon in Vrishabha, 1; Venus Karka 3, Mars Mesha 0, Saturn Tula 6):
        //   Venus: neutral by nature; Karka is 3rd (offset 2) friend -> mitra 15
        //   Mars: neutral by nature; Mesha is 12th (offset 11) friend -> mitra 15
        //   Saturn: neutral by nature; Tula is 6th (offset 5) enemy -> shatru 4
        //   D2 Karka is the Moon's own sign                           -> 30
        // AllVargas: Vrishabha in D3 and D12 is the moolatrikona sign -> 45 each.
        let all = saptavargaja_bala(Graha::Moon, &chart, MoolatrikonaScope::AllVargas, ladder)
            .expect("complete chart");
        let want_all = 15.0 + 30.0 + 45.0 + 15.0 + 4.0 + 45.0 + 15.0;
        assert!((all - want_all).abs() < 1e-12, "{all} != {want_all}");
        // RashiOnly: those two are Venus's sign, mitra, 15 each.
        let rashi = saptavargaja_bala(Graha::Moon, &chart, MoolatrikonaScope::RashiOnly, ladder)
            .expect("complete chart");
        let want_rashi = 15.0 + 30.0 + 15.0 + 15.0 + 4.0 + 15.0 + 15.0;
        assert!(
            (rashi - want_rashi).abs() < 1e-12,
            "{rashi} != {want_rashi}"
        );
    }

    #[test]
    fn saptavargaja_mars_in_its_arc_and_the_ladder_is_a_parameter() {
        let chart = friendship_chart();
        // Mars at 5.5 deg Mesha is inside the 0-12 arc: D1 is 45. Varga signs:
        // D2 Simha (Sun), D3 Mesha, D7 Vrishabha (Venus), D9 Vrishabha,
        // D12 Mithuna (Mercury), D30 Kumbha (Saturn). Mars's D1 sign is 0.
        //   Sun (Simha, 4): friend by nature; offset 4 -> temporary enemy -> sama 10
        //   D3 Mesha: AllVargas moolatrikona 45 / RashiOnly own 30
        //   Venus (Karka, 3): neutral; offset 3 friend -> mitra 15 (D7 and D9)
        //   Mercury (Mithuna, 2): enemy; offset 2 friend -> sama 10
        //   Saturn (Tula, 6): neutral; offset 6 enemy -> shatru 4
        let ladder = SaptavargajaLadder::BPHS;
        let all = saptavargaja_bala(Graha::Mars, &chart, MoolatrikonaScope::AllVargas, ladder)
            .expect("complete chart");
        assert!(
            (all - (45.0 + 10.0 + 45.0 + 15.0 + 15.0 + 10.0 + 4.0)).abs() < 1e-12,
            "{all}"
        );
        let rashi = saptavargaja_bala(Graha::Mars, &chart, MoolatrikonaScope::RashiOnly, ladder)
            .expect("complete chart");
        assert!(
            (rashi - (45.0 + 10.0 + 30.0 + 15.0 + 15.0 + 10.0 + 4.0)).abs() < 1e-12,
            "{rashi}"
        );

        // The ladder is a parameter: halve every grade, halve the answer.
        let half = SaptavargajaLadder {
            moolatrikona: 22.5,
            own: 15.0,
            adhimitra: 10.0,
            mitra: 7.5,
            sama: 5.0,
            shatru: 2.0,
            adhishatru: 1.0,
        };
        let halved = saptavargaja_bala(Graha::Mars, &chart, MoolatrikonaScope::AllVargas, half)
            .expect("complete chart");
        assert!((halved - all / 2.0).abs() < 1e-12);
        assert_eq!(SaptavargajaLadder::default(), SaptavargajaLadder::BPHS);
        assert_eq!(MoolatrikonaScope::default(), MoolatrikonaScope::AllVargas);
    }

    #[test]
    fn saptavargaja_edge_cases() {
        let chart = friendship_chart();
        let ladder = SaptavargajaLadder::BPHS;
        let scope = MoolatrikonaScope::AllVargas;
        // Rahu and Ketu are outside the classical rule.
        assert_eq!(
            saptavargaja_bala(Graha::Rahu, &chart, scope, ladder),
            Some(0.0)
        );
        assert_eq!(
            saptavargaja_bala(Graha::Ketu, &chart, scope, ladder),
            Some(0.0)
        );
        // A missing lord is a missing answer, not a guess: the Sun's D9 sign
        // is Vrishabha, whose lord Venus must be in the chart.
        let no_venus: Vec<GrahaPosition> = chart
            .iter()
            .copied()
            .filter(|p| p.planet != Graha::Venus)
            .collect();
        assert_eq!(
            saptavargaja_bala(Graha::Sun, &no_venus, scope, ladder),
            None
        );
        // And a missing graha itself.
        assert_eq!(
            saptavargaja_bala(Graha::Venus, &no_venus, scope, ladder),
            None
        );
        // Bounds: seven grades, each between the ladder's lowest and highest.
        for step in 0..360 {
            let mut c = chart;
            c[0] = gp(Graha::Sun, f64::from(step), 1);
            let v = saptavargaja_bala(Graha::Sun, &c, scope, ladder).expect("complete");
            assert!((14.0..=315.0).contains(&v), "{step}: {v}");
        }
    }

    // ── Nathonnata Bala (BPHS 27.8 and 27.9) ─────────────────────────

    #[test]
    fn nathonnata_is_continuous_between_midnight_and_noon() {
        // Midnight: unnata 0, nata 30 ghatis. Moon/Mars/Saturn get 2 * 30 =
        // 60; Sun/Jupiter/Venus get 60 - 60 = 0; Mercury 60.
        for night in [Graha::Moon, Graha::Mars, Graha::Saturn] {
            assert!((nathonnata_bala(night, 0.0) - 60.0).abs() < 1e-12);
            assert!(nathonnata_bala(night, 12.0).abs() < 1e-12);
        }
        for day in [Graha::Sun, Graha::Jupiter, Graha::Venus] {
            assert!(nathonnata_bala(day, 0.0).abs() < 1e-12);
            assert!((nathonnata_bala(day, 12.0) - 60.0).abs() < 1e-12);
        }
        // 2:20 pm apparent time: 14h20m is 215 degrees from midnight, folded
        // to d = 145 degrees (unnata 24.1667 ghatis, nata 5.8333).
        //   night group: 2 * nata = 11.6667   = (180 - 145) / 3
        //   day group:   60 - 11.6667 = 48.3333 = 145 / 3
        let t = 14.0 + 20.0 / 60.0;
        assert!((nathonnata_bala(Graha::Moon, t) - 35.0 / 3.0).abs() < 1e-9);
        assert!((nathonnata_bala(Graha::Sun, t) - 145.0 / 3.0).abs() < 1e-9);
        assert!((nathonnata_bala(Graha::Mercury, t) - 60.0).abs() < 1e-12);
        assert!(nathonnata_bala(Graha::Rahu, t).abs() < 1e-12);
    }

    #[test]
    fn nathonnata_day_and_night_groups_sum_to_a_rupa_and_are_symmetric() {
        for step in 0..=480 {
            let h = f64::from(step) * 0.05;
            let day = nathonnata_bala(Graha::Sun, h);
            let night = nathonnata_bala(Graha::Moon, h);
            assert!((day + night - 60.0).abs() < 1e-9, "at {h} h");
            // The same distance either side of midnight scores the same.
            let mirrored = nathonnata_bala(Graha::Sun, 24.0 - h);
            assert!((day - mirrored).abs() < 1e-9, "at {h} h");
            // Continuous: a minute's change is a small change (no 0-or-30
            // step).
            let next = nathonnata_bala(Graha::Sun, h + 1.0 / 60.0);
            assert!((next - day).abs() < 0.34, "step at {h} h");
        }
    }

    #[test]
    fn apparent_time_from_the_sun_hour_angle() {
        assert!((apparent_hours_from_sun_hour_angle(0.0) - 12.0).abs() < 1e-12);
        assert!((apparent_hours_from_sun_hour_angle(90.0) - 18.0).abs() < 1e-12);
        assert!(apparent_hours_from_sun_hour_angle(180.0).abs() < 1e-12);
        assert!((apparent_hours_from_sun_hour_angle(-90.0) - 6.0).abs() < 1e-12);
    }

    // ── Paksha Bala (BPHS 27.10 and 27.11) ───────────────────────────

    #[test]
    fn paksha_is_the_folded_elongation_over_three() {
        // Elongation 35 degrees: benefic 35/3 = 11.6667, malefic 48.3333.
        assert!((paksha_pool(135.0, 100.0) - 35.0 / 3.0).abs() < 1e-12);
        // Folded: Moon 325 degrees behind the Sun is 35 ahead of it... the
        // elongation 325 folds to 35.
        assert!((paksha_pool(100.0, 135.0) - 35.0 / 3.0).abs() < 1e-12);
        // New moon 0, full moon 60 (the benefic group), either side of 0.
        assert!(paksha_pool(250.0, 250.0).abs() < 1e-12);
        assert!((paksha_pool(70.0, 250.0) - 60.0).abs() < 1e-12);
        assert!((paksha_pool(1.0, 359.0) - 2.0 / 3.0).abs() < 1e-12);
        // Per graha: benefics the pool, the rest 60 less.
        let (moon, sun) = (135.0, 100.0);
        for b in BPHS_PAKSHA_BENEFICS {
            assert!((paksha_bala(b, moon, sun, &BPHS_PAKSHA_BENEFICS) - 35.0 / 3.0).abs() < 1e-12);
        }
        for m in [Graha::Sun, Graha::Mars, Graha::Saturn] {
            assert!(
                (paksha_bala(m, moon, sun, &BPHS_PAKSHA_BENEFICS) - (60.0 - 35.0 / 3.0)).abs()
                    < 1e-12
            );
        }
        assert!(paksha_bala(Graha::Rahu, moon, sun, &BPHS_PAKSHA_BENEFICS).abs() < 1e-12);
    }

    #[test]
    fn the_benefic_set_is_an_argument() {
        let (moon, sun) = (135.0, 100.0);
        // The verse names the Moon, Mercury, Venus, Jupiter.
        assert_eq!(
            BPHS_PAKSHA_BENEFICS,
            [Graha::Moon, Graha::Mercury, Graha::Venus, Graha::Jupiter]
        );
        // Mercury with a malefic is a malefic by doctrine the caller holds:
        // leave him out and he takes the complement.
        let without_mercury = [Graha::Moon, Graha::Venus, Graha::Jupiter];
        assert!(
            (paksha_bala(Graha::Mercury, moon, sun, &without_mercury) - (60.0 - 35.0 / 3.0)).abs()
                < 1e-12
        );
        // Put Saturn in and he takes the pool.
        let with_saturn = [Graha::Saturn];
        assert!((paksha_bala(Graha::Saturn, moon, sun, &with_saturn) - 35.0 / 3.0).abs() < 1e-12);
        // The two groups always make a rupa: a graha in the set and the same
        // graha out of it.
        for g in CLASSICAL {
            let inside = paksha_bala(g, moon, sun, &[g]);
            let outside = paksha_bala(g, moon, sun, &[]);
            assert!((inside + outside - 60.0).abs() < 1e-12);
        }
    }

    // ── Tribhaga Bala (BPHS 27.12) ───────────────────────────────────

    #[test]
    fn tribhaga_lords_by_third_of_day_and_night() {
        // Day 0.0 to 0.75 (thirds of 0.25), night 0.75 to 1.5 (0.25 each):
        // every boundary is a power of two, so the half-open rule is exact.
        let (sr, ss, nsr) = (0.0, 0.75, 1.5);
        let cases = [
            (0.0, Graha::Mercury),
            (0.2, Graha::Mercury),
            (0.25, Graha::Sun),
            (0.49, Graha::Sun),
            (0.5, Graha::Saturn),
            (0.74, Graha::Saturn),
            (0.75, Graha::Moon),
            (0.99, Graha::Moon),
            (1.0, Graha::Venus),
            (1.24, Graha::Venus),
            (1.25, Graha::Mars),
            (1.49, Graha::Mars),
        ];
        for (jd, want) in cases {
            assert_eq!(tribhaga_lord(jd, sr, ss, nsr), Some(want), "jd {jd}");
        }
        // Outside the day, or the instants out of order: no answer.
        assert_eq!(tribhaga_lord(-0.01, sr, ss, nsr), None);
        assert_eq!(tribhaga_lord(1.5, sr, ss, nsr), None);
        assert_eq!(tribhaga_lord(0.5, sr, nsr, ss), None);
    }

    #[test]
    fn tribhaga_bala_gives_a_rupa_to_jupiter_and_the_lord() {
        for lord in [
            Graha::Mercury,
            Graha::Sun,
            Graha::Saturn,
            Graha::Moon,
            Graha::Venus,
            Graha::Mars,
        ] {
            let total: f64 = CLASSICAL.iter().map(|&g| tribhaga_bala(g, lord)).sum();
            // Exactly two grahas share 60 each: Jupiter and the lord.
            assert!((total - 120.0).abs() < 1e-12, "{lord:?}");
            assert!((tribhaga_bala(lord, lord) - 60.0).abs() < 1e-12);
            assert!((tribhaga_bala(Graha::Jupiter, lord) - 60.0).abs() < 1e-12);
        }
    }

    // ── Year, month, day and hour lords (BPHS 27.13; SS I.51 and I.52) ─

    #[test]
    fn the_kali_epoch_ahargana_is_the_surya_siddhanta_count() {
        // 1,955,880,000 years elapsed at the Kali Yuga, at 1,577,917,828
        // civil days to the 4,320,000-year mahayuga: 452.75 mahayugas, a whole
        // number of days.
        let numerator = 1_955_880_000i128 * 1_577_917_828i128;
        assert_eq!(numerator % 4_320_000, 0, "a whole number of days");
        assert_eq!(numerator / 4_320_000, i128::from(AHARGANA_AT_KALI_EPOCH));
        // Kali Yuga began on a Friday.
        assert_eq!(vara_lord(ahargana_from_jdn(KALI_EPOCH_JDN)), Graha::Venus);
    }

    /// The weekday lord by the calendar, with no appeal to the ahargana:
    /// Julian Day Number plus one, mod seven, 0 = Sunday.
    fn calendar_lord(jdn: i64) -> Graha {
        const SUN_FIRST: [Graha; 7] = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ];
        SUN_FIRST[(jdn + 1).rem_euclid(7) as usize]
    }

    fn weekday_index(g: Graha) -> i64 {
        [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ]
        .iter()
        .position(|&x| x == g)
        .expect("a classical graha") as i64
    }

    #[test]
    fn the_day_lord_is_the_calendar_weekday() {
        // 2000-01-01 (JDN 2451545) was a Saturday; 2026-10-24 (2461338) too.
        assert_eq!(vara_lord(ahargana_from_jdn(2_451_545)), Graha::Saturn);
        assert_eq!(vara_lord(ahargana_from_jdn(2_461_338)), Graha::Saturn);
        // 1860-01-01 (2400411) was a Sunday.
        assert_eq!(vara_lord(ahargana_from_jdn(2_400_411)), Graha::Sun);
        for jdn in 2_400_000..2_400_000 + 5_000 {
            assert_eq!(
                vara_lord(ahargana_from_jdn(jdn)),
                calendar_lord(jdn),
                "JDN {jdn}"
            );
        }
        // Far from the epoch, both ways.
        for jdn in [1_000_000i64, 588_466, 3_000_000, 4_000_000] {
            assert_eq!(vara_lord(ahargana_from_jdn(jdn)), calendar_lord(jdn));
        }
    }

    #[test]
    fn month_and_year_lords_run_two_and_three_places_on() {
        // SS I.52: the lord of a 30-day "month" is the lord of its first day;
        // of a 360-day "year", the lord of its first day. Successive month
        // lords run two places on in the weekday order, year lords three.
        let start = ahargana_from_jdn(2_450_000);
        let first_month = start.div_euclid(30) * 30;
        let first_year = start.div_euclid(360) * 360;
        for q in 0..400i64 {
            let a = first_month + 30 * q;
            for day in 0..30 {
                assert_eq!(
                    masa_lord(a + day),
                    vara_lord(a),
                    "month block {q}, day {day}"
                );
            }
            let next = weekday_index(masa_lord(a + 30));
            assert_eq!(next, (weekday_index(masa_lord(a)) + 2) % 7, "month {q}");
        }
        for q in 0..60i64 {
            let a = first_year + 360 * q;
            for day in [0, 1, 29, 30, 180, 359] {
                assert_eq!(
                    abda_lord(a + day),
                    vara_lord(a),
                    "year block {q}, day {day}"
                );
            }
            let next = weekday_index(abda_lord(a + 360));
            assert_eq!(next, (weekday_index(abda_lord(a)) + 3) % 7, "year {q}");
        }
    }

    #[test]
    fn hindu_day_jdn_labels_the_day_by_the_local_date_of_its_sunrise() {
        // Sunrise 2026-10-24 06:00 UT, at UT: that date.
        assert_eq!(hindu_day_jdn(2_461_337.75, 0.0), 2_461_338);
        // The same sunrise seen from UTC+5.5 is 11:30 local on the same date.
        assert_eq!(hindu_day_jdn(2_461_337.75, 5.5), 2_461_338);
        // A sunrise at 20:00 UT of the 23rd is already the 24th at UTC+8
        // (04:00 local): the date moves with the offset.
        assert_eq!(hindu_day_jdn(2_461_337.333, 0.0), 2_461_337);
        assert_eq!(hindu_day_jdn(2_461_337.333, 8.0), 2_461_338);
    }

    #[test]
    fn hora_lords_follow_the_chaldean_order_from_the_day_lord() {
        // Equal horas over one day from sunrise at 0.0 to the next at 1.0.
        let (sr, ss, nsr) = (0.0, 0.5, 1.0);
        let sunday = [
            Graha::Sun,
            Graha::Venus,
            Graha::Mercury,
            Graha::Moon,
            Graha::Saturn,
            Graha::Jupiter,
            Graha::Mars,
        ];
        for hour in 0..24usize {
            let jd = (hour as f64 + 0.5) / 24.0;
            assert_eq!(
                hora_lord(HoraKind::Equal, jd, sr, ss, nsr, Graha::Sun),
                Some(sunday[hour % 7]),
                "hora {}",
                hour + 1
            );
        }
        // The 25th hora would be Monday's first: the Moon.
        assert_eq!(sunday[24 % 7], Graha::Moon);
        // Every day lord, not only the Sun: the first hora is the day lord's
        // and the order is Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon
        // (SS XII.79), wrapping.
        let chaldean = [
            Graha::Saturn,
            Graha::Jupiter,
            Graha::Mars,
            Graha::Sun,
            Graha::Venus,
            Graha::Mercury,
            Graha::Moon,
        ];
        for vara in CLASSICAL {
            let start = chaldean.iter().position(|&g| g == vara).expect("in order");
            for hour in 0..24usize {
                let jd = (hour as f64 + 0.5) / 24.0;
                assert_eq!(
                    hora_lord(HoraKind::Equal, jd, sr, ss, nsr, vara),
                    Some(chaldean[(start + hour) % 7]),
                    "{vara:?}, hora {}",
                    hour + 1
                );
            }
        }
        // Another day starts from its own lord.
        assert_eq!(
            hora_lord(HoraKind::Equal, 0.01, sr, ss, nsr, Graha::Moon),
            Some(Graha::Moon)
        );
        assert_eq!(
            hora_lord(HoraKind::Equal, 0.01, sr, ss, nsr, Graha::Saturn),
            Some(Graha::Saturn)
        );
        // Outside the day: no answer.
        assert_eq!(
            hora_lord(HoraKind::Equal, 1.0, sr, ss, nsr, Graha::Sun),
            None
        );
        assert_eq!(
            hora_lord(HoraKind::Equal, -0.1, sr, ss, nsr, Graha::Sun),
            None
        );
    }

    #[test]
    fn the_hora_division_is_a_parameter() {
        // An 18-hour day and a 6-hour night: the unequal horas are 1.5 h by
        // day and 0.5 h by night.
        let (sr, ss, nsr) = (0.0, 0.75, 1.0);
        let jd = 0.7; // 16.8 h after sunrise
        // Equal horas: hora index floor(16.8) = 16, 16 % 7 = 2 -> Mercury.
        assert_eq!(
            hora_lord(HoraKind::Equal, jd, sr, ss, nsr, Graha::Sun),
            Some(Graha::Mercury)
        );
        // Unequal: the 11th day hora (0.7 / 0.0625 = 11.2), index 11 % 7 = 4
        // -> Saturn.
        assert_eq!(
            hora_lord(HoraKind::Unequal, jd, sr, ss, nsr, Graha::Sun),
            Some(Graha::Saturn)
        );
        assert_eq!(HoraKind::default(), HoraKind::Equal);
        // With a 12-hour day the two coincide.
        for k in 0..48 {
            let t = (f64::from(k) + 0.5) / 48.0;
            assert_eq!(
                hora_lord(HoraKind::Equal, t, 0.0, 0.5, 1.0, Graha::Venus),
                hora_lord(HoraKind::Unequal, t, 0.0, 0.5, 1.0, Graha::Venus)
            );
        }
    }

    // ── Ayana Bala (BPHS 27.15 to 27.17) ─────────────────────────────

    #[test]
    fn ayana_khanda_worked_from_the_table() {
        // Khandas 45, 33, 12. For the Sun (north is favoured) the result is
        // (90 + scale) / 3 in the northern signs, (90 - scale) / 3 in the
        // southern.
        //   lon 0:   scale 0                         -> 30
        //   lon 15:  15 * 45/30 = 22.5               -> 37.5
        //   lon 30:  45                               -> 45
        //   lon 45:  45 + 15 * 33/30 = 61.5          -> 50.5
        //   lon 60:  45 + 33 = 78                    -> 56
        //   lon 75:  78 + 15 * 12/30 = 84            -> 58
        //   lon 90:  90                              -> 60
        let cases = [
            (0.0, 30.0),
            (15.0, 37.5),
            (30.0, 45.0),
            (45.0, 50.5),
            (60.0, 56.0),
            (75.0, 58.0),
            (90.0, 60.0),
        ];
        for (lon, want) in cases {
            let got = ayana_bala_khanda(Graha::Sun, lon);
            assert!((got - want).abs() < 1e-12, "lon {lon}: {got} != {want}");
            // The bhuja is the arc from the nearer equinox: 180 - lon is the
            // same arc on the other side of the solstice.
            let mirrored = ayana_bala_khanda(Graha::Sun, 180.0 - lon);
            assert!((mirrored - want).abs() < 1e-12, "lon {}", 180.0 - lon);
        }
        // The southern half is the Sun's weak side: 90 - scale.
        assert!((ayana_bala_khanda(Graha::Sun, 195.0) - 22.5).abs() < 1e-12);
        assert!(ayana_bala_khanda(Graha::Sun, 270.0).abs() < 1e-12);
        assert!((ayana_bala_khanda(Graha::Sun, 180.0) - 30.0).abs() < 1e-12);
        assert!((ayana_bala_khanda(Graha::Sun, 360.0) - 30.0).abs() < 1e-12);
    }

    #[test]
    fn ayana_sides_by_graha() {
        // North favoured: Sun, Mars, Jupiter, Venus.
        for g in [Graha::Sun, Graha::Mars, Graha::Jupiter, Graha::Venus] {
            assert!((ayana_bala_khanda(g, 90.0) - 60.0).abs() < 1e-12, "{g:?}");
            assert!(ayana_bala_khanda(g, 270.0).abs() < 1e-12, "{g:?}");
        }
        // South favoured: Moon, Saturn.
        for g in [Graha::Moon, Graha::Saturn] {
            assert!(ayana_bala_khanda(g, 90.0).abs() < 1e-12, "{g:?}");
            assert!((ayana_bala_khanda(g, 270.0) - 60.0).abs() < 1e-12, "{g:?}");
            // At the equinox, half.
            assert!((ayana_bala_khanda(g, 0.0) - 30.0).abs() < 1e-12);
        }
        // Mercury is always joined: 60 at either solstice, 30 at the equinoxes,
        // and the same at a longitude and its mirror across the equator.
        assert!((ayana_bala_khanda(Graha::Mercury, 90.0) - 60.0).abs() < 1e-12);
        assert!((ayana_bala_khanda(Graha::Mercury, 270.0) - 60.0).abs() < 1e-12);
        assert!((ayana_bala_khanda(Graha::Mercury, 0.0) - 30.0).abs() < 1e-12);
        assert!(
            (ayana_bala_khanda(Graha::Mercury, 15.0) - ayana_bala_khanda(Graha::Mercury, 195.0))
                .abs()
                < 1e-12
        );
        // The pair that favour opposite sides sum to a rupa at any longitude.
        for step in 0..360 {
            let lon = f64::from(step);
            let sum = ayana_bala_khanda(Graha::Sun, lon) + ayana_bala_khanda(Graha::Moon, lon);
            assert!((sum - 60.0).abs() < 1e-9, "lon {lon}");
        }
        assert!(ayana_bala_khanda(Graha::Rahu, 90.0).abs() < 1e-12);
    }

    #[test]
    fn the_khanda_table_follows_the_true_declination() {
        // The table is a coarse stand-in for the declination of the ecliptic
        // point, sin(eps) sin(lambda): it must stay within 0.7 degrees of it
        // over the whole circle (greatest at 30 and 60 degrees), and the
        // scale (0..90 for 0..24 degrees) must agree.
        let eps = 23.44_f64.to_radians();
        let mut worst = 0.0_f64;
        for step in 0..3600 {
            let lon = f64::from(step) / 10.0;
            let true_decl = (eps.sin() * lon.to_radians().sin()).asin().to_degrees();
            // Recover the khanda declination from the Sun's bala:
            // bala = (90 + scale) / 3 north, so scale = 3 bala - 90.
            let bala = ayana_bala_khanda(Graha::Sun, lon);
            let signed_scale = 3.0 * bala - 90.0;
            let khanda_decl = signed_scale / 90.0 * AYANA_GREATEST_DECLINATION_DEG;
            worst = worst.max((khanda_decl - true_decl).abs());
        }
        assert!(worst < 0.7, "worst khanda error {worst} degrees");
        assert!(worst > 0.3, "the test cannot see an error of {worst}");
    }

    #[test]
    fn ayana_declination_reading_is_24_plus_k_over_48_of_a_rupa() {
        // Sun: (24 + k) / 48 * 60.
        let cases = [
            (24.0, 60.0),
            (12.0, 45.0),
            (0.0, 30.0),
            (-12.0, 15.0),
            (-24.0, 0.0),
        ];
        for (k, want) in cases {
            let got = ayana_bala_declination(Graha::Sun, k);
            assert!((got - want).abs() < 1e-12, "k {k}: {got} != {want}");
            // The Moon is the mirror image.
            let moon = ayana_bala_declination(Graha::Moon, -k);
            assert!((moon - want).abs() < 1e-12);
        }
        // Mercury takes the magnitude.
        assert!((ayana_bala_declination(Graha::Mercury, 12.0) - 45.0).abs() < 1e-12);
        assert!((ayana_bala_declination(Graha::Mercury, -12.0) - 45.0).abs() < 1e-12);
        // The Moon can stand beyond 24 degrees: clamped, never outside 0..60.
        assert!((ayana_bala_declination(Graha::Sun, 28.0) - 60.0).abs() < 1e-12);
        assert!(ayana_bala_declination(Graha::Moon, 28.0).abs() < 1e-12);
        assert!(ayana_bala_declination(Graha::Ketu, 12.0).abs() < 1e-12);
        // The two readings agree for the Sun to within a virupa (the Sun has
        // no latitude, so the difference is the table's own).
        let eps = 23.44_f64.to_radians();
        for step in 0..360 {
            let lon = f64::from(step);
            let decl = (eps.sin() * lon.to_radians().sin()).asin().to_degrees();
            let d = (ayana_bala_khanda(Graha::Sun, lon) - ayana_bala_declination(Graha::Sun, decl))
                .abs();
            assert!(d < 1.0, "lon {lon}: readings differ by {d}");
        }
        assert_eq!(AyanaReading::default(), AyanaReading::KhandaOfLongitude);
    }

    // ── Yuddha Bala (BPHS 27.20) ─────────────────────────────────────

    fn war(graha: Graha, lon: f64, lat: f64, total: f64) -> WarCandidate {
        WarCandidate {
            graha,
            longitude_deg: lon,
            latitude_deg: lat,
            shadbala_before_war: total,
        }
    }

    #[test]
    fn the_victor_gains_what_the_vanquished_loses() {
        // Mars (300) and Saturn (250) 0.5 degrees apart; Saturn is further
        // north and wins. The difference of the two shadbalas, 50, moves
        // from Mars to Saturn.
        let c = [
            war(Graha::Mars, 100.0, 1.0, 300.0),
            war(Graha::Saturn, 100.5, 2.0, 250.0),
        ];
        let a = yuddha_adjustments(&c, YuddhaRule::NorthernLatitude);
        assert_eq!(a, vec![-50.0, 50.0]);
        // Whichever wins, the pair sums to zero, and the stronger one losing
        // still pays the difference.
        let b = yuddha_adjustments(&c, YuddhaRule::Brighter); // Mars outranks Saturn
        assert_eq!(b, vec![50.0, -50.0]);
        assert!(a.iter().sum::<f64>().abs() < 1e-12);
        // A weaker victor gains the same difference.
        let weak = [
            war(Graha::Mars, 100.0, 3.0, 200.0),
            war(Graha::Saturn, 100.5, 2.0, 260.0),
        ];
        assert_eq!(
            yuddha_adjustments(&weak, YuddhaRule::NorthernLatitude),
            vec![60.0, -60.0]
        );
    }

    #[test]
    fn no_war_no_adjustment() {
        // More than a degree apart.
        let far = [
            war(Graha::Mars, 100.0, 1.0, 300.0),
            war(Graha::Saturn, 101.5, 2.0, 250.0),
        ];
        assert_eq!(
            yuddha_adjustments(&far, YuddhaRule::NorthernLatitude),
            vec![0.0, 0.0]
        );
        // The Sun and Moon are never combatants, however close.
        let luminary = [
            war(Graha::Mars, 100.0, 1.0, 300.0),
            war(Graha::Sun, 100.2, 0.0, 400.0),
            war(Graha::Moon, 100.4, 0.0, 200.0),
        ];
        assert_eq!(
            yuddha_adjustments(&luminary, YuddhaRule::NorthernLatitude),
            vec![0.0, 0.0, 0.0]
        );
        assert!(yuddha_adjustments(&[], YuddhaRule::Brighter).is_empty());
    }

    #[test]
    fn a_three_way_war_is_three_pairs() {
        // Mars (300, lat 3), Mercury (200, lat 2), Jupiter (250, lat 1)
        // within a degree of one another, northerly wins each pair:
        //   Mars-Mercury: Mars wins, 100        Mars +100, Mercury -100
        //   Mars-Jupiter: Mars wins, 50         Mars +50,  Jupiter -50
        //   Mercury-Jupiter: Mercury wins, 50   Mercury +50, Jupiter -50
        let c = [
            war(Graha::Mars, 100.0, 3.0, 300.0),
            war(Graha::Mercury, 100.4, 2.0, 200.0),
            war(Graha::Jupiter, 100.8, 1.0, 250.0),
        ];
        let a = yuddha_adjustments(&c, YuddhaRule::NorthernLatitude);
        assert_eq!(a, vec![150.0, -50.0, -100.0]);
        assert!(a.iter().sum::<f64>().abs() < 1e-12);
    }

    // ── Cheshta Bala from the cheshta kendra (BPHS 27.24 and 27.25) ──

    #[test]
    fn the_kendra_of_mars_jupiter_saturn_is_mean_sun_less_the_mean_of_mean_and_true() {
        // Mean Sun at 100.
        // Opposition-like: Mars mean = true = 280; 100 - 280 = -180 -> 180.
        let k = cheshta_kendra(Graha::Mars, 100.0, 280.0, 280.0).expect("a star graha");
        assert!((k - 180.0).abs() < 1e-12);
        assert!((cheshta_bala_from_kendra(k) - 60.0).abs() < 1e-12);
        // Conjunction-like: Mars at the mean Sun, mean = true: 0.
        let k = cheshta_kendra(Graha::Mars, 100.0, 100.0, 100.0).expect("a star graha");
        assert!(k.abs() < 1e-12);
        // Quarter: 100 - 190 = -90 -> 90 -> 30.
        let k = cheshta_kendra(Graha::Jupiter, 100.0, 190.0, 190.0).expect("a star graha");
        assert!((k - 90.0).abs() < 1e-12);
        assert!((cheshta_bala_from_kendra(k) - 30.0).abs() < 1e-12);
        // Mean 200, true 210 average to 205: 100 - 205 = -105 -> 105.
        let k = cheshta_kendra(Graha::Saturn, 100.0, 200.0, 210.0).expect("a star graha");
        assert!((k - 105.0).abs() < 1e-12);
        // More than six signs is taken from the circle: 270 -> 90.
        let k = cheshta_kendra(Graha::Mars, 100.0, 190.0, 190.0).expect("a star graha");
        assert!((k - 90.0).abs() < 1e-12);
        let k = cheshta_kendra(
            Graha::Mars,
            100.0,
            100.0 - 270.0 + 360.0,
            100.0 - 270.0 + 360.0,
        )
        .expect("a star graha");
        assert!((k - 90.0).abs() < 1e-9);
    }

    #[test]
    fn the_kendra_of_mercury_and_venus_swaps_the_roles() {
        // The mean Sun is their "mean", the heliocentric mean longitude is the
        // sighrocca: kendra = helio mean - mean(mean Sun, true).
        // Helio mean 200, mean Sun 100, true 110: 200 - 105 = 95.
        let k = cheshta_kendra(Graha::Mercury, 100.0, 200.0, 110.0).expect("a star graha");
        assert!((k - 95.0).abs() < 1e-12);
        let k = cheshta_kendra(Graha::Venus, 100.0, 200.0, 110.0).expect("a star graha");
        assert!((k - 95.0).abs() < 1e-12);
        // The same numbers for Mars read the other way round: 100 - 155 = -55.
        let m = cheshta_kendra(Graha::Mars, 100.0, 200.0, 110.0).expect("a star graha");
        assert!((m - 55.0).abs() < 1e-12);
    }

    #[test]
    fn the_mean_of_mean_and_true_is_taken_along_the_short_arc() {
        // Mars mean 359, true 3: they average to 1, not 181.
        let k = cheshta_kendra(Graha::Mars, 100.0, 359.0, 3.0).expect("a star graha");
        assert!((k - 99.0).abs() < 1e-9, "{k}");
        // And the other way round.
        let k = cheshta_kendra(Graha::Mars, 100.0, 3.0, 359.0).expect("a star graha");
        assert!((k - 99.0).abs() < 1e-9, "{k}");
    }

    #[test]
    fn the_luminaries_and_nodes_have_no_kendra() {
        for g in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
            assert_eq!(cheshta_kendra(g, 100.0, 100.0, 100.0), None);
            assert_eq!(cheshta_bala_at(g, 2_460_000.5, 100.0), None);
        }
    }

    /// The tropical longitude of a body from the engine, degrees, at a TT
    /// Julian Day.
    fn engine_longitude(body: vedaksha_ephem_core::bodies::Body, jd_tt: f64) -> f64 {
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::coordinates::ecliptic_position_tt;
        ecliptic_position_tt(&AnalyticalProvider::new(), body, jd_tt)
            .expect("position")
            .longitude
            .to_degrees()
            .rem_euclid(360.0)
    }

    /// The JD in `[lo, hi]` (steps of half a day) where `score` is least.
    fn argmin(lo: f64, hi: f64, score: impl Fn(f64) -> f64) -> f64 {
        let mut best = (f64::INFINITY, lo);
        let mut jd = lo;
        while jd <= hi {
            let s = score(jd);
            if s < best.0 {
                best = (s, jd);
            }
            jd += 0.5;
        }
        best.1
    }

    #[test]
    fn cheshta_bala_is_greatest_at_opposition_and_least_at_conjunction() {
        use vedaksha_ephem_core::bodies::Body;
        // Mars was at opposition in late August 2003 and in conjunction with
        // the Sun in mid 2004 or so; find both from the engine rather than
        // trusting a remembered date. elongation = angular distance from the Sun.
        let elongation = |jd: f64| {
            let d = (engine_longitude(Body::Mars, jd) - engine_longitude(Body::Sun, jd))
                .rem_euclid(360.0);
            if d > 180.0 { 360.0 - d } else { d }
        };
        let opposition = argmin(2_452_850.0, 2_452_910.0, |jd| 180.0 - elongation(jd));
        let conjunction = argmin(2_453_100.0, 2_453_300.0, elongation);
        assert!(elongation(opposition) > 175.0, "opposition at {opposition}");
        assert!(
            elongation(conjunction) < 3.0,
            "conjunction at {conjunction}"
        );

        let bala = |jd: f64| {
            cheshta_bala_at(Graha::Mars, jd, engine_longitude(Body::Mars, jd)).expect("Mars")
        };
        // Opposition: the mean Sun is opposite Mars, less half the equation
        // of centre (at most ~5 degrees) and the Sun's own (under 2).
        let at_opposition = bala(opposition);
        assert!(at_opposition > 54.0, "opposition bala {at_opposition}");
        // Conjunction: the kendra is only the equation terms.
        let at_conjunction = bala(conjunction);
        assert!(at_conjunction < 4.0, "conjunction bala {at_conjunction}");
    }

    #[test]
    fn venus_is_strongest_at_inferior_and_weakest_at_superior_conjunction() {
        use vedaksha_ephem_core::bodies::Body;
        // Venus's sighrocca is its heliocentric mean longitude: at inferior
        // conjunction it lies opposite the Sun (kendra near 180), at
        // superior conjunction with it (near 0). Found from the engine.
        let elongation = |jd: f64| {
            let d = (engine_longitude(Body::Venus, jd) - engine_longitude(Body::Sun, jd))
                .rem_euclid(360.0);
            if d > 180.0 { 360.0 - d } else { d }
        };
        // Inferior: the elongation minimum near 2020-06 (JD 2459000).
        let inferior = argmin(2_458_990.0, 2_459_030.0, elongation);
        // Superior: the minimum near 2021-03 (JD 2459300).
        let superior = argmin(2_459_270.0, 2_459_330.0, elongation);
        let bala = |jd: f64| {
            cheshta_bala_at(Graha::Venus, jd, engine_longitude(Body::Venus, jd)).expect("Venus")
        };
        let mean_sun = |jd: f64| mean_longitude(Graha::Sun, jd).0;
        // Sanity of the two events themselves: Venus is between the Earth
        // and Sun at the first (small elongation, retrograde, so the mean Sun
        // is ahead of nothing -- the kendra is what we test).
        assert!(elongation(inferior) < 8.0 && elongation(superior) < 3.0);
        assert!(bala(inferior) > 52.0, "inferior bala {}", bala(inferior));
        assert!(bala(superior) < 8.0, "superior bala {}", bala(superior));
        let _ = mean_sun;
    }

    // ── The complete form, end to end ────────────────────────────────

    fn moment() -> KalaMoment {
        KalaMoment {
            jd: 2_461_338.001,
            jd_sunrise: 2_461_337.75,
            jd_sunset: 2_461_338.25,
            jd_next_sunrise: 2_461_338.75,
            hindu_day_jdn: 2_461_338,
            apparent_hours_from_midnight: 12.0,
        }
    }

    fn context() -> ShadbalaContext {
        ShadbalaContext {
            jd_tt: 2_461_338.0,
            ayanamsha_deg: 24.2,
            moment: moment(),
        }
    }

    fn data(position: GrahaPosition, latitude_deg: f64) -> CompletePlanetData {
        // A declination from the longitude, so the declination reading has
        // something to read.
        let sayana = position.longitude + 24.2;
        let declination = (23.44_f64.to_radians().sin() * sayana.to_radians().sin())
            .asin()
            .to_degrees();
        CompletePlanetData {
            position,
            latitude_deg,
            declination_deg: Some(declination),
            benefic_aspect_count: 1,
            malefic_aspect_count: 0,
        }
    }

    fn chart_data() -> Vec<CompletePlanetData> {
        friendship_chart().iter().map(|&p| data(p, 0.0)).collect()
    }

    fn compute(planets: &[CompletePlanetData]) -> Vec<CompleteShadbala> {
        compute_shadbala_complete(planets, &context(), &ShadbalaOptions::default())
            .expect("a complete chart")
    }

    #[test]
    fn every_part_is_its_standalone_function() {
        let planets = chart_data();
        let results = compute(&planets);
        assert_eq!(results.len(), 7);
        let chart = friendship_chart();
        let (sun_lon, moon_lon) = (chart[0].longitude, chart[1].longitude);
        // The day: JDN 2461338 is a Saturday; birth 6.02 h after sunrise is
        // in the second third of the day (the Sun's) and in the 7th hora,
        // the Moon's (Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon).
        let ahargana = ahargana_from_jdn(2_461_338);
        let (vara, masa, abda) = (
            vara_lord(ahargana),
            masa_lord(ahargana),
            abda_lord(ahargana),
        );
        assert_eq!(vara, Graha::Saturn);
        for r in &results {
            let p = chart
                .iter()
                .find(|c| c.planet == r.planet)
                .expect("in chart");
            let lon = p.longitude;
            assert!((r.sthana.uccha - uccha_bala(r.planet, lon)).abs() < 1e-12);
            assert!(
                (r.sthana.saptavargaja
                    - saptavargaja_bala(
                        r.planet,
                        &chart,
                        MoolatrikonaScope::AllVargas,
                        SaptavargajaLadder::BPHS
                    )
                    .expect("complete"))
                .abs()
                    < 1e-12
            );
            assert!((r.sthana.ojhayugma - ojhayugma_bala(r.planet, lon)).abs() < 1e-12);
            assert!((r.sthana.kendradi - kendradi_bala(p.bhava)).abs() < 1e-12);
            assert!((r.sthana.drekkana - drekkana_bala(r.planet, lon)).abs() < 1e-12);
            let sthana_sum = r.sthana.uccha
                + r.sthana.saptavargaja
                + r.sthana.ojhayugma
                + r.sthana.kendradi
                + r.sthana.drekkana;
            assert!((r.sthana.total - sthana_sum).abs() < 1e-9);

            assert!((r.kala.nathonnata - nathonnata_bala(r.planet, 12.0)).abs() < 1e-12);
            assert!(
                (r.kala.paksha - paksha_bala(r.planet, moon_lon, sun_lon, &BPHS_PAKSHA_BENEFICS))
                    .abs()
                    < 1e-12
            );
            assert!((r.kala.tribhaga - tribhaga_bala(r.planet, Graha::Sun)).abs() < 1e-12);
            assert!((r.kala.abda - lord_weight(r.planet, abda, 15.0)).abs() < 1e-12);
            assert!((r.kala.masa - lord_weight(r.planet, masa, 30.0)).abs() < 1e-12);
            assert!((r.kala.vara - lord_weight(r.planet, Graha::Saturn, 45.0)).abs() < 1e-12);
            assert!((r.kala.hora - lord_weight(r.planet, Graha::Moon, 60.0)).abs() < 1e-12);
            assert!((r.kala.ayana - ayana_bala_khanda(r.planet, lon + 24.2)).abs() < 1e-12);
            assert!(r.kala.yuddha.abs() < 1e-12, "no war in this chart");
            let kala_sum = r.kala.nathonnata
                + r.kala.paksha
                + r.kala.tribhaga
                + r.kala.abda
                + r.kala.masa
                + r.kala.vara
                + r.kala.hora
                + r.kala.ayana
                + r.kala.yuddha;
            assert!((r.kala.total - kala_sum).abs() < 1e-9);

            assert!((r.dig_bala - dig_bala(r.planet, p.bhava)).abs() < 1e-12);
            assert!((r.naisargika_bala - naisargika_bala(r.planet)).abs() < 1e-12);
            assert!((r.drik_bala - drik_bala(1, 0)).abs() < 1e-12);
            let total = r.sthana.total
                + r.dig_bala
                + r.kala.total
                + r.cheshta_bala
                + r.naisargika_bala
                + r.drik_bala;
            assert!((r.total - total).abs() < 1e-9, "{:?}", r.planet);
            let (ishta, kashta) = ishta_kashta_phala(r.sthana.uccha, r.cheshta_bala);
            assert!((r.ishta_phala - ishta).abs() < 1e-12);
            assert!((r.kashta_phala - kashta).abs() < 1e-12);
        }
    }

    #[test]
    fn the_sun_and_moon_cheshta_is_their_ayana_and_paksha_undoubled() {
        let results = compute(&chart_data());
        let sun = results
            .iter()
            .find(|r| r.planet == Graha::Sun)
            .expect("Sun");
        let moon = results
            .iter()
            .find(|r| r.planet == Graha::Moon)
            .expect("Moon");
        // BPHS 27.18: the cheshta bala is the very same number as the part in
        // kala bala, each counted once in each place.
        assert!((sun.cheshta_bala - sun.kala.ayana).abs() < 1e-12);
        assert!((moon.cheshta_bala - moon.kala.paksha).abs() < 1e-12);
        // The Moon is in the benefic group: the pool for the 94-degree
        // elongation (Sun 126, Moon 32), 94/3, not twice it.
        assert!((moon.kala.paksha - 94.0 / 3.0).abs() < 1e-9);
        // The star grahas' cheshta is the kendra over 3, within 0..60.
        for r in &results {
            assert!((0.0..=60.0).contains(&r.cheshta_bala), "{:?}", r.planet);
        }
    }

    #[test]
    fn the_four_lords_and_the_third_each_take_their_weight_once() {
        let results = compute(&chart_data());
        let sum = |f: fn(&CompleteShadbala) -> f64| results.iter().map(f).sum::<f64>();
        assert!((sum(|r| r.kala.abda) - 15.0).abs() < 1e-12);
        assert!((sum(|r| r.kala.masa) - 30.0).abs() < 1e-12);
        assert!((sum(|r| r.kala.vara) - 45.0).abs() < 1e-12);
        assert!((sum(|r| r.kala.hora) - 60.0).abs() < 1e-12);
        // Jupiter always, and the lord of the third.
        assert!((sum(|r| r.kala.tribhaga) - 120.0).abs() < 1e-12);
    }

    #[test]
    fn a_war_moves_strength_between_the_two_and_only_the_yuddha_part() {
        // Mars at 5.5 degrees of Mesha. Control: Saturn 2 degrees from him, no
        // war. War: Saturn half a degree from him, and further north.
        let with_saturn_at = |saturn_lon: f64| {
            let mut planets = chart_data();
            for d in &mut planets {
                match d.position.planet {
                    Graha::Mars => d.latitude_deg = 1.0,
                    Graha::Saturn => {
                        d.position = gp(Graha::Saturn, saturn_lon, 7);
                        d.latitude_deg = 2.0;
                        d.declination_deg = Some(0.0);
                    }
                    _ => {}
                }
            }
            compute(&planets)
        };
        let control = with_saturn_at(7.5);
        let warred = with_saturn_at(6.0);
        let pick =
            |rs: &[CompleteShadbala], g: Graha| *rs.iter().find(|r| r.planet == g).expect("g");

        for r in &control {
            assert!(r.kala.yuddha.abs() < 1e-12, "{:?} in the control", r.planet);
        }
        let (mars, saturn) = (pick(&warred, Graha::Mars), pick(&warred, Graha::Saturn));
        // Saturn is the northerner and wins; the victor gains exactly what
        // the vanquished loses.
        assert!(saturn.kala.yuddha > 0.0 && mars.kala.yuddha < 0.0);
        assert!((saturn.kala.yuddha + mars.kala.yuddha).abs() < 1e-9);
        // The size is the difference of the two shadbalas as they stood
        // before the war.
        let pre_mars = mars.total - mars.kala.yuddha;
        let pre_saturn = saturn.total - saturn.kala.yuddha;
        assert!((saturn.kala.yuddha - (pre_mars - pre_saturn).abs()).abs() < 1e-9);
        // Mars's every part but the yuddha is what it was in the control
        // (Saturn stays in Mesha, so nothing of Mars's friendships moves), and
        // so is every graha that was not in the war.
        let mars0 = pick(&control, Graha::Mars);
        assert!((pre_mars - mars0.total).abs() < 1e-9);
        assert!((mars.kala.total - mars.kala.yuddha - mars0.kala.total).abs() < 1e-9);
        assert!((mars.sthana.total - mars0.sthana.total).abs() < 1e-9);
        assert!((mars.cheshta_bala - mars0.cheshta_bala).abs() < 1e-9);
        for g in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
        ] {
            let (a, b) = (pick(&warred, g), pick(&control, g));
            assert!((a.total - b.total).abs() < 1e-9, "{g:?}");
            assert!(a.kala.yuddha.abs() < 1e-12, "{g:?}");
        }
    }

    #[test]
    fn the_options_change_exactly_what_they_say() {
        let planets = chart_data();
        let base = compute(&planets);
        let by = |rs: &[CompleteShadbala], g: Graha| *rs.iter().find(|r| r.planet == g).expect("g");
        let ctx = context();

        // Drekkana order: only the neuter and female grahas can move, by 15.
        let swapped = compute_shadbala_complete(
            &planets,
            &ctx,
            &ShadbalaOptions {
                drekkana: DrekkanaOrder::MaleFemaleNeuter,
                ..ShadbalaOptions::default()
            },
        )
        .expect("ok");
        for g in [Graha::Sun, Graha::Mars, Graha::Jupiter] {
            assert!(
                (by(&swapped, g).total - by(&base, g).total).abs() < 1e-9,
                "{g:?}"
            );
        }

        // Moolatrikona scope: saptavargaja only.
        let rashi = compute_shadbala_complete(
            &planets,
            &ctx,
            &ShadbalaOptions {
                moolatrikona: MoolatrikonaScope::RashiOnly,
                ..ShadbalaOptions::default()
            },
        )
        .expect("ok");
        let sun_all = by(&base, Graha::Sun);
        let sun_rashi = by(&rashi, Graha::Sun);
        assert!((sun_all.sthana.saptavargaja - sun_rashi.sthana.saptavargaja - 30.0).abs() < 1e-9);
        assert!((sun_all.kala.total - sun_rashi.kala.total).abs() < 1e-9);

        // Benefic set: leave Mercury out and his paksha is the complement.
        let benefics = [Graha::Moon, Graha::Venus, Graha::Jupiter];
        let custom = compute_shadbala_complete(
            &planets,
            &ctx,
            &ShadbalaOptions {
                paksha_benefics: &benefics,
                ..ShadbalaOptions::default()
            },
        )
        .expect("ok");
        let pool = 94.0 / 3.0;
        assert!((by(&base, Graha::Mercury).kala.paksha - pool).abs() < 1e-9);
        assert!((by(&custom, Graha::Mercury).kala.paksha - (60.0 - pool)).abs() < 1e-9);

        // Hora division: the 7th equal hora is the Moon's; the day is 12
        // hours and the night 12, so the unequal division here is the same.
        let unequal = compute_shadbala_complete(
            &planets,
            &ctx,
            &ShadbalaOptions {
                hora: HoraKind::Unequal,
                ..ShadbalaOptions::default()
            },
        )
        .expect("ok");
        assert!((by(&unequal, Graha::Moon).kala.hora - 60.0).abs() < 1e-9);

        // The declination reading changes ayana only.
        let decl = compute_shadbala_complete(
            &planets,
            &ctx,
            &ShadbalaOptions {
                ayana: AyanaReading::Declination,
                ..ShadbalaOptions::default()
            },
        )
        .expect("ok");
        let (a, b) = (by(&base, Graha::Moon), by(&decl, Graha::Moon));
        assert!((a.sthana.total - b.sthana.total).abs() < 1e-12);
        assert!((a.kala.paksha - b.kala.paksha).abs() < 1e-12);
        let moon_declination = planets[1].declination_deg.expect("set");
        assert!(
            (b.kala.ayana - ayana_bala_declination(Graha::Moon, moon_declination)).abs() < 1e-12
        );
    }

    #[test]
    fn the_complete_form_refuses_what_it_cannot_compute() {
        let planets = chart_data();
        let ctx = context();
        let opts = ShadbalaOptions::default();

        // A classical graha missing.
        let missing: Vec<_> = planets
            .iter()
            .copied()
            .filter(|d| d.position.planet != Graha::Venus)
            .collect();
        assert_eq!(
            compute_shadbala_complete(&missing, &ctx, &opts).unwrap_err(),
            ShadbalaError::MissingGraha(Graha::Venus)
        );

        // The birth before its sunrise.
        let mut early = ctx;
        early.moment.jd = early.moment.jd_sunrise - 0.01;
        assert_eq!(
            compute_shadbala_complete(&planets, &early, &opts).unwrap_err(),
            ShadbalaError::BadMoment
        );
        // Sunset after the next sunrise.
        let mut disordered = ctx;
        disordered.moment.jd_sunset = disordered.moment.jd_next_sunrise + 0.1;
        assert_eq!(
            compute_shadbala_complete(&planets, &disordered, &opts).unwrap_err(),
            ShadbalaError::BadMoment
        );
        let mut nan = ctx;
        nan.moment.apparent_hours_from_midnight = f64::NAN;
        assert_eq!(
            compute_shadbala_complete(&planets, &nan, &opts).unwrap_err(),
            ShadbalaError::BadMoment
        );

        // The declination reading without a declination.
        let mut no_decl = planets.clone();
        no_decl[3].declination_deg = None; // Mercury
        let declination = ShadbalaOptions {
            ayana: AyanaReading::Declination,
            ..ShadbalaOptions::default()
        };
        assert_eq!(
            compute_shadbala_complete(&no_decl, &ctx, &declination).unwrap_err(),
            ShadbalaError::MissingDeclination(Graha::Mercury)
        );
        // ...which the verse's reading never needs.
        assert!(compute_shadbala_complete(&no_decl, &ctx, &opts).is_ok());

        // Rahu and Ketu in the input are ignored, not an error.
        let mut with_nodes = planets.clone();
        with_nodes.push(data(gp(Graha::Rahu, 10.0, 1), 0.0));
        with_nodes.push(data(gp(Graha::Ketu, 190.0, 7), 0.0));
        assert_eq!(
            compute_shadbala_complete(&with_nodes, &ctx, &opts)
                .expect("ok")
                .len(),
            7
        );
        // The message names the graha.
        assert!(format!("{}", ShadbalaError::MissingGraha(Graha::Venus)).contains("Venus"));
    }

    #[test]
    fn the_coarse_form_is_untouched() {
        // The MCP and WASM surfaces serve compute_shadbala_full; its answers
        // are the 0-or-30 steps they always were.
        let data = [planet_data(Graha::Jupiter, 3, 4, -0.05, 0.08, 2, 1)];
        let sb = &compute_shadbala_full(&data, true, true)[0];
        assert!((sb.kala_bala - 60.0).abs() < f64::EPSILON);
        assert!((sb.cheshta_bala - 60.0).abs() < f64::EPSILON);
        let sb = &compute_shadbala_full(&data, false, false)[0];
        // Night and waning, Jupiter a benefic: nathonnata 0, paksha 0.
        assert!(sb.kala_bala.abs() < f64::EPSILON);
    }
}
