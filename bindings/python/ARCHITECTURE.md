# Architecture — the Python binding

This package runs the real Vedākṣha engine from Python. It reimplements none of
the engine's logic. The Rust workspace in this repository is compiled to a
WebAssembly module, and the Python package loads that module with
[`wasmtime`](https://pypi.org/project/wasmtime/) and drives it over a small C ABI.

## Why wasm, not a rewrite or a native extension

- **One artifact, every platform.** The engine becomes a single
  architecture-independent `.wasm` file. The Python package needs no per-platform
  wheels and no Rust toolchain on the user's machine — it publishes as
  `py3-none-any` and depends only on `wasmtime`, whose own wheels cover Linux
  (glibc + musl), macOS, Windows and Android, on x86-64 and arm64.
- **No logic written twice.** A pure-Python reimplementation would duplicate
  ~29,000 lines of astronomy and Jyotish code and then have to prove the two
  agree forever. Hosting the compiled engine means there is exactly one
  implementation; the binding is a transport.
- **Bit-identical results.** Because it *is* the engine, the wasm output matches
  the native Rust build to the bit — verified by `tests/conformance/`, which
  asserts exact equality against a fixture generated from the native binary, not
  a tolerance.

## Components

```
bindings/python/
  engine/                    # Rust cdylib shim → wasm32 (workspace member, publish = false)
    src/lib.rs               # the C ABI: vk_alloc/vk_free, vk_mcp_request/_take, vk_spk_*
  scripts/build-wasm.sh      # builds the blob into src/vedaksha/_wasm/ (artifact, not committed)
  src/vedaksha/
    _engine.py               # the only module that touches wasmtime
    client.py                # Vedaksha: list_tools / call_tool / natal_chart + SPK tier
    mcp/                     # stdio + HTTP transports (bearer auth on by default)
    rest.py                  # FastAPI projection of the tool list ([rest] extra)
    cli.py                   # `vedaksha` command
```

The four surfaces (library, CLI, MCP, REST) are all thin projections of one
JSON-RPC entry point — `McpServer::handle_request` inside the wasm module. None
of them contains computation.

## The C ABI

The wasm module imports nothing (no WASI). The host manages buffers in the
module's linear memory: `vk_alloc(n)` reserves bytes, the host writes into them,
and passes the pointer to a function. Two surfaces:

- **MCP** — `vk_mcp_request`/`vk_mcp_take` wrap the engine's JSON-RPC server, so
  all 15 tools are reachable with no extra glue.
- **SPK** — `vk_spk_load`/`vk_spk_state` expose the sub-arcsecond `SpkReader`
  for callers that supply a JPL DE440s kernel. `SpkReader::from_bytes` (added for
  this binding) lets the reader work without a filesystem.

## The blob is a build artifact

`src/vedaksha/_wasm/vedaksha.wasm` is **not** committed. The engine source is in
this repository, so the blob is built from it — locally with
`scripts/build-wasm.sh`, or in CI immediately before the wheel is built. Both the
wheel and the sdist carry the blob, so an end user installing from either never
needs a Rust toolchain. The blob's version therefore always matches the engine
commit it was built from; there is no separate pin to drift.

## Ephemeris tiers

- **Analytical** (default) — VSOP87A + ELP/MPP02, embedded in the wasm module,
  no external data. Every tool call works on this tier (< 25″ planets, < 1″ Moon).
- **SPK** — sub-arcsecond, requires a DE440s kernel (~32 MB) the caller feeds to
  `load_ephemeris`; the host copies it into wasm memory. Not bundled.

## Relationship to the other bindings

`crates/vedaksha-wasm` (npm) targets browsers and JS/edge runtimes via
wasm-bindgen; this package targets a `wasmtime` host from Python via a raw C ABI.
Same engine, different runtimes and calling conventions — siblings, not
alternatives.
