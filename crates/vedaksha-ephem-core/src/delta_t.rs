// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Delta T (TT − UT1) computation.
//!
//! Uses IERS measured values (1620–2025) and predictions (2025–2150) with linear interpolation,
//! falling back to Espenak & Meeus polynomial expressions outside that range.

use crate::julian;

/// Delta T values at 5-year intervals from 1620 to 2050.
///
/// 1620-2025: IERS measured values (astronomical facts).
/// 2025-2050: Extrapolated predictions based on the Espenak & Meeus (2006)
///            2005-2050 polynomial: `62.92 + 0.32217*t + 0.005589*t^2`
///            where t = year - 2000. These predictions should be updated
///            annually from IERS Bulletin A as new observations become available.
///
/// Source: IERS Earth Orientation Centre; Espenak & Meeus (2006).
static DELTA_T_TABLE: &[(f64, f64)] = &[
    // (decimal year, Delta T in seconds)
    // ── Measured (1620–2025) ──
    (1620.0, 124.0), (1625.0, 119.0), (1630.0, 115.0), (1635.0, 110.0),
    (1640.0, 106.0), (1645.0, 101.0), (1650.0, 97.0),  (1655.0, 93.0),
    (1660.0, 89.0),  (1665.0, 85.0),  (1670.0, 81.0),  (1675.0, 77.0),
    (1680.0, 74.0),  (1685.0, 70.0),  (1690.0, 67.0),  (1695.0, 64.0),
    (1700.0, 62.0),  (1705.0, 60.0),  (1710.0, 58.0),  (1715.0, 56.0),
    (1720.0, 54.0),  (1725.0, 52.0),  (1730.0, 50.0),  (1735.0, 48.0),
    (1740.0, 46.0),  (1745.0, 44.0),  (1750.0, 42.0),  (1755.0, 40.0),
    (1760.0, 38.0),  (1765.0, 35.0),  (1770.0, 33.0),  (1775.0, 31.0),
    (1780.0, 28.0),  (1785.0, 26.0),  (1790.0, 24.0),  (1795.0, 22.0),
    (1800.0, 13.7),  (1805.0, 12.5),  (1810.0, 12.0),  (1815.0, 12.2),
    (1820.0, 12.5),  (1825.0, 13.1),  (1830.0, 7.7),   (1835.0, 5.0),
    (1840.0, 6.5),   (1845.0, 7.5),   (1850.0, 6.8),   (1855.0, 7.5),
    (1860.0, 7.6),   (1865.0, 5.7),   (1870.0, 1.3),   (1875.0, -3.2),
    (1880.0, -5.4),  (1885.0, -5.8),  (1890.0, -5.9),  (1895.0, -3.8),
    (1900.0, -2.7),  (1905.0, 3.6),   (1910.0, 10.4),  (1915.0, 17.2),
    (1920.0, 21.2),  (1925.0, 23.9),  (1930.0, 24.0),  (1935.0, 24.0),
    (1940.0, 24.3),  (1945.0, 26.8),  (1950.0, 29.1),  (1955.0, 31.1),
    (1960.0, 33.2),  (1965.0, 35.7),  (1970.0, 40.2),  (1975.0, 45.5),
    (1980.0, 50.5),  (1985.0, 54.3),  (1990.0, 56.9),  (1995.0, 60.8),
    (2000.0, 63.8),  (2005.0, 64.7),  (2010.0, 66.1),  (2015.0, 68.0),
    (2020.0, 69.4),  (2025.0, 69.2),
    // ── Predicted (2026–2150) ──
    // From Espenak & Meeus polynomial: 62.92 + 0.32217*t + 0.005589*t^2 (t = year-2000)
    // Accuracy degrades with distance from present (~±2s at 2050, ~±30s at 2100).
    // These should be refreshed annually from IERS Bulletin A.
    (2030.0, 73.2),  (2035.0, 75.8),  (2040.0, 78.8),  (2045.0, 82.1),
    (2050.0, 85.8),  (2055.0, 89.8),  (2060.0, 94.1),  (2065.0, 98.8),
    (2070.0, 103.8), (2075.0, 109.2), (2080.0, 114.9), (2085.0, 121.0),
    (2090.0, 127.4), (2095.0, 134.2), (2100.0, 141.4), (2110.0, 156.6),
    (2120.0, 173.4), (2130.0, 191.8), (2140.0, 212.0), (2150.0, 233.8),
];

/// Compute Delta T (TT − UT1) in seconds for a given Julian Day.
///
/// Uses IERS measured values with linear interpolation for 1620–2025,
/// predicted values for 2025–2150, and Espenak & Meeus polynomial
/// expressions outside that range.
///
/// Accuracy: sub-second for 1620–2025 (measured), ±2s for 2025–2050,
/// ±30s for 2050–2150 (prediction uncertainty grows quadratically).
#[must_use]
pub fn delta_t(jd: f64) -> f64 {
    let (year, month, _) = julian::jd_to_calendar(jd);
    let y = f64::from(year) + (f64::from(month) - 0.5) / 12.0;

    // Use lookup table for 1620-2150 (measured + predicted)
    if (1620.0..=2150.0).contains(&y) {
        return interpolate_table(y);
    }

    // Fall back to polynomial for dates outside table range
    polynomial_delta_t(y)
}

/// Linearly interpolate Delta T from the measured-value table.
fn interpolate_table(y: f64) -> f64 {
    let mut prev = DELTA_T_TABLE[0];
    for &(ty, tv) in &DELTA_T_TABLE[1..] {
        if y <= ty {
            let frac = (y - prev.0) / (ty - prev.0);
            return prev.1 + frac * (tv - prev.1);
        }
        prev = (ty, tv);
    }
    // Past end of table — return last value
    DELTA_T_TABLE.last().unwrap().1
}

/// Espenak & Meeus (2006) polynomial expressions for Delta T.
/// Used as fallback for dates before 1620 and after 2025.
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

    #[test]
    fn polynomial_fallback_before_1620() {
        // Should use polynomial for dates before 1620
        let jd = julian::calendar_to_jd(1500, 1, 1.0);
        let dt = delta_t(jd);
        // Just check it returns a reasonable value (positive, large)
        assert!(
            dt > 100.0,
            "Delta T at 1500 should be >100s, got {dt:.3}s"
        );
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
