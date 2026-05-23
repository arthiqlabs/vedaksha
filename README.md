# Vedākṣha — Vision from Vedas

**Clean-room Rust ephemeris and Vedic astrology engine, built for the agentic-AI era.** Sub-arcsecond planetary precision, every algorithm traced to a primary source, every chart emitted as a queryable property graph.

*Celestial computation. Agentic precision.*

[Website](https://vedaksha.net) · [Docs](https://vedaksha.net/docs) · [Playground](https://vedaksha.net/playground) · [API reference](https://docs.rs/vedaksha) · [Blog](https://vedaksha.net/blog)

`clean-room` · `sub-arcsecond vs JPL Horizons` · `870 tests + 8,700 oracle rows` · `MCP-native` · `BSL 1.1 → Apache 2.0`

---

## Quick start

```rust
use vedaksha::prelude::*;

let jd = calendar_to_jd(2024, 3, 20, 12.0);
let chart = compute_chart(jd, 28.6139, 77.2090, &ChartConfig::vedic());
```

```bash
cargo add vedaksha          # Rust
pip install vedaksha        # Python (PyO3)
npm install vedaksha-wasm   # WebAssembly
```

## Why Vedākṣha

- **Clean-room, cited.** Every module that implements a cited algorithm carries a `// Source:` doc-comment pointing at the primary paper or treatise (VSOP87A, ELP/MPP02, IAU standards, BPHS, Jaimini) — never derived from other software, no GPL contamination. See [`DATA_PROVENANCE.md`](DATA_PROVENANCE.md) and [`docs/audit/`](docs/audit/).
- **Sub-arcsecond, proven.** Validated against JPL Horizons / DE441 — 870 tests plus 8,700 oracle reference rows on every CI run, on both Ubuntu and macOS.
- **Agentic-AI-native.** A 12-tool Model Context Protocol server, and every chart is a property graph you can query in Cypher, SurrealQL, or JSON-LD.
- **Runs everywhere.** One Rust codebase → native, Python, WebAssembly (no data files), and a containerized MCP server. No FFI to a C library, no platform-specific build.
- **Jyotish in the type system.** Nakshatras, dashas, vargas, yogas, shadbala, ayanamshas — first-class, not a Western afterthought.

## Workspace

| Crate | Description |
|-------|-------------|
| [vedaksha](crates/vedaksha) | Umbrella crate — `prelude`, `compute_chart`, `ChartConfig`. Optional `locale` feature (7 languages: en · hi · sa · ta · te · kn · bn). |
| [vedaksha-math](crates/vedaksha-math) | Chebyshev polynomials, angle arithmetic, interpolation, rotation matrices |
| [vedaksha-ephem-core](crates/vedaksha-ephem-core) | JPL DE440 SPK reader, **AnalyticalProvider** (VSOP87A + ELP/MPP02), coordinate pipeline, precession, nutation, ΔT |
| [vedaksha-astro](crates/vedaksha-astro) | 10 house systems, 44 ayanamshas (IAU 2006 P03 5th-order), aspects, dignities, transits |
| [vedaksha-vedic](crates/vedaksha-vedic) | 27 nakshatras, 5 dasha systems, 16 vargas, 50 yogas, Shadbala |
| [vedaksha-graph](crates/vedaksha-graph) | Property-graph ontology (10 node types, 13 edge types) + Cypher / SurrealQL / JSON-LD emitters |
| [vedaksha-mcp](crates/vedaksha-mcp) | Model Context Protocol server — 12 JSON-RPC tools for AI agents |
| [vedaksha-wasm](crates/vedaksha-wasm) | WebAssembly bindings — full chart computation in the browser, no data files |

Python bindings via PyO3 live in [bindings/python](bindings/python).

## Two ephemeris providers

| Provider | Accuracy | Data | Use case |
|----------|----------|------|----------|
| **SpkReader** | Sub-arcsecond | DE440s (~31 MB on disk) | Servers, containers |
| **AnalyticalProvider** | <15″ planets, <1″ Moon | Zero files (compiled constants) | WASM, edge, Cloudflare Workers, `no_std` |

The AnalyticalProvider evaluates VSOP87A (Bretagnon & Francou 1988) for planets and ELP/MPP02 (Chapront 2002) for the Moon — all coefficients are compile-time constants, so there are no runtime data files.

## Computation pipeline

```
JPL DE440 SPK → Chebyshev evaluation → ICRS barycentric
  → light-time correction → precession (IAU 2006 P03, 5th-order)
  → nutation (IAU 2000B) → frame bias (ICRS→J2000)
  → aberration → ecliptic coordinates
```

Zero-data path (WASM / edge):

```
VSOP87A / ELP coefficients (compiled) → Poisson series evaluation
  → heliocentric ecliptic → equatorial rotation → barycentric ICRS
  → same downstream pipeline
```

**Delta T:** IERS measured table (1620–2025) + Espenak–Meeus predictions to 2050.

## Vedic astrology

First-class Jyotish, drawn from primary classical sources.

- **Nakshatras** — 27 lunar mansions with padas, lords, symbols, deities
- **Dashas** — Vimshottari (120-yr), Yogini (36-yr), Ashtottari (108-yr), and Chara & Narayana (Jaimini, sign-based)
- **Vargas** — all 16 divisional charts (D-1 Rashi → D-60 Shashtiamsha)
- **Yogas** — 50 classical combinations (Pancha Mahapurusha, Dhana, Raja, Daridra, …)
- **Shadbala** — complete six-component planetary strength, with Ishta / Kashta phala
- **Ayanamsha** — 44 sidereal systems (Lahiri, Raman, KP, Fagan-Bradley, +40)
- **Lunar nodes** — Mean, True (Meeus 5-term, ~0.09°), and Osculating (<0.03° vs JPL DE441) — KP sub-lord ready
- **Panchanga** — full five limbs: Tithi (paksha, lord), Vara (Rahu / Gulika Kalam), Nakshatra (deity, yoni, nadi), Yoga (27), Karana (60)
- **Drishti** — graded aspects: Full, ¾ (75%), ½ (50%), ¼ (25%) per BPHS Ch. 26

## AI-native: MCP + property graph

Every computation produces a **property graph**, not flat structs — so an agent can ask "which planets aspect the 7th-house lord?" as a graph query instead of re-implementing chart logic. The MCP server exposes 12 tools, discoverable with a single `tools/list` call:

`compute_natal_chart` · `compute_dasha` · `compute_vargas` · `compute_karakas` · `compute_combustion` · `compute_shadbala` · `compute_ashtakavarga` · `compute_transit` · `compute_gochara` · `search_transits` · `search_muhurta` · `emit_graph`

```bash
cargo install vedaksha-mcp
vedaksha-mcp                      # stdio (Claude Desktop, Cursor, VS Code)
vedaksha-mcp --http --port 3100   # HTTP transport
docker run -p 3100:3100 ghcr.io/arthiqlabs/vedaksha-mcp
```

The tool surface is generated from the Rust definitions and locked by a snapshot test, so the published catalog can't silently drift from the code.

## Accuracy

Validated against two independent reference ephemerides across **8,700 oracle reference rows** in [`tests/oracle_jpl/`](tests/oracle_jpl):

| Metric | SpkReader (DE440s) | AnalyticalProvider |
|--------|--------------------|--------------------|
| Planetary longitude | Sub-arcsecond (avg 1.7″) | <15″ (avg 3.8″) |
| Moon longitude | Sub-arcsecond | <1″ (0.23″ avg, 0.60″ max, 1900–2100 vs JPL Horizons) |
| House cusps (10 systems) | <0.001° | <0.01° |
| Ayanamsha (44 systems) | avg 0.005° | same (pure math) |
| Dasha periods | Sum to 120 yr ± 0.01 days | same |
| Nakshatra boundaries | Reference-accurate | matches SpkReader |

## Install

| Platform | Install | Notes |
|----------|---------|-------|
| Rust | `cargo add vedaksha` | full pipeline |
| Python | `pip install vedaksha` | PyO3, type stubs |
| WASM | `npm install vedaksha-wasm` | browser & edge, no data files |
| MCP | `cargo install vedaksha-mcp` | 12 tools, stdio + HTTP |
| Docker | `docker run ghcr.io/arthiqlabs/vedaksha-mcp` | MCP server on port 3100 |

**Published:** crates.io — 7 crates (`vedaksha`, `vedaksha-math`, `vedaksha-ephem-core`, `vedaksha-astro`, `vedaksha-vedic`, `vedaksha-graph`, `vedaksha-mcp`) · PyPI `vedaksha` · npm `vedaksha-wasm` · Docker `ghcr.io/arthiqlabs/vedaksha-mcp`.

## License

**Business Source License 1.1.**

- **Non-commercial use** — free (personal projects, research, education, internal tools).
- **Commercial use** — $500 one-time per organization. [Purchase →](https://vedaksha.net/pricing)
- **Converts to Apache 2.0** five years after each version's release date.

See [LICENSE](LICENSE) for full terms.

---

Copyright © 2026 ArthIQ Labs LLC · Licensed under BSL 1.1.
