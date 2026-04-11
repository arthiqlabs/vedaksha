export default function McpIntegrationPage() {
  const tools = [
    {
      name: "compute_natal_chart",
      params: [
        { name: "julian_day", type: "f64", desc: "Julian Day in Terrestrial Time (TT)." },
        { name: "latitude", type: "f64", desc: "Geographic latitude in decimal degrees. Negative = south." },
        { name: "longitude", type: "f64", desc: "Geographic longitude in decimal degrees. Negative = west." },
        { name: "config", type: "ChartConfig?", desc: "Optional. House system, ayanamsha, extra bodies, data classification. Defaults to Vedic/Lahiri/Placidus." },
      ],
      returns: "ChartGraph — the full typed graph with all 10 node types.",
      desc: "Computes a complete natal chart and returns it as a ChartGraph JSON object. Includes planetary longitudes, house cusps, nakshatras, dashas, aspects, dignities, yogas, and patterns. This is the primary entry point for most workflows.",
    },
    {
      name: "compute_dasha",
      params: [
        { name: "chart_id", type: "string", desc: "Deterministic ID returned by compute_natal_chart." },
        { name: "system", type: "DashaSystem?", desc: "Optional. Vimshottari | Yogini | Chara. Default: Vimshottari." },
      ],
      returns: "DashaTree — recursive structure up to 5 levels of sub-periods.",
      desc: "Returns the dasha tree for a previously computed chart. Each DashaPeriod node carries the lord planet, start Julian Day, end Julian Day, and duration in days. The tree goes from Mahadasha down through Antardasha, Pratyantardasha, Sookshma, and Prana.",
    },
    {
      name: "compute_vargas",
      params: [
        { name: "chart_id", type: "string", desc: "Deterministic ID from compute_natal_chart." },
        { name: "divisions", type: "number[]", desc: "Array of divisional chart numbers, e.g. [2, 9, 12]. Valid range 1–60." },
      ],
      returns: "VargaMap — keyed by division number, each value is a full ChartGraph.",
      desc: "Computes one or more of the 16 Shodasha Varga divisional charts. Each varga is an independent ChartGraph with its own planetary positions, house cusps, and dignities. Request multiple divisions in one call — they are computed in parallel.",
    },
    {
      name: "emit_graph",
      params: [
        { name: "chart_id", type: "string", desc: "Deterministic ID from compute_natal_chart." },
        { name: "format", type: "EmitFormat", desc: "Cypher | SurrealQL | JsonLd | Json | EmbeddingText" },
      ],
      returns: "string — the emitted output in the requested format.",
      desc: "Emits the ChartGraph as Cypher (Neo4j MERGE statements), SurrealQL (RELATE syntax), JSON-LD (Schema.org-compatible), plain JSON, or RAG-optimised embedding text. All formats use deterministic node IDs.",
    },
    {
      name: "search_transits",
      params: [
        { name: "planet", type: "Planet", desc: "The transiting body (Sun through Pluto or lunar nodes)." },
        { name: "target", type: "TransitTarget", desc: "Planet | Sign | House | Degree — what the transit is to." },
        { name: "start_jd", type: "f64", desc: "Search window start (Julian Day)." },
        { name: "end_jd", type: "f64", desc: "Search window end (Julian Day)." },
        { name: "config", type: "TransitConfig?", desc: "Optional. Orb, aspect types to include, ingress-only flag." },
      ],
      returns: "TransitEvent[] — exact moment, direction, and description for each hit.",
      desc: "Finds exact transit moments in a Julian Day range. Each TransitEvent carries the exact JD, ingress/egress flag, aspect type, orb at exactitude, and a ready-to-use natural language description. Suited for streaming to AI agents.",
    },
    {
      name: "search_muhurta",
      params: [
        { name: "criteria", type: "MuhurtaCriteria", desc: "Tithi, nakshatra, weekday, hora, and yoga conditions. Any combination." },
        { name: "start_jd", type: "f64", desc: "Search window start." },
        { name: "end_jd", type: "f64", desc: "Search window end." },
        { name: "location", type: "GeoPoint", desc: "Latitude and longitude for local time and house-dependent criteria." },
      ],
      returns: "MuhurtaWindow[] — time windows sorted by combined auspiciousness score.",
      desc: "Finds auspicious time windows matching classical muhurta criteria. Results are scored from 0–100 by combining tithi quality, nakshatra quality, planetary hora strength, and any active auspicious yogas. Windows are sorted highest-score first.",
    },
    {
      name: "describe_chart",
      params: [
        { name: "chart_id", type: "string", desc: "Deterministic ID from compute_natal_chart." },
        { name: "locale", type: "Locale?", desc: "Optional. en | hi | sa | ta | te | kn | bn. Default: en." },
      ],
      returns: "ChartDescription — pre-written paragraphs keyed by topic.",
      desc: "Returns structured natural-language descriptions for the chart's most significant features: active yogas, planetary dignities and weaknesses, dominant dasha period, and notable patterns. Ready for agent relay without further interpretation.",
    },
  ];

  const errorCodes = [
    { code: "DATE_OUT_OF_RANGE", desc: "Julian Day is outside the ephemeris coverage window.", action: "Use a JD within DE440 range (1550–2650 CE) or DE441 (-13000 to +17000)." },
    { code: "BODY_NOT_AVAILABLE", desc: "Requested planet or asteroid not included in the current ephemeris file.", action: "Switch to an ephemeris file that covers the requested body, or omit it from the config." },
    { code: "INVALID_FORMAT", desc: "A parameter value did not match its expected type or enum.", action: "Inspect the JSON schema for the tool and correct the offending field." },
    { code: "CHART_NOT_FOUND", desc: "The chart_id supplied does not match any computed chart in the current session.", action: "Call compute_natal_chart first and use the returned chart_id." },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — MCP Integration
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        7 tools your agent <span className="text-[#D4A843]">already knows.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        The Model Context Protocol (MCP) is a standard transport for exposing
        typed tools to AI agents via JSON-RPC 2.0. Vedākṣha exposes 7 tools
        with full JSON schemas — any MCP-compatible agent can call them without
        custom prompting, output parsing, or glue code.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        Auth: OAuth 2.1. Transport: JSON-RPC 2.0 over HTTP or stdio. Errors are
        structured with machine-readable codes and <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">suggested_action</code> fields
        that let agents self-correct without human intervention.
      </p>

      {/* Starting the server */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Starting the Server
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          The Vedākṣha MCP server supports both stdio transport (for Claude Desktop, local agents)
          and HTTP transport (for remote agents and production deployments).
        </p>
        <div className="space-y-4">
          {[
            { label: "stdio", file: "terminal", cmd: "vedaksha-mcp --transport stdio" },
            { label: "HTTP", file: "terminal", cmd: "vedaksha-mcp --transport http --port 8080" },
            { label: "Config", file: "mcp.json", cmd: `{\n  "mcpServers": {\n    "vedaksha": {\n      "command": "vedaksha-mcp",\n      "args": ["--transport", "stdio"]\n    }\n  }\n}` },
          ].map((item) => (
            <div key={item.label}>
              <div className="flex items-center justify-between px-4 py-2 rounded-t-lg border border-b-0 border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">
                  {item.label}
                </span>
                <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">
                  {item.file}
                </span>
              </div>
              <pre className="rounded-b-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-4 py-3 text-sm font-mono text-[var(--color-brand-text-secondary)] overflow-x-auto">
                <code>{item.cmd}</code>
              </pre>
            </div>
          ))}
        </div>
      </div>

      {/* Tool List */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          The 7 Tools
        </h2>
        <div className="space-y-4">
          {tools.map((tool) => (
            <div
              key={tool.name}
              className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden"
            >
              <div className="px-5 py-4 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                <code className="text-base font-mono font-semibold text-[#D4A843]">
                  {tool.name}
                </code>
                <p className="text-sm text-[var(--color-brand-text-secondary)] mt-1">
                  {tool.desc}
                </p>
              </div>
              <div className="px-5 py-3">
                <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">Parameters</p>
                <div className="space-y-2">
                  {tool.params.map((p) => (
                    <div key={p.name} className="flex items-start gap-3">
                      <code className="text-xs font-mono text-[#D4A843] shrink-0 w-36">{p.name}</code>
                      <code className="text-xs font-mono text-[var(--color-brand-text-muted)] shrink-0 w-28">{p.type}</code>
                      <span className="text-xs text-[var(--color-brand-text-muted)]">{p.desc}</span>
                    </div>
                  ))}
                </div>
                <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mt-3 mb-1">Returns</p>
                <code className="text-xs font-mono text-[var(--color-brand-text-secondary)]">{tool.returns}</code>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* JSON-RPC Call Example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Calling Tools via JSON-RPC
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">request.json</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "tools/call",
  "params": {
    "name": "compute_natal_chart",
    "arguments": {
      "julian_day": 2460389.0,
      "latitude":   28.6139,
      "longitude":  77.2090,
      "config": {
        "house_system": "Placidus",
        "ayanamsha":    "Lahiri",
        "data_class":   "Anonymous"
      }
    }
  }
}`}
            </code>
          </pre>
        </div>
      </div>

      {/* Structured Errors */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Structured Error Responses
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Every error from the Vedākṣha MCP server follows a consistent shape.
          The <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">suggested_action</code> field
          is machine-readable — AI agents can read it and self-correct without escalating to a human.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm mb-6">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">error-response.json</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32001,
    "message": "Julian Day 1000000.0 is outside ephemeris coverage.",
    "data": {
      "error_code":       "DATE_OUT_OF_RANGE",
      "julian_day":       1000000.0,
      "valid_min":        2287184.5,
      "valid_max":        2816787.5,
      "suggested_action": "Use a Julian Day in the range 2287184.5–2816787.5 (1550–2650 CE), or switch to the DE441 ephemeris for extended coverage."
    }
  }
}`}
            </code>
          </pre>
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 gap-px overflow-hidden">
          {errorCodes.map((e) => (
            <div key={e.code} className="bg-[var(--color-brand-bg)] px-5 py-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <div className="flex flex-col sm:flex-row sm:items-start gap-2 mb-1">
                <code className="text-xs font-mono font-semibold text-[#D4A843] shrink-0 w-44">{e.code}</code>
                <p className="text-xs text-[var(--color-brand-text-secondary)]">{e.desc}</p>
              </div>
              <p className="text-xs text-[var(--color-brand-text-muted)] sm:ml-44 sm:pl-2">
                <span className="font-semibold text-[var(--color-brand-text-muted)]">suggested_action: </span>
                {e.action}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* PII-Blind Security Model */}
      <div className="mb-14 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          PII-Blind Security Model
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl mb-3">
          The Vedākṣha MCP server never receives a person&apos;s name, date of birth as
          a calendar date, or any other identifying string. All inputs are in
          astronomical units: Julian Day, decimal latitude, decimal longitude.
          The caller is responsible for the date-to-JD conversion before the
          tool call.
        </p>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
          When the caller sets <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">data_class: &quot;Anonymous&quot;</code>,
          the graph itself contains no PII. The mapping between a person and their
          Julian Day is never sent to the server.
        </p>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration/graph-output"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Graph Output"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/wasm-browser"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"WASM Browser →"}
        </a>
      </div>
    </div>
  );
}
