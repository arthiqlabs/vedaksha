# Vedākṣha v5.0.1 — the property graph, made reachable

**Release date:** 2026-08-17

A patch release. No numerical result changes, no breaking API changes.

It fixes a capability the project documented but could not actually perform.

---

## Fixed: a property graph nothing could produce

`vedaksha-graph` shipped an ontology (9 node types, 12 edge types), five
emitters — Cypher, SurrealQL, JSON-LD, JSON, RAG embedding text — 2,965 lines
and 16 passing tests. It had **no way to build a graph from a chart.**
`ChartGraph::new` returns an empty graph, and every other construction in the
repository was a test fixture. The emitters were verified against
hand-assembled input that no caller could obtain from a computation.

Three published statements were therefore false:

- the README's "every chart is a property graph you can query in Cypher,
  SurrealQL, or JSON-LD",
- the README's "every computation produces a property graph, not flat structs",
- the `emit_graph` tool schema's "ChartGraph JSON as returned by
  `compute_natal_chart`".

Doing exactly what that last one instructed — call `compute_natal_chart`, pass
its output to `emit_graph` — returned `Invalid parameter 'chart_json': missing
field 'nodes'`.

**`vedaksha_graph::chart_to_graph`** (behind the new non-default `from-chart`
feature) maps a `ComputedChart` to 4 of the 9 node types and 7 of the 12 edge
types: `Chart`, `Planet`, `Sign`, `House`; `BelongsTo`, `PlacedIn`, `Occupies`,
`Aspects`, `Rules`, `CuspOf`, `Disposits`. The five it does not produce —
`Nakshatra`, `Pada`, `Pattern`, `DashaPeriod`, `FixedStar` — need data a chart
does not carry, and the module documents that rather than leaving a reader to
discover an empty node set.

**`emit_graph` now accepts either shape**: an existing `ChartGraph`, or the
output of `compute_natal_chart`. Building from a computed chart **requires**
`latitude` and `longitude`, which are new optional parameters on the tool. A
chart result does not record where it was cast, and the graph's `Chart` node
does; defaulting to 0, 0 would have put a fabricated observer into the emitted
graph, indistinguishable downstream from a real one.

Two details worth stating, because they are the sort of thing that rots:

- The tool's published output is **not** a serialised `ComputedChart` — its
  `aspects` use `body1`/`body2`/`type`/`applying` where the struct has
  `body1_index`/`body2_index`/`aspect_type`/`motion`. The adapter reads the
  *published* shape, which is the contract callers actually see.
- The graph's vocabulary is the chart's vocabulary. A test asserts that sign
  names in the graph equal the chart's own `sign` strings, so the two cannot
  drift into needing a translation layer.

The pre-existing `emit_graph` test passed on an **empty** `ChartGraph` — zero
nodes, zero edges — which is how a green suite coexisted with an unreachable
feature. The new test chains `compute_natal_chart` into `emit_graph` and
asserts 35 nodes.

## Source comments

Source doc-comments now cite the classical texts by chapter — BPHS, the Jaimini
Sutras — or state the defining rule outright, so the code carries its own
derivation rather than pointing outward. `CONTRIBUTING.md` states that as the
standard for new contributions.

**No computation changed.** These are comments and docs.

## Also

- **`SECURITY.md` scope corrected.** It solicited reports about license-key
  signing and Stripe webhook security. This repository has neither — it is a
  computation library and an MCP server, handling no payments, issuing no keys,
  storing no user data. Replaced with the surface that does exist, including
  unsafe deserialization of ephemeris kernels and tool arguments.
- **README cut from 298 to 114 lines**, and two false claims in it fixed: the
  property-graph statement above, and a reproduction command that could not
  print the ayanamsha figures (they live in `vedaksha-astro`, not
  `vedaksha-ephem-core`). "What is *not* measured" is now its own section —
  house cusps unvalidated, 3 of 44 ayanamshas numerically validated, dasha and
  nakshatra tests being invariant tests rather than external comparisons.

---

Verified at this commit: 1,024 workspace tests pass, clippy `-D warnings`
clean, `cargo fmt --check` clean, clean-room check clean.
