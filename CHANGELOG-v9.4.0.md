# v9.4.0

Minor release. Seven new `vedaksha-vedic` modules (P0–P6 of the downstream
feature request), two documentation-defect fixes, and one ΔT accuracy fix.
Additive, with two small behavioural notes below.

## Added — Jaimini, day-division and lunisolar-calendar primitives

- **`vedaksha_vedic::arudha`** — `arudha_pada(bhava_rashi, adhipati_rashi,
  exception)`: Jaimini Sutra I.1 counting, with the BPHS-padas-chapter vs
  Jaimini-commentary exception disagreement exposed as `ArudhaException`
  (`TenthFromLanded` / `TenthFromBhava`; identical except on 7th-landings).
  The same function gives graha arudhas (graha rashi + dispositor rashi).
- **`vedaksha_vedic::kala_khanda`** — `hora` (24 lords, Chaldean order from
  the weekday lord), `choghadiya` (16 kinds; day steps forward from the
  weekday lord's name, night starts four weekdays on and steps back two),
  `yamaganda` (Jupiter's eighth), `abhijit_muhurta` (8th of 15 daytime
  muhurtas, Wednesday omission as a parameter). Source: Muhurta Chintamani,
  Kalaprakashika. Same family as `rahu_kalam_slot`/`gulika_kalam_slot`.
- **`vedaksha_vedic::upagraha`** — `gulika`/`mandi` (Saturn's day/night
  portion instant; start/end as a parameter — that choice IS the
  Gulika/Mandi distinction in the Parashari allocation) plus `dhuma`,
  `vyatipata`, `parivesha`, `indrachapa`, `upaketu` (fixed sidereal-Sun
  offsets: Dhuma = Sun + 133°20', mirrors of absolutes thereafter). Source:
  BPHS, the upagraha chapter.
- **`vedaksha_vedic::vishesha`** — `hora_lagna`/`ghati_lagna`/`bhava_lagna`
  (1 rashi per 2.5/1/5 ghatis from the sunrise Sun), `pranapada` (15 palas
  per sign + movable/fixed/dual offset), `sri_lagna` (Janma Lagna + Moon's
  nakshatra progress × 12 signs), `indu_lagna` (9th-lord kalas counted from
  the Moon), `varnada_lagna` (odd/even Mesha-forward/Mina-backward counts).
  Source: BPHS, the special-ascendants chapter.
- **`vedaksha_vedic::kala_mana`** — `lunar_month` (amanta/purnimanta, with
  adhika for sankranti-less lunations and kshaya for double-sankranti ones),
  `vikrama_samvat`/`shaka_samvat` (Chaitra turnover), `kali_samvat`,
  `samvatsara` (mean-Jupiter 60-cycle), `ayana`, `ritu`. Source: Surya
  Siddhanta; Reingold & Dershowitz for the algorithmic form.
- **`vedaksha_vedic::khagola`** — `sun_ingress`/`graha_ingress` (incl.
  retrograde), `station`, `heliacal_rise`/`heliacal_set` (geometry with a
  caller-supplied arcus-visionis altitude), `grahana` (lunar shadow-cone +
  topocentric solar sparsha/madhya/moksha). `muhurta::refine_crossing` is now
  public as the shared solver. Source: Meeus Ch. 12/13/40/54; Surya
  Siddhanta for the classical theory.
- **`vedaksha_vedic::cheshta`** — the eight classical motion states from
  speed vs mean motion (documented conventional bands — BPHS states no
  numeric thresholds) plus `graha_yuddha` (1° war, northern-latitude or
  brightness rule). Distinct from `shadbala`'s continuous score. Source:
  BPHS, the graha-bala chapter.

All new surface follows the crate's contract: locale-free canonical values
(Julian Days, indices), ephemeris via caller callbacks, tradition variants
as enums/parameters.

## Fixed

- **Quick-start obliquity mix** (`vedaksha` lib docs): the block computed
  the RAMC with the true obliquity but passed the mean obliquity to
  `compute_chart` — every cusp silently off by up to ~9". Both now use the
  true obliquity, with the hazard commented. **Behavioural note:** any
  caller that copied the old block gets cusps/ascendant moving by up to ~9".
- **`Ayanamsha::Krishnamurti` doc** now states it is the OLD (KSK's stated)
  definition, with the arcminute-precision caveat.
- **Pre-1900 ΔT** (`vedaksha-ephem-core`): 1830–1900 evaluates the Espenak &
  Meeus polynomial fit directly instead of interpolating 5-year knots, which
  aliased the 1895 minimum (−3.6 s read vs −6.2 s observed, 2.5 s error).
  Verified against the USNO historic rows within 0.35 s over 1830–1955;
  per-era node spacing is now documented. **Behavioural note:** 19th-century
  charts move by up to ~2.5 s of time (≈1.4" of lunar longitude).

## Compatibility

Additive apart from the two behavioural notes above (sub-arcminute both).
