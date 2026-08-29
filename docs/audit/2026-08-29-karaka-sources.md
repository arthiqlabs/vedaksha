# Chara-karaka sources — Rahu inclusion and degree-reflection convention

**Date:** 2026-08-29
**Scope:** primary-source research behind `crates/vedaksha-vedic/src/karaka.rs`'s citation
(rewritten in v7.6.0 — see `CHANGELOG-v7.6.0.md` and `DATA_PROVENANCE.md` Fix 10). This audit doc
itself is added in v8.0.0, as the module's first dedicated derivation record; the citation work it
documents already shipped.
This is the derivation record the module's own citation summarizes; it exists so the claim can
be checked and re-run, matching the precedent set for the ayanamsha and lunar-theory subsystems
(`docs/audit/2026-08-17-ayanamsha-cleanroom/`, `docs/audit/2026-05-09-elp-mpp02-cleanroom/`).

**Cleanroom note:** this research was conducted entirely from classical-text sources (a
scholarly English rendering of the Jaimini Sutras, cross-checked against independent secondary
commentaries and Parashara's own text). No reference-engine's output, source, or behavior was
consulted at any point, and the researcher was not told what any independent comparison had
found — the questions below were investigated cold, from primary text alone. Per this project's
source-citation hard gate, no modern translator, commercial edition, or page number is named
below — only classical-text names and, where attested, sutra/chapter numbers.

---

## What was consulted

- A full sutra-by-sutra scholarly English rendering of the Jaimini Sutras, Adhyaya 1 Pada 1
  (Sanskrit transliteration + gloss + explanatory notes attached to each sutra), read directly
  as the primary source for Jaimini's own wording.
- Several independent secondary discussions cross-checked against it: a differently-numbered
  sutra-by-sutra commentary, a Jaimini-specialist discussion of the 7-vs-8-karaka question, and
  summaries of Parashara's *Brihat Parashara Hora Shastra* (BPHS) chapter on Karakatwas.

## Question 1 — Does Jaimini's own text include Rahu, or is the 8th karaka a later addition?

**Finding, high confidence:** Jaimini's own root sutra defining Atmakaraka states both
alternatives in one line — "of the seven planets Sun-to-Saturn, **or** the eight planets
Sun-to-Rahu, whichever has traversed the most degrees, becomes Atmakaraka." The 8-karaka
scheme with Rahu is not a post-Jaimini invention; Jaimini's own sutra names Rahu as an
admissible 8th candidate.

**Nuance, not flattened away:** the primary text's explanatory notes describe Rahu's role as
*conditional* — invoked specifically to resolve a tie among the seven main grahas ("if two or
three planets obtain the same degrees and minutes, they are all merged into one Karaka... the
vacancies... have to be supplied by Rahu in reverse order"), not as an unconditional 8th slot
filled on every chart. This differs from how modern software (Vedaksha included) implements the
scheme — Rahu is always included when the 8-karaka scheme is requested, not only on a tie. A
genuinely old commentarial dispute exists here too: at least one classical Jaimini commentator
(Neelakantha, per secondary summary) favored dropping Rahu and using only 7.

**Not resolved:** which reading — unconditional-8, or 7-with-tie-break-8th — is "more correct"
to Jaimini. This is reported as an open, longstanding internal disagreement, not adjudicated
here. Vedaksha's current behavior (unconditional inclusion when scheme="8") matches the common
modern convention, not a specific textual mandate; this is a disclosed choice, not a resolved
question.

## Question 2 — Is Rahu's degree read directly, or reflected (30° − degree)?

**Finding, moderate-high confidence:** the *principle* of reading a retrograde/nodal body in
reverse is textually attested in Jaimini's own text — used explicitly, a few sutras earlier in
the same Pada, for Ketu's Argala ("Apasavyam"/"Apradakshinam", i.e. counting against the other
seven grahas' forward motion). Applying that same principle to the Atmakaraka-degree rule, the
primary text's own notes state Rahu/Ketu "will be considered as getting the highest number of
degrees when they are at the beginning of a sign" — functionally the same effect as reflecting
the degree (30 − actual, so a low actual degree from the sign's start reads as a high effective
degree for a body moving backward).

**Nuance, not flattened away:** this reverse-reading logic appears in the sutra's *explanatory
notes*, not as a separately numbered sutra spelling out an arithmetic formula. No sutra, in
either numbering scheme consulted, states "30 minus degree" as sutra text. The literal
arithmetic — "deduct [Rahu's] longitude in that sign from 30" — is attested instead in
Parashara's BPHS, in its chapter on Karakatwas (commonly numbered ch. 32), a different, later
text than Jaimini's own. One secondary Jaimini-focused discussion explicitly noted that
Jaimini's own text "has not shown the case of Rahu becoming the higher-longitude planet after
deducting from 30 degrees" — i.e. the numeric operationalization is not spelled out by Jaimini
himself, even though the underlying principle is his.

**Conclusion:** the reflection convention is a textually-grounded application of an attested
Jaimini principle, with its literal arithmetic attested in a different classical text (BPHS),
not a modern software invention and not itself verbatim Jaimini. Vedaksha's implementation
(`rahu_degrees_in_sign`, `30.0 - d`) follows BPHS's explicit formula, cited as such — not
attributed to Jaimini's own sutra wording, which does not state it.

## Question 3 — Specific sutra number for the degree-ranking rule

**This is the one place research found a real inconsistency rather than a single answer.** The
primary sutra-by-sutra text numbers the karaka chain starting at Atmakaraka = sutra 11 (of
Adhyaya 1, Pada 1); a second, independently-numbered secondary commentary numbers the identical
sequence of rules two positions later, Atmakaraka = sutra 13. The *content and sequence* of the
rules (Atmakaraka → Amatyakaraka → Bhratrukaraka → Matrukaraka → Putrakaraka → Gnatikaraka →
Darakaraka, each the next-highest degree after the previous) is identical and corroborated
across both numbering schemes; only the specific integer labels differ by which
commentarial/editorial numbering tradition is followed.

**Not resolved:** no single universally-agreed sutra number could be corroborated. The module's
citation states this as "commonly numbered around 1.1.11, though the exact numbering varies by
commentarial tradition" rather than asserting one number as if uncontested.

## What shipped, and what did not change

- `crates/vedaksha-vedic/src/karaka.rs`'s citation states the ranking-chain source (Jaimini
  Sutras, Adhyaya 1 Pada 1) and the reflection-arithmetic source (Parashara's BPHS, Karakatwas
  chapter) separately, and states the two open questions above explicitly (sutra numbering;
  conditional-vs-unconditional Rahu inclusion) rather than presenting the current behavior as
  textually settled.
- No ranking behavior changed as a result of this research. The 8-karaka scheme still includes
  Rahu unconditionally when requested, and `rahu_degrees_in_sign` still reflects the degree —
  both are legitimate, textually-grounded choices per the findings above, not adjustments made
  to match any external comparison.
- Separately, and independently of this research, `rahu_degrees_in_sign`'s boundary check was
  hardened from exact floating-point equality to an epsilon tolerance (see `CHANGELOG-v7.6.0.md`
  and `DATA_PROVENANCE.md` Fix 10) — a code-quality fix, not a convention change.
- All three of the above shipped in v7.6.0. This audit doc is new in v8.0.0.

## Explicit gaps, stated rather than papered over

- No single canonical sutra-numbering scheme for Adhyaya 1 Pada 1 could be confirmed.
- The unconditional-vs-tie-break-only question for Rahu's 8th-karaka inclusion is a live,
  unresolved traditional dispute this project does not adjudicate.
- This research was web-based secondary/primary-text reading, not a review of a critical
  physical edition — reasonable effort for this question, not the same rigor as the dedicated
  ayanamsha re-derivation. That is exactly why the citation states its open questions plainly
  rather than asserting a false precision.
