export default function OntologyPage() {
  const nodeTypes = [
    { name: "Chart", desc: "Root node. Carries Julian Day, coordinates, ayanamsha, house system, and data classification." },
    { name: "Planet", desc: "Celestial body. Longitude, latitude, distance, speed, retrograde flag, sign index, house number." },
    { name: "Sign", desc: "One of 12 zodiac signs. Element, modality, ruling planet, index." },
    { name: "House", desc: "One of 12 bhavas. Cusp longitude, house system, number." },
    { name: "Nakshatra", desc: "One of 27 lunar mansions. Lord, deity, index." },
    { name: "Pada", desc: "One of 108 nakshatra quarters. Pada number, start longitude, nakshatra index." },
    { name: "Pattern", desc: "Multi-body geometric pattern. Grand Trine, T-Square, Yod, Grand Cross, Stellium." },
    { name: "DashaPeriod", desc: "Dasha tree node. Lord, level, start/end Julian Day, duration in days." },
    { name: "Yoga", desc: "Classical Vedic yoga. Name, type, description, participating planets." },
    { name: "FixedStar", desc: "Notable fixed star. Name, longitude, latitude, magnitude." },
  ];

  const edgeTypes = [
    { rel: "PlacedIn", from: "Planet", to: "Sign", desc: "Planet occupies a zodiac sign." },
    { rel: "Occupies", from: "Planet", to: "House", desc: "Planet occupies a bhava." },
    { rel: "Aspects", from: "Planet", to: "Planet", desc: "Geometric aspect with orb, type, applying/separating flag, strength." },
    { rel: "Rules", from: "Planet", to: "Sign", desc: "Natural rulership (e.g. Mars rules Aries and Scorpio)." },
    { rel: "Disposits", from: "Planet", to: "Planet", desc: "Sign-lord chain. Planet A rules the sign of planet B." },
    { rel: "CuspOf", from: "House", to: "Sign", desc: "House cusp falls in a sign." },
    { rel: "BelongsTo", from: "Planet", to: "Chart", desc: "Node belongs to a chart." },
    { rel: "PartOfPattern", from: "Planet", to: "Pattern", desc: "Planet participates in a geometric pattern." },
    { rel: "InNakshatra", from: "Planet", to: "Nakshatra", desc: "Planet falls in a lunar mansion." },
    { rel: "ConjunctStar", from: "Planet", to: "FixedStar", desc: "Planet is conjunct a fixed star within orb." },
    { rel: "DashaLord", from: "DashaPeriod", to: "Planet", desc: "Planet lords a dasha period." },
    { rel: "ContainsPeriod", from: "DashaPeriod", to: "DashaPeriod", desc: "Parent period contains a child sub-period." },
    { rel: "HasYoga", from: "Chart", to: "Yoga", desc: "Chart contains a detected yoga formation." },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Ontology
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Graph <span className="text-[#D4A843]">Ontology</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        The Vedaksha ontology defines 10 node types and 13 edge types for
        astrological chart graphs. Every chart computation produces a typed
        property graph using this schema, emittable as Cypher, SurrealQL,
        JSON-LD, JSON, or embedding text.
      </p>

      {/* Namespace */}
      <div className="mb-12 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Namespace
        </h2>
        <code className="text-sm font-mono text-[#D4A843]">
          https://vedaksha.net/ontology/
        </code>
        <p className="text-xs text-[var(--color-brand-text-muted)] mt-2">
          All types and relationships are prefixed with this namespace in JSON-LD output.
        </p>
      </div>

      {/* Node types */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          10 Node Types
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {nodeTypes.map((item) => (
            <div key={item.name} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-xs font-mono text-[#D4A843] block mb-1">
                {item.name}
              </code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">
                {item.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Edge types */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          13 Edge Types
        </h2>
        <div className="space-y-2">
          {edgeTypes.map((item) => (
            <div key={item.rel} className="flex items-start gap-4 border border-[var(--color-brand-border)] rounded-lg px-4 py-3 bg-[var(--color-brand-bg-subtle)]">
              <code className="text-xs font-mono text-[#D4A843] shrink-0 pt-0.5 w-40">
                {item.rel}
              </code>
              <div className="flex items-start gap-3 flex-1 min-w-0">
                <span className="text-xs text-[var(--color-brand-text-muted)] shrink-0">
                  ({item.from} {"→"} {item.to})
                </span>
                <span className="text-xs text-[var(--color-brand-text-secondary)]">{item.desc}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* JSON-LD example */}
      <div>
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          JSON-LD Context
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <span className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider">JSON-LD</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-6 font-mono bg-[var(--color-brand-bg-code)]">
            <code>{`{
  "@context": {
    "vedaksha": "https://vedaksha.net/ontology/",
    "Chart": "vedaksha:Chart",
    "Planet": "vedaksha:Planet",
    "Sign": "vedaksha:Sign",
    "House": "vedaksha:House",
    "Nakshatra": "vedaksha:Nakshatra",
    "Pada": "vedaksha:Pada",
    "Pattern": "vedaksha:Pattern",
    "DashaPeriod": "vedaksha:DashaPeriod",
    "Yoga": "vedaksha:Yoga",
    "FixedStar": "vedaksha:FixedStar"
  },
  "@graph": [...]
}`}</code>
          </pre>
        </div>
      </div>
    </div>
  );
}
