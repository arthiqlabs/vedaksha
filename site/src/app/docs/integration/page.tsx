import Link from "next/link";

const guides = [
  {
    title: "Planetary Positions",
    desc: "Computing longitudes, latitudes, distances, and daily speed for any body at any date.",
    href: "/docs/integration/planetary-positions",
  },
  {
    title: "House Systems",
    desc: "10 house systems with polar fallback, from Placidus to Sripathi.",
    href: "/docs/integration/house-systems",
  },
  {
    title: "Vedic Astrology",
    desc: "Nakshatras, dashas, vargas, yogas, Shadbala — Vedākṣha's Vedic-first approach.",
    href: "/docs/integration/vedic-astrology",
  },
  {
    title: "Sidereal Zodiac",
    desc: "44 ayanamsha systems and tropical-to-sidereal conversion.",
    href: "/docs/integration/sidereal-zodiac",
  },
  {
    title: "Coordinate Systems",
    desc: "ICRS, ecliptic, equatorial transforms with the full IAU pipeline.",
    href: "/docs/integration/coordinate-systems",
  },
  {
    title: "Time Systems",
    desc: "Julian Day, UT, TT, Delta T, sidereal time — all conversions.",
    href: "/docs/integration/time-systems",
  },
  {
    title: "Aspects & Patterns",
    desc: "11 aspect types, applying/separating, Grand Trine, T-Square, Yod detection.",
    href: "/docs/integration/aspects-patterns",
  },
  {
    title: "Transit Search",
    desc: "Find exact transit moments, solar/lunar returns, synastry, muhurta.",
    href: "/docs/integration/transit-search",
  },
  {
    title: "Graph Output",
    desc: "ChartGraph with 10 node types, 13 edge types — emit to Neo4j, SurrealDB, JSON-LD.",
    href: "/docs/integration/graph-output",
  },
  {
    title: "MCP Integration",
    desc: "7 typed tools for AI agents with structured errors and streaming.",
    href: "/docs/integration/mcp-integration",
  },
  {
    title: "WASM Browser",
    desc: "Full computation client-side via WebAssembly. Zero server dependency.",
    href: "/docs/integration/wasm-browser",
  },
  {
    title: "Python Bindings",
    desc: "pip install vedaksha — 10 functions with type stubs for IDE support.",
    href: "/docs/integration/python-bindings",
  },
  {
    title: "Batch Computation",
    desc: "Computing thousands of charts efficiently with stateless parallelism.",
    href: "/docs/integration/batch-computation",
  },
  {
    title: "Error Handling",
    desc: "Structured errors, precision modes, polar fallback, self-correction hints.",
    href: "/docs/integration/error-handling",
  },
  {
    title: "Data Sources",
    desc: "NASA JPL, IAU standards, BPHS, Hipparcos — all public, all cited.",
    href: "/docs/integration/data-sources",
  },
  {
    title: "FAQ",
    desc: "Common questions about accuracy, licensing, Vedic support, and platform targets.",
    href: "/docs/integration/faq",
  },
];

export default function IntegrationGuidePage() {
  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight text-[var(--color-brand-text)] mb-3 uppercase">
        The complete <span className="text-[#D4A843]">Vedākṣha</span> guide
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Whether you&apos;re building your first astrological application, integrating
        celestial computation into an AI agent, or migrating from another library —
        start here.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {guides.map((guide) =>
          guide.href ? (
            <Link
              key={guide.title}
              href={guide.href}
              className="group block border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors no-underline"
            >
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-sm font-semibold text-[var(--color-brand-text)] uppercase tracking-wide">
                  {guide.title}
                </h2>
                <span className="text-xs font-semibold text-[#D4A843] group-hover:underline shrink-0 ml-3">
                  Read →
                </span>
              </div>
              <p className="text-sm text-[var(--color-brand-text-muted)]">
                {guide.desc}
              </p>
            </Link>
          ) : (
            <div
              key={guide.title}
              className="border border-[var(--color-brand-border)] rounded-xl p-5 opacity-50"
            >
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-sm font-semibold text-[var(--color-brand-text)] uppercase tracking-wide">
                  {guide.title}
                </h2>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text-muted)] border border-[var(--color-brand-border)]">
                  soon
                </span>
              </div>
              <p className="text-sm text-[var(--color-brand-text-muted)]">
                {guide.desc}
              </p>
            </div>
          )
        )}
      </div>
    </div>
  );
}
