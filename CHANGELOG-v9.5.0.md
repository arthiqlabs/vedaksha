# v9.5.0

Minor release. The Moon's light-time-corrected reads cost about half as many
ELP/MPP02 evaluations with no output bit moved, and a new Moon read returns
an analytic longitude speed. `vedaksha-vedic` gains eclipse, ingress, station
and heliacal searches, the complete shadbala of BPHS 27, bhava bala,
vimshopaka, sputa drishti, four upagrahas and five Jaimini and Parashari
dashas. Some lunar eclipse and purnimanta month answers change, each because
the old one was wrong; see "Changed answers".

## Changed — fewer lunar series evaluations, identical answers

The light-time iteration for the Moon (`vedaksha-ephem-core`, `coordinates`)
did two kinds of redundant work:

- **It evaluated the series twice at the same instant.** For the Moon the
  light-time correction is about 1.3 s, and its change between iterations is
  usually above the convergence threshold but far below the spacing of
  doubles near a modern Julian Day, so the next retarded instant was usually
  the same double as the last. The iteration now returns the evaluation it
  already has. That is exact: the Moon's vector depends only on the instant.
- **Its first evaluation was used only for its length.** Where the provider
  offers a distance-only read (`EphemerisProvider::moon_distance`, see Added),
  the first step now uses it. A guard proves, from the distance's stated error
  bound, that the iteration then takes the same path to the same answer, and
  falls back to a full first evaluation where it cannot.

Full series evaluations per read, measured over 1900 to 2100:

| Read | v9.4.1 | v9.5.0 |
|---|---|---|
| `ecliptic_position(Moon)` | 2.756 | 1.003, plus 1 distance-only |
| `apparent_positions(&[Moon])` | 6.0 | 3.015, plus 3 distance-only |

A distance-only evaluation runs the distance series alone, about 40% of a
full one. Every output bit of `ecliptic_position`, `ecliptic_position_tt`,
`apparent_position` and `apparent_positions` is unchanged, the
`apparent_positions` speed included: checked against a verbatim copy of the
v9.4.1 iteration at 4,000 instants, and against a table of v9.4.1's own
output captured before the change.

## Added

- **`coordinates::moon_apparent_position(provider, jd)`**: the Moon's
  apparent position with `longitude_speed` from the analytic derivative of the
  ephemeris (the provider's own velocity, the light-time factor `1 − dτ/dt`,
  and the rate at which the precession-nutation frame turns), at one
  position-and-velocity evaluation where `apparent_position` makes three
  light-time-corrected reads. Its position fields are `ecliptic_position`'s to
  the bit. Its speed is the derivative itself, so it differs from
  `apparent_position`'s ±0.5-day central difference by that difference's
  truncation error: worst 8.5e-3 °/day measured, with no bias. Against a
  Richardson-extrapolated difference it agrees to 1.3e-7 °/day. Per day of
  TT, where `apparent_position`'s speed is per UT1 day; the two differ by the
  rate of change of ΔT, below 1e-7 relative.
- **`EphemerisProvider::moon_distance(jd) -> Option<f64>`**, with a default
  of `None`, so an existing provider (SPK included) behaves exactly as
  before. `AnalyticalProvider` implements it from the ELP/MPP02 distance
  series (`elp_mpp02::elp_geocentric_distance`), and `CachingProvider`
  forwards it. An implementation must meet the relative error bound
  **`jpl::MOON_DISTANCE_REL_TOL`** (1e-12; the analytical one measures
  5.3e-16) and return `None` wherever `moon_position` would error.

## Changed answers

- **Lunar eclipses (`khagola::grahana`).** Earth's shadow is now enlarged by
  Danjon's rule, which NASA's catalogue uses; the old radii enlarged the umbra
  by 2% and the penumbra not at all. In a one-time run against all 228 lunar
  eclipses of NASA's 2001 to 2100 catalogue the module found every one and no
  other, with the same type, both magnitudes within 0.0009 and greatest
  eclipse within 2.2 s; the tests keep ten of them.
  Every umbral magnitude falls by about 0.008, and penumbral contacts widen by
  64 to 123 s. Three shallow penumbral eclipses missed before are found
  (2013-05-25, 2027-07-18, 2096-06-06), and 2042-09-29 is penumbral, not
  partial. Chauvenet's enlargement remains available
  (`ShadowEnlargement::Chauvenet`).
- **Solar eclipses (`khagola::grahana`).** An eclipse with the Sun below the
  horizon throughout is no longer reported, and contacts outside daylight are
  clipped to sunrise and sunset. `madhya_jd` and `magnitude` are unchanged.
- **Purnimanta months (`kala_mana::lunar_month`).** A full-moon-to-full-moon
  month now takes the name, adhika and kshaya of the amanta month whose shukla
  paksha closes it (Calendar Reform Committee report, chapter V, section 5.7).
  It was named by the sankranti it contains, which came out a month early on
  some shukla days. Amanta answers are unchanged. Lunar month boundaries move
  by at most the root finder's tolerance, about 8 ms.
- **MCP `compute_dasha` (Yogini).** Each period carries `"round": 1`. Values
  are unchanged.

## Added, vedaksha-vedic

- **`khagola`**: `grahana_with` and `ShadowEnlargement`, `SolarHorizon` and the
  new `Grahana` fields `penumbral_magnitude` and `horizon`; `graha_ingresses`,
  every sign ingress in a range, bit-identical to repeated `graha_ingress` at a
  fraction of the reads; `stations`, every retrograde station in a range;
  `next_udaya`, `next_asta` and `kalamsha_deg`, heliacal rising and setting by
  the Surya Siddhanta's kalamsha (IX.2 to 9, X.1) or a critical altitude.
- **`kala_mana`**: `lunar_month_span` and `MasaSpan`, a month with its exact
  half-open bounds; `year_turnover`.
- **`shadbala::compute_shadbala_complete`**, the shadbala of BPHS 27 part by
  part, with options where the text is silent (moolatrikona scope, equal or
  unequal horas, ayana by the verse's table or by declination, the yuddha
  winner). The existing `compute_shadbala_full` and its answers are unchanged.
  Also the parts as functions, among them `drekkana_bala_ordered`,
  `nathonnata_bala`, `paksha_bala`, `panchadha_maitri`, `saptavargaja_bala`,
  `tribhaga_bala`, the vara, masa, abda and hora lords, `ayana_bala_*`,
  `yuddha_adjustments` and `cheshta_bala_at`.
- **`cheshta`**: `cheshta_at` with sign-entry states (Anuvakra, Atichara) and
  `Cheshta::virupas`. **`mean_motion`**: `mean_longitude` and
  `mean_geocentric_motion`.
- **`bhava_bala::compute_bhava_bala`** (BPHS 27.26 to 31), **`vimshopaka`**
  (BPHS 7.17 to 27, all four schemes), **`drishti::sputa_drishti`** (degree
  aspects with the planets' special aspects), and the **`upagraha`** Kala,
  Mrityu, Ardhaprahara and Yamaghantaka (BPHS 3.66 to 70).
- **`dasha`**: Yogini beyond the first round (`compute_yogini_rounds`,
  `compute_yogini_until`); Chara antardashas (`compute_chara_antar`);
  `sthira`, `shoola` (niryana shoola) and `maheshwara`, on a shared Jaimini
  strength (`rashi_strength`). `compute_sthira` returns `None` for a chart with
  no Brahma, which the text gives no fallback for.
- `VargaType` and the vimshopaka results derive `Serialize`.

## Added, vedaksha-ephem-core

- `elp_mpp02::mean_longitude_of_date`, the Moon's mean longitude and rate.

## Documentation

- `YoginiLord::Mangala` is ruled by Chandra (BPHS 46.197), not Mars; the
  Yogini module cites the dasha chapter.
- The cheshta strengths are paired with their states as BPHS 27.22 and 23
  state them.
- `Ayanamsha::Krishnamurti` states that the crate has no separate "new KP"
  member.

## Toolchain and dependencies

Rust 1.99.1, and every dependency at its latest release.

## Compatibility

Additive, except as listed under "Changed answers". Code that builds a
`khagola::Grahana` with a struct literal must add the two new fields; nothing
in the workspace does. A consumer moving a Moon speed from
`apparent_position` to `moon_apparent_position` sees that speed shift by up
to about 1e-2 °/day, toward the true rate.
