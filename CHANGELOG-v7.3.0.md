# Vedaksha v7.3.0

**The MCP surface described what to send and never what came back.** Fifteen tools now declare an
output schema and return structured content that conforms to it, every tool declares itself
read-only, and nineteen parameters that had no description have one.

No computed value changes. Nothing here moves a longitude, a bala or a dasha boundary.

---

## `tools/list` now describes the response, not just the request

An agent calling this server received a JSON string and had to guess its shape. Fifteen of the
seventeen tools now carry an `outputSchema`, and `tools/call` returns `structuredContent` that
conforms to it.

Both halves ship together on purpose. MCP requires a tool that declares an output schema to return
structured content conforming to it, so a schema alone is a promise nothing keeps, and structured
content alone is a shape nothing checks. The code makes that pairing structural rather than
remembered: `structured_content()` returns nothing unless a schema exists.

**`content[].text` is unchanged.** `structuredContent` is added alongside it, so anything already
parsing the text block keeps working byte for byte. Six tools answer with a JSON array, which
`structuredContent` cannot be, so those are nested under a key named per tool rather than reshaped
in the handler.

Every schema was derived from a real response captured through the built binary over stdio, not
from reading the Rust types. `tests/output_schemas.rs` calls all fifteen through the production
server and validates what comes back, so a schema that drifts from the code fails the suite instead
of misleading an agent.

## Two tools deliberately declare nothing

`emit_graph` has no single output shape: `cypher`, `surreal` and `embedding` return plain text,
`jsonld` returns `@context`/`@graph`, and `json` returns `nodes`/`edges`. One schema would be false
for four of the five formats.

**`compute_vargas` is a disclosed defect, not a design choice.** Its description promises "a
ChartGraph JSON for each [division], including planetary positions and dignities". What it does:

- called with its four **required** parameters, it returns
  `{"status":"validated","message":"Input validated. Provide planet_longitude ..."}` — a stub, not
  an error;
- called with the **optional** `planet_longitude`, it returns `{"D1":3,"D9":7,"D10":4}`, a map of
  division to sign index for a single planet;
- `julian_day`, `latitude` and `longitude` are required and inert.

That contract needs a decision, and a schema written over the top of it would have codified the
break. The gap is stated here and in the module documentation rather than papered over. Callers
should not rely on the documented behaviour until it exists.

## Every tool declares itself read-only

`readOnlyHint: true`, `openWorldHint: false`, on all seventeen. Only those two hints: the MCP
specification defines `destructiveHint` and `idempotentHint` as meaningful **only when
`readOnlyHint` is false**, so emitting them here would be noise dressed as information.
`openWorldHint` is false on evidence — the MCP crate's dependency list contains no HTTP client, and
`tiny_http` is an inbound transport.

That every tool is read-only is not an accident of the current set. The engine is stateless by
construction, because determinism is what makes the oracles, the bit-identity claims and the
exact-equality Python fixture mean anything.

## Nineteen parameters had no description

Across `compute_combustion`, `compute_shadbala` and `compute_ashtakavarga`. Found by walking the
server's own `tools/list` output rather than by reading source. Each description now states what
the value does:

- the retrograde flags narrow the combustion orb, and by how much per planet — Mars 17° → 8°,
  Mercury 14° → 12°, Venus 10° → 8°. Jupiter's and Saturn's orbs are the same either way, so those
  two flags are documented as accepted and inert rather than left to imply an effect they lack;
- `is_daytime` feeds Nathonnatha Bala, `moon_phase_waxing` feeds Paksha Bala, `speed` and
  `average_speed` feed Cheshta Bala, `bhava` feeds Dig and Kendradi Bala, and the aspect counts
  feed Drik Bala.

## The container is now an installable package in the MCP Registry

`server.json` declares the published image as an OCI package, so a client discovering Vedaksha
through the official registry is told how to run it instead of only where the source lives. The
declared invocation is the one that was measured, not one derived from the Dockerfile:

```
docker run --rm -i ghcr.io/arthiqlabs/vedaksha-mcp:v7.3.0 --stdio
```

## Breaking for one caller shape

`ToolDefinition` gained two public fields, `output_schema` and `structured_key`. Anything outside
this workspace constructing that struct with a literal will no longer compile. We know of no such
caller, and the field is required rather than defaulted on purpose: a tool that is *not* read-only
should be a compile error its author has to answer, not a wrong hint inherited in silence. It is
recorded here rather than left for someone to discover.

## Internals

The camelCase wire projection existed in three places — the server handler, the `dump-tools-list`
binary and the snapshot drift guard — so adding a field failed the guard against a snapshot that
had just been correctly regenerated. All three now call `ToolDefinition::to_wire`, and a test
asserts the projection carries every key, because a guard that compares the snapshot to the same
projection it serialises cannot catch a field missing from both.
