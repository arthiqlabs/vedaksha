export default function GraphPage() {
  const nodeTypes = [
    { name: "Planet", desc: "A celestial body with longitude, sign, nakshatra, and dignity" },
    { name: "House", desc: "A house cusp with degree, sign, and ruling planet" },
    { name: "Sign", desc: "A zodiac sign with element, modality, and ruling planet" },
    { name: "Nakshatra", desc: "A lunar mansion with pada, deity, and ruling planet" },
    { name: "Aspect", desc: "A geometric relationship between two planets" },
    { name: "Yoga", desc: "A Vedic combination pattern with participating planets" },
    { name: "Dignity", desc: "A planet's strength state — exaltation, own sign, debilitation" },
    { name: "Dasha", desc: "A planetary period node in the dasha tree" },
    { name: "Transit", desc: "A planetary position at a specific moment" },
    { name: "Chart", desc: "The root node tying all elements together" },
  ];

  const edgeTypes = [
    { name: "PLACED_IN", from: "Planet", to: "House", desc: "Planet occupies a house" },
    { name: "IN_SIGN", from: "Planet", to: "Sign", desc: "Planet is positioned in a zodiac sign" },
    { name: "IN_NAKSHATRA", from: "Planet", to: "Nakshatra", desc: "Planet occupies a lunar mansion" },
    { name: "RULES", from: "Planet", to: "Sign", desc: "Planet is the ruler of a sign" },
    { name: "ASPECTS", from: "Planet", to: "Planet", desc: "Geometric aspect between two planets" },
    { name: "CONJOINS", from: "Planet", to: "Planet", desc: "Two planets share the same degree region" },
    { name: "HAS_DIGNITY", from: "Planet", to: "Dignity", desc: "Planet holds a dignity state" },
    { name: "FORMS_YOGA", from: "Planet", to: "Yoga", desc: "Planet participates in a yoga combination" },
    { name: "CUSP_IN", from: "House", to: "Sign", desc: "House cusp falls in a sign" },
    { name: "DISPOSITS", from: "Planet", to: "Planet", desc: "Dispositorship (sign lord) chain" },
    { name: "CONTAINS", from: "Chart", to: "Planet", desc: "Chart contains a planet" },
    { name: "HAS_HOUSE", from: "Chart", to: "House", desc: "Chart contains a house" },
    { name: "TRANSITS", from: "Transit", to: "Planet", desc: "Transit activates a natal planet" },
  ];

  const formats = [
    {
      name: "Cypher (Neo4j)",
      desc: "CREATE statements for nodes and MERGE for relationships. Load directly into Neo4j with a single query batch.",
      example: `CREATE (p:Planet {id: "planet_jupiter_40.2", name: "Jupiter", longitude: 40.2, sign: "Taurus"})
CREATE (h:House {id: "house_1_15.7", number: 1, degree: 15.7, sign: "Aries"})
MERGE (p)-[:PLACED_IN]->(h)`,
    },
    {
      name: "SurrealQL (SurrealDB)",
      desc: "INSERT statements with typed record links. Native graph traversal with SurrealDB's RELATE syntax.",
      example: `INSERT INTO planet {id: planet:jupiter_40_2, name: "Jupiter", longitude: 40.2, sign: "Taurus"};
RELATE planet:jupiter_40_2->placed_in->house:1_15_7;`,
    },
    {
      name: "JSON-LD",
      desc: "Linked data format with schema.org-compatible context. Interoperable with knowledge graph standards.",
      example: `{
  "@context": "https://vedaksha.net/schema/v1",
  "@type": "NatalChart",
  "planets": [{
    "@type": "Planet",
    "name": "Jupiter",
    "longitude": 40.2,
    "sign": "Taurus"
  }]
}`,
    },
    {
      name: "JSON",
      desc: "Standard JSON with typed fields. The default output format for all MCP tool responses.",
      example: `{
  "nodes": [{"id": "planet_jupiter", "type": "Planet", "longitude": 40.2}],
  "edges": [{"from": "planet_jupiter", "to": "house_1", "type": "PLACED_IN"}]
}`,
    },
    {
      name: "Embedding Text (RAG)",
      desc: "Optimized text chunks for vector embedding. Each chunk is a self-contained fact suitable for retrieval-augmented generation.",
      example: `Jupiter is at 40.2 degrees in Taurus in the 1st house.
Jupiter is exalted, indicating strong beneficial influence.
Jupiter aspects the 5th, 7th, and 9th houses from its position.`,
    },
  ];

  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="px-6 pt-28 pb-16">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Graph Output
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            Charts are <span className="text-[#D4A843]">graphs</span>. Finally.
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            An astrological chart is a web of relationships — planets in signs,
            signs ruled by planets, planets aspecting planets. A property graph
            is the natural representation of this structure, and AI agents excel
            at traversing structured relationships.
          </p>
        </div>
      </section>

      {/* Why graphs for AI */}
      <section className="px-6 py-16 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            Why Graphs Matter for AI
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-8">
            Structure that agents <span className="text-[#D4A843]">understand</span>
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {[
              {
                title: "Traversal over Parsing",
                desc: "An AI agent can follow edges from Jupiter to its sign, house, and aspects without parsing text or decoding integers. The relationships are explicit.",
              },
              {
                title: "Cross-Chart Queries",
                desc: "Store multiple charts in a graph database and query across them. \"Find all charts where Saturn aspects the 7th house lord\" becomes a simple graph pattern.",
              },
              {
                title: "RAG-Ready Chunks",
                desc: "The embedding text emitter produces one fact per line. Each fact is self-contained and embeddable, making vector search over astrological knowledge trivial.",
              },
            ].map((item) => (
              <div
                key={item.title}
                className="border border-[var(--color-brand-border)] rounded-xl p-6 bg-[var(--color-brand-bg)]"
              >
                <h3 className="text-sm font-semibold uppercase tracking-wide text-[#D4A843] mb-2">
                  {item.title}
                </h3>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {item.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Node types */}
      <section className="px-6 py-16 border-t border-[var(--color-brand-border)]">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            Ontology
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-8">
            10 <span className="text-[#D4A843]">node types</span>
          </h2>
          <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
            <div className="bg-[var(--color-brand-bg-subtle)] px-4 py-2 border-b border-[var(--color-brand-border)] grid grid-cols-[150px_1fr] gap-4">
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Node Type</span>
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Description</span>
            </div>
            {nodeTypes.map((node) => (
              <div
                key={node.name}
                className="px-4 py-2.5 grid grid-cols-[150px_1fr] gap-4 border-b border-[var(--color-brand-border)] last:border-b-0"
              >
                <code className="text-xs font-mono text-[#D4A843]">{node.name}</code>
                <span className="text-sm text-[var(--color-brand-text-secondary)]">{node.desc}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Edge types */}
      <section className="px-6 py-16 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            Relationships
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-8">
            13 <span className="text-[#D4A843]">edge types</span>
          </h2>
          <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
            <div className="bg-[var(--color-brand-bg)] px-4 py-2 border-b border-[var(--color-brand-border)] grid grid-cols-[140px_80px_80px_1fr] gap-4">
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Edge</span>
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">From</span>
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">To</span>
              <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Meaning</span>
            </div>
            {edgeTypes.map((edge) => (
              <div
                key={edge.name}
                className="px-4 py-2.5 grid grid-cols-[140px_80px_80px_1fr] gap-4 border-b border-[var(--color-brand-border)] last:border-b-0"
              >
                <code className="text-xs font-mono text-[#D4A843]">{edge.name}</code>
                <code className="text-xs font-mono text-[var(--color-brand-text-muted)]">{edge.from}</code>
                <code className="text-xs font-mono text-[var(--color-brand-text-muted)]">{edge.to}</code>
                <span className="text-sm text-[var(--color-brand-text-secondary)]">{edge.desc}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Output formats */}
      <section className="px-6 py-16 border-t border-[var(--color-brand-border)]">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            Output Formats
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-8">
            5 ways to <span className="text-[#D4A843]">emit</span> a chart
          </h2>
          <div className="space-y-8">
            {formats.map((fmt) => (
              <div key={fmt.name}>
                <h3 className="text-base font-semibold text-[var(--color-brand-text)] mb-1">
                  {fmt.name}
                </h3>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mb-3">
                  {fmt.desc}
                </p>
                <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                  <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                    <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">
                      {fmt.name.toLowerCase().replace(/[^a-z]/g, "_")}
                    </span>
                  </div>
                  <pre className="p-4 overflow-x-auto text-sm leading-6 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
                    <code>{fmt.example}</code>
                  </pre>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-16 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-xl mx-auto text-center">
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
            Build a knowledge <span className="text-[#D4A843]">graph</span>.
          </h2>
          <p className="text-sm text-[var(--color-brand-text-secondary)] mb-6">
            See how AI agents use graph output in real-world workflows.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href="/ai/patterns"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg bg-[var(--color-brand-text)] text-white hover:opacity-90 transition-opacity"
            >
              Agent Patterns
            </a>
            <a
              href="/ai/mcp-tools"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg border border-[var(--color-brand-border)] text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg)] transition-colors"
            >
              MCP Tool Catalog
            </a>
          </div>
        </div>
      </section>
    </div>
  );
}
