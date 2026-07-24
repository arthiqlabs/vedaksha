# Vedākṣha — Vision from Vedas

**Clean-room Rust ephemeris and Vedic astrology engine, built for the agentic-AI era.** Sub-arcsecond planetary precision, every algorithm traced to a primary source, every chart emitted as a queryable property graph.

*Celestial computation. Agentic precision.*

[Website](https://vedaksha.net) · [Docs](https://vedaksha.net/docs) · [Playground](https://vedaksha.net/playground) · [API reference](https://docs.rs/vedaksha) · [Blog](https://vedaksha.net/blog)

`clean-room` · `sub-arcsecond vs JPL Horizons` · `820 tests + 24,350 oracle rows` · `MCP-native` · `BSL 1.1 → Apache 2.0`

---

## Quick start

```rust
use vedaksha::prelude::*;

let jd = calendar_to_jd(2024, 3, 20, 12.0);
let chart = compute_chart(jd, 28.6139, 77.2090, &ChartConfig::vedic());
```

```bash
cargo add vedaksha          # Rust
npm install vedaksha-wasm   # WebAssembly
```

## Why Vedākṣha

- **Clean-room, cited.** Every module that implements a cited algorithm carries a `// Source:` doc-comment pointing at the primary paper or treatise (VSOP87A, ELP/MPP02, IAU standards, BPHS, Jaimini) — never derived from other software, no GPL contamination. See [`DATA_PROVENANCE.md`](DATA_PROVENANCE.md) and [`docs/audit/`](docs/audit/).
- **Sub-arcsecond, measured.** 820 tests on every push (Ubuntu and macOS); a scheduled full run adds 24,350 oracle comparisons against JPL Horizons / DE441 — mean residual **0.106″** over 1900–2025. Every number in [Accuracy](#accuracy) is printed by a test you can run.
- **Agentic-AI-native.** A 15-tool Model Context Protocol server, and every chart is a property graph you can query in Cypher, SurrealQL, or JSON-LD.
- **Runs everywhere.** One Rust codebase → native, WebAssembly (no data files), and a containerized MCP server. No FFI to a C library, no platform-specific build.
- **Jyotish in the type system.** Nakshatras, dashas, vargas, shadbala, ayanamshas — first-class, not a Western afterthought.

## In production

Vedākṣha is the calculation engine under ArthIQ Labs' Jyotish properties:

| Product | What it is |
|---------|------------|
| [kundalimcp.com](https://kundalimcp.com) | The B2B/developer engine — an agentic-AI Jyotish MCP with the full computation suite (yogas, all five dasha systems, shadbala, school-specific interpretation). Builds directly on the `vedaksha-*` crates. |
| [janampatri.net](https://janampatri.net) | Vedic astrology marketplace — expert consultations, plus BPHS-grounded charts and life-trajectory analysis. |
| [kundali.live](https://kundali.live) | Consumer endpoint — chat-based readings and self-serve PDF reports. |

## Workspace

| Crate | Description |
|-------|-------------|
| [vedaksha](crates/vedaksha) | Umbrella crate — `prelude`, `compute_chart`, `ChartConfig`. Optional `locale` feature (7 languages: en · hi · sa · ta · te · kn · bn). |
| [vedaksha-math](crates/vedaksha-math) | Chebyshev polynomials, angle arithmetic, interpolation, rotation matrices |
| [vedaksha-ephem-core](crates/vedaksha-ephem-core) | JPL DE440 SPK reader, **AnalyticalProvider** (VSOP87A + ELP/MPP02), coordinate pipeline, precession, nutation, ΔT |
| [vedaksha-astro](crates/vedaksha-astro) | 10 house systems, 44 ayanamshas (IAU 2006 P03 5th-order), aspects, dignities, transits |
| [vedaksha-vedic](crates/vedaksha-vedic) | 27 nakshatras, 5 dasha systems, 16 vargas, Shadbala |
| [vedaksha-graph](crates/vedaksha-graph) | Property-graph ontology (9 node types, 12 edge types) + Cypher / SurrealQL / JSON-LD emitters |
| [vedaksha-mcp](crates/vedaksha-mcp) | Model Context Protocol server — 15 JSON-RPC tools for AI agents |
| [vedaksha-wasm](crates/vedaksha-wasm) | WebAssembly bindings — full chart computation in the browser, no data files |

## Two ephemeris providers

| Provider | Accuracy | Data | Use case |
|----------|----------|------|----------|
| **SpkReader** | Sub-arcsecond | DE440s (~31 MB on disk) | Servers, containers |
| **AnalyticalProvider** | <25″ planets, <1″ Moon | Zero files (compiled constants) | WASM, edge, Cloudflare Workers, `no_std` |

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
- **Shadbala** — complete six-component planetary strength, with Ishta / Kashta phala
- **Ayanamsha** — 44 sidereal systems (Lahiri, Raman, KP, Fagan-Bradley, +40)
- **Lunar nodes** — Mean, True (Meeus 5-term, ~0.09°), and Osculating (0.6″ max vs JPL DE441 over 1900–2100) — KP sub-lord ready
- **Panchanga** — full five limbs: Tithi (paksha, lord), Vara (Rahu / Gulika Kalam), Nakshatra (deity, yoni, nadi), Yoga (27), Karana (60)
- **Drishti** — graded aspects: Full, ¾ (75%), ½ (50%), ¼ (25%) per BPHS Ch. 26

## AI-native: MCP + property graph

Every computation produces a **property graph**, not flat structs — so an agent can ask "which planets aspect the 7th-house lord?" as a graph query instead of re-implementing chart logic. The MCP server exposes 15 tools, discoverable with a single `tools/list` call:

`compute_natal_chart` · `compute_dasha` · `compute_vargas` · `compute_karakas` · `compute_combustion` · `compute_shadbala` · `compute_ashtakavarga` · `compute_transit` · `compute_gochara` · `search_transits` · `search_muhurta` · `compute_panchanga` · `compute_drishti` · `compute_bhavas` · `emit_graph`

```bash
cargo install vedaksha-mcp
vedaksha-mcp                                          # stdio (Claude Desktop, Cursor, VS Code)
VEDAKSHA_MCP_TOKEN=… vedaksha-mcp --http --port 3100  # HTTP transport (auth required)
docker run -e VEDAKSHA_MCP_TOKEN=… -p 3100:3100 ghcr.io/arthiqlabs/vedaksha-mcp
```

HTTP mode requires a bearer token (`Authorization: Bearer <token>` on every POST); the server refuses to start without `VEDAKSHA_MCP_TOKEN` unless you pass `--insecure-no-auth` for a trusted-network deployment. `/health` and the informational `GET` stay open. The Docker image is multi-arch (amd64 + arm64).

The tool surface is generated from the Rust definitions and locked by a snapshot test, so the published catalog can't silently drift from the code.

## Accuracy

Every figure below is printed by a named test. Reproduce them with
`bash scripts/download_de440s.sh` then
`cargo test -p vedaksha-ephem-core --release -- --include-ignored --nocapture`.

**SpkReader vs JPL Horizons (DE441)** — `oracle_comparison.rs`, over the
24,350 rows in [`tests/oracle_jpl/`](tests/oracle_jpl) (10 bodies × 2,435 dates,
1900–2100). Horizons serves DE441, so this measures our DE440s pipeline against
an independent kernel:

| Era | Comparisons | Mean | Max |
|-----|-------------|------|-----|
| 1900–2025 (ΔT measured) | 15,350 | **0.106″** | 1.184″ (Uranus) |
| 1900–2100 (all) | 24,350 | 0.880″ | 44.914″ (Moon, 2099) |

15,349 of 15,350 comparisons before 2026 are sub-arcsecond. Past 2025 the
residual is dominated by **ΔT prediction**, not ephemeris error: our Espenak &
Meeus extrapolation and Horizons' own ΔT diverge by ~68 s at 2099, which shows
up in proportion to a body's angular rate (the Moon, at 0.64″/s, picks up ~45″;
Pluto, essentially none). At 2099-02-06 the Sun, Moon, Mercury, Venus and Mars —
rates spanning 0.03–0.64″/s — all imply the same 66–71 s offset. ΔT beyond the
IERS measured record is unpredictable in principle, not a defect we can fix.

**AnalyticalProvider vs JPL Horizons (DE441)** — `analytical_oracle.rs`, the
same fixture over 1900–2025 (13,815 comparisons). VSOP87A is a truncated
analytical theory, so it is necessarily looser than the numerical kernel:

| Body | Mean | Max |
|------|------|-----|
| Venus | 4.83″ | **24.22″** |
| Mercury | 4.27″ | 11.49″ |
| Sun | 4.09″ | 7.00″ |
| Mars | 3.06″ | 18.08″ |
| Jupiter | 0.81″ | 1.97″ |
| Neptune | 0.50″ | 1.70″ |
| Saturn | 0.46″ | 1.11″ |
| Uranus | 0.33″ | 1.44″ |
| Moon | 0.17″ | 0.61″ |

Overall mean 2.06″. `analytical_accuracy.rs` reports a friendlier 13.09″ max for
the same provider, but it samples 10 dates against this test's 2,435 per body —
the sparse grid never lands near Venus's worst case. The table above is the
better-sampled number and the one to trust.

**ELP/MPP02 Moon vs JPL Horizons (DE441)** — `lunar_horizons.rs`, live-fetched
over −3000…+3000 CE: **0.015″ at J2000** (tolerance 0.06″), 0.020–0.053″ across
1500–2500 CE, degrading to 85.5″ in deep antiquity where ELP/MPP02's own
published precision is the limit.

**Ayanamsha** — Lahiri, Fagan-Bradley and KP are checked at J2000 to
0.003–0.005° (`sidereal.rs`), propagating documented epoch anchors with IAU 1976
/ Newcomb precession. Every anchor's origin is listed in
[`DATA_PROVENANCE.md`](DATA_PROVENANCE.md). The other 41 systems are
range-checked only — no accuracy figure is claimed for them.

Dasha totals and nakshatra boundaries are covered by invariant tests
(`vimshottari.rs`, `nakshatra.rs`). Those verify internal consistency — that our
BPHS constants sum to 120 years, that boundaries tile the circle — and are not
comparisons against an external reference. **House-cusp accuracy is not measured
against any reference.**

## Install

| Platform | Install | Notes |
|----------|---------|-------|
| Rust | `cargo add vedaksha` | full pipeline |
| WASM | `npm install vedaksha-wasm` | browser & edge, no data files |
| MCP | `cargo install vedaksha-mcp` | 15 tools, stdio + HTTP |
| Docker | `docker run ghcr.io/arthiqlabs/vedaksha-mcp` | MCP server on port 3100 |

**Published:** crates.io — 7 crates (`vedaksha`, `vedaksha-math`, `vedaksha-ephem-core`, `vedaksha-astro`, `vedaksha-vedic`, `vedaksha-graph`, `vedaksha-mcp`) · npm `vedaksha-wasm` · Docker `ghcr.io/arthiqlabs/vedaksha-mcp`.

## License

**Business Source License 1.1.**

- **Non-commercial use** — free (personal projects, research, education, internal tools).
- **Commercial use** — $500 one-time per organization. [Purchase →](https://vedaksha.net/pricing)
- **Converts to Apache 2.0** five years after each version's release date.

See [LICENSE](LICENSE) for full terms.

---

Copyright © 2026 ArthIQ Labs LLC · Licensed under BSL 1.1.
