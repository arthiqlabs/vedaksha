// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Chara (Jaimini) Dasha — sign-based variable-period system.
//!
//! Chara Dasha is one of the most important conditional dashas from Jaimini
//! astrology. Unlike planetary dashas, Chara assigns periods to signs (rashis).
//! Each sign gets a period of years counted from itself to the sign its lord
//! **actually occupies in the natal chart** — this is chart-dependent, not a
//! fixed table.
//!
//! **Superseded (2026-08):** earlier versions of this module used
//! `sign_lord_sign`, a hardcoded 12-entry table mapping each sign to its
//! lord's *other* domicile (e.g. Aries -> Mars -> Scorpio), with no chart
//! input at all. Every chart ever computed by that code therefore produced
//! identical Chara durations regardless of the actual positions of the
//! grahas. That table has zero attestation in any source consulted; the
//! attested rule is chart-dependent and is implemented below. See
//! `DATA_PROVENANCE.md` for the fix history.
//!
//! **Duration rule.** For a sign whose lord occupies a different sign, the
//! duration is the count of signs traversed getting there — an inclusive
//! count from the sign to the lord's sign, minus one — in a direction
//! determined per-sign (see below). A sign whose lord occupies the sign
//! itself gets **12 years**, not 0 and not 1.
//!
//! Attestation: Jaimini Upadesha Sutras A1P1, sutra *nathantah samah
//! prayena*, whose Sanskrit commentary glosses it *svasvadhipashritarashi*
//! ("the sign occupied by its own lord"); independently corroborated by
//! Brihat Parashara Hora Shastra ch. 46 ("reckoned from the Rasi up to the
//! house in which its lord is posited" — verse numbering for this passage is
//! unstable across recensions, hence cited by chapter and description rather
//! than a bare number). Grade (a).
//!
//! **Per-sign counting direction — do not conflate with [`chara_direction`].**
//! [`chara_direction`] decides one global thing: which way the *sequence* of
//! twelve dasha periods progresses, tested once at the 9th sign from the
//! lagna. The rule here is a *different* direction decision, made twelve
//! times independently — once per sign, from **that sign's own** pada
//! grouping, in threes from Aries:
//!
//! - Aries, Taurus, Gemini -> forward (zodiacal)
//! - Cancer, Leo, Virgo -> reverse
//! - Libra, Scorpio, Sagittarius -> forward
//! - Capricorn, Aquarius, Pisces -> reverse
//!
//! The forward set {Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius} is
//! identical to the vishama-pada set already defined by [`is_vishama_pada`]
//! for the (unrelated) sequence-direction rule — reused here for its
//! membership only, applied to a different subject (the sign being timed,
//! not the 9th-from-lagna sign).
//!
//! **Dual lordship.** Scorpio has co-lords Mars and Ketu; Aquarius has
//! co-lords Saturn and Rahu (attested by a *vriddha-karika* and by Brihat
//! Parashara Hora Shastra independently; the polarity — Rahu with Aquarius,
//! not Scorpio — is frequently inverted in secondary sources, so it is
//! stated explicitly here). Resolution, in the order below:
//!
//! 1. Both co-lords occupy the sign itself -> 12 years.
//! 2. One co-lord occupies the sign itself, the other does not -> ignore the
//!    one in the sign, count to the other.
//! 3. Both occupy some other sign -> count to the **stronger** by this
//!    cascade:
//!    1. conjoined with another graha beats not conjoined;
//!    2. if both conjoined, more conjoined grahas wins;
//!    3. the co-lord giving the **larger** duration wins;
//!    4. rashi-bala of the co-lord's occupied sign: dual > fixed > movable;
//!    5. an exalted co-lord wins.
//!
//! **Cascade ordering is not the textually stated order** — see below.
//!
//! **The cascade ordering was determined empirically against Brihat
//! Parashara Hora Shastra ch. 46's own worked Aquarius-lagna example, which
//! gives all twelve durations** (encoded as [`tests::bphs_46_worked_example`]
//! below). That chart's two dual-lord signs constrain the cascade:
//!
//! - **Aquarius -> 8 years**, which requires counting to Rahu (in Gemini),
//!   not Saturn (in Leo). Rahu is conjoined with Moon in Gemini; Saturn is
//!   alone in Leo. Step 1 (conjunction) alone explains this.
//! - **Scorpio -> 6 years**, which requires counting to Mars (in Taurus),
//!   not Ketu (in Sagittarius). Neither is conjoined with anything, so steps
//!   1-2 tie. The stated textual order would apply rashi-bala next and
//!   favour Ketu (Sagittarius is a dual sign, Taurus is fixed) — but the
//!   worked example needs Mars. The "larger duration wins" step, applied
//!   *before* rashi-bala, gives Mars (6 years, vs. Ketu's 1) and reproduces
//!   the example.
//!
//! So the textual source lists rashi-bala before the duration comparison,
//! but the worked example only reproduces if duration is compared **first**,
//! ahead of rashi-bala. That is the ordering implemented here: conjunction,
//! conjunction-count, duration, rashi-bala, exaltation. This makes step 3
//! (duration) and step 5 (exaltation) — moved earlier and unresolved by any
//! prose source respectively — the weakest-attested links: partly grade (d),
//! not (a). The BPHS worked example never reaches step 4 or 5 for either
//! dual-lord sign, so exaltation-of-Rahu/Ketu (itself a contested point
//! across traditions) is implemented but untested by the fixture.
//!
//! **Deliberately NOT implemented: the exaltation/debilitation +-1 year
//! adjustment.** A well-attested classical rule (a Brihat Parashara Hora
//! Shastra verse plus a *vriddha-karika*) adds one year to a sign's duration
//! when its lord is exalted, and subtracts one when the lord is debilitated.
//! Amit has decided not to implement it here, following a named
//! 20th-century teaching lineage that dropped the adjustment precisely
//! because it can produce a 0-year dasha (Sagittarius, when Jupiter is in
//! Capricorn) or a 13-year dasha (Virgo, when Mercury is in Virgo) — both
//! outside the system's own stated 1-12 year range. This is a deliberate,
//! cited omission of an attested rule, not an oversight. It is a different
//! rule from the exaltation tiebreak in the dual-lordship cascade above
//! (step 5), which is retained.
//!
//! **Direction — tested at the 9th sign from the lagna, not the lagna itself.**
//! The governing sutra is in Adhyaya 2, Pada 3 (its own number is unstable
//! across sources — direct readings of 22, 28, and 32 are on record, and a
//! fourth source independently reports 29): *panchame padakramat
//! prakpratyaktvam charadashayam*. Grammatically, *panchame* means "in the
//! fifth" — but Jaimini states his own letter-to-numeral convention
//! elsewhere (*sarvatra savarna bhava rashayash cha*, "everywhere, the
//! houses and signs are \[denoted\] by letters"), under which *panchame*
//! decodes arithmetically to **nine**, not five. That decoding is not
//! asserted here on trust — the cipher and an eight-word check are recorded
//! in full at
//! `docs/audit/2026-08-31-chara-dasha-panchame-cipher.md`
//! (high confidence — a verifiable arithmetic fact, not a judgment call).
//! The sutra is read here as: take the 9th sign from the lagna, and test it
//! (not the lagna itself) for direction.
//!
//! **The classification tested at that 9th sign: vishama-pada ("odd-footed")
//! vs. sama-pada ("even-footed")** — Aries, Taurus, Gemini, Libra, Scorpio,
//! Sagittarius are vishama-pada; the other six are sama-pada. This is a
//! distinct classification from plain odd/even sign numbering (it agrees
//! with plain odd/even on eight signs and disagrees on exactly Taurus, Leo,
//! Scorpio, Aquarius — the four signs a still-earlier version of this module
//! called out as a "fixed-sign exception" to a plain odd/even rule tested
//! directly on the lagna, before the 9th-house step was found; see
//! `DATA_PROVENANCE.md` Fix 11 for that history). If the 9th sign from the
//! lagna is vishama-pada, Chara Dasha counts forward; otherwise backward.
//!
//! **Attribution and confidence:** this reading is transmitted in a
//! commentary called the Subodhini, under the name Neelakantha — moderate
//! confidence on the specific authorship, since a critical edition compiled
//! from multiple manuscripts notes that copies circulating under that name
//! are sometimes misattributed. The same critical edition describes the
//! 9th-house reading as the view of "many commentators" — a documented
//! majority, not a claim of universal agreement — and separately records a
//! named minority reading, under the name Krishnananda Sarasvati, holding
//! that *panchame* keeps its literal sense and names a different dasha
//! system entirely, not a rival Chara Dasha direction rule. No source found
//! states a rival *Chara Dasha* direction table under any other name.
//!
//! **Superseded:** earlier versions of this module cited Adhyaya 1 Pada 1,
//! sutras 25-27, for direction. Those sutras are understood here to govern a
//! different rule — sign-to-lord counting, the concern of the duration
//! computation above, not sequence direction — and are no longer cited for
//! direction. See `DATA_PROVENANCE.md` Fix 12 for the full history of that
//! correction.

use serde::{Deserialize, Serialize};

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// English names of the 12 signs, zero-indexed from Aries.
const SIGN_NAMES: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// Zero-based sign index (0 = Aries .. 11 = Pisces) occupied by each of the
/// nine grahas of a natal chart, as needed to compute chart-dependent sign
/// lordship for Chara (and Narayana) Dasha.
///
/// Ketu is **derived**, not accepted, as `(rahu + 6) % 12` — the two lunar
/// nodes are always exactly opposite by definition, so accepting both
/// invites an internally-inconsistent chart. Callers pass the seven
/// classical grahas plus Rahu; use [`GrahaSigns::ketu`] to read Ketu's sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrahaSigns {
    /// Sun's sign, 0-11.
    pub sun: u8,
    /// Moon's sign, 0-11.
    pub moon: u8,
    /// Mars's sign, 0-11.
    pub mars: u8,
    /// Mercury's sign, 0-11.
    pub mercury: u8,
    /// Jupiter's sign, 0-11.
    pub jupiter: u8,
    /// Venus's sign, 0-11.
    pub venus: u8,
    /// Saturn's sign, 0-11.
    pub saturn: u8,
    /// Rahu's (north lunar node's) sign, 0-11.
    pub rahu: u8,
}

impl GrahaSigns {
    /// Ketu's sign, derived as the point opposite Rahu.
    #[must_use]
    pub fn ketu(&self) -> u8 {
        (self.rahu + 6) % 12
    }
}

/// One of the nine grahas used in dual-lordship resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Graha {
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

/// All nine graha (sign, identity) pairs, for conjunction counting.
fn all_grahas(pos: GrahaSigns) -> [(Graha, u8); 9] {
    [
        (Graha::Sun, pos.sun),
        (Graha::Moon, pos.moon),
        (Graha::Mars, pos.mars),
        (Graha::Mercury, pos.mercury),
        (Graha::Jupiter, pos.jupiter),
        (Graha::Venus, pos.venus),
        (Graha::Saturn, pos.saturn),
        (Graha::Rahu, pos.rahu),
        (Graha::Ketu, pos.ketu()),
    ]
}

/// Number of *other* grahas sharing `candidate`'s sign (conjunction count).
fn conjunction_count(candidate: Graha, candidate_sign: u8, all: &[(Graha, u8); 9]) -> usize {
    all.iter()
        .filter(|&&(g, s)| g != candidate && s == candidate_sign)
        .count()
}

/// Classical exaltation sign for a graha. Rahu/Ketu exaltation is genuinely
/// contested across traditions (this uses the *vriddha-karika* pairing —
/// Rahu exalted in Taurus, Ketu exalted in Scorpio — consistent with the
/// same source attesting the Scorpio/Aquarius dual lordships above); it is
/// exercised by no fixture in this module and sits at the bottom of the
/// dual-lordship cascade, so it decides no case in practice unless every
/// stronger criterion ties.
fn exaltation_sign(graha: Graha) -> u8 {
    match graha {
        Graha::Sun => 0,                // Aries
        Graha::Moon | Graha::Rahu => 1, // Taurus (Rahu shares Moon's exaltation sign)
        Graha::Mars => 9,               // Capricorn
        Graha::Mercury => 5,            // Virgo
        Graha::Jupiter => 3,            // Cancer
        Graha::Venus => 11,             // Pisces
        Graha::Saturn => 6,             // Libra
        Graha::Ketu => 7,               // Scorpio
    }
}

/// Rashi-bala rank of a sign by modality: dual (mutable) > fixed > movable
/// (cardinal). Higher is stronger.
fn rashi_bala_rank(sign: u8) -> u8 {
    match sign % 3 {
        // 0-indexed from Aries: movable signs are 0,3,6,9; fixed 1,4,7,10;
        // dual 2,5,8,11 -- i.e. sign % 3 == 2 is dual, == 1 is fixed, == 0 is movable.
        2 => 2, // dual
        1 => 1, // fixed
        _ => 0, // movable
    }
}

/// A single Chara Dasha sign period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharaPeriod {
    /// Zero-based sign index (0 = Aries, 11 = Pisces).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: &'static str,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in years.
    pub duration_years: f64,
}

/// Compute the Chara Dasha sequence for all 12 signs.
///
/// # Arguments
/// * `lagna_sign` — Zero-based sign index of the Lagna (Ascendant), 0–11.
/// * `birth_jd`   — Julian Day of birth.
/// * `positions`  — natal sign positions of the nine grahas; each sign's
///   duration is counted to the sign its lord actually occupies here (see
///   the module doc for the full rule, including dual lordship).
///
/// # Returns
///
/// A `Vec` of 12 [`CharaPeriod`] entries in the dasha order, starting from
/// `lagna_sign` and proceeding sign by sign through all 12 signs, in the
/// direction given by [`chara_direction`] (forward or backward, tested at
/// the 9th sign from the lagna; see the module doc for the sutra and its
/// confidence grading).
#[must_use]
pub fn compute_chara(lagna_sign: u8, birth_jd: f64, positions: GrahaSigns) -> Vec<CharaPeriod> {
    let lagna_sign = lagna_sign % 12;
    let direction = chara_direction(lagna_sign);
    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;

    for i in 0u8..12 {
        let sign = match direction {
            Direction::Forward => (lagna_sign + i) % 12,
            Direction::Backward => (lagna_sign + 12 - i) % 12,
        };
        let lord_sign = lord_sign_for(sign, positions);
        let duration_years = f64::from(duration_for(sign, lord_sign));
        let duration_days = duration_years * DASHA_YEAR_DAYS;

        periods.push(CharaPeriod {
            sign_index: sign,
            sign_name: SIGN_NAMES[sign as usize],
            start_jd: current_jd,
            end_jd: current_jd + duration_days,
            duration_years,
        });

        current_jd += duration_days;
    }

    periods
}

/// Chara Dasha's counting direction for a given lagna sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

/// Whether `sign` (0-indexed, 0 = Aries) is vishama-pada ("odd-footed"):
/// Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius.
///
/// This one classification serves two *unrelated* rules in this module:
/// (1) [`chara_direction`], where it is tested at the 9th sign from the
/// lagna to pick the sequence direction, and (2) the per-sign duration
/// count direction in [`duration_for`], where it is tested on the sign
/// whose duration is being computed. Do not assume the two uses share a
/// subject — they don't.
fn is_vishama_pada(sign: u8) -> bool {
    matches!(sign, 0 | 1 | 2 | 6 | 7 | 8)
}

/// Chara Dasha's counting direction: vishama-pada/sama-pada, tested at the
/// 9th sign from `lagna` — see the module doc for the sutra and its
/// confidence grading.
fn chara_direction(lagna: u8) -> Direction {
    let ninth_from_lagna = (lagna % 12 + 8) % 12;
    if is_vishama_pada(ninth_from_lagna) {
        Direction::Forward
    } else {
        Direction::Backward
    }
}

/// Resolve the sign occupied by `sign`'s lord (or lords), given natal graha
/// positions. See the module doc for the full dual-lordship rule.
fn lord_sign_for(sign: u8, pos: GrahaSigns) -> u8 {
    match sign {
        0 => pos.mars,        // Aries -> Mars
        1 | 6 => pos.venus,   // Taurus -> Venus, Libra -> Venus
        2 | 5 => pos.mercury, // Gemini -> Mercury, Virgo -> Mercury
        3 => pos.moon,        // Cancer -> Moon
        4 => pos.sun,         // Leo -> Sun
        7 => resolve_dual_lord(sign, Graha::Mars, pos.mars, Graha::Ketu, pos.ketu(), pos), // Scorpio -> Mars/Ketu
        8 | 11 => pos.jupiter, // Sagittarius -> Jupiter, Pisces -> Jupiter
        9 => pos.saturn,       // Capricorn -> Saturn
        10 => resolve_dual_lord(sign, Graha::Saturn, pos.saturn, Graha::Rahu, pos.rahu, pos), // Aquarius -> Saturn/Rahu
        _ => unreachable!("sign is always reduced mod 12"),
    }
}

/// Resolve a dual-lordship sign (Scorpio: Mars/Ketu; Aquarius: Saturn/Rahu)
/// to a single lord's sign, per the cascade documented at the top of this
/// module.
#[allow(clippy::too_many_arguments)]
fn resolve_dual_lord(sign: u8, a: Graha, a_sign: u8, b: Graha, b_sign: u8, pos: GrahaSigns) -> u8 {
    let a_in_sign = a_sign == sign;
    let b_in_sign = b_sign == sign;

    if a_in_sign && b_in_sign {
        // Both co-lords occupy the sign itself -> own-sign rule (12 years).
        return sign;
    }
    if a_in_sign {
        // Ignore the co-lord in the sign; count to the other.
        return b_sign;
    }
    if b_in_sign {
        return a_sign;
    }

    // Both co-lords occupy some other sign: cascade to the stronger.
    let all = all_grahas(pos);

    let a_conj = conjunction_count(a, a_sign, &all);
    let b_conj = conjunction_count(b, b_sign, &all);
    if (a_conj > 0) != (b_conj > 0) {
        return if a_conj > 0 { a_sign } else { b_sign };
    }
    if a_conj != b_conj {
        return if a_conj > b_conj { a_sign } else { b_sign };
    }

    // Duration comparison, ahead of rashi-bala -- see module doc for why
    // this ordering (not the textually stated one) is what the BPHS 46
    // worked example requires.
    let a_dur = duration_for(sign, a_sign);
    let b_dur = duration_for(sign, b_sign);
    if a_dur != b_dur {
        return if a_dur > b_dur { a_sign } else { b_sign };
    }

    let a_bala = rashi_bala_rank(a_sign);
    let b_bala = rashi_bala_rank(b_sign);
    if a_bala != b_bala {
        return if a_bala > b_bala { a_sign } else { b_sign };
    }

    let a_exalted = a_sign == exaltation_sign(a);
    let b_exalted = b_sign == exaltation_sign(b);
    if a_exalted != b_exalted {
        return if a_exalted { a_sign } else { b_sign };
    }

    // Total tie on every criterion: not reached by any known worked example.
    a_sign
}

/// Duration (in years, 1-12) for `sign`, given the sign its resolved lord
/// occupies (`lord_sign`).
///
/// A sign whose lord occupies the sign itself gets 12 years. Otherwise the
/// count runs forward (zodiacal) or backward from `sign`, chosen by
/// `sign`'s own vishama-pada/sama-pada grouping — see the module doc.
fn duration_for(sign: u8, lord_sign: u8) -> u8 {
    if lord_sign == sign {
        return 12;
    }
    #[allow(clippy::cast_possible_truncation)]
    let steps = if is_vishama_pada(sign) {
        (i16::from(lord_sign) - i16::from(sign)).rem_euclid(12) as u8
    } else {
        (i16::from(sign) - i16::from(lord_sign)).rem_euclid(12) as u8
    };
    steps
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JD: f64 = 2_451_545.0; // J2000.0

    /// All-Aries placements: every graha in Aries(0), Rahu in Aries(0) so
    /// Ketu is Libra(6). Used where the exact chart doesn't matter (only
    /// direction sequencing is under test).
    const ALL_ARIES: GrahaSigns = GrahaSigns {
        sun: 0,
        moon: 0,
        mars: 0,
        mercury: 0,
        jupiter: 0,
        venus: 0,
        saturn: 0,
        rahu: 0,
    };

    /// Brihat Parashara Hora Shastra ch. 46's worked Aquarius-lagna example,
    /// used throughout this module as the conformance oracle for the
    /// duration rule (see module doc for citation and confidence grading).
    const BPHS_46_CHART: GrahaSigns = GrahaSigns {
        sun: 9,      // Capricorn
        moon: 2,     // Gemini
        mars: 1,     // Taurus
        mercury: 10, // Aquarius
        jupiter: 10, // Aquarius
        venus: 10,   // Aquarius
        saturn: 4,   // Leo
        rahu: 2,     // Gemini (Ketu derives to Sagittarius, 8)
    };

    // 1. Chara dasha produces exactly 12 sign periods
    #[test]
    fn chara_dasha_produces_12_periods() {
        let periods = compute_chara(0, TEST_JD, ALL_ARIES);
        assert_eq!(
            periods.len(),
            12,
            "expected 12 Chara periods, got {}",
            periods.len()
        );
    }

    // 2. Periods are contiguous (end of one = start of next)
    #[test]
    fn periods_are_contiguous() {
        let periods = compute_chara(0, TEST_JD, ALL_ARIES);
        for i in 1..periods.len() {
            let gap = (periods[i].start_jd - periods[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between period {}-{}: {gap}", i - 1, i);
        }
    }

    // 3. All 12 signs appear exactly once
    #[test]
    fn all_12_signs_appear() {
        let periods = compute_chara(0, TEST_JD, ALL_ARIES);
        let mut seen = [false; 12];
        for p in &periods {
            assert!(
                (p.sign_index as usize) < 12,
                "sign_index out of range: {}",
                p.sign_index
            );
            assert!(
                !seen[p.sign_index as usize],
                "sign {} appears more than once",
                p.sign_index
            );
            seen[p.sign_index as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "not all 12 signs appeared");
    }

    // 4. Ketu derives from Rahu as the opposite sign.
    #[test]
    fn ketu_derives_as_opposite_of_rahu() {
        let pos = GrahaSigns {
            rahu: 2, // Gemini
            ..ALL_ARIES
        };
        assert_eq!(pos.ketu(), 8, "Ketu should be Sagittarius, opposite Gemini");

        let pos2 = GrahaSigns {
            rahu: 10, // Aquarius
            ..ALL_ARIES
        };
        assert_eq!(pos2.ketu(), 4, "Ketu should be Leo, opposite Aquarius");
    }

    // 5. The BPHS ch. 46 worked example: all twelve durations, exactly.
    // This is the conformance oracle for the whole duration rule -- forward
    // and reverse per-sign counting, own-sign, and both dual-lord signs.
    #[test]
    fn bphs_46_worked_example_all_twelve_durations() {
        let periods = compute_chara(10, TEST_JD, BPHS_46_CHART); // Aquarius lagna
        let mut by_sign = [0.0f64; 12];
        for p in &periods {
            by_sign[p.sign_index as usize] = p.duration_years;
        }
        let expected = [
            1.0, // Aries
            9.0, // Taurus
            8.0, // Gemini
            1.0, // Cancer
            7.0, // Leo
            7.0, // Virgo
            4.0, // Libra
            6.0, // Scorpio
            2.0, // Sagittarius
            5.0, // Capricorn
            8.0, // Aquarius
            1.0, // Pisces
        ];
        assert_eq!(
            by_sign, expected,
            "BPHS ch. 46 worked example durations do not match"
        );
    }

    // 6. Forward-pada sign duration direction (Aries: forward/zodiacal).
    #[test]
    fn forward_pada_sign_counts_zodiacally() {
        // Aries(0) lord Mars in Taurus(1): forward count = 1 step.
        assert_eq!(duration_for(0, 1), 1);
        // Aries(0) lord Mars in Pisces(11): forward count = 11 steps.
        assert_eq!(duration_for(0, 11), 11);
    }

    // 7. Reverse-pada sign duration direction (Cancer: reverse).
    #[test]
    fn reverse_pada_sign_counts_backward() {
        // Cancer(3) lord Moon in Gemini(2): backward count = 1 step.
        assert_eq!(duration_for(3, 2), 1);
        // Cancer(3) lord Moon in Leo(4): backward count = 11 steps
        // (counting backward all the way around).
        assert_eq!(duration_for(3, 4), 11);
    }

    // 8. Own-sign always gives 12, regardless of pada direction.
    #[test]
    fn own_sign_gives_twelve_years() {
        assert_eq!(duration_for(0, 0), 12, "forward-pada own-sign");
        assert_eq!(duration_for(3, 3), 12, "reverse-pada own-sign");
    }

    // 9. Scorpio dual lordship (Mars/Ketu): resolves to Mars per the BPHS
    // worked example, via the duration-comparison cascade step.
    #[test]
    fn scorpio_dual_lordship_resolves_to_mars() {
        let lord_sign = lord_sign_for(7, BPHS_46_CHART);
        assert_eq!(
            lord_sign, 1,
            "Scorpio's lord should resolve to Mars's sign (Taurus)"
        );
        assert_eq!(duration_for(7, lord_sign), 6);
    }

    // 10. Aquarius dual lordship (Saturn/Rahu): resolves to Rahu per the
    // BPHS worked example, via the conjunction cascade step.
    #[test]
    fn aquarius_dual_lordship_resolves_to_rahu() {
        let lord_sign = lord_sign_for(10, BPHS_46_CHART);
        assert_eq!(
            lord_sign, 2,
            "Aquarius's lord should resolve to Rahu's sign (Gemini)"
        );
        assert_eq!(duration_for(10, lord_sign), 8);
    }

    // 11. Both co-lords in the sign itself -> 12 years.
    #[test]
    fn both_dual_colords_in_own_sign_gives_twelve() {
        // Scorpio(7): Mars and Ketu both in Scorpio.
        let pos = GrahaSigns {
            mars: 7,
            rahu: 1, // Ketu = 1 + 6 = 7 (Scorpio)
            ..ALL_ARIES
        };
        assert_eq!(pos.ketu(), 7);
        let lord_sign = lord_sign_for(7, pos);
        assert_eq!(lord_sign, 7);
        assert_eq!(duration_for(7, lord_sign), 12);
    }

    // 12. One co-lord in the sign itself, the other elsewhere -> ignore the
    // one in the sign, count to the other.
    #[test]
    fn one_dual_colord_in_sign_counts_to_the_other() {
        // Aquarius(10): Saturn in Aquarius itself, Rahu elsewhere (Gemini).
        let pos = GrahaSigns {
            saturn: 10,
            rahu: 2,
            ..ALL_ARIES
        };
        let lord_sign = lord_sign_for(10, pos);
        assert_eq!(
            lord_sign, 2,
            "Saturn (in the sign) should be ignored in favour of Rahu"
        );
    }

    // 5-16 (direction). For every lagna, tested at the 9th sign from the
    // lagna (vishama-pada => forward, sama-pada => backward).
    #[test]
    fn aries_lagna_counts_forward() {
        let periods = compute_chara(0, TEST_JD, ALL_ARIES); // Aries; 9th = Sagittarius, vishama-pada
        assert_eq!(periods[0].sign_index, 0);
        assert_eq!(
            periods[1].sign_index, 1,
            "second period should be Taurus (forward)"
        );
    }

    #[test]
    fn taurus_lagna_counts_backward() {
        let periods = compute_chara(1, TEST_JD, ALL_ARIES); // Taurus; 9th = Capricorn, sama-pada
        assert_eq!(periods[0].sign_index, 1);
        assert_eq!(
            periods[1].sign_index, 0,
            "second period should be Aries (backward)"
        );
    }

    #[test]
    fn gemini_lagna_counts_backward() {
        let periods = compute_chara(2, TEST_JD, ALL_ARIES); // Gemini; 9th = Aquarius, sama-pada
        assert_eq!(periods[0].sign_index, 2);
        assert_eq!(
            periods[1].sign_index, 1,
            "second period should be Taurus (backward)"
        );
    }

    #[test]
    fn cancer_lagna_counts_backward() {
        let periods = compute_chara(3, TEST_JD, ALL_ARIES); // Cancer; 9th = Pisces, sama-pada
        assert_eq!(periods[0].sign_index, 3);
        assert_eq!(
            periods[1].sign_index, 2,
            "second period should be Gemini (backward)"
        );
    }

    #[test]
    fn leo_lagna_counts_forward() {
        let periods = compute_chara(4, TEST_JD, ALL_ARIES); // Leo; 9th = Aries, vishama-pada
        assert_eq!(periods[0].sign_index, 4);
        assert_eq!(
            periods[1].sign_index, 5,
            "second period should be Virgo (forward)"
        );
    }

    #[test]
    fn virgo_lagna_counts_forward() {
        let periods = compute_chara(5, TEST_JD, ALL_ARIES); // Virgo; 9th = Taurus, vishama-pada
        assert_eq!(periods[0].sign_index, 5);
        assert_eq!(
            periods[1].sign_index, 6,
            "second period should be Libra (forward)"
        );
    }

    #[test]
    fn libra_lagna_counts_forward() {
        let periods = compute_chara(6, TEST_JD, ALL_ARIES); // Libra; 9th = Gemini, vishama-pada
        assert_eq!(periods[0].sign_index, 6);
        assert_eq!(
            periods[1].sign_index, 7,
            "second period should be Scorpio (forward)"
        );
    }

    #[test]
    fn scorpio_lagna_counts_backward() {
        let periods = compute_chara(7, TEST_JD, ALL_ARIES); // Scorpio; 9th = Cancer, sama-pada
        assert_eq!(periods[0].sign_index, 7);
        assert_eq!(
            periods[1].sign_index, 6,
            "second period should be Libra (backward)"
        );
    }

    #[test]
    fn sagittarius_lagna_counts_backward() {
        let periods = compute_chara(8, TEST_JD, ALL_ARIES); // Sagittarius; 9th = Leo, sama-pada
        assert_eq!(periods[0].sign_index, 8);
        assert_eq!(
            periods[1].sign_index, 7,
            "second period should be Scorpio (backward)"
        );
    }

    #[test]
    fn capricorn_lagna_counts_backward() {
        let periods = compute_chara(9, TEST_JD, ALL_ARIES); // Capricorn; 9th = Virgo, sama-pada
        assert_eq!(periods[0].sign_index, 9);
        assert_eq!(
            periods[1].sign_index, 8,
            "second period should be Sagittarius (backward)"
        );
    }

    #[test]
    fn aquarius_lagna_counts_forward() {
        let periods = compute_chara(10, TEST_JD, ALL_ARIES); // Aquarius; 9th = Libra, vishama-pada
        assert_eq!(periods[0].sign_index, 10);
        assert_eq!(
            periods[1].sign_index, 11,
            "second period should be Pisces (forward)"
        );
    }

    #[test]
    fn pisces_lagna_counts_forward() {
        let periods = compute_chara(11, TEST_JD, ALL_ARIES); // Pisces; 9th = Scorpio, vishama-pada
        assert_eq!(periods[0].sign_index, 11);
        assert_eq!(
            periods[1].sign_index, 0,
            "second period should be Aries (forward)"
        );
    }

    // 17. The load-bearing claim behind the sequence-direction fix: is_vishama_pada's
    // true-set is exactly the four-sign-exception classification the previously-implemented
    // rule (DATA_PROVENANCE.md Fix 11) applied directly to the lagna.
    #[test]
    fn vishama_pada_set_matches_the_superseded_fixed_sign_exception() {
        fn fix_11_forward_on_lagna_directly(sign: u8) -> bool {
            match sign {
                1 | 7 => true,               // Taurus, Scorpio: fixed exception
                4 | 10 => false,             // Leo, Aquarius: fixed exception
                _ => sign.is_multiple_of(2), // plain odd/even (0-indexed even = 1-indexed odd)
            }
        }
        for sign in 0u8..12 {
            assert_eq!(
                is_vishama_pada(sign),
                fix_11_forward_on_lagna_directly(sign),
                "sign {sign} ({}): vishama-pada must match the superseded Fix 11 classification",
                SIGN_NAMES[sign as usize]
            );
        }
    }

    // 18. All 12 signs still appear exactly once regardless of direction (backward case)
    #[test]
    fn all_12_signs_appear_going_backward() {
        let periods = compute_chara(3, TEST_JD, ALL_ARIES); // Cancer, backward
        let mut seen = [false; 12];
        for p in &periods {
            assert!(
                !seen[p.sign_index as usize],
                "sign {} appears more than once",
                p.sign_index
            );
            seen[p.sign_index as usize] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "not all 12 signs appeared going backward"
        );
    }
}
