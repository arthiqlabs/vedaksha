const values = [
  {
    title: "Public Science",
    desc: "Every algorithm traces to a published NASA, IAU, or academic source. No proprietary data. No copyleft dependencies. Clean-room implementation validated against independent reference ephemerides.",
  },
  {
    title: "Vedic-First",
    desc: "Jyotish astrology deserves more than a plugin. Nakshatras, dashas, vargas, and yogas are in the core type system — with 7-language localization.",
  },
  {
    title: "AI-Native",
    desc: "Built for agents from day one. Pure functions, typed enums, property graph output, MCP protocol, structured errors. Zero state, zero initialization.",
  },
];

export default function AboutPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HERO ─── */}
      <section className="px-6 pt-24 pb-20 border-b border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
            About
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-8">
            The Axis of <span className="text-[#D4A843]">Wisdom.</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            Vedākṣha exists to make celestial computation accessible, precise, and native to the age
            of AI agents. We believe astronomical and astrological computation should be built on
            public science, not proprietary legacy code — and that every chart is a graph waiting
            to be queried.
          </p>
        </div>
      </section>

      {/* ─── VALUES ─── */}
      <section className="px-6 py-20">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-10 text-center">
            Our Values
          </p>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {values.map((v) => (
              <div
                key={v.title}
                className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]"
              >
                <h2 className="text-base font-semibold uppercase tracking-[0.1em] text-[#D4A843] mb-4">
                  {v.title}
                </h2>
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {v.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── COMPANY ─── */}
      <section className="px-6 py-20 border-t border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-8">
            Company
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-8">
            <div>
              <p className="text-2xl font-bold uppercase tracking-wide text-[var(--color-brand-text)] mb-1">
                ArthIQ Labs LLC
              </p>
              <p className="text-sm text-[var(--color-brand-text-muted)]">
                Illinois, United States
              </p>
            </div>
            <div className="space-y-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1">
                  Contact
                </p>
                <a
                  href="mailto:info@arthiq.net"
                  className="text-sm text-[var(--color-brand-link)] hover:underline"
                >
                  info@arthiq.net
                </a>
              </div>
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1">
                  License
                </p>
                <p className="text-sm text-[var(--color-brand-text-secondary)]">
                  BSL 1.1 — free for non-commercial use
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

    </div>
  );
}
