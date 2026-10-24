// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Survey: what `vedaksha_ephem_core::walk::BodyWalk` would save the long
//! scans of this crate and of `vedaksha-astro`, and what it would do to their
//! answers.
//!
//! # What it measures
//!
//! Reads (invocations of the callback), provider calls (`compute_state`,
//! `earth_state`, `moon_position`, `moon_distance`, counted by a wrapping
//! provider) and the answers, for each scan twice: the callback the way a
//! consumer builds it today (`apparent_position`, `apparent_positions`,
//! `ecliptic_position`) and the same callback served from a walk. Wall time is
//! not measured: this machine is shared.
//!
//! The scans are ignored by default (a minute or two on an idle machine):
//!
//! ```text
//! nice -n 10 cargo test -p vedaksha-vedic --test walk_survey -- --ignored --nocapture
//! ```

mod common;

use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use common::jd;
use vedaksha_astro::moon_riseset::{moon_equatorial, moon_rise_set};
use vedaksha_astro::riseset::{RiseSet, sun_equatorial_deg, sun_rise_set};
use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value, true_tropical_to_sidereal};
use vedaksha_astro::transits::{TransitEvent, TransitSearchConfig, lunar_return, search_transits};
use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    apparent_positions, ecliptic_position, ecliptic_to_equatorial_deg, frame_for,
};
use vedaksha_ephem_core::delta_t::ut1_to_tt;
use vedaksha_ephem_core::error::ComputeError;
use vedaksha_ephem_core::jpl::{AU_KM, EphemerisProvider, Position, StateVector};
use vedaksha_ephem_core::obliquity::mean_obliquity;
use vedaksha_vedic::graha::Graha;
use vedaksha_vedic::kala_mana::{MasaScheme, MasaSpan, lunar_month_span, year_turnover};
use vedaksha_vedic::khagola::{
    Grahana, HeliacalCriterion, HeliacalSide, LunarGeometry, Place, SolarGeometry, graha_ingress,
    graha_ingresses, grahana, next_udaya, stations,
};
use vedaksha_vedic::muhurta::{
    MoonAndSun, MuhurtaAssessment, compute_karana_end, compute_nakshatra_end, compute_tithi_end,
    compute_yoga_end, search_muhurta,
};

use vedaksha_ephem_core::walk::{BodyWalk, WalkSpec};

/// Nodes per segment of every walk the survey builds.
const NODES: usize = 12;
const AYAN: Ayanamsha = Ayanamsha::IndianOfficial;
/// New Delhi, the observer the other khagola tests use.
const LAT: f64 = 28.6139;
const LON: f64 = 77.2090;

// ── a counting provider ─────────────────────────────────────────

/// `AnalyticalProvider`, counting every call that reaches it.
struct Counting {
    state: AtomicU64,
    earth: AtomicU64,
    moon_position: AtomicU64,
    moon_distance: AtomicU64,
}

impl Counting {
    const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            earth: AtomicU64::new(0),
            moon_position: AtomicU64::new(0),
            moon_distance: AtomicU64::new(0),
        }
    }
}

impl EphemerisProvider for Counting {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        self.state.fetch_add(1, Relaxed);
        AnalyticalProvider.compute_state(body, jd)
    }
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        self.earth.fetch_add(1, Relaxed);
        AnalyticalProvider.earth_state(jd)
    }
    fn moon_position(&self, jd: f64) -> Result<Position, ComputeError> {
        self.moon_position.fetch_add(1, Relaxed);
        AnalyticalProvider.moon_position(jd)
    }
    fn moon_distance(&self, jd: f64) -> Option<f64> {
        self.moon_distance.fetch_add(1, Relaxed);
        AnalyticalProvider.moon_distance(jd)
    }
    fn time_range(&self) -> (f64, f64) {
        AnalyticalProvider.time_range()
    }
}

static PROVIDER: Counting = Counting::new();

#[derive(Clone, Copy, Default, Debug)]
struct Calls {
    state: u64,
    earth: u64,
    moon_position: u64,
    moon_distance: u64,
}

impl Calls {
    fn now() -> Self {
        Self {
            state: PROVIDER.state.load(Relaxed),
            earth: PROVIDER.earth.load(Relaxed),
            moon_position: PROVIDER.moon_position.load(Relaxed),
            moon_distance: PROVIDER.moon_distance.load(Relaxed),
        }
    }
    fn since(self, earlier: Self) -> Self {
        Self {
            state: self.state - earlier.state,
            earth: self.earth - earlier.earth,
            moon_position: self.moon_position - earlier.moon_position,
            moon_distance: self.moon_distance - earlier.moon_distance,
        }
    }
    fn total(self) -> u64 {
        self.state + self.earth + self.moon_position + self.moon_distance
    }
}

// ── the sources a scan reads ────────────────────────────────────

/// Walks for the bodies a scan reads, over one span.
struct Walked {
    moon: BodyWalk<'static>,
    sun: BodyWalk<'static>,
    planets: Vec<(Body, BodyWalk<'static>)>,
    fallbacks: AtomicU64,
}

impl Walked {
    fn new(start: f64, end: f64, planets: &[Body]) -> Self {
        let slow = WalkSpec {
            days: 8.0,
            nodes: NODES,
        };
        Self {
            moon: BodyWalk::moon(&PROVIDER, start, end),
            sun: BodyWalk::with_spec(&PROVIDER, Body::Sun, start, end, slow),
            planets: planets
                .iter()
                .map(|&b| (b, BodyWalk::with_spec(&PROVIDER, b, start, end, slow)))
                .collect(),
            fallbacks: AtomicU64::new(0),
        }
    }

    fn walk(&self, body: Body) -> Option<&BodyWalk<'static>> {
        match body {
            Body::Moon => Some(&self.moon),
            Body::Sun => Some(&self.sun),
            b => self.planets.iter().find(|(p, _)| *p == b).map(|(_, w)| w),
        }
    }

    fn fell_back(&self) {
        self.fallbacks.fetch_add(1, Relaxed);
    }

    fn lon_rate(&self, body: Body, t: f64) -> Option<(f64, f64)> {
        if let Some(w) = self.walk(body) {
            if let (Some(c), Some(s)) = (w.ecliptic(t), w.longitude_speed(t)) {
                return Some((c.longitude.to_degrees(), s));
            }
        }
        self.fell_back();
        common::longitude_and_rate(&PROVIDER, body, t)
    }

    fn lon(&self, body: Body, t: f64) -> Option<f64> {
        if let Some(c) = self.walk(body).and_then(|w| w.ecliptic(t)) {
            return Some(c.longitude.to_degrees());
        }
        self.fell_back();
        ecliptic_position(&PROVIDER, body, t)
            .ok()
            .map(|c| c.longitude.to_degrees())
    }

    fn segments(&self) -> (usize, usize) {
        let mut segs = self.moon.segments_fitted() + self.sun.segments_fitted();
        let mut fitted_nodes =
            self.moon.segments_fitted() * NODES + self.sun.segments_fitted() * NODES;
        for (_, w) in &self.planets {
            segs += w.segments_fitted();
            fitted_nodes += w.segments_fitted() * NODES;
        }
        (segs, fitted_nodes)
    }
}

/// Where a scan's positions come from: the engine's direct calls, as a
/// consumer builds callbacks today, or a walk.
struct Src {
    walked: Option<Walked>,
    reads: AtomicU64,
}

impl Src {
    fn direct() -> Self {
        Self {
            walked: None,
            reads: AtomicU64::new(0),
        }
    }

    fn walk(start: f64, end: f64, planets: &[Body]) -> Self {
        Self {
            walked: Some(Walked::new(start, end, planets)),
            reads: AtomicU64::new(0),
        }
    }

    fn tick(&self) {
        self.reads.fetch_add(1, Relaxed);
    }

    /// Tropical longitude and daily motion, degrees: `apparent_position`'s.
    fn lon_rate(&self, body: Body, t: f64) -> Option<(f64, f64)> {
        self.tick();
        match &self.walked {
            None => common::longitude_and_rate(&PROVIDER, body, t),
            Some(w) => w.lon_rate(body, t),
        }
    }

    /// Tropical longitude only, degrees: `ecliptic_position`'s.
    fn lon(&self, body: Body, t: f64) -> Option<f64> {
        self.tick();
        match &self.walked {
            None => ecliptic_position(&PROVIDER, body, t)
                .ok()
                .map(|c| c.longitude.to_degrees()),
            Some(w) => w.lon(body, t),
        }
    }

    /// Moon and Sun together, as the MCP server builds it.
    fn pair(&self, t: f64) -> Option<MoonAndSun> {
        self.tick();
        match &self.walked {
            None => {
                let mut it = apparent_positions(&PROVIDER, &[Body::Moon, Body::Sun], t).into_iter();
                let moon = it.next()?.1.ok()?;
                let sun = it.next()?.1.ok()?;
                Some((
                    (moon.ecliptic.longitude.to_degrees(), moon.longitude_speed),
                    (sun.ecliptic.longitude.to_degrees(), sun.longitude_speed),
                ))
            }
            Some(w) => Some((w.lon_rate(Body::Moon, t)?, w.lon_rate(Body::Sun, t)?)),
        }
    }

    /// The Sun's and Moon's apparent ecliptic geometry, as `common` builds it.
    fn lunar(&self, t: f64) -> Option<LunarGeometry> {
        self.tick();
        match &self.walked {
            None => common::lunar_geometry(&PROVIDER, t),
            Some(w) => {
                let (Some(sun), Some(moon)) = (w.sun.ecliptic(t), w.moon.ecliptic(t)) else {
                    w.fell_back();
                    return common::lunar_geometry(&PROVIDER, t);
                };
                Some(LunarGeometry {
                    sun_lon_deg: sun.longitude.to_degrees(),
                    sun_dist_au: sun.distance,
                    moon_lon_deg: moon.longitude.to_degrees(),
                    moon_lat_deg: moon.latitude.to_degrees(),
                    moon_dist_au: moon.distance,
                })
            }
        }
    }

    /// The Sun's and Moon's apparent equatorial geometry, as `common` builds it.
    fn solar(&self, t: f64) -> Option<SolarGeometry> {
        self.tick();
        match &self.walked {
            None => common::solar_geometry(&PROVIDER, t),
            Some(w) => {
                let (Some(sun), Some(moon)) = (w.sun.ecliptic(t), w.moon.ecliptic(t)) else {
                    w.fell_back();
                    return common::solar_geometry(&PROVIDER, t);
                };
                let eps = frame_for(ut1_to_tt(t)).true_obliquity().to_degrees();
                let eq = |c: &vedaksha_ephem_core::coordinates::EclipticCoords| {
                    ecliptic_to_equatorial_deg(
                        c.longitude.to_degrees(),
                        c.latitude.to_degrees(),
                        eps,
                    )
                };
                let (sun_ra, sun_dec) = eq(&sun);
                let (moon_ra, moon_dec) = eq(&moon);
                Some(SolarGeometry {
                    sun_ra_deg: sun_ra,
                    sun_dec_deg: sun_dec,
                    sun_dist_au: sun.distance,
                    moon_ra_deg: moon_ra,
                    moon_dec_deg: moon_dec,
                    moon_dist_au: moon.distance,
                })
            }
        }
    }

    /// The Sun's apparent `(ra, dec)` by the mean obliquity:
    /// `vedaksha_astro::riseset::sun_equatorial_deg`'s.
    fn sun_eq_mean(&self, t: f64) -> Option<(f64, f64)> {
        self.tick();
        match &self.walked {
            None => sun_equatorial_deg(&PROVIDER, t),
            Some(w) => {
                let Some(c) = w.sun.ecliptic(t) else {
                    w.fell_back();
                    return sun_equatorial_deg(&PROVIDER, t);
                };
                let eps = mean_obliquity(ut1_to_tt(t)).to_degrees();
                Some(ecliptic_to_equatorial_deg(
                    c.longitude.to_degrees(),
                    c.latitude.to_degrees(),
                    eps,
                ))
            }
        }
    }

    /// The Moon's `(ra, dec, distance_km)`: `moon_equatorial`'s.
    fn moon_eq(&self, t: f64) -> Option<(f64, f64, f64)> {
        self.tick();
        match &self.walked {
            None => moon_equatorial(&PROVIDER, t),
            Some(w) => {
                let Some(c) = w.moon.ecliptic(t) else {
                    w.fell_back();
                    return moon_equatorial(&PROVIDER, t);
                };
                let eps = frame_for(ut1_to_tt(t)).true_obliquity().to_degrees();
                let (ra, dec) = ecliptic_to_equatorial_deg(
                    c.longitude.to_degrees(),
                    c.latitude.to_degrees(),
                    eps,
                );
                Some((ra, dec, c.distance * AU_KM))
            }
        }
    }

    /// Apparent `(ra, dec)` by the true obliquity: `common::equatorial`'s. Only
    /// the Sun is served from the walk; the planets' walks are not accurate
    /// enough to use (see the sweep), so they stay direct.
    fn eq_true(&self, body: Body, t: f64) -> Option<(f64, f64)> {
        self.tick();
        match (&self.walked, body) {
            (Some(w), Body::Sun) => {
                let Some(c) = w.sun.ecliptic(t) else {
                    w.fell_back();
                    return common::equatorial(&PROVIDER, body, t);
                };
                let eps = frame_for(ut1_to_tt(t)).true_obliquity().to_degrees();
                Some(ecliptic_to_equatorial_deg(
                    c.longitude.to_degrees(),
                    c.latitude.to_degrees(),
                    eps,
                ))
            }
            _ => common::equatorial(&PROVIDER, body, t),
        }
    }

    /// Tropical Moon longitude to sidereal, as the MCP server does.
    fn sidereal(lon: f64, t: f64) -> f64 {
        true_tropical_to_sidereal(lon, AYAN, t)
    }
}

// ── comparison and reporting ────────────────────────────────────

/// Outcome of comparing two lists of event times (JD).
#[derive(Default, Debug)]
struct TimeDiff {
    direct: usize,
    walk: usize,
    /// Events in one list with none within half a day in the other.
    unmatched: usize,
    worst_s: f64,
    mean_s: f64,
    over_1ms: usize,
}

fn compare_times(a: &[f64], b: &[f64]) -> TimeDiff {
    let mut d = TimeDiff {
        direct: a.len(),
        walk: b.len(),
        ..TimeDiff::default()
    };
    let mut used = vec![false; b.len()];
    let mut total = 0.0;
    let mut matched = 0_usize;
    for &x in a {
        let near = b
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(i, &y)| (i, (x - y).abs()))
            .min_by(|p, q| p.1.total_cmp(&q.1));
        match near {
            Some((i, dt)) if dt < 0.5 => {
                used[i] = true;
                let s = dt * 86_400.0;
                d.worst_s = d.worst_s.max(s);
                total += s;
                matched += 1;
                if s > 1.0e-3 {
                    d.over_1ms += 1;
                }
            }
            _ => d.unmatched += 1,
        }
    }
    d.unmatched += used.iter().filter(|u| !**u).count();
    d.mean_s = if matched == 0 {
        0.0
    } else {
        total / matched as f64
    };
    d
}

struct Row {
    name: String,
    span_days: f64,
    reads_direct: u64,
    reads_walk: u64,
    calls_direct: Calls,
    calls_walk: Calls,
    segments: usize,
    fitted_nodes: usize,
    fallbacks: u64,
    diff: TimeDiff,
    note: String,
}

impl Row {
    fn print(&self) {
        let d = self.calls_direct.total();
        let w = self.calls_walk.total();
        println!("--- {}", self.name);
        println!(
            "    span {:.0} d | reads direct {} ({:.1}/d), walk {} ({:.1}/d) | provider calls direct {} ({:.1}/d), walk {} ({:.1}/d), ratio {:.1}x",
            self.span_days,
            self.reads_direct,
            self.reads_direct as f64 / self.span_days,
            self.reads_walk,
            self.reads_walk as f64 / self.span_days,
            d,
            d as f64 / self.span_days,
            w,
            w as f64 / self.span_days,
            d as f64 / w.max(1) as f64,
        );
        println!(
            "    walk: {} segments, {} fitted nodes, {} reads fell back to direct | direct calls {:?}, walk calls {:?}",
            self.segments, self.fitted_nodes, self.fallbacks, self.calls_direct, self.calls_walk
        );
        println!(
            "    answers: {} direct, {} walk, {} unmatched | dt worst {:.3e} s, mean {:.3e} s, {} over 1 ms | {}",
            self.diff.direct,
            self.diff.walk,
            self.diff.unmatched,
            self.diff.worst_s,
            self.diff.mean_s,
            self.diff.over_1ms,
            self.note
        );
    }
}

/// Run `scan` on a direct source and on a walk over `[start, end]`, and compare
/// the event times it returns.
fn survey(
    name: &str,
    start: f64,
    end: f64,
    planets: &[Body],
    scan: impl Fn(&Src) -> Vec<f64>,
    note: &str,
) -> Row {
    survey_raw(name, start, end, planets, scan, note).0
}

/// [`survey`], with the two lists of times as well.
fn survey_raw(
    name: &str,
    start: f64,
    end: f64,
    planets: &[Body],
    scan: impl Fn(&Src) -> Vec<f64>,
    note: &str,
) -> (Row, Vec<f64>, Vec<f64>) {
    let direct = Src::direct();
    let before = Calls::now();
    let a = scan(&direct);
    let calls_direct = Calls::now().since(before);
    dump(name, &a);

    let walk = Src::walk(start, end, planets);
    let before = Calls::now();
    let b = scan(&walk);
    let calls_walk = Calls::now().since(before);
    let w = walk.walked.as_ref().expect("a walk source");
    let (segments, fitted_nodes) = w.segments();
    let row = Row {
        name: name.to_owned(),
        span_days: end - start,
        reads_direct: direct.reads.load(Relaxed),
        reads_walk: walk.reads.load(Relaxed),
        calls_direct,
        calls_walk,
        segments,
        fitted_nodes,
        fallbacks: w.fallbacks.load(Relaxed),
        diff: compare_times(&a, &b),
        note: note.to_owned(),
    };
    row.print();
    (row, a, b)
}

/// Append a scan's direct-path answers to the file named by `WALK_SURVEY_DUMP`,
/// one line per scan, so two builds of the library can be compared answer by
/// answer (see `walk_survey_compare`).
fn dump(name: &str, values: &[f64]) {
    use std::io::Write as _;
    let Ok(path) = std::env::var("WALK_SURVEY_DUMP") else {
        return;
    };
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("dump file");
    let bits: Vec<String> = values
        .iter()
        .map(|v| format!("{:x}", v.to_bits()))
        .collect();
    writeln!(file, "{name}\t{}", bits.join(" ")).expect("write");
}

fn flat(options: impl IntoIterator<Item = Option<f64>>) -> Vec<f64> {
    options.into_iter().flatten().collect()
}

// ── helpers ─────────────────────────────────────────────────────

fn instants(seed: u64, count: usize, lo: f64, hi: f64) -> Vec<f64> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            lo + (state >> 11) as f64 / (1_u64 << 53) as f64 * (hi - lo)
        })
        .collect()
}

// ── the scans ───────────────────────────────────────────────────

/// How far a walk of the Sun and the planets (8-day segments, 12 nodes) and of
/// the Moon (the default 4-day segments) stands from the direct apparent place,
/// in arcseconds of longitude, over 2000 to 2050.
fn interpolation_error() {
    let (lo, hi) = (jd(2000, 1, 1, 0.0), jd(2050, 1, 1, 0.0));
    let src = Walked::new(
        lo,
        hi,
        &[
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
        ],
    );
    println!("--- walk interpolation error against ecliptic_position, 400 instants 2000 to 2050");
    for body in [
        Body::Moon,
        Body::Sun,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ] {
        let w = src.walk(body).expect("walk");
        let mut worst = 0.0_f64;
        for t in instants(0xE220 ^ body as u64, 400, lo, hi) {
            let want = ecliptic_position(&PROVIDER, body, t).unwrap().longitude;
            let got = w.ecliptic(t).unwrap().longitude;
            let mut d = (want - got).rem_euclid(std::f64::consts::TAU);
            if d > std::f64::consts::PI {
                d -= std::f64::consts::TAU;
            }
            worst = worst.max(d.abs() * 206_264.806_247_096_36);
        }
        println!("    {body:?}: worst {worst:.2e} arcsec");
    }
}

/// How far a walk of a planet stands from the direct apparent place, by segment
/// size. `walk.rs` states a bound for the Moon at [`WalkSpec::MOON`] only.
///
/// The large entries do not shrink with shorter segments or more nodes, which an
/// interpolation error would. They are a step in the apparent place itself:
/// `deflection::solar_deflection` returns zero when the straight ray crosses the
/// Sun's disc, and for Jupiter at JD(TT) 2454457.49435 `ecliptic_position_tt`
/// drops by 1.089 arcsec between two samples 1e-7 d apart, exactly where its
/// elongation crosses 0.2709 degree (the Sun's limb at 0.9836 AU). Any segment
/// that holds such a crossing is off by the step. The Moon and the Sun are not
/// deflected, so their walks have none. (Verified for that one instant; the
/// other bodies' large entries are consistent with the same cause.)
fn planet_spec_sweep() {
    let (lo, hi) = (jd(2000, 1, 1, 0.0), jd(2050, 1, 1, 0.0));
    println!("--- walk error in longitude by segment size, 300 instants 2000 to 2050, arcsec");
    for body in [
        Body::Sun,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
    ] {
        let mut line = format!("    {body:?}:");
        for (days, nodes) in [(8.0, 12), (4.0, 12), (2.0, 12), (4.0, 16), (2.0, 16)] {
            let w = BodyWalk::with_spec(&PROVIDER, body, lo, hi, WalkSpec { days, nodes });
            let mut worst = 0.0_f64;
            for t in instants(0xAB5 ^ body as u64, 300, lo, hi) {
                let want = ecliptic_position(&PROVIDER, body, t).unwrap().longitude;
                let got = w.ecliptic(t).unwrap().longitude;
                let mut d = (want - got).rem_euclid(std::f64::consts::TAU);
                if d > std::f64::consts::PI {
                    d -= std::f64::consts::TAU;
                }
                worst = worst.max(d.abs() * 206_264.806_247_096_36);
            }
            let _ = write!(line, "  {days}d/{nodes}n {worst:.1e}");
        }
        println!("{line}");
    }
}

fn moon_sidereal_of(src: &Src) -> impl Fn(f64) -> Option<(f64, f64)> + Sync + '_ {
    move |t: f64| {
        let (l, r) = src.lon_rate(Body::Moon, t)?;
        Some((Src::sidereal(l, t), r))
    }
}

fn pair_of(src: &Src) -> impl Fn(f64) -> Option<MoonAndSun> + Sync + '_ {
    move |t: f64| src.pair(t)
}

/// Newton to convergence (a step under half a JD quantum, 2.3e-10 d), where
/// `refine_crossing` stops within 1e-6 degree.
fn tight(
    target: f64,
    mut t: f64,
    angle_at: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<f64> {
    for _ in 0..12 {
        let (a, r) = angle_at(t)?;
        let mut f = (target - a).rem_euclid(360.0);
        if f > 180.0 {
            f -= 360.0;
        }
        let step = f / r;
        t += step;
        if step.abs() < 2.3e-10 {
            break;
        }
    }
    Some(t)
}

/// The walk moves the root of a Moon crossing by how much, over a century, and
/// how much of what `refine_crossing` returns is its own stopping band?
///
/// For each sample day, the nakshatra end (the Moon's sidereal longitude
/// reaching a multiple of 360/27) and every seventh day the tithi end are found
/// three ways: by `refine_crossing` on the direct callback, by `refine_crossing`
/// on the walk, and by Newton run to convergence on each callback. The converged
/// pair differs by the walk's own effect on the root; the `refine_crossing`
/// answers differ from their converged roots by the 1e-6 degree stopping band.
fn century_root_shift() {
    let (a, b) = (jd(1950, 1, 1, 0.0), jd(2050, 1, 1, 0.0));
    let walk = Src::walk(a - 2.0, b + 4.0, &[]);
    let direct = Src::direct();
    let (dsid, wsid) = (moon_sidereal_of(&direct), moon_sidereal_of(&walk));
    let span = 360.0 / 27.0;

    let mut samples = 0_u32;
    let (mut root_worst, mut root_sum) = (0.0_f64, 0.0_f64);
    let (mut walk_band, mut direct_band) = (0.0_f64, 0.0_f64);
    let (mut returned_worst, mut returned_over_1ms) = (0.0_f64, 0_u32);
    let mut day = a;
    while day < b {
        let (Some(tw), Some(td)) = (
            compute_nakshatra_end(day, &wsid),
            compute_nakshatra_end(day, &dsid),
        ) else {
            panic!("nakshatra end at {day}");
        };
        let (m, _) = wsid(tw).unwrap();
        let target = ((m / span).round() * span).rem_euclid(360.0);
        let rd = tight(target, td, &dsid).unwrap();
        let rw = tight(target, tw, &wsid).unwrap();
        root_worst = root_worst.max((rd - rw).abs());
        root_sum += (rd - rw).abs();
        walk_band = walk_band.max((tw - rw).abs());
        direct_band = direct_band.max((td - rd).abs());
        returned_worst = returned_worst.max((td - tw).abs());
        returned_over_1ms += u32::from((td - tw).abs() * 86_400.0e3 > 1.0);
        samples += 1;
        day += 3.0;
    }
    let ms = 86_400.0e3;
    println!(
        "--- Moon, nakshatra ends, every 3rd day 1950 to 2050 ({samples} samples): converged roots differ by worst {:.3e} ms, mean {:.3e} ms (one JD quantum is 4.0e-2 ms)",
        root_worst * ms,
        root_sum * ms / f64::from(samples)
    );
    println!(
        "    refine_crossing's answer sits from its own converged root by worst {:.3e} ms (walk), {:.3e} ms (direct); the two answers differ by worst {:.3e} ms, {returned_over_1ms} of {samples} over 1 ms",
        walk_band * ms,
        direct_band * ms,
        returned_worst * ms
    );

    // Tithi ends: the elongation reaching a multiple of 12 degrees.
    let (dpair, wpair) = (pair_of(&direct), pair_of(&walk));
    let angle = |p: &(dyn Fn(f64) -> Option<MoonAndSun> + Sync), t: f64| {
        let ((ml, ms_), (sl, ss)) = p(t)?;
        Some(((ml - sl).rem_euclid(360.0), ms_ - ss))
    };
    let (dang, wang) = (|t: f64| angle(&dpair, t), |t: f64| angle(&wpair, t));
    let mut samples = 0_u32;
    let (mut root_worst, mut root_sum) = (0.0_f64, 0.0_f64);
    let (mut walk_band, mut direct_band) = (0.0_f64, 0.0_f64);
    let (mut returned_worst, mut returned_over_1ms) = (0.0_f64, 0_u32);
    let mut day = a;
    while day < b {
        let (Some(tw), Some(td)) = (
            compute_tithi_end(day, &wpair),
            compute_tithi_end(day, &dpair),
        ) else {
            panic!("tithi end at {day}");
        };
        let (e, _) = wang(tw).unwrap();
        let target = ((e / 12.0).round() * 12.0).rem_euclid(360.0);
        let rd = tight(target, td, &dang).unwrap();
        let rw = tight(target, tw, &wang).unwrap();
        root_worst = root_worst.max((rd - rw).abs());
        root_sum += (rd - rw).abs();
        walk_band = walk_band.max((tw - rw).abs());
        direct_band = direct_band.max((td - rd).abs());
        returned_worst = returned_worst.max((td - tw).abs());
        returned_over_1ms += u32::from((td - tw).abs() * ms > 1.0);
        samples += 1;
        day += 7.0;
    }
    println!(
        "--- Moon and Sun, tithi ends, every 7th day 1950 to 2050 ({samples} samples): converged roots differ by worst {:.3e} ms, mean {:.3e} ms",
        root_worst * ms,
        root_sum * ms / f64::from(samples)
    );
    println!(
        "    refine_crossing's answer sits from its own converged root by worst {:.3e} ms (walk), {:.3e} ms (direct); the two answers differ by worst {:.3e} ms, {returned_over_1ms} of {samples} over 1 ms",
        walk_band * ms,
        direct_band * ms,
        returned_worst * ms
    );
}

#[test]
#[ignore = "a survey: a minute or two, prints a table"]
#[allow(clippy::too_many_lines)]
fn walk_survey() {
    interpolation_error();
    planet_spec_sweep();
    century_root_shift();

    // ── panchang range: the four limbs' ends, every day of a year ──
    let y0 = jd(2026, 1, 1, 0.0);
    let days: Vec<f64> = (0..366).map(|d| y0 + f64::from(d)).collect();
    survey(
        "panchang range: tithi, nakshatra, karana, yoga ends, each day for 366 days",
        y0 - 2.0,
        y0 + 368.0,
        &[],
        |src| {
            let pair = |t: f64| src.pair(t);
            let pair_sidereal = |t: f64| {
                let ((ml, ms), (sl, ss)) = src.pair(t)?;
                let aya = true_ayanamsha_value(AYAN, t);
                Some((
                    ((ml - aya).rem_euclid(360.0), ms),
                    ((sl - aya).rem_euclid(360.0), ss),
                ))
            };
            let moon_sidereal = |t: f64| {
                let (l, r) = src.lon_rate(Body::Moon, t)?;
                Some((Src::sidereal(l, t), r))
            };
            let mut out = Vec::new();
            for &d in &days {
                out.push(compute_tithi_end(d, &pair));
                out.push(compute_nakshatra_end(d, &moon_sidereal));
                out.push(compute_karana_end(d, &pair));
                out.push(compute_yoga_end(d, &pair_sidereal));
            }
            flat(out)
        },
        "event = one limb end; refine_crossing stops within 1e-6 deg",
    );

    // ── lunar month spans: a calendar's masa lookups ──
    let spans: Vec<f64> = (0..92).map(|k| y0 + 4.0 * f64::from(k)).collect();
    {
        let masa_key = |s: &MasaSpan| (s.masa.index, s.masa.adhika, s.masa.kshaya);
        let direct = Src::direct();
        let before = Calls::now();
        let a: Vec<Option<MasaSpan>> = spans
            .iter()
            .map(|&d| lunar_month_span(d, MasaScheme::Amanta, AYAN, &|t: f64| direct.pair(t)))
            .collect();
        let calls_direct = Calls::now().since(before);
        let walk = Src::walk(y0 - 80.0, y0 + 430.0, &[]);
        let before = Calls::now();
        let b: Vec<Option<MasaSpan>> = spans
            .iter()
            .map(|&d| lunar_month_span(d, MasaScheme::Amanta, AYAN, &|t: f64| walk.pair(t)))
            .collect();
        let calls_walk = Calls::now().since(before);
        let times = |v: &[Option<MasaSpan>]| -> Vec<f64> {
            v.iter()
                .flatten()
                .flat_map(|s| [s.start_jd, s.end_jd])
                .collect()
        };
        dump(
            "lunar_month_span, Amanta, every 4th day for a year",
            &times(&a),
        );
        // Spans repeat across the calls that fall in one lunation; compare per call.
        let mut worst = 0.0_f64;
        let mut masa_differs = 0;
        for (x, y) in a.iter().zip(&b) {
            match (x, y) {
                (Some(x), Some(y)) => {
                    worst = worst
                        .max((x.start_jd - y.start_jd).abs())
                        .max((x.end_jd - y.end_jd).abs());
                    masa_differs += usize::from(masa_key(x) != masa_key(y));
                }
                _ => masa_differs += 1,
            }
        }
        let w = walk.walked.as_ref().unwrap();
        let (segments, fitted_nodes) = w.segments();
        let mut diff = compare_times(&times(&a), &times(&b));
        diff.worst_s = worst * 86_400.0;
        Row {
            name: "lunar_month_span, Amanta, every 4th day for a year (92 calls)".into(),
            span_days: 368.0,
            reads_direct: direct.reads.load(Relaxed),
            reads_walk: walk.reads.load(Relaxed),
            calls_direct,
            calls_walk,
            segments,
            fitted_nodes,
            fallbacks: w.fallbacks.load(Relaxed),
            diff,
            note: format!("masa differs in {masa_differs} calls; compared start and end bounds"),
        }
        .print();
    }

    // ── ingress ──
    let to_1y = y0 + 365.25;
    survey(
        "graha_ingresses, Moon, 1 year",
        y0 - 2.0,
        to_1y + 45.0,
        &[],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Moon, t);
            graha_ingresses(y0, to_1y, Graha::Moon, AYAN, &f)
                .into_iter()
                .map(|(t, _)| t)
                .collect()
        },
        "ingress times; rashi entered compared below",
    );
    survey(
        "graha_ingress chain (re-seeded a second past each), Moon, 1 year",
        y0 - 2.0,
        to_1y + 450.0,
        &[],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Moon, t);
            let mut out = Vec::new();
            let mut seed = y0;
            while let Some((t, _)) = graha_ingress(seed, Graha::Moon, AYAN, &f) {
                if t >= to_1y {
                    break;
                }
                out.push(t);
                seed = t + 1.0 / 86_400.0;
            }
            out
        },
        "",
    );
    let to_4y = y0 + 4.0 * 365.25;
    survey(
        "graha_ingresses, Sun, 4 years",
        y0 - 2.0,
        to_4y + 45.0,
        &[],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Sun, t);
            graha_ingresses(y0, to_4y, Graha::Sun, AYAN, &f)
                .into_iter()
                .map(|(t, _)| t)
                .collect()
        },
        "",
    );
    survey(
        "graha_ingresses, Mercury, 4 years",
        y0 - 2.0,
        to_4y + 45.0,
        &[Body::Mercury],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Mercury, t);
            graha_ingresses(y0, to_4y, Graha::Mercury, AYAN, &f)
                .into_iter()
                .map(|(t, _)| t)
                .collect()
        },
        "a planet: 8-day segments, no bound stated by walk.rs",
    );
    survey(
        "graha_ingresses, Mars, 4 years",
        y0 - 2.0,
        to_4y + 45.0,
        &[Body::Mars],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Mars, t);
            graha_ingresses(y0, to_4y, Graha::Mars, AYAN, &f)
                .into_iter()
                .map(|(t, _)| t)
                .collect()
        },
        "",
    );

    // ── stations ──
    let to_3y = y0 + 3.0 * 365.25;
    survey(
        "stations, Mercury, 3 years",
        y0 - 2.0,
        to_3y + 70.0,
        &[Body::Mercury],
        |src| {
            let f = |t: f64| src.lon_rate(Body::Mercury, t);
            stations(y0, to_3y, Graha::Mercury, &f)
        },
        "the rate differs in kind: central difference over +-0.5 d (direct) vs the derivative (walk)",
    );

    // ── eclipses: the chain of next eclipses over six years ──
    let to_6y = y0 + 6.0 * 365.25;
    let (_, a, b) = survey_raw(
        "grahana chain (next eclipse seen from New Delhi), 6 years",
        y0 - 2.0,
        to_6y + 300.0,
        &[],
        |src| {
            let lunar = |t: f64| src.lunar(t);
            let solar = |t: f64| src.solar(t);
            let mut out = Vec::new();
            let mut seed = y0;
            while seed < to_6y {
                let Some(g) = grahana(seed, LAT, LON, &lunar, &solar) else {
                    break;
                };
                out.extend(grahana_times(&g));
                seed = g.madhya_jd + 10.0;
            }
            out
        },
        "sparsha, madhya, moksha of each; madhya is a ternary-search minimum",
    );
    if a.len() == b.len() {
        for (i, label) in ["sparsha", "madhya", "moksha"].iter().enumerate() {
            let worst = a
                .iter()
                .zip(&b)
                .skip(i)
                .step_by(3)
                .map(|(x, y)| (x - y).abs() * 86_400.0)
                .fold(0.0_f64, f64::max);
            println!("    grahana {label}: worst dt {worst:.3e} s");
        }
    }

    // ── transits: a year, one Moon and the seven classical bodies ──
    transit_survey(y0);

    // ── Moon rise and set: sixty days ──
    {
        let start = jd(2026, 3, 1, 0.0);
        let days: Vec<f64> = (0..60).map(|d| start + f64::from(d)).collect();
        let collect = |v: Vec<RiseSet>| -> Vec<f64> {
            v.into_iter()
                .flat_map(|r| [r.rise, r.set, r.transit])
                .flatten()
                .collect()
        };
        survey(
            "moon_rise_set, 60 consecutive days, New Delhi",
            start - 1.0,
            start + 62.0,
            &[],
            |src| {
                let f = |t: f64| src.moon_eq(t);
                collect(
                    days.iter()
                        .map(|&d| moon_rise_set(d, LAT, LON, 0.0, &f))
                        .collect(),
                )
            },
            "rise, set, transit",
        );
    }

    // ── Sun rise and set: a year ──
    {
        let days: Vec<f64> = (0..366).map(|d| y0 + f64::from(d)).collect();
        survey(
            "sun_rise_set, 366 consecutive days, New Delhi",
            y0 - 1.0,
            y0 + 368.0,
            &[],
            |src| {
                let f = |t: f64| src.sun_eq_mean(t);
                days.iter()
                    .flat_map(|&d| {
                        let r = sun_rise_set(d, LAT, LON, 0.0, &f);
                        [r.rise, r.set, r.transit]
                    })
                    .flatten()
                    .collect()
            },
            "rise, set, transit",
        );
    }

    // ── muhurta search: the 30 days the MCP tool allows ──
    {
        let start = jd(2026, 3, 1, 0.0);
        let end = start + 30.0;
        let direct = Src::direct();
        let before = Calls::now();
        let a = run_muhurta(&direct, start, end);
        let calls_direct = Calls::now().since(before);
        let walk = Src::walk(start - 1.0, end + 2.0, &[]);
        let before = Calls::now();
        let b = run_muhurta(&walk, start, end);
        let calls_walk = Calls::now().since(before);
        let w = walk.walked.as_ref().unwrap();
        let (segments, fitted_nodes) = w.segments();
        let same_len = a.len() == b.len();
        let mut worst_q = 0.0_f64;
        let mut discrete = 0;
        for (x, y) in a.iter().zip(&b) {
            worst_q = worst_q.max((x.quality_score - y.quality_score).abs());
            discrete += usize::from(
                x.jd.to_bits() != y.jd.to_bits()
                    || x.tithi.number != y.tithi.number
                    || format!("{:?}", x.nakshatra) != format!("{:?}", y.nakshatra)
                    || x.weekday != y.weekday
                    || x.factors != y.factors,
            );
        }
        Row {
            name: "search_muhurta, 30 days (the MCP cap), min_quality 0".into(),
            span_days: 30.0,
            reads_direct: direct.reads.load(Relaxed),
            reads_walk: walk.reads.load(Relaxed),
            calls_direct,
            calls_walk,
            segments,
            fitted_nodes,
            fallbacks: w.fallbacks.load(Relaxed),
            diff: TimeDiff {
                direct: a.len(),
                walk: b.len(),
                ..TimeDiff::default()
            },
            note: format!(
                "same length {same_len}; {discrete} assessments differ in jd/tithi/nakshatra/weekday/factors; worst quality_score difference {worst_q:.3e}"
            ),
        }
        .print();
    }

    // ── year turnover: the Vikrama year's new moon, five years ──
    {
        let first = 2026;
        survey(
            "year_turnover, Amanta, five calendar years",
            jd(first, 1, 1, 0.0) - 80.0,
            jd(first + 5, 1, 1, 0.0) + 5.0,
            &[],
            |src| {
                let f = |t: f64| src.pair(t);
                flat((first..first + 5).map(|y| year_turnover(y, MasaScheme::Amanta, AYAN, &f)))
            },
            "",
        );
    }

    // ── heliacal rising: the Sun walked, the planet direct ──
    {
        let place = Place {
            latitude_deg: LAT,
            longitude_deg_east: LON,
        };
        survey(
            "next_udaya (kalamsha), Jupiter and Mars on the east, from 2026-01-01; Sun walked, planet direct",
            y0 - 2.0,
            y0 + 1100.0,
            &[],
            |src| {
                let sun = |t: f64| src.eq_true(Body::Sun, t);
                let jupiter = |t: f64| src.eq_true(Body::Jupiter, t);
                let mars = |t: f64| src.eq_true(Body::Mars, t);
                flat([
                    next_udaya(
                        y0,
                        Graha::Jupiter,
                        HeliacalSide::East,
                        place,
                        &sun,
                        &jupiter,
                        HeliacalCriterion::Kalamsha,
                    ),
                    next_udaya(
                        y0,
                        Graha::Mars,
                        HeliacalSide::East,
                        place,
                        &sun,
                        &mars,
                        HeliacalCriterion::Kalamsha,
                    ),
                ])
            },
            "",
        );
    }

    // ── lunar returns: thirteen in a row ──
    {
        let start = jd(2026, 1, 1, 0.0);
        let natal = 123.456;
        survey(
            "lunar_return, 13 consecutive months",
            start - 1.0,
            start + 13.0 * 28.0 + 40.0,
            &[],
            |src| {
                let f = |t: f64| src.lon(Body::Moon, t);
                let mut out = Vec::new();
                let mut seed = start;
                for _ in 0..13 {
                    let Some(t) = lunar_return(natal, seed, &f) else {
                        break;
                    };
                    out.push(t);
                    seed = t + 20.0;
                }
                out
            },
            "bisection to 1e-10 d",
        );
    }
}

fn grahana_times(g: &Grahana) -> [f64; 3] {
    [g.sparsha_jd, g.madhya_jd, g.moksha_jd]
}

fn run_muhurta(src: &Src, start: f64, end: f64) -> Vec<MuhurtaAssessment> {
    let moon = |t: f64| src.lon(Body::Moon, t).map(|l| Src::sidereal(l, t));
    let sun = |t: f64| src.lon(Body::Sun, t).map(|l| Src::sidereal(l, t));
    let sun_eq = |t: f64| src.sun_eq_mean(t);
    search_muhurta(start, end, LAT, LON, 0.0, 330, &moon, &sun, &sun_eq, 0.0)
}

fn transit_survey(y0: f64) {
    let bodies = [
        ("Moon", Body::Moon),
        ("Sun", Body::Sun),
        ("Mercury", Body::Mercury),
        ("Venus", Body::Venus),
        ("Mars", Body::Mars),
        ("Jupiter", Body::Jupiter),
        ("Saturn", Body::Saturn),
    ];
    let natal: Vec<(String, f64)> = [12.3, 47.9, 101.5, 133.2, 187.7, 222.4, 268.8, 311.6]
        .iter()
        .enumerate()
        .map(|(i, &l)| (format!("N{i}"), l))
        .collect();
    let aspects: Vec<(String, f64)> = [
        ("conj", 0.0),
        ("sext", 60.0),
        ("sq", 90.0),
        ("tri", 120.0),
        ("opp", 180.0),
    ]
    .iter()
    .map(|&(n, a)| (n.to_owned(), a))
    .collect();
    for (label, which) in [
        ("Moon only", vec![0_usize]),
        ("seven bodies", (0..7).collect()),
    ] {
        let config = TransitSearchConfig {
            natal_positions: natal.clone(),
            start_jd: y0,
            end_jd: y0 + 365.0,
            transiting_bodies: which.iter().map(|&i| (bodies[i].0.to_owned(), i)).collect(),
            aspect_types: aspects.clone(),
            max_orb: 1.0,
            step_size: 1.0,
        };
        let run = |src: &Src| -> Vec<TransitEvent> {
            let f = |i: usize, t: f64| src.lon(bodies.get(i)?.1, t);
            let mut events = search_transits(&config, &f);
            events.sort_by(|p, q| {
                (&p.transiting_body, &p.natal_body, &p.aspect_type)
                    .cmp(&(&q.transiting_body, &q.natal_body, &q.aspect_type))
                    .then(p.exact_jd.total_cmp(&q.exact_jd))
            });
            events
        };
        let planets = [
            Body::Mercury,
            Body::Venus,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
        ];
        let direct = Src::direct();
        let before = Calls::now();
        let a = run(&direct);
        let calls_direct = Calls::now().since(before);
        let walk = Src::walk(y0 - 1.0, y0 + 367.0, &planets);
        let before = Calls::now();
        let b = run(&walk);
        let calls_walk = Calls::now().since(before);
        let w = walk.walked.as_ref().unwrap();
        let (segments, fitted_nodes) = w.segments();
        let mut diff = compare_times(
            &a.iter().map(|e| e.exact_jd).collect::<Vec<_>>(),
            &b.iter().map(|e| e.exact_jd).collect::<Vec<_>>(),
        );
        dump(
            &format!("search_transits, {label}"),
            &a.iter().map(|e| e.exact_jd).collect::<Vec<_>>(),
        );
        // The pairing above is by time alone; pair by (body, natal, aspect) too.
        let mut keyed_worst = 0.0_f64;
        let mut keyed_missing = 0;
        let mut applying_differs = 0;
        for x in &a {
            let candidate = b
                .iter()
                .filter(|y| {
                    y.transiting_body == x.transiting_body
                        && y.natal_body == x.natal_body
                        && y.aspect_type == x.aspect_type
                })
                .min_by(|p, q| {
                    (p.exact_jd - x.exact_jd)
                        .abs()
                        .total_cmp(&(q.exact_jd - x.exact_jd).abs())
                });
            match candidate {
                Some(y) if (y.exact_jd - x.exact_jd).abs() < 0.5 => {
                    keyed_worst = keyed_worst.max((y.exact_jd - x.exact_jd).abs());
                    applying_differs += usize::from(x.applying != y.applying);
                }
                _ => keyed_missing += 1,
            }
        }
        diff.worst_s = keyed_worst * 86_400.0;
        Row {
            name: format!("search_transits, {label}, 8 natal points x 5 aspects, 365 days"),
            span_days: 365.0,
            reads_direct: direct.reads.load(Relaxed),
            reads_walk: walk.reads.load(Relaxed),
            calls_direct,
            calls_walk,
            segments,
            fitted_nodes,
            fallbacks: w.fallbacks.load(Relaxed),
            diff,
            note: format!(
                "keyed by (body, natal, aspect): {keyed_missing} unmatched, {applying_differs} applying flags differ"
            ),
        }
        .print();
    }
}
