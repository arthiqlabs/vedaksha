import Link from "next/link";

export default function ChartsAsGraphsPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HEADER ─── */}
      <section className="px-6 pt-24 pb-14 border-b border-[var(--color-brand-border)]">
        <div className="max-w-2xl mx-auto">
          <Link
            href="/blog"
            className="inline-flex items-center gap-1.5 text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider mb-8 hover:text-[#D4A843] transition-colors no-underline"
          >
            ← Blog
          </Link>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Graph · AI · Neo4j
          </p>
          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight leading-[1.15] uppercase text-[var(--color-brand-text)] mb-6">
            Why Charts Should Be Graphs
          </h1>
          <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-6">
            Astrological charts are property graphs. They always were — we just represented them as flat structs. The case for 10 node types, 13 edge types, and emitters that make AI agents actually useful.
          </p>
          <div className="flex items-center gap-3 text-xs text-[var(--color-brand-text-muted)]">
            <span>December 3, 2025</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>9 min read</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>ArthIQ Labs</span>
          </div>
        </div>
      </section>

      {/* ─── BODY ─── */}
      <article className="px-6 py-16">
        <div className="max-w-2xl mx-auto">
          <div className="space-y-8 text-[var(--color-brand-text-secondary)] leading-relaxed">

            <p className="text-base">
              The standard representation of an astrological chart is a list of planets with floating-point longitudes. Ask most libraries for a chart and you get back something like a dictionary of planet names to degree values. This is not wrong — it is just profoundly incomplete.
            </p>

            <p className="text-base">
              A chart is not a list. It is a network of relationships: planets in signs, signs ruling houses, planets aspecting each other, lords of houses placed in other houses. The relationships are the point. They are what a practitioner reads, what a dasha system navigates, what a yoga rule tests. Flattening this network into a struct loses most of the semantically interesting structure before the application ever sees it.
            </p>

            {/* Section: The graph model */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                The graph model
              </h2>
              <p className="text-base">
                A <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">ChartGraph</code> has 10 node types and 13 edge types. The nodes are:
              </p>

              <div className="mt-5 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
                {[
                  { node: "Chart", desc: "Root node — metadata, ayanamsha, house system, epoch." },
                  { node: "Planet", desc: "Any graha: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu, plus Ascendant." },
                  { node: "Sign", desc: "One of 12 rasis. Carries modality, element, direction, ruler." },
                  { node: "House", desc: "One of 12 bhavas. Carries cusp longitude, natural karakatwa." },
                  { node: "Nakshatra", desc: "One of 27 lunar mansions. Carries lord, deity, shakti." },
                  { node: "Pada", desc: "One of 108 nakshatra quarters. Carries pada number, Navamsha sign, start longitude." },
                  { node: "Pattern", desc: "A multi-body geometric pattern — Grand Trine, T-Square, Yod, Grand Cross, Stellium." },
                  { node: "DashaPeriod", desc: "A dasha period — mahadasha, antardasha, or deeper sub-period with start/end dates." },
                  { node: "Yoga", desc: "A detected yoga rule with its component planets and activation status." },
                  { node: "FixedStar", desc: "A notable fixed star within orb of a planet or cusp. Carries magnitude and signification." },
                ].map((item) => (
                  <div key={item.node} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                    <code className="text-xs font-mono text-[#D4A843] block mb-1">
                      :{item.node}
                    </code>
                    <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">
                      {item.desc}
                    </p>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: Edge types */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                The 13 edge types
              </h2>
              <p className="text-base">
                Edges carry the relationships that practitioners actually reason about. Most of the interesting chart interpretation is a path traversal over these edges.
              </p>

              <div className="mt-5 space-y-2">
                {[
                  { rel: "PlacedIn", from: "Planet", to: "Sign", desc: "Planet occupies a rasi." },
                  { rel: "Occupies", from: "Planet", to: "House", desc: "Planet occupies a bhava." },
                  { rel: "InNakshatra", from: "Planet", to: "Nakshatra", desc: "Planet falls in a lunar mansion." },
                  { rel: "Rules", from: "Planet", to: "Sign", desc: "Natural rulership (e.g. Mars rules Aries and Scorpio)." },
                  { rel: "Disposits", from: "Planet", to: "Planet", desc: "Sign-lord chain — planet A rules the sign of planet B." },
                  { rel: "Aspects", from: "Planet", to: "Planet", desc: "Computed aspect with orb, type, and applying/separating." },
                  { rel: "CuspOf", from: "House", to: "Sign", desc: "House cusp falls in a sign." },
                  { rel: "BelongsTo", from: "Planet", to: "Chart", desc: "Node belongs to a chart." },
                  { rel: "PartOfPattern", from: "Planet", to: "Pattern", desc: "Planet participates in a geometric pattern." },
                  { rel: "ConjunctStar", from: "Planet", to: "FixedStar", desc: "Planet is conjunct a notable fixed star within orb." },
                  { rel: "DashaLord", from: "DashaPeriod", to: "Planet", desc: "Planet is the lord of this dasha period." },
                  { rel: "ContainsPeriod", from: "DashaPeriod", to: "DashaPeriod", desc: "Parent period contains a child sub-period." },
                  { rel: "HasYoga", from: "Chart", to: "Yoga", desc: "Chart contains a detected yoga formation." },
                ].map((item) => (
                  <div key={item.rel} className="flex items-start gap-4 border border-[var(--color-brand-border)] rounded-lg px-4 py-3 bg-[var(--color-brand-bg-subtle)]">
                    <code className="text-xs font-mono text-[#D4A843] shrink-0 pt-0.5 w-44">
                      {item.rel}
                    </code>
                    <div className="flex items-start gap-3 flex-1 min-w-0">
                      <span className="text-xs text-[var(--color-brand-text-muted)] shrink-0">
                        ({item.from} → {item.to})
                      </span>
                      <span className="text-xs text-[var(--color-brand-text-secondary)]">{item.desc}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: Emitters */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Four emitters
              </h2>
              <p className="text-base">
                A <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">ChartGraph</code> is an in-memory structure. Getting it into a useful target is the job of the emitters.
              </p>

              <div className="mt-5 rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <div className="flex items-center gap-1.5">
                    <span className="size-2.5 rounded-full bg-red-400/50" />
                    <span className="size-2.5 rounded-full bg-yellow-400/50" />
                    <span className="size-2.5 rounded-full bg-green-400/50" />
                  </div>
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">emit.rs</span>
                </div>
                <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
                  <code>
                    <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::graph::*;{"\n"}
                    {"\n"}
                    <span className="text-purple-600">let</span> graph = <span className="text-blue-700">compute_chart_graph</span>(jd, lat, lng, &amp;<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>());{"\n"}
                    {"\n"}
                    <span className="text-green-700">// Neo4j / any Cypher-compatible graph DB</span>{"\n"}
                    <span className="text-purple-600">let</span> cypher = graph.<span className="text-blue-700">emit_cypher</span>();{"\n"}
                    {"\n"}
                    <span className="text-green-700">// SurrealDB</span>{"\n"}
                    <span className="text-purple-600">let</span> surreal = graph.<span className="text-blue-700">emit_surrealql</span>();{"\n"}
                    {"\n"}
                    <span className="text-green-700">// JSON-LD for semantic web / RAG pipelines</span>{"\n"}
                    <span className="text-purple-600">let</span> jsonld = graph.<span className="text-blue-700">emit_jsonld</span>();{"\n"}
                    {"\n"}
                    <span className="text-green-700">// Pre-chunked text for vector embedding</span>{"\n"}
                    <span className="text-purple-600">let</span> chunks = graph.<span className="text-blue-700">emit_embedding_text</span>();
                  </code>
                </pre>
              </div>
            </div>

            {/* Section: Why it matters for AI */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Why this matters for AI agents
              </h2>
              <p className="text-base">
                When an AI agent receives a flat list of planet-degree pairs, it has to reconstruct the chart&apos;s relational structure from scratch. The agent effectively re-implements a simplified version of the chart interpretation layer to answer a question like &quot;which planets aspect the 7th house lord?&quot; This is fragile, slow, and inaccurate.
              </p>
              <p className="text-base mt-4">
                With a property graph emitter, the agent can run a Cypher query:
              </p>

              <div className="mt-4 rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <span className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider">Cypher</span>
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">neo4j</span>
                </div>
                <pre className="p-4 overflow-x-auto text-sm leading-6 font-mono bg-[var(--color-brand-bg-code)]">
                  <code>
                    <span className="text-purple-600">MATCH</span> (p:Planet)-[:Occupies]-&gt;(h:House {"{"}<span className="text-blue-700">number</span>: <span className="text-blue-700">7</span>{"}"}{"}"}){"\n"}
                    <span className="text-purple-600">MATCH</span> (aspector:Planet)-[:ASPECTS]-&gt;(p){"\n"}
                    <span className="text-purple-600">RETURN</span> aspector.name, aspector.sign, p.name{"\n"}
                  </code>
                </pre>
              </div>

              <p className="text-base mt-4">
                The graph model also enables multi-chart queries. Synastry (comparing two charts) is a join between two subgraphs. A transit lookup is an intersection between a natal graph and a transiting planet graph. These queries are awkward with flat arrays; they are natural with a graph schema.
              </p>
            </div>

            {/* Section: Deterministic IDs */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Deterministic node IDs
              </h2>
              <p className="text-base">
                Every node in the graph has a deterministic ID derived from its content — not a random UUID, not a database sequence. A <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">Planet</code> node ID is a hash of the chart ID plus the planet name. The <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">Chart</code> node ID is derived from the Julian Day, coordinates, and configuration.
              </p>
              <p className="text-base mt-4">
                This means two identical chart computations produce identical node IDs, which means upserts are idempotent. Load the same chart twice into Neo4j and you get the same graph. This also allows safe merging of multiple charts into a single database without ID collisions.
              </p>
            </div>

            {/* Section: Embedding text */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Embedding text for RAG pipelines
              </h2>
              <p className="text-base">
                The embedding text emitter generates pre-chunked natural language descriptions of each node and its immediate neighborhood. Each chunk is designed to be the right size for a vector embedding — not the full chart as a wall of text, but semantically coherent fragments like:
              </p>

              <div className="mt-4 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)] p-5">
                <p className="text-sm font-mono text-[var(--color-brand-text-secondary)] leading-relaxed">
                  &quot;Mars is placed in Scorpio in the 8th house at 14°22&apos;. It is the lord of the 1st and 8th houses. Mars aspects the 2nd house (4th drishti), the 3rd house (7th drishti), and the Ascendant (8th drishti). Mars is in its own sign, giving it strength. It activates the Ruchaka Mahapurusha Yoga.&quot;
                </p>
              </div>

              <p className="text-base mt-4">
                Every chunk includes the relevant node IDs so the vector store result can be linked back to the graph for follow-up queries. This enables a hybrid retrieval pattern: semantic search finds the relevant chart region, graph traversal answers the precise relational question.
              </p>
            </div>

            {/* Closing */}
            <p className="text-base">
              The flat struct representation of a chart is a compression artifact — it drops the relationships to fit into a simpler data model. A property graph is not an elaborate overengineering; it is the natural shape of the data. Everything else — the emitters, the deterministic IDs, the embedding chunks — follows from that starting point.
            </p>

          </div>

          {/* ─── FOOTER NAV ─── */}
          <div className="mt-16 pt-10 border-t border-[var(--color-brand-border)] flex items-center justify-between">
            <Link
              href="/blog/clean-room-ephemeris"
              className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider hover:text-[#D4A843] transition-colors no-underline"
            >
              ← Clean-Room Ephemeris
            </Link>
            <Link
              href="/blog/vedic-astrology"
              className="text-xs font-semibold text-[#D4A843] uppercase tracking-wider hover:underline no-underline"
            >
              Next: Vedic Astrology Deserves Better →
            </Link>
          </div>
        </div>
      </article>

    </div>
  );
}
