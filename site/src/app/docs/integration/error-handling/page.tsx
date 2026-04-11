export default function ErrorHandlingPage() {
  const computeErrors = [
    {
      variant: "DateOutOfRange",
      fields: "{ julian_day: f64, valid_min: f64, valid_max: f64 }",
      desc: "The requested Julian Day falls outside the coverage window of the loaded ephemeris. DE440s covers 1550–2650 CE (JD 2287184.5–2816787.5). DE441 covers approximately 13000 BCE to 17000 CE.",
      recovery: "Clamp the Julian Day to the valid range, or switch to DE441 for historical and far-future dates.",
    },
    {
      variant: "BodyNotAvailable",
      fields: "{ body: Planet }",
      desc: "The requested planet or asteroid is not present in the loaded ephemeris file. The embedded DE440s covers the Sun, Moon, and 8 planets. Asteroids and Chiron require DE440 or a supplementary file.",
      recovery: "Remove the body from the config, or load an ephemeris file that covers it.",
    },
    {
      variant: "InvalidFormat",
      fields: "{ field: String, value: String, expected: String }",
      desc: "A parameter value did not match its expected type, range, or enum. This includes out-of-range latitudes, unrecognised house system names, and malformed Julian Days.",
      recovery: "Inspect the field, value, and expected fields to correct the input.",
    },
    {
      variant: "IoError",
      fields: "{ path: PathBuf, source: io::Error }",
      desc: "An ephemeris file could not be read — permission denied, file not found, or truncated data. Only raised when a file-based ephemeris is configured.",
      recovery: "Verify the file path, permissions, and integrity of the ephemeris file.",
    },
  ];

  const mcpErrors = [
    { code: "DATE_OUT_OF_RANGE", http: 422, desc: "Julian Day outside coverage." },
    { code: "BODY_NOT_AVAILABLE", http: 422, desc: "Planet not in ephemeris." },
    { code: "INVALID_FORMAT", http: 400, desc: "Parameter type or value error." },
    { code: "CHART_NOT_FOUND", http: 404, desc: "chart_id not found in session." },
    { code: "EPHEMERIS_NOT_LOADED", http: 503, desc: "Function requires ephemeris data not yet loaded." },
    { code: "INTERNAL_ERROR", http: 500, desc: "Unexpected internal failure. File a bug." },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — Error Handling
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        No panics. <span className="text-[#D4A843]">No silent failures.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Every public Vedākṣha function returns <code className="font-mono text-sm bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5">Result&lt;T, ComputeError&gt;</code>.
        The library does not panic, does not silently degrade, and does not return
        sentinel values like <code className="font-mono text-sm bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5">0.0</code> or <code className="font-mono text-sm bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1.5 py-0.5">None</code> to
        indicate an error condition. If something is wrong, you get an error.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        The MCP layer wraps ComputeError into a structured JSON object with
        machine-readable <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">error_code</code> and
        a <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">suggested_action</code> that
        AI agents can read and act on autonomously.
      </p>

      {/* ComputeError Enum */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          The ComputeError Enum
        </h2>
        <div className="space-y-4">
          {computeErrors.map((e) => (
            <div
              key={e.variant}
              className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden"
            >
              <div className="px-5 py-4 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
                <div className="flex flex-col sm:flex-row sm:items-start gap-2">
                  <code className="text-sm font-mono font-semibold text-[#D4A843] shrink-0">
                    ComputeError::{e.variant}
                  </code>
                  <code className="text-xs font-mono text-[var(--color-brand-text-muted)] sm:mt-0.5">
                    {e.fields}
                  </code>
                </div>
              </div>
              <div className="px-5 py-4">
                <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] mb-3">
                  {e.desc}
                </p>
                <div className="flex items-start gap-2">
                  <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] shrink-0 mt-0.5">Recovery:</span>
                  <p className="text-xs text-[var(--color-brand-text-muted)]">{e.recovery}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Rust error handling example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Handling Errors in Rust
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">errors.rs</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
            <code>
              <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::prelude::*;{"\n"}
              <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::<span className="text-amber-700">ComputeError</span>;{"\n"}
              {"\n"}
              <span className="text-purple-600">fn</span> <span className="text-blue-700">process_birth</span>(jd: <span className="text-amber-700">f64</span>, lat: <span className="text-amber-700">f64</span>, lon: <span className="text-amber-700">f64</span>){"\n"}
              {"    "}{"-> "}<span className="text-amber-700">Result</span>{"<"}<span className="text-amber-700">ChartGraph</span>, <span className="text-amber-700">ComputeError</span>{">"}{"\n"}
              {"{"}{"\n"}
              {"    "}<span className="text-purple-600">match</span> <span className="text-blue-700">compute_chart</span>(jd, lat, lon, &<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>()) {"{"}{"\n"}
              {"        "}<span className="text-amber-700">Ok</span>(graph) {"=> Ok"}(graph),{"\n"}
              {"\n"}
              {"        "}<span className="text-amber-700">Err</span>(<span className="text-amber-700">ComputeError</span>::<span className="text-blue-700">DateOutOfRange</span> {"{ valid_min, valid_max, .. }"}) {"=> {"}{"\n"}
              {"            "}<span className="text-green-700">// Clamp to DE440s window and retry</span>{"\n"}
              {"            "}<span className="text-purple-600">let</span> clamped = jd.<span className="text-blue-700">clamp</span>(valid_min, valid_max);{"\n"}
              {"            "}<span className="text-blue-700">compute_chart</span>(clamped, lat, lon, &<span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>()){"\n"}
              {"        },"}{"\n"}
              {"\n"}
              {"        "}<span className="text-amber-700">Err</span>(<span className="text-amber-700">ComputeError</span>::<span className="text-blue-700">BodyNotAvailable</span> {"{ body }"}) {"=> {"}{"\n"}
              {"            "}<span className="text-green-700">// Retry without the unavailable body</span>{"\n"}
              {"            "}<span className="text-purple-600">let</span> config = <span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>(){"\n"}
              {"                "}.<span className="text-blue-700">without_body</span>(body);{"\n"}
              {"            "}<span className="text-blue-700">compute_chart</span>(jd, lat, lon, &config){"\n"}
              {"        },"}{"\n"}
              {"\n"}
              {"        "}<span className="text-amber-700">Err</span>(e) {"=> Err"}(e),<span className="text-green-700"> // propagate other errors</span>{"\n"}
              {"    "}{"}"}{"\n"}
              {"}"}
            </code>
          </pre>
        </div>
      </div>

      {/* Python error handling */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Handling Errors in Python
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">errors.py</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`import vedaksha as vk

try:
    chart = vk.compute_chart(1000000.0, 28.6, 77.2)

except vk.DateOutOfRange as e:
    print(f"JD {e.julian_day} out of range.")
    print(f"Valid: {e.valid_min:.1f} – {e.valid_max:.1f}")
    # e.suggested_action → human-readable recovery hint

except vk.BodyNotAvailable as e:
    print(f"Body {e.body} not in loaded ephemeris.")
    print(f"Suggested: {e.suggested_action}")

except vk.InvalidFormat as e:
    print(f"Bad value for '{e.field}': {e.value!r}")
    print(f"Expected: {e.expected}")

except vk.ComputeError as e:
    # Catch-all for any other ComputeError subclass
    print(f"Compute failed: {e}")
    print(f"Code: {e.error_code}")
    print(f"Suggested action: {e.suggested_action}")`}
            </code>
          </pre>
        </div>
      </div>

      {/* Polar Fallback */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Polar Fallback Warning
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          For latitudes above approximately 66°N or below 66°S, some house
          systems (notably Placidus and Koch) produce undefined cusps. Vedākṣha
          does not error — it automatically falls back to Whole Sign houses and
          attaches a <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 text-[#D4A843]">PolarFallback</code> warning
          to the result.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">polar.rs</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
            <code>
              <span className="text-purple-600">let</span> graph = <span className="text-blue-700">compute_chart</span>(jd, <span className="text-blue-700">70.0</span>, <span className="text-blue-700">25.0</span>, &config)?;{"\n"}
              {"\n"}
              <span className="text-purple-600">if let</span> <span className="text-amber-700">Some</span>(<span className="text-amber-700">Warning</span>::<span className="text-blue-700">PolarFallback</span> {"{ original, fallback }"}) = &graph.warning {"{"}{"\n"}
              {"    "}<span className="text-blue-700">println!</span>(<span className="text-green-700">"Polar latitude: {} → {} fell back to {}"</span>,{"\n"}
              {"             "}graph.latitude, original, fallback);{"\n"}
              {"    "}<span className="text-green-700">// House cusps in graph.houses are Whole Sign, not Placidus</span>{"\n"}
              {"}"}
            </code>
          </pre>
        </div>
      </div>

      {/* MCP Error table */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          MCP Error Codes
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Every MCP error includes <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">error_code</code>,
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 ml-1">message</code>, and
          <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5 ml-1">suggested_action</code> in the
          JSON-RPC error.data object. AI agents can parse <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">suggested_action</code> and
          retry the correct call without human intervention.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="grid grid-cols-3 px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">error_code</span>
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">HTTP status</span>
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Meaning</span>
          </div>
          {mcpErrors.map((e, i) => (
            <div
              key={e.code}
              className={`grid grid-cols-3 px-5 py-3 hover:bg-[var(--color-brand-bg-subtle)] transition-colors ${i !== mcpErrors.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""}`}
            >
              <code className="text-xs font-mono text-[#D4A843] font-semibold">{e.code}</code>
              <code className="text-xs font-mono text-[var(--color-brand-text-muted)]">{e.http}</code>
              <span className="text-xs text-[var(--color-brand-text-secondary)]">{e.desc}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Design philosophy */}
      <div className="mb-14 rounded-xl border border-[var(--color-brand-border)] p-6 bg-[var(--color-brand-bg-subtle)]">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-[var(--color-brand-text)] mb-2">
          Design Philosophy
        </h2>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl mb-3">
          Vedākṣha is designed for use in automated pipelines — data import jobs,
          AI agents, batch processes — where there is no human to observe a wrong
          result. The library therefore refuses to guess. If it cannot compute a
          correct answer, it returns an error.
        </p>
        <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)] max-w-2xl">
          The <code className="font-mono text-xs bg-[var(--color-brand-bg)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">suggested_action</code> field
          exists specifically for AI agents. An agent that receives an error can
          read the action, correct its parameters, and retry — without escalating
          to the user. This loop has been tested with Claude and other MCP-compatible
          agents.
        </p>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration/batch-computation"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Batch Computation"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/data-sources"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"Data Sources →"}
        </a>
      </div>
    </div>
  );
}
