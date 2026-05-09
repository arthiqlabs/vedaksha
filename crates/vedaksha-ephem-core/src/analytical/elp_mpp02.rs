// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.

//! ELP/MPP02 lunar theory — quarantine stub.
//!
//! The previous implementation derived structurally from a GPL-3.0
//! source. It has been quarantined pending a clean-room re-derivation
//! from primary sources (Chapront & Francou 2003 + IMCCE primary
//! distribution). See
//! `docs/superpowers/specs/2026-05-09-elp-mpp02-rederivation-design.md`.
//!
//! The public API (`MoonRectangular`, `elp_geocentric_of_date`,
//! `elp_geocentric`) is preserved bit-compatibly so callers continue to
//! compile during the interregnum. Calling either function panics.

/// Geocentric position and velocity of the Moon in J2000 ecliptic
/// rectangular coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonRectangular {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

/// Geocentric Moon in mean ecliptic of date. Quarantined.
pub fn elp_geocentric_of_date(_jd: f64) -> MoonRectangular {
    unimplemented!("ELP/MPP02 quarantined pending clean-room re-derivation");
}

/// Geocentric Moon in mean ecliptic of J2000. Quarantined.
pub fn elp_geocentric(_jd: f64) -> MoonRectangular {
    unimplemented!("ELP/MPP02 quarantined pending clean-room re-derivation");
}
