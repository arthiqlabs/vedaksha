// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Gravitational deflection of light by the Sun.
//!
//! Light passing the Sun is bent toward it, so a planet seen close to the Sun
//! appears displaced away from it. The shift is 4 milliarcseconds at 90 degrees
//! of elongation and grows as the planet nears the Sun: 47 milliarcseconds at
//! 10 degrees, 0.47 arcsecond at 1 degree, 1.75 arcseconds for a ray grazing the
//! limb. It is part of an apparent place, and the pipeline in
//! [`crate::coordinates`] applies it to every planet.
//!
//! # The model
//!
//! Only the Sun's spherically symmetric field is modelled, in the weak-field
//! (first post-Newtonian) limit. The planets bend light too, but far less and
//! only close by: Klioner's Table 1 gives Jupiter, the largest, as 16 milliarcseconds
//! for a ray grazing it and 10 microarcseconds at 11 degrees from it; Venus is
//! 0.5 milliarcsecond at the limb. Those are not modelled. The Sun's own
//! quadrupole and spin terms are about a microarcsecond and are not modelled
//! either.
//!
//! Take the observer at `x_o`, the source at its emission-time position `x_s`,
//! and the Sun at `x_sun`. Write `R = x_o - x_s`, `r_o = x_o - x_sun` and
//! `r_e = x_s - x_sun`. Then the unit vector of the light's direction of travel
//! at the observer is changed from `R/|R|` by
//!
//! ```text
//! δk = −(2GM/c²) · R × (r_e × r_o) / ( |R| · |r_o| · (|r_e|·|r_o| + r_o·r_e) )
//! ```
//!
//! The bending angle this gives is `(2GM/c²·|r_o|) · tan(φ/2)`, with `φ` the
//! angle between `r_e` and `r_o`. It depends on the observer's distance from
//! the Sun and on `φ`, but **not** on how far the source is from the Sun. For a
//! source at infinity `φ = π − ψ` with `ψ` the elongation, and the angle becomes
//! the familiar `(2GM/c²·|r_o|) · cot(ψ/2)`.
//!
//! Source: Klioner, S. A., "A Practical Relativistic Model for Microarcsecond
//! Astrometry in Space", *Astronomical Journal* 125, 1580 (2003)
//! (arXiv:astro-ph/0107457): equation (75) for the vector, (77) for the angle
//! with `γ = 1`, and (68) for the source-at-infinity limit.
//!
//! # Rays the Sun blocks
//!
//! The formula is singular for a source exactly behind the Sun (`φ = π`), and
//! for a ray whose straight-line path crosses the solar disc no light reaches the
//! observer at all. [`solar_deflection`] returns no displacement for such a
//! ray: the body is occulted, and its apparent place is not defined by this
//! model. The cut is the nominal solar radius (IAU 2015 Resolution B3,
//! 6.957 × 10⁸ m), so a ray that clears the limb by any margin keeps the
//! full deflection, 1.75 arcseconds at the limb itself.

/// Speed of light, m/s (exact, SI).
const C_M_S: f64 = 299_792_458.0;

/// One astronomical unit in metres (IAU 2012 Resolution B2, exact).
const AU_M: f64 = 149_597_870_700.0;

/// Nominal solar mass parameter `(GM)☉ᴺ`, m³/s² (IAU 2015 Resolution B3, exact).
const GM_SUN_NOMINAL_M3_S2: f64 = 1.327_124_4e20;

/// Nominal solar radius `R☉ᴺ`, metres (IAU 2015 Resolution B3, exact).
const SOLAR_RADIUS_M: f64 = 6.957e8;

/// `2GM☉/c²`, the Sun's Schwarzschild radius, in AU (about 1.974 × 10⁻⁸).
pub const SOLAR_SCHWARZSCHILD_RADIUS_AU: f64 = 2.0 * GM_SUN_NOMINAL_M3_S2 / (C_M_S * C_M_S) / AU_M;

/// The nominal solar radius in AU.
pub const SOLAR_RADIUS_AU: f64 = SOLAR_RADIUS_M / AU_M;

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: &[f64; 3]) -> f64 {
    libm::sqrt(dot(a, a))
}

/// The displacement the Sun's gravity adds to an astrometric geocentric vector,
/// in AU: add it to `astrometric` to get the vector toward the apparent
/// position.
///
/// * `astrometric` — geocentric vector from the observer to the source at the
///   instant the light left it, `x_s(t − τ) − x_o(t)`, with **no** aberration
///   term in it. Any frame will do provided `observer_from_sun` is in the same
///   one; only the geometry matters.
/// * `observer_from_sun` — `x_o(t) − x_sun(t)`, the observer relative to the
///   Sun at the time of observation.
///
/// The displacement has the length of `astrometric` times the bending angle, so
/// it moves the apparent direction by that angle and leaves the distance
/// unchanged to first order. It points away from the Sun, in the plane of the
/// Sun, the observer and the source.
///
/// Returns `[0.0; 3]` when the straight ray from the source to the observer
/// crosses the Sun's disc (see the module notes), and for a degenerate input
/// (zero length, or a source and an observer at the Sun's centre).
///
/// See the module documentation for the model and its source.
#[must_use]
pub fn solar_deflection(astrometric: [f64; 3], observer_from_sun: [f64; 3]) -> [f64; 3] {
    const NONE: [f64; 3] = [0.0; 3];

    let r_o = observer_from_sun;
    let r_e = [
        astrometric[0] + r_o[0],
        astrometric[1] + r_o[1],
        astrometric[2] + r_o[2],
    ];
    let range = norm(&astrometric);
    let ro = norm(&r_o);
    let re = norm(&r_e);
    if !(range > 0.0 && ro > 0.0 && re > 0.0) {
        return NONE;
    }

    // Does the straight line from the source to the observer pass through the
    // solar disc? The foot of the perpendicular from the Sun's centre onto the
    // line lies `along` from the observer toward the source; the ray can only
    // be blocked if that foot is between the two ends.
    let along = -dot(&r_o, &astrometric) / range;
    let impact = norm(&cross(&r_o, &astrometric)) / range;
    if along > 0.0 && along < range && impact < SOLAR_RADIUS_AU {
        return NONE;
    }

    let r_big = [-astrometric[0], -astrometric[1], -astrometric[2]];
    let denominator = ro * (re * ro + dot(&r_e, &r_o));
    if !denominator.is_finite() || denominator <= 0.0 {
        return NONE;
    }
    let numerator = cross(&r_big, &cross(&r_e, &r_o));
    let scale = SOLAR_SCHWARZSCHILD_RADIUS_AU / denominator;
    [
        numerator[0] * scale,
        numerator[1] * scale,
        numerator[2] * scale,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Arcseconds per radian.
    const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

    fn angle_between(a: &[f64; 3], b: &[f64; 3]) -> f64 {
        // atan2 of |a × b| and a·b is accurate for the tiny angles used here,
        // where acos of a dot product would lose every digit.
        libm::atan2(norm(&cross(a, b)), dot(a, b))
    }

    /// A source at distance `l` along a direction `elongation` radians from the
    /// Sun, seen by an observer 1 AU from the Sun on the +x axis (the Sun lies
    /// in the −x direction from the observer). Returns `(astrometric, r_o)`.
    fn far_source(elongation: f64, l: f64) -> ([f64; 3], [f64; 3]) {
        let p = [-libm::cos(elongation), libm::sin(elongation), 0.0];
        ([l * p[0], l * p[1], l * p[2]], [1.0, 0.0, 0.0])
    }

    #[test]
    fn schwarzschild_radius_is_the_published_figure() {
        // 2GM/c² for the nominal GM: 2 × 1.3271244e20 / c² = 2953.25 m, which is
        // 1.97412e-8 AU. Both are hand computed from the nominal constants and
        // do not go through the code under test.
        assert!((SOLAR_SCHWARZSCHILD_RADIUS_AU - 1.974_12e-8).abs() < 2e-13);
        assert!((SOLAR_RADIUS_AU - 4.650_47e-3).abs() < 1e-8);
    }

    /// At 90 degrees from the Sun the bending of a ray from infinity is
    /// (2GM/c²·E)·cot(45°) = 2GM/(c²·E): 4.07 milliarcseconds at 1 AU.
    #[test]
    fn deflection_at_ninety_degrees_is_four_milliarcseconds() {
        let (g, r_o) = far_source(core::f64::consts::FRAC_PI_2, 1.0e6);
        let d = solar_deflection(g, r_o);
        let shift = angle_between(&g, &[g[0] + d[0], g[1] + d[1], g[2] + d[2]]);
        let expected = SOLAR_SCHWARZSCHILD_RADIUS_AU; // × cot(45°) = 1, E = 1 AU
        assert!(
            (shift - expected).abs() < 1e-3 * expected,
            "shift {} mas, expected {} mas",
            shift * ARCSEC_PER_RAD * 1e3,
            expected * ARCSEC_PER_RAD * 1e3
        );
        assert!((shift * ARCSEC_PER_RAD * 1e3 - 4.072).abs() < 0.01);
    }

    /// For a ray from infinity the bending angle is (2GM/c²·E)·cot(ψ/2) at any
    /// elongation ψ (Klioner 2003, eq. 68).
    #[test]
    fn source_at_infinity_follows_cot_half_elongation() {
        for deg in [2.0_f64, 5.0, 10.0, 30.0, 60.0, 120.0, 170.0] {
            let psi = deg.to_radians();
            let (g, r_o) = far_source(psi, 1.0e9);
            let d = solar_deflection(g, r_o);
            let shift = angle_between(&g, &[g[0] + d[0], g[1] + d[1], g[2] + d[2]]);
            let expected = SOLAR_SCHWARZSCHILD_RADIUS_AU / libm::tan(psi / 2.0);
            assert!(
                (shift - expected).abs() < 1e-6 * expected,
                "{deg} deg: shift {shift:e}, expected {expected:e}"
            );
        }
    }

    /// A ray grazing the limb is bent by 4GM/(c²R☉) = 1.751 arcseconds, the
    /// textbook value, which the observer at 1 AU sees in full.
    #[test]
    fn a_ray_grazing_the_limb_is_bent_by_1_75_arcseconds() {
        // Impact parameter 1.0001 R☉ for a source at infinity: sin ψ = b / E.
        let b = 1.0001 * SOLAR_RADIUS_AU;
        let psi = libm::asin(b);
        let (g, r_o) = far_source(psi, 1.0e9);
        let d = solar_deflection(g, r_o);
        assert!(norm(&d) > 0.0, "a ray clearing the limb must be deflected");
        let shift = angle_between(&g, &[g[0] + d[0], g[1] + d[1], g[2] + d[2]]);
        let arcsec = shift * ARCSEC_PER_RAD;
        // 4GM/(c² b) with b = 1.0001 R☉ is 1.7510″; allow the finite observer
        // distance (1 AU against 215 solar radii).
        assert!(
            (arcsec - 1.7510).abs() < 0.003,
            "grazing deflection {arcsec}″, expected 1.751″"
        );
    }

    /// The displacement moves the apparent position away from the Sun.
    #[test]
    fn the_apparent_position_moves_away_from_the_sun() {
        for deg in [5.0_f64, 40.0, 100.0, 160.0] {
            let psi = deg.to_radians();
            let (g, r_o) = far_source(psi, 50.0);
            let d = solar_deflection(g, r_o);
            let g1 = [g[0] + d[0], g[1] + d[1], g[2] + d[2]];
            let to_sun = [-r_o[0], -r_o[1], -r_o[2]];
            let before = angle_between(&g, &to_sun);
            let after = angle_between(&g1, &to_sun);
            assert!(after > before, "{deg} deg: elongation {before} -> {after}");
            // And in the plane of the three bodies: no z displacement.
            assert!(d[2].abs() < 1e-30);
        }
    }

    /// The bending angle is (2GM/c²·|r_o|)·tan(φ/2) and does not depend on how
    /// far the source is, for a fixed φ (Klioner 2003, eq. 77 and the sentence
    /// after it). Three sources on the same line from the Sun, at 3, 30 and 3000
    /// AU, share one φ for a given observer only if they share `r_e`'s
    /// direction, so build them that way.
    #[test]
    fn bending_angle_depends_on_phi_not_on_source_distance() {
        let r_o = [1.0, 0.0, 0.0];
        let u = [libm::cos(2.0), libm::sin(2.0), 0.0]; // direction of r_e
        let phi = angle_between(&u, &r_o);
        let expected = SOLAR_SCHWARZSCHILD_RADIUS_AU * libm::tan(phi / 2.0);
        for s in [3.0_f64, 30.0, 3000.0] {
            let r_e = [s * u[0], s * u[1], s * u[2]];
            let g = [r_e[0] - r_o[0], r_e[1] - r_o[1], r_e[2] - r_o[2]];
            let d = solar_deflection(g, r_o);
            let shift = angle_between(&g, &[g[0] + d[0], g[1] + d[1], g[2] + d[2]]);
            assert!(
                (shift - expected).abs() < 1e-6 * expected,
                "source at {s} AU: shift {shift:e}, expected {expected:e}"
            );
        }
    }

    /// A source on the observer's side of the Sun, on the line through the Sun
    /// (inferior conjunction), is not deflected: φ = 0 so tan(φ/2) = 0.
    #[test]
    fn a_source_between_the_observer_and_the_sun_is_not_deflected() {
        let r_o = [1.0, 0.0, 0.0];
        let g = [-0.4, 0.0, 0.0]; // 0.4 AU toward the Sun
        let d = solar_deflection(g, r_o);
        assert_eq!(d, [0.0; 3]);
        // Slightly off the line: the angle is (2GM/c²)·tan(φ/2) with φ the
        // angle at the Sun between the source and the observer, here about
        // 0.0167 rad, so 1.6e-10 rad. Tiny, and equal to that closed form.
        let g = [-0.4, 0.01, 0.0];
        let d = solar_deflection(g, r_o);
        let r_e = [g[0] + r_o[0], g[1] + r_o[1], g[2] + r_o[2]];
        let phi = angle_between(&r_e, &r_o);
        let expected = SOLAR_SCHWARZSCHILD_RADIUS_AU * libm::tan(phi / 2.0);
        let shift = angle_between(&g, &[g[0] + d[0], g[1] + d[1], g[2] + d[2]]);
        assert!(
            expected < 2e-10 && expected > 1e-10,
            "expected {expected:e}"
        );
        assert!(
            (shift - expected).abs() < 1e-6 * expected,
            "shift {shift:e}"
        );
    }

    /// A ray through the solar disc is blocked and gets no displacement, with
    /// no NaN from the singular φ = π geometry; one that clears the limb gets
    /// the full deflection.
    #[test]
    fn a_ray_through_the_disc_is_left_alone_and_one_beside_it_is_not() {
        let r_o = [1.0, 0.0, 0.0];
        // Exactly behind the Sun.
        let d = solar_deflection([-31.0, 0.0, 0.0], r_o);
        assert_eq!(d, [0.0; 3]);
        // Inside the disc (impact parameter half a solar radius).
        let inside = [-31.0, 0.5 * SOLAR_RADIUS_AU * 31.0, 0.0];
        assert_eq!(solar_deflection(inside, r_o), [0.0; 3]);
        // Just outside the limb: 1.5 solar radii of impact parameter.
        let outside = [-31.0, 1.5 * SOLAR_RADIUS_AU * 31.0, 0.0];
        let d = solar_deflection(outside, r_o);
        let g1 = [outside[0] + d[0], outside[1] + d[1], outside[2] + d[2]];
        let shift = angle_between(&outside, &g1) * ARCSEC_PER_RAD;
        assert!(shift > 1.0 && shift < 1.4, "shift {shift}″");
        assert!(d.iter().all(|c| c.is_finite()));
    }

    /// The line only matters between its ends: a source on the far side of the
    /// Sun's line but nearer than the Sun is not blocked.
    #[test]
    fn a_source_nearer_than_the_foot_of_the_perpendicular_is_not_blocked() {
        // Observer 1 AU out, source 0.5 AU away along a direction that points
        // at the Sun's centre: the foot is 1 AU away, past the source.
        let r_o = [1.0, 0.0, 0.0];
        let g = [-0.5, 0.0, 0.0];
        // On the line through the Sun but short of it: no deflection (φ = 0),
        // and crucially not the blocked branch's zero by accident of geometry:
        // move it off-axis and the displacement must be non-zero and finite.
        let g_off = [-0.5, 0.2, 0.0];
        let d = solar_deflection(g_off, r_o);
        assert!(d.iter().all(|c| c.is_finite()));
        assert!(norm(&d) > 0.0);
        assert_eq!(solar_deflection(g, r_o), [0.0; 3]);
    }

    #[test]
    fn degenerate_input_gives_no_displacement() {
        assert_eq!(solar_deflection([0.0; 3], [1.0, 0.0, 0.0]), [0.0; 3]);
        assert_eq!(solar_deflection([1.0, 0.0, 0.0], [0.0; 3]), [0.0; 3]);
        assert_eq!(
            solar_deflection([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            [0.0; 3]
        );
    }
}
