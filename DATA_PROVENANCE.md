# Data Provenance

This file lists every external data source the Vedākṣha repository ingests, with primary URL, license/copyright status, fetch date, and content hash. Every dev shortcut, mock, or sample subset must be logged here per project convention.

## External scientific data

| Module | Primary source | License / status | Fetch date | Files | Hash reference |
|---|---|---|---|---|---|
| ELP/MPP02 lunar series (Chapront & Francou 2003, A&A 404, 735; DOI 10.1051/0004-6361:20030529) | IMCCE / SYRTE — `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/` | Public scientific data; IMCCE distribution README terms (`README.TXT`) | 2026-05-09 | `scripts/data/elpmpp02/{README.TXT, elpmpp02.pdf, ELPMPP02.for, ELP_MAIN.S{1,2,3}, ELP_PERT.S{1,2,3}}` (gitignored — fetched by `scripts/generate_elpmpp02.py`) | [`docs/audit/2026-05-09-elp-mpp02-cleanroom/imcce-fetch-manifest.txt`](docs/audit/2026-05-09-elp-mpp02-cleanroom/imcce-fetch-manifest.txt) |
| VSOP87A planetary series (Bretagnon & Francou 1988, A&A 202, 309) | IMCCE — `ftp://ftp.imcce.fr/pub/ephem/planets/vsop87/` | Public scientific data; IMCCE distribution README terms | 2026-04 (regenerable on demand via `scripts/generate_vsop87a.py`) | `scripts/data/vsop87a/*.A` (gitignored); generated `crates/vedaksha-ephem-core/src/analytical/coefficients/{mercury,venus,earth,mars,jupiter,saturn,uranus,neptune}.rs` (committed) | Re-fetch via `scripts/generate_vsop87a.py`; the generator script encodes the primary URL. SHA256 ledger TBD as a follow-up; sources are byte-stable IMCCE primary. |

## Test fixtures with non-trivial provenance

| File | Provenance | Notes |
|---|---|---|
| `tests/fixtures/lunar_legacy_oracle.bin` | Numerical outputs of the pre-rederivation contaminated lunar implementation, captured 2026-05-09 before quarantine | Tier-3 regression oracle. Numerical-only firewall crossing — the *values* cross (uncopyrightable facts), no source/structural information does. See [`tests/fixtures/lunar_legacy_oracle.README.md`](tests/fixtures/lunar_legacy_oracle.README.md) and [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](docs/audit/2026-05-09-elp-mpp02-cleanroom/). |

## Accuracy fixes — 2026-06-15

### Fix 1 — Moon Nutation (JPL Horizons test vector)
Source: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/), DE441 ephemeris.
Oracle: Moon apparent ecliptic longitude at JD 2451545.0 TT (J2000.0) = 223.3238°, tolerance 0.006°.
License: U.S. Government work, public domain.
Note: The test vector (numeric fact) crosses the cleanroom boundary; no JPL source code was consulted.

### Fix 2 — True Lunar Node Tolerance
Source: Meeus, J. (1998). "Astronomical Algorithms", 2nd ed., Willmann-Bell, Ch. 47.
Note: Tightened tolerance from 3° to 1.7° based on the amplitude of the dominant perturbation
term (1.4979° sin(2(D-F))). No new numerical constants were introduced. Ketu=Rahu+180° test
added per standard Vedic definition (BPHS).

### Fix 3 — Lahiri Ayanamsha (epoch-anchored IAU 1976)
Source 1: Lieske, J.H. et al. (1977). "Expressions for the Precession Quantities Based upon the
IAU (1976) System of Astronomical Constants." A&A 58, 1–16 (eq. A2). Academic publication.
Source 2: Indian Astronomical Ephemeris 1989, p. 556 (ICRC 1955 epoch definition).
Source 3: Swiss Ephemeris sweph.h, ayanamsa[1] (LGPL, aloistr/swisseph) — epoch JD 2435553.5
and value 23.245524743° identified from this file. No source code copied; only the numeric
constants (uncopyrightable facts) were transcribed and the precession formula was independently
re-derived from Lieske et al.
License: Academic formulas are not copyrightable; constants are uncopyrightable numerical facts.

### Fix 4 — KP/Krishnamurti Ayanamsha (epoch-anchored Newcomb)
Source 1: Newcomb, S. (1898). "A Compendium of Spherical Astronomy", p. 226. Public domain
(U.S. Naval Observatory publication, pre-1928).
Source 2: Swiss Ephemeris sweph.h, ayanamsa[5] (LGPL, aloistr/swisseph) — epoch JD 2415020.31352
and value 22.363889°. No source code copied; only numeric constants transcribed.
License: Public domain (Newcomb); numeric constants are uncopyrightable facts.

### Fix 5 — Fagan-Bradley Ayanamsha (epoch-anchored Newcomb)
Source 1: Newcomb, S. (1898). "A Compendium of Spherical Astronomy", p. 226. Public domain.
Source 2: Fagan, C. & Bradley, D. "The Synetic Vernal Point." American Astrology, 1967.
Source 3: American Sidereal Ephemeris (1976), Astro Computing Services — defines SVP at B1950.0.
Source 4: Swiss Ephemeris sweph.h, ayanamsa[0] (LGPL) — epoch JD 2433282.42346 and value
24.042044444°. No source code copied; only numeric constants transcribed.
License: Published definitions are not copyrightable; Newcomb formula is public domain.

### Fix 6 — Precession Matrix Long-Range Sanity Test
Source: Capitaine, N., Wallace, P.T. & Chapront, J. (2003). "Expressions for IAU 2000 precession
quantities." A&A 412, 567–586, Table 1. Academic publication.
Note: No new constants introduced. Test verifies existing `general_precession_in_longitude` and
`precession_matrix` functions at JD 2299160.5 (1582-10-15, Gregorian reform date) produce
physically reasonable output (~−20986 arcsec accumulated from J2000, negative = past date).

### Fix 7 — Ashtottari Dasha Ardradi Lookup Table
Source: BPHS (Brihat Parashara Hora Shastra), Ch. 35, vv. 17–20 (Ardradi Ashtottari variant).
Translation reference: S.P. Tata, cited at astrojyoti.com/bphspage6.htm.
License: Public domain (ancient Vedic literature; no modern copyright applies to the primary text).
Note: The 27-element mapping (ASHTOTTARI_LORDS_BY_NAK) was independently derived from the
BPHS Sanskrit source, not copied from any software implementation.

### Fix 8 — Bhakoot (Ashtakoota) Compatibility Score
Source 1: B.V. Raman. "Muhurtha (Electional Astrology)", 10th ed., UBS Publishers, 1979.
Ch. on Ashtakoota — defines Shadashtak and Nava-Pancham dosha conditions.
Source 2: BPHS, Stree Jataka Adhyaya — on compatibility (public domain).
License: Raman's authored text is copyrighted; the underlying rules (dosha conditions for specific
sign separations) are traditional Vedic computational rules, not copyrightable expression.
Note: The bhakoot_score() function implements the logical rule independently; no numerical tables
or code were copied from any published implementation.

## Standing rules

- Every PR that adds a new external data source must add a row here.
- Every sample / subset / mock used during development must be either deleted before merge or logged here as a dev shortcut.
- The hash column must reference a file that exists in the repo or in a public audit dir.
- Audit dirs under `docs/audit/<date>-<topic>/` are the canonical home for SHA256 manifests when a re-derivation or migration is performed.

## Forbidden upstream

Per [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](docs/audit/2026-05-09-elp-mpp02-cleanroom/), the lunar implementation must NEVER again derive structurally from `github.com/ytliu0/ElpMpp02` (GPL-3.0). Source code, structural conventions, and constant-table transliterations from that upstream are out of bounds. Numerical comparisons against its outputs (legacy oracle pattern) are permissible only as facts.
