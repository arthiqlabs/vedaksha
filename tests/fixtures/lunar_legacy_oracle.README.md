# Lunar legacy oracle fixture

**File:** `lunar_legacy_oracle.bin`
**Format:** 10 000 rows of 7 little-endian `f64`s: `(jd, x, y, z, vx, vy, vz)`.
**Frame:** geocentric J2000 mean ecliptic; position in km, velocity in km/day.
**Grid:** 10 000 evenly spaced JDs from `625673.5` (≈ −3000-01-01 TT) to `2816787.5` (≈ +3000-01-01 TT).
**Generated:** 2026-05-09.

## Provenance and clean-room status

This fixture contains numerical outputs of the **pre-rederivation** Vedākṣha lunar implementation, which derived structurally from `ytliu0/ElpMpp02` (GPL-3.0). It is committed for use as a Tier-3 regression oracle by the clean-room re-derivation that replaces that implementation.

Numerical values cross the clean-room firewall as facts (Feist v. Rural — uncopyrightable). No source code, structural information, or attribution from the contaminated implementation crosses. The capture binary that produced this file is part of the contaminated implementation and is deleted in the same commit that quarantines the rest.

The clean-room implementation must agree with this fixture to within 0.1″ angular over the captured grid (Tier-3 acceptance criterion). Tier 1 (JPL Horizons, ≤1″) is the truth bar; this fixture is informational regression coverage only.

See `docs/superpowers/specs/2026-05-09-elp-mpp02-rederivation-design.md` for the full design.
