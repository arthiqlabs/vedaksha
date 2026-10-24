// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `khagola::next_udaya`, `next_asta` and `kalamsha_deg`.
//!
//! # What these are checked against
//!
//! There is no independent published dataset of heliacal risings and settings
//! of the grahas for these to be compared with: the almanacs publish
//! conjunctions, elongations and stations, not the days a planet is first or
//! last seen. So the checks are of three other kinds. Hand-worked vectors from
//! the Surya Siddhanta's own rule (IX.5: the kalamsha is the difference of the
//! risings, in degrees of the equator). A model sky whose answer follows from
//! the rule by arithmetic. And, on the engine's real positions, properties the
//! rule implies and an independent computation of the kalamsha that finds the
//! rising times by solving for the altitude, sharing no formula with the
//! module: the day of an udaya has the kalamsha at or over the graha's limit
//! and the day before is under it; the Moon is first seen in the evening a day
//! or so after the new moon; a superior planet's udaya follows its conjunction
//! with the Sun by the limit divided by their relative motion.

mod common;

use common::{equatorial, jd, longitude_and_rate};
use vedaksha_astro::riseset::rise_set;
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::sidereal_time::gmst;
use vedaksha_vedic::combustion::combustion_orb_deg;
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::khagola::{
    HeliacalCriterion, HeliacalSide, Place, heliacal_rise, heliacal_set, kalamsha_deg, next_asta,
    next_udaya,
};

const SIDEREAL_DEG_PER_DAY: f64 = 360.985_647_366_29;
const PUNE: Place = Place {
    latitude_deg: 18.5204,
    longitude_deg_east: 73.8567,
};

fn altitude(t: f64, lat: f64, lon: f64, ra: f64, dec: f64) -> f64 {
    let h = (gmst(t).to_degrees() + lon - ra).to_radians();
    let (phi, d) = (lat.to_radians(), dec.to_radians());
    (phi.sin() * d.sin() + phi.cos() * d.cos() * h.cos())
        .asin()
        .to_degrees()
}

// ── kalamsha: hand-worked vectors ───────────────────────────────

/// Surya Siddhanta IX.5: the kalamsha is the difference of the oblique
/// ascensions, divided by sixty, of the risings (east) or of the settings with
/// six signs added (west). The rising of a body at `(alpha, delta)` is at the
/// sidereal time `alpha - H0`, setting `alpha + H0`, `cos H0 = -tan phi tan delta`.
#[test]
fn kalamsha_hand_vectors() {
    let near = |a: Option<f64>, b: f64| {
        let a = a.unwrap();
        assert!((a - b).abs() < 1.0e-9, "{a} vs {b}");
    };
    // At the equator H0 = 90 degrees for every declination, so the kalamsha is
    // the difference of right ascensions: Sun 10, planet 359 (declination 20):
    // east 10 - 359 = -349, which is 11; west 359 - 10 = 349, which is -11.
    near(
        kalamsha_deg(HeliacalSide::East, 0.0, (10.0, 0.0), (359.0, 20.0)),
        11.0,
    );
    near(
        kalamsha_deg(HeliacalSide::West, 0.0, (10.0, 0.0), (359.0, 20.0)),
        -11.0,
    );
    // Latitude 30, Sun (0, 0), planet (345, +20).
    // Sun: H0 = 90. Planet: cos H0 = -tan 30 tan 20 = -0.5773503 * 0.3639702
    // = -0.2101383, H0 = 102.130458.
    // East: (0 - 90) - (345 - 102.130458) = -90 - 242.869542 = -332.869542,
    // which is 27.130458. West: (345 + 102.130458) - (0 + 90) = 357.130458,
    // which is -2.869542.
    near(
        kalamsha_deg(HeliacalSide::East, 30.0, (0.0, 0.0), (345.0, 20.0)),
        27.130_457_844_576_6,
    );
    near(
        kalamsha_deg(HeliacalSide::West, 30.0, (0.0, 0.0), (345.0, 20.0)),
        -2.869_542_155_423_46,
    );
    // Latitude -35, Sun (100, -20), planet (80, -5).
    // H0 (Sun) = acos(-tan(-35) tan(-20)) = acos(-0.2548) = 104.764976;
    // H0 (planet) = acos(-tan(-35) tan(-5)) = acos(-0.0613) = 93.512151.
    // East: (100 - 104.764976) - (80 - 93.512151) = -4.764976 + 13.512151
    // = 8.747175. West: (80 + 93.512151) - (100 + 104.764976) = 173.512151 -
    // 204.764976 = -31.252825.
    near(
        kalamsha_deg(HeliacalSide::East, -35.0, (100.0, -20.0), (80.0, -5.0)),
        8.747_175_519_745_07,
    );
    near(
        kalamsha_deg(HeliacalSide::West, -35.0, (100.0, -20.0), (80.0, -5.0)),
        -31.252_824_480_254_2,
    );
    // Where a body never rises that day (|tan phi tan delta| >= 1) there is no
    // kalamsha: latitude 80, declination 20 gives 5.67 * 0.364 = 2.06.
    assert_eq!(
        kalamsha_deg(HeliacalSide::East, 80.0, (0.0, 0.0), (10.0, 20.0)),
        None
    );
    assert_eq!(
        kalamsha_deg(HeliacalSide::West, 80.0, (0.0, 20.0), (10.0, 0.0)),
        None
    );
}

/// The kalamsha found by solving for the instants at which the Sun and the
/// graha, held at the coordinates they have at the sunrise or sunset `t`, stand
/// on the horizon: the difference of those instants, in days, times the
/// sidereal rotation per day. No formula in common with the module's.
fn kalamsha_by_roots(
    side: HeliacalSide,
    lat: f64,
    lon: f64,
    t: f64,
    sun: (f64, f64),
    graha: (f64, f64),
) -> f64 {
    let up = side == HeliacalSide::East;
    // The instant the held body next crosses the horizon in the wanted
    // direction within half a sidereal day of `t`.
    let crossing = |eq: (f64, f64)| {
        let f = |tau: f64| altitude(tau, lat, lon, eq.0, eq.1);
        let half = 0.5 * 360.0 / SIDEREAL_DEG_PER_DAY;
        let n = 720;
        let mut prev_t = t - half;
        let mut prev = f(prev_t);
        for i in 1..=n {
            let cur_t = t - half + 2.0 * half * f64::from(i) / f64::from(n);
            let cur = f(cur_t);
            if (up && prev < 0.0 && cur >= 0.0) || (!up && prev >= 0.0 && cur < 0.0) {
                let (mut lo, mut hi) = (prev_t, cur_t);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    if (f(mid) >= 0.0) == up {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                return 0.5 * (lo + hi);
            }
            prev_t = cur_t;
            prev = cur;
        }
        panic!("no crossing");
    };
    let (ts, tp) = (crossing(sun), crossing(graha));
    // East: the graha rises ahead of the Sun when its time is earlier.
    let days = if up { ts - tp } else { tp - ts };
    // Into (-180, 180] degrees: the two crossings can be a rotation apart.
    (days * SIDEREAL_DEG_PER_DAY + 180.0).rem_euclid(360.0) - 180.0
}

#[test]
fn kalamsha_matches_the_difference_of_rising_times() {
    let t = jd(2025, 3, 1, 1.0);
    for (lat, lon, sun, graha) in [
        (0.0, 10.0, (10.0, 0.0), (359.0, 20.0)),
        (30.0, -90.0, (0.0, 0.0), (345.0, 20.0)),
        (-35.0, 150.0, (100.0, -20.0), (80.0, -5.0)),
        (18.5, 73.9, (340.0, -7.0), (320.0, -12.0)),
        (50.0, 8.0, (200.0, 10.0), (190.0, 22.0)),
        (-60.0, 0.0, (250.0, 3.0), (240.0, -2.0)),
    ] {
        for side in [HeliacalSide::East, HeliacalSide::West] {
            let k = kalamsha_deg(side, lat, sun, graha).unwrap();
            let r = kalamsha_by_roots(side, lat, lon, t, sun, graha);
            assert!((k - r).abs() < 1.0e-6, "{side:?} at {lat}: {k} vs {r}");
        }
    }
}

// ── a model sky ─────────────────────────────────────────────────

const T0: f64 = 2_460_000.0;

/// The Sun on the equator moving 0.9856 degrees a day; Jupiter on the equator
/// `s(t)` degrees west of it, `s` a sine of amplitude 20 and period 400 days. At
/// the equator the kalamsha at sunrise is exactly `s`, and Jupiter's altitude at
/// sunrise is `s` too (the hour angle is `s - 90`), so the 11 degree limit and
/// a 10 degree arc of vision are both read off `s`.
fn s_of(t: f64) -> f64 {
    20.0 * (std::f64::consts::TAU * (t - T0) / 400.0).sin()
}

fn model_sun(t: f64) -> Option<(f64, f64)> {
    Some((100.0 + 0.9856 * (t - T0), 0.0))
}

fn model_jupiter(t: f64) -> Option<(f64, f64)> {
    Some((model_sun(t)?.0 - s_of(t), 0.0))
}

const EQUATOR: Place = Place {
    latitude_deg: 0.0,
    longitude_deg_east: 0.0,
};

/// Sunrises at the equator are exactly a day apart: the hour angle advances
/// 360.9856 degrees a day and the Sun's right ascension 0.9856 of them.
const SUNRISE_GAP: f64 = 1.0;

/// How the model reads against the finders for a limit `limit` degrees: `s`
/// reaches it at the phase `asin(limit / 20)` of each 400 day cycle and leaves
/// it at `pi` minus that.
fn check_model(criterion: HeliacalCriterion, limit: f64) {
    let rise = 400.0 * (limit / 20.0).asin() / std::f64::consts::TAU;
    let fall = 200.0 - rise;
    let star = |t: f64| model_jupiter(t);
    let udaya = |start: f64| {
        next_udaya(
            start,
            Graha::Jupiter,
            HeliacalSide::East,
            EQUATOR,
            &model_sun,
            &star,
            criterion,
        )
    };
    let asta = |start: f64| {
        next_asta(
            start,
            Graha::Jupiter,
            HeliacalSide::East,
            EQUATOR,
            &model_sun,
            &star,
            criterion,
        )
    };

    // Udaya is the first sunrise at or after the instant s reaches the limit,
    // the same from anywhere in the invisibility before it (the hours between
    // the last invisible sunrise and the first visible one included).
    let first = udaya(T0 - 150.0).unwrap();
    assert!(
        first >= T0 + rise - 1.0e-6 && first < T0 + rise + SUNRISE_GAP,
        "udaya {} days after T0, s reaches the limit at {rise}",
        first - T0
    );
    // The morning it is seen s is over the limit; the morning before, under.
    assert!(s_of(first) >= limit && s_of(first - SUNRISE_GAP) < limit);
    for start in [
        T0 - 100.0,
        T0 - 1.0,
        T0 + rise - 3.0,
        T0 + rise - 1.0,
        first - 0.5,
    ] {
        let u = udaya(start).unwrap();
        assert!((u - first).abs() < 1.0e-6, "from {start}: {u} vs {first}");
    }
    // From inside the apparition: the next apparition's udaya, a cycle on.
    for start in [first + 0.5, T0 + 100.0, T0 + fall - 2.0] {
        let u = udaya(start).unwrap();
        assert!(
            (u - (first + 400.0)).abs() < 1.5,
            "from {start}: udaya {} days after T0",
            u - T0
        );
    }

    // Asta is the last sunrise with s over the limit, and from inside the
    // apparition never a later apparition's.
    let end = asta(T0 + 20.0).unwrap();
    assert!(
        end <= T0 + fall + 1.0e-6 && end > T0 + fall - SUNRISE_GAP,
        "asta {} days after T0, s falls under the limit at {fall}",
        end - T0
    );
    assert!(s_of(end) >= limit && s_of(end + SUNRISE_GAP) < limit);
    for start in [first, T0 + 100.0, end - 0.5] {
        let a = asta(start).unwrap();
        assert!((a - end).abs() < 1.0e-6, "asta from {start}: {a} vs {end}");
    }
    // From the invisibility after it: the next apparition's end.
    let after = asta(end + 0.5).unwrap();
    assert!(
        (after - (end + 400.0)).abs() < 1.5,
        "asta after: {} days on",
        after - end
    );
}

/// The kalamsha criterion with Jupiter's limit of 11 (Surya Siddhanta IX.6).
#[test]
fn model_sky_kalamsha_udaya_and_asta() {
    assert_eq!(combustion_orb_deg(Graha::Jupiter, false), Some(11.0));
    check_model(HeliacalCriterion::Kalamsha, 11.0);
}

/// The altitude criterion at 10 degrees, which the model reads off `s` as well.
#[test]
fn model_sky_altitude_udaya_and_asta() {
    check_model(
        HeliacalCriterion::Altitude {
            critical_altitude_deg: 10.0,
        },
        10.0,
    );
}

/// The Sun and the nodes have no heliacal events; nor does a missing sky.
#[test]
fn no_events_for_the_sun_the_nodes_or_a_missing_sky() {
    let star = |t: f64| model_jupiter(t);
    let none = |_: f64| None;
    for g in [Graha::Sun, Graha::Rahu, Graha::Ketu] {
        for side in [HeliacalSide::East, HeliacalSide::West] {
            assert_eq!(
                next_udaya(
                    T0,
                    g,
                    side,
                    EQUATOR,
                    &model_sun,
                    &star,
                    HeliacalCriterion::Kalamsha
                ),
                None
            );
            assert_eq!(
                next_asta(
                    T0,
                    g,
                    side,
                    EQUATOR,
                    &model_sun,
                    &star,
                    HeliacalCriterion::Kalamsha
                ),
                None
            );
        }
    }
    assert_eq!(
        next_udaya(
            T0,
            Graha::Jupiter,
            HeliacalSide::East,
            EQUATOR,
            &none,
            &star,
            HeliacalCriterion::Kalamsha
        ),
        None
    );
    assert_eq!(
        next_udaya(
            T0,
            Graha::Jupiter,
            HeliacalSide::East,
            EQUATOR,
            &model_sun,
            &none,
            HeliacalCriterion::Kalamsha
        ),
        None
    );
    // A graha that never leaves the Sun is never seen: no udaya in 1000 days.
    let glued = |t: f64| model_sun(t);
    assert_eq!(
        next_udaya(
            T0,
            Graha::Jupiter,
            HeliacalSide::East,
            EQUATOR,
            &model_sun,
            &glued,
            HeliacalCriterion::Kalamsha
        ),
        None
    );
}

// ── the engine's sky ────────────────────────────────────────────

struct Sky {
    p: AnalyticalProvider,
}

impl Sky {
    fn eq(&self, body: Body, t: f64) -> Option<(f64, f64)> {
        equatorial(&self.p, body, t)
    }
}

/// The geometric sunrise (`East`) or sunset (`West`, Sun's centre at the
/// horizon, as the module's) within half a day either side of `t`, by the
/// independent rise/set code of `vedaksha_astro`.
fn sun_event_near(sky: &Sky, t: f64, side: HeliacalSide) -> f64 {
    let sun = |tt: f64| sky.eq(Body::Sun, tt);
    let day = rise_set(
        t - 0.45,
        PUNE.latitude_deg,
        PUNE.longitude_deg_east,
        0.0,
        &sun,
    );
    match side {
        HeliacalSide::East => day.rise.unwrap(),
        HeliacalSide::West => day.set.unwrap(),
    }
}

/// The kalamsha at a sunrise or sunset, found without the module's formula.
fn independent_kalamsha(sky: &Sky, body: Body, t: f64, side: HeliacalSide) -> f64 {
    kalamsha_by_roots(
        side,
        PUNE.latitude_deg,
        PUNE.longitude_deg_east,
        t,
        sky.eq(Body::Sun, t).unwrap(),
        sky.eq(body, t).unwrap(),
    )
}

/// The udaya of a superior planet, found on the engine's sky: on the day it is
/// first seen the independent kalamsha is at or over the limit, the sunrise
/// before it is under, and it comes after the conjunction with the Sun by about
/// the limit over their relative motion.
#[test]
fn superior_planets_are_first_seen_when_the_kalamsha_reaches_the_limit() {
    let sky = Sky {
        p: AnalyticalProvider::new(),
    };
    let sun = |t: f64| sky.eq(Body::Sun, t);
    for (graha, body, start) in [
        (Graha::Jupiter, Body::Jupiter, jd(2025, 6, 1, 0.0)),
        (Graha::Saturn, Body::Saturn, jd(2025, 2, 1, 0.0)),
        (Graha::Mars, Body::Mars, jd(2026, 1, 1, 0.0)),
    ] {
        let star = |t: f64| sky.eq(body, t);
        let limit = combustion_orb_deg(graha, false).unwrap();
        let u = next_udaya(
            start,
            graha,
            HeliacalSide::East,
            PUNE,
            &sun,
            &star,
            HeliacalCriterion::Kalamsha,
        )
        .unwrap_or_else(|| panic!("{graha:?}: no udaya"));
        let before = sun_event_near(&sky, u - 1.0, HeliacalSide::East);
        let k = independent_kalamsha(&sky, body, u, HeliacalSide::East);
        let k_before = independent_kalamsha(&sky, body, before, HeliacalSide::East);
        println!(
            "{graha:?}: udaya {u:.4}, kalamsha {k:.3}, the sunrise before {k_before:.3} (limit {limit})"
        );
        assert!(
            k >= limit - 1.0e-6,
            "{graha:?}: kalamsha {k} on the day it is first seen"
        );
        assert!(
            k_before < limit,
            "{graha:?}: kalamsha {k_before} the day before"
        );
        // The sunrise it returns is the one the independent code finds.
        let rise = sun_event_near(&sky, u, HeliacalSide::East);
        assert!(
            (rise - u).abs() * 86_400.0 < 5.0,
            "{graha:?}: {} s from sunrise",
            (rise - u) * 86_400.0
        );

        // The conjunction: the longitude of the Sun passing the planet's.
        let lon = |b: Body, t: f64| longitude_and_rate(&sky.p, b, t).unwrap();
        let diff =
            |t: f64| ((lon(Body::Sun, t).0 - lon(body, t).0 + 180.0).rem_euclid(360.0)) - 180.0;
        let (mut lo, mut hi) = (u - 150.0, u);
        assert!(
            diff(lo) < 0.0 && diff(hi) > 0.0,
            "{graha:?}: conjunction not inside the 150 days"
        );
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if diff(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let conj = 0.5 * (lo + hi);
        let relative = lon(Body::Sun, conj).1 - lon(body, conj).1;
        let expected = limit / relative;
        let after = u - conj;
        println!(
            "{graha:?}: {after:.1} days after the conjunction, {expected:.1} from limit / relative motion"
        );
        // Degrees of the equator against degrees of the ecliptic differ at these
        // latitudes by up to a third, the relative motion changes over the
        // weeks, and the planet's latitude adds a little: the ratio of the
        // two is 0.9 to 1.5 here, and the check is for gross errors only.
        assert!(
            (0.6..1.6).contains(&(after / expected)),
            "{graha:?}: {after:.1} days after conjunction against {expected:.1}"
        );
    }
}

/// The Moon is first seen in the evening a day or so after the new moon
/// (Surya Siddhanta X.1: visible in the west at 12 degrees of time from the
/// Sun) and last seen in the morning a day or so before the next.
#[test]
fn the_moon_is_first_seen_in_the_evening_after_the_new_moon() {
    let sky = Sky {
        p: AnalyticalProvider::new(),
    };
    let sun = |t: f64| sky.eq(Body::Sun, t);
    let moon = |t: f64| sky.eq(Body::Moon, t);
    assert_eq!(combustion_orb_deg(Graha::Moon, false), Some(12.0));
    // The new moon after 2025-02-01: the elongation (Moon minus Sun) wrapping
    // from 360 to 0.
    let elong = |t: f64| {
        let m = longitude_and_rate(&sky.p, Body::Moon, t).unwrap().0;
        let s = longitude_and_rate(&sky.p, Body::Sun, t).unwrap().0;
        (m - s).rem_euclid(360.0)
    };
    let mut t = jd(2025, 2, 1, 0.0);
    while !(elong(t) > 300.0 && elong(t + 0.5) < 60.0) {
        t += 0.5;
    }
    let (mut lo, mut hi) = (t, t + 0.5);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if elong(mid) > 180.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let new_moon = 0.5 * (lo + hi);
    let seen = next_udaya(
        new_moon - 1.5,
        Graha::Moon,
        HeliacalSide::West,
        PUNE,
        &sun,
        &moon,
        HeliacalCriterion::Kalamsha,
    )
    .unwrap();
    println!(
        "new moon {new_moon:.3}, first seen {seen:.3} ({:.2} d after)",
        seen - new_moon
    );
    assert!(
        (0.3..2.5).contains(&(seen - new_moon)),
        "first seen {:.2} days after the new moon",
        seen - new_moon
    );
    // It is the sunset of that evening, with the independent code.
    let set = sun_event_near(&sky, seen, HeliacalSide::West);
    assert!((set - seen).abs() * 86_400.0 < 5.0);
    let k = independent_kalamsha(&sky, Body::Moon, seen, HeliacalSide::West);
    let prev = sun_event_near(&sky, seen - 1.0, HeliacalSide::West);
    let k_prev = independent_kalamsha(&sky, Body::Moon, prev, HeliacalSide::West);
    assert!(
        k >= 12.0 - 1.0e-6 && k_prev < 12.0,
        "kalamsha {k}, the evening before {k_prev}"
    );
    // And in the morning, the last sighting before the following new moon is
    // within two days of it: the same rule, the other horizon.
    let last = next_asta(
        new_moon + 15.0,
        Graha::Moon,
        HeliacalSide::East,
        PUNE,
        &sun,
        &moon,
        HeliacalCriterion::Kalamsha,
    )
    .unwrap();
    let next_new = new_moon + 29.53;
    println!("last seen {last:.3}, the next new moon near {next_new:.2}");
    assert!(
        (last - next_new).abs() < 3.5,
        "last seen {:.2} days from the next new moon",
        last - next_new
    );
    assert!(last < next_new + 0.5);
}

/// Venus's four events in order over a synodic cycle, with the motion each
/// needs (Surya Siddhanta IX.2-3): it is seen again in the morning after
/// inferior conjunction while retrograde, last seen in the evening before it
/// (retrograde); last seen in the morning before superior conjunction and seen
/// again in the evening after it, both direct.
#[test]
fn venus_has_its_four_events_in_order_with_the_right_motion() {
    let sky = Sky {
        p: AnalyticalProvider::new(),
    };
    let sun = |t: f64| sky.eq(Body::Sun, t);
    let venus = |t: f64| sky.eq(Body::Venus, t);
    let rate = |t: f64| longitude_and_rate(&sky.p, Body::Venus, t).unwrap().1;
    let k = HeliacalCriterion::Kalamsha;
    let mut events: Vec<(f64, &str)> = Vec::new();
    let start = jd(2024, 1, 1, 0.0);
    let end = start + 1200.0;
    for (name, side, asta) in [
        ("udaya east", HeliacalSide::East, false),
        ("asta east", HeliacalSide::East, true),
        ("udaya west", HeliacalSide::West, false),
        ("asta west", HeliacalSide::West, true),
    ] {
        let mut t = start;
        while t < end {
            let r = if asta {
                next_asta(t, Graha::Venus, side, PUNE, &sun, &venus, k)
            } else {
                next_udaya(t, Graha::Venus, side, PUNE, &sun, &venus, k)
            };
            let Some(e) = r else { break };
            if e < end {
                events.push((e, name));
            }
            t = e + 1.0;
        }
    }
    events.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (t, name) in &events {
        println!("{t:.2} {name} (rate {:+.3})", rate(*t));
    }
    // Direction: retrograde for the two near inferior conjunction, direct for
    // the two near superior.
    for (t, name) in &events {
        let r = rate(*t);
        match *name {
            "udaya east" | "asta west" => assert!(r < 0.0, "{name} at {t}: rate {r}"),
            "asta east" | "udaya west" => assert!(r > 0.0, "{name} at {t}: rate {r}"),
            _ => unreachable!(),
        }
    }
    // Venus is a morning object after inferior conjunction until superior, an
    // evening object after superior until inferior, so the events at the Sun
    // alternate: asta west, udaya east, asta east, udaya west, asta west, ...
    // Opposition-like events do not exist for Venus (it never leaves the Sun's
    // neighbourhood), so there are exactly four to a synodic period of 584
    // days.
    assert!(events.len() >= 7, "{} events in 1200 days", events.len());
    let order = ["asta west", "udaya east", "asta east", "udaya west"];
    let first = order.iter().position(|n| *n == events[0].1).unwrap();
    for (i, (t, name)) in events.iter().enumerate() {
        assert_eq!(*name, order[(first + i) % 4], "event {i} at {t}");
    }
    // A cycle apart.
    let gap = events[4].0 - events[0].0;
    assert!(
        (570.0..600.0).contains(&gap),
        "events a cycle apart are {gap} days apart"
    );
}

/// With the altitude criterion the udaya is `heliacal_rise` of an instant in
/// the invisibility: the first morning the graha stands high enough.
#[test]
fn next_udaya_agrees_with_heliacal_rise_from_an_invisibility() {
    let sky = Sky {
        p: AnalyticalProvider::new(),
    };
    let sun = |t: f64| sky.eq(Body::Sun, t);
    for (graha, body, start) in [
        (Graha::Jupiter, Body::Jupiter, jd(2025, 6, 10, 0.0)),
        (Graha::Saturn, Body::Saturn, jd(2025, 3, 1, 0.0)),
        (Graha::Mars, Body::Mars, jd(2026, 1, 15, 0.0)),
    ] {
        let star = |t: f64| sky.eq(body, t);
        let crit = HeliacalCriterion::Altitude {
            critical_altitude_deg: 10.0,
        };
        let old = heliacal_rise(
            start,
            graha,
            PUNE.latitude_deg,
            PUNE.longitude_deg_east,
            &sun,
            &star,
            10.0,
        )
        .unwrap();
        let new = next_udaya(start, graha, HeliacalSide::East, PUNE, &sun, &star, crit).unwrap();
        assert!(
            (new - old).abs() * 86_400.0 < 1.0,
            "{graha:?}: {new} vs {old}"
        );
    }
}

/// `heliacal_rise` and `heliacal_set` keep their 9.4.1 answers: the values of
/// the unchanged functions on a model sky, taken before the finders were added.
#[test]
fn heliacal_rise_and_set_keep_their_answers() {
    let star = |t: f64| model_jupiter(t);
    // (start, latitude, longitude, heliacal_rise, heliacal_set), critical
    // altitude 10 degrees, Jupiter.
    let table = [
        (
            T0 - 150.0,
            0.0,
            0.0,
            2_460_034.099_692_398_7,
            2_460_366.599_648_646,
        ),
        (
            T0 - 150.0,
            12.0,
            40.0,
            2_460_034.988_581_170_3,
            2_460_365.488_537_681_3,
        ),
        (
            T0 + 20.0,
            0.0,
            0.0,
            2_460_034.099_692_398_7,
            2_460_366.599_648_646,
        ),
        (
            T0 + 20.0,
            12.0,
            40.0,
            2_460_034.988_581_170_3,
            2_460_365.488_537_681_3,
        ),
        (
            T0 + 300.0,
            0.0,
            0.0,
            2_460_434.099_639_764,
            2_460_766.599_596_010_5,
        ),
        (
            T0 + 300.0,
            12.0,
            40.0,
            2_460_434.988_528_535_7,
            2_460_765.488_485_045_7,
        ),
    ];
    for (start, lat, lon, rise, set) in table {
        let r = heliacal_rise(start, Graha::Jupiter, lat, lon, &model_sun, &star, 10.0).unwrap();
        let s = heliacal_set(start, Graha::Jupiter, lat, lon, &model_sun, &star, 10.0).unwrap();
        assert!(
            (r - rise).abs() < 1.0e-9,
            "rise from {start} at {lat}: {r} vs {rise}"
        );
        assert!(
            (s - set).abs() < 1.0e-9,
            "set from {start} at {lat}: {s} vs {set}"
        );
    }
}
