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
//! ## Evaluation
//!
//! Every term is `amplitude · sin φ` or `cos φ` with `φ` an integer
//! combination of 13 slowly-varying arguments. Rather than one `sincos` per
//! term, each evaluation builds the sine and cosine of every needed combination
//! by angle addition over a prefix tree of the multiplier vectors, and each term
//! reads its pair. The method, its cost and its accuracy are in `phase_graph`.
//! The terms are then summed with four independent accumulators.
//!
//! Output is not bit-identical to the one-`sincos`-per-term evaluation this
//! replaced, and does not claim to be: the two differ by the old route's
//! rounding of each phase at the magnitude of the phase (`1e-10` rad for the
//! largest phases at the range edge), which the new route avoids. Measured over
//! 85,005 instants spanning the supported range, the series moved by at most
//! one unit in the last place of the unwrapped longitude `V` in 22 instants
//! (0.03%), and by less than `1e-7` arcsec otherwise; see
//! `tests::series_match_the_exact_phase_reference` for the comparison against
//! an exact-phase reference.
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
use std::cell::RefCell;
use std::sync::LazyLock;

use crate::analytical::coefficients::loader::{ElpMainTerm, ElpPertTerm};
use crate::analytical::coefficients::{moon_distance, moon_latitude, moon_longitude};
use crate::analytical::phase_graph::{Multipliers, N_ARGS, Pair, PhaseGraph};

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

/// Count of distance-only ELP/MPP02 evaluations ([`elp_geocentric_distance`]).
///
/// Kept apart from [`ELP_GEOCENTRIC_CALLS`] because a distance-only
/// evaluation runs the distance series alone, about 40% of a full one, and a
/// guard that counts full evaluations must not have it folded in. Same
/// instrumentation-only status as that counter.
#[doc(hidden)]
pub static ELP_DISTANCE_CALLS: AtomicU64 = AtomicU64::new(0);

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

/// Geocentric Moon **distance only**, in km (LLR fit).
///
/// Evaluates the distance series alone — the `r` third of
/// [`vur_series_position`], same terms, same accumulation order — so the
/// value is bit-for-bit identical to the `r` that [`elp_geocentric_position`]
/// converts to rectangular coordinates. The length of that rectangular vector
/// agrees with it to rounding only (a few ulp): the P/Q rotation is
/// orthogonal and the spherical-to-rectangular step preserves `r`.
///
/// It exists for the light-time iteration's first step, which needs only the
/// distance (`τ = r / c`) and so does not need the longitude and latitude
/// series, about 60% of the terms. See
/// [`crate::jpl::EphemerisProvider::moon_distance`] for the contract this
/// serves and `coordinates::moon_light_time` for how the result stays bit for
/// bit what a full first evaluation would give.
#[must_use]
pub fn elp_geocentric_distance(jd: f64) -> f64 {
    ELP_DISTANCE_CALLS.fetch_add(1, Ordering::Relaxed);
    let fit = Fit::Llr;
    let args = args_for(fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

    let a_arg = reduce_all_args_position(args, &t_pow);

    // The r-only graph: the longitude and latitude terms' nodes are not built.
    let tables = &*TABLES_R;
    with_workspace(tables.graph.pool_len(), |ws| {
        tables.graph.fill_phases(&a_arg, &mut ws.pool);
        let pool = &ws.pool[..];
        let r_main = eval_main_series::<false>(
            &tables.r.main,
            R_MAIN_AMPS.get(fit),
            pool,
            &[],
            SeriesKind::Cosine,
        )
        .value;
        let r_pert = eval_pert_series::<false>(&tables.r, &t_pow, pool, &[]).value;
        (r_main + r_pert) * RA0
    })
}

/// Mean longitude `W₁` of the Moon, referred to the mean ecliptic and equinox
/// of **date** (LLR fit), with its rate: `(longitude_deg, rate_deg_per_day)`.
///
/// This is the secular polynomial the longitude series is built on — the
/// constant `W₁` of elpmpp02.pdf Table 1 plus its per-fit corrections, with no
/// periodic term — moved to the equinox of date by the same precession
/// polynomial [`elp_geocentric_of_date`] adds to `V`. It is therefore the mean
/// that the true longitude from [`elp_geocentric_position_of_date`] oscillates
/// about, and the two are in one frame by construction.
///
/// The longitude is normalised to `[0, 360)`; the rate is the analytic
/// derivative of the same polynomial, in degrees per day.
///
/// It exists so a consumer that needs the Moon's mean longitude (the Jyotish
/// mean-motion table) reads the value this theory already carries instead of
/// restating a second set of lunar constants.
#[must_use]
pub fn mean_longitude_of_date(jd: f64) -> (f64, f64) {
    let args = args_for(Fit::Llr);
    let tau = (jd - J2000) / DAYS_PER_CENTURY;

    let prec = (P_PA_LIN + P_PA_DELTA) * tau
        + P_PA_QUAD * tau * tau
        + P_PA_CUBE * tau * tau * tau
        + P_PA_QUART * tau * tau * tau * tau;
    let prec_dot = (P_PA_LIN + P_PA_DELTA)
        + 2.0 * P_PA_QUAD * tau
        + 3.0 * P_PA_CUBE * tau * tau
        + 4.0 * P_PA_QUART * tau * tau * tau;

    let lon_rad = eval_poly(&args.w1, tau) + prec;
    let rate_rad_per_cy = eval_poly_dot(&args.w1, tau) + prec_dot;

    (
        lon_rad.to_degrees().rem_euclid(360.0),
        rate_rad_per_cy.to_degrees() / DAYS_PER_CENTURY,
    )
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

    let tables = &*TABLES_ALL;
    with_workspace(tables.graph.pool_len(), |ws| {
        // Every term's sin/cos and ω, once, for all three variables: see
        // `phase_graph`.
        tables.graph.fill_phases(&a_arg, &mut ws.pool);
        tables.graph.fill_omega(&adot_arg, &mut ws.omega);
        let (pool, omega) = (&ws.pool[..], &ws.omega[..]);

        // === Longitude (V), radians ===
        let v_main = eval_main_series::<true>(
            &tables.v.main,
            V_MAIN_AMPS.get(fit),
            pool,
            omega,
            SeriesKind::Sine,
        );
        let v_pert = eval_pert_series::<true>(&tables.v, &t_pow, pool, omega);
        let v_w1 = eval_poly(&args.w1, t);
        let v = v_w1 + v_main.value + v_pert.value;
        let vdot = eval_poly_dot(&args.w1, t) + v_main.dot + v_pert.dot;

        // === Latitude (U), radians ===
        let u_main = eval_main_series::<true>(
            &tables.u.main,
            U_MAIN_AMPS.get(fit),
            pool,
            omega,
            SeriesKind::Sine,
        );
        let u_pert = eval_pert_series::<true>(&tables.u, &t_pow, pool, omega);
        let u = u_main.value + u_pert.value;
        let udot = u_main.dot + u_pert.dot;

        // === Distance (r), km ===
        // S3 main is a *cosine* series (elpmpp02.pdf §2.2.1), and corrections
        // include the additional `−(2/3)·A·δν/ν` term per §4.4.1.
        let r_main = eval_main_series::<true>(
            &tables.r.main,
            R_MAIN_AMPS.get(fit),
            pool,
            omega,
            SeriesKind::Cosine,
        );
        let r_pert = eval_pert_series::<true>(&tables.r, &t_pow, pool, omega);
        let r = (r_main.value + r_pert.value) * RA0;
        let rdot = (r_main.dot + r_pert.dot) * RA0;

        (v, u, r, vdot, udot, rdot)
    })
}

/// Position-only sibling of [`vur_series`]: same series, same terms, same
/// accumulation order, but never computes `omega`/`adot_arg`/`dot` at all.
///
/// Returns `(V, U, r)` — V, U in radians, r in km — bit-for-bit identical to
/// the first three elements of [`vur_series`]'s tuple at the same `(jd,
/// fit)`: the two instantiate the same generic accumulation, which forms
/// `value` by the same operations whether or not it also forms `dot`. See
/// `tests::elp_geocentric_position_matches_elp_geocentric_bit_for_bit` and
/// `tests::elp_geocentric_position_of_date_matches_elp_geocentric_of_date_bit_for_bit`
/// for the end-to-end proof (there is no direct per-function test of
/// `vur_series_position` alone; those two tests exercise it through both
/// public entry points).
fn vur_series_position(jd: f64, fit: Fit) -> (f64, f64, f64) {
    ELP_GEOCENTRIC_CALLS.fetch_add(1, Ordering::Relaxed);
    let args = args_for(fit);
    let t = (jd - J2000) / DAYS_PER_CENTURY;
    let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];

    let a_arg = reduce_all_args_position(args, &t_pow);

    let tables = &*TABLES_ALL;
    with_workspace(tables.graph.pool_len(), |ws| {
        tables.graph.fill_phases(&a_arg, &mut ws.pool);
        let pool = &ws.pool[..];

        // === Longitude (V), radians ===
        let v_main = eval_main_series::<false>(
            &tables.v.main,
            V_MAIN_AMPS.get(fit),
            pool,
            &[],
            SeriesKind::Sine,
        )
        .value;
        let v_pert = eval_pert_series::<false>(&tables.v, &t_pow, pool, &[]).value;
        let v_w1 = eval_poly(&args.w1, t);
        let v = v_w1 + v_main + v_pert;

        // === Latitude (U), radians ===
        let u_main = eval_main_series::<false>(
            &tables.u.main,
            U_MAIN_AMPS.get(fit),
            pool,
            &[],
            SeriesKind::Sine,
        )
        .value;
        let u_pert = eval_pert_series::<false>(&tables.u, &t_pow, pool, &[]).value;
        let u = u_main + u_pert;

        // === Distance (r), km ===
        let r_main = eval_main_series::<false>(
            &tables.r.main,
            R_MAIN_AMPS.get(fit),
            pool,
            &[],
            SeriesKind::Cosine,
        )
        .value;
        let r_pert = eval_pert_series::<false>(&tables.r, &t_pow, pool, &[]).value;
        let r = (r_main + r_pert) * RA0;

        (v, u, r)
    })
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
/// The result is indexed by the argument numbering [`PhaseGraph`] uses:
/// D, F, l, l′, then the eight planetary mean longitudes, then ζ.
fn reduce_all_args(args: &Args, t_pow: &[f64; 5]) -> ([f64; N_ARGS], [f64; N_ARGS]) {
    let mut a_arg = [0.0_f64; N_ARGS];
    let mut adot_arg = [0.0_f64; N_ARGS];
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
fn reduce_all_args_position(args: &Args, t_pow: &[f64; 5]) -> [f64; N_ARGS] {
    let mut a_arg = [0.0_f64; N_ARGS];
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

// ─── Phase tables and series accumulation ───────────────────────────────────────

/// Which of the three lunar variables a series belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Var {
    V,
    U,
    R,
}

impl Var {
    fn main_terms(self) -> &'static [ElpMainTerm] {
        match self {
            Var::V => moon_longitude::MAIN.as_slice(),
            Var::U => moon_latitude::MAIN.as_slice(),
            Var::R => moon_distance::MAIN.as_slice(),
        }
    }

    /// The four power-grouped perturbation tables `t^0 ..= t^3`.
    fn pert_groups(self) -> [&'static [ElpPertTerm]; 4] {
        match self {
            Var::V => [
                moon_longitude::PERT_0.as_slice(),
                moon_longitude::PERT_1.as_slice(),
                moon_longitude::PERT_2.as_slice(),
                moon_longitude::PERT_3.as_slice(),
            ],
            Var::U => [
                moon_latitude::PERT_0.as_slice(),
                moon_latitude::PERT_1.as_slice(),
                moon_latitude::PERT_2.as_slice(),
                moon_latitude::PERT_3.as_slice(),
            ],
            Var::R => [
                moon_distance::PERT_0.as_slice(),
                moon_distance::PERT_1.as_slice(),
                moon_distance::PERT_2.as_slice(),
                moon_distance::PERT_3.as_slice(),
            ],
        }
    }

    /// Scale from the table's unit to the series' output unit: V and U are
    /// tabulated in arcseconds and returned in radians, r in km both ways.
    fn pert_scale(self) -> f64 {
        match self {
            Var::V | Var::U => 1.0 / ARCSEC_PER_RAD,
            Var::R => 1.0,
        }
    }
}

/// One power-grouped perturbation table, ready to evaluate.
struct PertGroup {
    /// Pool id of each term's `(sin φ, cos φ)` in the [`PhaseGraph`].
    node: Vec<u32>,
    /// `(s, c)` with the series' unit scale already applied (`term.s * scale`
    /// is jd-independent and used to be redone per term per call).
    sc: Vec<Pair>,
    /// Index of each entry in the source table, which the sort by node
    /// scrambled. Only the tests need it, to find a term's multipliers.
    #[cfg(test)]
    source: Vec<usize>,
}

impl PertGroup {
    /// The group with its terms in pool-id order.
    ///
    /// The order of terms within a group is free (the group is summed), and
    /// ordering them by id makes the loop that reads the pool walk it forward
    /// instead of jumping across 450 KB of pairs.
    fn sorted_by_node(node: Vec<u32>, sc: Vec<Pair>) -> Self {
        let mut order: Vec<usize> = (0..node.len()).collect();
        order.sort_by_key(|&i| node[i]);
        Self {
            node: order.iter().map(|&i| node[i]).collect(),
            sc: order.iter().map(|&i| sc[i]).collect(),
            #[cfg(test)]
            source: order,
        }
    }
}

/// Every term of one variable, as pool ids.
struct VarTerms {
    /// Pool id per main-problem term, parallel to the main table and to its
    /// per-fit [`MainAmplitudes`].
    main: Vec<u32>,
    pert: [PertGroup; 4],
}

impl VarTerms {
    fn empty() -> Self {
        let group = || PertGroup {
            node: Vec::new(),
            sc: Vec::new(),
            #[cfg(test)]
            source: Vec::new(),
        };
        Self {
            main: Vec::new(),
            pert: [group(), group(), group(), group()],
        }
    }
}

/// The phase graph over a set of variables' terms, and each term's pool id.
///
/// See `phase_graph` for the method and its accuracy. Two instances exist:
/// [`TABLES_ALL`] (V, U and r, which share most of their phases) and
/// [`TABLES_R`] (r alone, for [`elp_geocentric_distance`], which must not pay
/// for the nodes only the other two variables need). The second is the first
/// with the unneeded nodes dropped, and a node keeps its parent chain, so the
/// two agree to the bit on every r term.
struct LunarTables {
    graph: PhaseGraph,
    v: VarTerms,
    u: VarTerms,
    r: VarTerms,
}

fn multiplier(m: i32) -> i16 {
    i16::try_from(m).expect("an ELP multiplier does not fit in i16")
}

impl LunarTables {
    /// The graph over all three variables.
    fn build() -> Self {
        // One multiplier vector per term, in a fixed order the ids are read
        // back in below: per variable (V, U, R), main terms then the four
        // perturbation groups.
        let mut vectors: Vec<Multipliers> = Vec::new();
        for var in [Var::V, Var::U, Var::R] {
            for t in var.main_terms() {
                let mut m = [0_i16; N_ARGS];
                m[..4].copy_from_slice(&[
                    multiplier(t.i1),
                    multiplier(t.i2),
                    multiplier(t.i3),
                    multiplier(t.i4),
                ]);
                vectors.push(m);
            }
            for group in var.pert_groups() {
                for t in group {
                    vectors.push(dense_multipliers_of(t).map(multiplier));
                }
            }
        }
        let (graph, ids) = PhaseGraph::build(&vectors);

        let mut cursor = 0;
        let mut take = |n: usize| -> Vec<u32> {
            let taken = ids[cursor..cursor + n].to_vec();
            cursor += n;
            taken
        };
        let mut terms_of = |var: Var| -> VarTerms {
            let main = take(var.main_terms().len());
            let scale = var.pert_scale();
            let pert = var.pert_groups().map(|group| {
                let node = take(group.len());
                let sc: Vec<Pair> = group.iter().map(|t| [t.s * scale, t.c * scale]).collect();
                PertGroup::sorted_by_node(node, sc)
            });
            VarTerms { main, pert }
        };
        let v = terms_of(Var::V);
        let u = terms_of(Var::U);
        let r = terms_of(Var::R);
        debug_assert_eq!(cursor, ids.len());
        Self { graph, v, u, r }
    }

    /// The same tables with only the nodes the distance series needs.
    fn r_only(&self) -> Self {
        let roots: Vec<u32> = self
            .r
            .main
            .iter()
            .chain(self.r.pert.iter().flat_map(|g| &g.node))
            .copied()
            .collect();
        let (graph, ids) = self.graph.pruned(&roots);
        let mut next = ids.into_iter();
        let mut renumber =
            |old: &[u32]| -> Vec<u32> { old.iter().map(|_| next.next().unwrap()).collect() };
        let main = renumber(&self.r.main);
        let pert = [0, 1, 2, 3].map(|n| PertGroup {
            node: renumber(&self.r.pert[n].node),
            sc: self.r.pert[n].sc.clone(),
            #[cfg(test)]
            source: self.r.pert[n].source.clone(),
        });
        Self {
            graph,
            v: VarTerms::empty(),
            u: VarTerms::empty(),
            r: VarTerms { main, pert },
        }
    }
}

/// The 13 multipliers of one perturbation term, in `i1..i13` order.
#[inline]
fn dense_multipliers_of(term: &ElpPertTerm) -> [i32; N_ARGS] {
    [
        term.i1, term.i2, term.i3, term.i4, term.i5, term.i6, term.i7, term.i8, term.i9, term.i10,
        term.i11, term.i12, term.i13,
    ]
}

static TABLES_ALL: LazyLock<LunarTables> = LazyLock::new(LunarTables::build);
static TABLES_R: LazyLock<LunarTables> = LazyLock::new(|| TABLES_ALL.r_only());

/// Scratch the phase graph is evaluated into: one `(sin, cos)` pair and one ω
/// per pool id.
///
/// About 450 KB and 220 KB for [`TABLES_ALL`]. Kept per thread and reused,
/// because allocating that on every evaluation costs more than it saves (the
/// pages are faulted in fresh each time), and it is too large for the stack:
/// `search_muhurta` runs evaluations on worker threads with the default stack.
struct Workspace {
    pool: Vec<Pair>,
    omega: Vec<f64>,
}

thread_local! {
    static WORKSPACE: RefCell<Workspace> = const {
        RefCell::new(Workspace { pool: Vec::new(), omega: Vec::new() })
    };
}

/// Run `f` with this thread's workspace, grown to at least `len` entries.
fn with_workspace<T>(len: usize, f: impl FnOnce(&mut Workspace) -> T) -> T {
    WORKSPACE.with_borrow_mut(|ws| {
        if ws.pool.len() < len {
            ws.pool.resize(len, [0.0, 0.0]);
            ws.omega.resize(len, 0.0);
        }
        f(ws)
    })
}

/// Evaluate a main-problem series (S1, S2, or S3) from its terms' pool ids.
///
/// `amps` carries the per-term corrected amplitude already in the series'
/// output unit (see [`MainAmplitudes`]) and must be parallel to `ids`.
///
/// `VELOCITY` selects whether `dot` is formed; `value` is formed by the same
/// operations either way, so the two instantiations return the same `value`
/// to the bit. `omega` is read only when `VELOCITY` is true.
///
/// # Summation
///
/// Four independent accumulators, combined `(0 + 1) + (2 + 3)` and then the
/// at most three leftover terms added in order. One accumulator would make
/// every term wait on the add before it, which now that a term costs a few
/// instructions and not a `sincos` is the slowest thing in the loop.
fn eval_main_series<const VELOCITY: bool>(
    ids: &[u32],
    amps: &[f64],
    pool: &[Pair],
    omega: &[f64],
    kind: SeriesKind,
) -> SeriesPart {
    assert_eq!(
        ids.len(),
        amps.len(),
        "main-series amplitude table is not parallel to its term table"
    );
    // Sine series:   value = Σ a·sin φ,  dot = +Σ a·ω·cos φ.
    // Cosine series: value = Σ a·cos φ,  dot = −Σ a·ω·sin φ.
    let (value_col, dot_col, dot_sign) = match kind {
        SeriesKind::Sine => (0, 1, 1.0),
        SeriesKind::Cosine => (1, 0, -1.0),
    };

    let mut value = [0.0_f64; 4];
    let mut dot = [0.0_f64; 4];
    let (id_chunks, id_rest) = ids.as_chunks::<4>();
    let (amp_chunks, amp_rest) = amps.as_chunks::<4>();
    for (ic, ac) in id_chunks.iter().zip(amp_chunks) {
        for (lane, (&id, &a)) in ic.iter().zip(ac).enumerate() {
            let id = id as usize;
            let p = pool[id];
            value[lane] += a * p[value_col];
            if VELOCITY {
                dot[lane] += a * omega[id] * p[dot_col];
            }
        }
    }
    let mut v = (value[0] + value[1]) + (value[2] + value[3]);
    let mut d = (dot[0] + dot[1]) + (dot[2] + dot[3]);
    for (&id, &a) in id_rest.iter().zip(amp_rest) {
        let id = id as usize;
        let p = pool[id];
        v += a * p[value_col];
        if VELOCITY {
            d += a * omega[id] * p[dot_col];
        }
    }

    SeriesPart {
        value: v,
        dot: dot_sign * d,
    }
}

/// `(Σ s·sin φ + c·cos φ,  Σ ω·(s·cos φ − c·sin φ))` over one perturbation
/// group. The second is 0 unless `VELOCITY`.
///
/// Summed with four accumulators like [`eval_main_series`], and with the same
/// guarantee: the first sum is formed identically with or without `VELOCITY`.
fn eval_pert_group<const VELOCITY: bool>(
    group: &PertGroup,
    pool: &[Pair],
    omega: &[f64],
) -> (f64, f64) {
    let mut inner_sum = [0.0_f64; 4];
    let mut omega_sum = [0.0_f64; 4];
    let (id_chunks, id_rest) = group.node.as_chunks::<4>();
    let (sc_chunks, sc_rest) = group.sc.as_chunks::<4>();
    for (ic, scc) in id_chunks.iter().zip(sc_chunks) {
        for (lane, (&id, &[s, c])) in ic.iter().zip(scc).enumerate() {
            let id = id as usize;
            let p = pool[id];
            inner_sum[lane] += s * p[0] + c * p[1];
            if VELOCITY {
                let inner_dot = s * p[1] - c * p[0];
                omega_sum[lane] += omega[id] * inner_dot;
            }
        }
    }
    let mut inner = (inner_sum[0] + inner_sum[1]) + (inner_sum[2] + inner_sum[3]);
    let mut w = (omega_sum[0] + omega_sum[1]) + (omega_sum[2] + omega_sum[3]);
    for (&id, &[s, c]) in id_rest.iter().zip(sc_rest) {
        let id = id as usize;
        let p = pool[id];
        inner += s * p[0] + c * p[1];
        if VELOCITY {
            let inner_dot = s * p[1] - c * p[0];
            w += omega[id] * inner_dot;
        }
    }
    (inner, w)
}

/// Evaluate the four power-grouped perturbation series for one variable.
/// Each table entry is `(S, C, i1..i13)` and is evaluated as
/// `t^n · (S sin φ + C cos φ)`, with φ the term's integer combination of the
/// arguments, whose sine and cosine are read from the pool.
///
/// `VELOCITY` selects whether `dot` is formed; `value` is formed by the same
/// operations either way.
///
/// ```text
/// f(t)  = t^n · (s sin φ + c cos φ)
/// f'(t) = n·t^{n-1}·(s sin φ + c cos φ) + t^n · ω · (s cos φ − c sin φ)
/// ```
fn eval_pert_series<const VELOCITY: bool>(
    var: &VarTerms,
    t_pow: &[f64; 5],
    pool: &[Pair],
    omega: &[f64],
) -> SeriesPart {
    let mut value = 0.0;
    let mut dot = 0.0;
    for (n, group) in var.pert.iter().enumerate() {
        let tn = t_pow[n];
        let (inner, w) = eval_pert_group::<VELOCITY>(group, pool, omega);
        value += tn * inner;
        if VELOCITY {
            let dtn = if n >= 1 {
                (n as f64) * t_pow[n - 1]
            } else {
                0.0
            };
            dot += dtn * inner + tn * w;
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

    /// Epochs used by the phase-graph tests: both ends of the provider's
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

    // ─── Phase-graph tests ──────────────────────────────────────────────────

    /// `2π` as an unevaluated sum of two doubles, for reducing a phase of
    /// order 10⁶ rad without losing the digits the comparison is about.
    const TAU_HI: f64 = core::f64::consts::TAU;
    /// `2π − TAU_HI`, twice QD's `π_lo` (`1.224646799147353207e-16`), which
    /// `tau_lo_is_the_remainder_of_tau` checks against libm.
    const TAU_LO: f64 = 2.449_293_598_294_706_4e-16;

    #[test]
    fn tau_lo_is_the_remainder_of_tau() {
        // sin(x) for x within 1e-16 of π is π − x to every digit a double
        // holds, so libm's value of sin(π_hi) *is* π − π_hi.
        assert_eq!(
            libm::sin(core::f64::consts::PI),
            TAU_LO / 2.0,
            "TAU_LO is not twice (π − π_hi)"
        );
    }

    fn two_sum(a: f64, b: f64) -> (f64, f64) {
        let s = a + b;
        let bb = s - a;
        (s, (a - (s - bb)) + (b - bb))
    }

    /// `(sin φ, cos φ)` for `φ = Σ mⱼ·aⱼ`, with the sum carried in
    /// double-double and reduced modulo 2π in extended precision.
    ///
    /// Good to the last place or two of a double at any `|φ|` this crate
    /// reaches, which is what makes it a reference for both the phase-graph
    /// pair and the old route's rounded phase. (`f64::mul_add` here is the
    /// exact error of a product, wherever it runs; it is not a speed choice.)
    fn exact_pair(m: &Multipliers, a: &[f64; N_ARGS]) -> (f64, f64) {
        let (mut hi, mut lo) = (0.0_f64, 0.0_f64);
        for (&mj, &aj) in m.iter().zip(a) {
            if mj == 0 {
                continue;
            }
            let x = f64::from(mj);
            let p = x * aj;
            let p_err = x.mul_add(aj, -p);
            let (s, e) = two_sum(hi, p);
            hi = s;
            lo += e + p_err;
        }
        let (hi, lo) = two_sum(hi, lo);
        let n = (hi / TAU_HI).round();
        let p = n * TAU_HI;
        let p_err = n.mul_add(TAU_HI, -p);
        let d = hi - p; // exact: hi and p agree to within π
        let small = lo - p_err - n * TAU_LO;
        let (s, c) = libm::sincos(d);
        (s + small * c, c - small * s)
    }

    /// The old route: the phase summed in plain `f64`, then one libm `sincos`.
    fn dense_pair(m: &Multipliers, a: &[f64; N_ARGS]) -> (f64, f64) {
        let phase: f64 = m.iter().zip(a).map(|(&m, &a)| f64::from(m) * a).sum();
        libm::sincos(phase)
    }

    /// Every term's multiplier vector with its pool id, for one table set.
    fn all_term_vectors(tables: &LunarTables, vars: &[Var]) -> Vec<(Multipliers, u32)> {
        let mut out = Vec::new();
        for &var in vars {
            let terms = match var {
                Var::V => &tables.v,
                Var::U => &tables.u,
                Var::R => &tables.r,
            };
            for (t, &id) in var.main_terms().iter().zip(&terms.main) {
                let mut m = [0_i16; N_ARGS];
                m[..4].copy_from_slice(&[
                    multiplier(t.i1),
                    multiplier(t.i2),
                    multiplier(t.i3),
                    multiplier(t.i4),
                ]);
                out.push((m, id));
            }
            for (group, ids) in var.pert_groups().iter().zip(&terms.pert) {
                assert_eq!(group.len(), ids.node.len());
                for (&source, &id) in ids.source.iter().zip(&ids.node) {
                    out.push((dense_multipliers_of(&group[source]).map(multiplier), id));
                }
            }
        }
        out
    }

    /// The facts the phase-graph design is justified by, re-measured from the
    /// loaded tables rather than taken from a document.
    #[test]
    fn elp_tables_have_the_sharing_the_design_relies_on() {
        use std::collections::HashSet;

        let tables = &*TABLES_ALL;
        let terms = all_term_vectors(tables, &[Var::V, Var::U, Var::R]);
        assert_eq!(terms.len(), 35_758, "lunar term count moved");

        let unique: HashSet<Multipliers> = terms.iter().map(|(m, _)| *m).collect();
        assert_eq!(unique.len(), 21_325, "distinct multiplier vectors moved");

        let mut max_abs = 0_u16;
        let mut max_nonzero = 0_usize;
        for (m, _) in &terms {
            max_nonzero = max_nonzero.max(m.iter().filter(|&&x| x != 0).count());
            for &x in m {
                max_abs = max_abs.max(x.unsigned_abs());
            }
        }
        assert_eq!(max_abs, 75, "largest |multiplier| moved");
        assert_eq!(max_nonzero, 7, "most non-zero multipliers in a term moved");

        assert_eq!(
            tables.graph.entry_count(),
            394,
            "(argument, multiplier) pairs moved"
        );
        // 27,721 prefix-tree nodes, of which 117 are single-pair prefixes and so
        // are table entries rather than rotations.
        assert_eq!(
            tables.graph.derived_count(),
            27_604,
            "rotations per evaluation moved"
        );
        // The design replaces 35,758 sincos calls by these rotations; if the
        // tree ever stops being smaller than the term list the method has
        // stopped paying.
        assert!(tables.graph.derived_count() < terms.len());
    }

    /// Every term's `(sin φ, cos φ)` from the pool against the exact value, at
    /// 13 epochs spanning the supported range, with the old route's error
    /// alongside.
    ///
    /// The bound is on the new route. The old route is measured, and asserted
    /// to be worse only where its rounded phase is large enough that it must be.
    #[test]
    fn pool_pairs_match_the_exact_pair_for_every_term() {
        let tables = &*TABLES_ALL;
        let terms = all_term_vectors(tables, &[Var::V, Var::U, Var::R]);
        let mut pool = vec![[0.0; 2]; tables.graph.pool_len()];

        let mut worst_new = 0.0_f64;
        for t in sparse_test_epochs() {
            let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];
            let (a_arg, _) = reduce_all_args(args_for(Fit::Llr), &t_pow);
            tables.graph.fill_phases(&a_arg, &mut pool);

            let (mut new_err, mut old_err) = (0.0_f64, 0.0_f64);
            for (m, id) in &terms {
                let (es, ec) = exact_pair(m, &a_arg);
                let got = pool[*id as usize];
                new_err = new_err.max((got[0] - es).abs()).max((got[1] - ec).abs());
                let (ds, dc) = dense_pair(m, &a_arg);
                old_err = old_err.max((ds - es).abs()).max((dc - ec).abs());
            }
            println!(
                "t = {t:8.3} cy: pool-vs-exact {new_err:.2e}, dense-sincos-vs-exact {old_err:.2e}"
            );
            worst_new = worst_new.max(new_err);
            if t.abs() >= 20.0 {
                assert!(
                    new_err < old_err,
                    "t = {t}: the pool ({new_err:e}) should beat the rounded dense phase ({old_err:e})"
                );
            }
        }
        // A ladder rung `k` inherits `k` times the angle error of its
        // `sincos(A)` (about 1.1e-16), so the largest multiplier, 75, is worth
        // ~8e-15 by itself (measured 8.1e-15); the rotations on top add about
        // an ulp each, six at the deepest node. Terms with large multipliers
        // are the planetary resonances, whose amplitudes are tiny, and the
        // series test below measures the effect on a position.
        assert!(
            worst_new < 1.5e-14,
            "worst pool-vs-exact error {worst_new:e} exceeds 1.5e-14"
        );
    }

    /// `elp_geocentric_distance` runs on the r-only graph; its pairs must equal
    /// the full graph's, to the bit, for every r term.
    #[test]
    fn r_only_graph_agrees_with_the_full_graph_to_the_bit() {
        let all = &*TABLES_ALL;
        let only = &*TABLES_R;
        let mut pool_all = vec![[0.0; 2]; all.graph.pool_len()];
        let mut pool_r = vec![[0.0; 2]; only.graph.pool_len()];
        assert!(
            only.graph.pool_len() < all.graph.pool_len(),
            "the r-only graph is not smaller, so it saves nothing"
        );
        assert!(only.v.main.is_empty() && only.u.main.is_empty());
        for t in sparse_test_epochs() {
            let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];
            let (a_arg, _) = reduce_all_args(args_for(Fit::Llr), &t_pow);
            all.graph.fill_phases(&a_arg, &mut pool_all);
            only.graph.fill_phases(&a_arg, &mut pool_r);
            let a_ids: Vec<u32> = all
                .r
                .main
                .iter()
                .chain(all.r.pert.iter().flat_map(|g| &g.node))
                .copied()
                .collect();
            let r_ids: Vec<u32> = only
                .r
                .main
                .iter()
                .chain(only.r.pert.iter().flat_map(|g| &g.node))
                .copied()
                .collect();
            assert_eq!(a_ids.len(), r_ids.len());
            for (&ia, &ir) in a_ids.iter().zip(&r_ids) {
                let (x, y) = (pool_all[ia as usize], pool_r[ir as usize]);
                assert_eq!(x[0].to_bits(), y[0].to_bits(), "t = {t}: sin differs");
                assert_eq!(x[1].to_bits(), y[1].to_bits(), "t = {t}: cos differs");
            }
        }
    }

    /// Reference series evaluation: every term's pair from [`exact_pair`], ω
    /// by a dense dot product, summed in plain order. Returns
    /// `(V, U, r, V̇, U̇, ṙ)` as [`vur_series`] does.
    fn reference_vur(jd: f64, fit: Fit) -> (f64, f64, f64, f64, f64, f64) {
        let args = args_for(fit);
        let t = (jd - J2000) / DAYS_PER_CENTURY;
        let t_pow = [1.0, t, t * t, t * t * t, t * t * t * t];
        let (a_arg, adot_arg) = reduce_all_args(args, &t_pow);
        let omega_of =
            |m: &Multipliers| -> f64 { (0..N_ARGS).map(|j| f64::from(m[j]) * adot_arg[j]).sum() };

        // Returns (main value, main dot, pert value, pert dot), so the caller
        // can associate the final sum exactly as `vur_series` does.
        let one = |var: Var, amps: &[f64], kind: SeriesKind| -> (f64, f64, f64, f64) {
            let (mut value, mut dot) = (0.0, 0.0);
            for (term, &a) in var.main_terms().iter().zip(amps) {
                let mut m = [0_i16; N_ARGS];
                m[..4].copy_from_slice(&[
                    multiplier(term.i1),
                    multiplier(term.i2),
                    multiplier(term.i3),
                    multiplier(term.i4),
                ]);
                let (s, c) = exact_pair(&m, &a_arg);
                let w = omega_of(&m);
                match kind {
                    SeriesKind::Sine => {
                        value += a * s;
                        dot += a * w * c;
                    }
                    SeriesKind::Cosine => {
                        value += a * c;
                        dot += -a * w * s;
                    }
                }
            }
            let (mut pvalue, mut pdot) = (0.0, 0.0);
            let scale = var.pert_scale();
            for (n, group) in var.pert_groups().iter().enumerate() {
                let tn = t_pow[n];
                let dtn = if n >= 1 {
                    (n as f64) * t_pow[n - 1]
                } else {
                    0.0
                };
                for term in *group {
                    let m = dense_multipliers_of(term).map(multiplier);
                    let (sn, cs) = exact_pair(&m, &a_arg);
                    let w = omega_of(&m);
                    let (s, c) = (term.s * scale, term.c * scale);
                    let inner = s * sn + c * cs;
                    let inner_dot = s * cs - c * sn;
                    pvalue += tn * inner;
                    pdot += dtn * inner + tn * w * inner_dot;
                }
            }
            (value, dot, pvalue, pdot)
        };

        let (vm, vmd, vp, vpd) = one(Var::V, V_MAIN_AMPS.get(fit), SeriesKind::Sine);
        let (um, umd, up, upd) = one(Var::U, U_MAIN_AMPS.get(fit), SeriesKind::Sine);
        let (rm, rmd, rp, rpd) = one(Var::R, R_MAIN_AMPS.get(fit), SeriesKind::Cosine);
        (
            eval_poly(&args.w1, t) + vm + vp,
            um + up,
            (rm + rp) * RA0,
            eval_poly_dot(&args.w1, t) + vmd + vpd,
            umd + upd,
            (rmd + rpd) * RA0,
        )
    }

    /// The production series against the exact-phase reference, over the
    /// supported range.
    ///
    /// Bound: the pool's pair error (≤ 1.5e-14, `pool_pairs_match_...`) times
    /// the sum of the amplitudes (V, U ≲ 4e4 arcsec = 2e-1 rad; r ≲ 1e5 km) is
    /// ≲ 3e-9 arcsec and ≲ 1.5e-9 km; the stated bounds below add summation
    /// rounding. The bar this change was held to is 1e-6 arcsec.
    ///
    /// `V` is the unwrapped longitude `W₁ + …`, up to 3.4e5 rad at the range
    /// edge, so one ulp of it is 5.8e-11 rad = 1.2e-5 arcsec there: a sum that
    /// lands on the other side of a rounding boundary differs by that much and
    /// no more, whichever way it is formed. The tolerance for V is two such ulps
    /// plus the series bound; U and r have no such term.
    #[test]
    fn series_match_the_exact_phase_reference() {
        let arcsec = |rad: f64| rad * ARCSEC_PER_RAD;
        let ulp = |x: f64| f64::from_bits(x.abs().to_bits() + 1) - x.abs();
        let mut worst = [0.0_f64; 3];
        for t in sparse_test_epochs() {
            for frac in [0.0, 0.37] {
                let jd = J2000 + (t + frac * 0.01) * DAYS_PER_CENTURY;
                let (v, u, r, vd, ud, rd) = vur_series(jd, Fit::Llr);
                let (rv, ru, rr, rvd, rud, rrd) = reference_vur(jd, Fit::Llr);
                let v_tol = arcsec(2.0 * ulp(v)) + 3e-9;
                assert!(
                    arcsec((v - rv).abs()) <= v_tol,
                    "t={t}: V {v:e} vs {rv:e}, tolerance {v_tol:e} arcsec"
                );
                worst[0] = worst[0].max(arcsec((v - rv).abs()));
                worst[1] = worst[1].max(arcsec((u - ru).abs()));
                worst[2] = worst[2].max((r - rr).abs());
                // Velocity: relative to the size of the derivative.
                assert!(
                    (vd - rvd).abs() < 1e-9 * rvd.abs(),
                    "t={t}: dV/dt {vd} vs {rvd}"
                );
                assert!(
                    (ud - rud).abs() < 1e-9 * rvd.abs(),
                    "t={t}: dU/dt {ud} vs {rud}"
                );
                assert!(
                    (rd - rrd).abs() < 1e-9 * rrd.abs().max(1.0),
                    "t={t}: dr/dt {rd} vs {rrd}"
                );
            }
        }
        println!(
            "worst vs exact-phase reference: V {:.2e} arcsec, U {:.2e} arcsec, r {:.2e} km",
            worst[0], worst[1], worst[2]
        );
        assert!(worst[1] < 3e-9, "U off by {:e} arcsec", worst[1]);
        assert!(worst[2] < 3e-9, "r off by {:e} km", worst[2]);
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

    /// Wrap a longitude difference into `(-180, 180]`.
    fn wrap180(d: f64) -> f64 {
        let r = d.rem_euclid(360.0);
        if r > 180.0 { r - 360.0 } else { r }
    }

    #[test]
    fn mean_longitude_of_date_at_j2000_is_w1_and_its_rate() {
        // At tau = 0 the precession shift vanishes, so the value is W1's
        // constant: 218 deg 18' (59.95571 - 0.10525)" for the LLR fit.
        let (lon, rate) = mean_longitude_of_date(J2000);
        let expected = 218.0 + 18.0 / 60.0 + (59.95571 - 0.10525) / 3600.0;
        assert!((lon - expected).abs() < 1e-9, "{lon} != {expected}");
        // Rate: (1_732_559_343.73604 - 0.32311 + 5029.0966 - 0.29965) arcsec
        // per century, over 36525 days.
        let expected_rate =
            (1_732_559_343.73604 - 0.32311 + 5029.0966 - 0.29965) / 3600.0 / DAYS_PER_CENTURY;
        assert!(
            (rate - expected_rate).abs() < 1e-9,
            "{rate} != {expected_rate}"
        );
        // And it is the Moon's familiar 13.176 deg/day.
        assert!((rate - 13.1764).abs() < 1e-4);
    }

    #[test]
    fn mean_longitude_of_date_rate_is_the_derivative_of_the_longitude() {
        for jd in [2_415_020.5, 2_440_000.5, J2000, 2_460_000.5, 2_488_069.5] {
            let (_, rate) = mean_longitude_of_date(jd);
            let (a, _) = mean_longitude_of_date(jd - 0.5);
            let (b, _) = mean_longitude_of_date(jd + 0.5);
            let numeric = wrap180(b - a);
            // 13.18 deg/day against a 1-day difference of a smooth
            // polynomial: only rounding and a vanishing cubic term remain.
            assert!(
                (numeric - rate).abs() < 1e-8,
                "jd {jd}: analytic {rate} vs difference {numeric}"
            );
        }
    }

    #[test]
    fn mean_longitude_of_date_is_the_mean_the_true_longitude_swings_about() {
        // Same frame (equinox of date) by construction: the true longitude
        // from the series must stay within the Moon's total periodic swing of
        // the mean (equation of the centre 6.3 deg, evection 1.3 deg,
        // variation 0.7 deg, plus smaller terms) -- and must actually use
        // that swing, or the test would pass for any constant near it.
        let mut max_abs = 0.0_f64;
        let mut saw_ahead = false;
        let mut saw_behind = false;
        let mut jd = 2_415_020.5;
        while jd < 2_488_069.5 {
            let (x, y, _) = elp_geocentric_position_of_date(jd);
            let true_lon = libm::atan2(y, x).to_degrees().rem_euclid(360.0);
            let (mean, _) = mean_longitude_of_date(jd);
            let d = wrap180(true_lon - mean);
            assert!(d.abs() < 8.5, "jd {jd}: true - mean = {d} deg");
            max_abs = max_abs.max(d.abs());
            saw_ahead |= d > 3.0;
            saw_behind |= d < -3.0;
            jd += 37.3;
        }
        assert!(max_abs > 6.0, "swing never exceeded {max_abs} deg");
        assert!(saw_ahead && saw_behind);
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

    /// [`elp_geocentric_distance`] must be bit-for-bit the `r` that
    /// [`vur_series_position`] computes, across the full supported JD range.
    #[test]
    fn elp_geocentric_distance_matches_the_position_series_r_bit_for_bit() {
        for t in sparse_test_epochs() {
            let jd = J2000 + t * DAYS_PER_CENTURY;
            let (_, _, r) = vur_series_position(jd, Fit::Llr);
            let d = elp_geocentric_distance(jd);
            assert_eq!(d.to_bits(), r.to_bits(), "t={t} jd={jd}: {d:e} vs {r:e}");
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
        use crate::analytical::simd_trig::sincos_f64x4;
        use crate::analytical::{JD_MAX, JD_MIN};
        use wide::f64x4;

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
