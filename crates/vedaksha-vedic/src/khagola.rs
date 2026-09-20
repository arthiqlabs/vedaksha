// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Khagola solvers — ingress, station, heliacal visibility, eclipse.
//!
//! # Sources
//!
//! - Meeus, *Astronomical Algorithms*, 2nd ed.: Ch. 12 (sidereal time, via
//!   [`vedaksha_ephem_core::sidereal_time::gmst`]), Ch. 13 (altitude from
//!   hour angle), Ch. 40 (topocentric parallax, oblate-Earth vector form),
//!   Ch. 54 (eclipse shadow-cone geometry, including the 2% umbral
//!   enlargement).
//! - Surya Siddhanta: the classical grahana (eclipse) and udaya/asta
//!   (heliacal rise/set) theory behind the event definitions.
//!
//! # Contract
//!
//! Locale-free canonical values (Julian Days UT, degrees, rashi indices).
//! Every ephemeris query is a caller callback; the engine holds no tables
//! here. [`crate::muhurta::refine_crossing`] is the shared longitude solver
//! — this module brackets, it refines.
//!
//! # Frames and approximations (read before citing a result)
//!
//! - Altitudes use MEAN equinox LST (`gmst` + longitude): nutation (±1")
//!   is below any arcus-visionis criterion.
//! - Topocentric parallax uses the oblate Earth (WGS84 `1/298.257223`,
//!   equatorial radius 6378.137 km), exact vector form — no series.
//! - Constants: AU 149597870.7 km (IAU 2012, exact by definition);
//!   R_sun 695700 km (IAU 2015 nominal); R_moon 1737.4 km (standard mean).
//! - Heliacal visibility is GEOMETRY the caller thresholds: the classical
//!   arcus visionis (extinction, magnitude, season) is the caller's
//!   `critical_altitude_deg`, not an engine table.

use vedaksha_astro::sidereal::{Ayanamsha, true_ayanamsha_value};
use vedaksha_ephem_core::sidereal_time::gmst;

use crate::graha::Graha;
use crate::muhurta::refine_crossing;

// ── shared helpers ──────────────────────────────────────────────

/// Scan windows (documented, not tuned): ingress 400 d, station 800 d,
/// heliacal 600 d, eclipse 8 syzygies per kind.
const INGRESS_STEPS: u32 = 400;
const STATION_STEPS: u32 = 800;
const HELIACAL_WINDOW_D: f64 = 600.0;
const GRAHANA_SYZYGIES: u32 = 8;

/// Rashi index (0–11) of a sidereal longitude.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rashi_idx(lon_deg: f64) -> u8 {
    (lon_deg.rem_euclid(360.0) / 30.0).floor() as u8 % 12
}

/// (angle, rate) adapter over a position-only callback, rate by central
/// difference (h = 0.002 d ≈ 3 min), branch-cut unwrapped. For the smooth
/// scanning angles here (Sun, Moon, elongation) the truncation error is
/// ~1e-7 °/day — far below the 1e-6° refinement stop.
fn numeric_angle_rate(pos: &(dyn Fn(f64) -> Option<f64> + Sync), t: f64) -> Option<(f64, f64)> {
    let h = 0.002;
    let (a0, a1) = (pos(t - h)?, pos(t + h)?);
    let mut d = a1 - a0;
    if d > 180.0 {
        d -= 360.0;
    }
    if d < -180.0 {
        d += 360.0;
    }
    Some((pos(t)?, d / (2.0 * h)))
}
/// Same rule as `kala_mana`'s scanner: a step wider than 180° is the 0°
/// wrap, a crossing only for target 0°.
fn crossed(e0: f64, e1: f64, target: f64) -> bool {
    if target == 0.0 {
        e1 < e0 && (e0 > 300.0 || e1 < 60.0)
    } else {
        (e1 - e0).abs() < 180.0 && ((e0 < target && e1 >= target) || (e0 >= target && e1 < target))
    }
}

/// Bisection root of `f` (sign-changing) on `[a, b]`, 50 iterations.
fn bisect(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64) -> Option<f64> {
    let mut lo = a;
    let mut hi = b;
    let mut flo = f(lo)?;
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        let fmid = f(mid)?;
        if flo == 0.0 {
            return Some(lo);
        }
        if flo.signum() == fmid.signum() {
            lo = mid;
            flo = fmid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Minimum of convex `f` on `[a, b]` by ternary search, 60 iterations.
fn ternary_min(f: &(dyn Fn(f64) -> Option<f64> + Sync), a: f64, b: f64) -> Option<f64> {
    let mut lo = a;
    let mut hi = b;
    for _ in 0..60 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        let (f1, f2) = (f(m1)?, f(m2)?);
        if f1 > f2 {
            lo = m1;
        } else {
            hi = m2;
        }
    }
    Some(0.5 * (lo + hi))
}

// ── ingress ─────────────────────────────────────────────────────

/// Next ingress of a tropical longitude into a sidereal rashi: `(jd, rashi)`.
///
/// Brackets a rashi-index change on 1-day steps (no body moves 30°/day),
/// then refines the edge. Retrograde crossings (nodes) refine on the
/// sign-flipped angle and report the rashi entered downward. `rate` from
/// the callback seeds the direction; the crossing instant's own sign rules
/// where a station sits inside the bracket.
///
/// Returns `None` where the callback is unavailable or no ingress falls in
/// the 400-day window.
fn next_ingress(
    jd_start: f64,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    let sidereal = |t: f64| {
        let (lon, rate) = longitude(t)?;
        Some((
            (lon - true_ayanamsha_value(ayanamsha, t)).rem_euclid(360.0),
            rate,
        ))
    };
    let rashi = |t: f64| sidereal(t).map(|(lon, _)| rashi_idx(lon));
    let mut t0 = jd_start;
    let mut i0 = rashi(t0)?;
    // Step off an exact boundary: a start within a second BEFORE an edge
    // would otherwise report that edge as the "next" ingress.
    if rashi(t0 + 1.0 / 86400.0)? != i0 {
        t0 += 1.0 / 86400.0;
        i0 = rashi(t0)?;
    }
    let mut t = t0;
    for _ in 0..INGRESS_STEPS {
        let t1 = t + 1.0;
        let i1 = rashi(t1)?;
        if i1 != i0 {
            // Entered i1. Direct up-crossing refines edge i1*30; a
            // down-crossing (i1 == i0 - 1 mod 12) refines edge (i1+1)*30
            // on the flipped angle.
            let direct = i1 == (i0 + 1) % 12;
            let edge = if direct {
                f64::from(i1) * 30.0
            } else {
                f64::from((i1 + 1) % 12) * 30.0
            };
            let rate_now = sidereal(t1)?.1;
            let angle_at = |tt: f64| {
                let (lon, rate) = sidereal(tt)?;
                if direct || rate_now >= 0.0 {
                    Some((lon, rate))
                } else {
                    Some(((-lon).rem_euclid(360.0), -rate))
                }
            };
            let target = if direct || rate_now >= 0.0 {
                edge
            } else {
                (-edge).rem_euclid(360.0)
            };
            let jd = refine_crossing(target, t, &angle_at)?;
            return Some((jd, i1));
        }
        t = t1;
    }
    None
}

/// Next solar ingress (sankranti): `(jd, rashi entered)`, sidereal.
#[must_use]
pub fn sun_ingress(
    jd_start: f64,
    ayanamsha: Ayanamsha,
    sun: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    next_ingress(jd_start, ayanamsha, sun)
}

/// Next ingress of `graha` into a sidereal rashi: `(jd, rashi entered)`.
///
/// `graha` labels which body `longitude` serves (the math needs only the
/// callback); the nodes' retrograde crossings are handled, not rejected.
/// Residual risk, stated: where a station sits inside the 1-day detection
/// bracket, the edge is picked by the crossing instant's rate sign — a
/// station exactly ON the edge refines the wrong side's edge.
#[must_use]
pub fn graha_ingress(
    jd_start: f64,
    graha: Graha,
    ayanamsha: Ayanamsha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<(f64, u8)> {
    let _ = graha;
    next_ingress(jd_start, ayanamsha, longitude)
}

// ── station ─────────────────────────────────────────────────────

/// Next station (speed sign change) of a star-planet: the JD of the turning
/// point, located by bisection on the rate bracket. The Sun, Moon and nodes
/// do not station — `None` for them, as for an unavailable callback or an
/// empty 800-day window.
#[must_use]
pub fn station(
    jd_start: f64,
    graha: Graha,
    longitude: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => return None,
        Graha::Mars | Graha::Mercury | Graha::Jupiter | Graha::Venus | Graha::Saturn => {}
    }
    let rate = |t: f64| longitude(t).map(|(_, r)| r);
    let mut t0 = jd_start;
    let mut r0 = rate(t0)?;
    if r0 == 0.0 {
        return Some(t0);
    }
    for _ in 0..STATION_STEPS {
        let t1 = t0 + 1.0;
        let r1 = rate(t1)?;
        if r1 == 0.0 {
            return Some(t1);
        }
        if r0.signum() != r1.signum() {
            let f = |t: f64| rate(t);
            return bisect(&f, t0, t1);
        }
        t0 = t1;
        r0 = r1;
    }
    None
}

// ── heliacal visibility ─────────────────────────────────────────

/// Altitude (degrees) of an equatorial body over the observer.
fn altitude_deg(jd_ut: f64, lat_deg: f64, lon_east_deg: f64, ra_deg: f64, dec_deg: f64) -> f64 {
    let lst = gmst(jd_ut).to_degrees() + lon_east_deg;
    let h = (lst - ra_deg).to_radians();
    let (phi, dec) = (lat_deg.to_radians(), dec_deg.to_radians());
    (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

/// Next sunrise/sunset pair bracketing: returns the event JD or `None`.
fn sun_event(
    jd_start: f64,
    window_d: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    rising: bool,
) -> Option<f64> {
    // 6-hour strides: no sunrise/sunset is skipped outside the poles.
    let mut t = jd_start;
    let alt = |tt: f64| {
        let (ra, dec) = sun_eq(tt)?;
        Some(altitude_deg(tt, lat_deg, lon_east_deg, ra, dec))
    };
    let mut a0 = alt(t)?;
    let end = jd_start + window_d;
    while t < end {
        let t1 = (t + 0.25).min(end);
        let a1 = alt(t1)?;
        let is_event = if rising {
            a0 < 0.0 && a1 >= 0.0
        } else {
            a0 >= 0.0 && a1 < 0.0
        };
        if is_event {
            let f = |tt: f64| alt(tt);
            return bisect(&f, t, t1);
        }
        t = t1;
        a0 = a1;
    }
    None
}

/// First sunrise at which `graha` stands at least `critical_altitude_deg`
/// above the horizon — the geometric content of heliacal rise (udaya).
/// The criterion (extinction, magnitude, season) is the caller's: pass the
/// tradition's arcus visionis, typically ~10°. `None` for the Sun and nodes
/// (no visible body), an unavailable callback, or an empty 600-day window.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn heliacal_rise(
    jd_start: f64,
    graha: Graha,
    latitude_deg: f64,
    longitude_deg_east: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    critical_altitude_deg: f64,
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Rahu | Graha::Ketu => return None,
        _ => {}
    }
    let mut t = jd_start;
    let end = jd_start + HELIACAL_WINDOW_D;
    while t < end {
        let rise = sun_event(t, end - t, latitude_deg, longitude_deg_east, sun_eq, true)?;
        let (ra, dec) = graha_eq(rise)?;
        if altitude_deg(rise, latitude_deg, longitude_deg_east, ra, dec) >= critical_altitude_deg {
            return Some(rise);
        }
        t = rise + 0.5;
    }
    None
}

/// Last sunset at which `graha` still stands at least
/// `critical_altitude_deg` above the horizon — the geometric content of
/// heliacal set (asta). Window semantics: the LAST qualifying sunset in the
/// 600-day window; a caller hunting an apparition's end past the window
/// recalls with a later `jd_start`. `None` cases as in [`heliacal_rise`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn heliacal_set(
    jd_start: f64,
    graha: Graha,
    latitude_deg: f64,
    longitude_deg_east: f64,
    sun_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    graha_eq: &(dyn Fn(f64) -> Option<(f64, f64)> + Sync),
    critical_altitude_deg: f64,
) -> Option<f64> {
    match graha {
        Graha::Sun | Graha::Rahu | Graha::Ketu => return None,
        _ => {}
    }
    let mut t = jd_start;
    let end = jd_start + HELIACAL_WINDOW_D;
    let mut last: Option<f64> = None;
    while t < end {
        let Some(set) = sun_event(t, end - t, latitude_deg, longitude_deg_east, sun_eq, false)
        else {
            break;
        };
        let (ra, dec) = graha_eq(set)?;
        if altitude_deg(set, latitude_deg, longitude_deg_east, ra, dec) >= critical_altitude_deg {
            last = Some(set);
        }
        t = set + 0.5;
    }
    last
}

// ── eclipse ─────────────────────────────────────────────────────

/// Geocentric ecliptic geometry for the lunar computation.
#[derive(Debug, Clone, Copy)]
pub struct LunarGeometry {
    pub sun_lon_deg: f64,
    pub sun_dist_au: f64,
    pub moon_lon_deg: f64,
    pub moon_lat_deg: f64,
    pub moon_dist_au: f64,
}

/// Geocentric equatorial geometry for the solar computation.
#[derive(Debug, Clone, Copy)]
pub struct SolarGeometry {
    pub sun_ra_deg: f64,
    pub sun_dec_deg: f64,
    pub sun_dist_au: f64,
    pub moon_ra_deg: f64,
    pub moon_dec_deg: f64,
    pub moon_dist_au: f64,
}

/// Eclipse kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrahanaKind {
    LunarPenumbral,
    LunarPartial,
    LunarTotal,
    SolarPartial,
    SolarTotal,
    SolarAnnular,
}

/// An eclipse: the sparsha/madhya/moksha triple with kind and magnitude.
///
/// Lunar contacts are penumbral C1/greatest/C4 (umbral U1–U4 are NOT
/// separately returned); magnitude is umbral (lunar) or obscuration
/// (solar), > 1 for total.
#[derive(Debug, Clone, Copy)]
pub struct Grahana {
    pub kind: GrahanaKind,
    pub sparsha_jd: f64,
    pub madhya_jd: f64,
    pub moksha_jd: f64,
    pub magnitude: f64,
}

const AU_KM: f64 = 149_597_870.7;
const R_EARTH_KM: f64 = 6378.137;
const FLATTENING: f64 = 1.0 / 298.257_223;
const R_SUN_KM: f64 = 695_700.0;
const R_MOON_KM: f64 = 1737.4;

/// Angular semidiameter (degrees) of `radius_km` at `dist_au`.
fn semidiameter_deg(radius_km: f64, dist_au: f64) -> f64 {
    (radius_km / (dist_au * AU_KM)).asin().to_degrees()
}

/// Horizontal parallax (degrees) at `dist_au`.
fn parallax_deg(dist_au: f64) -> f64 {
    (R_EARTH_KM / (dist_au * AU_KM)).asin().to_degrees()
}

/// Angular separation of two ecliptic points (degrees).
fn ecl_sep(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let (l1, b1, l2, b2) = (
        lon1.to_radians(),
        lat1.to_radians(),
        lon2.to_radians(),
        lat2.to_radians(),
    );
    (b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos())
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// Topocentric RA/dec (degrees) from geocentric, via the oblate-Earth
/// vector (Meeus Ch. 40, exact form): station from geodetic latitude,
/// `theta` = mean LST in radians, distance in AU.
fn topocentric_ra_dec(
    ra_deg: f64,
    dec_deg: f64,
    dist_au: f64,
    lat_deg: f64,
    theta_rad: f64,
) -> (f64, f64) {
    let e2 = 2.0 * FLATTENING - FLATTENING * FLATTENING;
    let (phi_sin, phi_cos) = lat_deg.to_radians().sin_cos();
    let big_n = R_EARTH_KM / (1.0 - e2 * phi_sin * phi_sin).sqrt();
    let st = (
        (big_n * phi_cos) * theta_rad.cos(),
        (big_n * phi_cos) * theta_rad.sin(),
        (big_n * (1.0 - e2) * phi_sin),
    );
    let (ra, dec) = (ra_deg.to_radians(), dec_deg.to_radians());
    let d_km = dist_au * AU_KM;
    let geo = (
        d_km * dec.cos() * ra.cos(),
        d_km * dec.cos() * ra.sin(),
        d_km * dec.sin(),
    );
    let topo = (geo.0 - st.0, geo.1 - st.1, geo.2 - st.2);
    let ra_t = topo.1.atan2(topo.0).to_degrees().rem_euclid(360.0);
    let dec_t = (topo.2 / (topo.0.hypot(topo.1).hypot(topo.2)))
        .asin()
        .to_degrees();
    (ra_t, dec_t)
}

/// Topocentric Sun–Moon separation (degrees).
fn topo_sep(g: &SolarGeometry, jd_ut: f64, lat_deg: f64, lon_east_deg: f64) -> f64 {
    let theta = (gmst(jd_ut).to_degrees() + lon_east_deg).to_radians();
    let (sun_ra, sun_dec) =
        topocentric_ra_dec(g.sun_ra_deg, g.sun_dec_deg, g.sun_dist_au, lat_deg, theta);
    let (moon_ra, moon_dec) = topocentric_ra_dec(
        g.moon_ra_deg,
        g.moon_dec_deg,
        g.moon_dist_au,
        lat_deg,
        theta,
    );
    ecl_sep(sun_ra, sun_dec, moon_ra, moon_dec)
}

/// Next lunar eclipse at/after `jd_start` (geocentric shadow cone).
fn next_lunar(
    jd_start: f64,
    lunar: &(dyn Fn(f64) -> Option<LunarGeometry> + Sync),
) -> Option<Grahana> {
    let elong = |t: f64| {
        let g = lunar(t)?;
        Some((g.moon_lon_deg - g.sun_lon_deg).rem_euclid(360.0))
    };
    let gamma = |t: f64| {
        let g = lunar(t)?;
        Some(ecl_sep(
            g.moon_lon_deg,
            g.moon_lat_deg,
            (g.sun_lon_deg + 180.0).rem_euclid(360.0),
            0.0,
        ))
    };
    let mut t = jd_start;
    for _ in 0..GRAHANA_SYZYGIES {
        // Next full moon: first 180-crossing after t.
        let mut t0 = t;
        let mut e0 = elong(t0)?;
        let mut full: Option<f64> = None;
        for _ in 0..40 {
            let t1 = t0 + 1.0;
            let e1 = elong(t1)?;
            if crossed(e0, e1, 180.0) {
                let pos = |tt: f64| elong(tt);
                let f = |tt: f64| numeric_angle_rate(&pos, tt);
                full = refine_crossing(180.0, t0, &f);
                break;
            }
            t0 = t1;
            e0 = e1;
        }
        let fm = full?;
        let gmin_t = ternary_min(&gamma, fm - 2.0, fm + 2.0)?;
        let gmin = gamma(gmin_t)?;
        let g = lunar(gmin_t)?;
        let pi_m = parallax_deg(g.moon_dist_au);
        let pi_s = parallax_deg(g.sun_dist_au);
        let s_s = semidiameter_deg(R_SUN_KM, g.sun_dist_au);
        let s_m = semidiameter_deg(R_MOON_KM, g.moon_dist_au);
        // Meeus Ch. 54 shadow radii, umbra enlarged 2% for the atmosphere.
        let umbra = 1.02 * (pi_m + pi_s - s_s);
        let penumbra = pi_m + pi_s + s_s;
        if gmin < penumbra + s_m {
            let c1 = {
                let f = |tt: f64| gamma(tt).map(|x| x - (penumbra + s_m));
                bisect(&f, fm - 2.0, gmin_t)?
            };
            let c4 = {
                let f = |tt: f64| gamma(tt).map(|x| x - (penumbra + s_m));
                bisect(&f, gmin_t, fm + 2.0)?
            };
            let kind = if gmin < umbra - s_m {
                GrahanaKind::LunarTotal
            } else if gmin < umbra + s_m {
                GrahanaKind::LunarPartial
            } else {
                GrahanaKind::LunarPenumbral
            };
            return Some(Grahana {
                kind,
                sparsha_jd: c1,
                madhya_jd: gmin_t,
                moksha_jd: c4,
                magnitude: (umbra + s_m - gmin) / (2.0 * s_m),
            });
        }
        t = fm + 15.0;
    }
    None
}

/// Next solar eclipse at/after `jd_start`, topocentric at the observer.
fn next_solar(
    jd_start: f64,
    lat_deg: f64,
    lon_east_deg: f64,
    solar: &(dyn Fn(f64) -> Option<SolarGeometry> + Sync),
) -> Option<Grahana> {
    // Bracket by RA conjunction (ΔRA wraps +180 → −180 at new moon).
    let dra = |t: f64| {
        let g = solar(t)?;
        Some((g.moon_ra_deg - g.sun_ra_deg).rem_euclid(360.0))
    };
    let sep = |t: f64| {
        let g = solar(t)?;
        Some(topo_sep(&g, t, lat_deg, lon_east_deg))
    };
    let mut t = jd_start;
    for _ in 0..GRAHANA_SYZYGIES {
        let mut t0 = t;
        let mut e0 = dra(t0)?;
        let mut conj: Option<f64> = None;
        for _ in 0..40 {
            let t1 = t0 + 1.0;
            let e1 = dra(t1)?;
            if crossed(e0, e1, 0.0) {
                let pos = |tt: f64| dra(tt);
                let f = |tt: f64| numeric_angle_rate(&pos, tt);
                conj = refine_crossing(0.0, t0, &f);
                break;
            }
            t0 = t1;
            e0 = e1;
        }
        let nm = conj?;
        let smin_t = ternary_min(&sep, nm - 1.5, nm + 1.5)?;
        let smin = sep(smin_t)?;
        let g = solar(smin_t)?;
        let s_s = semidiameter_deg(R_SUN_KM, g.sun_dist_au);
        let s_m = semidiameter_deg(R_MOON_KM, g.moon_dist_au);
        if smin < s_s + s_m {
            let c1 = {
                let f = |tt: f64| sep(tt).map(|x| x - (s_s + s_m));
                bisect(&f, nm - 1.5, smin_t)?
            };
            let c4 = {
                let f = |tt: f64| sep(tt).map(|x| x - (s_s + s_m));
                bisect(&f, smin_t, nm + 1.5)?
            };
            let kind = if smin + s_s <= s_m {
                GrahanaKind::SolarTotal
            } else if smin + s_m <= s_s {
                GrahanaKind::SolarAnnular
            } else {
                GrahanaKind::SolarPartial
            };
            return Some(Grahana {
                kind,
                sparsha_jd: c1,
                madhya_jd: smin_t,
                moksha_jd: c4,
                magnitude: (s_s + s_m - smin) / (2.0 * s_s),
            });
        }
        t = nm + 15.0;
    }
    None
}

/// Next eclipse (grahana) at/after `jd_start`: the sparsha/madhya/moksha
/// triple, solar or lunar, whichever comes first.
///
/// Lunar is geocentric (shadow cone at the Moon — no observer needed);
/// solar is topocentric at (`latitude_deg`, `longitude_deg_east`). Either
/// callback returning `None` throughout skips that kind. Returns `None`
/// where neither kind occurs in the next 8 syzygies each.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn grahana(
    jd_start: f64,
    latitude_deg: f64,
    longitude_deg_east: f64,
    lunar: &(dyn Fn(f64) -> Option<LunarGeometry> + Sync),
    solar: &(dyn Fn(f64) -> Option<SolarGeometry> + Sync),
) -> Option<Grahana> {
    match (
        next_lunar(jd_start, lunar),
        next_solar(jd_start, latitude_deg, longitude_deg_east, solar),
    ) {
        (Some(l), Some(s)) => Some(if l.sparsha_jd <= s.sparsha_jd { l } else { s }),
        (Some(l), None) => Some(l),
        (None, Some(s)) => Some(s),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: f64 = 2_460_000.0;

    #[test]
    fn sun_ingress_finds_the_next_edge() {
        // Tropical sun 100° + 1°/day; sidereal ≈ 75.8 + 1°/day -> edge 90
        // (Karka, rashi 3) ~14.2 d on.
        let f = |t: f64| Some((100.0 + (t - T0), 1.0));
        let (jd, rashi) = sun_ingress(T0, Ayanamsha::IndianOfficial, &f).unwrap();
        assert_eq!(rashi, 3);
        assert!((jd - (T0 + 14.2)).abs() < 0.1, "jd = {jd}");
    }

    #[test]
    fn retrograde_ingress_reports_the_rashi_entered_downward() {
        // Nodes: tropical 100° falling 0.05°/day; sidereal ≈ 75.8 falling:
        // edge 60 downward into rashi 1, ~316 d on (inside the 400 d window).
        let f = |t: f64| Some((100.0 - 0.05 * (t - T0), -0.05));
        let (jd, rashi) = graha_ingress(T0, Graha::Rahu, Ayanamsha::IndianOfficial, &f).unwrap();
        assert_eq!(rashi, 1);
        assert!((jd - (T0 + 316.0)).abs() < 5.0, "jd = {jd}");
    }

    #[test]
    fn station_bisects_the_sign_change() {
        // lon = 100 + 5 sin(2π(t−T0)/100): rate zero at T0+25.
        let f = |t: f64| {
            let w = core::f64::consts::TAU * (t - T0) / 100.0;
            Some((
                100.0 + 5.0 * w.sin(),
                5.0 * core::f64::consts::TAU / 100.0 * w.cos(),
            ))
        };
        let st = station(T0, Graha::Mars, &f).unwrap();
        assert!((st - (T0 + 25.0)).abs() < 0.01, "station = {st}");
        // Luminaries and nodes never station.
        assert_eq!(station(T0, Graha::Sun, &f), None);
        assert_eq!(station(T0, Graha::Moon, &f), None);
        assert_eq!(station(T0, Graha::Rahu, &f), None);
    }

    #[test]
    fn constant_motion_never_stations() {
        let f = |t: f64| Some((100.0 + (t - T0), 1.0));
        assert_eq!(station(T0, Graha::Mars, &f), None);
    }

    #[test]
    fn heliacal_rise_finds_first_qualifying_sunrise() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // H(T0) = 280° increasing 360°/day: the morning sun is already up
        // (alt +10°), so the first sunrise forward is the NEXT one, H=630°,
        // +0.9724 d on. Graha 30° east of the sun in RA stands at alt +30°
        // there — qualifies.
        let lst0 = gmst(T0).to_degrees();
        let ra0 = lst0 - 280.0;
        let sun = move |t: f64| Some((ra0 + (t - T0), 0.0));
        let graha = move |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra - 30.0, 0.0))
        };
        let rise = heliacal_rise(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0).unwrap();
        assert!((rise - T0 - 0.9724).abs() < 0.02, "rise = {rise}");
    }

    #[test]
    fn heliacal_set_returns_last_qualifying_sunset_in_window() {
        // Graha 30° west of the sun: alt +30° at every sunset, −30° at
        // every sunrise. Every sunset in the 600-day window qualifies, so
        // the last sits near the window's end (window semantics, module doc).
        let sun = |t: f64| Some((280.0 + 1.0 * (t - T0), 0.0));
        let graha = |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra + 30.0, 0.0))
        };
        let set = heliacal_set(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0).unwrap();
        assert!((set - T0 - 599.5).abs() < 2.0, "set = {set}");
    }

    #[test]
    fn conjunct_graha_never_rises_heliacally() {
        // Graha glued to the sun: alt ~0 at every sunrise < 10° -> None.
        // (Full 600-day scan; stubs are cheap.)
        let sun = |t: f64| Some((280.0 + 1.0 * (t - T0), 0.0));
        let graha = |t: f64| {
            let (sra, _) = sun(t)?;
            Some((sra, 0.0))
        };
        assert_eq!(
            heliacal_rise(T0, Graha::Venus, 0.0, 0.0, &sun, &graha, 10.0),
            None
        );
        assert_eq!(
            heliacal_rise(T0, Graha::Sun, 0.0, 0.0, &sun, &graha, 10.0),
            None
        );
    }

    #[test]
    fn lunar_total_eclipse_triple_ordered() {
        // New moon T0−10, full T0+5, moon lat 0.3°: total (gamma 0.3 <
        // umbra − moon radius ≈ 0.44).
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 0.3,
                moon_dist_au: 0.00257,
            })
        };
        let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
        let g = grahana(T0, 0.0, 0.0, &lunar, &no_solar).unwrap();
        assert_eq!(g.kind, GrahanaKind::LunarTotal);
        assert!(g.sparsha_jd < g.madhya_jd && g.madhya_jd < g.moksha_jd);
        assert!(g.magnitude > 1.0);
        assert!((g.madhya_jd - (T0 + 5.0)).abs() < 0.5);
    }

    #[test]
    fn high_latitude_full_moon_is_no_eclipse() {
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 5.0,
                moon_dist_au: 0.00257,
            })
        };
        let no_solar: fn(f64) -> Option<SolarGeometry> = |_| None;
        assert!(grahana(T0, 0.0, 0.0, &lunar, &no_solar).is_none());
    }

    #[test]
    fn solar_partial_eclipse_at_subsolar_observer() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // Conjunction at T0 (RA), moon dec 0.1°: central sep 0.1° < 0.53°
        // but > |s_m − s_s| ≈ 0.007° -> partial. Observer subsolar at T0
        // so parallax ~0 and the geometry is exact.
        let lon_east = (-gmst(T0).to_degrees()).rem_euclid(360.0);
        let solar = move |t: f64| {
            Some(SolarGeometry {
                sun_ra_deg: 0.0,
                sun_dec_deg: 0.0,
                sun_dist_au: 1.0,
                moon_ra_deg: 13.0 * (t - T0),
                moon_dec_deg: 0.1,
                moon_dist_au: 0.00257,
            })
        };
        let no_lunar: fn(f64) -> Option<LunarGeometry> = |_| None;
        let g = grahana(T0 - 1.0, 0.0, lon_east, &no_lunar, &solar).unwrap();
        assert_eq!(g.kind, GrahanaKind::SolarPartial);
        assert!(g.sparsha_jd < g.madhya_jd && g.madhya_jd < g.moksha_jd);
        assert!((g.madhya_jd - T0).abs() < 0.1, "madhya = {}", g.madhya_jd);
    }

    #[test]
    fn earliest_sparsha_wins_between_kinds() {
        use vedaksha_ephem_core::sidereal_time::gmst;
        // Solar conjunction T0, lunar full moon T0+5 (total): solar first.
        // Subsolar observer again, so parallax cannot hide the solar event.
        let lon_east = (-gmst(T0).to_degrees()).rem_euclid(360.0);
        let lunar = |t: f64| {
            let elong = ((t - (T0 - 10.0)) * 12.0).rem_euclid(360.0);
            let sun_lon = 100.0;
            Some(LunarGeometry {
                sun_lon_deg: sun_lon,
                sun_dist_au: 1.0,
                moon_lon_deg: (sun_lon + elong).rem_euclid(360.0),
                moon_lat_deg: 0.3,
                moon_dist_au: 0.00257,
            })
        };
        let solar = |t: f64| {
            Some(SolarGeometry {
                sun_ra_deg: 0.0,
                sun_dec_deg: 0.0,
                sun_dist_au: 1.0,
                moon_ra_deg: 13.0 * (t - T0),
                moon_dec_deg: 0.1,
                moon_dist_au: 0.00257,
            })
        };
        let g = grahana(T0 - 1.0, 0.0, lon_east, &lunar, &solar).unwrap();
        assert!(g.sparsha_jd < T0 + 4.0, "sparsha = {}", g.sparsha_jd);
    }
}
