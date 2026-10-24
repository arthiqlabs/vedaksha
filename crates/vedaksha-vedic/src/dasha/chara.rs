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
//! Attestation: Jaimini Upadesha Sutras 1.1.28, *nathantah samah prayena*
//! (the years are, in general, up to the lord). The commentary on it has the
//! count run up to the rashi the rashi's lord occupies (*rashi-svami-
//! adhishthita-rashy-antam*) and takes "in general" to cover the lord at
//! home, the exalted and the debilitated lord, and the sign with two lords.
//! Brihat Parashara Hora Shastra ch. 46, slokas 155 and 156, states the count
//! (*lagnadi-vyaya-paryantam bhanam chara-dasham bruve; tasmat tad-isha-
//! paryantam samkhyam atra dasham viduh*), as does the fifth of the old
//! verses (the *vriddha-karikas*) edited with the sutras on 1.1. BPHS verse
//! numbers here are those of the recension this crate follows, and the
//! verses are also given by their incipit. Grade (a).
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
//! **Dual lordship.** Scorpio has the co-lords Mars and Ketu, Aquarius Saturn
//! and Rahu (BPHS 46.157, *vrishchikadhipati dvau cha ketu-bhaumau smritau
//! dvija; shani-rahu cha kumbhasya svaminau parikirtitau*; the old verses
//! say the same at 1.1.6). The polarity, Rahu with Aquarius and not Scorpio,
//! is frequently inverted in secondary sources, so it is stated explicitly
//! here. Which co-lord's rashi the count runs to is decided by BPHS 46.158 to
//! 166 and by the old verses at 1.1.7 to 1.1.13, which are largely the same
//! verses in another arrangement (another recension of BPHS sets them in its
//! chapter on length of life, at 31.77 to 86). The decision, in order:
//!
//! 1. Both co-lords stand in the sign itself -> 12 years (*dvav evadhipati
//!    vipra yuktau svarkshe sthitau yadi; varsham dvadashakam tatra*).
//! 2. One stands in the sign, the other does not -> count to the other
//!    (*ekah svakshetrago 'nyas tu paratra yadi samsthitah; tadanyatra
//!    sthitam natham parigrihya dasham nayet*).
//! 3. Both stand elsewhere -> count to the **stronger** (*tayor madhye cha yo
//!    bali*), by the first of these tests that separates them:
//!    1. a co-lord with another graha in its rashi beats one with none, and
//!       with grahas in both, the one with more (*agrahat sagraho bali; dvav
//!       eva sagrahau tau chet bali tatradhikagrahah*);
//!    2. the nature of the rashi it stands in, dual over fixed over movable
//!       (*grahayoga-samanatve jneyam rashi-balad balam; jneyash chara-sthira-
//!       dvandvah kramato bala-shalinah*);
//!    3. an exalted co-lord over one that is not (where this test falls is
//!       the one open point; see below);
//!    4. the rashi that gives the **larger** count (*rashi-sattva-samanatve
//!       bahuvarsho bali bhavet*).
//!
//! Grade (a) for the cases and for tests 1, 2 and 4, and for their order. The
//! larger count comes last, not first. Two different rashis always give
//! different counts (over the eleven rashis other than the one being timed
//! the count takes each value from 1 to 11 once), so test 4 settles whatever
//! the first three leave open, and the cascade has no tie to break by
//! position. "Another graha in its rashi" means any of the other eight,
//! nodes included; the co-lord itself cannot count, since every co-lord's
//! rashi holds it and test 1 would then never separate two of them.
//!
//! **Where the exalted co-lord's test falls (grade (b)).** The verses state
//! it as a case of its own, *ekah svochchagatash chanyah paratra yadi
//! samsthitah; grihniyad uchchakhetastham rashim anyam vihaya vai* (46.164;
//! 1.1.10 and 11 of the old verses): if one co-lord is exalted and the other
//! is merely elsewhere, take the exalted one's rashi. They set it behind the
//! larger count. A rule that waits behind a test which always decides could
//! never apply, and the recension at 31.81 and 82 says what the order is
//! meant to be: the exalted co-lord prevails over the larger count, even when
//! it gives fewer years (*yadyalpavarshado vipra tadapi tungago bali*), and
//! that place is directly after the nature of the rashi, since the larger
//! count is only reached when the rashis tie. This is
//! [`ExaltationRank::AfterRashiBala`], the default. The other reading takes
//! the verse as free-standing: whenever exactly one co-lord is exalted, its
//! rashi is taken, ahead of every other test ([`ExaltationRank::Decisive`]).
//! Exaltation of Rahu and Ketu is itself contested across traditions; the
//! pairing used is `rashi_strength::exaltation_sign`'s.
//!
//! **Not implemented: 1.1.14 and 1.1.15 of the old verses.** BPHS does not
//! carry them. The first appears to take the rashi without a graha when the
//! lord of that rashi is exalted, the second to prefer "the earlier" rashi to
//! a conjunction of two grahas. Their wording is too compressed to apply with
//! confidence, so the cascade above does not use them.
//!
//! **Deliberately NOT implemented: the exaltation/debilitation +-1 year
//! adjustment.** A well-attested classical rule (BPHS 46.165 plus the
//! *vriddha-karika* at 1.1.13) adds one year to a sign's duration
//! when its lord is exalted, and subtracts one when the lord is debilitated.
//! Amit has decided not to implement it here, following a named
//! 20th-century teaching lineage that dropped the adjustment precisely
//! because it can produce a 0-year dasha (Sagittarius, when Jupiter is in
//! Capricorn) or a 13-year dasha (Virgo, when Mercury is in Virgo) — both
//! outside the system's own stated 1-12 year range. This is a deliberate,
//! cited omission of an attested rule, not an oversight. It is a different
//! rule from the exaltation test in the dual-lordship cascade above, which is
//! implemented.
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
//!
//! **Antardashas.** [`compute_chara`] returns the twelve mahadashas only and
//! is unchanged; [`compute_chara_antar`] and [`compute_chara_with_antars`]
//! return the same twelve with their twelve antardashas each. The rule is
//! BPHS's, from the chapter on working out antardashas (ch. 51 in the
//! recension this crate follows, slokas 5 to 9): each antardasha a twelfth
//! of its mahadasha; the twelve starting from the dasha rashi or the seventh
//! from it, whichever is stronger; unbroken for a movable dasha rashi, every
//! sixth rashi for a fixed one, kendra groups for a dual one; forward from an
//! odd dasha rashi and in reverse from an even one. The Upadesha Sutras do
//! not state the length. The order shapes are the sutras *prathame
//! prak-pratyaktvam* (2.4.8), *dvitiye ravitah* (2.4.9) and *prithak-
//! kramena trtiye catushtayadi* (2.4.10), numbering of the critical edition
//! used throughout `dasha::rashi_antar`, whose commentary says these three
//! give the order "everywhere in the chara-paryaya dasha". Grade (a) for the
//! BPHS rule, whose own worked examples are encoded as tests in
//! `dasha::rashi_antar`; grade (b) for the sutras (the commentary's
//! extension to every chara dasha is the commentator's). The rules that the
//! texts leave open (where the series starts when the chapters conflict,
//! whether to keep the sixth-sixth and kendra shapes) are
//! [`AntarRules`](super::rashi_antar::AntarRules); see that module's doc.

use serde::{Deserialize, Serialize};

use super::rashi_antar::{AntarLength, AntarRules, RashiPeriod, build_antars};
use super::rashi_strength::{
    JaiminiStrength, NatalPlacements, Strength, exaltation_sign, natural_rank, rashi_lords,
};
use crate::graha::Graha;

/// Dasha year length in days (Julian year).
const DASHA_YEAR_DAYS: f64 = 365.25;

/// English names of the 12 signs, zero-indexed from Aries.
pub(crate) const SIGN_NAMES: [&str; 12] = [
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

/// Where an exalted co-lord's test falls among the tests that choose between
/// the two lords of Scorpio or Aquarius. See the module doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExaltationRank {
    /// After the nature of the rashi and ahead of the larger count: the
    /// exalted co-lord prevails over a co-lord that merely gives more years
    /// (BPHS 46.164, read with the recension that states it at 31.81 and 82).
    /// Grade (b). The default.
    #[default]
    AfterRashiBala,
    /// Ahead of every other test: whenever exactly one co-lord is exalted,
    /// its rashi is taken (BPHS 46.164 read as a free-standing rule). Grade
    /// (b).
    Decisive,
}

/// The readings the verses leave open when choosing between the two lords of
/// Scorpio or Aquarius.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DualLordRules {
    /// Where the exalted co-lord's test falls.
    pub exaltation: ExaltationRank,
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
///
/// The two-lord signs are resolved by `DualLordRules::default()`; use
/// [`compute_chara_with`] to choose another reading.
#[must_use]
pub fn compute_chara(lagna_sign: u8, birth_jd: f64, positions: GrahaSigns) -> Vec<CharaPeriod> {
    compute_chara_with(lagna_sign, birth_jd, positions, DualLordRules::default())
}

/// [`compute_chara`] with the reading of the two-lord signs chosen by
/// `rules`. Only Scorpio and Aquarius can differ from [`compute_chara`], and
/// only where both co-lords stand outside the sign.
#[must_use]
pub fn compute_chara_with(
    lagna_sign: u8,
    birth_jd: f64,
    positions: GrahaSigns,
    rules: DualLordRules,
) -> Vec<CharaPeriod> {
    let lagna_sign = lagna_sign % 12;
    let direction = chara_direction(lagna_sign);
    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;

    for i in 0u8..12 {
        let sign = match direction {
            Direction::Forward => (lagna_sign + i) % 12,
            Direction::Backward => (lagna_sign + 12 - i) % 12,
        };
        let lord_sign = lord_sign_with(sign, positions, rules);
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

/// Compute the Chara Dasha with the antardashas of every period.
///
/// The twelve mahadashas are exactly [`compute_chara`]'s: same signs, same
/// bounds, same durations, bit for bit. Each carries its twelve antardashas
/// under `rules`, each a twelfth of the mahadasha, tiling it exactly.
/// `strength` decides "the stronger of the dasha rashi and its seventh" when
/// `rules.start` asks for it; pass [`JaiminiStrength`] over the same chart
/// for the Jaimini comparison, or your own [`Strength`].
#[must_use]
pub fn compute_chara_antar(
    lagna_sign: u8,
    birth_jd: f64,
    positions: GrahaSigns,
    rules: AntarRules,
    strength: &dyn Strength,
) -> Vec<RashiPeriod> {
    let placements = NatalPlacements::from_signs(lagna_sign, positions);
    compute_chara(lagna_sign, birth_jd, positions)
        .into_iter()
        .map(|p| RashiPeriod {
            sign_index: p.sign_index,
            sign_name: p.sign_name.to_string(),
            start_jd: p.start_jd,
            end_jd: p.end_jd,
            duration_years: p.duration_years,
            antar: build_antars(
                p.sign_index,
                p.start_jd,
                p.end_jd,
                p.duration_years,
                rules,
                AntarLength::EqualTwelfths,
                &placements,
                strength,
            ),
        })
        .collect()
}

/// [`compute_chara_antar`] with the default rules (BPHS 51.5 to 51.9) and
/// the Jaimini strength comparison read from `positions`.
#[must_use]
pub fn compute_chara_with_antars(
    lagna_sign: u8,
    birth_jd: f64,
    positions: GrahaSigns,
) -> Vec<RashiPeriod> {
    let placements = NatalPlacements::from_signs(lagna_sign, positions);
    let strength = JaiminiStrength::new(&placements);
    compute_chara_antar(
        lagna_sign,
        birth_jd,
        positions,
        AntarRules::default(),
        &strength,
    )
}

/// Whether the chara sequence for `lagna_sign` runs forward through the
/// zodiac (the 9th-from-lagna test of the module doc).
pub(crate) fn sequence_runs_forward(lagna_sign: u8) -> bool {
    matches!(chara_direction(lagna_sign), Direction::Forward)
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

/// Whether a sign's own **duration count** runs zodiacally (forward).
///
/// Deliberately a separate function from [`is_vishama_pada`] despite
/// currently delegating to it. The two encode the same classification but
/// answer different questions about different subjects: this one is asked
/// twelve times, once per sign being measured, to pick that sign's
/// duration-count direction; [`is_vishama_pada`] is asked once, about the
/// 9th sign from the lagna, to pick the whole sequence's direction. They are
/// separately attested and could separately move. Collapsing this into a
/// direct call would mean a future edit driven by new sequence-direction
/// research silently changed every duration as well.
fn counts_zodiacally(sign: u8) -> bool {
    is_vishama_pada(sign)
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
/// positions, by `DualLordRules::default()`. See the module doc for the full
/// dual-lordship rule.
pub(crate) fn lord_sign_for(sign: u8, pos: GrahaSigns) -> u8 {
    lord_sign_with(sign, pos, DualLordRules::default())
}

/// [`lord_sign_for`] with the reading of the two-lord signs chosen by `rules`.
pub(crate) fn lord_sign_with(sign: u8, pos: GrahaSigns, rules: DualLordRules) -> u8 {
    let sign = sign % 12;
    // The lagna plays no part in any lord's placement.
    let placements = NatalPlacements::from_signs(0, pos);
    match *rashi_lords(sign) {
        [only] => placements.sign_of(only),
        [first, second] => {
            let stronger = stronger_co_lord(sign, first, second, &placements, rules);
            placements.sign_of(stronger)
        }
        _ => unreachable!("every rashi has one lord or two"),
    }
}

/// The co-lord whose rashi the count runs to for a two-lord `sign`
/// (Scorpio: Mars and Ketu; Aquarius: Saturn and Rahu), by the cases and
/// tests of the module doc. `first` wins when the two stand in one rashi,
/// where the choice cannot change the count.
fn stronger_co_lord(
    sign: u8,
    first: Graha,
    second: Graha,
    placements: &NatalPlacements,
    rules: DualLordRules,
) -> Graha {
    let at_home = |g: Graha| placements.sign_of(g) == sign;
    match (at_home(first), at_home(second)) {
        // Only the first at home -> ignore it, count to the second.
        (true, false) => return second,
        // Both at home (12 years: the count is to the sign itself), or only
        // the second -> the first.
        (true, true) | (false, true) => return first,
        (false, false) => {}
    }
    if co_lord_key(sign, first, placements, rules) >= co_lord_key(sign, second, placements, rules) {
        first
    } else {
        second
    }
}

/// The tests that choose between two co-lords standing outside the sign, as
/// a key that compares in the order the tests apply: the larger key is the
/// stronger co-lord. Each element is one test of the module doc; a pair of
/// co-lords the first leaves tied goes to the second element, and so on.
fn co_lord_key(
    sign: u8,
    lord: Graha,
    placements: &NatalPlacements,
    rules: DualLordRules,
) -> [u8; 4] {
    let lord_sign = placements.sign_of(lord);
    // Other grahas in the lord's rashi: presence and number in one figure,
    // since none is the fewest. At most eight others, so it fits in a byte.
    let companions = placements.companions_of(lord) as u8;
    let nature = natural_rank(lord_sign);
    let exalted = u8::from(lord_sign == exaltation_sign(lord));
    // 1 to 11, and different for different rashis.
    let years = duration_for(sign, lord_sign);
    match rules.exaltation {
        ExaltationRank::AfterRashiBala => [companions, nature, exalted, years],
        ExaltationRank::Decisive => [exalted, companions, nature, years],
    }
}

/// Duration (in years, 1-12) for `sign`, given the sign its resolved lord
/// occupies (`lord_sign`).
///
/// A sign whose lord occupies the sign itself gets 12 years. Otherwise the
/// count runs forward (zodiacal) or backward from `sign`, chosen by
/// `sign`'s own vishama-pada/sama-pada grouping — see the module doc.
///
/// Uses [`counts_zodiacally`], not [`is_vishama_pada`] directly. The two
/// currently return the same thing, but they answer different questions
/// about different subjects (this one about the sign being measured, the
/// other about the 9th from the lagna), and the indirection is what keeps
/// them independently editable if research ever moves one.
fn duration_for(sign: u8, lord_sign: u8) -> u8 {
    if lord_sign == sign {
        return 12;
    }
    // `rem_euclid(12)` on a non-equal pair yields 1..=11, so the `as u8`
    // cannot truncate and the result is never 0 -- a zero-length dasha is
    // unreachable here by construction, not by clamping.
    if counts_zodiacally(sign) {
        (i16::from(lord_sign) - i16::from(sign)).rem_euclid(12) as u8
    } else {
        (i16::from(sign) - i16::from(lord_sign)).rem_euclid(12) as u8
    }
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

    /// An Aquarius-lagna chart for the duration and antardasha tests. Every
    /// duration it gives is derived by hand from the rule in
    /// `all_twelve_durations_derived_by_hand`.
    const CHART_A: GrahaSigns = GrahaSigns {
        sun: 9,      // Capricorn
        moon: 2,     // Gemini
        mars: 1,     // Taurus
        mercury: 10, // Aquarius
        jupiter: 10, // Aquarius
        venus: 10,   // Aquarius
        saturn: 4,   // Leo
        rahu: 2,     // Gemini (Ketu derives to Sagittarius, 8)
    };

    /// A chart from the eight placements, in the order of [`GrahaSigns`].
    fn chart(
        sun: u8,
        moon: u8,
        mars: u8,
        mercury: u8,
        jupiter: u8,
        venus: u8,
        saturn: u8,
        rahu: u8,
    ) -> GrahaSigns {
        GrahaSigns {
            sun,
            moon,
            mars,
            mercury,
            jupiter,
            venus,
            saturn,
            rahu,
        }
    }

    /// Years the default rules give `sign` in `pos`.
    fn years(sign: u8, pos: GrahaSigns) -> u8 {
        duration_for(sign, lord_sign_for(sign, pos))
    }

    /// Years `rules` give `sign` in `pos`.
    fn years_with(sign: u8, pos: GrahaSigns, rules: DualLordRules) -> u8 {
        duration_for(sign, lord_sign_with(sign, pos, rules))
    }

    const DECISIVE: DualLordRules = DualLordRules {
        exaltation: ExaltationRank::Decisive,
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

    // 5. All twelve durations of one chart, derived by hand from the rule.
    // CHART_A: Sun Capricorn(9), Moon Gemini(2), Mars Taurus(1), Mercury,
    // Jupiter and Venus Aquarius(10), Saturn Leo(4), Rahu Gemini(2), so Ketu
    // Sagittarius(8). A sign's count runs forward to its lord's sign for
    // Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius ((lord - sign)
    // mod 12) and backward for the rest ((sign - lord) mod 12); a lord at
    // home gives 12.
    //   Aries(0)       lord Mars in Taurus(1):        1 - 0        = 1
    //   Taurus(1)      lord Venus in Aquarius(10):    10 - 1       = 9
    //   Gemini(2)      lord Mercury in Aquarius(10):  10 - 2       = 8
    //   Cancer(3)      lord Moon in Gemini(2):        3 - 2        = 1
    //   Leo(4)         lord Sun in Capricorn(9):      (4 - 9) % 12 = 7
    //   Virgo(5)       lord Mercury in Aquarius(10):  (5 - 10) % 12 = 7
    //   Libra(6)       lord Venus in Aquarius(10):    10 - 6       = 4
    //   Scorpio(7)     Mars in Taurus(1), Ketu in Sagittarius(8), neither at
    //                  home. Nothing else stands in either rashi (Taurus holds
    //                  Mars only, Sagittarius Ketu only), so test 1 ties; the
    //                  nature of the rashi decides, Sagittarius (dual) over
    //                  Taurus (fixed): Ketu. 8 - 7                  = 1
    //   Sagittarius(8) lord Jupiter in Aquarius(10):  10 - 8       = 2
    //   Capricorn(9)   lord Saturn in Leo(4):         9 - 4        = 5
    //   Aquarius(10)   Saturn in Leo(4) alone, Rahu in Gemini(2) with the
    //                  Moon: Rahu has a companion, Saturn none: Rahu.
    //                  10 - 2                                       = 8
    //   Pisces(11)     lord Jupiter in Aquarius(10):  11 - 10      = 1
    #[test]
    fn all_twelve_durations_derived_by_hand() {
        let periods = compute_chara(10, TEST_JD, CHART_A); // Aquarius lagna
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
            1.0, // Scorpio
            2.0, // Sagittarius
            5.0, // Capricorn
            8.0, // Aquarius
            1.0, // Pisces
        ];
        assert_eq!(by_sign, expected);
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

    // 9. Scorpio (Mars/Ketu) in CHART_A: neither stands at home, neither has
    // a companion, so the nature of the rashi decides. Ketu in Sagittarius
    // (dual) beats Mars in Taurus (fixed) although Mars would give 6 years
    // and Ketu 1: the longer count is the last test, not the first.
    #[test]
    fn scorpio_nature_of_the_rashi_outranks_the_longer_count() {
        let lord_sign = lord_sign_for(7, CHART_A);
        assert_eq!(
            lord_sign, 8,
            "Scorpio's count should run to Ketu's sign (Sagittarius)"
        );
        assert_eq!(duration_for(7, lord_sign), 1);
        // Mars's sign would have given 6.
        assert_eq!(duration_for(7, 1), 6);
    }

    // 10. Aquarius (Saturn/Rahu) in CHART_A: Rahu in Gemini with the Moon,
    // Saturn alone in Leo. Every test happens to point the same way here
    // (companion, dual over fixed, 8 years over 6), so this guards the
    // pairing of Rahu with Aquarius rather than the order of the tests: with
    // the nodes' pairing inverted (Ketu with Aquarius), Sagittarius would
    // stand against Leo and the count would run there.
    #[test]
    fn aquarius_resolves_to_rahu_in_chart_a() {
        let lord_sign = lord_sign_for(10, CHART_A);
        assert_eq!(
            lord_sign, 2,
            "Aquarius's count should run to Rahu's sign (Gemini)"
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

    // 12b. The order of the tests that choose between two co-lords standing
    // outside the sign. Scorpio counts forward, so the years it gives for a
    // lord in sign L are (L - 7) mod 12; Aquarius counts backward, (10 - L)
    // mod 12. Each chart is built so that every test but the one named is
    // tied, or points the other way, and the working is shown.

    // Test 1: a companion beats the nature of the rashi.
    // Mars in Taurus(1) with the Moon; Ketu (Rahu in Gemini(2)) in
    // Sagittarius(8) alone. Mars has one companion, Ketu none, so the count
    // runs to Taurus: (1 - 7) mod 12 = 6. Nature alone would give Sagittarius
    // (dual over fixed), 1 year.
    #[test]
    fn a_companion_outranks_the_nature_of_the_rashi() {
        let pos = chart(0, 1, 1, 5, 6, 9, 10, 2);
        assert_eq!(pos.ketu(), 8);
        assert_eq!(lord_sign_for(7, pos), 1);
        assert_eq!(years(7, pos), 6);
        // Aquarius: Saturn in Leo(4) with the Sun, Rahu in Gemini(2) alone
        // (Moon 0, Mars 1, Mercury 5, Jupiter 6, Venus 9): Saturn has the
        // companion, so (10 - 4) = 6, although Gemini is dual and Leo fixed.
        let pos = chart(4, 0, 1, 5, 6, 9, 4, 2);
        assert_eq!(lord_sign_for(10, pos), 4);
        assert_eq!(years(10, pos), 6);
    }

    // Test 1: more companions beat fewer.
    // Mars in Taurus(1) with the Sun and the Moon (two companions); Ketu in
    // Sagittarius(8) with Venus (one). Both have a companion, so presence
    // alone ties and the nature of the rashi would pick Ketu; the number
    // picks Mars: (1 - 7) mod 12 = 6.
    #[test]
    fn more_companions_outrank_fewer() {
        let pos = chart(1, 1, 1, 5, 6, 8, 10, 2);
        assert_eq!(pos.ketu(), 8);
        assert_eq!(lord_sign_for(7, pos), 1);
        assert_eq!(years(7, pos), 6);
    }

    // Test 2: the nature of the rashi beats the longer count, all three ways.
    // (a) Mars in Cancer(3, movable) years (3 - 7) mod 12 = 8; Ketu in
    //     Taurus(1, fixed) (Rahu in Scorpio(7)) years (1 - 7) mod 12 = 6.
    //     Nothing else stands in either rashi (Sun 0, Moon 9, Mercury 5,
    //     Jupiter 11, Venus 10, Saturn 4). Fixed over movable: 6, not 8.
    // (b) Mars in Pisces(11, dual) years (11 - 7) = 4; Ketu in Libra(6,
    //     movable) (Rahu in Aries(0)) years (6 - 7) mod 12 = 11. Dual over
    //     movable: 4, not 11.
    // (c) Aquarius: Saturn in Gemini(2, dual) years (10 - 2) = 8; Rahu in
    //     Aries(0, movable) years (10 - 0) = 10. Dual over movable: 8, not
    //     10. (Sun 1, Moon 4, Mars 5, Mercury 7, Jupiter 9, Venus 11; Ketu
    //     is in Libra.)
    // Dual over fixed is `scorpio_nature_of_the_rashi_outranks_the_longer_count`.
    #[test]
    fn nature_of_the_rashi_outranks_the_longer_count() {
        let a = chart(0, 9, 3, 5, 11, 10, 4, 7);
        assert_eq!(a.ketu(), 1);
        assert_eq!(years(7, a), 6);
        let b = chart(1, 2, 11, 3, 4, 5, 8, 0);
        assert_eq!(b.ketu(), 6);
        assert_eq!(years(7, b), 4);
        let c = chart(1, 4, 5, 7, 9, 11, 2, 0);
        assert_eq!(c.ketu(), 6);
        assert_eq!(years(10, c), 8);
    }

    // Test 3: an exalted co-lord beats the longer count.
    // (a) Mars in Capricorn(9, exalted, movable) years (9 - 7) = 2; Ketu in
    //     Libra(6, movable) (Rahu in Aries(0)) years (6 - 7) mod 12 = 11. No
    //     companions (Sun 1, Moon 4, Mercury 5, Jupiter 11, Venus 3, Saturn
    //     8), both movable, so the nature ties and exaltation decides: 2,
    //     not the 11 that the longer count would give.
    // (b) Aquarius: Saturn in Libra(6, exalted, movable) years (10 - 6) = 4;
    //     Rahu in Cancer(3, movable) years (10 - 3) = 7; Ketu is in
    //     Capricorn. Nothing else in Libra or Cancer (Sun 0, Moon 1, Mars 2,
    //     Mercury 4, Jupiter 5, Venus 11). Saturn's exaltation decides: 4,
    //     not 7.
    #[test]
    fn an_exalted_co_lord_outranks_the_longer_count() {
        let a = chart(1, 4, 9, 5, 11, 3, 8, 0);
        assert_eq!(a.ketu(), 6);
        assert_eq!(lord_sign_for(7, a), 9);
        assert_eq!(years(7, a), 2);
        let b = chart(0, 1, 2, 4, 5, 11, 6, 3);
        assert_eq!(b.ketu(), 9);
        assert_eq!(lord_sign_for(10, b), 6);
        assert_eq!(years(10, b), 4);
    }

    // Test 4: with nothing else to separate them, the longer count.
    // Mars in Aries(0) years (0 - 7) mod 12 = 5; Ketu in Cancer(3) (Rahu in
    // Capricorn(9)) years (3 - 7) mod 12 = 8. Both movable, no companions
    // (Sun 1, Moon 4, Mercury 5, Jupiter 6, Venus 11, Saturn 10), and
    // neither exalted (Mars exalts in Capricorn, Ketu in Scorpio): 8.
    #[test]
    fn the_longer_count_decides_last() {
        let pos = chart(1, 4, 0, 5, 6, 11, 10, 9);
        assert_eq!(pos.ketu(), 3);
        assert_eq!(lord_sign_for(7, pos), 3);
        assert_eq!(years(7, pos), 8);
    }

    // The readings of the exalted co-lord's place.
    // (a) Mars exalted in Capricorn(9) alone, Ketu in Sagittarius(8) alone
    //     (Rahu in Gemini(2); Sun 0, Moon 3, Mercury 5, Jupiter 6, Venus 11,
    //     Saturn 10). The nature of the rashi comes first by default: dual
    //     Sagittarius over movable Capricorn, (8 - 7) = 1. Decisive takes the
    //     exalted Mars: (9 - 7) = 2.
    // (b) Mars exalted in Capricorn(9) alone, Ketu (Rahu in Scorpio(7)) in
    //     Taurus(1) with the Moon (Sun 0, Mercury 5, Jupiter 3, Venus 11,
    //     Saturn 10). By default the companion decides: (1 - 7) mod 12 = 6.
    //     Decisive takes Mars: 2.
    #[test]
    fn the_place_of_the_exalted_co_lord_is_a_parameter() {
        let a = chart(0, 3, 9, 5, 6, 11, 10, 2);
        assert_eq!(a.ketu(), 8);
        assert_eq!(years(7, a), 1);
        assert_eq!(years_with(7, a, DualLordRules::default()), 1);
        assert_eq!(years_with(7, a, DECISIVE), 2);
        let b = chart(0, 1, 9, 5, 3, 11, 10, 7);
        assert_eq!(b.ketu(), 1);
        assert_eq!(years(7, b), 6);
        assert_eq!(years_with(7, b, DECISIVE), 2);
    }

    // The reading changes only the two-lord signs: in chart (a) above, with
    // Saturn at home in Aquarius the count there runs to Rahu either way, so
    // only Scorpio's duration differs between the two readings.
    #[test]
    fn the_readings_differ_only_in_the_two_lord_signs() {
        let a = chart(0, 3, 9, 5, 6, 11, 10, 2);
        let by_sign = |rules: DualLordRules| {
            let mut out = [0.0f64; 12];
            for p in compute_chara_with(10, TEST_JD, a, rules) {
                out[p.sign_index as usize] = p.duration_years;
            }
            out
        };
        let default = by_sign(DualLordRules::default());
        let decisive = by_sign(DECISIVE);
        for sign in 0..12 {
            if sign == 7 {
                assert_eq!((default[sign], decisive[sign]), (1.0, 2.0));
            } else {
                assert_eq!(default[sign], decisive[sign], "sign {sign}");
            }
        }
    }

    // `compute_chara` is `compute_chara_with` under the default rules, bit
    // for bit, on charts where the dual-lord signs are decided by different
    // tests.
    #[test]
    fn compute_chara_is_the_default_rules() {
        for pos in [CHART_A, chart(0, 3, 9, 5, 6, 11, 10, 2), ALL_ARIES] {
            for lagna in 0..12u8 {
                let plain = compute_chara(lagna, TEST_JD, pos);
                let with = compute_chara_with(lagna, TEST_JD, pos, DualLordRules::default());
                assert_eq!(plain.len(), with.len());
                for (a, b) in plain.iter().zip(&with) {
                    assert_eq!(a.sign_index, b.sign_index);
                    assert_eq!(a.start_jd.to_bits(), b.start_jd.to_bits());
                    assert_eq!(a.end_jd.to_bits(), b.end_jd.to_bits());
                    assert_eq!(a.duration_years.to_bits(), b.duration_years.to_bits());
                }
            }
        }
    }

    // The longer count always separates two different rashis: the count
    // from a sign to each of the eleven others takes every value from 1 to
    // 11 once, in either direction. This is why the cascade needs no
    // position tie-break, and why an exaltation test placed behind the
    // longer count could never apply.
    #[test]
    fn counts_to_different_rashis_are_different() {
        for sign in 0u8..12 {
            let mut counts: Vec<u8> = (0u8..12)
                .filter(|&l| l != sign)
                .map(|l| duration_for(sign, l))
                .collect();
            counts.sort_unstable();
            assert_eq!(counts, (1u8..=11).collect::<Vec<_>>(), "sign {sign}");
        }
    }

    // Where the verses and the strength section both speak, they agree: when
    // the two co-lords' companions differ, and when they tie with no
    // difference of exaltation and a difference in the nature of the rashi,
    // the co-lord the strength section calls stronger is the one the count
    // runs to. A sweep of charts from a fixed generator, with the number of
    // cases of each kind counted so that the check cannot pass on none.
    #[test]
    fn agrees_with_the_strength_section_where_both_apply() {
        use crate::dasha::rashi_strength::{JaiminiStrength, NatalPlacements, Strength};
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % 12) as u8
        };
        let (mut by_companions, mut by_nature) = (0u32, 0u32);
        for _ in 0..6000 {
            let pos = chart(
                next(),
                next(),
                next(),
                next(),
                next(),
                next(),
                next(),
                next(),
            );
            let placements = NatalPlacements::from_signs(0, pos);
            let jaimini = JaiminiStrength::new(&placements);
            for sign in [7u8, 10] {
                let &[first, second] = rashi_lords(sign) else {
                    unreachable!()
                };
                let (sa, sb) = (placements.sign_of(first), placements.sign_of(second));
                if sa == sign || sb == sign {
                    continue;
                }
                let (ca, cb) = (
                    placements.companions_of(first),
                    placements.companions_of(second),
                );
                let exalted = |g: Graha| placements.sign_of(g) == exaltation_sign(g);
                let separated_by_companions = ca != cb;
                let separated_by_nature = ca == cb
                    && exalted(first) == exalted(second)
                    && natural_rank(sa) != natural_rank(sb);
                if !(separated_by_companions || separated_by_nature) {
                    continue;
                }
                by_companions += u32::from(separated_by_companions);
                by_nature += u32::from(separated_by_nature);
                let expected = placements.sign_of(jaimini.stronger_graha(first, second));
                assert_eq!(lord_sign_for(sign, pos), expected, "sign {sign}, {pos:?}");
            }
        }
        assert!(by_companions > 500, "only {by_companions} companion cases");
        assert!(by_nature > 100, "only {by_nature} nature cases");
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

    // ── Antardashas ──────────────────────────────────────────────────────

    use crate::dasha::rashi_antar::{AntarOrder, AntarStart};

    // 19. The mahadashas of `compute_chara_antar` are `compute_chara`'s,
    // bit for bit, for every lagna and two charts.
    #[test]
    fn antar_variant_leaves_the_mahadashas_untouched() {
        for chart in [ALL_ARIES, CHART_A] {
            for lagna in 0..12u8 {
                let plain = compute_chara(lagna, TEST_JD, chart);
                let full = compute_chara_with_antars(lagna, TEST_JD, chart);
                assert_eq!(plain.len(), full.len());
                for (a, b) in plain.iter().zip(&full) {
                    assert_eq!(a.sign_index, b.sign_index);
                    assert_eq!(a.sign_name, b.sign_name);
                    assert_eq!(a.start_jd.to_bits(), b.start_jd.to_bits());
                    assert_eq!(a.end_jd.to_bits(), b.end_jd.to_bits());
                    assert_eq!(a.duration_years.to_bits(), b.duration_years.to_bits());
                }
            }
        }
    }

    // 20. Each mahadasha's twelve antars tile it exactly, each is a twelfth
    // of it, and together they name all twelve rashis once.
    #[test]
    fn antars_tile_each_mahadasha_and_name_every_rashi_once() {
        for lagna in 0..12u8 {
            for p in compute_chara_with_antars(lagna, TEST_JD, CHART_A) {
                assert_eq!(p.antar.len(), 12);
                assert_eq!(p.antar[0].start_jd, p.start_jd);
                assert_eq!(p.antar[11].end_jd, p.end_jd);
                let mut seen = [false; 12];
                for (k, a) in p.antar.iter().enumerate() {
                    assert_eq!(a.duration_years, p.duration_years / 12.0);
                    if k > 0 {
                        assert_eq!(a.start_jd, p.antar[k - 1].end_jd);
                    }
                    assert!(!seen[usize::from(a.sign_index)]);
                    seen[usize::from(a.sign_index)] = true;
                }
            }
        }
    }

    // 21. CHART_A (Aquarius lagna) with ch. 51's antar rule.
    // The mahadashas run Aquarius (8 years), Pisces (1), Aries (1), ...
    //
    // Aquarius is fixed and odd: from its own rashi, every sixth, forward:
    // 10 3 8 1 6 11 4 9 2 7 0 5. Aquarius, not Leo (its seventh), is the
    // start: Aquarius holds Mercury, Jupiter and Venus, Leo only Saturn.
    //
    // Pisces is dual and even: itself, then kendras in reverse, then the
    // groups from its fifth and ninth in reverse: 11 8 5 2 7 4 1 10 3 0 9 6.
    // Pisces and Virgo (its seventh) hold no graha and are both dual; their
    // lords, Jupiter and Mercury, each stand in Aquarius with two companions
    // and neither is exalted, so no test of 2.3.6 to 2.3.10 separates the two
    // rashis. Neither lord stands in its own rashi and both rashis are even,
    // so the comparison is a total tie, which keeps the dasha rashi first.
    //
    // Aries is movable and odd: 0 1 2 ... 11. Aries and Libra are both
    // empty movable signs; Mars (Aries's lord) stands in Taurus with no
    // companion and Venus (Libra's lord) in Aquarius with two, so Libra's
    // lord is the stronger by 2.3.10 and the series starts at Libra: 6 7 8 9
    // 10 11 0 1 2 3 4 5.
    #[test]
    fn bphs_51_worked_examples_through_the_default_rule() {
        let periods = compute_chara_with_antars(10, TEST_JD, CHART_A);
        let signs = |p: &RashiPeriod| p.antar.iter().map(|a| a.sign_index).collect::<Vec<_>>();
        assert_eq!(periods[0].sign_index, 10);
        assert_eq!(
            signs(&periods[0]),
            vec![10, 3, 8, 1, 6, 11, 4, 9, 2, 7, 0, 5]
        );
        // Eight years in twelve equal parts is eight months each.
        assert_eq!(periods[0].antar[0].duration_years * 12.0, 8.0);
        assert_eq!(periods[1].sign_index, 11);
        assert_eq!(
            signs(&periods[1]),
            vec![11, 8, 5, 2, 7, 4, 1, 10, 3, 0, 9, 6]
        );
        assert_eq!(periods[2].sign_index, 0);
        assert_eq!(
            signs(&periods[2]),
            vec![6, 7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5]
        );
    }

    // 22. The start rules differ where the two BPHS chapters conflict.
    // Aquarius's lord resolves to Rahu in Gemini(2) for this chart (see the
    // duration tests), so the lord-occupied start gives every sixth from
    // Gemini: 2 7 0 5 10 3 8 1 6 11 4 9. The itself-always start gives the
    // worked example; consecutive order ignores the sixth-sixth shape.
    #[test]
    fn antar_rules_change_the_series() {
        let placements = NatalPlacements::from_signs(10, CHART_A);
        let strength = JaiminiStrength::new(&placements);
        let series = |rules: AntarRules| {
            compute_chara_antar(10, TEST_JD, CHART_A, rules, &strength)[0]
                .antar
                .iter()
                .map(|a| a.sign_index)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            series(AntarRules {
                start: AntarStart::LordOccupied,
                order: AntarOrder::ByModality,
            }),
            vec![2, 7, 0, 5, 10, 3, 8, 1, 6, 11, 4, 9]
        );
        assert_eq!(
            series(AntarRules {
                start: AntarStart::RashiItself,
                order: AntarOrder::ByModality,
            }),
            vec![10, 3, 8, 1, 6, 11, 4, 9, 2, 7, 0, 5]
        );
        assert_eq!(
            series(AntarRules {
                start: AntarStart::RashiItself,
                order: AntarOrder::Consecutive,
            }),
            vec![10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }
}
