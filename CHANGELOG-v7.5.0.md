# Vedaksha v7.5.0

**`Body::TrueNode` now returns the osculating node — the standard orbital-element
definition of the Moon's true ascending node — instead of a 5-term Meeus series.**
Vedaksha has shipped two computations of the Moon's true ascending node since
v2.0.0: a 5-term Meeus approximation (previously `Body::TrueNode`) and a
computation from the Moon's actual instantaneous orbital elements
(`Body::TrueNodeOsculating`). This release switches which of the two
`Body::TrueNode` names. The osculating computation itself is unchanged, and has
been independently validated to 0.6 arcsecond against JPL Horizons since
2026-07-16 — but that figure is evidence the osculating method is computed
correctly, not evidence that switching `Body::TrueNode` to it is an accuracy
improvement. See "What changed, and what didn't" below for why those are
different claims.

Values move only on `Body::TrueNode` and `Body::TrueSouthNode`. Nothing else
changes directly — though anything derived from these two values (their
reported speed, and any aspect calculation involving them) moves as a
consequence.

---

## What changed, and what didn't

`Body::TrueNode` and `Body::TrueSouthNode` now compute exactly what
`Body::TrueNodeOsculating` and `Body::TrueSouthNodeOsculating` already computed.
All four `Body` variants remain — no `Body` enum variant is added, removed, or
renamed, and every existing call site that names a `Body` variant continues to
compile unchanged. What changes is the *value* those two variants return: up
to ~0.35° of difference from the previous truncated-series output, per this
project's own existing regression tests (`osculating_node_close_to_meeus_true_node`,
`osculating_node_multi_epoch_sanity`), which already bounded and documented
that gap before this release.

**This is a change of which quantity is reported, not a correction of an
error.** The osculating node and the Meeus 5-term series are two different,
legitimate orbital-element definitions that librate against each other by a
bounded ~0.3° — this project's own test comments in `nodes.rs` describe that
gap as a bounded libration between two different definitions, not as an
accuracy defect in either one. The osculating element is the definition most
jyotish software means by "the true node," which is the actual reason to make
it the default.

Separately from that quantity switch: the osculating computation itself
(`true_node_osculating` / `true_node_osculating_j2000`) has its own,
independent validation — 0.6″ max against JPL Horizons, measured 2026-07-16
across 2,435 epochs spanning 1900–2100. That number says the osculating
computation correctly computes the osculating orbital element; it says nothing
about whether the osculating node is a "better answer" than the Meeus series,
because they answer different questions. The Meeus series carries its own,
separately documented ~0.09° truncation residual against fuller lunar theory
(it uses 5 of the many terms in the full series) — a real, bounded error in
its own right, but not one this release's Horizons comparison speaks to at
all, in either direction.

The 5-term Meeus formula is not deleted. It survives privately inside
`crates/vedaksha-ephem-core/src/nodes.rs`, used only by that module's own
tests as an independently-transcribed method to cross-check the osculating
computation against — the same class of guard that caught the v7.0.0
node-frame defect, where two node methods silently drifted into different
reference frames.

## Why now

Both computations have existed side by side, undifferentiated in status,
since the very first commit in this codebase's history. Nothing in the
source, tests, or prior release notes ever stated which of the two
`Body::TrueNode` should report. This release resolves that ambiguity by
naming the osculating element — the standard orbital-mechanics definition,
and the one most jyotish software means by "true node" — as the one
`Body::TrueNode` reports.

## Measured impact

Regenerating the Python conformance fixture against this change shows the
concrete size of the shift at two real epochs (verified against
`bindings/python/tests/conformance/native_fixture.json`, comparing this
release to the pre-change fixture):

- **J2000** (`julian_day = 2451545.0`): `Body::TrueNode` moved from
  123.92226840680077° to 123.95402964225244° — a shift of **+114″**
  (0.0318°).
- **1985** (`julian_day = 2446270.104166667`): `Body::TrueNode` moved from
  44.880992595185965° to 44.925934350750175° — a shift of **+162″**
  (0.0449°).

A nakshatra pada is 3°20′ = 12,000″, so this shift is roughly 1–1.5% of a pada
at these two epochs — small, but real: it is large enough to move a value
across a pada or KP sub-lord boundary for a chart that happens to sit near
one of those edges.

## Release-policy note

This ships as a **minor** version bump despite (a) changing the value
`Body::TrueNode` and `Body::TrueSouthNode` return for every existing caller,
at every epoch, by up to ~0.35°, and (b) removing `nodes::true_node` as
public API — it is now private and test-only, renamed
`true_node_meeus_truncated`. `CHANGELOG-v7.0.0.md` called an identical "same
name, different meaning" pattern a reason to go major. Both of those points
were raised in review, and both are accepted here as a deliberate
release-policy choice for this release, not an oversight.

One visible consequence worth knowing about: a chart that requests both
`Body::TrueNode` and `Body::TrueNodeOsculating` now gets identical values for
both. If both are fed into an aspect calculation, that produces a degenerate
zero-orb conjunction between them — not a bug, just an expected result of the
two variants now computing the same thing.

## Version

Minor bump: `7.4.0` → `7.5.0`. See `DATA_PROVENANCE.md`'s superseded
annotation on the "Fix 2 — True Lunar Node Tolerance" entry for the
historical record this supersedes.
