// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
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

/// Earth-Moon mass ratio (DE440/441 value).
///
/// One definition for the whole crate: `coordinates`, `analytical` and
/// [`EphemerisProvider::earth_state`]'s default body all divide by this, and a
/// second copy that drifted would put the observer kilometres out without
/// failing to compile.
pub(crate) const EMRAT: f64 = 81.300_568_94;

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

    /// Compute the barycentric state of **Earth's centre** at Julian Day `jd`.
    ///
    /// `Earth = EMB - Moon_relative_to_EMB / EMRAT`
    ///
    /// Providers expose the Moon **relative to the EMB**, not geocentrically —
    /// the SPK file stores EMB (target=3, center=0) and Moon (target=301,
    /// center=3), and [`crate::analytical::AnalyticalProvider`] matches that
    /// convention. The divisor is therefore `EMRAT`, not `1 + EMRAT`:
    ///
    /// ```text
    /// EMB   = Earth + r/(1+EMRAT)              r = geocentric Moon
    /// M_rel = Moon - EMB = r·EMRAT/(1+EMRAT)   =>  r = M_rel·(1+EMRAT)/EMRAT
    /// Earth = EMB - r/(1+EMRAT) = EMB - M_rel/EMRAT
    /// ```
    ///
    /// Using `1/(1+EMRAT)` on a rel-EMB Moon — as `coordinates` did until
    /// 2026-08-20 — leaves the observer 56.8 km off, which is 0.078 arcsec at
    /// 1 AU and scales as 1/distance. It affected every provider, the SPK path
    /// included.
    ///
    /// # Why this is a trait method with a default body
    ///
    /// The default is the general, always-correct construction and is what the
    /// SPK path uses: there, EMB and Moon are independent kernel segments read
    /// from separate Chebyshev records, and nothing about the subtraction is
    /// redundant.
    ///
    /// A provider whose EMB is itself *built* from an Earth series plus the
    /// same Moon term can override this and return that Earth series directly —
    /// see `AnalyticalProvider`'s override, which skips two full 35,758-term
    /// ELP/MPP02 evaluations that algebraically cancel.
    ///
    /// # Errors
    ///
    /// Same conditions as [`Self::compute_state`] for
    /// [`Body::EarthMoonBarycenter`] and [`Body::Moon`].
    fn earth_state(&self, jd: f64) -> Result<StateVector, ComputeError> {
        let emb = self.compute_state(Body::EarthMoonBarycenter, jd)?;
        let moon = self.compute_state(Body::Moon, jd)?;

        let factor = 1.0 / EMRAT;

        Ok(StateVector {
            position: Position {
                x: emb.position.x - moon.position.x * factor,
                y: emb.position.y - moon.position.y * factor,
                z: emb.position.z - moon.position.z * factor,
            },
            velocity: Velocity {
                x: emb.velocity.x - moon.velocity.x * factor,
                y: emb.velocity.y - moon.velocity.y * factor,
                z: emb.velocity.z - moon.velocity.z * factor,
            },
        })
    }

    /// Returns the time range covered by this provider.
    fn time_range(&self) -> (f64, f64);
}
