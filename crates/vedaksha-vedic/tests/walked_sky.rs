// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! `WalkedSky` against the callbacks a consumer builds today, scan by scan: the
//! same scan run twice on the analytical provider, once with `apparent_position`,
//! `apparent_positions` and `ecliptic_position` behind its callbacks and once
//! with a `WalkedSky`. Each test holds the two to the same answers within a stated
//! time, and holds the walked run to a stated fraction of the provider calls.
//!
//! The time is 2e-9 day (0.17 ms, four Julian Day quanta) for the scans that
//! refine with `refine_crossing`, which runs to the quantum, and the scan's own
//! bisection width for those that bisect (`search_transits`, 1e-8 day).

mod common;

use common::{Counting, jd, longitude_and_rate};
use vedaksha_astro::moon_riseset::{moon_equatorial, moon_rise_set};
use vedaksha_astro::riseset::{RiseSet, sun_equatorial_deg, sun_rise_set};
use vedaksha_astro::sidereal::{Ayanamsha, true_tropical_to_sidereal};
use vedaksha_astro::transits::{TransitEvent, TransitSearchConfig, lunar_return, search_transits};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{apparent_positions, ecliptic_position};
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::kala_mana::{MasaScheme, MasaSpan, lunar_month_span, year_turnover};
use vedaksha_vedic::khagola::{Grahana, graha_ingress, graha_ingresses, grahana};
use vedaksha_vedic::muhurta::{
    MoonAndSun, MuhurtaAssessment, compute_karana_end, compute_nakshatra_end, compute_tithi_end,
    compute_yoga_end, search_muhurta,
};
use vedaksha_vedic::walked::WalkedSky;

const AYAN: Ayanamsha = Ayanamsha::IndianOfficial;
/// New Delhi.
const LAT: f64 = 28.6139;
const LON: f64 = 77.2090;
/// Four Julian Day quanta near 2.46 million.
const ROOT_TOL: f64 = 2.0e-9;

/// A scan's answer and the provider calls it made.
struct Run<T> {
    out: T,
    calls: u64,
}

fn direct<T>(scan: impl FnOnce(&Counting) -> T) -> Run<T> {
    let provider = Counting::new();
    let out = scan(&provider);
    Run {
        out,
        calls: provider.total(),
    }
}

fn walked<T>(start: f64, end: f64, scan: impl FnOnce(&WalkedSky) -> T) -> Run<T> {
    let provider = Counting::new();
    let sky = WalkedSky::new(&provider, start, end);
    let out = scan(&sky);
    Run {
        out,
        calls: provider.total(),
    }
}

/// The Moon and the Sun together as the MCP server builds the pair.
fn direct_pair(p: &Counting, t: f64) -> Option<MoonAndSun> {
    let mut it = apparent_positions(p, &[Body::Moon, Body::Sun], t).into_iter();
    let moon = it.next()?.1.ok()?;
    let sun = it.next()?.1.ok()?;
    Some((
        (moon.ecliptic.longitude.to_degrees(), moon.longitude_speed),
        (sun.ecliptic.longitude.to_degrees(), sun.longitude_speed),
    ))
}

fn sidereal(lon: f64, t: f64) -> f64 {
    true_tropical_to_sidereal(lon, AYAN, t)
}

/// The walked run made at most `1/factor` of the direct run's provider calls.
fn assert_saves<T, U>(what: &str, direct: &Run<T>, walked: &Run<U>, factor: u64) {
    println!(
        "{what}: {} provider calls direct, {} walked ({:.1}x)",
        direct.calls,
        walked.calls,
        direct.calls as f64 / walked.calls.max(1) as f64
    );
    assert!(
        walked.calls * factor <= direct.calls,
        "{what}: walked {} calls against direct {}, under the {factor}x it should save",
        walked.calls,
        direct.calls
    );
}

/// Two lists of event times pair up one for one and agree within `tol` days.
fn assert_times(what: &str, a: &[f64], b: &[f64], tol: f64) {
    assert_eq!(a.len(), b.len(), "{what}: number of events");
    assert!(!a.is_empty(), "{what}: no events, nothing is tested");
    let mut worst = 0.0_f64;
    for (x, y) in a.iter().zip(b) {
        worst = worst.max((x - y).abs());
        assert!(
            (x - y).abs() <= tol,
            "{what}: {x} direct, {y} walked ({:.3e} ms apart)",
            (x - y).abs() * 86_400.0e3
        );
    }
    println!(
        "{what}: {} events, worst {:.3e} ms apart",
        a.len(),
        worst * 86_400.0e3
    );
}

fn flat(options: impl IntoIterator<Item = Option<f64>>) -> Vec<f64> {
    options.into_iter().flatten().collect()
}

// ── panchang ends ───────────────────────────────────────────────

#[test]
fn the_panchang_ends_of_two_months() {
    let start = jd(2026, 3, 1, 0.0);
    let days: Vec<f64> = (0..60).map(|d| start + f64::from(d)).collect();

    let a = direct(|p| {
        let pair = |t: f64| direct_pair(p, t);
        let pair_sid = |t: f64| {
            let ((ml, mr), (sl, sr)) = direct_pair(p, t)?;
            Some(((sidereal(ml, t), mr), (sidereal(sl, t), sr)))
        };
        let moon_sid = |t: f64| {
            let (l, r) = longitude_and_rate(p, Body::Moon, t)?;
            Some((sidereal(l, t), r))
        };
        let mut out = Vec::new();
        for &d in &days {
            out.push(compute_tithi_end(d, &pair));
            out.push(compute_nakshatra_end(d, &moon_sid));
            out.push(compute_karana_end(d, &pair));
            out.push(compute_yoga_end(d, &pair_sid));
        }
        flat(out)
    });
    let b = walked(start - 2.0, start + 64.0, |sky| {
        let pair = |t: f64| sky.moon_and_sun(t);
        let pair_sid = |t: f64| sky.moon_and_sun_sidereal(AYAN, t);
        let moon_sid = |t: f64| sky.moon_sidereal_longitude_rate(AYAN, t);
        let mut out = Vec::new();
        for &d in &days {
            out.push(compute_tithi_end(d, &pair));
            out.push(compute_nakshatra_end(d, &moon_sid));
            out.push(compute_karana_end(d, &pair));
            out.push(compute_yoga_end(d, &pair_sid));
        }
        flat(out)
    });
    assert_eq!(a.out.len(), 240, "one end per limb per day");
    assert_times("panchang ends", &a.out, &b.out, ROOT_TOL);
    assert_saves("panchang ends", &a, &b, 10);
}

// ── the lunar calendar ──────────────────────────────────────────

#[test]
fn the_lunar_month_of_every_fourth_day_of_four_months() {
    let start = jd(2026, 3, 1, 0.0);
    let days: Vec<f64> = (0..30).map(|k| start + 4.0 * f64::from(k)).collect();
    let spans =
        |get: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync)| -> Vec<(MasaScheme, Option<MasaSpan>)> {
            let mut out = Vec::new();
            for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
                for &d in &days {
                    out.push((scheme, lunar_month_span(d, scheme, AYAN, get)));
                }
            }
            out
        };
    let a = direct(|p| spans(&|t| direct_pair(p, t)));
    let b = walked(start - 70.0, start + 190.0, |sky| {
        spans(&|t| sky.moon_and_sun(t))
    });
    let times = |v: &[(MasaScheme, Option<MasaSpan>)]| -> Vec<f64> {
        v.iter()
            .flat_map(|(_, s)| s.iter().flat_map(|s| [s.start_jd, s.end_jd]))
            .collect()
    };
    for ((_, x), (_, y)) in a.out.iter().zip(&b.out) {
        assert_eq!(x.map(|s| s.masa), y.map(|s| s.masa), "the month named");
    }
    assert_times(
        "lunar month spans",
        &times(&a.out),
        &times(&b.out),
        ROOT_TOL,
    );
    assert_saves("lunar month spans", &a, &b, 8);
}

#[test]
fn the_year_turnover_of_four_years() {
    let a = direct(|p| {
        let f = |t: f64| direct_pair(p, t);
        let mut out = Vec::new();
        for year in 2026..2030 {
            for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
                out.push(year_turnover(year, scheme, AYAN, &f));
            }
        }
        flat(out)
    });
    let b = walked(jd(2026, 1, 1, 0.0) - 90.0, jd(2030, 1, 1, 0.0), |sky| {
        let f = |t: f64| sky.moon_and_sun(t);
        let mut out = Vec::new();
        for year in 2026..2030 {
            for scheme in [MasaScheme::Amanta, MasaScheme::Purnimanta] {
                out.push(year_turnover(year, scheme, AYAN, &f));
            }
        }
        flat(out)
    });
    assert_times("year turnover", &a.out, &b.out, ROOT_TOL);
    // One call covers a few months and reads the Moon about once a day: the saving
    // is real and small. Four years of them through one sky is what it buys.
    assert_saves("year turnover", &a, &b, 1);
    assert!(b.calls < a.calls, "no saving at all");
}

// ── the Moon's ingresses ────────────────────────────────────────

#[test]
fn the_moons_ingresses_of_four_months() {
    let start = jd(2026, 3, 1, 0.0);
    let end = start + 120.0;
    let scan = |f: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync)| {
        let range: Vec<(f64, u8)> = graha_ingresses(start, end, Graha::Moon, AYAN, f);
        let mut chain = Vec::new();
        let mut seed = start;
        while let Some((t, r)) = graha_ingress(seed, Graha::Moon, AYAN, f) {
            if t >= end {
                break;
            }
            chain.push((t, r));
            seed = t + 1.0 / 86_400.0;
        }
        (range, chain)
    };
    let a = direct(|p| scan(&|t| longitude_and_rate(p, Body::Moon, t)));
    let b = walked(start - 2.0, end + 410.0, |sky| {
        scan(&|t| sky.moon_longitude_rate(t))
    });
    let times = |v: &[(f64, u8)]| v.iter().map(|e| e.0).collect::<Vec<_>>();
    let rashis = |v: &[(f64, u8)]| v.iter().map(|e| e.1).collect::<Vec<_>>();
    assert_eq!(rashis(&a.out.0), rashis(&b.out.0), "rashi entered, range");
    assert_eq!(rashis(&a.out.1), rashis(&b.out.1), "rashi entered, chain");
    assert_times(
        "Moon ingresses, range",
        &times(&a.out.0),
        &times(&b.out.0),
        ROOT_TOL,
    );
    assert_times(
        "Moon ingresses, chain",
        &times(&a.out.1),
        &times(&b.out.1),
        ROOT_TOL,
    );
    assert_saves("Moon ingresses", &a, &b, 3);
}

// ── eclipses ────────────────────────────────────────────────────

/// Sparsha and moksha, which are roots, agree to the quantum. Madhya, the instant
/// of least separation, is found by a ternary search on a function that is flat to
/// 1e-11 degree there: the direct path and the walked path each carry that
/// search's own uncertainty, about 10 ms, so madhya is held to 50 ms and the
/// walked eclipse is opt-in until the search is replaced by a root of the
/// separation's derivative.
#[test]
fn the_eclipses_of_two_years() {
    let start = jd(2026, 1, 1, 0.0);
    let end = start + 2.0 * 365.25;
    let scan =
        |lunar: &(dyn Fn(f64) -> Option<vedaksha_vedic::khagola::LunarGeometry> + Sync),
         solar: &(dyn Fn(f64) -> Option<vedaksha_vedic::khagola::SolarGeometry> + Sync)| {
            let mut out: Vec<Grahana> = Vec::new();
            let mut seed = start;
            while seed < end {
                let Some(g) = grahana(seed, LAT, LON, lunar, solar) else {
                    break;
                };
                out.push(g);
                seed = g.madhya_jd + 10.0;
            }
            out
        };
    let a = direct(|p| {
        scan(&|t| common::lunar_geometry(p, t), &|t| {
            common::solar_geometry(p, t)
        })
    });
    let b = walked(start - 2.0, end + 300.0, |sky| {
        scan(&|t| sky.lunar_geometry(t), &|t| sky.solar_geometry(t))
    });
    assert_eq!(a.out.len(), b.out.len(), "number of eclipses");
    assert!(a.out.len() >= 4, "{} eclipses in two years", a.out.len());
    for (x, y) in a.out.iter().zip(&b.out) {
        assert_eq!(x.kind, y.kind);
    }
    let pick = |v: &[Grahana], f: fn(&Grahana) -> f64| v.iter().map(f).collect::<Vec<_>>();
    assert_times(
        "eclipse sparsha",
        &pick(&a.out, |g| g.sparsha_jd),
        &pick(&b.out, |g| g.sparsha_jd),
        ROOT_TOL,
    );
    assert_times(
        "eclipse moksha",
        &pick(&a.out, |g| g.moksha_jd),
        &pick(&b.out, |g| g.moksha_jd),
        ROOT_TOL,
    );
    assert_times(
        "eclipse madhya",
        &pick(&a.out, |g| g.madhya_jd),
        &pick(&b.out, |g| g.madhya_jd),
        50.0e-3 / 86_400.0,
    );
    assert_saves("eclipses", &a, &b, 8);
}

// ── rise and set ────────────────────────────────────────────────

#[test]
fn moonrise_moonset_and_transit_of_a_month() {
    let start = jd(2026, 3, 1, 0.0);
    let days: Vec<f64> = (0..30).map(|d| start + f64::from(d)).collect();
    let times = |v: Vec<RiseSet>| -> Vec<f64> {
        v.into_iter()
            .flat_map(|r| [r.rise, r.set, r.transit])
            .flatten()
            .collect()
    };
    let a = direct(|p| {
        let f = |t: f64| moon_equatorial(p, t);
        times(
            days.iter()
                .map(|&d| moon_rise_set(d, LAT, LON, 0.0, &f))
                .collect(),
        )
    });
    let b = walked(start - 1.0, start + 32.0, |sky| {
        let f = |t: f64| sky.moon_equatorial(t);
        times(
            days.iter()
                .map(|&d| moon_rise_set(d, LAT, LON, 0.0, &f))
                .collect(),
        )
    });
    assert_times("moonrise, moonset, transit", &a.out, &b.out, ROOT_TOL);
    assert_saves("moonrise, moonset, transit", &a, &b, 8);
}

#[test]
fn sunrise_sunset_and_transit_of_a_year() {
    let start = jd(2026, 1, 1, 0.0);
    let days: Vec<f64> = (0..365).map(|d| start + f64::from(d)).collect();
    let times = |v: Vec<RiseSet>| -> Vec<f64> {
        v.into_iter()
            .flat_map(|r| [r.rise, r.set, r.transit])
            .flatten()
            .collect()
    };
    let a = direct(|p| {
        let f = |t: f64| sun_equatorial_deg(p, t);
        times(
            days.iter()
                .map(|&d| sun_rise_set(d, LAT, LON, 0.0, &f))
                .collect(),
        )
    });
    let b = walked(start - 1.0, start + 367.0, |sky| {
        let f = |t: f64| sky.sun_equatorial(t);
        times(
            days.iter()
                .map(|&d| sun_rise_set(d, LAT, LON, 0.0, &f))
                .collect(),
        )
    });
    // A rise, a set and a transit every day but the one or two whose UT day holds
    // none of one (the search takes the first event of each UT day).
    assert!(a.out.len() >= 3 * 365 - 3, "{} events", a.out.len());
    assert_times("sunrise, sunset, transit", &a.out, &b.out, ROOT_TOL);
    assert_saves("sunrise, sunset, transit", &a, &b, 8);
}

#[test]
fn lunar_returns_of_a_year() {
    let start = jd(2026, 1, 1, 0.0);
    let natal = 123.456;
    let scan = |f: &(dyn Fn(f64) -> Option<f64> + Sync)| {
        let mut out = Vec::new();
        let mut seed = start;
        for _ in 0..12 {
            let Some(t) = lunar_return(natal, seed, f) else {
                break;
            };
            out.push(t);
            seed = t + 20.0;
        }
        out
    };
    let a = direct(|p| {
        scan(&|t| {
            ecliptic_position(p, Body::Moon, t)
                .ok()
                .map(|c| c.longitude.to_degrees())
        })
    });
    let b = walked(start - 1.0, start + 400.0, |sky| {
        scan(&|t| sky.moon_longitude(t))
    });
    assert_eq!(a.out.len(), 12);
    // `find_return` bisects to 1e-10 day.
    assert_times("lunar returns", &a.out, &b.out, ROOT_TOL);
    assert_saves("lunar returns", &a, &b, 3);
}

// ── transits and the muhurta search ─────────────────────────────

#[test]
fn the_transits_of_the_moon_and_the_sun_over_a_year() {
    let start = jd(2026, 1, 1, 0.0);
    let natal: Vec<(String, f64)> = [12.3, 47.9, 101.5, 133.2, 187.7, 222.4, 268.8, 311.6]
        .iter()
        .enumerate()
        .map(|(i, &l)| (format!("N{i}"), l))
        .collect();
    let config = TransitSearchConfig {
        natal_positions: natal,
        start_jd: start,
        end_jd: start + 365.0,
        transiting_bodies: vec![("Moon".into(), 0), ("Sun".into(), 1)],
        aspect_types: [
            ("conj", 0.0),
            ("sext", 60.0),
            ("sq", 90.0),
            ("tri", 120.0),
            ("opp", 180.0),
        ]
        .iter()
        .map(|&(n, a)| (n.to_owned(), a))
        .collect(),
        max_orb: 1.0,
        step_size: 1.0,
    };
    let sorted = |mut v: Vec<TransitEvent>| {
        v.sort_by(|p, q| {
            (&p.transiting_body, &p.natal_body, &p.aspect_type)
                .cmp(&(&q.transiting_body, &q.natal_body, &q.aspect_type))
                .then(p.exact_jd.total_cmp(&q.exact_jd))
        });
        v
    };
    let a = direct(|p| {
        let f = |i: usize, t: f64| {
            let body = [Body::Moon, Body::Sun].get(i).copied()?;
            ecliptic_position(p, body, t)
                .ok()
                .map(|c| c.longitude.to_degrees())
        };
        sorted(search_transits(&config, &f))
    });
    let b = walked(start - 1.0, start + 367.0, |sky| {
        let f = |i: usize, t: f64| match i {
            0 => sky.moon_longitude(t),
            1 => sky.sun_longitude(t),
            _ => None,
        };
        sorted(search_transits(&config, &f))
    });
    for (x, y) in a.out.iter().zip(&b.out) {
        assert_eq!(
            (
                &x.transiting_body,
                &x.natal_body,
                &x.aspect_type,
                x.applying
            ),
            (
                &y.transiting_body,
                &y.natal_body,
                &y.aspect_type,
                y.applying
            )
        );
    }
    let times = |v: &[TransitEvent]| v.iter().map(|e| e.exact_jd).collect::<Vec<_>>();
    // The search bisects to 1e-8 day, 0.86 ms: a walked longitude that tips one
    // bisection step moves the answer by one cell.
    assert_times("transits", &times(&a.out), &times(&b.out), 1.0e-8);
    assert_saves("transits", &a, &b, 5);
}

#[test]
fn the_muhurta_search_of_thirty_days_is_the_same_search() {
    let start = jd(2026, 3, 1, 0.0);
    let end = start + 30.0;
    let a = direct(|p| -> Vec<MuhurtaAssessment> {
        let moon = |t: f64| {
            ecliptic_position(p, Body::Moon, t)
                .ok()
                .map(|c| sidereal(c.longitude.to_degrees(), t))
        };
        let sun = |t: f64| {
            ecliptic_position(p, Body::Sun, t)
                .ok()
                .map(|c| sidereal(c.longitude.to_degrees(), t))
        };
        let eq = |t: f64| sun_equatorial_deg(p, t);
        search_muhurta(start, end, LAT, LON, 0.0, 330, &moon, &sun, &eq, 0.0)
    });
    let b = walked(start - 1.0, end + 2.0, |sky| -> Vec<MuhurtaAssessment> {
        let moon = |t: f64| sky.moon_sidereal_longitude(AYAN, t);
        let sun = |t: f64| sky.sun_sidereal_longitude(AYAN, t);
        let eq = |t: f64| sky.sun_equatorial(t);
        search_muhurta(start, end, LAT, LON, 0.0, 330, &moon, &sun, &eq, 0.0)
    });
    assert_eq!(a.out.len(), 61);
    assert_eq!(a.out.len(), b.out.len());
    for (x, y) in a.out.iter().zip(&b.out) {
        assert_eq!(x.jd.to_bits(), y.jd.to_bits());
        assert_eq!(x.tithi.number, y.tithi.number, "tithi at {}", x.jd);
        assert_eq!(x.nakshatra, y.nakshatra, "nakshatra at {}", x.jd);
        assert_eq!(x.weekday, y.weekday, "weekday at {}", x.jd);
        assert_eq!(x.factors, y.factors, "factors at {}", x.jd);
        assert!(
            (x.quality_score - y.quality_score).abs() < 1.0e-12,
            "quality at {}",
            x.jd
        );
    }
    assert_saves("muhurta search", &a, &b, 5);
}
