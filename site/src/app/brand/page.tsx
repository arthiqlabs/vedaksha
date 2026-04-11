import { Logo } from "@/components/brand/Logo";

const palette = [
  { name: "Navy", hex: "#1B3A5C", bg: "#1B3A5C", text: "#FFFFFF" },
  { name: "Celestial Blue", hex: "#2E75B6", bg: "#2E75B6", text: "#FFFFFF" },
  { name: "Sky Blue", hex: "#4A90D9", bg: "#4A90D9", text: "#FFFFFF" },
  { name: "Vedic Gold", hex: "#D4A843", bg: "#D4A843", text: "#111111" },
  { name: "Ink", hex: "#111111", bg: "#111111", text: "#FFFFFF" },
  { name: "Slate", hex: "#666666", bg: "#666666", text: "#FFFFFF" },
  { name: "Silver", hex: "#999999", bg: "#999999", text: "#111111" },
  { name: "Code BG", hex: "#F6F8FA", bg: "#F6F8FA", text: "#111111" },
];

const lightBadgeHtml = `<a href="https://vedaksha.net"
  style="display:inline-flex;align-items:center;gap:6px;
         padding:5px 10px;border:1px solid #E5E5E5;
         border-radius:6px;text-decoration:none;">
  <img src="https://vedaksha.net/logo/logo-favicon.svg"
       width="14" height="14" alt="Vedaksha" />
  <span style="font-size:11px;font-weight:500;color:#999;">
    Powered by Vedākṣha
  </span>
</a>`;

const darkBadgeHtml = `<a href="https://vedaksha.net"
  style="display:inline-flex;align-items:center;gap:6px;
         padding:5px 10px;border:1px solid rgba(255,255,255,0.15);
         border-radius:6px;background:#0B1120;text-decoration:none;">
  <img src="https://vedaksha.net/logo/logo-favicon.svg"
       width="14" height="14" alt="Vedaksha" />
  <span style="font-size:11px;font-weight:500;color:rgba(255,255,255,0.7);">
    Powered by Vedākṣha
  </span>
</a>`;

export default function BrandPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HERO ─── */}
      <section className="px-6 pt-24 pb-16 border-b border-[var(--color-brand-border)]">
        <div className="max-w-3xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
            Brand
          </p>
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1] uppercase text-[var(--color-brand-text)]">
            Design <span className="text-[#D4A843]">System.</span>
          </h1>
        </div>
      </section>

      {/* ─── LOGO VARIANTS ─── */}
      <section className="px-6 py-16 border-b border-[var(--color-brand-border)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-8">
            Logo Variants
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
            {/* Light variant */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-10 bg-white flex flex-col items-center gap-6">
              <Logo size="full" className="size-24" />
              <div className="text-center">
                <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                  Light
                </p>
                <p className="text-[10px] font-mono text-[var(--color-brand-text-muted)] mt-1">
                  variant=&quot;light&quot;
                </p>
              </div>
            </div>
            {/* Dark variant */}
            <div className="rounded-xl p-10 flex flex-col items-center gap-6" style={{ backgroundColor: "#0B1120" }}>
              <Logo size="full" variant="dark" className="size-24" />
              <div className="text-center">
                <p className="text-xs font-semibold uppercase tracking-[0.15em] text-white/50">
                  Dark
                </p>
                <p className="text-[10px] font-mono text-white/40 mt-1">
                  variant=&quot;dark&quot;
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ─── COLOR PALETTE ─── */}
      <section className="px-6 py-16 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-8">
            Color Palette
          </p>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            {palette.map((color) => (
              <div
                key={color.hex}
                className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden"
              >
                <div
                  className="h-20 w-full"
                  style={{ backgroundColor: color.bg }}
                />
                <div className="px-3 py-3 bg-[var(--color-brand-bg)]">
                  <p className="text-xs font-semibold text-[var(--color-brand-text)] leading-tight">
                    {color.name}
                  </p>
                  <p className="text-[11px] font-mono text-[var(--color-brand-text-muted)] mt-0.5">
                    {color.hex}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── TYPOGRAPHY ─── */}
      <section className="px-6 py-16 border-b border-[var(--color-brand-border)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-8">
            Typography
          </p>
          <div className="space-y-4">

            {/* Inter */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]">
              <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-3">
                Inter — UI / Body / Headings
              </p>
              <p className="text-2xl font-semibold text-[var(--color-brand-text)] leading-snug" style={{ fontFamily: "var(--font-inter), sans-serif" }}>
                The nine navagraha orbit through 27 nakshatras.
              </p>
            </div>

            {/* JetBrains Mono */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-code)]">
              <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-3">
                JetBrains Mono — Code
              </p>
              <p className="text-xl font-mono text-[var(--color-brand-text)] leading-relaxed" style={{ fontFamily: "var(--font-jetbrains-mono), monospace" }}>
                vedaksha::compute(jd, lat, lon)
              </p>
            </div>

            {/* Noto Sans Devanagari */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg-subtle)]">
              <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-3">
                Noto Sans Devanagari — Sanskrit
              </p>
              <p className="text-2xl text-[var(--color-brand-text)] leading-relaxed" style={{ fontFamily: "var(--font-noto-sans-devanagari), sans-serif" }}>
                वेदाक्ष — अश्विनी · भरणी · कृत्तिका · रोहिणी
              </p>
            </div>

          </div>
        </div>
      </section>

      {/* ─── POWERED BY BADGE ─── */}
      <section className="px-6 py-16 bg-[var(--color-brand-bg-subtle)]">
        <div className="max-w-5xl mx-auto">
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-8">
            Powered By Badge
          </p>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">

            {/* Light badge */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg)]">
              <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-6">
                Light
              </p>
              <div className="mb-6">
                <a
                  href="https://vedaksha.net"
                  className="inline-flex items-center gap-2 px-3 py-1.5 border border-[var(--color-brand-border)] rounded-md no-underline"
                >
                  <Logo size="favicon" className="size-3.5 text-[var(--color-brand-primary)]" />
                  <span className="text-[11px] font-medium text-[var(--color-brand-text-muted)]">
                    Powered by Vedākṣha
                  </span>
                </a>
              </div>
              <div className="rounded-lg border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">HTML</span>
                </div>
                <pre className="p-4 overflow-x-auto text-[11px] font-mono leading-relaxed text-[var(--color-brand-text-secondary)] bg-[var(--color-brand-bg-code)]">
                  <code>{lightBadgeHtml}</code>
                </pre>
              </div>
            </div>

            {/* Dark badge */}
            <div className="border border-[var(--color-brand-border)] rounded-xl p-8 bg-[var(--color-brand-bg)]">
              <p className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)] mb-6">
                Dark
              </p>
              <div className="mb-6 p-4 rounded-lg" style={{ backgroundColor: "#0B1120" }}>
                <a
                  href="https://vedaksha.net"
                  className="inline-flex items-center gap-2 px-3 py-1.5 rounded-md no-underline"
                  style={{ border: "1px solid rgba(255,255,255,0.15)", backgroundColor: "#0B1120" }}
                >
                  <Logo size="favicon" variant="dark" className="size-3.5" />
                  <span className="text-[11px] font-medium" style={{ color: "rgba(255,255,255,0.7)" }}>
                    Powered by Vedākṣha
                  </span>
                </a>
              </div>
              <div className="rounded-lg border border-[var(--color-brand-border)] overflow-hidden">
                <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">HTML</span>
                </div>
                <pre className="p-4 overflow-x-auto text-[11px] font-mono leading-relaxed text-[var(--color-brand-text-secondary)] bg-[var(--color-brand-bg-code)]">
                  <code>{darkBadgeHtml}</code>
                </pre>
              </div>
            </div>

          </div>
        </div>
      </section>

    </div>
  );
}
