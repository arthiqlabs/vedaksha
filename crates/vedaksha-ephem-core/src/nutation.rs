// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! IAU 2000B nutation model (77-term truncated series).
//!
//! Computes nutation in longitude (dpsi) and nutation in obliquity (deps).
//!
//! Source: IERS Conventions (2010) Ch. 5; originally from the abridged
//! precession-nutation model by `McCarthy` & Luzum in *Celestial Mechanics*
//! 85 (2003), pp. 37--49.

use crate::julian;
use vedaksha_math::angle::deg_to_rad;
use vedaksha_math::matrix::Matrix3;

/// IAU 2000B nutation series: 77 terms.
///
/// Each row: `[l, l', F, D, Om, Sp, Spt, Cp, Ce, Cet, Se]`
///
/// - `dpsi += (Sp + Spt*T)*sin(arg) + Cp*cos(arg)`
/// - `deps += (Ce + Cet*T)*cos(arg) + Se*sin(arg)`
///
/// All coefficient values are in units of 0.1 microarcseconds.
#[rustfmt::skip]
static IAU2000B_NUTATION: [[i64; 11]; 77] = [
    [ 0,  0,  0,  0,  1, -172_064_161, -174_666,  33_386,  92_052_331,  9_086,  15_377],
    [ 0,  0,  2, -2,  2,  -13_170_906,   -1_675, -13_696,   5_730_336, -3_015,  -4_587],
    [ 0,  0,  2,  0,  2,   -2_276_413,     -234,   2_796,     978_459,   -485,   1_374],
    [ 0,  0,  0,  0,  2,    2_074_554,      207,    -698,    -897_492,    470,    -291],
    [ 0,  1,  0,  0,  0,    1_475_877,   -3_633,  11_817,      73_871,   -184,  -1_924],
    [ 0,  1,  2, -2,  2,     -516_821,    1_226,    -524,     224_386,   -677,    -174],
    [ 1,  0,  0,  0,  0,      711_159,       73,    -872,      -6_750,      0,     358],
    [ 0,  0,  2,  0,  1,     -387_298,     -367,     380,     200_728,     18,     318],
    [ 1,  0,  2,  0,  2,     -301_461,      -36,     816,     129_025,    -63,     367],
    [ 0, -1,  2, -2,  2,      215_829,     -494,     111,     -95_929,    299,     132],
    [ 0,  0,  2, -2,  1,      128_227,      137,     181,     -68_982,     -9,      39],
    [-1,  0,  2,  0,  2,      123_457,       11,      19,     -53_311,     32,      -4],
    [-1,  0,  0,  2,  0,      156_994,       10,    -168,      -1_235,      0,      82],
    [ 1,  0,  0,  0,  1,       63_110,       63,      27,     -33_228,      0,      -9],
    [-1,  0,  0,  0,  1,      -57_976,      -63,    -189,      31_429,      0,     -75],
    [-1,  0,  2,  2,  2,      -59_641,      -11,     149,      25_543,    -11,      66],
    [ 1,  0,  2,  0,  1,      -51_613,      -42,     129,      26_366,      0,      78],
    [-2,  0,  2,  0,  1,       45_893,       50,      31,     -24_236,    -10,      20],
    [ 0,  0,  0,  2,  0,       63_384,       11,    -150,      -1_220,      0,      29],
    [ 0,  0,  2,  2,  2,      -38_571,       -1,     158,      16_452,    -11,      68],
    [ 0, -2,  2, -2,  2,       32_481,        0,       0,     -13_870,      0,       0],
    [-2,  0,  0,  2,  0,      -47_722,        0,     -18,         477,      0,     -25],
    [ 2,  0,  2,  0,  2,      -31_046,       -1,     131,      13_238,    -11,      59],
    [ 1,  0,  2, -2,  2,       28_593,        0,      -1,     -12_338,     10,      -3],
    [-1,  0,  2,  0,  1,       20_441,       21,      10,     -10_758,      0,      -3],
    [ 2,  0,  0,  0,  0,       29_243,        0,     -74,        -609,      0,      13],
    [ 0,  0,  2,  0,  0,       25_887,        0,     -66,        -550,      0,      11],
    [ 0,  1,  0,  0,  1,      -14_053,      -25,      79,       8_551,     -2,     -45],
    [-1,  0,  0,  2,  1,       15_164,       10,      11,      -8_001,      0,      -1],
    [ 0,  2,  2, -2,  2,      -15_794,       72,     -16,       6_850,    -42,      -5],
    [ 0,  0, -2,  2,  0,       21_783,        0,      13,        -167,      0,      13],
    [ 1,  0,  0, -2,  1,      -12_873,      -10,     -37,       6_953,      0,     -14],
    [ 0, -1,  0,  0,  1,      -12_654,       11,      63,       6_415,      0,      26],
    [-1,  0,  2,  2,  1,      -10_204,        0,      25,       5_222,      0,      15],
    [ 0,  2,  0,  0,  0,       16_707,      -85,     -10,         168,     -1,      10],
    [ 1,  0,  2,  2,  2,       -7_691,        0,      44,       3_268,      0,      19],
    [-2,  0,  2,  0,  0,      -11_024,        0,     -14,         104,      0,       2],
    [ 0,  1,  2,  0,  2,        7_566,      -21,     -11,      -3_250,      0,       5],
    [ 0,  0,  2,  2,  1,       -6_637,      -11,      25,       3_353,      0,      14],
    [ 0, -1,  2,  0,  2,       -7_141,       21,       8,       3_070,      0,       4],
    [ 0,  0,  0,  2,  1,       -6_302,      -11,       2,       3_272,      0,       4],
    [ 1,  0,  2, -2,  1,        5_800,       10,       2,      -3_045,      0,      -1],
    [ 2,  0,  2, -2,  2,        6_443,        0,      -7,      -2_768,      0,      -4],
    [-2,  0,  0,  2,  1,       -5_774,      -11,     -15,       3_041,      0,      -5],
    [ 2,  0,  2,  0,  1,       -5_350,        0,      21,       2_695,      0,      12],
    [ 0, -1,  2, -2,  1,       -4_752,      -11,      -3,       2_719,      0,      -3],
    [ 0,  0,  0, -2,  1,       -4_940,      -11,     -21,       2_720,      0,      -9],
    [-1, -1,  0,  2,  0,        7_350,        0,      -8,         -51,      0,       4],
    [ 2,  0,  0, -2,  1,       -4_803,      -11,     -12,       2_244,      0,      -4],
    [ 1,  0,  0,  2,  0,       -4_649,        0,      12,          48,      0,       7],
    [ 0,  1,  2, -2,  1,       -4_355,      -10,       0,       2_158,      0,       3],
    [ 1, -1,  0,  0,  0,       -4_235,        0,       5,          41,      0,       0],
    [-2,  0,  2,  0,  2,       -4_093,        0,       7,       1_756,      0,      -2],
    [ 3,  0,  2,  0,  2,        3_857,        0,      -3,      -1_613,      0,      -1],
    [ 0, -1,  0,  2,  0,        3_747,        0,     -18,          28,      0,      -3],
    [ 1, -1,  2,  0,  2,       -3_655,        0,      11,       1_587,      0,       5],
    [ 0,  0,  0,  1,  0,        3_295,        0,       3,         -32,      0,      -1],
    [-1, -1,  2,  2,  2,       -3_210,        0,       4,       1_405,      0,       3],
    [-1,  0,  2,  0,  0,        3_334,        0,     -12,         -14,      0,       0],
    [ 0, -1,  2,  2,  2,       -3_008,        0,       7,       1_266,      0,       1],
    [-2,  0,  0,  0,  1,       -2_751,        0,      12,       1_395,      0,       2],
    [ 1,  1,  2,  0,  2,       -2_616,        0,       0,       1_131,      0,       2],
    [ 2,  0,  0,  0,  1,        2_354,        0,      -9,      -1_283,      0,      -1],
    [-1,  1,  0,  1,  0,       -2_280,        0,       0,           2,      0,      -1],
    [ 1,  1,  0,  0,  0,        2_177,        0,       2,         -34,      0,       0],
    [ 1,  0,  2,  0,  0,       -2_013,        0,      -4,          32,      0,      -3],
    [-1,  0,  2, -2,  1,        1_926,        0,      -5,      -1_051,      0,      -2],
    [ 1,  0,  0,  0,  2,        1_674,        0,      -4,        -726,      0,       1],
    [-1,  0,  0,  1,  0,       -1_600,        0,      10,          13,      0,       4],
    [ 0,  0,  2,  1,  2,       -1_580,        0,       3,         684,      0,       1],
    [-1,  0,  2,  4,  2,       -1_521,        0,       6,         621,      0,       0],
    [-1,  1,  0,  1,  1,        1_517,        0,      -3,        -810,      0,      -2],
    [ 0, -2,  2, -2,  1,        1_391,        0,       0,        -556,      0,       0],
    [-1,  0,  0,  0,  2,        1_405,        0,       0,        -600,      0,       2],
    [ 1,  0,  2, -1,  2,        1_283,        0,       0,        -543,      0,       2],
    [ 0,  0,  0,  4,  0,        1_270,        0,      -5,         -14,      0,       0],
    [-2,  1,  2,  0,  2,        1_264,        0,       7,        -558,      0,       0],
];

/// Compute IAU 2000B nutation in longitude and obliquity.
///
/// Returns `(dpsi, deps)` in radians, where:
/// - `dpsi` = nutation in longitude
/// - `deps` = nutation in obliquity
///
/// # Arguments
/// * `jd` — Julian Day number (TT or TDB)
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn nutation(jd: f64) -> (f64, f64) {
    let t = julian::centuries_from_j2000(jd);

    // Delaunay fundamental arguments (degrees -> radians)
    let l = deg_to_rad(
        134.963_402_51
            + t * (477_198.867_560_5 + t * (0.008_855_3 + t * (1.0 / 69_699.0 - t / 14_712_000.0))),
    );
    let lp =
        deg_to_rad(357.529_109_18 + t * (35_999.050_291_1 + t * (-0.000_153_7 - t / 24_490_000.0)));
    let f = deg_to_rad(
        93.272_090_62
            + t * (483_202.017_457_7
                + t * (-0.003_682_5 + t * (1.0 / 327_270.0 - t / 12_408_000.0))),
    );
    let d = deg_to_rad(
        297.850_195_47
            + t * (445_267.111_446_9
                + t * (-0.001_769_6 + t * (1.0 / 545_868.0 - t / 113_065_000.0))),
    );
    let om = deg_to_rad(
        125.044_555_01
            + t * (-1_934.136_184_9 + t * (0.002_075_6 + t * (1.0 / 467_441.0 - t / 60_616_000.0))),
    );

    let mut dpsi = 0.0_f64;
    let mut deps = 0.0_f64;

    for term in &IAU2000B_NUTATION {
        let arg = term[0] as f64 * l
            + term[1] as f64 * lp
            + term[2] as f64 * f
            + term[3] as f64 * d
            + term[4] as f64 * om;
        let (sin_arg, cos_arg) = libm::sincos(arg);

        dpsi += (term[5] as f64 + term[6] as f64 * t) * sin_arg + term[7] as f64 * cos_arg;
        deps += (term[8] as f64 + term[9] as f64 * t) * cos_arg + term[10] as f64 * sin_arg;
    }

    // Convert from 0.1 microarcseconds to radians
    // 0.1 μas = 0.1e-6 arcsec = 1e-7 arcsec
    let factor = 1e-7 * core::f64::consts::PI / (180.0 * 3600.0);
    (dpsi * factor, deps * factor)
}

/// ICRS frame bias matrix (ICRS -> mean J2000).
///
/// Accounts for the ~23 milliarcsecond rotation between the ICRS axes
/// and the mean dynamical frame of J2000.0.
///
/// Parameters:
/// - `da0`  = -14.6   mas  (right ascension of the pole)
/// - `xi0`  = -16.617 mas  (x-component of the pole offset)
/// - `eta0` = -6.8192 mas  (y-component of the pole offset)
///
/// The matrix is `B = Rz(da0) * Ry(-xi0) * Rx(-eta0)`, expanded to
/// second order in the small angles.
///
/// Source: IERS Conventions (2010), eq. 5.4.
#[must_use]
pub fn frame_bias_matrix() -> Matrix3 {
    let arcsec_to_rad = core::f64::consts::PI / (180.0 * 3600.0);
    let da0 = -14.6e-3 * arcsec_to_rad;
    let xi0 = -16.617e-3 * arcsec_to_rad;
    let eta0 = -6.8192e-3 * arcsec_to_rad;

    Matrix3 {
        data: [
            [
                1.0 - 0.5 * (da0 * da0 + xi0 * xi0),
                da0,
                -xi0,
            ],
            [
                -da0 - eta0 * xi0,
                1.0 - 0.5 * (da0 * da0 + eta0 * eta0),
                -eta0,
            ],
            [
                xi0 - eta0 * da0,
                eta0 + xi0 * da0,
                1.0 - 0.5 * (eta0 * eta0 + xi0 * xi0),
            ],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::J2000;

    /// Convert arcseconds to radians for comparison.
    fn arcsec_to_rad(arcsec: f64) -> f64 {
        arcsec * core::f64::consts::PI / (180.0 * 3600.0)
    }

    #[test]
    fn nutation_at_j2000() {
        // At J2000 (T=0) the IAU 2000B 77-term series gives
        // dpsi ~ -14.0 arcsec, deps ~ -6.0 arcsec.
        let (dpsi, deps) = nutation(J2000);
        let dpsi_arcsec = dpsi / arcsec_to_rad(1.0);
        let deps_arcsec = deps / arcsec_to_rad(1.0);

        assert!(
            (dpsi_arcsec - (-14.0)).abs() < 1.0,
            "dpsi at J2000: expected ~-14.0 arcsec, got {dpsi_arcsec} arcsec"
        );
        assert!(
            (deps_arcsec - (-6.0)).abs() < 1.0,
            "deps at J2000: expected ~-6.0 arcsec, got {deps_arcsec} arcsec"
        );
    }

    #[test]
    fn nutation_changes_over_time() {
        let (dpsi_0, deps_0) = nutation(J2000);
        let (dpsi_1, deps_1) = nutation(J2000 + 180.0);

        assert!(
            (dpsi_0 - dpsi_1).abs() > 1e-10,
            "dpsi should change over 180 days"
        );
        assert!(
            (deps_0 - deps_1).abs() > 1e-10,
            "deps should change over 180 days"
        );
    }

    #[test]
    fn nutation_within_bounds() {
        // Test at several epochs: values should be within physical bounds
        for &jd in &[J2000, J2000 + 10_000.0, J2000 - 10_000.0, J2000 + 36_525.0] {
            let (dpsi, deps) = nutation(jd);
            let dpsi_arcsec = dpsi.abs() / arcsec_to_rad(1.0);
            let deps_arcsec = deps.abs() / arcsec_to_rad(1.0);

            assert!(
                dpsi.is_finite() && dpsi_arcsec < 30.0,
                "dpsi out of bounds at JD {jd}: {dpsi_arcsec} arcsec"
            );
            assert!(
                deps.is_finite() && deps_arcsec < 15.0,
                "deps out of bounds at JD {jd}: {deps_arcsec} arcsec"
            );
        }
    }

    #[test]
    fn frame_bias_near_identity() {
        let b = frame_bias_matrix();
        // Diagonal elements should be very close to 1.0
        for i in 0..3 {
            assert!(
                (b.data[i][i] - 1.0).abs() < 1e-10,
                "diagonal [{i}][{i}] should be ~1.0, got {}",
                b.data[i][i]
            );
        }
        // Off-diagonal elements should be small (< 1e-4 rad ~ 20 arcsec)
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    assert!(
                        b.data[i][j].abs() < 1e-4,
                        "off-diagonal [{i}][{j}] should be < 1e-4, got {}",
                        b.data[i][j]
                    );
                }
            }
        }
    }

    #[test]
    fn nutation_18_6_year_period() {
        // The dominant period of nutation is ~18.6 years (from lunar node Omega).
        // dpsi at J2000 and J2000 + 18.6*365.25 days should be similar.
        let period_days = 18.6 * 365.25;
        let (dpsi_0, _) = nutation(J2000);
        let (dpsi_1, _) = nutation(J2000 + period_days);

        let diff_arcsec = (dpsi_0 - dpsi_1).abs() / arcsec_to_rad(1.0);
        // After one full dominant period, values should be within ~2 arcseconds
        assert!(
            diff_arcsec < 2.0,
            "dpsi should be similar after ~18.6 year period: diff = {diff_arcsec} arcsec"
        );
    }
}
