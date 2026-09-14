# v9.2.0

Minor release. One correction that changes computed results (combustion orbs),
three additions, and a lunar-kernel speed-up that changes no output bit.

## Changed — combustion orbs now follow Surya Siddhanta

`vedaksha_vedic::combustion` (and the `compute_combustion` MCP / wasm tool)
cited BPHS Ch.7 vv.28-29 for its per-graha orbs. Those verses state the
principle — a graha's strength is lost at conjunction with the Sun and whole at
opposition — and give no degrees. The orbs are now taken from the text that
does give them: Surya Siddhanta IX.6-8 for the planets and X.1 for the Moon.

| graha | v9.1.x | v9.2.0 |
|---|---|---|
| Moon | 12 | 12 |
| Mars, direct / retrograde | 17 / 8 | 17 / 17 |
| Mercury, direct / retrograde | 14 / 12 | 14 / 12 |
| Jupiter | 11 | 11 |
| Venus, direct / retrograde | 10 / 8 | 10 / 8 |
| Saturn | 16 | 15 |

What changes for a caller:

- **Saturn between 15° and 16° from the Sun** now reads `None`; it read
  `Combust`.
- **Saturn between 5° and 5⅓° from the Sun** now reads `Combust`; it read
  `DeeplyCombust`. The deep threshold is a third of the orb, so it moved with
  it. Saturn passes this close to the Sun every year, so real charts reach this
  band.
- **Retrograde Mars between 8° and 17°** now reads `Combust`; it read `None`,
  and between 2⅔° and 5⅔° it now reads `DeeplyCombust` where it read `Combust`.
  A superior planet retrogrades near opposition, so no real chart reaches
  either case — they matter only for synthetic inputs.
- The `mars_retrograde` input is still accepted and documented as not changing
  the result, the same way `jupiter_retrograde` and `saturn_retrograde` already
  were.

The Surya Siddhanta gives these figures as *kalamsha* — degrees of time, an arc
of the equator. Vedaksha applies them to the shortest ecliptic arc between the
graha and the Sun, which is the ordinary Jyotish reading; the module
documentation says so. `DeeplyCombust` is still one third of the orb, and still
labelled a modern convention.

## Added

- **`vedaksha_astro::moon_riseset::moon_rise_set`** — moonrise, moonset and the
  Moon's upper transit for a day, with `moon_equatorial(provider, jd_ut)` to
  supply the Moon's apparent right ascension, declination and distance, and
  `moon_standard_altitude_deg` (Meeus Ch.15: `0.7275·π − 34′`, upper limb at
  the refracted horizon). The existing `riseset` search is validated for the
  Sun and documents why it is not a Moon search; this is a separate bracketing
  search whose guarantee is stated in its documentation: every crossing is
  found unless another crossing lies within one minute of it.

  Measured against a 30-second bisection scan of the same model — over a
  synthetic Moon at every whole degree of latitude from −80 to 80 for a lunar
  month (57,960 comparisons), and over the real analytical Moon for five
  observers — zero disagreements about whether an event occurs, and a worst
  instant gap of one ULP. A day costs on average about 55 Moon evaluations at
  |lat| ≤ 60 (at most 115), where a four-minute scan costs 361 before any
  refinement. **Not measured:** agreement with an external almanac. The tests
  prove the search against the model, not the model against the sky.

- **`vedaksha_vedic::muhurta::compute_yoga_end` and `compute_karana_end`** —
  the instants the panchanga yoga and karana advance, on the same Newton
  refinement as `compute_tithi_end` and `compute_nakshatra_end`. Yoga needs
  sidereal longitudes (a sum doubles the ayanamsha instead of cancelling it);
  karana, like tithi, works in either frame. Tested against
  `compute_panchanga_yoga` / `compute_karana` on either side of twelve
  successive real-ephemeris ends of each.

- **`vedaksha_ephem_core::coordinates::ecliptic_to_equatorial_deg` and
  `declination_deg`**, plus `CelestialFrame::true_obliquity()` so a caller can
  pass the obliquity the apparent-position pipeline actually rotated by. Held
  to the inverse of that pipeline's own rotation.

## Performance

The ELP/MPP02 lunar series is about a third faster on Apple silicon. Measured on
an M5 Pro, back to back against v9.1.1:

| benchmark | v9.1.1 | v9.2.0 |
|---|---|---|
| `elp_mpp02_moon` | 221 µs | 149 µs |
| `moon_scan_365d` | 79.3 ms | 54.4 ms |
| `apparent_position_full_chart` | 4.98 ms | 2.94 ms |

The cause was in the vectorised `sin`/`cos`: nine of its constants were built at
run time through `[x; 4]` repeats, which the compiler turned into
`memset_pattern16` library calls on Apple platforms, inside the hottest loop.
On non-AVX targets Vedaksha now uses a copy of the same algorithm whose
constants are fixed at compile time.

**No output bit changes.** The analytical and lunar-series bit digests are
unchanged, and a new test holds the copy to the upstream implementation bit
for bit over ~7 million inputs, including ±0, subnormals, infinities and NaNs. x86-64
builds without AVX take the same new path; their speed was not measured, and
the library call that caused the cost exists only on Apple platforms. AVX
builds keep the upstream implementation untouched and were not profiled.

## Compatibility

All additions are new items; nothing was removed or renamed. The only change in
existing behaviour is the combustion table above and the two deep-combustion
bands it moves.
