# v9.5.0

Minor release. The Moon's light-time-corrected reads cost about half as many
ELP/MPP02 evaluations with no output bit moved, and a new Moon read returns
an analytic longitude speed. Additive.

## Changed — fewer lunar series evaluations, identical answers

The light-time iteration for the Moon (`vedaksha-ephem-core`, `coordinates`)
did two kinds of redundant work:

- **It evaluated the series twice at the same instant.** For the Moon the
  light-time correction is about 1.3 s, and its change between iterations is
  usually above the convergence threshold but far below the spacing of
  doubles near a modern Julian Day, so the next retarded instant was usually
  the same double as the last. The iteration now returns the evaluation it
  already has. That is exact: the Moon's vector depends only on the instant.
- **Its first evaluation was used only for its length.** Where the provider
  offers a distance-only read (`EphemerisProvider::moon_distance`, see Added),
  the first step now uses it. A guard proves, from the distance's stated error
  bound, that the iteration then takes the same path to the same answer, and
  falls back to a full first evaluation where it cannot.

Full series evaluations per read, measured over 1900 to 2100:

| Read | v9.4.1 | v9.5.0 |
|---|---|---|
| `ecliptic_position(Moon)` | 2.756 | 1.003, plus 1 distance-only |
| `apparent_positions(&[Moon])` | 6.0 | 3.015, plus 3 distance-only |

A distance-only evaluation runs the distance series alone, about 40% of a
full one. Every output bit of `ecliptic_position`, `ecliptic_position_tt`,
`apparent_position` and `apparent_positions` is unchanged, the
`apparent_positions` speed included: checked against a verbatim copy of the
v9.4.1 iteration at 4,000 instants, and against a table of v9.4.1's own
output captured before the change.

## Added

- **`coordinates::moon_apparent_position(provider, jd)`**: the Moon's
  apparent position with `longitude_speed` from the analytic derivative of the
  ephemeris (the provider's own velocity, the light-time factor `1 − dτ/dt`,
  and the rate at which the precession-nutation frame turns), at one
  position-and-velocity evaluation where `apparent_position` makes three
  light-time-corrected reads. Its position fields are `ecliptic_position`'s to
  the bit. Its speed is the derivative itself, so it differs from
  `apparent_position`'s ±0.5-day central difference by that difference's
  truncation error: worst 8.5e-3 °/day measured, with no bias. Against a
  Richardson-extrapolated difference it agrees to 1.3e-7 °/day. Per day of
  TT, where `apparent_position`'s speed is per UT1 day; the two differ by the
  rate of change of ΔT, below 1e-7 relative.
- **`EphemerisProvider::moon_distance(jd) -> Option<f64>`**, with a default
  of `None`, so an existing provider (SPK included) behaves exactly as
  before. `AnalyticalProvider` implements it from the ELP/MPP02 distance
  series (`elp_mpp02::elp_geocentric_distance`), and `CachingProvider`
  forwards it. An implementation must meet the relative error bound
  **`jpl::MOON_DISTANCE_REL_TOL`** (1e-12; the analytical one measures
  5.3e-16) and return `None` wherever `moon_position` would error.

## Compatibility

Additive. No computed value moves. A consumer moving a Moon speed from
`apparent_position` to `moon_apparent_position` sees that speed shift by up
to about 1e-2 °/day, toward the true rate.
