// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! SPK Type 2 Chebyshev position segment evaluator.
//!
//! Reads segment metadata (init, intlen, rsize, `n_records`) from the last
//! four doubles of a segment, locates the correct Chebyshev record for a
//! given epoch, and evaluates position and velocity via Clenshaw's algorithm.

use crate::error::ComputeError;
use vedaksha_math::chebyshev::chebyshev_compute;

/// SPK Type 2 segment metadata, stored at the end of each segment.
#[derive(Debug, Clone, Copy)]
pub struct Spk2Meta {
    /// Start epoch in seconds past J2000 (TDB).
    pub init: f64,
    /// Interval length in seconds.
    pub intlen: f64,
    /// Record size in doubles (2 + 3 * `n_coeffs`).
    pub rsize: usize,
    /// Number of Chebyshev records in this segment.
    pub n_records: usize,
}

impl Spk2Meta {
    /// Reads the SPK Type 2 metadata from the last 4 doubles of a segment.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeError::InvalidFormat`] if the segment data is too short.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_segment_data(segment_data: &[f64]) -> Result<Self, ComputeError> {
        let len = segment_data.len();
        if len < 4 {
            return Err(ComputeError::InvalidFormat {
                detail: "SPK Type 2 segment too short for metadata",
            });
        }
        let init = segment_data[len - 4];
        let intlen = segment_data[len - 3];
        let rsize = segment_data[len - 2] as usize;
        let n_records = segment_data[len - 1] as usize;
        Ok(Self {
            init,
            intlen,
            rsize,
            n_records,
        })
    }

    /// Returns the number of Chebyshev coefficients per component.
    #[must_use]
    pub fn n_coeffs(&self) -> usize {
        (self.rsize - 2) / 3
    }
}

/// Evaluates an SPK Type 2 segment at the given epoch.
///
/// Returns position in km `[x, y, z]` and velocity in km/s `[vx, vy, vz]`.
///
/// # Errors
///
/// Returns [`ComputeError::DateOutOfRange`] if `epoch_seconds` falls outside
/// the segment's time coverage.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn evaluate_type2(
    segment_data: &[f64],
    meta: &Spk2Meta,
    epoch_seconds: f64,
) -> Result<([f64; 3], [f64; 3]), ComputeError> {
    // Determine which record contains this epoch
    let dt = epoch_seconds - meta.init;
    if dt < 0.0 {
        return Err(ComputeError::DateOutOfRange {
            jd: 0.0,
            min: 0.0,
            max: 0.0,
        });
    }

    let mut idx = (dt / meta.intlen) as usize;
    // Clamp to last record if at exact end
    if idx >= meta.n_records {
        if idx == meta.n_records {
            idx = meta.n_records - 1;
        } else {
            return Err(ComputeError::DateOutOfRange {
                jd: 0.0,
                min: 0.0,
                max: 0.0,
            });
        }
    }

    let record_start = idx * meta.rsize;
    let midpoint = segment_data[record_start];
    let half_span = segment_data[record_start + 1];
    let n_coeffs = meta.n_coeffs();

    // Normalize time to [-1, 1]
    let tau = (epoch_seconds - midpoint) / half_span;

    let mut position = [0.0_f64; 3];
    let mut velocity = [0.0_f64; 3];

    for component in 0..3 {
        let coeff_start = record_start + 2 + component * n_coeffs;
        let coeffs = &segment_data[coeff_start..coeff_start + n_coeffs];
        let (val, deriv) = chebyshev_compute(coeffs, tau);
        position[component] = val;
        // derivative wrt tau -> derivative wrt seconds: divide by half_span
        velocity[component] = deriv / half_span;
    }

    Ok((position, velocity))
}
