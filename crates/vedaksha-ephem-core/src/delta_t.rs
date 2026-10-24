// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Delta T (TT − UT1) computation.
//!
//! Delta T is the difference between Terrestrial Time, which runs uniformly, and
//! UT1, which follows the Earth's rotation. It is measured, not computed, and a
//! prediction of it loses accuracy quickly.
//!
//! # Where the values come from, era by era
//!
//! | Era | Source | Between knots |
//! |---|---|---|
//! | before 1620 | Espenak & Meeus (2006) polynomials | evaluated directly |
//! | 1620–1825 | 5-year knots, IERS | linear |
//! | 1830–1900 | Espenak & Meeus (2006) fit to Morrison & Stephenson | evaluated directly |
//! | 1900.0–1973.0 | USNO half-year rows, McCarthy & Babcock (1986) | linear |
//! | 1973-02-01–2026-10-01 | USNO monthly rows, from IERS Earth orientation data | linear |
//! | 2026-10-01–2033.75 | USNO quarterly predictions, `deltat.preds` | linear |
//! | after 2033.75 | Espenak & Meeus (2006) expressions, shifted to meet the last prediction | evaluated directly |
//!
//! The 1900–2026 rows are the generated `delta_t_table.rs`, written by
//! `scripts/generate_delta_t_table.py` from two files retrieved from the U.S.
//! Naval Observatory on 2026-10-24: `historic_deltat.data` and `deltat.data`
//! (URLs, row counts and sha256 sums in the generated file's header and in
//! `DATA_PROVENANCE.md`). Both are public-domain US Government products
//! (Distribution Statement A). Over their overlap, 1973.5 to 1984, they agree to
//! 6 milliseconds.
//!
//! Time is continuous: a Julian Day is turned into a decimal year, so Delta T
//! changes smoothly through a month. This module used to read only the calendar
//! month, so within a month Delta T was a constant and at each month's end it
//! jumped by that month's change, 0.09 s in the 1970s.
//!
//! # Why 1900 onward is not five-year knots
//!
//! The 1900–2025 table was 5-year knots with linear interpolation. Delta T
//! wanders by up to half a second from a chord across five years, and the
//! Moon moves 0.55″ per second of time: against JPL Horizons the Moon's mean
//! longitude error fell from 0.166″ to 0.140″ on switching to the rows above,
//! with nothing else changed. The 1830–1900 era never had knots: five-year knots
//! alias the real minimum near 1895 (−6.2 s), reading −3.6 s there instead (2.5 s
//! error), so it is evaluated directly from the Espenak & Meeus (2006)
//! 1860–1900 polynomial, checked against the USNO historic rows to within 0.35 s
//! (see `delta_t_1890s_minimum_matches_usno`).
//!
//! # What this cannot be
//!
//! Delta T here is TT − **UT1**, and the public contract of this engine is that
//! every Julian Day handed to it is UT1. Civil time is UTC, which differs from
//! UT1 by up to 0.9 s; JPL Horizons treats its UT input as UTC from 1962, and as
//! UT1 before. Feeding UTC to this engine therefore puts the Moon up to 0.5″ off,
//! and no Delta T table can fix that: only the day's UT1 − UTC, which this crate
//! does not carry (see `MAINTENANCE.md` §1).
//!
//! # The prediction
//!
//! From 2026-10 to 2033.75 the knots are USNO's own predictions (`deltat.preds`),
//! quarterly, each with an error the file states: 0.33 s at 2027, 0.77 s at
//! 2030, 1 s from 2032.75. Before this table the 2030 to 2150 knots came from the
//! Espenak & Meeus expressions and ran about 4 s above USNO's prediction by 2033
//! (75.1 s against 71.25 s), which is 2″ for the Moon. Delta T has stopped rising:
//! it peaked at 69.44 s in 2020 and has stood between 69.1 and 69.2 s since 2023.
//!
//! After 2033.75 nothing better exists than extrapolation. This module continues
//! with the Espenak & Meeus expressions' increase since 2033.75 added to USNO's
//! last value, so there is no step where the table ends. The expressions
//! unshifted read 80.2 s there against USNO's 71.25 s, a step of 8.9 s; they date
//! from 2006 and assumed a faster slowing of the Earth than occurred, so their
//! growth after 2033 is probably still high. Treat Delta T beyond 2033.75 as a
//! guess, not a measurement.
//!
//! # Accuracy
//!
//! Sub-0.01 s from 1973 to 2026-10 (the rows are the measurements, interpolated
//! between months). Over 1900–1973 the half-year file states its own error: 0.18 s
//! at worst (1920) and 0.01 s or better from the mid-1940s. That is the
//! uncertainty of one reconstruction; the later one JPL Horizons uses before 1962,
//! Stephenson, Morrison & Hohenkerk (2016, *Proc. R. Soc. A* 472, 20160404), sits
//! 0.8 to 1.0 s above these rows in 1900–1910 and about 0.4 s below them in the
//! 1950s (measured 2026-10-24 from Horizons' `TDB-UT` output on the oracle's
//! 30-day grid). Both are published; which is closer to the truth is not settled
//! here. From 2026-10 to 2033.75, the error USNO states for its predictions.

use crate::julian;

#[path = "delta_t_table.rs"]
mod table;

/// Delta T knots before 1900: 5-year intervals from 1620 to 1825.
///
/// 1620–1825: IERS measured values (astronomical facts), at 5-year knots. The
/// next knot is the first row of [`table::MODERN`]; the 1830–1900 era carries no
/// knots at all, [`delta_t`] evaluates it directly from the Espenak & Meeus (2006)
/// polynomial because 5-year knots alias the minimum near 1895.
///
/// Known and not touched here: between the last knot here (1825, 13.1 s) and the
/// polynomial's start at 1830 the lookup runs a straight line to the 1900.0 knot
/// (12.0 s at 1830), where the polynomial reads 7.7 s, a 4.4 s step.
///
/// Source: IERS Earth Orientation Centre; Espenak & Meeus (2006).
static EARLY_KNOTS: &[(f64, f64)] = &[
    // (decimal year, Delta T in seconds)
    (1620.0, 124.0),
    (1625.0, 119.0),
    (1630.0, 115.0),
    (1635.0, 110.0),
    (1640.0, 106.0),
    (1645.0, 101.0),
    (1650.0, 97.0),
    (1655.0, 93.0),
    (1660.0, 89.0),
    (1665.0, 85.0),
    (1670.0, 81.0),
    (1675.0, 77.0),
    (1680.0, 74.0),
    (1685.0, 70.0),
    (1690.0, 67.0),
    (1695.0, 64.0),
    (1700.0, 62.0),
    (1705.0, 60.0),
    (1710.0, 58.0),
    (1715.0, 56.0),
    (1720.0, 54.0),
    (1725.0, 52.0),
    (1730.0, 50.0),
    (1735.0, 48.0),
    (1740.0, 46.0),
    (1745.0, 44.0),
    (1750.0, 42.0),
    (1755.0, 40.0),
    (1760.0, 38.0),
    (1765.0, 35.0),
    (1770.0, 33.0),
    (1775.0, 31.0),
    (1780.0, 28.0),
    (1785.0, 26.0),
    (1790.0, 24.0),
    (1795.0, 22.0),
    (1800.0, 13.7),
    (1805.0, 12.5),
    (1810.0, 12.0),
    (1815.0, 12.2),
    (1820.0, 12.5),
    (1825.0, 13.1),
];

/// Number of knots in the whole table: early, modern (measured), predicted.
fn knot_count() -> usize {
    EARLY_KNOTS.len() + table::MODERN.len() + table::PREDICTED.len()
}

/// The `i`th knot of the whole table, counting [`EARLY_KNOTS`], then
/// [`table::MODERN`], then [`table::PREDICTED`].
fn knot(i: usize) -> (f64, f64) {
    let early = EARLY_KNOTS.len();
    let modern = table::MODERN.len();
    if i < early {
        EARLY_KNOTS[i]
    } else if i < early + modern {
        table::MODERN[i - early]
    } else {
        table::PREDICTED[i - early - modern]
    }
}

/// The last knot: the end of the USNO prediction, 2033.75.
fn last_knot() -> (f64, f64) {
    knot(knot_count() - 1)
}

/// Decimal year of a Julian Day: `year + (JD − JD(Jan 1)) / length of year`,
/// continuous through the year (the calendar's month and day enter only through
/// the Julian Day of 1 January).
fn decimal_year(jd: f64) -> f64 {
    let (year, _, _) = julian::jd_to_calendar(jd);
    let start = julian::calendar_to_jd(year, 1, 1.0);
    let end = julian::calendar_to_jd(year + 1, 1, 1.0);
    f64::from(year) + (jd - start) / (end - start)
}

/// Compute Delta T (TT − UT1) in seconds for a given Julian Day.
///
/// Reads the measured rows by linear interpolation from 1900 to 2026-10, the
/// 5-year knots for 1620–1825 and USNO's predicted rows to 2033.75, and evaluates
/// the Espenak & Meeus polynomials for 1830–1900, before 1620, and (shifted to
/// meet the last prediction) after 2033.75. See the module notes for the sources,
/// the accuracy of each era and the limits of a UT1 argument.
///
/// Accuracy: below 0.01 s from 1973 to 2026-10, sub-second for 1620–1973, the
/// error USNO states for its predictions (0.3 s in 2027, 0.8 s in 2030, 1 s by
/// 2033), and after that a guess that grows quickly.
#[must_use]
pub fn delta_t(jd: f64) -> f64 {
    let y = decimal_year(jd);

    // 1830–1900 first: no table knots there (see the module doc). Five-year
    // knots alias the 1890s minimum, so this era resolves through the
    // Espenak & Meeus fit to the Morrison & Stephenson series instead.
    if (1830.0..1900.0).contains(&y) {
        return polynomial_delta_t(y);
    }

    // The lookup table, 1620 to the end of the USNO prediction (measured, then
    // predicted).
    let (y_last, v_last) = last_knot();
    if (1620.0..=y_last).contains(&y) {
        return interpolate_table(y);
    }

    // After the last prediction, continue with the Espenak & Meeus expressions'
    // increase since then, so that Delta T has no step where the table ends. The
    // expressions themselves, unshifted, read 8.9 s above USNO's last prediction
    // at 2033.75, because they were written in 2006 for a faster slowing of the
    // Earth than occurred.
    if y > y_last {
        return v_last + (polynomial_delta_t(y) - polynomial_delta_t(y_last));
    }

    // Before 1620 the polynomial stands alone.
    polynomial_delta_t(y)
}

/// Linearly interpolate Delta T from the knots, by binary search for the
/// bracketing pair. `y` must lie in `[1620, last knot]`, the table's span.
fn interpolate_table(y: f64) -> f64 {
    // Smallest index whose knot is at or after `y`.
    let mut lo = 0;
    let mut hi = knot_count() - 1;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if knot(mid).0 < y {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo == 0 {
        return knot(0).1;
    }
    let (y0, v0) = knot(lo - 1);
    let (y1, v1) = knot(lo);
    v0 + (y - y0) / (y1 - y0) * (v1 - v0)
}

/// Espenak & Meeus (2006) polynomial expressions for Delta T.
/// Used before 1620, for 1830–1900, and (shifted) after the last prediction.
fn polynomial_delta_t(y: f64) -> f64 {
    if y < -500.0 {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    } else if y < 500.0 {
        let u = y / 100.0;
        10_583.6
            + u * (-1_014.41
                + u * (33.783_11
                    + u * (-5.952_053
                        + u * (-0.179_845_2 + u * (0.022_174_192 + u * 0.009_031_652_1)))))
    } else if y < 1600.0 {
        let u = (y - 1000.0) / 100.0;
        1_574.2
            + u * (-556.01
                + u * (71.234_72
                    + u * (0.319_781
                        + u * (-0.850_346_3 + u * (-0.005_050_998 + u * 0.008_357_207_3)))))
    } else if y < 1700.0 {
        let t = y - 1600.0;
        120.0 + t * (-0.9808 + t * (-0.01532 + t * (1.0 / 7129.0)))
    } else if y < 1800.0 {
        let t = y - 1700.0;
        8.83 + t * (0.1603 + t * (-0.005_928_5 + t * (0.000_133_36 + t * (-1.0 / 1_174_000.0))))
    } else if y < 1860.0 {
        let t = y - 1800.0;
        13.72
            + t * (-0.332_447
                + t * (0.006_861_2
                    + t * (0.004_111_6
                        + t * (-0.000_374_36
                            + t * (0.000_012_127_2
                                + t * (-0.000_000_169_9 + t * 0.000_000_000_875))))))
    } else if y < 1900.0 {
        let t = y - 1860.0;
        7.62 + t
            * (0.5737
                + t * (-0.251_754
                    + t * (0.016_806_68 + t * (-0.000_447_362_4 + t * (1.0 / 233_174.0)))))
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + t * (1.494_119 + t * (-0.059_893_9 + t * (0.006_196_6 + t * (-0.000_197))))
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + t * (0.844_93 + t * (-0.076_100 + t * 0.002_093_6))
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + t * (0.407 + t * (-1.0 / 233.0 + t * (1.0 / 2547.0)))
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + t * (1.067 + t * (-1.0 / 260.0 + t * (-1.0 / 718.0)))
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86
            + t * (0.3345
                + t * (-0.060_374 + t * (0.001_727_5 + t * (0.000_651_814 + t * 0.000_023_735_99))))
    } else if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + t * (0.322_17 + t * 0.005_589)
    } else if y < 2150.0 {
        -20.0 + 32.0 * ((y - 1820.0) / 100.0).powi(2) - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Convert Julian Day in UT1 to TT (Terrestrial Time).
#[must_use]
pub fn ut1_to_tt(jd_ut1: f64) -> f64 {
    jd_ut1 + delta_t(jd_ut1) / 86400.0
}

/// Convert Julian Day in TT to UT1.
#[must_use]
pub fn tt_to_ut1(jd_tt: f64) -> f64 {
    jd_tt - delta_t(jd_tt) / 86400.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian;

    /// J2000.0 = 2000-Jan-1.5 = JD 2451545.0
    const J2000: f64 = 2_451_545.0;

    fn jd_for_year(year: i32) -> f64 {
        julian::calendar_to_jd(year, 1, 1.0)
    }

    #[test]
    fn delta_t_at_j2000_approx_63_8() {
        let dt = delta_t(J2000);
        assert!(
            (dt - 63.8).abs() < 1.0,
            "Delta T at J2000 expected ~63.8s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_at_1900_approx_neg_2_8() {
        let jd = jd_for_year(1900);
        let dt = delta_t(jd);
        assert!(
            (dt - (-2.8)).abs() < 1.0,
            "Delta T at 1900 expected ~-2.8s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_at_1950_approx_29() {
        let jd = jd_for_year(1950);
        let dt = delta_t(jd);
        assert!(
            (dt - 29.0).abs() < 1.0,
            "Delta T at 1950 expected ~29s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_at_2020_approx_69_4() {
        let jd = jd_for_year(2020);
        let dt = delta_t(jd);
        assert!(
            (dt - 69.4).abs() < 5.0,
            "Delta T at 2020 expected ~69.4s, got {dt:.3}s"
        );
    }

    #[test]
    fn ut1_to_tt_increases_jd() {
        // Delta T is positive in the modern era, so TT > UT1
        let jd_ut1 = J2000;
        let jd_tt = ut1_to_tt(jd_ut1);
        assert!(
            jd_tt > jd_ut1,
            "ut1_to_tt should increase JD: tt={jd_tt}, ut1={jd_ut1}"
        );
    }

    #[test]
    fn tt_to_ut1_roundtrip() {
        let jd_ut1 = J2000;
        let jd_tt = ut1_to_tt(jd_ut1);
        let jd_back = tt_to_ut1(jd_tt);
        // Roundtrip should be accurate to within a microsecond in JD
        assert!(
            (jd_back - jd_ut1).abs() < 1e-9,
            "Roundtrip failed: started={jd_ut1}, got={jd_back}, diff={}",
            (jd_back - jd_ut1).abs()
        );
    }

    // --- New tests for table-based Delta T ---

    #[test]
    fn delta_t_table_at_2000() {
        // Table value at 2000.0 is exactly 63.8s
        let jd = julian::calendar_to_jd(2000, 1, 1.0);
        let dt = delta_t(jd);
        assert!(
            (dt - 63.8).abs() < 0.5,
            "Delta T at 2000.0 expected ~63.8s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_table_at_2020() {
        // Table value at 2020.0 is exactly 69.4s
        let jd = julian::calendar_to_jd(2020, 1, 1.0);
        let dt = delta_t(jd);
        assert!(
            (dt - 69.4).abs() < 0.5,
            "Delta T at 2020.0 expected ~69.4s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_table_at_1900() {
        // Table value at 1900.0 is -2.7s
        let jd = julian::calendar_to_jd(1900, 1, 1.0);
        let dt = delta_t(jd);
        assert!(
            (dt - (-2.7)).abs() < 1.0,
            "Delta T at 1900.0 expected ~-2.7s, got {dt:.3}s"
        );
    }

    #[test]
    fn delta_t_interpolation_between_points() {
        // 2002.5 is halfway between 2000 (63.8) and 2005 (64.7)
        // Expected: 63.8 + 0.5 * (64.7 - 63.8) = 64.25
        let jd = julian::calendar_to_jd(2002, 7, 1.0);
        let dt = delta_t(jd);
        assert!(
            (dt - 64.25).abs() < 0.5,
            "Delta T at ~2002.5 expected ~64.25s, got {dt:.3}s"
        );
    }

    /// The 1890s minimum: five-year knots aliased it (−3.6 s at 1895
    /// against the observed −6.2 s). USNO historic half-year rows (the
    /// Morrison & Stephenson series), sampled 9 November each year.
    #[test]
    fn delta_t_1890s_minimum_matches_usno() {
        // (year, USNO ΔT in seconds, 9 November)
        const ROWS: [(i32, f64); 6] = [
            (1889, -5.840),
            (1891, -6.238),
            (1893, -6.497),
            (1895, -6.152),
            (1897, -4.806),
            (1899, -2.843),
        ];
        for (year, expected) in ROWS {
            let jd = julian::calendar_to_jd(year, 11, 9.0);
            let dt = delta_t(jd);
            assert!(
                (dt - expected).abs() < 0.35,
                "ΔT at 9 Nov {year}: expected {expected:.3}s (USNO), got {dt:.3}s"
            );
        }
    }

    // --- The 1900-2026 table: rows from the USNO files, read continuously ---

    /// Rows of USNO's `deltat.data` and `historic_deltat.data`, copied from the
    /// retrieved files (not from `delta_t_table.rs`), at dates the interpolation
    /// must hit exactly: the knot's own instant.
    ///
    /// `(year, month, ΔT s)` for the first-of-month rows of `deltat.data`:
    /// ```text
    ///  1973  2  1  43.4724     2000  1  1  63.8285     2025 12  1  69.1042
    ///  1973  3  1  43.5648     2000  2  1  63.8557     2026 10  1  69.2065
    ///  1985  7  1  54.6355     2012  7  1  66.7708
    /// ```
    /// and `(year, ΔT s)` for `historic_deltat.data` rows at 0h on 1 January:
    /// ```text
    ///  1900.000  -2.70    1950.000  29.15    1955.000  31.07    1962.000  33.992
    /// ```
    #[test]
    fn rows_of_the_usno_files_are_hit_exactly_at_their_own_instants() {
        const MONTHLY: [(i32, u32, f64); 8] = [
            (1973, 2, 43.4724),
            (1973, 3, 43.5648),
            (1985, 7, 54.6355),
            (2000, 1, 63.8285),
            (2000, 2, 63.8557),
            (2012, 7, 66.7708),
            (2025, 12, 69.1042),
            (2026, 10, 69.2065),
        ];
        for (year, month, expected) in MONTHLY {
            let dt = delta_t(julian::calendar_to_jd(year, month, 1.0));
            assert!(
                (dt - expected).abs() < 1e-6,
                "ΔT at {year}-{month:02}-01: expected {expected}, got {dt}"
            );
        }
        const HALF_YEAR: [(i32, f64); 5] = [
            (1900, -2.70),
            (1950, 29.15),
            (1955, 31.07),
            (1962, 33.992),
            (1973, 43.373),
        ];
        for (year, expected) in HALF_YEAR {
            let dt = delta_t(julian::calendar_to_jd(year, 1, 1.0));
            assert!(
                (dt - expected).abs() < 1e-6,
                "ΔT at {year}-01-01: expected {expected}, got {dt}"
            );
        }
    }

    /// Between two monthly rows Delta T is the straight line between them:
    /// 1973-02-15 is 14 of the 28 days of February, the midpoint of the 43.4724
    /// and 43.5648 rows, and 2000-01-16 is 15 of 31 days toward 63.8557.
    #[test]
    fn delta_t_is_linear_between_monthly_rows() {
        let mid_feb = delta_t(julian::calendar_to_jd(1973, 2, 15.0));
        assert!((mid_feb - 43.5186).abs() < 1e-6, "got {mid_feb}");
        let jan_2000 = delta_t(julian::calendar_to_jd(2000, 1, 16.0));
        let expected = 63.8285 + (15.0 / 31.0) * (63.8557 - 63.8285);
        assert!((jan_2000 - expected).abs() < 1e-6, "got {jan_2000}");
    }

    /// Delta T does not step at a month boundary. It used to read the calendar
    /// month alone, so it held one value all month and jumped by that month's
    /// change (0.09 s in the 1970s) at the turn. Straddle every boundary from
    /// 1960 to 2025 by one second and require the difference to be the true
    /// rate (about 3e-8 s) rather than a step.
    #[test]
    fn delta_t_does_not_step_at_month_boundaries() {
        let mut worst = 0.0_f64;
        let mut n = 0;
        for year in 1960..=2025 {
            for month in 1..=12 {
                let boundary = julian::calendar_to_jd(year, month, 1.0);
                let before = delta_t(boundary - 0.5 / 86_400.0);
                let after = delta_t(boundary + 0.5 / 86_400.0);
                worst = worst.max((after - before).abs());
                n += 1;
            }
        }
        assert_eq!(n, 66 * 12);
        assert!(
            worst < 1e-6,
            "ΔT steps by {worst} s across a month boundary"
        );
    }

    /// The knots are strictly increasing in time and Delta T moves by a
    /// believable amount between neighbours: at most 0.15 s a month (largest in
    /// the data 0.109) and between -0.2 and 1.0 s a half year (data -0.12 to
    /// 0.83). A transposed digit or a row out of order breaks one of these.
    #[test]
    fn the_modern_table_is_ordered_and_smooth() {
        let rows = table::MODERN;
        // 147 half-year rows (1900.0 to 1973.0) plus the monthly rows, 645 when
        // the table was made (1973-02 to 2026-10); a refresh adds more.
        assert!(rows.len() >= 147 + 645, "only {} knots", rows.len());
        assert_eq!(rows[0].0, 1900.0);
        for pair in rows.windows(2) {
            let ((y0, v0), (y1, v1)) = (pair[0], pair[1]);
            assert!(y1 > y0, "knots out of order at {y0} -> {y1}");
            let step = y1 - y0;
            let dv = v1 - v0;
            if step > 0.4 {
                assert!((-0.2..=1.0).contains(&dv), "half-year step {y0}: {dv} s");
            } else {
                assert!(dv.abs() < 0.15, "monthly step at {y0}: {dv} s");
            }
        }
    }

    /// An independent check on all 635 monthly rows through 2025: since 1972 UTC
    /// is kept within 0.9 s of UT1 by leap seconds, so Delta T cannot differ from
    /// `32.184 + (TAI - UTC)` by more than that. `TAI - UTC` is the leap-second
    /// count from USNO's `tai-utc.dat` (retrieved 2026-10-24), a different file
    /// from the one the table is made of. A row wrong by a second or more, or a
    /// column or a month misplaced, breaks the bound; this is the same physical
    /// quantity the oracle meets as the UT1-UTC floor, see the module notes.
    #[test]
    fn monthly_rows_stay_within_the_leap_second_bound_of_utc() {
        // (year, month, TAI - UTC in seconds from that date), from 1972.
        const LEAP: [(i32, u32, f64); 28] = [
            (1972, 1, 10.0),
            (1972, 7, 11.0),
            (1973, 1, 12.0),
            (1974, 1, 13.0),
            (1975, 1, 14.0),
            (1976, 1, 15.0),
            (1977, 1, 16.0),
            (1978, 1, 17.0),
            (1979, 1, 18.0),
            (1980, 1, 19.0),
            (1981, 7, 20.0),
            (1982, 7, 21.0),
            (1983, 7, 22.0),
            (1985, 7, 23.0),
            (1988, 1, 24.0),
            (1990, 1, 25.0),
            (1991, 1, 26.0),
            (1992, 7, 27.0),
            (1993, 7, 28.0),
            (1994, 7, 29.0),
            (1996, 1, 30.0),
            (1997, 7, 31.0),
            (1999, 1, 32.0),
            (2006, 1, 33.0),
            (2009, 1, 34.0),
            (2012, 7, 35.0),
            (2015, 7, 36.0),
            (2017, 1, 37.0),
        ];
        let tai_minus_utc = |jd: f64| {
            LEAP.iter()
                .rev()
                .find(|&&(y, m, _)| julian::calendar_to_jd(y, m, 1.0) <= jd)
                .map(|&(_, _, v)| v)
                .unwrap()
        };
        let mut worst = 0.0_f64;
        let mut n = 0;
        let (first, last) = (1973 * 12 + 1, 2025 * 12 + 11); // Feb 1973 .. Dec 2025
        for k in first..=last {
            let (year, month) = (k / 12, (k % 12) as u32 + 1);
            let jd = julian::calendar_to_jd(year, month, 1.0);
            let ut1_minus_utc = 32.184 + tai_minus_utc(jd) - delta_t(jd);
            worst = worst.max(ut1_minus_utc.abs());
            assert!(
                ut1_minus_utc.abs() < 0.9,
                "{year}-{month:02}: UT1-UTC would be {ut1_minus_utc:.3} s, beyond the 0.9 s bound"
            );
            n += 1;
        }
        assert_eq!(n, 635);
        // And the bound is not vacuous: the sawtooth really does swing most of
        // the way to its limit.
        assert!(worst > 0.7, "largest |UT1-UTC| over 1973-2025 only {worst}");
    }

    // --- The USNO predictions, 2026-10 to 2033.75, and the tail after them ---

    /// Julian Day of a Modified Julian Day.
    fn jd_of_mjd(mjd: f64) -> f64 {
        mjd + 2_400_000.5
    }

    /// Rows of USNO's `deltat.preds`, copied from the retrieved file as `(MJD,
    /// TT-UT in s)`:
    /// ```text
    /// 61406.000  2027.00  69.14      62502.000  2030.00  69.97
    /// 62867.000  2031.00  70.32      63871.000  2033.75  71.25
    /// ```
    #[test]
    fn rows_of_deltat_preds_are_hit_exactly_at_their_own_instants() {
        for (mjd, expected) in [
            (61_406.0, 69.14),
            (62_502.0, 69.97),
            (62_867.0, 70.32),
            (63_871.0, 71.25),
        ] {
            let dt = delta_t(jd_of_mjd(mjd));
            assert!(
                (dt - expected).abs() < 1e-6,
                "ΔT at MJD {mjd}: expected {expected}, got {dt}"
            );
        }
    }

    /// The predicted knots follow the last measured one in time, are quarterly,
    /// and rise slowly: at most 0.2 s a quarter (the data's largest is 0.12), and
    /// never fall. A misplaced digit or an overlap with the monthly rows breaks one.
    #[test]
    fn the_predicted_knots_are_ordered_and_slow() {
        let (measured, predicted) = (table::MODERN, table::PREDICTED);
        assert!(
            predicted.len() >= 28,
            "only {} predicted knots",
            predicted.len()
        );
        let last_measured = measured[measured.len() - 1];
        assert!(predicted[0].0 > last_measured.0);
        // The joint: from the last measurement to the first prediction.
        let joint = predicted[0].1 - last_measured.1;
        assert!(joint.abs() < 0.3, "joint steps {joint} s");
        for pair in predicted.windows(2) {
            let ((y0, v0), (y1, v1)) = (pair[0], pair[1]);
            assert!(y1 > y0, "predicted knots out of order at {y0}");
            assert!((y1 - y0) < 0.3, "gap of {} years at {y0}", y1 - y0);
            assert!(
                (0.0..=0.2).contains(&(v1 - v0)),
                "step {} s at {y0}",
                v1 - v0
            );
        }
    }

    /// Delta T is continuous where the table ends and the shifted expressions
    /// take over, at 2033-10-01 (MJD 63871), and keeps rising at about the
    /// expressions' rate after it: 0.32217 + 2 × 0.005589 × 34 = 0.70 s a year at
    /// 2034. A one-second window straddles the joint, which would show an 8.9 s
    /// step if the expressions were not shifted; and a UT1 → TT → UT1 round trip
    /// stays exact across it, which a step would break.
    #[test]
    fn delta_t_has_no_step_where_the_predictions_end() {
        let joint = jd_of_mjd(63_871.0);
        let before = delta_t(joint - 0.5 / 86_400.0);
        let after = delta_t(joint + 0.5 / 86_400.0);
        assert!(
            (after - before).abs() < 1e-6,
            "step of {} s",
            after - before
        );

        let year_on = delta_t(joint + 365.25) - delta_t(joint);
        assert!(
            (0.55..0.85).contains(&year_on),
            "ΔT rises {year_on} s in the year after the joint"
        );

        for k in -20..=20 {
            let jd = joint + f64::from(k) * 3.0;
            let back = tt_to_ut1(ut1_to_tt(jd));
            assert!((back - jd).abs() < 1e-9, "round trip off by {}", back - jd);
        }
    }

    /// Far past the table, Delta T is USNO's last value plus the increase of the
    /// Espenak & Meeus (2006) expressions since then. For 2100-01-01 the 2050 to
    /// 2150 expression `-20 + 32·u² - 0.5628·(2150 - y)`, `u = (y - 1820)/100`,
    /// is `-20 + 32·7.84 - 28.14 = 202.74` s; the 2005 to 2050 expression
    /// `62.92 + 0.32217·t + 0.005589·t²` at 2033.747945 (`t = 33.747945`) is
    /// `62.92 + 10.8722 + 6.3649 = 80.1571` s. Both are worked by hand here from
    /// the published coefficients, apart from the code under test.
    #[test]
    fn delta_t_beyond_the_table_continues_by_the_expressions_increase() {
        let t = 33.747_945_f64;
        let at_end = 62.92 + 0.322_17 * t + 0.005_589 * t * t;
        let at_2100 = -20.0 + 32.0 * 2.8 * 2.8 - 0.5628 * 50.0;
        let expected = 71.25 + (at_2100 - at_end);
        let dt = delta_t(julian::calendar_to_jd(2100, 1, 1.0));
        assert!(
            (dt - expected).abs() < 0.01,
            "ΔT(2100) {dt}, expected {expected}"
        );
        assert!((expected - 193.8).abs() < 0.1, "hand figure {expected}");
    }

    #[test]
    fn polynomial_fallback_before_1620() {
        // Should use polynomial for dates before 1620
        let jd = julian::calendar_to_jd(1500, 1, 1.0);
        let dt = delta_t(jd);
        // Just check it returns a reasonable value (positive, large)
        assert!(dt > 100.0, "Delta T at 1500 should be >100s, got {dt:.3}s");
    }

    #[test]
    fn polynomial_fallback_after_2025() {
        // Should use polynomial for dates after 2025
        let jd = julian::calendar_to_jd(2030, 1, 1.0);
        let dt = delta_t(jd);
        // Should be in a reasonable range near recent values
        assert!(
            dt > 60.0 && dt < 100.0,
            "Delta T at 2030 expected 60-100s, got {dt:.3}s"
        );
    }
}
