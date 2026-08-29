# v7.6.0

## Sidereal charts now rotate by the true ayanamsha, not the mean

Every sidereal chart, and the sidereal Sun/Moon positions used by `search_muhurta`'s quality
scoring and nakshatra-boundary refinement, previously subtracted the **mean** ayanamsha from a
tropical longitude that already had nutation-in-longitude applied (IAU 2000B). That left an
uncancelled nutation term — up to roughly 17-20 arcsec, oscillating with the ~18.6-year lunar
nutation period — live in every reported sidereal longitude, ascendant, MC and house cusp.

This is a change of which quantity is applied, not a change to any ephemeris, ayanamsha
derivation, or nutation series — all of which are unchanged and already independently validated
elsewhere in this project's test suite. The correction is internal-consistency, not a new
external comparison: `crates/vedaksha-astro/src/sidereal.rs`'s own module documentation already
stated the relationship (`true ayanamsha = mean ayanamsha + nutation in longitude`) before this
release; `compute_chart` and `search_muhurta`'s sidereal lookups simply were not using it.

**What changed:**
- New `vedaksha_astro::sidereal::true_ayanamsha_value` and `true_tropical_to_sidereal` functions
  (mean ayanamsha plus nutation-in-longitude). `ayanamsha_value` and `tropical_to_sidereal`
  themselves are unchanged — they remain the standalone mean-only quantities the module has
  always documented.
- `compute_chart`'s internal sidereal rotation, and `search_muhurta`'s sidereal Sun/Moon lookups,
  now use the true ayanamsha.
- The `ayanamsha_value` field reported by `compute_natal_chart` and `compute_vargas` (both the
  MCP and wasm surfaces) now reports the **true** ayanamsha — the actual rotation applied to the
  chart — rather than the mean. This keeps the field's own long-standing contract ("the chart
  reports the rotation it applied") true; before this release that contract and the mean-only
  value happened to coincide only because the rotation itself was (incorrectly) mean-only.

**Release-policy note, following the same reasoning as `CHANGELOG-v7.5.0.md`:** this changes the
returned magnitude of a value already served by two published tools' output on every sidereal
request, by up to ~17-20 arcsec. No function was removed or renamed, and no signature changed, so
this ships as a **minor** version bump rather than major.

**Scale, so it is not over- or understated:** ~17-20 arcsec is roughly 0.14-0.17% of a nakshatra
pada (3°20′ = 12,000″) — a small correction relative to a nakshatra boundary, but a real one for
callers computing near a boundary, and the first fix to reach Vedaksha's sidereal output path
specifically (as opposed to its tropical ephemeris, which is unaffected).

## Chara-karaka module: citation clarified, boundary check hardened

The chara-karaka module's citation previously read only "Jaimini Sutras 1.1" — one bare line
covering both the karaka-ranking chain and the Rahu-reflection rule, without distinguishing them.
Primary-source review found the ranking chain is Jaimini's own sutra text (Adhyaya 1 Pada 1,
commonly numbered around 1.1.11 for Atmakaraka, though numbering varies by tradition), and that
Rahu's inclusion in the 8-karaka scheme is explicit in Jaimini's own root sutra. The specific
reflection rule for a retrograde body's degree is a traditional application of the same
reverse-motion (Apasavya) principle Jaimini's own text uses for Ketu's Argala a few sutras
earlier; the literal "30 minus degree" arithmetic is attested in Parashara's *Brihat Parashara
Hora Shastra* (chapter on Karakatwas), not as a separately numbered Jaimini sutra. The module's
citation now states both sources separately, and states plainly the two questions this review
could not resolve (the exact sutra numbering; whether Rahu's inclusion is unconditional or a
tie-break-only fallback per some readings) rather than presenting the current behavior as
uncontested. No ranking behavior changed.

Separately, `rahu_degrees_in_sign`'s sign-boundary check was hardened from exact floating-point
equality (`d == 0.0`) to a `1e-9°` tolerance, matching this project's own established tolerance
for "should be exact" degree comparisons elsewhere. A longitude that should land exactly on a
sign boundary can arrive with a few-ULP residual rather than a literal zero; the old check would
have sent such a value down the reflection branch, producing a near-maximal rank instead of the
intended near-minimal one.
