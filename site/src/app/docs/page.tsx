import Link from "next/link";
import { Logo } from "@/components/brand/Logo";

const sections = [
  {
    id: "start",
    label: "Start Here",
    items: [
      { title: "Getting Started", desc: "Install Vedākṣha and compute your first chart in 5 minutes.", href: "/docs/getting-started" },
      { title: "FAQ", desc: "Accuracy, licensing, Vedic support, platforms, and common questions.", href: "/docs/integration/faq" },
      { title: "Data Sources", desc: "NASA JPL, IAU standards, BPHS, Meeus — all public, all cited.", href: "/docs/integration/data-sources" },
    ],
  },
  {
    id: "compute",
    label: "Core Computation",
    items: [
      { title: "Planetary Positions", desc: "Ecliptic longitude, latitude, distance, and daily speed for any body.", href: "/docs/integration/planetary-positions" },
      { title: "House Systems", desc: "10 systems from Placidus to Sripathi with polar fallback.", href: "/docs/integration/house-systems" },
      { title: "Coordinate Systems", desc: "ICRS → precession → nutation → aberration → ecliptic.", href: "/docs/integration/coordinate-systems" },
      { title: "Time Systems", desc: "Julian Day, UT/TT/TDB, Delta T, sidereal time.", href: "/docs/integration/time-systems" },
    ],
  },
  {
    id: "vedic",
    label: "Vedic Astrology",
    items: [
      { title: "Vedic Computation", desc: "Nakshatras, dashas, vargas, yogas, Shadbala, drishti — the complete engine.", href: "/docs/integration/vedic-astrology" },
      { title: "Sidereal Zodiac", desc: "44 ayanamsha systems. Every major tradition represented.", href: "/docs/integration/sidereal-zodiac" },
    ],
  },
  {
    id: "events",
    label: "Aspects & Events",
    items: [
      { title: "Aspects & Patterns", desc: "11 aspect types, Grand Trine, T-Square, Yod, Stellium.", href: "/docs/integration/aspects-patterns" },
      { title: "Transit Search", desc: "Exact transit moments, solar/lunar returns, synastry, muhurta.", href: "/docs/integration/transit-search" },
    ],
  },
  {
    id: "output",
    label: "Output & Errors",
    items: [
      { title: "Graph Output", desc: "10 node types, 13 edge types. Cypher, SurrealQL, JSON-LD, RAG text.", href: "/docs/integration/graph-output" },
      { title: "Error Handling", desc: "Structured errors, polar fallback, AI agent self-correction.", href: "/docs/integration/error-handling" },
    ],
  },
  {
    id: "ai",
    label: "AI Agents",
    items: [
      { title: "Why AI-First", desc: "10 design pillars — pure functions, semantic types, MCP native.", href: "/ai" },
      { title: "MCP Tool Catalog", desc: "All 7 tools with parameters, returns, and use cases.", href: "/ai/mcp-tools" },
      { title: "Agent Patterns", desc: "8 workflows: chart summary, transit alerts, compatibility.", href: "/ai/patterns" },
      { title: "Agent Quickstart", desc: "Zero to computed chart via MCP in 7 steps.", href: "/ai/quickstart" },
      { title: "Feature Comparison", desc: "Vedākṣha vs traditional C, Python, and REST approaches.", href: "/ai/comparison" },
    ],
  },
  {
    id: "platforms",
    label: "Platforms",
    items: [
      { title: "WASM / Browser", desc: "Client-side computation. 11 functions, zero server.", href: "/docs/integration/wasm-browser" },
      { title: "Python Bindings", desc: "pip install vedaksha — 10 functions with type stubs.", href: "/docs/integration/python-bindings" },
      { title: "Batch Computation", desc: "Stateless parallelism. Thread-safe. 10K charts in seconds.", href: "/docs/integration/batch-computation" },
    ],
  },
  {
    id: "reference",
    label: "API Reference",
    items: [
      { title: "Crate Reference", desc: "9 crates, 212 public API items — functions, structs, enums.", href: "/api-ref" },
    ],
  },
];

export default function DocsPage() {
  return (
    <div className="max-w-6xl mx-auto px-6 py-16">
      <div className="flex items-start justify-between mb-12">
        <div>
          <div className="flex items-center gap-3 mb-3">
            <Logo size="medium" className="size-8 text-[var(--color-brand-primary)]" />
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843]">
              Knowledge Base
            </p>
          </div>
          <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-2">
            Learn <span className="text-[#D4A843]">Vedākṣha</span>
          </h1>
          <p className="text-base text-[var(--color-brand-text-secondary)] max-w-lg">
            Everything in one place — computation, Vedic astrology, AI integration,
            graph output, and platform guides.
          </p>
        </div>
        <div className="hidden lg:flex flex-col items-end gap-2 pt-2">
          <code className="text-xs font-mono text-[var(--color-brand-text-muted)] bg-[var(--color-brand-bg-code)] border border-[var(--color-brand-border)] rounded px-2.5 py-1">
            cargo add vedaksha
          </code>
          <a href="mailto:info@arthiq.net" className="text-xs text-[var(--color-brand-text-muted)] hover:text-[var(--color-brand-text-secondary)] transition-colors">
            info@arthiq.net
          </a>
        </div>
      </div>

      <nav className="flex flex-wrap gap-2 mb-12 pb-6 border-b border-[var(--color-brand-border)]">
        {sections.map((s) => (
          <a
            key={s.id}
            href={`#${s.id}`}
            className="text-xs font-medium px-3 py-1.5 rounded-md border border-[var(--color-brand-border)] text-[var(--color-brand-text-muted)] hover:text-[var(--color-brand-text)] hover:border-[var(--color-brand-text-secondary)] transition-colors"
          >
            {s.label}
          </a>
        ))}
      </nav>

      <div className="space-y-14">
        {sections.map((section) => (
          <section key={section.id} id={section.id}>
            <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
              {section.label}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {section.items.map((item) => (
                <Link
                  key={item.title}
                  href={item.href}
                  className="group block border border-[var(--color-brand-border)] rounded-lg p-4 hover:bg-[var(--color-brand-bg-subtle)] hover:border-[var(--color-brand-text-muted)] transition-all no-underline"
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <h3 className="text-sm font-semibold text-[var(--color-brand-text)] group-hover:text-[#D4A843] transition-colors">
                      {item.title}
                    </h3>
                    <span className="text-[var(--color-brand-text-muted)] group-hover:text-[#D4A843] transition-colors text-xs">
                      →
                    </span>
                  </div>
                  <p className="text-xs leading-relaxed text-[var(--color-brand-text-muted)]">
                    {item.desc}
                  </p>
                </Link>
              ))}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
