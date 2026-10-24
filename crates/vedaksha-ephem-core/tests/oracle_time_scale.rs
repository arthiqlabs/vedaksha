// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! The UTC to TT conversion the two Horizons oracle tests apply to the fixture's
//! time tags (`common::horizons_jd_tt`), held to Horizons itself.
//!
//! Horizons' "UT" is UT1 before 1962 and UTC from 1962-01-20; the oracle tests
//! convert the UTC rows to TT with `TT = UTC + 32.184 s + (TAI - UTC)`, the
//! leap-second count from USNO's `tai-utc.dat` (`tests/common/tai_utc.rs`). A
//! wrong table, a wrong boundary or a sign slip here would put the Moon off by
//! seconds of time, 0.55 arcsec per second, and the oracle tests could not tell
//! it from engine error. So the conversion is checked here against numbers that
//! are not derived from the table: Horizons' own `TDB-UT` output.

mod common;

use common::{HORIZONS_UTC_START_JD, horizons_jd_tt, tai_minus_utc};

/// `TT - UTC` in seconds at a UTC Julian Day, through the helper under test.
fn tt_minus_utc(jd: f64) -> f64 {
    (horizons_jd_tt(jd).expect("a UTC-era epoch") - jd) * 86_400.0
}

/// Horizons' `TDB-UT` at oracle epochs, from the API
/// (`COMMAND='301'`, `CENTER='500@399'`, `EPHEM_TYPE='OBSERVER'`,
/// `QUANTITIES='30,31'`, `START_TIME='1900-01-01'`, `STOP_TIME='2100-01-01'`,
/// `STEP_SIZE='30d'`; retrieved 2026-10-24, EOP file `eop.261023.p270119`), as
/// `(JD of the UT tag, TDB-UT in s)`. After 1962 that is `TDB - UTC =
/// (TDB - TAI) + (TAI - UTC)` per the Horizons manual, which is `TT - UTC` up to
/// the periodic TDB - TT term of at most 1.7 ms. Chosen to span the rubber-second
/// era (1962 to 1972, three different rates), a leap second, 2000, 2017, the
/// present and the far future (a constant 37 s, as Horizons does).
const HORIZONS_TDB_MINUS_UT: [(f64, f64); 11] = [
    (2_437_700.5, 34.070_064), // 1962-Feb-05
    (2_438_810.5, 35.788_828), // 1965-Feb-19
    (2_439_140.5, 36.533_789), // 1966-Jan-15
    (2_439_890.5, 38.378_325), // 1968-Feb-04
    (2_441_510.5, 43.183_778), // 1972-Jul-12
    (2_441_690.5, 44.184_138), // 1973-Jan-08
    (2_451_530.5, 64.183_496), // 1999-Dec-18
    (2_457_740.5, 68.183_552), // 2016-Dec-18
    (2_458_010.5, 69.182_433), // 2017-Sep-14
    (2_461_040.5, 69.183_892), // 2025-Dec-31
    (2_488_010.5, 69.182_545), // 2099-Nov-03
];

#[test]
fn tt_minus_utc_agrees_with_horizons_tdb_minus_ut() {
    let mut worst = 0.0_f64;
    for (jd, horizons) in HORIZONS_TDB_MINUS_UT {
        let ours = tt_minus_utc(jd);
        worst = worst.max((ours - horizons).abs());
        // 1.7 ms is the largest TDB - TT; allow 3 ms.
        assert!(
            (ours - horizons).abs() < 0.003,
            "JD {jd}: TT - UTC is {ours} s here, Horizons' TDB-UT is {horizons} s"
        );
    }
    // The comparison could have failed: the two differ by the TDB - TT term, not
    // identically.
    assert!(worst > 1e-4, "suspiciously exact, worst {worst}");
}

/// Hand-worked values of `TAI - UTC`, from the rule and not the table:
/// whole leap seconds since 1972 (IERS Bulletin C: 10 s on 1972-01-01, 32 s from
/// 1999-01-01 to 2005-12-31, 37 s from 2017-01-01), and in the rubber-second era
/// `a + (MJD - MJD0) * rate`: on 1970-01-01 (MJD 40587), with the 1968-02-01 row
/// `4.213170 + (40587 - 39126) * 0.002592 = 4.213170 + 3.786912 = 8.000082` s.
#[test]
fn tai_minus_utc_has_the_published_values() {
    let at = |y: i32, m: u32, d: f64| {
        tai_minus_utc(vedaksha_ephem_core::julian::calendar_to_jd(y, m, d)).unwrap()
    };
    assert!((at(1970, 1, 1.0) - 8.000_082).abs() < 1e-6);
    assert!((at(1972, 1, 1.0) - 10.0).abs() < 1e-9);
    assert!((at(1972, 7, 1.0) - 11.0).abs() < 1e-9);
    assert!((at(2000, 1, 1.0) - 32.0).abs() < 1e-9);
    assert!((at(2016, 12, 31.5) - 36.0).abs() < 1e-9);
    assert!((at(2017, 1, 1.0) - 37.0).abs() < 1e-9);
    // No leap second since: the last row stays in force.
    assert!((at(2030, 6, 1.0) - 37.0).abs() < 1e-9);
    assert!((at(2100, 1, 1.0) - 37.0).abs() < 1e-9);
}

/// Rows before 1962-01-20 are UT1 and stay with the engine's own conversion;
/// rows from it on are UTC. The boundary is inclusive, and the table's rows are
/// strictly increasing and end at 37 s.
#[test]
fn the_epoch_boundary_and_the_table_are_as_documented() {
    assert_eq!(HORIZONS_UTC_START_JD, 2_437_684.5);
    assert!(horizons_jd_tt(2_437_670.5).is_none(), "1962-01-06 is UT1");
    assert!(horizons_jd_tt(2_437_684.4).is_none());
    assert!(horizons_jd_tt(2_437_684.5).is_some(), "1962-01-20 is UTC");
    assert!(horizons_jd_tt(2_437_700.5).is_some());

    let rows = common::tai_utc::TAI_UTC;
    assert!(rows.len() >= 41, "only {} rows", rows.len());
    assert!(rows.windows(2).all(|w| w[1].0 > w[0].0));
    let last = rows[rows.len() - 1];
    assert_eq!((last.1, last.3), (37.0, 0.0));
}
