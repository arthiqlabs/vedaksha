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
import os
import re
import sys
import urllib.request
import urllib.error

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


def download_file(suffix: str) -> str:
    """Download a VSOP87A file and return its path. Tries multiple URLs."""
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

    raise RuntimeError(
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


def generate_rust_file(
    planet_name: str,
    series: dict,
    full_series: dict,
    threshold: float,
) -> str:
    """Generate the Rust source file content for a planet."""
    lines = []

    # Count totals
    total_full = sum(len(v) for v in full_series.values())
    total_retained = sum(len(v) for v in series.values())

    # File header
    lines.append(f"// GENERATED FILE — do not edit manually")
    lines.append(f"//")
    lines.append(f"// Source: VSOP87A (Bretagnon & Francou 1988)")
    lines.append(f"// Planet: {planet_name.capitalize()}")
    lines.append(f"// Truncation threshold: {threshold:.0e} AU")
    lines.append(f"// Terms retained: {total_retained} of {total_full}")
    lines.append(f"//")
    lines.append(f"// Rectangular heliocentric ecliptic coordinates (J2000.0)")
    lines.append(f"// Each triple is (amplitude_AU, phase_rad, frequency_rad_per_millennium)")
    lines.append(f"// Evaluate: X_alpha(t) = t^alpha * sum_i [A_i * cos(B_i + C_i * t)]")
    lines.append(f"//   where t = Julian millennia from J2000.0 (JDE 2451545.0)")
    lines.append(f"")

    # Generate arrays for each coordinate and power
    for coord_idx, coord_label in enumerate(COORD_LABELS, start=1):
        for power in range(MAX_POWER + 1):
            array_name = f"{coord_label}{power}"
            key = (coord_idx, power)
            terms = series.get(key, [])
            full_count = len(full_series.get(key, []))

            lines.append(f"/// {coord_label} coordinate, T^{power} — {len(terms)} terms (of {full_count})")
            lines.append(f"pub static {array_name}: &[(f64, f64, f64)] = &[")
            for A, B, C in terms:
                lines.append(f"    ({A:>23.15e}, {B:>20.15f}, {C:>23.15f}),")
            lines.append(f"];")
            lines.append(f"")

    return "\n".join(lines)


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
    args = parser.parse_args()

    # Determine output directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    if args.output_dir:
        output_dir = args.output_dir
        if not os.path.isabs(output_dir):
            output_dir = os.path.join(project_root, output_dir)
    else:
        output_dir = os.path.join(
            project_root,
            "crates", "vedaksha-ephem-core", "src", "analytical", "coefficients",
        )

    os.makedirs(output_dir, exist_ok=True)

    print(f"VSOP87A Coefficient Generator")
    print(f"Threshold: {args.threshold:.0e} AU")
    print(f"Output:    {output_dir}")
    print()

    stats = {}

    for planet_name, suffix in PLANETS:
        print(f"Processing {planet_name.capitalize()}...")

        # Download
        filepath = download_file(suffix)

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

        # Generate Rust file
        rust_code = generate_rust_file(planet_name, truncated, full_series, args.threshold)
        out_path = os.path.join(output_dir, f"{planet_name}.rs")
        with open(out_path, "w") as f:
            f.write(rust_code)
        print(f"  Written to {out_path}")

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
    print("Done.")


if __name__ == "__main__":
    main()
