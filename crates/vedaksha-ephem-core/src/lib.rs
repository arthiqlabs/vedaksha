// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-ephem-core
//!
//! Astronomy engine for the Vedaksha platform providing:
//!
//! - **JPL SPK/DAF reader** — NAIF binary file parsing and Chebyshev
//!   interpolation for high-precision planetary positions
//! - **Coordinate transformations** — ICRS, ecliptic, equatorial frame conversions
//! - **Julian Day** — calendar conversions and epoch utilities

//!
//! **Not `no_std`.** This crate declared `#![cfg_attr(not(feature = "std"),
//! no_std)]` until v5.0.0 and could not honour it: `--no-default-features`
//! failed to compile, so the attribute was a claim no build ever checked.
//! `vedaksha-math` is the one crate in this workspace that is genuinely
//! `no_std`, and CI proves it. The `std` feature and the `alloc` shims are
//! kept — they gate real optional code and are the scaffolding a future
//! `no_std` effort would build on — but the claim is gone until something
//! verifies it.

// Scientific coefficient tables (VSOP87, ELP/MPP02) use unseparated and
// high-precision float literals imported verbatim from reference data.
// Astronomy code: many similar variable names (a0/a1/a2), casts between
// numeric types, and hand-tuned inline hints from reference algorithms.
#![deny(unsafe_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod aberration;
pub mod analytical;
pub mod bodies;
#[cfg(feature = "std")]
pub mod cache;
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
pub mod stars;
