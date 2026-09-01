# Mean node divergence: resolved, not a Vedaksha defect

**Date:** 2026-09-01
**Engine version:** 9.1.0
**Status:** CLOSED. Supersedes the "still open" state recorded in
`2026-08-29-lunar-node-theory-review.md` (Finding 2).

## The finding, as originally reported

An external parity-testing project measured Vedaksha's `MeanNode` **sidereal**
longitude against an independent third-party implementation over 168 evaluable
birth instants, 1899-1999 CE, with Vedaksha's `TrueChitra` ayanamsha configured
to match that implementation's own convention. The signed difference correlated
with time almost perfectly linearly:

```
delta ~= -50.35 x (years from J2000) - 2.5 arcsec      r = 1.00
```

The magnitude is suspiciously close to Vedaksha's own documented general
precession rate (`5_028.796_195"` per century = 50.288"/yr, `precession.rs`),
which made a J2000-versus-of-date frame mixup the obvious hypothesis.

## Conclusion

**Vedaksha's mean node is correct in frame and in secular rate.** Four
independent lines of evidence support this, one of them external and
convention-free. No code change was made, and none is warranted.

## Evidence

### 1. No secular drift against an externally-validated instantaneous node

`nodes::mean_node` was regressed against `nodes::true_node_osculating` over
1600-2400 CE (n = 9,740, 211-day step, chosen to be resonant with neither the
draconic month nor the year so the periodic term averages out rather than
aliasing into an apparent trend). Both are documented as referred to the mean
ecliptic and equinox of date; `true_node_osculating` is independently validated
to 0.6" against JPL Horizons.

```
measured slope   -0.0028 arcsec/yr
control slope   +50.2913 arcsec/yr   (same computation vs true_node_osculating_j2000)
```

Two functions in one frame differ by a bounded periodic term with no secular
part; two a precession apart differ by ~50.29"/yr. The **control is the
load-bearing half**: it demonstrates the test detects the precise error class
being hypothesised. A flat result from a test that cannot fail would prove
nothing.

Now a committed regression test,
`node_frame.rs::mean_node_carries_no_precession_trend_against_the_osculating_node`,
verified by mutation: planting a J2000 frame shift into `mean_node` makes it
report -50.2655"/yr and fail. It detects secular drift above 2"/yr.

### 2. Independent primary-source cross-check of the rate

`nodes.rs` transcribes Meeus, *Astronomical Algorithms* 2nd ed., Ch. 47 eq.
47.7. `nutation.rs` separately transcribes the IAU 2000 fundamental argument
Omega, from a different primary source. Across 1600-2400 the two agree to
**0.38 arcsec per century** (0.0038"/yr) -- roughly 13,000 times smaller than
the divergence under investigation.

### 3. The divergence is isolated to the node, ruling out the ayanamsha

The parity project re-ran its original measurement on a fresh independent
sample (200 birth records, 168 evaluable, seed 20260829), adding the seven
grahas alongside the node:

| Body | n | slope (arcsec/yr) | r |
|---|---|---|---|
| MeanNode | 168 | -50.32 | -0.9999 |
| Saturn | 167 | 0.053 | 0.079 |
| Moon | 168 | 0.054 | 0.083 |
| Jupiter | 167 | 0.047 | 0.072 |
| Mars | 168 | 0.045 | 0.069 |
| Mercury | 168 | 0.042 | 0.065 |
| Venus | 168 | 0.042 | 0.064 |
| Sun | 168 | 0.040 | 0.062 |

Every graha shows a slope two to three orders of magnitude smaller and,
decisively, **essentially no linear relationship at all** (r ~ 0.06-0.08,
scatter about zero rather than a weak shared trend). A shared
ayanamsha-realisation difference would have to produce a comparable trend on
every body in that frame. It does not. The sidereal conversion is therefore not
implicated.

### 4. External arbiter: no drift against the physical node derived from a kernel

This is the strand neither party could produce alone, and it is the one that
closes the question. Vedaksha cannot build it: the only candidate references
are kernels it would then have to trust as an oracle for the very quantity in
dispute. The parity project can.

**Why this is possible at all, given that validating a node against a kernel is
normally ill-posed.** What a kernel cannot supply is the *definitional* choice
-- which smoothing of the real, wobbling node counts as "the" mean node. What it
*can* supply, through its state vectors plus standard orbital mechanics, is the
**osculating** node of the actual orbit, which requires no convention. And while
the difference between a mean node and the physical node is
convention-dependent, in every convention it must be **bounded and periodic with
no secular term** -- a smoothing cannot drift away from what it smooths. Absence
of drift is therefore testable without adopting anyone's definition, even though
the value is not.

Method: at 1,801 instants (1900-01-01 to 2050-01-01, ~30-day spacing) the Moon's
Earth-relative state vector was read from DE440 and the osculating ascending-node
longitude computed directly from it (orbital angular momentum projected onto a
reference plane). Expressed in two frames via a general-purpose frame-rotation
library, and regressed against years from J2000 versus each side's **tropical**
mean node (ayanamsha-free on both sides).

```
CONTROL (fixed J2000 ecliptic, deliberately NOT precession-corrected):
  vs Vedaksha mean node      -50.4448 +/- 2.1091 arcsec/yr   (n=1801)
  vs the other implementation -50.4738 +/- 2.1091 arcsec/yr   (n=1801)

MAIN (mean ecliptic and equinox of date, properly precession-corrected):
  vs Vedaksha mean node       -0.0359 +/- 2.1074 arcsec/yr   (n=1801)
  vs the other implementation -0.0649 +/- 2.1074 arcsec/yr   (n=1801)
```

The control lands within 0.3% of the standard general-precession figure and is
more than 23 standard errors from zero: the method demonstrably detects a
precession-scale frame error. The main result is within 0.05 standard errors of
zero and roughly 700 times smaller than the control.

Sanity check on the pipeline itself: the residual spans roughly -8,900 to
+11,000" before correction and -7,000 to +7,000" after, matching the known
1.5-2 degree libration of an osculating node about its mean. The pipeline is
measuring the intended physical quantity, not an artefact.

A 150-year baseline was used rather than this project's 800-year one, a
deliberate choice by the parity project: it keeps every instant inside a range
already trusted, avoiding new extrapolation risk on a sensitive question, and
150 years is ample leverage given r = 0.9999 was obtained over roughly 100.

**Neither implementation's tropical mean node drifts against the physical node.**

## Where the divergence originates

Since both tropical mean nodes track the physical node to within noise, while
the *sidereal* values diverge at ~50"/yr, the difference necessarily enters at
the sidereal conversion for the node -- and, per strand 3, not through an
ayanamsha applied uniformly to all bodies.

The parity project reported one further output-level observation. The
third-party implementation's separately-reported ayanamsha value is very nearly
constant over time (measured at 16 points, 1900-2050 at 10-year spacing: slope
-0.0256"/yr). Algebraically, if its reconstructed tropical value tracks the
properly-precessing physical node while its own reported ayanamsha barely moves,
its sidereal output must advance at close to the tropical rate rather than a
precession-corrected sidereal rate -- which accounts for a gap of very nearly the
observed magnitude.

This is recorded as an **output-level observation only**, obtained the same way
any other output of that implementation is: as a sealed box. It is not a claim
about how that implementation works internally, and this project makes none.
Nothing here was derived from any third-party source, header, or documentation.

## Correction to earlier reasoning in this repository

`2026-08-29-lunar-node-theory-review.md` recorded the J2000-frame hypothesis as
ruled out on the strength of a self-consistency check comparing Vedaksha's
published PyPI package against direct source evaluation.

**That check had no power to detect this error.** Both sides run the same
formula, so a frame error would be present in both and cancel exactly. It could
only ever have caught a stale build, which is a different question. The
hypothesis was recorded as eliminated while remaining untested, and was only
genuinely tested by strand 1 above.

Generalisable lesson, consistent with this repository's existing
`test-the-value-not-the-mechanism` and `verify-the-gate-sees-the-change`
findings: **a check that compares an implementation against itself cannot
falsify a hypothesis about that implementation's shared assumptions.** Ruling
something out requires an oracle that does not inherit the assumption under
test, and a control demonstrating the test can fail.

## What is NOT established

- **Absolute accuracy of the mean node's value.** Strands 1 and 4 bound a
  *secular* frame error (to ~2"/yr in the external test). Neither constrains the
  bounded periodic difference between a mean element and an instantaneous one,
  which is order 1.5-2 degrees by construction and expected.
- **Which node convention is "correct" for sidereal jyotish use.** These tests
  establish that Vedaksha's mean node is self-consistent, correctly framed, and
  tracks the physical node without drift. Whether its convention is the one a
  given tradition or consumer expects is a separate question that measurement
  cannot settle.
- **Anything about why the other implementation behaves as observed.** Out of
  scope by clean-room policy and not investigated.

## Practical exposure for downstream consumers

At ~50.3"/yr the divergence reaches roughly **1.40 degrees at a 1900 birth
date**. A nakshatra pada is 3 degrees 20 minutes, so a discrepancy of that size
can move Rahu or Ketu across a pada boundary, and occasionally a nakshatra
boundary, for charts a century or more from J2000.

Downstream consumers comparing Rahu placement against other software for
historical dates should expect to see this, and it will superficially resemble a
Vedaksha defect. It is not. Direct enquiries to this document.

## Related

- `docs/audit/2026-08-29-lunar-node-theory-review.md` -- original finding
  (Finding 2 superseded by this document; Finding 1, TrueNode, was resolved in
  v7.5.0)
- `docs/audit/2026-08-29-lunar-node-self-consistency-results.md` -- the
  self-consistency check whose scope is corrected above
- `crates/vedaksha-ephem-core/tests/node_frame.rs` -- the committed regression
  guard from strand 1
