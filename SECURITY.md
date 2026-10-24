# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Vedaksha, please report it
responsibly by emailing: **info@arthiq.net**

**Do NOT file a public GitHub issue for security vulnerabilities.**

## What Qualifies

- MCP server vulnerabilities — injection, bearer-token bypass, authorization flaws
- Input validation bypasses that could cause a crash, a panic, or unbounded resource use
- Unsafe deserialization of ephemeris kernels or tool arguments
- Dependencies with known CVEs

This repository is a computation library and an MCP server. It handles no
payments, issues no license keys, and stores no user data, so there is no
billing or key-signing surface here to report against.

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

## Source-Code Clean-Room Rule

Vedaksha's ephemeris and astrological algorithms, as shipped, are derived
from NASA JPL ephemeris data and published primary sources. The shipped
implementation contains no source code translated or adapted from any GPL
or AGPL ephemeris implementation. Where earlier work did not meet that rule,
it was quarantined and re-derived from primary sources; the record of each
such case is in [DATA_PROVENANCE.md](DATA_PROVENANCE.md).

Algorithmic sources cited in code comments:

- Meeus — *Astronomical Algorithms* (2nd ed.)
- Capitaine et al. — IAU 2006 precession/nutation
- Bretagnon & Francou — VSOP87 planetary theory
- Chapront — ELP/MPP02 lunar theory
- BPHS — *Brihat Parashara Hora Shastra*, cited by chapter

## Supply Chain

- Release artifacts are published by `.github/workflows/release.yml` on `v*` tag push.
- No workspace crate has a build script, and no coefficient generator runs at build time. The VSOP87A and ELP/MPP02 coefficients are committed binary tables under `crates/vedaksha-ephem-core/src/analytical/coefficients/`; the Python generators in `scripts/` that produced them are not invoked by `cargo build` or by push CI. Third-party dependencies may carry their own build scripts and proc macros, as any Rust dependency can.
- MSRV (minimum supported Rust version): 1.89. Rust edition 2024.
- Dependabot alerts, secret scanning, and push protection are enabled on this repository.

## Cryptographic Integrity

- Release tags and commits are SSH-signed by an ArthIQ Labs maintainer key.

Maintainer signing key fingerprints (Ed25519, both registered as signing keys on the
`arthiqlabs` GitHub account):

```
SHA256:H2QUSGvE1R8Jq2NHihXoVpTsTRNt5zuqZrNPLt3vswU
SHA256:VO0RcM+XCrSmeZGk/BXmGr5IqUp7qUX6gFHAOVDiGTA
```

Verify the current release via:

```
git verify-tag <tag>
git verify-commit HEAD
```

---

ArthIQ Labs LLC
info@arthiq.net | https://vedaksha.net
