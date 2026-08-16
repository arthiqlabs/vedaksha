// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-vedic
//!
//! Vedic (Jyotish) astrology engine for the Vedākṣha platform.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
// Raising the MSRV to 1.89 brought two newly-stable constructs into the
// supported range — `is_multiple_of` (1.87) and let-chains (1.88) — so clippy
// now offers to rewrite `n % k == 0` and `if cond { if let … }` throughout.
// Both are stylistic rewrites of working, tested code, not defects, so they are
// declined rather than churned through the dasha and gochara rules.
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::collapsible_if)]

pub mod ashtakavarga;
pub mod bhava;
pub mod combustion;
pub mod dasha;
pub mod drishti;
pub mod gochara;
pub mod graha;
pub mod karaka;
pub mod muhurta;
pub mod nakshatra;
pub mod panchanga;
pub mod shadbala;
pub mod varga;
