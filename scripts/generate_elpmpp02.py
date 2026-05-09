#!/usr/bin/env python3
"""
ELP/MPP02 Coefficient Generator for Vedākṣha Ephemeris

Fetches the IMCCE primary distribution of the Chapront-Francou ELP/MPP02
lunar theory from cyrano-se.obspm.fr, verifies SHA256 digests against the
fetch manifest in the project's clean-room re-derivation spec, and emits
three Rust source files (`moon_longitude.rs`, `moon_latitude.rs`,
`moon_distance.rs`) holding the static main-problem and perturbation
series tables.

Sources (all primary):
    Chapront J., Francou G., 2003,
        "The lunar theory ELP revisited. Introduction of new planetary
        perturbations", Astronomy & Astrophysics 404, 735.
        DOI: 10.1051/0004-6361:20030529.
    IMCCE explanatory note `elpmpp02.pdf` (Chapront, Chapront, Francou —
        Observatoire de Paris / SYRTE, October 2002). Distributed with the
        coefficient files at:
        ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/

No third-party implementation has been consulted.

Usage:
    python3 scripts/generate_elpmpp02.py \\
        [--threshold 1e-5] \\
        [--output-dir crates/vedaksha-ephem-core/src/analytical/coefficients]

Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
Licensed under BSL 1.1.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
import urllib.error
import urllib.request

# ---------------------------------------------------------------------------
# Fetch manifest (transcribed from the clean-room spec §6).
# ---------------------------------------------------------------------------

FTP_BASE = "ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/"

# Each entry: (filename, sha256, expected_size_bytes).
MANIFEST: list[tuple[str, str, int]] = [
    ("README.TXT",   "aee2edbd7cc679fd6f1e871fb017493f075a6befc7f720f4f6bb8ee2b56e7fd8",     4445),
    ("elpmpp02.pdf", "08b988dda14deb8850f82ea4077115a6d44251c325dd48de137b15bc5c0c2c93",   215008),
    ("ELPMPP02.for", "3a95c77de63dddc4d438765da3f91598ddd4f1ce3601683cb2c1af8e4acd838f",    28112),
    ("ELP_MAIN.S1",  "3602147c43b77f86394c9034ea0e66807c6a674eeac87ada2a23aecd328706f1",   103360),
    ("ELP_MAIN.S2",  "c06fca782f973a5365a4a19dd8b8a2a5ce711063e007ad929be6206686e459b8",    92755),
    ("ELP_MAIN.S3",  "22f2cebde62d7451bc984ea67716b32091848c6f33ce746a9d1ba5de76074a56",    71141),
    ("ELP_PERT.S1",  "222b2895f476370e93b05c50bc207d5f637ca3cd7002f848054ff44b9f1742ba",  1209918),
    ("ELP_PERT.S2",  "0fd9af9d5e79fb9315c2ea295c8abe8f1ca385401fa93c8a032d64eb45c7d209",   668038),
    ("ELP_PERT.S3",  "15123e2eb0683ebffacc2b67339693532060a4db1f9c26eba502e0dad941d216",  1281928),
]

DATA_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data", "elpmpp02")

# Default amplitude truncation threshold matches the existing VSOP87A
# pipeline convention. Coefficient units: arcsec for S1/S2, km for S3.
DEFAULT_THRESHOLD = 1e-5


# ---------------------------------------------------------------------------
# Download + integrity verification
# ---------------------------------------------------------------------------

def sha256_of(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch_manifest_file(filename: str, expected_sha: str, expected_size: int) -> str:
    """Fetch one file via FTP and verify its SHA256."""
    os.makedirs(DATA_DIR, exist_ok=True)
    out_path = os.path.join(DATA_DIR, filename)

    # Reuse a previously fetched copy iff sha matches.
    if os.path.exists(out_path):
        actual = sha256_of(out_path)
        if actual == expected_sha:
            print(f"  [cache] {filename}  ({os.path.getsize(out_path):,} bytes)")
            return out_path
        else:
            print(f"  [cache miss] {filename} (sha mismatch, refetching)")

    url = FTP_BASE + filename
    print(f"  [fetch] {url}")
    try:
        urllib.request.urlretrieve(url, out_path)
    except (urllib.error.URLError, urllib.error.HTTPError, OSError) as exc:
        raise RuntimeError(f"failed to fetch {url}: {exc}") from exc

    size = os.path.getsize(out_path)
    actual = sha256_of(out_path)
    if actual != expected_sha:
        raise RuntimeError(
            f"SHA256 mismatch for {filename}: expected {expected_sha}, got {actual}"
        )
    if size != expected_size:
        # Size mismatch with sha match is impossible; size mismatch alone
        # (without sha mismatch) cannot occur. Defensive log only.
        print(f"  [warn] {filename} size {size} differs from expected {expected_size}")
    print(f"  [ok]    {filename}  sha256 verified  ({size:,} bytes)")
    return out_path


def fetch_all() -> dict[str, str]:
    """Fetch every file in the manifest. Returns name→local-path."""
    print(f"Distribution: {FTP_BASE}")
    print(f"Local cache:  {DATA_DIR}")
    paths: dict[str, str] = {}
    for filename, sha, size in MANIFEST:
        paths[filename] = fetch_manifest_file(filename, sha, size)
    return paths


# ---------------------------------------------------------------------------
# Parsers
# ---------------------------------------------------------------------------

# Main-problem record format (FORTRAN: 4i3, 2x, f13.5, 6f12.2):
#   i1 i2 i3 i4   A_i   B1 B2 B3 B4 B5 B6
# Indices i1..i4 are the multipliers of (D, F, l, l').
# A_i is the amplitude (arcsec for S1/S2, km for S3).
# B1..B6 are partial derivatives ∂A/∂σ_j, σ = (m, Γ, E, e', α, μ).
# (For S3, B values are scaled by 1/a0 — we keep the raw printed values.)

def parse_main_file(path: str) -> tuple[str, list[tuple]]:
    """Parse an ELP_MAIN.S{1,2,3} file. Returns (title, [(i1..i4, A, B1..B6)])."""
    with open(path, "r") as f:
        header = f.readline().rstrip()
        terms: list[tuple] = []
        for line in f:
            if not line.strip():
                continue
            # Fixed-width: i1 i2 i3 i4 are columns 1-12 (4i3), then 2x blank,
            # then f13.5 amplitude, then 6f12.2 partials.
            try:
                i1 = int(line[0:3])
                i2 = int(line[3:6])
                i3 = int(line[6:9])
                i4 = int(line[9:12])
                # 2 blank chars (cols 13-14), then f13.5 (cols 15-27)
                a = float(line[14:27])
                b = []
                pos = 27
                for _ in range(6):
                    b.append(float(line[pos:pos + 12]))
                    pos += 12
                terms.append((i1, i2, i3, i4, a, b[0], b[1], b[2], b[3], b[4], b[5]))
            except (ValueError, IndexError):
                # Tolerate tail / junk lines silently.
                continue
        return header, terms


# Perturbation-record format (FORTRAN: i5, 2d20.13, 16i3):
#   index   S    C   ifi(1..16)
# Of the 16 ints, only the first 13 are used physically: i1..i4 (Delaunay)
# + i5..i12 (eight planetary args) + i13 (zeta multiplier).

def parse_pert_file(path: str) -> tuple[str, list[list[tuple]]]:
    """
    Parse an ELP_PERT.S{1,2,3} file. The file is grouped by time-power
    n ∈ {0, 1, 2, 3}. Each group begins with a header line whose format
    is FORTRAN (25x, 2i10): a 25-char title, then a count (10 chars),
    then a power index (10 chars). Returns (first-header, [groups]) with
    groups[n] = [(S, C, i1..i13)].
    """
    with open(path, "r") as f:
        # Strip both \r\n and \n.
        lines = [ln.rstrip("\r\n") for ln in f.readlines()]

    groups: list[list[tuple]] = []
    idx = 0
    first_header: str | None = None
    while idx < len(lines):
        gh = lines[idx]
        if not gh.strip():
            idx += 1
            continue
        idx += 1
        try:
            count = int(gh[25:35])
            _it = int(gh[35:45])
        except (ValueError, IndexError):
            # Not a group header — finished.
            break
        if first_header is None:
            first_header = gh
        group: list[tuple] = []
        for _ in range(count):
            if idx >= len(lines):
                break
            line = lines[idx]
            idx += 1
            try:
                # i5  2d20.13  16i3
                _serial = int(line[0:5])
                s_str = line[5:25].replace("D", "E").replace("d", "e")
                c_str = line[25:45].replace("D", "E").replace("d", "e")
                s = float(s_str)
                c = float(c_str)
                ints: list[int] = []
                pos = 45
                for _k in range(13):
                    ints.append(int(line[pos:pos + 3]))
                    pos += 3
                group.append((s, c, *ints))
            except (ValueError, IndexError):
                continue
        groups.append(group)
    return (first_header or ""), groups


# ---------------------------------------------------------------------------
# Truncation
# ---------------------------------------------------------------------------

def truncate_main(terms: list[tuple], threshold: float) -> list[tuple]:
    return [t for t in terms if abs(t[4]) >= threshold]


def truncate_pert(group: list[tuple], threshold: float) -> list[tuple]:
    # Magnitude is sqrt(S^2 + C^2) — same as Fortran's `cper` amplitude.
    out = []
    for t in group:
        s, c = t[0], t[1]
        mag = (s * s + c * c) ** 0.5
        if mag >= threshold:
            out.append(t)
    return out


# ---------------------------------------------------------------------------
# Rust emit
# ---------------------------------------------------------------------------

RUST_HEADER = """// GENERATED FILE — do not edit manually.
//
// Source: ELP/MPP02 (Chapront & Francou 2003, A&A 404, 735;
//         IMCCE explanatory note `elpmpp02.pdf`, October 2002).
// Distribution: ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/
// Generator:    scripts/generate_elpmpp02.py
//
// {variable}
// Truncation threshold: |A| >= {threshold:.0e} ({units})
// Main terms retained:  {nmain} of {nmain_full}
// Perturbation terms retained per power group n=0..3:
{pert_summary}//
// Records:
//   MAIN[i] = (i1, i2, i3, i4, amp, b1, b2, b3, b4, b5, b6)
//     phase = i1*D + i2*F + i3*l + i4*l_prime
//   PERT_n[i] = (s, c, i1, i2, i3, i4, i5, i6, i7, i8, i9, i10, i11, i12, i13)
//     phase = i1*D + i2*F + i3*l + i4*l_prime
//             + i5*Me + i6*V + i7*T + i8*Ma + i9*J + i10*Sa + i11*U + i12*N
//             + i13*zeta
"""


def emit_main_array(name: str, terms: list[tuple], indent: str = "    ") -> list[str]:
    out = [f"pub static {name}: &[(i32, i32, i32, i32, f64, f64, f64, f64, f64, f64, f64)] = &["]
    for t in terms:
        i1, i2, i3, i4, a, b1, b2, b3, b4, b5, b6 = t
        out.append(
            f"{indent}({i1}, {i2}, {i3}, {i4}, "
            f"{a:.5f}, {b1:.2f}, {b2:.2f}, {b3:.2f}, {b4:.2f}, {b5:.2f}, {b6:.2f}),"
        )
    out.append("];")
    return out


def emit_pert_array(name: str, terms: list[tuple], indent: str = "    ") -> list[str]:
    out = [
        f"pub static {name}: &[(f64, f64, "
        "i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32)] = &["
    ]
    for t in terms:
        s, c = t[0], t[1]
        ints = t[2:]
        ints_fmt = ", ".join(str(x) for x in ints)
        out.append(f"{indent}({s:.13e}, {c:.13e}, {ints_fmt}),")
    out.append("];")
    return out


def write_rust_file(
    out_path: str,
    variable: str,
    units: str,
    threshold: float,
    main_full: list[tuple],
    main_kept: list[tuple],
    pert_full: list[list[tuple]],
    pert_kept: list[list[tuple]],
) -> None:
    pert_summary_lines = []
    for n, (full, kept) in enumerate(zip(pert_full, pert_kept)):
        pert_summary_lines.append(f"//   n={n}: {len(kept)} of {len(full)}\n")
    pert_summary = "".join(pert_summary_lines)

    lines = [
        RUST_HEADER.format(
            variable=variable,
            threshold=threshold,
            units=units,
            nmain=len(main_kept),
            nmain_full=len(main_full),
            pert_summary=pert_summary,
        ),
        "",
        "#![allow(clippy::approx_constant)]",
        "#![allow(clippy::excessive_precision)]",
        "",
    ]
    lines.append(f"/// Main-problem series ({variable}).")
    lines.extend(emit_main_array("MAIN", main_kept))
    lines.append("")
    for n, terms in enumerate(pert_kept):
        lines.append(f"/// Perturbation series, power t^{n} ({variable}).")
        lines.extend(emit_pert_array(f"PERT_{n}", terms))
        lines.append("")

    with open(out_path, "w") as f:
        f.write("\n".join(lines))
    print(f"  written {out_path}  ({sum(1 for _ in open(out_path)):,} lines)")


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description="ELP/MPP02 generator for Vedākṣha")
    ap.add_argument("--threshold", type=float, default=DEFAULT_THRESHOLD,
                    help=f"amplitude truncation threshold (default: {DEFAULT_THRESHOLD:g})")
    ap.add_argument("--output-dir", type=str, default=None,
                    help="output dir for generated Rust files")
    args = ap.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    if args.output_dir:
        out_dir = args.output_dir
        if not os.path.isabs(out_dir):
            out_dir = os.path.join(project_root, out_dir)
    else:
        out_dir = os.path.join(
            project_root, "crates", "vedaksha-ephem-core", "src", "analytical", "coefficients",
        )
    os.makedirs(out_dir, exist_ok=True)

    print("ELP/MPP02 Coefficient Generator")
    print(f"Threshold: {args.threshold:g}")
    print(f"Output:    {out_dir}")
    print()

    paths = fetch_all()
    print()

    plan = [
        ("longitude", "ELP_MAIN.S1", "ELP_PERT.S1", "moon_longitude.rs", "Longitude (V)", "arcsec"),
        ("latitude",  "ELP_MAIN.S2", "ELP_PERT.S2", "moon_latitude.rs",  "Latitude (U)",  "arcsec"),
        ("distance",  "ELP_MAIN.S3", "ELP_PERT.S3", "moon_distance.rs",  "Distance (r)",  "km"),
    ]

    for label, main_name, pert_name, out_file, variable, units in plan:
        print(f"=== {label} ===")
        _, main_full = parse_main_file(paths[main_name])
        _, pert_full = parse_pert_file(paths[pert_name])
        main_kept = truncate_main(main_full, args.threshold)
        pert_kept = [truncate_pert(g, args.threshold) for g in pert_full]
        full_pert = sum(len(g) for g in pert_full)
        kept_pert = sum(len(g) for g in pert_kept)
        print(f"  main: kept {len(main_kept)} of {len(main_full)}")
        print(f"  pert: kept {kept_pert} of {full_pert} (over {len(pert_full)} power groups)")
        write_rust_file(
            os.path.join(out_dir, out_file),
            variable, units, args.threshold,
            main_full, main_kept, pert_full, pert_kept,
        )
        print()

    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
