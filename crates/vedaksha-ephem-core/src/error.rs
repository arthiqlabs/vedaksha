// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
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

/// `ComputeError` participates in the standard error ecosystem, so callers can
/// use `?` into `Box<dyn Error>` / `anyhow` instead of unwrapping.
///
/// This is `core::error::Error`, not `std::error::Error`: the trait moved into
/// `core` in Rust 1.81 and the declared MSRV here is 1.89, so the impl costs
/// nothing on a `no_std` target should these crates ever get there. The absence
/// of this impl is why the README example reached for `.expect()` rather than
/// `?`.
///
/// No `source()` override: none of these variants wraps another error.
/// `IoError` carries a formatted `String`, not the `std::io::Error` itself.
impl core::error::Error for ComputeError {}

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

#[cfg(test)]
mod error_trait_tests {
    use super::ComputeError;

    /// The point of the `Error` impl is `?` in a caller that returns a boxed
    /// error — assert that, not merely that the trait is implemented.
    #[test]
    fn compute_error_flows_through_the_question_mark_operator() {
        fn fallible() -> Result<f64, ComputeError> {
            Err(ComputeError::DateOutOfRange {
                jd: 3_000_000.0,
                min: 2_414_864.5,
                max: 2_488_069.5,
            })
        }
        fn caller() -> Result<f64, Box<dyn core::error::Error>> {
            Ok(fallible()?)
        }

        let err = caller().expect_err("the inner call fails");
        assert!(
            err.to_string().contains("out of range"),
            "Display must survive the boxing: {err}"
        );
    }
}
