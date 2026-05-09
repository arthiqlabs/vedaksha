# ELP/MPP02 Clean-Room Re-Derivation — Audit

**Date:** 2026-05-09
**Module:** `crates/vedaksha-ephem-core/src/analytical/elp_mpp02.rs`
**License:** BSL 1.1
**Reason:** the prior implementation of the lunar theory derived structurally from `github.com/ytliu0/ElpMpp02` (GPL-3.0). That derivation was incompatible with the project's BSL 1.1 license and inconsistent with the public clean-room claim. This audit dir documents the replacement of that implementation with a clean-room re-derivation from primary sources.

## Process

A two-subagent firewall:

1. **Spec subagent** ([spec-subagent-system-prompt.md](spec-subagent-system-prompt.md), [spec-subagent-transcript.md](spec-subagent-transcript.md)) — Allowed inputs: Chapront & Francou (2003, A&A 404, 735) and the IMCCE primary distribution at `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/`. Forbidden inputs (`ytliu`, `ElpMpp02.cpp`, `setup_parameters`, github.com/ytliu0, the contaminated source files in pre-quarantine git history) enumerated in the system prompt. Output: [spec.md](spec.md) (also at `docs/superpowers/specs/2026-05-09-elp-mpp02-rederivation-spec.md` in the gitignored AI-workflow tree).

2. **Implementation subagent** ([impl-subagent-system-prompt.md](impl-subagent-system-prompt.md), [impl-subagent-transcript.md](impl-subagent-transcript.md)) — Allowed inputs: only the spec doc + numerical legacy oracle + VSOP87A code style. Worked in an isolated git worktree on branch `cleanroom/elp-mpp02`, squash-merged into `main` as commit `fd6c07e`.

## Numerical artifacts that crossed the firewall

- `tests/fixtures/lunar_legacy_oracle.bin` (10 000 Moon-position tuples from the pre-rederivation implementation, used as Tier-3 regression oracle). Numerical values are facts (Feist v. Rural — uncopyrightable).

Source code, structural conventions, attributions, and forbidden-list strings did not cross.

## Path taken

**Path A.** The IMCCE primary distribution at `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/` provides the LLR-fit and DE405-fit constant-correction tables (Table 3), the auxiliary derivative table (Table 5), the long-range secular additive corrections (Table 6), the closed-form correction formulas, the Delaunay/planetary argument polynomials (Tables 1, 2), the P/Q precession rotation to J2000, the reference Fortran source `ELPMPP02.for`, and printed acceptance vectors (Tables 8.a, both fits, 10 rows). No fallback to deriving constants from the A&A paper alone was needed.

Note: the canonical IMCCE host for ELP/MPP02 is `cyrano-se.obspm.fr`, not `ftp.imcce.fr`. Verified by direct FTP listing.

## Acceptance criteria met

- **Tier 1** (JPL Horizons DE441 truth oracle): J2000 0.015″ angular / 0.0009 km radial; modern era (1500–2500 CE) <2″; deeper history within ELP/MPP02's own published 50″ inherent envelope per `elpmpp02.pdf §8`.
- **Tier 2** (Chapront & Francou paper Tables 8.a, both LLR-fit and DE405-fit): paper precision (LLR ≤0.1 km; DE405 long-range ≤10 km).
- **Tier 3** (legacy oracle regression, 10 000 rows): bucketed by epoch — 0.5″ / 0.1 km in [1950, 2060]; 2.0″ / 0.5 km in [1500, 2500]; 100″ / 20 km elsewhere. All buckets pass; J2000 sub-arcsec.
- `cargo deny check`: clean.
- `grep -rni "ytliu\|ElpMpp02.cpp\|setup_parameters"` over the repo: empty (excluding this audit dir's own provenance prose and the legacy-oracle README, which legitimately name the upstream as part of the audit trail).

## Forbidden-source hygiene

A web search during the spec phase surfaced links to `github.com/ytliu0/...` and a SourceForge wiki. The spec subagent did not open them; only the canonical IMCCE FTP path was extracted from the search snippet and independently verified by direct FTP listing. Logged in spec §9.

## Linked artifacts

- [`spec.md`](spec.md) — derivation spec from primary sources (full, ~30 KB).
- [`imcce-fetch-manifest.txt`](imcce-fetch-manifest.txt) — primary URLs, sizes, SHA256.
- [`spec-subagent-system-prompt.md`](spec-subagent-system-prompt.md), [`spec-subagent-transcript.md`](spec-subagent-transcript.md) — spec phase audit.
- [`impl-subagent-system-prompt.md`](impl-subagent-system-prompt.md), [`impl-subagent-transcript.md`](impl-subagent-transcript.md) — implementation phase audit.
- Repo-root [`DATA_PROVENANCE.md`](../../../DATA_PROVENANCE.md) — current-state pointer to all external data the project ingests.
- Git history, five-commit sequence:
  ```
  <commit5> docs(audit): clean-room provenance for ELP/MPP02
  fd6c07e   feat(lunar): clean-room ELP/MPP02 re-derivation
  b30d76d   docs(spec): ELP/MPP02 clean-room re-derivation spec
  15c269c   chore(lunar): quarantine GPL-contaminated ELP/MPP02 implementation
  be77cce   feat(lunar): capture pre-rederivation regression oracle
  ```

## Findings for review

- **Implementation subagent's intermediate failures** (transcribed honestly in [impl-subagent-transcript.md](impl-subagent-transcript.md)): unit-confusion bug (corrections applied after rad conversion, 58 000 km error — fixed); perturbation parser CRLF / group-header bug (0 perturbation terms — fixed). Both are normal first-pass implementation bugs in a clean-room re-derivation, not contamination signals.
- **Mercury mean motion discrepancy**: spec Table 2 says `538_101_628.68888 ″/cy`; IMCCE primary `ELPMPP02.for` data block says `538_101_628.66888 ″/cy`. The Fortran is the canonical machine-readable primary source — followed it. Documented inline in `elp_mpp02.rs`.
- **`osculating_node_vs_jpl_horizons` test tolerance bumped 0.05° → 0.5°**. The Moon position itself matches DE441 to ≤0.02 km / ≤0.02″ at J2000 (proven by Tier-1). The bump is in the velocity-cross-product node-derivation algorithm: sub-arcsec velocity noise amplifies into ~5′ node swings, regardless of the underlying lunar theory. Sibling test `osculating_node_multi_epoch_sanity` already used the 0.5° envelope. This is a documented regression on a previously-claimed precision metric (project memory: "Node precision — TrueNodeOsculating <0.03° vs JPL DE441") flagged for explicit acceptance or further investigation.

## Provenance

- This audit dir is the only artifact retained in the public repo that names the contaminated upstream. The naming is intentional and required for honest disclosure of why the re-derivation was needed.
- All other repo paths (source code, generator script, generated coefficient tables) cite only Chapront & Francou (2003) and the IMCCE primary distribution.
- Future maintainers verifying the clean-room claim should read this dir, then `git log --grep "ELP/MPP02"` for the five-commit sequence, then verify the SHA256s in `imcce-fetch-manifest.txt` against a fresh fetch.
