// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! SPK ephemeris reader (std-only).
//!
//! Reads an entire NAIF SPK/DAF binary file into memory, parses the DAF
//! structure, and implements [`EphemerisProvider`] for computing planetary
//! state vectors via Chebyshev interpolation.

use std::path::Path;

use crate::bodies::Body;
use crate::error::ComputeError;
use crate::jpl::daf::{DafFile, DafSegment};
use crate::jpl::spk2::{Spk2Meta, evaluate_type2};
use crate::jpl::{AU_KM, EphemerisProvider, Position, StateVector, Velocity};
use crate::julian::J2000;

/// Seconds per Julian day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// SPK ephemeris reader backed by an in-memory copy of the file.
pub struct SpkReader {
    daf: DafFile,
    /// Entire file contents reinterpreted as f64 values (little-endian).
    data: Vec<f64>,
}

impl SpkReader {
    /// Opens and reads an SPK file from the given path.
    ///
    /// The entire file is loaded into memory and the DAF structure is parsed
    /// to extract segment descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeError::IoError`] if the file cannot be read, or
    /// [`ComputeError::InvalidFormat`] if the DAF structure is invalid.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ComputeError> {
        let raw = std::fs::read(path.as_ref()).map_err(|e| ComputeError::IoError {
            detail: e.to_string(),
        })?;

        let daf = DafFile::parse(&raw)?;

        // Convert entire file to f64 array (little-endian)
        // Pad to multiple of 8 if needed
        let n_doubles = raw.len() / 8;
        let mut data = Vec::with_capacity(n_doubles);
        for i in 0..n_doubles {
            let offset = i * 8;
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&raw[offset..offset + 8]);
            data.push(f64::from_le_bytes(bytes));
        }

        Ok(Self { daf, data })
    }

    /// Finds the segment matching the given NAIF target ID.
    fn find_segment(&self, target_id: i32) -> Option<&DafSegment> {
        self.daf
            .segments
            .iter()
            .find(|seg| seg.target_id == target_id)
    }

    /// Extracts the segment data slice and metadata for a segment.
    fn segment_data_and_meta(&self, seg: &DafSegment) -> Result<(&[f64], Spk2Meta), ComputeError> {
        if seg.spk_type != 2 {
            return Err(ComputeError::InvalidFormat {
                detail: "only SPK Type 2 is supported",
            });
        }

        // Addresses are 1-based f64 indices
        let start_idx = seg.start_addr - 1;
        let end_idx = seg.end_addr; // end_addr is inclusive, so slice to end_addr
        let segment_data = &self.data[start_idx..end_idx];

        let meta = Spk2Meta::from_segment_data(segment_data)?;
        Ok((segment_data, meta))
    }
}

impl EphemerisProvider for SpkReader {
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError> {
        let target_id = body.naif_id();
        let seg = self
            .find_segment(target_id)
            .ok_or(ComputeError::BodyNotAvailable { body_id: target_id })?;

        // Convert JD to seconds past J2000
        let epoch_seconds = (jd - J2000) * SECONDS_PER_DAY;

        // Check epoch is within segment range
        if epoch_seconds < seg.start_epoch || epoch_seconds > seg.end_epoch {
            // Convert segment bounds back to JD for error message
            let min_jd = seg.start_epoch / SECONDS_PER_DAY + J2000;
            let max_jd = seg.end_epoch / SECONDS_PER_DAY + J2000;
            return Err(ComputeError::DateOutOfRange {
                jd,
                min: min_jd,
                max: max_jd,
            });
        }

        let (segment_data, meta) = self.segment_data_and_meta(seg)?;
        let (pos_km, vel_km_s) = evaluate_type2(segment_data, &meta, epoch_seconds)?;

        // Convert km -> AU, km/s -> AU/day
        Ok(StateVector {
            position: Position {
                x: pos_km[0] / AU_KM,
                y: pos_km[1] / AU_KM,
                z: pos_km[2] / AU_KM,
            },
            velocity: Velocity {
                x: vel_km_s[0] * SECONDS_PER_DAY / AU_KM,
                y: vel_km_s[1] * SECONDS_PER_DAY / AU_KM,
                z: vel_km_s[2] * SECONDS_PER_DAY / AU_KM,
            },
        })
    }

    fn time_range(&self) -> (f64, f64) {
        let mut min_jd = f64::MAX;
        let mut max_jd = f64::MIN;
        for seg in &self.daf.segments {
            let seg_min = seg.start_epoch / SECONDS_PER_DAY + J2000;
            let seg_max = seg.end_epoch / SECONDS_PER_DAY + J2000;
            if seg_min < min_jd {
                min_jd = seg_min;
            }
            if seg_max > max_jd {
                max_jd = seg_max;
            }
        }
        (min_jd, max_jd)
    }
}
