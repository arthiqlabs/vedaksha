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
//! vedaksha = "4"
//! ```
//!
//! # Quick start
//!
//! This is the block the README shows. It is a doctest so that it is compiled
//! and run, not merely read: the README previously carried a four-argument
//! `compute_chart(jd, lat, lon, &cfg)` and a four-argument `calendar_to_jd`,
//! neither of which had ever existed, because nothing built it.
//!
//! ```
//! use vedaksha::ephem::{delta_t, nutation, obliquity, sidereal_time};
//! use vedaksha::prelude::*;
//!
//! // New Delhi, 2024-03-20 12:00 UT1. `calendar_to_jd` takes the day as a
//! // fraction — noon on the 20th is 20.5 — and it wants UT1, not TT/TDB.
//! let jd = calendar_to_jd(2024, 3, 20.5);
//! let (latitude, longitude): (f64, f64) = (28.6139, 77.2090);
//!
//! // 1. Apparent positions. `compute_chart` takes them as
//! //    (name, longitude°, latitude°, distance AU, speed °/day).
//! let provider = AnalyticalProvider;
//! let mut planets = Vec::new();
//! for (name, body) in [
//!     ("Sun", Body::Sun),
//!     ("Moon", Body::Moon),
//!     ("Mars", Body::Mars),
//!     ("Jupiter", Body::Jupiter),
//! ] {
//!     let p = apparent_position(&provider, body, jd).expect("analytical provider");
//!     planets.push((
//!         name.to_string(),
//!         p.ecliptic.longitude.to_degrees(),
//!         p.ecliptic.latitude.to_degrees(),
//!         p.ecliptic.distance,
//!         p.longitude_speed,
//!     ));
//! }
//!
//! // 2. Earth orientation, which fixes the houses. Nutation and obliquity are
//! //    dynamical terms and take TT; sidereal time is the Earth's rotation and
//! //    takes UT1. Mixing the two costs ΔT worth of rotation on every cusp.
//! let jd_tt = delta_t::ut1_to_tt(jd);
//! let (dpsi, deps) = nutation::nutation(jd_tt);
//! let eps_true = obliquity::true_obliquity(jd_tt, deps);
//! let ramc = sidereal_time::local_sidereal_time(jd, longitude.to_radians(), dpsi, eps_true)
//!     .to_degrees();
//!
//! // 3. The chart — sidereal (Lahiri) with whole-sign bhavas, i.e. a kundali.
//! let config = ChartConfig {
//!     house_system: HouseSystem::WholeSign,
//!     ayanamsha: Some(Ayanamsha::Lahiri),
//!     ..ChartConfig::default()
//! };
//! let chart = compute_chart(
//!     &planets,
//!     ramc,
//!     latitude,
//!     obliquity::mean_obliquity(jd_tt).to_degrees(),
//!     jd,
//!     &config,
//! );
//!
//! println!("Lagna {:.3}°", chart.houses.asc);
//! for planet in &chart.planets {
//!     println!("{:<8} {:>8.3}°", planet.name, planet.longitude);
//! }
//! # assert_eq!(chart.planets.len(), 4);
//! # assert!((0.0..360.0).contains(&chart.houses.asc));
//! ```
//!
//! # v3.0.0 surface changes
//!
//! - `vedaksha::emit` is gone; the emitters now live at
//!   `vedaksha::graph::emitters` (the umbrella always enables the
//!   `emitters` feature on `vedaksha-graph`).
//! - `vedaksha::locale` is gated by the `locale` feature on this crate.

pub use vedaksha_astro as astro;
pub use vedaksha_ephem_core as ephem;
pub use vedaksha_graph as graph;
pub use vedaksha_math as math;
pub use vedaksha_vedic as vedic;

/// Localization tables (planet/sign/nakshatra names in 7 languages).
///
/// Folded in from the standalone `vedaksha-locale` crate (v2.x).
/// Enable via the `locale` feature.
#[cfg(feature = "locale")]
pub mod locale;

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
