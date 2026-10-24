// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `khagola::graha_ingresses`: the range form against the one-ingress walk it
//! replaces, its read counts, and the speed bounds its steps rest on.
//!
//! The reference is `graha_ingress` itself, re-seeded one second past each
//! entry, and, where its 400-day window runs out, from the window's end. The
//! positions are the engine's apparent positions (UT1 in), the rate the
//! engine's daily motion.

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};

use common::{jd, longitude_and_rate};
use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::khagola::{graha_ingress, graha_ingresses, ingress_speed_bound_deg_per_day};

const AYAN: Ayanamsha = Ayanamsha::IndianOfficial;
const ONE_SECOND: f64 = 1.0 / 86_400.0;

/// `graha_ingress`, re-seeded a second past each entry, and from the end of its
/// 400-day window when it finds none: every ingress in `[from, to)`.
fn reseeded_walk(
    graha: Graha,
    f: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    from: f64,
    to: f64,
) -> Vec<(f64, u8)> {
    let mut out = Vec::new();
    let mut seed = from;
    while seed < to {
        match graha_ingress(seed, graha, AYAN, f) {
            Some((t, r)) => {
                if t >= to {
                    break;
                }
                out.push((t, r));
                seed = t + ONE_SECOND;
            }
            None => seed += 400.0,
        }
    }
    out
}

fn sidereal_rashi(f: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync), t: f64) -> u8 {
    let (lon, _) = f(t).unwrap();
    ((lon - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0) / 30.0).floor() as u8 % 12
}

/// (graha, body serving it, years from 2001-01-01, the most the range may read
/// the callback as a fraction of the one-day walk, where that is measured)
const CASES: [(Graha, Body, f64, Option<f64>); 9] = [
    (Graha::Saturn, Body::Saturn, 42.0, Some(0.09)),
    (Graha::Jupiter, Body::Jupiter, 42.0, Some(0.17)),
    (Graha::Mars, Body::Mars, 12.0, Some(0.38)),
    (Graha::Mercury, Body::Mercury, 4.0, None),
    (Graha::Venus, Body::Venus, 6.0, None),
    (Graha::Sun, Body::Sun, 4.0, None),
    (Graha::Moon, Body::Moon, 0.5, None),
    (Graha::Rahu, Body::TrueNode, 10.0, None),
    (Graha::Ketu, Body::TrueSouthNode, 10.0, None),
];

/// The range form against the walk it replaces, entry for entry, and what it
/// costs: the reads of the callback, which are the cost the range form exists
/// to cut (the one-day walk reads once a day; the range form once per whole-day
/// step the speed bound allows).
#[test]
fn the_range_is_the_reseeded_walk_entry_for_entry_and_reads_less() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    for (graha, body, years, at_most) in CASES {
        let to = from + years * 365.25;
        let reads = AtomicUsize::new(0);
        let f = |t: f64| {
            reads.fetch_add(1, Ordering::Relaxed);
            longitude_and_rate(&p, body, t)
        };
        let walk = reseeded_walk(graha, &f, from, to);
        let walk_reads = reads.swap(0, Ordering::Relaxed);
        let range = graha_ingresses(from, to, graha, AYAN, &f);
        let range_reads = reads.load(Ordering::Relaxed);
        assert!(
            !walk.is_empty(),
            "{graha:?}: the span holds no ingress, nothing is tested"
        );
        assert_eq!(range.len(), walk.len(), "{graha:?}: number of entries");
        let mut worst = 0.0_f64;
        for (a, b) in range.iter().zip(&walk) {
            assert_eq!(a.1, b.1, "{graha:?}: rashi entered at {}", a.0);
            worst = worst.max((a.0 - b.0).abs());
        }
        let ratio = range_reads as f64 / walk_reads as f64;
        println!(
            "{graha:?}: {} entries, worst difference {worst:e} d, {walk_reads} reads one day at a time, {range_reads} by the range: {ratio:.3}",
            walk.len()
        );
        // The refinement stops within 1e-6 deg of the edge, so two answers are
        // at most 2e-6 deg apart; at the daily motion at the crossing that is a
        // time (a station next to an edge only widens it).
        for (a, b) in range.iter().zip(&walk) {
            let rate = f(a.0).unwrap().1.abs().max(0.01);
            assert!(
                (a.0 - b.0).abs() * rate <= 2.0e-6,
                "{graha:?}: {} vs {} ({} s)",
                a.0,
                b.0,
                (a.0 - b.0) * 86_400.0
            );
        }
        if let Some(at_most) = at_most {
            assert!(
                ratio <= at_most,
                "{graha:?}: {ratio:.3} of the walk's reads, expected under {at_most}"
            );
        }
    }
}

/// Between two consecutive entries the graha stays in the rashi the first one
/// entered; and each entry moves to a neighbouring rashi (a graha does not
/// jump a sign), the first from the rashi it stood in at the start. Both come
/// from what an ingress is, not from the walk.
#[test]
fn between_entries_the_graha_stays_in_the_rashi_entered() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    for (graha, body, years) in [
        (Graha::Saturn, Body::Saturn, 30.0),
        (Graha::Mars, Body::Mars, 8.0),
        (Graha::Mercury, Body::Mercury, 3.0),
        (Graha::Rahu, Body::TrueNode, 10.0),
    ] {
        let to = from + years * 365.25;
        let f = |t: f64| longitude_and_rate(&p, body, t);
        let entries = graha_ingresses(from, to, graha, AYAN, &f);
        assert!(entries.len() >= 5, "{graha:?}: {} entries", entries.len());
        let mut before = sidereal_rashi(&f, from);
        for w in entries.windows(2) {
            let step = (w[0].1 + 12 - before) % 12;
            assert!(
                step == 1 || step == 11,
                "{graha:?}: {before} -> {} at {}",
                w[0].1,
                w[0].0
            );
            let mid = 0.5 * (w[0].0 + w[1].0);
            assert_eq!(
                sidereal_rashi(&f, mid),
                w[0].1,
                "{graha:?}: rashi at the midpoint of {:?}",
                w
            );
            before = w[0].1;
        }
        let last = entries.last().unwrap();
        assert_eq!(
            sidereal_rashi(&f, 0.5 * (last.0 + to)),
            last.1,
            "{graha:?}: after the last entry"
        );
    }
}

/// An entry is where the sidereal longitude is on a rashi edge, to the
/// refinement's 1e-6 degree.
#[test]
fn each_entry_sits_on_a_rashi_edge() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let f = |t: f64| longitude_and_rate(&p, Body::Mars, t);
    for (t, _) in graha_ingresses(from, from + 8.0 * 365.25, Graha::Mars, AYAN, &f) {
        let (lon, _) = f(t).unwrap();
        let sid = (lon - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0);
        let off = (sid + 15.0).rem_euclid(30.0) - 15.0;
        assert!(off.abs() < 2.0e-6, "entry at {t} is {off} deg from an edge");
    }
}

#[test]
fn degenerate_ranges_are_empty() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let f = |t: f64| longitude_and_rate(&p, Body::Saturn, t);
    // End at or before the start.
    assert!(graha_ingresses(from, from, Graha::Saturn, AYAN, &f).is_empty());
    assert!(graha_ingresses(from, from - 10.0, Graha::Saturn, AYAN, &f).is_empty());
    // A span inside one stay: from a second past an entry to a few days before
    // the next, which the one-day walk locates.
    let walk = reseeded_walk(Graha::Saturn, &f, from, from + 4000.0);
    let (entry, next) = (walk[0].0, walk[1].0);
    let quiet = graha_ingresses(entry + ONE_SECOND, next - 5.0, Graha::Saturn, AYAN, &f);
    assert!(quiet.is_empty(), "{quiet:?}");
    // No callback, no entries.
    let none = |_: f64| None;
    assert!(graha_ingresses(from, from + 400.0, Graha::Saturn, AYAN, &none).is_empty());
}

/// A range that ends just before an entry omits it, and one that ends just
/// after includes it: the interval is half open at `jd_end`.
#[test]
fn the_range_is_half_open_at_its_end() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let f = |t: f64| longitude_and_rate(&p, Body::Mars, t);
    let all = graha_ingresses(from, from + 3.0 * 365.25, Graha::Mars, AYAN, &f);
    let (t, r) = all[1];
    let before = graha_ingresses(from, t - 1.0e-4, Graha::Mars, AYAN, &f);
    let after = graha_ingresses(from, t + 1.0e-4, Graha::Mars, AYAN, &f);
    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 2);
    assert_eq!(after[1], (t, r));
    // And it starts where it is told: from just after the first entry the
    // second comes first.
    let later = graha_ingresses(
        all[0].0 + ONE_SECOND,
        from + 3.0 * 365.25,
        Graha::Mars,
        AYAN,
        &f,
    );
    assert_eq!(later.first(), Some(&all[1]));
}

/// A callback faster than the graha's bound: a body moving half a degree a
/// day labelled Saturn, whose bound is a few hundredths. The long steps the
/// bound allows run past an edge; the range form notices the sample has changed
/// rashi and falls back to single days, finding what the one-day walk finds.
#[test]
fn a_callback_faster_than_its_bound_is_still_walked_correctly() {
    let t0 = jd(2020, 1, 1, 0.0);
    let f = |t: f64| Some((100.0 + 0.5 * (t - t0), 0.5));
    let to = t0 + 390.0;
    let walk = reseeded_walk(Graha::Saturn, &f, t0, to);
    let range = graha_ingresses(t0, to, Graha::Saturn, AYAN, &f);
    assert!(
        walk.len() >= 6,
        "{} entries in 390 days at half a degree a day",
        walk.len()
    );
    assert_eq!(range, walk);
}

// ── the speed bounds ────────────────────────────────────────────

/// The largest rate, in degrees a day, at which the line from the Earth to a
/// planet turns when both move on Kepler ellipses, over every phase of each and
/// every relative orientation of the two orbits: `(direct, retrograde)`.
///
/// With the planet at `r_p` and the Earth at `r_e`, velocities `v_p`, `v_e`, the
/// line turns at `(d x dv) / |d|^2`, `d = r_p - r_e`, `dv = v_p - v_e`. A Kepler
/// orbit of semi-major axis `a` and eccentricity `e` has, at true anomaly `nu`,
/// `p = a (1 - e^2)`, `r = p / (1 + e cos nu)` and, in its own plane,
/// `v = (k / sqrt(p)) (-sin nu, e + cos nu)`, `k` the Gaussian constant.
fn two_body_supremum(a: f64, e: f64) -> (f64, f64) {
    const K: f64 = 0.017_202_098_95; // Gaussian gravitational constant, AU^1.5 / day
    let state = |a: f64, e: f64, nu: f64| {
        let p = a * (1.0 - e * e);
        let r = p / (1.0 + e * nu.cos());
        let f = K / p.sqrt();
        (
            (r * nu.cos(), r * nu.sin()),
            (-f * nu.sin(), f * (e + nu.cos())),
        )
    };
    let (mut direct, mut retro) = (f64::MIN, f64::MAX);
    const N: usize = 36;
    for i in 0..N {
        let (rp, vp) = state(a, e, std::f64::consts::TAU * i as f64 / N as f64);
        for j in 0..N {
            let (re, ve) = state(1.0, 0.01671, std::f64::consts::TAU * j as f64 / N as f64);
            for w in 0..N {
                // Turn the planet's orbit by the angle between the perihelia.
                let (s, c) = (std::f64::consts::TAU * w as f64 / N as f64).sin_cos();
                let rot = |v: (f64, f64)| (c * v.0 - s * v.1, s * v.0 + c * v.1);
                let (rpp, vpp) = (rot(rp), rot(vp));
                let (dx, dy) = (rpp.0 - re.0, rpp.1 - re.1);
                let (dvx, dvy) = (vpp.0 - ve.0, vpp.1 - ve.1);
                let rate = ((dx * dvy - dy * dvx) / (dx * dx + dy * dy)).to_degrees();
                direct = direct.max(rate);
                retro = retro.min(rate);
            }
        }
    }
    (direct, retro)
}

/// The bound is above the two-body supremum for each planet, which is a
/// property of the orbits and not of this engine, by at least 3 per cent, and
/// above it still if the elements are 0.5 per cent off in `a` and 0.01 in `e`.
#[test]
fn speed_bounds_exceed_the_two_body_supremum() {
    // (graha, a, e), the rounded mean values.
    let planets = [
        (Graha::Mercury, 0.3871, 0.2056),
        (Graha::Venus, 0.7233, 0.0068),
        (Graha::Mars, 1.5237, 0.0934),
        (Graha::Jupiter, 5.2034, 0.0484),
        (Graha::Saturn, 9.5371, 0.0542),
    ];
    for (graha, a, e) in planets {
        let bound = ingress_speed_bound_deg_per_day(graha);
        let (direct, retro) = two_body_supremum(a, e);
        println!("{graha:?}: supremum {direct:.4} / {retro:.4}, bound {bound}");
        assert!(
            bound >= 1.03 * direct,
            "{graha:?}: bound {bound} against supremum {direct}"
        );
        assert!(
            bound >= 1.03 * retro.abs(),
            "{graha:?}: retrograde supremum {retro}"
        );
        // The figure in the table of the bound's doc.
        let (d2, _) = two_body_supremum(a * 1.005, (e + 0.01).min(0.9));
        assert!(
            bound >= d2,
            "{graha:?}: a slightly different orbit reaches {d2}"
        );
    }
    // The Sun: the Earth's own rate at perihelion, n (1+e)^2 / (1-e^2)^1.5.
    let n = 360.0 / 365.256_36;
    let e: f64 = 0.01671;
    let sun = n * (1.0 + e).powi(2) / (1.0 - e * e).powf(1.5);
    assert!(ingress_speed_bound_deg_per_day(Graha::Sun) >= 1.03 * sun);
}

/// The bound against this engine's positions: the largest change of sidereal
/// longitude over a window of one day, anywhere in the span, stays inside it.
/// `scale` stretches the span of each case (and thins its sampling to match:
/// 1 is the default test, 20 the two centuries of the full run).
fn assert_bounds_hold(from: f64, scale: f64) {
    let p = AnalyticalProvider::new();
    // (graha, body, years, sampling step in days)
    let cases = [
        (Graha::Sun, Body::Sun, 10.0, 2.0),
        (Graha::Mercury, Body::Mercury, 10.0, 1.0),
        (Graha::Venus, Body::Venus, 10.0, 2.0),
        (Graha::Mars, Body::Mars, 10.0, 2.0),
        (Graha::Jupiter, Body::Jupiter, 10.0, 4.0),
        (Graha::Saturn, Body::Saturn, 10.0, 4.0),
        (Graha::Rahu, Body::TrueNode, 3.0, 0.5),
        (Graha::Rahu, Body::MeanNode, 10.0, 80.0),
        (Graha::Moon, Body::Moon, 1.0, 0.25),
    ];
    for (graha, body, years, step) in cases {
        let bound = ingress_speed_bound_deg_per_day(graha);
        let sid = |t: f64| {
            let (lon, rate) = longitude_and_rate(&p, body, t).unwrap();
            (
                (lon - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0),
                rate,
            )
        };
        let (mut worst_window, mut worst_rate) = (0.0_f64, 0.0_f64);
        let (mut t, to) = (from, from + years * scale * 365.25);
        while t < to {
            let (a, rate) = sid(t);
            let (b, _) = sid(t + 1.0);
            let moved = ((b - a + 180.0).rem_euclid(360.0) - 180.0).abs();
            worst_window = worst_window.max(moved);
            worst_rate = worst_rate.max(rate.abs());
            t += step;
        }
        println!(
            "{graha:?} ({body:?}): at most {worst_window:.4} deg in a day, rate {worst_rate:.4}, bound {bound}"
        );
        assert!(
            worst_window <= bound,
            "{graha:?}: {worst_window} deg in a day against {bound}"
        );
        assert!(
            worst_rate <= bound,
            "{graha:?}: rate {worst_rate} against {bound}"
        );
    }
}

#[test]
fn speed_bounds_hold_over_a_decade() {
    assert_bounds_hold(jd(2010, 1, 1, 0.0), 1.0);
}

/// 1900 to 2100 (twenty times the decade, the Moon and the true node for a
/// twentieth of it), the span the table in the bound's doc was measured over.
#[test]
#[ignore = "about five minutes in the test profile; run with --include-ignored (full validation)"]
fn speed_bounds_hold_over_two_centuries() {
    assert_bounds_hold(jd(1900, 1, 1, 0.0), 20.0);
}
