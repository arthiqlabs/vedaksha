# Vedaksha technical report

Technical report describing **Vedaksha v10.0.0**, measured **2026-10-24**.
Not peer reviewed and not submitted anywhere. Published as a technical report by
ArthIQ Labs, an AI platforms company. It is not a preprint: there is no current
commitment to submit it for publication. BUSL-1.1 describes source-available software.

`main.tex` is self-contained apart from `references.bib`. The author block is unchanged.
The PDF and its source hash are refreshed together.

## Reproduce the measurements

Run from the repository root. Check `uptime` before each measurement and wait if
load is above approximately 20. The measurements used Rust 1.99.1 on macOS.
The ephemeris source, `Cargo.toml`, and `Cargo.lock` match tag `v10.0.0` at checkout
`21e9a23`; intervening changes do not alter the measured engine.

The default `target` path here points outside the worktree. These runs instead used
`docs/paper/build/cargo`, keeping build products inside this worktree.

```sh
uptime
mkdir -p docs/paper/build
export CARGO_TARGET_DIR="$PWD/docs/paper/build/cargo"
export VEDAKSHA_REQUIRE_FIXTURES=1
bash scripts/download_de440s.sh

# A: numerical path, including all per-row residuals and the era/body summaries
nice cargo test -p vedaksha-ephem-core --release --locked --test oracle_comparison -- --include-ignored --nocapture > docs/paper/build/oracle-spk.log 2>&1

# B: native aarch64 analytical path and both per-body tables
nice cargo test -p vedaksha-ephem-core --release --locked --test analytical_oracle -- --include-ignored --nocapture > docs/paper/build/analytical-aarch64.log 2>&1

# C: x86-64 analytical path, executed locally under Rosetta
CARGO_BUILD_JOBS=2 nice cargo test -p vedaksha-ephem-core --release --locked --target x86_64-apple-darwin --test analytical_oracle -- --include-ignored --nocapture > docs/paper/build/analytical-x86_64.log 2>&1

# D: ayanamsha agreement and fixture coverage
CARGO_BUILD_JOBS=2 nice cargo test -p vedaksha-astro --release --locked --test ayanamsha_fixture -- --nocapture > docs/paper/build/ayanamsha.log 2>&1

# E: oracle time-tag convention
CARGO_BUILD_JOBS=2 nice cargo test -p vedaksha-ephem-core --release --locked --test oracle_time_scale -- --nocapture > docs/paper/build/oracle-time-scale.log 2>&1
```

The x86-64 target and Rosetta were already installed. `arch -x86_64 /usr/bin/uname -m`
returned `x86_64`. C used the target's default feature configuration; it does not
establish a native x86-64 or explicitly enabled AVX2 result.

The DE440s download verified SHA256
`c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2`.
The committed Horizons fixture verified against its sidecar:
`26d0b70253c696f172066639113209f449388e5109741a6a783ac1bad8e2241b`.
The fixture was not re-fetched from Horizons.

To reproduce the future-interval mean and whole-fixture arithmetic from A's output:

```sh
python3 - <<'PY'
from pathlib import Path
import re
from statistics import mean
text = Path('docs/paper/build/oracle-spk.log').read_text()
rows = re.findall(
    r'^(\w+)\s+(\d{4}-\w{3}-\d{2})\s+00:00\s+[\d.]+\s+[\d.]+\s+([\d.]+)\s*$',
    text, re.M,
)
assert len(rows) == 24350
for label, sample in (
    ('1900-2025', [r for r in rows if r[1] < '2026']),
    ('2026-2100', [r for r in rows if r[1] >= '2026']),
    ('whole fixture', rows),
):
    print(label, len(sample), mean(float(r[2]) for r in sample),
          max(sample, key=lambda r: float(r[2])))
PY
```

The printed per-row values have three decimal places. Their future mean is
0.074753 arcseconds, reported as **0.075**. Their weighted whole-fixture mean is
0.0878363039 arcseconds, agreeing at three decimals with A's directly printed
**0.088**. The two rounded interval means likewise give
`(15350 * 0.096 + 9000 * 0.075) / 24350`, which rounds to 0.088.
No result was calculated by subtracting a release-to-release delta.

The parsers and comparison functions in `scripts/check_metrics_accuracy.py` were
applied to the captured logs without re-running either oracle. All published
per-body fields and the future maximum agreed with `metrics.json`.

## External parity figures

The cross-validation table (`tab:crossval`) and the full-range table (`tab:fullrange`)
come from the parity harness at https://github.com/arthiqlabs/vedaksha-parity, run on
2026-10-24 against the published `vedaksha==10.0.0` Python package. This repository
cannot run that harness, because it does not ship the DE440 or INPOP21a kernels. The
figures are recorded in `docs/accuracy/2026-10-24-parity-results.json`. The harness
commits its configuration (`configs/crossval-2000.yaml`), the sweep script
(`scripts/fullrange_tt_sweep.py`) and the run artifacts (`results/canonical/`):

```sh
vedaksha-parity run-config configs/crossval-2000.yaml --out <dir>
python -I scripts/fullrange_tt_sweep.py vendor/kernels/de440.bsp 2292184.5 9131.25 43 <out.json>
```

The harness is maintained by the same authors as Vedaksha. What is independent is the
reference ephemerides, not the party running the measurement.

## Not measured for this report

Latitude, distance, speed, lunar nodes, house cusps, derived jyotish events, raw SPK
state vectors, performance, native x86-64 hardware, explicitly selected AVX2, bit
digests, and live registry installations. The full-range table is a 25-year grid with
at most eight samples per cell, so each maximum is a lower bound on the worst case.
Coefficient generators and clean-room derivations were not re-run. Full-suite counts
are attributed to the tag.

## Build and freshness

Requires `tectonic`; visual verification uses Poppler.

```sh
cd docs/paper
make pdf && cp build/main.pdf main.pdf && shasum -a 256 main.tex > main.tex.sha256
cd ../..
python3 scripts/check_paper_pdf_fresh.py
```

Build products and logs under `build/` are local ignored artifacts. `main.pdf` and
`main.tex.sha256` are the deliverables beside the sources.
