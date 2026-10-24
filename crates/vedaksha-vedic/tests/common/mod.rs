// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Positions for the khagola integration tests, taken from the engine the way a
//! consumer takes them: apparent positions of date (UT1 in), true obliquity for
//! the equatorial form.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::{
    apparent_position, apparent_positions, ecliptic_to_equatorial_deg, frame_for,
};
use vedaksha_ephem_core::delta_t;
use vedaksha_ephem_core::error::ComputeError;
use vedaksha_ephem_core::jpl::{EphemerisProvider, Position, StateVector};
use vedaksha_vedic::khagola::{LunarGeometry, SolarGeometry};

/// Julian Day of a Gregorian calendar date and fractional hours (Meeus Ch. 7).
pub fn jd(y: i32, m: u32, d: u32, hours: f64) -> f64 {
    let (mut yy, mut mm) = (f64::from(y), f64::from(m));
    if mm <= 2.0 {
        yy -= 1.0;
        mm += 12.0;
    }
    let a = (yy / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (yy + 4716.0)).floor() + (30.6001 * (mm + 1.0)).floor() + f64::from(d) + b - 1524.5
        + hours / 24.0
}

/// TD minus UT, seconds, as the engine itself uses it.
pub fn delta_t_s(jd_ut: f64) -> f64 {
    delta_t::delta_t(jd_ut)
}

/// The Sun's and Moon's apparent ecliptic geometry at a UT1 instant.
pub fn lunar_geometry(p: &(dyn EphemerisProvider + Sync), jd_ut: f64) -> Option<LunarGeometry> {
    let r = apparent_positions(p, &[Body::Sun, Body::Moon], jd_ut);
    let sun = r[0].1.as_ref().ok()?;
    let moon = r[1].1.as_ref().ok()?;
    Some(LunarGeometry {
        sun_lon_deg: sun.ecliptic.longitude.to_degrees(),
        sun_dist_au: sun.ecliptic.distance,
        moon_lon_deg: moon.ecliptic.longitude.to_degrees(),
        moon_lat_deg: moon.ecliptic.latitude.to_degrees(),
        moon_dist_au: moon.ecliptic.distance,
    })
}

/// The Sun's and Moon's apparent equatorial geometry at a UT1 instant, the
/// ecliptic positions rotated by the true obliquity of date.
pub fn solar_geometry(p: &(dyn EphemerisProvider + Sync), jd_ut: f64) -> Option<SolarGeometry> {
    let r = apparent_positions(p, &[Body::Sun, Body::Moon], jd_ut);
    let sun = r[0].1.as_ref().ok()?;
    let moon = r[1].1.as_ref().ok()?;
    let eps = frame_for(delta_t::ut1_to_tt(jd_ut))
        .true_obliquity()
        .to_degrees();
    let eq = |c: &vedaksha_ephem_core::coordinates::EclipticCoords| {
        ecliptic_to_equatorial_deg(c.longitude.to_degrees(), c.latitude.to_degrees(), eps)
    };
    let (sun_ra, sun_dec) = eq(&sun.ecliptic);
    let (moon_ra, moon_dec) = eq(&moon.ecliptic);
    Some(SolarGeometry {
        sun_ra_deg: sun_ra,
        sun_dec_deg: sun_dec,
        sun_dist_au: sun.ecliptic.distance,
        moon_ra_deg: moon_ra,
        moon_dec_deg: moon_dec,
        moon_dist_au: moon.ecliptic.distance,
    })
}

/// A body's apparent tropical longitude and daily motion, degrees.
pub fn longitude_and_rate(
    p: &(dyn EphemerisProvider + Sync),
    body: Body,
    jd_ut: f64,
) -> Option<(f64, f64)> {
    let a = apparent_position(p, body, jd_ut).ok()?;
    Some((a.ecliptic.longitude.to_degrees(), a.longitude_speed))
}

/// A body's apparent `(right ascension, declination)`, degrees.
pub fn equatorial(
    p: &(dyn EphemerisProvider + Sync),
    body: Body,
    jd_ut: f64,
) -> Option<(f64, f64)> {
    let a = apparent_position(p, body, jd_ut).ok()?;
    let eps = frame_for(delta_t::ut1_to_tt(jd_ut))
        .true_obliquity()
        .to_degrees();
    Some(ecliptic_to_equatorial_deg(
        a.ecliptic.longitude.to_degrees(),
        a.ecliptic.latitude.to_degrees(),
        eps,
    ))
}

/// `AnalyticalProvider`, counting every call that reaches it: the instrument for
/// "how many provider calls did this scan make".
pub struct Counting {
    pub state: AtomicU64,
    pub earth: AtomicU64,
    pub moon_position: AtomicU64,
    pub moon_distance: AtomicU64,
}

impl Counting {
    pub const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            earth: AtomicU64::new(0),
            moon_position: AtomicU64::new(0),
            moon_distance: AtomicU64::new(0),
        }
    }

    /// Every call so far, of any kind.
    pub fn total(&self) -> u64 {
        self.state.load(Relaxed)
            + self.earth.load(Relaxed)
            + self.moon_position.load(Relaxed)
            + self.moon_distance.load(Relaxed)
    }
}

impl EphemerisProvider for Counting {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        self.state.fetch_add(1, Relaxed);
        AnalyticalProvider.compute_state(body, jd)
    }
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        self.earth.fetch_add(1, Relaxed);
        AnalyticalProvider.earth_state(jd)
    }
    fn moon_position(&self, jd: f64) -> Result<Position, ComputeError> {
        self.moon_position.fetch_add(1, Relaxed);
        AnalyticalProvider.moon_position(jd)
    }
    fn moon_distance(&self, jd: f64) -> Option<f64> {
        self.moon_distance.fetch_add(1, Relaxed);
        AnalyticalProvider.moon_distance(jd)
    }
    fn time_range(&self) -> (f64, f64) {
        AnalyticalProvider.time_range()
    }
}
