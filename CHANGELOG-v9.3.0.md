# v9.3.0

Minor release. One addition and one packaging fix. No computed value changes.

## Fixed — sibling crates now require the sibling version they use

Each workspace crate declares the other Vedaksha crates it depends on with a
version requirement, and through v9.2.0 those requirements had been left at
`9.0.0`. On crates.io that is `^9.0.0`: any 9.x sibling satisfies it.

At v9.2.0 that stopped being harmless. `vedaksha-astro` 9.2.0's
`moon_riseset::moon_equatorial` calls `coordinates::ecliptic_to_equatorial_deg`
and `CelestialFrame::true_obliquity`, both added in `vedaksha-ephem-core`
9.2.0. A project whose lockfile already held `vedaksha-ephem-core` 9.1.x and
upgraded only `vedaksha-astro` could resolve a combination that does not
compile. Upgrading the crates together, which `cargo update` does, was never
affected.

From 9.3.0 every published crate requires its siblings at exactly the release
version (`^9.3.0`), and CI fails if any requirement falls behind the workspace
version. The 9.2.0 crates on crates.io cannot be amended; if you use
`vedaksha-astro` 9.2.0, make sure `vedaksha-ephem-core` is also at 9.2.0, or
move everything to 9.3.0.

## Added

- **`vedaksha_vedic::combustion::combustion_orb_deg(planet, is_retrograde)`** —
  the combustion orb in degrees (`None` for the Sun, Rahu and Ketu).
  `combustion_state` reads this function and nothing else, so a caller that
  grades combustion more finely, or sizes a time window from the orb, stays
  consistent with the engine. `DeeplyCombust` begins at a third of the orb. A
  test holds every published orb to the boundaries `combustion_state` actually
  applies.

## Compatibility

Additive. The only behavioural difference a build can see is that Cargo will no
longer pair a 9.3.0 crate with an older sibling.
