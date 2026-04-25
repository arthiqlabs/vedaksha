// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! # Vedākṣha
//!
//! Umbrella crate that re-exports every public sub-crate under a single
//! dependency.  Add `vedaksha` to your `Cargo.toml` and access the full
//! engine through one import.
//!
//! ```toml
//! [dependencies]
//! vedaksha = "1.6"
//! ```

pub use vedaksha_astro as astro;
pub use vedaksha_emit as emit;
pub use vedaksha_ephem_core as ephem;
pub use vedaksha_graph as graph;
pub use vedaksha_locale as locale;
pub use vedaksha_math as math;
pub use vedaksha_vedic as vedic;

/// Convenience re-exports for the most common entry points.
pub mod prelude {
    // Julian day conversion
    pub use vedaksha_ephem_core::julian::calendar_to_jd;
    pub use vedaksha_ephem_core::julian::jd_to_calendar;

    // Ephemeris provider
    pub use vedaksha_ephem_core::analytical::AnalyticalProvider;
    pub use vedaksha_ephem_core::bodies::Body;
    pub use vedaksha_ephem_core::coordinates::apparent_position;

    // Chart computation
    pub use vedaksha_astro::aspects::AspectType;
    pub use vedaksha_astro::chart::{ChartConfig, ComputedChart, compute_chart};
    pub use vedaksha_astro::dignity::RulershipScheme;
    pub use vedaksha_astro::houses::HouseSystem;
    pub use vedaksha_astro::sidereal::Ayanamsha;

    // Vedic
    pub use vedaksha_vedic::dasha;
    pub use vedaksha_vedic::nakshatra;
    pub use vedaksha_vedic::panchanga;
}
