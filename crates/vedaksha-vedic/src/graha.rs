// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Grahas and their placements.
//!
//! These types are the common currency of the Jyotish modules — ashtakavarga,
//! combustion, gochara, karaka and shadbala all speak in terms of a graha and
//! where it sits. They lived in `yoga` as `YogaPlanet` and `PlanetPosition`
//! until that module was removed, which left them named after a concept the
//! crate no longer has.
//!
//! The variant names are unchanged, and neither type carries a serde tag or
//! rename, so nothing on the wire moved with the rename.

use serde::{Deserialize, Serialize};

/// The nine grahas of Jyotish.
///
/// Rahu and Ketu are the lunar nodes, which is why this is a Jyotish-specific
/// enum rather than a re-export of [`vedaksha_ephem_core::bodies::Body`]:
/// the outer planets are absent and the nodes are first-class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Graha {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

/// A graha's placement in a chart.
#[derive(Debug, Clone, Copy)]
pub struct GrahaPosition {
    /// Which planet.
    pub planet: Graha,
    /// Sign index (0 = Aries, 1 = Taurus, ..., 11 = Pisces).
    pub sign: u8,
    /// Sidereal longitude in degrees.
    pub longitude: f64,
    /// House (bhava) 1-12.
    pub bhava: u8,
}
