import Link from "next/link";

const posts = [
  {
    slug: "clean-room-ephemeris",
    title: "Building a Clean-Room Ephemeris",
    date: "2025-11-14",
    readingTime: "10 min read",
    summary:
      "How we implemented planetary computation from scratch using only published NASA and IAU sources — JPL DE440 SPK kernels, Chebyshev polynomial evaluation, a full ICRS coordinate pipeline, and Delta T handling that actually agrees with published tables.",
    tags: ["Astronomy", "Rust", "JPL DE440"],
  },
  {
    slug: "charts-as-graphs",
    title: "Why Charts Should Be Graphs",
    date: "2025-12-03",
    readingTime: "9 min read",
    summary:
      "Astrological charts are property graphs. They always were — we just represented them as flat structs. Here is the case for 10 node types and 13 edge types, and why Cypher, SurrealQL, JSON-LD, and embedding text emitters make AI agents actually useful.",
    tags: ["Graph", "AI", "Neo4j"],
  },
  {
    slug: "vedic-astrology",
    title: "Vedic Astrology Deserves Better Software",
    date: "2026-01-22",
    readingTime: "11 min read",
    summary:
      "27 nakshatras, 50 yogas, 5 dasha systems, 16 vargas, 44 ayanamshas, 7 languages. Why we made Jyotish a first-class citizen in the type system — not a plugin, not a flag, not a translation table bolted on at the end.",
    tags: ["Jyotish", "Vedic", "Localization"],
  },
];

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

export default function BlogPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HERO ─── */}
      <section className="px-6 pt-24 pb-16 border-b border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
            Blog
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)] mb-6">
            Dispatches from the <span className="text-[#D4A843]">Observatory.</span>
          </h1>
          <p className="text-lg leading-relaxed text-[var(--color-brand-text-secondary)]">
            Technical deep-dives on astronomical computation, Vedic astrology, and AI-native data structures.
          </p>
        </div>
      </section>

      {/* ─── POSTS ─── */}
      <section className="px-6 py-16">
        <div className="max-w-3xl mx-auto">
          <div className="grid grid-cols-1 gap-6">
            {posts.map((post) => (
              <Link
                key={post.slug}
                href={`/blog/${post.slug}`}
                className="group block border border-[var(--color-brand-border)] rounded-xl p-7 bg-[var(--color-brand-bg-subtle)] hover:bg-[var(--color-brand-bg)] transition-colors no-underline"
              >
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-xs text-[var(--color-brand-text-muted)]">
                    {formatDate(post.date)}
                  </span>
                  <span className="text-[var(--color-brand-border)]">·</span>
                  <span className="text-xs text-[var(--color-brand-text-muted)]">
                    {post.readingTime}
                  </span>
                </div>

                <h2 className="text-lg font-semibold text-[var(--color-brand-text)] leading-snug mb-3 group-hover:text-[#D4A843] transition-colors">
                  {post.title}
                </h2>

                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mb-4">
                  {post.summary}
                </p>

                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {post.tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-[11px] font-semibold px-2.5 py-1 rounded-full"
                        style={{ color: "#D4A843", backgroundColor: "rgba(212,168,67,0.1)" }}
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                  <span className="text-xs font-semibold text-[#D4A843] group-hover:underline">
                    Read post →
                  </span>
                </div>
              </Link>
            ))}
          </div>
        </div>
      </section>

    </div>
  );
}
