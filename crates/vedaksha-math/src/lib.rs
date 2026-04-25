// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-math
//!
//! Numeric primitives for astronomical and astrological computation.
//!
//! This crate provides foundational mathematical operations used throughout
//! the Vedākṣha platform:
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
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod angle;
pub mod chebyshev;
pub mod interpolation;
pub mod matrix;
