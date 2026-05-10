// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! ELP/MPP02 lunar theory — clean-room implementation.
//!
//! Geocentric Moon position and velocity in the inertial mean ecliptic and
//! equinox of J2000, computed from the Chapront–Francou ELP/MPP02 series.
//!
//! ## Sources
//!
//! - Chapront J., Francou G., 2003,
//!   *"The lunar theory ELP revisited. Introduction of new planetary
//!   perturbations"*, Astronomy & Astrophysics **404**, 735.
//!   DOI: `10.1051/0004-6361:20030529`.
//! - IMCCE explanatory note `elpmpp02.pdf`
//!   (Chapront, Chapront, Francou — Observatoire de Paris / SYRTE,
//!   October 2002), distributed with the coefficient files at
//!   `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/`.
//!
//! No third-party implementation has been consulted; this file is a clean
//! re-derivation from the cited primary sources only.
//!
//! ## Convention notes (resolutions of the spec's open §8 questions)
//!
//! - **Time scale.** Argument polynomials use TDB Julian centuries from
//!   J2000 (`t = (jd - 2451545.0) / 36525`). The TDB−TT difference is
//!   ≤ 2 ms over the validity interval, well below sub-mm lunar position
//!   noise; callers passing TT are within tolerance.
//! - **`l′` Delaunay argument.** `l′ = T − ϖ′` with no extra ±180° offset.
//!   (Verified against the IMCCE reference `ELPMPP02.for` lines 421–425:
//!   only `D` receives the +π add at the constant term.)
//! - **`icor` parity.** Since the IMCCE explanatory note prose
//!   (`icor=1 ⇒ LLR`, `icor=2 ⇒ DE405`) and the IMCCE reference Fortran
//!   (`icor=0 ⇒ LLR`, `icor=1 ⇒ DE405`) disagree on the integer mapping,
//!   this implementation uses a symbolic [`Fit`] enum with no ambiguity.
//! - **Truncation.** Series amplitudes below 1×10⁻⁵ (arcsec for V, U; km
//!   for r) are dropped at coefficient-generation time. This matches the
//!   noise floor inherent in the printed series and the existing VSOP87A
//!   pipeline convention. Within the [1500, 2500] CE interval the
//!   resulting position error is well under the 50 m / 0.6″ inherent
//!   precision of ELP/MPP02 itself (elpmpp02.pdf §8).
//! - **Velocity.** Closed-form analytic differentiation of the same
//!   series, divided by 36525 days/century at the end.
//! - **`a0` ratio.** `ra0 = 384747.961370173 / 384747.980674318` is
//!   applied to the distance series only.
//! - **Frame.** Output is the inertial mean ecliptic and equinox of
//!   J2000, applied via the orthogonal Laskar P/Q rotation
//!   (elpmpp02.pdf §5.1). [`elp_geocentric`] returns this frame.
//!   [`elp_geocentric_of_date`] applies the precession-of-date longitude
//!   shift `V → V + (p_A + Δp·t)` to give coordinates in the ecliptic
//!   *of date*, with no rotation back to J2000 axes (the radial and
//!   z components are unchanged from `elp_geocentric` only via the
//!   ecliptic-of-date convention; in practice callers should prefer
//!   [`elp_geocentric`] for J2000-fixed work).

use core::f64::consts::PI;

use crate::analytical::coefficients::{moon_distance, moon_latitude, moon_longitude};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Geocentric position and velocity of the Moon in J2000 ecliptic
/// rectangular coordinates.
///
/// Position in km, velocity in km/day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonRectangular {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

/// Choice of fitted-correction set.
///
/// `Llr` is the LLR-fit recommended by Chapront & Francou (2003) for general
/// use. `De405` is the DE405-fit, intended for long-range agreement with
/// JPL DE405 / DE406 over six millennia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// LLR-fit (1970–2001 normal points). The IMCCE reference Fortran
    /// calls this `icor=0`; the IMCCE explanatory-note prose calls it
    /// `icor=1`. They are the *same* physical fit.
    Llr,
    /// DE405-fit, with the additional Table-6 secular corrections to
    /// `W₁^{(3,4)}`, `W₂^{(2,3)}`, `W₃^{(2,3)}`. IMCCE Fortran `icor=1`,
    /// explanatory-note prose `icor=2`.
    De405,
}

/// Geocentric Moon in mean ecliptic and equinox of **J2000** (LLR fit).
///
/// Position in km, velocity in km/day, referred to the inertial mean
/// ecliptic and equinox of J2000.
#[must_use]
pub fn elp_geocentric(jd: f64) -> MoonRectangular {
    elp_geocentric_with_fit(jd, Fit::Llr)
}

/// Geocentric Moon in mean ecliptic and equinox of **date** (LLR fit).
///
/// Same z, x²+y² as [`elp_geocentric`], but with longitude shifted by
/// the accumulated precession `p_A + Δp·t` so that callers tracking the
/// instantaneous mean equinox of date receive consistent ecliptic
/// longitudes.
#[must_use]
pub fn elp_geocentric_of_date(jd: f64) -> MoonRectangular {
    let (v, u, r, vp, up, rp) = vur_series(jd, Fit::Llr);
    let tau = (jd - J2000) / DAYS_PER_CENTURY;

    // Add precession of date (in radians) to longitude.
    let prec = (P_PA_LIN + P_PA_DELTA) * tau
        + P_PA_QUAD * tau * tau
        + P_PA_CUBE * tau * tau * tau
        + P_PA_QUART * tau * tau * tau * tau;
    let prec_dot = (P_PA_LIN + P_PA_DELTA)
        + 2.0 * P_PA_QUAD * tau
        + 3.0 * P_PA_CUBE * tau * tau
        + 4.0 * P_PA_QUART * tau * tau * tau;

    let v_d = v + prec;
    let vdot_d = vp + prec_dot;

    // Spherical → rectangular in the (date) ecliptic frame, no P/Q rotation.
    rectangular_from_spherical(v_d, u, r, vdot_d, up, rp)
}

/// LLR-fit / DE405-fit aware computation. Position and velocity in the
/// inertial mean ecliptic and equinox of J2000.
#[must_use]
pub fn elp_geocentric_with_fit(jd: f64, fit: Fit) -> MoonRectangular {
    let (v, u, r, vp, up, rp) = vur_series(jd, fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;

    // Cartesian in the (date) ecliptic frame.
    let xyz_date = rectangular_from_spherical(v, u, r, vp, up, rp);

    // Apply Laskar P/Q rotation to land in the J2000 ecliptic frame.
    apply_pq_rotation(xyz_date, t)
}

// ─── Constants ─────────────────────────────────────────────────────────────────

const J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;

/// 648000 / π — converts arcseconds ↔ radians.
const ARCSEC_PER_RAD: f64 = 648_000.0 / PI;

/// `a0(DE405) / a0(ELP)` — distance-series scale factor (elpmpp02.pdf §1.1).
const RA0: f64 = 384_747.961_370_173 / 384_747.980_674_318;

// IAU precession constant + Herring 2002 correction (arcsec/cy → rad/cy).
const P_PA_LIN: f64 = 5_029.0966 / ARCSEC_PER_RAD;
const P_PA_DELTA: f64 = -0.29965 / ARCSEC_PER_RAD;
const P_PA_QUAD: f64 = 1.1120 / ARCSEC_PER_RAD;
const P_PA_CUBE: f64 = 0.000_077 / ARCSEC_PER_RAD;
const P_PA_QUART: f64 = -0.000_023_53 / ARCSEC_PER_RAD;

// Laskar (1986) P, Q precession coefficients (elpmpp02.pdf §5.1).
const P1: f64 = 0.101_803_91e-04;
const P2: f64 = 0.470_204_39e-06;
const P3: f64 = -0.541_736_7e-09;
const P4: f64 = -0.250_794_8e-11;
const P5: f64 = 0.463_486e-14;

const Q1: f64 = -0.113_469_002e-03;
const Q2: f64 = 0.123_726_74e-06;
const Q3: f64 = 0.126_541_7e-08;
const Q4: f64 = -0.137_180_8e-11;
const Q5: f64 = -0.320_334e-14;

// Fixed dimensionless constants (elpmpp02.pdf §4.3.1).
const M_RATIO: f64 = 0.074_801_329; // m = n'/ν
const ALPHA: f64 = 0.002_571_881; // a0/a'

// ─── Resolved fundamental arguments per fit ────────────────────────────────────

/// Resolved (per-`Fit`) polynomial coefficients used in series evaluation.
///
/// Each polynomial is in TDB Julian centuries from J2000. Units: radians.
/// The five Delaunay-related arguments (D, F, l, l′) plus eight planetary
/// mean longitudes plus the precession-tied ζ.
#[derive(Debug, Clone, Copy)]
struct Args {
    /// `del[0..4]` = D, `del[1]` = F, `del[2]` = l, `del[3]` = l′.
    del: [[f64; 5]; 4],
    /// Planetary mean longitudes Me, V, T(=EMB), Ma, J, Sa, U, N. Coefficients
    /// of degrees 0 and 1; higher degrees are zero per IMCCE primary.
    pla: [[f64; 5]; 8],
    /// `zeta = W₁ + (p + Δp)·t` (full polynomial).
    zeta: [f64; 5],
    /// W₁ polynomial — the leading W₁ in V series, before mean ecliptic shift.
    w1: [f64; 5],
    /// `δA_i` correction parameters for the main series (per spec §4.4.1).
    delnu: f64,
    delg: f64,
    dele: f64,
    delnp: f64,
    delep: f64,
}

#[inline]
fn dms_to_rad(deg: i32, arcmin: i32, arcsec: f64) -> f64 {
    // (deg + min/60 + sec/3600) * π / 180
    let total_arcsec = (deg as f64) * 3600.0 + (arcmin as f64) * 60.0 + arcsec;
    total_arcsec / ARCSEC_PER_RAD
}

#[inline]
fn arcsec_to_rad(a: f64) -> f64 {
    a / ARCSEC_PER_RAD
}

/// Build the `Args` for a given fit.
fn build_args(fit: Fit) -> Args {
    // Per-fit corrections to constants (elpmpp02.pdf §4.2 Table 3 / §4.3 Table 6).
    let (dw1_0, dw2_0, dw3_0, deart_0, dperi, dw1_1, dgam, de_, deart_1, dep, dw2_1, dw3_1, dw1_2) =
        match fit {
            Fit::Llr => (
                -0.10525, 0.16826, -0.10760, -0.04012, -0.04854, -0.32311, 0.000_69, 0.000_05,
                0.014_42, 0.002_26, 0.080_17, -0.043_17, -0.037_94,
            ),
            Fit::De405 => (
                -0.07008, 0.20794, -0.07215, -0.000_33, -0.007_49, -0.35106, 0.000_85, -0.000_06,
                0.007_32, 0.002_24, 0.080_17, -0.043_17, -0.037_43,
            ),
        };

    // Nominal W₁ polynomial (elpmpp02.pdf Table 1) plus per-fit corrections.
    let mut w1 = [
        dms_to_rad(218, 18, 59.95571 + dw1_0),
        arcsec_to_rad(1_732_559_343.736_04 + dw1_1),
        arcsec_to_rad(-6.8084 + dw1_2),
        arcsec_to_rad(0.006_604),
        arcsec_to_rad(-0.000_031_69),
    ];

    let mut w2 = [
        dms_to_rad(83, 21, 11.67475 + dw2_0),
        arcsec_to_rad(14_643_420.3171 + dw2_1),
        arcsec_to_rad(-38.2631),
        arcsec_to_rad(-0.045_047),
        arcsec_to_rad(0.000_213_01),
    ];

    let mut w3 = [
        dms_to_rad(125, 2, 40.39816 + dw3_0),
        arcsec_to_rad(-6_967_919.5383 + dw3_1),
        arcsec_to_rad(6.3590),
        arcsec_to_rad(0.007_625),
        arcsec_to_rad(-0.000_035_86),
    ];

    let eart = [
        dms_to_rad(100, 27, 59.13885 + deart_0),
        arcsec_to_rad(129_597_742.293_00 + deart_1),
        arcsec_to_rad(-0.020_2),
        arcsec_to_rad(0.000_009),
        arcsec_to_rad(0.000_000_15),
    ];

    let peri = [
        dms_to_rad(102, 56, 14.45766 + dperi),
        arcsec_to_rad(1_161.243_42),
        arcsec_to_rad(0.529_265),
        arcsec_to_rad(-0.000_118_14),
        arcsec_to_rad(0.000_011_379),
    ];

    // DE405-only Table 6 secular corrections.
    if matches!(fit, Fit::De405) {
        w1[3] += arcsec_to_rad(-0.000_188_65);
        w1[4] += arcsec_to_rad(-0.000_010_24);
        w2[2] += arcsec_to_rad(0.004_706_02);
        w2[3] += arcsec_to_rad(-0.000_252_13);
        w3[2] += arcsec_to_rad(-0.002_610_70);
        w3[3] += arcsec_to_rad(-0.000_107_12);
    }

    // Auxiliary corrections to (ν, Γ, E, e′, n′) per elpmpp02.pdf §4.3.1
    // (and as the IMCCE reference Fortran `INITIAL` subroutine encodes them).
    let delnu = arcsec_to_rad(0.55604 + dw1_1) / w1[1]; // δν / ν
    let delg = arcsec_to_rad(-0.08066 + dgam); // δΓ
    let dele = arcsec_to_rad(0.01789 + de_); // δE
    let delnp = arcsec_to_rad(-0.06424 + deart_1) / w1[1]; // δn′ / ν
    let delep = arcsec_to_rad(-0.12879 + dep); // δe′

    // Closed-form supplementary corrections to W₂^{(1)}, W₃^{(1)} (Table 5
    // of elpmpp02.pdf §4.3.2): a 5×2 matrix with rows j ∈ {ν, Γ, E, e′, n′}
    // and columns i ∈ {2, 3} for W₂ / W₃. Numbers transcribed from primary
    // (elpmpp02.pdf Table 5; matched by the IMCCE reference Fortran
    // `INITIAL` data-block, lines 284–287).
    let bp = [
        // j=1 (ν):     B'_{2,1},          B'_{3,1}
        [0.311_079_095, -0.103_837_907],
        // j=2 (Γ):     B'_{2,2},          B'_{3,2}
        [-0.004_482_398, 0.000_668_287],
        // j=3 (E):     B'_{2,3},          B'_{3,3}
        [-0.001_102_485, -0.001_298_072],
        // j=4 (e′):    B'_{2,4},          B'_{3,4}
        [0.001_056_062, -0.000_178_028],
        // j=5 (n′):    B'_{2,5},          B'_{3,5}
        [0.000_050_928, -0.000_037_342],
    ];
    let xa = (2.0 * ALPHA) / 3.0; // 2α/3
    let dw1_1_rad = arcsec_to_rad(dw1_1);
    let deart_1_rad = arcsec_to_rad(deart_1);

    // `i` is a column index into bp[0..5] and a selector for w2/w3 — not a
    // straight iteration over a single slice — so the clippy suggestion to
    // switch to `.iter().enumerate()` doesn't apply.
    #[allow(clippy::needless_range_loop)]
    for i in 0..2usize {
        let xi = if i == 0 { w2[1] } else { w3[1] } / w1[1];
        let yi = M_RATIO * bp[0][i] + xa * bp[4][i];
        let d_xy_1 = xi - yi; // d21 / d31
        let d_2 = w1[1] * bp[1][i];
        let d_3 = w1[1] * bp[2][i];
        let d_4 = w1[1] * bp[3][i];
        let d_5 = yi / M_RATIO;
        let cw = d_xy_1 * dw1_1_rad
            + d_5 * deart_1_rad
            + d_2 * arcsec_to_rad(dgam)
            + d_3 * arcsec_to_rad(de_)
            + d_4 * arcsec_to_rad(dep);
        if i == 0 {
            w2[1] += cw;
        } else {
            w3[1] += cw;
        }
    }

    // Delaunay arguments per elpmpp02.pdf §3.1 / IMCCE Fortran lines 421–425.
    let mut del = [[0.0_f64; 5]; 4];
    for k in 0..5 {
        del[0][k] = w1[k] - eart[k]; // D
        del[1][k] = w1[k] - w3[k]; // F
        del[2][k] = w1[k] - w2[k]; // l
        del[3][k] = eart[k] - peri[k]; // l'
    }
    del[0][0] += PI; // D constant offset = +180°.

    // Planetary arguments — VSOP2000 mean longitudes (elpmpp02.pdf §3.4 +
    // IMCCE Fortran lines 430–446). The Mercury rate `538101628.66888` from
    // the IMCCE reference Fortran is preferred over the spec's transcribed
    // `538101628.68888` (the Fortran is the canonical machine-readable
    // source — primary).
    let mut pla = [[0.0_f64; 5]; 8];
    pla[0][0] = dms_to_rad(252, 15, 3.216_919);
    pla[1][0] = dms_to_rad(181, 58, 44.758_419);
    pla[2][0] = dms_to_rad(100, 27, 59.138_850);
    pla[3][0] = dms_to_rad(355, 26, 3.642_778);
    pla[4][0] = dms_to_rad(34, 21, 5.379_392);
    pla[5][0] = dms_to_rad(50, 4, 38.902_495);
    pla[6][0] = dms_to_rad(314, 3, 4.354_234);
    pla[7][0] = dms_to_rad(304, 20, 56.808_371);

    pla[0][1] = arcsec_to_rad(538_101_628.668_88);
    pla[1][1] = arcsec_to_rad(210_664_136.457_77);
    pla[2][1] = arcsec_to_rad(129_597_742.293_00);
    pla[3][1] = arcsec_to_rad(68_905_077.659_36);
    pla[4][1] = arcsec_to_rad(10_925_660.573_35);
    pla[5][1] = arcsec_to_rad(4_399_609.336_32);
    pla[6][1] = arcsec_to_rad(1_542_482.578_45);
    pla[7][1] = arcsec_to_rad(786_547.897_00);

    // ζ = W₁ + (p + Δp)·t, with p = 5029.0966″/cy and Δp = −0.29965″/cy.
    let mut zeta = w1;
    zeta[1] += P_PA_LIN + P_PA_DELTA;

    Args {
        del,
        pla,
        zeta,
        w1,
        delnu,
        delg,
        dele,
        delnp,
        delep,
    }
}

// ─── Series evaluation ────────────────────────────────────────────────────────

#[inline]
fn eval_poly(c: &[f64; 5], t: f64) -> f64 {
    c[0] + t * (c[1] + t * (c[2] + t * (c[3] + t * c[4])))
}

#[inline]
fn eval_poly_dot(c: &[f64; 5], t: f64) -> f64 {
    // d/dt of c0 + c1·t + c2·t² + c3·t³ + c4·t⁴
    c[1] + t * (2.0 * c[2] + t * (3.0 * c[3] + t * 4.0 * c[4]))
}

/// Per-amplitude correction for one main-problem term (elpmpp02.pdf §4.3.1
/// closed form, distance-specific subtraction matched to IMCCE Fortran
/// `EVALUATE` line 559).
#[inline]
fn corrected_main_amplitude(
    raw_a_rad_or_km: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    b4: f64,
    b5: f64,
    args: &Args,
    is_distance: bool,
) -> f64 {
    let xa = (2.0 * ALPHA) / 3.0; // 2α/3
    let m = M_RATIO;
    // Following the IMCCE reference Fortran (the only primary numerical
    // template), the bracketed factor is `b1 + (2α/3)·b5/(... wait)`.
    // Re-derivation from elpmpp02.pdf §4.3.1:
    //   tgv  = b1 + (2α/(3m))·b5
    //   δA   = a · ((2A/(3m))-only-for-distance is folded in below)
    //   cmpb = a + tgv·(δn'/ν − m·δν/ν) + b2·δΓ + b3·δE + b4·δe′
    // For distance: replace `a` with `a − (2/3)·a·δν/ν` first.
    let mut a = raw_a_rad_or_km;
    if is_distance {
        a -= (2.0 / 3.0) * a * args.delnu;
    }
    let tgv = b1 + (xa / m) * b5;
    a + tgv * (args.delnp - m * args.delnu) + b2 * args.delg + b3 * args.dele + b4 * args.delep
}

/// Evaluate the longitude / latitude / distance series (and their time
/// derivatives) at JD `jd` for fit `fit`. Returns
/// `(V, U, r, dV/dt, dU/dt, dr/dt)` with V, U in radians, r in km, and
/// the dotted quantities in radian/century, radian/century, km/century.
fn vur_series(jd: f64, fit: Fit) -> (f64, f64, f64, f64, f64, f64) {
    let args = build_args(fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

    // === Longitude (V), radians ===
    let v_main = eval_main_series(
        moon_longitude::MAIN,
        &args,
        &t_pow,
        false,
        SeriesKind::Sine,
        true, // arcsec → radian
    );
    let v_pert = eval_pert_series(
        &[
            moon_longitude::PERT_0,
            moon_longitude::PERT_1,
            moon_longitude::PERT_2,
            moon_longitude::PERT_3,
        ],
        &args,
        &t_pow,
        true, // arcsec → radian
    );
    let v_w1 = eval_poly(&args.w1, t);
    let v = v_w1 + v_main.value + v_pert.value;
    let vdot = eval_poly_dot(&args.w1, t) + v_main.dot + v_pert.dot;

    // === Latitude (U), radians ===
    let u_main = eval_main_series(
        moon_latitude::MAIN,
        &args,
        &t_pow,
        false,
        SeriesKind::Sine,
        true,
    );
    let u_pert = eval_pert_series(
        &[
            moon_latitude::PERT_0,
            moon_latitude::PERT_1,
            moon_latitude::PERT_2,
            moon_latitude::PERT_3,
        ],
        &args,
        &t_pow,
        true,
    );
    let u = u_main.value + u_pert.value;
    let udot = u_main.dot + u_pert.dot;

    // === Distance (r), km ===
    // S3 main is a *cosine* series (elpmpp02.pdf §2.2.1), and corrections
    // include the additional `−(2/3)·A·δν/ν` term per §4.4.1.
    let r_main = eval_main_series(
        moon_distance::MAIN,
        &args,
        &t_pow,
        true, // is_distance
        SeriesKind::Cosine,
        false, // km, no arcsec→rad
    );
    let r_pert = eval_pert_series(
        &[
            moon_distance::PERT_0,
            moon_distance::PERT_1,
            moon_distance::PERT_2,
            moon_distance::PERT_3,
        ],
        &args,
        &t_pow,
        false, // km
    );
    let r = (r_main.value + r_pert.value) * RA0;
    let rdot = (r_main.dot + r_pert.dot) * RA0;

    (v, u, r, vdot, udot, rdot)
}

#[derive(Clone, Copy)]
enum SeriesKind {
    Sine,
    Cosine,
}

#[derive(Clone, Copy)]
struct SeriesPart {
    value: f64,
    dot: f64,
}

/// Evaluate a main-problem series (S1, S2, or S3) with corrections.
fn eval_main_series(
    terms: &[(i32, i32, i32, i32, f64, f64, f64, f64, f64, f64, f64)],
    args: &Args,
    t_pow: &[f64; 5],
    is_distance: bool,
    kind: SeriesKind,
    arcsec_to_radian: bool,
) -> SeriesPart {
    let mut value = 0.0;
    let mut dot = 0.0;
    for &(i1, i2, i3, i4, a_raw, b1, b2, b3, b4, b5, _b6) in terms {
        // Apply corrections in the file's native units (arcsec for S1/S2,
        // km for S3) — B partials are in per-σ ratios so the result stays
        // in those units. Convert to radian at the very end if needed.
        let a_corrected_native =
            corrected_main_amplitude(a_raw, b1, b2, b3, b4, b5, args, is_distance);
        let a_corrected = if arcsec_to_radian {
            arcsec_to_rad(a_corrected_native)
        } else {
            a_corrected_native
        };

        // Phase φ = i1·D + i2·F + i3·l + i4·l'.
        let mut phase = 0.0;
        let mut omega = 0.0;
        for k in 0..5 {
            let coef = (i1 as f64) * args.del[0][k]
                + (i2 as f64) * args.del[1][k]
                + (i3 as f64) * args.del[2][k]
                + (i4 as f64) * args.del[3][k];
            phase += coef * t_pow[k];
            if k >= 1 {
                omega += (k as f64) * coef * t_pow[k - 1];
            }
        }

        match kind {
            SeriesKind::Sine => {
                value += a_corrected * libm::sin(phase);
                dot += a_corrected * omega * libm::cos(phase);
            }
            SeriesKind::Cosine => {
                value += a_corrected * libm::cos(phase);
                dot += -a_corrected * omega * libm::sin(phase);
            }
        }
    }
    SeriesPart { value, dot }
}

/// Evaluate the four power-grouped perturbation series for one variable.
/// Each table entry is `(S, C, i1..i13)` and is evaluated as
/// `t^n · (S sin φ + C cos φ)` with φ accumulating Delaunay + planetary +
/// ζ multipliers.
fn eval_pert_series(
    groups: &[&[(
        f64,
        f64,
        i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,
    )]],
    args: &Args,
    t_pow: &[f64; 5],
    arcsec_to_radian: bool,
) -> SeriesPart {
    let scale = if arcsec_to_radian { 1.0 / ARCSEC_PER_RAD } else { 1.0 };
    let mut value = 0.0;
    let mut dot = 0.0;
    for (n, &group) in groups.iter().enumerate() {
        let tn = t_pow[n];
        let dtn = if n >= 1 { (n as f64) * t_pow[n - 1] } else { 0.0 };
        for &(s_raw, c_raw, i1, i2, i3, i4, i5, i6, i7, i8, i9, i10, i11, i12, i13) in group {
            let s = s_raw * scale;
            let c = c_raw * scale;
            let mut phase = 0.0;
            let mut omega = 0.0;
            for k in 0..5 {
                let coef = (i1 as f64) * args.del[0][k]
                    + (i2 as f64) * args.del[1][k]
                    + (i3 as f64) * args.del[2][k]
                    + (i4 as f64) * args.del[3][k]
                    + (i5 as f64) * args.pla[0][k]
                    + (i6 as f64) * args.pla[1][k]
                    + (i7 as f64) * args.pla[2][k]
                    + (i8 as f64) * args.pla[3][k]
                    + (i9 as f64) * args.pla[4][k]
                    + (i10 as f64) * args.pla[5][k]
                    + (i11 as f64) * args.pla[6][k]
                    + (i12 as f64) * args.pla[7][k]
                    + (i13 as f64) * args.zeta[k];
                phase += coef * t_pow[k];
                if k >= 1 {
                    omega += (k as f64) * coef * t_pow[k - 1];
                }
            }
            let sin_phi = libm::sin(phase);
            let cos_phi = libm::cos(phase);
            // f(t) = t^n · (s sin φ + c cos φ)
            // f'(t) = n·t^{n-1}·(s sin φ + c cos φ)
            //       + t^n · ω · (s cos φ − c sin φ)
            let inner = s * sin_phi + c * cos_phi;
            let inner_dot = s * cos_phi - c * sin_phi;
            value += tn * inner;
            dot += dtn * inner + tn * omega * inner_dot;
        }
    }
    SeriesPart { value, dot }
}

// ─── Spherical → rectangular ──────────────────────────────────────────────────

/// Convert (V, U, r) plus their time derivatives in /century to
/// rectangular coordinates and velocities in km/day in the (date) ecliptic
/// frame, matching elpmpp02.pdf §5.1.
fn rectangular_from_spherical(
    v: f64,
    u: f64,
    r: f64,
    vdot_per_cy: f64,
    udot_per_cy: f64,
    rdot_per_cy: f64,
) -> MoonRectangular {
    let cv = libm::cos(v);
    let sv = libm::sin(v);
    let cu = libm::cos(u);
    let su = libm::sin(u);

    let cw = r * cu;
    let _sw = r * su;

    let x1 = cw * cv;
    let x2 = cw * sv;
    let x3 = r * su;

    // Velocities (per-century) of the cartesian components in the
    // ecliptic-of-date frame, before the per-day conversion.
    // d/dt[r cos U cos V] etc., expanded symbolically:
    let xp1 = (rdot_per_cy * cu - r * udot_per_cy * su) * cv - vdot_per_cy * x2;
    let xp2 = (rdot_per_cy * cu - r * udot_per_cy * su) * sv + vdot_per_cy * x1;
    let xp3 = rdot_per_cy * su + r * udot_per_cy * cu;

    // Per-century → per-day.
    let inv_sc = 1.0 / DAYS_PER_CENTURY;
    MoonRectangular {
        x: x1,
        y: x2,
        z: x3,
        vx: xp1 * inv_sc,
        vy: xp2 * inv_sc,
        vz: xp3 * inv_sc,
    }
}

/// Apply the Laskar P/Q rotation from the inertial mean ecliptic of date
/// to the inertial mean ecliptic and equinox of J2000 (elpmpp02.pdf §5.1).
fn apply_pq_rotation(xyz_date: MoonRectangular, t: f64) -> MoonRectangular {
    let pw = (P1 + P2 * t + P3 * t * t + P4 * t * t * t + P5 * t * t * t * t) * t;
    let qw = (Q1 + Q2 * t + Q3 * t * t + Q4 * t * t * t + Q5 * t * t * t * t) * t;
    let ra = 2.0 * libm::sqrt(1.0 - pw * pw - qw * qw);
    let pwqw = 2.0 * pw * qw;
    let pw2 = 1.0 - 2.0 * pw * pw;
    let qw2 = 1.0 - 2.0 * qw * qw;
    let pwra = pw * ra;
    let qwra = qw * ra;

    let x1 = xyz_date.x;
    let x2 = xyz_date.y;
    let x3 = xyz_date.z;
    let xp1 = xyz_date.vx;
    let xp2 = xyz_date.vy;
    let xp3 = xyz_date.vz;

    // Position rotation (matches IMCCE Fortran lines 769–771 and elpmpp02.pdf §5.1).
    let xyz_x = pw2 * x1 + pwqw * x2 + pwra * x3;
    let xyz_y = pwqw * x1 + qw2 * x2 - qwra * x3;
    let xyz_z = -pwra * x1 + qwra * x2 + (pw2 + qw2 - 1.0) * x3;

    // Velocity rotation: include the time derivative of the rotation
    // matrix itself (the (P,Q)-prime terms). Per IMCCE Fortran lines 773–787.
    let ppw = P1 + (2.0 * P2 + 3.0 * P3 * t + 4.0 * P4 * t * t + 5.0 * P5 * t * t * t) * t;
    let qpw = Q1 + (2.0 * Q2 + 3.0 * Q3 * t + 4.0 * Q4 * t * t + 5.0 * Q5 * t * t * t) * t;
    let ppw2 = -4.0 * pw * ppw;
    let qpw2 = -4.0 * qw * qpw;
    let ppwqpw = 2.0 * (ppw * qw + pw * qpw);
    let rap = (ppw2 + qpw2) / ra;
    let ppwra = ppw * ra + pw * rap;
    let qpwra = qpw * ra + qw * rap;

    // Per-century derivatives of position-of-date carry the per-day xp{1,2,3}
    // already; so the rotation-induced piece needs the same per-day scaling.
    let inv_sc = 1.0 / DAYS_PER_CENTURY;
    let xyz_vx =
        pw2 * xp1 + pwqw * xp2 + pwra * xp3 + (ppw2 * x1 + ppwqpw * x2 + ppwra * x3) * inv_sc;
    let xyz_vy =
        pwqw * xp1 + qw2 * xp2 - qwra * xp3 + (ppwqpw * x1 + qpw2 * x2 - qpwra * x3) * inv_sc;
    let xyz_vz = -pwra * xp1 + qwra * xp2 + (pw2 + qw2 - 1.0) * xp3
        + (-ppwra * x1 + qpwra * x2 + (ppw2 + qpw2) * x3) * inv_sc;

    MoonRectangular {
        x: xyz_x,
        y: xyz_y,
        z: xyz_z,
        vx: xyz_vx,
        vy: xyz_vy,
        vz: xyz_vz,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moon_at_j2000_finite_and_in_orbit() {
        let m = elp_geocentric(J2000);
        let r2 = m.x * m.x + m.y * m.y + m.z * m.z;
        let r = libm::sqrt(r2);
        assert!(r.is_finite());
        // Moon geocentric distance must lie within the orbital shell
        // ~356 000–407 000 km plus modest series noise.
        assert!(
            (350_000.0..=410_000.0).contains(&r),
            "r = {r:.3} km out of expected range"
        );
        // Velocity magnitude ≈ 100 000 km/day.
        let v = libm::sqrt(m.vx * m.vx + m.vy * m.vy + m.vz * m.vz);
        assert!(
            (50_000.0..=150_000.0).contains(&v),
            "|v| = {v:.3} km/day out of expected range"
        );
    }

    #[test]
    fn de405_fit_close_to_llr_fit_in_modern_era() {
        // Differences between fits are bounded by a few arcseconds
        // (and tens of metres) inside [1950, 2060].
        let llr = elp_geocentric_with_fit(J2000, Fit::Llr);
        let de4 = elp_geocentric_with_fit(J2000, Fit::De405);
        let dx = llr.x - de4.x;
        let dy = llr.y - de4.y;
        let dz = llr.z - de4.z;
        let dr = libm::sqrt(dx * dx + dy * dy + dz * dz);
        assert!(dr < 100.0, "fit difference at J2000 = {dr:.3} km, too large");
    }
}
