#!/usr/bin/env python3
"""
VSOP87A Coefficient Generator for Vedaksha Ephemeris

Downloads VSOP87A ASCII coefficient files from IMCCE for 8 planets,
parses the (amplitude, phase, frequency) triples, truncates by amplitude
threshold, and generates Rust source files with static coefficient arrays.

Usage:
    python3 scripts/generate_vsop87a.py [--threshold 1e-7] [--output-dir crates/vedaksha-ephem-core/src/analytical/coefficients]

Source: VSOP87A — Bretagnon & Francou, Astronomy and Astrophysics 202, 309 (1988)
Data:   https://ftp.imcce.fr/pub/ephem/planets/vsop87/

Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
"""

import argparse
import hashlib
import os
import re
import shutil
import struct
import sys
import tempfile
import urllib.error
import urllib.request

# Planet names and their VSOP87A file suffixes
PLANETS = [
    ("mercury", "mer"),
    ("venus",   "ven"),
    ("earth",   "ear"),
    ("mars",    "mar"),
    ("jupiter", "jup"),
    ("saturn",  "sat"),
    ("uranus",  "ura"),
    ("neptune", "nep"),
]

# Coordinate labels for VSOP87A (rectangular heliocentric ecliptic J2000)
COORD_LABELS = ["X", "Y", "Z"]

# Powers of T (0 through 5)
MAX_POWER = 5

# Base URLs to try (IMCCE FTP via HTTPS, then alternative mirrors)
BASE_URLS = [
    "https://ftp.imcce.fr/pub/ephem/planets/vsop87/VSOP87A.{suffix}",
    "http://ftp.imcce.fr/pub/ephem/planets/vsop87/VSOP87A.{suffix}",
    # VizieR mirror
    "https://cdsarc.cds.unistra.fr/ftp/VI/81/VSOP87A.{suffix}",
]

DATA_DIR = os.path.join(os.path.dirname(__file__), "data")


class SourceUnreachable(RuntimeError):
    """No IMCCE mirror could be reached.

    Distinct from a SHA mismatch on purpose. `--verify` exists to catch
    upstream *drift*; a network timeout is not drift, it is infrastructure we
    do not control. Conflating the two made the scheduled Full Validation job
    go red on network flakes, which trains everyone to ignore it.
    """


def download_file(suffix: str) -> str:
    """Download a VSOP87A file and return its path. Tries multiple URLs.

    Raises `SourceUnreachable` when every mirror fails. A file that downloads
    but whose regenerated blob mismatches is real drift and is reported by the
    `--verify` comparison below, not here.
    """
    os.makedirs(DATA_DIR, exist_ok=True)
    filename = f"VSOP87A.{suffix}"
    filepath = os.path.join(DATA_DIR, filename)

    if os.path.exists(filepath) and os.path.getsize(filepath) > 1000:
        print(f"  Using cached {filename}")
        return filepath

    for url_template in BASE_URLS:
        url = url_template.format(suffix=suffix)
        print(f"  Trying {url} ...")
        try:
            urllib.request.urlretrieve(url, filepath)
            size = os.path.getsize(filepath)
            if size > 1000:
                print(f"  Downloaded {filename} ({size:,} bytes)")
                return filepath
            else:
                os.remove(filepath)
                print(f"  File too small ({size} bytes), trying next URL")
        except (urllib.error.URLError, urllib.error.HTTPError, OSError) as e:
            print(f"  Failed: {e}")
            continue

    raise SourceUnreachable(
        f"Could not download VSOP87A.{suffix} from any source. "
        f"Check network connectivity or manually place the file in {DATA_DIR}/"
    )


def parse_vsop87a_file(filepath: str) -> dict:
    """
    Parse a VSOP87A file and return a dict:
        {(coord_index, power): [(amplitude, phase, frequency), ...]}
    where coord_index is 1=X, 2=Y, 3=Z and power is 0-5.

    VSOP87A file format (IMCCE):
      Header lines contain "VSOP87" with fields like:
        "VSOP87 VERSION A1  EARTH  VARIABLE 1 (XYZ)  *T**0  843 TERMS ..."
      Term lines are fixed-width with the last 3 whitespace-delimited numbers
      being A (amplitude in AU), B (phase in radians), C (frequency in rad/millennium).
    """
    series = {}
    current_key = None

    # Regex to extract VARIABLE N and *T**N from header lines
    header_var_re = re.compile(r"VARIABLE\s+(\d+)")
    header_pow_re = re.compile(r"\*T\*\*(\d+)")

    with open(filepath, "r") as f:
        for line in f:
            # Header lines contain "VSOP87" and define the current series
            if "VSOP87" in line:
                var_match = header_var_re.search(line)
                pow_match = header_pow_re.search(line)
                if var_match and pow_match:
                    ivar = int(var_match.group(1))  # 1=X, 2=Y, 3=Z
                    power = int(pow_match.group(1))  # 0-5
                    current_key = (ivar, power)
                    if current_key not in series:
                        series[current_key] = []
                else:
                    current_key = None
                continue

            if current_key is None:
                continue

            # Term line: the last 3 whitespace-delimited numbers are A, B, C
            line_stripped = line.strip()
            if not line_stripped:
                continue

            try:
                # Replace Fortran D notation with E (some VSOP87 versions use it)
                cleaned = line_stripped.replace("D", "E").replace("d", "e")
                tokens = cleaned.split()
                if len(tokens) >= 3:
                    A = float(tokens[-3])  # amplitude (AU)
                    B = float(tokens[-2])  # phase (radians)
                    C = float(tokens[-1])  # frequency (radians/millennium)
                    series[current_key].append((A, B, C))
            except (ValueError, IndexError):
                continue

    return series


def truncate_and_sort(series: dict, threshold: float) -> dict:
    """Truncate terms below threshold and sort by descending amplitude."""
    result = {}
    for key, terms in series.items():
        retained = [(A, B, C) for A, B, C in terms if abs(A) >= threshold]
        retained.sort(key=lambda t: abs(t[0]), reverse=True)
        result[key] = retained
    return result


MAGIC = b"VDKBLOB1"
VERSION = 1
VSOP_RECORD = struct.Struct("<ddd")  # (amplitude, phase, frequency)


def round_term_to_print_precision(term: tuple[float, float, float]) -> tuple[float, float, float]:
    """Round each component through the same `{:.15e}` / `{:.15f}` format-then-parse
    cycle the legacy text emitter performed. This makes the .bin and the
    (long-deleted) `.rs` literals exactly round-trip bit-equivalent."""
    A, B, C = term
    return (
        float(f"{A:.15e}"),
        float(f"{B:.15f}"),
        float(f"{C:.15f}"),
    )


def emit_blob(out_path: str, records: list[tuple[float, float, float]]) -> bytes:
    """Write the VDKBLOB1 blob + return its raw bytes (also written to .sha256)."""
    payload = bytearray()
    for term in records:
        payload.extend(VSOP_RECORD.pack(*term))
    header = bytearray()
    header.extend(MAGIC)
    header.extend(struct.pack("<I", VERSION))
    header.extend(struct.pack("<I", VSOP_RECORD.size))
    header.extend(struct.pack("<I", len(records)))
    header.extend(struct.pack("<I", 0))
    blob = bytes(header) + bytes(payload)
    with open(out_path, "wb") as f:
        f.write(blob)
    digest = hashlib.sha256(blob).hexdigest()
    with open(out_path + ".sha256", "w") as f:
        f.write(f"{digest}  {os.path.basename(out_path)}\n")
    return blob


def generate_wrapper_rs(planet_name: str) -> str:
    """Generate the thin LazyLock wrapper .rs file for a planet."""
    lines = []
    lines.append("// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.")
    lines.append("// Vedākṣha — Vision from Vedas")
    lines.append("// Licensed under BSL 1.1. See LICENSE file.")
    lines.append("//")
    lines.append("// GENERATED FILE — do not edit manually.")
    lines.append("//")
    lines.append("// Source: VSOP87A (Bretagnon & Francou 1988)")
    lines.append(f"// Planet: {planet_name.capitalize()}")
    lines.append("//")
    lines.append("// Each table is a packed little-endian VDKBLOB1 blob alongside this file")
    lines.append("// and is decoded into a `Vec<Vsop87Term>` at first access.")
    lines.append("")
    lines.append("use std::sync::LazyLock;")
    lines.append("")
    lines.append("use super::loader::{Vsop87Term, parse_vsop87};")
    lines.append("")
    for coord_label in COORD_LABELS:
        for power in range(MAX_POWER + 1):
            name = f"{coord_label}{power}"
            bin_file = f"{planet_name}/{name.lower()}.bin"
            panic = f"malformed {bin_file}"
            lines.append(f"pub static {name}: LazyLock<Vec<Vsop87Term>> = LazyLock::new(|| {{")
            lines.append(f'    parse_vsop87(include_bytes!("{bin_file}")).expect("{panic}")')
            lines.append("});")
            lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def emit_planet(
    planet_name: str,
    series: dict,
    output_dir: str,
) -> None:
    """Write per-table .bin + .sha256 + wrapper .rs for one planet."""
    bin_dir = os.path.join(output_dir, planet_name)
    os.makedirs(bin_dir, exist_ok=True)
    for coord_idx, coord_label in enumerate(COORD_LABELS, start=1):
        for power in range(MAX_POWER + 1):
            name = f"{coord_label}{power}"
            terms = [round_term_to_print_precision(t) for t in series.get((coord_idx, power), [])]
            emit_blob(os.path.join(bin_dir, f"{name.lower()}.bin"), terms)
    rs_path = os.path.join(output_dir, f"{planet_name}.rs")
    with open(rs_path, "w") as f:
        f.write(generate_wrapper_rs(planet_name))


def main():
    parser = argparse.ArgumentParser(
        description="Generate Rust VSOP87A coefficient files for Vedaksha"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=1e-7,
        help="Amplitude truncation threshold in AU (default: 1e-7)",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default=None,
        help="Output directory for generated Rust files",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Re-fetch from IMCCE primary, regenerate the .bin files into a temp"
        " directory, and compare SHA256s against the committed .sha256 sidecars."
        " Exits non-zero on mismatch.",
    )
    args = parser.parse_args()

    # Determine output directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    canonical_dir = os.path.join(
        project_root,
        "crates", "vedaksha-ephem-core", "src", "analytical", "coefficients",
    )
    if args.verify:
        verify_dir = tempfile.mkdtemp(prefix="vsop87a-verify-")
        output_dir = verify_dir
    elif args.output_dir:
        output_dir = args.output_dir
        if not os.path.isabs(output_dir):
            output_dir = os.path.join(project_root, output_dir)
    else:
        output_dir = canonical_dir

    os.makedirs(output_dir, exist_ok=True)

    print(f"VSOP87A Coefficient Generator")
    print(f"Threshold: {args.threshold:.0e} AU")
    print(f"Output:    {output_dir}")
    print()

    stats = {}

    for planet_name, suffix in PLANETS:
        print(f"Processing {planet_name.capitalize()}...")

        # Download
        try:
            filepath = download_file(suffix)
        except SourceUnreachable as exc:
            if args.verify:
                # Skip, don't fail. See SourceUnreachable — a red run here
                # would mean "the coefficients drifted", and that is not what
                # happened.
                print(f"\nSKIP: IMCCE mirrors unreachable — {exc}")
                print(
                    "Not a drift signal; nothing was verified. "
                    "Re-run when the source is up."
                )
                return 0
            raise

        # Parse
        full_series = parse_vsop87a_file(filepath)
        if not full_series:
            print(f"  ERROR: No series found in {filepath}!")
            print(f"  The file format may differ from expected. First 5 lines:")
            with open(filepath) as f:
                for i, line in enumerate(f):
                    if i >= 5:
                        break
                    print(f"    {line.rstrip()}")
            sys.exit(1)

        # Truncate
        truncated = truncate_and_sort(full_series, args.threshold)

        # Stats
        full_count = sum(len(v) for v in full_series.values())
        retained_count = sum(len(v) for v in truncated.values())
        stats[planet_name] = (retained_count, full_count)
        print(f"  {retained_count} / {full_count} terms retained")

        # Emit packed binary blobs + thin LazyLock wrapper.
        emit_planet(planet_name, truncated, output_dir)
        print(f"  Wrote {planet_name}/ + {planet_name}.rs to {output_dir}")

    # Summary
    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    total_retained = 0
    total_full = 0
    for planet_name, (retained, full) in stats.items():
        pct = 100 * retained / full if full else 0
        print(f"  {planet_name:>10s}: {retained:5d} / {full:5d} terms ({pct:5.1f}%)")
        total_retained += retained
        total_full += full
    pct = 100 * total_retained / total_full if total_full else 0
    print(f"  {'TOTAL':>10s}: {total_retained:5d} / {total_full:5d} terms ({pct:5.1f}%)")
    print()

    if args.verify:
        print("Verifying regenerated blobs against committed .sha256 sidecars...")
        mismatches = 0
        for planet_name, _ in PLANETS:
            for coord_label in COORD_LABELS:
                for power in range(MAX_POWER + 1):
                    name = f"{coord_label}{power}".lower()
                    fresh = os.path.join(output_dir, planet_name, f"{name}.bin")
                    committed_sha = os.path.join(
                        canonical_dir, planet_name, f"{name}.bin.sha256"
                    )
                    with open(fresh, "rb") as f:
                        fresh_digest = hashlib.sha256(f.read()).hexdigest()
                    with open(committed_sha) as f:
                        committed_digest = f.read().split()[0]
                    if fresh_digest != committed_digest:
                        print(f"  MISMATCH {planet_name}/{name}.bin: "
                              f"committed={committed_digest[:16]}…, fresh={fresh_digest[:16]}…")
                        mismatches += 1
        shutil.rmtree(output_dir, ignore_errors=True)
        if mismatches:
            print(f"\nverification failed: {mismatches} mismatched blob(s)")
            sys.exit(2)
        print("verification ok: all VSOP87A blobs match committed sha256s")

    print("Done.")


if __name__ == "__main__":
    main()
