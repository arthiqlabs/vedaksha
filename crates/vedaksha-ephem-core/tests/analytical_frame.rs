// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1

//! The frame of the analytical planets, measured against DE440s.
//!
//! `AnalyticalProvider` rotates VSOP87A out of the theory's dynamical ecliptic
//! and equinox of J2000 with the matrix its authors publish. Until that was
//! done it used the ecliptic-to-equator rotation about x by 84381.448 arcsec
//! alone, which left a rotation of 106 mas (38.9 about x, 99.0 about z)
//! between the Sun, Mercury, Venus and Mars and the SPK path, and put the
//! analytical path's floor at 0.18 arcsec against Horizons where the SPK
//! path's is 0.09.
//!
//! This compares **geometric heliocentric vectors**, with no light time and no
//! aberration, so nothing but the series and the frame can separate the two
//! providers. The provider's Sun is the origin (Sun ≈ SSB), so the SPK side is
//! the planet minus the Sun, both from the kernel.
//!
//! Disjoint from `analytical_oracle.rs`, which measures apparent longitudes
//! against Horizons: that one cannot tell a frame rotation from the Delta T,
//! nutation and aberration terms that share its residual.

mod common;

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_ephem_core::jpl::reader::SpkReader;

const ARCSEC_PER_RAD: f64 = 180.0 * 3600.0 / core::f64::consts::PI;

/// 1900-01-01 to 2100-01-01, every 120 days: 609 epochs. The same range as the
/// Horizons fixture at a quarter of the 30-day density, which reproduces the
/// 30-day means to 0.0001 arcsec for all four bodies (measured).
fn epochs() -> impl Iterator<Item = f64> {
    (0..)
        .map(|k| 2_415_020.5 + 120.0 * f64::from(k))
        .take_while(|&jd| jd <= 2_488_069.5)
}

/// Angle in arcsec between two vectors of nearly equal length: the chord of
/// their difference over the radius.
fn chord_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    let r = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    d / r * ARCSEC_PER_RAD
}

/// Mean heliocentric angular residual of `AnalyticalProvider` against DE440s.
///
/// Measured 2026-10-24 on a 30-day grid, 1900 to 2100, in arcsec:
///
/// | | rotation `Rx(84381.448)` | published VSOP87 matrix |
/// |---|---|---|
/// | Mercury | 0.102 | 0.021 |
/// | Venus | 0.090 | 0.014 |
/// | Earth | 0.088 | 0.019 |
/// | Mars | 0.075 | 0.032 |
///
/// ceiling 0.05: 1.5 times the largest value with the matrix and two thirds of
/// the smallest without it, so it fails the old rotation for every one of the
/// four bodies.
#[test]
fn inner_planets_agree_with_de440s_in_frame() {
    if !common::require_bsp() {
        return;
    }
    const CEILING_ARCSEC: f64 = 0.05;
    let spk = SpkReader::open(common::bsp_path()).expect("failed to open DE440s");
    let ana = AnalyticalProvider::new();

    let bodies = [
        ("Mercury", Body::Mercury),
        ("Venus", Body::Venus),
        ("Earth", Body::EarthMoonBarycenter),
        ("Mars", Body::Mars),
    ];
    let mut failures = Vec::new();
    for (name, body) in bodies {
        let (mut sum, mut n) = (0.0_f64, 0_u32);
        for jd in epochs() {
            let sun = spk.compute_state(Body::Sun, jd).unwrap().position;
            let (s, a) = if body == Body::EarthMoonBarycenter {
                (
                    spk.earth_state(jd).unwrap().position,
                    ana.earth_state(jd).unwrap().position,
                )
            } else {
                (
                    spk.compute_state(body, jd).unwrap().position,
                    ana.compute_state(body, jd).unwrap().position,
                )
            };
            let helio_spk = [s.x - sun.x, s.y - sun.y, s.z - sun.z];
            sum += chord_arcsec([a.x, a.y, a.z], helio_spk);
            n += 1;
        }
        let mean = sum / f64::from(n);
        eprintln!("{name:8} mean heliocentric residual vs DE440s: {mean:.4}\" over {n} epochs");
        if mean >= CEILING_ARCSEC {
            failures.push(format!("{name} {mean:.4}\""));
        }
    }
    assert!(
        failures.is_empty(),
        "mean heliocentric residual against DE440s at or above {CEILING_ARCSEC}\": {failures:?}. \
         A frame rotation between VSOP87A and the equatorial frame puts all four near 0.09\"."
    );
}
