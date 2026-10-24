// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Sthira Dasha (Jaimini) and the Brahma graha it starts from.
//!
//! Sthira dasha gives each rashi a fixed number of years by its modality, and
//! runs the twelve rashis from the rashi of the Brahma graha. The numbering
//! and grading conventions are those of [`super::rashi_strength`]; the
//! antardasha rule is that of [`super::rashi_antar`].
//!
//! # The rules and what the text says of each
//!
//! - **Years.** 7 for a movable rashi, 8 for a fixed one, 9 for a dual one:
//!   *shashi-nanda-pavakah kramad abdah sthira-dashayam* (Jaimini 2.3.3),
//!   which the commentary reads Aries 7, Taurus 8, Gemini 9; BPHS ch. 46
//!   (the dasha chapter, in the recension this crate follows throughout)
//!   sloka 168, *chare sapta sthire chashtau* ... *samah smritah*.
//!   Grade (a). There is no lord-distance count in this dasha, and twelve
//!   periods make 96 years.
//! - **Anchor.** *brahma-adir esha* (2.3.4): the dasha begins from the rashi
//!   the Brahma graha occupies; BPHS 46.169, *brahma-kheta-ashrita-riksha-
//!   adi*. Grade (a). Some commentators anchor on the lagna or the
//!   karakamsha instead; [`SthiraAnchor::Lagna`] and
//!   [`SthiraAnchor::Explicit`] carry those and any other reading, and the
//!   default is the text's.
//! - **Who is Brahma.** [`brahma_graha`] below; grade (b).
//! - **Which way it walks.** Neither 2.3.3, 2.3.4 nor BPHS 46.168 and 169
//!   states a direction in words. Grade (c) for every option:
//!   [`SthiraDirection::AnchorParity`] (the default) takes forward from an
//!   odd anchor and in reverse from an even one. It is what the same chapter
//!   states, in so many words, for each of the dashas it describes as having
//!   "years as in sthira" (BPHS 46.180 for mandooka, 46.182 for shoola,
//!   46.184 for trikona, and 46.174 for yogardha), what the English of the
//!   recension reads into 46.169, and what the critical edition's commentary
//!   gives for every antardasha series (2.3.1 and 2.3.2). The other options
//!   are rules in circulation: chara's vishama-pada test at the ninth from
//!   the lagna (2.3.32, which names the *chara* dasha), the modality of the
//!   anchor (forward from a movable or dual rashi, reverse from a fixed
//!   one), and an explicit direction.
//! - **Antardashas.** BPHS 51.5 to 51.9 give the antar rule, and slokas 11 and
//!   12 of the same chapter, on the paka and bhoga rashis, name "the chara
//!   paryaya, the sthira paryaya and the trikona dasha" as the dashas it
//!   covers; so the defaults of [`super::rashi_antar`] apply. Grade (a) for the rule,
//!   (c) for the proportional-length alternative.
//!
//! # The Brahma graha (Jaimini 2.1.52 to 2.1.57; BPHS 46.170 to 173)
//!
//! *prabhu-bhava-vairi-isha-prani pitri-labha-prani-anucaro vishamastho
//! brahma* (2.1.52): Brahma is the strong lord of the sixth, eighth or
//! twelfth, standing in an odd rashi, behind the stronger of the lagna and
//! its seventh. The pieces, as [`brahma_graha`] reads them:
//!
//! 1. The candidates are the lords of the sixth, eighth and twelfth rashis
//!    from the lagna (a rashi with two lords contributes both) that stand in
//!    an odd rashi inside the *window* behind the base. The base is the
//!    stronger of the lagna and its seventh. "Behind" is *prishtha-bhaga*;
//!    the commentary counts the seventh to twelfth houses as the back of the
//!    lagna and the first to sixth as the back of the seventh, i.e. the six
//!    rashis before the base ([`BrahmaWindow::BehindExclusive`], the
//!    default). The worked example printed with BPHS 46.171 to 173 counts the
//!    base rashi itself as the first of the six ([`BrahmaWindow::
//!    BehindInclusive`]). Grade (b): the sutra's *anucara* does not settle
//!    which. Treating the three marks (a lord of the sixth, eighth or
//!    twelfth; odd; behind) as filters, with "strong" as the arbiter between
//!    survivors, follows BPHS 46.173, *bahavo lakshanakranta*, "many endowed
//!    with the marks".
//! 2. *bahunam yoge svajatiyah* (2.1.54), BPHS 46.173 *adhikamshakah*: of
//!    several, the one with the most degrees in its rashi. *rahu-yoge
//!    viparitam* (2.1.55): if Rahu is among them, the reckoning is reversed
//!    and the one with the fewest degrees stands. Grade (b); a placement
//!    built without degrees skips this step.
//! 3. *vivade bali* (2.1.57): still tied, the stronger, by
//!    [`Strength`](super::rashi_strength::Strength).
//! 4. *brahma svabhavesho bhavasthah* (2.1.56): if the lord of the eighth
//!    from the atmakaraka stands in that eighth rashi, he is Brahma "in
//!    preference to any other", with no odd-rashi or window condition (BPHS
//!    46.172, *karakad ashtameshah va*). This is tried first. Grade (b): the
//!    commentary also allows reading *bhavasthah* as "or one standing in the
//!    eighth".
//! 5. *brahmani shanau patayor va tatah* (2.1.53), BPHS 46.172 *shanau pate
//!    cha brahmatve brahma tat-shashtha-khecharah*: if Saturn, Rahu or Ketu
//!    would be Brahma, the sixth graha from him is, counting him as the
//!    first through Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu,
//!    Ketu (Mars, Mercury and Jupiter respectively). Applied last, to a
//!    graha chosen by either route. Grade (a) for the rule, (b) for the
//!    order of application.
//!
//! The marks can leave no graha: this function then returns `None`, and
//! [`compute_sthira`] with the Brahma anchor returns `None` with it. The text
//! does not say what a chart without a Brahma uses; choose another anchor.

use serde::{Deserialize, Serialize};

use super::chara::SIGN_NAMES;
use super::rashi_antar::{
    AntarLength, AntarRules, DASHA_YEAR_DAYS, DashaDirection, RashiPeriod, build_antars,
    sthira_years, walk,
};
use super::rashi_strength::{
    GRAHA_CYCLE, Modality, NatalPlacements, Strength, graha_after, is_odd_sign, modality,
    rashi_lords,
};
use crate::graha::Graha;

/// Which rashis count as "behind" the stronger of the lagna and its seventh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrahmaWindow {
    /// The six rashis before the base: the base minus one through the base
    /// minus six. The critical edition's commentary on 2.1.52. The default.
    #[default]
    BehindExclusive,
    /// The base and the five before it, as the worked example printed with
    /// BPHS 46.171 to 173 counts.
    BehindInclusive,
}

/// The rules [`brahma_graha`] leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BrahmaRules {
    /// Which six rashis are "behind" the base.
    pub window: BrahmaWindow,
}

fn window_signs(base: u8, window: BrahmaWindow) -> [u8; 6] {
    let first = match window {
        BrahmaWindow::BehindExclusive => 1,
        BrahmaWindow::BehindInclusive => 0,
    };
    let mut out = [0u8; 6];
    for (k, slot) in out.iter_mut().enumerate() {
        *slot = walk(base, DashaDirection::AntiZodiacal, first + k as u8);
    }
    out
}

/// The Brahma graha of a chart, if the text's marks leave one.
///
/// `atmakaraka` is the graha of highest degree in its rashi among the chara
/// karakas (see `karaka::compute_karakas`). See the module doc for each step
/// and its grade.
#[must_use]
pub fn brahma_graha(
    placements: &NatalPlacements,
    atmakaraka: Graha,
    strength: &dyn Strength,
    rules: BrahmaRules,
) -> Option<Graha> {
    let p = placements;
    let lagna = p.lagna_sign();

    // 2.1.56: the lord of the eighth from the atmakaraka, standing in that
    // eighth, is Brahma before any other.
    let eighth_from_ak = walk(p.sign_of(atmakaraka), DashaDirection::Zodiacal, 7);
    let at_home: Vec<Graha> = rashi_lords(eighth_from_ak)
        .iter()
        .copied()
        .filter(|&l| p.sign_of(l) == eighth_from_ak)
        .collect();
    let chosen = if let Some(g) = fold_strongest(&at_home, strength) {
        g
    } else {
        // 2.1.52: lords of the sixth, eighth and twelfth, odd, behind.
        let base = strength.stronger_rashi(lagna, walk(lagna, DashaDirection::Zodiacal, 6));
        let window = window_signs(base, rules.window);
        let mut candidates: Vec<Graha> = Vec::new();
        for offset in [5u8, 7, 11] {
            let rashi = walk(lagna, DashaDirection::Zodiacal, offset);
            for &lord in rashi_lords(rashi) {
                let at = p.sign_of(lord);
                if is_odd_sign(at) && window.contains(&at) && !candidates.contains(&lord) {
                    candidates.push(lord);
                }
            }
        }
        candidates.sort_by_key(|&g| GRAHA_CYCLE.iter().position(|&c| c == g));
        pick_among(&candidates, p, strength)?
    };

    // 2.1.53: Saturn or a node is replaced by the sixth graha from him.
    Some(match chosen {
        Graha::Saturn | Graha::Rahu | Graha::Ketu => graha_after(chosen, 5),
        other => other,
    })
}

/// The strongest of `grahas` by pairwise comparison, or `None` if empty.
fn fold_strongest(grahas: &[Graha], strength: &dyn Strength) -> Option<Graha> {
    let (&first, rest) = grahas.split_first()?;
    Some(
        rest.iter()
            .fold(first, |acc, &g| strength.stronger_graha(acc, g)),
    )
}

/// 2.1.54, 2.1.55 and 2.1.57 over the candidates.
fn pick_among(
    candidates: &[Graha],
    placements: &NatalPlacements,
    strength: &dyn Strength,
) -> Option<Graha> {
    if candidates.len() <= 1 {
        return candidates.first().copied();
    }
    let degrees: Option<Vec<f64>> = candidates
        .iter()
        .map(|&g| placements.degrees_of(g))
        .collect();
    let finalists: Vec<Graha> = if let Some(d) = degrees {
        let reverse = candidates.contains(&Graha::Rahu);
        let key = |x: f64| if reverse { -x } else { x };
        let best = d.iter().map(|&x| key(x)).fold(f64::NEG_INFINITY, f64::max);
        candidates
            .iter()
            .zip(&d)
            .filter(|&(_, &x)| (key(x) - best).abs() < 1e-9)
            .map(|(&g, _)| g)
            .collect()
    } else {
        candidates.to_vec()
    };
    fold_strongest(&finalists, strength)
}

/// Where the sthira dasha starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SthiraAnchor {
    /// The rashi the Brahma graha occupies (2.3.4; BPHS 46.169). Grade (a).
    /// The default. The dasha is `None` for a chart with no Brahma.
    #[default]
    Brahma,
    /// The lagna rashi. A commentators' anchor; grade (c).
    Lagna,
    /// A rashi the caller names, zero-based (the karakamsha, for the
    /// commentators who anchor there, or any other reading).
    Explicit(u8),
}

/// Which way the sthira dasha walks the zodiac. See the module doc: no
/// option is stated by the text for this dasha; grade (c) throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SthiraDirection {
    /// Forward from an odd anchor, in reverse from an even one. The default.
    #[default]
    AnchorParity,
    /// Chara dasha's rule: forward if the ninth from the lagna is
    /// vishama-pada, else reverse.
    NinthFromLagna,
    /// Forward from a movable or dual anchor, in reverse from a fixed one.
    AnchorModality,
    /// Always forward.
    Zodiacal,
    /// Always in reverse.
    AntiZodiacal,
}

/// Everything [`compute_sthira`] leaves open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SthiraOptions {
    /// Where the twelve periods start.
    pub anchor: SthiraAnchor,
    /// Which way they run.
    pub direction: SthiraDirection,
    /// Start and order of each period's antardashas.
    pub antar: AntarRules,
    /// Length of each antardasha.
    pub antar_length: AntarLength,
    /// The open rules of the Brahma identification.
    pub brahma: BrahmaRules,
}

/// A computed sthira dasha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SthiraDasha {
    /// Zero-based sign the first period is of.
    pub anchor_sign: u8,
    /// The Brahma graha, when the anchor was [`SthiraAnchor::Brahma`].
    pub brahma: Option<Graha>,
    /// The direction the periods walk.
    pub direction: DashaDirection,
    /// The twelve periods, in order, with their antardashas.
    pub periods: Vec<RashiPeriod>,
}

/// Compute the sthira dasha: twelve periods of 7, 8 or 9 years from the
/// anchor rashi, each with twelve antardashas.
///
/// Returns `None` only when the anchor is [`SthiraAnchor::Brahma`] and the
/// chart has no Brahma graha under the rules in `options.brahma`.
#[must_use]
pub fn compute_sthira(
    placements: &NatalPlacements,
    atmakaraka: Graha,
    birth_jd: f64,
    options: &SthiraOptions,
    strength: &dyn Strength,
) -> Option<SthiraDasha> {
    let (anchor_sign, brahma) = match options.anchor {
        SthiraAnchor::Brahma => {
            let b = brahma_graha(placements, atmakaraka, strength, options.brahma)?;
            (placements.sign_of(b), Some(b))
        }
        SthiraAnchor::Lagna => (placements.lagna_sign(), None),
        SthiraAnchor::Explicit(sign) => (sign % 12, None),
    };
    let direction = match options.direction {
        SthiraDirection::AnchorParity => DashaDirection::by_parity(anchor_sign),
        SthiraDirection::NinthFromLagna => {
            if super::chara::sequence_runs_forward(placements.lagna_sign()) {
                DashaDirection::Zodiacal
            } else {
                DashaDirection::AntiZodiacal
            }
        }
        SthiraDirection::AnchorModality => {
            if modality(anchor_sign) == Modality::Fixed {
                DashaDirection::AntiZodiacal
            } else {
                DashaDirection::Zodiacal
            }
        }
        SthiraDirection::Zodiacal => DashaDirection::Zodiacal,
        SthiraDirection::AntiZodiacal => DashaDirection::AntiZodiacal,
    };

    let mut periods = Vec::with_capacity(12);
    let mut current_jd = birth_jd;
    for i in 0u8..12 {
        let sign = walk(anchor_sign, direction, i);
        let years = sthira_years(sign);
        let days = years * DASHA_YEAR_DAYS;
        let end_jd = current_jd + days;
        periods.push(RashiPeriod {
            sign_index: sign,
            sign_name: SIGN_NAMES[usize::from(sign)].to_string(),
            start_jd: current_jd,
            end_jd,
            duration_years: years,
            antar: build_antars(
                sign,
                current_jd,
                end_jd,
                years,
                options.antar,
                options.antar_length,
                placements,
                strength,
            ),
        });
        current_jd = end_jd;
    }
    Some(SthiraDasha {
        anchor_sign,
        brahma,
        direction,
        periods,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::chara::GrahaSigns;
    use crate::dasha::rashi_antar::AntarStart;
    use crate::dasha::rashi_strength::JaiminiStrength;

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

    fn brahma(p: &NatalPlacements, ak: Graha, rules: BrahmaRules) -> Option<Graha> {
        brahma_graha(p, ak, &JaiminiStrength::new(p), rules)
    }

    // Lagna Aries(0). The sixth, eighth and twelfth rashis are Virgo
    // (Mercury), Scorpio (Mars and Ketu) and Pisces (Jupiter). The seventh
    // is Libra. Behind Aries, excluding it: Pisces(11) Aquarius(10)
    // Capricorn(9) Sagittarius(8) Scorpio(7) Libra(6); of those the odd
    // rashis are Aquarius, Sagittarius and Libra.

    // 1. One candidate. Sun (the atmakaraka, passed as a parameter) in
    // Aries makes Aries the stronger of Aries and Libra (it holds a graha,
    // Libra none). Jupiter, lord of the twelfth, stands in Sagittarius(8):
    // odd, in the window. Mercury (Taurus), Mars (Cancer) and Ketu (Virgo,
    // opposite Rahu in Pisces) stand outside it. The lord of the eighth
    // from the atmakaraka's Aries is Scorpio's Mars/Ketu, neither in
    // Scorpio. So Brahma is Jupiter.
    #[test]
    fn brahma_is_the_odd_lord_behind_the_stronger_of_lagna_and_seventh() {
        let p = chart(0, [0, 3, 3, 1, 8, 2, 4, 11]);
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Jupiter)
        );
    }

    // 2. Mercury, lord of the sixth, stands in Aries itself, with the Sun:
    // inside the inclusive window, outside the exclusive one. Jupiter now
    // stands in Cancer, outside both. Exclusive: no candidate, no Brahma.
    // Inclusive: Mercury.
    #[test]
    fn the_two_windows_differ_on_a_graha_in_the_base_rashi() {
        let p = chart(0, [0, 3, 3, 0, 3, 2, 4, 11]);
        let exclusive = BrahmaRules {
            window: BrahmaWindow::BehindExclusive,
        };
        let inclusive = BrahmaRules {
            window: BrahmaWindow::BehindInclusive,
        };
        assert_eq!(brahma(&p, Graha::Sun, exclusive), None);
        assert_eq!(brahma(&p, Graha::Sun, inclusive), Some(Graha::Mercury));
    }

    // 3. 2.1.56 comes first and needs no odd rashi or window: the atmakaraka
    // Sun in Aries, the eighth from it Scorpio, its lord Mars standing in
    // Scorpio(7, an even rashi). Jupiter in Sagittarius would be the 2.1.52
    // candidate; Mars is chosen over him.
    #[test]
    fn the_eighth_lord_in_the_eighth_is_brahma_before_any_other() {
        let p = chart(0, [0, 3, 7, 1, 8, 2, 4, 11]);
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Mars)
        );
    }

    // 4. 2.1.53: Saturn, Rahu or Ketu chosen is replaced by the sixth graha
    // from him. Lagna Cancer(3): the sixth Sagittarius (Jupiter), eighth
    // Aquarius (Saturn and Rahu), twelfth Gemini (Mercury). Cancer holds the
    // Moon and Jupiter, Capricorn nothing, so Cancer is the base; behind it,
    // excluding it: Gemini(2) Taurus(1) Aries(0) Pisces(11) Aquarius(10)
    // Capricorn(9), odd ones Gemini, Aries, Aquarius. Saturn in Gemini is the
    // only candidate (Rahu in Pisces is in an even rashi, Mercury in Virgo
    // and Jupiter in Cancer are outside). Saturn is replaced by Mars.
    #[test]
    fn saturn_is_replaced_by_the_sixth_graha_from_him() {
        let p = chart(3, [5, 3, 6, 5, 3, 7, 2, 11]);
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Mars)
        );
    }

    // 5. 2.1.55: with Rahu among the candidates the degrees are reckoned in
    // reverse, the fewest winning. Lagna Cancer again; Jupiter (lord of the
    // sixth) in Aquarius at 10 deg and Rahu (lord of the eighth) in Aries at
    // 20 deg are the candidates (both odd, both in the window); Mercury is
    // in Taurus, Saturn in Capricorn, so neither competes. Reversed: Jupiter
    // (10 < 20). Not reversed it would be Rahu, replaced by Mercury.
    #[test]
    fn rahu_among_the_candidates_reverses_the_degree_comparison() {
        // Sun 90 (Cancer), Moon 100 (Cancer), Mars 200, Mercury 40 (Taurus
        // 10), Jupiter 310 (Aquarius 10), Venus 210, Saturn 280 (Capricorn
        // 10), Rahu 20 (Aries 20).
        let p = NatalPlacements::from_longitudes(
            100.0,
            [90.0, 100.0, 200.0, 40.0, 310.0, 210.0, 280.0, 20.0],
        );
        assert_eq!(p.lagna_sign(), 3);
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Jupiter)
        );
    }

    // 6. 2.1.54 without Rahu: the most degrees. Lagna Aries: Mercury (lord
    // of the sixth) in Aquarius at 20 deg and Jupiter (lord of the twelfth)
    // in Sagittarius at 10 deg are both odd and behind; Mercury has more.
    #[test]
    fn more_degrees_wins_among_candidates() {
        // Sun 15 (Aries), Moon 45, Mars 100 (Cancer), Mercury 320 (Aquarius
        // 20), Jupiter 250 (Sagittarius 10), Venus 70, Saturn 130, Rahu 340
        // (Pisces 10; Ketu in Virgo).
        let p = NatalPlacements::from_longitudes(
            10.0,
            [15.0, 45.0, 100.0, 320.0, 250.0, 70.0, 130.0, 340.0],
        );
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Mercury)
        );
    }

    // 6b. The base is the stronger of the lagna and its seventh, not always
    // the lagna. Lagna Aries(0); the Sun, Moon and Mars in Libra(6), so
    // Libra, holding three grahas to Aries's none, is the base. The window
    // behind Libra excluding it is Virgo(5) Leo(4) Cancer(3) Gemini(2)
    // Taurus(1) Aries(0); the odd ones are Leo, Gemini and Aries. Mercury,
    // lord of the sixth, stands in Gemini(2): the only candidate (Mars, lord
    // of the eighth, stands in Libra, outside this window though inside the
    // one behind Aries; Jupiter and Venus are in Cancer, an even rashi; Rahu
    // in Pisces puts Ketu in Virgo, also even). The lord of the eighth from
    // the Sun in Libra is Taurus's Venus, in Cancer, not at home. So Brahma
    // is Mercury; with the lagna as base it would be Mars.
    #[test]
    fn the_seventh_can_be_the_base() {
        let p = chart(0, [6, 6, 6, 2, 3, 3, 9, 11]);
        assert_eq!(
            brahma(&p, Graha::Sun, BrahmaRules::default()),
            Some(Graha::Mercury)
        );
    }

    // 7. Years by modality and the 96-year total, in every direction, from
    // every anchor: Aries 7, Taurus 8, Gemini 9 and so on.
    #[test]
    fn years_follow_the_modality_of_each_rashi() {
        let p = chart(0, [0; 8]);
        for anchor in 0..12u8 {
            for direction in [SthiraDirection::Zodiacal, SthiraDirection::AntiZodiacal] {
                let options = SthiraOptions {
                    anchor: SthiraAnchor::Explicit(anchor),
                    direction,
                    ..SthiraOptions::default()
                };
                let d =
                    compute_sthira(&p, Graha::Sun, TEST_JD, &options, &JaiminiStrength::new(&p))
                        .expect("explicit anchor always computes");
                assert_eq!(d.periods.len(), 12);
                let total: f64 = d.periods.iter().map(|x| x.duration_years).sum();
                assert_eq!(total, 96.0);
                for x in &d.periods {
                    let want = match x.sign_index % 3 {
                        0 => 7.0,
                        1 => 8.0,
                        _ => 9.0,
                    };
                    assert_eq!(x.duration_years, want, "sign {}", x.sign_index);
                }
                assert_eq!(d.periods[0].sign_index, anchor);
            }
        }
    }

    // 8. The request's F1 vector, re-derived by hand. Lagna Leo(4), anchored
    // on the lagna, direction from the anchor's modality (Leo is fixed, so
    // in reverse), proportional antars from the dasha rashi itself.
    // Leo 8, Cancer 7, Gemini 9, Taurus 8, Aries 7, Pisces 9, Aquarius 8,
    // Capricorn 7, Sagittarius 9, Scorpio 8, Libra 7, Virgo 9. The first
    // antar of Leo is its own, 8 x 8 / 96 = 2/3 year.
    #[test]
    fn request_vector_f1_leo_lagna_modality_walk_proportional_antars() {
        let p = chart(4, [1, 6, 0, 0, 1, 0, 3, 5]);
        let options = SthiraOptions {
            anchor: SthiraAnchor::Lagna,
            direction: SthiraDirection::AnchorModality,
            antar: AntarRules {
                start: AntarStart::RashiItself,
                ..AntarRules::default()
            },
            antar_length: AntarLength::ProportionalToSthiraYears,
            ..SthiraOptions::default()
        };
        let d = compute_sthira(
            &p,
            Graha::Mercury,
            TEST_JD,
            &options,
            &JaiminiStrength::new(&p),
        )
        .unwrap();
        let got: Vec<(u8, f64)> = d
            .periods
            .iter()
            .map(|x| (x.sign_index, x.duration_years))
            .collect();
        assert_eq!(
            got,
            vec![
                (4, 8.0),
                (3, 7.0),
                (2, 9.0),
                (1, 8.0),
                (0, 7.0),
                (11, 9.0),
                (10, 8.0),
                (9, 7.0),
                (8, 9.0),
                (7, 8.0),
                (6, 7.0),
                (5, 9.0),
            ]
        );
        assert_eq!(d.direction, DashaDirection::AntiZodiacal);
        assert_eq!(d.periods[0].antar[0].sign_index, 4);
        assert_eq!(d.periods[0].antar[0].duration_years, 8.0 * 8.0 / 96.0);
    }

    // 9. The request's F2 vector. Lagna Pisces(11), dual, so forward.
    // Pisces 9, Aries 7, Taurus 8, Gemini 9, Cancer 7, Leo 8, Virgo 9,
    // Libra 7, Scorpio 8, Sagittarius 9, Capricorn 7, Aquarius 8. First
    // antar of Pisces: 9 x 9 / 96 = 0.84375 year.
    #[test]
    fn request_vector_f2_pisces_lagna_modality_walk_proportional_antars() {
        let p = chart(11, [5, 5, 6, 5, 4, 4, 9, 8]);
        let options = SthiraOptions {
            anchor: SthiraAnchor::Lagna,
            direction: SthiraDirection::AnchorModality,
            antar: AntarRules {
                start: AntarStart::RashiItself,
                ..AntarRules::default()
            },
            antar_length: AntarLength::ProportionalToSthiraYears,
            ..SthiraOptions::default()
        };
        let d = compute_sthira(
            &p,
            Graha::Moon,
            TEST_JD,
            &options,
            &JaiminiStrength::new(&p),
        )
        .unwrap();
        let got: Vec<(u8, f64)> = d
            .periods
            .iter()
            .map(|x| (x.sign_index, x.duration_years))
            .collect();
        assert_eq!(
            got,
            vec![
                (11, 9.0),
                (0, 7.0),
                (1, 8.0),
                (2, 9.0),
                (3, 7.0),
                (4, 8.0),
                (5, 9.0),
                (6, 7.0),
                (7, 8.0),
                (8, 9.0),
                (9, 7.0),
                (10, 8.0),
            ]
        );
        assert_eq!(d.periods[0].antar[0].duration_years, 0.84375);
    }

    // 10. Direction options. Aries anchor (odd): parity forward, modality
    // forward (movable). Taurus anchor (even, fixed): parity reverse,
    // modality reverse. Gemini anchor (odd, dual): parity forward, modality
    // forward. Cancer anchor (even, movable): parity reverse, modality
    // forward: the one place the two differ among these. The ninth-from-
    // lagna rule ignores the anchor: lagna Aries has Sagittarius ninth
    // (vishama-pada, forward); lagna Taurus has Capricorn ninth (sama-pada,
    // reverse).
    #[test]
    fn direction_options() {
        let dir = |lagna: u8, anchor: u8, how: SthiraDirection| {
            let p = chart(lagna, [0; 8]);
            let options = SthiraOptions {
                anchor: SthiraAnchor::Explicit(anchor),
                direction: how,
                ..SthiraOptions::default()
            };
            compute_sthira(&p, Graha::Sun, TEST_JD, &options, &JaiminiStrength::new(&p))
                .unwrap()
                .direction
        };
        use DashaDirection::{AntiZodiacal as Back, Zodiacal as Fwd};
        use SthiraDirection::{AnchorModality, AnchorParity, NinthFromLagna};
        assert_eq!(dir(0, 0, AnchorParity), Fwd);
        assert_eq!(dir(0, 1, AnchorParity), Back);
        assert_eq!(dir(0, 2, AnchorParity), Fwd);
        assert_eq!(dir(0, 3, AnchorParity), Back);
        assert_eq!(dir(0, 3, AnchorModality), Fwd);
        assert_eq!(dir(0, 1, AnchorModality), Back);
        assert_eq!(dir(0, 7, AnchorModality), Back);
        assert_eq!(dir(0, 3, NinthFromLagna), Fwd);
        assert_eq!(dir(1, 3, NinthFromLagna), Back);
        assert_eq!(dir(1, 3, SthiraDirection::Zodiacal), Fwd);
        assert_eq!(dir(0, 0, SthiraDirection::AntiZodiacal), Back);
    }

    // 11. The Brahma anchor: Chart of test 2 under the inclusive window has
    // Mercury as Brahma, standing in Aries, so the dasha starts at Aries and,
    // Aries being odd, runs forward: Aries 7, Taurus 8, Gemini 9, ... Under
    // the exclusive window there is no Brahma and no dasha.
    #[test]
    fn brahma_anchor_starts_at_his_rashi_or_not_at_all() {
        let p = chart(0, [0, 3, 3, 0, 3, 2, 4, 11]);
        let s = JaiminiStrength::new(&p);
        let inclusive = SthiraOptions {
            brahma: BrahmaRules {
                window: BrahmaWindow::BehindInclusive,
            },
            ..SthiraOptions::default()
        };
        let d = compute_sthira(&p, Graha::Sun, TEST_JD, &inclusive, &s).unwrap();
        assert_eq!(d.brahma, Some(Graha::Mercury));
        assert_eq!(d.anchor_sign, 0);
        assert_eq!(d.direction, DashaDirection::Zodiacal);
        assert_eq!(d.periods[1].sign_index, 1);
        assert!(compute_sthira(&p, Graha::Sun, TEST_JD, &SthiraOptions::default(), &s).is_none());
    }

    // 11b. The antar rules reach every period. Every graha in Aries(0), Ketu
    // in Libra; anchor Taurus(1), whose lord Venus stands in Aries. With the
    // dasha rashi as the start the first antar is Taurus; with the lord's
    // rashi it is Aries.
    #[test]
    fn antar_rules_reach_each_period() {
        let p = chart(0, [0; 8]);
        let s = JaiminiStrength::new(&p);
        let first_antar = |start| {
            let options = SthiraOptions {
                anchor: SthiraAnchor::Explicit(1),
                antar: AntarRules {
                    start,
                    ..AntarRules::default()
                },
                ..SthiraOptions::default()
            };
            let d = compute_sthira(&p, Graha::Sun, TEST_JD, &options, &s).unwrap();
            d.periods[0].antar[0].sign_index
        };
        assert_eq!(first_antar(AntarStart::RashiItself), 1);
        assert_eq!(first_antar(AntarStart::LordOccupied), 0);
    }

    // 12. Periods are contiguous from birth and each one's antars tile it.
    #[test]
    fn periods_are_contiguous_and_antars_tile() {
        let p = chart(0, [0, 3, 3, 0, 3, 2, 4, 11]);
        let s = JaiminiStrength::new(&p);
        let options = SthiraOptions {
            anchor: SthiraAnchor::Lagna,
            ..SthiraOptions::default()
        };
        let d = compute_sthira(&p, Graha::Sun, TEST_JD, &options, &s).unwrap();
        assert_eq!(d.periods[0].start_jd, TEST_JD);
        for (k, x) in d.periods.iter().enumerate() {
            assert_eq!(x.end_jd - x.start_jd, x.duration_years * 365.25);
            if k > 0 {
                assert_eq!(x.start_jd, d.periods[k - 1].end_jd);
            }
            assert_eq!(x.antar.len(), 12);
            assert_eq!(x.antar[0].start_jd, x.start_jd);
            assert_eq!(x.antar[11].end_jd, x.end_jd);
        }
    }
}
