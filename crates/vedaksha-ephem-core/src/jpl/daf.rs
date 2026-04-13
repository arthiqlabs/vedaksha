// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! NAIF DAF (Double precision Array File) parser.
//!
//! Parses the file record and summary records from a DAF/SPK binary file
//! to extract segment metadata (target, center, epoch range, data addresses).

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::ComputeError;

/// Size of a DAF record in bytes.
const RECORD_SIZE: usize = 1024;

/// A segment descriptor from the DAF summary.
#[derive(Debug, Clone)]
pub struct DafSegment {
    /// NAIF target body ID.
    pub target_id: i32,
    /// NAIF center body ID.
    pub center_id: i32,
    /// Reference frame ID.
    pub frame_id: i32,
    /// SPK data type (e.g. 2 for Chebyshev position).
    pub spk_type: i32,
    /// Start epoch in seconds past J2000.
    pub start_epoch: f64,
    /// End epoch in seconds past J2000.
    pub end_epoch: f64,
    /// Start address (1-based f64 index into file).
    pub start_addr: usize,
    /// End address (1-based f64 index into file).
    pub end_addr: usize,
}

/// Parsed DAF file metadata.
#[derive(Debug)]
pub struct DafFile {
    /// Number of double-precision components per summary (ND).
    pub nd: usize,
    /// Number of integer components per summary (NI).
    pub ni: usize,
    /// Segment descriptors extracted from all summary records.
    pub segments: Vec<DafSegment>,
}

/// Reads a little-endian `i32` from a byte slice at the given offset.
///
/// # Panics
///
/// Panics if `offset + 4 > data.len()`.
#[must_use]
fn read_i32_le(data: &[u8], offset: usize) -> i32 {
    let bytes: [u8; 4] = [
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ];
    i32::from_le_bytes(bytes)
}

/// Reads a little-endian `f64` from a byte slice at the given offset.
///
/// # Panics
///
/// Panics if `offset + 8 > data.len()`.
#[must_use]
fn read_f64_le(data: &[u8], offset: usize) -> f64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[offset..offset + 8]);
    f64::from_le_bytes(bytes)
}

impl DafFile {
    /// Parses DAF file record and all summary records from raw file bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeError::InvalidFormat`] if the file ID is not `"DAF/SPK "`
    /// or if the summary structure is unexpected.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn parse(data: &[u8]) -> Result<Self, ComputeError> {
        if data.len() < RECORD_SIZE {
            return Err(ComputeError::InvalidFormat {
                detail: "file too small for DAF record",
            });
        }

        // Validate file ID
        let file_id = &data[0..8];
        if file_id != b"DAF/SPK " {
            return Err(ComputeError::InvalidFormat {
                detail: "not a DAF/SPK file",
            });
        }

        let nd = read_i32_le(data, 8) as usize;
        let ni = read_i32_le(data, 12) as usize;

        // Summary size: nd doubles + ceil(ni / 2) doubles
        let summary_size_doubles = nd + ni.div_ceil(2);
        let summary_size_bytes = summary_size_doubles * 8;

        // FWARD: record number (1-based) of first summary record
        let fward = read_i32_le(data, 76) as usize;

        let mut segments = Vec::new();
        let mut current_record = fward;

        while current_record != 0 {
            let record_offset = (current_record - 1) * RECORD_SIZE;
            if record_offset + RECORD_SIZE > data.len() {
                return Err(ComputeError::InvalidFormat {
                    detail: "summary record beyond file end",
                });
            }

            // First 3 doubles: next_record, prev_record, n_summaries
            let next_record = read_f64_le(data, record_offset) as usize;
            let n_summaries = read_f64_le(data, record_offset + 16) as usize;

            let summaries_start = record_offset + 24;
            for i in 0..n_summaries {
                let s_offset = summaries_start + i * summary_size_bytes;

                // ND=2 doubles: start_epoch, end_epoch
                let start_epoch = read_f64_le(data, s_offset);
                let end_epoch = read_f64_le(data, s_offset + 8);

                // NI=6 integers: target_id, center_id, frame_id, spk_type, start_addr, end_addr
                let int_offset = s_offset + nd * 8;
                let target_id = read_i32_le(data, int_offset);
                let center_id = read_i32_le(data, int_offset + 4);
                let frame_id = read_i32_le(data, int_offset + 8);
                let spk_type = read_i32_le(data, int_offset + 12);
                let start_addr = read_i32_le(data, int_offset + 16) as usize;
                let end_addr = read_i32_le(data, int_offset + 20) as usize;

                segments.push(DafSegment {
                    target_id,
                    center_id,
                    frame_id,
                    spk_type,
                    start_epoch,
                    end_epoch,
                    start_addr,
                    end_addr,
                });
            }

            current_record = next_record;
        }

        Ok(Self { nd, ni, segments })
    }
}
