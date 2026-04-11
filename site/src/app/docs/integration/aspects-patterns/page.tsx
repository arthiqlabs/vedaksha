export default function AspectsPatternsPage() {
  const aspects = [
    { name: "Conjunction", angle: "0°", orb: "8°", category: "Major", desc: "Planets at the same degree blend their energies completely." },
    { name: "Opposition", angle: "180°", orb: "8°", category: "Major", desc: "Planets face each other across the chart axis." },
    { name: "Trine", angle: "120°", orb: "8°", category: "Major", desc: "Planets in the same element, harmonious flow." },
    { name: "Square", angle: "90°", orb: "7°", category: "Major", desc: "Tension and challenge between the two principles." },
    { name: "Sextile", angle: "60°", orb: "6°", category: "Major", desc: "Opportunity and cooperation between compatible elements." },
    { name: "Quincunx", angle: "150°", orb: "3°", category: "Minor", desc: "Adjustment required; the two planets share nothing in modality or element." },
    { name: "Semi-Sextile", angle: "30°", orb: "2°", category: "Minor", desc: "Adjacent signs; subtle friction or resource sharing." },
    { name: "Semi-Square", angle: "45°", orb: "2°", category: "Minor", desc: "Minor irritant, like a Square but lighter." },
    { name: "Sesquiquadrate", angle: "135°", orb: "2°", category: "Minor", desc: "Internal friction, agitation toward resolution." },
    { name: "Quintile", angle: "72°", orb: "1.5°", category: "Minor", desc: "Creative talent, linked to the 5th harmonic." },
    { name: "Bi-Quintile", angle: "144°", orb: "1.5°", category: "Minor", desc: "Expression of quintile creativity in more complex form." },
  ];

  const patterns = [
    { name: "Grand Trine", desc: "Three planets each in trine to the other two, forming an equilateral triangle. Fluid, self-contained energy within one element." },
    { name: "T-Square", desc: "Two planets in opposition, both squared by a third. Dynamic tension that focuses pressure on the apex planet." },
    { name: "Yod", desc: "Two planets in sextile, both quincunx to a third (the apex). A configuration of adjustment and fate." },
    { name: "Grand Cross", desc: "Four planets in mutual squares and oppositions, forming a cross. Intense, multi-directional tension that can generate great productivity." },
    { name: "Stellium", desc: "Three or more planets within an 8° band. Concentration of energy in one sign or house." },
  ];

  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Aspects &amp; <span className="text-[#D4A843]">Patterns</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Vedākṣha computes 11 aspect types with configurable orbs, detects applying and
        separating motion, assigns a strength score to each aspect, and identifies 5
        major pattern configurations across the chart.
      </p>

      {/* Aspect types table */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          The 11 Aspect Types
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Default orbs are applied per aspect type as shown. All orbs are configurable
          via{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">AspectConfig</code>.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="grid grid-cols-[1fr_auto_auto_1fr] border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)]">Aspect</div>
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] text-center">Angle</div>
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] text-center">Default Orb</div>
            <div className="px-4 py-2.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)]">Type</div>
          </div>
          {aspects.map((a, i) => (
            <div
              key={a.name}
              className={`grid grid-cols-[1fr_auto_auto_1fr] ${i < aspects.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""} bg-[var(--color-brand-bg)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors`}
            >
              <div className="px-4 py-3">
                <p className="text-xs font-semibold text-[var(--color-brand-text)]">{a.name}</p>
                <p className="text-[10px] text-[var(--color-brand-text-muted)] mt-0.5">{a.desc}</p>
              </div>
              <div className="px-4 py-3 flex items-center">
                <code className="text-xs font-mono text-[#D4A843]">{a.angle}</code>
              </div>
              <div className="px-4 py-3 flex items-center">
                <code className="text-xs font-mono text-[var(--color-brand-text-secondary)]">{a.orb}</code>
              </div>
              <div className="px-4 py-3 flex items-center">
                <span className={`text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded ${
                  a.category === "Major"
                    ? "bg-[#D4A843]/10 text-[#D4A843]"
                    : "bg-[var(--color-brand-bg-subtle)] text-[var(--color-brand-text-muted)]"
                }`}>
                  {a.category}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* find_aspects */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Finding Aspects
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Call{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">find_aspects</code>
          {" "}with a slice of planetary positions. Pass{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">AspectConfig::default()</code>
          {" "}to use standard orbs, or customize per aspect type.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">aspects.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

let chart = compute_chart(jd, lat, lon, &ChartConfig::tropical())?;

// Default orbs
let aspects = find_aspects(&chart.planets, &AspectConfig::default())?;

for asp in &aspects {
    println!(
        "{} {} {} — orb {:.2}° {} strength {:.2}",
        asp.body_a,
        asp.aspect_type,
        asp.body_b,
        asp.orb,
        if asp.applying { "applying" } else { "separating" },
        asp.strength,
    );
}
// Venus Trine Mars — orb 1.34° applying  strength 0.83
// Sun   Square Saturn — orb 4.72° separating strength 0.33`}</code>
          </pre>
        </div>

        {/* Custom orbs */}
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">custom_orbs.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let config = AspectConfig {
    conjunction:    10.0,
    opposition:     10.0,
    trine:           8.0,
    square:          7.0,
    sextile:         5.0,
    quincunx:        3.0,
    // Minor aspects default to 2° if not specified
    ..AspectConfig::default()
};

let aspects = find_aspects(&chart.planets, &config)?;`}</code>
          </pre>
        </div>
      </div>

      {/* Applying vs separating */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Applying vs Separating
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          An aspect is <strong className="text-[var(--color-brand-text)]">applying</strong> when the angular separation between the two
          planets is moving toward the exact aspect angle, and{" "}
          <strong className="text-[var(--color-brand-text)]">separating</strong> when it is
          moving away. Applying aspects are generally considered more potent because they
          represent energy building toward a peak.
        </p>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5">
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            Vedākṣha computes application/separation by comparing the current angular
            separation with the separation one day later (using the planets&apos; speed
            vectors). Retrograde planets are handled correctly — a retrograde faster
            planet can be applying to a direct slower planet even if it appears to be
            moving away in longitude.
          </p>
        </div>
      </div>

      {/* Aspect strength */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Aspect Strength
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Each aspect carries a{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">strength</code>
          {" "}score from 0.0 to 1.0. The score is computed using a linear falloff from
          the exact angle to the edge of the orb: an exact aspect scores 1.0, an aspect
          at the orb boundary scores 0.0.
        </p>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5 font-mono text-xs text-[var(--color-brand-text-secondary)] bg-[var(--color-brand-bg-code)]">
          <p>strength = 1.0 − (|orb| / max_orb)</p>
          <p className="mt-2 text-[var(--color-brand-text-muted)]">// Example: trine with orb = 3.0°, max_orb = 8.0°</p>
          <p>// strength = 1.0 − (3.0 / 8.0) = 0.625</p>
        </div>
      </div>

      {/* Pattern detection */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Pattern Detection
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Pass the computed aspect list to{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">detect_patterns</code>
          {" "}to find composite configurations.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">patterns.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let aspects  = find_aspects(&chart.planets, &AspectConfig::default())?;
let patterns = detect_patterns(&aspects)?;

for pattern in &patterns {
    println!("{:?} — planets: {:?}", pattern.kind, pattern.bodies);
}
// GrandTrine — planets: [Venus, Mars, Jupiter]
// TSquare    — planets: [Sun, Moon, Saturn]  apex: Saturn`}</code>
          </pre>
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px overflow-hidden">
          {patterns.map((p) => (
            <div key={p.name} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <p className="text-xs font-semibold text-[#D4A843] mb-2">{p.name}</p>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{p.desc}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/time-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Time Systems
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/transit-search" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Transit Search →
        </a>
      </div>
    </div>
  );
}
