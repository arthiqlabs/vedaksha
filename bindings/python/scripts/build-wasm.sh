#!/usr/bin/env bash
# Build the Vedākṣha engine into the wasm blob the Python package hosts.
#
# The blob is NOT committed — the engine source lives in this same repo, so the
# blob is a pure build artifact produced here (locally) or in CI before the
# wheel is built. Requires the wasm32 target:
#   rustup target add wasm32-unknown-unknown
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$here/src/vedaksha/_wasm/vedaksha.wasm"

cargo build --release --target wasm32-unknown-unknown -p vedaksha-py-engine

built="$root/target/wasm32-unknown-unknown/release/vedaksha_py_engine.wasm"
mkdir -p "$(dirname "$out")"
cp "$built" "$out"

echo "wrote $out"
echo "engine commit : $(git -C "$root" rev-parse --short HEAD)"
echo "size          : $(wc -c < "$out" | tr -d ' ') bytes"
