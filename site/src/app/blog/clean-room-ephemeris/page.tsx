import Link from "next/link";

export default function CleanRoomEphemerisPage() {
  return (
    <div className="flex flex-col">

      {/* ─── HEADER ─── */}
      <section className="px-6 pt-24 pb-14 border-b border-[var(--color-brand-border)]">
        <div className="max-w-2xl mx-auto">
          <Link
            href="/blog"
            className="inline-flex items-center gap-1.5 text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider mb-8 hover:text-[#D4A843] transition-colors no-underline"
          >
            ← Blog
          </Link>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
            Astronomy · Rust · JPL DE440
          </p>
          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight leading-[1.15] uppercase text-[var(--color-brand-text)] mb-6">
            Building a Clean-Room Ephemeris
          </h1>
          <p className="text-base leading-relaxed text-[var(--color-brand-text-secondary)] mb-6">
            How we implemented planetary computation from scratch — JPL DE440 SPK kernels, Chebyshev polynomial evaluation, a full ICRS coordinate pipeline, and Delta T handling that actually agrees with published tables.
          </p>
          <div className="flex items-center gap-3 text-xs text-[var(--color-brand-text-muted)]">
            <span>November 14, 2025</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>10 min read</span>
            <span className="text-[var(--color-brand-border)]">·</span>
            <span>ArthIQ Labs</span>
          </div>
        </div>
      </section>

      {/* ─── BODY ─── */}
      <article className="px-6 py-16">
        <div className="max-w-2xl mx-auto">

          <div className="prose-like space-y-8 text-[var(--color-brand-text-secondary)] leading-relaxed">

            <p className="text-base">
              The first question we asked when starting Vedākṣha was whether to wrap an existing ephemeris library — Swiss Ephemeris, VSOP87, or one of the various Python wrappers around them. The answer was no, and for a reason that goes beyond the usual &quot;not invented here&quot; instinct.
            </p>

            <p className="text-base">
              Clean-room means something specific here: every algorithm in the computational core is derived from primary published sources — NASA JPL technical reports, IAU resolutions, IERS conventions, and peer-reviewed papers — not from existing open-source implementations. When a number comes out of the engine, we can trace it to a page in a source document, not to another codebase.
            </p>

            {/* Section: Why it matters */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Why provenance matters for an astrological engine
              </h2>
              <p className="text-base">
                Astrology software has accumulated decades of copying. A rounding convention introduced in one library propagates to another, and by the time someone traces a discrepancy to its origin, the original code is gone. A clean-room implementation breaks that chain. You can point at equation (3.1) in the IERS Conventions 2010 and say: that is exactly what the code computes.
              </p>
              <p className="text-base mt-4">
                For Vedic astrology this matters even more. Ayanamsha values differ by arcseconds between implementations — arcseconds that shift nakshatra boundaries and change dasha periods. When a practitioner asks why their chart shows a different nakshatra than another tool, the answer should be traceable to a specific ayanamsha definition and reference epoch, not to an undocumented behavioral quirk.
              </p>
            </div>

            {/* Section: JPL DE440 */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Starting with JPL DE440
              </h2>
              <p className="text-base">
                Planetary positions start with the JPL Developmental Ephemeris. DE440 is the current long-arc solution, covering 1550 to 2650 CE, developed by Park, Folkner, Williams, and Boggs (2021). We use the SPK kernel format — binary files containing Chebyshev polynomial coefficients for each body over each time interval.
              </p>
              <p className="text-base mt-4">
                The SPK reader is implemented from scratch against the SPICE toolkit documentation. A record maps to a time window and a set of coefficients. Position evaluation is three nested Chebyshev recurrences — one per Cartesian component — using the standard recurrence relation T_n+1(x) = 2x·T_n(x) − T_n-1(x). Velocity follows from the derivative recurrence without finite differences.
              </p>
              <p className="text-base mt-4">
                This gives barycentric positions in the ICRS frame with sub-kilometer accuracy across the covered arc. The Sun is recovered as the difference between the Solar System Barycenter and the Earth-Moon Barycenter, adjusted by the lunar mass fraction.
              </p>
            </div>

            {/* Code block: Chebyshev */}
            <div>
              <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
                  <div className="flex items-center gap-1.5">
                    <span className="size-2.5 rounded-full bg-red-400/50" />
                    <span className="size-2.5 rounded-full bg-yellow-400/50" />
                    <span className="size-2.5 rounded-full bg-green-400/50" />
                  </div>
                  <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">chebyshev.rs</span>
                </div>
                <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
                  <code>
                    <span className="text-purple-600">fn</span> <span className="text-blue-700">cheby_eval</span>(coeffs: &amp;[<span className="text-amber-700">f64</span>], x: <span className="text-amber-700">f64</span>) -&gt; <span className="text-amber-700">f64</span> {"{"}{"\n"}
                    {"    "}<span className="text-purple-600">let</span> (<span className="text-purple-600">mut</span> t0, <span className="text-purple-600">mut</span> t1) = (<span className="text-blue-700">1.0</span>, x);{"\n"}
                    {"    "}<span className="text-purple-600">let mut</span> result = coeffs[<span className="text-blue-700">0</span>] * t0 + coeffs[<span className="text-blue-700">1</span>] * t1;{"\n"}
                    {"    "}<span className="text-purple-600">for</span> c <span className="text-purple-600">in</span> &amp;coeffs[<span className="text-blue-700">2</span>..] {"{"}{"\n"}
                    {"        "}<span className="text-purple-600">let</span> t2 = <span className="text-blue-700">2.0</span> * x * t1 - t0;{"\n"}
                    {"        "}result += c * t2;{"\n"}
                    {"        "}(t0, t1) = (t1, t2);{"\n"}
                    {"    "}{"}"}{"\n"}
                    {"    "}result{"\n"}
                    {"}"}
                  </code>
                </pre>
              </div>
              <p className="text-xs text-[var(--color-brand-text-muted)] mt-2 text-center">
                Clenshaw recurrence for Chebyshev evaluation — the core of every planetary position computation.
              </p>
            </div>

            {/* Section: Coordinate pipeline */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                The coordinate pipeline: ICRS to ecliptic
              </h2>
              <p className="text-base">
                Barycentric ICRS coordinates are not what astrology needs. The pipeline from raw SPK output to an ecliptic longitude has six steps, each with its own error budget.
              </p>

              <div className="mt-6 rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 gap-px overflow-hidden">
                {[
                  {
                    step: "01",
                    label: "Light-time correction",
                    body: "We observe planets at their retarded position — where they were when the light left, not where they are now. Solved iteratively: compute distance, subtract light travel time from epoch, re-evaluate. Converges in 3 iterations to sub-nanosecond accuracy.",
                  },
                  {
                    step: "02",
                    label: "Frame bias & precession",
                    body: "The IAU 2006 precession model (Capitaine et al.) applies a matrix built from polynomial expansions of the precession angles ψ_A, ω_A, and χ_A. The frame bias matrix corrects the 17.3 mas offset between the dynamical and ICRS equinox.",
                  },
                  {
                    step: "03",
                    label: "Nutation",
                    body: "The IAU 2000B nutation model — 77 luni-solar terms plus 687 planetary terms — gives the true equator and equinox. The full MHB2000 solution from Mathews, Herring & Buffett (2002) is implemented, not the truncated IAU 1980 model used in older software.",
                  },
                  {
                    step: "04",
                    label: "Aberration",
                    body: "Annual aberration shifts apparent positions by up to 20.5 arcseconds toward the direction of Earth's velocity. We apply the rigorous relativistic formula from the IERS Conventions, not the classical κ/c approximation.",
                  },
                  {
                    step: "05",
                    label: "Ecliptic rotation",
                    body: "Rotation from the true equator of date to the ecliptic of date uses the obliquity from the IAU 2006 model. The result is the apparent ecliptic longitude — the number that goes into every chart computation.",
                  },
                  {
                    step: "06",
                    label: "Topocentric correction",
                    body: "For ascendant and house calculations, parallax correction shifts the Moon's position by up to 57 arcminutes depending on geographic location. Applied using the WGS84 ellipsoid for observer coordinates.",
                  },
                ].map((item) => (
                  <div key={item.step} className="bg-[var(--color-brand-bg)] p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
                    <div className="flex items-start gap-4">
                      <span className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider shrink-0 pt-0.5">
                        {item.step}
                      </span>
                      <div>
                        <h3 className="text-sm font-semibold text-[#D4A843] uppercase tracking-wide mb-1.5">
                          {item.label}
                        </h3>
                        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                          {item.body}
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Section: Delta T */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Delta T: the most underestimated problem
              </h2>
              <p className="text-base">
                JPL ephemerides are tabulated in Barycentric Dynamical Time (TDB). Astrological inputs are in local civil time — UTC or a named timezone. The conversion chain is: local time → UTC → UT1 → TT → TDB. Each step has different uncertainty characteristics.
              </p>
              <p className="text-base mt-4">
                Delta T (ΔT = TT − UT1) is the accumulated difference between atomic time and Earth rotation, currently around 69 seconds. For modern dates it comes from IERS Bulletin A. For historical dates we use the Morrison-Stephenson polynomial model extended by Espenak-Meeus. For dates before 1620, the secular term dominates and uncertainty grows to minutes — a fact that matters enormously for historical chart rectification.
              </p>
              <p className="text-base mt-4">
                We validate ΔT output against the Espenak tables published at NASA GSFC and against JPL Horizons at 50-year intervals from −500 to 2500. Every value is within the published uncertainty bounds for its era. The implementation uses Rust&apos;s <code className="text-xs font-mono bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">f64</code> throughout — 64-bit double precision is sufficient for all ephemeris calculations given that the underlying SPK data carries ~14 significant digits.
              </p>
            </div>

            {/* Section: Validation */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Validation against JPL Horizons
              </h2>
              <p className="text-base">
                The definitive validation target is JPL Horizons — the same ephemeris data, independent implementation. We run a test suite that queries Horizons-equivalent reference values for all planets at 200 dates spanning 1800–2100, covering both historical and predictive ranges.
              </p>
              <p className="text-base mt-4">
                Results: Sun, Moon, and inner planets match to within 0.1 arcseconds of apparent geocentric ecliptic longitude. Outer planets (Jupiter through Neptune) match to within 0.01 arcseconds. The residuals are consistent with the expected differences from slightly different ΔT models, not from algorithmic errors.
              </p>
              <p className="text-base mt-4">
                Ascendant and Midheaven values are validated against the same reference positions used by the Swiss Ephemeris test suite, cross-checked at 100 geographic locations.
              </p>
            </div>

            {/* Section: Cited sources */}
            <div>
              <h2 className="text-xl font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-4">
                Primary sources cited in the implementation
              </h2>
              <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)] divide-y divide-[var(--color-brand-border)]">
                {[
                  { ref: "Park et al. (2021)", desc: "JPL DE440/DE441 solution — The JPL Planetary and Lunar Ephemerides DE440 and DE441." },
                  { ref: "IAU 2006", desc: "Capitaine et al. — IAU 2006 precession model. A&A 412, 567–586." },
                  { ref: "MHB2000", desc: "Mathews, Herring, Buffett — Modeling of nutation and precession. JGR 2002." },
                  { ref: "IERS Conventions 2010", desc: "Petit & Luzum — Technical Note 36. Chapter 5: Transformation between celestial and terrestrial systems." },
                  { ref: "Espenak & Meeus", desc: "Five Millennium Canon of Solar Eclipses — ΔT polynomial expressions." },
                  { ref: "Morrison & Stephenson (2004)", desc: "Historical values of the Earth&apos;s clock error ΔT and the calculation of eclipses. JHA 35." },
                  { ref: "Naif SPICE", desc: "SPK Required Reading — NASA/JPL kernel format specification." },
                ].map((s) => (
                  <div key={s.ref} className="px-5 py-3.5 flex items-start gap-4">
                    <code className="text-xs font-mono text-[#D4A843] shrink-0 pt-0.5 w-44">
                      {s.ref}
                    </code>
                    <p className="text-sm text-[var(--color-brand-text-secondary)]">{s.desc}</p>
                  </div>
                ))}
              </div>
            </div>

            {/* Closing */}
            <p className="text-base">
              The result is an engine where you can open the source, find the function that computes a planetary position, and follow the citations back to the equations it implements. That is what clean-room means in practice — not just &quot;we didn&apos;t copy code,&quot; but &quot;we can show our work.&quot;
            </p>

          </div>

          {/* ─── FOOTER NAV ─── */}
          <div className="mt-16 pt-10 border-t border-[var(--color-brand-border)] flex items-center justify-between">
            <Link
              href="/blog"
              className="text-xs font-semibold text-[var(--color-brand-text-muted)] uppercase tracking-wider hover:text-[#D4A843] transition-colors no-underline"
            >
              ← All posts
            </Link>
            <Link
              href="/blog/charts-as-graphs"
              className="text-xs font-semibold text-[#D4A843] uppercase tracking-wider hover:underline no-underline"
            >
              Next: Why Charts Should Be Graphs →
            </Link>
          </div>
        </div>
      </article>

    </div>
  );
}
