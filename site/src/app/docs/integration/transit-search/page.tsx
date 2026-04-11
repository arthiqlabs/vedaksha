export default function TransitSearchPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Transit <span className="text-[#D4A843]">Search</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Vedākṣha includes a transit search engine that finds the exact moment of any
        planetary event — aspects to natal positions, solar and lunar returns, or
        user-defined celestial conditions — using an adaptive step and bisection algorithm.
      </p>

      {/* Engine overview */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          How the Search Engine Works
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Brute-force transit finding is slow: computing a full chart for every minute
          of a year requires ~525,000 ephemeris calls. Vedākṣha uses a two-phase approach
          that is both fast and accurate.
        </p>
        <div className="space-y-3">
          {[
            {
              step: "1",
              label: "Adaptive Step Scan",
              desc: "The engine starts with a coarse step (default: 1 day) and monitors the sign of the separation between the transiting planet and the target. When the sign changes, a transit has occurred somewhere in that interval.",
            },
            {
              step: "2",
              label: "Bisection Refinement",
              desc: "Once a bracket is found, bisection narrows the exact crossing to within a configurable tolerance (default: 1 second of time). The algorithm converges in ~17 iterations regardless of the initial step size.",
            },
          ].map((s) => (
            <div key={s.step} className="flex gap-4 border border-[var(--color-brand-border)] rounded-lg p-5">
              <span className="flex items-center justify-center size-7 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-xs font-bold shrink-0 mt-0.5">
                {s.step}
              </span>
              <div>
                <p className="text-sm font-semibold text-[var(--color-brand-text)] mb-1">{s.label}</p>
                <p className="text-sm text-[var(--color-brand-text-secondary)] leading-relaxed">{s.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Basic transit search */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Setting Up a Transit Search
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Construct a{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">TransitSearch</code>
          {" "}with the natal chart, date range, and which transiting bodies and aspect
          types to look for. The result is an iterator of{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">TransitEvent</code>
          {" "}values in chronological order.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">transit_search.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

let birth_jd = calendar_to_jd(1990, 6, 15, 6.0);
let natal    = compute_chart(birth_jd, 28.6139, 77.2090, &ChartConfig::tropical())?;

// Search window: 2024 calendar year
let start_jd = calendar_to_jd(2024, 1, 1, 0.0);
let end_jd   = calendar_to_jd(2024, 12, 31, 23.99);

let search = TransitSearch::new(&natal)
    .date_range(start_jd, end_jd)
    .bodies(&[Body::Jupiter, Body::Saturn, Body::Uranus])
    .aspects(&[AspectType::Conjunction, AspectType::Opposition, AspectType::Trine])
    .orb(1.0)
    .build();

for event in search.find()? {
    let date = jd_to_calendar(event.exact_jd);
    println!(
        "{}-{:02}-{:02}: transiting {} {} natal {}",
        date.year, date.month, date.day,
        event.transiting_body,
        event.aspect_type,
        event.natal_body,
    );
}`}</code>
          </pre>
        </div>
      </div>

      {/* TransitEvent fields */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          TransitEvent Fields
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {[
            { field: "exact_jd", type: "f64", desc: "Julian Day of the exact aspect moment, accurate to within the configured tolerance." },
            { field: "transiting_body", type: "Body", desc: "The transiting planet — the one moving through the natal chart." },
            { field: "natal_body", type: "Body", desc: "The natal planet being aspected." },
            { field: "aspect_type", type: "AspectType", desc: "The aspect type formed (Conjunction, Trine, etc.)." },
            { field: "applying", type: "bool", desc: "True if this event represents the ingress into orb (applying); false for the exit (separating)." },
            { field: "transiting_longitude", type: "f64", desc: "Ecliptic longitude of the transiting planet at the exact moment." },
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

      {/* Solar & Lunar returns */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Solar and Lunar Returns
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          A solar return is the moment the Sun returns to its exact natal longitude.
          A lunar return is the equivalent for the Moon, occurring approximately once
          per month. Both use the same bisection engine internally.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">returns.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`// Solar return — Sun back to natal longitude
let sr = solar_return(&natal, 2024)?;
let sr_date = jd_to_calendar(sr.exact_jd);
println!("Solar return: {}-{:02}-{:02} {:05.2}h UT",
    sr_date.year, sr_date.month, sr_date.day, sr_date.hour_ut);

// Full solar return chart at the exact moment
let sr_chart = compute_chart(
    sr.exact_jd,
    sr_location_lat,  // location at time of return, if relocating
    sr_location_lon,
    &ChartConfig::tropical()
)?;

// Lunar return — Moon back to natal longitude
// Returns the next one after start_jd
let lr = lunar_return(&natal, start_jd)?;`}</code>
          </pre>
        </div>
      </div>

      {/* Synastry */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Synastry (Inter-Chart Aspects)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Synastry compares two natal charts and finds all aspects formed between their
          planets. Each aspect in the result identifies which person&apos;s planet is
          aspecting the other.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">synastry.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let chart_a = compute_chart(jd_a, lat_a, lon_a, &config)?;
let chart_b = compute_chart(jd_b, lat_b, lon_b, &config)?;

let synastry = find_synastry_aspects(
    &chart_a,
    &chart_b,
    &AspectConfig::default(),
)?;

for asp in &synastry.aspects {
    println!(
        "A's {} {} B's {} (orb {:.2}°)",
        asp.body_a, asp.aspect_type, asp.body_b, asp.orb
    );
}`}</code>
          </pre>
        </div>
      </div>

      {/* Composite chart */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Composite Chart (Midpoint Method)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          A composite chart is constructed by taking the midpoint of each pair of
          corresponding planets from two natal charts. The result is a single synthetic
          chart representing the relationship itself. Vedākṣha uses the near-midpoint
          method (selecting the midpoint that produces a coherent chart rather than its
          opposite).
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">composite.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let composite = compute_composite_chart(&chart_a, &chart_b)?;

// composite.planets[i].longitude is the near-midpoint
// of chart_a.planets[i] and chart_b.planets[i]

for planet in &composite.planets {
    println!("{}: {:.4}°", planet.body, planet.longitude);
}`}</code>
          </pre>
        </div>
      </div>

      {/* Muhurta search */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Muhurta Search
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Muhurta is the Vedic practice of selecting an auspicious moment for an
          activity. Vedākṣha&apos;s muhurta engine searches a date range for windows that
          satisfy a set of configurable criteria drawn from classical Jyotish.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">muhurta.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let criteria = MuhurtaCriteria {
    avoid_nakshatras: vec![Nakshatra::Bharani, Nakshatra::Krittika],
    require_weekdays:  vec![Weekday::Monday, Weekday::Wednesday, Weekday::Thursday],
    avoid_tithis:      vec![Tithi::Amavasya, Tithi::Chaturdashi],
    require_moon_sign: Some(Sign::Taurus),
    min_score:         0.7,
};

let windows = find_muhurta(start_jd, end_jd, lat, lon, &criteria)?;

for window in windows.iter().take(5) {
    let start = jd_to_calendar(window.start_jd);
    let end   = jd_to_calendar(window.end_jd);
    println!(
        "{}-{:02}-{:02} {:04.1}h – {:04.1}h UT  score: {:.2}",
        start.year, start.month, start.day,
        start.hour_ut, end.hour_ut,
        window.score,
    );
}`}</code>
          </pre>
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-2 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { filter: "avoid_nakshatras", desc: "Exclude specific lunar mansions." },
            { filter: "require_weekdays", desc: "Only return results on specified weekdays." },
            { filter: "avoid_tithis", desc: "Exclude specific lunar days (1–30)." },
            { filter: "require_moon_sign", desc: "Moon must be in a specified sign." },
            { filter: "planetary_hora", desc: "Match a specific planetary hora (hourly ruler)." },
            { filter: "min_score", desc: "Threshold for the overall auspiciousness score." },
          ].map((item) => (
            <div key={item.filter} className="bg-[var(--color-brand-bg)] p-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-xs font-mono text-[#D4A843] block mb-1">{item.filter}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/aspects-patterns" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Aspects &amp; Patterns
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration" className="text-sm font-semibold text-[#D4A843] hover:underline">
          All Guides →
        </a>
      </div>
    </div>
  );
}
