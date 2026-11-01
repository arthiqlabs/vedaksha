# v10.1.1

Patch release. One correction to v10.1.0's numbers, and one change to dasha
bounds at the last few binary digits. No ephemeris value changes.

## Corrected

- **`graha_ingresses_by_longitude` and the true node.** v10.1.0 stated that
  its instants agree with `graha_ingresses` within 0.44 ms for the nodes. That
  figure came from two entries per node and was wrong for the true node.
  Measured over 1900 to 2100, the mean node agrees within 0.040 ms (129
  entries). The true node (`Body::TrueNode`, `Body::TrueSouthNode`, 137
  entries each) agrees within 215 ms and 277 ms, and 121 and 122 entries are
  above 0.44 ms. Count and rashi agree on every entry. The planets are
  unchanged at 0.04 ms.

  The cause is the true node's longitude, not either search. Evaluated a few
  seconds apart it scatters by up to 7.4e-4 arcsecond, which at the node's
  slowest crossing rates spans about 0.3 s. Neither function's instant is
  closer than that to a smooth crossing, and `graha_ingresses`'s Newton
  refinement stops at its pass limit there instead of converging. The doc
  comment now states the bound per node body, and a test holds it.

## Changed answers, Vedic

- **Dasha sub-periods tile their parent to the bit (Vimshottari, Ashtottari,
  Yogini).** The last sub-period of each level ended where the running sum of
  its siblings' lengths put it, which could fall a few units in the last place
  short of its parent's end. It now ends exactly at its parent's end, at every
  level and in every maha. The largest move is a few units in the last place
  of a Julian Day, about 0.1 ms. Rounded to the second, that moved one
  boundary in about one chart in 4,000 in a consumer's measurement.

## Compatibility

No type, field or signature changes.
