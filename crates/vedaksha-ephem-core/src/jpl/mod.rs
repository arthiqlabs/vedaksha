// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! JPL planetary ephemeris reader.
//!
//! Reads NAIF SPK/DAF binary ephemeris files and evaluates Chebyshev
//! polynomials to compute planetary positions and velocities.

pub mod daf;
#[cfg(feature = "std")]
pub mod reader;
pub mod spk2;

use crate::bodies::Body;
use crate::error::ComputeError;

/// Astronomical Unit in km (IAU 2012 exact definition).
pub const AU_KM: f64 = 149_597_870.700;

/// A 3D position vector in AU (astronomical units), ICRS frame.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A 3D velocity vector in AU/day, ICRS frame.
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Position and velocity state vector for a body.
#[derive(Debug, Clone, Copy)]
pub struct StateVector {
    pub position: Position,
    pub velocity: Velocity,
}

/// Trait for ephemeris data providers.
pub trait EphemerisProvider {
    /// Compute the state vector of `body` at Julian Day `jd`.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeError::DateOutOfRange`] if `jd` is outside the provider's
    /// time range, or [`ComputeError::BodyNotAvailable`] if the body is not
    /// present in the ephemeris data.
    fn compute_state(&self, body: Body, jd: f64) -> Result<StateVector, ComputeError>;

    /// Returns the time range covered by this provider.
    fn time_range(&self) -> (f64, f64);
}
