# v10.2.0

Minor release. Three answers change: the true node's longitude, in its last
milliarcseconds; the ingress entries the three ingress walks return at a
station on or near a rashi edge; and dasha sub-period bounds, in their last
binary digits. One figure stated in v10.1.0 is corrected.

## Changed answers, ephemeris

- **The true node takes the lunar series' own velocity.** `Body::TrueNode`,
  `Body::TrueSouthNode`, `Body::TrueNodeOsculating`,
  `Body::TrueSouthNodeOsculating`, and the functions
  `nodes::true_node_osculating`, `true_node_osculating_j2000`,
  `south_node_true` and `south_node_osculating` (all the osculating node)
  were computed from a
  velocity found by differencing three lunar positions 0.001 day apart. The
  series' evaluation noise, divided by that step, made the node's longitude
  scatter by up to 7.4e-4 arcsecond between instants seconds apart. The
  velocity is now the analytic derivative of the ELP/MPP02 series, which
  carries no differencing step. Measured 2026-11-01:
  - the scatter falls to under 4.2e-7 arcsecond, about 1,000 times less;
  - over 1900 to 2100 (197,430 instants) the value moves by at most 1.25e-3
    arcsecond and on average 7.1e-5 arcsecond in the ecliptic of date, and
    by at most 1.22e-3 arcsecond in the J2000 ecliptic;
  - one lunar evaluation replaces three, so a chart evaluates the lunar
    series 6 times instead of 12.

  The node's agreement with JPL Horizons, measured in the J2000 ecliptic at
  0.6 arcsecond at most over 1900 to 2100, can move by no more than the
  1.22e-3 arcsecond change in that frame.
  The five Horizons dates in the test suite still pass at 0.001 degree. The
  dense Horizons comparison was not re-run for this release.

## Changed answers, Vedic

- **`graha_ingress` and `graha_ingresses` at a station near a rashi edge.**
  Each crossing is refined by Newton's method from the daily motion, starting
  at the first of the two daily samples that bracket it. Near a station the
  rate there can be close to zero or of the wrong sign, and the step left the
  bracket. The instant was returned as it stood. A vanishing rate ended the
  walk. Both now fall back to a search inside the bracket, the one
  `graha_ingresses_by_longitude` uses. Every instant Newton already placed
  inside its bracket is unchanged to the bit.

  Over 1900 to 2100, checked against every rashi change seen by sampling
  every 0.02 day (nodes) or 0.05 day (Mercury to Saturn):
  - Mercury: v10.1.0 returned 11,687 entries where there are 2,967. The
    extra 8,720 were one instant, JD 2420104.0858 in 1913, repeated: Mercury
    stations on the Tula-Vrishchika edge there and spends 4.75 hours back in
    Tula.
  - True node, with this release's node: v10.1.0's walk would have returned
    one entry centuries outside the requested range and one in the wrong
    place, missed two, and for Ketu stopped 10 entries early. With the old
    node the two walks agreed on every entry; that was not checked against
    the sampling.
  - Venus, Mars, Jupiter and Saturn were already exact.

  After the fix, both walks return exactly the sampled entries for every
  graha.

- **`graha_ingress`, `graha_ingresses` and `graha_ingresses_by_longitude`
  find a dip across an edge and back between two daily samples.** No rashi
  index changes across such a dip, so every version through v10.1.0 stepped
  over it unless a daily sample happened to fall inside. Each walk now looks
  for the extremum the dip needs: where the step between consecutive samples
  reverses near an edge, it searches the two days around for the extremum
  and, if it passes the edge, reports both crossings. The 1913 Mercury dip
  (0.198 day) is the only dip under a day that sampling every 0.05 day finds
  for any graha from 1900 to 2100; it is now found from any seed, where a
  walk seeded anywhere from JD 2420070 to 2420100 used to miss it. The search
  assumes one extremum at most in two days. Consecutive stations are at least
  19.8 days apart for Mercury to Saturn over that span, but the true node has
  pairs closer than 0.02 day, where it only wiggles, so a dip inside such a
  wiggle can still be missed; walks seeded on three grid phases gave the same
  137 true-node entries. The cost is one more read per entry: for the Moon
  over a year, 1,766 reads against 1,605, and 2,282 against 2,121.

  These functions are Rust API only. No MCP tool, WASM export or Python call
  uses them.

- **Dasha sub-periods tile their parent to the bit (Vimshottari, Ashtottari,
  Yogini).** The last sub-period of each level ended where the running sum of
  its siblings' lengths put it, which could fall a few units in the last place
  short of its parent's end. It now ends exactly at its parent's end, at every
  level and in every maha. The largest move is a few units in the last place
  of a Julian Day, about 0.1 ms. Rounded to the second, that moved one
  boundary in about one chart in 4,000 in a consumer's measurement.

## Corrected

- **`graha_ingresses_by_longitude` against `graha_ingresses` for the nodes.**
  v10.1.0 stated the two agree within 0.44 ms for the nodes, measured on two
  entries per node. For the true node it was wrong: over 1900 to 2100 they
  differed by up to 277 ms. With this release's node and walk, they agree
  within 0.20 ms for the true node and 0.040 ms for the mean node and the
  planets, with the same entries.

## Compatibility

No type, field or signature changes.
