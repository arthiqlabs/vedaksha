# Ayanamsha — primary-source re-derivation

**Module:** `crates/vedaksha-astro/src/sidereal.rs`
**Started:** 2026-08-17 · **Landed:** 2026-08-18
**Precedent:** [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](../2026-05-09-elp-mpp02-cleanroom/)
**License context:** BSL 1.1. This directory is the evidence a BSL user checks.

---

## Why this exists

Vedaksha 5 shipped forty-four ayanamshas. Three had provenance recorded, and that
provenance named Swiss Ephemeris (LGPL) as the source of their constants. Of the
remaining forty-one, fifteen carried code comments of the form *"adjusted by ±X°
… to match independent reference"* — a reference named nowhere in the
repository.

That is the shape of the problem, and it is not primarily a licensing one.
**Values produced by aiming at someone else's answer are not derived, whatever
citation is attached to them afterwards.** A constant tuned to match, then
footnoted to a siddhanta, misrepresents both the constant and the siddhanta.

So the values were removed and computed again, forward from primaries, without
ever seeing what they were replacing.

## The governing rule

> The direction of derivation is what makes this clean-room, not the citations
> attached to it. Computing forward from a documented primary and accepting the
> result is primary research. Tuning toward a known output is reverse
> engineering, and it remains reverse engineering however the result is cited
> afterwards.

Binding consequence: **no acceptance test references the values Vedaksha 5
shipped, Swiss Ephemeris, or any other implementation.** A derived value is
accepted because it follows from its stated inputs, not because it resembles
anything.

## Firewall — what was actually enforced

| Control | How |
|---|---|
| The implementer could not see the target | The forty-four constants were deleted in `4033c6a`, **before** the derivation branch was cut. The working tree contained no ayanamsha value. |
| History was not consulted | The git history of `sidereal.rs` was not read at any point. `4033c6a` was inspected with `--stat` only. |
| The spec could not leak the target | `scripts/check_spec_hygiene.py` greps the spec for the previously shipped digit strings and for comparison phrasings that make a delta recoverable. Patterns live in the script, never in the spec. It now runs in CI on every push. |
| No forbidden upstream | No Swiss Ephemeris source, header, documentation or output, including `swetest`; no wrapper or service backed by it; no other astrology software. See the fetch manifest. |
| Post-freeze comparison is not a gate | The delta against the old constants was computed **once, after the freeze commit**, by a separate script, and appears only in the migration note. It has no pass/fail. |

**Where this diverges from the ELP precedent, deliberately.** That work captured
a legacy oracle of 10,000 pre-rederivation tuples and regression-tested against
it. Sound there: ELP's contamination was *structural*, while the coefficient
values existed independently in the IMCCE primary, so the old outputs were a
legitimate fact-check. **Here the values themselves were the contaminated
artefact.** Regression-testing against them would reinstate the exact
aim-at-the-known-answer failure this work exists to undo. There is no legacy
oracle, and there must never be one.

## Result: 44 → 11

Eleven systems, each traceable to a chapter, a star, a committee, or a named
proposer's own publication. Plus a tropical identity, which is not a system.

| # | System | Primary | Declared assumptions |
|---|---|---|---|
| 1 | Indian official (Lahiri / Chitra-paksha) | *Indian Astronomical Ephemeris 2022*, p. 380 | — |
| 2 | Fagan-Bradley | Fagan & Firebrace, *Primer of Sidereal Astrology*, pp. 13, 16 | DA-3 |
| 3 | Krishnamurti (KP) | *Krishnamurti Padhdhati Vol-I*, p. 140 | DA-4, DA-5 |
| 4 | Raman | *A Manual of Hindu Astrology* (1935), Ch. III Art. 49 | DA-7, DA-8 |
| 5 | Surya Siddhanta | Surya Siddhanta Ch. 3 vv. 9–12 | DA-2, DA-10 |
| 6 | Yukteshwar | *The Holy Science* (1894) | DA-6 |
| 7 | Revati-paksha | Revati at the sidereal initial point — majority reading, against SS Ch. 8's own 359°50′; ζ Piscium | DA-1 |
| 8 | Pushya-paksha | Narasimha Rao, *Introducing Pushya-paksha Ayanamsa* | — |
| 9 | True Chitra | Self-describing condition; Spica | — |
| 10 | True Mula (Chandra Hari) | *Indian J. History of Science* 33(4), 1998 | — |
| 11 | Galactic Centre at 0° Sagittarius | Gordon, de Witt & Jacobs (2023), *AJ* 165, 49 | DA-9 |

Every removed name **hard-errors on parse** with a message saying what happened
to it, and never silently maps to a neighbour or a default. A silent remap would
move a caller's chart without telling them, which is the failure mode of the
whole episode in miniature. The dispositions are in `sidereal.rs` and are tested
for reachability.

### Why the other thirty-six names went

Vedaksha 5 had 44 variants. Seven sidereal names plus `Tropical` still parse, so
**36 names needed a disposition**: 8 collapsed into a system that survives, 4 point
at a successor answering a different defining condition, and **24 were dropped with
no successor at all**. Those counts are asserted by a test, because three documents
previously quoted three different numbers from the same table.

The 24 fall into two categories, and the second is the interesting one.

**No primary located, and the search recorded.** *A drop is only final when the
spec records what was searched and where; an undocumented negative search is not
a finding.* De Luce, Usha-Shashi, Gil Brand, Wilhelm's mid-Mula and Sassanian are
**unverified, not refuted** — their definitions exist in books that must be
bought and read. Aryabhata's two variants went because the Aryabhatiya contains
no precession rule at all: its sole basis is a second-hand quotation that the
text's own editor says he could not find in the text.

**The definition/determination test**, which the Huber case forced into the open:

> A system qualifies only if someone **stipulated** a zero point. It does not
> qualify if a scholar **estimated** where a historical zero point lay.

A stipulation is exact by construction, cannot be wrong, and does not change when
new evidence arrives. A determination carries uncertainty, has a revision
history, and is *superseded* rather than amended. The evidence that Huber's
figure is the second kind is in the disagreement itself: two secondary accounts
give 4°22′ and 4°28′ for the same zero point. **A definition does not have
competing values; a measurement does.** That test drops the Babylonian systems as
a class — Huber, Kugler ×3, ETPSC — and would exclude Britton too.

If a Babylonian system is wanted as a product feature, the route is to obtain
Britton (the current determination), ship it as an explicitly modern scholarly
reconstruction with its uncertainty stated, and accept that it will be
superseded. That is a product decision, not a provenance one.

## What came out that nobody was looking for

Three results that no acceptance criterion asked for, and that no amount of
tuning would have produced.

**True Chitra and the Indian official system invert to the same year.** IAE 2022
states its initial point "coincides with the vernal equinoctial point of vernal
equinox day of 285 A.D.". Inverting the IAE J2000 anchor gives **285.7 CE**.
Inverting True Chitra — Spica held at 180°, a live star from a modern catalogue,
sharing no input with IAE's polynomial — gives **285.4 CE**. Two derivations with
nothing in common landing on a year that one of them documents.

**A 19th-century commentary's dates are reproduced to three years.** Burgess
records that ζ Piscium met the vernal equinox in A.D. 572, and that the Surya
Siddhanta's own zero point (10′ east of it) did so about A.D. 560. Hipparcos
astrometry plus P03 gives **575 CE** and **563 CE** — and reproduces the
twelve-year separation, which is what 10′ at ~50.2″/yr must be. This is also what
settled a contradiction inside the spec; see §11.2.

**The Surya Siddhanta's three numbers only agree under one reading.** The text
gives amplitude 27°, rate 54″/yr, and 600 revolutions per Mahayuga
independently. Under linear (zigzag) folding, 27° at 54″/yr takes exactly 1800
years and the quarter period is 1800.04 years. Under a sinusoidal reading the
rate would be a maximum rather than a constant and the amplitude would not land
on the quarter period. **Burgess's zigzag is not merely his interpretation; it is
the only reading under which the text agrees with itself.**

## What the tests can and cannot establish

Stated plainly, because the alternative is letting a green suite imply more than
it proves.

- **§6.1 anchor reproduction** (every system returns its own defining value to
  ≤1e-9°) tests *self-consistency*. A reverse-engineered anchor reproduces itself
  perfectly. This criterion cannot detect contamination.
- **§6.2 zero-year inversion** is the only criterion with discriminating power,
  and it only covers systems whose primary documents a zero year.
- **§6.3 astronomy cross-check** against ERFA validates precession, obliquity and
  the star transform — *not* ayanamshas.

**Derivation integrity is established by the audit trail — sanitised spec,
recorded inputs, runnable generator — not by the test suite.** No one should
later claim the tests prove independence.

What §6.7 adds is that the derivation stays *executable*:
`scripts/generate_ayanamsha.py` re-derives every system in Python from
`derivation-inputs.json` and emits the fixture the Rust engine is asserted
against. The two implementations share no code. They agree to **5.7e-14° over
192 comparisons spanning 6000 years**, which is not proof of provenance but is
proof that no constant entered by being typed. `--verify` needs no network, so —
unlike the coefficient-blob jobs — it cannot pass vacuously.

## Files

| File | Role |
|---|---|
| `spec.md` | The specification. The **only** input the implementation was permitted. §11 records every correction made against it during implementation. |
| `derivation-inputs.json` | Every primary value the derivation consumes, with its citation. The generator reads this; the engine encodes it independently. **Change this, never the Rust constants.** |
| `vizier-fetch-manifest.txt` | What was fetched, from where, with sha256s — and what was deliberately not fetched. |
| `implementation-transcript.md` | What the implementation actually did, including the two claims in the spec that turned out to be false. |
| `migration-note.md` | The §6.4 post-freeze comparison, and what callers must change. Written **after** the freeze commit. |

## Standing constraints

- Never open `sweph.h`, at any point, for any reason — including "just to check".
- `spec.md` is normative. Any change to an anchor, model, convention or declared
  assumption is a change to the derivation and belongs there first, not in the
  generator and not in the Rust.
- No ayanamsha value may ever be compared against another implementation's
  output, in a test or anywhere else.
- This is engineering. The disclosure and licensing questions raised by the
  original exposure are the project owner's, not settled here.
