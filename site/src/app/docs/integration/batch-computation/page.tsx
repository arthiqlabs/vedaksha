export default function BatchComputationPage() {
  const principles = [
    {
      num: "01",
      title: "Stateless Architecture",
      desc: "Every Vedākṣha function takes its full input as parameters and returns a value. There is no mutable global state, no singleton, no session object. Functions can be called from any thread at any time.",
    },
    {
      num: "02",
      title: "Pure Functions",
      desc: "The same inputs always produce the same outputs. No hidden dependencies, no environment reads, no network calls inside the compute functions themselves. This makes every function trivially cacheable and trivially parallelisable.",
    },
    {
      num: "03",
      title: "Thread-Safe by Construction",
      desc: "The Rust type system enforces the absence of shared mutable state at compile time. There are no locks, no mutexes, no atomic operations in the hot path. Parallelism is free.",
    },
    {
      num: "04",
      title: "Zero Coordination Overhead",
      desc: "Batch jobs need no coordinator process, no work queue, no message broker. Split your input list, hand shards to threads or workers, collect results. That is the entire architecture.",
    },
  ];

  const perfTable = [
    { task: "Dasha tree (Vimshottari, 5 levels)", single: "~0.08 ms", batch1k: "~80 ms (1 core)", batch1k_par: "~12 ms (8 cores)" },
    { task: "Nakshatra lookup", single: "~0.002 ms", batch1k: "~2 ms (1 core)", batch1k_par: "<1 ms (8 cores)" },
    { task: "House cusps (Placidus)", single: "~0.15 ms", batch1k: "~150 ms (1 core)", batch1k_par: "~20 ms (8 cores)" },
    { task: "Full natal chart (with DE440s)", single: "~1.2 ms", batch1k: "~1.2 s (1 core)", batch1k_par: "~170 ms (8 cores)" },
    { task: "Aspect detection (10 bodies)", single: "~0.05 ms", batch1k: "~50 ms (1 core)", batch1k_par: "~7 ms (8 cores)" },
  ];

  return (
    <div className="max-w-5xl mx-auto px-6 py-20">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-3">
        Integration Guide — Batch Computation
      </p>
      <h1 className="text-3xl font-bold tracking-tight uppercase text-[var(--color-brand-text)] mb-3">
        1000 charts. <span className="text-[#D4A843]">Trivial parallelism.</span>
      </h1>
      <p className="text-base text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
        Vedākṣha&apos;s stateless, pure-function architecture makes batch computation
        a first-class use case. There is no special batch API — you simply call
        the same functions you already use, in parallel, across as many threads
        or processes as you have available.
      </p>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-12 max-w-2xl">
        The library ships with built-in Rayon integration for Rust consumers.
        Python and WASM consumers can use their platform&apos;s native parallelism
        primitives — the GIL is not held during computation.
      </p>

      {/* Design Principles */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Why It Works
        </h2>
        <div className="rounded-xl border border-[var(--color-brand-border)] bg-[var(--color-brand-border)] grid grid-cols-1 md:grid-cols-2 gap-px overflow-hidden">
          {principles.map((p) => (
            <div key={p.num} className="bg-[var(--color-brand-bg)] p-6 hover:bg-[var(--color-brand-bg-subtle)] transition-colors">
              <span className="text-xs font-semibold uppercase tracking-[0.15em] text-[var(--color-brand-text-muted)]">
                {p.num}
              </span>
              <h3 className="text-sm font-semibold uppercase tracking-wide text-[#D4A843] mt-2 mb-2">
                {p.title}
              </h3>
              <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
                {p.desc}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Rust + Rayon Example */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Rust — Parallel Batch with Rayon
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Add <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">rayon</code> to
          your dependencies and replace <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">.iter()</code> with <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">.par_iter()</code>.
          Rayon handles thread pool management automatically.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">batch.rs</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)]">
            <code>
              <span className="text-purple-600">use</span> <span className="text-blue-700">rayon</span>::prelude::*;{"\n"}
              <span className="text-purple-600">use</span> <span className="text-blue-700">vedaksha</span>::prelude::*;{"\n"}
              {"\n"}
              <span className="text-green-700">/// A minimal birth record: Julian Day + geographic coordinates</span>{"\n"}
              <span className="text-purple-600">struct</span> <span className="text-amber-700">BirthRecord</span> {"{"}{"\n"}
              {"    "}julian_day: <span className="text-amber-700">f64</span>,{"\n"}
              {"    "}latitude:   <span className="text-amber-700">f64</span>,{"\n"}
              {"    "}longitude:  <span className="text-amber-700">f64</span>,{"\n"}
              {"}"}{"\n"}
              {"\n"}
              <span className="text-purple-600">fn</span> <span className="text-blue-700">batch_compute</span>(records: <span className="text-amber-700">Vec</span>{"<"}<span className="text-amber-700">BirthRecord</span>{">"}) {"{"}{"\n"}
              {"    "}<span className="text-purple-600">let</span> config = <span className="text-amber-700">ChartConfig</span>::<span className="text-blue-700">vedic</span>();{"\n"}
              {"\n"}
              {"    "}<span className="text-green-700">// par_iter() distributes across all available CPU cores</span>{"\n"}
              {"    "}<span className="text-purple-600">let</span> results: <span className="text-amber-700">Vec</span>{"<_>"} = records{"\n"}
              {"        "}.<span className="text-blue-700">par_iter</span>(){"\n"}
              {"        "}.<span className="text-blue-700">map</span>(|rec| {"{"}{"\n"}
              {"            "}<span className="text-blue-700">compute_chart</span>({"\n"}
              {"                "}rec.julian_day,{"\n"}
              {"                "}rec.latitude,{"\n"}
              {"                "}rec.longitude,{"\n"}
              {"                "}&config,{"\n"}
              {"            "}){"\n"}
              {"        "}{"}"}).{"\n"}
              {"        "}<span className="text-blue-700">collect</span>();{"\n"}
              {"\n"}
              {"    "}<span className="text-green-700">// Results are in the same order as records.</span>{"\n"}
              {"    "}<span className="text-green-700">// Errors are per-record — one bad JD does not abort the batch.</span>{"\n"}
              {"    "}<span className="text-purple-600">for</span> (i, result) <span className="text-purple-600">in</span> results.<span className="text-blue-700">iter</span>().<span className="text-blue-700">enumerate</span>() {"{"}{"\n"}
              {"        "}<span className="text-purple-600">match</span> result {"{"}{"\n"}
              {"            "}Ok(graph) =&gt; println!(&quot;chart id: ...&quot;, graph.id),{"\n"}
              {"            "}Err(e) =&gt; eprintln!(&quot;error: ...&quot;, e),{"\n"}
              {"        "}{"}"}{"\n"}
              {"    "}{"}"}{"\n"}
              {"}"}
            </code>
          </pre>
        </div>
      </div>

      {/* Python parallel */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Python — Parallel Batch with ProcessPoolExecutor
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          The Python GIL is not held during Vedākṣha computation — the extension
          releases it before entering Rust. Use <code className="font-mono text-xs bg-[var(--color-brand-bg-subtle)] border border-[var(--color-brand-border)] rounded px-1 py-0.5">ProcessPoolExecutor</code> for
          the best parallelism on multi-core systems.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden shadow-sm">
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg)]">
            <div className="flex items-center gap-1.5">
              <span className="size-2.5 rounded-full bg-red-400/50" />
              <span className="size-2.5 rounded-full bg-yellow-400/50" />
              <span className="size-2.5 rounded-full bg-green-400/50" />
            </div>
            <span className="text-[10px] font-mono text-[var(--color-brand-text-muted)]">batch.py</span>
          </div>
          <pre className="p-4 overflow-x-auto text-sm leading-7 font-mono bg-[var(--color-brand-bg-code)] text-[var(--color-brand-text-secondary)]">
            <code>{`from concurrent.futures import ProcessPoolExecutor
import vedaksha as vk

def compute_one(record: dict) -> dict | str:
    """Run in a worker process — no shared state needed."""
    try:
        chart = vk.compute_chart(
            julian_day = record["jd"],
            latitude   = record["lat"],
            longitude  = record["lon"],
        )
        return {"id": chart.graph.id, "asc": chart.houses.ascendant}
    except vk.ComputeError as e:
        return {"error": str(e), "suggested_action": e.suggested_action}

def batch_compute(records: list[dict]) -> list[dict]:
    with ProcessPoolExecutor() as executor:
        results = list(executor.map(compute_one, records))
    return results

# Example: 1000 records, computed across all CPU cores
records = [{"jd": 2448057.9 + i, "lat": 28.6, "lon": 77.2}
           for i in range(1000)]

results = batch_compute(records)
print(f"Computed {len(results)} charts")`}
            </code>
          </pre>
        </div>
      </div>

      {/* Performance table */}
      <div className="mb-14">
        <h2 className="text-xs font-semibold uppercase tracking-[0.2em] text-[#D4A843] mb-5">
          Performance Characteristics
        </h2>
        <p className="text-sm text-[var(--color-brand-text-secondary)] mb-4 max-w-2xl">
          Measured on Apple M2 Pro. All timings are wall-clock including function
          call overhead. Batch timings use Rayon on Rust / ProcessPoolExecutor on Python.
        </p>
        <div className="rounded-xl border border-[var(--color-brand-border)] overflow-hidden">
          <div className="grid grid-cols-4 px-5 py-3 border-b border-[var(--color-brand-border)] bg-[var(--color-brand-bg-subtle)]">
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Task</span>
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">Single call</span>
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">1 000 serial</span>
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)]">1 000 parallel</span>
          </div>
          {perfTable.map((row, i) => (
            <div
              key={row.task}
              className={`grid grid-cols-4 px-5 py-4 hover:bg-[var(--color-brand-bg-subtle)] transition-colors ${i !== perfTable.length - 1 ? "border-b border-[var(--color-brand-border)]" : ""}`}
            >
              <span className="text-xs text-[var(--color-brand-text-secondary)] pr-4">{row.task}</span>
              <code className="text-xs font-mono text-[var(--color-brand-text-muted)]">{row.single}</code>
              <code className="text-xs font-mono text-[var(--color-brand-text-muted)]">{row.batch1k}</code>
              <code className="text-xs font-mono text-[#D4A843] font-semibold">{row.batch1k_par}</code>
            </div>
          ))}
        </div>
        <p className="text-xs text-[var(--color-brand-text-muted)] mt-3">
          Parallel timings scale linearly with core count. No synchronisation overhead.
        </p>
      </div>

      <div className="flex items-center gap-6">
        <a
          href="/docs/integration/python-bindings"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"← Python Bindings"}
        </a>
        <span className="text-[var(--color-brand-text-muted)]">·</span>
        <a
          href="/docs/integration/error-handling"
          className="inline-flex items-center text-sm font-semibold text-[#D4A843] hover:underline"
        >
          {"Error Handling →"}
        </a>
      </div>
    </div>
  );
}
