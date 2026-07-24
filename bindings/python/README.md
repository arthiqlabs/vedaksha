# Vedākṣha for Python

The clean-room [Vedākṣha](https://vedaksha.net) Vedic-astronomy and Jyotish engine,
runnable from Python with **no Rust toolchain and no per-platform build**.

This package is not a reimplementation. It ships the real Rust engine compiled to
WebAssembly and runs it via [`wasmtime`](https://pypi.org/project/wasmtime/). Every value it
returns is the Rust engine's own — verified bit-for-bit against the native build, including
the sub-arcsecond JPL ephemeris tier. Because the engine is a single architecture-independent
`.wasm` blob, one `py3-none-any` wheel works on every OS and CPU `wasmtime` supports (Linux
glibc/musl, macOS, Windows, Android — x86-64 and arm64), on any Python ≥ 3.9.

## Install

```bash
pip install vedaksha
```

## Library

```python
from vedaksha import Vedaksha

vk = Vedaksha()

# All 15 engine tools are available.
for tool in vk.list_tools():
    print(tool["name"])

# Convenience wrappers for the common ones (analytical ephemeris tier):
chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6139, longitude=77.2090)
panch = vk.panchanga(julian_day=2451545.0, latitude=28.6139, longitude=77.2090)

# Or call any tool generically:
vargas = vk.call_tool("compute_vargas", julian_day=2451545.0,
                      latitude=28.6, longitude=77.2)
```

### Sub-arcsecond positions (SPK tier)

The tool surface uses the built-in analytical ephemeris (< 25″ planets, < 1″ Moon), which
needs no data files. For sub-arcsecond accuracy, supply a JPL DE440s kernel:

```python
with open("de440s.bsp", "rb") as f:
    vk.load_ephemeris(f.read())

vk.state_vector("moon", julian_day=2451545.0)
# {'x': ..., 'y': ..., 'z': ..., 'vx': ..., 'vy': ..., 'vz': ...}  # AU, ICRS
```

## Command line

```bash
vedaksha tools
vedaksha chart --julian-day 2451545.0 --latitude 28.6 --longitude 77.2
vedaksha call compute_vargas --args '{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2}'
```

## MCP server

Self-host the engine as a [Model Context Protocol](https://modelcontextprotocol.io) server —
the original motivation for this package, since it needs no Rust toolchain anywhere:

```bash
# stdio (for Claude Desktop, Cursor, etc.)
python -m vedaksha.mcp

# HTTP, bearer-token auth on by default, bound to localhost
VEDAKSHA_MCP_TOKEN=secret python -m vedaksha.mcp --http --port 3100
```

Unlike the Rust engine's HTTP server, this one **requires auth by default** and binds to
`127.0.0.1`; pass `--insecure-no-auth` and `--host 0.0.0.0` only behind a trusted boundary.

## REST (optional)

```bash
pip install "vedaksha[rest]"
python -m vedaksha.rest      # FastAPI app; each tool is POST /v1/<tool_name>
```

## Licensing

Business Source License 1.1, identical terms to the [Rust engine](https://github.com/arthiqlabs/vedaksha).
Production deployment is a commercial use — confirm your commercial licence before go-live.

## How it relates to the Rust engine

The Rust engine at [`arthiqlabs/vedaksha`](https://github.com/arthiqlabs/vedaksha) is
normative. This package pins a specific engine commit, compiles it to wasm, and ships the
blob. Version numbers track the engine. See `docs/` for the design and the
SPK-over-wasm validation.
