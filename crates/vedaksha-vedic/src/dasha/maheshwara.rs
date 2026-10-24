// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Maheshwara (Jaimini) — the graha of the eighth from the atmakaraka, and
//! what the sutras read from the sthira dasha with it.
//!
//! The numbering and grading conventions are those of
//! [`super::rashi_strength`]. The Upadesha Sutras (adhyaya 2, pada 1) name
//! three deities among the grahas: Rudra, Brahma ([`super::sthira`]) and
//! Maheshwara. BPHS has no Maheshwara.
//!
//! # Who is Maheshwara
//!
//! - **2.1.49**, *sva-bhava-isho maheshvarah*: the lord of the eighth
//!   (*bhava*, coded eight) from *sva*, the atmakaraka's rashi. A rashi with
//!   two lords (Scorpio, Aquarius) is represented by the stronger. Grade (a).
//! - **2.1.50**, *svochche sa-grahe ripu-bhava-isha-prani*: when the
//!   atmakaraka's rashi of exaltation holds grahas, the stronger of the lords
//!   of the twelfth (*ripu*) and the eighth from the atmakaraka. Grade (b):
//!   the critical edition's commentary reads *sa-grahe* as the exaltation
//!   rashi holding a graha, and its English as one *other* than the
//!   atmakaraka; [`MaheshwaraRules::count_atmakaraka`] admits the atmakaraka
//!   itself. A manuscript variant, *svabhe*, "in its own rashi", is
//!   recorded in the edition and not implemented.
//! - **2.1.51**, *patabhyam yoge svasya tayor va roge tatah*: when the
//!   atmakaraka is conjoined with Rahu or Ketu, or a node stands in the
//!   eighth (*roga*) from it, the substitute *tatah* names. Grade (b): the
//!   word is the open point, and the commentary offers three readings.
//!   [`MaheshwaraSubstitute`] carries them. The default is the sixth graha
//!   from the atmakaraka, because the same formula at 2.1.53 (*brahmani
//!   shanau patayor va tatah*) is read "the sixth graha from him" by BPHS
//!   (46.172, *tat-shashtha-khecharah*), and because decoding *tatah* with the
//!   letter-to-numeral rule the sutras state (*sarvatra savarna bhava
//!   rashayash cha*; see `docs/audit/2026-08-31-chara-dasha-panchame-
//!   cipher.md`) gives 66, which is 6 mod 12. The reading that repeats
//!   2.1.50's pair is the critical edition's own translation. If both 2.1.50
//!   and 2.1.51 apply, 2.1.51, the later and more specific, governs.
//!
//! # What the sutras read from it
//!
//! - **2.1.58**, *brahmano yavan maheshvararksha-dasha-antam ayuh*: the span
//!   of life runs from the dasha of Brahma's rashi to the end of the dasha
//!   of the rashi Maheshwara occupies, in the sthira dasha. Grade (a) for
//!   the statement; (b) for "the dasha": the commentary says the dashas of
//!   the chara, sthira and other kinds all serve, and the sthira kind is the
//!   one this module computes.
//! - **2.1.59**, *tatrapi maheshvara-bhava-isha-trikona-abde*: within that
//!   last dasha, death falls in the antardasha ("year", *abda*) of the
//!   trikona of the rashi where the lord of the eighth from Maheshwara
//!   stands. Grade (b): *abda* is an antardasha; the commentary reads it as
//!   one year of a nine-year dasha divided into nine, "some" holding that
//!   whatever the antardasha's computed length it is one year here. The
//!   antardasha series are the twelfths of [`super::rashi_antar`] and
//!   [`MaheshwaraDasha::death_antars`] marks those whose rashis are the
//!   trikona; the one-year reading is not computed.
//!
//! [`compute_maheshwara`] returns exactly this. It is not a dasha with
//! periods of its own, and no sutra gives it one.
//!
//! # The commentators' twelve-year window (a convention)
//!
//! A reading in modern circulation takes a twelve-year window opening when
//! the *chara* dasha reaches the eighth rashi from the atmakaraka, with
//! twelve one-year antardashas cycling the three rashis of that rashi's
//! trikona. **No sutra or verse states it.** It is offered as
//! [`commentator_window`], a separate function a caller opts into, with the
//! chara periods and the rashi supplied by the caller. Grade (c).

use serde::{Deserialize, Serialize};

use super::chara::SIGN_NAMES;
use super::rashi_antar::{DASHA_YEAR_DAYS, DashaDirection, RashiAntar, RashiPeriod, walk};
use super::rashi_strength::{
    NatalPlacements, Strength, effective_lord, exaltation_sign, graha_after,
};
use super::shoola::trikona_of;
use super::sthira::{SthiraOptions, compute_sthira};
use crate::graha::Graha;

/// What replaces the eighth lord when 2.1.51 applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MaheshwaraSubstitute {
    /// The sixth graha from the atmakaraka, counting it as the first through
    /// Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu. The
    /// default.
    #[default]
    SixthFromAtmakaraka,
    /// The stronger of the lords of the eighth and twelfth from the
    /// atmakaraka (the critical edition's translation).
    StrongerOfEighthAndTwelfthLords,
    /// The third graha from the atmakaraka, in the same cycle (the
    /// commentary's reading of *tatah* modulo nine).
    ThirdFromAtmakaraka,
}

/// The rules [`maheshwara_graha`] leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MaheshwaraRules {
    /// What 2.1.51 substitutes.
    pub substitute: MaheshwaraSubstitute,
    /// Whether the atmakaraka itself, standing in its own rashi of
    /// exaltation, makes that rashi "hold grahas" for 2.1.50. Off by default.
    pub count_atmakaraka: bool,
}

/// The Maheshwara graha.
///
/// `atmakaraka` is the graha of highest degree in its rashi among the chara
/// karakas (see `karaka::compute_karakas`).
#[must_use]
pub fn maheshwara_graha(
    placements: &NatalPlacements,
    atmakaraka: Graha,
    strength: &dyn Strength,
    rules: MaheshwaraRules,
) -> Graha {
    let p = placements;
    let ak_sign = p.sign_of(atmakaraka);
    let eighth = walk(ak_sign, DashaDirection::Zodiacal, 7);
    let twelfth = walk(ak_sign, DashaDirection::Zodiacal, 11);
    let eighth_lord = effective_lord(eighth, strength);
    let twelfth_lord = effective_lord(twelfth, strength);
    // A tie keeps the eighth lord, the default of 2.1.49.
    let stronger_pair = || strength.stronger_graha(eighth_lord, twelfth_lord);

    // 2.1.51: the atmakaraka with a node, or a node in the eighth from it.
    let node_with_atmakaraka = [Graha::Rahu, Graha::Ketu]
        .iter()
        .any(|&n| n != atmakaraka && p.sign_of(n) == ak_sign);
    let node_in_eighth = [Graha::Rahu, Graha::Ketu]
        .iter()
        .any(|&n| p.sign_of(n) == eighth);
    if node_with_atmakaraka || node_in_eighth {
        return match rules.substitute {
            MaheshwaraSubstitute::SixthFromAtmakaraka => graha_after(atmakaraka, 5),
            MaheshwaraSubstitute::ThirdFromAtmakaraka => graha_after(atmakaraka, 2),
            MaheshwaraSubstitute::StrongerOfEighthAndTwelfthLords => stronger_pair(),
        };
    }

    // 2.1.50: the atmakaraka's rashi of exaltation holds grahas.
    let exaltation_rashi = exaltation_sign(atmakaraka);
    let holds_grahas = p
        .occupants_of(exaltation_rashi)
        .iter()
        .any(|&g| rules.count_atmakaraka || g != atmakaraka);
    if holds_grahas {
        return stronger_pair();
    }

    // 2.1.49.
    eighth_lord
}

/// Everything [`compute_maheshwara`] leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MaheshwaraOptions {
    /// Who Maheshwara is.
    pub rules: MaheshwaraRules,
    /// The sthira dasha the life span is read from, and its antardashas.
    pub sthira: SthiraOptions,
}

/// What 2.1.58 and 2.1.59 read from the sthira dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaheshwaraDasha {
    /// The Maheshwara graha.
    pub maheshwara: Graha,
    /// The rashi Maheshwara occupies, zero-based.
    pub rashi: u8,
    /// The lord of the eighth from `rashi`.
    pub eighth_lord: Graha,
    /// The rashi that lord occupies.
    pub eighth_lord_rashi: u8,
    /// The trikona of `eighth_lord_rashi`: the rashis whose antardashas, in
    /// the dasha of `rashi`, can carry death.
    pub death_trikona: [u8; 3],
    /// The span of life: the years of the sthira dasha from its first period
    /// through the end of the period of `rashi` (2.1.58).
    pub life_span_years: f64,
    /// The sthira period of `rashi`, with its antardashas.
    pub period: RashiPeriod,
    /// Indices into `period.antar` of the antardashas whose rashi is in
    /// `death_trikona` (2.1.59).
    pub death_antars: Vec<usize>,
}

/// Maheshwara, the span of life to the end of his rashi's sthira period, and
/// the antardashas of that period that can carry death.
///
/// `None` only if the sthira anchor is the Brahma graha
/// ([`SthiraAnchor::Brahma`](super::sthira::SthiraAnchor::Brahma)) and the
/// chart has none.
#[must_use]
pub fn compute_maheshwara(
    placements: &NatalPlacements,
    atmakaraka: Graha,
    birth_jd: f64,
    options: &MaheshwaraOptions,
    strength: &dyn Strength,
) -> Option<MaheshwaraDasha> {
    let maheshwara = maheshwara_graha(placements, atmakaraka, strength, options.rules);
    let rashi = placements.sign_of(maheshwara);
    let eighth_rashi = walk(rashi, DashaDirection::Zodiacal, 7);
    let eighth_lord = effective_lord(eighth_rashi, strength);
    let eighth_lord_rashi = placements.sign_of(eighth_lord);
    let death_trikona = trikona_of(eighth_lord_rashi);

    let sthira = compute_sthira(placements, atmakaraka, birth_jd, &options.sthira, strength)?;
    let index = sthira.periods.iter().position(|p| p.sign_index == rashi)?;
    let life_span_years = sthira.periods[..=index]
        .iter()
        .map(|p| p.duration_years)
        .sum();
    let period = sthira.periods[index].clone();
    let death_antars = period
        .antar
        .iter()
        .enumerate()
        .filter(|(_, a)| death_trikona.contains(&a.sign_index))
        .map(|(i, _)| i)
        .collect();
    Some(MaheshwaraDasha {
        maheshwara,
        rashi,
        eighth_lord,
        eighth_lord_rashi,
        death_trikona,
        life_span_years,
        period,
        death_antars,
    })
}

/// The twelve-year window of the commentators' convention (see the module
/// doc; no text states it).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentatorWindow {
    /// The rashi the window is of, zero-based.
    pub rashi: u8,
    /// Start: the start of that rashi's period in the chara dasha.
    pub start_jd: f64,
    /// End: twelve years later.
    pub end_jd: f64,
    /// Twelve one-year antardashas cycling the trikona of `rashi`: forward
    /// from an odd rashi, in reverse from an even one.
    pub antars: Vec<RashiAntar>,
}

/// The commentators' window for `rashi`, over the caller's chara periods
/// (from `chara::compute_chara_with_antars` or a subset of it). The
/// convention takes `rashi` to be the eighth from the atmakaraka. `None` if
/// `rashi` has no period in `chara`.
#[must_use]
pub fn commentator_window(chara: &[RashiPeriod], rashi: u8) -> Option<CommentatorWindow> {
    let rashi = rashi % 12;
    let opening = chara.iter().find(|p| p.sign_index == rashi)?;
    let direction = DashaDirection::by_parity(rashi);
    let year = DASHA_YEAR_DAYS;
    // The trikona walked in the dasha's own direction (reverse from an even
    // rashi), not `trikona_of`'s forward one.
    let rashis = [rashi, walk(rashi, direction, 4), walk(rashi, direction, 8)];
    let antars = (0..12u8)
        .map(|k| {
            let sign = rashis[usize::from(k) % 3];
            RashiAntar {
                sign_index: sign,
                sign_name: SIGN_NAMES[usize::from(sign)].to_string(),
                start_jd: opening.start_jd + f64::from(k) * year,
                end_jd: opening.start_jd + f64::from(k + 1) * year,
                duration_years: 1.0,
            }
        })
        .collect();
    Some(CommentatorWindow {
        rashi,
        start_jd: opening.start_jd,
        end_jd: opening.start_jd + 12.0 * year,
        antars,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::chara::{GrahaSigns, compute_chara_with_antars};
    use crate::dasha::rashi_strength::JaiminiStrength;
    use crate::dasha::sthira::SthiraAnchor;

    const TEST_JD: f64 = 2_451_545.0;

    fn chart(lagna: u8, s: [u8; 8]) -> NatalPlacements {
        NatalPlacements::from_signs(
            lagna,
            GrahaSigns {
                sun: s[0],
                moon: s[1],
                mars: s[2],
                mercury: s[3],
                jupiter: s[4],
                venus: s[5],
                saturn: s[6],
                rahu: s[7],
            },
        )
    }

    fn maheshwara(p: &NatalPlacements, ak: Graha, rules: MaheshwaraRules) -> Graha {
        maheshwara_graha(p, ak, &JaiminiStrength::new(p), rules)
    }

    // Chart A. Lagna Aries. The atmakaraka is the Sun, in Gemini(2). Moon 3,
    // Mars 5, Mercury 6, Jupiter 7, Venus 8, Saturn 11, Rahu 4 (Ketu 10).
    // The eighth from Gemini is Capricorn(9), lord Saturn; the twelfth is
    // Taurus(1), lord Venus. The Sun's rashi of exaltation, Aries(0), is
    // empty; Rahu is in Leo and Ketu in Aquarius, neither with the Sun nor in
    // Capricorn. So 2.1.49 alone: Saturn.
    fn chart_a() -> NatalPlacements {
        chart(0, [2, 3, 5, 6, 7, 8, 11, 4])
    }

    // 1. 2.1.49.
    #[test]
    fn maheshwara_is_the_eighth_lord_from_the_atmakaraka() {
        assert_eq!(
            maheshwara(&chart_a(), Graha::Sun, MaheshwaraRules::default()),
            Graha::Saturn
        );
    }

    // 2. 2.1.50. Chart B is Chart A with Mars and Venus in Aries(0), the
    // Sun's rashi of exaltation. Now the stronger of the eighth lord Saturn
    // (Pisces, alone) and the twelfth lord Venus (Aries, with Mars: one
    // companion) is Venus. The Sun itself standing in Aries instead would
    // not count by default.
    #[test]
    fn a_graha_in_the_exaltation_rashi_brings_in_the_twelfth_lord() {
        let b = chart(0, [2, 3, 0, 6, 7, 0, 11, 4]);
        assert_eq!(
            maheshwara(&b, Graha::Sun, MaheshwaraRules::default()),
            Graha::Venus
        );
        // Sun in Aries is the atmakaraka itself: 8th from Aries is Scorpio,
        // lords Mars and Ketu. Moon 3, Mars 7 (in Scorpio), Mercury 6,
        // Jupiter 5, Venus 8, Saturn 11, Rahu 4 (Ketu 10). Nobody else is in
        // Aries.
        let c = chart(0, [0, 3, 7, 6, 5, 8, 11, 4]);
        let by_default = maheshwara(&c, Graha::Sun, MaheshwaraRules::default());
        let counting_itself = maheshwara(
            &c,
            Graha::Sun,
            MaheshwaraRules {
                count_atmakaraka: true,
                ..MaheshwaraRules::default()
            },
        );
        // Default: 2.1.49 -> effective lord of Scorpio: Mars (alone in
        // Scorpio, 0 companions) against Ketu (Aquarius, alone): tie on
        // companions, tie on exaltation, Mars in a fixed rashi and Ketu in a
        // fixed rashi tie; the first-named lord, Mars.
        assert_eq!(by_default, Graha::Mars);
        // Counting the Sun: the stronger of the eighth lord Mars and the
        // twelfth lord (Pisces: Jupiter in Virgo, alone): Mars in a fixed
        // rashi against Jupiter in a dual one: Jupiter.
        assert_eq!(counting_itself, Graha::Jupiter);
    }

    // 3. 2.1.51 with each reading. Chart D puts Rahu with the Sun in
    // Gemini(2); Ketu is in Sagittarius. Sixth from the Sun counting it as
    // first: Sun Moon Mars Mercury Jupiter Venus: Venus. Third: Mars. The
    // stronger of the eighth lord (Saturn, Capricorn's lord, in Pisces) and
    // the twelfth lord (Venus, Taurus's lord, in Capricorn): both alone and
    // not exalted; Pisces is dual and Capricorn movable: Saturn.
    #[test]
    fn nodes_with_the_atmakaraka_use_the_substitute() {
        let d = chart(0, [2, 3, 5, 6, 7, 9, 11, 2]);
        let with = |substitute| {
            maheshwara(
                &d,
                Graha::Sun,
                MaheshwaraRules {
                    substitute,
                    ..MaheshwaraRules::default()
                },
            )
        };
        assert_eq!(
            with(MaheshwaraSubstitute::SixthFromAtmakaraka),
            Graha::Venus
        );
        assert_eq!(with(MaheshwaraSubstitute::ThirdFromAtmakaraka), Graha::Mars);
        assert_eq!(
            with(MaheshwaraSubstitute::StrongerOfEighthAndTwelfthLords),
            Graha::Saturn
        );
        assert_eq!(
            maheshwara(&d, Graha::Sun, MaheshwaraRules::default()),
            Graha::Venus
        );
    }

    // 4. 2.1.51 also fires on a node in the eighth from the atmakaraka:
    // Rahu in Capricorn(9), the eighth from the Sun in Gemini (Ketu in
    // Cancer). The default substitute is again the sixth from the Sun: Venus.
    #[test]
    fn a_node_in_the_eighth_uses_the_substitute() {
        let e = chart(0, [2, 7, 5, 6, 4, 8, 11, 9]);
        assert_eq!(
            maheshwara(&e, Graha::Sun, MaheshwaraRules::default()),
            Graha::Venus
        );
    }

    // 4b. The request's two births, re-derived by hand under this module's
    // reading (not the request's own numbering).
    //
    // F1: lagna Leo, atmakaraka Mercury in Aries(0); Sun and Jupiter in
    // Taurus, Moon in Libra, Mars, Mercury and Venus in Aries, Saturn in
    // Cancer, Rahu in Virgo, Ketu in Pisces. The eighth from Aries is
    // Scorpio, lords Mars and Ketu. No node is with Mercury or in Scorpio.
    // Mercury's rashi of exaltation, Virgo(5), holds Rahu, so 2.1.50 applies:
    // the stronger of the eighth lord and the twelfth lord (Pisces: Jupiter).
    // The eighth lord is the stronger of Mars (Aries, with Mercury and Venus:
    // two companions) and Ketu (Pisces, alone): Mars. Against Jupiter
    // (Taurus, with the Sun: one companion), Mars has more. Maheshwara is
    // Mars. (The request guessed Jupiter if its 2.1.45 applied; here the
    // companions decide for Mars.)
    //
    // F2: lagna Pisces, atmakaraka the Moon in Virgo(5); Sun, Moon and
    // Mercury in Virgo, Mars in Libra, Jupiter and Venus in Leo, Saturn in
    // Capricorn, Rahu in Sagittarius, Ketu in Gemini. The eighth from Virgo
    // is Aries, lord Mars. The Moon's exaltation rashi, Taurus, is empty and
    // no node is with the Moon or in Aries: 2.1.49 alone, Mars.
    #[test]
    fn request_births_f1_and_f2() {
        let f1 = chart(4, [1, 6, 0, 0, 1, 0, 3, 5]);
        assert_eq!(
            maheshwara(&f1, Graha::Mercury, MaheshwaraRules::default()),
            Graha::Mars
        );
        let f2 = chart(11, [5, 5, 6, 5, 4, 4, 9, 8]);
        assert_eq!(
            maheshwara(&f2, Graha::Moon, MaheshwaraRules::default()),
            Graha::Mars
        );
    }

    // 5. The span of life and the death antars, end to end. Chart A (Saturn
    // is Maheshwara, in Pisces(11)), the sthira dasha anchored on the lagna
    // (Aries, odd, forward): Aries 7, Taurus 8, Gemini 9, Cancer 7, Leo 8,
    // Virgo 9, Libra 7, Scorpio 8, Sagittarius 9, Capricorn 7, Aquarius 8,
    // Pisces 9. Pisces is the last, so the span is all 96 years and its period
    // starts at 87 years.
    //
    // The eighth from Pisces is Libra(6), whose lord Venus stands in
    // Sagittarius(8): the death trikona is Sagittarius, Aries, Leo (8, 0, 4).
    //
    // The antars of the 9-year Pisces period start at Pisces or Virgo,
    // whichever is stronger. Pisces holds Saturn and Virgo holds Mars, one
    // graha each, neither exalted, both dual; their lords Jupiter (Scorpio,
    // alone) and Mercury (Libra, alone) tie on companions and exaltation, and
    // Jupiter's rashi is fixed against Mercury's movable, so Jupiter, Pisces's
    // lord, is stronger: the series starts at Pisces. Pisces is dual and even,
    // so reverse: 11 8 5 2 7 4 1 10 3 0 9 6. Those in {8, 0, 4} are at
    // positions 1 (8), 5 (4), 9 (0).
    #[test]
    fn life_span_and_death_antars() {
        let p = chart_a();
        let options = MaheshwaraOptions {
            sthira: SthiraOptions {
                anchor: SthiraAnchor::Lagna,
                ..SthiraOptions::default()
            },
            ..MaheshwaraOptions::default()
        };
        let m = compute_maheshwara(&p, Graha::Sun, TEST_JD, &options, &JaiminiStrength::new(&p))
            .unwrap();
        assert_eq!(m.maheshwara, Graha::Saturn);
        assert_eq!(m.rashi, 11);
        assert_eq!(m.eighth_lord, Graha::Venus);
        assert_eq!(m.eighth_lord_rashi, 8);
        assert_eq!(m.death_trikona, [8, 0, 4]);
        assert_eq!(m.life_span_years, 96.0);
        assert_eq!(m.period.sign_index, 11);
        assert_eq!(m.period.start_jd, TEST_JD + 87.0 * 365.25);
        let series: Vec<u8> = m.period.antar.iter().map(|a| a.sign_index).collect();
        assert_eq!(series, vec![11, 8, 5, 2, 7, 4, 1, 10, 3, 0, 9, 6]);
        assert_eq!(m.death_antars, vec![1, 5, 9]);
    }

    // 6. The span is the years up to and including Maheshwara's period: from
    // an explicit Sagittarius(8) anchor (odd, forward) the order is
    // Sagittarius 9, Capricorn 7, Aquarius 8, Pisces 9, ...: Pisces is the
    // fourth, so 9 + 7 + 8 + 9 = 33 years, and its period starts at 24.
    #[test]
    fn life_span_runs_to_the_end_of_maheshwaras_period() {
        let p = chart_a();
        let options = MaheshwaraOptions {
            sthira: SthiraOptions {
                anchor: SthiraAnchor::Explicit(8),
                ..SthiraOptions::default()
            },
            ..MaheshwaraOptions::default()
        };
        let m = compute_maheshwara(&p, Graha::Sun, TEST_JD, &options, &JaiminiStrength::new(&p))
            .unwrap();
        assert_eq!(m.life_span_years, 33.0);
        assert_eq!(m.period.start_jd, TEST_JD + 24.0 * 365.25);
        assert_eq!(m.period.end_jd, TEST_JD + 33.0 * 365.25);
    }

    // 7. No Brahma, no span: the default anchor propagates the `None`.
    #[test]
    fn no_brahma_gives_no_dasha() {
        // Every graha in Aries but Rahu, in Cancer (so Ketu is in
        // Capricorn, an even rashi): the lords of the sixth, eighth and
        // twelfth from an Aries lagna (Mercury, Mars and Ketu, Jupiter) stand
        // in Aries, outside the default window, or in an even rashi, and the
        // lords of the eighth from the atmakaraka (Scorpio's Mars and Ketu)
        // are not in Scorpio.
        let p = chart(0, [0, 0, 0, 0, 0, 0, 0, 3]);
        assert!(
            compute_maheshwara(
                &p,
                Graha::Sun,
                TEST_JD,
                &MaheshwaraOptions::default(),
                &JaiminiStrength::new(&p)
            )
            .is_none()
        );
    }

    // 8. The commentators' window. Its antar cycles are the ones the request
    // gives for its two births (Scorpio, Cancer, Pisces repeating; Aries,
    // Leo, Sagittarius repeating). The chara dasha here is from an Aries
    // lagna with every graha in Aries. The window of a rashi opens at its
    // chara period and lasts twelve years of twelve one-year antars cycling
    // its trikona: Scorpio(7), even, in reverse: 7 3 11 repeating; Aries(0),
    // odd, forward: 0 4 8 repeating.
    #[test]
    fn commentator_window_cycles_the_trikona() {
        let positions = GrahaSigns {
            sun: 0,
            moon: 0,
            mars: 0,
            mercury: 0,
            jupiter: 0,
            venus: 0,
            saturn: 0,
            rahu: 0,
        };
        let chara = compute_chara_with_antars(0, TEST_JD, positions);
        let scorpio = chara.iter().find(|p| p.sign_index == 7).unwrap();
        let w = commentator_window(&chara, 7).unwrap();
        assert_eq!(w.start_jd, scorpio.start_jd);
        assert_eq!(w.end_jd, scorpio.start_jd + 12.0 * 365.25);
        let signs: Vec<u8> = w.antars.iter().map(|a| a.sign_index).collect();
        assert_eq!(signs, vec![7, 3, 11, 7, 3, 11, 7, 3, 11, 7, 3, 11]);
        assert!(w.antars.iter().all(|a| a.duration_years == 1.0));
        assert_eq!(w.antars[0].start_jd, w.start_jd);
        assert_eq!(w.antars[11].end_jd, w.end_jd);
        let w0 = commentator_window(&chara, 0).unwrap();
        let signs0: Vec<u8> = w0.antars.iter().map(|a| a.sign_index).collect();
        assert_eq!(signs0, vec![0, 4, 8, 0, 4, 8, 0, 4, 8, 0, 4, 8]);
        assert_eq!(w0.start_jd, TEST_JD);
        assert!(commentator_window(&chara[..3], 7).is_none());
    }
}
