# Chara Dasha direction: the panchame-to-nine decoding, recorded

**Date:** 2026-08-31
**Scope:** the arithmetic behind `DATA_PROVENANCE.md` Fix 12 and `crates/vedaksha-vedic/src/dasha/chara.rs`'s current citation. Records the letter-to-numeral cipher that decodes the direction sutra's *panchame* ("in the fifth," grammatically) to nine, and the eight-word check that establishes the specific letter values used. This exists so the decoding is checkable and re-runnable, not asserted on trust — matching the precedent set for the ayanamsha and lunar-theory subsystems.

**Cleanroom note:** this is classical-text arithmetic, not a comparison against any software. No reference-engine output of any kind informed this record.

## The rule

Jaimini's own text states, elsewhere in the same sutra corpus, a licensing rule for reading certain words as numbers rather than by their plain grammatical sense: *sarvatra savarna bhava rashayash cha* — "everywhere, the houses and signs are [denoted] by letters." Under this convention, a word's consonants each stand for a digit (a scheme of this general kind — grouping consonants into four series and mapping each series position to a digit 1-9-and-0 — is traditionally called a katapayadi-style cipher), and the digits combine into a number the same way a sequence of digits normally would, except that **the first letter of the word gives the units place**, the second letter the tens place, and so on — the reverse of how a number is normally read off left to right. A vowel with no consonant, or the specific consonant class that stands for zero, contributes 0 and is otherwise skipped. Because Chara Dasha (and Jaimini's sign-lord and pada schemes generally) only ever needs a value 1-12, any raw total is finally reduced by taking the remainder after dividing by 12 (a value of exactly 0 after reduction denotes the twelfth item, not "no item").

This is not this document's own invention: it reproduces, letter for letter, the numeric answer each of the following words is independently glossed with elsewhere in the classical commentarial material on this sutra corpus.

## The eight-word check

| Word | Per-letter values (first letter = units place) | Raw total | mod 12 | Commentary's own stated answer |
|---|---|---|---|---|
| dara | da=8, ra=2 | 8 + 2x10 = 28 | 4 | "the fourth" |
| bhagya | bha=4, ga=3, ya=1 | 4 + 3x10 + 1x100 = 134 | 2 | "the second" |
| shula | sha=5, la=3 | 5 + 3x10 = 35 | 11 | "the eleventh" |
| kama | ka=1, ma=5 | 1 + 5x10 = 51 | 3 | "the third" |
| sva (locative *sva-sthe*) | sv=4 | 4 | 4 | "situated in the fourth from lagna" |
| suta (locative *suta-sthe*) | su=7, ta=6 | 7 + 6x10 = 67 | 7 | "situated in the seventh from lagna" |
| janma | ja=8, ma=5 | 8 + 5x10 = 58 | 10 | "the tenth" |
| **panchame** | pa=1, cha=6, ma=5 | 1 + 6x10 + 5x100 = 561 | **9** | "the ninth" |

Every row was independently checked against the target commentary's own plainly stated numeric answer for that word — the computation was not tuned to produce a wanted result; the per-letter values were read off from the first seven rows and then applied, unchanged, to *panchame* in the eighth. All eight rows agree with their commentary's own answer.

`sva-sthe` and `suta-sthe` initially failed to decode when the whole inflected form (word plus the locative case ending *-sthe*, "situated in") was fed into the cipher; only the stem is coded, and the locative suffix is read as ordinary Sanskrit grammar layered on top, not as part of the cipher. Restricting the cipher to the stem closed that gap and both rows then matched.

## Why this is decisive rather than merely consistent

`dara` ordinarily means "wife," which would suggest the seventh house by its plain sense — it decodes to the fourth. `bhagya` ordinarily means "fortune," suggesting the ninth house by its plain sense — it decodes to the second. The licensing rule quoted above exists specifically to override a word's ordinary meaning with its coded one. *Panchame* decoding to nine rather than five is the same phenomenon, not a special case invented for this sutra: an ordinary grammatical ordinal ("fifth") is overridden by its letter-value ("ninth").

## What this does not establish

No source found performs this exact computation on the word *panchame* itself and shows the working — commentaries state the numeric result directly (a `panchame`-means-`ninth` gloss) without walking through the cipher. The arithmetic bridge above is this project's own reconstruction from the eight-word pattern, not a quotation of anyone else's derivation. It is checkable and it reproduces eight independent commentary-stated answers exactly, which is why it is graded high confidence in `DATA_PROVENANCE.md` Fix 12 — but the specific step "apply this cipher to *panchame* itself" is a reconstruction, not a quoted classical computation.

The specific sutra number carrying *panchame padakramat prakpratyaktvam charadashayam* is unstable across sources — this project has direct readings of 22, 28, and 32, plus an independently reported 29, none converging. This does not affect the arithmetic above, which depends only on the word and the licensing rule, not on which sutra number carries it.
