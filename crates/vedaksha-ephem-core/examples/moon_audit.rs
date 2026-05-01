//! Regression check: vedaksha apparent ecliptic positions vs JPL Horizons
//! reference values. Run with `cargo run --example moon_audit --release`.

use vedaksha_ephem_core::analytical::AnalyticalProvider;
use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::coordinates;

fn main() {
    let provider = AnalyticalProvider::new();

    // (body, jd_ut, label, ref_lon°, ref_lat°, ref_dist_au or 0)
    let cases: [(Body, f64, &str, f64, f64, f64); 6] = [
        (
            Body::Moon,
            2415020.5,
            "Moon  1900-01-01",
            272.4162663,
            1.1082671,
            0.00246250044096,
        ),
        (
            Body::Moon,
            2451545.0,
            "Moon  J2000     ",
            223.3237860,
            5.1707422,
            0.0,
        ),
        (
            Body::Moon,
            2455197.5,
            "Moon  2010-01-01",
            103.2443922,
            0.7231931,
            0.00240220510381,
        ),
        (
            Body::Moon,
            2461159.5,
            "Moon  2026-04-29",
            187.8256229,
            -2.5819510,
            0.00263707380126,
        ),
        (
            Body::Sun,
            2451545.0,
            "Sun   J2000     ",
            280.3689092,
            0.0002381,
            0.0,
        ),
        (
            Body::Mars,
            2451545.0,
            "Mars  J2000     ",
            327.9632921,
            -1.0677752,
            0.0,
        ),
    ];

    println!("=== Vedaksha apparent position vs JPL Horizons ===\n");

    for (body, jd_ut, label, ref_lon, ref_lat, ref_dist) in cases {
        let pos = coordinates::apparent_position(&provider, body, jd_ut)
            .expect("apparent_position failed");
        let lon = pos.ecliptic.longitude.to_degrees();
        let lat = pos.ecliptic.latitude.to_degrees();
        let dist_au = pos.ecliptic.distance;

        let mut dlon = lon - ref_lon;
        if dlon > 180.0 {
            dlon -= 360.0;
        }
        if dlon < -180.0 {
            dlon += 360.0;
        }
        let dlat = lat - ref_lat;

        print!(
            "{label}  Δlon = {:+7.3}\"  Δlat = {:+7.3}\"",
            dlon * 3600.0,
            dlat * 3600.0
        );
        if ref_dist > 0.0 {
            let ddist_km = (dist_au - ref_dist) * 149_597_870.7;
            println!("  Δdist = {ddist_km:+6.1} km");
        } else {
            println!();
        }
    }
}
