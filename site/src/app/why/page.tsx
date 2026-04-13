export default function WhyPage() {
  const problems = [
    {
      num: "01",
      title: "Copyleft Licensing",
      problem: "Most ephemeris engines use AGPL or GPL. If you build a commercial product on top, you must disclose your entire source code. This forces teams to either open-source their app, pay license fees, or build risky workarounds.",
      solution: "Vedaksha uses BSL 1.1 — free for non-commercial use, $500 one-time for commercial. No source disclosure. No copyleft. Converts to Apache 2.0 after five years.",
    },
    {
      num: "02",
      title: "Vedic as an Afterthought",
      problem: "Existing computation engines were designed for Western tropical astrology in the 1990s. Vedic features — nakshatras, dashas, vargas, yogas, Shadbala — are bolted on through wrapper libraries maintained by third parties, with no guarantee of correctness.",
      solution: "Vedaksha is Vedic-first. 5 dasha systems, complete 5-limb panchanga, 27 nakshatras with deity/yoni/nadi, graded drishti, degree-precise Shadbala, 16 vargas with school-variant support, 44 ayanamsha systems. All in the core type system, cited to BPHS.",
    },
    {
      num: "03",
      title: "C-Only, Desktop-Era Architecture",
      problem: "Legacy engines are written in C, designed for desktop GUI applications. Integrating them into modern stacks — mobile apps, web browsers, serverless functions, edge compute — requires FFI bindings, build system complexity, and platform-specific compilation.",
      solution: "Vedaksha is pure Rust. Compiles natively, to WebAssembly (972 KB, zero data files), and to Python via PyO3. Runs in browsers, Cloudflare Workers, AWS Lambda, Docker, and bare metal. One codebase, every target.",
    },
    {
      num: "04",
      title: "Not Built for AI",
      problem: "AI agents need typed, structured, queryable output. Legacy engines return arrays of floating-point numbers. Making them useful for an LLM requires building a translation layer — parsing output, adding context, structuring responses.",
      solution: "Vedaksha produces property graphs with 10 typed node types and 13 edge types. Emit directly to Neo4j Cypher, SurrealDB, JSON-LD, or RAG-optimized text. The MCP server exposes 7 typed tools with JSON schemas — any MCP-compatible agent can call them without custom prompting.",
    },
    {
      num: "05",
      title: "Unverifiable Accuracy",
      problem: "Most engines derive their algorithms from other software rather than from primary published sources. This creates a chain of implementation copies where errors propagate and correctness cannot be independently verified.",
      solution: "Vedaksha is a clean-room implementation. Every algorithm traces to a published NASA, IAU, or academic source — Meeus, Chapront, Bretagnon, BPHS, Phaladipika. Planetary positions are validated against JPL Horizons DE441. The osculating node achieves <0.03\u00b0 accuracy vs JPL. 528 tests with 24,000+ oracle validation data points.",
    },
    {
      num: "06",
      title: "No Privacy Model",
      problem: "Ephemeris computation inherently processes birth data — date, time, location. Most engines have no concept of data classification, making GDPR and privacy compliance an application-level burden.",
      solution: "Vedaksha is PII-blind by design. The engine accepts Julian Day and geographic coordinates — no names, no dates in human-readable form. The graph output carries a DataClassification tag (Anonymous, Pseudonymized, Identified) so downstream systems can enforce retention and access policies.",
    },
  ];

  const numbers = [
    { value: "528+", label: "Automated tests" },
    { value: "24K+", label: "Oracle validation points" },
    { value: "<0.03\u00b0", label: "Node accuracy vs JPL DE441" },
    { value: "44", label: "Ayanamsha systems" },
    { value: "5", label: "Dasha systems" },
    { value: "50", label: "Vedic yogas detected" },
    { value: "7", label: "Languages (localization)" },
    { value: "972 KB", label: "WASM binary (zero data files)" },
  ];

  return (
    <div className="flex flex-col">

      {/* ─── HERO ─── */}
      <section className="px-6 pt-24 pb-20 border-b border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
            Why Vedaksha
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-8">
            The ephemeris engine <span className="text-[#D4A843]">should have existed.</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            We built Vedaksha because the astronomical computation industry has a set of
            structural problems that no amount of wrapper libraries can fix. The engine
            itself needs to be redesigned — for modern platforms, for Vedic traditions,
            for AI agents, and for developers who ship commercial products.
          </p>
        </div>
      </section>

      {/* ─── PROBLEMS & SOLUTIONS ─── */}
      <section className="px-6 py-20">
        <div className="max-w-4xl mx-auto space-y-12">
          {problems.map((item) => (
            <div
              key={item.num}
              className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden"
            >
              <div className="grid grid-cols-1 md:grid-cols-2 gap-px bg-[var(--color-brand-border)]">
                <div className="bg-[var(--color-brand-bg)] p-8">
                  <div className="flex items-center gap-3 mb-4">
                    <span className="text-xs font-mono text-[var(--color-brand-text-muted)]">
                      {item.num}
                    </span>
                    <h2 className="text-base font-semibold uppercase tracking-[0.1em] text-[var(--color-brand-text)]">
                      {item.title}
                    </h2>
                  </div>
                  <p className="text-xs font-semibold uppercase tracking-[0.15em] text-red-500/70 mb-2">
                    The Problem
                  </p>
                  <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                    {item.problem}
                  </p>
                </div>
                <div className="bg-[var(--color-brand-bg-subtle)] p-8">
                  <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[#D4A843] mb-2">
                    Vedaksha
                  </p>
                  <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                    {item.solution}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ─── BY THE NUMBERS ─── */}
      <section className="px-6 py-20 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-10 text-center">
            By the Numbers
          </p>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            {numbers.map((n) => (
              <div key={n.label} className="text-center">
                <p className="text-3xl font-bold text-[var(--color-brand-text)] mb-1">
                  {n.value}
                </p>
                <p className="text-xs text-[var(--color-brand-text-muted)] uppercase tracking-wider">
                  {n.label}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── CTA ─── */}
      <section className="px-6 py-20 border-t border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto text-center">
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-4">
            Ready to try it?
          </h2>
          <p className="text-base text-[var(--color-brand-text-secondary)] mb-8 max-w-lg mx-auto">
            Install from crates.io, pip, or npm. Run the MCP server locally or via Docker.
            Free for non-commercial use.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <a
              href="/docs"
              className="inline-flex items-center rounded-md bg-[var(--color-brand-text)] text-white px-6 py-3 text-sm font-semibold hover:opacity-90 transition-opacity"
            >
              Get Started
            </a>
            <a
              href="/playground"
              className="inline-flex items-center rounded-md border border-[#D4A843] text-[#D4A843] px-6 py-3 text-sm font-semibold hover:bg-[#D4A843]/10 transition-colors"
            >
              Try the Playground
            </a>
          </div>
        </div>
      </section>

    </div>
  );
}
