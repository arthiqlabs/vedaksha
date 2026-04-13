// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-vedic
//!
//! Vedic (Jyotish) astrology engine for the Vedākṣa platform.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod bhava;
pub mod dasha;
pub mod drishti;
pub mod muhurta;
pub mod nakshatra;
pub mod shadbala;
pub mod varga;
pub mod yoga;
pub mod panchanga;
