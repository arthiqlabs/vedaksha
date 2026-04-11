export default function GraphPage() {
  const nodeTypes = [
    "Chart",
    "Planet",
    "Sign",
    "House",
    "Nakshatra",
    "Pada",
    "Pattern",
    "DashaPeriod",
    "Yoga",
    "FixedStar",
  ];

  const edgeTypes = [
    { from: "Planet", rel: "IN_SIGN", to: "Sign" },
    { from: "Planet", rel: "IN_HOUSE", to: "House" },
    { from: "Planet", rel: "IN_NAKSHATRA", to: "Nakshatra" },
    { from: "Planet", rel: "IN_PADA", to: "Pada" },
    { from: "Planet", rel: "ASPECTS", to: "Planet" },
    { from: "Planet", rel: "RULES", to: "Sign" },
    { from: "Planet", rel: "PARTICIPATES_IN", to: "Yoga" },
    { from: "House", rel: "STARTS_IN", to: "Sign" },
    { from: "Chart", rel: "HAS_PLANET", to: "Planet" },
    { from: "Chart", rel: "HAS_PATTERN", to: "Pattern" },
    { from: "Chart", rel: "HAS_DASHA", to: "DashaPeriod" },
    { from: "DashaPeriod", rel: "HAS_ANTARDASHA", to: "DashaPeriod" },
    { from: "Chart", rel: "CONTAINS_FIXED_STAR", to: "FixedStar" },
  ];

  const emitters = [
    {
      name: "Cypher",
      target: "Neo4j",
      desc: "MERGE statements with deterministic IDs. Drop-in to any Neo4j import pipeline.",
    },
    {
      name: "SurrealQL",
      target: "SurrealDB",
      desc: "RELATE syntax with typed record IDs. Works with SurrealDB graph traversal queries.",
    },
    {
      name: "JSON-LD",
      target: "Linked Data",
      desc: "Schema.org-compatible context with custom Vedic terms. SPARQL and RDF compatible.",
    },
    {
      name: "JSON",
      target: "General",
      desc: "Flat JSON representation of the full ChartGraph. Zero dependencies, portable everywhere.",
    },
    {
      name: "Embedding Text",
      target: "Vector Stores",
      desc: "RAG-optimized text chunks, one per node. Feed directly to OpenAI, Cohere, or any embedding model.",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Graph Output
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Charts are <span className="text-[#D4A843]">graphs.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Every natal chart Vedākṣha computes is simultaneously a property graph.
        The same computation that returns planetary longitudes also produces a
        ChartGraph — a typed, traversable graph with 10 node types and 13 edge
        types, ready for Neo4j, SurrealDB, or a vector store.
      </p>

      {/* Ontology */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Ontology — 10 Node Types
        </h2>
        <div className="flex flex-wrap gap-2">
          {nodeTypes.map((node) => (
            <span
              key={node}
              className="font-mono text-xs px-3 py-1.5 rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text)]"
            >
              {node}
            </span>
          ))}
        </div>
      </div>

      {/* Relationship Diagram */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          13 Edge Types — Relationships
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {edgeTypes.map((edge) => (
            <div
              key={`${edge.from}-${edge.rel}-${edge.to}`}
              className="bg-[var(--color-brand-bg)] px-5 py-3 flex items-center gap-2 font-mono text-xs hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              <span className="text-[var(--color-brand-text-muted)]">{edge.from}</span>
              <span className="text-[var(--color-brand-text-muted)]">{"→"}</span>
              <span className="text-[#D4A843] font-semibold">{edge.rel}</span>
              <span className="text-[var(--color-brand-text-muted)]">{"→"}</span>
              <span className="text-[var(--color-brand-text-muted)]">{edge.to}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Emitters */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          5 Emitter Formats
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {emitters.map((e) => (
            <div
              key={e.name}
              className="border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="font-mono text-sm font-semibold text-[var(--color-brand-text)]">
                  {e.name}
                </span>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[#D4A843]/10 text-[#D4A843] border border-[#D4A843]/20">
                  {e.target}
                </span>
              </div>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-muted)]">
                {e.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Deterministic IDs */}
      <div className="rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Deterministic IDs
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
          Every node in a ChartGraph has an ID derived deterministically from its
          content — the same Julian Day and coordinates always produce the same
          graph with the same node IDs. This makes MERGE-based upserts in Neo4j
          and SurrealDB idempotent: re-import the same chart and nothing changes.
          It also means graph IDs can be used as stable foreign keys in relational
          or document stores.
        </p>
      </div>

      <div className="mt-12 flex items-center gap-6">
        <a
          href="/docs/vedic"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Vedic Astrology"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/mcp"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"MCP Integration →"}
        </a>
      </div>
    </div>
  );
}
