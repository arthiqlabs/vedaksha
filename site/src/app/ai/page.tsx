export default function AIPage() {
  const pillars = [
    {
      num: "01",
      title: "Pure Functions, Zero State",
      desc: "No initialization, no cleanup, no global state. Pass inputs, get outputs. Every computation is a pure function call — ideal for stateless AI agent invocations.",
    },
    {
      num: "02",
      title: "Semantic Type System",
      desc: "Body::Jupiter, not integer 5. Sign::Aries, not 0. Your agent reads typed enums that carry meaning, eliminating an entire class of mapping errors.",
    },
    {
      num: "03",
      title: "Graph-Native Output",
      desc: "Every chart is a ChartGraph with typed nodes and edges. AI agents excel at traversing structured relationships — and that is exactly what a chart becomes.",
    },
    {
      num: "04",
      title: "MCP-Native Protocol",
      desc: "7 tools exposed via the Model Context Protocol. OAuth 2.1 authentication, JSON-RPC 2.0 transport, streaming support. Your agent connects and starts computing.",
    },
    {
      num: "05",
      title: "Chart Highlights",
      desc: "Not every aspect matters equally. Vedaksha ranks significant chart features by strength and relevance, so your agent can summarize what matters most.",
    },
    {
      num: "06",
      title: "Natural Language Fields",
      desc: "Every transit event, yoga, and dasha period includes an nl_description field — a pre-written natural language explanation ready for your agent to relay.",
    },
    {
      num: "07",
      title: "Embedding-Ready Text",
      desc: "The EmbeddingTextEmitter produces optimized text chunks for vector stores. Build RAG pipelines over astrological knowledge without custom text extraction.",
    },
    {
      num: "08",
      title: "Streaming Results",
      desc: "Transit searches return Stream<TransitEvent> via MCP streaming. Your agent can process results as they arrive, not after the full search completes.",
    },
    {
      num: "09",
      title: "PII-Blind",
      desc: "The computation engine never sees personal data. It receives a Julian Day and coordinates — no names, no birth certificates, no data to protect.",
    },
    {
      num: "10",
      title: "Deterministic IDs",
      desc: "Same input always produces the same graph node and edge IDs. Your agent can compare charts across sessions, build incremental knowledge graphs, and detect duplicates.",
    },
  ];

  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="relative flex flex-col items-center px-6 pt-28 pb-20 overflow-hidden">
        <div className="relative flex flex-col items-center text-center max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            AI-Native Architecture
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            Why Vedākṣha is{" "}
            <span className="text-[#D4A843]">Built for AI</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            Vedākṣha is the first astronomical computation platform designed from
            the ground up for AI agents. Every API decision, every data
            structure, every output format was chosen to make AI integration
            effortless and reliable.
          </p>
        </div>
      </section>

      {/* Navigation */}
      <section className="border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto px-6 py-6">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Explore the AI Documentation
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {[
              { href: "/ai/mcp-tools", label: "MCP Tool Catalog", desc: "7 tools your agent can call" },
              { href: "/ai/graph", label: "Graph Output", desc: "Charts as property graphs" },
              { href: "/ai/patterns", label: "Agent Patterns", desc: "8 real-world workflows" },
              { href: "/ai/comparison", label: "Why Vedākṣha", desc: "Feature comparison matrix" },
              { href: "/ai/quickstart", label: "5-Min Quickstart", desc: "Start building in minutes" },
            ].map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="flex flex-col border border-[var(--color-brand-border)] rounded-lg px-4 py-3 hover:bg-[var(--color-brand-bg)] transition-colors"
              >
                <span className="text-sm font-semibold text-[var(--color-brand-text)]">{link.label}</span>
                <span className="text-xs text-[var(--color-brand-text-muted)]">{link.desc}</span>
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* 10 Design Pillars */}
      <section className="py-20 px-6 border-t border-[var(--color-brand-border)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] text-center mb-3">
            Design Pillars
          </p>
          <h2 className="text-3xl font-bold tracking-tight uppercase text-center text-[var(--color-brand-text)] mb-4">
            10 reasons AI agents{" "}
            <span className="text-[#D4A843]">prefer Vedaksha</span>
          </h2>
          <p className="text-base text-center text-[var(--color-brand-text-secondary)] max-w-2xl mx-auto mb-12">
            These are not afterthoughts bolted onto a legacy library. They are
            foundational decisions that shape every line of the codebase.
          </p>

          <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 md:grid-cols-2 gap-px overflow-hidden">
            {pillars.map((p) => (
              <div
                key={p.num}
                className="bg-[var(--color-brand-bg)] p-6 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
              >
                <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                  {p.num}
                </span>
                <h3 className="text-base font-semibold uppercase tracking-wide mt-1.5 mb-2">
                  <span className="text-[#D4A843]">{p.title}</span>
                </h3>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {p.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* The AI Agent Contract */}
      <section className="py-20 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
            The Contract
          </p>
          <h2 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-6">
            What your agent <span className="text-[#D4A843]">can rely on</span>
          </h2>
          <div className="space-y-4">
            {[
              "Every MCP tool has a complete JSON schema — your agent knows the input format before calling.",
              "Every error includes a structured error code and a self-correction hint — your agent can retry intelligently.",
              "Every output field uses semantic types — your agent never needs to decode magic numbers.",
              "Every computation is deterministic — same inputs always produce the same outputs.",
              "Every chart can be emitted as a graph — your agent can store, query, and traverse relationships.",
              "No computation requires prior state — your agent can call any tool at any time.",
            ].map((item) => (
              <div key={item} className="flex items-start gap-3">
                <span className="mt-1.5 size-1.5 rounded-full bg-[#D4A843] shrink-0" />
                <span className="text-base leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {item}
                </span>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 px-6 border-t border-[var(--color-brand-border)]">
        <div className="max-w-xl mx-auto text-center">
          <h2 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
            Start <span className="text-[#D4A843]">building</span>.
          </h2>
          <p className="text-sm text-[var(--color-brand-text-secondary)] mb-8">
            Connect your AI agent to Vedākṣha in under 5 minutes.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href="/ai/quickstart"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg bg-[var(--color-brand-text)] text-white hover:opacity-90 transition-opacity"
            >
              Quickstart Guide
            </a>
            <a
              href="/ai/mcp-tools"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg border border-[var(--color-brand-border)] text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              MCP Tool Catalog
            </a>
          </div>
        </div>
      </section>
    </div>
  );
}
