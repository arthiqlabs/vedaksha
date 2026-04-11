export default function DataSourcesPage() {
  const sources = [
    {
      num: "01",
      short: "JPL DE440 / DE441",
      full: "NASA Jet Propulsion Laboratory Development Ephemeris 440 / 441",
      domain: "Planetary positions",
      coverage: "DE440: 1550–2650 CE · DE441: ~13000 BCE – 17000 CE",
      license: "Public domain. Released by JPL/NASA.",
      desc: "The definitive modern planetary ephemeris, produced by numerical integration of the equations of motion for the solar system bodies. DE440 is the current standard file. DE441 is the extended-range variant for historical and far-future calculations. Both are public domain and freely downloadable from the JPL Solar System Dynamics website.",
      precision: "Sub-arcsecond agreement with JPL Horizons for all major bodies.",
    },
    {
      num: "02",
      short: "IAU 2006 Precession",
      full: "International Astronomical Union Precession Model 2006 (Capitaine, Wallace & Chapront)",
      domain: "Ecliptic & equatorial coordinate transforms",
      coverage: "Applicable from J2000.0 ± several centuries",
      license: "IAU standards. Open access.",
      desc: "The current IAU standard for computing the precession of the equinoxes and the ecliptic. Vedākṣha uses IAU 2006 precession throughout its coordinate transformation pipeline to convert from ICRS (barycentric) to geocentric ecliptic coordinates and equatorial coordinates.",
      precision: "Arcsecond-level agreement with SOFA/ERFA reference implementations.",
    },
    {
      num: "03",
      short: "IAU 2000B Nutation",
      full: "International Astronomical Union Nutation Model 2000B (McCarthy & Luzum, 2003)",
      domain: "True ecliptic coordinates",
      coverage: "Continuous — algebraic series",
      license: "IAU standards. Open access.",
      desc: "The 2000B truncated nutation series, comprising 77 luni-solar terms and 1 planetary term. Used by Vedākṣha to compute the true obliquity of the ecliptic and nutation in longitude — the small periodic perturbation of Earth&apos;s orientation relative to inertial space.",
      precision: "Agreement within 1 mas of IAU 2000A over the years 1995–2050.",
    },
    {
      num: "04",
      short: "Meeus — Astronomical Algorithms",
      full: "Jean Meeus, Astronomical Algorithms, 2nd edition (Willmann-Bell, 1998)",
      domain: "Julian Day conversion, sidereal time, solar corrections",
      coverage: "N/A — algorithmic reference",
      license: "Widely cited textbook. Algorithms in public domain.",
      desc: "The standard algorithmic reference for practical astronomical computing. Vedākṣha derives its Julian Day conversion, Greenwich Apparent Sidereal Time (GAST), Delta T approximation for historical dates, and various solar/lunar correction algorithms directly from Meeus. Every chapter is cited in the source code at the point of use.",
      precision: "Meeus algorithms are consistent with JPL Horizons to the level documented in the book.",
    },
    {
      num: "05",
      short: "BPHS — Brihat Parashara Hora Shastra",
      full: "Brihat Parashara Hora Shastra (traditional; translated by G.C. Sharma and others)",
      domain: "Vedic astrological algorithms",
      coverage: "All Vedic features: nakshatras, dashas, yogas, shadbala, vargas",
      license: "Traditional text. Public domain.",
      desc: "The foundational text of Vedic astrology, attributed to the sage Parashara. Vedākṣha&apos;s implementations of Vimshottari dasha periods, nakshatra boundaries, yoga definitions, Shadbala strength components, and the 16 Shodasha Varga divisional charts are all sourced from BPHS. Chapter and verse references are embedded in the source code.",
      precision: "Algorithmic precision; classical definitions are exact by construction.",
    },
    {
      num: "06",
      short: "Holden — Elements of House Division",
      full: "Ralph William Holden, The Elements of House Division (Fowler, 1977)",
      domain: "House system algorithms",
      coverage: "Placidus, Koch, Regiomontanus, Campanus, Morinus, Equal, Whole Sign, and more",
      license: "Published reference. Algorithms widely cited.",
      desc: "The definitive algorithmic treatment of astrological house division. Vedākṣha implements all major house systems using the mathematical derivations in Holden, extended with the Sripathi house system from classical Vedic sources. The polar fallback to Whole Sign for extreme latitudes follows Holden&apos;s recommended practice.",
      precision: "Agreement with professional astrological software for all tested locations and dates.",
    },
    {
      num: "07",
      short: "Hipparcos Star Catalogue",
      full: "ESA Hipparcos Catalogue (ESA, 1997) — HIP main catalogue",
      domain: "Fixed star positions",
      coverage: "~118 000 stars; positions for epoch J1991.25",
      license: "Public domain. Released by ESA.",
      desc: "The astrometric catalogue produced by the ESA Hipparcos space mission. Vedākṣha uses Hipparcos star positions (propagated to J2000.0 using proper motion) for fixed star placements in the ChartGraph FixedStar nodes. Integration is planned for a future release; the catalogue reference is included here for completeness.",
      precision: "Milliarcsecond-level astrometry for the included stars.",
      tag: "planned",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — Data Sources
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        All data. <span className="text-[#D4A843]">All public.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha does not use proprietary data files, licensed ephemeris services,
        or undocumented algorithms. Every data source is public, cited, and linked.
        You can independently verify every number the library produces.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        Source citations are embedded in the Rust source at the point of use —
        not just in documentation. The chapter, verse, or paper section is in the
        comment above the function.
      </p>

      <div className="space-y-6">
        {sources.map((s) => (
          <div
            key={s.num}
            className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
          >
            <div className="px-6 py-5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <div className="flex items-center gap-3 mb-1">
                    <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                      {s.num}
                    </span>
                    {s.tag && (
                      <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text-muted)] border border-[var(--color-brand-border)]">
                        {s.tag}
                      </span>
                    )}
                  </div>
                  <h2 className="text-base font-semibold text-[#D4A843] uppercase tracking-wide mb-0.5">
                    {s.short}
                  </h2>
                  <p className="text-xs text-[var(--color-brand-text-muted)]">{s.full}</p>
                </div>
              </div>
            </div>
            <div className="px-6 py-5">
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mb-4">
                {s.desc}
              </p>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-1">Domain</p>
                  <p className="text-xs text-[var(--color-brand-text-secondary)]">{s.domain}</p>
                </div>
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-1">Coverage</p>
                  <p className="text-xs text-[var(--color-brand-text-secondary)]">{s.coverage}</p>
                </div>
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-1">License</p>
                  <p className="text-xs text-[var(--color-brand-text-secondary)]">{s.license}</p>
                </div>
              </div>
              <div className="mt-3 pt-3 border-t border-[var(--color-brand-border)]">
                <p className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-1">Precision</p>
                <p className="text-xs text-[var(--color-brand-text-secondary)]">{s.precision}</p>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Summary callout */}
      <div className="mt-12 rounded-xl border border-[#D4A843]/20 bg-[#D4A843]/5 p-6">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[#D4A843] mb-2">
          No Black Boxes
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
          Every algorithm in Vedākṣha is traceable to a public, citable source.
          The library ships with no proprietary data files. The embedded DE440s
          is the same file downloadable from the JPL Solar System Dynamics FTP.
          If a number looks wrong, you can trace it back to the source and verify it
          against JPL Horizons or the SOFA/ERFA reference implementation.
        </p>
      </div>

      <div className="mt-12 flex items-center gap-6">
        <a
          href="/docs/integration/error-handling"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Error Handling"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/faq"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"FAQ →"}
        </a>
      </div>
    </div>
  );
}
