// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Mean longitudes and mean daily motions of the grahas.
//!
//! The Jyotish strength rules that compare a graha's true motion with its
//! mean one ([`crate::cheshta`], the cheshta kendra of [`crate::shadbala`])
//! need a mean longitude and a mean rate at a date. This module is where the
//! crate keeps them, so a consumer does not have to invent the constants.
//!
//! # Source
//!
//! The planets are the VSOP87 mean elements of Simon, Bretagnon, Chapront,
//! Chapront-Touze, Francou and Laskar, "Numerical expressions for precession
//! formulae and mean elements for the Moon and the planets", Astronomy and
//! Astrophysics 282, 663 (1994). The mean longitudes `lambda` are taken from
//! Section 5.9, "Mean elements of the planets referred to the mean dynamical
//! ecliptic and equinox of date" (5.9.1 Mercury, 5.9.2 Venus, 5.9.3 Earth,
//! 5.9.4 Mars, 5.9.5 Jupiter, 5.9.6 Saturn): a polynomial of degree six in
//! `t`, the TDB time in thousands of Julian years from J2000
//! (`t = (JD - 2451545.0) / 365250`), with the constant term in degrees and
//! the coefficient of `t^n` in arcseconds. The same elements, refitted as a
//! cubic in Julian centuries, are tabulated in Meeus, *Astronomical
//! Algorithms*, 2nd ed., Table 31.A; the two agree in the linear coefficient
//! to about 1e-4 degree per century and the paper is used here because it is
//! the primary and carries the higher terms.
//!
//! Time is taken as TT. TDB and TT differ by at most 2 ms, far below what a
//! mean longitude can resolve.
//!
//! # What "mean longitude" means, per graha
//!
//! Every longitude here is **tropical**, referred to the **mean equinox of
//! date**, in `[0, 360)`: the frame an ayanamsha is subtracted from, and the
//! frame the crate's other of-date longitudes use (cf. the node module of
//! `vedaksha-ephem-core`). A sidereal consumer subtracts its ayanamsha, or
//! adds it to the true longitude it compares against; differences between
//! longitudes in one frame need no conversion.
//!
//! | Graha | Mean longitude |
//! |---|---|
//! | Mercury, Venus, Mars, Jupiter, Saturn | the **heliocentric** mean longitude `lambda` of the planet (Simon et al. 5.9), the mean planet on its own orbit. This is the quantity the cheshta kendra of BPHS 27.24 wants as the `sighrocca` of Mercury and Venus, and as the mean planet of Mars, Jupiter and Saturn. |
//! | Sun | the mean Sun: the Earth's heliocentric mean longitude (Simon et al. 5.9.3, which is the Earth-Moon barycentre) plus 180 degrees. The Hindu `madhyama surya`. |
//! | Moon | `W1` of ELP/MPP02, the mean longitude the crate's lunar series is built on, carried to the equinox of date by the precession the series itself applies (`vedaksha_ephem_core::analytical::elp_mpp02::mean_longitude_of_date`). No second set of lunar constants. |
//! | Rahu | the mean ascending node of `vedaksha_ephem_core::nodes::mean_node` (Meeus, Ch. 47). |
//! | Ketu | Rahu plus 180 degrees. |
//!
//! # Mean longitude rate against mean *geocentric* motion
//!
//! [`mean_longitude`] returns the rate of the longitude it returns, so for
//! Mercury and Venus the rate is the *heliocentric* one (about 4.09 and 1.60
//! degrees per day). That is not the pace at which those two grahas move
//! across the sky as seen from the Earth: averaged over a synodic cycle their
//! geocentric longitude keeps up with the Sun, at about 0.9856 degrees per
//! day, and an inferior planet's apparent speed (Mercury -1.4 to +2.2 degrees
//! per day) would read as slow against 4.09 at every date. For the outer
//! planets the two coincide, because the Earth's motion averages out of a
//! superior planet's geocentric longitude. [`mean_geocentric_motion`] is the
//! rate to hand to [`crate::cheshta::cheshta`] as `mean_speed`; for Mercury
//! and Venus it is the mean Sun's rate and for every other graha it is
//! [`mean_longitude`]'s own.

use crate::graha::Graha;
use vedaksha_math::angle::normalize_degrees;

/// J2000.0 as a Julian Day.
const J2000_JD: f64 = 2_451_545.0;

/// Days in a thousand Julian years, the unit of `t` in Simon et al. Sect. 5.
const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// Arcseconds per degree.
const ARCSEC_PER_DEG: f64 = 3600.0;

/// A mean-longitude polynomial of Simon et al. (1994) Sect. 5.9: the
/// constant term in degrees, then the coefficients of `t^1 ..= t^6` in
/// arcseconds, `t` in thousands of Julian years from J2000.
struct MeanLongitudePoly {
    /// `lambda` at J2000, degrees.
    constant_deg: f64,
    /// Coefficients of `t`, `t^2`, ... `t^6`, arcseconds.
    coeff_arcsec: [f64; 6],
}

/// Simon et al. 1994, Sect. 5.9.1, Mercury.
const MERCURY: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 252.250_905_52,
    coeff_arcsec: [
        5_381_066_598.200_37,
        109.259_43,
        0.065_22,
        -0.235_00,
        -0.001_79,
        0.000_20,
    ],
};

/// Simon et al. 1994, Sect. 5.9.2, Venus.
const VENUS: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 181.979_800_85,
    coeff_arcsec: [
        2_106_691_666.319_89,
        111.650_21,
        0.053_68,
        -0.235_16,
        -0.001_79,
        0.000_20,
    ],
};

/// Simon et al. 1994, Sect. 5.9.3, the Earth (the Earth-Moon barycentre).
const EARTH: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 100.466_456_83,
    coeff_arcsec: [
        1_296_027_711.034_29,
        109.158_09,
        0.072_07,
        -0.235_30,
        -0.001_80,
        0.000_20,
    ],
};

/// Simon et al. 1994, Sect. 5.9.4, Mars.
const MARS: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 355.432_999_58,
    coeff_arcsec: [
        689_101_069.330_69,
        111.786_74,
        0.056_24,
        -0.235_16,
        -0.001_80,
        0.000_20,
    ],
};

/// Simon et al. 1994, Sect. 5.9.5, Jupiter.
const JUPITER: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 34.351_518_74,
    coeff_arcsec: [
        109_306_899.894_53,
        80.387_00,
        0.133_27,
        -0.188_50,
        0.004_11,
        -0.000_14,
    ],
};

/// Simon et al. 1994, Sect. 5.9.6, Saturn.
const SATURN: MeanLongitudePoly = MeanLongitudePoly {
    constant_deg: 50.077_444_30,
    coeff_arcsec: [
        44_046_398.470_38,
        186.868_17,
        -0.107_48,
        -0.350_04,
        -0.016_30,
        0.001_03,
    ],
};

impl MeanLongitudePoly {
    /// `(lambda in degrees, unnormalised; d lambda / d JD in degrees per day)`.
    fn eval(&self, jd_tt: f64) -> (f64, f64) {
        let t = (jd_tt - J2000_JD) / DAYS_PER_MILLENNIUM;
        // c1 t + c2 t^2 + ... + c6 t^6 = t (c1 + t (c2 + ... )), by Horner.
        let mut inner = 0.0;
        for c in self.coeff_arcsec.iter().rev() {
            inner = inner * t + c;
        }
        let lambda_deg = self.constant_deg + inner * t / ARCSEC_PER_DEG;
        (lambda_deg, deriv_per_day(&self.coeff_arcsec, t))
    }
}

/// `d lambda / d JD` in degrees per day for the polynomial with coefficients
/// `c[0..6]` of `t^1 ..= t^6` (arcseconds), at `t` thousand years.
fn deriv_per_day(c: &[f64; 6], t: f64) -> f64 {
    // d/dt (c1 t + c2 t^2 + ... + c6 t^6) = c1 + 2 c2 t + ... + 6 c6 t^5.
    let mut acc = 0.0;
    for (n, coeff) in c.iter().enumerate().rev() {
        acc = acc * t + coeff * (n + 1) as f64;
    }
    acc / ARCSEC_PER_DEG / DAYS_PER_MILLENNIUM
}

/// The Simon et al. 5.9 polynomial for a planet graha; `None` for the Sun,
/// Moon and nodes, which are not planetary mean elements.
fn planet_poly(graha: Graha) -> Option<&'static MeanLongitudePoly> {
    match graha {
        Graha::Mercury => Some(&MERCURY),
        Graha::Venus => Some(&VENUS),
        Graha::Mars => Some(&MARS),
        Graha::Jupiter => Some(&JUPITER),
        Graha::Saturn => Some(&SATURN),
        Graha::Sun | Graha::Moon | Graha::Rahu | Graha::Ketu => None,
    }
}

/// Shortest signed arc from `a` to `b`, degrees in `(-180, 180]`.
fn signed_arc(a: f64, b: f64) -> f64 {
    let d = (b - a).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// Mean longitude of a graha at `jd_tt`, and its rate: `(degrees in
/// [0, 360), degrees per day)`, tropical, mean equinox of date.
///
/// What counts as the mean longitude of each graha, and the source of every
/// number, is set out in the [module documentation](self): the five planets
/// are the heliocentric mean longitudes of Simon et al. (1994) Sect. 5.9, the
/// Sun is the Earth's plus 180 degrees, the Moon is ELP/MPP02's `W1` carried
/// to the date, and the nodes are the mean node.
///
/// The rate is the analytic derivative of the polynomial the longitude is
/// read from. Mercury's and Venus's rate is therefore their heliocentric
/// rate; hand [`mean_geocentric_motion`], not this, to
/// [`crate::cheshta::cheshta`].
#[must_use]
pub fn mean_longitude(graha: Graha, jd_tt: f64) -> (f64, f64) {
    if let Some(poly) = planet_poly(graha) {
        let (lambda, rate) = poly.eval(jd_tt);
        return (normalize_degrees(lambda), rate);
    }
    match graha {
        Graha::Sun => {
            let (lambda, rate) = EARTH.eval(jd_tt);
            (normalize_degrees(lambda + 180.0), rate)
        }
        Graha::Moon => vedaksha_ephem_core::analytical::elp_mpp02::mean_longitude_of_date(jd_tt),
        Graha::Rahu => mean_node_with_rate(jd_tt),
        Graha::Ketu => {
            let (lon, rate) = mean_node_with_rate(jd_tt);
            (normalize_degrees(lon + 180.0), rate)
        }
        Graha::Mercury | Graha::Venus | Graha::Mars | Graha::Jupiter | Graha::Saturn => {
            unreachable!("planets are handled above")
        }
    }
}

/// The mean node and its rate. The rate is a one-day central difference of
/// `vedaksha_ephem_core::nodes::mean_node`, so the node polynomial stays in
/// one place; the polynomial is smooth to the fourth order, so the difference
/// is exact to rounding.
fn mean_node_with_rate(jd_tt: f64) -> (f64, f64) {
    use vedaksha_ephem_core::nodes::mean_node;
    let lon = mean_node(jd_tt);
    let rate = signed_arc(mean_node(jd_tt - 0.5), mean_node(jd_tt + 0.5));
    (lon, rate)
}

/// Mean daily motion of a graha **as seen from the Earth**, in degrees per
/// day: the `mean_speed` to hand to [`crate::cheshta::cheshta`].
///
/// - Mars, Jupiter, Saturn: their heliocentric mean rate. Over a synodic cycle
///   the geocentric longitude of a superior planet gains exactly as many
///   revolutions as the planet makes on its orbit, because the Earth's own
///   motion is bounded in the geocentric direction and averages out.
/// - Mercury, Venus: the mean Sun's rate. An inferior planet stays near the
///   Sun, so its geocentric longitude keeps pace with it.
/// - Sun, Moon, Rahu, Ketu: the rate of [`mean_longitude`] (the nodes' is
///   negative).
#[must_use]
pub fn mean_geocentric_motion(graha: Graha, jd_tt: f64) -> f64 {
    match graha {
        Graha::Mercury | Graha::Venus => mean_longitude(Graha::Sun, jd_tt).1,
        _ => mean_longitude(graha, jd_tt).1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly_of(g: Graha) -> &'static MeanLongitudePoly {
        planet_poly(g).expect("a planet")
    }

    #[test]
    fn at_j2000_the_rate_is_the_published_linear_coefficient() {
        // t = 0, so d lambda / d JD is the t^1 coefficient (arcsec per
        // thousand years) over 3600 * 365250. The independent expectation is
        // the printed coefficient divided out by hand below.
        let cases = [
            (Graha::Mercury, 5_381_066_598.200_37_f64),
            (Graha::Venus, 2_106_691_666.319_89),
            (Graha::Mars, 689_101_069.330_69),
            (Graha::Jupiter, 109_306_899.894_53),
            (Graha::Saturn, 44_046_398.470_38),
        ];
        for (g, c1) in cases {
            let (_, rate) = mean_longitude(g, J2000_JD);
            let expected = c1 / 3600.0 / 365_250.0;
            assert!(
                (rate - expected).abs() < 1e-12,
                "{g:?}: {rate} != {expected}"
            );
        }
        let (_, earth) = mean_longitude(Graha::Sun, J2000_JD);
        assert!((earth - 1_296_027_711.034_29 / 3600.0 / 365_250.0).abs() < 1e-12);
    }

    #[test]
    fn at_j2000_the_longitude_is_the_published_constant() {
        let cases = [
            (Graha::Mercury, 252.250_905_52_f64),
            (Graha::Venus, 181.979_800_85),
            (Graha::Mars, 355.432_999_58),
            (Graha::Jupiter, 34.351_518_74),
            (Graha::Saturn, 50.077_444_30),
            // The Sun is the Earth's 100.46645683 plus half a turn.
            (Graha::Sun, 280.466_456_83),
        ];
        for (g, lambda0) in cases {
            let (lon, _) = mean_longitude(g, J2000_JD);
            assert!((lon - lambda0).abs() < 1e-9, "{g:?}: {lon} != {lambda0}");
        }
    }

    #[test]
    fn the_rate_is_the_derivative_of_the_longitude() {
        // A central difference over one day must reproduce the analytic
        // rate, at dates far from J2000 where the t^2 ... t^6 terms matter.
        // This is what catches a mis-ordered coefficient or a derivative
        // that dropped a power.
        let all = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mercury,
            Graha::Venus,
            Graha::Mars,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
        ];
        for jd in [
            2_086_302.5,
            2_415_020.5,
            2_451_545.0,
            2_470_000.5,
            2_816_787.5,
        ] {
            for g in all {
                let (_, rate) = mean_longitude(g, jd);
                let (a, _) = mean_longitude(g, jd - 0.5);
                let (b, _) = mean_longitude(g, jd + 0.5);
                let numeric = signed_arc(a, b);
                // Fast Mercury: 4.09 deg/day; a one-day difference of a
                // smooth polynomial is exact to rounding plus a vanishing
                // third-derivative term.
                assert!(
                    (numeric - rate).abs() < 1e-7,
                    "{g:?} at JD {jd}: analytic {rate} vs difference {numeric}"
                );
            }
        }
    }

    #[test]
    fn the_mean_sidereal_periods_are_the_known_ones() {
        // Independent of the transcription's last digits: the tropical rate
        // less the general precession (0.0000382 deg/day) must give the
        // planet's well-known sidereal period in days.
        let precession_deg_per_day = 5028.796_195 / 3600.0 / 36_525.0;
        let periods = [
            (Graha::Mercury, 87.969_f64),
            (Graha::Venus, 224.701),
            (Graha::Mars, 686.980),
            (Graha::Jupiter, 4_332.589),
            (Graha::Saturn, 10_759.22),
        ];
        for (g, period) in periods {
            let (_, rate) = mean_longitude(g, J2000_JD);
            let got = 360.0 / (rate - precession_deg_per_day);
            assert!(
                (got - period).abs() / period < 3e-5,
                "{g:?}: sidereal period {got} d, expected {period} d"
            );
        }
        // The Earth: the sidereal year.
        let (_, earth) = mean_longitude(Graha::Sun, J2000_JD);
        let year = 360.0 / (earth - precession_deg_per_day);
        assert!((year - 365.256_36).abs() < 1e-3, "sidereal year {year} d");
    }

    #[test]
    fn of_date_longitudes_run_ahead_of_j2000_by_the_precession() {
        // Sect. 5.9 (of date) less Sect. 5.8 (J2000) is, per thousand years,
        // the general precession in longitude, 50288 arcsec, to within the
        // few arcseconds each planet's plane motion adds. The J2000
        // coefficients below are Sect. 5.8's, read separately from 5.9's, so
        // a misread leading digit of either shows here.
        let j2000_c1 = [
            (Graha::Mercury, 5_381_016_286.889_82_f64),
            (Graha::Venus, 2_106_641_364.335_48),
            (Graha::Mars, 689_050_774.939_88),
            (Graha::Jupiter, 109_256_603.779_91),
            (Graha::Saturn, 43_996_098.557_32),
        ];
        for (g, c1_j2000) in j2000_c1 {
            let c1_date = poly_of(g).coeff_arcsec[0];
            let diff = c1_date - c1_j2000;
            assert!(
                (diff - 50_288.0).abs() < 30.0,
                "{g:?}: of-date minus J2000 = {diff} arcsec per millennium"
            );
        }
    }

    #[test]
    fn longitudes_are_normalised() {
        for g in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
        ] {
            for jd in [2_086_302.5, 2_415_020.5, 2_451_545.0, 2_816_787.5] {
                let (lon, _) = mean_longitude(g, jd);
                assert!((0.0..360.0).contains(&lon), "{g:?} at {jd}: {lon}");
            }
        }
    }

    #[test]
    fn the_sun_is_half_a_turn_from_the_earth() {
        // The mean Sun is the Earth's heliocentric mean longitude + 180.
        for jd in [2_415_020.5, 2_451_545.0, 2_470_000.5] {
            let (sun, sun_rate) = mean_longitude(Graha::Sun, jd);
            let (earth, earth_rate) = EARTH.eval(jd);
            assert!((signed_arc(earth, sun) - 180.0).abs() < 1e-9);
            assert!((sun_rate - earth_rate).abs() < 1e-15);
        }
    }

    #[test]
    fn the_moon_and_nodes_reuse_the_ephemeris_values() {
        use vedaksha_ephem_core::analytical::elp_mpp02::mean_longitude_of_date;
        use vedaksha_ephem_core::nodes::mean_node;
        for jd in [2_415_020.5, 2_451_545.0, 2_470_000.5] {
            // Bit-for-bit the ephemeris crate's own numbers: no second set.
            assert_eq!(mean_longitude(Graha::Moon, jd), mean_longitude_of_date(jd));
            assert_eq!(mean_longitude(Graha::Rahu, jd).0, mean_node(jd));
            let ketu = mean_longitude(Graha::Ketu, jd).0;
            assert!((signed_arc(mean_node(jd), ketu) - 180.0).abs() < 1e-9);
            // The mean node regresses: about -0.0529 deg/day.
            let (_, rahu_rate) = mean_longitude(Graha::Rahu, jd);
            assert!((rahu_rate + 0.0529).abs() < 1e-3, "rate {rahu_rate}");
            assert_eq!(rahu_rate, mean_longitude(Graha::Ketu, jd).1);
        }
    }

    #[test]
    fn mean_geocentric_motion_is_the_suns_for_the_inferior_planets() {
        let jd = 2_460_000.5;
        let sun = mean_longitude(Graha::Sun, jd).1;
        assert_eq!(mean_geocentric_motion(Graha::Mercury, jd), sun);
        assert_eq!(mean_geocentric_motion(Graha::Venus, jd), sun);
        // ~0.9856 deg/day: not Mercury's heliocentric 4.09.
        assert!((sun - 0.9856).abs() < 1e-3);
        assert!(mean_longitude(Graha::Mercury, jd).1 > 4.0);
        // The superior planets keep their own heliocentric rate.
        for g in [Graha::Mars, Graha::Jupiter, Graha::Saturn] {
            assert_eq!(mean_geocentric_motion(g, jd), mean_longitude(g, jd).1);
        }
        // The Moon is a sensible 13.18 deg/day; the nodes run backward.
        assert!((mean_geocentric_motion(Graha::Moon, jd) - 13.1764).abs() < 1e-3);
        assert!(mean_geocentric_motion(Graha::Rahu, jd) < 0.0);
    }

    #[test]
    fn the_grahas_true_geocentric_pace_averages_to_the_mean_geocentric_motion() {
        // The claim behind mean_geocentric_motion, checked against the
        // engine: over a long span a planet's geocentric longitude advances
        // at the mean geocentric motion. Mars over 30 years and Mercury over
        // 40 are long enough that the periodic part is a small
        // fraction of the whole.
        use vedaksha_ephem_core::analytical::AnalyticalProvider;
        use vedaksha_ephem_core::bodies::Body;
        use vedaksha_ephem_core::coordinates::ecliptic_position_tt;

        let provider = AnalyticalProvider::new();
        let span = |body: Body, days: f64| -> f64 {
            // Unwrap the geocentric longitude over the span in 5-day steps
            // and return its average rate.
            let start = 2_455_000.5;
            let mut prev = ecliptic_position_tt(&provider, body, start)
                .expect("position")
                .longitude
                .to_degrees();
            let mut total = 0.0;
            let mut jd = start;
            while jd < start + days {
                jd += 5.0;
                let lon = ecliptic_position_tt(&provider, body, jd)
                    .expect("position")
                    .longitude
                    .to_degrees();
                total += signed_arc(prev, lon);
                prev = lon;
            }
            total / days
        };
        let mars = span(Body::Mars, 30.0 * 365.25);
        let mars_mean = mean_geocentric_motion(Graha::Mars, 2_455_000.5);
        assert!(
            (mars - mars_mean).abs() / mars_mean < 0.03,
            "Mars: observed {mars}, mean {mars_mean}"
        );
        let mercury = span(Body::Mercury, 40.0 * 365.25);
        let mercury_mean = mean_geocentric_motion(Graha::Mercury, 2_455_000.5);
        assert!(
            (mercury - mercury_mean).abs() / mercury_mean < 0.02,
            "Mercury: observed {mercury}, mean {mercury_mean}"
        );
        // And not the heliocentric rate: the figure the first design used.
        let heliocentric = mean_longitude(Graha::Mercury, 2_455_000.5).1;
        assert!((mercury - heliocentric).abs() / heliocentric > 0.5);
    }
}
