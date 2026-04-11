export default function PatternsPage() {
  const patterns = [
    {
      num: "01",
      title: "Birth Chart Summary Agent",
      intent:
        "A user provides their birth date, time, and place. The agent returns a readable summary of their natal chart.",
      workflow: [
        "Convert birth datetime and location to Julian Day and coordinates.",
        "Call compute_natal_chart with the preferred ayanamsha and house system.",
        "Read chart_highlights to identify the most significant features.",
        "Use nl_description fields to compose a natural language summary.",
        "Optionally call emit_graph with EmbeddingText to store facts in a vector database.",
      ],
      tools: ["compute_natal_chart", "emit_graph"],
    },
    {
      num: "02",
      title: "Transit Alert Agent",
      intent:
        "A user wants to be notified when significant planetary transits are approaching — sign ingresses, retrogrades, or conjunctions with natal planets.",
      workflow: [
        "Call search_transits with a 30-day window and the user's preferred event types.",
        "Stream results as they arrive — no need to wait for the full search to complete.",
        "Compare transit positions against the user's stored natal chart (from a prior compute_natal_chart call).",
        "For each significant transit, use the nl_description field to compose an alert message.",
        "Schedule re-runs daily to detect new events entering the window.",
      ],
      tools: ["search_transits", "compute_transit", "compute_natal_chart"],
    },
    {
      num: "03",
      title: "Compatibility Agent (Synastry)",
      intent:
        "Two users want to understand the astrological compatibility between their charts.",
      workflow: [
        "Compute natal charts for both individuals using compute_natal_chart.",
        "Emit both charts to a graph database using emit_graph with Cypher format.",
        "Query the graph for inter-chart aspects — Planet A in Chart 1 aspecting Planet B in Chart 2.",
        "Identify key synastry patterns: conjunctions to angles, mutual receptions, shared sign placements.",
        "Summarize findings using nl_description fields and chart_highlights from both charts.",
      ],
      tools: ["compute_natal_chart", "emit_graph"],
    },
    {
      num: "04",
      title: "Muhurta Advisor Agent",
      intent:
        "A user wants to find an auspicious time for an important event — a wedding, business launch, or journey.",
      workflow: [
        "Determine the user's location and the date range they are considering.",
        "Call search_muhurta with criteria appropriate to the event type.",
        "Rank the returned windows by score and present the top options.",
        "For each recommended window, explain the active nakshatra, tithi, and yoga.",
        "Optionally overlay the muhurta against the user's natal chart for personalized scoring.",
      ],
      tools: ["search_muhurta", "compute_natal_chart"],
    },
    {
      num: "05",
      title: "Vedic Dasha Timeline Agent",
      intent:
        "A user wants to understand their current and upcoming planetary periods (dashas) and what they signify.",
      workflow: [
        "Call compute_natal_chart to get the Moon's sidereal longitude.",
        "Call compute_dasha with the Moon longitude and the preferred dasha system (Vimshottari, Yogini, or Chara).",
        "Navigate the DashaTree to find the current active period at each level.",
        "Use nl_description fields to explain the significance of each active period.",
        "Project upcoming period transitions and highlight when the next Mahadasha change occurs.",
      ],
      tools: ["compute_natal_chart", "compute_dasha"],
    },
    {
      num: "06",
      title: "Knowledge Graph Builder",
      intent:
        "An application wants to build a persistent, queryable knowledge graph of astrological charts for research or multi-user analysis.",
      workflow: [
        "For each chart request, call compute_natal_chart.",
        "Call emit_graph with Cypher or SurrealQL to generate database statements.",
        "Execute the statements against Neo4j or SurrealDB.",
        "Deterministic IDs ensure that repeated computations for the same input merge cleanly without duplicates.",
        "Query the graph across charts: \"Which charts have Jupiter in the 10th house aspecting the Ascendant lord?\"",
      ],
      tools: ["compute_natal_chart", "compute_vargas", "emit_graph"],
    },
    {
      num: "07",
      title: "RAG-Powered Astrology Chat",
      intent:
        "A chat application answers astrological questions by retrieving relevant facts from a vector store of chart data.",
      workflow: [
        "When a chart is computed, call emit_graph with EmbeddingText format.",
        "Split the output into chunks and embed each chunk using a text embedding model.",
        "Store the embeddings in a vector database with the chart ID as metadata.",
        "When a user asks a question, embed the question and retrieve the most relevant chart facts.",
        "Pass the retrieved facts as context to the language model for grounded, accurate answers.",
      ],
      tools: ["compute_natal_chart", "emit_graph"],
    },
    {
      num: "08",
      title: "Multi-Chart Research Agent",
      intent:
        "A researcher wants to analyze patterns across hundreds of charts — for example, finding correlations between specific planetary configurations and life events.",
      workflow: [
        "Batch-compute charts using compute_natal_chart for each subject in the dataset.",
        "Emit all charts to a graph database using emit_graph.",
        "Write graph queries to find statistical patterns across the corpus.",
        "Use compute_vargas to add divisional chart data for deeper analysis.",
        "Export findings as structured data for further statistical analysis.",
      ],
      tools: [
        "compute_natal_chart",
        "compute_vargas",
        "compute_dasha",
        "emit_graph",
      ],
    },
  ];

  return (
    <div className="flex flex-col">
      {/* Hero */}
      <section className="px-6 pt-28 pb-16">
        <div className="max-w-4xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Agent Patterns
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            8 real-world <span className="text-[#D4A843]">workflows</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            These patterns show how AI agents use Vedākṣha in production. Each
            pattern documents the user intent, the agent workflow, and the
            Vedaksha tools involved.
          </p>
        </div>
      </section>

      {/* Patterns */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto space-y-12">
          {patterns.map((pattern) => (
            <div
              key={pattern.num}
              className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden"
            >
              {/* Header */}
              <div className="bg-[var(--color-brand-bg-subtle)] px-6 py-4 border-b border-[var(--color-brand-border)]">
                <div className="flex items-center gap-3">
                  <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                    Pattern {pattern.num}
                  </span>
                </div>
                <h2 className="text-lg font-bold uppercase tracking-wide text-[var(--color-brand-text)] mt-1">
                  {pattern.title}
                </h2>
              </div>

              <div className="p-6 space-y-5">
                {/* Intent */}
                <div>
                  <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-2">
                    User Intent
                  </h3>
                  <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                    {pattern.intent}
                  </p>
                </div>

                {/* Workflow */}
                <div>
                  <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-brand-text-muted)] mb-2">
                    Agent Workflow
                  </h3>
                  <ol className="space-y-2">
                    {pattern.workflow.map((step, i) => (
                      <li key={i} className="flex items-start gap-3">
                        <span className="mt-0.5 text-xs font-mono font-bold text-[#D4A843] shrink-0 w-5">
                          {i + 1}.
                        </span>
                        <span className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                          {step}
                        </span>
                      </li>
                    ))}
                  </ol>
                </div>

                {/* Tools */}
                <div>
                  <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-brand-text-muted)] mb-2">
                    Vedākṣha Tools Used
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    {pattern.tools.map((tool) => (
                      <a
                        key={tool}
                        href={`/ai/mcp-tools#${tool}`}
                        className="inline-flex items-center rounded-md border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-2.5 py-1 text-xs font-mono text-[#D4A843] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
                      >
                        {tool}
                      </a>
                    ))}
                  </div>
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
            Build your <span className="text-[#D4A843]">pattern</span>.
          </h2>
          <p className="text-sm text-[var(--color-brand-text-secondary)] mb-6">
            Start with the quickstart guide and have your agent running in
            minutes.
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
