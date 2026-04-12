export default function GraphOutputPage() {
  const nodeTypes = [
    {
      name: "Chart",
      desc: "The root node. Carries the Julian Day, geographic coordinates, ayanamsha value, house system, and the data classification tag. Every other node in the graph hangs off a Chart.",
    },
    {
      name: "Planet",
      desc: "One node per computed body. Carries longitude, latitude, distance, daily speed, retrograde flag, dignity score, and localized name. Includes Sun through Ketu plus any additional bodies in the config.",
    },
    {
      name: "Sign",
      desc: "One of the 12 zodiac signs. Carries element, modality, ruling planet reference, and start/end longitude boundaries. Sign nodes are shared across charts — same sign, same node ID.",
    },
    {
      name: "House",
      desc: "One per cusp in the selected house system (1–12 for most systems, 1–8 for equal-time systems). Carries cusp longitude, natural significations, and the sign it starts in.",
    },
    {
      name: "Nakshatra",
      desc: "One of the 27 lunar mansions. Carries its ruling planet, start longitude, end longitude, deity, and motivational quality (Dharma / Artha / Kama / Moksha). Shared across charts.",
    },
    {
      name: "Pada",
      desc: "One of 108 padas — each nakshatra's four quarters. Carries the pada number (1–4), its Navamsha sign, and its Varga lord. Used for KP sub-lord resolution.",
    },
    {
      name: "Pattern",
      desc: "A multi-body geometric pattern detected in the chart — Grand Trine, T-Square, Yod, Grand Cross, or Kite. Carries the pattern type, participating planet IDs, and an orb score.",
    },
    {
      name: "DashaPeriod",
      desc: "A node in the dasha tree. Carries the dasha lord planet reference, start Julian Day, end Julian Day, duration in days, and level (1 = Mahadasha, 2 = Antardasha, etc.).",
    },
    {
      name: "Yoga",
      desc: "A classical Vedic yoga detected in the chart. Carries the yoga name, source text (e.g. BPHS), participating planets, a strength score, and a brief natural language description.",
    },
    {
      name: "FixedStar",
      desc: "A notable fixed star within configured orb of a planet or house cusp. Carries the star name, Hipparcos catalogue ID, longitude, latitude, magnitude, and traditional signification.",
    },
  ];

  const edgeTypes = [
    { from: "Planet", rel: "PlacedIn", to: "Sign", desc: "Connects a planet to the zodiac sign it occupies at the chart moment." },
    { from: "Planet", rel: "Occupies", to: "House", desc: "Connects a planet to its house placement in the selected house system." },
    { from: "Planet", rel: "InNakshatra", to: "Nakshatra", desc: "Connects a planet to its lunar mansion." },
    { from: "Planet", rel: "Aspects", to: "Planet", desc: "Carries orb (degrees), applying/separating flag, aspect type, and strength score." },
    { from: "Planet", rel: "Rules", to: "Sign", desc: "Encodes planetary rulership. Each sign has a traditional lord." },
    { from: "Planet", rel: "Disposits", to: "Planet", desc: "Sign-lord chain — planet A rules the sign of planet B." },
    { from: "House", rel: "CuspOf", to: "Sign", desc: "Connects a house cusp to the sign where it falls. Enables sign-based house analysis." },
    { from: "Planet", rel: "BelongsTo", to: "Chart", desc: "Connects a node to the chart it belongs to." },
    { from: "Planet", rel: "PartOfPattern", to: "Pattern", desc: "Connects a planet to a multi-body geometric pattern it participates in." },
    { from: "Planet", rel: "ConjunctStar", to: "FixedStar", desc: "Connects a planet to a fixed star within the configured orb." },
    { from: "DashaPeriod", rel: "DashaLord", to: "Planet", desc: "Connects a dasha period to the planet that lords it." },
    { from: "DashaPeriod", rel: "ContainsPeriod", to: "DashaPeriod", desc: "Recursive edge building the full dasha tree (up to 5 levels)." },
    { from: "Chart", rel: "HasYoga", to: "Yoga", desc: "Connects a chart to a detected yoga formation." },
  ];

  const dataClasses = [
    {
      label: "Anonymous",
      tag: "Anon",
      color: "text-emerald-500",
      border: "border-emerald-500/20",
      bg: "bg-emerald-500/10",
      desc: "No birth data in the graph. Only Julian Day and coordinates. Cannot be linked back to a person without external information. Safe for unrestricted storage and sharing.",
    },
    {
      label: "Pseudonymized",
      tag: "Pseudo",
      color: "text-amber-400",
      border: "border-amber-400/20",
      bg: "bg-amber-400/10",
      desc: "Chart linked to an opaque ID (e.g. UUID) rather than a name. The mapping between ID and person is stored separately. Compliant with most data minimisation requirements.",
    },
    {
      label: "Identified",
      tag: "Identified",
      color: "text-rose-400",
      border: "border-rose-400/20",
      bg: "bg-rose-400/10",
      desc: "Chart carries a name or other identifying attribute in a node property. Treat as personal data. Apply appropriate access controls and retention policies.",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — Graph Output
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Every chart is a <span className="text-[#D4A843]">graph.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha does not produce a flat list of planetary positions. Every
        computation also yields a ChartGraph — a typed property graph with 10
        node types and 13 edge types. The same function call that returns
        longitudes also returns a structure you can stream directly into Neo4j,
        SurrealDB, or a vector store.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        Graph construction is zero-cost — it happens during the same traversal
        that computes dignities, aspects, and yogas. There is no separate
        &quot;export&quot; step.
      </p>

      {/* Node Types */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          10 Node Types
        </h2>
        <div className="space-y-3">
          {nodeTypes.map((node) => (
            <div
              key={node.name}
              className="border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              <div className="flex items-start gap-4">
                <code className="text-sm font-mono font-semibold text-[#D4A843] shrink-0 mt-0.5 w-28">
                  {node.name}
                </code>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {node.desc}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Edge Types */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          13 Edge Types
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          {edgeTypes.map((edge, i) => (
            <div
              key={`${edge.from}-${edge.rel}-${edge.to}`}
              className={`px-5 py-4 flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors ${i !== edgeTypes.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""}`}
            >
              <div className="flex items-center gap-2 font-mono text-xs shrink-0">
                <span className="text-[var(--color-brand-text-muted)]">{edge.from}</span>
                <span className="text-[var(--color-brand-text-muted)]">{"→"}</span>
                <span className="text-[#D4A843] font-semibold">{edge.rel}</span>
                <span className="text-[var(--color-brand-text-muted)]">{"→"}</span>
                <span className="text-[var(--color-brand-text-muted)]">{edge.to}</span>
              </div>
              <p className="text-xs text-[var(--color-brand-text-muted)] sm:ml-2">{edge.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Deterministic IDs */}
      <div className="mb-14 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Deterministic IDs
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl mb-3">
          Every node ID is derived deterministically from its content. The same
          Julian Day and coordinates always produce the same graph with the same
          node IDs. Shared nodes — Signs, Nakshatras, Padas — have IDs derived
          from their intrinsic properties and are identical across every chart.
        </p>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
          This makes MERGE-based upserts idempotent: you can re-import the same
          chart any number of times and nothing changes. It also means graph IDs
          are stable foreign keys for relational or document stores.
        </p>
      </div>

      {/* Emitter: Cypher */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Emitting to Neo4j — Cypher
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Call <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">emit_cypher()</code> on any ChartGraph.
          The output is a sequence of MERGE statements — safe to run multiple times.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">import.cypher</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`// Generated by vedaksha::emit::cypher()
// Deterministic IDs — safe to re-run

MERGE (c:Chart {id: "chart_2j5k9x"})
  ON CREATE SET
    c.julian_day   = 2460389.0,
    c.latitude     = 28.6139,
    c.longitude    = 77.2090,
    c.ayanamsha    = "Lahiri",
    c.house_system = "Placidus"

MERGE (p:Planet {id: "planet_sun_2j5k9x"})
  ON CREATE SET
    p.name      = "Sun",
    p.longitude = 336.14,
    p.speed     = 1.01,
    p.retrograde = false

MERGE (s:Sign {id: "sign_pisces"})
  ON CREATE SET s.name = "Pisces", s.element = "Water"

MERGE (p)-[:BelongsTo]->(c)
MERGE (p)-[:PlacedIn]->(s)`}
            </code>
          </pre>
        </div>
        <p className="text-xs text-[var(--color-brand-text-muted)] mt-3">
          Full output includes all 10 node types and 13 relationship types with all properties set.
        </p>
      </div>

      {/* Emitter: SurrealDB */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Emitting to SurrealDB — SurrealQL
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Call <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">emit_surrealql()</code>.
          Uses SurrealDB typed record IDs and RELATE syntax for graph edges.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">import.surql</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`-- Generated by vedaksha::emit::surrealql()

INSERT OR IGNORE INTO chart {
  id: chart:⟨2j5k9x⟩,
  julian_day:   2460389.0,
  latitude:     28.6139,
  longitude:    77.2090,
  ayanamsha:    "Lahiri",
  house_system: "Placidus"
};

INSERT OR IGNORE INTO planet {
  id:        planet:⟨sun_2j5k9x⟩,
  name:      "Sun",
  longitude: 336.14,
  speed:     1.01,
  retrograde: false
};

INSERT OR IGNORE INTO sign {
  id: sign:⟨pisces⟩, name: "Pisces", element: "Water"
};

RELATE planet:⟨sun_2j5k9x⟩->belongs_to->chart:⟨2j5k9x⟩;
RELATE planet:⟨sun_2j5k9x⟩->placed_in->sign:⟨pisces⟩;`}
            </code>
          </pre>
        </div>
      </div>

      {/* Emitter: JSON-LD */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Emitting to JSON-LD
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Call <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">emit_jsonld()</code>.
          Uses a Schema.org-compatible context with custom Vedic terms. Output is
          SPARQL and RDF compatible.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">chart.jsonld</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`{
  "@context": {
    "@vocab": "https://vedaksha.net/ontology/",
    "schema": "https://schema.org/"
  },
  "@type": "Chart",
  "@id": "urn:vedaksha:chart:2j5k9x",
  "julianDay":   2460389.0,
  "latitude":    28.6139,
  "longitude":   77.2090,
  "ayanamsha":   "Lahiri",
  "hasPlanet": [
    {
      "@type": "Planet",
      "@id": "urn:vedaksha:planet:sun:2j5k9x",
      "name":      "Sun",
      "longitude": 336.14,
      "inSign": {
        "@type": "Sign",
        "@id": "urn:vedaksha:sign:pisces",
        "name": "Pisces"
      }
    }
  ]
}`}
            </code>
          </pre>
        </div>
      </div>

      {/* RAG / Embedding */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Embedding Text for RAG Pipelines
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Call <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">emit_embedding_text()</code>.
          Returns one natural-language chunk per node, optimised for retrieval.
          Each chunk carries the node ID as metadata so retrieved context can be
          re-hydrated into the full graph.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {[
            { node: "Chart", text: "Natal chart computed for 20 March 2024, 12:00 UT at 28.61°N 77.21°E. Ayanamsha: Lahiri (23°51′). House system: Placidus." },
            { node: "Planet — Sun", text: "Sun at 336°08′ Pisces, House 12. Speed +1.01°/day. Direct. Dignity: own-sign triplicity. Nakshatra: Uttara Bhadrapada, Pada 4." },
            { node: "Yoga", text: "Hamsa Yoga active. Jupiter in Kendra in own sign Sagittarius. Strength score 84/100. Source: Brihat Parashara Hora Shastra ch. 35." },
            { node: "DashaPeriod", text: "Venus Mahadasha. Active 2021-08-14 to 2041-08-14. Current Antardasha: Venus–Jupiter, ending 2024-08-08." },
          ].map((item) => (
            <div key={item.node} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-xs font-mono text-[#D4A843] block mb-2">{item.node}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">
                &ldquo;{item.text}&rdquo;
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Data Classification */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Data Classification
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 max-w-2xl">
          Every ChartGraph carries a data classification tag in its Chart node.
          The tag is set by the caller at construction time and flows through to
          all emitter outputs as a property on the Chart record.
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {dataClasses.map((dc) => (
            <div key={dc.label} className={`border ${dc.border} rounded-xl p-5`}>
              <div className={`inline-flex items-center px-2 py-0.5 rounded text-[10px] font-mono font-semibold ${dc.bg} ${dc.color} border ${dc.border} mb-3`}>
                {dc.tag}
              </div>
              <h3 className={`text-sm font-semibold uppercase tracking-wide mb-2 ${dc.color}`}>
                {dc.label}
              </h3>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-muted)]">
                {dc.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Full Code Example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Code Example — Compute Chart and Emit Cypher
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">emit_to_neo4j.rs</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
            <code>
              <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::prelude::*;{"\n"}
              <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::emit::Cypher;{"\n"}
              {"\n"}
              <span className="text-purple-600">fn</span> <span className="text-blue-700">main</span>() {"{"}{"\n"}
              {"    "}<span className="text-purple-600">let</span> jd = <span className="text-blue-700">calendar_to_jd</span>(<span className="text-blue-700">2024</span>, <span className="text-blue-700">3</span>, <span className="text-blue-700">20</span>, <span className="text-blue-700">12.0</span>);{"\n"}
              {"\n"}
              {"    "}<span className="text-purple-600">let</span> config = <span className="text-amber-700">ChartConfig</span> {"{"}{"\n"}
              {"        "}house_system: <span className="text-amber-700">HouseSystem</span>::<span className="text-blue-700">Placidus</span>,{"\n"}
              {"        "}ayanamsha: <span className="text-amber-700">Ayanamsha</span>::<span className="text-blue-700">Lahiri</span>,{"\n"}
              {"        "}data_class: <span className="text-amber-700">DataClass</span>::<span className="text-blue-700">Anonymous</span>,{"\n"}
              {"        "}...<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>(){"\n"}
              {"    "}{"}"};{"\n"}
              {"\n"}
              {"    "}<span className="text-green-700">// compute_chart returns Result&lt;ChartGraph, ComputeError&gt;</span>{"\n"}
              {"    "}<span className="text-purple-600">let</span> graph = <span className="text-blue-700">compute_chart</span>(jd, <span className="text-blue-700">28.6139</span>, <span className="text-blue-700">77.2090</span>, &config)?;{"\n"}
              {"\n"}
              {"    "}<span className="text-green-700">// Emit deterministic MERGE statements</span>{"\n"}
              {"    "}<span className="text-purple-600">let</span> cypher = graph.<span className="text-blue-700">emit</span>::&lt;<span className="text-amber-700">Cypher</span>&gt;();{"\n"}
              {"\n"}
              {"    "}<span className="text-green-700">// stream to Neo4j — one statement per line</span>{"\n"}
              {"    "}<span className="text-purple-600">for</span> stmt <span className="text-purple-600">in</span> &cypher.statements {"{"}{"\n"}
              {"        "}<span className="text-blue-700">neo4j_session</span>.<span className="text-blue-700">run</span>(stmt).<span className="text-blue-700">await</span>?;{"\n"}
              {"    "}{"}"}{"\n"}
              {"\n"}
              {"    "}<span className="text-amber-700">Ok</span>(()){"  "}<span className="text-green-700">// ComputeError propagated via ? operator</span>{"\n"}
              {"}"}
            </code>
          </pre>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Integration Guides"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/mcp-integration"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"MCP Integration →"}
        </a>
      </div>
    </div>
  );
}
