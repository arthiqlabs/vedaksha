// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Tier-2 acceptance: hard-coded numerical examples from the IMCCE
//! explanatory note `elpmpp02.pdf` §6 / Tables 8.a, transcribed via the
//! clean-room re-derivation spec
//! (`docs/superpowers/specs/2026-05-09-elp-mpp02-rederivation-spec.md`).
//!
//! All coordinates are geocentric in the inertial mean ecliptic and equinox
//! of J2000, in km and km/day. The five LLR rows use [`Fit::Llr`]; the
//! five long-range rows use [`Fit::De405`].

use vedaksha_ephem_core::analytical::elp_mpp02::{
    Fit, MoonRectangular, elp_geocentric_with_fit,
};

/// Position tolerance (km). Spec §7 calls for ±5×10⁻⁵ km matching the
/// printed digits; the coefficient series here is truncated at |A|≥1×10⁻⁵
/// (arcsec / km) which keeps an at-most-few-tens-of-metres residual against
/// the full-precision IMCCE reference subroutine. We use 100 m here to
/// stay well inside ELP/MPP02's own inherent precision (50 m / 0.6″ across
/// [1500, 2500]; 4 m across [1950, 2060]).
const POS_TOL_KM: f64 = 0.1;

/// Velocity tolerance (km/day). Same reasoning as POS_TOL_KM, but
/// derivative of a 1×10⁻⁵-truncated series picks up modest extra noise
/// from the higher-frequency terms; 1×10⁻³ km/day ≈ 1 cm/s is well below
/// the inherent precision of the velocity subroutine.
const VEL_TOL_KM_PER_DAY: f64 = 1.0e-3;

/// Long-range tolerance for the DE405 examples reaching back to 489 CE
/// or out to 2132 CE. The inherent series precision is 50″/5″/10 km over
/// [−3000, 3000]. We expect to reproduce printed digits well; allow
/// 10 km / 0.05 km/day which is far inside the published bound.
const LONG_RANGE_POS_TOL_KM: f64 = 10.0;
const LONG_RANGE_VEL_TOL_KM_PER_DAY: f64 = 0.05;

#[derive(Clone, Copy)]
struct Example {
    jd: f64,
    expected: MoonRectangular,
}

const LLR_EXAMPLES: &[Example] = &[
    Example {
        jd: 2_444_239.5,
        expected: MoonRectangular {
            x: 43_890.282_40,
            y: 381_188.727_45,
            z: -31_633.381_65,
            vx: -87_516.197_48,
            vy: 13_707.664_44,
            vz: 2_754.221_24,
        },
    },
    Example {
        jd: 2_446_239.5,
        expected: MoonRectangular {
            x: -313_664.596_45,
            y: 212_007.266_74,
            z: 33_744.751_20,
            vx: -47_315.912_81,
            vy: -75_710.875_01,
            vz: -1_475.628_69,
        },
    },
    Example {
        jd: 2_448_239.5,
        expected: MoonRectangular {
            x: -273_220.060_67,
            y: -296_859.768_22,
            z: -34_604.357_00,
            vx: 60_542.327_59,
            vy: -58_162.316_74,
            vz: 2_270.886_91,
        },
    },
    Example {
        jd: 2_450_239.5,
        expected: MoonRectangular {
            x: 171_613.142_80,
            y: -318_097.337_50,
            z: 31_293.548_24,
            vx: 83_266.779_90,
            vy: 42_585.830_28,
            vz: -1_695.826_11,
        },
    },
    Example {
        jd: 2_452_239.5,
        expected: MoonRectangular {
            x: 396_530.006_35,
            y: 47_487.922_49,
            z: -36_085.309_03,
            vx: -12_664.286_94,
            vy: 83_512.757_19,
            vz: 1_507.367_56,
        },
    },
];

const DE405_EXAMPLES: &[Example] = &[
    Example {
        jd: 2_500_000.5,
        expected: MoonRectangular {
            x: 274_034.591_03,
            y: 252_067.536_89,
            z: -18_998.755_19,
            vx: -62_463.613_38,
            vy: 65_693.963_92,
            vz: 6_595.328_90,
        },
    },
    Example {
        jd: 2_300_000.5,
        expected: MoonRectangular {
            x: 353_104.313_59,
            y: -195_254.118_08,
            z: 34_943.545_92,
            vx: 39_543.136_78,
            vy: 74_373.180_70,
            vz: -700.653_51,
        },
    },
    Example {
        jd: 2_100_000.5,
        expected: MoonRectangular {
            x: -19_851.276_74,
            y: -385_646.177_17,
            z: -27_597.661_34,
            vx: 87_539.407_44,
            vy: -7_599.684_84,
            vz: -4_960.443_60,
        },
    },
    Example {
        jd: 1_900_000.5,
        expected: MoonRectangular {
            x: -370_342.792_54,
            y: -37_574.255_33,
            z: -4_527.918_40,
            vx: 12_255.287_46,
            vy: -89_710.975_08,
            vz: 7_649.442_85,
        },
    },
    Example {
        jd: 1_700_000.5,
        expected: MoonRectangular {
            x: -164_673.047_20,
            y: 367_791.713_29,
            z: 31_603.980_27,
            vx: -75_884.688_15,
            vy: -35_802.265_58,
            vz: -4_239.598_95,
        },
    },
];

fn check(label: &str, ex: Example, fit: Fit, pos_tol: f64, vel_tol: f64) {
    let got = elp_geocentric_with_fit(ex.jd, fit);
    let dx = got.x - ex.expected.x;
    let dy = got.y - ex.expected.y;
    let dz = got.z - ex.expected.z;
    let dr = (dx * dx + dy * dy + dz * dz).sqrt();
    let dvx = got.vx - ex.expected.vx;
    let dvy = got.vy - ex.expected.vy;
    let dvz = got.vz - ex.expected.vz;
    let dv = (dvx * dvx + dvy * dvy + dvz * dvz).sqrt();
    assert!(
        dr <= pos_tol,
        "{label} JD {jd}: position residual {dr:.6} km > {pos_tol} km \
         (got=({gx:.6},{gy:.6},{gz:.6}) expected=({ex_x:.6},{ex_y:.6},{ex_z:.6}))",
        jd = ex.jd,
        gx = got.x,
        gy = got.y,
        gz = got.z,
        ex_x = ex.expected.x,
        ex_y = ex.expected.y,
        ex_z = ex.expected.z,
    );
    assert!(
        dv <= vel_tol,
        "{label} JD {jd}: velocity residual {dv:.9} km/day > {vel_tol} km/day",
        jd = ex.jd,
    );
}

#[test]
fn llr_paper_examples() {
    for ex in LLR_EXAMPLES {
        check("LLR", *ex, Fit::Llr, POS_TOL_KM, VEL_TOL_KM_PER_DAY);
    }
}

#[test]
fn de405_paper_examples_long_range() {
    for ex in DE405_EXAMPLES {
        check(
            "DE405",
            *ex,
            Fit::De405,
            LONG_RANGE_POS_TOL_KM,
            LONG_RANGE_VEL_TOL_KM_PER_DAY,
        );
    }
}
