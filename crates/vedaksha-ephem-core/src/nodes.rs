// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Lunar nodes: Rahu and Ketu, mean, true and osculating.
//!
//! # Frame
//!
//! Every public longitude in this module is referred to the **mean ecliptic
//! and equinox of date**, which is the frame a chart is drawn in and the
//! frame an ayanamsa is defined against. The six node longitudes reachable
//! through [`crate::bodies::Body`] are therefore interchangeable: swapping
//! one for another changes the *method* or the *end*, never the reference
//! frame.
//!
//! The single exception carries the frame in its name.
//! [`true_node_osculating_j2000`] returns the J2000 mean ecliptic, and exists
//! only so the osculating node can be compared against JPL Horizons' `OM`,
//! which is published in that frame. It is not for chart use: subtracting an
//! ayanamsa from it leaves the accumulated precession in the result, which
//! reaches 1.5° at 1900.
//!
//! Source: Meeus, "Astronomical Algorithms" 2nd ed., Ch. 47.

use crate::analytical::elp_mpp02::{self, MoonRectangular};
use crate::bodies::Body;
use crate::julian;

/// Compute the mean longitude of the ascending node of the Moon (Rahu).
///
/// Source: Meeus Ch. 47, eq. 47.7.
#[must_use]
pub fn mean_node(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    let omega = 125.044_547_9
        + t * (-1_934.136_289_1
            + t * (0.002_075_4 + t * (1.0 / 467_441.0 + t * (-1.0 / 60_616_000.0))));
    vedaksha_math::angle::normalize_degrees(omega)
}

/// Meeus 5-term truncated approximation of the true node — kept as a
/// private, independently-transcribed cross-check for
/// [`true_node_osculating`], not for production use.
///
/// [`Body::TrueNode`] returns [`true_node_osculating`] (see its doc
/// comment for the accuracy claim). This function exists only so this
/// module's own tests keep a second, independently-sourced method to
/// diff the osculating computation against — the exact class of
/// regression guard that caught the v7.0.0 node-frame defect, where two
/// node methods silently drifted into different reference frames.
///
/// Uses the 5 largest perturbation terms from Meeus Ch. 47.
/// Residual vs full lunar theory: ~0.09° max for modern dates.
///
/// Note: `tests/node_frame.rs` keeps its own independently-transcribed
/// replica of this same 5-term series (using `f64::sin` instead of
/// `libm::sin`) for its dense two-century integration test, so any
/// transcription fix here should be mirrored there.
///
/// Source: Meeus, *Astronomical Algorithms*, 2nd ed., Ch. 47.
#[cfg(test)]
fn true_node_meeus_truncated(jd: f64) -> f64 {
    let t = julian::centuries_from_j2000(jd);
    let mean = mean_node(jd);

    // Fundamental arguments (degrees → radians)
    let d = vedaksha_math::angle::deg_to_rad(297.850_192_1 + 445_267.111_403_4 * t);
    let m = vedaksha_math::angle::deg_to_rad(357.529_109_2 + 35_999.050_290_9 * t);
    let mp = vedaksha_math::angle::deg_to_rad(134.963_396_4 + 477_198.867_505_5 * t);
    let f = vedaksha_math::angle::deg_to_rad(93.272_095_0 + 483_202.017_523_3 * t);

    // Perturbation in longitude (degrees) — 5 principal terms.
    let correction =
        -1.4979 * libm::sin(2.0 * (d - f)) - 0.1500 * libm::sin(m) - 0.1226 * libm::sin(2.0 * d)
            + 0.1176 * libm::sin(2.0 * f)
            - 0.0801 * libm::sin(2.0 * (mp - f));

    vedaksha_math::angle::normalize_degrees(mean + correction)
}

/// Compute the true longitude of the ascending node from osculating
/// orbital elements derived from the Moon's position and velocity.
///
/// Standard osculating node method (Bate, Mueller & White 1971): given the
/// Moon's geocentric ecliptic position `(x, y, z)` and velocity
/// `(vx, vy, vz)`, find where the instantaneous orbital plane crosses
/// the ecliptic plane (z = 0, ascending).
///
/// The node direction is `r - (z/vz) * v`, with a sign correction to
/// select the ascending (not descending) node. This is exact to the
/// accuracy of the underlying lunar ephemeris.
///
/// **Frame:** mean ecliptic and equinox of date, matching [`mean_node`].
/// The crossing is computed against the ecliptic *of date*,
/// which is the same reference plane the Meeus node polynomial uses — not
/// the J2000 plane rotated into of-date coordinates, which is a different
/// quantity. For the J2000 value see [`true_node_osculating_j2000`].
///
/// Requires the `analytical` module (ELP/MPP02).
///
/// Source: Standard orbital mechanics (Bate, Mueller & White 1971;
/// Montenbruck & Gill 2000).
#[must_use]
pub fn true_node_osculating(jd: f64) -> f64 {
    osculating_node(elp_mpp02::elp_geocentric_of_date, jd)
}

/// The osculating ascending node in the **J2000** mean ecliptic.
///
/// Identical method to [`true_node_osculating`], evaluated against the J2000
/// ecliptic plane instead of the ecliptic of date. This is the frame JPL
/// Horizons publishes `OM` in (`EPHEM_TYPE='ELEMENTS'`,
/// `REF_SYSTEM='J2000'`), so it is the value the Horizons oracle pins.
///
/// **Not for chart use.** It differs from [`true_node_osculating`] by the
/// accumulated precession — roughly 50.3″ per year from J2000, reaching 1.5°
/// at 1900 — so subtracting an ayanamsa from it leaves that term in the
/// result. Sidereal and tropical-of-date consumers want
/// [`true_node_osculating`].
///
/// Requires the `analytical` module (ELP/MPP02).
#[must_use]
pub fn true_node_osculating_j2000(jd: f64) -> f64 {
    osculating_node(elp_mpp02::elp_geocentric, jd)
}

/// Shared osculating-node evaluation, parameterised by the frame the lunar
/// state vector is sampled in. The returned longitude is in whatever frame
/// `sample` produces.
fn osculating_node(sample: fn(f64) -> MoonRectangular, jd: f64) -> f64 {
    // Use a tight differentiation step for velocity (0.001 day ≈ 86 seconds).
    // The osculating node is sensitive to velocity direction.
    let moon = {
        let dt = 0.001_f64;
        let m0 = sample(jd);
        let mp = sample(jd + dt);
        let mm = sample(jd - dt);
        let inv_2dt = 1.0 / (2.0 * dt);
        MoonRectangular {
            x: m0.x,
            y: m0.y,
            z: m0.z,
            vx: (mp.x - mm.x) * inv_2dt,
            vy: (mp.y - mm.y) * inv_2dt,
            vz: (mp.z - mm.z) * inv_2dt,
        }
    };

    // Avoid division by zero if Moon is exactly on the ecliptic
    // with zero vertical velocity (astronomically impossible, but safe).
    let vz = if moon.vz.abs() < 1e-15 {
        1e-15
    } else {
        moon.vz
    };

    // Find where the tangent line to the orbit crosses z = 0.
    let fac = moon.z / vz;
    let sgn = vz.signum();

    // Node direction vector (ascending node selected by sign of vz).
    let nx = (moon.x - fac * moon.vx) * sgn;
    let ny = (moon.y - fac * moon.vy) * sgn;

    let lon_rad = libm::atan2(ny, nx);
    let lon_deg = lon_rad * 180.0 / core::f64::consts::PI;
    vedaksha_math::angle::normalize_degrees(lon_deg)
}

/// South Node (Ketu) = North Node + 180° (mean).
///
/// This is the theory-layer value: mean ecliptic and equinox of date, without
/// nutation, exactly like [`mean_node`]. [`Body::MeanSouthNode`] routes through
/// [`crate::coordinates`] and picks up nutation in longitude on the way, as
/// [`Body::MeanNode`] does — so pair like with like, theory with theory and
/// body with body. Through v7.1.1 there was no `Body` variant for Ketu at all,
/// so mixing the layers was the only option available and left the pair up to
/// 17.2 arcsec off 180 degrees.
#[must_use]
pub fn south_node_mean(jd: f64) -> f64 {
    vedaksha_math::angle::normalize_degrees(mean_node(jd) + 180.0)
}

/// South Node (Ketu) = North Node + 180° (true).
///
/// As of this release [`Body::TrueNode`] itself returns
/// [`true_node_osculating`] rather than a truncated series, so this is
/// now numerically identical to [`south_node_osculating`] and simply
/// delegates to it. Both names are kept so `Body::TrueSouthNode` and
/// `Body::TrueSouthNodeOsculating` remain distinct, stable API surface.
#[must_use]
pub fn south_node_true(jd: f64) -> f64 {
    south_node_osculating(jd)
}

/// South Node (Ketu) = North Node + 180° (osculating).
#[must_use]
pub fn south_node_osculating(jd: f64) -> f64 {
    vedaksha_math::angle::normalize_degrees(true_node_osculating(jd) + 180.0)
}

/// Ecliptic longitude of a node body in degrees, mean equinox of date, or
/// `None` if `body` is not a lunar node.
///
/// The nodes are directions, not places: they have no state vector, no
/// distance and no light-time, so [`crate::coordinates`] resolves them here
/// rather than through an [`crate::jpl::EphemerisProvider`].
#[must_use]
pub fn node_longitude(body: Body, jd_tt: f64) -> Option<f64> {
    match body {
        Body::MeanNode => Some(mean_node(jd_tt)),
        Body::TrueNode | Body::TrueNodeOsculating => Some(true_node_osculating(jd_tt)),
        Body::MeanSouthNode => Some(south_node_mean(jd_tt)),
        Body::TrueSouthNode => Some(south_node_true(jd_tt)),
        Body::TrueSouthNodeOsculating => Some(south_node_osculating(jd_tt)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 = JD 2451545.0
    const J2000: f64 = 2_451_545.0;

    #[test]
    fn mean_node_at_j2000_approx_125_04() {
        let mn = mean_node(J2000);
        assert!(
            (mn - 125.04).abs() < 0.5,
            "Mean node at J2000 expected ~125.04°, got {mn:.4}°"
        );
    }

    #[test]
    fn mean_node_always_in_range() {
        // Test several dates spanning millennia
        for offset in [-36525.0_f64, -10000.0, 0.0, 10000.0, 36525.0] {
            let jd = J2000 + offset;
            let mn = mean_node(jd);
            assert!(
                (0.0..360.0).contains(&mn),
                "Mean node out of [0,360) at JD {jd}: {mn:.4}°"
            );
        }
    }

    #[test]
    fn mean_node_retrograde_over_time() {
        // The lunar node is retrograde: decreases ~19.3°/year (mod 360)
        // Over one year (365.25 days), the node moves back ~19.3°
        let jd1 = J2000;
        let jd2 = J2000 + 365.25;
        let mn1 = mean_node(jd1);
        let mn2 = mean_node(jd2);
        // Compute signed difference accounting for wrap
        let mut diff = mn2 - mn1;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }
        // Should be approximately -19.3° (retrograde)
        assert!(
            (diff + 19.3).abs() < 1.0,
            "Mean node annual motion expected ~-19.3°, got {diff:.4}°"
        );
    }

    #[test]
    fn south_node_is_north_plus_180() {
        let mn = mean_node(J2000);
        let sn = south_node_mean(J2000);
        let expected = vedaksha_math::angle::normalize_degrees(mn + 180.0);
        assert!(
            (sn - expected).abs() < 1e-10,
            "South node should be north + 180°: south={sn:.6}, north+180={expected:.6}"
        );
    }

    #[test]
    fn osculating_node_at_j2000_close_to_mean_node() {
        // The geometric osculating node (r×v method) should agree with the mean node
        // within the maximum perturbation amplitude ≈ 1.60°; using 1.7° as the bound.
        // Reference: Bate, Mueller & White (1971), "Fundamentals of Astrodynamics".
        let mn = mean_node(J2000);
        let osc = true_node_osculating(J2000);
        let mut diff = (osc - mn).abs();
        if diff > 180.0 {
            diff = 360.0 - diff;
        }
        assert!(
            diff < 1.7,
            "Osculating node at J2000 should be within 1.7° of mean node, diff={diff:.4}°"
        );
    }

    #[test]
    fn osculating_ketu_is_rahu_plus_180() {
        // Ketu (south node) is always exactly 180° opposite Rahu (north node).
        // This verifies the geometric consistency of the osculating node calculation.
        // Reference: BPHS standard definition; Meeus Ch. 47.
        let rahu = true_node_osculating(J2000);
        let ketu = south_node_osculating(J2000);
        let expected_ketu = vedaksha_math::angle::normalize_degrees(rahu + 180.0);
        assert!(
            (ketu - expected_ketu).abs() < 1e-10,
            "Ketu (osculating) should be Rahu + 180°: Rahu={rahu:.6}°, Ketu={ketu:.6}°, expected={expected_ketu:.6}°"
        );
    }

    #[test]
    fn osculating_node_close_to_meeus_true_node() {
        // The osculating node librates about the Meeus 5-term series by
        // ≈0.3°, bounded, because one is an instantaneous orbital element and
        // the other is smoothed.
        //
        // This used to assert `< 0.5°` at J2000 alone, which proved nothing:
        // J2000 is the single epoch where a frame difference between the two
        // vanishes, and a frame difference is exactly what was there. The
        // epoch range below spans two centuries, and
        // `tests/node_frame.rs::every_node_method_shares_one_frame` sweeps it
        // densely.
        for (jd, label) in [
            (2_415_020.5, "1900"),
            (2_451_545.0, "J2000"),
            (2_469_807.5, "2050"),
            (2_488_069.5, "2100"),
        ] {
            let tn = true_node_meeus_truncated(jd);
            let osc = true_node_osculating(jd);
            let mut diff = (osc - tn).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }
            assert!(
                diff < 0.35,
                "{label}: osculating and Meeus true nodes should agree within \
                 0.35°, diff={diff:.4}°"
            );
        }
    }

    #[test]
    fn osculating_node_always_in_range() {
        for offset in [-3652.5_f64, 0.0, 3652.5, 7305.0, 10957.5] {
            let jd = J2000 + offset;
            let osc = true_node_osculating(jd);
            assert!(
                (0.0..360.0).contains(&osc),
                "Osculating node out of [0,360) at JD {jd}: {osc:.4}°"
            );
        }
    }

    #[test]
    fn osculating_node_vs_jpl_horizons() {
        // Oracle: JPL Horizons DE441 osculating node longitude `OM` in the
        // J2000 ecliptic. Query: COMMAND='301', CENTER='500@399',
        // EPHEM_TYPE='ELEMENTS', REF_PLANE='ECLIPTIC', REF_SYSTEM='J2000'.
        // `jd` here is TDB — `true_node_osculating_j2000` feeds
        // `elp_geocentric` directly and applies no ΔT, matching Horizons'
        // ELEMENTS time scale.
        //
        // This is the one test that wants the J2000 frame, and the reason
        // `true_node_osculating_j2000` exists at all. Pointing it at the
        // of-date `true_node_osculating` would fail by the accumulated
        // precession, which is the whole difference between the two.
        //
        // Re-measured 2026-07-16 against 2,435 Horizons rows spanning
        // 1900-2100: mean 0.00008°, max 0.00017° (≈0.6″), flat across every
        // era. The agreement is far tighter than this test used to imply.
        //
        // The J2000 and 2004 values below were previously 123.984 and 49.237 —
        // wrong by 0.0259° and 0.0106°. The other three matched Horizons to
        // 0.0004°, which is what exposed them: our evaluator tracks Horizons to
        // 0.0002°, so a 0.026° "residual" was the reference being wrong, not
        // the code. The old 0.05° tolerance existed to accommodate those bad
        // values. Corrected, and tightened to 0.001° — ~6× the observed max,
        // still tight enough to catch a real regression.
        //
        // Source: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/).
        let oracle = [
            (2451545.0, 123.9581, "J2000"),     // 2000-01-01
            (2453006.0, 49.2476, "2004-01-01"), // JD from Horizons
            (2455197.5, 290.9234, "2010-01-01"),
            (2457388.5, 174.6562, "2016-01-01"),
            (2459580.5, 60.9368, "2022-01-01"),
        ];

        for (jd, jpl_node, label) in &oracle {
            let osc = true_node_osculating_j2000(*jd);

            let mut diff = (osc - jpl_node).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }

            assert!(
                diff < 0.001,
                "{label}: osculating vs JPL DE441 diff too large: {diff:.4}° \
                 (ours={osc:.4}°, JPL={jpl_node:.4}°)"
            );
        }
    }

    #[test]
    fn osculating_node_multi_epoch_sanity() {
        // The osculating node is the Moon's instantaneous ascending node
        // derived from position + velocity. It differs from the Meeus
        // 5-term "true node" because:
        //   - Meeus smooths perturbations into 5 terms (~0.09° residual)
        //   - Osculating captures the full instantaneous orbital plane
        //
        // The two librate against each other by ≈0.3°, and that difference is
        // *bounded*. It is not the same thing as a frame difference, which
        // grows without limit — the distinction this test previously could
        // not make, because its epochs stopped at 2026 and its tolerance was
        // 0.5°. At 26 years from J2000 the frame term was still only 0.36°,
        // so it fit underneath. The range now runs 1900–2100, where a frame
        // term reaches 1.5°.
        //
        // For validation, we check:
        // 1. Values are in [0°, 360°)
        // 2. Osculating is within 3° of mean node (sanity bound)
        // 3. Osculating and Meeus agree within 0.35° (bounded libration)
        let epochs = [
            (2415020.5, "1900-01-01"),
            (2433282.5, "1950-01-01"),
            (2451545.0, "J2000"),
            (2455197.5, "2010-01-01"),
            (2459580.5, "2022-01-01"),
            (2469807.5, "2050-01-01"),
            (2488069.5, "2100-01-01"),
        ];

        for (jd, label) in &epochs {
            let mean = mean_node(*jd);
            let true_m = true_node_meeus_truncated(*jd);
            let osc = true_node_osculating(*jd);

            // Valid range
            assert!(
                (0.0..360.0).contains(&osc),
                "{label}: osculating out of range: {osc:.4}°"
            );

            // Osculating vs Meeus: within 0.35° (bounded libration)
            let mut diff_meeus = (osc - true_m).abs();
            if diff_meeus > 180.0 {
                diff_meeus = 360.0 - diff_meeus;
            }
            assert!(
                diff_meeus < 0.35,
                "{label}: osculating vs Meeus diff too large: {diff_meeus:.4}°"
            );

            // Osculating vs mean: within 3° (sanity)
            let mut diff_mean = (osc - mean).abs();
            if diff_mean > 180.0 {
                diff_mean = 360.0 - diff_mean;
            }
            assert!(
                diff_mean < 3.0,
                "{label}: osculating vs mean diff too large: {diff_mean:.4}°"
            );
        }
    }

    #[test]
    fn osculating_south_node_is_north_plus_180() {
        let north = true_node_osculating(J2000);
        let south = south_node_osculating(J2000);
        let expected = vedaksha_math::angle::normalize_degrees(north + 180.0);
        assert!(
            (south - expected).abs() < 1e-10,
            "Osculating south node should be north + 180°"
        );
    }

    /// Ketu is Rahu's opposite point by definition, in whichever frame Rahu
    /// was computed. Nothing in the crate calls the `south_node_*` functions,
    /// so without this nothing would notice one of the three drifting.
    #[test]
    fn every_south_node_is_exactly_opposite_its_north_node() {
        for jd in [2_415_020.5, 2_451_545.0, 2_460_000.5, 2_488_070.0] {
            for (north, south, label) in [
                (mean_node(jd), south_node_mean(jd), "mean"),
                (true_node_osculating(jd), south_node_true(jd), "true"),
                (
                    true_node_osculating(jd),
                    south_node_osculating(jd),
                    "osculating",
                ),
            ] {
                let sep = (south - north + 360.0) % 360.0;
                assert!(
                    (sep - 180.0).abs() < 1e-9,
                    "{label} node at jd {jd}: north {north}, south {south}, separation {sep}"
                );
            }
        }
    }
}
