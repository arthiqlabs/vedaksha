import { Logo } from "@/components/brand/Logo";
import { InstallBar } from "@/components/ui/InstallBar";

const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "Vedākṣha",
  description:
    "The astronomical ephemeris for the agentic age. Clean-room Rust implementation with sub-arcsecond planetary precision. Vedic astrology in the type system — nakshatras, dashas, yogas, and divisional charts for AI agents.",
  applicationCategory: "DeveloperApplication",
  operatingSystem: "Linux, macOS, Windows, WebAssembly",
  url: "https://vedaksha.net",
  offers: {
    "@type": "Offer",
    price: "500",
    priceCurrency: "USD",
    description: "One-time commercial license",
    seller: {
      "@type": "Organization",
      name: "ArthIQ Labs LLC",
    },
  },
  provider: {
    "@type": "Organization",
    name: "ArthIQ Labs LLC",
    url: "https://vedaksha.net",
  },
};

export default function Home() {
  return (
    <div className="flex flex-col">
      {/* JSON-LD structured data — static object, no user input */}
      {/* eslint-disable-next-line react/no-danger */}
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
      />

      {/* ─── HERO ─── */}
      <section className="relative flex flex-col items-center px-6 pt-28 pb-24 overflow-hidden">
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none opacity-[0.03]">
          <svg viewBox="0 0 800 800" className="size-[700px]" fill="none">
            <circle cx="400" cy="400" r="350" stroke="currentColor" strokeWidth="0.5" />
            <circle cx="400" cy="400" r="250" stroke="currentColor" strokeWidth="0.5" />
            <circle cx="400" cy="400" r="150" stroke="currentColor" strokeWidth="0.5" />
          </svg>
        </div>

        <div className="relative flex flex-col items-center text-center max-w-3xl mx-auto">
          <Logo
            size="full"
            className="size-20 text-[var(--color-brand-primary)] mb-10"
          />

          <h1 className="text-5xl sm:text-6xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            Celestial computation.
            <br />
            <span className="text-[#D4A843]">Agentic precision.</span>
          </h1>

          <p className="text-lg sm:text-xl leading-relaxed text-[var(--color-brand-text-secondary)] max-w-lg mb-10">
            The astronomical engine built from the ground up for AI agents.
            Sub-arcsecond planetary positions. Vedic astrology in the type system.
            Charts as property graphs.
          </p>

          <InstallBar />

          <div className="flex items-center gap-4 mt-6">
            <a
              href="/docs"
              className="inline-flex items-center rounded-lg bg-[var(--color-brand-text)] text-white px-6 py-2.5 text-sm font-semibold hover:opacity-90 transition-opacity"
            >
              Get Started
            </a>
            <a
              href="/playground"
              className="inline-flex items-center rounded-lg border border-[#D4A843]/40 bg-[#D4A843]/5 px-6 py-2.5 text-sm font-semibold text-[#D4A843] hover:bg-[#D4A843]/10 transition-colors"
            >
              Try Playground
            </a>
            <a
              href="/ai"
              className="inline-flex items-center rounded-lg border border-[var(--color-brand-border)] px-6 py-2.5 text-sm font-semibold text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              Why AI-First
            </a>
          </div>
        </div>
      </section>

      {/* ─── THREE PILLARS ─── */}
      <section className="border-t border-[var(--color-brand-border)]">
        <div className="max-w-5xl mx-auto grid grid-cols-1 md:grid-cols-3">

          <div className="p-8 md:p-10 border-b md:border-b-0 md:border-r border-[var(--color-brand-border)]">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
              AI-First
            </p>
            <h2 className="text-2xl font-bold text-[var(--color-brand-text)] md:min-h-[4.5rem] mb-3 leading-tight uppercase tracking-wide">
              Built for agents, not adapted.
            </h2>
            <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-5">
              Pure functions. Zero state. No initialization. Your AI agent calls one function and gets a complete chart back — typed enums, not integer codes.
            </p>
            <div className="space-y-3">
              {[
                "7 MCP tools with JSON schemas",
                "Structured errors with self-correction hints",
                "Streaming transit search",
                "PII-blind — no personal data touches the engine",
              ].map((item) => (
                <div key={item} className="flex items-start gap-2">
                  <span className="mt-1.5 size-1.5 rounded-full bg-[#D4A843] shrink-0" />
                  <span className="text-sm text-[var(--color-brand-text-muted)]">{item}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="p-8 md:p-10 border-b md:border-b-0 md:border-r border-[var(--color-brand-border)]">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
              Vedic-First
            </p>
            <h2 className="text-2xl font-bold text-[var(--color-brand-text)] md:min-h-[4.5rem] mb-3 leading-tight uppercase tracking-wide">
              Jyotish in the type system.
            </h2>
            <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-5">
              27 nakshatras, 50 yogas, 3 dasha systems, and 16 divisional charts — all with 7-language localization in English, Hindi, Sanskrit, Tamil, Telugu, Kannada, and Bengali.
            </p>
            <div className="space-y-3">
              {[
                "Vimshottari, Yogini, and Chara dashas",
                "Complete Shadbala (all 6 components)",
                "44 ayanamsha systems",
                "Muhurta search with tithi scoring",
              ].map((item) => (
                <div key={item} className="flex items-start gap-2">
                  <span className="mt-1.5 size-1.5 rounded-full bg-[#D4A843] shrink-0" />
                  <span className="text-sm text-[var(--color-brand-text-muted)]">{item}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="p-8 md:p-10">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
              Graph-Native
            </p>
            <h2 className="text-2xl font-bold text-[var(--color-brand-text)] md:min-h-[4.5rem] mb-3 leading-tight uppercase tracking-wide">
              Charts are graphs. Finally.
            </h2>
            <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-5">
              Every chart is a property graph with 10 node types and 13 edge types. Emit directly to Neo4j, SurrealDB, or JSON-LD. Feed into RAG pipelines for semantic search.
            </p>
            <div className="space-y-3">
              {[
                "Deterministic IDs — same input, same graph",
                "Cypher, SurrealQL, JSON-LD emitters",
                "Embedding text for vector stores",
                "Data classification for GDPR compliance",
              ].map((item) => (
                <div key={item} className="flex items-start gap-2">
                  <span className="mt-1.5 size-1.5 rounded-full bg-[#D4A843] shrink-0" />
                  <span className="text-sm text-[var(--color-brand-text-muted)]">{item}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ─── CODE EXAMPLE ─── */}
      <section className="py-20 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">

            <div className="lg:pt-4">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
                Five lines of Rust
              </p>
              <h2 className="text-3xl font-bold tracking-tight text-[var(--color-brand-text)] mb-3">
                Your first chart.
              </h2>
              <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-6">
                Pass a date, latitude, and longitude. Get back planetary positions,
                house cusps, nakshatras, aspects, dignities, and yogas — all typed,
                all in one call.
              </p>
              <div className="space-y-4">
                {[
                  { label: "Rust", cmd: "cargo add vedaksha" },
                  { label: "Python", cmd: "pip install vedaksha" },
                  { label: "WASM", cmd: "npm add vedaksha-wasm" },
                ].map((item) => (
                  <div key={item.label} className="flex items-center gap-3">
                    <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] w-12">{item.label}</span>
                    <code className="text-xs font-mono text-[var(--color-brand-text-secondary)] bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-2.5 py-1">
                      {item.cmd}
                    </code>
                  </div>
                ))}
              </div>
            </div>

            <div>
              <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <div className="flex items-center gap-1.5">
                    <span className="size-2.5 rounded-full bg-red-400/50" />
                    <span className="size-2.5 rounded-full bg-yellow-400/50" />
                    <span className="size-2.5 rounded-full bg-green-400/50" />
                  </div>
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">main.rs</span>
                </div>
                <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
                  <code>
                    <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::prelude::*;{"\n"}
                    {"\n"}
                    <span className="text-purple-600">let</span> jd = <span className="text-blue-700">calendar_to_jd</span>(<span className="text-blue-700">2024</span>, <span className="text-blue-700">3</span>, <span className="text-blue-700">20</span>, <span className="text-blue-700">12.0</span>);{"\n"}
                    <span className="text-purple-600">let</span> chart = <span className="text-blue-700">compute_chart</span>({"\n"}
                    {"  "}jd, <span className="text-blue-700">28.6139</span>, <span className="text-blue-700">77.2090</span>,{"\n"}
                    {"  "}&amp;<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>(){"\n"}
                    );{"\n"}
                    {"\n"}
                    <span className="text-purple-600">for</span> planet <span className="text-purple-600">in</span> &amp;chart.planets {"{"}{"\n"}
                    {"  "}<span className="text-blue-700">println!</span>(<span className="text-green-700">{'"'}{'{}'}: {'{}'}{'\u00B0'} {'{}'}{'"'}</span>,{"\n"}
                    {"    "}planet.name, planet.longitude,{"\n"}
                    {"    "}planet.sign);{"\n"}
                    {"}"}
                  </code>
                </pre>
              </div>

              <div className="mt-3 rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <pre className="px-4 py-3 text-sm leading-6 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-muted)]">
                  <code>
                    <span className="text-[#D4A843]">Sun</span>: 359.8° Pisces{"\n"}
                    <span className="text-[#D4A843]">Moon</span>: 127.5° Leo{"\n"}
                    <span className="text-[#D4A843]">Mars</span>: 309.1° Aquarius{"\n"}
                    <span className="text-[#D4A843]">Jupiter</span>: 40.2° Taurus
                  </code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ─── NUMBERS ─── */}
      <section className="border-t border-b border-[var(--color-brand-border)] py-14 px-6">
        <div className="max-w-3xl mx-auto grid grid-cols-2 sm:grid-cols-4 gap-10">
          {[
            { value: "619", label: "automated tests" },
            { value: "3", label: "platforms" },
            { value: "212", label: "public API items" },
            { value: "15+", label: "cited sources" },
          ].map((stat) => (
            <div key={stat.label} className="text-center">
              <span className="block text-4xl sm:text-5xl font-bold tracking-tight text-[#D4A843]">
                {stat.value}
              </span>
              <span className="block text-sm text-[var(--color-brand-text-muted)] mt-1.5 uppercase tracking-wider">
                {stat.label}
              </span>
            </div>
          ))}
        </div>
      </section>

      {/* ─── WHAT'S INSIDE ─── */}
      <section className="py-20 px-6">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] text-center mb-3">
            What&apos;s inside
          </p>
          <h2 className="text-3xl font-bold tracking-tight uppercase text-center text-[var(--color-brand-text)] mb-12">
            Everything you need. <span className="text-[#D4A843]">Nothing you don&apos;t.</span>
          </h2>

          <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 md:grid-cols-3 gap-px overflow-hidden">
            {[
              { num: "01", first: "Planetary", rest: "Engine", desc: "Sub-arcsecond positions from NASA JPL DE440/441. Full IAU precession, nutation, aberration pipeline." },
              { num: "02", first: "Vedic", rest: "Astrology", desc: "27 nakshatras, 3 dasha systems, 50 yogas, 16 vargas, Shadbala, osculating node (<0.03° vs JPL) — not a plugin." },
              { num: "03", first: "House", rest: "Systems", desc: "Placidus, Koch, Whole Sign, Equal, Campanus, Regiomontanus, Porphyry, Morinus, Alcabitius, Sripathi." },
              { num: "04", first: "MCP", rest: "Server", desc: "7 typed tools for AI agents. JSON-RPC 2.0 with structured errors and self-correction hints." },
              { num: "05", first: "Graph", rest: "Emitters", desc: "Output to Neo4j Cypher, SurrealDB, JSON-LD, or RAG-optimized text for vector embedding." },
              { num: "06", first: "44", rest: "Ayanamsha", desc: "Every major sidereal tradition — Lahiri, Fagan-Bradley, Krishnamurti, Aryabhata, and 40 more." },
              { num: "07", first: "Runs", rest: "Everywhere", desc: "Native Rust, WebAssembly for browsers, Python via PyO3. One codebase, three targets." },
              { num: "08", first: "7", rest: "Languages", desc: "Every planet, sign, nakshatra, and yoga name in English, Hindi, Sanskrit, Tamil, Telugu, Kannada, Bengali." },
              { num: "09", first: "Zero", rest: "Legacy", desc: "Built from scratch using NASA and academic sources. No borrowed code. Free for non-commercial use." },
            ].map((f) => (
              <div key={f.num} className="bg-[var(--color-brand-bg)] p-6 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                  {f.num}
                </span>
                <h3 className="text-base font-semibold uppercase tracking-wide mt-1.5 mb-1.5">
                  <span className="text-[#D4A843]">{f.first} </span>
                  <span className="text-[var(--color-brand-text)]">{f.rest}</span>
                </h3>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {f.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── MCP TOOLS ─── */}
      <section className="py-20 px-6 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-4xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-5 gap-10">
            <div className="lg:col-span-2">
              <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
                For AI builders
              </p>
              <h2 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
                Your agent calls. <span className="text-[#D4A843]">Gets a chart.</span>
              </h2>
              <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)]">
                Connect any MCP-compatible AI agent — Claude, GPT, or custom.
                JSON schemas mean your agent already knows the input format.
                Structured errors mean it can self-correct.
              </p>
            </div>
            <div className="lg:col-span-3 space-y-2">
              {[
                { tool: "compute_natal_chart", desc: "Full chart → ChartGraph JSON" },
                { tool: "compute_dasha", desc: "Dasha tree with 5 levels of sub-periods" },
                { tool: "compute_vargas", desc: "Any of 16 divisional charts" },
                { tool: "emit_graph", desc: "ChartGraph → Cypher / SurrealQL / JSON-LD" },
                { tool: "search_transits", desc: "Find exact transit moments in a date range" },
                { tool: "search_muhurta", desc: "Find auspicious times by nakshatra and tithi" },
              ].map((item) => (
                <div key={item.tool} className="flex items-center gap-3 bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded-lg px-4 py-2.5">
                  <code className="text-xs font-mono text-[#D4A843] shrink-0">
                    {item.tool}
                  </code>
                  <span className="text-xs text-[var(--color-brand-text-muted)]">— {item.desc}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ─── CTA ─── */}
      <section className="py-24 px-6 border-t border-[var(--color-brand-border)]">
        <div className="max-w-xl mx-auto text-center">
          <Logo size="medium" className="size-10 text-[var(--color-brand-primary)] mx-auto mb-6" />
          <h2 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
            Start <span className="text-[#D4A843]">computing</span>.
          </h2>
          <p className="text-base text-[var(--color-brand-text-secondary)] mb-8">
            Free for non-commercial use. $500 one-time for commercial.
          </p>
          <div className="flex justify-center gap-4 mb-10">
            <a
              href="/docs"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg bg-[var(--color-brand-text)] text-white hover:opacity-90 transition-opacity"
            >
              Get Started
            </a>
            <a
              href="/pricing"
              className="inline-flex items-center px-7 py-3 text-sm font-semibold rounded-lg border border-[var(--color-brand-border)] text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              Pricing
            </a>
          </div>

          <p className="text-xs text-[var(--color-brand-text-muted)] mb-4">
            All implementations must display attribution
          </p>
          <div className="flex justify-center items-center gap-3">
            <a
              href="https://vedaksha.net"
              className="inline-flex items-center gap-2 px-3 py-1.5 border border-[var(--color-brand-border)] rounded-md no-underline"
            >
              <Logo size="favicon" className="size-3.5 text-[var(--color-brand-primary)]" />
              <span className="text-[11px] font-medium text-[var(--color-brand-text-muted)]">
                Powered by Vedākṣha
              </span>
            </a>
            <a
              href="https://vedaksha.net"
              className="inline-flex items-center gap-2 px-3 py-1.5 border border-white/15 rounded-md bg-[#0B1120] no-underline"
            >
              <Logo size="favicon" variant="dark" className="size-3.5" />
              <span className="text-[11px] font-medium text-white/70">
                Powered by Vedākṣha
              </span>
            </a>
          </div>
        </div>
      </section>
    </div>
  );
}
