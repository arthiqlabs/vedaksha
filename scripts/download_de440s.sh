#!/bin/bash
# Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
# Vedākṣha — Vision from Vedas
#
# Download the JPL DE440s SPK kernel that SpkReader reads.
#
# data/de440s.bsp is gitignored (32 MB binary), so a fresh clone has to fetch
# it. Without it, every SpkReader-backed test skips itself and the suite goes
# green having verified nothing. Run this once after cloning.
#
# Source:  NAIF generic kernels (public domain, US Government work)
# Coverage: ~1849-2150 CE
#
# Note: DE440s is the *bundled numerical* kernel. It is distinct from DE441,
# which is fetched live from JPL Horizons by the oracle tests as an
# independent reference — see scripts/generate_horizons_oracle.py.

set -euo pipefail

DATA_DIR="$(cd "$(dirname "$0")/.." && pwd)/data"
URL="https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp"
OUTPUT="${DATA_DIR}/de440s.bsp"
EXPECTED_SHA256="c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2"
EXPECTED_BYTES=32726016

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

mkdir -p "$DATA_DIR"

if [ -f "$OUTPUT" ]; then
    if [ "$(sha256_of "$OUTPUT")" = "$EXPECTED_SHA256" ]; then
        echo "DE440s already present and verified: $OUTPUT"
        exit 0
    fi
    echo "DE440s present but checksum does not match — refetching." >&2
fi

echo "Downloading DE440s (~31 MiB) from NAIF..."
curl -L --fail --progress-bar -o "${OUTPUT}.part" "$URL"

ACTUAL_BYTES=$(wc -c < "${OUTPUT}.part" | tr -d ' ')
if [ "$ACTUAL_BYTES" != "$EXPECTED_BYTES" ]; then
    rm -f "${OUTPUT}.part"
    echo "ERROR: expected ${EXPECTED_BYTES} bytes, got ${ACTUAL_BYTES}." >&2
    exit 1
fi

ACTUAL_SHA256=$(sha256_of "${OUTPUT}.part")
if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
    rm -f "${OUTPUT}.part"
    echo "ERROR: sha256 mismatch." >&2
    echo "  expected ${EXPECTED_SHA256}" >&2
    echo "  actual   ${ACTUAL_SHA256}" >&2
    echo "NAIF may have republished the kernel. Verify before trusting it." >&2
    exit 1
fi

mv "${OUTPUT}.part" "$OUTPUT"
echo "Verified and installed: $OUTPUT"
echo "sha256 ${ACTUAL_SHA256}"
