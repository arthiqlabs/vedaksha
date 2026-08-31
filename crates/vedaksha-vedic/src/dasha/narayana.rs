// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Narayana Dasha — a documented alias of Chara Dasha.
//!
//! **No classical attestation for this name.** "Narayana Dasha" appears in
//! no classical corpus consulted; it is a coinage from a 1998 book by a
//! living author. Its duration and sequencing rule, as popularised under
//! that name, is identical to Chara (Jaimini) Dasha's — see
//! [`super::chara`] for the fully-attested rule and its citations. This
//! module previously cited "Jaimini Sutras Ch. 2" for Narayana Dasha; that
//! citation could not be supported by any source consulted and has been
//! removed. The name is kept only because downstream consumers and the MCP
//! surface resolve dasha systems by name, and "Narayana" is a name people
//! ask this system for.
//!
//! **Superseded (2026-08):** earlier versions of this module (a) duplicated
//! Chara's now-removed hardcoded lord-domicile table (see `dasha::chara`'s
//! module doc for why that table was wrong), and (b) used its own
//! sequence-direction rule — plain odd/even tested directly on the lagna,
//! with no exception — which this module's own doc flagged as an
//! unresearched, disclosed gap relative to Chara's 9th-house vishama-pada
//! test. Making this module a genuine alias of Chara resolves both at once:
//! Narayana Dasha, as computed here, is `compute_chara` under another name,
//! full stop. If Narayana Dasha's own source material turns out to specify
//! a distinct sequence-direction rule after independent research, that
//! would be grounds to fork this module again — it is not assumed here.
//!
//! **Out of scope:** the distinguishing rules that make Narayana Dasha its
//! own system in the source that coined it — most notably a different
//! start-sign rule — are not implemented. They exist only in that
//! copyrighted modern book and are deliberately left unimplemented rather
//! than reconstructed from secondary description.

use super::chara::{GrahaSigns, compute_chara};
use serde::{Deserialize, Serialize};

/// A single Narayana Dasha sign period.
///
/// Field-for-field identical in meaning to `chara::CharaPeriod`, since
/// Narayana Dasha is computed as an alias of Chara (see the module doc) —
/// kept as its own owned-`String` type, rather than a type alias to
/// `CharaPeriod`, purely so this struct's `Deserialize` derive does not
/// have to propagate `CharaPeriod`'s `&'static str` field through a nested
/// generic impl (which does not compile: the borrowed field forces
/// `CharaPeriod`'s own `Deserialize<'de>` impl to require `'de: 'static`,
/// a bound `NarayanaDasha`'s derive cannot satisfy for its own generic
/// `'de`). No behavioural difference results — this is purely a
/// serde-mechanics accommodation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarayanaPeriod {
    /// Zero-based sign index (0 = Aries, 11 = Pisces).
    pub sign_index: u8,
    /// English name of the sign.
    pub sign_name: String,
    /// Start date as Julian Day.
    pub start_jd: f64,
    /// End date as Julian Day.
    pub end_jd: f64,
    /// Duration in years.
    pub duration_years: f64,
}

/// Complete Narayana Dasha sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarayanaDasha {
    /// Zero-based sign index of the lagna.
    pub lagna_sign: u8,
    /// All 12 sign periods in dasha order.
    pub periods: Vec<NarayanaPeriod>,
}

/// Compute the Narayana Dasha sequence for all 12 signs.
///
/// This is a thin wrapper over [`compute_chara`] — see the module doc for
/// why Narayana Dasha is implemented as a documented alias of Chara Dasha
/// rather than an independent system.
///
/// # Arguments
/// * `lagna_sign` — Zero-based sign index of the Lagna (Ascendant), 0-11.
/// * `birth_jd`   — Julian Day of birth.
/// * `positions`  — natal sign positions of the nine grahas; see
///   [`super::chara::compute_chara`] for how these determine each sign's
///   duration.
#[must_use]
pub fn compute_narayana(lagna_sign: u8, birth_jd: f64, positions: GrahaSigns) -> NarayanaDasha {
    let lagna_sign = lagna_sign % 12;
    let periods = compute_chara(lagna_sign, birth_jd, positions)
        .into_iter()
        .map(|p| NarayanaPeriod {
            sign_index: p.sign_index,
            sign_name: p.sign_name.to_string(),
            start_jd: p.start_jd,
            end_jd: p.end_jd,
            duration_years: p.duration_years,
        })
        .collect();
    NarayanaDasha {
        lagna_sign,
        periods,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JD: f64 = 2_451_545.0; // J2000.0

    const ALL_ARIES: GrahaSigns = GrahaSigns {
        sun: 0,
        moon: 0,
        mars: 0,
        mercury: 0,
        jupiter: 0,
        venus: 0,
        saturn: 0,
        rahu: 0,
    };

    // 1. Narayana dasha produces exactly 12 periods
    #[test]
    fn narayana_has_12_periods() {
        let result = compute_narayana(0, TEST_JD, ALL_ARIES);
        assert_eq!(
            result.periods.len(),
            12,
            "expected 12 Narayana periods, got {}",
            result.periods.len()
        );
    }

    // 2. Narayana Dasha is a genuine alias: identical to compute_chara for
    // the same inputs.
    #[test]
    fn narayana_matches_chara_exactly() {
        let narayana = compute_narayana(10, TEST_JD, ALL_ARIES);
        let chara = super::super::chara::compute_chara(10, TEST_JD, ALL_ARIES);
        assert_eq!(narayana.lagna_sign, 10);
        assert_eq!(narayana.periods.len(), chara.len());
        for (n, c) in narayana.periods.iter().zip(chara.iter()) {
            assert_eq!(n.sign_index, c.sign_index);
            assert_eq!(n.duration_years, c.duration_years);
            assert_eq!(n.start_jd, c.start_jd);
            assert_eq!(n.end_jd, c.end_jd);
        }
    }

    // 3. Periods are contiguous
    #[test]
    fn periods_are_contiguous() {
        let result = compute_narayana(0, TEST_JD, ALL_ARIES);
        for i in 1..result.periods.len() {
            let gap = (result.periods[i].start_jd - result.periods[i - 1].end_jd).abs();
            assert!(gap < 1e-9, "gap between periods {}-{}: {gap}", i - 1, i);
        }
    }

    // 4. All 12 signs appear exactly once
    #[test]
    fn all_12_signs_appear() {
        let result = compute_narayana(0, TEST_JD, ALL_ARIES);
        let mut seen = [false; 12];
        for p in &result.periods {
            assert!(
                (p.sign_index as usize) < 12,
                "sign_index out of range: {}",
                p.sign_index
            );
            assert!(
                !seen[p.sign_index as usize],
                "sign {} appears more than once",
                p.sign_index
            );
            seen[p.sign_index as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "not all 12 signs appeared");
    }
}
