<!-- mcp-name: io.github.arthiqlabs/vedaksha -->

# Vedaksha for Python — Vedic astrology & ephemeris

The clean-room [Vedaksha](https://vedaksha.net) Vedic-astronomy and Jyotish engine,
runnable from Python with **no Rust toolchain and no per-platform build**. Compute
**kundali / janam kundali** (natal charts), **panchanga**, **vimshottari and other
dashas**, **nakshatras**, **vargas** (divisional charts), **shadbala**, **ashtakavarga**,
**muhurta**, and **transits / gochara** — from a sub-arcsecond ephemeris (JPL
DE440s/DE441, worst case 0.272″ against Horizons) or from the data-file-free VSOP87A and
ELP/MPP02 series (mean 0.160″, worst case 1.473″ over 1900–2025).

This package is not a reimplementation. It ships the real Rust engine compiled to
WebAssembly and runs it via [`wasmtime`](https://pypi.org/project/wasmtime/). Every value it
returns is the Rust engine's own, checked bit-for-bit against the native build on a committed
conformance fixture that CI regenerates on every push, including the JPL ephemeris tier. Because the engine is a single architecture-independent
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

# Every engine tool is available; the catalog is the engine's own.
for tool in vk.list_tools():
    print(tool["name"])

# Typed wrapper for a natal chart (analytical ephemeris tier). Pass a sidereal
# ayanamsha for a kundali; without one the chart is tropical.
# Every Julian Day on the tool surface is UT1 (Universal Time) — not TT, not
# TDB. The engine converts to TT internally for the dynamical terms and uses
# UT1 directly for the Earth's rotation, which sets the ascendant and all
# twelve house cusps; passing TDB rotates every cusp by ~0.29 degrees.
chart = vk.natal_chart(julian_day=2451545.0, latitude=28.6139, longitude=77.2090,
                       ayanamsha="IndianOfficial", house_system="WholeSign")

# Every tool is callable by name — inspect its inputs with list_tools():
vargas = vk.call_tool("compute_vargas", julian_day=2451545.0,
                      latitude=28.6, longitude=77.2, divisions=["D9"])
# compute_panchanga needs an observer: the vara (weekday) is reckoned from
# LOCAL SUNRISE, so latitude and longitude are required, and elevation_m /
# tz_offset_minutes refine which sunrise and what it is called.
panchanga = vk.call_tool("compute_panchanga", jd=2451545.0, sun=280.0, moon=120.4,
                         latitude=28.6139, longitude=77.2090,
                         elevation_m=216.0, tz_offset_minutes=330)
dasha = vk.call_tool("compute_dasha", birth_jd=2451545.0, moon_longitude=120.4)
# compute_shadbala takes a moment and a place and returns every part of each
# graha's strength; the ayanamsha is required, and the Sun must rise and set.
shadbala = vk.call_tool("compute_shadbala", julian_day=2461338.0,
                        latitude=13.08, longitude=80.27,
                        ayanamsha="IndianOfficial", tz_offset_minutes=330)
```

### Sub-arcsecond positions (SPK tier)

The tool surface uses the built-in analytical ephemeris, which needs no data files. Against
JPL Horizons over 1900 to 2025 it measures 0.160″ mean and 1.473″ worst case across nine bodies
(the Moon 0.094″ mean, 0.276″ worst case). For the 0.096″ mean and 0.272″ worst case the JPL
DE440s kernel measures against the same reference, supply that kernel:

```python
with open("de440s.bsp", "rb") as f:
    vk.load_ephemeris(f.read())

# NOTE: this one Julian Day is TDB, not UT1. It is a raw SPK segment query —
# the kernel's coefficients are tabulated against TDB and no Delta T
# conversion is applied on this path.
vk.state_vector("moon", julian_day=2451545.0)
# {'x': ..., 'y': ..., 'z': ..., 'vx': ..., 'vy': ..., 'vz': ...}  # AU, ICRS
```

## Command line

```bash
vedaksha tools
vedaksha chart --julian-day 2451545.0 --latitude 28.6 --longitude 77.2 --ayanamsha IndianOfficial
vedaksha call compute_vargas --args '{"julian_day":2451545.0,"latitude":28.6,"longitude":77.2,"ayanamsha":"IndianOfficial","divisions":["D9"]}'
```

## MCP server

Self-host the engine as a [Model Context Protocol](https://modelcontextprotocol.io) server —
the original motivation for this package, since it needs no Rust toolchain anywhere. Three
equivalent ways to start it over stdio:

```bash
uvx vedaksha mcp      # no install
vedaksha-mcp          # after `pip install vedaksha`
python -m vedaksha.mcp
```

HTTP is available from the `python -m` form only, with bearer-token auth on by default:

```bash
VEDAKSHA_MCP_TOKEN=secret python -m vedaksha.mcp --http --port 3100
```

As with the Rust engine's HTTP server, auth is on by default; this one also binds to
`127.0.0.1`. Pass `--insecure-no-auth` and `--host 0.0.0.0` only behind a trusted boundary.

## REST (optional)

```bash
pip install "vedaksha[rest]"
python -m vedaksha.rest      # FastAPI app; each tool is POST /v1/<tool_name>
```

## Licensing

Business Source License 1.1, identical terms to the [Rust engine](https://github.com/arthiqlabs/vedaksha).
Non-commercial use is free. Commercial use (any product or service generating revenue from
external customers) needs a one-time commercial licence, USD $500 per organisation; each
release converts to Apache-2.0 four years after it ships. See [LICENSE][license].

## How it relates to the Rust engine

The Rust engine at [`arthiqlabs/vedaksha`](https://github.com/arthiqlabs/vedaksha) is
normative. This package pins a specific engine commit, compiles it to wasm, and ships the
blob. Version numbers track the engine. See [ARCHITECTURE.md](https://github.com/arthiqlabs/vedaksha/blob/main/bindings/python/ARCHITECTURE.md)
for the design and the SPK-over-wasm validation.

[license]: https://github.com/arthiqlabs/vedaksha/blob/main/LICENSE
