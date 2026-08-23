// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-vedic
//!
//! Vedic (Jyotish) astrology engine for the Vedaksha platform.

//!
//! **Not `no_std`.** This crate declared `#![cfg_attr(not(feature = "std"),
//! no_std)]` until v5.0.0 and could not honour it: `--no-default-features`
//! failed to compile, so the attribute was a claim no build ever checked.
//! `vedaksha-math` is the one crate in this workspace that is genuinely
//! `no_std`, and CI proves it. The `std` feature and the `alloc` shims are
//! kept — they gate real optional code and are the scaffolding a future
//! `no_std` effort would build on — but the claim is gone until something
//! verifies it.
#![deny(unsafe_code)]

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
