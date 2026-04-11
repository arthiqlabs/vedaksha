export default function MCPToolsPage() {
  const tools = [
    {
      name: "compute_natal_chart",
      description:
        "Computes a complete natal (birth) chart including planetary positions, house cusps, nakshatras, aspects, dignities, and yogas.",
      params: [
        { name: "julian_day", type: "f64", desc: "Julian Day number for the moment of birth" },
        { name: "latitude", type: "f64", desc: "Geographic latitude in decimal degrees" },
        { name: "longitude", type: "f64", desc: "Geographic longitude in decimal degrees" },
        { name: "ayanamsha", type: "Ayanamsha", desc: "Sidereal zodiac system (e.g., Lahiri, Fagan-Bradley)" },
        { name: "house_system", type: "HouseSystem", desc: "House division method (e.g., Placidus, WholeSign)" },
      ],
      returns:
        "ChartGraph — a property graph with planetary nodes, house nodes, sign nodes, and typed edges representing aspects, placements, and dignities. Includes chart_highlights ranked by significance.",
      useCase:
        "A user says: \"What does my birth chart look like?\" The agent converts the birth datetime to a Julian Day, calls compute_natal_chart, and receives a fully structured chart. It reads chart_highlights to summarize the most significant features, and uses nl_description fields for natural language output.",
    },
    {
      name: "compute_dasha",
      description:
        "Computes a Vedic dasha (planetary period) timeline with up to 5 levels of sub-periods.",
      params: [
        { name: "julian_day", type: "f64", desc: "Julian Day number for the birth moment" },
        { name: "moon_longitude", type: "f64", desc: "Sidereal longitude of the Moon at birth" },
        { name: "dasha_system", type: "DashaSystem", desc: "Vimshottari, Yogini, or Chara" },
        { name: "levels", type: "u8", desc: "Depth of sub-periods (1-5)" },
      ],
      returns:
        "DashaTree — a hierarchical tree of planetary periods with start/end dates, ruling planet, and nl_description for each node.",
      useCase:
        "A user asks: \"What planetary period am I in?\" The agent computes the dasha tree, finds the current date within the hierarchy, and reports the active Mahadasha, Antardasha, and Pratyantardasha with their ruling planets and interpretive descriptions.",
    },
    {
      name: "compute_vargas",
      description:
        "Computes any of the 16 Shodasha Varga divisional charts used in Vedic astrology.",
      params: [
        { name: "julian_day", type: "f64", desc: "Julian Day number" },
        { name: "latitude", type: "f64", desc: "Geographic latitude" },
        { name: "longitude", type: "f64", desc: "Geographic longitude" },
        { name: "varga", type: "Varga", desc: "Divisional chart type (D1 through D60)" },
        { name: "ayanamsha", type: "Ayanamsha", desc: "Sidereal zodiac system" },
      ],
      returns:
        "ChartGraph — the divisional chart as a property graph, with planetary positions recalculated for the selected harmonic division.",
      useCase:
        "A user asks: \"Show me my Navamsha chart.\" The agent calls compute_vargas with Varga::D9 to get the ninth-harmonic divisional chart, which reveals partnership and dharmic patterns.",
    },
    {
      name: "emit_graph",
      description:
        "Converts a ChartGraph into a database-ready or embedding-ready format.",
      params: [
        { name: "chart_graph", type: "ChartGraph", desc: "The chart graph to emit" },
        { name: "format", type: "EmitFormat", desc: "Cypher, SurrealQL, JsonLd, Json, or EmbeddingText" },
      ],
      returns:
        "String — the chart serialized in the requested format. Cypher produces CREATE statements for Neo4j. SurrealQL produces INSERT statements. JSON-LD produces linked data. EmbeddingText produces optimized chunks for vector stores.",
      useCase:
        "An agent building a knowledge graph calls emit_graph with EmitFormat::Cypher after computing a chart, then executes the resulting Cypher statements against a Neo4j database. Later, it queries the graph to find cross-chart patterns.",
    },
    {
      name: "compute_transit",
      description:
        "Computes planetary positions for a given moment — used for current sky, progressions, or transit overlays.",
      params: [
        { name: "julian_day", type: "f64", desc: "Julian Day number for the transit moment" },
        { name: "ayanamsha", type: "Ayanamsha", desc: "Sidereal zodiac system" },
      ],
      returns:
        "TransitSnapshot — planetary positions at the specified moment, with sign, nakshatra, and degree for each body.",
      useCase:
        "A user asks: \"Where are the planets right now?\" The agent converts the current UTC time to a Julian Day and calls compute_transit. It can then overlay these positions on the user\u2019s natal chart to identify active transits.",
    },
    {
      name: "search_transits",
      description:
        "Searches for exact transit events within a date range — conjunctions, sign ingresses, retrograde stations, and aspect formations.",
      params: [
        { name: "start_jd", type: "f64", desc: "Start of the search window (Julian Day)" },
        { name: "end_jd", type: "f64", desc: "End of the search window (Julian Day)" },
        { name: "bodies", type: "Vec<Body>", desc: "Which planets to track" },
        { name: "event_types", type: "Vec<TransitEventType>", desc: "Types of events to find" },
        { name: "ayanamsha", type: "Ayanamsha", desc: "Sidereal zodiac system" },
      ],
      returns:
        "Stream<TransitEvent> — a streaming sequence of transit events, each with exact Julian Day, involved bodies, event type, and nl_description.",
      useCase:
        "A user asks: \"When does Jupiter enter Taurus?\" The agent searches for SignIngress events for Body::Jupiter in the relevant date range. Results stream back as they are found, so the agent can report the first match immediately.",
    },
    {
      name: "search_muhurta",
      description:
        "Finds auspicious time windows based on Vedic electional criteria — nakshatra, tithi, yoga, karana, and custom scoring rules.",
      params: [
        { name: "start_jd", type: "f64", desc: "Start of the search window (Julian Day)" },
        { name: "end_jd", type: "f64", desc: "End of the search window (Julian Day)" },
        { name: "latitude", type: "f64", desc: "Geographic latitude" },
        { name: "longitude", type: "f64", desc: "Geographic longitude" },
        { name: "criteria", type: "MuhurtaCriteria", desc: "Scoring rules for auspiciousness" },
      ],
      returns:
        "Vec<MuhurtaWindow> — ranked time windows with scores, active nakshatra, tithi, yoga, and karana for each window.",
      useCase:
        "A user asks: \"When is a good time to start a business this month?\" The agent calls search_muhurta with criteria favoring nakshatras associated with new ventures, and returns the top-ranked windows with explanations.",
    },
  ];

  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="px-6 pt-28 pb-16">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            MCP Tool Catalog
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            7 tools. One <span className="text-[#D4A843]">protocol</span>.
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            Every Vedākṣha computation is available as an MCP tool with a
            complete JSON schema. Your AI agent discovers the tools, reads their
            schemas, and calls them — no custom integration code required.
          </p>
        </div>
      </section>

      {/* Tool overview */}
      <section className="px-6 pb-8 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto py-8">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            At a Glance
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {tools.map((tool) => (
              <a
                key={tool.name}
                href={`#${tool.name}`}
                className="flex items-center gap-3 border border-[var(--color-brand-border)] rounded-lg px-4 py-2.5 bg-[var(--color-brand-bg)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
              >
                <code className="text-xs font-mono text-[#D4A843] shrink-0">
                  {tool.name}
                </code>
              </a>
            ))}
          </div>
        </div>
      </section>

      {/* Tool details */}
      <section className="px-6 py-16 border-t border-[var(--color-brand-border)]">
        <div className="max-w-4xl mx-auto space-y-16">
          {tools.map((tool) => (
            <div key={tool.name} id={tool.name} className="scroll-mt-20">
              <div className="flex items-start gap-3 mb-4">
                <code className="text-lg font-mono font-bold text-[#D4A843]">
                  {tool.name}
                </code>
              </div>
              <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-6">
                {tool.description}
              </p>

              {/* Parameters */}
              <div className="mb-6">
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-brand-text-muted)] mb-3">
                  Input Parameters
                </h3>
                <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                  <div className="bg-[var(--color-brand-bg-subtle)] px-4 py-2 border-b border-[var(--color-brand-border)] grid grid-cols-[1fr_1fr_2fr] gap-4">
                    <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Parameter</span>
                    <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Type</span>
                    <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Description</span>
                  </div>
                  {tool.params.map((param) => (
                    <div
                      key={param.name}
                      className="px-4 py-2.5 grid grid-cols-[1fr_1fr_2fr] gap-4 border-b border-[var(--color-brand-border)] last:border-b-0"
                    >
                      <code className="text-xs font-mono text-[var(--color-brand-text)]">{param.name}</code>
                      <code className="text-xs font-mono text-[#D4A843]">{param.type}</code>
                      <span className="text-xs text-[var(--color-brand-text-secondary)]">{param.desc}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Returns */}
              <div className="mb-6">
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-brand-text-muted)] mb-3">
                  Returns
                </h3>
                <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-4 py-3">
                  <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                    {tool.returns}
                  </p>
                </div>
              </div>

              {/* Use case */}
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
                  How an AI Agent Uses This
                </h3>
                <div className="rounded-xl border border-[#D4A843]/20 bg-[#D4A843]/5 px-4 py-3">
                  <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                    {tool.useCase}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* CTA */}
      <section className="py-16 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-xl mx-auto text-center">
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
            Ready to <span className="text-[#D4A843]">connect</span>?
          </h2>
          <p className="text-sm text-[var(--color-brand-text-secondary)] mb-6">
            See the quickstart guide to wire up your agent in 5 minutes.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href="/ai/quickstart"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg bg-[var(--color-brand-text)] text-white hover:opacity-90 transition-opacity"
            >
              Quickstart Guide
            </a>
            <a
              href="/ai/patterns"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg border border-[var(--color-brand-border)] text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg)] transition-colors"
            >
              Agent Patterns
            </a>
          </div>
        </div>
      </section>
    </div>
  );
}
