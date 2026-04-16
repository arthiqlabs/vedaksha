// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Nakshatra (lunar mansion) definitions and utilities.
//!
//! 27 nakshatras divide 360° of the sidereal zodiac into equal 13°20'
//! (13.3333°) segments. Each nakshatra has 4 padas of 3°20' (3.3333°) each.
//!
//! Source: BPHS (Brihat Parashara Hora Shastra);
//! B.V. Raman, "Hindu Predictive Astrology".

use serde::{Deserialize, Serialize};

/// The 27 Vedic nakshatras (lunar mansions).
///
/// Each spans exactly 13°20' (13.3333°) of the sidereal zodiac.
/// Source: BPHS; B.V. Raman, "Hindu Predictive Astrology".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Nakshatra {
    Ashwini = 0,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Moola,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishta,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

/// Planet that rules a Vimshottari dasha period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DashaLord {
    Sun,
    Moon,
    Mars,
    Rahu,
    Jupiter,
    Saturn,
    Mercury,
    Ketu,
    Venus,
}

impl DashaLord {
    /// Total years for this lord's Maha Dasha period.
    /// Sum of all 9 = 120 years.
    #[must_use]
    pub const fn maha_dasha_years(&self) -> f64 {
        match self {
            Self::Sun => 6.0,
            Self::Moon => 10.0,
            Self::Mars | Self::Ketu => 7.0,
            Self::Rahu => 18.0,
            Self::Jupiter => 16.0,
            Self::Saturn => 19.0,
            Self::Mercury => 17.0,
            Self::Venus => 20.0,
        }
    }
}

/// Three gunas (primary qualities) from Samkhya philosophy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guna {
    Sattva,
    Rajas,
    Tamas,
}

/// Temperament (nature) of a nakshatra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gana {
    Deva,
    Manushya,
    Rakshasa,
}

/// Yoni (animal symbol) for Ashtakoota compatibility matching.
/// Source: BPHS Ch. 20.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Yoni {
    Horse,
    Elephant,
    Goat,
    Serpent,
    Dog,
    Cat,
    Rat,
    Cow,
    Buffalo,
    Tiger,
    Deer,
    Monkey,
    Mongoose,
    Lion,
}

/// Yoni gender for compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YoniGender {
    Male,
    Female,
}

/// Nadi (pulse/constitution) for Ashtakoota matching.
/// Nadi dosha is the highest-weighted koota (8 points).
/// Source: BPHS Ch. 20.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nadi {
    Aadi,
    Madhya,
    Antya,
}

impl Nakshatra {
    /// Span of each nakshatra in degrees (13°20').
    pub const SPAN: f64 = 13.333_333_333_333_334;

    /// Span of each pada in degrees (3°20').
    pub const PADA_SPAN: f64 = 3.333_333_333_333_333;

    /// Get the nakshatra for a sidereal longitude (degrees).
    ///
    /// Uses `>=` for lower bound, `<` for upper bound (standard BPHS convention).
    #[must_use]
    pub fn from_longitude(sidereal_lon_deg: f64) -> Self {
        let normalized = vedaksha_math::angle::normalize_degrees(sidereal_lon_deg);
        let raw = normalized / Self::SPAN;
        // Epsilon guard: when the raw quotient is within ε of an integer,
        // snap to that integer. This makes boundary assignment deterministic
        // regardless of upstream floating-point accumulation.
        // Convention: [lower, upper) — exact boundaries start the NEXT nakshatra.
        // So snapping 12.9999999999 → 13 puts it in Bharani (index 1), and
        // snapping 13.0000000001 → 13 also puts it in Bharani.
        // 1e-10° ≈ 0.0000004 arcseconds — well below any ephemeris precision.
        let snapped = if (raw - raw.round()).abs() < 1e-10 {
            raw.round()
        } else {
            raw
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = (snapped.floor() as u8) % 27;
        Self::from_index(index)
    }

    /// Get the pada (1–4) for a sidereal longitude.
    ///
    /// BPHS convention: exact pada boundaries (3°20', 6°40', 10°) close
    /// the preceding pada. So 10.0° within a nakshatra → pada 3 (not 4).
    #[must_use]
    pub fn pada_from_longitude(sidereal_lon_deg: f64) -> u8 {
        let normalized = vedaksha_math::angle::normalize_degrees(sidereal_lon_deg);
        let within_nakshatra = normalized % Self::SPAN;
        // Epsilon guard: subtract ε so values at exact boundaries belong
        // to the preceding pada. 1e-9° ≈ 0.004 arcseconds — well below
        // any ephemeris precision, consistent with BPHS boundary convention.
        let adjusted = (within_nakshatra - 1e-9).max(0.0);
        let raw = adjusted / Self::PADA_SPAN;
        // Additional epsilon snap: if raw is within 1e-10 of an integer
        // after the adjustment, snap to floor deterministically.
        let snapped = if (raw - raw.round()).abs() < 1e-10 {
            raw.round()
        } else {
            raw
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pada_index = (snapped.floor() as u8) % 4;
        pada_index + 1
    }

    /// Create from 0-based index (0 = Ashwini … 26 = Revati).
    ///
    /// # Panics
    ///
    /// Panics if `index > 26`.
    #[must_use]
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Ashwini,
            1 => Self::Bharani,
            2 => Self::Krittika,
            3 => Self::Rohini,
            4 => Self::Mrigashira,
            5 => Self::Ardra,
            6 => Self::Punarvasu,
            7 => Self::Pushya,
            8 => Self::Ashlesha,
            9 => Self::Magha,
            10 => Self::PurvaPhalguni,
            11 => Self::UttaraPhalguni,
            12 => Self::Hasta,
            13 => Self::Chitra,
            14 => Self::Swati,
            15 => Self::Vishakha,
            16 => Self::Anuradha,
            17 => Self::Jyeshtha,
            18 => Self::Moola,
            19 => Self::PurvaAshadha,
            20 => Self::UttaraAshadha,
            21 => Self::Shravana,
            22 => Self::Dhanishta,
            23 => Self::Shatabhisha,
            24 => Self::PurvaBhadrapada,
            25 => Self::UttaraBhadrapada,
            26 => Self::Revati,
            _ => panic!("nakshatra index out of range: {index}"),
        }
    }

    /// 0-based index of this nakshatra.
    #[must_use]
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// Sanskrit transliterated name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ashwini => "Ashwini",
            Self::Bharani => "Bharani",
            Self::Krittika => "Krittika",
            Self::Rohini => "Rohini",
            Self::Mrigashira => "Mrigashira",
            Self::Ardra => "Ardra",
            Self::Punarvasu => "Punarvasu",
            Self::Pushya => "Pushya",
            Self::Ashlesha => "Ashlesha",
            Self::Magha => "Magha",
            Self::PurvaPhalguni => "Purva Phalguni",
            Self::UttaraPhalguni => "Uttara Phalguni",
            Self::Hasta => "Hasta",
            Self::Chitra => "Chitra",
            Self::Swati => "Swati",
            Self::Vishakha => "Vishakha",
            Self::Anuradha => "Anuradha",
            Self::Jyeshtha => "Jyeshtha",
            Self::Moola => "Moola",
            Self::PurvaAshadha => "Purva Ashadha",
            Self::UttaraAshadha => "Uttara Ashadha",
            Self::Shravana => "Shravana",
            Self::Dhanishta => "Dhanishta",
            Self::Shatabhisha => "Shatabhisha",
            Self::PurvaBhadrapada => "Purva Bhadrapada",
            Self::UttaraBhadrapada => "Uttara Bhadrapada",
            Self::Revati => "Revati",
        }
    }

    /// Vimshottari dasha lord for this nakshatra.
    ///
    /// The cycle of 9 lords repeats every 9 nakshatras:
    /// Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury.
    #[must_use]
    pub fn dasha_lord(&self) -> DashaLord {
        const LORDS: [DashaLord; 9] = [
            DashaLord::Ketu,
            DashaLord::Venus,
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Mars,
            DashaLord::Rahu,
            DashaLord::Jupiter,
            DashaLord::Saturn,
            DashaLord::Mercury,
        ];
        LORDS[(self.index() % 9) as usize]
    }

    /// Guna (primary quality) of this nakshatra.
    ///
    /// Source: BPHS standard assignment.
    #[must_use]
    pub fn guna(&self) -> Guna {
        match self {
            // Sattva group
            Self::Ashwini
            | Self::Krittika
            | Self::Punarvasu
            | Self::Pushya
            | Self::Hasta
            | Self::Swati
            | Self::Anuradha
            | Self::PurvaAshadha
            | Self::UttaraBhadrapada => Guna::Sattva,
            // Rajas group
            Self::Bharani
            | Self::Rohini
            | Self::Ardra
            | Self::Ashlesha
            | Self::Chitra
            | Self::Vishakha
            | Self::Jyeshtha
            | Self::UttaraAshadha
            | Self::Revati => Guna::Rajas,
            // Tamas group
            Self::Mrigashira
            | Self::PurvaPhalguni
            | Self::UttaraPhalguni
            | Self::Magha
            | Self::Moola
            | Self::Shravana
            | Self::Dhanishta
            | Self::Shatabhisha
            | Self::PurvaBhadrapada => Guna::Tamas,
        }
    }

    /// Gana (temperament) of this nakshatra.
    ///
    /// Source: BPHS.
    #[must_use]
    pub fn gana(&self) -> Gana {
        match self {
            // Deva (divine)
            Self::Ashwini
            | Self::Mrigashira
            | Self::Punarvasu
            | Self::Pushya
            | Self::Hasta
            | Self::Swati
            | Self::Anuradha
            | Self::Shravana
            | Self::Revati => Gana::Deva,
            // Manushya (human)
            Self::Bharani
            | Self::Rohini
            | Self::Ardra
            | Self::PurvaPhalguni
            | Self::UttaraPhalguni
            | Self::PurvaAshadha
            | Self::UttaraAshadha
            | Self::PurvaBhadrapada
            | Self::UttaraBhadrapada => Gana::Manushya,
            // Rakshasa (demonic)
            Self::Krittika
            | Self::Ashlesha
            | Self::Magha
            | Self::Chitra
            | Self::Vishakha
            | Self::Jyeshtha
            | Self::Moola
            | Self::Dhanishta
            | Self::Shatabhisha => Gana::Rakshasa,
        }
    }

    /// Start longitude of this nakshatra in sidereal degrees.
    #[must_use]
    pub fn start_longitude(&self) -> f64 {
        f64::from(self.index()) * Self::SPAN
    }

    /// End longitude of this nakshatra in sidereal degrees.
    #[must_use]
    pub fn end_longitude(&self) -> f64 {
        (f64::from(self.index()) + 1.0) * Self::SPAN
    }

    /// Presiding deity (Adhidevata) per BPHS Ch. 3.
    #[must_use]
    pub fn deity(&self) -> &'static str {
        const DEITIES: [&str; 27] = [
            "Ashwini Kumaras",
            "Yama",
            "Agni",
            "Brahma",
            "Soma",
            "Rudra",
            "Aditi",
            "Brihaspati",
            "Sarpas",
            "Pitrs",
            "Bhaga",
            "Aryaman",
            "Savitar",
            "Tvashtar",
            "Vayu",
            "Indragni",
            "Mitra",
            "Indra",
            "Nirrti",
            "Apas",
            "Vishvedeva",
            "Vishnu",
            "Vasu",
            "Varuna",
            "Ajaikapada",
            "Ahirbudhnya",
            "Pushan",
        ];
        DEITIES[*self as usize]
    }

    /// Yoni (animal symbol) and gender for Ashtakoota matching.
    /// Source: BPHS Ch. 20.
    #[must_use]
    pub fn yoni(&self) -> (Yoni, YoniGender) {
        use Yoni::*;
        use YoniGender::*;
        const YONIS: [(Yoni, YoniGender); 27] = [
            (Horse, Male),
            (Elephant, Male),
            (Goat, Female),
            (Serpent, Male),
            (Serpent, Female),
            (Dog, Female),
            (Cat, Female),
            (Goat, Male),
            (Cat, Male),
            (Rat, Male),
            (Rat, Female),
            (Cow, Male),
            (Buffalo, Female),
            (Tiger, Female),
            (Buffalo, Male),
            (Tiger, Male),
            (Deer, Female),
            (Deer, Male),
            (Dog, Male),
            (Monkey, Male),
            (Mongoose, Male),
            (Monkey, Female),
            (Lion, Female),
            (Horse, Female),
            (Lion, Male),
            (Cow, Female),
            (Elephant, Female),
        ];
        YONIS[*self as usize]
    }

    /// Nadi (pulse/constitution) for Ashtakoota matching.
    /// Cycles: Aadi, Madhya, Antya through the 27 nakshatras.
    /// Source: BPHS Ch. 20.
    #[must_use]
    pub fn nadi(&self) -> Nadi {
        match *self as u8 % 3 {
            0 => Nadi::Aadi,
            1 => Nadi::Madhya,
            _ => Nadi::Antya,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- from_longitude ---

    #[test]
    fn ashwini_at_0_degrees() {
        assert_eq!(Nakshatra::from_longitude(0.0), Nakshatra::Ashwini);
    }

    #[test]
    fn bharani_at_15_degrees() {
        assert_eq!(Nakshatra::from_longitude(15.0), Nakshatra::Bharani);
    }

    #[test]
    fn revati_at_359_degrees() {
        assert_eq!(Nakshatra::from_longitude(359.0), Nakshatra::Revati);
    }

    #[test]
    fn boundary_at_13_333_is_bharani() {
        // 13.333... is the exact boundary; index = 1 → Bharani
        assert_eq!(
            Nakshatra::from_longitude(Nakshatra::SPAN),
            Nakshatra::Bharani
        );
    }

    // --- pada_from_longitude ---

    #[test]
    fn pada_at_0_is_1() {
        assert_eq!(Nakshatra::pada_from_longitude(0.0), 1);
    }

    #[test]
    fn pada_at_3_34_is_2() {
        assert_eq!(Nakshatra::pada_from_longitude(3.34), 2);
    }

    #[test]
    fn pada_at_10_is_3() {
        // 10.0 / 3.333... = 3.0 → pada index 3, +1 = 4? Let's check:
        // within_nakshatra = 10.0 % 13.333 = 10.0
        // 10.0 / 3.333... = 3.0 → as u8 = 3, +1 = 4
        // Actually 10.0 / 3.333... = 3.00000..., u8 truncation = 3, pada = 4
        // The spec says pada at 10° = 3. Let's verify: 10/3.333 = 2.9999... → 2+1=3
        // 10.0 / (10.0/3.0) = 3.0 exactly, but PADA_SPAN = 10/3 = 3.3333...
        // 10.0 / 3.3333... = 2.99997... → truncate to 2 → pada 3. Correct.
        assert_eq!(Nakshatra::pada_from_longitude(10.0), 3);
    }

    #[test]
    fn pada_at_13_is_4() {
        // within = 13.0 % 13.333... = 13.0
        // 13.0 / 3.333... = 3.9000... → truncate to 3 → pada 4
        assert_eq!(Nakshatra::pada_from_longitude(13.0), 4);
    }

    // --- start/end longitudes ---

    #[test]
    fn all_27_start_end_longitudes() {
        for i in 0u8..27 {
            let n = Nakshatra::from_index(i);
            let expected_start = f64::from(i) * Nakshatra::SPAN;
            let expected_end = f64::from(i + 1) * Nakshatra::SPAN;
            assert!(
                (n.start_longitude() - expected_start).abs() < EPS,
                "start mismatch for {i}: {} vs {}",
                n.start_longitude(),
                expected_start
            );
            assert!(
                (n.end_longitude() - expected_end).abs() < EPS,
                "end mismatch for {i}: {} vs {}",
                n.end_longitude(),
                expected_end
            );
        }
    }

    // --- dasha lords ---

    #[test]
    fn dasha_lord_ashwini_is_ketu() {
        assert_eq!(Nakshatra::Ashwini.dasha_lord(), DashaLord::Ketu);
    }

    #[test]
    fn dasha_lord_bharani_is_venus() {
        assert_eq!(Nakshatra::Bharani.dasha_lord(), DashaLord::Venus);
    }

    #[test]
    fn dasha_lord_krittika_is_sun() {
        assert_eq!(Nakshatra::Krittika.dasha_lord(), DashaLord::Sun);
    }

    #[test]
    fn dasha_lord_magha_is_ketu_repeating_cycle() {
        // Magha index = 9; 9 % 9 = 0 → Ketu (same as Ashwini)
        assert_eq!(Nakshatra::Magha.dasha_lord(), DashaLord::Ketu);
    }

    // --- maha dasha years sum ---

    #[test]
    fn all_maha_dasha_years_sum_to_120() {
        let lords = [
            DashaLord::Sun,
            DashaLord::Moon,
            DashaLord::Mars,
            DashaLord::Rahu,
            DashaLord::Jupiter,
            DashaLord::Saturn,
            DashaLord::Mercury,
            DashaLord::Ketu,
            DashaLord::Venus,
        ];
        let total: f64 = lords.iter().map(|l| l.maha_dasha_years()).sum();
        assert!((total - 120.0).abs() < EPS, "sum was {total}");
    }

    // --- gana ---

    #[test]
    fn gana_ashwini_is_deva() {
        assert_eq!(Nakshatra::Ashwini.gana(), Gana::Deva);
    }

    #[test]
    fn gana_bharani_is_manushya() {
        assert_eq!(Nakshatra::Bharani.gana(), Gana::Manushya);
    }

    #[test]
    fn gana_krittika_is_rakshasa() {
        assert_eq!(Nakshatra::Krittika.gana(), Gana::Rakshasa);
    }

    // --- deity ---

    #[test]
    fn ashwini_deity_is_ashwini_kumaras() {
        assert_eq!(Nakshatra::Ashwini.deity(), "Ashwini Kumaras");
    }

    #[test]
    fn revati_deity_is_pushan() {
        assert_eq!(Nakshatra::Revati.deity(), "Pushan");
    }

    #[test]
    fn all_27_have_deities() {
        for i in 0..27 {
            let n = Nakshatra::from_index(i);
            assert!(!n.deity().is_empty());
        }
    }

    // --- yoni ---

    #[test]
    fn ashwini_yoni_is_horse_male() {
        assert_eq!(Nakshatra::Ashwini.yoni(), (Yoni::Horse, YoniGender::Male));
    }

    // --- nadi ---

    #[test]
    fn nadi_cycles_through_27() {
        assert_eq!(Nakshatra::Ashwini.nadi(), Nadi::Aadi);
        assert_eq!(Nakshatra::Bharani.nadi(), Nadi::Madhya);
        assert_eq!(Nakshatra::Krittika.nadi(), Nadi::Antya);
        assert_eq!(Nakshatra::Rohini.nadi(), Nadi::Aadi);
    }
}
