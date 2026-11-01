// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `khagola::graha_ingresses_by_longitude`: the longitude-only walk against
//! `graha_ingresses`, which refines with the daily motion.
//!
//! Both are fed by the same source, the engine's apparent positions (UT1 in);
//! the longitude-only callback is the `.0` of the one with the rate.

mod common;

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use common::{jd, longitude_and_rate};
use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::khagola::{graha_ingresses, graha_ingresses_by_longitude};

const AYAN: Ayanamsha = Ayanamsha::IndianOfficial;
const ONE_SECOND: f64 = 1.0 / 86_400.0;

/// The root tolerance stated in the function's doc: 1e-9 day.
const ROOT_TOL_D: f64 = 1.0e-9;

/// The bound the two variants' instants must agree within, in days. The
/// measured maximum is in the doc of `graha_ingresses_by_longitude`; this is
/// a hundredth of a second, a margin over it.
const AGREE_D: f64 = 0.01 / 86_400.0;

/// Same spans and grahas as `the_range_is_the_reseeded_walk_entry_for_entry`
/// in `khagola_ingress.rs`.
const CASES: [(Graha, Body, f64); 9] = [
    (Graha::Saturn, Body::Saturn, 10.0),
    (Graha::Jupiter, Body::Jupiter, 10.0),
    (Graha::Mars, Body::Mars, 6.0),
    (Graha::Mercury, Body::Mercury, 4.0),
    (Graha::Venus, Body::Venus, 6.0),
    (Graha::Sun, Body::Sun, 4.0),
    (Graha::Moon, Body::Moon, 0.5),
    (Graha::Rahu, Body::TrueNode, 4.0),
    (Graha::Ketu, Body::TrueSouthNode, 4.0),
];

fn sidereal(p: &(dyn EphemerisProvider + Sync), body: Body, t: f64) -> f64 {
    let (lon, _) = longitude_and_rate(p, body, t).unwrap();
    (lon - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0)
}

/// Both variants over every case: the same entries (count and rashi), the
/// instants within `AGREE_D`. Prints the largest difference it saw.
#[test]
fn by_longitude_returns_the_same_entries_as_the_rate_based_walk() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let mut worst = 0.0_f64;
    for (graha, body, years) in CASES {
        let to = from + years * 365.25;
        let with_rate = |t: f64| longitude_and_rate(&p, body, t);
        let lon_only = |t: f64| longitude_and_rate(&p, body, t).map(|x| x.0);
        let old = graha_ingresses(from, to, graha, AYAN, &with_rate);
        let new = graha_ingresses_by_longitude(from, to, graha, AYAN, &lon_only);
        assert!(!old.is_empty(), "{graha:?}: the span holds no ingress");
        assert_eq!(new.len(), old.len(), "{graha:?}: number of entries");
        let mut worst_case = 0.0_f64;
        for (a, b) in new.iter().zip(&old) {
            assert_eq!(a.1, b.1, "{graha:?}: rashi entered at {}", a.0);
            let dt = (a.0 - b.0).abs();
            worst_case = worst_case.max(dt);
            assert!(
                dt <= AGREE_D,
                "{graha:?}: by_longitude {} vs graha_ingresses {} ({} s apart)",
                a.0,
                b.0,
                dt * 86_400.0
            );
        }
        println!(
            "{graha:?}: {} entries, max |dt| {:.3e} d = {:.4} ms",
            new.len(),
            worst_case,
            worst_case * 86_400.0e3
        );
        worst = worst.max(worst_case);
    }
    println!(
        "MAX |dt| over all cases: {:.3e} d = {:.4} ms",
        worst,
        worst * 86_400.0e3
    );
}

/// An entry is where the sidereal longitude is on a rashi edge: the angle from
/// the nearest edge is the longitude's travel in a few root tolerances at
/// most, and the rashi entered is the one just after the instant, not the one
/// just before.
#[test]
fn each_entry_sits_on_a_rashi_edge() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    for (graha, body, years) in CASES {
        let to = from + years * 365.25;
        let lon_only = |t: f64| longitude_and_rate(&p, body, t).map(|x| x.0);
        let entries = graha_ingresses_by_longitude(from, to, graha, AYAN, &lon_only);
        assert!(!entries.is_empty(), "{graha:?}: no entries");
        for (t, r) in entries {
            let sid = sidereal(&p, body, t);
            let off = (sid + 15.0).rem_euclid(30.0) - 15.0;
            // The rate from a central difference of the positions, times the
            // tolerance, with a factor 4 for the callback's own rounding.
            let h = 0.001;
            let rate = ((sidereal(&p, body, t + h) - sidereal(&p, body, t - h) + 540.0)
                .rem_euclid(360.0)
                - 180.0)
                / (2.0 * h);
            assert!(
                off.abs() <= 4.0 * ROOT_TOL_D * rate.abs().max(0.01) + 1.0e-9,
                "{graha:?}: entry at {t} is {off} deg from an edge (rate {rate})"
            );
            let idx = |x: f64| (x / 30.0).floor() as u8 % 12;
            assert_eq!(
                idx(sidereal(&p, body, t + 1.0e-4)),
                r,
                "{graha:?}: rashi just after {t}"
            );
            assert_ne!(
                idx(sidereal(&p, body, t - 1.0e-4)),
                r,
                "{graha:?}: rashi just before {t}"
            );
        }
    }
}

#[test]
fn degenerate_ranges_are_empty() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let f = |t: f64| longitude_and_rate(&p, Body::Saturn, t).map(|x| x.0);
    let none: Vec<(f64, u8)> = Vec::new();
    // End at or before the start.
    assert_eq!(
        graha_ingresses_by_longitude(from, from, Graha::Saturn, AYAN, &f),
        none
    );
    assert_eq!(
        graha_ingresses_by_longitude(from, from - 10.0, Graha::Saturn, AYAN, &f),
        none
    );
    // A span inside one stay: from a second past an entry to a few days before
    // the next.
    let all = graha_ingresses_by_longitude(from, from + 4000.0, Graha::Saturn, AYAN, &f);
    let (entry, next) = (all[0].0, all[1].0);
    let quiet =
        graha_ingresses_by_longitude(entry + ONE_SECOND, next - 5.0, Graha::Saturn, AYAN, &f);
    assert!(quiet.is_empty(), "{quiet:?}");
    // No callback, no entries.
    let nothing = |_: f64| None;
    assert_eq!(
        graha_ingresses_by_longitude(from, from + 400.0, Graha::Saturn, AYAN, &nothing),
        none
    );
}

/// The interval is half open at `jd_end`, and starts where it is told.
#[test]
fn the_range_is_half_open_at_its_end() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let f = |t: f64| longitude_and_rate(&p, Body::Mars, t).map(|x| x.0);
    let run = |a: f64, b: f64| graha_ingresses_by_longitude(a, b, Graha::Mars, AYAN, &f);
    let all = run(from, from + 3.0 * 365.25);
    let (t, r) = all[1];
    assert_eq!(run(from, t - 1.0e-4).len(), 1);
    let after = run(from, t + 1.0e-4);
    assert_eq!(after.len(), 2);
    assert_eq!(after[1], (t, r));
    let later = run(all[0].0 + ONE_SECOND, from + 3.0 * 365.25);
    assert_eq!(later.first(), Some(&all[1]));
}

/// A callback faster than the bound (half a degree a day, labelled Saturn): the
/// one-day fall back finds every crossing, including the one at the 360/0 wrap,
/// at the instants the rate-based walk finds.
#[test]
fn a_callback_faster_than_its_bound_is_still_walked_correctly() {
    let t0 = jd(2020, 1, 1, 0.0);
    let ay = |t: f64| true_ayanamsha_value(AYAN, t);
    // Sidereal longitude 340 + 0.5 (t - t0): reaches 360 after 40 days.
    let f = |t: f64| Some((340.0 + 0.5 * (t - t0) + ay(t)).rem_euclid(360.0));
    let to = t0 + 390.0;
    let entries = graha_ingresses_by_longitude(t0, to, Graha::Saturn, AYAN, &f);
    assert!(entries.len() >= 6, "{} entries", entries.len());
    let reference = |t: f64| Some((f(t).unwrap(), 0.5));
    let old = graha_ingresses(t0, to, Graha::Saturn, AYAN, &reference);
    assert_eq!(entries.len(), old.len());
    for (a, b) in entries.iter().zip(&old) {
        assert_eq!(a.1, b.1);
        assert!((a.0 - b.0).abs() < 1.0e-6, "{} vs {}", a.0, b.0);
    }
    assert_eq!(entries[0].1, 0, "the first entry is Aries, across the wrap");
}

/// A body moving down: the crossing reports the rashi entered downward, read
/// from the rashi indices, across the 0/360 wrap.
#[test]
fn a_downward_crossing_is_the_rashi_entered_downward() {
    let t0 = jd(2020, 1, 1, 0.0);
    let ay = |t: f64| true_ayanamsha_value(AYAN, t);
    // Sidereal longitude 40 - 0.3 (t - t0): falls through 30 (Taurus into
    // Aries), 0 (Aries into Pisces) and 330 (Pisces into Aquarius).
    let f = |t: f64| Some((40.0 - 0.3 * (t - t0) + ay(t)).rem_euclid(360.0));
    let entries = graha_ingresses_by_longitude(t0, t0 + 260.0, Graha::Rahu, AYAN, &f);
    assert_eq!(
        entries.iter().map(|e| e.1).collect::<Vec<_>>(),
        vec![0, 11, 10]
    );
    for (t, _) in &entries {
        let sid = (f(*t).unwrap() - ay(*t)).rem_euclid(360.0);
        let off = (sid + 15.0).rem_euclid(30.0) - 15.0;
        assert!(off.abs() < 1.0e-8, "{off}");
    }
}

/// Reads of the callback, Moon over one year from 2001-01-01: the rate-based
/// walk against the longitude-only one. The variant exists for a source whose
/// motion costs about three position evaluations (a central difference), so
/// it must read fewer than twice as often as the rate-based walk: measured
/// 2,121 against 1,605 (2026-11-01). Plain bisection read 5,627 and fails.
#[test]
fn read_counts_for_the_moon_over_a_year() {
    let p = AnalyticalProvider::new();
    let from = jd(2001, 1, 1, 0.0);
    let to = from + 365.25;
    let calls_old = AtomicU64::new(0);
    let calls_new = AtomicU64::new(0);
    let with_rate = |t: f64| {
        calls_old.fetch_add(1, Relaxed);
        longitude_and_rate(&p, Body::Moon, t)
    };
    let lon_only = |t: f64| {
        calls_new.fetch_add(1, Relaxed);
        longitude_and_rate(&p, Body::Moon, t).map(|x| x.0)
    };
    let old = graha_ingresses(from, to, Graha::Moon, AYAN, &with_rate);
    let new = graha_ingresses_by_longitude(from, to, Graha::Moon, AYAN, &lon_only);
    let (o, n) = (calls_old.load(Relaxed), calls_new.load(Relaxed));
    println!(
        "Moon, 1 year, {} entries: rate-based {o} callback calls, longitude-only {n}",
        old.len()
    );
    assert_eq!(old.len(), new.len());
    assert!(
        n < 2 * o,
        "{n} longitude reads against {o} rate-based reads"
    );
}
