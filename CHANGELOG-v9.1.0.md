# v9.1.0

Additive release. One new public entry point, a behaviour-preserving refactor
behind it, and four documentation defects corrected -- two of which were
pre-existing and became actively misleading once the new surface landed.

No computed value changes in this release. Every number Vedaksha served at
v9.0.0 it serves identically at v9.1.0; see "How that was verified" below.

## Added

### A TT-addressed analytical position entry point

`ecliptic_position_tt(provider, body, jd_tt)` in `vedaksha-ephem-core`, with a
C-ABI export (`vk_analytical_position_tt`) and a Python method
(`Vedaksha.analytical_position`).

Every existing analytical entry point takes UT1 and converts to TT internally,
and the Delta T value is not exposed anywhere. For anyone validating Vedaksha
against an independent ephemeris, that entangles two error sources: the
analytical theories' own truncation error, and the divergence between two
independently computed Delta T extrapolations. Outside the measured-Delta T
record the residual is the sum of the two, with no way to apportion it.

Addressing both sides in TT removes the time scale from the comparison
entirely, so the only remaining limit is each side's coverage span. This is the
analytical sibling of `spk_state` / `Vedaksha.state_vector`, which has always
been TDB-addressed for the same reason.

Two deliberate omissions, both documented in the code:

- **No MCP tool.** Houses and the ascendant are functions of Earth's rotation
  and therefore genuinely need UT1, so a TT-addressed *chart* is ill-defined.
  This entry point is body positions only.
- **No TT sibling for `apparent_position`.** Its daily-motion term is a
  +/-0.5-day central difference, and a TT-space difference is a different
  quantity from the UT1-space one (they differ by the local slope of Delta T).
  A TT central difference is well-defined and may be added later on its own
  merits; it is not needed for position comparison and is not guessed at here.

## Changed

`coordinates.rs` now converts UT1 to TT exactly once, at each public UT1
function's own boundary, and passes TT inward. Previously the conversion
happened at eight sites, including a private helper that re-converted a UT1
argument its caller had separately already converted to build the frame.

This is internal. `apparent_position`, `ecliptic_position` and
`apparent_positions` keep their signatures and their UT1 contract.

## Fixed

- **The C-ABI error-code contract did not match the code.** The documentation
  said a body the analytical tier does not model returns `ERR_UNKNOWN_BODY`,
  giving Pluto as the example. Pluto in fact resolved to a valid `Body`, then
  returned `BodyNotAvailable`, which flattened to `ERR_COMPUTE` -- surfacing as
  "ephemeris computation failed (out of range?)". A caller sweeping bodies
  would have gone hunting a date-range bug that does not exist. The new export
  now maps `BodyNotAvailable` to `ERR_UNKNOWN_BODY`, and the docs match.

- **`moon_longitude_at_j2000_matches_jpl_horizons` mislabelled its oracle's
  time scale.** The test called its anchor "JD 2451545.0 TT"; the generator
  that produced the value records `"time_scale": "UT (Horizons JDUT)"`. The
  generator is right. Measured: the anchor evaluated as UT1 lands 0.18 arcsec
  from the oracle, evaluated as TT it lands 32.15 arcsec away -- matching the
  ~35 arcsec UT/TT mixup signature that the generator's own `verify_anchor()`
  warns about. Pre-existing and numerically harmless until now, but a TT entry
  point now sits twenty lines below it, so anyone sanity-checking the new
  surface against that anchor would have found 32 arcsec and wrongly concluded
  the new code was broken.

- **`scripts/generate_horizons_oracle.py` cited a parameter this release
  deleted.** It documented the time-scale contract in terms of
  `compute_ecliptic_with_frame`'s `jd_ut` parameter and the conversion inside
  it; after the refactor the parameter is `jd_tt` and the conversion has moved
  outward. This file is the provenance record for the 24,350-row fixture behind
  the accuracy claim, so a stale citation there degrades a guardrail rather
  than a comment.

- A stale rustdoc heading (`Two surfaces` with three bullets under it).

## How that was verified

The refactor's only real risk is a unit mismatch silently moving every
published number, so it was checked two independent ways:

- `analytical_bit_digest` reproduces its existing pinned digest over 21,915
  rows, unchanged and **not** re-pinned.
- Base and this release were built into separate clean trees and the full MCP
  natal-chart fixture generated from each: **byte-identical**, across all
  bodies, houses, panchanga and aspects.

The second check is the load-bearing one. The bit-digest covers only
`apparent_position`; the two lunar-series digests call `elp_mpp02` directly and
never enter `coordinates.rs` at all, so they cannot move under this refactor
and are not evidence for it. `ecliptic_position` and `apparent_positions` have
no digest coverage, and the fixture comparison is what actually exercises them.

`analytical_oracle` is unchanged at mean 0.239 arcsec, max 1.896 arcsec
(Neptune, 2007-Feb-09).

New tests: an absolute check of `ecliptic_position_tt` against the pinned JPL
Horizons Moon anchor (this is the one that constrains a value -- its tolerance
discriminates a time-scale mixup, which lands outside the band), a bit-identity
change-detector against the UT1 path, a test that the TT path applies no
conversion, and Python tests covering the FFI round trip and the Pluto error
path.

Test counts re-measured at this commit on aarch64 with `data/de440s.bsp`
present so the SPK-backed tests actually execute: perPush 1147, full 1155, 0
failures.

## Known issues

Carried forward from v8.1.0, unchanged and not introduced here: the bit-digest
fixtures are architecture-dependent, because the SIMD trigonometric kernel they
fingerprint maps to one AVX2 register on x86_64 and two NEON registers on
aarch64. Since v9.0.0 the digests are pinned per architecture and Full
Validation is green on both. Accuracy is unaffected and separately gated by the
Horizons oracle, which passes on both architectures -- measured identical to
three decimal places across the two (n=13,815, mean 0.239 arcsec, max 1.896
arcsec on each).
