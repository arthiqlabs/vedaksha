# v8.1.0

## Chara Dasha direction is tested at the 9th sign from lagna, not the lagna itself

**Severity: significant computed-value correction on a live API surface, no API shape change.**
`compute_dasha`'s Chara system determines which way the 12 dasha signs progress (forward or
backward through the zodiac) from the lagna sign. Since v8.0.0, that direction was determined by
testing the lagna sign itself against a plain odd/even rule with a four-sign "fixed exception,"
cited to Jaimini Sutras Adhyaya 1 Pada 1, sutras 25-27.

Further primary-source research (classical Jaimini sutra text and commentarial tradition; no
comparison against any other implementation informed this fix) found that those sutras actually
govern a different rule — sign-to-lord counting for dasha duration — not sequence direction. The
sutra that actually governs direction is in Adhyaya 2, Pada 3: *panchame padakramat
prakpratyaktvam charadashayam*. Its key word, *panchame* ("in the fifth," grammatically), is
decoded by a separate, explicitly-stated Jaimini sutra convention (a letter-to-numeral cipher) to
mean **nine**, not five — verified against eight other words coded the same way elsewhere in the
same sutra layer, each checked against its own commentary's stated numeric answer, all eight
matching; see `docs/audit/2026-08-31-chara-dasha-panchame-cipher.md` for the full cipher and
check. Read this way, the rule is: take the **9th sign from the lagna**, and test that sign —
not the lagna directly — against the "vishama-pada" (odd-footed: Aries, Taurus, Gemini, Libra,
Scorpio, Sagittarius) classification. If the 9th sign from the lagna is vishama-pada, Chara Dasha
counts forward; otherwise backward.

The vishama-pada classification is mathematically identical to the four-sign exception the
previous version already special-cased, so the classification logic itself did not change — only
which sign gets classified did. This reading is transmitted in a commentary called the Subodhini,
under the name Neelakantha (moderate confidence on that specific authorship — see
`DATA_PROVENANCE.md` Fix 12), and is documented in a critical edition compiled from multiple
manuscripts as the view of "many commentators" — a documented majority position, not a claim of
universal agreement.

**This changes computed Chara Dasha output for eight of the twelve lagna signs** — Taurus,
Gemini, Leo, Virgo, Scorpio, Sagittarius, Aquarius, Pisces. The four movable lagnas (Aries,
Cancer, Libra, Capricorn) are unaffected — the 9th sign from each happens to land on the same
answer the previous, direct-on-the-lagna test already gave them. See `DATA_PROVENANCE.md` Fix 12
for the full citation, confidence grading, and the specific correction to sutras previously cited.

`sign_lord_sign` (lordship/duration assignment) is unchanged by this fix.

## Performance: Wave 2 of the analytical-path optimization

Three further items from `docs/audit/2026-08-29-perf-investigation.md`, following the three that
shipped in v8.0.0. **The headline: a full natal chart is ~1.9x faster on x86_64 and ~8% faster on
aarch64.** The split is real and is explained below — it is not measurement noise.

Measured end to end, v8.0.0 to this release, criterion, both machines otherwise idle:

| benchmark | x86_64 (AVX2, shipped flags) | aarch64 (Apple Silicon) |
|---|---|---|
| `apparent_position_full_chart` | 12.159 ms -> 6.464 ms (**-46.8%**) | 6.136 ms -> 5.666 ms (**-7.7%**) |
| `apparent_positions_batch_chart` | 10.071 ms -> 5.310 ms (**-47.3%**) | 5.079 ms -> 4.668 ms (**-8.1%**) |
| `ecliptic_position(Sun)` | 40.53 us -> 12.39 us (**-69%**) | unchanged |
| VSOP87A per planet | 25.9-76.5 us -> 7.0-20.6 us (**-73%**) | +1-4% *slower* |

- **Position-only ELP/MPP02.** The lunar series computed position and velocity together, but the
  two consumers on the real runtime path (`retarded_geocentric`'s Moon branch, and the osculating
  node computation, which derives its own velocity by central difference) read only position.
  A new `EphemerisProvider::moon_position` trait method routes those callers through a
  position-only pipeline that skips the velocity half. **Bit-identical** — verified by a new
  420,002-evaluation pinned digest plus per-entry-point comparisons on raw bit patterns. This is
  the entire aarch64 win and part of the x86_64 win.
- **`search_transits` parallelized across transiting bodies**, mirroring the existing threaded
  muhurta scan (same `available_parallelism`-gated, serial-fallback shape, so wasm32 is
  unaffected). Output is identical whichever path runs — chunks are contiguous, joined in spawn
  order, and the final sort is stable; a bit-for-bit equivalence test covers it. Not separately
  benchmarked; the MCP tool's body catalog tops out at 10, so a caller requesting only a few
  bodies stays on the serial path by design.
- **VSOP87A's scalar trigonometric loop vectorized**, applying the SIMD pattern ELP has used
  since v3.1.0. This is the change responsible for the x86_64/aarch64 asymmetry above:
  `wide`'s `f64x4` is one 256-bit AVX2 register on x86_64 (genuinely four-wide) but two 128-bit
  NEON registers on aarch64, where the gain does not materialize and the array round-trip shows
  as a ~1-2% loss. It is kept because x86_64 is the deployed target for the container images and
  Linux binaries, and a 3.7x improvement there outweighs a low-single-digit regression on
  development machines.

Unlike the other two, the VSOP87A vectorization **moves output bits** — SIMD and scalar
transcendental evaluation round differently, exactly as the already-shipped ELP vectorization
does. The shift was measured, not assumed: 8,943 of the 21,915 fixture rows changed, by at most
3.20e-12 arcsec in longitude, 8.99e-13 arcsec in latitude, and ~1.6 mm in distance — nine to
eleven orders of magnitude below VSOP87A's own ~2 arcsec accuracy floor, and invisible to the
Horizons oracle, whose measured residuals are unchanged. An accuracy pre-check confirmed the SIMD
kernel matches scalar `libm` to 1-2 ULP across every real per-term argument VSOP87A can produce
over the supported date range before any vectorization was done.

**Known issue, pre-existing and not introduced here:** the bit-digest fixtures are
architecture-dependent, because the SIMD trigonometric kernel they fingerprint computes
differently on AVX2 than on NEON. Every pinned digest in this repository was measured on
aarch64 and does not reproduce on x86_64, which means the weekly Full Validation job has been
failing on that assertion since well before this release — v8.0.0 shipped in the same state.
Accuracy is unaffected and is separately gated by the Horizons oracle, which passes on both
architectures. "Bit-identical", as this project uses the term, should be read as a
within-one-architecture claim. Making the digests architecture-aware is tracked as follow-up
work and is deliberately not bundled into this release.
