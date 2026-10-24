// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `refine_crossing` and the `compute_*_end` functions on the engine's own
//! positions, against roots found a different way.
//!
//! The reference for each end is a bisection on the angle itself, taken from
//! `ecliptic_position` (position only, no daily motion): no Newton step, no
//! rate, halved until the bracket is a single Julian Day quantum wide. The
//! function under test reads `apparent_position`, whose rate is a central
//! difference over a day and so differs from the true rate by a few parts in a
//! thousand for the Moon. The rate may steer the iteration but must not move its
//! root, which until v9.5.0 it did, by up to the 1e-6 degree the iteration
//! stopped at (7.9 ms for the Moon).

mod common;

use common::{jd, longitude_and_rate};
use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::ecliptic_position;
use vedaksha_vedic::muhurta::{
    MoonAndSun, compute_karana_end, compute_nakshatra_end, compute_tithi_end, compute_yoga_end,
};

const AYAN: Ayanamsha = Ayanamsha::IndianOfficial;

/// Two Julian Day quanta near 2.46 million is 9.3e-10 day; four is the bound.
const TOLERANCE_DAYS: f64 = 2.0e-9;

fn lon(p: &AnalyticalProvider, body: Body, t: f64) -> f64 {
    ecliptic_position(p, body, t)
        .unwrap()
        .longitude
        .to_degrees()
}

/// `(target - angle)` wrapped to `(-180, 180]`: positive before the crossing.
fn gap(target: f64, angle: f64) -> f64 {
    let g = (target - angle).rem_euclid(360.0);
    if g > 180.0 { g - 360.0 } else { g }
}

/// The instant `angle` reaches `target`, by bisection around `near`.
fn bisected_root(target: f64, near: f64, angle: &dyn Fn(f64) -> f64) -> f64 {
    let (mut lo, mut hi) = (near - 0.02, near + 0.02);
    assert!(
        gap(target, angle(lo)) > 0.0 && gap(target, angle(hi)) < 0.0,
        "no crossing of {target} within 0.02 day of {near}"
    );
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi {
            break;
        }
        if gap(target, angle(mid)) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Run `end` for every day of 90 and check each answer against the bisected root
/// of the boundary it aimed at; the boundary is the nearest multiple of `span`
/// to the angle at the answer.
fn check(label: &str, span: f64, angle: &dyn Fn(f64) -> f64, end: &dyn Fn(f64) -> Option<f64>) {
    let start = jd(2026, 3, 1, 0.0);
    let mut worst = 0.0_f64;
    let mut checked = 0;
    for day in 0..90 {
        let t = start + f64::from(day) + 0.37;
        let got = end(t).unwrap_or_else(|| panic!("{label}: no end after {t}"));
        let a = angle(got);
        let target = ((a / span).round() * span).rem_euclid(360.0);
        let root = bisected_root(target, got, angle);
        let off = (got - root).abs();
        assert!(
            off <= TOLERANCE_DAYS,
            "{label} from {t}: {got} is {:.3e} ms from the bisected root {root}",
            off * 86_400.0e3
        );
        worst = worst.max(off);
        checked += 1;
    }
    assert_eq!(checked, 90);
    println!(
        "{label}: worst {:.3e} ms from the bisected root",
        worst * 86_400.0e3
    );
}

#[test]
fn the_nakshatra_end_is_the_bisected_root() {
    let p = AnalyticalProvider::new();
    let sid = |t: f64| (lon(&p, Body::Moon, t) - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0);
    let moon = |t: f64| {
        let (l, r) = longitude_and_rate(&p, Body::Moon, t)?;
        Some(((l - true_ayanamsha_value(AYAN, t)).rem_euclid(360.0), r))
    };
    check("nakshatra end", 360.0 / 27.0, &sid, &|t| {
        compute_nakshatra_end(t, &moon)
    });
}

fn pair(p: &AnalyticalProvider, t: f64) -> Option<MoonAndSun> {
    Some((
        longitude_and_rate(p, Body::Moon, t)?,
        longitude_and_rate(p, Body::Sun, t)?,
    ))
}

#[test]
fn the_tithi_and_karana_ends_are_the_bisected_root() {
    let p = AnalyticalProvider::new();
    let elongation = |t: f64| (lon(&p, Body::Moon, t) - lon(&p, Body::Sun, t)).rem_euclid(360.0);
    let both = |t: f64| pair(&p, t);
    check("tithi end", 12.0, &elongation, &|t| {
        compute_tithi_end(t, &both)
    });
    check("karana end", 6.0, &elongation, &|t| {
        compute_karana_end(t, &both)
    });
}

#[test]
fn the_yoga_end_is_the_bisected_root() {
    let p = AnalyticalProvider::new();
    let sum = |t: f64| {
        (lon(&p, Body::Moon, t) + lon(&p, Body::Sun, t) - 2.0 * true_ayanamsha_value(AYAN, t))
            .rem_euclid(360.0)
    };
    let sidereal_pair = |t: f64| {
        let ((ml, ms), (sl, ss)) = pair(&p, t)?;
        let aya = true_ayanamsha_value(AYAN, t);
        Some((
            ((ml - aya).rem_euclid(360.0), ms),
            ((sl - aya).rem_euclid(360.0), ss),
        ))
    };
    check("yoga end", 360.0 / 27.0, &sum, &|t| {
        compute_yoga_end(t, &sidereal_pair)
    });
}
