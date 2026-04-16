// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Integration tests for the coordinate transformation pipeline.
//!
//! These tests require the JPL DE440s ephemeris file (`data/de440s.bsp`)
//! which is not checked into the repository. Tests are automatically
//! skipped when the file is absent (e.g. in CI).

use core::f64::consts::PI;

use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates::apparent_position;
use vedaksha_ephem_core::jpl::reader::SpkReader;
use vedaksha_ephem_core::julian::J2000;

/// Path to DE440s test data relative to crate root.
const BSP_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/de440s.bsp");

/// Open the SPK reader, or return `None` if the file is missing.
fn try_open_reader() -> Option<SpkReader> {
    SpkReader::open(BSP_PATH).ok()
}

/// Helper macro: skip the test when de440s.bsp is absent.
macro_rules! require_bsp {
    () => {
        match try_open_reader() {
            Some(r) => r,
            None => {
                eprintln!("SKIPPED: de440s.bsp not found at {BSP_PATH}");
                return;
            }
        }
    };
}

#[test]
fn sun_longitude_at_j2000() {
    // J2000 = Jan 1.5 2000, Sun longitude should be roughly 280.5 degrees (Capricorn)
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Sun, J2000).unwrap();
    let lon_deg = pos.ecliptic.longitude * 180.0 / PI;
    assert!(
        lon_deg > 275.0 && lon_deg < 285.0,
        "Sun longitude at J2000: {lon_deg} degrees (expected ~280.5)"
    );
}

#[test]
fn sun_distance_at_j2000() {
    // Sun distance around Jan 1.5 is approximately 0.983 AU (near perihelion ~Jan 3)
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Sun, J2000).unwrap();
    assert!(
        pos.ecliptic.distance > 0.97 && pos.ecliptic.distance < 1.0,
        "Sun distance at J2000: {} AU (expected ~0.983)",
        pos.ecliptic.distance
    );
}

#[test]
fn sun_latitude_near_zero() {
    // The Sun should always be very close to the ecliptic plane
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Sun, J2000).unwrap();
    let lat_deg = pos.ecliptic.latitude.abs() * 180.0 / PI;
    assert!(
        lat_deg < 0.01,
        "Sun ecliptic latitude should be near zero: {lat_deg} degrees"
    );
}

#[test]
fn moon_longitude_reasonable() {
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Moon, J2000).unwrap();
    let lon_deg = pos.ecliptic.longitude * 180.0 / PI;
    assert!(
        (0.0..360.0).contains(&lon_deg),
        "Moon longitude out of range: {lon_deg} degrees"
    );
}

#[test]
fn moon_distance_reasonable() {
    // Moon distance from Earth ~ 0.00257 AU (384400 km)
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Moon, J2000).unwrap();
    assert!(
        pos.ecliptic.distance > 0.002 && pos.ecliptic.distance < 0.003,
        "Moon distance: {} AU (expected ~0.00257)",
        pos.ecliptic.distance
    );
}

#[test]
fn mars_longitude_reasonable() {
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Mars, J2000).unwrap();
    let lon_deg = pos.ecliptic.longitude * 180.0 / PI;
    assert!(
        (0.0..360.0).contains(&lon_deg),
        "Mars longitude out of range: {lon_deg} degrees"
    );
    // Mars geocentric distance at J2000 should be reasonable (1-3 AU)
    assert!(
        pos.ecliptic.distance > 1.0 && pos.ecliptic.distance < 3.0,
        "Mars distance: {} AU",
        pos.ecliptic.distance
    );
}

#[test]
fn sun_speed_is_about_1_degree_per_day() {
    // Sun moves roughly 1 degree/day in ecliptic longitude
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Sun, J2000).unwrap();
    assert!(
        pos.longitude_speed > 0.9 && pos.longitude_speed < 1.1,
        "Sun speed: {} deg/day (expected ~1.0)",
        pos.longitude_speed
    );
}

#[test]
fn all_planets_compute_successfully() {
    let reader = require_bsp!();
    let bodies = [
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];
    for body in &bodies {
        let result = apparent_position(&reader, *body, J2000);
        assert!(result.is_ok(), "{} failed: {:?}", body.name(), result.err());
        let pos = result.unwrap();
        let lon = pos.ecliptic.longitude * 180.0 / PI;
        assert!(
            (0.0..360.0).contains(&lon),
            "{} longitude out of range: {lon} degrees",
            body.name()
        );
    }
}

#[test]
fn moon_speed_reasonable() {
    // Moon moves roughly 13 degrees/day
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Moon, J2000).unwrap();
    assert!(
        pos.longitude_speed > 10.0 && pos.longitude_speed < 16.0,
        "Moon speed: {} deg/day (expected ~13)",
        pos.longitude_speed
    );
}

#[test]
fn jupiter_speed_reasonable() {
    // Jupiter moves roughly 0.08 degrees/day (5 arcminutes)
    let reader = require_bsp!();
    let pos = apparent_position(&reader, Body::Jupiter, J2000).unwrap();
    assert!(
        pos.longitude_speed.abs() < 0.5,
        "Jupiter speed: {} deg/day (expected small)",
        pos.longitude_speed
    );
}
