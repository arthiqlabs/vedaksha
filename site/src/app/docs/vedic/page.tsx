export default function VedicPage() {
  const features = [
    {
      num: "01",
      title: "27 Nakshatras",
      subtitle: "108 Padas",
      desc: "Every planet placed in one of the 27 lunar mansions with its pada (quarter). Nakshatra lord, sub-lord, and sub-sub-lord for KP system compatibility.",
    },
    {
      num: "02",
      title: "3 Dasha Systems",
      subtitle: "Vimshottari · Yogini · Chara",
      desc: "Full dasha trees to 5 levels of sub-periods. Vimshottari (120-year), Yogini (36-year), and Chara (sign-based) with exact start and end dates.",
    },
    {
      num: "03",
      title: "16 Shodasha Varga",
      subtitle: "Divisional Charts",
      desc: "All 16 divisional charts from D-1 (Rasi) to D-60 (Shashtiamsha). Each with independently computed planetary positions and house cusps.",
    },
    {
      num: "04",
      title: "50 Vedic Yogas",
      subtitle: "BPHS & Phaladipika",
      desc: "Rajayoga, Dhana yoga, Parivartana, Neecha Bhanga, Viparita Raja — all 50 yogas sourced from Brihat Parashara Hora Shastra and Phaladipika with strength scoring.",
    },
    {
      num: "05",
      title: "Complete Shadbala",
      subtitle: "6 Strength Components",
      desc: "Sthana Bala, Dig Bala, Kala Bala, Chesta Bala, Naisargika Bala, and Drig Bala. Total Shadbala and Bhava Bala for every planet and house.",
    },
    {
      num: "06",
      title: "Vedic Drishti",
      subtitle: "Special Aspects",
      desc: "Full Vedic aspect system with the special 3rd/10th aspects for Mars, 5th/9th for Jupiter, and 4th/8th for Saturn — alongside the universal 7th aspect.",
    },
    {
      num: "07",
      title: "Muhurta Search",
      subtitle: "Tithi · Nakshatra · Weekday",
      desc: "Find auspicious moments in a date range. Filter by tithi, nakshatra, weekday, and planetary hora. Returns streaming results with scoring.",
    },
    {
      num: "08",
      title: "44 Ayanamsha Systems",
      subtitle: "Every Major Tradition",
      desc: "Lahiri (Indian standard), Fagan-Bradley, Krishnamurti, Raman, Yukteshwar, Aryabhata, galactic center, equator systems, and 36 more.",
    },
    {
      num: "09",
      title: "Lunar Nodes",
      subtitle: "Mean · True · Osculating",
      desc: "Three node computation methods. Mean node (polynomial), True node (Meeus 5-term, ~0.09°), and Osculating node from ELP/MPP02 orbital mechanics (<0.03° vs JPL DE441). KP sub-lord ready.",
    },
    {
      num: "10",
      title: "7-Language Localization",
      subtitle: "All Names Translated",
      desc: "Planet, sign, nakshatra, and yoga names in English, Hindi, Sanskrit, Tamil, Telugu, Kannada, and Bengali. Switch locale per request.",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Vedic Astrology
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Jyotish in the <span className="text-[#D4A843]">type system.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha&apos;s Vedic layer is not a plugin or an afterthought. Every classical
        concept — nakshatra, dasha, varga, yoga — is a first-class type in the API,
        sourced from BPHS, Phaladipika, and other primary texts.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        All 9 feature areas below are available in Rust, Python, and WASM. Vedic
        computation is not gated behind a separate module — it is part of every chart.
      </p>

      <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-px overflow-hidden">
        {features.map((f) => (
          <div
            key={f.num}
            className="bg-[var(--color-brand-bg)] p-6 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
          >
            <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
              {f.num}
            </span>
            <h2 className="text-base font-semibold uppercase tracking-wide mt-2 mb-0.5">
              <span className="text-[#D4A843]">{f.title}</span>
            </h2>
            <p className="text-xs font-medium text-[var(--color-brand-text-muted)] uppercase tracking-wider mb-3">
              {f.subtitle}
            </p>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              {f.desc}
            </p>
          </div>
        ))}
      </div>

      <div className="mt-12 flex items-center gap-6">
        <a
          href="/docs/getting-started"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          ← Getting Started
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          Integration Guides →
        </a>
      </div>
    </div>
  );
}
