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

Vedaksha's ephemeris and astrological algorithms are derived exclusively
from NASA JPL ephemeris data and published academic sources. No source
code was inspected, translated, or adapted from any GPL or AGPL
ephemeris implementation.

Algorithmic sources cited in code comments:

- Meeus — *Astronomical Algorithms* (2nd ed.)
- Capitaine et al. — IAU 2006 precession/nutation
- Bretagnon & Francou — VSOP87 planetary theory
- Chapront — ELP/MPP02 lunar theory
- BPHS — *Brihat Parashara Hora Shastra*, cited by chapter

## Supply Chain

- Release artifacts are published by `.github/workflows/release.yml` on `v*` tag push.
- No third-party build-time code execution. VSOP87A and ELP/MPP02 coefficient tables are committed as pre-generated Rust source. The Python generators in `scripts/` are not invoked by `cargo build`.
- MSRV (minimum supported Rust version): 1.89. Rust edition 2024.
- Dependabot alerts, secret scanning, and push protection are enabled on this repository.

## Cryptographic Integrity

- Release tags and commits are SSH-signed by the ArthIQ Labs maintainer key.
- Commercial license keys are Ed25519-signed; the public key is shipped in the `vedaksha` crate and the MCP server.

Maintainer signing key fingerprint:

```
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
