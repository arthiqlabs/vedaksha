# Migrating from the Vedaksha 5 ayanamsha surface

**Every ayanamsha name changed, and every ayanamsha value changed.** This is a
breaking change on purpose. If you compute sidereal charts with Vedaksha, read
this before upgrading.

---

## What happened

Vedaksha 5 shipped forty-four ayanamshas. Three had recorded provenance, and it
named Swiss Ephemeris as the source of their constants. Fifteen of the others
carried comments saying they had been *"adjusted by ±X° … to match independent
reference"* — a reference named nowhere in the repository.

They were removed and derived again from primaries, without ever seeing what
they were replacing. Eleven survived. The full account is in
[`README.md`](README.md); the specification is [`spec.md`](spec.md).

**The values you get are different, and the new ones are the ones to keep.** A
value produced by aiming at someone else's answer is not derived, whatever
citation is attached afterwards. Where a new value lands close to the old one,
that is a welcome result and nothing more. Where it lands further away, the
derived value is what ships.

## What you have to change

### 1. Names

Nothing silently remaps. A retired name is **refused at parse time** with a
message saying what happened to it, because a silent remap would move your chart
without telling you.

| If you passed | Pass this | Note |
|---|---|---|
| `Lahiri` | `IndianOfficial` | `Lahiri` still parses as an alias, so string callers keep working. The canonical name is `IndianOfficial`, and that is what the MCP schema's `enum` advertises — a strict client validating against the schema will reject `Lahiri`. |
| `FaganBradley` | `FaganBradley` | unchanged name |
| `Krishnamurti`, `Krishnamurti2` | `Krishnamurti` | KSK published one anchor and one rate |
| `Raman`, `BvRamanMean` | `Raman` | Art. 49 gives one rule |
| `SuryaSiddhanta`, `SuryaSiddhantaMean`, `SsCitra`, `SsDrevJul` | `SuryaSiddhanta` | the text defines one libration |
| `Lahiri1940`, `LahiriVp285`, `AyanamshaOfDate` | `IndianOfficial` | one system, not a family of epochs |
| `TrueChitrapaksha` | `TrueChitra` | renamed and re-derived from Spica at 180° |
| `Yukteshwar` | `Yukteshwar` | unchanged name |
| `GalacticCenter0Sag` | `GalacticCenter0Sag` | unchanged name |
| `TrueRevati` | **choose** `RevatiPaksha` or `SuryaSiddhanta` | the old name did not say which longitude Revati was held at, and that is the whole discriminator — the two differ by 10′ |
| `TruePushya` | `PushyaPaksha` | the old variant's longitude matched no located primary; Rao defines δ Cancri at 106° |
| `TrueMula` | `ChandraHari` | the old name carried a galactic-alignment reading with no primary; `ChandraHari` is λ Scorpii at 240° per *IJHS* 33(4), 1998 |
| the other 22 | — | dropped; see below |

**Dropped with a documented search:** De Luce, Usha-Shashi, Gil Brand,
Wilhelm's mid-Mula, Sassanian / Aldebaran-15-Taurus, JN Bhasin, Sundara Rajan,
Djwhal Khul (both), Skydram, Valensmoon, True Moon's Node, Hipparchos, Aryabhata
(both), and the remaining galactic variants. These are **unverified, not
refuted** — most exist only in books that must be bought and read. If you need
one, the route is to obtain the book, not to search harder.

**Dropped on principle:** every Babylonian system — Huber, ETPSC, Kugler ×3.
They fail the definition/determination test: a scholar *estimating* where a
historical zero point lay is not the same as someone *stipulating* one. The
evidence is that two secondary accounts of Huber's figure differ by six
arcminutes. A definition does not have competing values; a measurement does.

### 2. Values

You will get different numbers. If you have stored charts, cached longitudes, or
tests pinned to a Vedaksha 5 ayanamsha, they will not match.

**This is an observation, not a gate.** It was computed once, after the values
were frozen and committed, by a script separate from the generator, and it has no
pass/fail. Where a derived value lands close to what was shipped, that is a
welcome result and nothing more.

### Delta against v5.0.2, at J2000.0

| v5 name | v6 name | relation | delta (arcsec) |
|---|---|---|---|
| `Lahiri` | `IndianOfficial` | renamed | -0.00 |
| `FaganBradley` | `FaganBradley` | renamed | +1.58 |
| `Krishnamurti` | `Krishnamurti` | renamed | -4.71 |
| `Raman` | `Raman` | renamed | +4.84 |
| `Yukteshwar` | `Yukteshwar` | renamed | +128.67 |
| `SuryaSiddhanta` | `SuryaSiddhanta` | renamed | +182.42 |
| `GalacticCenter0Sag` | `GalacticCenter0Sag` | renamed | -0.96 |
| `TrueChitrapaksha` | `TrueChitra` | renamed | +1.34 |
| `TrueRevati` | `RevatiPaksha` | rebased | -811.58 |
| `TruePushya` | `PushyaPaksha` | rebased | +3.61 |
| `TrueMula` | `ChandraHari` | rebased | -0.99 |

### Range of the delta across the sampled epochs

| v5 name | v6 name | min (arcsec) | max (arcsec) | worst epoch |
|---|---|---|---|---|
| `Lahiri` | `IndianOfficial` | -130.18 | +4.31 | Kali epoch (-130.18) |
| `FaganBradley` | `FaganBradley` | -329.89 | +32.47 | Kali epoch (-329.89) |
| `Krishnamurti` | `Krishnamurti` | -2802.97 | -4.05 | Kali epoch (-2802.97) |
| `Raman` | `Raman` | -3639.77 | +8.35 | Kali epoch (-3639.77) |
| `Yukteshwar` | `Yukteshwar` | -22217.18 | +3588.40 | Kali epoch (-22217.18) |
| `SuryaSiddhanta` | `SuryaSiddhanta` | -72031.10 | +172243.18 | Kali epoch (+172243.18) |
| `GalacticCenter0Sag` | `GalacticCenter0Sag` | -638.80 | +53.71 | Kali epoch (-638.80) |
| `TrueChitrapaksha` | `TrueChitra` | -471.71 | +108.31 | Kali epoch (-471.71) |
| `TrueRevati` | `RevatiPaksha` | -2089.11 | -810.24 | Kali epoch (-2089.11) |
| `TruePushya` | `PushyaPaksha` | -911.99 | +17.99 | Kali epoch (-911.99) |
| `TrueMula` | `ChandraHari` | -529.42 | +64.35 | Kali epoch (-529.42) |

### How to read that

**In the modern era the well-attested systems barely moved.** The Indian
official system is within 0.01″ of what v5 shipped at J2000 — unsurprising, since
its operative definition is a published government anchor and there is only one
right answer to converge on. Fagan-Bradley, Krishnamurti, Raman, the Galactic
Centre, True Chitra, Pushya-paksha and Chandra Hari all sit within about 5″.

**Two systems moved by minutes, and those are the two where the derivation
stopped approximating the text.**

- **Surya Siddhanta, +182″ at J2000.** v5 propagated it as a linear precession.
  The text specifies a *trepidation* — a bounded zigzag libration of ±27° — and a
  derivation that linearises it has changed the definition. This is the largest
  modern-era move and it is the most defensible one.
- **Yukteshwar, +129″ at J2000.** v5 evidently propagated his anchor with a
  modern precession model. His anchor is his own model's output — 1394 × 54″ is
  exactly the figure printed — so his own 54″/yr is the rate that belongs with
  it. Propagating it with P03 instead inverts to a zero year near 390 CE,
  missing his documented 500 CE by a century.

**The far-epoch column is not a defect.** Every system's delta blows up at the
Kali epoch (−3101), because v5's values were flat or linear and this derivation
propagates each system by the model its own definition implies. The Surya
Siddhanta's +172,243″ there is a bounded libration being compared against a
linear model that has run away by 47°; the libration is the one obeying the text.

**`TrueRevati` → `RevatiPaksha` is −811″, and that is why the old name
hard-errors.** If v5's `TrueRevati` had held ζ Piscium at the Surya Siddhanta's
359°50′, the gap would have been exactly 600″. It is not, so v5's variant was
neither of the two readings the primaries support. Aliasing that name to either
one would have silently moved charts to a system the caller never asked for.

*Note on what this table discloses: a delta plus a new value recovers an old
value. The v5 constants are already public — they shipped in a published crate
and remain in git history — so nothing here is newly disclosed. The spec directs
this comparison into the migration note precisely so that it exists in one place
with its purpose stated, rather than being reconstructed ad hoc by users.*

Read the `rebased` rows as information, not as regressions: those pairs answer
*different defining conditions*, so a delta between them is expected and does not
measure an improvement in either.

### 3. Behaviour that is now stated rather than implied

- **The output is the MEAN ayanamsha.** Nutation in longitude is never included.
  If you compare against a published daily table, check which column you have:
  the *Indian Astronomical Ephemeris* prints `True Ayanamsa = Mean Ayanamsa +
  nutation in longitude`, and the tables most panchanga-makers consume are the
  **true** values, up to ~17″ away. Add nutation yourself if that is what you
  want. This was never stated before, so a comparison against the wrong column
  would previously have looked like an engine error.
- **Star-anchored systems use the mean geometric place of date** — proper motion
  applied, aberration and nutation not. "The tropical longitude of the star" is
  ambiguous by ~20″ of aberration plus ~17″ of nutation, which is larger than the
  gap between neighbouring systems, so the convention is part of the definition.
- **Five of the eleven track a live star**, so their values depend on catalogue
  astrometry. The honest uncertainty is the catalogue's: the two Hipparcos
  reductions differ by up to 3.3 mas/yr in proper motion, ≈0.5″ of longitude per
  thousand years from the catalogue epoch.
- **`ChartConfig::default()` is still tropical** (`ayanamsha: None`).
  `Ayanamsha::default()` is `IndianOfficial`, and applies only where an
  `Ayanamsha` value is required and none was given.

### 4. API changes beyond the names

- `Ayanamsha` gained `FromStr`, `Display`, `key()`, `primary_source()`,
  `is_star_anchored()`, `ALL` and `SIDEREAL`, and is now `#[non_exhaustive]` —
  downstream `match` needs a catch-all arm. The list may grow: a system dropped
  as *unverified* can return if its primary is obtained.
- `Ayanamsha::name()` takes `self` by value rather than `&self`.
- `config_summary` now renders the ayanamsha via `key()` rather than the `Debug`
  derive, so it reads `Zodiac: IndianOfficial`. **This string feeds the graph
  chart-id**, so stored chart-ids computed with a sidereal config will change.
- `compute_vargas` no longer takes an `ayanamsha` parameter. It never used it —
  the tool takes an already-sidereal longitude — and advertising a parameter that
  does nothing was worse than removing it.
- The MCP schema now carries a real `enum` for `ayanamsha`, generated from
  `Ayanamsha::ALL`, so it can no longer drift from the engine. Previously both
  properties were free-form strings whose descriptions named three systems while
  the engine had forty-four.
- The wasm and MCP surfaces now share the engine's parser. They had drifted from
  each other (they disagreed about `"tropical"`) and between them reached five of
  the forty-four systems.

### 5. One change that is not about ayanamshas

`precession_matrix` composed its four Fukushima-Williams rotations in the wrong
order. The error is 0.014 mas at J2000 and **0.56″ at 499 CE** — invisible across
the era the JPL Horizons oracle covers, growing outside it. It is fixed.

If you compute historical charts far from the present, planetary positions move
slightly. Modern-era results are unchanged at the milliarcsecond level, and the
24,350-row Horizons comparison is unaffected.

## Downstream: KundaliMCP

KundaliMCP is on Vedaksha 4.0.0 and hardcodes `Lahiri` in several call paths.
On upgrade:

1. `Lahiri` still parses, so nothing breaks immediately at the string layer.
2. But every value moves, so cached or stored charts must be recomputed.
3. `config_summary` changes, so any stored graph chart-id derived from a sidereal
   config changes with it.
4. If any client validates arguments against the published MCP schema, `Lahiri`
   will fail that validation — send `IndianOfficial`.

## Release status

This work is **not released**. The version is unchanged and no crate has been
published. Cutting a release for a breaking change of this size — the version
bump, the yank/deprecation strategy, the disclosure question raised by the
original provenance, and the downstream notice — is the project owner's call, not
part of the derivation.
