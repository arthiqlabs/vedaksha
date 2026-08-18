# Vedaksha v6.0.0

**Breaking.** Every ayanamsha name and every ayanamsha value has changed. If you compute
sidereal charts, read
[`docs/audit/2026-08-17-ayanamsha-cleanroom/migration-note.md`](docs/audit/2026-08-17-ayanamsha-cleanroom/migration-note.md)
before upgrading.

---

## The sidereal surface is now derived from primary sources

Vedaksha 5 shipped forty-four ayanamshas. For most of them there was **no derivation in the
record** — no path from a primary source to the number. Three had provenance recorded, and
that record was published with the releases at the time.

Rather than audit those values in place, they were removed and computed again, forward from
primaries, by an implementation that never saw what it was replacing. Eleven systems survived
that process. Each is traceable to a chapter, a star, a committee, or a named proposer's own
publication.

This is a standard being raised, not a defect being patched. The v5 values were treated as
underived — whatever their actual origin — because a value whose derivation cannot be shown
is not one a licensee can check.

### What you have to change

| If you passed | Pass this |
|---|---|
| `Lahiri` | `IndianOfficial` (the v5 string still parses) |
| `Krishnamurti2`, `SuryaSiddhantaMean`, `SsCitra`, `SsDrevJul`, `BvRamanMean`, `Lahiri1940`, `LahiriVp285`, `AyanamshaOfDate` | their surviving head system |
| `TrueChitrapaksha` | `TrueChitra` |
| `TrueRevati` | **choose** `RevatiPaksha` or `SuryaSiddhanta` — they differ by 10′ and the old name did not say which |
| `TruePushya` | `PushyaPaksha` |
| `TrueMula` | `ChandraHari` |
| 24 others | no successor — see the migration note |

**Nothing silently remaps.** A retired name is refused at parse time, and on serde
deserialization, with a message saying what happened to it. A silent remap would move your
chart without telling you.

### Behaviour now stated rather than implied

- **The engine returns the MEAN ayanamsha.** Nutation is never included. Published daily
  tables are usually the *true* values, up to ~17″ away; add nutation yourself if that is
  what you want to compare against. This was never stated before, so a comparison against
  the wrong column would previously have looked like an engine error.
- **Star-anchored systems use the mean geometric place of date** — proper motion applied,
  aberration and nutation not. "The tropical longitude of the star" is otherwise ambiguous by
  more than the gap between neighbouring systems.
- Five of the eleven track a live star, so their values follow catalogue astrometry and will
  move when a catalogue is superseded. The honest uncertainty is stated in the audit
  directory.

## Corrections to the astronomy, which outlast the ayanamshas

Two engine defects surfaced while validating the derivation. Both predate this work and both
affect more than the sidereal surface.

- **`precession_matrix` composed its four Fukushima-Williams rotations in the wrong order.**
  The error is 0.014 mas at J2000 and **0.56″ at 499 CE** — invisible across the 1900–2100
  window the JPL Horizons oracle covers, and growing without bound outside it. Historical
  planetary positions move slightly; modern-era results are unchanged at the milliarcsecond
  level.
- **`general_precession_in_longitude` returns `ψ̄_A`, not `p_A`.** They differ by
  ~9.7″/century and neither is distinguishable from the other by inspecting a result. A new
  `general_precession_p03` supplies the general precession an epoch-anchored sidereal system
  needs; both are now documented together and a test asserts they are not interchangeable.

Two systems also moved by minutes because the derivation stopped approximating their texts:
the Surya Siddhanta is a bounded trepidation rather than a linear precession (+182″ at
J2000), and Yukteshwar's anchor is his own model's output, so his own rate belongs with it
(+129″).

## How the claim can be checked rather than trusted

`scripts/generate_ayanamsha.py` re-derives every system in Python from the recorded primary
inputs and emits the fixture the Rust engine is asserted against. `--verify` regenerates and
compares by hash, needs no network, and is wired into full validation, so it cannot pass
having checked nothing. `--check-catalogue` re-queries VizieR and reports what it could not
cover.

The audit directory records the primaries, the process, the declared assumptions, and — for
every system that was dropped — what was searched and where. A drop is only final when the
search is recorded; an undocumented negative search is not a finding.

**What the tests do not establish, stated plainly:** anchor reproduction is self-consistency,
and for the epoch-anchored systems it is algebraically vacuous. Derivation integrity rests on
the audit trail, not on the suite. That is written into the audit directory rather than left
for a reader to discover.

## Other changes

- `compute_vargas` no longer takes an `ayanamsha` parameter. It never read it — the tool
  takes an already-sidereal longitude.
- The MCP `ayanamsha` parameter now carries a real JSON-Schema `enum`, generated from the
  engine, so the schema cannot drift from what the engine has.
- The wasm and MCP surfaces share the engine's parser. They had drifted from each other and
  between them reached five of the forty-four systems.
- `config_summary` renders the ayanamsha via `key()` rather than the `Debug` derive, so it
  reads `Zodiac: IndianOfficial`. **This string feeds the graph chart-id**, so stored
  chart-ids computed with a sidereal config will change.
- `Ayanamsha` gained `FromStr`, `Display`, serde, `key()`, `primary_source()`,
  `is_star_anchored()`, `ALL` and `SIDEREAL`, and is `#[non_exhaustive]`.

## Upgrading

Recompute anything cached. Stored charts, cached longitudes, tests pinned to a v5 ayanamsha,
and graph chart-ids derived from a sidereal config all change.
