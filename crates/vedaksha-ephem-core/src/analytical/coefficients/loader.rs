// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! Binary coefficient blob loader.
//!
//! VSOP87A and ELP/MPP02 coefficient tables are stored as packed
//! little-endian binary blobs (`*.bin`) alongside thin `LazyLock`
//! wrapper modules. This module defines the on-disk format, the
//! record types, and the loaders.
//!
//! ## On-disk format
//!
//! Every coefficient blob starts with a 24-byte header followed by
//! `record_count * record_size_bytes` of packed records. All multi-byte
//! integers are little-endian; doubles are IEEE 754 little-endian (the
//! representation guaranteed by [`f64::to_le_bytes`] /
//! [`f64::from_le_bytes`]).
//!
//! ```text
//! offset  size  field
//! ──────  ────  ────────────────────────────────────────────
//!   0      8    magic = b"VDKBLOB1"
//!   8      4    version (u32 LE; current = 1)
//!  12      4    record_size_bytes (u32 LE)
//!  16      4    record_count (u32 LE)
//!  20      4    reserved (u32 LE; zero)
//!  24      …    record_count packed records
//! ```
//!
//! No padding inside records. Field order matches the source-tuple field
//! order documented for each record type.

#![allow(clippy::module_name_repetitions)]

/// Magic bytes prefixed to every Vedākṣha coefficient blob.
pub const MAGIC: [u8; 8] = *b"VDKBLOB1";

/// Current binary format version.
pub const VERSION: u32 = 1;

/// Header length in bytes.
pub const HEADER_LEN: usize = 24;

/// On-disk size of a [`Vsop87Term`] record.
pub const VSOP87_RECORD_SIZE: u32 = 24;

/// On-disk size of an [`ElpMainTerm`] record.
pub const ELP_MAIN_RECORD_SIZE: u32 = 72;

/// On-disk size of an [`ElpPertTerm`] record.
pub const ELP_PERT_RECORD_SIZE: u32 = 68;

/// VSOP87A Poisson-series term: `A * cos(B + C * t)`.
///
/// Field order matches the source-tuple order:
/// `(amplitude, phase, frequency)`. On disk this is three packed `f64`
/// little-endian values, 24 bytes total.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vsop87Term {
    /// Amplitude `A` (AU for position).
    pub amplitude: f64,
    /// Phase offset `B` (radians).
    pub phase: f64,
    /// Frequency `C` (radians per Julian millennium).
    pub frequency: f64,
}

/// ELP/MPP02 main-problem term.
///
/// Field order matches the source-tuple order:
/// `(i1, i2, i3, i4, amp, b1, b2, b3, b4, b5, b6)`. On disk this is four
/// packed `i32` little-endian integers followed by seven packed `f64`
/// little-endian doubles, 72 bytes total.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct ElpMainTerm {
    /// Multiplier of Delaunay D.
    pub i1: i32,
    /// Multiplier of Delaunay F.
    pub i2: i32,
    /// Multiplier of Delaunay l.
    pub i3: i32,
    /// Multiplier of Delaunay l'.
    pub i4: i32,
    /// Raw amplitude (arcsec for V/U, km for r).
    pub amp: f64,
    /// Partial ∂A/∂m.
    pub b1: f64,
    /// Partial ∂A/∂Γ.
    pub b2: f64,
    /// Partial ∂A/∂E.
    pub b3: f64,
    /// Partial ∂A/∂e'.
    pub b4: f64,
    /// Partial ∂A/∂α.
    pub b5: f64,
    /// Partial ∂A/∂μ.
    pub b6: f64,
}

/// ELP/MPP02 perturbation term.
///
/// Field order matches the source-tuple order:
/// `(s, c, i1..i13)`. On disk this is two packed `f64` little-endian
/// doubles followed by thirteen packed `i32` little-endian integers,
/// 68 bytes total.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct ElpPertTerm {
    /// Sine coefficient.
    pub s: f64,
    /// Cosine coefficient.
    pub c: f64,
    /// Multiplier of D.
    pub i1: i32,
    /// Multiplier of F.
    pub i2: i32,
    /// Multiplier of l.
    pub i3: i32,
    /// Multiplier of l'.
    pub i4: i32,
    /// Multiplier of Mercury mean longitude.
    pub i5: i32,
    /// Multiplier of Venus mean longitude.
    pub i6: i32,
    /// Multiplier of Earth mean longitude.
    pub i7: i32,
    /// Multiplier of Mars mean longitude.
    pub i8: i32,
    /// Multiplier of Jupiter mean longitude.
    pub i9: i32,
    /// Multiplier of Saturn mean longitude.
    pub i10: i32,
    /// Multiplier of Uranus mean longitude.
    pub i11: i32,
    /// Multiplier of Neptune mean longitude.
    pub i12: i32,
    /// Multiplier of ζ (precession argument).
    pub i13: i32,
}

/// Errors produced while parsing a coefficient blob.
#[derive(Debug, PartialEq, Eq)]
pub enum LoadError {
    /// Blob is shorter than the fixed 24-byte header.
    HeaderTooShort,
    /// Magic prefix did not match `b"VDKBLOB1"`.
    BadMagic,
    /// Version number not recognised by this build.
    UnsupportedVersion(u32),
    /// `record_size_bytes` did not match the expected record size for the
    /// requested record type.
    RecordSizeMismatch { expected: u32, actual: u32 },
    /// Header `record_count * record_size_bytes` did not equal the trailing
    /// payload length.
    PayloadLengthMismatch { expected: usize, actual: usize },
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::HeaderTooShort => f.write_str("coefficient blob shorter than header"),
            Self::BadMagic => f.write_str("coefficient blob magic mismatch"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported blob version {v}"),
            Self::RecordSizeMismatch { expected, actual } => write!(
                f,
                "record size mismatch: expected {expected}, found {actual}"
            ),
            Self::PayloadLengthMismatch { expected, actual } => write!(
                f,
                "payload length mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// Validated blob header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// Format version.
    pub version: u32,
    /// Bytes per record.
    pub record_size: u32,
    /// Number of records.
    pub record_count: u32,
}

/// Parse and validate the 24-byte header.
///
/// # Errors
/// Returns [`LoadError`] if the blob is too short, has the wrong magic,
/// or uses an unsupported version.
pub fn parse_header(bytes: &[u8]) -> Result<Header, LoadError> {
    let header: &[u8; HEADER_LEN] = bytes
        .get(..HEADER_LEN)
        .and_then(|s| s.try_into().ok())
        .ok_or(LoadError::HeaderTooShort)?;
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&header[0..8]);
    if magic != MAGIC {
        return Err(LoadError::BadMagic);
    }
    let mut buf4 = [0u8; 4];
    buf4.copy_from_slice(&header[8..12]);
    let version = u32::from_le_bytes(buf4);
    if version != VERSION {
        return Err(LoadError::UnsupportedVersion(version));
    }
    buf4.copy_from_slice(&header[12..16]);
    let record_size = u32::from_le_bytes(buf4);
    buf4.copy_from_slice(&header[16..20]);
    let record_count = u32::from_le_bytes(buf4);
    Ok(Header {
        version,
        record_size,
        record_count,
    })
}

fn check_payload(bytes: &[u8], expected_record_size: u32) -> Result<(Header, &[u8]), LoadError> {
    let header = parse_header(bytes)?;
    if header.record_size != expected_record_size {
        return Err(LoadError::RecordSizeMismatch {
            expected: expected_record_size,
            actual: header.record_size,
        });
    }
    let payload_expected = (header.record_count as usize) * (header.record_size as usize);
    let payload = &bytes[HEADER_LEN..];
    if payload.len() != payload_expected {
        return Err(LoadError::PayloadLengthMismatch {
            expected: payload_expected,
            actual: payload.len(),
        });
    }
    Ok((header, payload))
}

/// Bounds-checked little-endian `f64` read. Caller guarantees `off + 8 <= b.len()`
/// (we verify `payload.len() == record_count * record_size` upstream).
#[inline]
fn read_f64(b: &[u8], off: usize) -> f64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&b[off..off + 8]);
    f64::from_le_bytes(buf)
}

/// Bounds-checked little-endian `i32` read. Caller guarantees `off + 4 <= b.len()`.
#[inline]
fn read_i32(b: &[u8], off: usize) -> i32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&b[off..off + 4]);
    i32::from_le_bytes(buf)
}

/// Decode a VSOP87A coefficient blob.
///
/// # Errors
/// Returns [`LoadError`] for any header / size mismatch.
pub fn parse_vsop87(bytes: &[u8]) -> Result<Vec<Vsop87Term>, LoadError> {
    let (header, payload) = check_payload(bytes, VSOP87_RECORD_SIZE)?;
    let mut out = Vec::with_capacity(header.record_count as usize);
    let stride = VSOP87_RECORD_SIZE as usize;
    for i in 0..header.record_count as usize {
        let off = i * stride;
        out.push(Vsop87Term {
            amplitude: read_f64(payload, off),
            phase: read_f64(payload, off + 8),
            frequency: read_f64(payload, off + 16),
        });
    }
    Ok(out)
}

/// Decode an ELP/MPP02 main-problem blob.
///
/// # Errors
/// Returns [`LoadError`] for any header / size mismatch.
pub fn parse_elp_main(bytes: &[u8]) -> Result<Vec<ElpMainTerm>, LoadError> {
    let (header, payload) = check_payload(bytes, ELP_MAIN_RECORD_SIZE)?;
    let mut out = Vec::with_capacity(header.record_count as usize);
    let stride = ELP_MAIN_RECORD_SIZE as usize;
    for i in 0..header.record_count as usize {
        let off = i * stride;
        out.push(ElpMainTerm {
            i1: read_i32(payload, off),
            i2: read_i32(payload, off + 4),
            i3: read_i32(payload, off + 8),
            i4: read_i32(payload, off + 12),
            amp: read_f64(payload, off + 16),
            b1: read_f64(payload, off + 24),
            b2: read_f64(payload, off + 32),
            b3: read_f64(payload, off + 40),
            b4: read_f64(payload, off + 48),
            b5: read_f64(payload, off + 56),
            b6: read_f64(payload, off + 64),
        });
    }
    Ok(out)
}

/// Decode an ELP/MPP02 perturbation blob.
///
/// # Errors
/// Returns [`LoadError`] for any header / size mismatch.
pub fn parse_elp_pert(bytes: &[u8]) -> Result<Vec<ElpPertTerm>, LoadError> {
    let (header, payload) = check_payload(bytes, ELP_PERT_RECORD_SIZE)?;
    let mut out = Vec::with_capacity(header.record_count as usize);
    let stride = ELP_PERT_RECORD_SIZE as usize;
    for i in 0..header.record_count as usize {
        let off = i * stride;
        out.push(ElpPertTerm {
            s: read_f64(payload, off),
            c: read_f64(payload, off + 8),
            i1: read_i32(payload, off + 16),
            i2: read_i32(payload, off + 20),
            i3: read_i32(payload, off + 24),
            i4: read_i32(payload, off + 28),
            i5: read_i32(payload, off + 32),
            i6: read_i32(payload, off + 36),
            i7: read_i32(payload, off + 40),
            i8: read_i32(payload, off + 44),
            i9: read_i32(payload, off + 48),
            i10: read_i32(payload, off + 52),
            i11: read_i32(payload, off + 56),
            i12: read_i32(payload, off + 60),
            i13: read_i32(payload, off + 64),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_blob(version: u32, record_size: u32, records: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(HEADER_LEN + records.len());
        v.extend_from_slice(&MAGIC);
        v.extend_from_slice(&version.to_le_bytes());
        v.extend_from_slice(&record_size.to_le_bytes());
        let count = (records.len() / record_size as usize) as u32;
        v.extend_from_slice(&count.to_le_bytes());
        v.extend_from_slice(&0u32.to_le_bytes());
        v.extend_from_slice(records);
        v
    }

    #[test]
    fn header_round_trip() {
        let blob = build_blob(1, VSOP87_RECORD_SIZE, &[0u8; 0]);
        let h = parse_header(&blob).unwrap();
        assert_eq!(h.version, 1);
        assert_eq!(h.record_size, VSOP87_RECORD_SIZE);
        assert_eq!(h.record_count, 0);
    }

    #[test]
    fn rejects_short_header() {
        assert_eq!(parse_header(&[0u8; 8]), Err(LoadError::HeaderTooShort));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut blob = build_blob(1, VSOP87_RECORD_SIZE, &[]);
        blob[0] = b'X';
        assert_eq!(parse_header(&blob), Err(LoadError::BadMagic));
    }

    #[test]
    fn rejects_unsupported_version() {
        let blob = build_blob(99, VSOP87_RECORD_SIZE, &[]);
        assert_eq!(parse_header(&blob), Err(LoadError::UnsupportedVersion(99)));
    }

    #[test]
    fn vsop87_term_round_trip() {
        let term = Vsop87Term {
            amplitude: 0.123_456_789_012_345,
            phase: 1.234_567_890,
            frequency: 26_087.903_141_574_2,
        };
        let mut rec = Vec::with_capacity(VSOP87_RECORD_SIZE as usize);
        rec.extend_from_slice(&term.amplitude.to_le_bytes());
        rec.extend_from_slice(&term.phase.to_le_bytes());
        rec.extend_from_slice(&term.frequency.to_le_bytes());
        let blob = build_blob(1, VSOP87_RECORD_SIZE, &rec);
        let parsed = parse_vsop87(&blob).unwrap();
        assert_eq!(parsed, vec![term]);
    }

    #[test]
    fn elp_main_term_round_trip() {
        let term = ElpMainTerm {
            i1: 0,
            i2: 2,
            i3: -1,
            i4: 0,
            amp: 385_000.527_19,
            b1: -7_992.63,
            b2: -11.06,
            b3: 21_578.08,
            b4: -4.53,
            b5: 11.39,
            b6: -0.06,
        };
        let mut rec = Vec::with_capacity(ELP_MAIN_RECORD_SIZE as usize);
        rec.extend_from_slice(&term.i1.to_le_bytes());
        rec.extend_from_slice(&term.i2.to_le_bytes());
        rec.extend_from_slice(&term.i3.to_le_bytes());
        rec.extend_from_slice(&term.i4.to_le_bytes());
        rec.extend_from_slice(&term.amp.to_le_bytes());
        rec.extend_from_slice(&term.b1.to_le_bytes());
        rec.extend_from_slice(&term.b2.to_le_bytes());
        rec.extend_from_slice(&term.b3.to_le_bytes());
        rec.extend_from_slice(&term.b4.to_le_bytes());
        rec.extend_from_slice(&term.b5.to_le_bytes());
        rec.extend_from_slice(&term.b6.to_le_bytes());
        let blob = build_blob(1, ELP_MAIN_RECORD_SIZE, &rec);
        let parsed = parse_elp_main(&blob).unwrap();
        assert_eq!(parsed, vec![term]);
    }

    #[test]
    fn elp_pert_term_round_trip() {
        let term = ElpPertTerm {
            s: 1.234_567_890_123_456e-3,
            c: -2.345_678_901_234_567e-4,
            i1: 1,
            i2: -2,
            i3: 3,
            i4: -4,
            i5: 5,
            i6: -6,
            i7: 7,
            i8: -8,
            i9: 9,
            i10: -10,
            i11: 11,
            i12: -12,
            i13: 13,
        };
        let mut rec = Vec::with_capacity(ELP_PERT_RECORD_SIZE as usize);
        rec.extend_from_slice(&term.s.to_le_bytes());
        rec.extend_from_slice(&term.c.to_le_bytes());
        for v in [
            term.i1, term.i2, term.i3, term.i4, term.i5, term.i6, term.i7, term.i8, term.i9,
            term.i10, term.i11, term.i12, term.i13,
        ] {
            rec.extend_from_slice(&v.to_le_bytes());
        }
        let blob = build_blob(1, ELP_PERT_RECORD_SIZE, &rec);
        let parsed = parse_elp_pert(&blob).unwrap();
        assert_eq!(parsed, vec![term]);
    }

    #[test]
    fn record_size_mismatch_detected() {
        let blob = build_blob(1, ELP_MAIN_RECORD_SIZE, &[]);
        let err = parse_vsop87(&blob).unwrap_err();
        assert_eq!(
            err,
            LoadError::RecordSizeMismatch {
                expected: VSOP87_RECORD_SIZE,
                actual: ELP_MAIN_RECORD_SIZE,
            }
        );
    }

    #[test]
    fn payload_length_mismatch_detected() {
        // Header says 2 records of 24 bytes, but payload has only 24 bytes.
        let mut blob = Vec::new();
        blob.extend_from_slice(&MAGIC);
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(&VSOP87_RECORD_SIZE.to_le_bytes());
        blob.extend_from_slice(&2u32.to_le_bytes());
        blob.extend_from_slice(&0u32.to_le_bytes());
        blob.extend_from_slice(&[0u8; 24]);
        let err = parse_vsop87(&blob).unwrap_err();
        assert!(matches!(err, LoadError::PayloadLengthMismatch { .. }));
    }
}
