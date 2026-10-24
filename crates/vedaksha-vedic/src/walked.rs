// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The Moon and the Sun served from interpolated walks, for the long scans.
//!
//! Every scan in this crate and in `vedaksha-astro` takes its positions through
//! a caller callback, and every one of them asks for the same few bodies many
//! times over a span: the end of each tithi, nakshatra, karana and yoga of a
//! year, the Moon's ingresses, the lunar month and the sankrantis, an eclipse,
//! a night's moonrise or a year of sunrises, the transits of the Moon and the
//! Sun over natal points. [`WalkedSky`] is the one place that turns a provider
//! into those callbacks, each served from a
//! [`BodyWalk`](vedaksha_ephem_core::walk::BodyWalk) of the Moon or of the Sun
//! rather than from the apparent-place pipeline.
//!
//! # What it saves, measured
//!
//! On the analytical provider, counting provider calls per day of span
//! (`tests/walk_survey.rs`, once, 2026-10-24; wall time is not measured, the
//! machine was loaded):
//!
//! | scan | reads a day | calls a day, direct to walked |
//! |---|---|---|
//! | tithi, nakshatra, karana, yoga ends, every day | 18 | 271 to 12 |
//! | `lunar_month_span`, every fourth day | 13 | 228 to 13 |
//! | `grahana`, a chain of eclipses | 14 | 253 to 11 |
//! | `moon_rise_set`, consecutive days | 49 | 98 to 6 |
//! | `sun_rise_set`, consecutive days | 15 | 59 to 6 |
//! | `search_transits`, the Moon | 31 | 62 to 6 |
//! | `search_muhurta`, the 30 days the MCP tool allows | 37 | 145 to 15 |
//! | `graha_ingresses`, the Moon | 3.7 | 22 to 5 |
//! | `lunar_return`, monthly | 4.3 | 9 to 2 |
//!
//! A scan that asks for the Moon about once a day or less gains nothing, and a
//! single call that covers a few days gains little: the walk fits a segment of
//! four days (the Sun's, eight) the first time a time inside it is asked for and
//! answers from it afterwards, so it pays when one [`WalkedSky`] serves many
//! calls over a span.
//!
//! # What it does to the answers
//!
//! The walk differs from the direct pipeline by interpolation error only: at most
//! [`WalkedSky::MOON_LONGITUDE_ERROR_ARCSEC`] for the Moon and
//! [`WalkedSky::SUN_LONGITUDE_ERROR_ARCSEC`] for the Sun. At the Moon's 11 to 16
//! degrees a day that moves the root of a crossing by 0.1 ms at most, which is
//! two Julian Day quanta. [`crate::muhurta::refine_crossing`] runs to that
//! resolution, so the answers of the scans that refine with it agree with the
//! direct path to the same 0.1 ms (measured over a century: 0.08 ms worst of
//! 17,000 ends). A scan that stops on a coarser tolerance of its own agrees to
//! that tolerance: `search_transits` bisects to 1e-8 day (0.86 ms) and moved by
//! 0.64 ms at most.
//!
//! The rate a walk reports is the derivative of the interpolant, per TT day. A
//! callback built on [`apparent_position`](vedaksha_ephem_core::coordinates::apparent_position)
//! reports a central difference over a day, which differs from the derivative by
//! up to 8.5e-3 degrees a day for the Moon. A root finder that uses the rate only
//! to steer is not affected; a caller that reports the rate to its own caller
//! should know which one it has. The planets' stations (a sign change of the
//! rate) are the case where the definition decides the answer, and they are not
//! served here.
//!
//! # What is not served, and why
//!
//! - **The planets.** `deflection::solar_deflection` returns zero when the ray
//!   from a planet crosses the Sun's disc, so a planet's apparent longitude steps
//!   by about an arcsecond at that elongation, and peaks as one over the
//!   elongation near it; no segment follows either. See
//!   [`vedaksha_ephem_core::walk`].
//! - **Stations.** The rate is the whole of what they find, and the two
//!   definitions above differ in kind.
//! - **Reads outside the span.** Each method falls back to the direct pipeline
//!   for a time outside [`WalkedSky::span`] (or one the provider cannot supply a
//!   segment for), so a scan that reaches past the span is still right and only
//!   pays for the reads it makes there.
//!
//! # Threads
//!
//! A [`WalkedSky`] is `Send + Sync`: share one between the worker threads of
//! `search_muhurta` or `search_transits` and each segment is fitted once.

use vedaksha_astro::moon_riseset::moon_equatorial_walked;
use vedaksha_astro::riseset::sun_equatorial_deg_walked;
use vedaksha_astro::sidereal::{Ayanamsha, true_tropical_to_sidereal};
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    EclipticCoords, apparent_position, ecliptic_position, ecliptic_to_equatorial_deg, frame_for,
    moon_apparent_position,
};
use vedaksha_ephem_core::delta_t::ut1_to_tt;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_ephem_core::walk::{BodyWalk, WalkSpec};

use crate::khagola::{LunarGeometry, SolarGeometry};
use crate::muhurta::MoonAndSun;

/// The Moon and the Sun over one span of time, each served from a walk.
///
/// All times are Julian Days in UT1, longitudes are apparent and tropical in
/// degrees `[0, 360)`, rates are degrees a day. See the [module
/// documentation](self) for what this saves and what it does to the answers.
pub struct WalkedSky<'p> {
    provider: &'p (dyn EphemerisProvider + Sync),
    moon: BodyWalk<'p>,
    sun: BodyWalk<'p>,
}

impl<'p> WalkedSky<'p> {
    /// The Sun's segment size: eight days and twelve nodes.
    pub const SUN_SPEC: WalkSpec = WalkSpec {
        days: 8.0,
        nodes: 12,
    };

    /// Largest difference, in arcseconds, between the Moon's walked apparent
    /// longitude and [`ecliptic_position`]'s; the bound of
    /// [`BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC`].
    pub const MOON_LONGITUDE_ERROR_ARCSEC: f64 = BodyWalk::MOON_LONGITUDE_ERROR_ARCSEC;

    /// Largest difference, in arcseconds, between the Sun's walked apparent
    /// longitude and [`ecliptic_position`]'s at [`Self::SUN_SPEC`]. Measured at
    /// 1.1e-7 over 400 random instants of 2000 to 2050 and held at 1e-5 over
    /// 1900 to 2100, and across the Delta T table's joints, by this module's
    /// tests; 100 times the measurement, for the places it was not looked.
    pub const SUN_LONGITUDE_ERROR_ARCSEC: f64 = 1.0e-5;

    /// A sky over `[jd_ut_start, jd_ut_end]`. Nothing is evaluated until a time
    /// is asked for.
    ///
    /// # Panics
    ///
    /// If `jd_ut_end < jd_ut_start`.
    #[must_use]
    pub fn new(
        provider: &'p (dyn EphemerisProvider + Sync),
        jd_ut_start: f64,
        jd_ut_end: f64,
    ) -> Self {
        Self {
            provider,
            moon: BodyWalk::moon(provider, jd_ut_start, jd_ut_end),
            sun: BodyWalk::with_spec(provider, Body::Sun, jd_ut_start, jd_ut_end, Self::SUN_SPEC),
        }
    }

    /// The span the walks answer for, `(start, end)`, in UT1.
    #[must_use]
    pub fn span(&self) -> (f64, f64) {
        self.moon.span()
    }

    /// Segments fitted so far, the Moon's and the Sun's; each cost twelve calls
    /// of [`ecliptic_position_tt`](vedaksha_ephem_core::coordinates::ecliptic_position_tt).
    #[must_use]
    pub fn segments_fitted(&self) -> usize {
        self.moon.segments_fitted() + self.sun.segments_fitted()
    }

    fn moon_ecliptic(&self, jd: f64) -> Option<EclipticCoords> {
        self.moon
            .ecliptic(jd)
            .or_else(|| ecliptic_position(self.provider, Body::Moon, jd).ok())
    }

    fn sun_ecliptic(&self, jd: f64) -> Option<EclipticCoords> {
        self.sun
            .ecliptic(jd)
            .or_else(|| ecliptic_position(self.provider, Body::Sun, jd).ok())
    }

    /// The Moon's apparent tropical longitude, degrees.
    #[must_use]
    pub fn moon_longitude(&self, jd: f64) -> Option<f64> {
        self.moon_ecliptic(jd).map(|c| c.longitude.to_degrees())
    }

    /// The Sun's apparent tropical longitude, degrees.
    #[must_use]
    pub fn sun_longitude(&self, jd: f64) -> Option<f64> {
        self.sun_ecliptic(jd).map(|c| c.longitude.to_degrees())
    }

    /// The Moon's apparent tropical longitude and its rate, the derivative of
    /// the interpolant. Outside the span, from
    /// [`moon_apparent_position`], whose rate is the analytic derivative too.
    #[must_use]
    pub fn moon_longitude_rate(&self, jd: f64) -> Option<(f64, f64)> {
        if let (Some(c), Some(rate)) = (self.moon.ecliptic(jd), self.moon.longitude_speed(jd)) {
            return Some((c.longitude.to_degrees(), rate));
        }
        let p = moon_apparent_position(self.provider, jd).ok()?;
        Some((p.ecliptic.longitude.to_degrees(), p.longitude_speed))
    }

    /// The Sun's apparent tropical longitude and its rate, the derivative of the
    /// interpolant. Outside the span, from
    /// [`apparent_position`], whose rate is a central difference over a day.
    #[must_use]
    pub fn sun_longitude_rate(&self, jd: f64) -> Option<(f64, f64)> {
        if let (Some(c), Some(rate)) = (self.sun.ecliptic(jd), self.sun.longitude_speed(jd)) {
            return Some((c.longitude.to_degrees(), rate));
        }
        let p = apparent_position(self.provider, Body::Sun, jd).ok()?;
        Some((p.ecliptic.longitude.to_degrees(), p.longitude_speed))
    }

    /// The Moon and the Sun at one instant, tropical: what
    /// [`compute_tithi_end`](crate::muhurta::compute_tithi_end),
    /// [`compute_karana_end`](crate::muhurta::compute_karana_end) and the
    /// [`kala_mana`](crate::kala_mana) functions take.
    #[must_use]
    pub fn moon_and_sun(&self, jd: f64) -> Option<MoonAndSun> {
        Some((self.moon_longitude_rate(jd)?, self.sun_longitude_rate(jd)?))
    }

    /// The Moon's sidereal longitude and its rate: what
    /// [`compute_nakshatra_end`](crate::muhurta::compute_nakshatra_end) takes,
    /// and the callback of [`graha_ingresses`](crate::khagola::graha_ingresses)
    /// once the ayanamsha is removed there. The true ayanamsha of date is
    /// removed, as the MCP server removes it.
    #[must_use]
    pub fn moon_sidereal_longitude_rate(
        &self,
        ayanamsha: Ayanamsha,
        jd: f64,
    ) -> Option<(f64, f64)> {
        let (lon, rate) = self.moon_longitude_rate(jd)?;
        Some((true_tropical_to_sidereal(lon, ayanamsha, jd), rate))
    }

    /// The Moon and the Sun, sidereal: what
    /// [`compute_yoga_end`](crate::muhurta::compute_yoga_end) takes, which, unlike
    /// the tithi, must not be tropical.
    #[must_use]
    pub fn moon_and_sun_sidereal(&self, ayanamsha: Ayanamsha, jd: f64) -> Option<MoonAndSun> {
        let ((ml, mr), (sl, sr)) = self.moon_and_sun(jd)?;
        Some((
            (true_tropical_to_sidereal(ml, ayanamsha, jd), mr),
            (true_tropical_to_sidereal(sl, ayanamsha, jd), sr),
        ))
    }

    /// The Moon's sidereal longitude alone, degrees: the `get_moon_lon` of
    /// [`search_muhurta`](crate::muhurta::search_muhurta).
    #[must_use]
    pub fn moon_sidereal_longitude(&self, ayanamsha: Ayanamsha, jd: f64) -> Option<f64> {
        self.moon_longitude(jd)
            .map(|lon| true_tropical_to_sidereal(lon, ayanamsha, jd))
    }

    /// The Sun's sidereal longitude alone, degrees: the `get_sun_lon` of
    /// [`search_muhurta`](crate::muhurta::search_muhurta).
    #[must_use]
    pub fn sun_sidereal_longitude(&self, ayanamsha: Ayanamsha, jd: f64) -> Option<f64> {
        self.sun_longitude(jd)
            .map(|lon| true_tropical_to_sidereal(lon, ayanamsha, jd))
    }

    /// The Sun's and Moon's apparent ecliptic geometry: the `lunar` callback of
    /// [`grahana`](crate::khagola::grahana).
    #[must_use]
    pub fn lunar_geometry(&self, jd: f64) -> Option<LunarGeometry> {
        let (sun, moon) = (self.sun_ecliptic(jd)?, self.moon_ecliptic(jd)?);
        Some(LunarGeometry {
            sun_lon_deg: sun.longitude.to_degrees(),
            sun_dist_au: sun.distance,
            moon_lon_deg: moon.longitude.to_degrees(),
            moon_lat_deg: moon.latitude.to_degrees(),
            moon_dist_au: moon.distance,
        })
    }

    /// The Sun's and Moon's apparent equatorial geometry, by the true obliquity
    /// of date: the `solar` callback of [`grahana`](crate::khagola::grahana).
    #[must_use]
    pub fn solar_geometry(&self, jd: f64) -> Option<SolarGeometry> {
        let (sun, moon) = (self.sun_ecliptic(jd)?, self.moon_ecliptic(jd)?);
        let eps = frame_for(ut1_to_tt(jd)).true_obliquity().to_degrees();
        let equatorial = |c: &EclipticCoords| {
            ecliptic_to_equatorial_deg(c.longitude.to_degrees(), c.latitude.to_degrees(), eps)
        };
        let (sun_ra, sun_dec) = equatorial(&sun);
        let (moon_ra, moon_dec) = equatorial(&moon);
        Some(SolarGeometry {
            sun_ra_deg: sun_ra,
            sun_dec_deg: sun_dec,
            sun_dist_au: sun.distance,
            moon_ra_deg: moon_ra,
            moon_dec_deg: moon_dec,
            moon_dist_au: moon.distance,
        })
    }

    /// The Sun's apparent `(right_ascension, declination)` in degrees, by the
    /// mean obliquity: the `equatorial` callback of
    /// [`sun_rise_set`](vedaksha_astro::riseset::sun_rise_set) and
    /// [`search_muhurta`](crate::muhurta::search_muhurta); the walked form of
    /// [`sun_equatorial_deg`](vedaksha_astro::riseset::sun_equatorial_deg).
    #[must_use]
    pub fn sun_equatorial(&self, jd: f64) -> Option<(f64, f64)> {
        sun_equatorial_deg_walked(&self.sun, self.provider, jd)
    }

    /// The Moon's apparent `(right_ascension, declination, distance_km)`: the
    /// `moon` callback of
    /// [`moon_rise_set`](vedaksha_astro::moon_riseset::moon_rise_set); the walked
    /// form of [`moon_equatorial`](vedaksha_astro::moon_riseset::moon_equatorial).
    #[must_use]
    pub fn moon_equatorial(&self, jd: f64) -> Option<(f64, f64, f64)> {
        moon_equatorial_walked(&self.moon, self.provider, jd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vedaksha_astro::moon_riseset::moon_equatorial;
    use vedaksha_astro::riseset::sun_equatorial_deg;
    use vedaksha_astro::sidereal::true_ayanamsha_value;
    use vedaksha_ephem_core::analytical::AnalyticalProvider;
    use vedaksha_ephem_core::delta_t::ut1_to_tt;

    fn instants(seed: u64, count: usize, lo: f64, hi: f64) -> Vec<f64> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                lo + (state >> 11) as f64 / (1_u64 << 53) as f64 * (hi - lo)
            })
            .collect()
    }

    /// Difference of two longitudes in degrees, as arcseconds, around the wrap.
    fn arcsec_apart(a_deg: f64, b_deg: f64) -> f64 {
        ((a_deg - b_deg + 180.0).rem_euclid(360.0) - 180.0).abs() * 3600.0
    }

    #[test]
    fn a_sky_is_send_and_sync() {
        fn both<T: Send + Sync>() {}
        both::<WalkedSky<'static>>();
    }

    /// The Moon's and the Sun's walked longitudes stay inside the stated bounds
    /// at random instants over 1900 to 2100.
    #[test]
    fn walked_longitudes_stay_within_their_bounds() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_415_020.0, 2_488_070.0);
        let sky = WalkedSky::new(&provider, lo, hi);
        let (mut moon, mut sun) = (0.0_f64, 0.0_f64);
        for jd in instants(0x5C4, 300, lo, hi) {
            let want_moon = ecliptic_position(&provider, Body::Moon, jd).unwrap();
            let want_sun = ecliptic_position(&provider, Body::Sun, jd).unwrap();
            moon = moon.max(arcsec_apart(
                sky.moon_longitude(jd).unwrap(),
                want_moon.longitude.to_degrees(),
            ));
            sun = sun.max(arcsec_apart(
                sky.sun_longitude(jd).unwrap(),
                want_sun.longitude.to_degrees(),
            ));
        }
        println!("walked sky: Moon {moon:.2e}, Sun {sun:.2e} arcsec");
        assert!(
            moon <= WalkedSky::MOON_LONGITUDE_ERROR_ARCSEC,
            "Moon {moon:e}"
        );
        assert!(sun <= WalkedSky::SUN_LONGITUDE_ERROR_ARCSEC, "Sun {sun:e}");
    }

    /// The Delta T table steps; the walk applies the same table to the time it is
    /// asked about, so the Sun's bound holds across the steps too.
    #[test]
    fn the_suns_bound_holds_across_the_delta_t_table_joints() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_415_020.0, 2_450_000.0);
        let sky = WalkedSky::new(&provider, lo, hi);
        let mut joints = Vec::new();
        let mut jd = lo;
        while jd < hi && joints.len() < 30 {
            if (ut1_to_tt(jd + 0.5) - ut1_to_tt(jd) - 0.5).abs() * 86_400.0 > 1.0e-3 {
                joints.push(jd);
            }
            jd += 0.5;
        }
        assert!(
            joints.len() >= 10,
            "only {} Delta T steps found",
            joints.len()
        );
        let mut worst = 0.0_f64;
        for &j in &joints {
            for k in 0..=20 {
                let jd = j + 0.5 * f64::from(k) / 20.0;
                let want = ecliptic_position(&provider, Body::Sun, jd).unwrap();
                worst = worst.max(arcsec_apart(
                    sky.sun_longitude(jd).unwrap(),
                    want.longitude.to_degrees(),
                ));
            }
        }
        assert!(
            worst <= WalkedSky::SUN_LONGITUDE_ERROR_ARCSEC,
            "Sun off by {worst:e} arcsec across a Delta T step"
        );
    }

    /// The Moon's walked rate is the analytic derivative `moon_apparent_position`
    /// reports (they differ by a few 1e-7 degrees a day), the Sun's the central
    /// difference `apparent_position` reports to the central difference's own
    /// truncation error.
    #[test]
    fn rates_agree_with_the_direct_rates() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_455_000.0, 2_465_000.0);
        let sky = WalkedSky::new(&provider, lo, hi);
        let (mut moon, mut sun) = (0.0_f64, 0.0_f64);
        for jd in instants(0x4A7E, 200, lo, hi) {
            let walked = sky.moon_longitude_rate(jd).unwrap().1;
            let direct = moon_apparent_position(&provider, jd)
                .unwrap()
                .longitude_speed;
            moon = moon.max((walked - direct).abs());
            let walked = sky.sun_longitude_rate(jd).unwrap().1;
            let direct = apparent_position(&provider, Body::Sun, jd)
                .unwrap()
                .longitude_speed;
            sun = sun.max((walked - direct).abs());
        }
        println!("walked rates: Moon {moon:.2e}, Sun {sun:.2e} degrees a day");
        assert!(moon <= 1.0e-5, "Moon rate off by {moon:e}");
        // The Sun's one-day central difference is off by about 1e-6 degree a day.
        assert!(sun <= 1.0e-5, "Sun rate off by {sun:e}");
    }

    /// A time outside the span, or one the walk cannot serve, is read from the
    /// provider and comes back as the direct pipeline's answer to the bit; no
    /// segment is fitted for it.
    #[test]
    fn outside_the_span_a_read_is_the_direct_read() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_460_000.0, 2_460_010.0);
        let sky = WalkedSky::new(&provider, lo, hi);
        assert_eq!(sky.span(), (lo, hi));
        for jd in [lo - 5.3, hi + 7.9] {
            let moon = ecliptic_position(&provider, Body::Moon, jd).unwrap();
            let sun = ecliptic_position(&provider, Body::Sun, jd).unwrap();
            assert_eq!(
                sky.moon_longitude(jd).unwrap().to_bits(),
                moon.longitude.to_degrees().to_bits()
            );
            assert_eq!(
                sky.sun_longitude(jd).unwrap().to_bits(),
                sun.longitude.to_degrees().to_bits()
            );
            let rate = moon_apparent_position(&provider, jd)
                .unwrap()
                .longitude_speed;
            assert_eq!(
                sky.moon_longitude_rate(jd).unwrap().1.to_bits(),
                rate.to_bits()
            );
            assert_eq!(
                sky.sun_equatorial(jd),
                sun_equatorial_deg(&provider, jd),
                "sun equatorial at {jd}"
            );
            assert_eq!(
                sky.moon_equatorial(jd),
                moon_equatorial(&provider, jd),
                "moon equatorial at {jd}"
            );
        }
        assert_eq!(sky.segments_fitted(), 0, "outside reads fitted a segment");
        let _ = sky.moon_longitude(lo + 1.0).unwrap();
        assert_eq!(sky.segments_fitted(), 1);
        let _ = sky.sun_longitude(lo + 1.0).unwrap();
        assert_eq!(sky.segments_fitted(), 2);
    }

    /// The equatorial and geometry forms are the provider forms through the walk:
    /// the same formulas, so they agree to what the longitude, latitude and
    /// distance do (1.4e-8 degree, 5e-5 km).
    #[test]
    fn the_derived_shapes_agree_with_the_provider_forms() {
        let provider = AnalyticalProvider::new();
        let (lo, hi) = (2_455_000.0, 2_460_000.0);
        let sky = WalkedSky::new(&provider, lo, hi);
        let mut worst = [0.0_f64; 4];
        for jd in instants(0xD15C, 150, lo, hi) {
            let (ra, dec, km) = sky.moon_equatorial(jd).unwrap();
            let (wra, wdec, wkm) = moon_equatorial(&provider, jd).unwrap();
            worst[0] = worst[0]
                .max(arcsec_apart(ra, wra))
                .max(arcsec_apart(dec, wdec));
            worst[1] = worst[1].max((km - wkm).abs());
            let (ra, dec) = sky.sun_equatorial(jd).unwrap();
            let (wra, wdec) = sun_equatorial_deg(&provider, jd).unwrap();
            worst[2] = worst[2]
                .max(arcsec_apart(ra, wra))
                .max(arcsec_apart(dec, wdec));

            let g = sky.lunar_geometry(jd).unwrap();
            let moon = ecliptic_position(&provider, Body::Moon, jd).unwrap();
            let sun = ecliptic_position(&provider, Body::Sun, jd).unwrap();
            worst[3] = worst[3]
                .max(arcsec_apart(g.moon_lon_deg, moon.longitude.to_degrees()))
                .max((g.moon_lat_deg - moon.latitude.to_degrees()).abs() * 3600.0)
                .max(arcsec_apart(g.sun_lon_deg, sun.longitude.to_degrees()));
            assert!((g.moon_dist_au - moon.distance).abs() < 1.0e-12);
            assert!((g.sun_dist_au - sun.distance).abs() < 1.0e-12);

            let s = sky.solar_geometry(jd).unwrap();
            let eps = frame_for(ut1_to_tt(jd)).true_obliquity().to_degrees();
            let (moon_ra, moon_dec) = ecliptic_to_equatorial_deg(
                moon.longitude.to_degrees(),
                moon.latitude.to_degrees(),
                eps,
            );
            worst[3] = worst[3]
                .max(arcsec_apart(s.moon_ra_deg, moon_ra))
                .max((s.moon_dec_deg - moon_dec).abs() * 3600.0);
        }
        println!("derived shapes: {worst:?} (arcsec, km, arcsec, arcsec)");
        assert!(worst[0] < 1.0e-4, "Moon ra/dec {:e} arcsec", worst[0]);
        assert!(worst[1] < 1.0e-3, "Moon distance {:e} km", worst[1]);
        assert!(worst[2] < 1.0e-4, "Sun ra/dec {:e} arcsec", worst[2]);
        assert!(worst[3] < 1.0e-4, "geometry {:e} arcsec", worst[3]);
    }

    /// The sidereal forms remove the true ayanamsha of date, as the MCP server
    /// does, from the tropical ones.
    #[test]
    fn the_sidereal_forms_remove_the_true_ayanamsha() {
        let provider = AnalyticalProvider::new();
        let sky = WalkedSky::new(&provider, 2_460_000.0, 2_460_100.0);
        let aya = Ayanamsha::IndianOfficial;
        for jd in [2_460_003.25, 2_460_050.5, 2_460_099.75] {
            let shift = true_ayanamsha_value(aya, jd);
            let tropical = sky.moon_longitude(jd).unwrap();
            let sidereal = sky.moon_sidereal_longitude(aya, jd).unwrap();
            assert!(arcsec_apart(sidereal, tropical - shift) < 1.0e-9);
            let ((ml, mr), (sl, sr)) = sky.moon_and_sun_sidereal(aya, jd).unwrap();
            let ((tml, tmr), (tsl, tsr)) = sky.moon_and_sun(jd).unwrap();
            assert!(arcsec_apart(ml, tml - shift) < 1.0e-9);
            assert!(arcsec_apart(sl, tsl - shift) < 1.0e-9);
            assert_eq!((mr.to_bits(), sr.to_bits()), (tmr.to_bits(), tsr.to_bits()));
            let (lon, rate) = sky.moon_sidereal_longitude_rate(aya, jd).unwrap();
            assert_eq!(
                (lon.to_bits(), rate.to_bits()),
                (ml.to_bits(), mr.to_bits())
            );
            assert!(arcsec_apart(sky.sun_sidereal_longitude(aya, jd).unwrap(), sl) < 1.0e-9);
        }
    }
}
