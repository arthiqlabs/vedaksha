// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! IAU 2000B nutation model (77-term truncated series).
//!
//! Computes nutation in longitude (dpsi) and nutation in obliquity (deps).
//!
//! Source: IERS Conventions (2010) Ch. 5; originally from the abridged
//! precession-nutation model by `McCarthy` & Luzum in *Celestial Mechanics*
//! 85 (2003), pp. 37--49.
//!
//! # What the series is
//!
//! The 77 luni-solar terms of IAU 2000B are the first 77 rows of the IAU 2000A
//! luni-solar series (Mathews, Herring & Buffett 2002, *J. Geophys. Res.* 107,
//! B4), in the same order and with the same coefficients. The table here was
//! checked row by row against that series as the ERFA library implements it
//! (`src/nut00a.c`, `src/nut00b.c` of liberfa/erfa, retrieved 2026-10-24); its
//! reference vector `eraNut00a(2400000.5, 53736.0)` is a test below.
//!
//! Until that check, 30 of the 77 rows (38 and 49 to 77) held coefficients
//! that are not in the IAU 2000A series at all: wrong in value (17 of them
//! with the sign of the main longitude term reversed, and four rows with the
//! wrong argument multipliers too). They moved Delta psi by up to 6.4 mas
//! (mean 1.6 mas) and Delta epsilon by up to 2.0 mas against the full theory
//! over 1900 to 2100. With the
//! rows corrected the truncation error is the model's own: up to 2.6 mas in
//! Delta psi and 1.3 mas in Delta epsilon (mean 0.4 mas each), measured
//! against the full 1365-term IAU 2000A (free core nutation omitted) at the
//! 2,435 epochs of the Horizons oracle grid.
//!
//! Not modelled: the planetary terms of 2000A (687 of them), the fixed
//! "planetary bias" offsets that 2000B adds in their place (Delta psi -0.135 mas
//! and Delta epsilon +0.388 mas for the rigorous bias-precession-nutation
//! composition this crate uses, after Luzum 2001 as ERFA documents them; they
//! trim the Delta epsilon error to 0.9 mas and leave Delta psi about as it is), and the
//! free core nutation. The full 2000A series would remove the remaining 2.6 mas
//! at about eighteen times the cost per call and is not worth it: an astrological
//! longitude carries Delta T uncertainty of tenths of an arcsecond.

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
    [ 0,  1,  2,  0,  2,        7_566,      -21,     -11,      -3_250,      0,      -5],
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
    [ 2,  0,  0, -2,  1,        4_065,        0,       6,      -2_206,      0,       1],
    [ 1,  0,  0,  2,  0,        6_579,        0,     -24,        -199,      0,       2],
    [ 0,  1,  2, -2,  1,        3_579,        0,       5,      -1_900,      0,       1],
    [ 1, -1,  0,  0,  0,        4_725,        0,      -6,         -41,      0,       3],
    [-2,  0,  2,  0,  2,       -3_075,        0,      -2,       1_313,      0,      -1],
    [ 3,  0,  2,  0,  2,       -2_904,        0,      15,       1_233,      0,       7],
    [ 0, -1,  0,  2,  0,        4_348,        0,     -10,         -81,      0,       2],
    [ 1, -1,  2,  0,  2,       -2_878,        0,       8,       1_232,      0,       4],
    [ 0,  0,  0,  1,  0,       -4_230,        0,       5,         -20,      0,      -2],
    [-1, -1,  2,  2,  2,       -2_819,        0,       7,       1_207,      0,       3],
    [-1,  0,  2,  0,  0,       -4_056,        0,       5,          40,      0,      -2],
    [ 0, -1,  2,  2,  2,       -2_647,        0,      11,       1_129,      0,       5],
    [-2,  0,  0,  0,  1,       -2_294,        0,     -10,       1_266,      0,      -4],
    [ 1,  1,  2,  0,  2,        2_481,        0,      -7,      -1_062,      0,      -3],
    [ 2,  0,  0,  0,  1,        2_179,        0,      -2,      -1_129,      0,      -2],
    [-1,  1,  0,  1,  0,        3_276,        0,       1,          -9,      0,       0],
    [ 1,  1,  0,  0,  0,       -3_389,        0,       5,          35,      0,      -2],
    [ 1,  0,  2,  0,  0,        3_339,        0,     -13,        -107,      0,       1],
    [-1,  0,  2, -2,  1,       -1_987,        0,      -6,       1_073,      0,      -2],
    [ 1,  0,  0,  0,  2,       -1_981,        0,       0,         854,      0,       0],
    [-1,  0,  0,  1,  0,        4_026,        0,    -353,        -553,      0,    -139],
    [ 0,  0,  2,  1,  2,        1_660,        0,      -5,        -710,      0,      -2],
    [-1,  0,  2,  4,  2,       -1_521,        0,       9,         647,      0,       4],
    [-1,  1,  0,  1,  1,        1_314,        0,       0,        -700,      0,       0],
    [ 0, -2,  2, -2,  1,       -1_283,        0,       0,         672,      0,       0],
    [ 1,  0,  2,  2,  1,       -1_331,        0,       8,         663,      0,       4],
    [-2,  0,  2,  2,  2,        1_383,        0,      -2,        -594,      0,      -2],
    [-1,  0,  0,  0,  2,        1_405,        0,       4,        -610,      0,       2],
    [ 1,  1,  2, -2,  2,        1_290,        0,       0,        -556,      0,       0],
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
            [1.0 - 0.5 * (da0 * da0 + xi0 * xi0), da0, -xi0],
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

    /// The full IAU 2000A nutation (luni-solar and planetary, free core
    /// nutation omitted) at 25 epochs, radians: `(JD TT, Delta psi, Delta
    /// epsilon)`. The first row is the reference vector published with ERFA's
    /// `eraNut00a` (`date1 = 2400000.5`, `date2 = 53736.0`); the other 24 were
    /// computed with the 1365-term series and the ERFA algorithm, and chosen
    /// from the Horizons oracle's epochs because they are where the table with the
    /// 30 wrong rows was furthest from the truth (14 epochs off by more than
    /// 3.1 mas in Delta psi, 10 by more than 1.65 mas in Delta epsilon).
    const IAU2000A: [(f64, f64, f64); 25] = [
        (
            2_453_736.5,
            -9.630_909_107_115_518_431e-6,
            4.063_239_174_001_678_710e-5,
        ),
        (
            2_422_370.5,
            7.176_558_754_307_209_8e-05,
            -2.791_738_341_640_319_1e-05,
        ),
        (
            2_425_580.5,
            -7.543_980_560_426_746_6e-05,
            2.003_209_390_465_656_7e-05,
        ),
        (
            2_434_790.5,
            8.423_880_575_131_421_0e-05,
            1.853_838_086_463_506_8e-05,
        ),
        (
            2_435_030.5,
            7.606_446_640_061_731_4e-05,
            9.524_090_270_998_693_0e-06,
        ),
        (
            2_435_060.5,
            7.630_351_612_656_552_2e-05,
            5.534_221_501_055_507_2e-06,
        ),
        (
            2_435_510.5,
            8.773_840_425_448_442_6e-05,
            -1.208_190_257_009_178_0e-05,
        ),
        (
            2_435_540.5,
            8.447_209_246_127_825_8e-05,
            -1.068_761_187_073_941_8e-05,
        ),
        (
            2_437_100.5,
            -1.502_695_685_308_570_2e-05,
            -4.679_366_547_261_974_9e-05,
        ),
        (
            2_440_460.5,
            1.717_544_287_019_232_5e-05,
            4.567_942_733_313_774_9e-05,
        ),
        (
            2_443_070.5,
            4.196_423_401_270_378_0e-05,
            -3.521_412_336_562_335_9e-05,
        ),
        (
            2_444_270.5,
            -3.441_236_244_364_943_6e-05,
            -3.939_591_191_661_528_4e-05,
        ),
        (
            2_447_960.5,
            6.249_435_669_590_069_6e-05,
            3.441_119_834_388_766_0e-05,
        ),
        (
            2_448_200.5,
            6.210_414_927_929_798_9e-05,
            2.370_236_649_244_462_6e-05,
        ),
        (
            2_448_680.5,
            8.834_565_837_122_339_2e-05,
            7.258_505_320_052_491_3e-06,
        ),
        (
            2_450_270.5,
            2.206_238_977_730_228_9e-05,
            -4.602_939_044_303_637_1e-05,
        ),
        (
            2_463_020.5,
            6.637_404_507_630_674_0e-05,
            -2.609_441_303_015_012_9e-05,
        ),
        (
            2_464_550.5,
            -2.896_359_919_612_077_4e-05,
            -4.084_983_820_955_887_9e-05,
        ),
        (
            2_468_240.5,
            5.182_998_284_965_629_1e-05,
            3.675_264_619_118_120_1e-05,
        ),
        (
            2_476_190.5,
            8.171_883_543_581_372_9e-05,
            -1.029_856_312_064_092_3e-05,
        ),
        (
            2_478_740.5,
            -6.953_937_195_503_784_2e-05,
            -2.771_995_019_007_469_7e-05,
        ),
        (
            2_480_330.5,
            -6.057_235_925_045_690_0e-05,
            3.460_340_093_942_865_8e-05,
        ),
        (
            2_482_550.5,
            7.512_485_307_891_501_1e-05,
            9.626_130_871_833_929_3e-06,
        ),
        (
            2_483_630.5,
            5.395_448_061_146_998_0e-05,
            -3.043_030_195_500_909_7e-05,
        ),
        (
            2_485_100.5,
            -5.110_016_140_149_516_4e-05,
            -3.926_429_437_656_160_8e-05,
        ),
    ];

    /// The 77-term series must reproduce the full IAU 2000A to within its own
    /// truncation error: 2.62 mas (1.27e-8 rad) in Delta psi and 0.90 mas
    /// (4.4e-9 rad) in Delta epsilon at these epochs. The tolerances, 3.1 mas
    /// and 1.65 mas, sit 18 to 80 percent above that, and below what a single
    /// wrong coefficient does: the engine's table before the 30 rows were
    /// corrected fails this at 14 and 10 of the 25 epochs. A wrong row cannot
    /// hide in the arguments, because every term's argument changes through the
    /// epochs.
    #[test]
    fn the_77_term_series_stays_within_its_truncation_error_of_iau_2000a() {
        const TOL_PSI: f64 = 1.5e-8; // 3.09 mas
        const TOL_EPS: f64 = 8.0e-9; // 1.65 mas
        let mut worst_psi = 0.0_f64;
        let mut worst_eps = 0.0_f64;
        for (jd, psi_a, eps_a) in IAU2000A {
            let (psi, eps) = nutation(jd);
            worst_psi = worst_psi.max((psi - psi_a).abs());
            worst_eps = worst_eps.max((eps - eps_a).abs());
            assert!(
                (psi - psi_a).abs() < TOL_PSI,
                "JD {jd}: Delta psi {psi:e} vs IAU 2000A {psi_a:e}, off by {} mas",
                (psi - psi_a) / arcsec_to_rad(0.001)
            );
            assert!(
                (eps - eps_a).abs() < TOL_EPS,
                "JD {jd}: Delta epsilon {eps:e} vs IAU 2000A {eps_a:e}, off by {} mas",
                (eps - eps_a) / arcsec_to_rad(0.001)
            );
        }
        // The check could have failed: the corrected series is not exact.
        assert!(worst_psi > 1e-9, "suspiciously exact: {worst_psi}");
        assert!(worst_eps > 1e-9, "suspiciously exact: {worst_eps}");
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
