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
            Built by <span className="text-[#D4A843]">ArthIQ Labs.</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
            ArthIQ Labs designs, builds, and delivers IT, Digital and AI solutions
            for businesses seeking results without traditional consultancy overhead.
            Human intent, meets AI-native execution.
          </p>
        </div>
      </section>

      {/* ─── WHAT WE DO ─── */}
      <section className="px-6 py-20">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-10 text-center">
            What We Do
          </p>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]">
              <h2 className="text-base font-semibold uppercase tracking-[0.1em] text-[#D4A843] mb-4">
                Build
              </h2>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                Solution design and development from architecture through deployment.
                Full-stack engineering with AI-native tooling at every layer.
              </p>
            </div>
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]">
              <h2 className="text-base font-semibold uppercase tracking-[0.1em] text-[#D4A843] mb-4">
                Advise
              </h2>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                Strategic guidance and fractional CTO/CIO leadership. We help teams
                adopt AI-native workflows without the overhead of a large consultancy.
              </p>
            </div>
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]">
              <h2 className="text-base font-semibold uppercase tracking-[0.1em] text-[#D4A843] mb-4">
                Operate
              </h2>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                Managed services, ongoing operations, and AI-powered business process
                support. Enterprise outcomes, no large team required.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ─── VEDAKSHA ─── */}
      <section className="px-6 py-20 border-t border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
            Vedaksha
          </p>
          <h2 className="text-2xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-6">
            The Axis of Wisdom.
          </h2>
          <div className="space-y-4 text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
            <p>
              Vedaksha is our clean-room astronomical ephemeris and Vedic astrology
              computation engine. Every algorithm traces to a published NASA, IAU, or
              academic source. No proprietary data. No copyleft dependencies.
            </p>
            <p>
              9 Rust crates. Sub-arcsecond precision. 44 ayanamsha systems. 10 house
              systems. 50 yogas. 27 nakshatras with deity, yoni, nadi. 5 dasha systems.
              16 vargas with school-variant support. Complete 5-limb panchanga. Graded
              drishti. Degree-precise Shadbala. A property graph ontology with 10 node
              types and 13 edge types. 7 MCP tools for AI agents. WASM for browsers.
              Python bindings via PyO3.
            </p>
            <p>
              Built for the age of AI agents — every chart is a graph waiting to be queried.
            </p>
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
                Hawthorn Woods, Illinois
              </p>
            </div>
            <div className="space-y-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-1">
                  Web
                </p>
                <a
                  href="https://arthiq.net"
                  className="text-sm text-[var(--color-brand-link)] hover:underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  arthiq.net
                </a>
              </div>
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
