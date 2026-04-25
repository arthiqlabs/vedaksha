// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! Data classification tags for GDPR/DPDP compliance.

use serde::{Deserialize, Serialize};

/// Data classification for GDPR/DPDP compliance.
///
/// Propagates to all emitted graph nodes so downstream systems
/// know the privacy level of the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataClassification {
    /// No personal data — computed from coordinates only.
    Anonymous,
    /// Contains hashed/tokenized identifiers.
    Pseudonymized,
    /// Contains or is linked to personal data (name, birth date, etc.).
    Identified,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_serialize_deserialize() {
        let variants = [
            DataClassification::Anonymous,
            DataClassification::Pseudonymized,
            DataClassification::Identified,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let back: DataClassification = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*variant, back);
        }
    }

    #[test]
    fn all_variants_exist() {
        // Ensure all three variants are distinct.
        let a = DataClassification::Anonymous;
        let p = DataClassification::Pseudonymized;
        let i = DataClassification::Identified;
        assert_ne!(a, p);
        assert_ne!(p, i);
        assert_ne!(a, i);
    }
}
