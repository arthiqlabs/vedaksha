# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Vedaksha, please report it
responsibly by emailing: **info@arthiq.net**

**Do NOT file a public GitHub issue for security vulnerabilities.**

## What Qualifies

- MCP server vulnerabilities (injection, authentication bypass, authorization flaws)
- Input validation bypasses that could cause crashes or undefined behavior
- Cryptographic issues in license key signing/verification
- Webhook security issues in the Stripe integration
- Dependencies with known CVEs

## What Does Not Qualify

- Inaccuracies in astronomical computation (file a regular bug report)
- Astrological interpretation disagreements
- Feature requests

## Response Timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 7 days
- **Fix timeline:** depends on severity, typically within 30 days for critical issues

## Disclosure Policy

We follow coordinated disclosure. We ask reporters to give us 90 days
to address the issue before public disclosure.

---

## Repository Hygiene

The Vedaksha public repository intentionally excludes internal development
artifacts — design specs, plan files, scratch notes, and strategy documents.
These live outside the git tree or under gitignored paths. The published
source code is the authoritative public surface.

Internal implementation strategy and comparative analysis of prior-art
implementations are not part of the project's public record.

## Source-Code Clean-Room Rule

Source files (`*.rs`) in `crates/` and `bindings/` MUST NOT reference external
ephemeris software (Swiss Ephemeris, Jagannatha Hora, pyswisseph, etc.) by name.

The only permitted occurrence of `swe_` in source is in
`#[serde(alias = "swe_…")]` attributes on test-oracle struct fields — these
match JSON field names in reference data files and do not constitute a
software reference.

Primary algorithmic sources cited in code comments:

- Meeus — *Astronomical Algorithms* (2nd ed.)
- Capitaine et al. — IAU 2006 precession/nutation
- Bretagnon & Francou — VSOP87 planetary theory
- Chapront — ELP/MPP02 lunar theory
- BPHS — *Brihat Parashara Hora Shastra* (R. Santhanam translation)
- Raman — *Hindu Predictive Astrology*, *Three Hundred Important Combinations*

Clean-room attribution: Vedaksha's ephemeris and astrological algorithms are
derived exclusively from NASA JPL ephemeris data (DE440s), the published
academic sources above, and classical Sanskrit texts. No source code was
inspected, translated, or adapted from AGPL/GPL-licensed ephemeris
implementations.

## History Rewrites

ArthIQ Labs may force-push to `main` and rewrite published tags in response to:

- Accidental commits of internal artifacts (design docs, strategy files, scratch notes)
- Clean-room rule violations
- Credential or secret leakage

Force-pushes are rare. When they occur, downstream consumers who cloned
before the rewrite date may need to re-clone. Published artifacts on
crates.io, npm, and PyPI are unaffected — those contain only crate source,
not repository metadata.

**Recent history rewrites:**

| Date | Scope | Reason |
|------|-------|--------|
| 2026-04-11 | Source comments | Remove accidental external-software references from `crates/*` |
| 2026-04-24 | `docs/superpowers/`, `site/` paths; all tags v1.0.0 – v1.7.0 | Remove internal planning/spec files and legacy website blob from public history |

## Supply Chain

- **Release artifacts** are published by `.github/workflows/release.yml` on
  `v*` tag push. Release tokens (`CARGO_REGISTRY_TOKEN`, `NPM_TOKEN`,
  `PYPI_API_TOKEN`) are stored as GitHub Actions repository secrets.
- **No third-party build-time code execution.** VSOP87A and ELP/MPP02
  coefficient tables are committed as pre-generated Rust source under
  `crates/vedaksha-ephem-core/src/analytical/coefficients/`. The Python
  generators in `scripts/` are not invoked by `cargo build`.
- **MSRV (minimum supported Rust version):** 1.85. Rust edition 2024.
- **Automated scanning.** Dependabot alerts and GitHub secret scanning are
  enabled on this repository. Push-protection is enabled to block secret
  commits at `git push` time.

## Cryptographic Integrity

- Release tags are Git-signed by the ArthIQ Labs maintainer key.
- Commercial license keys are Ed25519-signed; the public key is shipped in
  the `vedaksha` crate and the MCP server.

---

ArthIQ Labs LLC
info@arthiq.net | https://vedaksha.net
