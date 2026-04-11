export default function PythonBindingsPage() {
  const functions = [
    {
      name: "compute_chart",
      sig: "(julian_day: float, latitude: float, longitude: float, config: ChartConfig | None = None) -> ChartGraph",
      desc: "Compute a full natal chart. Returns a ChartGraph dataclass with .planets, .houses, .yogas, .aspects, and .graph. Requires ephemeris data loaded via load_ephemeris().",
      status: "stable",
    },
    {
      name: "compute_dasha",
      sig: "(julian_day: float, moon_longitude: float, system: DashaSystem = DashaSystem.VIMSHOTTARI) -> DashaTree",
      desc: "Compute the dasha tree from a Moon longitude. Returns a recursive DashaTree with up to 5 levels of sub-periods. No ephemeris file needed.",
      status: "stable",
    },
    {
      name: "get_nakshatra",
      sig: "(longitude: float) -> Nakshatra",
      desc: "Return the nakshatra and pada for any ecliptic longitude. Instant geometric calculation. No ephemeris needed.",
      status: "stable",
    },
    {
      name: "compute_varga",
      sig: "(chart: ChartGraph, division: int) -> ChartGraph",
      desc: "Compute a divisional chart (D-1 through D-60) from an existing ChartGraph. Returns a new ChartGraph with independently computed positions.",
      status: "stable",
    },
    {
      name: "compute_houses",
      sig: "(sidereal_time: float, latitude: float, system: HouseSystem = HouseSystem.PLACIDUS) -> HouseCusps",
      desc: "Compute house cusps from local sidereal time and latitude. Supports all 10 house systems including Placidus, Koch, Whole Sign, Equal, and Sripathi.",
      status: "stable",
    },
    {
      name: "find_aspects",
      sig: "(positions: list[BodyPosition], orbs: OrbConfig | None = None) -> list[Aspect]",
      desc: "Detect aspects among a list of body positions you supply. Returns typed Aspect objects with orb, applying/separating flag, and strength score.",
      status: "stable",
    },
    {
      name: "calendar_to_jd",
      sig: "(year: int, month: int, day: int, hour_ut: float = 0.0) -> float",
      desc: "Convert a proleptic Gregorian calendar date and UT hour to a Julian Day number. Zero dependencies.",
      status: "stable",
    },
    {
      name: "compute_ayanamsha",
      sig: "(julian_day: float, system: Ayanamsha = Ayanamsha.LAHIRI) -> float",
      desc: "Return the ayanamsha value in decimal degrees for any of the 44 supported systems at any Julian Day.",
      status: "stable",
    },
    {
      name: "planet_name",
      sig: "(planet: Planet, locale: Locale = Locale.EN) -> str",
      desc: "Return the localized name of a planet. Supports English, Hindi, Sanskrit, Tamil, Telugu, Kannada, and Bengali.",
      status: "stable",
    },
    {
      name: "sign_name",
      sig: "(sign: Sign, locale: Locale = Locale.EN) -> str",
      desc: "Return the localized name of a zodiac sign in any of the 7 supported languages.",
      status: "stable",
    },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — Python Bindings
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        Native speed. <span className="text-[#D4A843]">Python ergonomics.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        The <code className="font-mono text-sm bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5">vedaksha</code> Python
        package exposes the full Rust computation engine via PyO3. You get
        native Rust performance, Python ergonomics, and full type stubs for IDE
        autocompletion and runtime validation.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        The wheel ships with the DE440s embedded ephemeris for dates 1550–2650 CE.
        Extended coverage (DE441) requires loading an external file.
      </p>

      {/* Installation */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Installation
        </h2>
        <div className="space-y-4">
          {[
            { label: "pip", file: "terminal", cmd: "pip install vedaksha" },
            { label: "uv", file: "terminal", cmd: "uv add vedaksha" },
            { label: "poetry", file: "terminal", cmd: "poetry add vedaksha" },
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

      {/* Code Example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Code Example
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">chart.py</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`import vedaksha as vk
from vedaksha import (
    ChartConfig, HouseSystem, Ayanamsha, DashaSystem,
    DataClass, Locale
)

# 1. Convert calendar date to Julian Day
jd = vk.calendar_to_jd(1990, 6, 15, hour_ut=10.5)
print(f"Julian Day: {jd}")    # → 2448057.9375

# 2. Compute a full natal chart
config = ChartConfig(
    house_system = HouseSystem.PLACIDUS,
    ayanamsha    = Ayanamsha.LAHIRI,
    data_class   = DataClass.ANONYMOUS,
)
chart = vk.compute_chart(jd, latitude=28.6139, longitude=77.2090, config=config)

# 3. Inspect planets
for planet in chart.planets:
    name = vk.planet_name(planet.body, locale=Locale.EN)
    sign = vk.sign_name(planet.sign, locale=Locale.EN)
    print(f"{name:8s}  {planet.longitude:8.3f}°  {sign:12s}  H{planet.house}")

# → Sun       336.142°  Pisces        H12
# → Moon       45.231°  Taurus        H1
# ...

# 4. Dashas — no ephemeris file needed for this
moon_longitude = chart.planets["Moon"].longitude
dasha = vk.compute_dasha(jd, moon_longitude=moon_longitude)
print(f"Mahadasha: {dasha.current.lord}, ends {dasha.current.end_date}")

# 5. Nakshatras — instant
nk = vk.get_nakshatra(moon_longitude)
print(f"Moon nakshatra: {nk.name} Pada {nk.pada}")  # → Rohini Pada 2

# 6. Emit to Cypher for Neo4j
cypher_statements = chart.graph.emit_cypher()
for stmt in cypher_statements:
    print(stmt)  # MERGE (c:Chart {id: ...}) ...`}
            </code>
          </pre>
        </div>
      </div>

      {/* Functions */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Available Functions
        </h2>
        <div className="space-y-3">
          {functions.map((fn) => (
            <div
              key={fn.name}
              className="border border-[var(--color-brand-border)] rounded-xl p-5 hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            >
              <div className="flex items-start justify-between gap-4 mb-2">
                <div>
                  <code className="text-sm font-mono font-semibold text-[#D4A843] block mb-1">
                    {fn.name}
                  </code>
                  <code className="text-xs font-mono text-[var(--color-brand-text-muted)] leading-relaxed">
                    {fn.sig}
                  </code>
                </div>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded shrink-0 bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
                  {fn.status}
                </span>
              </div>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mt-2">
                {fn.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Type Stubs */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Type Stubs and IDE Support
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          The package ships with <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">.pyi</code> stub
          files for every public function and dataclass. This gives you full
          autocompletion, inline parameter hints, and type checking in any
          IDE or type checker that supports PEP 484.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">vedaksha/__init__.pyi (excerpt)</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`from dataclasses import dataclass
from enum import Enum
from typing import Optional

class HouseSystem(Enum):
    PLACIDUS = "Placidus"
    WHOLE_SIGN = "WholeSign"
    EQUAL = "Equal"
    KOCH = "Koch"
    CAMPANUS = "Campanus"
    REGIOMONTANUS = "Regiomontanus"
    SRIPATHI = "Sripathi"
    # ... 3 more

class Ayanamsha(Enum):
    LAHIRI = "Lahiri"
    FAGAN_BRADLEY = "FaganBradley"
    KRISHNAMURTI = "Krishnamurti"
    # ... 41 more

@dataclass
class Planet:
    body:       "PlanetBody"
    longitude:  float           # ecliptic, 0–360
    latitude:   float
    distance:   float           # AU
    speed:      float           # degrees/day
    retrograde: bool
    sign:       "Sign"
    house:      int             # 1–12
    nakshatra:  "Nakshatra"
    dignity:    "Dignity"

def compute_chart(
    julian_day: float,
    latitude:   float,
    longitude:  float,
    config:     Optional["ChartConfig"] = None,
) -> "ChartGraph": ...

def calendar_to_jd(
    year:    int,
    month:   int,
    day:     int,
    hour_ut: float = 0.0,
) -> float: ...`}
            </code>
          </pre>
        </div>
      </div>

      {/* Localization */}
      <div className="mb-14 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Localized Names
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl mb-3">
          Every astronomical name can be rendered in 7 languages.
          Pass a <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">Locale</code> value
          to <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">planet_name()</code>, <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">sign_name()</code>,
          and <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">nakshatra_name()</code>.
        </p>
        <div className="flex flex-wrap gap-2">
          {["Locale.EN", "Locale.HI", "Locale.SA", "Locale.TA", "Locale.TE", "Locale.KN", "Locale.BN"].map((l) => (
            <code key={l} className="font-mono text-xs px-3 py-1.5 rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg)] text-[var(--color-brand-text)]">
              {l}
            </code>
          ))}
        </div>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration/wasm-browser"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← WASM Browser"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/batch-computation"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"Batch Computation →"}
        </a>
      </div>
    </div>
  );
}
