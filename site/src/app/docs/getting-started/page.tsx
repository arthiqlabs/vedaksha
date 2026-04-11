export default function GettingStartedPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Getting Started
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Up and running in <span className="text-[#D4A843]">5 minutes.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Vedākṣha ships as native Rust, a Python wheel via PyO3, and a WebAssembly
        package for browsers and edge runtimes. Pick your platform and follow the
        steps below.
      </p>

      {/* Step 1 — Install */}
      <div className="mb-14">
        <div className="flex items-center gap-4 mb-5">
          <span className="flex items-center justify-center size-8 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-sm font-bold shrink-0">
            1
          </span>
          <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)]">
            Install
          </h2>
        </div>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 ml-12">
          Add Vedākṣha to your project using the package manager for your platform.
        </p>
        <div className="ml-12 space-y-4">
          {[
            { label: "Rust", file: "Cargo.toml", cmd: "cargo add vedaksha" },
            { label: "Python", file: "terminal", cmd: "pip install vedaksha" },
            { label: "WASM", file: "package.json", cmd: "npm add vedaksha-wasm" },
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

      {/* Step 2 — Compute a chart */}
      <div className="mb-14">
        <div className="flex items-center gap-4 mb-5">
          <span className="flex items-center justify-center size-8 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-sm font-bold shrink-0">
            2
          </span>
          <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)]">
            Compute a Chart
          </h2>
        </div>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 ml-12">
          Pass a Julian Day and coordinates. One function call returns a complete natal chart.
        </p>
        <div className="ml-12">
          <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
              <div className="flex items-center gap-1.5">
                <span className="size-2.5 rounded-full bg-red-400/50" />
                <span className="size-2.5 rounded-full bg-yellow-400/50" />
                <span className="size-2.5 rounded-full bg-green-400/50" />
              </div>
              <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">main.rs</span>
            </div>
            <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
              <code>
                <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::prelude::*;{"\n"}
                {"\n"}
                <span className="text-purple-600">let</span> jd = <span className="text-blue-700">calendar_to_jd</span>(<span className="text-blue-700">2024</span>, <span className="text-blue-700">3</span>, <span className="text-blue-700">20</span>, <span className="text-blue-700">12.0</span>);{"\n"}
                <span className="text-purple-600">let</span> chart = <span className="text-blue-700">compute_chart</span>({"\n"}
                {"  "}jd, <span className="text-blue-700">28.6139</span>, <span className="text-blue-700">77.2090</span>,{"\n"}
                {"  "}&amp;<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>(){"\n"}
                );{"\n"}
                {"\n"}
                <span className="text-purple-600">for</span> planet <span className="text-purple-600">in</span> &amp;chart.planets {"{"}{"\n"}
                {"  "}<span className="text-blue-700">println!</span>(<span className="text-green-700">&quot;{"{}"}: {}° {"{}"}&quot;</span>,{"\n"}
                {"    "}planet.name, planet.longitude,{"\n"}
                {"    "}planet.sign);{"\n"}
                {"}"}
              </code>
            </pre>
          </div>
        </div>
      </div>

      {/* Step 3 — Read the output */}
      <div className="mb-14">
        <div className="flex items-center gap-4 mb-5">
          <span className="flex items-center justify-center size-8 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-sm font-bold shrink-0">
            3
          </span>
          <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)]">
            Read the Output
          </h2>
        </div>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5 ml-12">
          Every planet in <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">chart.planets</code> carries a full set of computed fields. No post-processing required.
        </p>
        <div className="ml-12">
          <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px overflow-hidden">
            {[
              { field: "longitude", desc: "Ecliptic longitude in degrees (0–360). Sub-arcsecond precision from JPL DE440." },
              { field: "sign", desc: "Zodiac sign as a typed enum — Sign::Aries, not integer 0." },
              { field: "house", desc: "Whole-number house placement (1–12) in the selected house system." },
              { field: "speed", desc: "Daily motion in degrees. Negative values indicate retrograde motion." },
              { field: "retrograde", desc: "Boolean flag. True when daily speed is negative." },
              { field: "nakshatra", desc: "One of 27 nakshatras with pada (quarter, 1–4) for Vedic charts." },
            ].map((item) => (
              <div key={item.field} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                <code className="text-xs font-mono text-[#D4A843] block mb-2">
                  .{item.field}
                </code>
                <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">
                  {item.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Step 4 — Next steps */}
      <div className="mb-6">
        <div className="flex items-center gap-4 mb-5">
          <span className="flex items-center justify-center size-8 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-sm font-bold shrink-0">
            4
          </span>
          <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)]">
            Next Steps
          </h2>
        </div>
        <div className="ml-12 grid grid-cols-1 sm:grid-cols-3 gap-4">
          {[
            {
              href: "/docs",
              label: "Full Documentation",
              desc: "House systems, coordinate pipeline, Shadbala, ayanamshas, and every API surface.",
              cta: "Browse docs →",
            },
            {
              href: "/ai",
              label: "AI Integration",
              desc: "Connect Vedākṣha to Claude, GPT, or any MCP-compatible agent in minutes.",
              cta: "AI overview →",
            },
            {
              href: "/docs/integration",
              label: "Integration Guides",
              desc: "Step-by-step guides for every workflow, from natal charts to transit searches.",
              cta: "View guides →",
            },
          ].map((card) => (
            <a
              key={card.href}
              href={card.href}
              className="group block border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors no-underline"
            >
              <h3 className="text-sm font-semibold text-[var(--color-brand-text)] uppercase tracking-wide mb-2">
                {card.label}
              </h3>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-muted)] mb-3">
                {card.desc}
              </p>
              <span className="text-xs font-semibold text-[#D4A843] group-hover:underline">
                {card.cta}
              </span>
            </a>
          ))}
        </div>
      </div>
    </div>
  );
}
