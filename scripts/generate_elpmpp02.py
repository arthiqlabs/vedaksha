#!/usr/bin/env python3
"""
ELP/MPP02 Coefficient Generator for Vedaksha Ephemeris

Downloads ELP/MPP02 ASCII coefficient files from the ytliu0/ElpMpp02 GitHub
repository (a C++ re-packaging of the original IMCCE Fortran data), parses
them with the LLR-fit adjustment factors, truncates by amplitude threshold,
and generates three Rust source files:
    moon_longitude.rs, moon_latitude.rs, moon_distance.rs

Usage:
    python3 scripts/generate_elpmpp02.py [--threshold 1e-5] \
        [--output-dir crates/vedaksha-ephem-core/src/analytical/coefficients]

Source: ELP/MPP02 — Chapront & Francou, A&A 404, 735 (2003)
Data:   https://github.com/ytliu0/ElpMpp02

Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
"""

import argparse
import math
import os
import sys
import urllib.request
import urllib.error

# ---------------------------------------------------------------------------
# Data file names and download URL
# ---------------------------------------------------------------------------
BASE_URL = "https://raw.githubusercontent.com/ytliu0/ElpMpp02/master/{filename}"

MAIN_FILES = {
    "longitude": "elp_main.long",
    "latitude":  "elp_main.lat",
    "distance":  "elp_main.dist",
}

PERT_FILES = {
    "longitude": ["elp_pert.longT0", "elp_pert.longT1", "elp_pert.longT2", "elp_pert.longT3"],
    "latitude":  ["elp_pert.latT0",  "elp_pert.latT1",  "elp_pert.latT2"],
    "distance":  ["elp_pert.distT0", "elp_pert.distT1", "elp_pert.distT2", "elp_pert.distT3"],
}

DATA_DIR = os.path.join(os.path.dirname(__file__), "data", "elpmpp02")

# ---------------------------------------------------------------------------
# LLR-fit parameters (corr=0) — from ElpMpp02.cpp setup_parameters()
# ---------------------------------------------------------------------------
PI = math.pi
SEC = PI / 648000.0  # arcseconds -> radians

# Adjustment parameters (LLR fit)
Dw1_1 = -0.32311
De = 0.00005
Dgam = 0.00069
Deart_1 = 0.01442
Dep = 0.00226

am = 0.074801329
alpha = 0.002571881
dtsm = 2.0 * alpha / (3.0 * am)
xa = 2.0 * alpha / 3.0

w11 = (1732559343.73604 + Dw1_1) * SEC

delnu_nu = (0.55604 + Dw1_1) * SEC / w11
dele = (0.01789 + De) * SEC
delg = (-0.08066 + Dgam) * SEC
delnp_nu = (-0.06424 + Deart_1) * SEC / w11
delep = (-0.12879 + Dep) * SEC

fB1 = -am * delnu_nu + delnp_nu
fB2 = delg
fB3 = dele
fB4 = delep
fB5 = -xa * delnu_nu + dtsm * delnp_nu
fA_dist = 1.0 - 2.0 / 3.0 * delnu_nu  # factor for distance A column


def download_file(filename: str, data_dir: str) -> str:
    """Download a data file if not already present. Return local path."""
    os.makedirs(data_dir, exist_ok=True)
    local_path = os.path.join(data_dir, filename)
    if os.path.exists(local_path):
        return local_path
    url = BASE_URL.format(filename=filename)
    print(f"  Downloading {url} ...")
    try:
        urllib.request.urlretrieve(url, local_path)
    except Exception as e:
        print(f"  ERROR downloading {filename}: {e}", file=sys.stderr)
        sys.exit(1)
    return local_path


def parse_main_problem(filepath: str, coord: str):
    """
    Parse a main-problem file (elp_main.long / .lat / .dist).

    Each line after the count line has:
        i_D  i_F  i_L  i_Lp  A  B1  B2  B3  B4  B5  B6

    Returns list of (i_D, i_F, i_L, i_Lp, adjusted_amplitude).
    Amplitude is in radians.
    """
    fA = fA_dist if coord == "distance" else 1.0

    with open(filepath) as f:
        lines = f.read().strip().split("\n")

    n_terms = int(lines[0].strip())
    terms = []
    for line in lines[1: n_terms + 1]:
        parts = line.split()
        i_D, i_F, i_L, i_Lp = int(parts[0]), int(parts[1]), int(parts[2]), int(parts[3])
        A = float(parts[4])
        B1 = float(parts[5])
        B2 = float(parts[6])
        B3 = float(parts[7])
        B4 = float(parts[8])
        B5 = float(parts[9])
        # B6 = float(parts[10])  # not used in adjustment
        adjusted_A = fA * A + fB1 * B1 + fB2 * B2 + fB3 * B3 + fB4 * B4 + fB5 * B5
        terms.append((i_D, i_F, i_L, i_Lp, adjusted_A))

    assert len(terms) == n_terms, f"Expected {n_terms} terms, got {len(terms)}"
    return terms


def parse_perturbation(filepath: str):
    """
    Parse a perturbation file (elp_pert.*).

    Each line after the count line has:
        i_D i_F i_L i_Lp i_Me i_Ve i_EM i_Ma i_Ju i_Sa i_Ur i_Ne i_zeta  amplitude  phase

    Returns list of (i_D, i_F, i_L, i_Lp, i_Me, i_Ve, i_EM, i_Ma, i_Ju, i_Sa, i_Ur, i_Ne, i_zeta, amplitude, phase).
    Amplitude and phase are in radians.
    """
    with open(filepath) as f:
        lines = f.read().strip().split("\n")

    n_terms = int(lines[0].strip())
    terms = []
    for line in lines[1: n_terms + 1]:
        parts = line.split()
        ints = [int(parts[i]) for i in range(13)]
        amp = float(parts[13])
        phase = float(parts[14])
        terms.append(tuple(ints) + (amp, phase))

    assert len(terms) == n_terms, f"Expected {n_terms} terms, got {len(terms)}"
    return terms


def generate_rust_file(coord: str, main_terms, pert_series, threshold: float, output_dir: str):
    """
    Generate a Rust source file for one coordinate (longitude, latitude, or distance).

    The main problem terms are stored as:
        (i_D: i8, i_F: i8, i_L: i8, i_Lp: i8, amplitude: f64)
    where amplitude is the pre-adjusted coefficient in radians.

    Perturbation terms (for each T-power) are stored as:
        (i_D: i8, i_F: i8, i_L: i8, i_Lp: i8,
         i_Me: i8, i_Ve: i8, i_EM: i8, i_Ma: i8,
         i_Ju: i8, i_Sa: i8, i_Ur: i8, i_Ne: i8, i_zeta: i8,
         amplitude: f64, phase: f64)

    Truncation: terms with |amplitude| < threshold are dropped.
    For main problem, threshold is applied in radians.
    For perturbations, threshold is applied in radians.
    """
    coord_title = coord.capitalize()

    # Truncate main terms
    total_main = len(main_terms)
    main_filtered = [(d, f, l, lp, a) for (d, f, l, lp, a) in main_terms if abs(a) >= threshold]
    main_filtered.sort(key=lambda t: abs(t[4]), reverse=True)
    retained_main = len(main_filtered)

    # Truncate perturbation terms per T-power
    pert_filtered = []
    total_pert = 0
    retained_pert = 0
    for series in pert_series:
        total_pert += len(series)
        filtered = [t for t in series if abs(t[13]) >= threshold]
        filtered.sort(key=lambda t: abs(t[13]), reverse=True)
        retained_pert += len(filtered)
        pert_filtered.append(filtered)

    total_all = total_main + total_pert
    retained_all = retained_main + retained_pert

    # Build Rust source
    lines = []
    lines.append("// GENERATED FILE — do not edit manually")
    lines.append("//")
    lines.append("// Source: ELP/MPP02 (Chapront & Francou 2003, A&A 404, 735)")
    lines.append(f"// Coordinate: {coord_title}")
    threshold_unit = "km" if coord == "distance" else "rad"
    lines.append(f"// Truncation threshold: {threshold:.2e} {threshold_unit}")
    lines.append(f"// Terms retained: {retained_all} of {total_all}")
    lines.append(f"//   Main problem: {retained_main} of {total_main}")
    for i, series in enumerate(pert_filtered):
        lines.append(f"//   Perturbation T^{i}: {len(series)} of {len(pert_series[i])}")
    lines.append("//")
    lines.append("// Adjustment: LLR-fit parameters (corr=0)")
    lines.append("// Main problem amplitudes are pre-adjusted (A + fB*B corrections) in radians.")

    if coord == "distance":
        lines.append("// Distance main problem uses COSINE series; perturbations use SINE.")
        lines.append("// Distance amplitudes represent dimensionless corrections to mean distance.")
    else:
        lines.append("// Angular series use SINE for main problem and SINE for perturbations.")
        lines.append("// Amplitudes are in radians.")

    lines.append("//")
    lines.append(f"// Evaluate main: sum_i A_i * {'cos' if coord == 'distance' else 'sin'}(i_D*D + i_F*F + i_L*L + i_Lp*Lp)")
    lines.append("// Evaluate pert:  sum_i A_i * sin(phase_i + i_D*D + i_F*F + i_L*L + i_Lp*Lp")
    lines.append("//                              + i_Me*Me + i_Ve*Ve + i_EM*EM + i_Ma*Ma")
    lines.append("//                              + i_Ju*Ju + i_Sa*Sa + i_Ur*Ur + i_Ne*Ne + i_zeta*zeta)")
    lines.append("// where D, F, L, Lp are Delaunay arguments and Me..Ne, zeta are planetary arguments.")
    lines.append("")
    lines.append("#![allow(clippy::excessive_precision)]")
    lines.append("#![allow(clippy::unreadable_literal)]")
    lines.append("")

    # Main problem array
    lines.append(f"/// Main problem — {retained_main} terms (of {total_main})")
    lines.append("/// (i_D, i_F, i_L, i_Lp, amplitude_rad)")
    lines.append("pub static MAIN: &[(i8, i8, i8, i8, f64)] = &[")
    for (d, f, l, lp, a) in main_filtered:
        lines.append(f"    ({d:3}, {f:3}, {l:3}, {lp:3}, {a:>25.17e}),")
    lines.append("];")
    lines.append("")

    # Perturbation arrays
    for i, series in enumerate(pert_filtered):
        lines.append(f"/// Perturbation T^{i} — {len(series)} terms (of {len(pert_series[i])})")
        lines.append("/// (i_D, i_F, i_L, i_Lp, i_Me, i_Ve, i_EM, i_Ma, i_Ju, i_Sa, i_Ur, i_Ne, i_zeta, amplitude_rad, phase_rad)")
        lines.append(f"pub static PERT_T{i}: &[(i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, i8, f64, f64)] = &[")
        for t in series:
            ints = t[:13]
            amp = t[13]
            phase = t[14]
            int_str = ", ".join(f"{v:3}" for v in ints)
            lines.append(f"    ({int_str}, {amp:>25.17e}, {phase:>25.17e}),")
        lines.append("];")
        lines.append("")

    rust_src = "\n".join(lines)
    out_path = os.path.join(output_dir, f"moon_{coord}.rs")
    with open(out_path, "w") as f:
        f.write(rust_src)
    print(f"  Wrote {out_path}")
    print(f"    Main: {retained_main}/{total_main}, Pert: {retained_pert}/{total_pert}, Total: {retained_all}/{total_all}")

    return retained_all, total_all


def main():
    parser = argparse.ArgumentParser(
        description="Generate ELP/MPP02 Moon coefficient Rust source files"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=1e-5,
        help="Amplitude truncation threshold in radians for lon/lat (default: 1e-5, ~2 arcsec)",
    )
    parser.add_argument(
        "--dist-threshold",
        type=float,
        default=1.0,
        help="Amplitude truncation threshold in km for distance (default: 1.0 km, ~0.5 arcsec)",
    )
    parser.add_argument(
        "--output-dir",
        default=os.path.join(
            os.path.dirname(__file__),
            "..",
            "crates",
            "vedaksha-ephem-core",
            "src",
            "analytical",
            "coefficients",
        ),
        help="Output directory for generated Rust files",
    )
    parser.add_argument(
        "--data-dir",
        default=DATA_DIR,
        help="Directory for downloaded data files",
    )
    args = parser.parse_args()

    output_dir = os.path.normpath(args.output_dir)
    data_dir = os.path.normpath(args.data_dir)

    print(f"ELP/MPP02 Coefficient Generator")
    print(f"  Threshold (lon/lat): {args.threshold:.2e} rad ({args.threshold / SEC:.2f} arcsec)")
    print(f"  Threshold (distance): {args.dist_threshold:.2e} km")
    print(f"  Output:    {output_dir}")
    print(f"  Data:      {data_dir}")
    print()

    # Download all required files
    print("Downloading data files...")
    all_filenames = []
    for fname in MAIN_FILES.values():
        all_filenames.append(fname)
    for flist in PERT_FILES.values():
        all_filenames.extend(flist)

    for fname in all_filenames:
        download_file(fname, data_dir)
    print()

    # Process each coordinate
    grand_retained = 0
    grand_total = 0

    for coord in ["longitude", "latitude", "distance"]:
        print(f"Processing {coord}...")

        # Use distance-specific threshold for distance coordinate
        threshold = args.dist_threshold if coord == "distance" else args.threshold

        # Parse main problem
        main_path = os.path.join(data_dir, MAIN_FILES[coord])
        main_terms = parse_main_problem(main_path, coord)

        # Parse perturbation series
        pert_series = []
        for pert_fname in PERT_FILES[coord]:
            pert_path = os.path.join(data_dir, pert_fname)
            pert_terms = parse_perturbation(pert_path)
            pert_series.append(pert_terms)

        # Generate Rust file
        retained, total = generate_rust_file(
            coord, main_terms, pert_series, threshold, output_dir
        )
        grand_retained += retained
        grand_total += total
        print()

    print(f"Grand total: {grand_retained} terms retained of {grand_total}")
    print("Done.")


if __name__ == "__main__":
    main()
