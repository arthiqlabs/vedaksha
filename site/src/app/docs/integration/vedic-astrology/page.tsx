export default function VedicAstrologyPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Vedic <span className="text-[#D4A843]">Astrology</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha is designed around Jyotish from the ground up. Every classical concept —
        nakshatra, dasha, varga, yoga, Shadbala — is a typed first-class value in the API,
        not an afterthought layered on top of a Western engine.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        All features documented here are available in Rust, Python, and WASM. They are
        part of every chart computation, not gated behind a separate module or license.
      </p>

      {/* Nakshatra */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Nakshatra Lookup
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Every sidereal longitude maps to one of the 27 nakshatras. Call{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">nakshatra_from_longitude</code>
          {" "}with a sidereal ecliptic longitude to receive the full nakshatra record.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">nakshatra.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let sidereal_lon = 83.24; // degrees

let nak = nakshatra_from_longitude(sidereal_lon)?;

println!("Nakshatra : {}", nak.name);       // Mrigashira
println!("Pada      : {}", nak.pada);       // 4
println!("Lord      : {}", nak.lord);       // Mars
println!("Deity     : {}", nak.deity);      // Soma
println!("Guna      : {}", nak.guna);       // Tamas
println!("Gana      : {}", nak.gana);       // Deva`}</code>
          </pre>
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-2 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { field: "name", desc: "Nakshatra name as a typed Nakshatra enum (e.g. Nakshatra::Mrigashira)." },
            { field: "pada", desc: "Quarter of the nakshatra, 1–4. Each pada spans 3°20′." },
            { field: "lord", desc: "Planetary ruler of the nakshatra used in Vimshottari Dasha sequencing." },
            { field: "deity", desc: "Associated deity from traditional sources (BPHS, Taittiriya Brahmana)." },
            { field: "guna", desc: "Tamas, Rajas, or Sattva — the three fundamental qualities." },
            { field: "gana", desc: "Deva, Manushya, or Rakshasa — used in compatibility assessment." },
          ].map((item) => (
            <div key={item.field} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-xs font-mono text-[#D4A843] block mb-1">.{item.field}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Vimshottari Dasha */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Vimshottari Dasha
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          The Vimshottari system is a 120-year planetary period cycle keyed to the Moon&apos;s
          nakshatra at birth. Vedākṣha computes the full dasha tree to 5 levels of
          sub-periods (Maha Dasha → Antar → Pratyantar → Sookshma → Prana Dasha).
          Every node carries exact start and end dates as Julian Day numbers.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">dasha.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let birth_jd = calendar_to_jd(1990, 6, 15.25);
let moon_sidereal = 113.45; // Moon's sidereal longitude at birth

let tree = vimshottari_dasha(moon_sidereal, birth_jd)?;

// Iterate the top-level maha dashas
for maha in &tree.periods {
    println!(
        "{} dasha: {} to {}",
        maha.lord,
        jd_to_calendar(maha.start_jd),
        jd_to_calendar(maha.end_jd),
    );

    // Each period has .sub_periods with the antar dashas, and so on
    for antar in &maha.sub_periods {
        println!("  {} / {}", maha.lord, antar.lord);
    }
}`}</code>
          </pre>
        </div>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5 bg-[var(--color-brand-bg-subtle)]">
          <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">Depth Control</p>
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            Pass a{" "}
            <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">DashaConfig</code>
            {" "}to limit tree depth and reduce memory use. Depth 1 returns only the maha
            dashas; depth 5 (default) returns all sub-period levels.
          </p>
        </div>
      </div>

      {/* Yogini & Chara Dasha */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Yogini and Chara Dasha
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Both alternate dasha systems follow the same tree API. Yogini Dasha runs a
          36-year cycle through eight yoginis. Chara Dasha is sign-based rather than
          nakshatra-based, computed from the position of the Atmakaraka planet.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">other_dashas.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`// Yogini Dasha
let yogini = yogini_dasha(moon_sidereal, birth_jd)?;

// Chara Dasha — requires a full chart with atmakaraka computed
let chart = compute_chart(birth_jd, lat, lon, &ChartConfig::vedic())?;
let chara = chara_dasha(&chart)?;

// Both return the same DashaTree type
for period in &chara.periods {
    println!("{} sign dasha", period.lord);
}`}</code>
          </pre>
        </div>
      </div>

      {/* Shodasha Varga */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Shodasha Varga — All 16 Divisional Charts
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedākṣha computes all 16 divisional charts with independently placed planets
          and house cusps. Each varga is a complete mini-chart, not just a rearrangement
          of the D-1 positions.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-2 sm:grid-cols-4 gap-px overflow-hidden mb-5">
          {[
            ["D-1", "Rasi — Natal"],
            ["D-2", "Hora — Wealth"],
            ["D-3", "Drekkana — Siblings"],
            ["D-4", "Chaturthamsha — Fortune"],
            ["D-7", "Saptamsha — Children"],
            ["D-9", "Navamsha — Marriage / Dharma"],
            ["D-10", "Dashamsha — Career"],
            ["D-12", "Dvadashamsha — Parents"],
            ["D-16", "Shodashamsha — Vehicles"],
            ["D-20", "Vimshamsha — Spiritual Practice"],
            ["D-24", "Chaturvimshamsha — Education"],
            ["D-27", "Saptavimshamsha — Strength"],
            ["D-30", "Trimshamsha — Misfortune / Debt"],
            ["D-40", "Khavedamsha — Auspiciousness"],
            ["D-45", "Akshavedamsha — General Indications"],
            ["D-60", "Shashtiamsha — Past Life Karma"],
          ].map(([div, label]) => (
            <div key={div} className="bg-[var(--color-brand-bg)] px-3 py-3 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-xs font-mono text-[#D4A843] block mb-0.5">{div}</code>
              <p className="text-[10px] leading-relaxed text-[var(--color-brand-text-secondary)]">{label}</p>
            </div>
          ))}
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">vargas.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let chart = compute_chart(birth_jd, lat, lon, &ChartConfig::vedic())?;

// Access any varga directly
let navamsha = &chart.vargas[Varga::D9];
let dashamsha = &chart.vargas[Varga::D10];

for planet in &navamsha.planets {
    println!("{}: {}° in {}", planet.body, planet.longitude, planet.sign);
}`}</code>
          </pre>
        </div>
      </div>

      {/* Yoga detection */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Yoga Detection — 50 Yogas
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedākṣha evaluates 50 classical yogas from Brihat Parashara Hora Shastra and
          Phaladipika. Each yoga in the result carries its formation strength as a score
          from 0.0 to 1.0, the planets involved, and the source text.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">yogas.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let yogas = detect_yogas(&chart)?;

for yoga in yogas.iter().filter(|y| y.strength > 0.7) {
    println!(
        "{} (strength: {:.2}) — {}",
        yoga.name, yoga.strength, yoga.description
    );
    for planet in &yoga.planets_involved {
        println!("  {}", planet);
    }
}`}</code>
          </pre>
        </div>
      </div>

      {/* Shadbala */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Shadbala — Six-Fold Strength
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Shadbala is the classical Vedic system for measuring planetary strength across
          six independent dimensions. Vedākṣha computes all six and their total, plus
          Bhava Bala (house strength) for each of the 12 houses.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden mb-5">
          {[
            { name: "Sthana Bala", desc: "Positional strength: exaltation, own sign, moolatrikona, and other dignities." },
            { name: "Dig Bala", desc: "Directional strength. Each planet is strongest in a specific angular house." },
            { name: "Kala Bala", desc: "Temporal strength based on time of birth: day/night, hora, paksha, etc." },
            { name: "Chesta Bala", desc: "Motional strength. Direct, fast-moving planets score higher. Retrograde adds a different component." },
            { name: "Naisargika Bala", desc: "Natural strength. A fixed hierarchy: Sun highest, Moon, Venus, Jupiter, Mercury, Mars, Saturn." },
            { name: "Drig Bala", desc: "Aspectual strength. Net gain or loss from benefic and malefic aspects received." },
          ].map((item) => (
            <div key={item.name} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <p className="text-xs font-semibold text-[#D4A843] mb-1">{item.name}</p>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">shadbala.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let shadbala = compute_shadbala(&chart)?;

for entry in &shadbala.planets {
    println!(
        "{}: total = {:.2} rupas  (sthana={:.2}, dig={:.2}, kala={:.2})",
        entry.body,
        entry.total,
        entry.sthana_bala,
        entry.dig_bala,
        entry.kala_bala,
    );
}`}</code>
          </pre>
        </div>
      </div>

      {/* Vedic Drishti */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Vedic Drishti (Aspects)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedic aspects are house-based, not degree-based. Every planet fully aspects
          the 7th house from itself. Mars additionally aspects the 4th and 8th, Jupiter
          the 5th and 9th, and Saturn the 3rd and 10th. Vedākṣha computes both full
          (100%) and partial aspects with their traditional strength weights.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">drishti.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let drishti = compute_vedic_drishti(&chart)?;

for aspect in &drishti.aspects {
    println!(
        "{} aspects {} with {:.0}% strength",
        aspect.from_body, aspect.to_body, aspect.strength * 100.0
    );
}`}</code>
          </pre>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/house-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← House Systems
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/sidereal-zodiac" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Sidereal Zodiac →
        </a>
      </div>
    </div>
  );
}
