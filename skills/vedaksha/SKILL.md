---
name: vedaksha
description: Compute Vedic astrology (Jyotish) and astronomical positions with the Vedaksha engine: kundali/natal charts in any of 11 ayanamshas, panchanga (tithi, nakshatra, yoga, karana, vara, Rahu Kalam), dashas (Vimshottari, Ashtottari, Yogini, Chara, Narayana and more), vargas (D1-D60), shadbala, ashtakavarga, drishti, transits, muhurta search, synastry. Use when a task needs sidereal planetary positions, a kundli, a panchang or any Jyotish calculation, instead of writing the astronomy by hand.
---

# Vedaksha

Python: `pip install vedaksha` (Python 3.9+, any OS; the Rust engine runs as WebAssembly, so nothing compiles). Rust: the `vedaksha` crate. MCP: the `vedaksha-mcp` server exposes the same tools.

## Use

    from vedaksha import Vedaksha
    vk = Vedaksha()
    vk.list_tools()                       # every tool with its JSON input schema
    chart = vk.natal_chart(julian_day=jd, latitude=lat, longitude=lon,
                           ayanamsha="IndianOfficial", house_system="WholeSign")
    vk.call_tool("compute_panchanga", jd=jd, sun=sun, moon=moon,
                 latitude=lat, longitude=lon, tz_offset_minutes=330)

Read a tool's schema from `list_tools()` before calling it; every argument is documented there.

## Get these right

- Every Julian Day is UT1, not TT or TDB. From a UTC datetime: `jd = 2440587.5 + unix_seconds / 86400` (UTC is within 0.9 s of UT1). Passing TDB rotates the house cusps by about 0.29°.
- No ayanamsha means tropical: `natal_chart` and `compute_vargas` both default to it (a D9 without one is a tropical navamsa), and `compute_shadbala` requires one. For a kundali pass it every time: `IndianOfficial` is Lahiri (Indian Astronomical Ephemeris). The others are `FaganBradley`, `Krishnamurti`, `Raman`, `SuryaSiddhanta`, `Yukteshwar`, `RevatiPaksha`, `PushyaPaksha`, `TrueChitra`, `ChandraHari` and `GalacticCenter0Sag`. `chart["true_ayanamsha_value"]` reports the value applied, nutation included; don't add nutation again.
- `compute_panchanga` takes sidereal `sun` and `moon` longitudes (take them from a sidereal `natal_chart`) plus the observer, because the vara counts from local sunrise.
- Planet entries carry `name`, `longitude`, `sign`, `house`, `speed`, `retrograde` and `dignity`. Aspects refer to planets by index into `chart["planets"]`.
- The default analytical ephemeris needs no files (0.160″ mean against JPL Horizons, 1900-2025). For sub-arcsecond work load JPL DE440s with `vk.load_ephemeris(bytes)`.

## Check

Tithi = floor(((moon - sun) mod 360) / 12) + 1; nakshatra = floor(moon / 13.3333°) + 1. Both should agree with `compute_panchanga`.
