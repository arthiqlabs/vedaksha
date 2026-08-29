# Data Provenance

This file lists every external data source the Vedaksha repository ingests, with primary URL, license/copyright status, fetch date, and content hash. Every dev shortcut, mock, or sample subset must be logged here per project convention.

## External scientific data

| Module | Primary source | License / status | Fetch date | Files | Hash reference |
|---|---|---|---|---|---|
| ELP/MPP02 lunar series (Chapront & Francou 2003, A&A 404, 735; DOI 10.1051/0004-6361:20030529) | IMCCE / SYRTE — `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/` | Public scientific data; IMCCE distribution README terms (`README.TXT`) | 2026-05-09 | `scripts/data/elpmpp02/{README.TXT, elpmpp02.pdf, ELPMPP02.for, ELP_MAIN.S{1,2,3}, ELP_PERT.S{1,2,3}}` (gitignored — fetched by `scripts/generate_elpmpp02.py`) | [`docs/audit/2026-05-09-elp-mpp02-cleanroom/imcce-fetch-manifest.txt`](docs/audit/2026-05-09-elp-mpp02-cleanroom/imcce-fetch-manifest.txt) |
| VSOP87A planetary series (Bretagnon & Francou 1988, A&A 202, 309) | IMCCE — `ftp://ftp.imcce.fr/pub/ephem/planets/vsop87/` | Public scientific data; IMCCE distribution README terms | 2026-04 (regenerable on demand via `scripts/generate_vsop87a.py`) | `scripts/data/vsop87a/*.A` (gitignored); generated `crates/vedaksha-ephem-core/src/analytical/coefficients/{mercury,venus,earth,mars,jupiter,saturn,uranus,neptune}.rs` (committed) | Re-fetch via `scripts/generate_vsop87a.py`; the generator script encodes the primary URL. SHA256 ledger TBD as a follow-up; sources are byte-stable IMCCE primary. |
| JPL DE440s SPK kernel (numerical ephemeris read by `SpkReader`) | NAIF — `https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp` | Public domain (US Government work) | 2026-07-16 | `data/de440s.bsp` (gitignored, 32,726,016 bytes; fetched by `scripts/download_de440s.sh`) | `c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2` — pinned and verified by the download script, which refuses a mismatch. |
| JPL Horizons oracle (apparent geocentric ecliptic positions, DE441) | NASA/JPL Horizons System — `https://ssd.jpl.nasa.gov/api/horizons.api` | Public domain (US Government work) | 2026-07-16 | `tests/oracle_jpl/reference_positions.json` (committed, 24,350 rows; regenerable via `scripts/generate_horizons_oracle.py`) | `tests/oracle_jpl/reference_positions.json.sha256`; `--verify` re-fetches and compares. See [`tests/oracle_jpl/README.md`](tests/oracle_jpl/README.md). |

## Test fixtures with non-trivial provenance

| File | Provenance | Notes |
|---|---|---|
| `tests/fixtures/lunar_legacy_oracle.bin` | Numerical outputs of the pre-rederivation contaminated lunar implementation, captured 2026-05-09 before quarantine | Tier-3 regression oracle. Numerical-only firewall crossing — the *values* cross (uncopyrightable facts), no source/structural information does. See [`tests/fixtures/lunar_legacy_oracle.README.md`](tests/fixtures/lunar_legacy_oracle.README.md) and [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](docs/audit/2026-05-09-elp-mpp02-cleanroom/). |
| `tests/oracle_jpl/reference_positions.json` | Generated 2026-07-16 from NASA/JPL Horizons (DE441) by `scripts/generate_horizons_oracle.py` | Independent accuracy reference for `oracle_comparison.rs` (SpkReader) and `analytical_oracle.rs` (AnalyticalProvider). Horizons serves DE441 while `SpkReader` reads DE440s, so the comparison is against a separate kernel rather than our own input. Public-domain source, chosen over any third-party ephemeris library so no copyleft question touches the BUSL-1.1 clean-room position. See [`tests/oracle_jpl/README.md`](tests/oracle_jpl/README.md). |

## Accuracy fixes — 2026-06-15

### Fix 1 — Moon Nutation (JPL Horizons test vector)
Source: NASA/JPL Horizons System (https://ssd.jpl.nasa.gov/horizons/), DE441 ephemeris.
Oracle: Moon apparent ecliptic longitude at JD 2451545.0 TT (J2000.0) = 223.3238°, tolerance 0.006°.
License: U.S. Government work, public domain.
Note: The test vector (numeric fact) crosses the cleanroom boundary; no JPL source code was consulted.

> **Superseded 2026-08-29 — `Body::TrueNode` no longer uses the tolerance this entry
> describes.** As of v7.5.0, `Body::TrueNode` (and `Body::TrueSouthNode`) return the
> osculating node computation (`true_node_osculating` — the underlying osculating-node
> method, validated via its J2000-frame variant, `true_node_osculating_j2000`, to 0.6″
> against JPL Horizons — see the `osculating_node_vs_jpl_horizons` test in
> `crates/vedaksha-ephem-core/src/nodes.rs`) rather than the 5-term Meeus
> series this entry tightened a tolerance for. The Meeus formula is retained privately in
> `crates/vedaksha-ephem-core/src/nodes.rs` as a cross-check for the osculating computation's
> own regression tests; it is no longer reachable through any `Body` variant. This entry is
> left in place because a provenance record is not improved by being tidied after the fact —
> it documents what was true when it was written. See `CHANGELOG-v7.5.0.md`.

### Fix 2 — True Lunar Node Tolerance
Source: Meeus, J. (1998). "Astronomical Algorithms", 2nd ed., Willmann-Bell, Ch. 47.
Note: Tightened tolerance from 3° to 1.7° based on the amplitude of the dominant perturbation
term (1.4979° sin(2(D-F))). No new numerical constants were introduced. Ketu=Rahu+180° test
added per standard Vedic definition (BPHS).

> **Superseded 2026-08-18 — Fixes 3, 4 and 5 below describe constants that no longer ship.**
> Every ayanamsha constant they cover was removed in `4033c6a` and re-derived from primary
> definitions behind a two-agent firewall; see
> [`docs/audit/2026-08-17-ayanamsha-cleanroom/`](docs/audit/2026-08-17-ayanamsha-cleanroom/) and
> the "Ayanamsha — primary-source derivation" section below for what ships now. The three entries
> are left in place because a provenance record is not improved by being tidied after the fact;
> they document what was believed when they were written.
>
> **Licence correction, 2026-08-18.** Those three entries described the upstream project as
> LGPL. That was wrong. `aloistr/swisseph` is dual-licensed: the GNU **Affero** General Public
> License, or a separate Swiss Ephemeris Professional licence — verified against the upstream
> `LICENSE` file, which states the developer must choose one or the other. AGPL is the stricter
> of the two readings, so the earlier description understated the obligation rather than
> overstating it. The licence name is corrected here.
>
> **Citation sweep, 2026-08-18.** These entries also carried book page numbers and a named
> modern translator with a link. Both are forbidden by this project's citation rule and by
> `CONTRIBUTING.md`, and that rule binds the record as much as it binds new text: leaving them
> in place keeps pointing a reader at exactly what the rule exists to stop. They were removed.
> The sources themselves, and every claim these entries make, are unchanged.
>
> The claim this project makes going forward is procedural, not historical: as of v6.0.0 every
> ayanamsha is generated from a cited primary, a second implementation re-derives it, and CI
> checks that continuously. See the section below.

### Fix 3 — Lahiri Ayanamsha (epoch-anchored IAU 1976)
Source 1: Lieske, J.H. et al. (1977). "Expressions for the Precession Quantities Based upon the
IAU (1976) System of Astronomical Constants." A&A 58, 1–16 (eq. A2). Academic publication.
Source 2: Indian Astronomical Ephemeris 1989, the ICRC 1955 epoch definition.
Source 3: Swiss Ephemeris sweph.h, ayanamsa[1] (aloistr/swisseph — see the licence correction above) — epoch JD 2435553.5
and value 23.245524743° identified from this file. No source code copied; only the numeric
constants (uncopyrightable facts) were transcribed and the precession formula was independently
re-derived from Lieske et al.
License: Academic formulas are not copyrightable; constants are uncopyrightable numerical facts.

### Fix 4 — KP/Krishnamurti Ayanamsha (epoch-anchored Newcomb)
Source 1: Newcomb, S. (1898). "A Compendium of Spherical Astronomy". Public domain
(U.S. Naval Observatory publication, pre-1928).
Source 2: Swiss Ephemeris sweph.h, ayanamsa[5] (aloistr/swisseph — see the licence correction above) — epoch JD 2415020.31352
and value 22.363889°. No source code copied; only numeric constants transcribed.
License: Public domain (Newcomb); numeric constants are uncopyrightable facts.

### Fix 5 — Fagan-Bradley Ayanamsha (epoch-anchored Newcomb)
Source 1: Newcomb, S. (1898). "A Compendium of Spherical Astronomy". Public domain.
Source 2: Fagan, C. & Bradley, D. "The Synetic Vernal Point." American Astrology, 1967.
Source 3: American Sidereal Ephemeris (1976), Astro Computing Services — defines SVP at B1950.0.
Source 4: Swiss Ephemeris sweph.h, ayanamsa[0] (see the licence correction above) — epoch JD 2433282.42346 and value
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
License: Public domain (ancient Vedic literature; no modern copyright applies to the primary text).
Note: The 27-element mapping (ASHTOTTARI_LORDS_BY_NAK) was independently derived from the
BPHS Sanskrit source, not copied from any software implementation.

### Fix 8 — Bhakoot (Ashtakoota) Compatibility Score

> **Superseded 2026-07-23 — this code no longer ships.** `bhakoot_score()` and the whole
> Ashtakoota surface were removed in `87290fd`, released in v3.3.0. The entry is left in place
> for the same reason as Fixes 3–5: it records what was done when it was done.
Source: BPHS, Stree Jataka Adhyaya — on compatibility (public domain).
License: Public domain. The rules implemented (dosha conditions for specific sign separations)
are traditional Vedic computational rules, not copyrightable expression.
Note: The bhakoot_score() function implements the logical rule independently; no numerical tables
or code were copied from any published implementation.

## Ayanamsha — primary-source derivation, 2026-08-18

Eleven sidereal systems, each derived forward from a primary and none checked against, or tuned
toward, the output of any other implementation. The direction of derivation is what makes this
clean-room: computing forward from a documented primary and accepting the result is primary
research; tuning toward a known output is reverse engineering however it is cited afterwards.

Full spec, declared assumptions, search records for every system that was dropped, and the
machine-readable input manifest are in
[`docs/audit/2026-08-17-ayanamsha-cleanroom/`](docs/audit/2026-08-17-ayanamsha-cleanroom/).

| System | Primary | Status |
|---|---|---|
| Indian official (Lahiri / Chitra-paksha) | *The Indian Astronomical Ephemeris 2022*, Positional Astronomy Centre, Government of India — "AYANAMSA" section | Free, unrestricted; archive.org |
| Fagan-Bradley | Fagan & Firebrace, *Primer of Sidereal Astrology* — the synetic vernal point at B1950.0, and ayanamsha = 360° − SVP | Published definition; not copyrightable as a fact |
| Krishnamurti (KP) | Krishnamurti, *Krishnamurti Padhdhati Vol-I* — his anchor at the 1st of Chitra 1900 and his stated rate | archive.org |
| Raman | B. V. Raman, *A Manual of Hindu Astrology* (1935), Ch. III Art. 49 | Free and unrestricted; archive.org |
| Surya Siddhanta | Surya Siddhanta Ch. 3 vv. 9–12 | Public domain, cited by chapter |
| Yukteshwar | Yukteswar, *The Holy Science* (1894) | Public domain; archive.org |
| Revati-paksha | Revati at the sidereal initial point — the majority reading, against Surya Siddhanta Ch. 8's own 359°50′; ζ Piscium from Hipparcos | Public domain + catalogue |
| Pushya-paksha | P.V.R. Narasimha Rao, *Introducing Pushya-paksha Ayanamsa* | Freely published by the proposer |
| True Chitra | Self-describing condition; Spica from Hipparcos | Catalogue |
| True Mula (Chandra Hari) | K. Chandra Hari, *Indian Journal of History of Science* 33(4), 1998 | Peer-reviewed, free from INSA |
| Galactic Centre at 0° Sagittarius | Gordon, de Witt & Jacobs (2023), *AJ* 165, 49, doi:10.3847/1538-3881/aca65b | Peer-reviewed; arXiv:2212.00632 |

**Astrometric data.** Star positions and proper motions are from van Leeuwen, F. (2007), *A&A*
474, 653 — the re-reduction of the Hipparcos raw data — via VizieR table `I/311/hip2`, rows
HIP 5737, 42911, 65474 and 85927. Cross-checked against the original ESA 1997 catalogue
(`I/239/hip_main`); the positions agree to within 5.2 mas and the proper motions differ by up to
3.3 mas/yr, which is the honest uncertainty on a star-anchored system far from the catalogue
epoch. Redistributable with attribution (CDS/VizieR terms).

**Verification tooling.** Precession, obliquity and the ICRS→ecliptic transform are checked
against ERFA (BSD-3, the IAU SOFA board's approved relicensing) to better than 0.01″ across a
six-millennium span. ERFA is used for astronomy only and never for an ayanamsha value; verifying
our own computation against a permissive implementation taints nothing.

**No implementation was consulted.** No Swiss Ephemeris source, header, documentation or output —
including `swetest` — and no wrapper or service backed by it, at any point, for any purpose. No
worked-value table from any commercial edition was used as a conformance oracle. The values
Vedaksha 5 shipped were removed from the working tree *before* the derivation branch was cut, so
the derivation could not see its own target.

**How the derivation stays executable.** `scripts/generate_ayanamsha.py` re-derives every system
in Python from `derivation-inputs.json` and emits
`crates/vedaksha-astro/tests/fixtures/ayanamsha.json`; the Rust engine is asserted against that
fixture by `crates/vedaksha-astro/tests/ayanamsha_fixture.rs`. The two implementations share no
code, and agree to 5.7e-14° over 192 comparisons spanning 6000 years. `--verify` regenerates and
compares sha256, and needs no network, so unlike the coefficient-blob jobs it cannot pass
vacuously.

## Standing rules

- Every PR that adds a new external data source must add a row here.
- Every sample / subset / mock used during development must be either deleted before merge or logged here as a dev shortcut.
- The hash column must reference a file that exists in the repo or in a public audit dir.
- Audit dirs under `docs/audit/<date>-<topic>/` are the canonical home for SHA256 manifests when a re-derivation or migration is performed.

## Forbidden upstream

Per [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](docs/audit/2026-05-09-elp-mpp02-cleanroom/), the lunar implementation must NEVER again derive structurally from `github.com/ytliu0/ElpMpp02` (GPL-3.0). Source code, structural conventions, and constant-table transliterations from that upstream are out of bounds. Numerical comparisons against its outputs (legacy oracle pattern) are permissible only as facts.

Per [`docs/audit/2026-08-17-ayanamsha-cleanroom/`](docs/audit/2026-08-17-ayanamsha-cleanroom/),
the sidereal surface must NEVER again derive from Swiss Ephemeris in any form — source, headers,
documentation, `swetest` output, or any wrapper or service backed by it. Note the difference from
the lunar case: there, the coefficient values existed independently in the IMCCE primary, so the
old outputs were a legitimate fact-check. **Here the values themselves are the thing with no
derivation in the record**, so the legacy-oracle pattern is specifically forbidden —
regression-testing against the old ayanamsha numbers would defeat the purpose of re-deriving
them.
