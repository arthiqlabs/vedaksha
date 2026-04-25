// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # vedaksha-astro
//!
//! Western astrology engine for the Vedākṣha platform.
//!
//! This crate implements classical and modern Western astrological
//! computation:
//!
//! - **House systems** — Placidus, Koch, Equal, Whole Sign, Porphyry,
//!   Regiomontanus, and Campanus
//! - **Aspects** — major and minor aspects with configurable orbs
//! - **Dignities** — essential and accidental dignities, receptions
//! - **Chart computation** — natal, transit, synastry, and composite charts

#![cfg_attr(not(feature = "std"), no_std)]
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
pub mod sidereal;
pub mod synastry;
pub mod transits;
