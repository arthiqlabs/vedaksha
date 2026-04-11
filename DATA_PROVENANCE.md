# Data Provenance & Production Readiness Tracker

> **MUST be reviewed and every item resolved before v1.0 release.**
> This document tracks every place where development uses sample data, subsets,
> approximations, or relaxed tolerances that differ from production requirements.

---

## Status Legend

| Status | Meaning |
|--------|---------|
| **DEV** | Using development/subset data — must be upgraded for production |
| **APPROX** | Using a simplified model — evaluate if upgrade needed |
| **RELAXED** | Test tolerance is wider than spec requires — must tighten |
| **MISSING** | Data/feature not yet implemented — required by spec |
| **OK** | Production-ready |

---

## 1. Ephemeris Data File

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **File** | `data/de440s.bsp` (DE440 short, 31 MB) | DE441 full or DE440 full | **DEV** |
| **Time range** | 1849–2150 CE | 1800–2400 CE (spec Section 8, Phase 2) | **DEV** |
| **Accuracy** | DE440 = DE441 for modern era | DE441 for extended range | **DEV** |
| **Format** | NAIF SPK/DAF binary | Same (pivot from legacy Linux binary was correct) | **OK** |
| **Embedded data** | Not yet implemented | `include_bytes!` for 1800–2400 CE subset (spec Section 9.1) | **MISSING** |
| **EphemerisProvider trait** | File-based only (`SpkReader`) | File, embedded, HTTP, custom (spec Section 9.1) | **DEV** |

**Action:** Download `de440.bsp` (114 MB, 1550–2650 CE) or `de441.bsp` (3.3 GB) for production. Implement embedded provider for WASM use.

---

## 2. Nutation Model

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Model** | IAU 2000B (77 terms, ~1 mas accuracy) | Spec says "IAU 2000A nutation" (1365 terms, 0.1 mas) | **APPROX** |
| **Accuracy** | ~1 milliarcsecond | 0.001 arcsecond = 1 mas (spec Section 9.2) | **APPROX** |
| **Source** | McCarthy & Luzum (2003) | Mathews, Herring & Buffett (2002) for full 2000A | **APPROX** |

**Action:** Evaluate whether IAU 2000B's 1 mas accuracy is sufficient for the 0.001 arcsecond spec target. If not, implement full IAU 2000A (1365 terms). For astrological use, 2000B is more than adequate. For matching JPL Horizons to sub-arcsecond, it should be fine since nutation is a small correction.

---

## 3. Test Oracle Data

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **JPL Horizons comparison** | No Horizons data yet — tests use sanity-check ranges (e.g., "Sun at 275–285 degrees") | 10,000+ planetary position data points from JPL Horizons (spec Section 9, Acceptance Criteria) | **MISSING** |
| **House cusp oracle** | Not yet implemented | 1,000+ house cusp data points (spec Section 9) | **MISSING** |
| **IAU SOFA comparison** | Not yet implemented | Sidereal time matches SOFA to < 0.001 seconds (spec Section 9.2) | **MISSING** |
| **Precision tolerance** | Tests use 1–10 degree ranges | Planetary longitudes match Horizons to < 0.001 arcsecond | **RELAXED** |

**Action:** Generate test data from JPL Horizons API for 100+ dates spanning 1900–2100 CE. Compare computed longitudes. Current integration tests are sanity checks, not precision validation.

---

## 4. Earth Orientation Parameters (EOP)

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **UT1-UTC** | Not implemented | Required for precise sidereal time | **MISSING** |
| **Polar motion** | Not implemented | Required for precise topocentric positions | **MISSING** |
| **EOP data file** | Not present | `finals2000A.all` from IERS | **MISSING** |

**Action:** Implement `eop.rs` module. For astrological use, EOP omission introduces ~0.5–1" error (acceptable). For spec compliance (< 0.001" positions), EOP is needed.

---

## 5. Delta T (TT − UT1)

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Implementation** | Espenak & Meeus polynomial expressions covering -500 to 2150+ CE | Complete | **OK** |
| **Historical table** | IERS measured table 1620-2025, predictions to 2150 | Complete — measured values 1620-2025, polynomial predictions beyond | **OK** |
| **Prediction polynomial** | Included in delta_t.rs for all eras | Complete (degrades for distant future) | **OK** |

**Action:** Consider adding discrete historical table for sub-second accuracy in 1900-2020 range. Current polynomial is sufficient for astrological use.

---

## 6. Coordinate Pipeline Limitations

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Frame tie rotation** | ICRS-to-J2000 frame bias applied (23 mas) | ICRS ↔ FK5 frame tie (0.01" level) | **OK** |
| **Relativistic light deflection** | Not applied | Deflection by Sun (~1.75" max) | **MISSING** |
| **Equation of the equinoxes** | Basic (dpsi * cos(eps)) | Full with complementary terms | **APPROX** |
| **Diurnal aberration** | Not implemented | Required for topocentric positions | **MISSING** |

**Action:** These are sub-arcsecond corrections. Implement for spec compliance. Solar light deflection is the most significant (~1.75" for bodies near the Sun).

---

## 7. Astrology Layer (Phase 3) — vedaksha-astro

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Ayanamsha systems** | 44 systems implemented (43 named + Tropical) | Spec says 43+ systems — exceeded | **OK** |
| **Ayanamsha precision** | Quadratic precession model, 20 systems tested, avg 0.004° | Polynomial models for all major systems | **OK** |
| **House system tests** | Validated against oracle — all 9 systems sub-0.001°, 12,600 cusp comparisons | Validated against secondary oracle with 1,000+ data points (spec Section 9) | **OK** |
| **Placidus convergence** | RA iteration, sub-0.001° verified | May need Newton-Raphson for edge cases near polar boundary | **OK** |
| **Koch implementation** | Rewritten, 100% oracle match, sub-0.001° | Verify against Holden reference values | **OK** |
| **Chart orchestrator** | `compute_chart()` wires positions → houses → aspects → dignities, 8 tests | Complete | **OK** |
| **Accidental dignities** | Cazimi, Combust, Under Sunbeams, Retrograde, Angular/Succedent/Cadent, 9 tests | Complete | **OK** |
| **Aspect pattern tests** | Basic pattern detection with synthetic data | Validate against known charts with verified patterns | **RELAXED** |

**Action:** Expand ayanamsha to 43+ systems. Implement chart orchestrator. Add accidental dignities. Validate house cusps against oracle data.

---

## 8. Vedic Astrology Layer (Phase 4) — vedaksha-vedic

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Nakshatras** | 27 nakshatras with padas, dasha lords, guna, gana | Complete per BPHS | **OK** |
| **Vimshottari Dasha** | 120-year cycle, 5 levels (Maha→Prana), balance calculation | Complete per BPHS Ch. 46-47 | **OK** |
| **Yogini Dasha** | 36-year 8-lord cycle, recursive sub-periods, 4 tests | Complete per BPHS | **OK** |
| **Chara Dasha** | Jaimini sign-based, 12 sign periods, 4 tests | Simplified implementation | **APPROX** |
| **Vargas** | 16 Shodasha Varga charts (D-1 through D-60) | Complete per BPHS Ch. 6-7 | **OK** |
| **Yogas** | 50 yoga types implemented from BPHS and Phaladipika | Spec requirement met (50+) | **OK** |
| **Shadbala** | All 6 components: Sthana, Dig, Naisargika, Kala, Cheshta, Drik Bala | Complete per BPHS Ch. 27 | **OK** |
| **Drishti** | Full + special aspects (Mars/Jupiter/Saturn) | Complete per BPHS Ch. 26 | **OK** |
| **Bhava** | Whole-sign houses with kendra/trikona/dusthana classification | Complete per BPHS | **OK** |
| **Yoga tests** | Tested with synthetic planet positions | Validate against known charts with verified yogas | **RELAXED** |
| **Shadbala tests** | Unit tests for implemented components | Validate complete Shadbala against Raman's worked examples | **RELAXED** |

**Action:** Implement Yogini + Chara dasha. Expand yogas to 50+. Complete Shadbala (Kala/Cheshta/Drik Bala). Validate against textbook examples.

---

## 9. Graph & Emission Layers (Phases 5-6) — vedaksha-graph + vedaksha-emit

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Graph ontology** | 10 node types, 13 edge types, deterministic IDs, serde | Complete per spec | **OK** |
| **ChartGraph** | Construction, traversal, JSON serialization | Complete | **OK** |
| **CypherEmitter** | MERGE for global, CREATE for chart-scoped, edge properties | Validate against real Neo4j | **RELAXED** |
| **SurrealEmitter** | CREATE/RELATE with SurrealQL syntax | Validate against real SurrealDB | **RELAXED** |
| **JsonLdEmitter** | @context, @graph, reified edges | Validate against JSON-LD spec validators | **RELAXED** |
| **JsonGraphEmitter** | Serde roundtrip verified | Production-ready (canonical MCP format) | **OK** |
| **EmbeddingTextEmitter** | Natural language with planet/sign/aspect descriptions | Evaluate embedding quality with real RAG pipeline | **RELAXED** |
| **InMemoryGraphEmitter** | Not implemented | In-process traversable graph for pattern detection (spec) | **MISSING** |
| **Ontology JSON file** | Created at `ontology/vedaksha-ontology.json` | Complete | **OK** |

**Action:** Validate Cypher/SurrealQL/JSON-LD against real databases. Create published ontology JSON. Implement InMemoryGraphEmitter.

---

## 10. MCP Server (Phase 7) — vedaksha-mcp

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Tool count** | 4 tools defined (compute_natal, compute_dasha, compute_vargas, emit_graph) | 7 tools (+ compute_transit, search_transits, search_muhurta from Phase 10) | **DEV** |
| **compute_natal_chart** | Validates input, returns stub (needs EphemerisProvider) | Full chart computation returning ChartGraph JSON | **DEV** |
| **compute_dasha** | Fully functional — computes Vimshottari via vedaksha-vedic | Production-ready | **OK** |
| **compute_vargas** | Wired to vedaksha_vedic::varga — functional for single longitude | Full multi-planet computation needs positions | **OK** |
| **emit_graph** | Fully functional — all 5 emitter formats work | Production-ready | **OK** |
| **OAuth 2.1 + PKCE** | Not implemented | Required for remote deployments (spec Section 8) | **MISSING** |
| **Rate limiting** | Not implemented | 100 computations/min, 10 searches/min (spec) | **MISSING** |
| **Tool description SHA-256** | Not implemented | Hash published in server metadata for tamper detection | **MISSING** |
| **Scoped access tokens** | Not implemented | compute:chart, search:transits, emit:graph scopes | **MISSING** |

**Action:** Wire compute_natal/vargas to real EphemerisProvider. Implement OAuth + rate limiting. Add transit tools in Phase 10.

---

## 11. Bodies & Features Not Yet Implemented

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Mean/True Node** | Implemented in nodes.rs (Meeus Ch. 47) with 5 perturbation terms | Complete | **OK** |
| **Fixed stars** | Not implemented | Hipparcos catalog (`data/stars_hip.bin`) | **MISSING** |
| **Asteroids** | Not implemented | MPC orbital elements (`data/asteroids_essential.bin`) | **MISSING** |
| **Analytical fallback** | Not implemented | Truncated VSOP87 + ELP-2000 (spec Phase 2C) | **MISSING** |

---

## 10. Data Files Referenced in Spec But Not Present

| File | Purpose | Status |
|------|---------|--------|
| `data/de441_1800_2400.bin` | Embedded JPL DE441 coefficients | **MISSING** — using de440s.bsp instead |
| `data/stars_hip.bin` | Hipparcos fixed star catalog | **MISSING** |
| `data/asteroids_essential.bin` | Curated asteroid orbital elements | **MISSING** |
| `ontology/vedaksha-ontology.json` | Published graph ontology spec | **OK** — created |
| `wit/vedaksha.wit` | WASM Component Model interfaces | **MISSING** |

---

## 11. Test Infrastructure Gaps

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Property-based tests** | None | `proptest` for random input invariants (spec Section 9) | **MISSING** |
| **Cross-platform tests** | macOS only | Native x86_64, aarch64, WASM (spec Section 9) | **MISSING** |
| **Benchmark suite** | None | Criterion benchmarks with regression detection (spec Section 9) | **MISSING** |
| **Coverage** | Not measured | 95%+ code coverage (spec Section 15) | **MISSING** |

---

## Pre-Release Checklist

Before v1.0, every **DEV**, **APPROX**, **RELAXED**, and **MISSING** item above must be resolved or explicitly accepted with documented rationale. Run this review:

```bash
# Count unresolved items
grep -c "DEV\|APPROX\|RELAXED\|MISSING" DATA_PROVENANCE.md
```

Current count: ~64 items to resolve.

---

---

## 12. Localization Layer (Phase 8) — vedaksha-locale

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Implementation strategy** | Static `&[&str]` lookup tables — `no_std` compatible, zero-allocation | Spec called for Mozilla Fluent (.ftl files); static tables are simpler, faster, and sufficient for fixed vocabulary | **OK** |
| **Languages** | 7 Tier 1: English, Hindi, Sanskrit (IAST), Tamil, Telugu, Kannada, Bengali | Same 7 languages per spec | **OK** |
| **Planet names** | 9 Vimshottari graha in all 7 languages | Complete per BPHS Ch. 3 | **OK** |
| **Zodiac sign names** | 12 rāśi in all 7 languages | Complete per BPHS Ch. 4 | **OK** |
| **Nakshatra names** | 27 nakshatras in all 7 languages | Complete per BPHS Ch. 3-6 | **OK** |
| **Aspect names** | 11 aspect types in all 7 languages | Vedic terms for conjunction/trine; neologisms for Western-only aspects | **OK** |
| **Dasha lord names** | 9 Vimshottari lords via `planets::planet_name` delegation | Complete | **OK** |
| **Yoga names** | 8 yogas in 7 languages | Extended set for remaining yogas | **OK** |
| **House names** | 12 houses in 7 languages | Complete | **OK** |
| **Dignity terms** | 5 dignity states in 7 languages | Complete | **OK** |

**Note on Fluent:** The spec mentioned Mozilla Fluent (.ftl files) for runtime message resolution. We implemented static lookup tables instead because: (a) the vocabulary is fully enumerable and fixed at compile time, (b) `no_std` compatibility is required for WASM builds, and (c) zero-allocation lookups are preferable for hot paths. The trade-off is losing Fluent's plural rules and parameter interpolation, neither of which is needed for bare noun lookups.

---

## 13. WASM & Python Bindings (Phase 9)

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **WASM functions** | 11 functions (dasha, nakshatra, varga, houses, aspects, sidereal, i18n) | Chart computation via embedded ephemeris (spec Section 8) | **DEV** |
| **WASM chart computation** | Not available (no embedded ephemeris) | `compute_chart` callable from JS with embedded DE441 data | **MISSING** |
| **WASM binary size** | Not measured | < 500 KB without data, < 8 MB with embedded data (spec) | **MISSING** |
| **WASM WASI target** | Not built | wasm32-wasip2 for Cloudflare Workers/Deno (spec) | **MISSING** |
| **WIT interfaces** | Not created | `wit/vedaksha.wit` WASM Component Model (spec) | **MISSING** |
| **Python functions** | 10 functions (dasha, nakshatra, varga, houses, sidereal, i18n) | Full API coverage including chart computation | **DEV** |
| **Python chart computation** | Not available (no ephemeris binding) | Full natal chart from Python | **MISSING** |
| **Python NumPy integration** | Not implemented | Accept/return numpy arrays for batch (spec) | **MISSING** |
| **Python type stubs** | `.pyi` file created | Complete per spec | **OK** |
| **Python package** | Not published | `pip install vedaksha` on PyPI (spec) | **MISSING** |

**Action:** Integrate embedded ephemeris for WASM chart computation. Build WASI target. Create WIT. Add NumPy batch support to Python. Full chart pipeline for both bindings.

---

## 14. Transit Search & Advanced Features (Phase 10)

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Transit search** | Adaptive step + bisection with callback-based longitude lookup, 14 tests | Stream<TransitEvent> via MCP streaming (spec) | **DEV** |
| **Transit search with real ephemeris** | Tests use synthetic linear longitude | Must work with real EphemerisProvider | **RELAXED** |
| **Solar return** | Implemented with bisection in 2-day window | Complete | **OK** |
| **Lunar return** | Implemented with 30-day scan + bisection | Complete | **OK** |
| **Synastry** | Inter-chart aspect detection, 6 tests | Combined ChartGraph with inter-chart ASPECTS edges (spec) | **DEV** |
| **Composite chart** | Midpoint method, 6 tests | Complete | **OK** |
| **Muhurta search** | Tithi, weekday, nakshatra quality assessment, 15 tests | Full criteria from Muhurta Chintamani (spec) | **APPROX** |
| **Muhurta yoga/karana** | Not implemented | Spec mentions yoga and karana panchanga elements | **MISSING** |
| **MCP transit tools** | Tool definitions + validation added, dispatched as stubs | Full computation needs EphemerisProvider | **DEV** |

**Action:** Wire transit/muhurta to MCP tools. Test with real ephemeris. Add yoga/karana panchanga elements. Produce ChartGraph output for synastry.

---

## 15. Website (Phase 12) — vedaksha.net

| Item | Current State | Production Requirement | Status |
|------|--------------|----------------------|--------|
| **Homepage** | Hero, 3 pillars, code example, stats, 9-feature grid, MCP tools, CTA | Complete per brand spec | **OK** |
| **Logo assets** | Orbital Astrolabe SVG (full, medium, favicon) inline in React | All 17 assets (including raster, lockups) | **DEV** |
| **Nav/Footer** | Frosted glass nav, dark mode toggle, attribution badges, VEDĀKṢHA wordmark | Complete | **OK** |
| **Fonts** | Inter + JetBrains Mono + Noto Sans Devanagari via next/font | Complete | **OK** |
| **Knowledge Base** | Unified /docs with 8 categories, 22 linked guide pages | Complete | **OK** |
| **Integration Guides** | 16 full guide pages (planetary, houses, vedic, sidereal, coords, time, aspects, transits, graph, MCP, WASM, Python, batch, errors, data sources, FAQ) | Complete | **OK** |
| **API Reference** | /api-ref with 9 crates, 212 API items | Complete | **OK** |
| **AI Documentation** | 6 pages: why AI-first, MCP tools, graph, patterns, comparison, quickstart | Complete | **OK** |
| **Pricing page** | /pricing with $500 one-time, Stripe Checkout link | Complete | **OK** |
| **Stripe integration** | Product + price + payment link created in Stripe (test mode) | Live mode switch needed | **DEV** |
| **Legal pages** | /terms (14 sections), /privacy (10 sections), /legal/bsl (full license text) | Complete | **OK** |
| **Blog** | /blog with 3 coming-soon article placeholders | Content needed | **DEV** |
| **About** | /about with mission, values, company info | Complete | **OK** |
| **Brand** | /brand with logo variants, 8-color palette, 3 font specimens, badge embeds | Complete | **OK** |
| **Playground** | /playground with coming-soon placeholder | WASM interactive (Phase 13) | **MISSING** |
| **SEO** | Basic metadata, favicon | Sitemap, OG images, JSON-LD structured data | **DEV** |
| **Deployment** | Local only | Vercel deployment | **MISSING** |
| **Repo legal files** | SECURITY.md, CONTRIBUTING.md in repo root | Complete | **OK** |

**Total site: 38 routes, all building clean. 22 documentation pages with real content.**

---

*Last updated: 2026-04-11*
*© 2026 ArthIQ Labs LLC. All rights reserved.*
