export default function TimeSystemsPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Time <span className="text-[#D4A843]">Systems</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        All Vedākṣha computations use Julian Day numbers as their primary time input.
        This page covers what Julian Days are, how to convert to and from calendar dates,
        and the relationships between the various time scales used internally.
      </p>

      {/* Julian Day */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Julian Day Numbers
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4">
          A Julian Day (JD) number is a continuous count of days since noon on 1 January
          4713 BCE in the proleptic Julian calendar (approximately 6 November 4714 BCE
          in the Gregorian calendar). The fractional part of the JD represents the time
          of day, where 0.0 = noon and 0.5 = midnight.
        </p>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Using a single monotonically increasing number eliminates ambiguities around
          calendar reforms, year-zero conventions, and timezone offsets. Every celestial
          event throughout history has a unique, unambiguous JD.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { label: "J2000.0", jd: "2451545.0", desc: "1 January 2000, 12:00 TT. The standard astronomical reference epoch." },
            { label: "20 Mar 2024, noon", jd: "2460389.0", desc: "2024 vernal equinox. All examples in this guide use this date." },
            { label: "1 Jan 1900, noon", jd: "2415021.0", desc: "Common epoch for older astronomical tables and algorithms." },
          ].map((item) => (
            <div key={item.label} className="bg-[var(--color-brand-bg)] p-5">
              <p className="text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] mb-1">{item.label}</p>
              <code className="text-sm font-mono text-[#D4A843] block mb-2">JD {item.jd}</code>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Calendar conversions */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Calendar Conversions
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Vedākṣha provides two functions for converting between Gregorian calendar
          dates and Julian Day numbers. Both use the proleptic Gregorian calendar for
          dates before 15 October 1582.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-4">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">calendar.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

// calendar_to_jd(year, month, day, hour_ut)
// hour_ut is decimal hours in Universal Time
let jd = calendar_to_jd(2024, 3, 20, 6.5);
// 6.5 = 06:30 UT
// → 2460388.771

// jd_to_calendar(jd) → CalendarDate
let date = jd_to_calendar(jd);
println!("{}-{:02}-{:02} {:05.2}h UT",
    date.year, date.month, date.day, date.hour_ut);
// → 2024-03-20 06.50h UT

// Handling local time with UTC offset
let local_hour = 12.0_f64;  // noon local
let utc_offset = 5.5_f64;   // IST = UTC+5:30
let ut = local_hour - utc_offset;
let jd_local = calendar_to_jd(2024, 3, 20, ut);`}</code>
          </pre>
        </div>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5 bg-[var(--color-brand-bg-subtle)]">
          <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">Time Zone Responsibility</p>
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            Vedākṣha works in Universal Time (UT). Converting a local birth time to UT
            before calling{" "}
            <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">calendar_to_jd</code>
            {" "}is the caller&apos;s responsibility. Subtracting the UTC offset from the local
            hour (as shown above) is the standard approach.
          </p>
        </div>
      </div>

      {/* Time scales */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Time Scale Relationships
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Several time scales are used within an ephemeris computation. Understanding
          their relationships helps when debugging precision issues or integrating with
          external data sources.
        </p>
        <div className="space-y-3">
          {[
            {
              label: "UTC — Coordinated Universal Time",
              desc: "Civil timekeeping standard. Stays within 0.9 seconds of UT1 by inserting leap seconds. What your system clock returns.",
            },
            {
              label: "UT1 — Universal Time 1",
              desc: "Earth's rotation angle relative to the Sun. Slightly irregular due to tidal braking and geophysical processes. Used to compute GMST (sidereal time).",
            },
            {
              label: "TT — Terrestrial Time",
              desc: "Uniform atomic time scale used for ephemeris calculations. Currently runs about 69 seconds ahead of UT1 (ΔT = TT − UT1 ≈ 69 s in 2024).",
            },
            {
              label: "TDB — Barycentric Dynamical Time",
              desc: "Like TT, but accounts for relativistic time dilation as Earth moves around the Sun. Differs from TT by at most ±1.7 ms. The JPL DE440 ephemeris is referenced to TDB.",
            },
          ].map((ts) => (
            <div key={ts.label} className="border border-[var(--color-brand-border)] rounded-lg p-4">
              <p className="text-sm font-semibold text-[var(--color-brand-text)] mb-1">{ts.label}</p>
              <p className="text-sm text-[var(--color-brand-text-secondary)]">{ts.desc}</p>
            </div>
          ))}
        </div>
        <div className="mt-4 border border-[#D4A843]/20 bg-[#D4A843]/5 rounded-xl p-5">
          <p className="text-xs font-semibold uppercase tracking-wider text-[#D4A843] mb-2">What Vedākṣha Does For You</p>
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            When you pass a Julian Day derived from a civil time to{" "}
            <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute</code>
            , Vedākṣha automatically applies ΔT to convert it to TT (and then TDB for
            the ephemeris query). You do not need to add ΔT manually. If you need to
            supply a TT-based Julian Day directly, use{" "}
            <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute_tt</code>
            {" "}instead.
          </p>
        </div>
      </div>

      {/* Delta T */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Delta T (ΔT)
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          ΔT is the accumulated difference between TT and UT1. It is always positive and
          grows over time because Earth&apos;s rotation is slowly decelerating due to tidal
          friction from the Moon. Vedākṣha uses published USNO and IERS tables for
          historical dates and a polynomial extrapolation for future dates.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden mb-5">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">delta_t.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`// Query ΔT for any Julian Day
let jd  = calendar_to_jd(2024, 3, 20, 12.0);
let dt  = delta_t(jd)?;
println!("ΔT = {:.2} seconds", dt);  // ≈ 69.18 s

// Convert UT Julian Day to TT Julian Day manually
let jd_tt = ut_to_tt(jd)?;

// Or override ΔT if you have a more precise value
let custom = DeltaTConfig::fixed(69.22);
let jd_tt2 = ut_to_tt_with_config(jd, &custom)?;`}</code>
          </pre>
        </div>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { era: "500 BCE", dt: "~17,190 s", note: "Dominates ancient chart uncertainty." },
            { era: "1900 CE", dt: "~3 s", note: "Well-known from historical records." },
            { era: "2024 CE", dt: "~69 s", note: "Published monthly by IERS." },
          ].map((row) => (
            <div key={row.era} className="bg-[var(--color-brand-bg)] p-5">
              <p className="text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] mb-1">{row.era}</p>
              <code className="text-sm font-mono text-[#D4A843] block mb-1">ΔT ≈ {row.dt}</code>
              <p className="text-xs text-[var(--color-brand-text-secondary)]">{row.note}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Sidereal time */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Sidereal Time Functions
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">sidereal.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let jd      = calendar_to_jd(2024, 3, 20, 12.0);
let geo_lon = 77.2090; // New Delhi, degrees east

// Greenwich Mean Sidereal Time (hours)
let gmst = greenwich_mean_sidereal_time(jd)?;

// Greenwich Apparent Sidereal Time (hours, includes nutation)
let gast = greenwich_apparent_sidereal_time(jd)?;

// Local Apparent Sidereal Time (hours)
let last = local_apparent_sidereal_time(jd, geo_lon)?;

// RAMC in degrees (used for house computation)
let ramc = last * 15.0;

println!("GMST : {:.6} h", gmst);
println!("GAST : {:.6} h", gast);
println!("LAST : {:.6} h", last);
println!("RAMC : {:.4}°", ramc);`}</code>
          </pre>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration/coordinate-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Coordinate Systems
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/aspects-patterns" className="text-sm font-semibold text-[#D4A843] hover:underline">
          Aspects &amp; Patterns →
        </a>
      </div>
    </div>
  );
}
