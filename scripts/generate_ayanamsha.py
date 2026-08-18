#!/usr/bin/env python3
"""
Ayanamsha conformance-fixture generator for Vedaksha.

Derives every ayanamsha this engine ships, from the primary anchors recorded in
`docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json`, and writes a
fixture that the Rust engine is tested against.

Why this exists
---------------
Sections 6.1-6.3 of the derivation spec cannot, by themselves, catch a
reverse-engineered value: an anchor reproduces itself perfectly no matter where
it came from. Derivation integrity is established by the audit trail. What this
script adds is that the derivation stays *executable* — a constant can no longer
enter the engine by being typed, because a second, independent implementation
has to agree with it.

This is deliberately a separate implementation, not a binding to the Rust one.
It reads the same published polynomials and the same catalogue rows and does the
arithmetic again in Python. If the two agree to a nanodegree across six
millennia, neither is a transcription of the other.

Usage:
    python3 scripts/generate_ayanamsha.py                # write the fixture
    python3 scripts/generate_ayanamsha.py --verify       # regenerate, compare sha256
    python3 scripts/generate_ayanamsha.py --check-catalogue   # re-query VizieR

Unlike the coefficient-blob generators, `--verify` needs no network and
therefore cannot pass vacuously: everything it derives from is committed.

Copyright (c) 2026 ArthIQ Labs LLC. All rights reserved.
"""

import argparse
import hashlib
import json
import math
import os
import sys
import tempfile
import urllib.error
import urllib.request

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INPUTS = os.path.join(
    PROJECT_ROOT, "docs", "audit", "2026-08-17-ayanamsha-cleanroom", "derivation-inputs.json"
)
DEFAULT_FIXTURE = os.path.join(
    PROJECT_ROOT, "crates", "vedaksha-astro", "tests", "fixtures", "ayanamsha.json"
)

ARCSEC_TO_DEG = 1.0 / 3600.0
ARCSEC_TO_RAD = math.pi / (180.0 * 3600.0)
MAS_TO_RAD = ARCSEC_TO_RAD / 1000.0
J2000 = 2451545.0
JULIAN_YEAR = 365.25
DAYS_PER_CENTURY = 36525.0


# ── Time ──────────────────────────────────────────────────────────────────────


def centuries(jd):
    return (jd - J2000) / DAYS_PER_CENTURY


def poly(coeffs, t):
    """Horner evaluation of coeffs[0] + coeffs[1]*t + ..."""
    acc = 0.0
    for c in reversed(coeffs):
        acc = acc * t + c
    return acc


def calendar_to_jd(year, month, day):
    """Meeus, Astronomical Algorithms 2nd ed., eq. 7.1.

    Julian calendar before 1582 October 15, Gregorian on and after. Matches
    `vedaksha_ephem_core::julian::calendar_to_jd`.
    """
    y, m = year, month
    if m <= 2:
        y -= 1
        m += 12
    # The Gregorian correction applies from 1582-10-15 onward.
    gregorian = (year, month, day) >= (1582, 10, 15.0)
    if gregorian:
        a = y // 100 if y >= 0 else -((-y + 99) // 100)
        b = 2 - a + (a // 4 if a >= 0 else -((-a + 3) // 4))
    else:
        b = 0
    return (
        math.floor(365.25 * (y + 4716))
        + math.floor(30.6001 * (m + 1))
        + day
        + b
        - 1524.5
    )


def jd_to_calendar(jd):
    """Inverse of `calendar_to_jd`. Meeus Ch. 7."""
    jd = jd + 0.5
    z = math.floor(jd)
    f = jd - z
    if z < 2299161:
        a = z
    else:
        alpha = math.floor((z - 1867216.25) / 36524.25)
        a = z + 1 + alpha - math.floor(alpha / 4)
    b = a + 1524
    c = math.floor((b - 122.1) / 365.25)
    d = math.floor(365.25 * c)
    e = math.floor((b - d) / 30.6001)
    day = b - d - math.floor(30.6001 * e) + f
    month = e - 1 if e < 14 else e - 13
    year = c - 4716 if month > 2 else c - 4715
    return int(year), int(month), day


# ── Precession and frames (IAU 2006 P03) ──────────────────────────────────────


class Precession:
    def __init__(self, inputs):
        self.p_a = inputs["precession_p03_general_longitude_arcsec"]["coefficients"]
        fw = inputs["precession_fw_angles_arcsec"]
        self.gamma_bar = fw["gamma_bar"]
        self.phi_bar = fw["phi_bar"]
        self.psi_bar = fw["psi_bar"]
        self.epsilon_a = fw["epsilon_a"]

    def general_longitude_arcsec(self, jd):
        """p_A — general precession in longitude along the moving ecliptic."""
        return poly(self.p_a, centuries(jd))

    def mean_obliquity_rad(self, jd):
        return poly(self.epsilon_a, centuries(jd)) * ARCSEC_TO_RAD

    def matrix(self, jd):
        """ICRS -> mean equator and equinox of date, frame bias included.

        P = Rx(-eps_A) . Rz(-psi_bar) . Rx(phi_bar) . Rz(gamma_bar)

        The order is load-bearing: the rotations do not commute, and a wrong
        order stays inside a milliarcsecond near J2000 while growing without
        bound away from it.
        """
        t = centuries(jd)
        g = poly(self.gamma_bar, t) * ARCSEC_TO_RAD
        p = poly(self.phi_bar, t) * ARCSEC_TO_RAD
        s = poly(self.psi_bar, t) * ARCSEC_TO_RAD
        e = self.mean_obliquity_rad(jd)
        return mat_mul(
            rot_x(-e), mat_mul(rot_z(-s), mat_mul(rot_x(p), rot_z(g)))
        )


def rot_x(th):
    c, s = math.cos(th), math.sin(th)
    return [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]]


def rot_z(th):
    c, s = math.cos(th), math.sin(th)
    return [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]


def mat_mul(a, b):
    return [[sum(a[r][k] * b[k][col] for k in range(3)) for col in range(3)] for r in range(3)]


def mat_apply(m, v):
    return [sum(m[r][k] * v[k] for k in range(3)) for r in range(3)]


def star_mean_ecliptic_longitude(prec, star, jd):
    """Mean geometric ecliptic longitude of date, in degrees.

    Proper motion applied; aberration, nutation, parallax and deflection not.
    """
    a = math.radians(star["ra_deg"])
    d = math.radians(star["dec_deg"])
    sa, ca, sd, cd = math.sin(a), math.cos(a), math.sin(d), math.cos(d)
    p = [cd * ca, cd * sa, sd]
    e_ra = [-sa, ca, 0.0]
    e_dec = [-sd * ca, -sd * sa, cd]
    years = (jd - star["epoch_jd_tt"]) / JULIAN_YEAR
    mu_ra = star["pm_ra_cosdec_mas_per_year"] * MAS_TO_RAD * years
    mu_dec = star["pm_dec_mas_per_year"] * MAS_TO_RAD * years
    v = [p[i] + mu_ra * e_ra[i] + mu_dec * e_dec[i] for i in range(3)]
    n = math.sqrt(sum(c * c for c in v))
    v = [c / n for c in v]

    eq = mat_apply(prec.matrix(jd), v)
    eps = prec.mean_obliquity_rad(jd)
    x = eq[0]
    y = eq[1] * math.cos(eps) + eq[2] * math.sin(eps)
    return math.degrees(math.atan2(y, x)) % 360.0


# ── The derivations ───────────────────────────────────────────────────────────


def dms_to_deg(dms):
    return dms[0] + dms[1] / 60.0 + dms[2] / 3600.0


def signed(deg):
    return ((deg + 180.0) % 360.0) - 180.0


class Derivation:
    def __init__(self, inputs):
        self.inputs = inputs
        self.prec = Precession(inputs)
        self.by_system = {}
        for e in inputs["group_a_precession_anchored"]:
            self.by_system[e["system"]] = ("precession", e)
        for e in inputs["group_a_rate_anchored"]:
            self.by_system[e["system"]] = ("rate", e)
        for e in inputs["group_a_calendar_year_rule"]:
            self.by_system[e["system"]] = ("calendar", e)
        for e in inputs["group_c_own_model"]:
            self.by_system[e["system"]] = ("libration", e)
        for e in inputs["group_b_star_anchored"]:
            self.by_system[e["system"]] = ("star", e)
        self.by_system["Tropical"] = ("identity", {"system": "Tropical"})

    def systems(self):
        # Declaration order in the engine's `Ayanamsha::ALL`.
        return [
            "IndianOfficial",
            "FaganBradley",
            "Krishnamurti",
            "Raman",
            "SuryaSiddhanta",
            "Yukteshwar",
            "RevatiPaksha",
            "PushyaPaksha",
            "TrueChitra",
            "ChandraHari",
            "GalacticCenter0Sag",
            "Tropical",
        ]

    def value(self, system, jd):
        kind, e = self.by_system[system]
        if kind == "identity":
            return 0.0
        if kind == "precession":
            anchor = (
                360.0 - dms_to_deg(e["svp_dms"]) if "svp_dms" in e else dms_to_deg(e["anchor_dms"])
            )
            delta = self.prec.general_longitude_arcsec(jd) - self.prec.general_longitude_arcsec(
                e["epoch_jd_tt"]
            )
            return anchor + delta * ARCSEC_TO_DEG
        if kind == "rate":
            anchor = dms_to_deg(e["anchor_dms"])
            years = (jd - e["epoch_jd_tt"]) / JULIAN_YEAR
            return anchor + years * e["rate_arcsec_per_julian_year"] * ARCSEC_TO_DEG
        if kind == "calendar":
            num, den = e["rate_arcsec_per_calendar_year"]
            rate = num / den
            year, _, _ = jd_to_calendar(jd)
            start = calendar_to_jd(year, 1, 1.0)
            nxt = calendar_to_jd(year + 1, 1, 1.0)
            elapsed = (year - e["zero_year"]) + (jd - start) / (nxt - start)
            return elapsed * rate * ARCSEC_TO_DEG
        if kind == "libration":
            period = e["mahayuga_days"] / e["revolutions_per_mahayuga"]
            phase = ((jd - e["kali_epoch_jd"]) / period) % 1.0
            if phase <= 0.25:
                tri = 4.0 * phase
            elif phase <= 0.75:
                tri = 2.0 - 4.0 * phase
            else:
                tri = 4.0 * phase - 4.0
            return -e["amplitude_deg"] * tri
        if kind == "star":
            lam = star_mean_ecliptic_longitude(self.prec, e, jd)
            return signed(lam - e["assigned_sidereal_longitude_deg"])
        raise AssertionError(f"unhandled kind {kind}")


# ── The fixture ───────────────────────────────────────────────────────────────

# Epochs the fixture samples. Chosen to cover every system's own anchor, the
# span this engine claims, and the far tails where a frame or rotation-order
# error becomes visible while staying invisible near J2000.
GRID = [
    ("Kali epoch", 588465.75),
    ("-3000 (proleptic)", 625673.5),
    ("year 285 CE", 1825401.756),
    ("year 499 CE", 1903397.0),
    ("year 1000 CE", 2086308.0),
    ("1582 Gregorian changeover", 2299160.5),
    ("B1900.0", 2415020.31352),
    ("KP anchor, 1900 Apr 13", 2415122.5),
    ("Yukteshwar anchor, 1894 equinox", 2412908.124077082),
    ("B1950.0", 2433282.42346),
    ("1985-06-15", 2446232.0),
    ("J2000.0", 2451545.0),
    ("2026-08-17", 2461269.5),
    ("J2100.0", 2488070.0),
    ("year 2299 CE", 2560863.0),
    ("year 3000 CE", 2816787.5),
]


def build_fixture(deriv):
    rows = []
    for label, jd in GRID:
        rows.append(
            {
                "label": label,
                "jd_tt": jd,
                "ayanamsha_deg": {s: deriv.value(s, jd) for s in deriv.systems()},
            }
        )
    return {
        "_comment": [
            "Generated by scripts/generate_ayanamsha.py from",
            "docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json.",
            "Do not edit by hand: a hand-typed fixture is another unsourced constant table.",
            "Regenerate with: python3 scripts/generate_ayanamsha.py",
            "Verify with:     python3 scripts/generate_ayanamsha.py --verify",
        ],
        "generator": "scripts/generate_ayanamsha.py",
        "inputs": "docs/audit/2026-08-17-ayanamsha-cleanroom/derivation-inputs.json",
        "units": "decimal degrees, mean ayanamsha, nutation excluded",
        "tolerance_deg": 1e-9,
        "systems": deriv.systems(),
        "rows": rows,
    }


def serialize(fixture):
    return json.dumps(fixture, indent=1, sort_keys=False) + "\n"


def sha256_of(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


# ── Self-checks the generator runs before it will emit anything ───────────────


def self_check(deriv):
    """Reproduce every anchor and every documented zero year, or refuse to emit.

    A generator that emits a fixture it has not checked is just a slower way of
    typing constants.
    """
    failures = []

    def check(name, got, want, tol, unit="deg"):
        if abs(got - want) > tol:
            failures.append(f"{name}: got {got!r}, want {want!r} (tol {tol} {unit})")

    ins = deriv.inputs
    for e in ins["group_a_precession_anchored"]:
        anchor = 360.0 - dms_to_deg(e["svp_dms"]) if "svp_dms" in e else dms_to_deg(e["anchor_dms"])
        check(
            f"{e['system']} anchor",
            deriv.value(e["system"], e["epoch_jd_tt"]),
            anchor,
            1e-9,
        )
    for e in ins["group_a_rate_anchored"]:
        check(
            f"{e['system']} anchor",
            deriv.value(e["system"], e["epoch_jd_tt"]),
            dms_to_deg(e["anchor_dms"]),
            1e-9,
        )
    for e in ins["group_a_calendar_year_rule"]:
        for ex in e["printed_examples"]:
            jd = calendar_to_jd(ex["year"], 1, 1.0)
            check(
                f"{e['system']} printed example {ex['year']}",
                deriv.value(e["system"], jd) * 3600.0,
                float(ex["arcsec"]),
                1e-6,
                "arcsec",
            )
    for e in ins["group_c_own_model"]:
        check(f"{e['system']} zero at Kali epoch", deriv.value(e["system"], e["kali_epoch_jd"]), 0.0, 1e-9)
    for e in ins["group_b_star_anchored"]:
        # The defining identity, at four epochs: put the star back where the
        # system says it sits.
        for jd in (1721060.0, 2415020.0, J2000, 2524594.0):
            lam = star_mean_ecliptic_longitude(deriv.prec, e, jd)
            sid = (lam - deriv.value(e["system"], jd)) % 360.0
            check(
                f"{e['system']} star identity at jd={jd}",
                signed(sid - e["assigned_sidereal_longitude_deg"]) * 3600.0,
                0.0,
                1e-6,
                "arcsec",
            )

    # Documented zero years, where the primary states one and the system is not
    # exempt.
    for group in ("group_a_precession_anchored", "group_a_rate_anchored",
                  "group_a_calendar_year_rule", "group_c_own_model"):
        for e in ins[group]:
            zy = e.get("documented_zero_year")
            if zy is None or e.get("exempt_from_zero_year_inversion"):
                continue
            got = zero_year(deriv, e["system"])
            if abs(got - zy) > 1.0:
                failures.append(
                    f"{e['system']} zero-year inversion: {got:.2f} CE against a documented {zy} CE"
                )

    if failures:
        print("SELF-CHECK FAILED — refusing to emit a fixture:", file=sys.stderr)
        for f in failures:
            print("  " + f, file=sys.stderr)
        return False
    return True


def zero_year(deriv, system, lo=1700000.0, hi=J2000):
    f = lambda jd: deriv.value(system, jd)
    if f(lo) * f(hi) >= 0:
        raise ValueError(f"{system}: no zero bracketed in [{lo}, {hi}]")
    for _ in range(200):
        mid = (lo + hi) / 2.0
        if f(lo) * f(mid) <= 0:
            hi = mid
        else:
            lo = mid
    jd = (lo + hi) / 2.0
    y, _, _ = jd_to_calendar(jd)
    return y + (jd - calendar_to_jd(y, 1, 1.0)) / 365.25


# ── Optional: re-query the catalogue ──────────────────────────────────────────

VIZIER = "https://vizier.cds.unistra.fr/viz-bin/asu-tsv"


def check_catalogue(inputs):
    """Re-query VizieR and compare against the committed catalogue rows.

    Network-dependent and therefore NOT part of `--verify`: an unreachable
    mirror must never look like agreement. Run it deliberately.
    """
    ok = True
    for e in inputs["group_b_star_anchored"]:
        cat, ident = e["catalogue"], e["identifier"]
        if not ident.startswith("HIP "):
            print(f"SKIP {e['system']}: {cat} is a paper, not a queryable catalogue")
            continue
        hip = ident.split()[1]
        url = (
            f"{VIZIER}?-source={cat}&-out=HIP,RArad,DErad,pmRA,pmDE&HIP={hip}"
            if cat == "I/311/hip2"
            else f"{VIZIER}?-source={cat}&-out=HIP,RAICRS,DEICRS,pmRA,pmDE&HIP={hip}"
        )
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                body = r.read().decode("utf-8", "replace")
        except (urllib.error.URLError, OSError) as exc:
            print(f"UNREACHABLE {e['system']}: {exc} — nothing was checked", file=sys.stderr)
            ok = False
            continue
        rows = [ln for ln in body.splitlines() if ln.strip() and not ln.startswith("#")]
        data = [ln for ln in rows if ln.strip().startswith(hip)]
        if not data:
            print(f"NO ROW {e['system']}: {ident} not returned by {cat}", file=sys.stderr)
            ok = False
            continue
        fields = data[0].split("\t")
        got_ra, got_de = float(fields[1]), float(fields[2])
        d_ra = abs(got_ra - e["ra_deg"]) * 3600.0 * 1000.0
        d_de = abs(got_de - e["dec_deg"]) * 3600.0 * 1000.0
        status = "OK" if max(d_ra, d_de) < 1.0 else "DRIFT"
        if status == "DRIFT":
            ok = False
        print(f"{status} {e['system']} {ident}: dRA {d_ra:.3f} mas, dDec {d_de:.3f} mas")
    return ok


# ── main ──────────────────────────────────────────────────────────────────────


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--output", default=DEFAULT_FIXTURE, help="fixture path")
    ap.add_argument(
        "--verify",
        action="store_true",
        help="regenerate into a temp file and compare sha256 against the committed fixture",
    )
    ap.add_argument(
        "--check-catalogue",
        action="store_true",
        help="re-query VizieR and compare against the committed catalogue rows (network)",
    )
    args = ap.parse_args()

    with open(INPUTS, encoding="utf-8") as fh:
        inputs = json.load(fh)

    if args.check_catalogue:
        return 0 if check_catalogue(inputs) else 2

    deriv = Derivation(inputs)
    if not self_check(deriv):
        return 1

    text = serialize(build_fixture(deriv))
    fresh = sha256_of(text)

    if args.verify:
        if not os.path.exists(args.output):
            print(f"MISSING: {args.output} does not exist", file=sys.stderr)
            return 2
        with open(args.output, encoding="utf-8") as fh:
            committed = fh.read()
        if sha256_of(committed) != fresh:
            print(
                f"MISMATCH {os.path.relpath(args.output, PROJECT_ROOT)}:\n"
                f"  committed sha256 = {sha256_of(committed)}\n"
                f"  fresh     sha256 = {fresh}\n"
                "The fixture no longer follows from the recorded primary inputs. Either the\n"
                "inputs changed (which is a change to the derivation and belongs in the spec\n"
                "first) or the fixture was edited by hand.",
                file=sys.stderr,
            )
            with tempfile.NamedTemporaryFile(
                "w", suffix=".json", delete=False, encoding="utf-8"
            ) as tmp:
                tmp.write(text)
                print(f"  fresh fixture written to {tmp.name}", file=sys.stderr)
            return 2
        print(f"verification ok: {os.path.relpath(args.output, PROJECT_ROOT)} matches ({fresh[:16]}...)")
        return 0

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as fh:
        fh.write(text)
    print(f"wrote {os.path.relpath(args.output, PROJECT_ROOT)} ({len(GRID)} epochs x {len(deriv.systems())} systems, sha256 {fresh[:16]}...)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
