# v10.1.0

Minor release. One Vedic answer changes, for every chart: the sub-periods of
the first maha dasha. One function is added. No ephemeris value changes.

## Changed answers, Vedic

- **The first maha's sub-periods (Vimshottari, Ashtottari, Yogini).** A maha
  always has its full length, and its sub-periods divide that full length in
  their fixed shares, from the maha's notional start: the birth less the
  elapsed share of its full length, where the elapsed share is the share of
  its nakshatra the Moon has traversed. The three builders divided only the first maha's remaining
  balance, from the birth. So every sub-period of the first maha was too
  short by the factor `initial_balance`, the first one always opened at the
  birth as the maha lord's own, and the sub-period named as running at birth
  was wrong whenever the elapsed share exceeded the maha lord's own share.
  The error reached every lower level. It has been there since v2.0.0.

  The first maha's sub-periods now run over its full length from its
  notional start, and all of them are kept: those with `end_jd <= birth_jd`
  ended before birth, and the one running at birth starts at or before it,
  with its true start. The maha itself still opens at the birth with
  `initial_balance` of its length, so its sub-periods sum to its full
  length, not to its `duration_days`. `initial_balance`, every maha boundary
  and every later maha are unchanged. The MCP `compute_dasha` tool, the WASM
  export and the Python binding all serve the corrected tree. See
  `DATA_PROVENANCE.md`, Fix 15.

  Example, Moon at 100° sidereal (Pushya, Saturn maha, elapsed share 0.5):
  the antar running at birth was Saturn-Saturn, starting at the birth. It is
  Saturn-Venus, which began 983.13125 days before the birth.

## Added

- **`khagola::graha_ingresses_by_longitude`.** `graha_ingresses` for a
  callback that returns the tropical longitude alone, with no daily motion.
  It walks the same grid as `graha_ingresses`, steps from the same
  `ingress_speed_bound_deg_per_day`, reads the direction of a crossing from
  the rashi indices, and finds the instant inside the one-day bracket by a
  safeguarded Illinois search to 1e-9 day. Measured on the analytical
  positions over nine spans and every graha (332 entries), it returns the
  same entries as `graha_ingresses` with instants within 0.04 ms for the
  planets and 0.44 ms for the nodes. They are not bit-identical. For the Moon
  over a year it reads the callback 2,121 times, where `graha_ingresses`
  reads 1,605 times a callback that must also give the motion; a source whose
  motion is a central difference spends about 4,800 position evaluations on
  the latter. `graha_ingresses` and its results are unchanged.

## Compatibility

No type, field or signature changes. A consumer that assumed the first maha's
sub-periods start at the birth, or tile its `duration_days`, needs to skip
the ones that end at or before `birth_jd` (the first maha's `start_jd`).
