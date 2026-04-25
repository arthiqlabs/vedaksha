#!/bin/bash
# Download JPL DE441 binary ephemeris test data
# Source: https://ssd.jpl.nasa.gov/ftp/eph/planets/Linux/de441/
# DE441 is public domain (US Government work)

set -euo pipefail

DATA_DIR="$(cd "$(dirname "$0")/../data" && pwd)"
URL="https://ssd.jpl.nasa.gov/ftp/eph/planets/Linux/de441/linux_p1550p2650.441"
OUTPUT="${DATA_DIR}/linux_p1550p2650.441"

if [ -f "$OUTPUT" ]; then
    echo "DE441 file already exists: $OUTPUT"
    exit 0
fi

echo "Downloading JPL DE441 (~100MB)..."
curl -L -o "$OUTPUT" "$URL"
echo "Downloaded to $OUTPUT"
