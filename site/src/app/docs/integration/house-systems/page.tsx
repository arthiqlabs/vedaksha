export default function HouseSystemsPage() {
  const systems = [
    {
      name: "Placidus",
      variant: "HouseSystem::Placidus",
      when: "Time-based division of the diurnal arc. Most widely used in modern Western practice.",
    },
    {
      name: "Koch",
      variant: "HouseSystem::Koch",
      when: "Birthplace system. Derived from the RAMC and geographic latitude using house cusps based on the time of day.",
    },
    {
      name: "Equal",
      variant: "HouseSystem::Equal",
      when: "Each house spans exactly 30° from the Ascendant. Simple, predictable, and recommended at extreme latitudes.",
    },
    {
      name: "Whole Sign",
      variant: "HouseSystem::WholeSign",
      when: "Each sign is one entire house. The oldest known system, standard in Hellenistic and traditional Vedic practice.",
    },
    {
      name: "Campanus",
      variant: "HouseSystem::Campanus",
      when: "Divides the prime vertical into 12 equal arcs. Associated with Campananus of Novara (13th century).",
    },
    {
      name: "Regiomontanus",
      variant: "HouseSystem::Regiomontanus",
      when: "Divides the celestial equator into 12 equal arcs. Common in horary and traditional European astrology.",
    },
    {
      name: "Porphyry",
      variant: "HouseSystem::Porphyry",
      when: "Trisects each quadrant between the four angles. Simple and works at all latitudes.",
    },
    {
      name: "Morinus",
      variant: "HouseSystem::Morinus",
      when: "Divides the celestial equator equally from the MC. Avoids the distortion problems of time-based systems.",
    },
    {
      name: "Alcabitius",
      variant: "HouseSystem::Alcabitius",
      when: "Semi-arc system from medieval Arabic tradition. Divides the diurnal semi-arc of the Ascendant degree.",
    },
    {
      name: "Sripathi",
      variant: "HouseSystem::Sripathi",
      when: "Vedic Porphyry variant. Trisects the quadrants using sidereal positions; used in some Jyotish traditions.",
    },
  ];

  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        House <span className="text-[#D4A843]">Systems</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Vedākṣha supports 10 house systems through a single unified API. Pass any{" "}
        <code className="font-mono text-sm bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">HouseSystem</code>
        {" "}variant and receive the 12 cusp longitudes, the Ascendant, and the Midheaven.
      </p>

      {/* How to compute */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Computing House Cusps
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          House computation requires four inputs: the house system, the Right Ascension of
          the Midheaven (RAMC), the geographic latitude of the location, and the true
          obliquity of the ecliptic at the moment of interest. Vedākṣha computes obliquity
          and RAMC internally when you call{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute_houses</code>.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">houses.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

fn main() -> Result<(), VedakshaError> {
    let jd = calendar_to_jd(2024, 3, 20, 12.0);

    // Geographic coordinates: New Delhi
    let latitude  = 28.6139_f64;
    let longitude = 77.2090_f64;

    let houses = compute_houses(
        HouseSystem::Placidus,
        jd,
        latitude,
        longitude,
    )?;

    println!("ASC : {:.4}°", houses.ascendant);
    println!("MC  : {:.4}°", houses.midheaven);

    for (i, cusp) in houses.cusps.iter().enumerate() {
        println!("H{:<2} : {:.4}°", i + 1, cusp);
    }

    Ok(())
}`}</code>
          </pre>
        </div>
      </div>

      {/* Output fields */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Output Structure
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {[
            { field: "cusps", type: "[f64; 12]", desc: "Array of 12 ecliptic longitudes (0–360°), one per house cusp, starting with the 1st house." },
            { field: "ascendant", type: "f64", desc: "Ecliptic longitude of the Ascendant (1st house cusp). Equivalent to cusps[0] in most systems." },
            { field: "midheaven", type: "f64", desc: "Ecliptic longitude of the Midheaven (MC). The 10th house cusp in most systems." },
            { field: "system", type: "HouseSystem", desc: "The system used for this computation, echoed back for validation and serialization." },
          ].map((item) => (
            <div key={item.field} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <div className="flex items-baseline gap-2 mb-2">
                <code className="text-xs font-mono text-[#D4A843]">.{item.field}</code>
                <code className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">{item.type}</code>
              </div>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Switching systems */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Switching Systems at Runtime
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          The house system is a plain enum variant passed per-call. There is no global
          state to set. Computing the same chart in multiple systems is a loop over variants.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">compare_systems.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let systems = [
    HouseSystem::Placidus,
    HouseSystem::Koch,
    HouseSystem::WholeSign,
    HouseSystem::Sripathi,
];

for system in systems {
    let h = compute_houses(system, jd, lat, lon)?;
    println!("{:?} ASC: {:.2}°", system, h.ascendant);
}`}</code>
          </pre>
        </div>
      </div>

      {/* Polar latitudes */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Polar Latitude Fallback
        </h2>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5">
          <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4">
            Time-based systems such as Placidus and Koch become geometrically undefined
            at latitudes above roughly 66° (the Arctic/Antarctic circles). When a
            computation fails due to an extreme latitude, Vedākṣha automatically retries
            with the Equal house system and attaches a structured warning to the result.
          </p>
          <pre className="rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-4 py-3 text-xs font-mono text-[var(--color-brand-text-secondary)] overflow-x-auto">
            <code>{`if let Some(warning) = houses.warning {
    // warning.code == "POLAR_FALLBACK"
    // warning.system_used == HouseSystem::Equal
    println!("Fell back to {:?}: {}", warning.system_used, warning.message);
}`}</code>
          </pre>
        </div>
      </div>

      {/* System table */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          All 10 Systems
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="grid grid-cols-[auto_1fr] border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] border-r border-[var(--color-brand-border)]">Variant</div>
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)]">When Used</div>
          </div>
          {systems.map((s, i) => (
            <div
              key={s.name}
              className={`grid grid-cols-[auto_1fr] ${i < systems.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""}`}
            >
              <div className="px-4 py-3 border-r border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                <code className="text-xs font-mono text-[#D4A843] whitespace-nowrap">{s.variant}</code>
              </div>
              <div className="px-4 py-3 bg-[var(--color-brand-bg)]">
                <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{s.when}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/planetary-positions" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Planetary Positions
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/vedic-astrology" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Vedic Astrology →
        </a>
      </div>
    </div>
  );
}
