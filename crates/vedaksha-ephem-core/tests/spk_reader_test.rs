// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Integration tests for the SPK reader against DE440s data.

use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::jpl::EphemerisProvider;
use vedaksha_ephem_core::jpl::reader::SpkReader;
use vedaksha_ephem_core::julian::J2000;

/// Path to DE440s test data relative to crate root.
const BSP_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/de440s.bsp");

fn open_reader() -> SpkReader {
    SpkReader::open(BSP_PATH).expect("failed to open de440s.bsp")
}

fn distance(pos: &vedaksha_ephem_core::jpl::Position) -> f64 {
    (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt()
}

#[test]
fn earth_emb_at_j2000_is_about_1au() {
    let reader = open_reader();
    let state = reader
        .compute_state(Body::EarthMoonBarycenter, J2000)
        .expect("EMB at J2000");
    let r = distance(&state.position);
    assert!(
        (0.98..=1.02).contains(&r),
        "EMB distance from SSB at J2000 should be ~1 AU, got {r}"
    );
}

#[test]
fn mars_at_j2000_is_about_1_5au() {
    let reader = open_reader();
    let state = reader
        .compute_state(Body::Mars, J2000)
        .expect("Mars at J2000");
    let r = distance(&state.position);
    assert!(
        (1.3..=1.7).contains(&r),
        "Mars distance from SSB at J2000 should be 1.3-1.7 AU, got {r}"
    );
}

#[test]
fn sun_at_j2000_is_near_origin() {
    let reader = open_reader();
    let state = reader
        .compute_state(Body::Sun, J2000)
        .expect("Sun at J2000");
    let r = distance(&state.position);
    assert!(
        r < 0.01,
        "Sun distance from SSB at J2000 should be < 0.01 AU, got {r}"
    );
}

#[test]
fn mercury_position_changes_over_time() {
    let reader = open_reader();
    let state1 = reader
        .compute_state(Body::Mercury, J2000)
        .expect("Mercury at J2000");
    let state2 = reader
        .compute_state(Body::Mercury, J2000 + 30.0)
        .expect("Mercury at J2000+30d");

    let dx = state2.position.x - state1.position.x;
    let dy = state2.position.y - state1.position.y;
    let dz = state2.position.z - state1.position.z;
    let dr = (dx * dx + dy * dy + dz * dz).sqrt();

    assert!(
        dr > 0.01,
        "Mercury should move significantly over 30 days, got dr={dr}"
    );
}

#[test]
fn out_of_range_returns_error() {
    let reader = open_reader();
    // JD far outside 1849-2150 range: year 1000
    let jd_year_1000 = 2_086_302.5;
    let result = reader.compute_state(Body::Mars, jd_year_1000);
    assert!(
        result.is_err(),
        "should return error for epoch far outside range"
    );
}

#[test]
fn moon_position_relative_to_emb() {
    let reader = open_reader();
    // Moon (target=301, center=3/EMB) — the reader returns Moon relative to EMB
    let state = reader
        .compute_state(Body::Moon, J2000)
        .expect("Moon at J2000");
    let r = distance(&state.position);
    // Moon is ~384400 km from Earth, ~0.00257 AU from EMB
    assert!(
        (0.001..=0.005).contains(&r),
        "Moon distance from EMB at J2000 should be ~0.00257 AU, got {r}"
    );
}

#[test]
fn velocity_is_nonzero() {
    let reader = open_reader();
    let state = reader
        .compute_state(Body::EarthMoonBarycenter, J2000)
        .expect("EMB at J2000");
    let v = (state.velocity.x.powi(2) + state.velocity.y.powi(2) + state.velocity.z.powi(2)).sqrt();
    // Earth orbital velocity ~30 km/s ≈ 0.0173 AU/day
    assert!(
        (0.01..=0.02).contains(&v),
        "EMB velocity should be ~0.0173 AU/day, got {v}"
    );
}

#[test]
fn time_range_covers_expected_period() {
    let reader = open_reader();
    let (min_jd, max_jd) = reader.time_range();
    // DE440s covers ~1849-2150 CE
    // 1849 Jan 1 ≈ JD 2396758, 2150 Dec 31 ≈ JD 2506985
    assert!(min_jd < 2_400_000.0, "min JD should be before 1849");
    assert!(max_jd > 2_500_000.0, "max JD should be after 2150");
}
