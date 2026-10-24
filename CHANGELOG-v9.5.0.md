# v9.5.0

Accuracy, speed, and a much larger Vedic surface, in one release.

- **Accuracy.** Mean agreement with JPL Horizons over the measured Delta T
  era (1900 to 2025) goes from 0.103″ to 0.096″ on the SPK path and from
  0.239″ to 0.160″ on the analytical path. The analytical Moon goes from
  0.169″ to 0.094″, and to 0.055″ after 1962. Four engine defects are fixed
  to get there; see "Changed answers, the engine".
- **Speed.** The analytical Moon is about three times faster, a nine-body
  chart about three and a half times, and the long Vedic scans make a tenth
  to a twentieth of the ephemeris calls they did. A batch of MCP muhurta and
  transit searches runs in 0.46 s where v9.4.1 took 17.5 s. See
  "Performance".
- **Vedic.** The complete shadbala of BPHS 27, bhava bala, vimshopaka, sputa
  drishti, four upagrahas, the Sthira, Niryana Shoola and Maheshwara dashas,
  Yogini beyond its first round, Chara antardashas, and eclipse, ingress,
  station and heliacal searches.

Several answers change, each because the old one was wrong or less exact;
every one is listed under "Changed answers". The MCP `compute_shadbala` tool
and the WASM `compute_dasha` and `compute_shadbala` exports take new inputs;
see "Breaking changes".

## Changed answers, the engine

- **Delta T is read from the measurements and the published predictions.**
  Before 1962, the cubic spline of Stephenson, Morrison and Hohenkerk (2016,
  Table S15); 1962 to 2026-10, the USNO rows; to 2033.75, USNO's own
  predictions (`deltat.preds`); after that, the Espenak and Meeus (2006)
  expressions shifted to meet the last prediction, so there is no step. Delta
  T is now continuous through each month; it was constant within a month and
  jumped at its end. Before 1900 the change is largest: at 1700, 1650 and
  1620 Delta T is 14.6, 43.7 and 67.1 s, where it was 8.8, 50.2 and 95.4 s.
  After 2033.75 Delta T is a prediction, and the new one differs from the old
  by up to about a minute by 2099, which moves a 2099 Moon by up to 33″.
  Sources, retrieval dates and checksums are in `DATA_PROVENANCE.md`.
- **Nutation.** 30 of the 77 IAU 2000B luni-solar rows held coefficients that
  are not in the series; 17 had the sign of the main longitude term reversed.
  They moved Delta psi by up to 6.4 mas. The table now matches the IAU 2000A
  series row for row, and what remains is the model's own truncation, up to
  2.6 mas.
- **Light deflection.** The apparent place of every planet now includes the
  Sun's gravitational deflection (`deflection::solar_deflection`): 4 mas at
  90° of elongation, 0.47″ at 1°, zero for a ray that passes inside the solar
  disc. The Sun and the Moon are not deflected.
- **The VSOP87A frame.** Planet and Sun vectors are rotated into the
  equatorial frame by the matrix the VSOP87 authors publish for VSOP87A, in
  place of a rotation by the IAU 1976 obliquity alone. The analytical Sun,
  Mercury, Venus and Mars go from about 0.18″ to about 0.09″ mean.
- **Crossings are refined to the resolution of a Julian Day.**
  `muhurta::refine_crossing`, which times tithi, nakshatra, karana and yoga
  ends, sign ingresses, lunar months and sankrantis and the syzygies eclipses
  are searched from, ran Newton's method until a step was under 1e-6°. It now
  runs until the time stops changing. Answers move toward the true root by up
  to 7.7 ms for the Moon, 77 ms for a Sun ingress, 85 ms for Mars, 128 ms for
  Mercury and 16 ms along a chain of eclipses.

Measured against Horizons, 1900 to 2025, mean and max:

| Path | v9.4.1 | v9.5.0 |
|---|---|---|
| SPK (DE440s) | 0.103″, 1.187″ | 0.096″, 0.272″ |
| Analytical | 0.239″, 1.896″ | 0.160″, 1.473″ |
| Analytical Moon | 0.169″, 0.607″ | 0.094″, 0.276″ |

One body is worse at its maximum: analytical Uranus, 1.111″ to 1.201″. Its
mean improved, 0.267″ to 0.257″. After 1962 every body on the SPK path is
within 0.054″ mean; the rest is the difference between Horizons' IAU 1976/1980
precession and nutation and the IAU 2006/2000B this engine uses. The oracle
now converts Horizons' UTC tags after 1962 to TT with the leap-second count,
as Horizons itself does.

## Changed answers, Vedic

- **Chara Dasha, the two-lord signs.** For Scorpio and Aquarius with both
  lords outside the sign, the stronger lord is judged in the verses' order
  (BPHS 46.158 to 166): companions first, then the nature of the rashi, then
  exaltation, then the larger count last. The count was compared first, and
  since two signs always give different counts the other tests never ran.
  Over every placement of the grahas, Scorpio's length changes in 14.16% and
  Aquarius's in 11.42%; Narayana, an alias, changes identically. The other
  reading of where exaltation falls is `ExaltationRank::Decisive`. See
  `DATA_PROVENANCE.md`, Fix 14.
- **Lunar eclipses (`khagola::grahana`).** Earth's shadow is enlarged by
  Danjon's rule, as NASA's catalogue does; the old radii enlarged the umbra by
  2% and the penumbra not at all. Against all 228 lunar eclipses of NASA's
  2001 to 2100 catalogue the module finds every one and no other, with the
  same type, both magnitudes within 0.0009 and greatest eclipse within 2.2 s.
  Umbral magnitudes fall by about 0.008 and penumbral contacts widen by 64 to
  123 s. Three shallow penumbral eclipses missed before are found
  (2013-05-25, 2027-07-18, 2096-06-06), and 2042-09-29 is penumbral, not
  partial. `ShadowEnlargement::Chauvenet` keeps the old enlargement.
- **Solar eclipses (`khagola::grahana`).** An eclipse with the Sun below the
  horizon throughout is no longer reported, and contacts outside daylight are
  clipped to sunrise and sunset.
- **Purnimanta months (`kala_mana::lunar_month`).** A full-moon-to-full-moon
  month takes the name, adhika and kshaya of the amanta month whose shukla
  paksha closes it (Calendar Reform Committee report, chapter V, section 5.7).
  It was named by the sankranti it contains, a month early on some shukla
  days. Amanta answers are unchanged.
- **Shadbala on MCP and WASM is the complete form.** See "Breaking changes".
  Its dig bala is continuous in the arc from the cusp (BPHS 27.7) and its drik
  bala is a quarter of the sputa drishti, with Mercury's and Jupiter's aspects
  whole (27.19). `shadbala::compute_shadbala_full` and its answers are
  unchanged.
- **MCP `compute_dasha`, Yogini.** Each period carries `"round": 1`; values
  are unchanged.
- **MCP `search_muhurta` and `search_transits`** read the Moon and the Sun
  from an interpolated walk (see Performance). Measured against the direct
  path on 1,220 muhurta windows and 928 transit events, every field but the
  times is identical, and the times move by at most 0.08 ms and 0.64 ms.

## Breaking changes

- **MCP `compute_shadbala`** takes a moment and a place: `julian_day`,
  `latitude`, `longitude`, `ayanamsha` (all required), and optional
  `house_system`, `elevation_m` and `tz_offset_minutes`. It casts the chart
  itself and returns every part of BPHS 27 for the seven grahas, the birth
  moment it used, the four angular cusps, `polar_fallback` (true where the
  house system is undefined at the latitude and the chart's fallback cusps are
  the ones Dig Bala reads) and each graha's motion state. The
  earlier `planets` array and day and paksha flags are refused by name, and
  `PlanetEntry` and `parse_planet` are removed from `vedaksha-mcp`. A
  `Tropical` ayanamsha is refused, since shadbala reads the sidereal signs.
  Where the Sun does not both rise and set (polar day and night) the tool
  returns an error naming that, because there is no Hindu day to divide.
- **WASM `compute_shadbala(input_json)`** takes the MCP tool's arguments key
  for key, the same Julian Day range, and returns the same result; it refuses
  what the tool refuses, in the same order and words.
- **WASM `compute_dasha(input_json)`** takes the MCP `compute_dasha` arguments
  as one JSON object and serves all eight systems; it took three positional
  numbers and served Vimshottari only. `{"moon_longitude": m, "birth_jd": j,
  "levels": l}` reproduces the old call. `levels` outside 1 to 5, a
  `birth_jd` outside 1800 to 2400 CE and a `moon_longitude` outside [0, 360)
  are refused, with the MCP tool's messages.
- **MCP and WASM `compute_dasha`, Yogini** refuse a request for more than
  50,000 periods (`rounds`, or `until` far ahead, at deep `levels`), counting
  the rounds an `until` reaches as the engine does. The longitude inputs
  declare `exclusiveMaximum: 360`, as the handler always refused 360.

## Performance

Criterion medians, `cargo bench -p vedaksha-ephem-core --bench ephemeris`,
v9.4.1 and v9.5.0 run alternately twice on one Apple M5 Pro under a load
average of 10 to 22. The ratio from the other run is in brackets.

| Benchmark | v9.4.1 | v9.5.0 | Faster |
|---|---|---|---|
| ELP/MPP02 Moon, one evaluation | 188 µs | 56 µs | 3.3× (3.0×) |
| Apparent positions, nine-body chart | 3.73 ms | 0.97 ms | 3.8× (3.4×) |
| Same, batched (`apparent_positions`) | 2.97 ms | 1.01 ms | 2.9× (2.9×) |
| Moon, daily for a year | 72.4 ms | 17.9 ms | 4.0× (2.9×) |
| Sun, `ecliptic_position` | 6.06 µs | 2.90 µs | 2.1× |
| VSOP87A Mercury / Earth / Jupiter / Neptune | 4.6 / 6.1 / 12.8 / 6.4 µs | 1.8 / 2.9 / 6.0 / 3.5 µs | 1.8× to 2.5× |

The Sun's other run was disturbed by load and is not quoted.

End to end over MCP: one batch of 25 requests (20 `search_muhurta` windows of
30 days at four places and five dates, and five one-year `search_transits`
scans of the Moon, the Sun and Mars), the same 1,220 windows and 928 events
from both releases, best of three runs at a load of 13 to 17:

| | v9.4.1 | v9.5.0 |
|---|---|---|
| Wall time | 17.5 s | 0.46 s |

How it is done, none of it an approximation of the answer:

- **ELP/MPP02 by angle addition.** Each term's phase is a sum of multiples of
  a few fundamental arguments. The series now evaluates the sine and cosine of
  those fundamentals once and builds every term's from them by the addition
  formulas, through a prefix tree shared between terms
  (`analytical::phase_graph`), instead of one sine and cosine per term. The
  answer moves by at most 7.5e-7″ over 1610 to 2390, against the series' own
  arcsecond-level fit.
- **VSOP87A by angle addition** over the frequencies the terms share (Earth:
  624 distinct among 2,202 terms), one sine and cosine per frequency. Output
  moves by at most 5.7e-8″ over 1900 to 2100.
- **The Moon's light time.** The light-time iteration returns the evaluation
  it already has when the next retarded instant is the same double, and takes
  its first step from the distance series alone where the provider offers one
  (`EphemerisProvider::moon_distance`). This part changes no output bit,
  checked against a verbatim copy of the old iteration at 4,000 instants.
  `ecliptic_position(Moon)` makes 1.003 full series evaluations where it made
  2.756.
- **Interpolated walks for long scans** (`walk::BodyWalk`,
  `vedaksha_vedic::walked::WalkedSky`): the Moon's and the Sun's apparent place
  fitted with Chebyshev polynomials over short segments the first time a time
  inside one is asked for, within 5e-5″ for the Moon and 1e-5″ for the Sun. Provider calls per day of
  span: panchanga ends 271 to 12, `lunar_month_span` 228 to 13, eclipses 253
  to 11, moonrise 98 to 6, the MCP muhurta search 145 to 15. Planets are not
  walked, because deflection switches off where a ray crosses the solar disc
  and no polynomial follows that step.
- **`ChebyshevProvider`**, opt-in: the analytical series fitted per segment,
  within 0.1 m for the Moon and 3 m for a planet over the whole range.
  `AnalyticalProvider` is unchanged and remains the exact series.
- Fused multiply-add was measured and not adopted: no gain outside the noise,
  and it would have made output depend on the target. Without it the lunar
  series uses only IEEE multiply and add and pure-Rust libm.

## Added, vedaksha-ephem-core

- `coordinates::moon_apparent_position`: the Moon's apparent position with an
  analytic longitude speed, from one position-and-velocity evaluation. Its
  position is `ecliptic_position`'s to the bit; its speed differs from
  `apparent_position`'s central difference by that difference's truncation
  error, at most 8.5e-3 °/day, and agrees with a Richardson-extrapolated
  difference to 1.3e-7 °/day. Per day of TT.
- `EphemerisProvider::moon_distance(jd) -> Option<f64>`, default `None`, with
  the error bound `jpl::MOON_DISTANCE_REL_TOL` (1e-12; the analytical one
  measures 5.3e-16). `AnalyticalProvider` implements it from
  `elp_mpp02::elp_geocentric_distance`, and `CachingProvider` forwards it.
- `walk::BodyWalk`, `walk::WalkSpec`, `chebyshev::Chebyshev`,
  `analytical::chebyshev_provider::{ChebyshevProvider, ChebyshevConfig}`.
- `deflection::solar_deflection`, `SOLAR_RADIUS_AU`,
  `SOLAR_SCHWARZSCHILD_RADIUS_AU`.
- `elp_mpp02::mean_longitude_of_date`.

## Added, vedaksha-astro

- `riseset::sun_equatorial_deg_walked` and
  `moon_riseset::moon_equatorial_walked`, the walked forms of the Sun and Moon
  equatorial callbacks.

## Added, vedaksha-vedic

- `walked::WalkedSky`: the Moon and the Sun over a span, as the callbacks every
  scan in the crate takes. `Send + Sync`.
- `khagola`: `grahana_with`, `ShadowEnlargement`, `SolarHorizon` and the
  `Grahana` fields `penumbral_magnitude` and `horizon`; `graha_ingresses`,
  every sign ingress in a range; `stations`, every retrograde station in a
  range; `next_udaya`, `next_asta` and `kalamsha_deg`, heliacal rising and
  setting by the Surya Siddhanta's kalamsha (IX.2 to 9, X.1) or a critical
  altitude.
- `kala_mana`: `lunar_month_span` and `MasaSpan`; `year_turnover`.
- `shadbala::compute_shadbala_complete`, the shadbala of BPHS 27 part by part,
  with options where the text is silent (moolatrikona scope, equal or unequal
  horas, ayana by the verse's table or by declination, the yuddha winner, the
  recension of the special aspects); and the parts as functions, among them
  `dig_bala_at`, `drik_bala_sputa`, `drekkana_bala_ordered`,
  `nathonnata_bala`, `paksha_bala`, `panchadha_maitri`, `saptavargaja_bala`,
  `tribhaga_bala`, the vara, masa, abda and hora lords, `ayana_bala_khanda`,
  `ayana_bala_declination`, `yuddha_adjustments` and `cheshta_bala_at`.
- `shadbala_chart::{shadbala_for_chart, kala_moment}`: shadbala for a computed
  chart and a place, with the birth moment from the engine's own sunrise and
  sunset. The MCP and WASM surfaces serve this.
- `cheshta::cheshta_at` with the sign-entry states (Anuvakra, Atichara) and
  `Cheshta::virupas`; `mean_motion::{mean_longitude, mean_geocentric_motion}`.
- `bhava_bala::compute_bhava_bala` (BPHS 27.26 to 31); `vimshopaka` (BPHS 7.17
  to 27, all four schemes); `drishti::sputa_drishti`; the upagrahas Kala,
  Mrityu, Ardhaprahara and Yamaghantaka (BPHS 3.66 to 70).
- `dasha`: `compute_yogini_rounds` and `compute_yogini_until`;
  `compute_chara_antar`; `sthira`, `shoola` (niryana shoola) and `maheshwara`,
  on a shared Jaimini strength (`rashi_strength`). `compute_sthira` returns
  `None` for a chart with no Brahma graha, for which the text gives no
  fallback.
- `VargaType` and the vimshopaka results derive `Serialize`.

## Added, MCP and WASM

- `compute_dasha` gains `Sthira`, `NiryanaShoola` and `Maheshwara` (with
  `lagna_sign`, `graha_longitudes` and `karaka_scheme`), Yogini `rounds` and
  `until`, and Chara `antardashas`. A chart with no Brahma graha returns
  `"no_brahma": true` with a message rather than an error. An option that does
  not apply to the system asked for is refused.

## Documentation

- `YoginiLord::Mangala` is ruled by Chandra (BPHS 46.197), not Mars.
- The cheshta strengths are paired with their states as BPHS 27.22 and 23
  state them.
- `Ayanamsha::Krishnamurti` states that the crate has no separate "new KP"
  member.

## Toolchain and dependencies

Rust 1.99.1, and every dependency at its latest release.

## Bit digests

`analytical_bit_digest` is re-pinned on both architectures. The two lunar
series digests are now one value on aarch64 and x86_64: the series no longer
calls the SIMD trig kernel whose rounding differs between them.

## Compatibility

Beyond "Breaking changes": code that builds a `khagola::Grahana` with a struct
literal must add its two new fields. A consumer moving a Moon speed from
`apparent_position` to `moon_apparent_position` sees that speed shift by up to
about 1e-2 °/day, toward the true rate. Snapshots of any position, eclipse,
crossing time or dasha length will move as listed above.
