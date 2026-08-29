# Vedaksha v7.5.0

**`Body::TrueNode` now returns the osculating node, not a truncated series.** Vedaksha has
shipped two computations of the Moon's true ascending node since v2.0.0: a 5-term Meeus
approximation (`Body::TrueNode`) and a computation from the Moon's actual instantaneous
orbital elements (`Body::TrueNodeOsculating`). The second has been independently validated to
0.6 arcsecond against JPL Horizons since 2026-07-16; the first documented its own larger,
bounded residual against fuller lunar theory in its own doc comments from day one. Nothing
ever pointed `Body::TrueNode` at the more accurate of the two.

Values move only on `Body::TrueNode` and `Body::TrueSouthNode`. Nothing else changes.

---

## What changed, and what didn't

`Body::TrueNode` and `Body::TrueSouthNode` now compute exactly what `Body::TrueNodeOsculating`
and `Body::TrueSouthNodeOsculating` already computed. All four `Body` variants remain — this is
not a breaking removal of API surface, and every existing call site that names a `Body` variant
continues to compile unchanged. What changes is the *value* those two variants return: up to
~0.35° of difference from the previous truncated-series output, per this project's own existing
regression tests (`osculating_node_close_to_meeus_true_node`,
`osculating_node_multi_epoch_sanity`), which already bounded and documented that gap before this
release.

The 5-term Meeus formula is not deleted. It survives privately inside
`crates/vedaksha-ephem-core/src/nodes.rs`, used only by that module's own tests as an
independently-derived method to cross-check the osculating computation against — the same class
of guard that caught the v7.0.0 node-frame defect, where two node methods silently drifted into
different reference frames.

## Why now

Both computations have existed side by side, undifferentiated in status, since the very first
commit in this codebase's history. Nothing in the source, tests, or prior release notes ever
stated a reason `Body::TrueNode` should return the less accurate of the two. This release closes
that gap in favor of the already-more-accurate, already-tested implementation.

## Version

Minor bump: `7.4.0` → `7.5.0`. See `DATA_PROVENANCE.md`'s superseded annotation on the "Fix 2 —
True Lunar Node Tolerance" entry for the historical record this supersedes.
