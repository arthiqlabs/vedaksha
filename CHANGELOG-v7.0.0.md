# Vedaksha v7.0.0

**Every lunar node value changes, on every surface.** Rahu and Ketu were wrong — not by a
refinement, by tens of degrees — and separately the three node methods were not in the same
reference frame. Both are fixed here. If you have stored anything containing a node position,
recompute it.

Nothing else moves. Planets, houses, panchanga, dashas and ayanamsas are byte-identical to
6.1.0.

Major, not minor, for two reasons given under Migration below: one published function now
means something different, and one provider call now fails where it used to succeed. Neither
is caught by the compiler, so the version is doing the work instead.

---

## 1. The nodes were being treated as bodies

`AnalyticalProvider` answered `Body::MeanNode` with a synthetic state vector — a unit vector
one AU from the barycentre, pointing along the node. The apparent-position pipeline then did to
it exactly what it does to a planet: subtracted the Earth's barycentric position, iterated
light-time, and precessed the result. What came back was the direction from the Earth to a
fictitious point one AU away, which is not the node and is not close to it.

At J2000 the mean node was reported as **200.505°**, where the mean node is at **125.045°**.

| epoch | reported | actual | error |
|---|---|---|---|
| 1900 | 268.861° | 259.156° | 9.70° |
| 1950 | 326.205° | 12.113° | 45.91° |
| 2000 | 200.505° | 125.045° | **75.46°** |
| 2020 | 42.089° | 98.244° | 56.16° |

Motion was wrong in magnitude *and sign*: **+0.384°/day, reported as direct**, against an
actual −0.053°/day retrograde. At 2020 it reported −19.2°/day, because the fictitious point
passes close to the Earth and its apparent motion diverges. Reported latitude drifted off the
ecliptic, and `distance` carried the meaningless Earth-to-unit-vector separation.

This was present since **v2.0.0** and reached every surface that selects a node through
`Body`: the `compute_natal_chart`, `compute_transits` and `search_transits` MCP tools, the WASM
build, and the Python package. Callers of `nodes::mean_node` and friends were never affected —
those functions were always correct.

### What changed

Nodes are directions, not places. They have no state vector, no light-time, no aberration and
no distance, so they no longer go near the provider: `coordinates` resolves them from
`nodes::node_longitude` before any provider is consulted, and `AnalyticalProvider` now returns
`BodyNotAvailable` rather than inventing a vector.

The one correction a node does take is nutation in longitude. The node functions return the
*mean* equinox of date and every other longitude in a chart is referred to the *true* equinox,
so `CelestialFrame` now carries Δψ for them. Reported node longitudes differ from the node
functions by that term and nothing else — at most 17.2″.

## 2. The three node methods were in different frames

`mean_node` and `true_node` were referred to the mean equinox of date. `true_node_osculating`
was referred to J2000. All three are selectable through one `Body` enum, at call sites that
mention no frame at all, so switching methods silently switched frames.

The separation grows at the precession rate, without bound:

| epoch | osculating − true |
|---|---|
| 1900 | 5463″ (1.52°) |
| 1950 | 2657″ |
| 2000 | 114″ |
| 2020 | −833″ |
| 2050 | −2383″ |

The documented remedy — "sidereal consumers should subtract their chosen ayanamsa" — was not
sufficient for the J2000 function, because an ayanamsa is defined against the equinox of date.
Every Vedic consumer of `true_node_osculating` was leaving the precession term in the result,
and it is largest for historical birth data.

### What changed

All three methods now return the **mean ecliptic and equinox of date**. Selecting between them
changes the method and never the frame.

The osculating node's crossing is evaluated against the of-date plane, rather than by rotating
the J2000 answer into of-date coordinates — those are different quantities. Going from the
J2000 ecliptic to the ecliptic of date moves the reference *plane*, not only the equinox, and
because the Moon's orbit is inclined 5.145°, a plane tilt of ≈47″/century displaces the
crossing point along the orbit by roughly `cot(i)` ≈ 11× that. The full frame term is therefore
≈**1.54°/century**, not the 1.40° that general precession in longitude alone predicts.

The J2000 value survives as **`true_node_osculating_j2000`**, documented as being for
comparison against JPL Horizons' `OM` — which is published in that frame — and not for chart
use. It still tracks Horizons to 0.001°.

With the frame term removed, the osculating node and the Meeus 5-term series differ by at most
**0.297°** across 1900–2100 sampled every 3 days. That residual is the real libration of the
instantaneous orbital element about a smoothed one, and it is bounded, which is the property
that distinguishes it from a frame difference.

## Why the tests did not catch either

Worth stating plainly, because both guards reported healthy.

The pipeline guard asserted that a node's state vector contained **finite numbers**. That
stayed true through a 75° error, because nothing in it ever compared a longitude to anything.

The frame guards asserted that the osculating and Meeus nodes agreed within **0.5°**, at epochs
running from J2000 to 2026. J2000 is the single epoch where a frame difference between them
vanishes, and at 26 years out the frame term is 0.36° — still under the bound. The tolerance
was sized from the genuine method difference, which is the reasonable thing to do; what it
could not do is tell a bounded difference from an unbounded one. Left alone, that suite would
have begun failing on its own around 2035.

`crates/vedaksha-ephem-core/tests/node_frame.rs` replaces both. It pins the reported longitude
against the node functions, the draconic regression rate by displacement over a fixed baseline,
the of-date/J2000 relationship at 1900, 2050 and 2100, and sweeps 1900–2100 at 3-day steps for
the frame bound. Each test was checked against a reconstruction of the defect it targets.

## Migration

- **`true_node_osculating` returns a different frame.** Same signature, same name, different
  meaning: of-date rather than J2000. If you were applying your own precession correction
  downstream, remove it. If you need the old value, call `true_node_osculating_j2000`.
- **`EphemerisProvider::compute_state` on `Body::MeanNode`, `Body::TrueNode` or
  `Body::TrueNodeOsculating` now returns `Err(BodyNotAvailable)`.** Use
  `coordinates::apparent_position` or `nodes::node_longitude` instead.
- **Node `distance` is reported as `0`** rather than a meaningless value. Node latitude is
  exactly `0`.
- Anyone pinned to `vedaksha = "6"` keeps the old behaviour until they move to 7 deliberately.
  That is why this is a major.

## Validation

1,064 tests passed, 0 failed, 0 ignored across 27 suites, in release with `--include-ignored`
and `VEDAKSHA_REQUIRE_FIXTURES=1` so a missing kernel or oracle fixture fails rather than
skips. Clippy at `-D warnings`, format check, and the four generator drift checks — VSOP87A,
ELP/MPP02, ayanamsha re-derivation and the VizieR catalogue re-query — all green. The Python
parity fixture was regenerated and the WASM build reproduces it byte-for-byte, which is the
evidence the fix reaches the packaged surface rather than only the source.
