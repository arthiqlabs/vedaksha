# v9.0.0

## Chara and Narayana dasha durations ignored the birth chart entirely

**Severity: high. Every Chara and Narayana dasha this engine has ever served carried
chart-independent period lengths.** `compute_dasha` computed each sign's duration from a
hardcoded twelve-entry table mapping the sign to its lord's *other domicile* — Aries to Mars to
Scorpio, and so on — and neither `compute_chara` nor `compute_narayana` accepted any planetary
position at all. Two charts born decades apart, with every graha in a different sign, received
identical durations: Aries always exactly 7 years, Cancer always 12, Capricorn always 1. The
defect was structural, not a boundary case — the functions had no parameter through which a
chart could arrive.

The classical rule is **chart-dependent**. A sign's dasha length is the count from that sign to
**the sign its lord actually occupies in the natal chart**. This is stated in two independent
classical corpora: the Jaimini Upadesha Sutras, Adhyaya 1 Pada 1, in the sutra beginning
*nathantah samah prayena* ("the years end at the lord"), whose Sanskrit commentary glosses it
*svasvadhipashritarashi* — "the sign resorted to by their own respective lords"; and Parashara's
*Brihat Parashara Hora Shastra*, in its chapter on the sign-based dashas, which states the count
runs "from the Rasi up to the house in which its lord is posited". The domicile table that
shipped instead has **no attestation in any source consulted**. A chart-independent duration
table does exist classically — Sthira Dasha assigns 7/8/9 years by sign modality — but that is a
different dasha, and this was not it.

### What the corrected rule computes

For each of the twelve signs: the inclusive count from that sign to its lord's occupied sign,
minus one, giving the number of signs traversed. A lord in its own sign gives 12 years. The count
runs **forward for signs in an odd pada** (Aries/Taurus/Gemini and Libra/Scorpio/Sagittarius) and
**backward for signs in an even pada** (Cancer/Leo/Virgo and Capricorn/Aquarius/Pisces).

That per-sign count direction is a **different rule** from the sequence direction corrected in
v8.1.0, which is a single global decision taken from the pada of the 9th sign from the lagna. The
two share a classification but not a subject: one is asked twelve times, about each sign being
measured; the other once, about the 9th from the lagna. Both are now implemented, separately.

Scorpio and Aquarius have two lords each — Mars and Ketu, Saturn and Rahu respectively, a
co-lordship attested by a *vriddha-karika* and by BPHS independently — and are resolved by the
classical strength cascade, with one documented divergence: the texts list rashi-bala before the
"larger number of years" criterion, but the classical worked example requires the reverse order,
so the implementation follows the worked example and says so in place.

**Verified against a classical conformance oracle.** The BPHS chapter carries a worked chart
giving all twelve dasha lengths; the implemented rule reproduces all twelve exactly, exercising
forward and reverse counting, the own-sign case, and both dual-lord signs. It ships as a
regression test. Its constraint on the cascade is honestly weak and recorded as such: its Aquarius
case is decided identically by three separate criteria, so the single ordering constraint it
actually establishes is that "larger number of years" outranks rashi-bala.

### Deliberately not implemented

A well-attested classical rule (a BPHS verse, and a *vriddha-karika* in the Jaimini line) adds one
year when a sign's lord is exalted and subtracts one when debilitated. It is **omitted here on an
explicit decision**, following a named 20th-century teaching lineage that removed it because it
produces a zero-year dasha for Sagittarius when Jupiter is in Capricorn, and a thirteen-year dasha
for Virgo when Mercury is in Virgo. No classical source addresses either boundary; the tradition's
response to those two cases was to question whether the adjustment belongs at all, and no school
clamps the result. This is a disclosed omission of an attested rule, not an oversight. See
`DATA_PROVENANCE.md` Fix 13.

## Narayana Dasha is now a documented alias of Chara Dasha

`narayana.rs` cited "Jaimini Sutras Ch. 2" as its source. That citation could not be supported and
has been removed. The name "Narayana Dasha" appears in no classical corpus searched — including
the BPHS passage that enumerates the sign-based dasha systems by name, where Chara and Sthira
appear and Narayana does not — and traces instead to a late-20th-century work by a living author.
Its duration rule is identical to Chara's in every source found, so it now receives the identical
corrected computation.

Its one genuinely distinguishing feature, a different starting-sign rule, is **deliberately not
implemented**: its only written source is a modern copyrighted work, which is out of bounds for
this project's clean-room position.

## Breaking API changes

### `compute_dasha` (MCP): Chara and Narayana require new input

Both systems now require a **`graha_signs`** object alongside `lagna_sign`:

```json
{
  "system": "Chara",
  "birth_jd": 2451545.0,
  "lagna_sign": 10,
  "graha_signs": {
    "sun": 9, "moon": 2, "mars": 1, "mercury": 10,
    "jupiter": 10, "venus": 10, "saturn": 4, "rahu": 2
  }
}
```

All eight fields are required and take sidereal sign indices 0-11 — the same values
`compute_natal_chart` already returns as `sign_index`, so they pass straight through. **Ketu is
not an input** and is rejected if supplied: it is derived internally as the sign opposite Rahu,
and accepting both would allow an internally inconsistent chart.

Sign indices were chosen over longitudes deliberately. A longitude would silently accept a
*tropical* value and produce wrong dashas with no error, where a sign index carries a choice the
caller has already made; it also avoids the boundary question of a graha at exactly 30.0 degrees.

### `compute_dasha` (MCP): `lagna_sign` is now 0-indexed

`lagna_sign` now takes **0-11 (0 = Aries)**, matching the `sign_index` convention that
`compute_natal_chart`, `compute_bhavas` and `compute_vargas` all serve. Through v8.1.0 this single
parameter was 1-indexed while every other tool was 0-indexed — a mismatch `CHANGELOG-v8.0.0.md`
disclosed as unresolved, and the direct cause of the off-by-one defect that went unnoticed from
v2.4.0 through v8.0.0. The dispatch no longer performs any conversion, so that defect class is
removed rather than guarded.

### `compute_dasha` (MCP): Chara's response envelope changed

Chara previously returned a bare JSON array while Narayana returned `{lagna_sign, periods}`, for
two systems that compute identically. Both now return the object envelope. The echoed
`lagna_sign` is 0-indexed, matching the rest of the served surface.

### `compute_dasha` (MCP): output schema corrected

The declared `output_schema` described only the Vimshottari shape, with `moon_nakshatra`,
`initial_balance` and `maha_dashas` all marked required. That was already false for Ashtottari
(which serves `periods` and `starting_lord`) and Yogini (`maha_periods`,
`starting_yogini_index`), and false for Chara/Narayana, which return no nakshatra at all. It is
now a `oneOf` over the four envelopes the five systems actually return.

### Published Rust API (`vedaksha-vedic`, crates.io)

- `dasha::chara::compute_chara(lagna_sign, birth_jd)` -> `compute_chara(lagna_sign, birth_jd, graha_signs)`
- `dasha::narayana::compute_narayana(lagna_sign, birth_jd)` -> `compute_narayana(lagna_sign, birth_jd, graha_signs)`
- New public type `dasha::chara::GrahaSigns` (eight `u8` fields, `Copy`).

Both functions take `lagna_sign` 0-indexed, unchanged from before — the indexing change above is
to the MCP surface, which previously converted.

## Migration

For any caller of `compute_dasha` with `system` of `Chara` or `Narayana`:

1. **Subtract 1 from `lagna_sign`.** A chart with an Aquarius ascendant sent `11`; it must now
   send `10`. If the value comes from `compute_natal_chart`'s `sign_index`, pass it through
   unchanged and delete whatever `+ 1` was previously applied.
2. **Add `graha_signs`** with all eight fields, taken from the same natal chart's `sign_index`
   values. Do not send `ketu`.
3. **Chara callers: unwrap the response.** What was a bare array is now `response.periods`.
4. **Expect different numbers.** Durations now vary by chart. Any stored or cached Chara/Narayana
   result computed by an earlier version is wrong and should be recomputed, not migrated.

Callers of the other three systems (Vimshottari, Ashtottari, Yogini) are unaffected.

## Known issues, disclosed

- The bit-digest fixtures remain architecture-dependent (the SIMD trigonometric kernel they
  fingerprint computes differently on AVX2 than on NEON). Since v8.1.0 they carry per-architecture
  pins and pass on both x86_64 and aarch64; an unpinned architecture fails loudly rather than
  skipping. "Bit-identical", as this project uses the term, is a within-one-architecture claim.
- `structured_key` remains `None` for `compute_dasha`, so no `structuredContent` is emitted and
  the corrected `output_schema` above is documentation rather than an enforced contract. The
  output-schema tests therefore still have nothing to compare against.
