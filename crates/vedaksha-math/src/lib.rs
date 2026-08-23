// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-math
//!
//! Numeric primitives for astronomical and astrological computation.
//!
//! This crate provides foundational mathematical operations used throughout
//! the Vedaksha platform:
//!
//! - **Chebyshev polynomials** — evaluation of Chebyshev polynomials of the
//!   first kind, used for JPL ephemeris interpolation
//! - **Angle arithmetic** — normalization, conversion between degrees, radians,
//!   DMS, and HMS representations
//! - **Interpolation** — Hermite and Lagrange polynomial interpolation
//! - **Rotation matrices** — 3×3 rotation matrices for coordinate frame
//!   transformations
//!
//! All functions are `no_std` compatible with no `unsafe` code.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod angle;
pub mod chebyshev;
pub mod interpolation;
pub mod matrix;
