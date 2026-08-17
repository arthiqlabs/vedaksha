// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-astro
//!
//! Western astrology engine for the Vedaksha platform.
//!
//! This crate implements classical and modern Western astrological
//! computation:
//!
//! - **House systems** — Placidus, Koch, Equal, Whole Sign, Porphyry,
//!   Regiomontanus, and Campanus
//! - **Aspects** — major and minor aspects with configurable orbs
//! - **Dignities** — essential and accidental dignities, receptions
//! - **Chart computation** — natal, transit, synastry, and composite charts

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
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::question_mark)]

pub mod aspects;
pub mod chart;
pub mod composite;
pub mod dignity;
pub mod houses;
pub mod riseset;
pub mod sidereal;
pub mod synastry;
pub mod transits;
