export default function McpPage() {
  const tools = [
    {
      name: "compute_natal_chart",
      signature: "(julian_day, latitude, longitude, config?)",
      desc: "Computes a full natal chart and returns the complete ChartGraph JSON. Includes planets, house cusps, nakshatras, aspects, dignities, and yogas.",
    },
    {
      name: "compute_dasha",
      signature: "(chart_id, system?)",
      desc: "Returns the dasha tree for a previously computed chart. Supports Vimshottari, Yogini, and Chara dasha systems with up to 5 levels of sub-periods.",
    },
    {
      name: "compute_vargas",
      signature: "(chart_id, divisions[])",
      desc: "Computes one or more of the 16 Shodasha Varga divisional charts. Each varga is returned as an independent ChartGraph with its own planetary positions.",
    },
    {
      name: "emit_graph",
      signature: "(chart_id, format)",
      desc: "Emits the ChartGraph in the requested format. Supported formats: Cypher (Neo4j), SurrealQL (SurrealDB), JSON-LD, JSON, and EmbeddingText for RAG pipelines.",
    },
    {
      name: "search_transits",
      signature: "(planet, target, start_jd, end_jd, config?)",
      desc: "Streams exact transit moments in a date range. Returns TransitEvent objects with Julian Day, ingress/egress flag, and natural language description.",
    },
    {
      name: "search_muhurta",
      signature: "(criteria, start_jd, end_jd, location)",
      desc: "Finds auspicious time windows matching tithi, nakshatra, weekday, and hora criteria. Results are scored and sorted by combined auspiciousness.",
    },
    {
      name: "describe_chart",
      signature: "(chart_id, locale?)",
      desc: "Returns pre-written natural language descriptions for the chart's most significant features — yogas, planetary dignities, and dasha summary. Ready for agent relay.",
    },
  ];

  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        MCP Integration
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        7 tools your agent <span className="text-[#D4A843]">already knows.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha exposes 7 typed tools via the Model Context Protocol (MCP).
        Every tool has a JSON schema — so any MCP-compatible agent can call them
        without custom prompting or output parsing.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-4 max-w-2xl">
        Transport: stdio (local) or Streamable HTTP (remote). JSON-RPC 2.0.
        Errors are structured with machine-readable codes and self-correction
        hints so your agent can retry intelligently.
      </p>
      <div className="rounded-xl border border-[var(--color-brand-border)] p-5 bg-[var(--color-brand-bg-subtle)] mb-12 max-w-2xl">
        <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[#D4A843] mb-3">Quick start</p>
        <pre className="text-xs font-mono leading-relaxed text-[var(--color-brand-text-secondary)]">
{`cargo install vedaksha-mcp
vedaksha-mcp              # stdio (Claude Desktop, VS Code)
vedaksha-mcp --http       # HTTP on port 3100 (remote)
docker run -p 3100:3100 ghcr.io/arthiqlabs/vedaksha-mcp`}
        </pre>
      </div>

      {/* Tool list */}
      <div className="space-y-3 mb-14">
        {tools.map((tool) => (
          <div
            key={tool.name}
            className="border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
          >
            <div className="flex flex-col sm:flex-row sm:items-start gap-2 mb-2">
              <code className="text-sm font-mono font-semibold text-[#D4A843] shrink-0">
                {tool.name}
              </code>
              <code className="text-xs font-mono text-[var(--color-brand-text-muted)] sm:mt-0.5">
                {tool.signature}
              </code>
            </div>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              {tool.desc}
            </p>
          </div>
        ))}
      </div>

      {/* Links */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-12">
        <a
          href="/ai/mcp-tools"
          className="group block border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors no-underline"
        >
          <h3 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
            Full Tool Catalog
          </h3>
          <p className="text-xs text-[var(--color-brand-text-muted)] mb-3">
            Complete JSON schemas, parameter types, return shapes, and error codes for all 7 tools.
          </p>
          <span className="text-xs font-semibold text-[#D4A843] group-hover:underline">
            {"View catalog →"}
          </span>
        </a>
        <a
          href="/ai/quickstart"
          className="group block border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors no-underline"
        >
          <h3 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
            Agent Quickstart
          </h3>
          <p className="text-xs text-[var(--color-brand-text-muted)] mb-3">
            Connect Claude, GPT, or a custom agent to Vedākṣha MCP in under 10 minutes.
          </p>
          <span className="text-xs font-semibold text-[#D4A843] group-hover:underline">
            {"Setup guide →"}
          </span>
        </a>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/graph"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Graph Output"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/ai"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"AI Overview →"}
        </a>
      </div>
    </div>
  );
}
