# Vedākṣha v4.0.0 — Secure MCP, multi-arch, and a Python package

**Release date:** 2026-07-23

A major release. The one **breaking** change is HTTP MCP authentication; the rest
is additive — a new Python binding, a multi-arch Docker image, and a
no-filesystem entry point on the SPK reader. No numerical results change.

## Breaking — MCP HTTP server now requires authentication

The HTTP transport (`vedaksha-mcp --http`) previously bound `0.0.0.0` with
`Access-Control-Allow-Origin: *` and **no authentication**. It now:

- **requires** `Authorization: Bearer <token>` on every `POST`, where the token
  comes from `VEDAKSHA_MCP_TOKEN` (constant-time compared; missing/wrong → 401);
- **refuses to start** in HTTP mode without a token, unless `--insecure-no-auth`
  is passed (for trusted-network deployments);
- makes the bind host configurable (`--host` / `VEDAKSHA_MCP_HOST`, default
  `0.0.0.0`) and the CORS origin configurable (`VEDAKSHA_MCP_CORS_ORIGIN`,
  default `*`), and answers `OPTIONS` preflight so browser clients can send the
  Authorization header;
- keeps `/health` and the informational `GET` open (no computation).

**Migration:** set `VEDAKSHA_MCP_TOKEN` and send it as a bearer token, or pass
`--insecure-no-auth` to restore the previous open behaviour behind a network
boundary. The stdio transport is unchanged.

## New — Python package (`pip install vedaksha`)

A first-class Python binding under `bindings/python/`, distributed on PyPI. It
runs the real engine compiled to WebAssembly via `wasmtime` — not a
reimplementation — so results are bit-identical to the native build, and the
wheel is `py3-none-any`: no Rust toolchain, no per-platform build, any OS/CPU
`wasmtime` supports, any Python ≥ 3.9.

Four surfaces: a typed library, a `vedaksha` CLI, a self-hostable MCP server
(stdio + HTTP, auth on by default), and an optional FastAPI REST projection
(`vedaksha[rest]`). Both analytical and sub-arcsecond (DE440s) ephemeris tiers
are available.

## New — multi-arch Docker image

The published image is now built for **linux/amd64 and linux/arm64**. The
`Dockerfile` selects codegen per architecture (amd64 → AVX2 `x86-64-v3`,
arm64 → NEON baseline), so it no longer SIGILLs on Graviton / Apple Silicon.

## Added — `SpkReader::from_bytes`

The JPL SPK reader can now be constructed from an in-memory image, not only a
file path — the entry point the wasm/Python host uses to supply a DE440s kernel
without a filesystem. `SpkReader::open` is now a thin wrapper over it. No
behaviour change for existing callers.

## Release-safety

The tag `version-check` gate now also asserts the Python package version
(`pyproject.toml` and `__version__`) and the MCP tool snapshot's
`engineVersion` match the tag — the drift classes that let a stale wheel and a
stale lockfile ship in earlier releases.
