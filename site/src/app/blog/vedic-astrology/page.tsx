import Link from "next/link";

export default function VedicAstrologyPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HEADER ─── */}
      <section className="px-6 pt-24 pb-14 border-b border-[var(--color-brand-border)]">
        <div className="max-w-2xl mx-auto">
          <Link
            href="/blog"
            className="inline-flex items-center gap-1.5 text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider mb-8 hover:text-[#D4A843] transition-colors no-underline"
          >
            ← Blog
          </Link>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Jyotish · Vedic · Localization
          </p>
          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight leading-[1.15] uppercase text-[var(--color-brand-text)] mb-6">
            Vedic Astrology Deserves Better Software
          </h1>
          <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-6">
            27 nakshatras, 50 yogas, 3 dasha systems, 16 vargas, 44 ayanamshas, 7 languages. Why we made Jyotish a first-class citizen in the type system — not a plugin, not a flag, not a translation table bolted on at the end.
          </p>
          <div className="flex items-center gap-3 text-xs text-[var(--color-brand-text-muted)]">
            <span>January 22, 2026</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>11 min read</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>ArthIQ Labs</span>
          </div>
        </div>
      </section>

      {/* ─── BODY ─── */}
      <article className="px-6 py-16">
        <div className="max-w-2xl mx-auto">
          <div className="space-y-8 text-[var(--color-brand-text-secondary)] leading-relaxed">

            <p className="text-base">
              Most astrological software was built for Western astrology and then extended to handle Vedic. The extension is always visible in the seams: a checkbox labeled &ldquo;sidereal mode,&rdquo; an ayanamsha dropdown added to an existing tropical chart computation, nakshatra names appended as a post-processing step. The tropical chart is the primary artifact; Vedic is a modifier.
            </p>

            <p className="text-base">
              This leads to predictable problems. Dasha systems that do not handle the Moon&apos;s nakshatra balance correctly at birth. Varga charts computed from a rounded tropical longitude rather than from the precise sidereal position. Shadbala implementations that cover three of the six components and call it done. Ayanamsha values that differ from published definitions because they were copied from an existing library rather than computed from the source.
            </p>

            <p className="text-base">
              Vedākṣha does not extend Western astrology with Vedic features. The Jyotish computation layer is built from scratch against primary sources — classical texts, IERS reference frames, and published ayanamsha definitions — with the same rigor applied to the Western coordinate pipeline.
            </p>

            {/* Section: Ayanamsha */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                44 ayanamsha systems
              </h2>
              <p className="text-base">
                Ayanamsha is the correction applied to convert tropical ecliptic longitude to sidereal. The choice of ayanamsha determines nakshatra boundaries, varga placements, and dasha start times. It is not a secondary detail; it is foundational.
              </p>
              <p className="text-base mt-4">
                Different Jyotish traditions use different definitions. Lahiri (Chitrapaksha) is the official standard of the Indian government and the most widely used. Fagan-Bradley defines the Western sidereal tradition. Krishnamurti is specific to KP astrology. Yukteshwar, Raman, Djwhal Khul, Suryasiddhantic, and many others serve various school traditions.
              </p>
              <p className="text-base mt-4">
                Each ayanamsha in Vedākṣha is implemented from its published definition: a reference epoch, a reference star or star group, and a precession model. The value is not stored as a constant — it is computed from the reference epoch to the requested date using the IAU 2006 precession model, which ensures that the value drifts correctly as the precession cycle advances.
              </p>

              <div className="mt-6 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
                {[
                  { name: "Lahiri (Chitrapaksha)", note: "Indian government standard. Spica at 0° Virgo in 285 CE." },
                  { name: "Fagan-Bradley", note: "Western sidereal. Spica at 29°06′24″ Virgo for J2000." },
                  { name: "Krishnamurti (KP)", note: "0.5′ offset from Lahiri. Basis of KP astrology." },
                  { name: "Yukteshwar", note: "Based on The Holy Science (1894). Ref epoch 499 CE." },
                  { name: "Raman (B.V. Raman)", note: "Published by B.V. Raman. Different from Lahiri by ~20–23′." },
                  { name: "True Chitrapaksha", note: "Uses true position of Spica rather than mean." },
                  { name: "Suryasiddhantic", note: "Derived from the Surya Siddhanta text. Historical sidereal zero." },
                  { name: "Aryabhata 522", note: "Siddhantic tradition. Reference epoch 522 CE." },
                ].map((item) => (
                  <div key={item.name} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                    <p className="text-xs font-semibold text-[#D4A843] mb-1">{item.name}</p>
                    <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.note}</p>
                  </div>
                ))}
              </div>
              <p className="text-xs text-[var(--color-brand-text-muted)] mt-2 text-center">
                Eight of the 44 implemented ayanamshas. The full list covers every major Jyotish and Western sidereal tradition.
              </p>
            </div>

            {/* Section: Nakshatras */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                27 nakshatras, complete
              </h2>
              <p className="text-base">
                A nakshatra is a 13°20′ division of the ecliptic, giving 27 lunar mansions of equal size. Each nakshatra has four padas (quarters) of 3°20′ each, for 108 total padas — corresponding to the 108 navamsha divisions of the zodiac.
              </p>
              <p className="text-base mt-4">
                In Vedākṣha, each <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">Nakshatra</code> enum variant carries its full set of attributes: the presiding deity, the ruling graha (for dasha calculation), the syllables for name selection, the gana (deva/manushya/rakshasa), varna, yoni, and nadi classifications, and the symbol. These are not looked up at runtime from a JSON file; they are part of the type definition, accessible at zero cost.
              </p>
              <p className="text-base mt-4">
                The Moon&apos;s nakshatra at birth is the starting point for Vimshottari dasha. The balance remaining in the birth nakshatra determines the start date of the first mahadasha. This calculation depends on the exact fractional position within the nakshatra, which in turn depends on the ayanamsha. An off-by-one-arcminute ayanamsha translates to an off-by-several-days dasha start — a meaningful error for predictive work.
              </p>
            </div>

            {/* Section: Dasha systems */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Three dasha systems, five levels deep
              </h2>
              <p className="text-base">
                Dasha systems are the temporal dimension of Vedic astrology — they partition a lifetime into periods ruled by different planets. All three implemented systems produce a tree of periods, each node carrying its start date, end date, duration, and ruling planet.
              </p>

              <div className="mt-5 space-y-4">
                {[
                  {
                    name: "Vimshottari",
                    cycle: "120-year cycle",
                    body: "The most widely used dasha system. Nine planetary periods with fixed durations: Ketu 7 years, Venus 20, Sun 6, Moon 10, Mars 7, Rahu 18, Jupiter 16, Saturn 19, Mercury 17. The sequence and start point are determined by the Moon&apos;s nakshatra. Vedākṣha computes all five levels: mahadasha, antardasha, pratyantardasha, sookshmadasha, and prana dasha.",
                  },
                  {
                    name: "Yogini",
                    cycle: "36-year cycle",
                    body: "Eight-period system based on the Moon&apos;s nakshatra with a shorter 36-year cycle. Used in Northern and some Eastern traditions. Period durations are 1 through 8 years for Mangala through Sankata, in nakshatra-determined sequence. Less commonly implemented in software despite its widespread use in practice.",
                  },
                  {
                    name: "Chara (Jaimini)",
                    cycle: "Rasi-based",
                    body: "Sign-based dasha system from the Jaimini tradition. Periods are ruled by rasis (signs) rather than grahas, based on the computed chara karakas — the seven planetary significators ranked by degree. The duration calculation depends on the lord&apos;s position, making it structurally different from nakshatra-based systems.",
                  },
                ].map((item) => (
                  <div key={item.name} className="border border-[var(--color-brand-border)] rounded-xl p-5 bg-[var(--color-brand-bg-subtle)]">
                    <div className="flex items-center justify-between mb-3">
                      <h3 className="text-sm font-semibold text-[#D4A843] uppercase tracking-wide">
                        {item.name}
                      </h3>
                      <span
                        className="text-[11px] font-semibold px-2.5 py-1 rounded-full"
                        style={{ color: "#D4A843", backgroundColor: "rgba(212,168,67,0.1)" }}
                      >
                        {item.cycle}
                      </span>
                    </div>
                    <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                      {item.body}
                    </p>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: Vargas */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                16 divisional charts
              </h2>
              <p className="text-base">
                Varga charts subdivide each sign into equal parts and redistribute the planets into a new 12-sign framework. Each varga has a specific domain of life it illuminates — the rasi (D1) is the body and overall life, the navamsha (D9) is marriage and dharma, the dasamsha (D10) is career, and so on.
              </p>

              <div className="mt-5 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-2 sm:grid-cols-4 gap-px overflow-hidden">
                {[
                  { d: "D1", name: "Rasi", domain: "Body, overall life" },
                  { d: "D2", name: "Hora", domain: "Wealth" },
                  { d: "D3", name: "Drekkana", domain: "Siblings, courage" },
                  { d: "D4", name: "Chaturthamsha", domain: "Property, fortune" },
                  { d: "D5", name: "Panchamsha", domain: "Children, creativity" },
                  { d: "D6", name: "Shashthamsha", domain: "Enemies, disease" },
                  { d: "D7", name: "Saptamsha", domain: "Children, progeny" },
                  { d: "D8", name: "Ashtamsha", domain: "Longevity, obstacles" },
                  { d: "D9", name: "Navamsha", domain: "Spouse, dharma" },
                  { d: "D10", name: "Dasamsha", domain: "Career, status" },
                  { d: "D12", name: "Dvadasamsha", domain: "Parents" },
                  { d: "D16", name: "Shodasamsha", domain: "Vehicles, comforts" },
                  { d: "D20", name: "Vimsamsha", domain: "Spiritual practice" },
                  { d: "D24", name: "Chaturvimsamsha", domain: "Education, learning" },
                  { d: "D27", name: "Saptavimsamsha", domain: "Strength, vitality" },
                  { d: "D60", name: "Shashtiamsha", domain: "Past karma, subtle body" },
                ].map((item) => (
                  <div key={item.d} className="bg-[var(--color-brand-bg)] p-3.5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                    <div className="flex items-center gap-2 mb-1">
                      <code className="text-xs font-mono text-[#D4A843]">{item.d}</code>
                      <span className="text-xs font-semibold text-[var(--color-brand-text)]">{item.name}</span>
                    </div>
                    <p className="text-[11px] leading-snug text-[var(--color-brand-text-muted)]">{item.domain}</p>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: Shadbala */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Shadbala: all six components
              </h2>
              <p className="text-base">
                Shadbala (&ldquo;six strengths&rdquo;) is the classical Vedic method for quantifying planetary strength. Many implementations cover two or three components and call it complete. All six are implemented in Vedākṣha:
              </p>

              <div className="mt-5 space-y-2">
                {[
                  { bala: "Sthana Bala", desc: "Positional strength — exaltation, moolatrikona, own sign, friend&apos;s sign, neutral, enemy, debilitation. Computed from the planet&apos;s exact degree." },
                  { bala: "Dig Bala", desc: "Directional strength. Each planet has a preferred house quadrant: Jupiter and Mercury gain strength in the 1st house, Sun and Mars in the 10th, Saturn in the 7th, Moon and Venus in the 4th." },
                  { bala: "Kala Bala", desc: "Temporal strength — time of day (day/night birth), season, paksha (lunar fortnight), year lord, month lord, weekday lord, and hour lord. Seven sub-components." },
                  { bala: "Chesta Bala", desc: "Motional strength — based on a planet&apos;s speed relative to its mean motion. Retrograde planets gain Chesta Bala; planets at maximum speed lose it." },
                  { bala: "Naisargika Bala", desc: "Natural strength — fixed hierarchy: Sun, Moon, Venus, Jupiter, Mercury, Mars, Saturn in descending order. Does not vary by chart." },
                  { bala: "Drig Bala", desc: "Aspectual strength — net of benefic and malefic aspects received. Computed from the full aspect matrix using classical graha drishti strengths (full, three-quarter, half, quarter)." },
                ].map((item) => (
                  <div key={item.bala} className="flex items-start gap-4 border border-[var(--color-brand-border)] rounded-lg px-4 py-3 bg-[var(--color-brand-bg-subtle)]">
                    <span className="text-xs font-semibold text-[#D4A843] shrink-0 pt-0.5 w-32">{item.bala}</span>
                    <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: 7-language localization */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                7-language localization
              </h2>
              <p className="text-base">
                Jyotish is practiced across South Asia in many languages. The classical terminology originated in Sanskrit, was documented in Tamil and Malayalam texts, spread through Hindi as a modern lingua franca, and is practiced regionally in Telugu, Kannada, and Bengali. A library that returns &ldquo;Ashwini&rdquo; in English for practitioners who read their textbooks in Telugu has not solved localization.
              </p>
              <p className="text-base mt-4">
                Every named entity in Vedākṣha — planets, signs, nakshatras, yogas, dasha lords, house names — has translations in all seven supported languages: English, Hindi, Sanskrit, Tamil, Telugu, Kannada, and Bengali. The translations are part of the type definitions, not a separate lookup table.
              </p>

              <div className="mt-6 rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <span className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider">Nakshatra · Ashwini</span>
                </div>
                <div className="divide-y divide-[var(--color-brand-border)]">
                  {[
                    { lang: "English", val: "Ashwini" },
                    { lang: "Sanskrit", val: "अश्विनी" },
                    { lang: "Hindi", val: "अश्विनी" },
                    { lang: "Tamil", val: "அசுவினி" },
                    { lang: "Telugu", val: "అశ్వని" },
                    { lang: "Kannada", val: "ಅಶ್ವಿನಿ" },
                    { lang: "Bengali", val: "অশ্বিনী" },
                  ].map((item) => (
                    <div key={item.lang} className="flex items-center gap-6 px-4 py-2.5 bg-[var(--color-brand-bg-subtle)]">
                      <span className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider w-20 shrink-0">
                        {item.lang}
                      </span>
                      <span className="text-sm text-[var(--color-brand-text-secondary)] font-medium">
                        {item.val}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Section: 50 yogas */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                50 yoga rules
              </h2>
              <p className="text-base">
                A yoga is a named planetary combination — a set of conditions on planet placement, house lordship, aspects, and dignity that, when met, produces a specific effect. The classical texts describe hundreds; Vedākṣha implements 50, covering the most consequential and well-defined combinations:
              </p>

              <div className="mt-5 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)] p-5">
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-2">
                  {[
                    "Pancha Mahapurusha Yogas (5)",
                    "Dhana Yogas (9)",
                    "Raja Yogas (7)",
                    "Parivartana (Mutual Exchange) Yogas",
                    "Neecha Bhanga Raja Yoga",
                    "Viparita Raja Yogas (3)",
                    "Kemadruma Yoga",
                    "Gajakesari Yoga",
                    "Budhaditya Yoga",
                    "Chandra-Mangala Yoga",
                    "Amala Yoga",
                    "Vasumati Yoga",
                    "Saraswati Yoga",
                    "Hamsa Yoga",
                    "Malavya Yoga",
                    "Bhadra Yoga",
                  ].map((yoga) => (
                    <div key={yoga} className="flex items-start gap-2">
                      <span className="mt-1.5 size-1.5 rounded-full bg-[#D4A843] shrink-0" />
                      <span className="text-xs text-[var(--color-brand-text-secondary)]">{yoga}</span>
                    </div>
                  ))}
                </div>
              </div>

              <p className="text-base mt-4">
                Each yoga rule is expressed as a predicate over the chart graph — it queries planet placements, house lords, and aspect relationships. When the predicate passes, the yoga is added to the chart with the activating planets identified. This makes yoga detection transparent: you can inspect exactly which condition was met, not just whether a flag is set.
              </p>
            </div>

            {/* Closing */}
            <p className="text-base">
              Jyotish is a sophisticated astronomical and interpretive system with 2000 years of continuous development. It does not need to be bolted onto a Western framework as an afterthought. Implementing it correctly requires the same rigor as the coordinate pipeline — primary sources, verified calculations, precise type definitions. That is what first-class means in practice.
            </p>

          </div>

          {/* ─── FOOTER NAV ─── */}
          <div className="mt-16 pt-10 border-t border-[var(--color-brand-border)] flex items-center justify-between">
            <Link
              href="/blog/charts-as-graphs"
              className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider hover:text-[#D4A843] transition-colors no-underline"
            >
              ← Charts as Graphs
            </Link>
            <Link
              href="/blog"
              className="text-xs font-semibold text-[#D4A843] uppercase tracking-wider hover:underline no-underline"
            >
              All posts →
            </Link>
          </div>
        </div>
      </article>

    </div>
  );
}
