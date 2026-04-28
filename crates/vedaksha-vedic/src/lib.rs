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

pub mod ashtakavarga;
pub mod bhava;
pub mod combustion;
pub mod dasha;
pub mod drishti;
pub mod karaka;
pub mod muhurta;
pub mod nakshatra;
pub mod panchanga;
pub mod shadbala;
pub mod varga;
pub mod yoga;
