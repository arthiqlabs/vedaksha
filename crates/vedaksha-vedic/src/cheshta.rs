// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Cheshta (motion states) and graha yuddha (planetary war).
//!
//! # Sources
//!
//! BPHS, the chapter on graha strength (Spashtabala adhyaya), slokas 21 to 23,
//! read in the Sanskrit. Chapter numbers follow the recension in which this is
//! chapter 27; a longer recension that counts two extra earlier chapters
//! numbers it 29, and the sloka numbers agree. Each rule is graded the way
//! [`crate::dasha::chara`] grades its sutras: (a) stated at the verse and read
//! there, (d) this crate's convention where the verse is silent.
//!
//! - **27.21, the eight states (a).** The verse names them: `vakra`,
//!   `anuvakra`, `vikala`, `manda`, `mandatara`, `sama`, `chara`, `atichara`.
//! - **27.22 and 27.23, their strengths (a).** Sixty for `vakra`; half of it
//!   for `anuvakra` (30); a quarter for `vikala` (15); half for `madhyagati`
//!   (30), the state at the mean pace, which is `sama`; a quarter for `manda`
//!   (15) and half *of that* for `mandatara` (7.5); three quarters for `chara`
//!   (45); half for `atichara` (30). The strengths attach to the words of
//!   27.22 and 27.23, not to the order in which 27.21 lists the names: read in
//!   list order the values would give `sama`, the mean pace, the weakest of
//!   the eight, which the verse does not say. [`Cheshta::virupas`] returns
//!   them.
//! - **The speed bands (d).** The verse states no numeric speed fraction, so
//!   any speed-band implementation is a conventional operationalisation, not
//!   a verse. The bands below are this engine's stated reading, documented so
//!   a consumer can map them differently.
//! - **Anuvakra and atichara are sign-entry states (d).** The verse names
//!   them and gives their strengths but not their definitions. This module
//!   reads `anuvakra` as retrograde motion crossing into the previous sign and
//!   `atichara` as accelerated direct motion crossing into the next sign:
//!   instantaneous speed alone cannot tell them from `vakra` and `chara`.
//!   [`cheshta`] resolves clearly retrograde motion as `vakra` and the
//!   near-stationary retrograde sliver as `anuvakra`, a speed-only proxy;
//!   [`cheshta_at`] takes the sidereal longitude and does it properly, and its
//!   window for "crossing" is stated at [`SignEntryWindow`].
//!
//! The mean motion the bands are ratios of is not a constant of this module.
//! [`crate::mean_motion::mean_geocentric_motion`] returns it at a date from
//! the VSOP87 mean elements; for Mercury and Venus it is the mean Sun's pace,
//! not their own heliocentric rate.
//!
//! This is distinct from [`crate::shadbala`] `cheshta_bala`/`cheshta_rasmi`,
//! which return a continuous 0–60 strength score: a consumer needs both the
//! category (here) and the score (there).

use crate::graha::Graha;

/// The eight classical motion states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cheshta {
    /// Clearly retrograde (speed < −1'/day).
    Vakra,
    /// Marginally retrograde (−1'/day ≤ speed < 0): the station
    /// neighbourhood. Sign-entry proper needs boundary tracking (module doc).
    Anuvakra,
    /// Stationary neighbourhood: exact zero up to +1'/day direct (one kala,
    /// the classical small-motion unit, used here as the conventional
    /// tolerance). Negative speeds route to Vakra/Anuvakra first, so this
    /// band is one-sided by construction.
    Vikala,
    /// Slow direct: a third up to two thirds of the mean motion.
    Manda,
    /// Slower than Manda: below a third of the mean motion (and above the
    /// stationary neighbourhood).
    Mandatara,
    /// Near the mean pace: 2/3–4/3 of mean motion.
    Sama,
    /// Fast: 4/3–2× mean motion.
    Chara,
    /// Excessive: above 2× mean motion.
    Atichara,
}

impl Cheshta {
    /// The Cheshta Bala this state carries, in virupas (BPHS 27.22 and 27.23):
    /// `Vakra` 60, `Anuvakra` 30, `Vikala` 15, `Manda` 15, `Mandatara` 7.5,
    /// `Sama` 30, `Chara` 45, `Atichara` 30.
    #[must_use]
    pub fn virupas(self) -> f64 {
        match self {
            Self::Vakra => 60.0,
            Self::Anuvakra | Self::Sama | Self::Atichara => 30.0,
            Self::Vikala | Self::Manda => 15.0,
            Self::Mandatara => 7.5,
            Self::Chara => 45.0,
        }
    }
}

/// One arcminute in degrees — the stationary neighbourhood.
const KALA_DEG: f64 = 1.0 / 60.0;

/// Classical motion state from instantaneous vs mean daily motion.
///
/// `graha` is currently unused by the bands (they are pure speed ratios)
/// and is carried for the sign-entry refinement (Vakra/Anuvakra,
/// Chara/Atichara) that [`cheshta_at`] performs. A non-positive
/// `mean_speed_deg_per_day` is caller error; the answer stays deterministic
/// (the stationary checks fire first, anything else reads `Sama`).
///
/// This is the speed-only form: it cannot see a sign boundary, so it never
/// returns `Anuvakra` for entering the previous sign (only for the
/// near-stationary retrograde sliver) and returns `Atichara` on speed alone.
/// Use [`cheshta_at`] where the sidereal longitude is known.
#[must_use]
pub fn cheshta(graha: Graha, speed_deg_per_day: f64, mean_speed_deg_per_day: f64) -> Cheshta {
    let _ = graha;
    let s = speed_deg_per_day;
    if s < -KALA_DEG {
        return Cheshta::Vakra;
    }
    if s < 0.0 {
        return Cheshta::Anuvakra;
    }
    if s <= KALA_DEG {
        return Cheshta::Vikala;
    }
    if mean_speed_deg_per_day <= 0.0 {
        return Cheshta::Sama;
    }
    let r = s / mean_speed_deg_per_day;
    if r < 1.0 / 3.0 {
        Cheshta::Mandatara
    } else if r < 2.0 / 3.0 {
        Cheshta::Manda
    } else if r <= 4.0 / 3.0 {
        Cheshta::Sama
    } else if r <= 2.0 {
        Cheshta::Chara
    } else {
        Cheshta::Atichara
    }
}

/// How near a sign boundary a graha must be for [`cheshta_at`] to call it
/// "entering" the neighbouring sign.
///
/// The verse (BPHS 27.21 to 27.23) names anuvakra and atichara but gives no
/// distance, so this is a convention (grade (d) of the module doc), exposed so
/// a consumer can choose another. A graha is *entering* a sign when it is
/// crossing a boundary: within the window of the boundary ahead of it in its
/// direction of motion, or within the window of the boundary it has just
/// crossed. Boundaries are the multiples of 30 degrees of the sidereal
/// longitude passed in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignEntryWindow {
    /// Within this many days of the graha's present speed: the window is
    /// `|speed| * days` degrees, so a slow Saturn and a quick Mercury are each
    /// "entering" for the same stretch of time. The default is one day.
    Days(f64),
    /// Within this many degrees of the boundary, whatever the speed.
    Degrees(f64),
}

/// The default window: one day of the graha's present motion.
pub const DEFAULT_SIGN_ENTRY_WINDOW: SignEntryWindow = SignEntryWindow::Days(1.0);

impl SignEntryWindow {
    /// The window in degrees at a given speed.
    fn degrees(self, speed_deg_per_day: f64) -> f64 {
        match self {
            Self::Days(d) => speed_deg_per_day.abs() * d,
            Self::Degrees(w) => w,
        }
    }
}

/// Whether a graha at `sidereal_longitude_deg` moving at `speed_deg_per_day`
/// is crossing a sign boundary, within `window`.
///
/// "Crossing" is the boundary ahead in the direction of motion or the one just
/// behind (see [`SignEntryWindow`]). Returns the distance-to-boundary test
/// only; the direction is the sign of the speed, handled by the caller.
fn is_crossing_sign_boundary(
    sidereal_longitude_deg: f64,
    speed_deg_per_day: f64,
    window: SignEntryWindow,
) -> bool {
    let within_sign = sidereal_longitude_deg.rem_euclid(30.0);
    let (ahead, behind) = if speed_deg_per_day < 0.0 {
        // Moving toward the sign's start.
        (within_sign, 30.0 - within_sign)
    } else {
        (30.0 - within_sign, within_sign)
    };
    let w = window.degrees(speed_deg_per_day);
    ahead <= w || behind <= w
}

/// Classical motion state with the sign boundary taken into account.
///
/// As [`cheshta`], plus the two states that are about sign entry (see the
/// module doc):
///
/// - **`Anuvakra`** is retrograde motion that is crossing a sign boundary,
///   going into the previous sign. Retrograde motion elsewhere in the sign is
///   `Vakra`.
/// - **`Atichara`** is accelerated direct motion (faster than 4/3 of the mean,
///   the `Chara` band or beyond) that is crossing a sign boundary, going into
///   the next sign. Accelerated motion elsewhere in the sign is `Chara`, even
///   above twice the mean: unlike [`cheshta`], speed alone does not make
///   `Atichara` here.
/// - **`Vikala`** is the stationary neighbourhood on *either* side of zero
///   (|speed| up to one arcminute per day). [`cheshta`] files its retrograde
///   half under `Anuvakra` as a speed-only proxy; with the longitude known
///   that sliver is a station, not a sign entry.
///
/// Everything else is [`cheshta`]'s answer unchanged. The boundary window is
/// [`DEFAULT_SIGN_ENTRY_WINDOW`]; see [`cheshta_at_with_window`] to choose.
///
/// `sidereal_longitude_deg` is the graha's longitude in the sidereal zodiac the
/// signs are counted in (boundaries at multiples of 30 degrees). Hand it the
/// speed in the same frame: a sidereal longitude with a tropical speed differs
/// by the precession rate, 0.000038 degrees per day, which only matters to a
/// graha crawling through a station.
#[must_use]
pub fn cheshta_at(
    graha: Graha,
    sidereal_longitude_deg: f64,
    speed_deg_per_day: f64,
    mean_speed_deg_per_day: f64,
) -> Cheshta {
    cheshta_at_with_window(
        graha,
        sidereal_longitude_deg,
        speed_deg_per_day,
        mean_speed_deg_per_day,
        DEFAULT_SIGN_ENTRY_WINDOW,
    )
}

/// [`cheshta_at`] with an explicit [`SignEntryWindow`].
#[must_use]
pub fn cheshta_at_with_window(
    graha: Graha,
    sidereal_longitude_deg: f64,
    speed_deg_per_day: f64,
    mean_speed_deg_per_day: f64,
    window: SignEntryWindow,
) -> Cheshta {
    let s = speed_deg_per_day;
    if s.abs() <= KALA_DEG {
        return Cheshta::Vikala;
    }
    let crossing = is_crossing_sign_boundary(sidereal_longitude_deg, s, window);
    if s < 0.0 {
        return if crossing {
            Cheshta::Anuvakra
        } else {
            Cheshta::Vakra
        };
    }
    match cheshta(graha, s, mean_speed_deg_per_day) {
        Cheshta::Chara | Cheshta::Atichara => {
            if crossing {
                Cheshta::Atichara
            } else {
                Cheshta::Chara
            }
        }
        other => other,
    }
}

/// How a yuddha winner is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YuddhaRule {
    /// The northerly graha (greater latitude) wins.
    NorthernLatitude,
    /// The brighter graha wins, in the conventional order Venus, Jupiter,
    /// Mars, Mercury, Saturn. (Apparent magnitudes vary with phase and
    /// distance; this fixed order is the traditional reading.)
    Brighter,
}

/// Brightness rank (lower = brighter) for [`YuddhaRule::Brighter`].
fn brightness_rank(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Venus => Some(0),
        Graha::Jupiter => Some(1),
        Graha::Mars => Some(2),
        Graha::Mercury => Some(3),
        Graha::Saturn => Some(4),
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => None,
    }
}

/// Graha yuddha — planetary war — winner, or `None` where there is no war.
///
/// A war needs the two grahas within 1° of longitude (shortest arc). The
/// classical war is among the five star-planets; involving the Sun, Moon or
/// nodes is not a yuddha and yields `None` under EITHER rule, as does an
/// exact tie under the selected rule. Latitudes are ecliptic degrees, north
/// positive.
///
/// Source: BPHS, the chapter on graha bala.
#[must_use]
pub fn graha_yuddha(
    a: Graha,
    a_longitude_deg: f64,
    a_latitude_deg: f64,
    b: Graha,
    b_longitude_deg: f64,
    b_latitude_deg: f64,
    rule: YuddhaRule,
) -> Option<Graha> {
    if brightness_rank(a).is_none() || brightness_rank(b).is_none() {
        return None;
    }
    let mut sep = (a_longitude_deg - b_longitude_deg).rem_euclid(360.0);
    if sep > 180.0 {
        sep = 360.0 - sep;
    }
    if sep > 1.0 {
        return None;
    }
    match rule {
        YuddhaRule::NorthernLatitude => {
            if (a_latitude_deg - b_latitude_deg).abs() < 1e-12 {
                None
            } else if a_latitude_deg > b_latitude_deg {
                Some(a)
            } else {
                Some(b)
            }
        }
        YuddhaRule::Brighter => match (brightness_rank(a), brightness_rank(b)) {
            (Some(ra), Some(rb)) => match ra.cmp(&rb) {
                core::cmp::Ordering::Equal => None,
                core::cmp::Ordering::Less => Some(a),
                core::cmp::Ordering::Greater => Some(b),
            },
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrograde_and_station() {
        // Mean motion only shapes the DIRECT bands; these fire first.
        assert_eq!(cheshta(Graha::Mars, -0.5, 0.5), Cheshta::Vakra);
        assert_eq!(cheshta(Graha::Mars, -0.01, 0.5), Cheshta::Anuvakra);
        assert_eq!(cheshta(Graha::Mars, 0.0, 0.5), Cheshta::Vikala);
        assert_eq!(cheshta(Graha::Mars, 0.01, 0.5), Cheshta::Vikala);
    }

    #[test]
    fn direct_bands_scale_with_mean() {
        let m = 1.0;
        let g = Graha::Mars;
        assert_eq!(cheshta(g, 0.1, m), Cheshta::Mandatara);
        assert_eq!(cheshta(g, 0.5, m), Cheshta::Manda);
        assert_eq!(cheshta(g, 1.0, m), Cheshta::Sama);
        assert_eq!(cheshta(g, 1.5, m), Cheshta::Chara);
        assert_eq!(cheshta(g, 2.5, m), Cheshta::Atichara);
        // Bands are relative: doubling both speeds keeps the state.
        assert_eq!(cheshta(g, 3.0, 2.0), Cheshta::Chara);
        // The graha parameter is inert by design (see fn doc): the Sun at
        // its own mean pace reads Sama like anyone else.
        assert_eq!(cheshta(Graha::Sun, 1.0, 1.0), Cheshta::Sama);
    }

    // The strengths of BPHS 27.22 and 27.23, from the words of the verse:
    // sixty; half; a quarter; half (madhyagati = sama); a quarter, and half of
    // that; three quarters; half.
    #[test]
    fn strengths_follow_the_words_of_the_verse() {
        let sixty = 60.0;
        assert!((Cheshta::Vakra.virupas() - sixty).abs() < f64::EPSILON);
        assert!((Cheshta::Anuvakra.virupas() - sixty / 2.0).abs() < f64::EPSILON);
        assert!((Cheshta::Vikala.virupas() - sixty / 4.0).abs() < f64::EPSILON);
        assert!((Cheshta::Sama.virupas() - sixty / 2.0).abs() < f64::EPSILON);
        assert!((Cheshta::Manda.virupas() - sixty / 4.0).abs() < f64::EPSILON);
        // "tasya dalam mandatarasya": half of manda's quarter.
        assert!(
            (Cheshta::Mandatara.virupas() - Cheshta::Manda.virupas() / 2.0).abs() < f64::EPSILON
        );
        assert!((Cheshta::Chara.virupas() - sixty * 3.0 / 4.0).abs() < f64::EPSILON);
        assert!((Cheshta::Atichara.virupas() - sixty / 2.0).abs() < f64::EPSILON);
        // The mean pace is not the weakest state.
        assert!(Cheshta::Sama.virupas() > Cheshta::Manda.virupas());
    }

    // `Manda` is 1/3 to 2/3 of the mean motion, `Mandatara` below a third:
    // the doc says what the code does.
    #[test]
    fn manda_is_a_third_to_two_thirds_of_the_mean() {
        let m = 3.0;
        let g = Graha::Saturn;
        assert_eq!(cheshta(g, 0.99, m), Cheshta::Mandatara); // 0.33 of mean
        assert_eq!(cheshta(g, 1.01, m), Cheshta::Manda); // 0.34
        assert_eq!(cheshta(g, 1.99, m), Cheshta::Manda); // 0.66
        assert_eq!(cheshta(g, 2.01, m), Cheshta::Sama); // 0.67
    }

    // ── cheshta_at: the sign-entry states ───────────────────────────

    // Mars: mean 0.5240 deg/day. A day's travel at -0.4 is 0.4 degrees.
    #[test]
    fn retrograde_is_anuvakra_only_where_it_crosses_a_sign_boundary() {
        let (g, m) = (Graha::Mars, 0.524);
        // Mid-sign (15 deg into the sign), retrograde: Vakra.
        assert_eq!(cheshta_at(g, 105.0, -0.4, m), Cheshta::Vakra);
        // 0.3 deg above the sign start, going down at 0.4/day: crosses within
        // the day, into the previous sign.
        assert_eq!(cheshta_at(g, 90.3, -0.4, m), Cheshta::Anuvakra);
        // Just crossed (0.3 deg below the boundary it passed -- that is, 29.7
        // into the previous sign): still crossing.
        assert_eq!(cheshta_at(g, 119.7, -0.4, m), Cheshta::Anuvakra);
        // 0.5 deg above the start at 0.4/day: more than a day away.
        assert_eq!(cheshta_at(g, 90.5, -0.4, m), Cheshta::Vakra);
        // The speed-only form calls the same clear retrograde Vakra at every
        // longitude, which is exactly what cheshta_at corrects.
        assert_eq!(cheshta(g, -0.4, m), Cheshta::Vakra);
    }

    #[test]
    fn accelerated_direct_motion_is_atichara_only_where_it_crosses_a_boundary() {
        let (g, m) = (Graha::Mars, 0.524);
        // 1.5x the mean, mid-sign: Chara.
        let fast = 1.5 * m;
        assert_eq!(cheshta_at(g, 105.0, fast, m), Cheshta::Chara);
        // 0.5 deg short of the next sign at 0.786/day: crosses within a day.
        assert_eq!(cheshta_at(g, 119.5, fast, m), Cheshta::Atichara);
        // Above twice the mean but mid-sign: Chara, not Atichara -- speed
        // alone does not make atichara here (cheshta() would say Atichara).
        assert_eq!(cheshta(g, 3.0 * m, m), Cheshta::Atichara);
        assert_eq!(cheshta_at(g, 105.0, 3.0 * m, m), Cheshta::Chara);
        // At the mean pace, crossing a boundary changes nothing: Sama.
        assert_eq!(cheshta_at(g, 119.9, m, m), Cheshta::Sama);
        // Slow direct motion near an edge is not accelerated: Manda stays.
        assert_eq!(cheshta_at(g, 119.99, 0.5 * m, m), Cheshta::Manda);
    }

    #[test]
    fn the_stationary_neighbourhood_is_vikala_on_both_sides_of_zero() {
        let (g, m) = (Graha::Saturn, 0.0335);
        for s in [-0.016, -0.001, 0.0, 0.001, 0.016] {
            assert_eq!(cheshta_at(g, 100.0, s, m), Cheshta::Vikala, "speed {s}");
        }
        // cheshta() files the retrograde half under Anuvakra by proxy.
        assert_eq!(cheshta(g, -0.001, m), Cheshta::Anuvakra);
        // Even at a sign boundary a station is not a sign entry.
        assert_eq!(cheshta_at(g, 90.0, -0.001, m), Cheshta::Vikala);
    }

    #[test]
    fn the_window_is_a_parameter() {
        let (g, m) = (Graha::Jupiter, 0.0831);
        // 2 degrees above a sign start, retrograde at 0.1 deg/day: a day is
        // 0.1 deg, nowhere near; a 3-degree window is.
        assert_eq!(cheshta_at(g, 92.0, -0.1, m), Cheshta::Vakra);
        assert_eq!(
            cheshta_at_with_window(g, 92.0, -0.1, m, SignEntryWindow::Degrees(3.0)),
            Cheshta::Anuvakra
        );
        // Thirty days of travel at 0.1 deg/day is 3 degrees as well.
        assert_eq!(
            cheshta_at_with_window(g, 92.0, -0.1, m, SignEntryWindow::Days(30.0)),
            Cheshta::Anuvakra
        );
        // A window of 1.9 degrees does not reach 2.
        assert_eq!(
            cheshta_at_with_window(g, 92.0, -0.1, m, SignEntryWindow::Degrees(1.9)),
            Cheshta::Vakra
        );
    }

    // cheshta_at differs from cheshta in four ways only, and only by the
    // documented rule. Everywhere else it is the same answer.
    #[test]
    fn cheshta_at_changes_cheshta_only_in_the_documented_ways() {
        let g = Graha::Mars;
        let m = 0.524;
        for lon_step in 0..720 {
            let lon = f64::from(lon_step) * 0.5;
            for speed in [
                -1.0, -0.4, -0.016, -0.001, 0.0, 0.01, 0.1, 0.3, 0.52, 0.7, 0.9, 1.2, 2.0,
            ] {
                let plain = cheshta(g, speed, m);
                let at = cheshta_at(g, lon, speed, m);
                if at == plain {
                    continue;
                }
                let crossing = is_crossing_sign_boundary(lon, speed, DEFAULT_SIGN_ENTRY_WINDOW);
                let allowed = match (plain, at) {
                    // Retrograde sliver: proxy Anuvakra -> station.
                    (Cheshta::Anuvakra, Cheshta::Vikala) => speed.abs() <= KALA_DEG,
                    // Clear retrograde, or accelerated direct motion, crossing a
                    // boundary.
                    (Cheshta::Vakra, Cheshta::Anuvakra)
                    | (Cheshta::Chara | Cheshta::Atichara, Cheshta::Atichara) => crossing,
                    (Cheshta::Atichara, Cheshta::Chara) => !crossing,
                    _ => false,
                };
                assert!(
                    allowed,
                    "lon {lon}, speed {speed}: cheshta {plain:?} vs cheshta_at {at:?}"
                );
            }
        }
    }

    #[test]
    fn no_war_beyond_one_degree() {
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                101.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }

    #[test]
    fn northerner_wins_latitude_war() {
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            Some(Graha::Saturn)
        );
        // Exact tie is no decision.
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                1.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }

    #[test]
    fn brighter_wins_brightness_war() {
        // Venus outranks Jupiter; luminaries/nodes are not combatants.
        assert_eq!(
            graha_yuddha(
                Graha::Jupiter,
                100.0,
                5.0,
                Graha::Venus,
                100.5,
                -1.0,
                YuddhaRule::Brighter
            ),
            Some(Graha::Venus)
        );
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Sun,
                100.5,
                0.0,
                YuddhaRule::Brighter
            ),
            None
        );
    }

    #[test]
    fn non_planets_are_no_war_under_either_rule() {
        // The combatant gate sits before the rule match: latitude rule
        // included, not just brightness.
        assert_eq!(
            graha_yuddha(
                Graha::Mars,
                100.0,
                1.0,
                Graha::Sun,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
        assert_eq!(
            graha_yuddha(
                Graha::Rahu,
                100.0,
                1.0,
                Graha::Saturn,
                100.5,
                2.0,
                YuddhaRule::NorthernLatitude
            ),
            None
        );
    }
}
