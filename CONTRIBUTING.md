# Contributing to Vedaksha

Thank you for your interest in contributing to Vedaksha. This document outlines
the requirements for all contributions.

## Contributor License Agreement (CLA)

All contributions to Vedaksha require a signed Contributor License Agreement.
By submitting a pull request, you agree to the following:

- You grant ArthIQ Labs LLC a perpetual, worldwide, non-exclusive, royalty-free
  license to use, reproduce, modify, and distribute your contribution under the
  terms of the BUSL-1.1 license (and, after the Change Date, under the Apache
  License 2.0).
- You represent that you have the right to grant this license.
- You understand that your contribution will be licensed under the same terms
  as the rest of the project (BUSL-1.1).

The CLA does NOT transfer copyright ownership of your contribution to ArthIQ
Labs LLC. You retain copyright of your original work.

## Code Quality Requirements

All contributions must:

1. **Pass all tests:** `cargo test --workspace`
2. **Pass clippy with zero warnings:** `cargo clippy --workspace -- -D warnings`
3. **Be formatted:** `cargo fmt --all -- --check`
4. **Include tests** for new functionality
5. **Include doc comments** on all public items

## Source Citation Requirement

Every public function must include a `///` doc comment citing its primary source.

```rust
/// Compute the precession matrix from J2000 to the mean equator of date.
///
/// Source: Capitaine, Wallace & Chapront (2003), A&A 412, pp. 567-586.
pub fn precession_matrix(jd: f64) -> Matrix3 { ... }
```

Acceptable sources: NASA JPL, IAU standards, published academic papers, the
classical Sanskrit texts (BPHS, Jaimini Sutras, Surya Siddhanta), and standard
astronomical references such as Meeus "Astronomical Algorithms" **for frame
transforms, time scales and numerical method only** — never for the value of
anything a classical text or an issuing authority defines.

Two rules follow, and they guard different things.

**Cite a classical text by chapter, never through an edition.** Do not name a
20th-century commercial edition of one, a modern translator, or a page number
from either. Cite the chapter, or state the defining rule in the doc-comment so
the code needs no external reference at all. A translation is one scholar's
reading of the text; the chapter is what every reading has in common.

**No book's worked-value table is ever a conformance oracle.** This binds every
source, the acceptable ones included: cite Meeus eq. 15.1 for the method, never
his worked example for the answer. Tuning code until it reproduces a printed
value is reverse engineering whatever the book happens to be, and citing that
book afterwards does not change what was done.

## Prohibited References

**NEVER consult, read, copy from, or reference:**

- Source code from any AGPL or GPL-licensed ephemeris project (including all
  forks and ports in any language)
- Proprietary data files or compressed ephemeris formats not published by
  NASA/JPL directly
- Internal implementation documentation of any copyleft-licensed project

**If you encounter copyleft-licensed code during web search, IMMEDIATELY STOP,
discard the content, and re-derive the implementation from a permitted primary
source.**

## Copyright Header

Every source file must include:

```rust
// Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
// Vedaksha -- Vision from Vedas
// SPDX-License-Identifier: BUSL-1.1
// Contact: info@arthiq.net | https://vedaksha.net
```

## How to Contribute

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes following the requirements above
4. Run the full test suite: `cargo test --workspace`
5. Run clippy: `cargo clippy --workspace -- -D warnings`
6. Run format check: `cargo fmt --all -- --check`
7. Submit a pull request with a clear description of the change

## Issue Templates

### Bug Report
- Julian Day and coordinates used
- Expected output (from JPL Horizons or textbook)
- Actual output from Vedaksha
- Vedaksha version

### Feature Request
- Description of the feature
- Primary source citation (if astronomical/astrological)
- Use case

## Questions?

Contact: info@arthiq.net

---

*ArthIQ Labs LLC | https://vedaksha.net*
