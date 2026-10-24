// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The Sun's light deflection, held to JPL Horizons where it matters most:
//! planets seen close to the Sun.
//!
//! Far from the Sun the bending is a few milliarcseconds and hides inside the
//! oracle's 0.1 arcsecond floor. Within two degrees it is 0.1 to 1.7
//! arcseconds, and an engine that leaves it out shows it plainly: before
//! `deflection.rs` existed the worst residual of the whole 24,350-row fixture,
//! 1.187 arcseconds (Uranus, 1948), was a planet 0.36 degrees from the Sun, and
//! the next five worst were within a degree of it too.
//!
//! This test selects, from the fixture itself, every planet row whose Horizons
//! longitude and latitude lie within two degrees of Horizons' own Sun at the
//! same instant (so the selection does not depend on the engine under test) and
//! requires the engine to agree with Horizons on those rows as well as it does
//! anywhere else. With the deflection step disabled the maximum here is 1.19
//! arcseconds and this fails.
//!
//! The selection also contains the rows where the ray crosses the solar disc
//! (Neptune 2003, Saturn 1961, Uranus 1945 and 1984, 0.04 to 0.25 degrees from
//! the Sun; 155 rows in all lie within two degrees). There Horizons applies no deflection, the formula is singular, and
//! the engine returns the undeflected place; applying the formula anyway leaves
//! errors of 1.7 to 6.1 arcseconds on those four rows. They are asserted
//! separately so the cut at the limb cannot be lost unnoticed.

mod common;

use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;
use vedaksha_ephem_core::jpl::reader::SpkReader;

#[derive(serde::Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(serde::Deserialize)]
struct Row {
    jd: f64,
    body: String,
    ref_longitude: f64,
    ref_latitude: f64,
}

/// JD of 2026-01-01: the oracle's boundary between measured and predicted
/// Delta T (see `oracle_comparison.rs`).
const MEASURED_DT_JD: f64 = 2_461_041.5;

/// The Sun's apparent angular radius at 1 AU, in degrees: 0.2665.
const SOLAR_DISC_DEG: f64 = 0.2665;

fn planet(name: &str) -> Option<Body> {
    match name {
        "Mercury" => Some(Body::Mercury),
        "Venus" => Some(Body::Venus),
        "Mars" => Some(Body::Mars),
        "Jupiter" => Some(Body::Jupiter),
        "Saturn" => Some(Body::Saturn),
        "Uranus" => Some(Body::Uranus),
        "Neptune" => Some(Body::Neptune),
        "Pluto" => Some(Body::Pluto),
        _ => None,
    }
}

/// Angular separation in degrees between two ecliptic positions given in
/// degrees (haversine form, accurate down to arcseconds).
fn separation_deg(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let (l1, b1, l2, b2) = (
        lon1.to_radians(),
        lat1.to_radians(),
        lon2.to_radians(),
        lat2.to_radians(),
    );
    let h = ((b2 - b1) / 2.0).sin().powi(2) + b1.cos() * b2.cos() * ((l2 - l1) / 2.0).sin().powi(2);
    (2.0 * h.sqrt().asin()).to_degrees()
}

#[test]
fn planets_near_the_sun_agree_with_horizons() {
    if !common::require_bsp() || !common::require_horizons_oracle() {
        return;
    }
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(common::horizons_oracle_path()).unwrap())
            .unwrap();
    let reader = SpkReader::open(&common::bsp_path()).unwrap();

    // Horizons' own Sun, by instant.
    let sun: std::collections::HashMap<u64, (f64, f64)> = fixture
        .rows
        .iter()
        .filter(|r| r.body == "Sun")
        .map(|r| (r.jd.to_bits(), (r.ref_longitude, r.ref_latitude)))
        .collect();
    assert!(sun.len() > 2_000, "Sun rows in the fixture: {}", sun.len());

    let mut near = Vec::new();
    for r in &fixture.rows {
        let Some(body) = planet(&r.body) else {
            continue;
        };
        if r.jd >= MEASURED_DT_JD {
            continue;
        }
        let &(sun_lon, sun_lat) = sun
            .get(&r.jd.to_bits())
            .expect("a Sun row at every instant");
        let sep = separation_deg(r.ref_longitude, r.ref_latitude, sun_lon, sun_lat);
        if sep >= 2.0 {
            continue;
        }
        let got = coordinates::ecliptic_position(&reader, body, r.jd).unwrap();
        let mut d = (got.longitude.to_degrees() - r.ref_longitude).abs();
        if d > 180.0 {
            d = 360.0 - d;
        }
        near.push((sep, r.body.clone(), r.jd, d * 3600.0));
    }

    // The denominator: this must be a real population, with the disc cases in
    // it, or the assertions below could not have failed.
    assert!(
        near.len() >= 120,
        "only {} planet rows within 2° of the Sun; the selection is not what it was",
        near.len()
    );
    let disc: Vec<_> = near.iter().filter(|r| r.0 < SOLAR_DISC_DEG).collect();
    assert!(
        disc.len() >= 3,
        "expected the fixture's rows behind the solar disc, found {}",
        disc.len()
    );

    let worst = near.iter().max_by(|a, b| a.3.total_cmp(&b.3)).unwrap();
    assert!(
        worst.3 < 0.35,
        "worst planet row within 2° of the Sun: {} at JD {} ({:.3}° from the Sun) is off \
         Horizons by {:.3}″",
        worst.1,
        worst.2,
        worst.0,
        worst.3
    );

    let worst_disc = disc.iter().max_by(|a, b| a.3.total_cmp(&b.3)).unwrap();
    assert!(
        worst_disc.3 < 0.35,
        "a ray through the solar disc was deflected: {} at JD {} ({:.3}° from the Sun) is off \
         Horizons by {:.3}″",
        worst_disc.1,
        worst_disc.2,
        worst_disc.0,
        worst_disc.3
    );

    eprintln!(
        "{} planet rows within 2° of the Sun ({} behind the disc): worst {:.3}″, \
         worst behind the disc {:.3}″",
        near.len(),
        disc.len(),
        worst.3,
        worst_disc.3
    );
}
