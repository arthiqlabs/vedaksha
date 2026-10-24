# Technical report metadata

- **Title:** Vedaksha: A Clean-Room, Sub-Arcsecond Ephemeris Engine for Computational Vedic Astrology
- **Report:** Technical report describing Vedaksha v10.0.0
- **Date:** 2026-10-24
- **Author:** Amit Gupta; ArthIQ Labs LLC; info@arthiq.net; ORCID https://orcid.org/0009-0008-9450-8134
- **Status:** Not peer reviewed. Not submitted anywhere.

This file records the current technical report, not an active preprint submission.
The author block in `main.tex` is unchanged. Earlier arXiv category and endorsement
research does not establish a current submission or a current policy check.
No account, endorsement, deposit, DOI, or publication action was taken for this revision.
If submission is resumed, categories, endorsement requirements, and the paper's deposit
license must be checked then. No paper-specific Creative Commons license is selected here.
The software remains source-available under BUSL-1.1, converting to Apache-2.0 four years
after each version's release; that does not select a separate license for the report text.

## Abstract

Synchronized with `main.tex`; LaTeX math and range notation are normalized below.

> Vedaksha is a clean-room ephemeris and Vedic astrology (jyotish) computation engine written
> in Rust. It computes planetary positions from the analytical VSOP87A and ELP/MPP02 series
> and from the JPL DE440s numerical kernel. Re-measured on 2026-10-24 against NASA/JPL
> Horizons DE441, the numerical path has a mean apparent geocentric ecliptic-longitude
> residual of 0.096 arcseconds and a maximum of 0.272 arcseconds over
> 15,350 comparisons spanning 1900-2025. The analytical path has a mean of 0.160 arcseconds
> and a maximum of 1.473 arcseconds over 13,815 comparisons in the same interval.
> The sub-arcsecond description refers to mean longitude agreement, not a bound on every
> analytical result or on other coordinates. From 1962-01-20, the oracle converts Horizons'
> UTC time tags to Terrestrial Time using the historical TAI minus UTC record, removing
> both Delta T and UT1 minus UTC from that part of the comparison.
> Version 10 corrects the nutation coefficients and the VSOP87A frame rotation, includes
> solar gravitational light deflection, and updates Delta T from measured and predicted
> records. The engine exposes derived jyotish computations through native Rust, Python,
> WebAssembly, and Model Context Protocol interfaces. Primary sources and derivation records
> are documented in a public provenance ledger. An external parity harness measures the
> analytical path against JPL DE440 and IMCCE INPOP21a and over six centuries either side
> of J2000.0.
> The software is source-available under BUSL-1.1, with conversion to Apache-2.0 four years
> after each version's release.

## Evidence and deliverables

See `README.md` for exact measurement commands, the source of the external parity
figures, and verification limits. Native aarch64 and Rosetta x86-64 analytical runs were
measured at this checkout. External parity and full-range figures come from the parity
harness run of 2026-10-24 against `vedaksha==10.0.0`.

The current package is `main.tex`, `references.bib`, `main.pdf`, and `main.tex.sha256`.
The PDF is built locally with Tectonic and checked with `scripts/check_paper_pdf_fresh.py`.
No submission archive was created or verified for v10.0.0.
