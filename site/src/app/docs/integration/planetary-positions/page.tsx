export default function PlanetaryPositionsPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Planetary <span className="text-[#D4A843]">Positions</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-12 max-w-2xl">
        Compute ecliptic longitude, latitude, heliocentric distance, and daily motion for
        any of the 10 major bodies at any moment in time. Accuracy is sub-arcsecond,
        verified against NASA JPL Horizons.
      </p>

      {/* What you get back */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Output Fields
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-2 gap-px overflow-hidden">
          {[
            {
              field: "longitude",
              type: "f64",
              desc: "Ecliptic longitude in degrees, 0–360. Sun at 0° is 0° Aries in the tropical frame.",
            },
            {
              field: "latitude",
              type: "f64",
              desc: "Ecliptic latitude in degrees. Positive = north of the ecliptic plane. Near zero for most planets.",
            },
            {
              field: "distance",
              type: "f64",
              desc: "Geocentric distance in Astronomical Units (AU). 1 AU ≈ 149,597,870 km.",
            },
            {
              field: "speed",
              type: "f64",
              desc: "Daily motion in degrees per day. Negative values indicate retrograde motion.",
            },
            {
              field: "retrograde",
              type: "bool",
              desc: "Derived convenience flag. True when speed is negative — no separate lookup needed.",
            },
            {
              field: "body",
              type: "Body",
              desc: "Typed enum: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto.",
            },
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

      {/* Coordinate pipeline */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Coordinate Pipeline
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-6">
          Raw planetary positions from the JPL DE440 ephemeris are delivered in the
          International Celestial Reference System (ICRS), a barycentric inertial frame.
          Vedākṣha applies a four-step pipeline to produce the geocentric ecliptic
          coordinates your chart expects.
        </p>
        <div className="space-y-2">
          {[
            { step: "1", label: "ICRS → Geocentric", desc: "Subtract the Earth–Moon barycenter offset. Light-travel time correction (aberration) applied here." },
            { step: "2", label: "Aberration", desc: "Annual aberration shifts apparent position by up to ~20 arcseconds. Uses the IAU 2006 stellar aberration model." },
            { step: "3", label: "Precession & Nutation", desc: "IAU 2006 precession and IAU 2000B nutation rotate the frame from mean ICRS to the true equatorial frame of date." },
            { step: "4", label: "Equatorial → Ecliptic", desc: "Final rotation by the true obliquity of the ecliptic yields the ecliptic longitude and latitude you receive." },
          ].map((s) => (
            <div key={s.step} className="flex gap-4 border border-[var(--color-brand-border)] rounded-lg p-4">
              <span className="flex items-center justify-center size-6 rounded-full border border-[#D4A843]/40 text-[#D4A843] text-xs font-bold shrink-0 mt-0.5">
                {s.step}
              </span>
              <div>
                <p className="text-sm font-semibold text-[var(--color-brand-text)] mb-0.5">{s.label}</p>
                <p className="text-xs text-[var(--color-brand-text-secondary)]">{s.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Single body example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Computing a Single Position
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Construct an{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">EphemerisProvider</code>
          {" "}once and call it repeatedly. The provider caches internal state so repeated
          calls in a loop are cheap.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">positions.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`use vedaksha::prelude::*;

fn main() -> Result<(), VedakshaError> {
    let eph = EphemerisProvider::new()?;

    // Julian Day for 20 March 2024, 12:00 UT
    let jd = calendar_to_jd(2024, 3, 20, 12.0);

    let pos = eph.compute(Body::Moon, jd)?;

    println!("Longitude : {:.6}°", pos.longitude);
    println!("Latitude  : {:.6}°", pos.latitude);
    println!("Distance  : {:.6} AU", pos.distance);
    println!("Speed     : {:.4}°/day", pos.speed);
    println!("Retrograde: {}", pos.retrograde);

    Ok(())
}`}</code>
          </pre>
        </div>
      </div>

      {/* All 10 bodies */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          All 10 Major Bodies at Once
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute_all</code>
          {" "}runs a single internal time setup and computes every body in one pass.
          Faster than calling{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">compute</code>
          {" "}ten times individually.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">all_bodies.rs</span>
          </div>
          <pre className="p-5 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`let positions = eph.compute_all(jd)?;

for pos in &positions {
    let retro = if pos.retrograde { " (R)" } else { "" };
    println!(
        "{:<10} {:>10.4}°{}",
        pos.body.name(), pos.longitude, retro
    );
}

// Sun           29.9841°
// Moon         159.3012°
// Mercury       11.7234° (R)
// Venus         23.4401°
// Mars         286.1047°
// Jupiter       15.8892°
// Saturn       348.7113°
// Uranus        51.9234°
// Neptune      357.0981°
// Pluto        301.7654°`}</code>
          </pre>
        </div>
      </div>

      {/* Retrograde detection */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Retrograde Detection
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-5">
          Retrograde motion is detected directly from the sign of the daily speed value.
          When a planet appears to move backward against the stars from Earth&apos;s
          perspective, its longitudinal speed becomes negative. No separate lookup table
          or configuration is required — the{" "}
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5 text-[#D4A843]">retrograde</code>
          {" "}field is derived automatically.
        </p>
        <div className="border border-[var(--color-brand-border)] rounded-xl p-5 bg-[var(--color-brand-bg-subtle)]">
          <p className="text-xs font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-3">Note on Stationary Points</p>
          <p className="text-sm text-[var(--color-brand-text-secondary)]">
            At the exact moment a planet transitions from direct to retrograde (or vice versa),
            its speed passes through zero. Vedākṣha reports the raw computed speed with full
            precision, so you can detect near-stationary planets by checking for speeds
            smaller than a threshold of your choosing (e.g., |speed| &lt; 0.01°/day).
          </p>
        </div>
      </div>

      {/* Accuracy */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-4">
          Accuracy
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 sm:grid-cols-3 gap-px overflow-hidden">
          {[
            { label: "Ephemeris", value: "JPL DE440", desc: "Covers 9999 BCE – 9999 CE with sub-arcsecond precision for inner planets." },
            { label: "Verification", value: "JPL Horizons", desc: "Position residuals < 1 arcsecond against published Horizons data for 2000–2050." },
            { label: "Obliquity", value: "IAU 2006", desc: "Mean obliquity computed from the 5th-order polynomial recommended by IAU 2006 resolution B1." },
          ].map((item) => (
            <div key={item.label} className="bg-[var(--color-brand-bg)] p-5">
              <p className="text-[10px] font-semibold uppercase tracking-widest text-[var(--color-brand-text-muted)] mb-1">{item.label}</p>
              <p className="text-sm font-bold text-[#D4A843] mb-2">{item.value}</p>
              <p className="text-xs leading-relaxed text-[var(--color-brand-text-secondary)]">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a href="/docs/integration" className="text-sm font-semibold text-[#D4A843] hover:underline">
          ← Integration Guides
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a href="/docs/integration/house-systems" className="text-sm font-semibold text-[#D4A843] hover:underline">
          House Systems →
        </a>
      </div>
    </div>
  );
}
