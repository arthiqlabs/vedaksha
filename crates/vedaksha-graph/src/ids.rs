// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣa — Axis of Wisdom
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Deterministic ID generation for graph nodes.

use core::fmt;

use serde::{Deserialize, Serialize};

/// A node identifier in the graph.
///
/// Global nodes (Sign, Nakshatra, `FixedStar`) have fixed IDs shared across charts.
/// Chart-scoped nodes use a deterministic hash of (`julian_day`, latitude, longitude, config).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a global node ID (shared across all charts).
    /// Example: `"sign:aries"`, `"nakshatra:ashwini"`
    #[must_use]
    pub fn global(category: &str, name: &str) -> Self {
        Self(format!("{category}:{name}"))
    }

    /// Create a chart-scoped node ID.
    /// Uses a deterministic hash of the chart parameters.
    /// Example: `"chart:a1b2c3d4:planet:mars"`
    #[must_use]
    pub fn chart_scoped(chart_id: &str, category: &str, name: &str) -> Self {
        Self(format!("chart:{chart_id}:{category}:{name}"))
    }

    /// Generate a deterministic chart ID from computation parameters.
    /// Same (jd, lat, lon, `config_hash`) always produces the same ID.
    #[must_use]
    pub fn chart_hash(jd: f64, latitude: f64, longitude: f64, config_hash: u64) -> String {
        // FNV-1a 64-bit hash over the raw bytes of all parameters.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
        for byte in jd
            .to_le_bytes()
            .iter()
            .chain(latitude.to_le_bytes().iter())
            .chain(longitude.to_le_bytes().iter())
            .chain(config_hash.to_le_bytes().iter())
        {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3); // FNV prime
        }
        format!("{hash:016x}")
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_id_format() {
        let id = NodeId::global("sign", "aries");
        assert_eq!(id.0, "sign:aries");
    }

    #[test]
    fn chart_scoped_id_includes_chart_hash() {
        let id = NodeId::chart_scoped("abc123", "planet", "mars");
        assert_eq!(id.0, "chart:abc123:planet:mars");
    }

    #[test]
    fn deterministic_same_params_same_hash() {
        let h1 = NodeId::chart_hash(2_451_545.0, 28.6139, 77.2090, 42);
        let h2 = NodeId::chart_hash(2_451_545.0, 28.6139, 77.2090, 42);
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_params_different_hash() {
        let h1 = NodeId::chart_hash(2_451_545.0, 28.6139, 77.2090, 42);
        let h2 = NodeId::chart_hash(2_451_546.0, 28.6139, 77.2090, 42);
        assert_ne!(h1, h2);

        let h3 = NodeId::chart_hash(2_451_545.0, 28.6139, 77.2090, 43);
        assert_ne!(h1, h3);
    }
}
