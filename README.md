# Vedākṣha — Vision from Vedas

Astronomical ephemeris and Vedic astrology platform. Clean-room Rust implementation with sub-arcsecond planetary precision.

**Celestial computation. Agentic precision.**

[Website](https://vedaksha.net) · [Docs](https://vedaksha.net/docs) · [API Reference](https://vedaksha.net/api-ref) · [Blog](https://vedaksha.net/blog)

---

## Quick Start

```rust
use vedaksha::prelude::*;

let jd = calendar_to_jd(2024, 3, 20, 12.0);
let chart = compute_chart(
    jd, 28.6139, 77.2090,
    &ChartConfig::vedic()
);
```

```bash
cargo add vedaksha-astro vedaksha-ephem-core
```

```bash
pip install vedaksha
```

## Workspace

| Crate | Description |
|-------|-------------|
| [vedaksha-math](crates/vedaksha-math) | Chebyshev polynomials, angle arithmetic, interpolation, rotation matrices |
| [vedaksha-ephem-core](crates/vedaksha-ephem-core) | JPL DE440 SPK reader, **AnalyticalProvider** (VSOP87A + ELP/MPP02), coordinate pipeline, precession, nutation, Delta T |
| [vedaksha-astro](crates/vedaksha-astro) | 10 house systems, 44 ayanamsha systems (IAU 2006 P03 5th-order precession), aspects, dignities, transits |
| [vedaksha-vedic](crates/vedaksha-vedic) | 27 nakshatras, 3 dasha systems, 16 vargas, 50 yogas, Shadbala |
| [vedaksha-graph](crates/vedaksha-graph) | Property graph ontology — 10 node types, 13 edge types |
| [vedaksha-emit](crates/vedaksha-emit) | Cypher, SurrealQL, JSON-LD, JSON, embedding text emitters |
| [vedaksha-mcp](crates/vedaksha-mcp) | Model Context Protocol server — 7 fully functional JSON-RPC tools for AI agents |
| [vedaksha-locale](crates/vedaksha-locale) | 7-language localization (English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Bengali) |
| [vedaksha-wasm](crates/vedaksha-wasm) | WebAssembly bindings — 972 KB binary, full chart computation in browser |

Plus [Python bindings](bindings/python) via PyO3 (`pip install vedaksha`).

## Two Ephemeris Providers

| Provider | Accuracy | Data | Use Case |
|----------|----------|------|----------|
| **SpkReader** | Sub-arcsecond | DE440s (31 MB on disk) | Servers, containers |
| **AnalyticalProvider** | <15" planets, <1" Moon | Zero files (compiled constants) | WASM, Cloudflare Workers, edge, `no_std` |

The AnalyticalProvider uses VSOP87A (Bretagnon & Francou 1988) for planets and ELP/MPP02 (Chapront 2002) for the Moon. All coefficients are compile-time constants — no runtime data files needed.

## Computation Pipeline

```
JPL DE440 SPK → Chebyshev evaluation → ICRS barycentric
  → light-time correction → precession (IAU 2006 P03, 5th-order)
  → nutation (IAU 2000B) → frame bias (ICRS→J2000)
  → aberration → ecliptic coordinates
```

Or for zero-data environments:

```
VSOP87A/ELP coefficients (compiled) → Poisson series evaluation
  → heliocentric ecliptic → equatorial rotation → barycentric ICRS
  → same downstream pipeline as above
```

**Delta T:** IERS measured table (1620-2025) + Espenak-Meeus predictions to 2050.

## Vedic Astrology

First-class Jyotish support — not a Western afterthought.

- **Nakshatras:** 27 lunar mansions with padas, lords, symbols, deities
- **Dashas:** Vimshottari (120-year), Yogini (36-year), Chara (sign-based), Ashtottari (108-year), Narayana (Jaimini)
- **Vargas:** All 16 divisional charts (Rashi through Shashtiamsha)
- **Yogas:** 50 classical combinations (Pancha Mahapurusha, Dhana, Raja, Daridra, etc.)
- **Shadbala:** Complete 6-component planetary strength
- **Ayanamsha:** 44 sidereal systems (Lahiri, Raman, KP, Fagan-Bradley, and 40 more)
- **Lunar nodes:** Mean, True (Meeus 5-term, ~0.09°), and Osculating (<0.03° vs JPL DE441) — KP sub-lord ready
- **Panchanga:** Complete 5-limb day — Tithi (with paksha, lord), Vara (with Rahu/Gulika Kalam), Nakshatra (with deity, yoni, nadi), Yoga (27 astronomical), Karana (60 half-tithis)
- **Drishti:** Graded aspect strengths — Full, ThreeQuarter (75%), Half (50%), Quarter (25%) per BPHS Ch. 26

## AI-First Architecture

Every chart computation produces a **property graph** — not flat structs. AI agents query chart data with Cypher, SurrealQL, or JSON-LD. The MCP server exposes 7 fully functional tools:

- `compute_natal_chart` — Full natal chart with houses, planets, aspects, dignities
- `compute_dasha` — Vimshottari dasha periods to any depth
- `compute_vargas` — Divisional chart positions
- `compute_transit` — Transit positions against natal with aspects
- `search_transits` — Find exact transit events in a date range
- `search_muhurta` — Find auspicious times with quality scoring
- `emit_graph` — Emit chart as Cypher, SurrealQL, JSON-LD, or embedding text

**Run the MCP server:**

```bash
cargo install vedaksha-mcp          # install
vedaksha-mcp                        # stdio (Claude Desktop, VS Code, Cursor)
vedaksha-mcp --http                 # HTTP on port 3100 (Smithery, remote)
vedaksha-mcp --http --port 8080     # custom port
docker run -p 3100:3100 ghcr.io/arthiqlabs/vedaksha-mcp  # Docker
```

## Accuracy

Validated against independent reference ephemerides across 24,000+ oracle data points:

| Metric | SpkReader (DE440s) | AnalyticalProvider |
|--------|-------------------|-------------------|
| Planetary longitude | Sub-arcsecond (avg 1.7") | <15" (avg 3.8") |
| Moon longitude | Sub-arcsecond | <1" (0.36") |
| House cusps (10 systems) | Sub-0.001° | Sub-0.01° |
| Ayanamsha (44 systems) | avg 0.005° | Same (pure math) |
| Dasha periods | Sum to 120 years ± 0.01 days | Same |
| Nakshatra boundaries | Reference-accurate | Matches SpkReader at all tested boundaries |

## Bindings

| Platform | Install | Chart Computation |
|----------|---------|-------------------|
| Rust | `cargo add vedaksha-astro vedaksha-ephem-core` | Full pipeline |
| Python | `pip install vedaksha` | `vedaksha.compute_natal_chart(...)` |
| WASM | `wasm-pack build crates/vedaksha-wasm` | 972 KB, zero data files |
| MCP | stdio + HTTP transport | 7 tools, JSON-RPC 2.0 |
| Docker | `docker run -p 3100:3100 ghcr.io/arthiqlabs/vedaksha-mcp` | HTTP on port 3100 |

## Published Packages

- **crates.io:** 9 crates at v1.5.1
- **PyPI:** `vedaksha` v1.5.1 (source + macOS arm64 wheel)

## License

Business Source License 1.1 (BSL).

- **Non-commercial use:** Free. Personal projects, research, education, internal tools.
- **Commercial use:** $500 one-time per organization. [Purchase license](https://vedaksha.net/pricing).
- **Converts to Apache 2.0** five years after each version's release date.

See [LICENSE](LICENSE) for full terms.

## Links

- Website: [vedaksha.net](https://vedaksha.net)
- Documentation: [vedaksha.net/docs](https://vedaksha.net/docs)
- AI Integration: [vedaksha.net/ai](https://vedaksha.net/ai)
- crates.io: [crates.io/crates/vedaksha-ephem-core](https://crates.io/crates/vedaksha-ephem-core)
- PyPI: [pypi.org/project/vedaksha](https://pypi.org/project/vedaksha)

---

Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
