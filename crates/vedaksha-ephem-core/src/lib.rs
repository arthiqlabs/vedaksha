// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-ephem-core
//!
//! Astronomy engine for the Vedākṣa platform providing:
//!
//! - **JPL SPK/DAF reader** — NAIF binary file parsing and Chebyshev
//!   interpolation for high-precision planetary positions
//! - **Coordinate transformations** — ICRS, ecliptic, equatorial frame conversions
//! - **Julian Day** — calendar conversions and epoch utilities

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod aberration;
pub mod bodies;
pub mod coordinates;
pub mod delta_t;
pub mod error;
pub mod jpl;
pub mod julian;
pub mod light_time;
pub mod nodes;
pub mod nutation;
pub mod obliquity;
pub mod precession;
pub mod sidereal_time;
