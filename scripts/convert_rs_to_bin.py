#!/usr/bin/env python3
"""
One-shot converter: parse the existing committed coefficient .rs files
(generated as `pub static X: &[(...)] = &[ ... ];`) and emit packed
little-endian .bin blobs + SHA256 sidecars + thin LazyLock .rs wrappers.

After this script runs, the heavyweight per-tuple .rs literals are gone;
rustc only has to parse a small wrapper module per planet / lunar
component, and the bytes are deserialised into Vec<...> at first access.

Run once from /workspace/vedaksha:

    python3 scripts/convert_rs_to_bin.py

The script reads:
    crates/vedaksha-ephem-core/src/analytical/coefficients/<name>.rs
and writes:
    .../coefficients/<name>/<table>.bin
    .../coefficients/<name>/<table>.bin.sha256
    .../coefficients/<name>.rs  (replaced with LazyLock wrapper)

Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
Licensed under BSL 1.1.
"""

from __future__ import annotations

import hashlib
import os
import re
import struct
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
COEFFS_DIR = os.path.join(
    ROOT, "crates", "vedaksha-ephem-core", "src", "analytical", "coefficients"
)

VSOP_PLANETS = [
    "mercury", "venus", "earth", "mars",
    "jupiter", "saturn", "uranus", "neptune",
]
ELP_TABLES = ["moon_distance", "moon_latitude", "moon_longitude"]

# ─── Rust tuple-literal parser ────────────────────────────────────────────────

# Captures one `pub static NAME: TYPE = &[ ... ];` block.
STATIC_RE = re.compile(
    r"pub\s+static\s+(?P<name>[A-Z0-9_]+)\s*:\s*&\[\s*\((?P<typ>[^)]*)\)\s*\]\s*=\s*&\[(?P<body>.*?)\];",
    re.DOTALL,
)

# Splits the body into `(...)` tuple literals, ignoring whitespace/newlines.
TUPLE_RE = re.compile(r"\(([^()]*)\)", re.DOTALL)


def parse_static_arrays(rs_text: str):
    """Yield (name, typ_fields, [tuple, …]) for every `pub static` in the file."""
    for m in STATIC_RE.finditer(rs_text):
        name = m.group("name")
        typ_fields = [t.strip() for t in m.group("typ").split(",")]
        body = m.group("body")
        tuples = []
        for t in TUPLE_RE.finditer(body):
            inner = t.group(1)
            # Split on commas, then evaluate each scalar in Python.
            parts = [p.strip() for p in inner.split(",") if p.strip()]
            if len(parts) != len(typ_fields):
                raise RuntimeError(
                    f"static {name}: tuple arity mismatch (got {len(parts)}, "
                    f"expected {len(typ_fields)}): {inner!r}"
                )
            row = []
            for field, raw in zip(typ_fields, parts):
                if field == "f64":
                    row.append(float(raw))
                elif field == "i32":
                    row.append(int(raw))
                else:
                    raise RuntimeError(f"unsupported field type {field!r}")
            tuples.append(tuple(row))
        yield name, typ_fields, tuples


# ─── Binary blob writer ───────────────────────────────────────────────────────

MAGIC = b"VDKBLOB1"
VERSION = 1


def write_blob(out_path: str, record_struct: struct.Struct, records: list[tuple]) -> None:
    """Write a VDKBLOB1 file + its <name>.sha256 sidecar."""
    record_size = record_struct.size
    record_count = len(records)
    payload = bytearray()
    for row in records:
        payload.extend(record_struct.pack(*row))

    header = bytearray()
    header.extend(MAGIC)
    header.extend(struct.pack("<I", VERSION))
    header.extend(struct.pack("<I", record_size))
    header.extend(struct.pack("<I", record_count))
    header.extend(struct.pack("<I", 0))

    blob = bytes(header) + bytes(payload)
    with open(out_path, "wb") as f:
        f.write(blob)

    digest = hashlib.sha256(blob).hexdigest()
    sidecar = out_path + ".sha256"
    with open(sidecar, "w") as f:
        f.write(f"{digest}  {os.path.basename(out_path)}\n")


def struct_for_fields(typ_fields: list[str]) -> struct.Struct:
    """Build a little-endian struct.Struct matching the field-by-field layout."""
    fmt = "<"
    for field in typ_fields:
        if field == "f64":
            fmt += "d"
        elif field == "i32":
            fmt += "i"
        else:
            raise RuntimeError(f"unsupported field {field!r}")
    return struct.Struct(fmt)


# ─── Rust wrapper emitters ────────────────────────────────────────────────────

VSOP_HEADER = """// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: VSOP87A (Bretagnon & Francou 1988)
// Planet: {planet_cap}
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded into a `Vec<Vsop87Term>` at first access.

use std::sync::LazyLock;

use super::loader::{{Vsop87Term, parse_vsop87}};

"""

ELP_HEADER = """// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
//
// GENERATED FILE — do not edit manually.
//
// Source: ELP/MPP02 (Chapront & Francou 2003, A&A 404, 735;
//         IMCCE explanatory note `elpmpp02.pdf`, October 2002).
// Distribution: ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/
// Component: {component}
//
// Each table is a packed little-endian VDKBLOB1 blob alongside this file
// and is decoded at first access.

use std::sync::LazyLock;

use super::loader::{{ElpMainTerm, ElpPertTerm, parse_elp_main, parse_elp_pert}};

"""


def lazy_decl(table_name: str, rs_type: str, parse_fn: str, bin_subdir: str) -> str:
    bin_file = table_name.lower() + ".bin"
    panic_msg = f"malformed {bin_subdir}/{bin_file}"
    return (
        f'pub static {table_name}: LazyLock<Vec<{rs_type}>> = LazyLock::new(|| {{\n'
        f'    {parse_fn}(include_bytes!("{bin_subdir}/{bin_file}")).expect("{panic_msg}")\n'
        f'}});\n'
    )


# ─── Per-file conversion drivers ──────────────────────────────────────────────

def convert_vsop_file(planet: str) -> None:
    rs_path = os.path.join(COEFFS_DIR, f"{planet}.rs")
    with open(rs_path, "r") as f:
        rs_text = f.read()

    bin_subdir = planet
    bin_dir = os.path.join(COEFFS_DIR, bin_subdir)
    os.makedirs(bin_dir, exist_ok=True)

    arrays = list(parse_static_arrays(rs_text))
    if not arrays:
        raise RuntimeError(f"no `pub static` arrays found in {rs_path}")

    # Sanity: every VSOP table is (f64, f64, f64) — triple of doubles.
    for name, fields, _ in arrays:
        if fields != ["f64", "f64", "f64"]:
            raise RuntimeError(f"{planet}::{name}: unexpected field types {fields}")

    rec_struct = struct_for_fields(["f64", "f64", "f64"])

    wrapper_lines = [VSOP_HEADER.format(planet_cap=planet.capitalize())]
    for name, _fields, tuples in arrays:
        out_bin = os.path.join(bin_dir, f"{name.lower()}.bin")
        write_blob(out_bin, rec_struct, tuples)
        wrapper_lines.append(
            lazy_decl(name, "Vsop87Term", "parse_vsop87", bin_subdir)
        )
        wrapper_lines.append("")

    with open(rs_path, "w") as f:
        f.write("\n".join(wrapper_lines).rstrip() + "\n")
    print(f"  {planet}: {len(arrays)} tables, "
          f"{sum(len(t) for _, _, t in arrays):,} total records")


def convert_elp_file(component: str) -> None:
    rs_path = os.path.join(COEFFS_DIR, f"{component}.rs")
    with open(rs_path, "r") as f:
        rs_text = f.read()

    bin_subdir = component
    bin_dir = os.path.join(COEFFS_DIR, bin_subdir)
    os.makedirs(bin_dir, exist_ok=True)

    arrays = list(parse_static_arrays(rs_text))
    if not arrays:
        raise RuntimeError(f"no `pub static` arrays found in {rs_path}")

    wrapper_lines = [ELP_HEADER.format(component=component.replace("_", " "))]
    total = 0
    for name, fields, tuples in arrays:
        rec_struct = struct_for_fields(fields)
        out_bin = os.path.join(bin_dir, f"{name.lower()}.bin")
        write_blob(out_bin, rec_struct, tuples)

        if name == "MAIN":
            rs_type, parse_fn = "ElpMainTerm", "parse_elp_main"
        elif name.startswith("PERT_"):
            rs_type, parse_fn = "ElpPertTerm", "parse_elp_pert"
        else:
            raise RuntimeError(f"unexpected ELP table name {name!r}")

        wrapper_lines.append(
            lazy_decl(name, rs_type, parse_fn, bin_subdir)
        )
        wrapper_lines.append("")
        total += len(tuples)

    with open(rs_path, "w") as f:
        f.write("\n".join(wrapper_lines).rstrip() + "\n")
    print(f"  {component}: {len(arrays)} tables, {total:,} total records")


def main() -> int:
    print(f"Converting coefficient .rs files in {COEFFS_DIR}")
    print()
    print("VSOP87A planets:")
    for planet in VSOP_PLANETS:
        convert_vsop_file(planet)
    print()
    print("ELP/MPP02 components:")
    for component in ELP_TABLES:
        convert_elp_file(component)
    print()
    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
