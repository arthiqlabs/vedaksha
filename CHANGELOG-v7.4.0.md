# Vedaksha v7.4.0

**`compute_vargas` now computes the charts it has described since it shipped.** Called with its
four required parameters it used to return a `status`/`message` stub; three of those parameters
were parsed, validated and never read.

Values move only on `compute_vargas`. Nothing else changes by more than 0.

---

## What it returned, and what it returns

The description promised "a ChartGraph JSON for each [division], including planetary positions and
dignities within each varga". What you actually got:

```json
{"status":"validated","message":"Input validated. Provide planet_longitude for direct varga computation."}
```

The only computing path needed the **optional** `planet_longitude` and answered `{"D9": 7}` — one
sign index, for one body, with no dignity and no lagna.

Now, per requested division: the varga lagna, and for each of the ten bodies
`compute_natal_chart` returns — the seven grahas plus the mean, true and osculating lunar node —
its rashi longitude, the sign it occupies in that varga, its dignity in that sign, and its
whole-sign bhava counted from the varga lagna. Ketu is not listed separately; it is the node's
opposite point. The nodes carry no dignity, so that field is absent for them rather than null.

## Derived from the D-1, and tested to be

Positions and the ascendant come from `compute_sidereal_chart`, extracted from the natal handler
and now shared by both tools. A varga computed from its own copy of the ephemeris plumbing could
disagree with the D-1 it claims to divide, and nothing would catch it — the two surfaces have no
oracle in common.

So the claim is a test rather than a comment: `mcp_compute_vargas_d1_agrees_with_the_natal_chart`
asserts that the D-1 sign and longitude equal `compute_natal_chart`'s, body for body. `D-1 is the
rashi chart` is exactly the identity that makes this checkable.

## BREAKING: the response shape

The flat `{division: sign}` map is gone. Both input shapes return one envelope:

```
result.vargas[0].placements[0].varga_sign     // was result["D9"]
```

**The signs themselves are unchanged.** The single-longitude path is preserved, and the existing
1,000-point consistency test still compares it against `varga_sign` called directly.

On that path, `planet`, `dignity`, `lagna_sign` and `bhava` are **absent, not null**: a bare
longitude names no graha, so it has no dignity, and there is no lagna to count a bhava from. The
output schema requires none of them for that reason.

## `tradition`, and what it may change

BPHS and Phala Deepika diverge on **D16, D20, D30 and D45** — modality-based against element-based
starting signs. The engine has known this for releases; the tool silently used the modality
reading. It is now a parameter, with both sources named in the schema.

A test asserts the element reading changes those four vargas and **nothing else**. A parameter that
quietly does nothing is worse than no parameter.

## Sixteen of seventeen tools now declare an output schema

`compute_vargas` could not before: no schema can be honest about a contract that is not kept. Only
`emit_graph` still declares none, because its shape genuinely depends on `format`.

## What would have caught this

`mcp_compute_vargas_documented_path_returns_charts` asserts what the description claims. It is
red-proven: putting the stub back fails it with the stub's own text, and the restored tree passes.
The lesson is the same one v7.2.0's Shadbala defect taught — a description and a behaviour that
nothing compares will drift, and only a test that reads the description can notice.
