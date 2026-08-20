# Vedaksha v6.1.0

**Every value from the analytical path changes.** No API changes, no renames, no migration
steps — but if you compute with `AnalyticalProvider`, or through any surface that does (WASM,
the Python package, edge deployments), recompute anything you have stored.

Positions were never wrong by an astrologically visible amount. **Timing was.**

---

## The observer was in the wrong place

Two independent defects displaced the observer from the Earth's centre.

**1. The analytical provider answered "Earth-Moon barycentre" with the Earth.** VSOP87A's `ear`
series is the Earth's centre. `coordinates::earth_state` derives the Earth *from* the
barycentre by subtracting the Moon term — so it subtracted that term from a position which
never contained it, putting the observer about **4,671 km** out. Analytical path only.

**2. `earth_state` used the wrong divisor.** The Moon state it receives is relative to the
barycentre, which requires dividing by `EMRAT`; it divided by `1 + EMRAT`. A further **56.8
km**, and this one reached **every provider, the SPK path included**.

Both errors scale as 1/geocentric-distance. That is why the inner planets looked far worse than
the outer ones, and why the Moon looked clean throughout — the lunar path scales the
barycentre-relative vector directly and never calls `earth_state`.

### What it measures now

Mean apparent ecliptic longitude against JPL Horizons (DE441), 1,535 dates per body, 1900–2025:

| body | 6.0.1 | 6.1.0 |
|---|---|---|
| Sun | 4.091″ | **0.180″** |
| Venus | 4.828″ | **0.180″** |
| Mercury | 4.267″ | **0.180″** |
| Mars | 3.060″ | **0.178″** |
| Jupiter | 0.805″ | 0.239″ |
| Saturn | 0.460″ | 0.251″ |
| Uranus | 0.334″ | 0.267″ |
| Neptune | 0.503″ | 0.503″ |
| Moon | 0.169″ | 0.169″ |

Analytical overall **2.058″ → 0.239″**, worst case **24.223″ → 1.896″**. Neptune is unchanged
because at 30 AU the observer error was never significant; that residual is the theory.

The SPK path improved from the second defect alone: **0.106″ → 0.103″** mean.

### Why the timing mattered more than the positions

The Sun moves 0.0411″/s. A 4.09″ mean error is about **100 seconds of time** on a solar
ingress, with a worst case near 170 s. Anything derived from an ingress inherited it —
saṅkrānti, and through it masa, samvat, and ingress-timed muhurta. Sunrise moved too: 0.44 s at
Chennai, and far more at high latitude where the Sun crosses the horizon slowly.

### How it was found

An accuracy report from a consumer, measuring inner-planet residuals against the accuracy
Bretagnon & Francou publish for VSOP87A itself — no comparison to any other implementation.
Their figures were reproduced from our own committed oracle before their reasoning was read.

Two candidate explanations were refuted by measurement rather than argument:

- **Truncation.** Regenerating the VSOP87A coefficient set at a threshold 100× finer produced
  **byte-identical** blobs. The shipped set was never the constraint.
- **The time argument.** The Moon moves ~13× faster than the Sun, so a clock error large enough
  to put the Sun 4″ out would have put the Moon ~55″ out. The Moon measured 0.169″.

The Moon being *clean* is what located the defect.

## Pinned literals regenerated

Six moved with the correction, each regenerated from its own reference rather than from the
code under test, with tolerances untouched: the real-Sun scan oracle (via
`record_the_real_sun_scan_oracle_table`, which drives the brute-force scan, not the analytic
path), two polar rise/set pins, a muhurta sunrise pin, the WASM Rahu/Gulika windows, and
`analytical_bit_digest`. The superseded digest is retained as `PRE_OBSERVER_FIX_DIGEST`.

## Documentation

`README.md` described the old 2.06″ as VSOP87A being "a truncated theory, necessarily looser
than the numerical kernel". That attributed our defect to the theory, and it is corrected.

## Validation

1,059 tests pass, 0 fail, across 34 suites — release profile, `--include-ignored`, with the
DE440s kernel present. The Python conformance fixture was regenerated and its values moved,
which is the evidence that this reaches the packaged WASM surface and not only the native
build.
