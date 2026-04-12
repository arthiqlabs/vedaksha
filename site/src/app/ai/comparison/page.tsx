const rows = [
  { capability: "Pure functions / stateless", trad: false, python: false, rest: true, vedaksha: true, note: "Zero initialization, no global state, no cleanup." },
  { capability: "MCP native", trad: false, python: false, rest: false, vedaksha: true, note: "7 typed tools with JSON schemas." },
  { capability: "Graph output", trad: false, python: false, rest: false, vedaksha: true, note: "10 node types, 13 edge types." },
  { capability: "Semantic types", trad: false, python: false, rest: false, vedaksha: true, note: "Body::Jupiter, not integer 5." },
  { capability: "Embedding output (RAG)", trad: false, python: false, rest: false, vedaksha: true, note: "Text optimized for vector stores." },
  { capability: "PII-blind", trad: false, python: false, rest: false, vedaksha: true, note: "Julian Day + coordinates only." },
  { capability: "Vedic first-class", trad: false, python: false, rest: false, vedaksha: true, note: "50 yogas, 3 dashas, 16 vargas, osculating node." },
  { capability: "Neo4j / SurrealDB emit", trad: false, python: false, rest: false, vedaksha: true, note: "Direct Cypher and SurrealQL." },
  { capability: "WASM browser", trad: false, python: false, rest: false, vedaksha: true, note: "Full computation client-side." },
  { capability: "44 ayanamsha", trad: false, python: false, rest: false, vedaksha: true, note: "Every major tradition." },
];

export default function ComparisonPage() {
  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">Comparison</p>
      <h1 className="text-3xl font-bold tracking-tight text-[var(--color-brand-text)] mb-3 uppercase">
        Why Vedākṣha for <span className="text-[#D4A843]">AI agents</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-10 max-w-2xl">
        How Vedaksha compares to traditional approaches — by capability, not by product name.
      </p>
      <div className="overflow-x-auto border border-[var(--color-brand-border)] rounded-xl">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
              <th className="text-left p-4 font-semibold text-[var(--color-brand-text)]">Capability</th>
              <th className="p-4 font-semibold text-[var(--color-brand-text-muted)] text-center">Traditional C</th>
              <th className="p-4 font-semibold text-[var(--color-brand-text-muted)] text-center">Python</th>
              <th className="p-4 font-semibold text-[var(--color-brand-text-muted)] text-center">REST APIs</th>
              <th className="p-4 font-semibold text-[#D4A843] text-center">Vedaksha</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.capability} className="border-b border-[var(--color-brand-border)]">
                <td className="p-4">
                  <span className="font-medium text-[var(--color-brand-text)]">{row.capability}</span>
                  <span className="block text-xs text-[var(--color-brand-text-muted)] mt-0.5">{row.note}</span>
                </td>
                <td className="p-4 text-center">{row.trad ? "\u2713" : "\u2717"}</td>
                <td className="p-4 text-center">{row.python ? "\u2713" : "\u2717"}</td>
                <td className="p-4 text-center">{row.rest ? "\u2713" : "\u2717"}</td>
                <td className="p-4 text-center text-[#D4A843] font-bold">{row.vedaksha ? "\u2713" : "\u2717"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
