"use client";

import { useState } from "react";

const faqs = [
  {
    q: "Do I need to download any data files?",
    a: "It depends on what you want to compute. Dashas, nakshatras, house cusps, aspects, and ayanamsha values are all fully self-contained — they work immediately after installing the package with no additional files. Full planetary positions (longitudes, latitudes, distances, speeds) require an ephemeris file. The Python wheel and the WASM package ship with the DE440s embedded ephemeris covering 1550–2650 CE (17 MB). For dates outside that range you need DE441, which you download once from the JPL website and point the library at.",
  },
  {
    q: "How accurate is it?",
    a: "Sub-arcsecond for all major solar system bodies over the DE440s coverage window (1550–2650 CE). Vedākṣha&apos;s planetary positions agree with JPL Horizons to within 1 arcsecond for the Sun, Moon, and planets Mercury through Neptune. This is the same precision used by professional astronomical software. The validation suite compares against JPL Horizons for 10 000 random dates and reports any deviation greater than 0.001 degrees.",
  },
  {
    q: "Can I use it commercially?",
    a: "Yes. Vedākṣha is available under a Business Source License (BSL). Commercial use requires a one-time license fee of USD 500. This grants perpetual rights to use the library in production commercial applications. The BSL converts to an open-source license (MIT) after 4 years from each version&apos;s release date. Non-commercial use, research, and personal projects are free under the BSL without a commercial license. Contact info@arthiq.net for invoicing.",
  },
  {
    q: "Does it support Vedic astrology?",
    a: "Yes — Vedic is first-class, not a plugin. Vedākṣha computes: 27 nakshatras and 108 padas for every planet; complete Vimshottari, Yogini, and Chara dasha trees to 5 sub-period levels; all 16 Shodasha Varga divisional charts (D-1 through D-60); 50 classical Vedic yogas sourced from BPHS and Phaladipika with strength scores; complete Shadbala (all 6 components) and Bhava Bala; Vedic Drishti with special aspects for Mars, Jupiter, and Saturn; 44 ayanamsha systems including Lahiri, Krishnamurti, Raman, and Yukteshwar; and 7-language localization for all names.",
  },
  {
    q: "Can I use it from Python, JavaScript, or WASM?",
    a: "Yes to all three. The Rust library is the primary implementation. Python bindings are available via PyO3 — install with pip install vedaksha. The WASM module runs in any modern browser and in Node.js — install with npm add vedaksha-wasm. All three expose the same functions with the same semantics. Type stubs are provided for Python IDE support.",
  },
  {
    q: "How does the graph output work?",
    a: "Every chart computation also produces a ChartGraph — a typed property graph with 10 node types (Chart, Planet, Sign, House, Nakshatra, Pada, Pattern, DashaPeriod, Yoga, FixedStar) and 13 edge types. Every node has a deterministic ID derived from its content, so the same input always produces the same graph. You can emit the graph as Cypher for Neo4j, SurrealQL for SurrealDB, JSON-LD for linked data / SPARQL, plain JSON, or RAG-optimised embedding text for vector stores. Graph construction is zero-cost — it happens during the same pass that computes the chart.",
  },
  {
    q: "Is it thread-safe?",
    a: "Yes, completely. Every public function is a pure function — it takes its full input as parameters, returns a value, and touches no shared mutable state. There are no globals, no singletons, no internal caches that require locking. The Rust type system enforces this at compile time. You can call any function from any thread at any time. For batch workloads, add rayon and replace .iter() with .par_iter() — that is the entire change needed to use all CPU cores.",
  },
  {
    q: "What&apos;s the difference between embedded and file-based ephemeris?",
    a: "The embedded ephemeris (DE440s, 17 MB) is bundled inside the Python wheel and the WASM module. It covers dates from 1550 to 2650 CE. It is loaded automatically — no configuration required. The file-based ephemeris (DE440, 117 MB; or DE441, ~3 GB) must be downloaded separately and configured with a file path or loaded into memory. DE440 covers the same date range as DE440s but includes more bodies (asteroids, Pluto&apos;s moons). DE441 extends coverage to approximately 13000 BCE and 17000 CE for historical and speculative future calculations.",
  },
  {
    q: "What ayanamsha systems are supported?",
    a: "44 systems are supported. This includes: Lahiri (Indian national standard), Fagan-Bradley, Krishnamurti (KP), Raman, Yukteshwar, Aryabhata, Djwhal Khul, galactic center at 0° Sagittarius (Mardyks), galactic center at 0° Capricorn, Aldebaran at 15° Taurus (Hipparchos), Revati at 0° Aries (Usha-Shashi, Sassanian), and 33 further systems including equatorial and precession-corrected variants. Pass the ayanamsha name as a string or use the typed enum.",
  },
  {
    q: "Where can I get help?",
    a: "For bug reports and feature requests, open an issue on the GitHub repository. For commercial licensing, integration consulting, and support contracts, email info@arthiq.net. For quick questions, the GitHub Discussions section is monitored. Response time for email is typically one business day.",
  },
];

function FaqItem({ item, isOpen, onToggle }: { item: typeof faqs[0]; isOpen: boolean; onToggle: () => void }) {
  return (
    <div className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden">
      <button
        onClick={onToggle}
        className="w-full text-left px-6 py-5 flex items-start justify-between gap-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
      >
        <span className="text-sm font-semibold text-[var(--color-brand-text)] leading-snug">
          {item.q}
        </span>
        <span
          className="text-[#D4A843] text-lg leading-none shrink-0 mt-0.5 transition-transform duration-200"
          style={{ transform: isOpen ? "rotate(45deg)" : "rotate(0deg)" }}
        >
          +
        </span>
      </button>
      {isOpen && (
        <div className="px-6 pb-6 border-t border-[var(--color-brand-border)] pt-4">
          <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
            {item.a}
          </p>
        </div>
      )}
    </div>
  );
}

export default function FaqPage() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — FAQ
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Frequently asked <span className="text-[#D4A843]">questions.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Common questions about accuracy, licensing, platform support, Vedic
        capabilities, and how to get help.
      </p>

      <div className="space-y-3">
        {faqs.map((item, i) => (
          <FaqItem
            key={i}
            item={item}
            isOpen={openIndex === i}
            onToggle={() => setOpenIndex(openIndex === i ? null : i)}
          />
        ))}
      </div>

      <div className="mt-14 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Still have questions?
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-xl mb-4">
          Reach us at{" "}
          <a href="mailto:info@arthiq.net" className="text-[#D4A843] hover:underline font-semibold">
            info@arthiq.net
          </a>{" "}
          for licensing, integration consulting, and support. For bugs and feature
          requests, open an issue on GitHub.
        </p>
        <div className="flex flex-wrap gap-3">
          <a
            href="mailto:info@arthiq.net"
            className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
          >
            Email us →
          </a>
          <span className="text-[var(--color-brand-text-muted)]">·</span>
          <a
            href="https://github.com/arthiq/vedaksha/issues"
            className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
          >
            GitHub Issues →
          </a>
        </div>
      </div>

      <div className="mt-12 flex items-center gap-6">
        <a
          href="/docs/integration/data-sources"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Data Sources"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"Integration Index →"}
        </a>
      </div>
    </div>
  );
}
