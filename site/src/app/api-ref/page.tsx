const crates = [
  {
    name: "vedaksha-math",
    description: "Numeric primitives — Chebyshev polynomials, angle arithmetic, interpolation, rotation matrices",
    items: ["chebyshev_compute()", "normalize_degrees()", "lagrange_interpolate()", "Matrix3, Vector3"],
  },
  {
    name: "vedaksha-ephem-core",
    description: "Astronomy engine — JPL SPK reader, coordinate transforms, precession, nutation",
    items: ["apparent_position()", "SpkReader", "precession_matrix()", "nutation()", "delta_t()", "mean_node() / true_node() / true_node_osculating()"],
  },
  {
    name: "vedaksha-astro",
    description: "Western astrology — houses, aspects, dignities, transits, chart orchestrator",
    items: ["compute_chart()", "compute_houses()", "find_aspects()", "search_transits()", "solar_return()", "44 ayanamsha systems"],
  },
  {
    name: "vedaksha-vedic",
    description: "Vedic astrology — nakshatras, dashas, vargas, yogas, shadbala, muhurta",
    items: ["compute_vimshottari()", "compute_yogini()", "detect_yogas()", "compute_shadbala_full()", "assess_muhurta()", "varga_sign()"],
  },
  {
    name: "vedaksha-graph",
    description: "Property graph model — 10 node types, 13 edge types, deterministic IDs",
    items: ["ChartGraph", "Node / Edge", "NodeId", "DataClassification"],
  },
  {
    name: "vedaksha-emit",
    description: "Graph emitters — Cypher, SurrealQL, JSON-LD, JSON, embedding text",
    items: ["CypherEmitter", "SurrealEmitter", "JsonLdEmitter", "JsonGraphEmitter", "EmbeddingTextEmitter"],
  },
  {
    name: "vedaksha-mcp",
    description: "MCP server — 7 JSON-RPC tools for AI agents",
    items: ["McpServer", "compute_natal_chart", "compute_dasha", "emit_graph", "search_transits"],
  },
  {
    name: "vedaksha-locale",
    description: "Localization — planet, sign, nakshatra names in 7 languages",
    items: ["planet_name()", "sign_name()", "nakshatra_name()", "Language enum"],
  },
  {
    name: "vedaksha-wasm",
    description: "WebAssembly bindings — browser-ready computation with zero server",
    items: ["compute_dasha()", "get_nakshatra()", "compute_houses()", "find_aspects()", "tropical_to_sidereal()"],
  },
];

export default function ApiPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <h1 className="text-2xl font-bold tracking-tight text-[var(--color-brand-text)] mb-2">
        API Reference
      </h1>
      <p className="text-[var(--color-brand-text-secondary)] mb-10 max-w-2xl">
        9 workspace crates, 212 public API items. Full rustdoc available after
        build with <code className="text-sm bg-[var(--color-brand-bg-code)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5">cargo doc --workspace --open</code>
      </p>

      <div className="space-y-6">
        {crates.map((crate) => (
          <div
            key={crate.name}
            className="border border-[var(--color-brand-border)] rounded-xl p-6"
          >
            <h2 className="font-mono text-base font-semibold text-[var(--color-brand-text)] mb-1">
              {crate.name}
            </h2>
            <p className="text-sm text-[var(--color-brand-text-secondary)] mb-3">
              {crate.description}
            </p>
            <div className="flex flex-wrap gap-2">
              {crate.items.map((item) => (
                <span
                  key={item}
                  className="text-xs font-mono bg-[var(--color-brand-bg-code)] border border-[var(--color-brand-border)] rounded px-2 py-1 text-[var(--color-brand-text-secondary)]"
                >
                  {item}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
