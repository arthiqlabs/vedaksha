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
//! | before −720 | Espenak & Meeus (2006) long-term parabola | evaluated directly |
//! | −720 to 1962.0 | Stephenson, Morrison & Hohenkerk (2016) cubic spline, Table S15 | cubic, per interval |
//! | 1962.0–1973.0 | USNO half-year rows, McCarthy & Babcock (1986) | linear |
//! | 1973-02-01–2026-10-01 | USNO monthly rows, from IERS Earth orientation data | linear |
//! | 2026-10-01–2033.75 | USNO quarterly predictions, `deltat.preds` | linear |
//! | after 2033.75 | Espenak & Meeus (2006) expressions, shifted to meet the last prediction | evaluated directly |
//!
//! The spline coefficients and the USNO rows are in the generated
//! `delta_t_table.rs`, written by `scripts/generate_delta_t_table.py`; its header
//! has every source, retrieval date, row count, licence and sha256 sum, and
//! `DATA_PROVENANCE.md` has the same.
//!
//! Time is continuous: a Julian Day is turned into a decimal year, so Delta T
//! changes smoothly through a month. This module used to read only the calendar
//! month, so within a month Delta T was a constant and at each month's end it
//! jumped by that month's change, 0.09 s in the 1970s.
//!
//! # The spline, −720 to 1962
//!
//! Stephenson, Morrison and Hohenkerk, "Measurement of the Earth's rotation: 720
//! BC to AD 2015", *Proc. R. Soc. A* 472, 20160404 (2016),
//! doi:10.1098/rspa.2016.0404 (SMH2016), fitted a cubic spline to Delta T from
//! ancient and medieval eclipse records (720 BC to AD 1600) and lunar-occultation
//! timings from AD 1623, the latter reduced by Herald and Gault against the JPL
//! lunar ephemerides (DE422 and DE423 in the archive's ReadMe, DE430 in the paper).
//! Table S15 of the electronic supplement (Creative Commons Attribution 4.0) is the
//! spline: 54 intervals from −720.0 to 2016.0, each with `a_0..a_3`. It is
//! evaluated as the table's notes say: for the year and fraction `Y`, take the row
//! with `K_i <= Y <= K_{i+1}`, form `t = (Y − K_i)/(K_{i+1} − K_i)`, and
//! `DT = a_0 + a_1·t + a_2·t² + a_3·t³` seconds. Years are astronomical, as the
//! paper says: "−708 is equivalent to 709 BC".
//!
//! That rule is checked three ways in the tests, none of them a number from this
//! engine. (1) The spline is continuous in value, slope and curvature at every one
//! of the 35 interior knots to 0.001 s, 0.0007 s/yr and 0.0004 s/yr² (the paper
//! chose splines for the continuity of their first derivative, the day length);
//! with `t` taken in years it fails by thousands. (2) The paper says the spline
//! "satisfies all the constraints of the total and annular solar eclipses [Table
//! S10], except that of −600": of the 34 limits printed there, 33 hold (three of
//! them to within 0.8 percent of the limit, which the paper says constrain the
//! spline) and −600 is 2,700 s outside, as the paper says. (3) The paper puts Delta T "close to zero"
//! at 1900; the spline reads −1.977 s. And Horizons' own `TDB-UT` before 1962
//! (which its manual says is this series) agrees with this evaluation to 0.033 s
//! at the 755 grid epochs before 1962, rms 0.013 s.
//!
//! The supplement's data file defines its `Y` as `2000 + (JD − 2451546.5)/365.25`;
//! here `Y` is the calendar decimal year (`year + (JD − JD of 1 January)/length of
//! year`) that the other sources use. The two differ by up to 0.007 year, which is
//! 0.008 s of Delta T in 1900–1962, 0.004 s in 1700–1900, 0.16 s in 1000–1700, 0.33
//! s in 0–1000 and 0.6 s before 0, all far inside the published uncertainty.
//!
//! # Why the handover is at 1962.0
//!
//! From 1962.0 the USNO rows, not the spline. The sources say so themselves. SMH2016
//! (section 2(a)(vii), "Atomic time data") states that "the values of TAI − UT1 are
//! tabulated by the IERS at a daily interval from 1962 to the present", and (section
//! 7, Conclusion) that after AD 1600 occultation timings "provide the most accurate
//! measure of the Earth's rotation on time scales longer than about a year, until
//! the introduction of the atomic time scale from 1962 onwards". The authors used
//! the atomic data over 1962–2015 only "as a comparative control on the smoothing
//! of the occultation dataset", which is their business and not a statement that
//! the spline beats a direct measurement. The USNO file states an error of 0.001 s
//! from 1962.0 (0.003 s from 1956, 0.01 s in 1950, 0.18 s at worst before); the
//! spline smooths occultation timings through 3-year knots and tabulates no error.
//! The paper's own text also notes the spline's Delta T is tied to the lunar
//! ephemeris used in the reductions ("the precise value of ΔT is dependent on the
//! lunar ephemeris used in the reduction of the observations"), a JPL one, the same
//! series as the DE440s the SPK path reads, which is a reason to prefer it to the
//! older McCarthy and Babcock rows before 1962.
//!
//! # The joins, and the steps that remain
//!
//! Every join is continuous except two, and those two are stated, not smoothed:
//! there is no published rule for blending these sources, and none is invented.
//!
//! * **−720:** the long-term parabola reads 20,625.1 s and the spline starts at
//!   20,550.6 s, a step of −74.5 s (0.36 percent of a 20,000 s Delta T that the
//!   eclipse limits, hundreds to thousands of seconds wide, cannot resolve).
//! * **1962.0:** the spline reads 33.621 s and the USNO row 33.992 s, a step of
//!   +0.371 s (0.20 arcsecond of Moon longitude). It is the disagreement of
//!   occultation timings smoothed through 3-year knots with the first atomic-clock
//!   measurement. Horizons has the same step, at the same epoch, between its
//!   pre-1962 series and its EOP file.
//!
//! The spline's 35 knots are continuous to its published rounding (0.001 s). The
//! USNO rows are linear between knots, so continuous; the monthly onto the
//! half-year, the predictions onto the measurements, and the tail onto the
//! predictions (below) are continuous by construction. The tests
//! `delta_t_is_continuous_at_every_join` and `delta_t_has_no_unstated_step`
//! hold all of this.
//!
//! # What this replaced
//!
//! Before the spline, Delta T below 1900 was the Espenak & Meeus (2006)
//! polynomials (<https://eclipse.gsfc.nasa.gov/SEhelp/deltatpoly2004.html>), which
//! fit the older Morrison & Stephenson (2004) values and before them 5-year knots
//! for 1620–1825 that were not data (a near-straight fill from 124 s at 1620 to 22
//! s at 1795, 62 s at 1700 against about 9–15 s). 1900–1962 was McCarthy and
//! Babcock's half-year rows, which the spline reads 1.0 s above in 1905, 0.7 s
//! above in 1900 and 1910, 0.2 s below in 1950 and 0.7 s below in 1955. At
//! 1700, 1650, 1620 and 1830 the spline reads 14.6, 43.7, 67.1 and 10.8 s;
//! the polynomials read 8.8, 50.2, 95.4 and 7.7 s.
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
//! 2030, 1 s from 2032.75. Delta T has stopped rising: it peaked at 69.44 s in
//! 2020 and has stood between 69.1 and 69.2 s since 2023.
//!
//! After 2033.75 nothing better exists than extrapolation. This module continues
//! with the Espenak & Meeus (2006) expressions' increase since 2033.75 added to
//! USNO's last value, so there is no step where the table ends. The expressions
//! unshifted read 80.2 s there against USNO's 71.25 s, a step of 8.9 s; they date
//! from 2006 and assumed a faster slowing of the Earth than occurred, so their
//! growth after 2033 is probably still high. Treat Delta T beyond 2033.75 as a
//! guess, not a measurement.
//!
//! # Accuracy
//!
//! Sub-0.01 s from 1973 to 2026-10 (the rows are the measurements, interpolated
//! between months), and 0.001 s stated for the USNO rows over 1962–1973. Before 1962
//! the spline: SMH2016 tabulates no uncertainty with it. Its fit to occultation
//! timings "given nominally to a precision of 1 s or better" is, in the authors'
//! words, "certainly poorer than this in the seventeenth century", and before 1600
//! the eclipse limits it is held to are hundreds to thousands of seconds wide.
//! From 2026-10 to 2033.75, the error USNO states for its predictions.

use crate::julian;

#[path = "delta_t_table.rs"]
mod table;

/// Number of knots in the USNO table: measured (from 1962.0), then predicted.
fn knot_count() -> usize {
    table::MODERN.len() + table::PREDICTED.len()
}

/// The `i`th knot of the USNO table, counting [`table::MODERN`], then
/// [`table::PREDICTED`].
fn knot(i: usize) -> (f64, f64) {
    let modern = table::MODERN.len();
    if i < modern {
        table::MODERN[i]
    } else {
        table::PREDICTED[i - modern]
    }
}

/// The first USNO knot, 1962.0: where the spline hands over to the measurements.
fn first_knot() -> (f64, f64) {
    knot(0)
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
/// The SMH2016 spline from −720 to 1962.0, the USNO measurements by linear
/// interpolation from 1962.0 to 2026-10, USNO's predictions to 2033.75, and the
/// Espenak & Meeus expressions outside those: their long-term parabola before
/// −720, and their increase since 2033.75 added to the last prediction after it.
/// See the module notes for the sources, the accuracy of each era, the joins (two
/// of them have a stated step) and the limits of a UT1 argument.
///
/// Accuracy: below 0.01 s from 1973 to 2026-10, 0.001 s stated by USNO over
/// 1962–1973, the spline's fit before 1962, the error USNO states for its
/// predictions (0.3 s in 2027, 0.8 s in 2030, 1 s by 2033), and after that a
/// guess that grows quickly.
#[must_use]
pub fn delta_t(jd: f64) -> f64 {
    let y = decimal_year(jd);
    let (y_handover, _) = first_knot();
    let (y_last, v_last) = last_knot();

    // Before the spline's first knot: the Espenak & Meeus parabola. It meets the
    // spline in a step of −74.5 s (module notes).
    if y < table::SPLINE[0].0 {
        return long_term_parabola(y);
    }

    // The spline, to the handover to the measurements. It meets the first USNO
    // row in a step of +0.371 s (module notes).
    if y < y_handover {
        return spline(y);
    }

    // USNO's measurements, then its predictions.
    if y <= y_last {
        return interpolate_table(y);
    }

    // After the last prediction, continue with the Espenak & Meeus expressions'
    // increase since then, so that Delta T has no step where the table ends. The
    // expressions themselves, unshifted, read 8.9 s above USNO's last prediction
    // at 2033.75, because they were written in 2006 for a faster slowing of the
    // Earth than occurred.
    v_last + (future_expression(y) - future_expression(y_last))
}

/// SMH2016 Table S15 at the decimal year `y`, which must lie in `[−720, 1962)`.
///
/// Row `i` with `K_i <= y < K_{i+1}`, `t = (y − K_i)/(K_{i+1} − K_i)`,
/// `DT = a_0 + a_1·t + a_2·t² + a_3·t³`, as the table's notes state. At a knot
/// the later row applies, with `t = 0`.
fn spline(y: f64) -> f64 {
    let rows = table::SPLINE;
    // The first row whose end lies after `y`.
    let i = rows.partition_point(|row| row.1 <= y).min(rows.len() - 1);
    let (k0, k1, a0, a1, a2, a3) = rows[i];
    let t = (y - k0) / (k1 - k0);
    a0 + t * (a1 + t * (a2 + t * a3))
}

/// Linearly interpolate Delta T from the USNO knots, by binary search for the
/// bracketing pair. `y` must lie in `[1962, last knot]`, the table's span.
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

/// Espenak & Meeus (2006) long-term parabola, `−20 + 32·u²` with
/// `u = (y − 1820)/100`: their expression "before the year −500" and the one they
/// give "after 2150". Used before −720 here, and in [`future_expression`] after
/// 2150.
///
/// Source: NASA GSFC, "Polynomial Expressions for Delta T", adapted from *Five
/// Millennium Canon of Solar Eclipses* (Espenak and Meeus),
/// <https://eclipse.gsfc.nasa.gov/SEhelp/deltatpoly2004.html>, retrieved
/// 2026-10-24; coefficients checked against the page.
fn long_term_parabola(y: f64) -> f64 {
    let u = (y - 1820.0) / 100.0;
    -20.0 + 32.0 * u * u
}

/// Espenak & Meeus (2006) expressions for the years after 2005, as published on
/// the page cited at [`long_term_parabola`]: `62.92 + 0.32217·t + 0.005589·t²`
/// (`t = y − 2000`) for 2005–2050, `−20 + 32·u² − 0.5628·(2150 − y)` for
/// 2050–2150 (the last term "introduced to eliminate the discontinuity at 2050"),
/// and the parabola after 2150. Used here only for its increase after 2033.75.
fn future_expression(y: f64) -> f64 {
    if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + t * (0.322_17 + t * 0.005_589)
    } else if y < 2150.0 {
        long_term_parabola(y) - 0.5628 * (2150.0 - y)
    } else {
        long_term_parabola(y)
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
    /// rate (about 3e-8 s) rather than a step. The boundary of 1962-01-01 is the
    /// stated step of +0.371 s where the spline hands over to the measurements, and
    /// is held by `delta_t_is_continuous_at_every_join`.
    #[test]
    fn delta_t_does_not_step_at_month_boundaries() {
        let mut worst = 0.0_f64;
        let mut n = 0;
        for year in 1960..=2025 {
            for month in 1..=12 {
                if (year, month) == (1962, 1) {
                    continue;
                }
                let boundary = julian::calendar_to_jd(year, month, 1.0);
                let before = delta_t(boundary - 0.5 / 86_400.0);
                let after = delta_t(boundary + 0.5 / 86_400.0);
                worst = worst.max((after - before).abs());
                n += 1;
            }
        }
        assert_eq!(n, 66 * 12 - 1);
        assert!(
            worst < 1e-6,
            "ΔT steps by {worst} s across a month boundary"
        );
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

    // --- Continuity across every join ---

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
    ///  1962.000  33.992    1973.000  43.373
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
        const HALF_YEAR: [(i32, f64); 2] = [(1962, 33.992), (1973, 43.373)];
        for (year, expected) in HALF_YEAR {
            let dt = delta_t(julian::calendar_to_jd(year, 1, 1.0));
            assert!(
                (dt - expected).abs() < 1e-6,
                "ΔT at {year}-01-01: expected {expected}, got {dt}"
            );
        }
    }

    /// The knots are strictly increasing in time and Delta T moves by a
    /// believable amount between neighbours: at most 0.15 s a month (largest in
    /// the data 0.109) and between -0.2 and 1.0 s a half year (data -0.12 to
    /// 0.83). A transposed digit or a row out of order breaks one of these.
    #[test]
    fn the_modern_table_is_ordered_and_smooth() {
        let rows = table::MODERN;
        // 23 half-year rows (1962.0 to 1973.0) plus the monthly rows, 645 when
        // the table was made (1973-02 to 2026-10); a refresh adds more.
        assert!(rows.len() >= 23 + 645, "only {} knots", rows.len());
        assert_eq!(rows[0].0, 1962.0);
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

    // --- The SMH2016 spline, −720 to 1962 ---

    /// Delta T at 1 January of a (astronomical) year, 0h, through the public function.
    fn at(year: i32) -> f64 {
        delta_t(julian::calendar_to_jd(year, 1, 1.0))
    }

    /// The spline's Table S15 values at its own knots and midpoints, worked by hand
    /// from the printed coefficients (`t = 0` gives `a_0`; `t = 1/2` gives
    /// `a_0 + a_1/2 + a_2/4 + a_3/8`), so that the interval lookup, the `t`
    /// normalisation and the polynomial are what the table says:
    ///
    /// ```text
    /// row 1, −720..400:   t = 0.5 at −160   20550.593 − 10634.239 + 2965.8545 − 567.6411 = 12314.567
    /// row 6, 1650..1720:  t = 0.5 at 1685   43.736 − 24.5215 + 0.05675 + 1.976375 = 21.2476
    /// row 22, 1895..1900: t = 0   at 1895   a_0 = −5.017
    /// row 23, 1900..1905: t = 0   at 1900   a_0 = −1.977
    /// row 33, 1950..1953: t = 0   at 1950   a_0 = 28.932
    /// ```
    ///
    /// The first interval is 1120 years long, so −160 is its midpoint; the sixth is
    /// 70 years, so 1685 is its midpoint.
    #[test]
    fn delta_t_is_the_spline_where_the_table_says() {
        assert!((at(-160) - 12_314.567).abs() < 0.001, "−160: {}", at(-160));
        assert!((at(1685) - 21.2476).abs() < 0.001, "1685: {}", at(1685));
        assert!((at(1895) - (-5.017)).abs() < 0.001, "1895: {}", at(1895));
        assert!((at(1900) - (-1.977)).abs() < 0.001, "1900: {}", at(1900));
        assert!((at(1950) - 28.932).abs() < 0.001, "1950: {}", at(1950));
    }

    /// At 1892.5 the 1890–1895 interval is at `t = (1892.5 − 1890)/5 = 0.5` and reads
    /// `−3.884 − 0.2655 − 0.61825 + 0.23375 = −4.534` s; the minimum of Delta T near
    /// 1895 is in the spline, about −5.0 s at 1895 and nowhere below −5.6 s, the dip
    /// that 5-year knots read as −3.6 s.
    #[test]
    fn the_1890s_minimum_is_in_the_spline() {
        // Julian day of decimal year 1892.5 (calendar): 1892 has 366 days, so half
        // of it is 183 days after 1 January 0h.
        let jd = julian::calendar_to_jd(1892, 1, 1.0) + 183.0 + 0.0;
        // 1892.5 in the table's variable is 1892 + 183/366.
        let y = decimal_year(jd);
        assert!((y - 1892.5).abs() < 1e-9, "decimal year {y}");
        let expected: f64 = -3.884 - 0.531 * 0.5 - 2.473 * 0.25 + 1.870 * 0.125;
        assert!((expected + 4.534).abs() < 0.0005, "hand figure {expected}");
        assert!(
            (delta_t(jd) - expected).abs() < 1e-6,
            "1892.5: {}",
            delta_t(jd)
        );
        let lowest = (1885..=1900)
            .map(|k| {
                delta_t(julian::calendar_to_jd(1885, 1, 1.0) + f64::from(k - 1885) * 365.25 + 0.5)
            })
            .fold(f64::INFINITY, f64::min);
        assert!(
            lowest < -5.0 && lowest > -5.6,
            "lowest value in 1885–1900 is {lowest}"
        );
        assert!(at(1895) < -4.9, "1895: {}", at(1895));
    }

    /// The cubic spline is continuous in value, slope and curvature at its interior
    /// knots, to the rounding of its published coefficients (three decimals): the
    /// paper chose splines for "the continuity of their first derivative". This holds
    /// only for the `t = (Y − K_i)/(K_{i+1} − K_i)` evaluation the table states: with
    /// `t` in years the slope jumps by thousands of seconds a year.
    #[test]
    fn the_spline_is_continuous_at_every_knot_under_the_stated_evaluation() {
        let rows = table::SPLINE;
        assert_eq!(rows.len(), 36);
        let (mut worst_value, mut worst_slope, mut worst_curve, mut wrong_slope) =
            (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
        for pair in rows.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert_eq!(a.1, b.0, "intervals not contiguous at {}", a.1);
            let (ha, hb) = (a.1 - a.0, b.1 - b.0);
            // End of row a (t = 1) against start of row b (t = 0).
            worst_value = worst_value.max((a.2 + a.3 + a.4 + a.5 - b.2).abs());
            worst_slope = worst_slope.max(((a.3 + 2.0 * a.4 + 3.0 * a.5) / ha - b.3 / hb).abs());
            worst_curve = worst_curve
                .max(((2.0 * a.4 + 6.0 * a.5) / (ha * ha) - 2.0 * b.4 / (hb * hb)).abs());
            // The same slopes with t read in years rather than interval fractions.
            wrong_slope = wrong_slope.max(((a.3 + 2.0 * a.4 + 3.0 * a.5) - b.3).abs());
        }
        assert!(worst_value < 0.002, "value jumps by {worst_value} s");
        assert!(worst_slope < 0.002, "slope jumps by {worst_slope} s/yr");
        assert!(
            worst_curve < 0.001,
            "curvature jumps by {worst_curve} s/yr²"
        );
        assert!(
            wrong_slope > 1000.0,
            "the wrong convention would not have shown: {wrong_slope}"
        );
    }

    /// Table S10 of the supplement: the limits on Delta T that untimed total and
    /// annular solar eclipses give, `(year, DT-U, DT-L)` in seconds. The paper says
    /// of its spline that it "satisfies all the constraints of the total and annular
    /// solar eclipses, except that of −600", and that the lower bound of 454 and the
    /// upper bound of 71 constrain it (so it touches them). Checked at the middle of
    /// each year, since only the year is printed: the bounds are hundreds to
    /// thousands of seconds wide except 1567 (145 to 165 s).
    #[test]
    fn the_spline_satisfies_the_eclipse_limits_the_paper_prints() {
        const S10: [(i32, f64, f64); 34] = [
            (-708, 21100.0, 20160.0),
            (-600, 21900.0, 21120.0),
            (-548, 21640.0, 16160.0),
            (-309, 17160.0, 13300.0),
            (-180, 12700.0, 11780.0),
            (-135, 12140.0, 11220.0),
            (-9, 10720.0, 8640.0),
            (71, 9440.0, 9830.0),
            (454, 7900.0, 6120.0),
            (761, 3260.0, 1700.0),
            (840, 6800.0, 1940.0),
            (873, 3740.0, 1820.0),
            (912, 2580.0, 840.0),
            (968, 2620.0, 1560.0),
            (975, 4480.0, 1160.0),
            (1061, 2140.0, 800.0),
            (1124, 2700.0, 980.0),
            (1133, 1160.0, 520.0),
            (1147, 1160.0, 280.0),
            (1176, 1600.0, 600.0),
            (1178, 1900.0, 780.0),
            (1185, 10500.0, -2100.0),
            (1221, 960.0, -9600.0),
            (1239, 1420.0, -500.0),
            (1241, 1380.0, 620.0),
            (1267, 800.0, -900.0),
            (1275, 1280.0, -740.0),
            (1292, 1820.0, -60.0),
            (1406, 740.0, 160.0),
            (1415, 680.0, -800.0),
            (1431, 680.0, -200.0),
            (1485, 780.0, -5500.0),
            (1560, 210.0, -480.0),
            (1567, 165.0, 145.0),
        ];
        let mut outside = Vec::new();
        for (year, u, l) in S10 {
            let (lo, hi) = (u.min(l), u.max(l));
            let value = delta_t(julian::calendar_to_jd(year, 7, 2.0));
            // 1.5 percent of the larger limit for the bounds the spline is held to.
            let slack = 0.015 * lo.abs().max(hi.abs());
            if value < lo - slack || value > hi + slack {
                outside.push((year, value));
            }
        }
        assert_eq!(outside.len(), 1, "outside their limits: {outside:?}");
        let (year, value) = outside[0];
        assert_eq!(year, -600, "the paper's one exception is −600");
        assert!(
            value < 21_120.0 - 1000.0,
            "−600 should be well below its lower limit, got {value}"
        );
    }

    // --- Continuity across every join ---

    /// The joins of the pieces, with their steps stated rather than smoothed. The
    /// two steps that exist are the sources' disagreement with each other:
    ///
    /// * at −720 the Espenak & Meeus parabola reads `−20 + 32·u²`, `u = (−720 −
    ///   1820)/100 = −25.4`: `−20 + 32·645.16 = 20625.12` s, and the spline starts at
    ///   Table S15's `a_0 = 20550.593`: a step of −74.527 s;
    /// * at 1962.0 the spline is at the end of row 36 (`t = 1`):
    ///   `32.652 + 1.577 − 1.115 + 0.507 = 33.621` s, and the first USNO row is
    ///   33.992 s: a step of +0.371 s.
    ///
    /// Every other join is continuous: the tail onto the predictions (shifted to meet
    /// it), the predictions onto the measurements, the monthly rows onto the
    /// half-year rows (linear interpolation), and the Espenak & Meeus tail's own
    /// 2050 and 2150 joins to 0.001 s (the page itself added a term at 2050 "to
    /// eliminate the discontinuity"). An unstated step anywhere fails this.
    #[test]
    fn delta_t_is_continuous_at_every_join() {
        let half_second = 0.5 / 86_400.0;
        let step = |jd: f64| delta_t(jd + half_second) - delta_t(jd - half_second);

        // The two stated steps.
        let at_minus_720 = julian::calendar_to_jd(-720, 1, 1.0);
        assert!(
            (step(at_minus_720) - (-74.527)).abs() < 0.01,
            "−720: {}",
            step(at_minus_720)
        );
        let at_1962 = julian::calendar_to_jd(1962, 1, 1.0);
        assert!(
            (step(at_1962) - 0.371).abs() < 0.005,
            "1962: {}",
            step(at_1962)
        );

        // The joins that are continuous.
        let mut seen = 0;
        for jd in [
            julian::calendar_to_jd(1973, 1, 1.0),
            julian::calendar_to_jd(1973, 2, 1.0),
            jd_of_mjd(61_314.0), // 2026-10-01, last measurement
            jd_of_mjd(63_871.0), // 2033-10-01, last prediction
            julian::calendar_to_jd(2050, 1, 1.0),
            julian::calendar_to_jd(2150, 1, 1.0),
        ] {
            assert!(
                step(jd).abs() < 1e-3,
                "ΔT steps by {} s at JD {jd}",
                step(jd)
            );
            seen += 1;
        }
        // And every one of the spline's 35 interior knots, to its rounding.
        for pair in table::SPLINE.windows(2) {
            let knot_year = pair[0].1;
            let jd = jd_of_decimal_year(knot_year);
            assert!(
                step(jd).abs() < 0.003,
                "ΔT steps by {} s at {knot_year}",
                step(jd)
            );
            seen += 1;
        }
        assert_eq!(seen, 6 + 35);
    }

    /// Julian Day whose [`decimal_year`] is `y`, for a test.
    fn jd_of_decimal_year(y: f64) -> f64 {
        let year = y.floor() as i32;
        let start = julian::calendar_to_jd(year, 1, 1.0);
        let end = julian::calendar_to_jd(year + 1, 1, 1.0);
        start + (y - f64::from(year)) * (end - start)
    }

    /// No unstated step from −700 to 2026.7. The second difference of Delta T at
    /// spacing 0.05 year is `h²·f''` for a smooth curve, plus `h` times the slope
    /// change at a kink. Measured at this spacing: across the spline at most 0.0016 s
    /// (the rounding of its published coefficients at a knot, and curvature), across
    /// the half-year USNO rows at most 0.007 s and across the monthly rows at most
    /// 0.0125 s (their slope changes by up to 0.25 s/yr between months). The bounds
    /// are 0.005 s on the spline and 0.03 s on the USNO rows, so any step over those,
    /// 0.003″ and 0.017″ of Moon longitude, fails. The two stated steps, at −720 and
    /// 1962.0, are held exactly by `delta_t_is_continuous_at_every_join` (the sweep
    /// stays clear of both). The span stops at the last measurement: the predictions
    /// and the tail after them have slope changes of their own (0.5 s/yr onto the
    /// Espenak & Meeus tail, 1.2 s/yr at its 2050 join and 0.6 s/yr at 2150, from the
    /// published expressions) that are not steps, and are covered by that test's value
    /// checks. The old 4.4 s and 28.6 s steps of 5-year knots and polynomials, and any
    /// 0.01 s one in the spline, fail here.
    #[test]
    fn delta_t_has_no_unstated_step() {
        let h = 0.05;
        let mut y = -700.0;
        let mut worst_spline = 0.0_f64;
        let mut worst_usno = 0.0_f64;
        let mut worst_at = (0.0, 0.0);
        let mut samples = 0;
        while y < 2026.7 {
            let near_stated = (y - 1962.0_f64).abs() < 0.2 || (y + 720.0_f64).abs() < 0.2;
            if !near_stated {
                let f = |q: f64| delta_t(jd_of_decimal_year(q));
                let second = (f(y + h) - 2.0 * f(y) + f(y - h)).abs();
                if y < 1961.8 {
                    if second > worst_spline {
                        worst_spline = second;
                        worst_at.0 = y;
                    }
                } else if second > worst_usno {
                    worst_usno = second;
                    worst_at.1 = y;
                }
                samples += 1;
            }
            y += h;
        }
        assert!(samples > 50_000, "only {samples} samples");
        assert!(
            worst_spline < 0.005,
            "second difference {worst_spline} s near {} in the spline",
            worst_at.0
        );
        assert!(
            worst_usno < 0.03,
            "second difference {worst_usno} s near {} in the USNO rows",
            worst_at.1
        );
        // The sweep can see a step: the spline's rounding at a knot and the USNO
        // kinks are there, so a zero would mean it was not looking.
        assert!(
            worst_spline > 1e-4 && worst_usno > 1e-3,
            "sweep saw nothing"
        );
    }

    /// Before −720 the Espenak & Meeus parabola applies, `−20 + 32·u²`: at −1000
    /// `u = −28.2`, `32·795.24 − 20 = 25427.68` s.
    #[test]
    fn delta_t_before_the_spline_is_the_long_term_parabola() {
        let dt = at(-1000);
        assert!((dt - 25_427.68).abs() < 0.01, "−1000: {dt}");
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
