// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Shared planet identifiers and chart positions.
//!
//! These types are the common currency of the Jyotish modules — ashtakavarga,
//! combustion, gochara, karaka and shadbala all speak in terms of a graha and
//! where it sits. They lived in `yoga` until that module was removed; the
//! names are kept as-is so dependent code and serialized payloads do not
//! shift in the same change that removed the yoga rules.

use serde::{Deserialize, Serialize};

/// The nine grahas of Jyotish.
///
/// Rahu and Ketu are the lunar nodes, which is why this is a Jyotish-specific
/// enum rather than a re-export of [`vedaksha_ephem_core::bodies::Body`]:
/// the outer planets are absent and the nodes are first-class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YogaPlanet {
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
pub struct PlanetPosition {
    /// Which planet.
    pub planet: YogaPlanet,
    /// Sign index (0 = Aries, 1 = Taurus, ..., 11 = Pisces).
    pub sign: u8,
    /// Sidereal longitude in degrees.
    pub longitude: f64,
    /// House (bhava) 1-12.
    pub bhava: u8,
}
