import { FeatureCard } from "./FeatureCard";

const features = [
  {
    number: "01",
    title: "Planetary Engine",
    description:
      "Sub-arcsecond precision. JPL DE440/441 ephemeris with IAU 2006 precession, IAU 2000B nutation, annual aberration, and iterative light-time correction.",
  },
  {
    number: "02",
    title: "Vedic First-Class",
    description:
      "Complete 5-limb panchanga, 5 dasha systems, 50 yogas, 16 vargas, degree-precise Shadbala, graded drishti, Ashtakoota matching \u2014 not a plugin.",
  },
  {
    number: "03",
    title: "10 House Systems",
    description:
      "Placidus, Koch, Whole Sign, Equal, Campanus, Regiomontanus, Porphyry, Morinus, Alcabitius, Sripathi.",
  },
  {
    number: "04",
    title: "MCP Native",
    description:
      "7 typed tools with JSON schemas, structured errors, input validation, and streaming. Drop into any MCP-compatible agent.",
  },
  {
    number: "05",
    title: "Graph Output",
    description:
      "ChartGraph with 10 node types and 13 edge types. Deterministic IDs. Emit Cypher, SurrealQL, or vector embeddings.",
  },
  {
    number: "06",
    title: "Runs Everywhere",
    description:
      "One Rust codebase compiles to native binaries, WebAssembly for browsers, and Python wheels via PyO3.",
  },
  {
    number: "07",
    title: "PII-Blind",
    description:
      "Computation accepts ONLY Julian Day + coordinates. No names, birthdates, or personal data ever touches the engine.",
  },
  {
    number: "08",
    title: "44 Ayanamsha",
    description:
      "Lahiri, Fagan-Bradley, Krishnamurti, Raman, Aryabhata, Surya Siddhanta, Galactic Center, and 37 more.",
  },
  {
    number: "09",
    title: "Zero Legacy",
    description:
      "Built from scratch using only published NASA and academic sources. No code borrowed from existing libraries. Free for non-commercial use, $500 one-time for commercial.",
  },
];

export function FeatureGrid() {
  return (
    <div className="w-full max-w-5xl mx-auto px-6">
      <div className="rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 md:grid-cols-3 gap-px overflow-hidden">
        {features.map((feature) => (
          <FeatureCard key={feature.number} {...feature} />
        ))}
      </div>
    </div>
  );
}
