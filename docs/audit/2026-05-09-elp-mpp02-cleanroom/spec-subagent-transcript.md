# Spec Subagent — Final Report (audit artifact)

**Project:** Vedākṣha
**Date:** 2026-05-09
**Phase:** Spec — clean-room re-derivation of ELP/MPP02 lunar theory
**Model:** claude-opus-4-7 (1M context)
**Dispatch:** general-purpose subagent (fresh context, no parent-session memory)

---

## Final report verbatim

**Spec path:** `/workspace/vedaksha/docs/superpowers/specs/2026-05-09-elp-mpp02-rederivation-spec.md`

**Path taken:** **Path A.** The IMCCE primary distribution at `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/` contains both the explanatory note `elpmpp02.pdf` (Chapront & Francou, October 2002) and the reference Fortran source `ELPMPP02.for`. Together these provide the LLR-fit and DE405-fit constant-correction tables (Table 3), the auxiliary derivative table (Table 5), the long-range secular additive corrections (Table 6), the closed-form correction formulas, the Delaunay/planetary argument polynomials (Tables 1, 2), the P/Q precession rotation to J2000, and printed acceptance vectors (Tables 8.a, both fits, 10 rows). No fallback to deriving constants from the A&A paper alone was needed. The DOI `10.1051/0004-6361:20030529` was independently confirmed as the canonical reference paper.

**Note on task brief:** the suggested path `ftp://ftp.imcce.fr/pub/ephem/moon/elpmpp02/` does not exist on `ftp.imcce.fr` (that host serves only ELP82B and a handful of other lunar products under `/pub/ephem/moon/`). The actual canonical IMCCE distribution lives on `cyrano-se.obspm.fr`. Documented in the spec's fetch manifest section.

**IMCCE primary files fetched** (saved to `/workspace/vedaksha/scripts/data/elpmpp02/`, all SHA256 in spec §6):
- `README.TXT` (4 445 B)
- `elpmpp02.pdf` (215 008 B)
- `ELPMPP02.for` (28 112 B)
- `ELP_MAIN.S1` (103 360 B)
- `ELP_MAIN.S2` (92 755 B)
- `ELP_MAIN.S3` (71 141 B)
- `ELP_PERT.S1` (1 209 918 B)
- `ELP_PERT.S2` (668 038 B)
- `ELP_PERT.S3` (1 281 928 B)

**Forbidden-source hygiene:** A web search surfaced two third-party implementations (`github.com/ytliu0/...`, a SourceForge wiki). Neither was opened; only the canonical IMCCE FTP path was extracted from the search snippet and independently verified by direct FTP listing. Spec §9 records this incident.

**Key ambiguities flagged for the implementer (spec §8):** `icor` enum parity between prose and reference Fortran (off-by-one), Table 1 t³/t⁴ typographical ambiguity for ϖ′, time-scale choice (TDB vs TT), and the analytic derivation required for velocities (the note prints velocity check values but no closed-form series).

---

## Audit notes

- The Agent tool returns only the subagent's final response, not its full intermediate tool-call transcript. The above is the complete final report.
- Subagent confirmed Path A — LLR-fit and DE405-fit constants both available from the IMCCE primary distribution. Implementation can choose either.
- Forbidden-source hygiene incident logged: search results surfaced `github.com/ytliu0/...`; subagent did not open. Recorded in spec §9.
- Discrepancy from task brief: actual canonical IMCCE host is `cyrano-se.obspm.fr`, not `ftp.imcce.fr`. The brief was an external-knowledge guess; the subagent's primary-source verification superseded it. This is the correct clean-room behavior.
