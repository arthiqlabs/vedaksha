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

## Standing rules

- Every PR that adds a new external data source must add a row here.
- Every sample / subset / mock used during development must be either deleted before merge or logged here as a dev shortcut.
- The hash column must reference a file that exists in the repo or in a public audit dir.
- Audit dirs under `docs/audit/<date>-<topic>/` are the canonical home for SHA256 manifests when a re-derivation or migration is performed.

## Forbidden upstream

Per [`docs/audit/2026-05-09-elp-mpp02-cleanroom/`](docs/audit/2026-05-09-elp-mpp02-cleanroom/), the lunar implementation must NEVER again derive structurally from `github.com/ytliu0/ElpMpp02` (GPL-3.0). Source code, structural conventions, and constant-table transliterations from that upstream are out of bounds. Numerical comparisons against its outputs (legacy oracle pattern) are permissible only as facts.
