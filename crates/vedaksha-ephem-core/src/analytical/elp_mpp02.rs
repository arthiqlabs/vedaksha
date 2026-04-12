// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.

//! ELP/MPP02 lunar theory evaluator.
//!
//! Evaluates the ELP/MPP02 series (Chapront & Francou 2003, A&A 404, 735)
//! to compute the Moon's geocentric ecliptic coordinates.
//!
//! The series is evaluated in the ecliptic-of-date frame, then rotated to
//! J2000.0 mean ecliptic via the P,Q precession matrix (Chapront 2002).
//!
//! # Coordinate system (output)
//! - Origin: Earth (geocentric)
//! - Plane: mean ecliptic of J2000.0
//! - Units: km (position), km/day (velocity)
//!
//! # Reference implementation
//! Based on ytliu0/ElpMpp02 (C++) with corr=0 (LLR-fit) parameters.

use super::coefficients::{moon_distance, moon_latitude, moon_longitude};

/// Degrees to radians conversion factor.
const DEG_TO_RAD: f64 = core::f64::consts::PI / 180.0;

/// Arcseconds to radians conversion factor.
const SEC_TO_RAD: f64 = core::f64::consts::PI / 648_000.0;

/// Distance scale factor: ra0 = 384747.961370173 / 384747.980674318
const RA0: f64 = 384_747.961_370_173 / 384_747.980_674_318;

// ─── LLR-fit parameters (corr=0) ──────────────────────────────────────────

const DW1_0: f64 = -0.10525;
const DW1_1: f64 = -0.32311;
const DW1_2: f64 = -0.03794;
const DW1_3: f64 = 0.0;
const DW1_4: f64 = 0.0;

const DW2_0: f64 = 0.16826;
const DW2_1: f64 = 0.08017;

const DW3_0: f64 = -0.10760;
const DW3_1: f64 = -0.04317;

const DEART_0: f64 = -0.04012;
const DEART_1: f64 = 0.01442;

const DPERI: f64 = -0.04854;

// Derived: Cw2_1 and Cw3_1 (coupling corrections)
const DGAM: f64 = 0.00069;
const DE: f64 = 0.00005;
const DEP: f64 = 0.00226;

/// Geocentric position and velocity of the Moon in J2000 ecliptic rectangular coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonRectangular {
    /// X position in km (J2000 ecliptic)
    pub x: f64,
    /// Y position in km (J2000 ecliptic)
    pub y: f64,
    /// Z position in km (J2000 ecliptic)
    pub z: f64,
    /// X velocity in km/day (J2000 ecliptic)
    pub vx: f64,
    /// Y velocity in km/day (J2000 ecliptic)
    pub vy: f64,
    /// Z velocity in km/day (J2000 ecliptic)
    pub vz: f64,
}

/// Arguments bundle: Delaunay + planetary arguments, all in radians.
struct Arguments {
    w1: f64,
    d: f64,
    f: f64,
    l: f64,
    lp: f64,
    me: f64,
    ve: f64,
    em: f64,
    ma: f64,
    ju: f64,
    sa: f64,
    ur: f64,
    ne: f64,
    zeta: f64,
}

/// Compute coupling corrections Cw2_1 and Cw3_1.
fn compute_coupling() -> (f64, f64) {
    let am: f64 = 0.074801329;
    let alpha: f64 = 0.002571881;
    let _dtsm = 2.0 * alpha / (3.0 * am);
    let xa = 2.0 * alpha / 3.0;

    let w11 = (1_732_559_343.73604 + DW1_1) * SEC_TO_RAD;
    let w21 = (14_643_420.3171 + DW2_1) * SEC_TO_RAD;
    let w31 = (-6_967_919.5383 + DW3_1) * SEC_TO_RAD;

    let bp: [[f64; 2]; 5] = [
        [0.311079095, -0.103837907],
        [-0.004482398, 0.000668287],
        [-0.001102485, -0.001298072],
        [0.001056062, -0.000178028],
        [0.000050928, -0.000037342],
    ];

    let x2 = w21 / w11;
    let x3 = w31 / w11;
    let y2 = am * bp[0][0] + xa * bp[4][0];
    let y3 = am * bp[0][1] + xa * bp[4][1];
    let d21 = x2 - y2;
    let d22 = w11 * bp[1][0];
    let d23 = w11 * bp[2][0];
    let d24 = w11 * bp[3][0];
    let d25 = y2 / am;
    let d31 = x3 - y3;
    let d32 = w11 * bp[1][1];
    let d33 = w11 * bp[2][1];
    let d34 = w11 * bp[3][1];
    let d35 = y3 / am;

    let cw2_1 = d21 * DW1_1 + d25 * DEART_1 + d22 * DGAM + d23 * DE + d24 * DEP;
    let cw3_1 = d31 * DW1_1 + d35 * DEART_1 + d32 * DGAM + d33 * DE + d34 * DEP;
    (cw2_1, cw3_1)
}

/// Compute all arguments (Delaunay + planetary) at Julian centuries `t`.
///
/// Uses ecliptic-of-date polynomials from ELP/MPP02 with corr=0 LLR-fit.
fn compute_arguments(t: f64) -> Arguments {
    let t2 = t * t;
    let t3 = t * t2;
    let t4 = t2 * t2;

    let (cw2_1, cw3_1) = compute_coupling();

    // W1 = Moon mean longitude (ecliptic of date)
    let w1_0 = (-142.0 + 18.0 / 60.0 + (59.95571 + DW1_0) / 3600.0) * DEG_TO_RAD;
    let w1_1 = (1_732_559_343.73604 + DW1_1) * t * SEC_TO_RAD;
    let w1_2 = (-6.8084 + DW1_2) * t2 * SEC_TO_RAD;
    let w1_3 = (0.006604 + DW1_3) * t3 * SEC_TO_RAD;
    let w1_4 = (-3.169e-5 + DW1_4) * t4 * SEC_TO_RAD;
    let w1 = w1_0 + w1_1 + w1_2 + w1_3 + w1_4;

    // W2 = Moon perigee longitude
    let w2_0 = (83.0 + 21.0 / 60.0 + (11.67475 + DW2_0) / 3600.0) * DEG_TO_RAD;
    let w2_1 = (14_643_420.3171 + DW2_1 + cw2_1) * t * SEC_TO_RAD;
    let w2_2 = (-38.2631 + 0.0) * t2 * SEC_TO_RAD; // Dw2_2 = 0 for corr=0
    let w2_3 = (-0.045047 + 0.0) * t3 * SEC_TO_RAD;
    let w2_4 = 0.00021301 * t4 * SEC_TO_RAD;
    let w2 = w2_0 + w2_1 + w2_2 + w2_3 + w2_4;

    // W3 = Moon node longitude
    let w3_0 = (125.0 + 2.0 / 60.0 + (40.39816 + DW3_0) / 3600.0) * DEG_TO_RAD;
    let w3_1 = (-6_967_919.5383 + DW3_1 + cw3_1) * t * SEC_TO_RAD;
    let w3_2 = (6.359 + 0.0) * t2 * SEC_TO_RAD;
    let w3_3 = (0.007625 + 0.0) * t3 * SEC_TO_RAD;
    let w3_4 = -3.586e-5 * t4 * SEC_TO_RAD;
    let w3 = w3_0 + w3_1 + w3_2 + w3_3 + w3_4;

    // Ea = Sun mean longitude (Earth)
    let ea_0 = (100.0 + 27.0 / 60.0 + (59.13885 + DEART_0) / 3600.0) * DEG_TO_RAD;
    let ea_1 = (129_597_742.293 + DEART_1) * t * SEC_TO_RAD;
    let ea_2 = -0.0202 * t2 * SEC_TO_RAD;
    let ea_3 = 9e-6 * t3 * SEC_TO_RAD;
    let ea_4 = 1.5e-7 * t4 * SEC_TO_RAD;
    let ea = ea_0 + ea_1 + ea_2 + ea_3 + ea_4;

    // pomp = perihelion of Sun (longitude)
    let p_0 = (102.0 + 56.0 / 60.0 + (14.45766 + DPERI) / 3600.0) * DEG_TO_RAD;
    let p_1 = 1161.24342 * t * SEC_TO_RAD;
    let p_2 = 0.529265 * t2 * SEC_TO_RAD;
    let p_3 = -1.1814e-4 * t3 * SEC_TO_RAD;
    let p_4 = 1.1379e-5 * t4 * SEC_TO_RAD;
    let pomp = p_0 + p_1 + p_2 + p_3 + p_4;

    // Delaunay arguments
    let d = w1 - ea + core::f64::consts::PI;
    let f = w1 - w3;
    let l = w1 - w2;
    let lp = ea - pomp;

    // zeta
    let zeta = w1 + 0.024_380_295_608_819_07 * t;

    // Planetary arguments (mean longitudes)
    let me = (-108.0 + 15.0 / 60.0 + 3.216919 / 3600.0) * DEG_TO_RAD
        + 538_101_628.66888 * t * SEC_TO_RAD;
    let ve = (-179.0 + 58.0 / 60.0 + 44.758419 / 3600.0) * DEG_TO_RAD
        + 210_664_136.45777 * t * SEC_TO_RAD;
    let em = (100.0 + 27.0 / 60.0 + 59.13885 / 3600.0) * DEG_TO_RAD
        + 129_597_742.293 * t * SEC_TO_RAD;
    let ma = (-5.0 + 26.0 / 60.0 + 3.642778 / 3600.0) * DEG_TO_RAD
        + 68_905_077.65936 * t * SEC_TO_RAD;
    let ju = (34.0 + 21.0 / 60.0 + 5.379392 / 3600.0) * DEG_TO_RAD
        + 10_925_660.57335 * t * SEC_TO_RAD;
    let sa = (50.0 + 4.0 / 60.0 + 38.902495 / 3600.0) * DEG_TO_RAD
        + 4_399_609.33632 * t * SEC_TO_RAD;
    let ur = (-46.0 + 3.0 / 60.0 + 4.354234 / 3600.0) * DEG_TO_RAD
        + 1_542_482.57845 * t * SEC_TO_RAD;
    let ne = (-56.0 + 20.0 / 60.0 + 56.808371 / 3600.0) * DEG_TO_RAD
        + 786_547.897 * t * SEC_TO_RAD;

    Arguments { w1, d, f, l, lp, me, ve, em, ma, ju, sa, ur, ne, zeta }
}

/// Evaluate the Delaunay argument combination for a main-problem term.
#[inline(always)]
fn delaunay_arg(
    args: &Arguments,
    id: i8, i_f: i8, il: i8, ilp: i8,
) -> f64 {
    id as f64 * args.d
        + i_f as f64 * args.f
        + il as f64 * args.l
        + ilp as f64 * args.lp
}

/// Evaluate the full argument combination for a perturbation term.
#[inline(always)]
fn pert_arg(
    args: &Arguments,
    id: i8, i_f: i8, il: i8, ilp: i8,
    i_me: i8, i_ve: i8, i_em: i8, i_ma: i8,
    i_ju: i8, i_sa: i8, i_ur: i8, i_ne: i8,
    i_zeta: i8,
    phase: f64,
) -> f64 {
    phase
        + id as f64 * args.d
        + i_f as f64 * args.f
        + il as f64 * args.l
        + ilp as f64 * args.lp
        + i_me as f64 * args.me
        + i_ve as f64 * args.ve
        + i_em as f64 * args.em
        + i_ma as f64 * args.ma
        + i_ju as f64 * args.ju
        + i_sa as f64 * args.sa
        + i_ur as f64 * args.ur
        + i_ne as f64 * args.ne
        + i_zeta as f64 * args.zeta
}

/// Evaluate the ELP/MPP02 series at Julian centuries `t`.
///
/// Returns `(longitude_rad, latitude_rad, distance_km)` in ecliptic-of-date frame.
fn eval_series(t: f64, args: &Arguments) -> (f64, f64, f64) {
    let t2 = t * t;
    let t3 = t * t2;

    // ── Longitude ─────────────────────────────────────────────────────────
    let mut lon: f64 = 0.0;

    for &(id, i_f, il, ilp, amp) in moon_longitude::MAIN {
        let arg = delaunay_arg(args, id, i_f, il, ilp);
        lon += amp * libm::sin(arg);
    }

    let mut pert_t0: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_longitude::PERT_T0
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        pert_t0 += amp * libm::sin(arg);
    }
    lon += pert_t0;

    let mut pert_t1: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_longitude::PERT_T1
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        pert_t1 += amp * libm::sin(arg);
    }
    lon += pert_t1 * t;

    let mut pert_t2: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_longitude::PERT_T2
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        pert_t2 += amp * libm::sin(arg);
    }
    lon += pert_t2 * t2;

    let mut pert_t3: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_longitude::PERT_T3
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        pert_t3 += amp * libm::sin(arg);
    }
    lon += pert_t3 * t3;

    // ── Latitude ──────────────────────────────────────────────────────────
    let mut lat: f64 = 0.0;

    for &(id, i_f, il, ilp, amp) in moon_latitude::MAIN {
        let arg = delaunay_arg(args, id, i_f, il, ilp);
        lat += amp * libm::sin(arg);
    }

    let mut lat_t0: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_latitude::PERT_T0
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        lat_t0 += amp * libm::sin(arg);
    }
    lat += lat_t0;

    let mut lat_t1: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_latitude::PERT_T1
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        lat_t1 += amp * libm::sin(arg);
    }
    lat += lat_t1 * t;

    let mut lat_t2: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_latitude::PERT_T2
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        lat_t2 += amp * libm::sin(arg);
    }
    lat += lat_t2 * t2;

    // ── Distance ──────────────────────────────────────────────────────────
    // Main uses COSINE; perturbations use SINE.
    let mut dist: f64 = 0.0;

    for &(id, i_f, il, ilp, amp) in moon_distance::MAIN {
        let arg = delaunay_arg(args, id, i_f, il, ilp);
        dist += amp * libm::cos(arg);
    }

    let mut dist_t0: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_distance::PERT_T0
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        dist_t0 += amp * libm::sin(arg);
    }
    dist += dist_t0;

    let mut dist_t1: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_distance::PERT_T1
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        dist_t1 += amp * libm::sin(arg);
    }
    dist += dist_t1 * t;

    let mut dist_t2: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_distance::PERT_T2
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        dist_t2 += amp * libm::sin(arg);
    }
    dist += dist_t2 * t2;

    let mut dist_t3: f64 = 0.0;
    for &(id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, amp, phase) in
        moon_distance::PERT_T3
    {
        let arg = pert_arg(args, id, i_f, il, ilp, i_me, i_ve, i_em, i_ma, i_ju, i_sa, i_ur, i_ne, i_zeta, phase);
        dist_t3 += amp * libm::sin(arg);
    }
    dist += dist_t3 * t3;

    // Final coordinates in ecliptic-of-date
    let longitude = args.w1 + lon;
    let latitude = lat;
    let distance = RA0 * dist;

    (longitude, latitude, distance)
}

/// Apply the Chapront ecliptic precession from ecliptic-of-date to J2000 ecliptic.
///
/// Uses the P,Q precession parameters from Chapront (2002).
///
/// Input: rectangular ecliptic-of-date (km).
/// Output: rectangular J2000 ecliptic (km).
#[inline]
fn precess_to_j2000(t: f64, x0: f64, y0: f64, z0: f64) -> (f64, f64, f64) {
    let t2 = t * t;
    let t3 = t * t2;
    let t4 = t2 * t2;
    let t5 = t2 * t3;

    let p = 0.101_803_91e-4 * t + 0.470_204_39e-6 * t2 - 0.541_736_7e-9 * t3
        - 0.250_794_8e-11 * t4 + 0.463_486e-14 * t5;
    let q = -0.113_469_002e-3 * t + 0.123_726_74e-6 * t2 + 0.126_541_70e-8 * t3
        - 0.137_180_8e-11 * t4 - 0.320_334e-14 * t5;

    let sq = libm::sqrt(1.0 - p * p - q * q);

    let p11 = 1.0 - 2.0 * p * p;
    let p12 = 2.0 * p * q;
    let p13 = 2.0 * p * sq;
    let p21 = p12;
    let p22 = 1.0 - 2.0 * q * q;
    let p23 = -2.0 * q * sq;
    let p31 = -p13;
    let p32 = -p23;
    let p33 = 1.0 - 2.0 * p * p - 2.0 * q * q;

    (
        p11 * x0 + p12 * y0 + p13 * z0,
        p21 * x0 + p22 * y0 + p23 * z0,
        p31 * x0 + p32 * y0 + p33 * z0,
    )
}



/// Compute the Moon's geocentric rectangular coordinates in the **ecliptic of date**
/// (before precession to J2000). Used for osculating node computation, where the
/// node must be on the ecliptic of date — not the J2000 ecliptic.
///
/// Units: km (position), km/day (velocity).
pub fn elp_geocentric_of_date(jd: f64) -> MoonRectangular {
    let t = (jd - 2_451_545.0) / 36_525.0;

    let args = compute_arguments(t);
    let (longitude, latitude, distance) = eval_series(t, &args);

    let x = distance * libm::cos(latitude) * libm::cos(longitude);
    let y = distance * libm::cos(latitude) * libm::sin(longitude);
    let z = distance * libm::sin(latitude);

    // Velocity via numerical differentiation: ±0.5 day (ecliptic of date)
    let dt = 0.5_f64;
    let t_plus = (jd + dt - 2_451_545.0) / 36_525.0;
    let t_minus = (jd - dt - 2_451_545.0) / 36_525.0;

    let args_p = compute_arguments(t_plus);
    let (lon_p, lat_p, dist_p) = eval_series(t_plus, &args_p);
    let xp = dist_p * libm::cos(lat_p) * libm::cos(lon_p);
    let yp = dist_p * libm::cos(lat_p) * libm::sin(lon_p);
    let zp = dist_p * libm::sin(lat_p);

    let args_m = compute_arguments(t_minus);
    let (lon_m, lat_m, dist_m) = eval_series(t_minus, &args_m);
    let xm = dist_m * libm::cos(lat_m) * libm::cos(lon_m);
    let ym = dist_m * libm::cos(lat_m) * libm::sin(lon_m);
    let zm = dist_m * libm::sin(lat_m);

    let inv_2dt = 1.0 / (2.0 * dt);

    MoonRectangular {
        x, y, z,
        vx: (xp - xm) * inv_2dt,
        vy: (yp - ym) * inv_2dt,
        vz: (zp - zm) * inv_2dt,
    }
}

/// Compute Moon's geocentric rectangular coordinates in the **J2000 ecliptic**
/// (after precession from ecliptic of date). This is used for the coordinate
/// pipeline that produces apparent positions.
///
/// Units: km (position), km/day (velocity).
pub fn elp_geocentric(jd: f64) -> MoonRectangular {
    let t = (jd - 2_451_545.0) / 36_525.0;

    let args = compute_arguments(t);
    let (longitude, latitude, distance) = eval_series(t, &args);

    // Convert spherical to rectangular (ecliptic-of-date, km)
    let cos_lon = libm::cos(longitude);
    let sin_lon = libm::sin(longitude);
    let cos_lat = libm::cos(latitude);
    let sin_lat = libm::sin(latitude);

    let x0 = distance * cos_lat * cos_lon;
    let y0 = distance * cos_lat * sin_lon;
    let z0 = distance * sin_lat;

    // Precess from ecliptic-of-date to J2000 ecliptic
    let (x, y, z) = precess_to_j2000(t, x0, y0, z0);

    // Velocity via numerical differentiation: ±0.5 day
    let dt = 0.5_f64;
    let t_plus = (jd + dt - 2_451_545.0) / 36_525.0;
    let t_minus = (jd - dt - 2_451_545.0) / 36_525.0;

    let args_plus = compute_arguments(t_plus);
    let (lon_p, lat_p, dist_p) = eval_series(t_plus, &args_plus);
    let xp0 = dist_p * libm::cos(lat_p) * libm::cos(lon_p);
    let yp0 = dist_p * libm::cos(lat_p) * libm::sin(lon_p);
    let zp0 = dist_p * libm::sin(lat_p);
    let (xp, yp, zp) = precess_to_j2000(t_plus, xp0, yp0, zp0);

    let args_minus = compute_arguments(t_minus);
    let (lon_m, lat_m, dist_m) = eval_series(t_minus, &args_minus);
    let xm0 = dist_m * libm::cos(lat_m) * libm::cos(lon_m);
    let ym0 = dist_m * libm::cos(lat_m) * libm::sin(lon_m);
    let zm0 = dist_m * libm::sin(lat_m);
    let (xm, ym, zm) = precess_to_j2000(t_minus, xm0, ym0, zm0);

    let inv_2dt = 1.0 / (2.0 * dt);

    MoonRectangular {
        x,
        y,
        z,
        vx: (xp - xm) * inv_2dt,
        vy: (yp - ym) * inv_2dt,
        vz: (zp - zm) * inv_2dt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moon_distance_reasonable_at_j2000() {
        let result = elp_geocentric(2_451_545.0);
        let dist = libm::sqrt(result.x * result.x + result.y * result.y + result.z * result.z);
        // Moon ~356,000-406,000 km from Earth
        assert!(
            dist > 350_000.0 && dist < 410_000.0,
            "Moon distance should be 350-410k km, got {:.0}",
            dist
        );
    }

    #[test]
    fn moon_position_nonzero_at_j2000() {
        let result = elp_geocentric(2_451_545.0);
        assert!(
            result.x.abs() > 1.0 && result.y.abs() > 1.0,
            "Moon position should be nonzero: x={}, y={}, z={}",
            result.x, result.y, result.z,
        );
    }

    #[test]
    fn moon_velocity_nonzero_at_j2000() {
        let result = elp_geocentric(2_451_545.0);
        // Moon moves ~1 km/s = ~86400 km/day; should be nonzero
        let speed = libm::sqrt(result.vx * result.vx + result.vy * result.vy + result.vz * result.vz);
        assert!(
            speed > 10_000.0 && speed < 200_000.0,
            "Moon speed should be ~86000 km/day, got {speed:.0}"
        );
    }
}
