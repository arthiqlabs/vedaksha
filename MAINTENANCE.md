# Vedākṣha — Ongoing Maintenance Guide

> **Annual review recommended: April 1st each year.**
> This document lists every component that may need periodic updates. If ArthIQ Labs LLC ceases maintenance, any fork maintainer can follow this guide to keep the platform accurate and secure.

---

## 1. Earth Orientation Parameters (EOP)

**What:** The Earth's rotation is irregular and unpredictable. The IERS (International Earth Rotation and Reference Systems Service) publishes measured values for UT1-UTC, polar motion, and LOD (Length of Day) in weekly bulletins.

**Why it matters:** Sidereal time computation and coordinate transforms use UT1. Stale EOP data introduces a slowly growing error in sidereal time, which propagates to house cusps and planetary positions relative to the local horizon.

**Impact if neglected:** After 1 year of stale data, error is ~0.5–1.0 arcsecond in sidereal time. Astrological applications will not notice this. Astronomical research applications may.

**How to update:**
1. Download the latest `finals2000A.all` file from: https://datacenter.iers.org/data/latestVersion/finals2000A.all
2. Parse the UT1-UTC and polar motion columns
3. Update the embedded EOP table in `crates/vedaksha-ephem-core/src/eop.rs` (or the data file it reads)
4. Run `cargo test --package vedaksha-ephem-core` to verify sidereal time accuracy against IAU SOFA

**Frequency:** Annually (or quarterly for research-grade accuracy)

---

## 2. Leap Seconds

**What:** A leap second is occasionally inserted into UTC to keep it within 0.9 seconds of UT1. The IERS announces new leap seconds ~6 months in advance via Bulletin C.

**Why it matters:** The Julian Day ↔ UTC conversion and the TT-UTC offset depend on a complete leap second table. A missing leap second causes a 1-second error in all time conversions after the insertion date.

**Impact if neglected:** If a new leap second is announced and not added, all computations for dates after the insertion will be off by 1 second. For astrological use this is negligible. For the Moon (which moves ~0.5 arcsec/second) it matters at high precision.

**How to update:**
1. Check IERS Bulletin C at: https://datacenter.iers.org/data/latestVersion/16_BULLETIN_C16.txt
2. If a new leap second is announced, add the date to the leap second table in `crates/vedaksha-ephem-core/src/time.rs`
3. The entry is a single line: the Julian Day of the new leap second insertion
4. Run `cargo test --package vedaksha-ephem-core` — the JD↔UTC round-trip tests will catch any error

**Frequency:** Check Bulletin C every January and July (leap seconds are only inserted on June 30 or December 31). As of 2026, no new leap second has been added since December 31, 2016. The international community is considering abolishing leap seconds by 2035 (Resolution 4 of the 27th CGPM, 2022).

---

## 3. Delta T (TT − UT1)

**What:** The difference between Terrestrial Time (TT, uniform atomic time) and UT1 (Earth rotation time). Historical values are measured; future values are predicted using polynomial approximations.

**Why it matters:** Converting between the Julian Day used in ephemeris computation (TDB/TT) and the UTC/UT1 used by humans requires Delta T. The prediction polynomial drifts over time as Earth's rotation deviates from the model.

**Impact if neglected:** The prediction error grows roughly quadratically. After 5 years without update, the error is typically < 2 seconds. After 20 years, it could be 10–30 seconds. For the Moon (fastest-moving body), 1 second ≈ 0.5 arcsecond.

**How to update:**
1. Get the latest Delta T observations from: https://datacenter.iers.org/data/latestVersion/finals2000A.all (column for TT-UT1) or Espenak & Meeus tables
2. Update the polynomial coefficients in `crates/vedaksha-ephem-core/src/delta_t.rs`
3. Extend the historical measured table with new data points
4. Run accuracy tests to verify the new polynomial matches observed values

**Frequency:** Every 5 years is sufficient for astrological use. Every 1–2 years for research-grade.

---

## 4. JPL Planetary Ephemeris

**What:** NASA JPL periodically releases improved Development Ephemeris versions (DE441 → DE442 → etc.) incorporating new observations from spacecraft and ground-based telescopes.

**Why it matters:** Each new version has slightly improved accuracy, particularly for outer planets and the Moon. DE441 (currently used) covers 1800–2400 CE.

**Impact if neglected:** DE441 will remain accurate for all practical purposes for decades. The differences between DE441 and a hypothetical DE442 would be sub-milliarcsecond for inner planets — completely invisible to any astrological application.

**How to update:**
1. Check for new releases at: https://ssd.jpl.nasa.gov/planets/eph_export.html
2. If a new DE version is released, download the SPK (SPICE) file
3. Replace the file the `SpkReader` is configured to load
4. Run the full accuracy test suite against JPL Horizons

**Frequency:** Only when NASA releases a new major version (happens every 5–15 years). DE441 was released in 2021. There is no urgency to upgrade.

---

## 5. Fixed Star Catalog (Hipparcos)

**What:** The positions of fixed stars drift due to proper motion (their actual movement through space). The Hipparcos catalog includes proper motion values, so positions can be computed for any epoch. However, the catalog itself may receive corrections.

**Why it matters:** For fixed star conjunctions and parans (fixed star astrology), accurate star positions are needed.

**Impact if neglected:** The code already applies proper motion correction using Hipparcos data, so positions remain accurate indefinitely for the catalog's ~118,000 stars. No update needed unless ESA releases a major Hipparcos revision or you want to incorporate Gaia DR4+ data for higher precision.

**How to update:**
1. If incorporating Gaia data: download from https://gea.esac.esa.int/archive/
2. Replace the star data file with new positions and proper motions
3. Verify against known bright star positions

**Frequency:** Optional. Current data is sufficient for all foreseeable use.

---

## 6. Asteroid Orbital Elements

**What:** The Minor Planet Center (MPC) continuously refines asteroid orbital elements as new observations come in.

**Why it matters:** If users compute positions for specific asteroids (Chiron, Ceres, Juno, Vesta, Pallas, etc.), the orbital elements determine accuracy. Major asteroids are well-determined; minor ones less so.

**Impact if neglected:** For the ~20 astrologically significant asteroids, current orbital elements will remain accurate to within 1 arcsecond for 10+ years. For newly discovered or poorly observed asteroids, elements go stale faster.

**How to update:**
1. Download current orbital elements from: https://minorplanetcenter.net/data
2. Update the asteroid data file
3. Run position tests against JPL Horizons small-body lookup

**Frequency:** Every 2–3 years for the standard asteroid set. Only if asteroid astrology is actively used.

---

## 7. Ayanamsha Values

**What:** Some ayanamsha systems (like Lahiri) have official values that are periodically revised by the Indian government or astronomical bodies.

**Why it matters:** The Lahiri ayanamsha, used by the majority of Vedic astrologers, is officially defined by the Indian Astronomical Ephemeris. Minor revisions to its polynomial coefficients occur occasionally.

**Impact if neglected:** Revisions are typically < 1 arcsecond. Most practitioners would never notice.

**How to update:**
1. Check the latest Indian Astronomical Ephemeris publication
2. Update polynomial coefficients in `crates/vedaksha-astro/src/sidereal.rs`
3. Run Vedic chart tests to verify nakshatra boundary cases

**Frequency:** Only when the Indian government publishes a revision (rare — last major update was decades ago).

---

## 8. Rust Toolchain & Dependencies

**What:** Rust compiler updates, dependency security advisories, and breaking changes in upstream crates.

**How to update:**
```bash
rustup update
cargo update
cargo audit          # Check for known vulnerabilities
cargo deny check     # Check license compliance
```

**Frequency:** Quarterly. Security advisories should be checked monthly.

---

## 9. WASM Toolchain

**What:** `wasm-pack`, `wasm-bindgen`, and browser WASM standards evolve. The WASM Component Model is still maturing.

**How to update:**
```bash
cargo install wasm-pack --force
# Rebuild and test WASM targets
wasm-pack build --target web crates/vedaksha-wasm/
```

**Frequency:** With each release, or when targeting new WASM runtimes.

---

## Quick Reference: Annual Maintenance Checklist

```
[ ] EOP data — download latest finals2000A.all, update eop.rs
[ ] Leap seconds — check IERS Bulletin C for announcements
[ ] Delta T — compare predicted vs. observed, update if drift > 0.5s
[ ] JPL ephemeris — check if NASA released a new DE version
[ ] Asteroid orbits — update if asteroid features are actively used
[ ] Ayanamsha — check for Indian government revisions (rare)
[ ] cargo audit — run and resolve any advisories
[ ] cargo deny check — verify no disallowed licenses crept in
[ ] Run full test suite — cargo test --workspace
[ ] Run accuracy suite — compare against fresh JPL Horizons data
```

---

## What Happens If Nobody Maintains This

If Vedākṣha is abandoned entirely, here is the degradation timeline:

| Years without maintenance | Impact |
|---|---|
| 0–2 years | **No noticeable impact.** All outputs remain accurate for any astrological purpose. |
| 2–5 years | **Negligible.** Delta T prediction drifts by ~1–2 seconds. Moon position may be off by ~1 arcsecond. No astrologer would notice. Dependencies may have unpatched vulnerabilities. |
| 5–10 years | **Minor.** Delta T drift grows to 5–10 seconds. Sidereal time off by a comparable amount. House cusps could shift by a few arcseconds. Still well below astrological significance. Rust edition may require a `cargo fix --edition`. |
| 10–20 years | **Moderate for precision users.** Delta T error could reach 30+ seconds. Moon error ~15 arcseconds. Planetary positions still accurate to < 1 arcsecond. Astrologically still perfectly usable. |
| 20+ years | **Core math and ephemeris remain valid.** DE441 covers until 2400 CE. IAU precession/nutation models valid for centuries. The Rust code itself will compile and produce correct results. |

**Bottom line:** The astronomical computation engine is designed to be durable. The math doesn't expire. The data (DE441) doesn't expire until 2400 CE. The things that decay are the same things that decay in any software project — toolchains and library dependencies.

---

*© 2026 ArthIQ Labs LLC*
*Vedākṣha — Vision from Vedas*
*Contact: info@arthiq.net | https://vedaksha.net*
