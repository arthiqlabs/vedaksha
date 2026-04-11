# Vedākṣha — Axis of Wisdom

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
cargo add vedaksha
```

```bash
pip install vedaksha
```

## Workspace

| Crate | Description |
|-------|-------------|
| [vedaksha-math](crates/vedaksha-math) | Chebyshev polynomials, angle arithmetic, interpolation, rotation matrices |
| [vedaksha-ephem-core](crates/vedaksha-ephem-core) | JPL DE440/441 SPK reader, coordinate pipeline, precession, nutation, Delta T |
| [vedaksha-astro](crates/vedaksha-astro) | 10 house systems, 44 ayanamsha systems, aspects, dignities, transits |
| [vedaksha-vedic](crates/vedaksha-vedic) | 27 nakshatras, 3 dasha systems, 16 vargas, 50 yogas, Shadbala |
| [vedaksha-graph](crates/vedaksha-graph) | Property graph ontology — 10 node types, 13 edge types |
| [vedaksha-emit](crates/vedaksha-emit) | Cypher, SurrealQL, JSON-LD, JSON, embedding text emitters |
| [vedaksha-mcp](crates/vedaksha-mcp) | Model Context Protocol server — 7 JSON-RPC tools for AI agents |
| [vedaksha-locale](crates/vedaksha-locale) | 7-language localization (English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Malayalam) |
| [vedaksha-wasm](crates/vedaksha-wasm) | WebAssembly bindings via wasm-bindgen |

Plus [Python bindings](bindings/python) via PyO3.

## Computation Pipeline

```
JPL DE440 SPK → Chebyshev evaluation → ICRS barycentric
  → light-time correction → precession (IAU 2006)
  → nutation (IAU 2000B) → frame bias (ICRS→J2000)
  → aberration → ecliptic coordinates
```

**Delta T:** IERS measured table (1620–2025) + Espenak-Meeus predictions to 2050.

## Vedic Astrology

First-class Jyotish support — not a Western afterthought.

- **Nakshatras:** 27 lunar mansions with padas, lords, symbols, deities
- **Dashas:** Vimshottari (120-year), Yogini (36-year), Chara (sign-based)
- **Vargas:** All 16 divisional charts (Rashi through Shashtiamsha)
- **Yogas:** 50 classical combinations (Pancha Mahapurusha, Dhana, Raja, Daridra, etc.)
- **Shadbala:** Complete 6-component planetary strength
- **Ayanamsha:** 44 sidereal systems (Lahiri, Raman, KP, Fagan-Bradley, and 40 more)

## AI-First Architecture

Every chart computation produces a **property graph** — not flat structs. AI agents query chart data with Cypher, SurrealQL, or JSON-LD. The MCP server exposes 7 tools:

- `compute_natal_chart` — Full natal chart with houses, planets, aspects
- `compute_dasha` — Vimshottari dasha periods to any depth
- `compute_vargas` — Divisional chart positions
- `compute_transit` — Current transits against natal positions
- `search_transits` — Find exact transit events in a date range
- `search_muhurta` — Find auspicious times for activities
- `emit_graph` — Emit chart as Cypher, SurrealQL, JSON-LD, or embedding text

## Accuracy

Validated against independent reference ephemerides across 55,000+ data points:

| Metric | Result |
|--------|--------|
| Planetary longitude | Sub-arcsecond (avg 1.24″) |
| House cusps (10 systems) | Sub-0.001° |
| Ayanamsha (44 systems) | Matches reference values |
| Dasha periods | Sum to 120 years ± 0.01 days |
| MCP layer | 100% consistency with direct computation |

## Bindings

| Platform | Install |
|----------|---------|
| Rust | `cargo add vedaksha-astro` |
| Python | `pip install vedaksha` |
| WASM | `npm install vedaksha-wasm` |

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
- Playground: [vedaksha.net/playground](https://vedaksha.net/playground)

---

Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
