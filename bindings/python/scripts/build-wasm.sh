#!/usr/bin/env bash
# SPDX-License-Identifier: BUSL-1.1
# Build the Vedaksha engine into the wasm blob the Python package hosts.
#
# The blob is NOT committed — the engine source lives in this same repo, so the
# blob is a pure build artifact produced here (locally) or in CI before the
# wheel is built. Requires the wasm32 target:
#   rustup target add wasm32-unknown-unknown
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$here/src/vedaksha/_wasm/vedaksha.wasm"

# +simd128 makes wide's f64x4 vectorize the lunar kernel (otherwise it falls
# back to scalar) -- matches release.yml's wasm job, which sets the same flag
# for the npm package's wasm build. Verified bit-identical against the
# scalar build via bindings/python/tests/conformance/test_parity.py before
# this was enabled; see docs/audit/2026-08-29-perf-investigation.md #7b.
RUSTFLAGS="-C target-feature=+simd128" cargo build --release --target wasm32-unknown-unknown -p vedaksha-py-engine

built="$root/target/wasm32-unknown-unknown/release/vedaksha_py_engine.wasm"
mkdir -p "$(dirname "$out")"
cp "$built" "$out"

echo "wrote $out"
echo "engine commit : $(git -C "$root" rev-parse --short HEAD)"
echo "size          : $(wc -c < "$out" | tr -d ' ') bytes"
