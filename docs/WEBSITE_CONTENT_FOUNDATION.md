# Vedaksha Website Content Foundation

> **Purpose:** Single source of truth for all website hero, landing page, and marketing content.
> Every claim is backed by actual implementation. Every number is from the codebase.
> Use this document to build impactful hero sections, feature pages, and comparison content.

---

## THE ELEVATOR PITCH

**Vedaksha is the first astronomical computation platform built from the ground up for AI agents.**

It is a clean-room Rust implementation of planetary ephemeris and astrological computation — derived exclusively from NASA JPL data and published academic sources — that outputs property graphs, speaks MCP natively, and treats Vedic astrology as a first-class citizen.

---

## TECH STACK — WHAT POWERS VEDAKSHA

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Core Language** | Rust (Edition 2024) | Zero-cost abstractions, memory safety, no GC pauses |
| **Ephemeris (SpkReader)** | NASA JPL DE440 (SPK/DAF format) | Sub-arcsecond planetary positions, public domain |
| **Ephemeris (Analytical)** | VSOP87A (Bretagnon 1988) + ELP/MPP02 (Chapront 2002) | Zero-data-file ephemeris, <15" planets, <1" Moon, WASM/edge compatible |
| **Math Foundation** | Clenshaw recurrence, Hermite/Lagrange interpolation, Poisson series | Chebyshev + trigonometric series evaluation |
| **Precession** | IAU 2006 P03 (Capitaine, Wallace & Chapront), 5th-order | Fukushima-Williams 4-angle parameterization, single source of truth |
| **Nutation** | IAU 2000B (McCarthy & Luzum, 77 terms) | Milliarcsecond-accurate nutation in longitude/obliquity |
| **Aberration** | Annual aberration (Meeus Ch. 23) | First-order correction from Earth's velocity |
| **Time Systems** | Delta T (Espenak & Meeus polynomials) | TT/UT1 conversion covering -500 to 2150+ CE |
| **Graph Model** | Custom property graph (serde + JSON) | 10 node types, 13 edge types, deterministic IDs |
| **Graph Emission** | Neo4j Cypher, SurrealDB SurrealQL, JSON-LD | Direct database ingestion, no ETL needed |
| **AI Protocol** | MCP (Model Context Protocol), JSON-RPC 2.0 | Native AI agent integration |
| **Browser Runtime** | WebAssembly (wasm-bindgen) | Full computation in the browser, zero server |
| **Python Bindings** | PyO3 + maturin | `pip install vedaksha` for data science |
| **Localization** | Static lookup tables, 7 languages | English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Bengali |
| **CI/CD** | GitHub Actions + cargo-deny | License compliance, vulnerability scanning |
| **License** | BSL 1.1 → Apache 2.0 (5-year conversion) | Open-source trajectory with commercial sustainability |

---

## BY THE NUMBERS

| Metric | Value |
|--------|-------|
| **Automated tests** | 528+ |
| **Oracle validation points** | 24,000+ (planetary positions, house cusps, ayanamsha, nutation, sidereal time) |
| **Workspace crates** | 9 (all published to crates.io v1.5.0) |
| **Binding targets** | 3 (native, WASM, Python) + MCP |
| **Ephemeris providers** | 2 (SpkReader: sub-arcsecond from DE440s; AnalyticalProvider: <15" planets, zero data files) |
| **WASM binary size** | 972 KB uncompressed (654 KB gzipped) — Cloudflare Workers compatible |
| **Ayanamsha systems** | 44 (IAU 2006 P03 5th-order precession) |
| **House systems** | 10 |
| **Vedic yogas** | 50 |
| **Divisional charts (vargas)** | 16 (D-1 through D-60) |
| **Dasha systems** | 5 (Vimshottari, Yogini, Chara, Ashtottari, Narayana) |
| **Nakshatras** | 27 (with 108 padas) |
| **Lunar node methods** | 3 (Mean, True/Meeus ~0.09°, Osculating <0.03° vs JPL DE441) |
| **Aspect types** | 11 (5 major + 6 minor) |
| **Aspect patterns** | 5 (Grand Trine, T-Square, Yod, Grand Cross, Stellium) |
| **Graph node types** | 10 |
| **Graph edge types** | 13 |
| **Graph emitter formats** | 5 (Cypher, SurrealQL, JSON-LD, JSON, RAG text) |
| **MCP tools** | 7 (all fully functional — natal, dasha, vargas, transit, search, muhurta, graph) |
| **MCP transports** | 2 (stdio for local, Streamable HTTP for remote) |
| **Docker image** | ghcr.io/arthiqlabs/vedaksha-mcp |
| **Localized languages** | 7 (en, hi, sa, ta, te, kn, bn) |
| **Primary sources cited** | 15+ (Meeus, BPHS, Holden, IAU, JPL, Bretagnon, Chapront, B.V. Raman, ...) |
| **External runtime dependencies** | 3 (serde, libm, thiserror) — no heavy frameworks |
| **Unsafe code blocks** | 0 (enforced by `#![deny(unsafe_code)]` in every crate) |
| **Published packages** | crates.io (9 crates v1.5.0) + PyPI (vedaksha v1.5.0) + npm (vedaksha-wasm v1.5.0) |

---

## KEY DIFFERENTIATORS — WHY VEDAKSHA EXISTS

### 1. AI-First Architecture (Not AI-Bolted-On)

Every other ephemeris library was built in the 1990s-2000s for desktop developers writing GUI apps in C. Vedaksha was designed in 2026 asking one question: **"How would an AI agent consume astronomical computation?"**

**What this means in practice:**

| Traditional Libraries | Vedaksha |
|----------------------|----------|
| Initialize global state, set file paths, configure flags | Pure functions. Pass inputs, get outputs. Zero state. |
| Return C structs with integer codes (planet=3, sign=7) | Return rich typed enums (`Body::Jupiter`, `Sign::Sagittarius`, `Nakshatra::Pushya`) |
| Flat arrays of numbers that need domain expertise to interpret | Property graph (`ChartGraph`) with typed nodes and edges — self-describing |
| No protocol for AI communication — custom REST wrappers needed | MCP native — 7 typed tools with JSON schemas, OAuth 2.1, streaming |
| No concept of "what matters" — dump all data equally | `highlights` array ranking most significant features by astrological importance |
| Raw numbers for transit events | `nl_description` — pre-formatted natural language suitable for AI output |
| No vector embedding support | `EmbeddingTextEmitter` — RAG-optimized text for semantic search |
| Require personal data (name, birthdate strings) | **PII-blind** — computation layer sees ONLY Julian Day + coordinates |

### 2. Clean-Room Legal Foundation

Vedaksha is the only modern ephemeris that can guarantee zero copyleft contamination:

- **Every algorithm** traces to a cited primary source (NASA JPL, IAU standards, Meeus textbook, BPHS)
- **Zero lines of code** derived from any AGPL/GPL-licensed project
- **cargo-deny** enforces license compliance — AGPL, GPL, LGPL, SSPL are explicitly blocked
- **Implementation Choice Audit** reviews every arbitrary constant to verify independent derivation
- **BSL 1.1 license** with Apache 2.0 conversion after 5 years — commercial clarity

### 3. Vedic Astrology as First-Class Citizen

Every other library treats Vedic (Jyotish) astrology as an afterthought plugin. In Vedaksha, it's in the core type system:

- **27 nakshatras** with padas, dasha lords, guna, gana — not a lookup table bolted on
- **3 dasha systems** (Vimshottari 120-year, Yogini 36-year, Chara sign-based) with recursive sub-periods up to 5 levels deep
- **16 Shodasha Varga** divisional charts (D-1 through D-60) — not "D-9 only"
- **50 Vedic yogas** from BPHS and Phaladipika — detected automatically
- **Complete Shadbala** (six-fold planetary strength) — all 6 components
- **Vedic aspects (drishti)** with Mars/Jupiter/Saturn special aspects
- **44 ayanamsha systems** — every major tradition represented
- **Muhurta search** with tithi, nakshatra, and weekday quality scoring
- **7-language localization** — every term available in Hindi, Sanskrit (IAST), Tamil, Telugu, Kannada, Bengali

### 4. Graph-Native Output

No other ephemeris library produces graph-structured output. This matters because astrological relationships ARE a graph:

```
Planet → PlacedIn → Sign
Planet → Occupies → House
Planet → Aspects → Planet
Planet → InNakshatra → Nakshatra
DashaPeriod → ContainsPeriod → DashaPeriod
Chart → HasYoga → Yoga
```

Vedaksha produces a `ChartGraph` with **10 node types and 13 edge types**, emittable directly to:
- **Neo4j** (Cypher MERGE/CREATE) — for knowledge graphs combining astrology with other domains
- **SurrealDB** (SurrealQL CREATE/RELATE) — for modern multi-model databases
- **JSON-LD** (with `@context` ontology) — for linked data and semantic web
- **RAG text** — natural language optimized for vector embedding in AI retrieval pipelines

**Deterministic IDs** — same computation inputs always produce the same graph node IDs, enabling caching, deduplication, and cross-chart comparison.

### 5. Runs Everywhere — One Codebase

| Target | How | Use Case |
|--------|-----|----------|
| **Native (x86/ARM)** | `cargo build` | Server-side computation, batch processing |
| **WebAssembly** | `wasm-pack build` | Browser-based interactive charts, zero backend |
| **Python** | `pip install vedaksha` | Data science, Jupyter notebooks, research |
| **MCP Server** | JSON-RPC over stdio/HTTP | AI agent integration (Claude, GPT, custom) |
| **Any language** | FFI via C ABI | Embed in Go, Swift, Java, etc. |

### 6. Extreme Minimal Dependencies

The core computation crates (`vedaksha-math`, `vedaksha-ephem-core`) have only **one external dependency: `libm`** (pure-Rust math functions for `no_std`). No OpenSSL, no tokio, no heavy runtimes. The entire computation stack compiles on bare metal.

Compare: typical ephemeris libraries pull in dozens of C dependencies, require specific compiler toolchains, and can't run in WASM or embedded contexts.

---

## DESIGN CHOICES THAT MAKE VEDAKSHA ROBUST

### Pure Functions, Zero State

Every computation function in Vedaksha is pure — given the same inputs, it always returns the same output. No global state, no initialization sequence, no cleanup. This makes Vedaksha:
- **Thread-safe by construction** — call from any number of threads freely
- **Trivially parallelizable** — batch 10,000 charts with `rayon::par_iter()`
- **Deterministic** — same chart computed on different machines = identical output
- **Testable** — every function is independently testable with known inputs

### `no_std` Compatible Core

The foundation crates (`vedaksha-math`, `vedaksha-ephem-core`) compile without the Rust standard library. This means:
- **WASM ready** — runs in browsers and edge runtimes (Cloudflare Workers, Deno)
- **Embedded ready** — can run on microcontrollers (future: astronomical instruments)
- **Zero heap allocation** in hot paths (Chebyshev evaluation, angle normalization)

### Zero `unsafe` Code

Every crate in the workspace enforces `#![deny(unsafe_code)]`. There are zero `unsafe` blocks anywhere in the codebase. Memory safety is guaranteed by the Rust compiler, not by developer discipline.

### Structured Errors, Never Panics

Every public function returns `Result<T, ComputeError>`. The library never panics on any external input:
- Out-of-range dates → `DateOutOfRange` with valid range in the error
- Unknown bodies → `BodyNotAvailable`
- Convergence failures → `ConvergenceFailure` with iterations attempted
- Each error includes a `suggested_action` for AI agents to self-correct

### PII-Blind by Design

The computation layer accepts ONLY:
- `f64` Julian Day (a number, not a date string)
- `f64` latitude, `f64` longitude (coordinates, not place names)

It never sees, logs, or processes names, birth dates as strings, place names, email addresses, or any personal identifier. The mapping from "John Smith, born March 15 1990 in Chicago" to "JD 2447967.5, lat 41.88, lon -87.63" happens in the CALLER's application, not in Vedaksha. This is privacy by architecture, not by policy.

### Deterministic Graph IDs

Every `ChartGraph` node ID is deterministic — computed via FNV-1a hash of (julian_day, latitude, longitude, config_hash). This means:
- Same person's chart computed twice = same graph IDs
- Charts can be cached, merged, and compared by ID
- Graph databases can use MERGE (upsert) safely
- Global nodes (signs, nakshatras) share IDs across ALL charts

### Source Citations as Code

Every public function includes a `///` doc comment citing its primary source:

```rust
/// Compute the precession matrix from J2000 to the mean equator of date.
///
/// Source: Capitaine, Wallace & Chapront (2003), A&A 412, pp. 567-586.
///         Fukushima-Williams 4-angle parameterization.
pub fn precession_matrix(jd: f64) -> Matrix3 { ... }
```

This isn't just documentation — it's a legal guarantee that every algorithm is independently derived from a public academic source. The Implementation Choice Audit verifies this for every function before release.

---

## THE COORDINATE TRANSFORMATION PIPELINE

This is the core scientific engine. Each step follows the cited IAU standard:

```
NASA JPL DE440/441 (Chebyshev coefficients)
    ↓ Clenshaw recurrence → Barycentric ICRS position (km)
    ↓ Light-time iteration (converges in 2-3 iterations)
    ↓ Geocentric ICRS (subtract Earth position from EMB + Moon)
    ↓ IAU 2006 Precession (Fukushima-Williams 4 angles)
    ↓ IAU 2000B Nutation (77-term series)
    ↓ Annual Aberration (Earth velocity / speed of light)
    ↓ Ecliptic rotation (true obliquity)
    ↓ Apparent ecliptic longitude, latitude, distance
    ↓ Daily speed via numerical differentiation
```

**Result:** Sub-arcsecond agreement with NASA JPL Horizons for all major solar system bodies.

---

## VEDIC COMPUTATION DEPTH

No other platform implements Vedic astrology with this completeness:

### Nakshatras (27 Lunar Mansions)
Each carries: Sanskrit name, transliterated name, Vimshottari dasha lord, deity, guna (Sattva/Rajas/Tamas), gana (Deva/Manushya/Rakshasa). 4 padas per nakshatra = 108 padas total.

### Dasha Systems (3)
- **Vimshottari** — 120-year cycle, 9 lords, 5 hierarchical levels (Maha → Antar → Pratyantar → Sookshma → Prana). That's 9 × 9 × 9 × 9 × 9 = 59,049 leaf periods per 120-year cycle.
- **Yogini** — 36-year cycle, 8 lords, recursive sub-periods
- **Chara (Jaimini)** — Sign-based, 12 sign periods from lagna

### Yogas (50 Planetary Combinations)
Automatically detected from chart positions:
- 5 Pancha Mahapurusha (Ruchaka, Bhadra, Hamsa, Malavya, Sasa)
- Moon-based (Gajakesari, Kemadruma, Sunapha, Anapha, Durudhara, Shakata, Vish)
- Wealth (Lakshmi, Saraswati, Dhana)
- Power (Chamara, Parvata, Raja, Neechabhanga)
- Spiritual (Sanyasa)
- Negative (Daridra, Graha Yuddha)
- Sankhya (Rajju, Musala, Nala — sign distribution patterns)

### Shadbala (Six-Fold Strength)
All 6 components computed per BPHS Ch. 27:
1. **Sthana Bala** — positional (exalted/own/friendly/neutral/debilitated)
2. **Dig Bala** — directional (angular strength by house)
3. **Kala Bala** — temporal (day/night, lunar phase)
4. **Cheshta Bala** — motional (retrograde/direct/stationary)
5. **Naisargika Bala** — natural (inherent planetary strength)
6. **Drik Bala** — aspectual (benefic/malefic aspects)

### Divisional Charts (16 Shodasha Vargas)
D-1 (Rashi) through D-60 (Shashtiamsha), including the critical D-9 (Navamsha) with correct sign-counting rules for movable/fixed/dual signs per BPHS Ch. 6.

---

## WHAT MAKES THE MCP INTEGRATION SPECIAL

Vedaksha doesn't just "support" MCP — it was architected around it:

```json
{
  "tools": [
    "compute_natal_chart",   // Full chart → ChartGraph JSON
    "compute_dasha",         // Dasha tree (fully functional)
    "compute_vargas",        // Divisional charts (fully functional)
    "compute_transit",       // Transit computation
    "search_transits",       // Find exact transit moments
    "search_muhurta",        // Find auspicious times
    "emit_graph"             // ChartGraph → Cypher/SurrealQL/JSON-LD
  ]
}
```

Every tool has:
- **JSON Schema** for input validation (AI agents auto-generate correct calls)
- **Structured errors** with `error_code`, `message`, `suggested_action` (AI agents self-correct)
- **Input validation** with clear range constraints (JD, lat, lon, search spans)

**An AI agent workflow:**
1. User: "What's in my birth chart?"
2. Agent geocodes birth place → (lat, lon), converts birth datetime → Julian Day
3. Agent calls `compute_natal_chart` with JD + coordinates
4. Receives `ChartGraph` with typed nodes and edges
5. Reads `highlights` → generates natural language summary
6. Optionally calls `emit_graph` → ingests into Neo4j for long-term storage

Zero custom integration code. Zero prompt engineering for output parsing. The types ARE the documentation.

---

## HERO SECTION CONTENT OPTIONS

### Option A: Developer-Tool Angle
**Headline:** The astronomical ephemeris for the agentic age.
**Subhead:** Clean-room Rust implementation. Sub-arcsecond precision. Built from the ground up for AI agents, MCP, and graph-native output.
**Install:** `$ cargo add vedaksha`

### Option B: Vedic-First Angle
**Headline:** Vedic astrology deserves a modern engine.
**Subhead:** 27 nakshatras. 50 yogas. 3 dasha systems. 16 divisional charts. 44 ayanamsha. All in Rust, all in the type system, all with sub-arcsecond precision.
**Install:** `$ cargo add vedaksha`

### Option C: Graph-Native Angle
**Headline:** Charts are graphs. Finally, software agrees.
**Subhead:** Every natal chart is a property graph with 10 node types and 13 edge types. Emit to Neo4j, SurrealDB, or JSON-LD. Embed in RAG pipelines. Query with Cypher.
**Install:** `$ cargo add vedaksha`

### Option D: Clean-Room Angle
**Headline:** Zero copyleft. Zero compromise.
**Subhead:** Every algorithm traced to a published source. Every constant independently derived. No GPL code was consulted, referenced, or adapted. BSL 1.1 with Apache 2.0 conversion.
**Install:** `$ cargo add vedaksha`

**Recommendation:** Use Option A as the primary hero. The other angles work as section leads further down the page.

---

## FEATURE GRID CONTENT (9 Cards)

| # | Label | Title | Description |
|---|-------|-------|-------------|
| 01 | PLANETARY ENGINE | Sub-arcsecond precision. | JPL DE440/441 ephemeris with IAU 2006 precession, IAU 2000B nutation, annual aberration, and iterative light-time correction. Covers 1800-2400 CE. |
| 02 | VEDIC FIRST-CLASS | Nakshatras, dashas, yogas. | 27 nakshatras, 3 dasha systems (Vimshottari/Yogini/Chara), 50 yogas, 16 vargas, complete Shadbala, and Vedic drishti — in the core type system, not a plugin. |
| 03 | 10 HOUSE SYSTEMS | Every tradition covered. | Placidus, Koch, Whole Sign, Equal, Campanus, Regiomontanus, Porphyry, Morinus, Alcabitius, Sripathi. Automatic polar fallback. |
| 04 | MCP NATIVE | Built for AI agents. | 7 typed tools with JSON schemas, structured errors with `suggested_action`, input validation, and streaming. Drop into any MCP-compatible agent. |
| 05 | GRAPH OUTPUT | Neo4j. SurrealDB. JSON-LD. | ChartGraph with 10 node types and 13 edge types. Deterministic IDs. Emit Cypher, SurrealQL, JSON-LD, or vector embeddings for RAG. |
| 06 | RUNS EVERYWHERE | Native. WASM. Python. | One Rust codebase compiles to native binaries, WebAssembly for browsers, and Python wheels via PyO3. 11 WASM functions, 10 Python functions. |
| 07 | PII-BLIND | Privacy by architecture. | Computation accepts ONLY Julian Day + coordinates. No names, birthdates, or personal data ever touches the engine. GDPR/DPDP compliant by design. |
| 08 | 44 AYANAMSHA | Every sidereal tradition. | Lahiri, Fagan-Bradley, Krishnamurti, Raman, Aryabhata, Surya Siddhanta, Galactic Center, Babylonian, and 36 more. All with precession-corrected computation. |
| 09 | CLEAN ROOM | Zero copyleft risk. | Derived exclusively from NASA JPL, IAU standards, Meeus, BPHS, and Holden. Every function cites its source. cargo-deny blocks AGPL/GPL/LGPL/SSPL. BSL 1.1. |

---

## COMPARISON: WHAT EXISTS TODAY VS VEDAKSHA

**Note:** This comparison uses GENERIC categories per our legal guardrails. No named products.

| Capability | Traditional C Libraries | Python Packages | REST API Services | **Vedaksha** |
|---|---|---|---|---|
| Pure functions / stateless | Global state, init required | Often stateful | Stateless | **Pure functions, zero state** |
| MCP native | No | No | No | **7 typed tools built-in** |
| Graph output | Flat C structs | Dicts/lists | JSON blobs | **10 node types, 13 edge types** |
| Graph database emit | No | No | No | **Cypher + SurrealQL + JSON-LD** |
| Vedic first-class | Afterthought | Partial | Partial | **50 yogas, 3 dashas, 16 vargas, 27 nakshatras** |
| Ayanamsha systems | 10-30 | 5-15 | 1-5 | **44 systems** |
| House systems | 8-12 | 3-6 | 1-3 | **10 systems** |
| WASM browser | No | No | N/A | **11 WASM functions** |
| Python bindings | N/A (C) | Native | REST | **10 PyO3 functions with type stubs** |
| PII-blind | No (accepts names) | No (accepts names) | No (requires PII) | **JD + coordinates only** |
| Clean-room | Varies | Wraps C libs | Unknown | **Every function cites source** |
| Unsafe code | Yes (C) | Yes (C FFI) | N/A | **Zero `unsafe` blocks** |
| Embedding/RAG output | No | No | No | **EmbeddingTextEmitter** |
| Deterministic IDs | No | No | No | **FNV-1a hash, same input = same ID** |
| Localization | English only | English only | English + few | **7 languages (en/hi/sa/ta/te/kn/bn)** |
| License risk | AGPL/GPL common | MIT/GPL mixed | Proprietary | **BSL 1.1 → Apache 2.0** |

---

*© 2026 ArthIQ Labs LLC. All rights reserved.*
*Vedaksha — Vision from Vedas*
*info@arthiq.net | vedaksha.net*
