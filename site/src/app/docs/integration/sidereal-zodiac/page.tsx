export default function SiderealZodiacPage() {
  const ayanamshas = [
    { name: "Lahiri", variant: "Ayanamsha::Lahiri", desc: "Adopted as the Indian national standard in 1955. Based on the star Spica (Chitra) at 180° of sidereal longitude." },
    { name: "Fagan–Bradley", variant: "Ayanamsha::FaganBradley", desc: "Western sidereal standard, placing the vernal point of 221 BCE as the epoch. Used in siderealist Western practice." },
    { name: "Krishnamurti (KP)", variant: "Ayanamsha::Krishnamurti", desc: "A refinement of Lahiri by K.S. Krishnamurti for use in his system of sub-lord prediction. Slightly different epoch." },
    { name: "Raman", variant: "Ayanamsha::Raman", desc: "Proposed by B.V. Raman. Places the vernal point based on a 360 BCE epoch, giving a larger value than Lahiri." },
    { name: "Yukteshwar", variant: "Ayanamsha::Yukteshwar", desc: "Based on Sri Yukteshwar Giri's The Holy Science. Significantly smaller than most others (~22° for 2000 CE)." },
    { name: "True Chitra Paksha", variant: "Ayanamsha::TrueCHitraPaksha", desc: "Dynamic variant: always places the fixed star Spica exactly at 180°. Value changes nightly." },
    { name: "Galactic Center 0° Sag", variant: "Ayanamsha::GalacticCenter0Sag", desc: "Aligns the galactic center with 0° Sagittarius. Used in some cosmologically oriented traditions." },
    { name: "Aryabhata", variant: "Ayanamsha::Aryabhata", desc: "Based on the 499 CE computation from the Aryabhatiya. Historical and scholarly interest." },
  ];

  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Sidereal <span className="text-[#D4A843]">Zodiac</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Vedākṣha supports 44 ayanamsha systems for converting between the tropical and
        sidereal zodiacs. All conversions are bidirectional and accurate to the same
        sub-arcsecond precision as the underlying ephemeris.
      </p>

      {/* Tropical vs Sidereal */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Tropical vs Sidereal
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          <div className="bg-[var(--color-brand-bg)] p-6">
            <p className="text-sm font-semibold text-[var(--color-brand-text)] uppercase tracking-wide mb-3">Tropical Zodiac</p>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              The tropical zodiac is anchored to the seasons. 0° Aries is always the
              Northern Hemisphere vernal equinox — the moment the Sun crosses the
              celestial equator heading north. This reference point drifts slowly
              westward against the fixed stars due to Earth&apos;s axial precession
              (roughly 50 arcseconds per year).
            </p>
          </div>
          <div className="bg-[var(--color-brand-bg)] p-6">
            <p className="text-sm font-semibold text-[var(--color-brand-text)] uppercase tracking-wide mb-3">Sidereal Zodiac</p>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              The sidereal zodiac is anchored to the fixed stars. 0° Aries is defined
              relative to a specific star or star field, keeping the constellations
              roughly aligned with the signs over long periods. Different traditions
              disagree on exactly where the reference point should be — which is why
              there are 44 different ayanamsha systems.
            </p>
          </div>
        </div>
        <div className="mt-4 border border-[var(--color-brand-border)] rounded-xl p-5">
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            The <strong className="text-[var(--color-brand-text)]">ayanamsha</strong> is the angular difference between the two zodiacs at a given
            moment. For the Lahiri system in 2024, this value is approximately 23°51′.
            To convert a tropical longitude to sidereal, subtract the ayanamsha. To go
            the other direction, add it.
          </p>
        </div>
      </div>

      {/* API */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Computing Sidereal Positions
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Pass an{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">Ayanamsha</code>
          {" "}variant in the chart config to receive all planetary longitudes in the
          sidereal frame directly. Conversion is applied after the full coordinate
          pipeline, so precision is not lost.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">sidereal.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

let jd = calendar_to_jd(2024, 3, 20, 12.0);

// Vedic chart with Lahiri ayanamsha
let config = ChartConfig {
    ayanamsha: Ayanamsha::Lahiri,
    house_system: HouseSystem::WholeSign,
    ..ChartConfig::default()
};

let chart = compute_chart(jd, 28.6139, 77.2090, &config)?;

// All longitudes are now sidereal
for planet in &chart.planets {
    println!("{}: {:.4}° (sidereal)", planet.body, planet.longitude);
}

// Or convert a single longitude directly
let tropical_lon = 29.98;
let ayanamsha_val = ayanamsha_value(Ayanamsha::Lahiri, jd)?;
let sidereal_lon  = tropical_to_sidereal(tropical_lon, ayanamsha_val);
let back_to_trop  = sidereal_to_tropical(sidereal_lon, ayanamsha_val);`}</code>
          </pre>
        </div>

        {/* Standalone functions */}
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { fn: "ayanamsha_value()", desc: "Returns the current ayanamsha offset in degrees for a given system and Julian Day." },
            { fn: "tropical_to_sidereal()", desc: "Subtracts the ayanamsha from a tropical longitude, normalizing the result to 0–360°." },
            { fn: "sidereal_to_tropical()", desc: "Adds the ayanamsha back to a sidereal longitude, normalizing to 0–360°." },
          ].map((item) => (
            <div key={item.fn} className="bg-[var(--color-brand-bg)] p-5">
              <code className="text-xs font-mono text-[#D4A843] block mb-2">{item.fn}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Ayanamsha table */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Major Ayanamsha Systems
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedākṣha includes 44 systems in the{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">Ayanamsha</code>
          {" "}enum. Below are the most commonly used.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="grid grid-cols-[auto_1fr] border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] border-r border-[var(--color-brand-border)]">Variant</div>
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)]">Description</div>
          </div>
          {ayanamshas.map((a, i) => (
            <div
              key={a.name}
              className={`grid grid-cols-[auto_1fr] ${i < ayanamshas.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""}`}
            >
              <div className="px-4 py-3 border-r border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                <code className="text-xs font-mono text-[#D4A843] whitespace-nowrap block">{a.variant}</code>
              </div>
              <div className="px-4 py-3 bg-[var(--color-brand-bg)]">
                <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{a.desc}</p>
              </div>
            </div>
          ))}
        </div>
        <p className="text-xs text-[var(--color-brand-text-muted)] mt-3">
          The full list of 44 variants is in the{" "}
          <code className="font-mono text-[10px] bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 text-[#D4A843]">Ayanamsha</code>
          {" "}enum documentation.
        </p>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/vedic-astrology" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Vedic Astrology
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/coordinate-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Coordinate Systems →
        </a>
      </div>
    </div>
  );
}
