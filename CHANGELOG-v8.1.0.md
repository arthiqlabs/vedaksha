# v8.1.0 (draft)

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
