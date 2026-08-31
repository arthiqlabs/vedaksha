// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

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
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

use wide::f64x4;

use crate::analytical::coefficients::loader::{ElpMainTerm, ElpPertTerm};
use crate::analytical::coefficients::{moon_distance, moon_latitude, moon_longitude};
use crate::analytical::simd_trig::sincos_f64x4;

/// Count of ELP/MPP02 evaluations performed by this process.
///
/// Instrumentation, not part of the numerical API — exists so regression
/// guards (`tests/chart_lunar_evals.rs`) can count *every* real ELP/MPP02
/// evaluation. Incremented inside [`vur_series`] and its position-only
/// sibling [`vur_series_position`] — together the two functions every public
/// entry point ([`elp_geocentric`], [`elp_geocentric_of_date`],
/// [`elp_geocentric_with_fit`], [`elp_geocentric_position`],
/// [`elp_geocentric_position_of_date`]) shares — so it counts evaluations
/// invisible to a trait-level wrapper around
/// [`crate::jpl::EphemerisProvider::compute_state`] —
/// `AnalyticalProvider::compute_state(EarthMoonBarycenter, jd)` calls
/// `moon_state(jd)` -> [`elp_geocentric`] internally, never passing through
/// `compute_state(Body::Moon, ·)` (see
/// `docs/audit/2026-08-29-perf-investigation.md` #1) — and evaluations that
/// never go through `compute_state` at all, such as the true node's
/// osculating computation via [`elp_geocentric_of_date`]. An earlier version
/// of this counter lived at the [`elp_geocentric`] call site instead and
/// missed both `elp_geocentric_of_date` and `elp_geocentric_with_fit`
/// entirely.
///
/// Always compiled rather than `#[cfg(test)]`-gated: an integration test
/// under `tests/` links against the library built as an ordinary dependency,
/// not with the crate's own `cfg(test)`, so a `cfg(test)` counter would be
/// invisible to it. The cost is one relaxed atomic increment per evaluation
/// — negligible next to the ~35,758 series terms each evaluation performs —
/// and it has no effect on any computed value.
#[doc(hidden)]
pub static ELP_GEOCENTRIC_CALLS: AtomicU64 = AtomicU64::new(0);

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

/// Geocentric Moon **position only** (no velocity) in mean ecliptic and
/// equinox of **J2000** (LLR fit).
///
/// Position in km, in the inertial mean ecliptic and equinox of J2000 — the
/// same frame and units as [`elp_geocentric`]'s position fields. Every value
/// this returns is bit-for-bit identical to the corresponding
/// `(elp_geocentric(jd).x, .y, .z)`; see the `elp_geocentric_position_*`
/// tests below. It exists because two production callers
/// ([`crate::coordinates::retarded_geocentric`] for the Moon body, and
/// [`crate::nodes::osculating_node`]) only ever read the position half of a
/// lunar state, so computing the velocity/omega half for them is pure waste
/// (`docs/audit/2026-08-29-perf-investigation.md` #5).
#[must_use]
pub fn elp_geocentric_position(jd: f64) -> (f64, f64, f64) {
    let (v, u, r) = vur_series_position(jd, Fit::Llr);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let xyz_date = rectangular_from_spherical_position(v, u, r);
    apply_pq_rotation_position(xyz_date, t)
}

/// Geocentric Moon **position only** (no velocity) in mean ecliptic and
/// equinox of **date** (LLR fit).
///
/// Position-only counterpart of [`elp_geocentric_of_date`]; see
/// [`elp_geocentric_position`] for why this exists. Bit-for-bit identical to
/// `(elp_geocentric_of_date(jd).x, .y, .z)`.
#[must_use]
pub fn elp_geocentric_position_of_date(jd: f64) -> (f64, f64, f64) {
    let (v, u, r) = vur_series_position(jd, Fit::Llr);
    let tau = (jd - J2000) / DAYS_PER_CENTURY;

    // Add precession of date (in radians) to longitude — same formula as
    // elp_geocentric_of_date, position terms only.
    let prec = (P_PA_LIN + P_PA_DELTA) * tau
        + P_PA_QUAD * tau * tau
        + P_PA_CUBE * tau * tau * tau
        + P_PA_QUART * tau * tau * tau * tau;

    let v_d = v + prec;

    // Spherical → rectangular in the (date) ecliptic frame, no P/Q rotation.
    rectangular_from_spherical_position(v_d, u, r)
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

/// The two `Args` sets, built once each on first use.
///
/// [`build_args`] depends only on [`Fit`] — no `jd`, no series data — but was
/// re-run on every [`elp_geocentric`] call until 2026-08-29
/// (`docs/audit/2026-08-29-perf-investigation.md` #4). There are exactly two
/// distinct results and they are constants of the theory, so they are computed
/// once and shared.
///
/// Bit-identity is immediate: [`build_args`] is a pure function of `fit` over
/// `f64` arithmetic with no ambient state, so the memoized value is the same
/// bit pattern the per-call construction produced. Nothing is reordered.
static ARGS_LLR: LazyLock<Args> = LazyLock::new(|| build_args(Fit::Llr));
static ARGS_DE405: LazyLock<Args> = LazyLock::new(|| build_args(Fit::De405));

/// The memoized [`Args`] for `fit`.
#[inline]
fn args_for(fit: Fit) -> &'static Args {
    match fit {
        Fit::Llr => &ARGS_LLR,
        Fit::De405 => &ARGS_DE405,
    }
}

/// Build the `Args` for a given fit.
///
/// Called only from [`ARGS_LLR`] / [`ARGS_DE405`] in production; production
/// code paths should use [`args_for`].
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
    ELP_GEOCENTRIC_CALLS.fetch_add(1, Ordering::Relaxed);
    let args = args_for(fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

    // The 13 argument polynomials are reduced once here and shared by all six
    // series evaluations below; see `reduce_all_args`.
    let (a_arg, adot_arg) = reduce_all_args(args, &t_pow);

    // === Longitude (V), radians ===
    let v_main = eval_main_series(
        moon_longitude::MAIN.as_slice(),
        V_MAIN_AMPS.get(fit),
        &a_arg,
        &adot_arg,
        SeriesKind::Sine,
    );
    let v_pert = eval_pert_series(&V_PERT_SPARSE, &a_arg, &adot_arg, &t_pow);
    let v_w1 = eval_poly(&args.w1, t);
    let v = v_w1 + v_main.value + v_pert.value;
    let vdot = eval_poly_dot(&args.w1, t) + v_main.dot + v_pert.dot;

    // === Latitude (U), radians ===
    let u_main = eval_main_series(
        moon_latitude::MAIN.as_slice(),
        U_MAIN_AMPS.get(fit),
        &a_arg,
        &adot_arg,
        SeriesKind::Sine,
    );
    let u_pert = eval_pert_series(&U_PERT_SPARSE, &a_arg, &adot_arg, &t_pow);
    let u = u_main.value + u_pert.value;
    let udot = u_main.dot + u_pert.dot;

    // === Distance (r), km ===
    // S3 main is a *cosine* series (elpmpp02.pdf §2.2.1), and corrections
    // include the additional `−(2/3)·A·δν/ν` term per §4.4.1.
    let r_main = eval_main_series(
        moon_distance::MAIN.as_slice(),
        R_MAIN_AMPS.get(fit),
        &a_arg,
        &adot_arg,
        SeriesKind::Cosine,
    );
    let r_pert = eval_pert_series(&R_PERT_SPARSE, &a_arg, &adot_arg, &t_pow);
    let r = (r_main.value + r_pert.value) * RA0;
    let rdot = (r_main.dot + r_pert.dot) * RA0;

    (v, u, r, vdot, udot, rdot)
}

/// Position-only sibling of [`vur_series`]: same series, same terms, same
/// accumulation order, but never computes `omega`/`adot_arg`/`dot` at all.
///
/// Returns `(V, U, r)` — V, U in radians, r in km — bit-for-bit identical to
/// the first three elements of [`vur_series`]'s tuple at the same `(jd,
/// fit)`. See `tests::elp_geocentric_position_matches_elp_geocentric_bit_for_bit`
/// and `tests::elp_geocentric_position_of_date_matches_elp_geocentric_of_date_bit_for_bit`
/// for the end-to-end proof (there is no direct per-function test of
/// `vur_series_position` alone; those two tests exercise it through both
/// public entry points).
fn vur_series_position(jd: f64, fit: Fit) -> (f64, f64, f64) {
    ELP_GEOCENTRIC_CALLS.fetch_add(1, Ordering::Relaxed);
    let args = args_for(fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

    let a_arg = reduce_all_args_position(args, &t_pow);

    // === Longitude (V), radians ===
    let v_main = eval_main_series_position(
        moon_longitude::MAIN.as_slice(),
        V_MAIN_AMPS.get(fit),
        &a_arg,
        SeriesKind::Sine,
    );
    let v_pert = eval_pert_series_position(&V_PERT_SPARSE, &a_arg, &t_pow);
    let v_w1 = eval_poly(&args.w1, t);
    let v = v_w1 + v_main + v_pert;

    // === Latitude (U), radians ===
    let u_main = eval_main_series_position(
        moon_latitude::MAIN.as_slice(),
        U_MAIN_AMPS.get(fit),
        &a_arg,
        SeriesKind::Sine,
    );
    let u_pert = eval_pert_series_position(&U_PERT_SPARSE, &a_arg, &t_pow);
    let u = u_main + u_pert;

    // === Distance (r), km ===
    let r_main = eval_main_series_position(
        moon_distance::MAIN.as_slice(),
        R_MAIN_AMPS.get(fit),
        &a_arg,
        SeriesKind::Cosine,
    );
    let r_pert = eval_pert_series_position(&R_PERT_SPARSE, &a_arg, &t_pow);
    let r = (r_main + r_pert) * RA0;

    (v, u, r)
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

/// Reduce one degree-4 argument polynomial to its value `A = Σ_k c[k]·t^k`
/// and time derivative `Ȧ = Σ_{k≥1} k·c[k]·t^{k-1}` at the current epoch.
///
/// Each series term's phase is then a dot product of its integer multipliers
/// with these per-argument values, instead of recomputing the inner
/// `Σ_k Σ_j` sum for every term — far fewer operations per term. (This
/// reassociates the phase sum versus the term-by-term form, so results differ
/// at the ULP level; validated against the oracle / paper-example nets.)
#[inline]
fn reduce_arg(coeffs: &[f64; 5], t_pow: &[f64; 5]) -> (f64, f64) {
    let mut a = 0.0;
    let mut adot = 0.0;
    for k in 0..5 {
        a += coeffs[k] * t_pow[k];
        if k >= 1 {
            adot += (k as f64) * coeffs[k] * t_pow[k - 1];
        }
    }
    (a, adot)
}

/// Number of distinct arguments a perturbation phase can reference: the four
/// Delaunay arguments D, F, l, l′, then the eight planetary mean longitudes
/// Me…N, then ζ.
const N_ARGS: usize = 13;

/// Reduce all 13 argument polynomials to `(value, derivative)` at one epoch.
///
/// Hoisted out of the series evaluators on 2026-08-29
/// (`docs/audit/2026-08-29-perf-investigation.md` #4). `eval_main_series` used
/// to reduce the 4 Delaunay arguments and `eval_pert_series` all 13, each of
/// them three times per [`vur_series`] call (once for V, once for U, once for
/// r) — 51 `reduce_arg` calls per evaluation where 13 suffice, all of them on
/// the same `args` and the same `t_pow`.
///
/// Bit-identity: [`reduce_arg`] is deterministic in `(coeffs, t_pow)`, and the
/// hoist changes only *how many times* it runs, never its inputs, its internal
/// summation order, or the order the results are consumed in. The first four
/// entries are exactly what `eval_main_series` used to build for itself, in
/// the same index order.
///
/// The arrays are [`N_SLOTS`] (16) wide rather than [`N_ARGS`] (13): entries
/// 13..16 stay `0.0` and are what the sparse index's padding slots point at
/// (see [`PAD_ARG`]). A power-of-two width is also what lets the inner loop's
/// `& 15` mask eliminate its bounds check.
fn reduce_all_args(args: &Args, t_pow: &[f64; 5]) -> ([f64; N_SLOTS], [f64; N_SLOTS]) {
    let mut a_arg = [0.0_f64; N_SLOTS];
    let mut adot_arg = [0.0_f64; N_SLOTS];
    for (d, coeffs) in args.del.iter().enumerate() {
        let (a, adot) = reduce_arg(coeffs, t_pow);
        a_arg[d] = a;
        adot_arg[d] = adot;
    }
    for (p, coeffs) in args.pla.iter().enumerate() {
        let (a, adot) = reduce_arg(coeffs, t_pow);
        a_arg[4 + p] = a;
        adot_arg[4 + p] = adot;
    }
    {
        let (a, adot) = reduce_arg(&args.zeta, t_pow);
        a_arg[12] = a;
        adot_arg[12] = adot;
    }
    (a_arg, adot_arg)
}

/// Position-only sibling of [`reduce_arg`]: value `A` only, no derivative.
///
/// Same accumulation (`for k in 0..5 { a += coeffs[k] * t_pow[k] }`) as the
/// `a` half of [`reduce_arg`], so its result is bit-for-bit identical to
/// `reduce_arg(coeffs, t_pow).0`.
#[inline]
fn reduce_arg_position(coeffs: &[f64; 5], t_pow: &[f64; 5]) -> f64 {
    let mut a = 0.0;
    for k in 0..5 {
        a += coeffs[k] * t_pow[k];
    }
    a
}

/// Position-only sibling of [`reduce_all_args`]: skips `adot_arg` entirely.
fn reduce_all_args_position(args: &Args, t_pow: &[f64; 5]) -> [f64; N_SLOTS] {
    let mut a_arg = [0.0_f64; N_SLOTS];
    for (d, coeffs) in args.del.iter().enumerate() {
        a_arg[d] = reduce_arg_position(coeffs, t_pow);
    }
    for (p, coeffs) in args.pla.iter().enumerate() {
        a_arg[4 + p] = reduce_arg_position(coeffs, t_pow);
    }
    a_arg[12] = reduce_arg_position(&args.zeta, t_pow);
    a_arg
}

/// Corrected, unit-converted main-problem amplitudes for one series, one entry
/// per term, precomputed per [`Fit`].
///
/// [`corrected_main_amplitude`] is a pure function of the term's own
/// `(amp, b1..b5)`, the fit's `Args`, and the `is_distance` flag — no `jd`
/// anywhere — yet it ran once per main term per [`elp_geocentric`] call until
/// 2026-08-29 (`docs/audit/2026-08-29-perf-investigation.md` #4): 2,636 terms
/// × ~8 flops and a division, rebuilt every evaluation to the same values.
///
/// The `arcsec → radian` conversion `eval_main_series` applied immediately
/// afterwards is folded in here too, since it is the same fixed per-series
/// choice.
///
/// Bit-identity: the stored value is the result of exactly the same operations
/// on exactly the same inputs, in the same order — only evaluated once instead
/// of once per call.
struct MainAmplitudes {
    llr: Vec<f64>,
    de405: Vec<f64>,
}

impl MainAmplitudes {
    fn build(terms: &[ElpMainTerm], is_distance: bool, arcsec_to_radian: bool) -> Self {
        let one = |fit: Fit| -> Vec<f64> {
            let args = args_for(fit);
            terms
                .iter()
                .map(|term| {
                    let a_native = corrected_main_amplitude(
                        term.amp,
                        term.b1,
                        term.b2,
                        term.b3,
                        term.b4,
                        term.b5,
                        args,
                        is_distance,
                    );
                    if arcsec_to_radian {
                        arcsec_to_rad(a_native)
                    } else {
                        a_native
                    }
                })
                .collect()
        };
        Self {
            llr: one(Fit::Llr),
            de405: one(Fit::De405),
        }
    }

    #[inline]
    fn get(&self, fit: Fit) -> &[f64] {
        match fit {
            Fit::Llr => &self.llr,
            Fit::De405 => &self.de405,
        }
    }
}

static V_MAIN_AMPS: LazyLock<MainAmplitudes> =
    LazyLock::new(|| MainAmplitudes::build(moon_longitude::MAIN.as_slice(), false, true));
static U_MAIN_AMPS: LazyLock<MainAmplitudes> =
    LazyLock::new(|| MainAmplitudes::build(moon_latitude::MAIN.as_slice(), false, true));
static R_MAIN_AMPS: LazyLock<MainAmplitudes> =
    LazyLock::new(|| MainAmplitudes::build(moon_distance::MAIN.as_slice(), true, false));

/// Evaluate a main-problem series (S1, S2, or S3).
///
/// `amps` carries the per-term corrected amplitude already in the series'
/// output unit (see [`MainAmplitudes`]) and must be parallel to `terms`.
fn eval_main_series(
    terms: &[ElpMainTerm],
    amps: &[f64],
    a_arg: &[f64; N_SLOTS],
    adot_arg: &[f64; N_SLOTS],
    kind: SeriesKind,
) -> SeriesPart {
    assert_eq!(
        terms.len(),
        amps.len(),
        "main-series amplitude table is not parallel to its term table"
    );

    let mut value = 0.0;
    let mut dot = 0.0;

    // Entries 0..4 of the caller's hoisted argument arrays are the four
    // Delaunay arguments' values Aⱼ and derivatives Ȧⱼ, in the same order this
    // function used to build for itself; every term's phase
    // φ = i1·D + i2·F + i3·l + i4·l' is a 4-term dot product with them.
    //
    // The main series is 77.2% dense (mean 3.09 nonzero multipliers of 4), so
    // it keeps the straight-line 4-term form; only the 30.0%-dense
    // perturbation series is worth indexing sparsely.
    let term_po = |term: &ElpMainTerm| -> (f64, f64) {
        let phase = (term.i1 as f64) * a_arg[0]
            + (term.i2 as f64) * a_arg[1]
            + (term.i3 as f64) * a_arg[2]
            + (term.i4 as f64) * a_arg[3];
        let omega = (term.i1 as f64) * adot_arg[0]
            + (term.i2 as f64) * adot_arg[1]
            + (term.i3 as f64) * adot_arg[2]
            + (term.i4 as f64) * adot_arg[3];
        (phase, omega)
    };

    // The 4-lane split must stay exactly where it was: `sincos_f64x4` and
    // scalar `libm::sincos` agree only to 1-2 ULP, so which terms land in the
    // vector body and which in the scalar tail is part of the output's bit
    // pattern, not an implementation detail.
    let n_chunks = terms.len() / 4;
    for ci in 0..n_chunks {
        let base = ci * 4;
        let mut phases = [0.0_f64; 4];
        let mut omegas = [0.0_f64; 4];
        for lane in 0..4 {
            let (p, o) = term_po(&terms[base + lane]);
            phases[lane] = p;
            omegas[lane] = o;
        }
        let (sin4, cos4) = sincos_f64x4(f64x4::from(phases));
        let (sin4, cos4) = (sin4.to_array(), cos4.to_array());
        for lane in 0..4 {
            let a = amps[base + lane];
            let omega = omegas[lane];
            let (sin_p, cos_p) = (sin4[lane], cos4[lane]);
            match kind {
                SeriesKind::Sine => {
                    value += a * sin_p;
                    dot += a * omega * cos_p;
                }
                SeriesKind::Cosine => {
                    value += a * cos_p;
                    dot += -a * omega * sin_p;
                }
            }
        }
    }
    for i in (n_chunks * 4)..terms.len() {
        let (phase, omega) = term_po(&terms[i]);
        let a = amps[i];
        let (sin_p, cos_p) = libm::sincos(phase);
        match kind {
            SeriesKind::Sine => {
                value += a * sin_p;
                dot += a * omega * cos_p;
            }
            SeriesKind::Cosine => {
                value += a * cos_p;
                dot += -a * omega * sin_p;
            }
        }
    }

    SeriesPart { value, dot }
}

/// Position-only sibling of [`eval_main_series`]: computes `value` only, no
/// `omega`/`dot` at all — the phase is a straight-line dot product, no
/// derivative dot product alongside it.
///
/// The `value` accumulation below is identical, term for term and in the
/// same order, to [`eval_main_series`]'s own `value` accumulator, so the
/// result is bit-for-bit identical to `eval_main_series(..).value`.
fn eval_main_series_position(
    terms: &[ElpMainTerm],
    amps: &[f64],
    a_arg: &[f64; N_SLOTS],
    kind: SeriesKind,
) -> f64 {
    assert_eq!(
        terms.len(),
        amps.len(),
        "main-series amplitude table is not parallel to its term table"
    );

    let mut value = 0.0;

    let term_phase = |term: &ElpMainTerm| -> f64 {
        (term.i1 as f64) * a_arg[0]
            + (term.i2 as f64) * a_arg[1]
            + (term.i3 as f64) * a_arg[2]
            + (term.i4 as f64) * a_arg[3]
    };

    let n_chunks = terms.len() / 4;
    for ci in 0..n_chunks {
        let base = ci * 4;
        let mut phases = [0.0_f64; 4];
        for (lane, phase) in phases.iter_mut().enumerate() {
            *phase = term_phase(&terms[base + lane]);
        }
        let (sin4, cos4) = sincos_f64x4(f64x4::from(phases));
        let (sin4, cos4) = (sin4.to_array(), cos4.to_array());
        for lane in 0..4 {
            let a = amps[base + lane];
            let (sin_p, cos_p) = (sin4[lane], cos4[lane]);
            match kind {
                SeriesKind::Sine => value += a * sin_p,
                SeriesKind::Cosine => value += a * cos_p,
            }
        }
    }
    for i in (n_chunks * 4)..terms.len() {
        let phase = term_phase(&terms[i]);
        let a = amps[i];
        let (sin_p, cos_p) = libm::sincos(phase);
        match kind {
            SeriesKind::Sine => value += a * sin_p,
            SeriesKind::Cosine => value += a * cos_p,
        }
    }

    value
}

// ─── Sparse perturbation multipliers ────────────────────────────────────────────

/// Multiplier slots every perturbation term's dot product visits
/// unconditionally.
///
/// # Why a fixed count and not the term's own nonzero count
///
/// Because the fixed count is **much faster**, which is the opposite of what
/// the arithmetic suggests. The first version of this change used a
/// variable-length CSR list of exactly the nonzero multipliers — mean 3.90 per
/// term, the minimum possible arithmetic — and it made `elp_mpp02_moon` **43%
/// slower**: 261.80 µs → 375.57 µs, measured, not predicted.
///
/// The nonzero counts cluster tightly (histogram over the 33,122 terms, counts
/// 0..7: 3, 136, 1896, 8607, 14071, 7435, 895, 79), so a loop whose length is
/// the term's own count mispredicts its exit constantly — 33,122 unpredictable
/// branches per evaluation, which cost far more than the multiply-adds they
/// saved. A fixed slot count is straight-line, fully unrolled code with no
/// data-dependent branch at all.
///
/// # Why 5
///
/// Measured, on this machine, with everything else held fixed
/// (`cargo bench -p vedaksha-ephem-core -- elp_mpp02_moon`, aarch64, bench
/// profile, against a 261.80 µs dense baseline):
///
/// | fixed slots | tail taken | `elp_mpp02_moon` |
/// |---|---|---|
/// | 4 | 25.4% | 254.02 µs |
/// | **5** | **2.9%** | **219.31 µs** |
/// | 6 | 0.24% | 231.01 µs |
/// | 7 (no tail at all) | — | 241.02 µs |
/// | 13 (every position, via this same gather) | — | 312.05 µs |
///
/// Each slot removed is worth ~11.8 µs, and each is a *gathered* load rather
/// than the dense form's register-resident argument — which is why 13 slots
/// through this path (312 µs) is slower than the dense 13 positions it
/// replaced (262 µs). 4 is worse than 5 because a tail taken 25% of the time
/// stops predicting. 5 is the measured optimum, not a guess.
const PERT_SLOTS: usize = 5;

/// Total slots stored per term, including the rarely-read tail.
///
/// A power of two so `Vec<[u8; 8]>` / `Vec<[i16; 8]>` have 8- and 16-byte
/// strides and each term's slots are an aligned load rather than a
/// cache-line-straddling read at stride 5 / 10. 8 also covers the measured
/// maximum of 7 nonzero multipliers, so the tail needs no side table at all:
/// the flag and the overflow slots both live in the term's own record. Routing
/// the tail through a separate CSR side table instead measured 238.37 µs
/// against this layout's 219.31 µs — the two extra offset loads and the slice
/// construction cost more than the two slots they saved.
const PERT_SLOTS_MAX: usize = 8;

/// Argument-array index used by padding slots. Slots 13..16 are always `+0.0`,
/// so a padding slot contributes exactly `0.0 * 0.0 = +0.0`.
const PAD_ARG: u8 = 13;

/// Width of the reduced-argument arrays: the 13 real arguments padded to 16.
///
/// A power of two so `index & 15` proves every sparse index in bounds and the
/// bounds check leaves the inner loop.
const N_SLOTS: usize = 16;

/// One perturbation group's multipliers, sparsified to a fixed slot list.
///
/// # Why
///
/// Each of the 33,122 perturbation terms carries 13 integer multipliers
/// `i1..i13`, and `eval_pert_series` formed a full 13-multiply / 12-add dot
/// product with them twice per term — once for the phase φ, once for its
/// derivative ω. That is 861,172 multiply-adds per [`elp_geocentric`] call,
/// and the module's own ablation (`simd_trig.rs`, "Where the time actually
/// goes") puts them at **36.5%** (35.0–38.1%) of `elp_mpp02_moon`.
///
/// Measured over the real tables, most of that work is multiplication by
/// zero: mean **3.90** nonzero multipliers of 13 — 30.0% dense — maximum 7,
/// 97.1% of terms with 5 or fewer. (Re-measured from the loaded tables by
/// `tests::pert_multiplier_density_matches_the_investigation`, not taken on
/// trust from `docs/audit/2026-08-29-perf-investigation.md` #2.) This index
/// stores each term's nonzero multipliers, in their original `i1..i13`
/// position order, in [`PERT_SLOTS_MAX`] slots of which [`PERT_SLOTS`] are
/// always read.
///
/// # Why it is bit-identical, and how that is checked
///
/// In IEEE 754, `0.0 × A` is exactly `±0.0` for every finite `A`, and
/// `fl(x + ±0.0) = x` for every finite `x` except `x = −0.0`. So dropping the
/// zero-multiplier products — and appending zero-valued padding products after
/// the real ones — cannot change any partial sum, *as long as the surviving
/// products are added in their original left-to-right order*. That is why
/// [`SparsePertGroup::idx`] keeps each term's slots in original position order
/// and [`sparse_phase_omega`] seeds the accumulator with slot 0's product
/// rather than with `0.0` (`0.0 + (−0.0)` is `+0.0`, so a `0.0` seed would not
/// be a no-op if the leading product were a negative zero).
///
/// The one place that argument is not airtight is a term with **no** nonzero
/// multiplier at all — there are exactly 3 — where the dense form's partial
/// sum is `−0.0` if every one of the 13 reduced arguments is negative and
/// `+0.0` otherwise, while this form always yields `+0.0`. That is a
/// difference in the sign of a zero phase, whose `sin` is `∓0.0` and whose
/// `cos` is `1.0` either way, so the resulting `±0.0` contribution changes no
/// accumulated sum.
///
/// That is not hypothetical and is not asserted away:
/// `tests::sparse_slots_are_bit_identical_to_the_dense_form` compares all
/// 33,122 terms at 13 epochs — 430,586 pairs — and finds every one identical
/// *to the bit* except **15** of the 39 all-zero-multiplier term-epoch pairs,
/// which differ in the sign of a zero and nothing else. Whether that ever
/// reaches a published number is a separate, measured question, and the answer
/// is no: `tests/lunar_series_bit_digest.rs` pins 3,780,018 output bit
/// patterns from 630,003 whole-series evaluations and they are unchanged.
///
/// # Layout
///
/// `i16` for the multiplier. Note that the maximum |multiplier| in the tables
/// is 75, so `i8` would in fact hold it —
/// `docs/audit/2026-08-29-perf-investigation.md` #2 says "`i8` overflows",
/// which is wrong on its own figure. `i16` is kept anyway: it costs nothing
/// here (the slot arrays are not the bottleneck, and the 8-slot record is
/// 24 bytes either way once `idx` is padded to a power-of-two stride) and it
/// leaves headroom if the tables are ever regenerated less aggressively.
///
/// Indices and multipliers are parallel arrays rather than interleaved pairs:
/// `[(u8, i16); 8]` pads to 32 bytes against 24, and both are read as straight
/// sequential streams. The reduced arguments are the reverse — they were tried
/// interleaved as `[(a, ȧ); 16]`, one paired load per slot instead of two, and
/// that measured **3.4% slower** (241.02 µs → 250.06 µs at 7 slots), so they
/// stay as two arrays.
///
/// The `.bin` coefficient blobs are **not** touched: this is a derived,
/// load-time transform of the already-parsed tables, which leaves the blobs
/// under their existing provenance gates unchanged.
struct SparsePertGroup {
    /// Argument index per slot, in the term's original `i1..i13` position
    /// order, then [`PAD_ARG`] for the unused slots.
    idx: Vec<[u8; PERT_SLOTS_MAX]>,
    /// Multiplier per slot, parallel to [`Self::idx`]; `0` in padding slots.
    ///
    /// `mul[PERT_SLOTS]` doubles as the flag for the rarely-taken tail: it is
    /// nonzero exactly when the term has more than [`PERT_SLOTS`] nonzero
    /// multipliers, which is 2.9% of them.
    mul: Vec<[i16; PERT_SLOTS_MAX]>,
    /// `(s, c)` with the series' `arcsec -> radian` scale already applied --
    /// `term.s * scale` was recomputed per term per call and is jd-independent
    /// (`docs/audit/2026-08-29-perf-investigation.md` #4). Same multiply, same
    /// operands, evaluated once.
    sc: Vec<(f64, f64)>,
}

/// The 13 multipliers of one perturbation term, in `i1..i13` order.
#[inline]
fn dense_multipliers_of(term: &ElpPertTerm) -> [i32; N_ARGS] {
    [
        term.i1, term.i2, term.i3, term.i4, term.i5, term.i6, term.i7, term.i8, term.i9, term.i10,
        term.i11, term.i12, term.i13,
    ]
}

impl SparsePertGroup {
    fn build(terms: &[ElpPertTerm], scale: f64) -> Self {
        let mut idx = Vec::with_capacity(terms.len());
        let mut mul = Vec::with_capacity(terms.len());
        let mut sc = Vec::with_capacity(terms.len());

        for term in terms {
            let mut term_idx = [PAD_ARG; PERT_SLOTS_MAX];
            let mut term_mul = [0i16; PERT_SLOTS_MAX];
            let mut slot = 0usize;
            // Original i1..i13 order. Anything else would reassociate the sum.
            for (j, &m) in dense_multipliers_of(term).iter().enumerate() {
                if m != 0 {
                    let j8 = j as u8; // j < 13, cannot truncate
                    let m16 = i16::try_from(m).unwrap_or_else(|_| {
                        panic!("ELP perturbation multiplier {m} does not fit in i16")
                    });
                    assert!(
                        slot < PERT_SLOTS_MAX,
                        "an ELP perturbation term has more than {PERT_SLOTS_MAX} nonzero \
                         multipliers; PERT_SLOTS_MAX is sized from the measured maximum \
                         (see tests::pert_multiplier_density_matches_the_investigation)"
                    );
                    term_idx[slot] = j8;
                    term_mul[slot] = m16;
                    slot += 1;
                }
            }
            idx.push(term_idx);
            mul.push(term_mul);
            sc.push((term.s * scale, term.c * scale));
        }

        Self { idx, mul, sc }
    }
}

/// The four power-grouped perturbation tables (`t^0`..`t^3`) for one variable.
struct SparsePertSeries {
    groups: [SparsePertGroup; 4],
}

impl SparsePertSeries {
    fn build(groups: [&[ElpPertTerm]; 4], arcsec_to_radian: bool) -> Self {
        let scale = if arcsec_to_radian {
            1.0 / ARCSEC_PER_RAD
        } else {
            1.0
        };
        Self {
            groups: groups.map(|g| SparsePertGroup::build(g, scale)),
        }
    }
}

/// Phase φ and its derivative ω for one term, from its stored slots.
///
/// Equivalent, bit-for-bit, to the dense `i1·A₀ + i2·A₁ + … + i13·A₁₂` pair it
/// replaces (see [`SparsePertGroup`] for the argument and for its one stated
/// boundary). Three properties carry that:
///
/// 1. Slots are in original `i1..i13` position order, so the surviving
///    products are summed left to right exactly as before.
/// 2. Padding slots carry multiplier `0` and point at [`PAD_ARG`], whose
///    argument value is `+0.0`, so each contributes exactly `+0.0` — and they
///    are always *after* the real slots, never interleaved.
/// 3. The accumulators are *seeded* with slot 0's product rather than starting
///    from `0.0`, mirroring the dense expression's own leading term.
///
/// φ and ω are accumulated in one pass over the slots rather than two. That
/// does not reassociate either sum: each keeps its own accumulator and its own
/// left-to-right order.
#[inline]
fn sparse_phase_omega(
    idx: [u8; PERT_SLOTS_MAX],
    mul: &[i16; PERT_SLOTS_MAX],
    a: &[f64; N_SLOTS],
    adot: &[f64; N_SLOTS],
) -> (f64, f64) {
    // `& 15` is what lets the compiler drop the bounds check: the arrays are
    // 16 wide, so a masked index is in range by construction.
    let j0 = (idx[0] & 15) as usize;
    let m0 = f64::from(mul[0]);
    let mut phase = m0 * a[j0];
    let mut omega = m0 * adot[j0];
    for s in 1..PERT_SLOTS {
        let j = (idx[s] & 15) as usize;
        let m = f64::from(mul[s]);
        phase += m * a[j];
        omega += m * adot[j];
    }
    // The tail, for the 2.9% of terms with more than PERT_SLOTS nonzero
    // multipliers. The condition reads a value the loop above already had in
    // hand and is false for 97.1% of terms, so it predicts; a loop whose
    // length is the term's own nonzero count does not, which is what made the
    // first attempt at this change 43% SLOWER than the dense form. Any
    // padding slots swept in here carry multiplier 0 and so contribute
    // exactly +0.0, appended after the real products as always.
    if mul[PERT_SLOTS] != 0 {
        for s in PERT_SLOTS..PERT_SLOTS_MAX {
            let j = (idx[s] & 15) as usize;
            let m = f64::from(mul[s]);
            phase += m * a[j];
            omega += m * adot[j];
        }
    }
    (phase, omega)
}

/// Position-only sibling of [`sparse_phase_omega`]: phase `phi` only, no
/// `omega`. Same slot order, same seeded accumulator, so bit-for-bit
/// identical to `sparse_phase_omega(..).0`.
#[inline]
fn sparse_phase_position(
    idx: [u8; PERT_SLOTS_MAX],
    mul: &[i16; PERT_SLOTS_MAX],
    a: &[f64; N_SLOTS],
) -> f64 {
    let j0 = (idx[0] & 15) as usize;
    let m0 = f64::from(mul[0]);
    let mut phase = m0 * a[j0];
    for s in 1..PERT_SLOTS {
        let j = (idx[s] & 15) as usize;
        let m = f64::from(mul[s]);
        phase += m * a[j];
    }
    if mul[PERT_SLOTS] != 0 {
        for s in PERT_SLOTS..PERT_SLOTS_MAX {
            let j = (idx[s] & 15) as usize;
            let m = f64::from(mul[s]);
            phase += m * a[j];
        }
    }
    phase
}

static V_PERT_SPARSE: LazyLock<SparsePertSeries> = LazyLock::new(|| {
    SparsePertSeries::build(
        [
            moon_longitude::PERT_0.as_slice(),
            moon_longitude::PERT_1.as_slice(),
            moon_longitude::PERT_2.as_slice(),
            moon_longitude::PERT_3.as_slice(),
        ],
        true, // arcsec -> radian
    )
});
static U_PERT_SPARSE: LazyLock<SparsePertSeries> = LazyLock::new(|| {
    SparsePertSeries::build(
        [
            moon_latitude::PERT_0.as_slice(),
            moon_latitude::PERT_1.as_slice(),
            moon_latitude::PERT_2.as_slice(),
            moon_latitude::PERT_3.as_slice(),
        ],
        true, // arcsec -> radian
    )
});
static R_PERT_SPARSE: LazyLock<SparsePertSeries> = LazyLock::new(|| {
    SparsePertSeries::build(
        [
            moon_distance::PERT_0.as_slice(),
            moon_distance::PERT_1.as_slice(),
            moon_distance::PERT_2.as_slice(),
            moon_distance::PERT_3.as_slice(),
        ],
        false, // km
    )
});

/// Evaluate the four power-grouped perturbation series for one variable.
/// Each table entry is `(S, C, i1..i13)` and is evaluated as
/// `t^n · (S sin φ + C cos φ)` with φ accumulating Delaunay + planetary +
/// ζ multipliers.
fn eval_pert_series(
    sparse: &SparsePertSeries,
    a_arg: &[f64; N_SLOTS],
    adot_arg: &[f64; N_SLOTS],
    t_pow: &[f64; 5],
) -> SeriesPart {
    let mut value = 0.0;
    let mut dot = 0.0;

    // The caller supplies the 13 argument values Aⱼ and derivatives Ȧⱼ
    // (4 Delaunay + 8 planetary + ζ, padded to 16), reduced once per
    // `vur_series` call instead of once per variable. Each term's phase is a
    // dot product of its multipliers with them, taken over its stored
    // nonzero-multiplier slots rather than all 13 positions — see
    // `SparsePertGroup`.

    // f(t)  = t^n · (s sin φ + c cos φ)
    // f'(t) = n·t^{n-1}·(s sin φ + c cos φ) + t^n · ω · (s cos φ − c sin φ)
    for (n, group) in sparse.groups.iter().enumerate() {
        let tn = t_pow[n];
        let dtn = if n >= 1 {
            (n as f64) * t_pow[n - 1]
        } else {
            0.0
        };

        // The 4-lane split must stay exactly where the dense form put it:
        // `sincos_f64x4` and scalar `libm::sincos` agree only to 1-2 ULP, so
        // which terms land in the vector body and which in the scalar tail is
        // part of the output's bit pattern.
        let n_terms = group.sc.len();
        let n_chunks = n_terms / 4;
        for ci in 0..n_chunks {
            let base = ci * 4;
            let mut phases = [0.0_f64; 4];
            let mut omegas = [0.0_f64; 4];
            for lane in 0..4 {
                let (p, o) = sparse_phase_omega(
                    group.idx[base + lane],
                    &group.mul[base + lane],
                    a_arg,
                    adot_arg,
                );
                phases[lane] = p;
                omegas[lane] = o;
            }
            let (sin4, cos4) = sincos_f64x4(f64x4::from(phases));
            let (sin4, cos4) = (sin4.to_array(), cos4.to_array());
            for lane in 0..4 {
                let (s, c) = group.sc[base + lane];
                let (sin_p, cos_p) = (sin4[lane], cos4[lane]);
                let omega = omegas[lane];
                let inner = s * sin_p + c * cos_p;
                let inner_dot = s * cos_p - c * sin_p;
                value += tn * inner;
                dot += dtn * inner + tn * omega * inner_dot;
            }
        }
        for i in (n_chunks * 4)..n_terms {
            let (phase, omega) = sparse_phase_omega(group.idx[i], &group.mul[i], a_arg, adot_arg);
            let (s, c) = group.sc[i];
            let (sin_p, cos_p) = libm::sincos(phase);
            let inner = s * sin_p + c * cos_p;
            let inner_dot = s * cos_p - c * sin_p;
            value += tn * inner;
            dot += dtn * inner + tn * omega * inner_dot;
        }
    }

    SeriesPart { value, dot }
}

/// Position-only sibling of [`eval_pert_series`]: `value` only, no `dot`,
/// `omega` or `dtn` at all.
///
/// The `value` accumulation is identical, group for group and term for term
/// in the same order, to [`eval_pert_series`]'s own `value` accumulator —
/// bit-for-bit identical to `eval_pert_series(..).value`.
fn eval_pert_series_position(
    sparse: &SparsePertSeries,
    a_arg: &[f64; N_SLOTS],
    t_pow: &[f64; 5],
) -> f64 {
    let mut value = 0.0;

    for (n, group) in sparse.groups.iter().enumerate() {
        let tn = t_pow[n];

        let n_terms = group.sc.len();
        let n_chunks = n_terms / 4;
        for ci in 0..n_chunks {
            let base = ci * 4;
            let mut phases = [0.0_f64; 4];
            for (lane, phase) in phases.iter_mut().enumerate() {
                *phase =
                    sparse_phase_position(group.idx[base + lane], &group.mul[base + lane], a_arg);
            }
            let (sin4, cos4) = sincos_f64x4(f64x4::from(phases));
            let (sin4, cos4) = (sin4.to_array(), cos4.to_array());
            for lane in 0..4 {
                let (s, c) = group.sc[base + lane];
                let (sin_p, cos_p) = (sin4[lane], cos4[lane]);
                let inner = s * sin_p + c * cos_p;
                value += tn * inner;
            }
        }
        for i in (n_chunks * 4)..n_terms {
            let phase = sparse_phase_position(group.idx[i], &group.mul[i], a_arg);
            let (s, c) = group.sc[i];
            let (sin_p, cos_p) = libm::sincos(phase);
            let inner = s * sin_p + c * cos_p;
            value += tn * inner;
        }
    }

    value
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

/// Position-only sibling of [`rectangular_from_spherical`]: (x, y, z) only,
/// no velocity at all.
///
/// `x1`/`x2`/`x3` are computed by the identical expressions, in the same
/// order, as [`rectangular_from_spherical`]'s own `x1`/`x2`/`x3` — bit-for-bit
/// identical results.
fn rectangular_from_spherical_position(v: f64, u: f64, r: f64) -> (f64, f64, f64) {
    let cv = libm::cos(v);
    let sv = libm::sin(v);
    let cu = libm::cos(u);
    let su = libm::sin(u);

    let cw = r * cu;

    let x1 = cw * cv;
    let x2 = cw * sv;
    let x3 = r * su;

    (x1, x2, x3)
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
    let xyz_vz = -pwra * xp1
        + qwra * xp2
        + (pw2 + qw2 - 1.0) * xp3
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

/// Position-only sibling of [`apply_pq_rotation`]: rotates `(x, y, z)` only,
/// no velocity rotation (no `(P, Q)`-prime terms) at all.
///
/// `xyz_x`/`xyz_y`/`xyz_z` are computed by the identical expressions, in the
/// same order and from the same `pw`/`qw`/`ra`/`pwqw`/`pw2`/`qw2`/`pwra`/
/// `qwra` intermediates, as [`apply_pq_rotation`]'s own position half — so
/// bit-for-bit identical results.
fn apply_pq_rotation_position(xyz_date: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    let pw = (P1 + P2 * t + P3 * t * t + P4 * t * t * t + P5 * t * t * t * t) * t;
    let qw = (Q1 + Q2 * t + Q3 * t * t + Q4 * t * t * t + Q5 * t * t * t * t) * t;
    let ra = 2.0 * libm::sqrt(1.0 - pw * pw - qw * qw);
    let pwqw = 2.0 * pw * qw;
    let pw2 = 1.0 - 2.0 * pw * pw;
    let qw2 = 1.0 - 2.0 * qw * qw;
    let pwra = pw * ra;
    let qwra = qw * ra;

    let (x1, x2, x3) = xyz_date;

    let xyz_x = pw2 * x1 + pwqw * x2 + pwra * x3;
    let xyz_y = pwqw * x1 + qw2 * x2 - qwra * x3;
    let xyz_z = -pwra * x1 + qwra * x2 + (pw2 + qw2 - 1.0) * x3;

    (xyz_x, xyz_y, xyz_z)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Epochs used by the sparse-index tests: both ends of the provider's
    /// supported JD range, J2000 itself, and a spread in between. `t` is in
    /// Julian centuries from J2000.
    fn sparse_test_epochs() -> Vec<f64> {
        use crate::analytical::{JD_MAX, JD_MIN};
        let t_min = (JD_MIN - J2000) / DAYS_PER_CENTURY;
        let t_max = (JD_MAX - J2000) / DAYS_PER_CENTURY;
        vec![
            t_min, -30.0, -20.0, -10.0, -5.0, -1.0, 0.0, 0.253, 1.0, 2.0, 5.0, 8.0, t_max,
        ]
    }

    /// Every perturbation group in the loaded tables, with a label.
    fn all_pert_groups() -> Vec<(&'static str, &'static [ElpPertTerm])> {
        vec![
            ("V0", moon_longitude::PERT_0.as_slice()),
            ("V1", moon_longitude::PERT_1.as_slice()),
            ("V2", moon_longitude::PERT_2.as_slice()),
            ("V3", moon_longitude::PERT_3.as_slice()),
            ("U0", moon_latitude::PERT_0.as_slice()),
            ("U1", moon_latitude::PERT_1.as_slice()),
            ("U2", moon_latitude::PERT_2.as_slice()),
            ("U3", moon_latitude::PERT_3.as_slice()),
            ("R0", moon_distance::PERT_0.as_slice()),
            ("R1", moon_distance::PERT_1.as_slice()),
            ("R2", moon_distance::PERT_2.as_slice()),
            ("R3", moon_distance::PERT_3.as_slice()),
        ]
    }

    fn dense_multipliers(term: &ElpPertTerm) -> [i32; N_ARGS] {
        [
            term.i1, term.i2, term.i3, term.i4, term.i5, term.i6, term.i7, term.i8, term.i9,
            term.i10, term.i11, term.i12, term.i13,
        ]
    }

    /// The density figures the sparse rewrite is justified by, re-measured
    /// from the loaded coefficient tables.
    ///
    /// `docs/audit/2026-08-29-perf-investigation.md` #2 states mean 3.90
    /// nonzero of 13 (30.0% dense), maximum 7, 97.0% of terms with ≤5, and
    /// max |multiplier| 75. That document also claims 75 overflows `i8` —
    /// it doesn't (`i8` holds -128..127); see this module's own `# Layout`
    /// doc for the actual (unrelated) reason `i16` was kept. These figures
    /// are load-bearing for the performance argument, so they are asserted
    /// here rather than taken on trust from a document.
    #[test]
    fn pert_multiplier_density_matches_the_investigation() {
        let mut total_terms = 0u64;
        let mut total_nonzero = 0u64;
        let mut max_nonzero = 0usize;
        let mut max_abs_multiplier = 0i32;
        let mut le5 = 0u64;
        let mut histogram = [0u64; N_ARGS + 1];

        for (label, group) in all_pert_groups() {
            let mut group_max = 0usize;
            for term in group {
                let nz = dense_multipliers(term).iter().filter(|&&m| m != 0).count();
                total_terms += 1;
                total_nonzero += nz as u64;
                histogram[nz] += 1;
                max_nonzero = max_nonzero.max(nz);
                group_max = group_max.max(nz);
                if nz <= 5 {
                    le5 += 1;
                }
                for m in dense_multipliers(term) {
                    max_abs_multiplier = max_abs_multiplier.max(m.abs());
                }
            }
            println!("{label}: {} terms, max nonzero {group_max}", group.len());
        }

        let mean = total_nonzero as f64 / total_terms as f64;
        println!(
            "pert terms {total_terms}, mean nonzero {mean:.2} of {N_ARGS} \
             ({:.1}% dense), max nonzero {max_nonzero}, \
             <=5 nonzero {:.1}%, max |multiplier| {max_abs_multiplier}",
            100.0 * mean / N_ARGS as f64,
            100.0 * le5 as f64 / total_terms as f64,
        );
        println!("nonzero-count histogram: {histogram:?}");

        assert_eq!(total_terms, 33_122, "perturbation term count moved");
        assert!(
            (3.85..3.95).contains(&mean),
            "mean nonzero multipliers {mean:.3}, investigation says 3.90"
        );
        assert_eq!(max_nonzero, 7, "investigation says max 7 nonzero of 13");
        assert!(
            le5 as f64 / total_terms as f64 > 0.96,
            "investigation says 97.0% of terms have <=5 nonzero multipliers"
        );
        // 75 fits in i8 (-128..127) fine — see the module's `# Layout` doc
        // for why i16 was kept anyway; it isn't this.
        assert_eq!(
            max_abs_multiplier, 75,
            "investigation says max |multiplier| is 75"
        );
        assert!(
            i16::try_from(max_abs_multiplier).is_ok(),
            "multiplier does not fit the i16 the sparse index stores"
        );
    }

    /// The sparse slot list reproduces the dense 13-term dot product
    /// **bit for bit**, for every perturbation term, at every epoch tried.
    ///
    /// This is the direct check of the claim [`SparsePertGroup`] is built on.
    /// It compares `to_bits()`, not values, so `+0.0` and `−0.0` are
    /// distinguished — the whole argument turns on signed zeros, and `==`
    /// would hide exactly the case that could go wrong.
    ///
    /// It covers all 33,122 terms × 13 epochs spanning the provider's full
    /// supported JD range = 430,586 phase comparisons and as many for ω. That
    /// is the *unit-level* evidence; `tests/lunar_series_bit_digest.rs` is the
    /// end-to-end evidence over 630,003 whole-series evaluations.
    ///
    /// The 3 terms with no nonzero multiplier at all are held to a weaker
    /// statement, deliberately and explicitly: both forms must produce a zero,
    /// but the *sign* of that zero may differ, because the dense form's
    /// thirteen `±0.0` products sum to `−0.0` only when every reduced argument
    /// is negative while the sparse form always yields `+0.0`. `sin(±0.0)` is
    /// `±0.0` and `cos(±0.0)` is `1.0`, so the term's contribution is the same
    /// number either way; the assertion below records the boundary rather than
    /// papering over it, and reports how often it is actually reached.
    #[test]
    fn sparse_slots_are_bit_identical_to_the_dense_form() {
        let args = args_for(Fit::Llr);
        let mut compared = 0u64;
        let mut zero_multiplier_terms = 0u64;
        let mut zero_sign_differences = 0u64;

        for t in sparse_test_epochs() {
            let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];
            let (a_arg, adot_arg) = reduce_all_args(args, &t_pow);

            for (label, group) in all_pert_groups() {
                // Build the same side table production builds, from the same
                // parsed terms. `scale` is irrelevant to phase/omega, so any
                // value does; 1.0 keeps `sc` readable if this ever prints.
                let sparse = SparsePertGroup::build(group, 1.0);

                for (i, term) in group.iter().enumerate() {
                    let m = dense_multipliers_of(term);

                    // The dense expression `term_poc` used, verbatim in shape:
                    // thirteen products summed left to right, seeded by the
                    // first product rather than by zero.
                    let dense_phase = (m[0] as f64) * a_arg[0]
                        + (m[1] as f64) * a_arg[1]
                        + (m[2] as f64) * a_arg[2]
                        + (m[3] as f64) * a_arg[3]
                        + (m[4] as f64) * a_arg[4]
                        + (m[5] as f64) * a_arg[5]
                        + (m[6] as f64) * a_arg[6]
                        + (m[7] as f64) * a_arg[7]
                        + (m[8] as f64) * a_arg[8]
                        + (m[9] as f64) * a_arg[9]
                        + (m[10] as f64) * a_arg[10]
                        + (m[11] as f64) * a_arg[11]
                        + (m[12] as f64) * a_arg[12];
                    let dense_omega = (m[0] as f64) * adot_arg[0]
                        + (m[1] as f64) * adot_arg[1]
                        + (m[2] as f64) * adot_arg[2]
                        + (m[3] as f64) * adot_arg[3]
                        + (m[4] as f64) * adot_arg[4]
                        + (m[5] as f64) * adot_arg[5]
                        + (m[6] as f64) * adot_arg[6]
                        + (m[7] as f64) * adot_arg[7]
                        + (m[8] as f64) * adot_arg[8]
                        + (m[9] as f64) * adot_arg[9]
                        + (m[10] as f64) * adot_arg[10]
                        + (m[11] as f64) * adot_arg[11]
                        + (m[12] as f64) * adot_arg[12];

                    let (sparse_phase, sparse_omega) =
                        sparse_phase_omega(sparse.idx[i], &sparse.mul[i], &a_arg, &adot_arg);

                    compared += 1;

                    if m.iter().all(|&x| x == 0) {
                        zero_multiplier_terms += 1;
                        assert_eq!(
                            dense_phase, 0.0,
                            "{label}[{i}] t={t}: dense phase of an all-zero-multiplier \
                             term is not a zero"
                        );
                        assert_eq!(
                            sparse_phase, 0.0,
                            "{label}[{i}] t={t}: sparse phase of an all-zero-multiplier \
                             term is not a zero"
                        );
                        if dense_phase.to_bits() != sparse_phase.to_bits() {
                            zero_sign_differences += 1;
                        }
                        continue;
                    }

                    assert_eq!(
                        dense_phase.to_bits(),
                        sparse_phase.to_bits(),
                        "{label}[{i}] t={t}: phase differs — dense {dense_phase:e} \
                         ({:016x}) vs sparse {sparse_phase:e} ({:016x})",
                        dense_phase.to_bits(),
                        sparse_phase.to_bits(),
                    );
                    assert_eq!(
                        dense_omega.to_bits(),
                        sparse_omega.to_bits(),
                        "{label}[{i}] t={t}: omega differs — dense {dense_omega:e} \
                         ({:016x}) vs sparse {sparse_omega:e} ({:016x})",
                        dense_omega.to_bits(),
                        sparse_omega.to_bits(),
                    );
                }
            }
        }

        println!(
            "sparse vs dense: {compared} term-epoch pairs compared bit-for-bit, \
             all identical except {zero_sign_differences} sign-of-zero differences \
             among {zero_multiplier_terms} all-zero-multiplier term-epoch pairs"
        );
        assert_eq!(
            compared,
            33_122 * 13,
            "expected every term at every epoch to be compared"
        );
    }

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

    /// [`elp_geocentric_position`] must be bit-for-bit `(x, y, z)` of
    /// [`elp_geocentric`], across the full supported JD range.
    ///
    /// This is the direct end-to-end proof that skipping the velocity/omega
    /// computation changes no computed position value.
    #[test]
    fn elp_geocentric_position_matches_elp_geocentric_bit_for_bit() {
        for t in sparse_test_epochs() {
            let jd = J2000 + t * DAYS_PER_CENTURY;
            let full = elp_geocentric(jd);
            let (x, y, z) = elp_geocentric_position(jd);
            assert_eq!(
                x.to_bits(),
                full.x.to_bits(),
                "t={t} jd={jd}: x differs — position-only {x:e} vs full {:e}",
                full.x
            );
            assert_eq!(
                y.to_bits(),
                full.y.to_bits(),
                "t={t} jd={jd}: y differs — position-only {y:e} vs full {:e}",
                full.y
            );
            assert_eq!(
                z.to_bits(),
                full.z.to_bits(),
                "t={t} jd={jd}: z differs — position-only {z:e} vs full {:e}",
                full.z
            );
        }
    }

    /// [`elp_geocentric_position_of_date`] must be bit-for-bit `(x, y, z)` of
    /// [`elp_geocentric_of_date`], across the full supported JD range.
    #[test]
    fn elp_geocentric_position_of_date_matches_elp_geocentric_of_date_bit_for_bit() {
        for t in sparse_test_epochs() {
            let jd = J2000 + t * DAYS_PER_CENTURY;
            let full = elp_geocentric_of_date(jd);
            let (x, y, z) = elp_geocentric_position_of_date(jd);
            assert_eq!(
                x.to_bits(),
                full.x.to_bits(),
                "t={t} jd={jd}: x differs — position-only {x:e} vs full {:e}",
                full.x
            );
            assert_eq!(
                y.to_bits(),
                full.y.to_bits(),
                "t={t} jd={jd}: y differs — position-only {y:e} vs full {:e}",
                full.y
            );
            assert_eq!(
                z.to_bits(),
                full.z.to_bits(),
                "t={t} jd={jd}: z differs — position-only {z:e} vs full {:e}",
                full.z
            );
        }
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
        assert!(
            dr < 100.0,
            "fit difference at J2000 = {dr:.3} km, too large"
        );
    }

    /// `simd_trig::sincos_f64x4` was validated (before this test existed)
    /// only over `|x| ≤ 3000`. This test computes the ACTUAL `|phase|`
    /// values `eval_main_series`/`eval_pert_series` feed to that kernel
    /// across `AnalyticalProvider`'s full supported JD range (`JD_MIN` /
    /// `JD_MAX` in `analytical/mod.rs`, ~-2000 CE to ~+3000 CE), by
    /// evaluating the real argument polynomials and the real per-term
    /// integer multipliers from every coefficient table — not an assumed
    /// or guessed bound. See docs/audit/2026-08-29-perf-investigation.md
    /// §3b, which independently derived 3,303,561 rad at t=-40 centuries
    /// by the same method.
    ///
    /// It then measures `sincos_f64x4` vs. scalar `libm::sincos` at every
    /// one of those real phases, plus a dense sweep across the full
    /// reachable interval, and records the actual error — this is the
    /// number that decides whether the kernel is safe to keep using here
    /// (and whether it would be safe to extend to VSOP87A, which runs to
    /// even larger arguments).
    #[test]
    fn sincos_matches_libm_at_real_elp_phase_domain() {
        use crate::analytical::{JD_MAX, JD_MIN};

        let t_min = (JD_MIN - J2000) / DAYS_PER_CENTURY;
        let t_max = (JD_MAX - J2000) / DAYS_PER_CENTURY;
        // J2000, present-day (~2025), several points spanning to both
        // range extremes.
        let epochs = [
            t_min, -30.0, -20.0, -10.0, -5.0, 0.0, 0.253, 2.0, 5.0, 8.0, t_max,
        ];

        let args = build_args(Fit::Llr);

        let mut max_abs_phase = 0.0_f64;
        let mut max_abs_phase_t = 0.0_f64;
        let mut real_phases: Vec<f64> = Vec::new();

        for &t in &epochs {
            let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

            // Main series: 4-arg dot products (D, F, l, l').
            let mut a_arg4 = [0.0_f64; 4];
            for (d, coeffs) in args.del.iter().enumerate() {
                a_arg4[d] = reduce_arg(coeffs, &t_pow).0;
            }
            for terms in [
                moon_longitude::MAIN.as_slice(),
                moon_latitude::MAIN.as_slice(),
                moon_distance::MAIN.as_slice(),
            ] {
                for term in terms {
                    let phase = f64::from(term.i1) * a_arg4[0]
                        + f64::from(term.i2) * a_arg4[1]
                        + f64::from(term.i3) * a_arg4[2]
                        + f64::from(term.i4) * a_arg4[3];
                    real_phases.push(phase);
                    if phase.abs() > max_abs_phase {
                        max_abs_phase = phase.abs();
                        max_abs_phase_t = t;
                    }
                }
            }

            // Perturbation series: 13-arg dot products (4 Delaunay + 8
            // planetary mean longitudes + zeta).
            let mut a_arg13 = [0.0_f64; 13];
            for (d, coeffs) in args.del.iter().enumerate() {
                a_arg13[d] = reduce_arg(coeffs, &t_pow).0;
            }
            for (p, coeffs) in args.pla.iter().enumerate() {
                a_arg13[4 + p] = reduce_arg(coeffs, &t_pow).0;
            }
            a_arg13[12] = reduce_arg(&args.zeta, &t_pow).0;

            for groups in [
                [
                    moon_longitude::PERT_0.as_slice(),
                    moon_longitude::PERT_1.as_slice(),
                    moon_longitude::PERT_2.as_slice(),
                    moon_longitude::PERT_3.as_slice(),
                ],
                [
                    moon_latitude::PERT_0.as_slice(),
                    moon_latitude::PERT_1.as_slice(),
                    moon_latitude::PERT_2.as_slice(),
                    moon_latitude::PERT_3.as_slice(),
                ],
                [
                    moon_distance::PERT_0.as_slice(),
                    moon_distance::PERT_1.as_slice(),
                    moon_distance::PERT_2.as_slice(),
                    moon_distance::PERT_3.as_slice(),
                ],
            ] {
                for group in groups {
                    for term in group {
                        let phase = f64::from(term.i1) * a_arg13[0]
                            + f64::from(term.i2) * a_arg13[1]
                            + f64::from(term.i3) * a_arg13[2]
                            + f64::from(term.i4) * a_arg13[3]
                            + f64::from(term.i5) * a_arg13[4]
                            + f64::from(term.i6) * a_arg13[5]
                            + f64::from(term.i7) * a_arg13[6]
                            + f64::from(term.i8) * a_arg13[7]
                            + f64::from(term.i9) * a_arg13[8]
                            + f64::from(term.i10) * a_arg13[9]
                            + f64::from(term.i11) * a_arg13[10]
                            + f64::from(term.i12) * a_arg13[11]
                            + f64::from(term.i13) * a_arg13[12];
                        real_phases.push(phase);
                        if phase.abs() > max_abs_phase {
                            max_abs_phase = phase.abs();
                            max_abs_phase_t = t;
                        }
                    }
                }
            }
        }

        println!(
            "max |phase| fed to sincos_f64x4 across the supported JD range \
             [{JD_MIN}, {JD_MAX}]: {max_abs_phase:e} rad at t={max_abs_phase_t} cy \
             ({} real term-phase samples)",
            real_phases.len()
        );
        // Cross-check against the investigation's independently-derived
        // figure at the JD_MIN edge (t=-40 cy): ~3,303,561 rad. Our grid
        // uses the exact t_min rather than -40.0, so allow a wide band —
        // this is a sanity check that we are in the same regime, not a
        // precise reproduction.
        assert!(
            max_abs_phase > 3_000_000.0,
            "expected max |phase| > 3e6 rad near JD_MIN (t_min={t_min}), got {max_abs_phase:e}"
        );

        // Dense sweep across the full reachable interval (irrational step,
        // as in simd_trig::tests::matches_libm_across_domain) to catch
        // reduction-boundary cases the discrete real term phases don't
        // happen to land on.
        let mut x = -max_abs_phase;
        while x <= max_abs_phase {
            real_phases.push(x);
            x += 137.035_999; // fine, irrational-ish step to vary the reduction
        }

        let mut max_sin_err = 0.0_f64;
        let mut max_cos_err = 0.0_f64;
        for chunk in real_phases.chunks(4) {
            let mut lane = [0.0_f64; 4];
            lane[..chunk.len()].copy_from_slice(chunk);
            let (s, c) = sincos_f64x4(f64x4::from(lane));
            let (s, c) = (s.to_array(), c.to_array());
            for i in 0..chunk.len() {
                let (ls, lc) = libm::sincos(lane[i]);
                max_sin_err = max_sin_err.max((s[i] - ls).abs());
                max_cos_err = max_cos_err.max((c[i] - lc).abs());
            }
        }

        println!(
            "max sin err = {max_sin_err:e}, max cos err = {max_cos_err:e} \
             over the real ELP phase domain (|x| up to {max_abs_phase:e} rad)"
        );

        // Measured 2026-08-29 (aarch64, debug profile): max sin err
        // 1.11e-16, max cos err 2.22e-16 — 1-2 ULP, the same order as the
        // existing |x| ≤ 3000 test's own measured error. The vector
        // kernel's large-argument reduction is NOT degrading here; use
        // the same 1e-12 bound `simd_trig::tests::matches_libm_across_domain`
        // uses, so both tests hold the kernel to one documented standard.
        assert!(
            max_sin_err < 1e-12,
            "max sin abs error {max_sin_err:e} over the real production phase \
             domain exceeds 1e-12 — the vector sincos kernel is not safe at \
             this range; see docs/audit/2026-08-29-perf-investigation.md §3b"
        );
        assert!(
            max_cos_err < 1e-12,
            "max cos abs error {max_cos_err:e} over the real production phase \
             domain exceeds 1e-12 — the vector sincos kernel is not safe at \
             this range; see docs/audit/2026-08-29-perf-investigation.md §3b"
        );
    }
}
