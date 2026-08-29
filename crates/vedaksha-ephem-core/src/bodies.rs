// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha — Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net

//! Celestial body identifiers and metadata.

/// Celestial bodies supported by the ephemeris engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Body {
    /// The Sun.
    Sun,
    /// The Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Earth-Moon Barycenter.
    EarthMoonBarycenter,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// Mean ascending lunar node.
    MeanNode,
    /// True ascending lunar node from osculating orbital elements. Derived
    /// from Moon position/velocity via ELP/MPP02. The underlying
    /// osculating-node method — shared by this variant and
    /// [`Self::TrueNodeOsculating`], which differ only by the documented
    /// frame term — is validated to 0.6″ max vs JPL DE441 over 1900–2100
    /// (measured 2026-07-16 across 2,435 epochs, on the J2000-frame
    /// evaluation used for that comparison). As of this release, this
    /// variant is numerically identical to [`Self::TrueNodeOsculating`] —
    /// kept as a separate variant for API stability. Previously a 5-term
    /// Meeus approximation; see `DATA_PROVENANCE.md`'s "Fix 2" entry for
    /// that history.
    TrueNode,
    /// True ascending lunar node from osculating orbital elements. Derived
    /// from Moon position/velocity via ELP/MPP02. Suitable for KP sub-lord
    /// work. Shares the same underlying osculating-node method as
    /// [`Self::TrueNode`] — see that variant's doc comment for the validated
    /// accuracy figure and which evaluation it was measured on.
    TrueNodeOsculating,
    /// Mean descending lunar node (Ketu) — [`Self::MeanNode`] + 180°.
    MeanSouthNode,
    /// True descending lunar node (Ketu) — [`Self::TrueNode`] + 180°. As of
    /// this release, numerically identical to
    /// [`Self::TrueSouthNodeOsculating`].
    TrueSouthNode,
    /// Osculating descending lunar node (Ketu) — [`Self::TrueNodeOsculating`]
    /// + 180°.
    TrueSouthNodeOsculating,
}

impl Body {
    /// Returns the DE441 component index for this body, or `None` if not
    /// directly available in JPL DE441.
    ///
    /// Indices: Mercury=0, Venus=1, EMB=2, Mars=3, Jupiter=4, Saturn=5,
    /// Uranus=6, Neptune=7, Pluto=8, Moon=9, Sun=10.
    #[must_use]
    pub fn de441_component_index(&self) -> Option<usize> {
        match self {
            Self::Mercury => Some(0),
            Self::Venus => Some(1),
            Self::EarthMoonBarycenter => Some(2),
            Self::Mars => Some(3),
            Self::Jupiter => Some(4),
            Self::Saturn => Some(5),
            Self::Uranus => Some(6),
            Self::Neptune => Some(7),
            Self::Pluto => Some(8),
            Self::Moon => Some(9),
            Self::Sun => Some(10),
            Self::MeanNode
            | Self::TrueNode
            | Self::TrueNodeOsculating
            | Self::MeanSouthNode
            | Self::TrueSouthNode
            | Self::TrueSouthNodeOsculating => None,
        }
    }

    /// Returns the NAIF target ID for this body.
    ///
    /// Mercury=1, Venus=2, EMB=3, Mars=4, Jupiter=5, Saturn=6,
    /// Uranus=7, Neptune=8, Pluto=9, Moon=301, Sun=10.
    /// The six lunar nodes are not NAIF bodies and take placeholder IDs
    /// 10000-10005; nothing resolves them against a kernel.
    #[must_use]
    pub fn naif_id(&self) -> i32 {
        match self {
            Self::Mercury => 1,
            Self::Venus => 2,
            Self::EarthMoonBarycenter => 3,
            Self::Mars => 4,
            Self::Jupiter => 5,
            Self::Saturn => 6,
            Self::Uranus => 7,
            Self::Neptune => 8,
            Self::Pluto => 9,
            Self::Moon => 301,
            Self::Sun => 10,
            Self::MeanNode => 10000,
            Self::TrueNode => 10001,
            Self::TrueNodeOsculating => 10002,
            Self::MeanSouthNode => 10003,
            Self::TrueSouthNode => 10004,
            Self::TrueSouthNodeOsculating => 10005,
        }
    }

    /// Returns the number of Cartesian components for state vectors (always 3).
    #[must_use]
    pub fn num_components(&self) -> usize {
        3
    }

    /// Returns a human-readable name for this body.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mercury => "Mercury",
            Self::Venus => "Venus",
            Self::EarthMoonBarycenter => "Earth-Moon Barycenter",
            Self::Mars => "Mars",
            Self::Jupiter => "Jupiter",
            Self::Saturn => "Saturn",
            Self::Uranus => "Uranus",
            Self::Neptune => "Neptune",
            Self::Pluto => "Pluto",
            Self::MeanNode => "Mean Node",
            Self::TrueNode => "True Node",
            Self::TrueNodeOsculating => "True Node (Osculating)",
            Self::MeanSouthNode => "Mean South Node",
            Self::TrueSouthNode => "True South Node",
            Self::TrueSouthNodeOsculating => "True South Node (Osculating)",
        }
    }

    /// Returns `true` if DE441 data is available for this body.
    #[must_use]
    pub fn has_de441_data(&self) -> bool {
        self.de441_component_index().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mercury_is_component_0() {
        assert_eq!(Body::Mercury.de441_component_index(), Some(0));
    }

    #[test]
    fn sun_is_component_10() {
        assert_eq!(Body::Sun.de441_component_index(), Some(10));
    }

    #[test]
    fn moon_is_component_9() {
        assert_eq!(Body::Moon.de441_component_index(), Some(9));
    }

    #[test]
    fn nodes_have_no_de441_data() {
        assert!(!Body::MeanNode.has_de441_data());
        assert!(!Body::TrueNode.has_de441_data());
        assert!(!Body::TrueNodeOsculating.has_de441_data());
    }

    #[test]
    fn all_planets_have_de441_data() {
        let planets = [
            Body::Sun,
            Body::Moon,
            Body::Mercury,
            Body::Venus,
            Body::EarthMoonBarycenter,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
            Body::Pluto,
        ];
        for body in &planets {
            assert!(
                body.has_de441_data(),
                "{} should have DE441 data",
                body.name()
            );
        }
    }

    #[test]
    fn all_bodies_have_3_components() {
        let all = [
            Body::Sun,
            Body::Moon,
            Body::Mercury,
            Body::Venus,
            Body::EarthMoonBarycenter,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
            Body::Pluto,
            Body::MeanNode,
            Body::TrueNode,
            Body::TrueNodeOsculating,
        ];
        for body in &all {
            assert_eq!(
                body.num_components(),
                3,
                "{} should have 3 components",
                body.name()
            );
        }
    }

    #[test]
    fn body_names_correct() {
        assert_eq!(Body::Sun.name(), "Sun");
        assert_eq!(Body::Moon.name(), "Moon");
        assert_eq!(Body::Mercury.name(), "Mercury");
        assert_eq!(Body::EarthMoonBarycenter.name(), "Earth-Moon Barycenter");
        assert_eq!(Body::MeanNode.name(), "Mean Node");
        assert_eq!(Body::TrueNode.name(), "True Node");
    }

    #[test]
    fn naif_ids_correct() {
        assert_eq!(Body::Mercury.naif_id(), 1);
        assert_eq!(Body::Moon.naif_id(), 301);
        assert_eq!(Body::Sun.naif_id(), 10);
    }
}
