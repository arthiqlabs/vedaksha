# Implementation transcript

**Date:** 2026-08-18
**Model:** Claude Opus 5 (1M context)
**Branch:** `cleanroom/ayanamsha-quarantine`
**Authoritative input:** `spec.md` in this directory, and nothing else.

This is the honest record of what the implementation did, including the parts
where the spec turned out to be wrong. It is committed because an audit trail
that only records successes is not an audit trail.

---

## Firewall, as actually enforced

The precedent's two-agent firewall put the implementer in a separate worktree
with a separate system prompt. Here the arrangement was different in one respect
and should be stated rather than glossed: **the implementation ran in the same
session that read the spec, not as a separately-prompted agent.** What made that
acceptable is that the firewall in this work is structural rather than
procedural — the target was already gone.

| Control | Status |
|---|---|
| Working tree contained no ayanamsha value | Yes. All forty-four were deleted in `4033c6a`, before this work began. |
| Git history of `sidereal.rs` consulted | No. `4033c6a` was inspected with `git show --stat` only; no `-p`, no `git diff`, no `git show <ref>:<path>`. |
| Swiss Ephemeris source, headers, docs or output | Not opened. Not at any point, for any purpose. |
| Any other astrology software or its output | Not consulted. |
| Subagents dispatched | Four, for repository surface-mapping and for Sgr A* literature. Each carried an explicit instruction not to read the quarantined file's history and not to report historical constants. None returned one. |
| Spec leak check | `scripts/check_spec_hygiene.py` run against `spec.md`, `derivation-inputs.json` and this directory's `README.md`. Clean — 208 value patterns and 7 phrase patterns, zero hits. Now enforced in CI. |

The one number in the engine that this project computed rather than read from a
primary is Yukteshwar's epoch, the 1894 March equinox instant, which DA-6
explicitly asks to be computed. It is pinned by a test that recomputes it from
the solar theory, so it cannot survive as a typed constant.

## What was built

**In `vedaksha-ephem-core`:**

- `precession::general_precession_p03` — `p_A`, which the engine did not have.
  See §11.1; the spec believed it did.
- `precession::precession_matrix` — rotation order corrected. See §11.7.
- `stars` — a new module: catalogue astrometry, proper-motion propagation, and
  the ICRS → mean-ecliptic-of-date transform. There was no star machinery in the
  workspace at all.

**In `vedaksha-astro`:**

- `sidereal.rs` rewritten wholesale. Eleven systems plus a tropical identity, a
  canonical `FromStr`/`Display`, and a disposition table that makes every retired
  name hard-error with an explanation.

**Around it:**

- `scripts/generate_ayanamsha.py` — a second, independent derivation in Python.
- `crates/vedaksha-astro/tests/fixtures/ayanamsha.json` — the contract between
  the two.
- Both surfaces' string parsers replaced by delegation to the engine's own, which
  also fixed a pre-existing divergence: the wasm and MCP parsers had drifted
  (they disagreed about `"tropical"`), and between them reached five of the
  forty-four systems.

## Verification, and what it is worth

| Check | Result |
|---|---|
| §6.1 anchor reproduction, all systems | Exact to ≤1e-9° |
| §6.1 for star systems — the star lands on its assigned longitude | ≤1e-6″ at four epochs each |
| §6.2 zero-year inversion, where the primary documents one | Indian official → 285.7 CE (documented 285); Yukteshwar → 500.0 (documented 500); Raman → 397.0 (documented 397); Surya Siddhanta → 499.2 (documented 499) |
| §6.2 exemption | KP only, for the reason §1.3.4 gives. Recorded in `derivation-inputs.json`, not silently skipped. |
| §6.3 astronomy vs ERFA | `p_A` to machine precision at six epochs; the star transform to <0.01″ against `eraEqec06` across six millennia |
| §6.7 regeneration | Rust vs Python: **5.7e-14° worst case over 192 comparisons**, 6000-year span |
| Cross-corroborations nobody asked for | True Chitra and the Indian official system invert to the same year, 285 CE, from disjoint inputs. Revati-paksha reproduces Burgess's A.D. 572 to three years, and the SS/Revati twelve-year separation exactly. |

**What none of this establishes.** §6.1 tests self-consistency and a
reverse-engineered anchor passes it perfectly. §6.3 validates astronomy, not
ayanamshas. Only §6.2 discriminates, and only for four systems. The audit trail
is the evidence of provenance; the tests are evidence that the derivation is
executable and that no constant was typed. Those are different claims and the
second does not imply the first.

## Where the spec was wrong

Six corrections, recorded in full as `spec.md` §11. Two are worth calling out
here because of what they say about the process.

**§11.1 — the precession function.** The spec asserted, twice, that the engine
already implemented the polynomial the Indian official system needed, and used
that to conclude no computational work was required. It implemented a different
angle, 9.7″/century away. The spec even quoted the right coefficient
(−0.000023857) next to a function carrying a different one (−0.000026452) without
the mismatch being noticed. **A frame error is indistinguishable from a wrong
constant, and this one was hiding inside a sentence arguing that nothing needed
checking.**

**§11.2 — Revati-paksha's longitude.** The normative anchor table and the prose
disagreed by 10′, and the prose had a primary quotation behind it. Rather than
pick, the derivation ran both readings out to their zero years and compared
against two dates the commentary states independently. Both readings matched a
stated date; the pairing identified which belonged to which system. That is the
spec's own §6.2 machinery resolving an ambiguity in the spec.

## Deliberate omissions

- **No locale table for ayanamsha names.** Filling one would mean inventing
  translations. See §11.8.
- **No legacy oracle.** Deliberately unlike the ELP precedent; see the README.
- **The three superseded `DATA_PROVENANCE.md` entries were not rewritten.** They
  are marked superseded with a pointer, and left otherwise untouched. Deciding
  how to characterise the historical exposure is the project owner's call, not an
  implementation detail, and editing a provenance record after the fact is not a
  neutral act.

## Incidental defects found and fixed

- `precession_matrix` rotation order (§11.7) — 0.56″ at 499 CE. Pre-existing,
  affects every historical planetary position, found only because the star
  transform was checked against ERFA far from J2000.
- `compute_vargas` advertised an `ayanamsha` parameter its handler never read.
- The wasm and MCP name parsers had drifted from each other and from the engine.
- Three `riseset` scan-oracle literals were stale after the precession fix. They
  were **regenerated from the scan reference**; no tolerance was relaxed.
