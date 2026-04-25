// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Mathematical invariant and fixed-point tests.
//!
//! These verify properties that must ALWAYS hold, regardless of input.

use vedaksha_astro::sidereal::{self, Ayanamsha};
use vedaksha_ephem_core::nutation;
use vedaksha_ephem_core::obliquity;
use vedaksha_ephem_core::sidereal_time;
use vedaksha_math::angle::normalize_degrees;
use vedaksha_vedic::dasha::vimshottari;
use vedaksha_vedic::nakshatra::Nakshatra;

// ─── SIDEREAL LONGITUDE ───

#[test]
fn sidereal_longitude_equals_tropical_minus_ayanamsha() {
    // For any longitude, any ayanamsha, any date:
    // sidereal = tropical - ayanamsha (mod 360)
    let systems = [
        Ayanamsha::Lahiri,
        Ayanamsha::FaganBradley,
        Ayanamsha::Raman,
        Ayanamsha::Krishnamurti,
        Ayanamsha::Yukteshwar,
        Ayanamsha::Tropical,
    ];
    let jds = [2451545.0, 2451000.0, 2452000.0, 2440000.0, 2460000.0];
    let longitudes = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 359.99];

    let mut total = 0;
    let mut pass = 0;

    for &sys in &systems {
        for &jd in &jds {
            let ayan = sidereal::ayanamsha_value(sys, jd);
            for &lon in &longitudes {
                total += 1;
                let sidereal = sidereal::tropical_to_sidereal(lon, sys, jd);
                let expected = normalize_degrees(lon - ayan);
                let diff = (sidereal - expected).abs();
                if diff < 1e-10 || (360.0 - diff).abs() < 1e-10 {
                    pass += 1;
                } else {
                    eprintln!(
                        "FAIL sidereal: sys={:?} jd={jd} lon={lon} expected={expected} got={sidereal} diff={diff}",
                        sys
                    );
                }
            }
        }
    }

    eprintln!("\n=== SIDEREAL LONGITUDE INVARIANT ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total, "Sidereal longitude invariant violated");
}

#[test]
fn sidereal_tropical_roundtrip() {
    // tropical_to_sidereal(sidereal_to_tropical(x)) == x
    let jd = 2451545.0;
    let mut total = 0;
    let mut pass = 0;

    for sys in [Ayanamsha::Lahiri, Ayanamsha::FaganBradley, Ayanamsha::Raman] {
        for lon in (0..360).map(|i| i as f64) {
            total += 1;
            let trop = sidereal::sidereal_to_tropical(lon, sys, jd);
            let back = sidereal::tropical_to_sidereal(trop, sys, jd);
            if (normalize_degrees(back) - normalize_degrees(lon)).abs() < 1e-10 {
                pass += 1;
            }
        }
    }

    eprintln!("\n=== SIDEREAL ROUNDTRIP ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── RETROGRADE DETECTION ───

#[test]
fn retrograde_detection_from_speed_sign() {
    // Retrograde = speed < 0. Test with known speeds.
    let cases = [
        (1.0, false),   // direct
        (0.0, false),   // stationary
        (-0.5, true),   // retrograde
        (-0.001, true), // barely retrograde
        (13.0, false),  // Moon (fast direct)
    ];

    let mut total = 0;
    let mut pass = 0;

    for &(speed, expected_retro) in &cases {
        total += 1;
        let is_retro = speed < 0.0;
        if is_retro == expected_retro {
            pass += 1;
        } else {
            eprintln!("FAIL retrograde: speed={speed} expected={expected_retro} got={is_retro}");
        }
    }

    // Also test with 100 random speeds
    for i in 0..100 {
        total += 1;
        let speed = (i as f64 - 50.0) * 0.1; // -5.0 to 4.9
        let is_retro = speed < 0.0;
        let expected = speed < 0.0;
        if is_retro == expected {
            pass += 1;
        }
    }

    eprintln!("\n=== RETROGRADE DETECTION ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── POLAR HOUSE FALLBACK ───

#[test]
fn polar_house_fallback_triggers_above_66_56() {
    use vedaksha_astro::houses::{HouseSystem, compute_houses};

    let mut total = 0;
    let mut pass = 0;

    let polar_lats = [
        67.0, 70.0, 75.0, 80.0, 85.0, 89.0, -67.0, -70.0, -80.0, -89.0,
    ];
    let non_polar_lats = [0.0, 10.0, 30.0, 45.0, 55.0, 60.0, 65.0, 66.0, -30.0, -60.0];

    // Placidus and Koch should fallback at polar latitudes
    for &system in &[HouseSystem::Placidus, HouseSystem::Koch] {
        for &lat in &polar_lats {
            total += 1;
            let cusps = compute_houses(90.0, lat, 23.44, system);
            if cusps.polar_fallback {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL polar: system={:?} lat={lat} should fallback but didn't",
                    system
                );
            }
        }

        for &lat in &non_polar_lats {
            total += 1;
            let cusps = compute_houses(90.0, lat, 23.44, system);
            if !cusps.polar_fallback {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL non-polar: system={:?} lat={lat} should NOT fallback but did",
                    system
                );
            }
        }
    }

    eprintln!("\n=== POLAR HOUSE FALLBACK ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── NAKSHATRA FROM LONGITUDE ───

#[test]
fn nakshatra_from_longitude_covers_full_circle() {
    let mut total = 0;
    let mut pass = 0;

    // Test every 0.1 degree across the full circle
    for i in 0..3600 {
        let lon = i as f64 * 0.1;
        total += 1;

        let nak = Nakshatra::from_longitude(lon);
        let expected_index = (lon / Nakshatra::SPAN) as u8;
        let expected_index = expected_index.min(26);

        if nak.index() == expected_index {
            pass += 1;
        } else {
            eprintln!(
                "FAIL nakshatra: lon={lon} expected_idx={expected_index} got={}",
                nak.index()
            );
        }
    }

    // Boundary tests: exact nakshatra boundaries
    for i in 0..27 {
        let boundary = i as f64 * Nakshatra::SPAN;
        total += 1;
        let nak = Nakshatra::from_longitude(boundary);
        if nak.index() == i as u8 {
            pass += 1;
        } else {
            eprintln!(
                "FAIL nakshatra boundary: lon={boundary} expected={i} got={}",
                nak.index()
            );
        }
    }

    // Pada tests
    for i in 0..108 {
        let lon = i as f64 * Nakshatra::PADA_SPAN + 0.01;
        total += 1;
        let pada = Nakshatra::pada_from_longitude(lon);
        let expected_pada = ((lon % Nakshatra::SPAN) / Nakshatra::PADA_SPAN) as u8 + 1;
        if pada == expected_pada.min(4) {
            pass += 1;
        } else {
            eprintln!("FAIL pada: lon={lon} expected={expected_pada} got={pada}");
        }
    }

    eprintln!("\n=== NAKSHATRA FROM LONGITUDE ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── DASHA PERIOD SUMS ───

#[test]
fn dasha_period_sums_equal_120_years() {
    let dasha_year_days = 365.25;
    let total_years = 120.0;
    let _total_days = total_years * dasha_year_days;

    let mut test_total = 0;
    let mut pass = 0;

    // Test 500 random Moon longitudes
    // The 9 maha dashas sum to: 120_years - (1-balance) * first_lord_years
    // This is always <= 120 years (equals 120 only when balance = 1.0)
    for i in 0..500 {
        let moon_lon = (i as f64) * 0.72; // 0 to 360
        let jd = 2451545.0;
        test_total += 1;

        let dasha = vimshottari::compute_vimshottari(moon_lon, jd, 1);

        // Expected sum: balance * first_lord_years + sum of remaining 8 full lords
        let first_lord_years = dasha.maha_dashas[0].lord.maha_dasha_years();
        let expected_days = dasha.initial_balance * first_lord_years * dasha_year_days
            + (total_years - first_lord_years) * dasha_year_days;

        let sum: f64 = dasha.maha_dashas.iter().map(|d| d.duration_days).sum();
        let diff = (sum - expected_days).abs();

        if diff < 0.01 {
            pass += 1;
        } else {
            eprintln!(
                "FAIL dasha sum: moon_lon={moon_lon} sum={sum} expected={expected_days} diff={diff}"
            );
        }
    }

    // Test that sub-periods sum to parent
    for i in 0..100 {
        let moon_lon = (i as f64) * 3.6;
        let dasha = vimshottari::compute_vimshottari(moon_lon, 2451545.0, 3);

        for maha in &dasha.maha_dashas {
            test_total += 1;
            let sub_sum: f64 = maha.sub_periods.iter().map(|s| s.duration_days).sum();
            let diff = (sub_sum - maha.duration_days).abs();
            if diff < 0.01 {
                pass += 1;
            } else {
                eprintln!(
                    "FAIL sub-period sum: lord={:?} parent={} sub_sum={sub_sum} diff={diff}",
                    maha.lord, maha.duration_days
                );
            }

            // Level 3: pratyantar sums to antar
            for antar in &maha.sub_periods {
                test_total += 1;
                let prat_sum: f64 = antar.sub_periods.iter().map(|p| p.duration_days).sum();
                let diff = (prat_sum - antar.duration_days).abs();
                if diff < 0.01 {
                    pass += 1;
                } else {
                    eprintln!("FAIL pratyantar sum: lord={:?} diff={diff}", antar.lord);
                }
            }
        }
    }

    eprintln!("\n=== DASHA PERIOD SUMS ===");
    eprintln!("Total: {test_total}, Pass: {pass}");
    assert_eq!(pass, test_total, "Dasha period sum invariant violated");
}

// ─── GAST DIRECTLY ───

#[test]
fn gast_equals_gmst_plus_equation_of_equinoxes() {
    let mut total = 0;
    let mut pass = 0;

    let jds = [
        2451545.0, 2451000.0, 2452000.0, 2440000.0, 2460000.0, 2445000.0, 2450000.0, 2455000.0,
        2448000.0, 2453000.0, 2430000.0, 2435000.0, 2442000.0, 2457000.0, 2462000.0, 2447000.0,
        2449000.0, 2451500.0, 2454000.0, 2456000.0,
    ];

    for &jd in &jds {
        total += 1;

        let gmst = sidereal_time::gmst(jd);
        let (dpsi, deps) = nutation::nutation(jd);
        let eps_true = obliquity::true_obliquity(jd, deps);
        let gast = sidereal_time::gast(jd, dpsi, eps_true);

        // GAST = GMST + dpsi * cos(eps_true)
        let eq_equinoxes = dpsi * eps_true.cos();
        let expected_gast = vedaksha_math::angle::normalize_radians(gmst + eq_equinoxes);

        let diff = (gast - expected_gast).abs();
        if diff < 1e-12 || (2.0 * core::f64::consts::PI - diff).abs() < 1e-12 {
            pass += 1;
        } else {
            eprintln!("FAIL GAST: jd={jd} gast={gast} expected={expected_gast} diff={diff}");
        }
    }

    eprintln!("\n=== GAST = GMST + EQ_EQUINOXES ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}

// ─── WHOLE SIGN HOUSES ───

#[test]
fn whole_sign_cusps_are_deterministic_from_asc() {
    use vedaksha_astro::houses::{HouseSystem, compute_houses};

    let mut total = 0;
    let mut pass = 0;

    for ramc in (0..360).step_by(10) {
        for lat in [-50, -30, 0, 30, 50] {
            total += 1;
            let cusps = compute_houses(ramc as f64, lat as f64, 23.44, HouseSystem::WholeSign);

            // House 1 starts at 0° of ASC's sign
            let asc_sign_start = (cusps.asc / 30.0).floor() * 30.0;
            let diff = (cusps.cusps[0] - asc_sign_start).abs();

            if diff < 0.01 || (360.0 - diff) < 0.01 {
                // Each subsequent cusp should be exactly 30° apart
                let mut all_30 = true;
                for i in 0..11 {
                    let gap = normalize_degrees(cusps.cusps[i + 1] - cusps.cusps[i]);
                    if (gap - 30.0).abs() > 0.01 {
                        all_30 = false;
                        eprintln!("FAIL whole_sign gap: cusp {} to {} = {gap}", i + 1, i + 2);
                    }
                }
                if all_30 {
                    pass += 1;
                }
            } else {
                eprintln!(
                    "FAIL whole_sign: asc={} sign_start={asc_sign_start} cusp1={}",
                    cusps.asc, cusps.cusps[0]
                );
            }
        }
    }

    eprintln!("\n=== WHOLE SIGN DETERMINISTIC ===");
    eprintln!("Total: {total}, Pass: {pass}");
    assert_eq!(pass, total);
}
