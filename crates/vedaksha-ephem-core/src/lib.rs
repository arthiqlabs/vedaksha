// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
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
// Scientific coefficient tables (VSOP87, ELP/MPP02) use unseparated and
// high-precision float literals imported verbatim from reference data.
#![allow(clippy::unreadable_literal)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::inconsistent_digit_grouping)]
// Astronomy code: many similar variable names (a0/a1/a2), casts between
// numeric types, and hand-tuned inline hints from reference algorithms.
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::inline_always)]
#![allow(clippy::approx_constant)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::double_must_use)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::type_complexity)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::no_effect_underscore_binding)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod aberration;
pub mod analytical;
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
