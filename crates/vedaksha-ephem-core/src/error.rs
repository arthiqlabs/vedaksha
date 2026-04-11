// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Error types for ephemeris computation.

use core::fmt;

/// Errors that can occur during ephemeris computation.
#[derive(Debug, Clone)]
pub enum ComputeError {
    /// The requested Julian Day is outside the ephemeris data range.
    DateOutOfRange { jd: f64, min: f64, max: f64 },
    /// The requested body is not available in the ephemeris data.
    BodyNotAvailable { body_id: i32 },
    /// The ephemeris data has an invalid format.
    InvalidFormat { detail: &'static str },
    /// I/O error (only with `std` feature).
    #[cfg(feature = "std")]
    IoError { detail: String },
}

impl fmt::Display for ComputeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateOutOfRange { jd, min, max } => {
                write!(f, "Julian Day {jd} out of range [{min}, {max}]")
            }
            Self::BodyNotAvailable { body_id } => {
                write!(f, "Body with ID {body_id} not available in ephemeris")
            }
            Self::InvalidFormat { detail } => {
                write!(f, "Invalid ephemeris format: {detail}")
            }
            #[cfg(feature = "std")]
            Self::IoError { detail } => {
                write!(f, "I/O error: {detail}")
            }
        }
    }
}
