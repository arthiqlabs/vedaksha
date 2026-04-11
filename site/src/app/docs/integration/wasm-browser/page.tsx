export default function WasmBrowserPage() {
  const noEphemerisNeeded = [
    {
      fn: "compute_dasha",
      sig: "(julian_day_of_birth, moon_longitude)",
      desc: "Compute the full Vimshottari dasha tree from the Moon's natal longitude. No planetary data file required — the calculation is purely algorithmic.",
    },
    {
      fn: "get_nakshatra",
      sig: "(moon_longitude)",
      desc: "Return the nakshatra and pada for any ecliptic longitude. Instant — divides the zodiac geometrically.",
    },
    {
      fn: "compute_houses",
      sig: "(sidereal_time, latitude, system)",
      desc: "Compute all 12 house cusps for any house system using the local sidereal time and geographic latitude. Does not need planetary positions.",
    },
    {
      fn: "find_aspects",
      sig: "(positions[], orbs?)",
      desc: "Detect all aspects among an array of longitudes you supply. The engine applies orb logic and returns typed Aspect objects.",
    },
    {
      fn: "compute_ayanamsha",
      sig: "(julian_day, system)",
      desc: "Return the ayanamsha value for any of the 44 supported systems at any Julian Day. Fully self-contained calculation.",
    },
    {
      fn: "calendar_to_jd",
      sig: "(year, month, day, hour_ut)",
      desc: "Convert a proleptic Gregorian calendar date and UT hour to a Julian Day number. Zero dependencies.",
    },
  ];

  const needsEphemeris = [
    {
      fn: "compute_chart",
      desc: "Full natal chart with planetary longitudes, latitudes, distances, and speeds. Requires DE440s (17 MB) or DE440 (117 MB) ephemeris file loaded into WASM memory.",
    },
    {
      fn: "search_transits",
      desc: "Exact transit moment search over a date range. Requires iterative planetary position computation — needs ephemeris data.",
    },
    {
      fn: "compute_vargas",
      desc: "Divisional charts require accurate planetary longitudes as input — which in turn require ephemeris data.",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — WASM Browser
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Full computation. <span className="text-[#D4A843]">Zero server.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        The Vedākṣha WASM module runs the same Rust computation engine in any
        modern browser. No backend required. No user data leaves the device.
        Dashas, nakshatras, house cusps, and aspects work instantly — full
        planetary positions require the DE440s ephemeris file loaded alongside
        the module.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        Try it live at{" "}
        <a href="https://vedaksha.net/playground" className="text-[#D4A843] hover:underline font-semibold">
          vedaksha.net/playground
        </a>{" "}
        — no install required.
      </p>

      {/* Installation */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Installation
        </h2>
        <div className="space-y-4">
          {[
            { label: "npm", file: "terminal", cmd: "npm add vedaksha-wasm" },
            { label: "pnpm", file: "terminal", cmd: "pnpm add vedaksha-wasm" },
            { label: "yarn", file: "terminal", cmd: "yarn add vedaksha-wasm" },
          ].map((item) => (
            <div key={item.label}>
              <div className="flex items-center justify-between px-4 py-2 rounded-t-lg border border-b-0 border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">
                  {item.label}
                </span>
                <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">
                  {item.file}
                </span>
              </div>
              <pre className="rounded-b-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-4 py-3 text-sm font-mono text-[var(--color-brand-text-secondary)] overflow-x-auto">
                <code>{item.cmd}</code>
              </pre>
            </div>
          ))}
        </div>
      </div>

      {/* Loading the Module */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Loading the WASM Module
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">chart.js</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`import init, {
  calendar_to_jd,
  compute_dasha,
  get_nakshatra,
  compute_houses,
  find_aspects,
  compute_ayanamsha,
} from "vedaksha-wasm";

// initialise the WASM module once at app start
await init();

// ── functions that need NO ephemeris file ──────────────────────────────

const jd = calendar_to_jd(1990, 6, 15, 10.5);
// → 2448057.9375  (Julian Day)

const nakshatra = get_nakshatra(45.23);
// → { name: "Rohini", pada: 2, lord: "Moon" }

const dasha = compute_dasha(jd, /*moon_longitude=*/ 45.23);
// → full DashaTree from birth to present + 20 years

const ayanamsha = compute_ayanamsha(jd, "Lahiri");
// → 23.6712  (degrees)

const houses = compute_houses(
  /*sidereal_time=*/ 6.42,   // hours
  /*latitude=*/     28.6139, // degrees
  /*system=*/       "Placidus"
);
// → { cusps: [0°, 32.1°, 64.8°, ...], ascendant: 32.1° }

const aspects = find_aspects(
  [{ id: "Sun", longitude: 45.2 }, { id: "Moon", longitude: 135.8 }],
  { major_only: true }
);
// → [{ from: "Sun", to: "Moon", type: "Square", orb: 0.6, applying: false }]`}
            </code>
          </pre>
        </div>
      </div>

      {/* With Ephemeris */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Loading Ephemeris Data (Full Chart Computation)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          For full planetary positions, fetch the DE440s file (17 MB, covers 1550–2650 CE)
          and pass it to the module before calling <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 text-[#D4A843]">compute_chart</code>.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">full-chart.js</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`import init, { load_ephemeris, compute_chart } from "vedaksha-wasm";

await init();

// Fetch DE440s — 17 MB, covers 1550–2650 CE
// Host this from your CDN or use the Vedākṣha CDN endpoint
const ephemResponse = await fetch("/static/de440s.bin");
const ephemData     = await ephemResponse.arrayBuffer();
load_ephemeris(new Uint8Array(ephemData));

// Now compute_chart is available
const jd    = calendar_to_jd(1990, 6, 15, 10.5);
const chart = compute_chart(jd, 28.6139, 77.2090, {
  house_system: "Placidus",
  ayanamsha:    "Lahiri",
});

// chart.planets → all bodies with longitude, speed, nakshatra, dignity
// chart.houses  → all 12 cusps
// chart.yogas   → detected Vedic yogas
// chart.graph   → the full ChartGraph`}
            </code>
          </pre>
        </div>
      </div>

      {/* What works without ephemeris */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-2">
          What Works Without Ephemeris Data
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 max-w-2xl">
          These functions are fully self-contained. They work immediately after
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 mx-1">await init()</code>
          with no additional data loading.
        </p>
        <div className="space-y-3 mb-10">
          {noEphemerisNeeded.map((fn) => (
            <div
              key={fn.fn}
              className="border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              <div className="flex flex-col sm:flex-row sm:items-start gap-1 mb-2">
                <code className="text-sm font-mono font-semibold text-[#D4A843] shrink-0">{fn.fn}</code>
                <code className="text-xs font-mono text-[var(--color-brand-text-muted)] sm:ml-1 sm:mt-0.5">{fn.sig}</code>
              </div>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">{fn.desc}</p>
            </div>
          ))}
        </div>

        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-2">
          What Needs Ephemeris Data
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 max-w-2xl">
          These functions require <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">load_ephemeris()</code> to be called first.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 gap-px overflow-hidden">
          {needsEphemeris.map((fn) => (
            <div key={fn.fn} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <code className="text-sm font-mono font-semibold text-[#D4A843] block mb-1">{fn.fn}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-muted)]">{fn.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Playground callout */}
      <div className="mb-14 rounded-xl border border-[#D4A843]/20 bg-[#D4A843]/5 p-6">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[#D4A843] mb-2">
          Interactive Playground
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl mb-4">
          Try every WASM function in the browser at{" "}
          <strong>vedaksha.net/playground</strong>. No install, no account. The
          playground loads the DE440s ephemeris and exposes a live console where
          you can call any function and inspect the output.
        </p>
        <a
          href="https://vedaksha.net/playground"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          Open playground →
        </a>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration/mcp-integration"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← MCP Integration"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/python-bindings"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"Python Bindings →"}
        </a>
      </div>
    </div>
  );
}
