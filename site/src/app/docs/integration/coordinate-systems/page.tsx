export default function CoordinateSystemsPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Coordinate <span className="text-[#D4A843]">Systems</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Planetary positions begin as raw barycentric vectors from the JPL ephemeris and
        travel through a multi-step transformation pipeline before reaching the ecliptic
        longitudes your chart uses. This page documents each step.
      </p>

      {/* The full pipeline */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          The Coordinate Pipeline
        </h2>
        <div className="space-y-3">
          {[
            {
              step: "1",
              label: "ICRS Barycentric",
              tag: "Source",
              desc: "The International Celestial Reference System is the starting frame. Positions are given as vectors from the Solar System barycenter (center of mass of the whole system) in the J2000.0 epoch.",
            },
            {
              step: "2",
              label: "Geocentric ICRS",
              tag: "Barycentric → Earth",
              desc: "The Earth–Moon barycenter offset is subtracted to shift the origin to Earth's center. This is where apparent positions are measured from for geocentric astrology.",
            },
            {
              step: "3",
              label: "Aberration Correction",
              tag: "IAU 2006",
              desc: "Earth's orbital velocity causes apparent stellar positions to shift by up to ~20 arcseconds (annual aberration). The IAU 2006 stellar aberration model corrects for this, giving the apparent position as seen from Earth's surface.",
            },
            {
              step: "4",
              label: "Precession",
              tag: "IAU 2006A",
              desc: "Earth's rotation axis wobbles slowly over ~26,000 years (precession of the equinoxes). The IAU 2006A model rotates the frame from mean J2000.0 ICRS to the mean equatorial frame of the date being computed.",
            },
            {
              step: "5",
              label: "Nutation",
              tag: "IAU 2000B",
              desc: "Shorter-period oscillations of Earth's axis (nutation) are superimposed on precession. The IAU 2000B model (a fast approximation accurate to ~1 milliarcsecond) adjusts the frame from mean to true equatorial of date.",
            },
            {
              step: "6",
              label: "Ecliptic Longitude & Latitude",
              tag: "Final Output",
              desc: "A final rotation by the true obliquity of the ecliptic transforms the true equatorial coordinates into ecliptic longitude and latitude — the values returned by the Vedākṣha API.",
            },
          ].map((s) => (
            <div key={s.step} className="flex gap-4 border border-[var(--color-brand-border)] rounded-lg p-5">
              <span className="flex items-center justify-center size-7 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-xs font-bold shrink-0 mt-0.5">
                {s.step}
              </span>
              <div className="flex-1">
                <div className="flex items-baseline gap-3 mb-1">
                  <p className="text-sm font-semibold text-[var(--color-brand-text)]">{s.label}</p>
                  <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] text-[var(--color-brand-text-muted)]">
                    {s.tag}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-brand-text-secondary)] leading-relaxed">{s.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Code example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Accessing Intermediate Coordinates
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          By default,{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute</code>
          {" "}returns the fully transformed ecliptic coordinates. If you need intermediate
          frames — for example, to verify a specific transformation step — use the
          lower-level functions directly.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">pipeline.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;
use vedaksha::coords::*;

let jd = calendar_to_jd(2024, 3, 20, 12.0);

// Step 1–2: barycentric → geocentric ICRS vector
let geo_icrs = geocentric_icrs(Body::Mars, jd)?;

// Step 3: apply aberration
let apparent = apply_aberration(geo_icrs, jd)?;

// Step 4–5: apply precession and nutation
let (psi, eps) = nutation(jd)?;           // IAU 2000B
let obliquity   = true_obliquity(jd)?;    // including nutation
let true_equat  = apply_precession_nutation(apparent, jd, psi, eps)?;

// Step 6: rotate to ecliptic
let (lon, lat) = equatorial_to_ecliptic(true_equat, obliquity)?;

println!("λ = {:.6}°", lon);
println!("β = {:.6}°", lat);`}</code>
          </pre>
        </div>
      </div>

      {/* Precession & Nutation detail */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Precession and Nutation Models
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          <div className="bg-[var(--color-brand-bg)] p-6">
            <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">IAU 2006A Precession</p>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              The complete IAU 2006A precession model uses a polynomial expansion to
              rotate coordinates from J2000.0 to the mean equatorial frame of date.
              The dominant period is ~25,772 years. Accuracy degrades beyond ±200,000
              years from J2000.0 — well outside any practical use case.
            </p>
          </div>
          <div className="bg-[var(--color-brand-bg)] p-6">
            <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">IAU 2000B Nutation</p>
            <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
              IAU 2000B is a truncated version of the full 1365-term IAU 2000A model.
              It retains 77 lunisolar terms and achieves accuracy of 1 milliarcsecond
              over a 50-year interval around J2000.0. The dominant nutation period is
              ~18.6 years (the lunar nodal cycle).
            </p>
          </div>
        </div>
      </div>

      {/* Sidereal time */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Sidereal Time
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Sidereal time is Earth&apos;s rotation angle relative to the distant stars rather
          than the Sun. It is required to compute the RAMC (Right Ascension of the
          Midheaven), and from RAMC, the house cusps.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-3 gap-px overflow-hidden mb-5">
          {[
            { label: "GMST", name: "Greenwich Mean Sidereal Time", desc: "Earth's rotation angle measured from the mean (non-nutated) vernal equinox at Greenwich." },
            { label: "GAST", name: "Greenwich Apparent Sidereal Time", desc: "GMST corrected for the equation of the equinoxes (nutation in right ascension). More accurate." },
            { label: "LAST", name: "Local Apparent Sidereal Time", desc: "GAST plus the observer's geographic longitude (in time units). This is what drives the Ascendant and RAMC." },
          ].map((item) => (
            <div key={item.label} className="bg-[var(--color-brand-bg)] p-5">
              <p className="text-sm font-bold text-[#D4A843] mb-0.5">{item.label}</p>
              <p className="text-xs text-[var(--color-brand-text-muted)] mb-2">{item.name}</p>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">sidereal_time.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let jd       = calendar_to_jd(2024, 3, 20, 12.0);
let lon_deg  = 77.2090; // observer's geographic longitude

let gmst  = greenwich_mean_sidereal_time(jd)?;   // hours
let gast  = greenwich_apparent_sidereal_time(jd)?;
let last  = local_apparent_sidereal_time(jd, lon_deg)?;
let ramc  = last * 15.0;                          // degrees

println!("GMST : {:.6} h", gmst);
println!("GAST : {:.6} h", gast);
println!("LAST : {:.6} h", last);
println!("RAMC : {:.4}°", ramc);`}</code>
          </pre>
        </div>
      </div>

      {/* Delta T */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Delta T (ΔT)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4">
          Ephemeris calculations use Terrestrial Time (TT), which runs at a uniform rate.
          Civil time (UT1) is based on Earth&apos;s actual rotation, which is slightly
          irregular. Delta T is the difference between them: <strong className="text-[var(--color-brand-text)]">ΔT = TT − UT1</strong>.
        </p>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedākṣha applies Delta T automatically. When you pass a Julian Day derived from
          a civil date (UTC), the library converts it to TT internally before querying
          the ephemeris. For historical dates, it uses published tables from USNO and the
          International Earth Rotation Service. For future dates, it uses a polynomial
          extrapolation.
        </p>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5 bg-[var(--color-brand-bg-subtle)]">
          <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">Current Value</p>
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            For dates near 2024, ΔT is approximately 69 seconds. For ancient historical
            dates (e.g., 500 BCE), ΔT can exceed several hours and dominates the
            uncertainty in planetary position calculations.
          </p>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/sidereal-zodiac" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Sidereal Zodiac
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/time-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Time Systems →
        </a>
      </div>
    </div>
  );
}
