# Ayanamsha Primary-Source Derivation — Spec

**Module:** `crates/vedaksha-astro/src/sidereal.rs`
**Precedent:** `docs/audit/2026-05-09-elp-mpp02-cleanroom/` — same two-agent firewall,
same manifest discipline, same audit-dir shape.

This document is the **authoritative and only** input to the implementation agent. It
states every anchor, model, convention and declared assumption needed to compute the
sidereal surface from primary sources. It deliberately contains no value previously
shipped by this project and none from any other implementation; see §0.

Verified by `scripts/check_spec_hygiene.py` before handoff.

---


## 0. The governing rule

**The direction of derivation is what makes this clean-room, not the citations attached to
it.** Computing forward from a documented primary and accepting the result is primary
research. Tuning toward a known output is reverse engineering, and it remains reverse
engineering however the result is cited afterwards.

The file being replaced already demonstrates the failure mode. Fifteen of its arms carry
comments of the form *"adjusted by ±X° … to match independent reference"*. Whatever that
reference was, those values were produced by aiming at an answer.

Therefore, binding on every phase of this work:

> **No acceptance test may reference the values Vedaksha currently ships, Swiss Ephemeris,
> or any other implementation.** A derived value is accepted because it follows from its
> stated inputs, not because it resembles anything.

The comparison against the current constants happens **once, after the derived values are
frozen and committed** (§6.4). It exists to size the migration note. It is an observation,
never a gate. If a derived value lands close to what we ship today, that is a welcome
result and nothing more; if it lands 16″ away, we ship the derived value.

**This spec contains no current constant, no value from any other implementation, and no
delta from which either is recoverable.** The implementation agent must never see a target.

This was **not** true of the 2026-08-17 draft, and the failure is instructive. That draft
made the same declaration while stating two shipped constants verbatim and several deltas
from which the rest fell out by one subtraction — because a sentence of the form "reproduces
the shipped value to within *d*" *is* the shipped value once the derived value sits in the
same paragraph. A declaration is not a control. The observations were moved to
a quarantined companion note, whose destination is the migration note written
after the freeze commit.

**Pre-handoff hygiene gate (mandatory).** Before this document is given to any agent or
committed, run `scripts/check_spec_hygiene.py` over it. The gate greps for the previously
shipped digit strings and for the comparison phrasings that make a delta recoverable. **The
pattern list lives in that script, never in this file** — an inline list would make the
document match its own check and would itself reintroduce the strings it exists to exclude.
A hit is a blocking defect, not a style note.

**Do not hand the agent** `the internal provenance audit note`, the review reports, or the quarantined observations file — all four contain current values.

---

## 1. Scope

### 1.0 The list is rebuilt from primaries, not inherited 

The existing enum is itself an artefact, not merely its values. Labels such as
`SsDrevJul`, `LahiriVp285`, `BabylonianEtpsc`, `GalacticEquatorMidMula`,
`GalacticCenterGalAlign`, `Skydram`, `ValensMoon`, `UshaShashi`, `DjwhalKhulTibetan2` and
`AyanamshaOfDate` are software enumeration identifiers; no siddhanta, journal or committee
names a system that way. Searching for a primary that defines "SS Drev-Jul" would presume
the list, which is reverse engineering one level up from the numbers.

**Therefore: enumerate the sidereal systems that primary literature actually defines, and
let our list fall out of that.** The names that emerge are ours. Swiss Ephemeris
*documentation* is on the forbidden list alongside its source, for the same reason — the
overlap must not be checked by looking.

Rebuilding immediately collapses labels. A distinct enum variant is not evidence of a
distinct system:

| Existing labels | Primary reality |
|---|---|
| `SuryaSiddhanta`, `SuryaSiddhantaMean`, `SsCitra`, `SsDrevJul` | **one** system — SS Ch. 3 vv. 9–12 |
| `Krishnamurti`, `Krishnamurti2` | **one** — KSK *Vol-I* p. 140 |
| `Lahiri`, `Lahiri1940`, `LahiriVp285`, `AyanamshaOfDate` | **one** — IAE 2022 p. 380 |
| `Aryabhata`, `Aryabhata528` | unknown until the text is read |

### 1.1 Rebuilt enumeration — status

**Established. Primary held, definition complete.**

1. **Indian official (Chitra-paksha / "Lahiri")** — IAE 2022 p. 380: anchor 23°51'25".53 at
   J2000.0, propagated by Capitaine et al. (2003). Defining rule — Spica at 180° — from
   CRC 1954.
2. **Fagan-Bradley** — Fagan & Firebrace 1971, p. 13 (SVP 335°57'28.64" at 1950.0) and
   p. 16 (ayanamsa = 360° − SVP).
3. **Krishnamurti (KP)** — KSK *Vol-I* p. 140 (22°22'00" at the 1st of Chitra, 1900),
   derived from Rajan 1941 p. 110 by a flat +2'; rate 50.2388475"/yr per KSK.
4. **Surya Siddhanta** — SS Ch. 3 vv. 9–12: libration of ±27° about the fixed point near
   ζ Piscium, 600 revolutions per Mahayuga (period 7200 yr), max rate 54"/yr. **This is a
   trepidation, not a linear precession**; an implementation that linearises it has changed
   the definition. Zero point is Revati-anchored, per the text.

5. **Yukteshwar** — *The Holy Science* (1894), public domain,
   <https://archive.org/details/the-holy-science-by-swami-sri-yukteswar>. Verbatim: *"The
   astronomical reference books show the Vernal Equinox now to be **20°54'36" distant from
   the first point of Aries (the fixed star Revati)**"*, stated for the vernal equinox of
   1894. Complete anchor — value, epoch and zero point (Revati). Propagated to J2000 with
   P03 this gives **22°23'18.7"**. (The adversarial review: this propagation is wrong — his anchor is his own
   model output, 20°54'36" = exactly 1394 x 54", so §2.3 requires his own 54"/yr. Recompute
   and re-run the §6.2 inversion, which under P03 misses the documented zero year by a
   century.)

6. **Star-anchored pakshas — SS Ch. 8 vv. 1–9. EXTRACTED.**

   The table need not be transcribed at all: **v. 1 states the encoding rule**, so the 27
   positions are *computed* from the verse, which is far more robust than reading OCR or
   page scans.

   > "Now are set forth the positions of the asterisms (bha), in minutes. If the share of
   > each one, then, be multiplied by ten, and increased by the minutes in the portions
   > (bhoga) of the past asterisms (dhishnya), the result will be the polar longitudes
   > (dhruva)."

   `dhruva(n) = 10 × share(n) + 800′ × n`. Shares from vv. 2–3 and 5–6; polar latitudes
   (*vikṣepa*) from vv. 6–9. The result **self-validates**: with no tuning, Chitra lands on
   exactly 180°00′ and Revati on exactly 359°50′ — the two canonical anchors.

   | anchor | dhruva | vikṣepa | consequence |
   |---|---|---|---|
   | **Revati** | **359°50′** | **0** | latitude zero ⇒ dhruva **is** the ecliptic longitude; no reduction |
   | **Chitra** | **180°00′** | **−2°** | reduction **required**; its ecliptic longitude is *not* 180° |
   | Pushya | 106°00′ | 0 | clean anchor, *if* Pushya-paksha is attested as a system |
   | Mula | 241°00′ | −9° | reduction required |

   **The critical finding, and it is not a detail.** SS Ch. 8 vv. 14–15 state the
   coordinates are *polar*. Revati's polar latitude is zero, so Revati-paksha is exact as
   read. **Chitra's is −2°, so "Chitra at 180°" in the Surya Siddhanta is a statement about
   polar longitude, while CRC 1954's "180° away from the star Citra (Spica)" is a modern
   ecliptic statement. These are different conditions and they do not yield the same zero
   point.** Any implementation that treats them as one has silently merged two systems.
   The polar→true reduction formulas are given in the same public-domain source.

   Still open: whether Pushya-paksha is attested as an *ayanamsha system* rather than
   merely a star with a listed position. A position is not a definition.

7. **Galactic Centre at 0° Sagittarius — DERIVED.** This one qualifies where the other
   galactic variants do not, because **the condition is self-describing**: Sgr A* at
   sidereal 240°. No astrologer's publication is needed to know what it means, and the
   astrometry has peer-reviewed primaries (Reid & Brunthaler VLBA; ICRF3, ApJ/AJ).

   Sgr A* J2000 ICRS α = 266.416837°, δ = −29.007811° → tropical ecliptic longitude
   **266°51'06.2"** at J2000, hence **ayanamsha(J2000) = 26°51'06.2" = 26.851731°**.

### 1.3 Corrections from adversarial review — these SUPERSEDE the items above

**1.3.1 Raman — ADDED. This was the largest research miss.**

Primary: B. V. Raman, *A Manual of Hindu Astrology* (1935), Ch. III Art. 49, free and
unrestricted at <https://archive.org/details/in.ernet.dli.2015.509148>. Verbatim:

> "**Determination of (Approximate) Ayanamsa.** — (1) Subtract 397 from the year of birth
> (a.d.) (2) Multiply the remainder by 50⅓″ and reduce the product into degrees, minutes and
> seconds."

Both printed worked examples reproduce **exactly**: 1912 → 76,255″ = 21°10'55″; 1918 →
76,557″ = 21°15'57″. So the rule is `ayanamsa(year) = (year − 397) × 151/3 ″`, zero year
397 CE, rate exactly 151/3 ″/yr.

Three properties must be carried, not smoothed:
- **Raman himself labels it "(Approximate)"** — in the article title. Do not present it as
  more precise than its author does.
- **It is year-granular.** Art. 49 takes "the year of birth"; the point *within* the year is
  unspecified. Art. 50 ("Ayanamsa for odd days") says the intra-year change "can conveniently
  be neglected" but may be added. The epoch-within-year is therefore a **declared assumption**
  (§2.6) — recommend 1 January, stated.
- Raman explicitly declines to defend 397 over the alternatives he lists (361, 394, 498,
  559 CE), calling the question one of "considerable doubt". That is his system's basis, and
  the honest citation says so.

**1.3.2 Surya Siddhanta — supersedes item 4. Not implementable as previously written.**

- **The folding is a ZIGZAG, not a sinusoid.** Verse 10's *bhuja* rule ("multiply by three,
  divide by ten", i.e. 90 → 27) is linear folding. A sinusoidal reading diverges by ~3.6° at
  J2000. "Max rate 54″/yr" was misleading phrasing: under the zigzag the rate is **constant**
  at 54″/yr within each limb.
- **The phase origin is the ahargana from the Kali epoch** — 600 libration revolutions per
  Mahayuga of 1,577,917,828 days, period 2,629,863.05 d ≈ 7200.17 yr. The zero crossings fall
  at the Kali epoch and one half-period later at **499 CE**, which I have confirmed
  numerically.
- **DECLARED ASSUMPTION (DA-2).** The formula's 499 CE zero crossing disagrees with the
  Ch. 8 star table's own epoch (the initial point 10′ east of ζ Piscium matching the equinox
  around 560–570 CE) by roughly 1°. Chapters 3 and 8 of the same text do not agree; Burgess
  calls the passage incoherent. Whichever is adopted is an assumption and must be declared —
  §11 previously marked this system "no assumption", which was wrong.
- **Sign and direction convention must be pinned in the spec.** My scratch implementation of
  the zigzag returns the wrong sign at J2000, which is exactly the failure an implementer
  would reproduce if left to infer it. Specify the direction of travel after 499 CE
  explicitly, with a worked check value.

**1.3.3 Yukteshwar — supersedes item 5. Propagate at his own 54″/yr, not P03.**

His anchor is his own model output, not an observation: **1394 × 54″ = 75,276″ = 20°54'36″**,
exactly the figure printed. The Holy Science says so in the same passage ("by calculation it
will appear that 1394 years have passed"). §2.3 therefore requires his own rate.

Adopted: `ayanamsa(year) = (year − 500) × 54″/yr`, zero year 500 CE. Propagating his anchor
with P03 instead inverts to a zero year near 390 CE — **failing §6.2 by a century** — which
is the acceptance test doing its job.

**1.3.4 Krishnamurti — resolves the §4.1 / §6.2 contradiction.**

KSK's three published numbers are **mutually inconsistent**: his anchor (22°22'00″ at the 1st
of Chitra 1900) divided by his stated rate (50.2388475″/yr) gives a zero year of **297.5 CE**,
against the **291 AD** he states — a 6.5-year, ~5-arcminute disagreement.

Adopt the **anchor plus his stated rate** (together these are the operative definition, and
they are what his table reproduces), record the zero-year inconsistency as a documented
property of the system, and **exempt KP from §6.2** with that reason stated. Do not
reconcile it by choosing a rate that makes the inversion work — that would be tuning toward
an answer, which is the thing this spec exists to prevent.

Also correct: the claim that "the primaries publish arcminutes only, ±30″ quantisation" is
**false for Rajan**, whose body tables give arcseconds (Example 50: base 22°40'39″ plus
Table 7/8 increments). Only his appendix is arcminute. Pull Table 7/8 from the scans; the
±30″ uncertainty floor was overstated by an order of magnitude. KSK's own table is
arcminute — that part stands. The mid-April epoch is an **inference** from the +2′
correlation, hence a declared assumption.

**1.3.5 Re-opened drops — evaluate before any of these are final.**

Each was recorded as "no primary located" where "no search performed" was the truth:
Chandra Hari / True Mula (peer-reviewed, *IJHS* 33(4) 1998, free INSA PDF — better attested
than a system already kept); Huber 1958 and Kugler 1900, with Britton 2010 as the modern
determination; **True Chitra (Spica at 180°)**, never considered at all despite the largest
user base and a self-describing condition; Ernst Wilhelm's mid-Mula; Sassanian via Mercier;
DeLuce 1963; Usha-Shashi 1978.

**Standing rule from this:** a drop is only final when the spec records *what was searched
and where*. An undocumented negative search is not a finding.

### 1.4 Re-opened evaluations — RESOLVED 2026-08-17

Each of these had been recorded as "no primary located" where "no search performed" was the
truth. Every one is now either added with a primary or dropped with a **documented search
record**, per the §1.3.5 standing rule.

**ADDED — 10. True Mula (Chandra Hari).** K. Chandra Hari, *"On the Origin of Sidereal
Zodiac and Astronomy"*, **Indian Journal of History of Science 33(4), 1998**, free from INSA
(<https://insa.nic.in/writereaddata/UpLoadedFiles/IJHS/Vol33_4_1_KCHari.pdf>). The abstract
states the defining condition outright:

> "the initial point of the ancient Babylonian Zodiac was the same as that of Hindu's which
> had the star **Mūla of sidereal longitude 240°** as fiducial"

with the keyword line identifying Mūla as **λ Scorpii**. Complete, self-describing, and the
proposer names the star himself — so, like Pushya-paksha, **no declared assumption is
needed**. Peer-reviewed and freely available, i.e. better attested than a system already
retained. It is a true star-tracking system and takes the §2.8.2 convention.

Note this is **not** the same as SS Ch. 8's Mula (dhruva 241°00′, vikṣepa −9°, a *polar*
figure). Chandra Hari assigns an *ecliptic* longitude of 240° to λ Sco. Distinct conditions;
do not merge them.

**ADDED — 11. True Chitra (Spica at 180°).** Never evaluated before, despite being the
star-tracking system with the largest user base. The condition is self-describing in exactly
the way §1.1(7) accepted for Sgr A*: Spica (α Virginis) fixed at sidereal 180°, tracked live.
No proposer's text is needed to know what it means, and Spica's identity is not contested.

Two honesty notes that must travel with it:
- It is **distinct from the Indian official system**, which IAE 2022 defines by a J2000
  anchor and the 285 CE coincidence — *not* by Spica. The "Chitra-paksha" name implies the
  star; IAE's operative definition does not use it. Fixing Spica strictly is a different
  rule from evaluating a polynomial.
- The "initial point 180° from Spica" statements in the CRC report sit in **summaries of
  correspondents' submissions, not in the Committee's recommendations**. Its attestation is
  as a named tradition whose condition is self-describing — not as a Committee-adopted rule.
  Earlier drafts of this spec over-attributed that to CRC; corrected here.

**DROPPED — the Kugler triple.** *Sternkunde und Sterndienst in Babel* (1900) is public
domain and on archive.org (`gri_33125008379030`, `sternkundeundst00kuglgoog` and others), so
availability was never the obstacle. The obstacle is categorical: **Kugler analysed Babylonian
star data; he did not define a sidereal zero point for chart calculation, still less three of
them.** "Babylonian (Kugler Star 1 / 2 / 3)" is precisely the software-enumeration artefact
§1.0 describes — a source of data reified into three named systems. Dropped on grounds of
what the source is, not on failure to find it.

**DROPPED — Huber 1958, on principle rather than on availability.**

*"Über den Nullpunkt der babylonischen Ekliptik"*, **Centaurus 5, 192–208**. I could not
obtain it: Centaurus is Wiley-paywalled, Britton 2010 (*AHES* 64, 617–663) is Springer-
paywalled, and two open-access routes returned 403. That failure alone would only make the
drop provisional. What makes it final is the following test, which the Huber case forced into
the open and which the spec now adopts generally.

> **The definition/determination test.** A system qualifies only if someone **stipulated** a
> zero point. It does not qualify if a scholar **estimated** where a historical zero point
> lay.

A stipulation is a definition: Fagan's SVP, Rao's δ Cancri at 106°, Chandra Hari's λ Sco at
240°, KSK's anchor, Raman's 397 CE, the Calendar Reform Committee's 23°15′, a siddhanta's own
verses. It is exact by construction, it cannot be *wrong*, and it does not change when new
evidence arrives.

A determination is a measurement: Huber's and Kugler's and Britton's reconstructions of where
Babylonian longitudes were reckoned from. It carries uncertainty, it has a revision history,
and it is *superseded* rather than amended — Britton 2010 is the current determination
precisely because Huber's is no longer it. Building an ayanamsha on one means shipping a
historical estimate, with its error bars suppressed, that the field has already moved past.

**The evidence for this is in the disagreement itself.** Two secondary accounts of Huber's
figure give **4°22′** and **4°28′** for the −100 zero point — six arcminutes apart. A
definition does not have competing values; a measurement does. That spread is not a citation
problem to be resolved by finding a better source, it is the signature of the category.

This test **confirms the Kugler drop** on the same grounds and **excludes Britton** too, so
no Babylonian system is retained. It also explains why the systems that *are* retained
survive: every one of the eleven rests on somebody's stipulation.

Recorded honestly: if the project owner wants a Babylonian system as a product feature, the route is to
obtain Britton (the current determination), ship it as an explicitly modern scholarly
reconstruction with its uncertainty stated, and accept that it will be superseded. That is a
product decision, not a provenance one.

**DROPPED, with search records.** Searched archive.org by title on 2026-08-17; zero results
for each. Their definitions exist only in books that must be bought and read, and no
description obtained from the open web is admissible — the adversarial review established that most secondary
pages on this topic are derivatives of Swiss Ephemeris documentation.

| System | Primary that would settle it | Search result |
|---|---|---|
| De Luce | *Constellational Astrology* (1963) | not on archive.org |
| Usha-Shashi | *Hindu Astrological Calculations* (1978) | not on archive.org |
| Gil Brand | *Himmlische Matrix* (Chiron Verlag) | not on archive.org |
| Wilhelm mid-Mula | Wilhelm's own essays/software docs (2006) | no citable primary located; web descriptions contamination-hazardous |
| Sassanian | Mercier, *Studies on the Transmission of Medieval Mathematical Astronomy* (2004) | commercial monograph, not obtained; the "10′ east of ζ Psc / ~560 CE" attribution reached me only via secondary pages and is **not** verified |

None of these five is refuted — each is *unverified*, which under the ratified rule means
dropped. If any is later wanted, the route is to obtain the book, not to search harder.

### Revati-paksha vs Surya Siddhanta — RESOLVED: distinct, by exactly 10′

They are separate systems, and the Surya Siddhanta is the **minority** position among
Indian authorities, not the norm. On Revati's junction-star, Burgess:

> "all authorities agree in placing it upon the ecliptic and **all excepting our treatise
> and the Cakalya** make its position exactly mark the initial point of the fixed sidereal
> sphere"

| system | zero point | Revati's longitude |
|---|---|---|
| **Revati-paksha** (the majority of authorities) | *at* ζ Piscium | 0°00′ |
| **Surya Siddhanta** (+ Śākalya-saṁhitā) | **10′ east of** ζ Piscium | 359°50′ |

The offset is exactly the complement of the dhruva 359°50′ computed from the v. 1 rule, so
the two readings of Ch. 8 agree. **A three-way internal cross-check confirms it**: the text
states the SS zero point coincided with the vernal equinox about A.D. 560, and that
ζ Piscium itself did so in A.D. 572. Ten arcminutes at ~50.2″/yr is 12.0 years, and
572 − 560 = 12. The verse-derived offset and two independently stated dates agree exactly.

**Both stay, and the naming must make the relationship explicit** — they differ by 10′ in
anchor *and* by propagation (SS uses its own trepidation, Ch. 3 vv. 9–12).

**Ratified 2026-08-17: Revati-paksha is retained, and the ζ Piscium identification is
stated explicitly.** See §2.6.

*Also record:* Ch. 8 assigns Revati a vikṣepa of 0, while the commentary notes ζ Piscium
actually has ~13′ of south latitude. The text idealises it. That is a property of the
system, not an error to correct.

### A principled line on "true" star-tracking variants

The Surya Siddhanta gives **coordinates, not modern catalogue identities**. "Revati =
ζ Piscium" is a scholarly identification made by translators, not a statement of the text.
Any variant that tracks a live star therefore rests on an inference the primary does not
supply, and should be dropped or explicitly flagged as an identification we assert.

Sgr A* is categorically different — an unambiguous radio source with its own astrometry,
not a modern guess at what an ancient text meant. It stays.

This kills the star-tracking variants while leaving the textual systems (SS's own
trepidation, the IAE polynomial) untouched, since neither needs a star identified.

**Dropped, with evidence.**

- **Aryabhata and Aryabhata 528.** The Aryabhatiya contains **no precession or libration
  rule**. Clark's edition records that the sole basis is a second-hand quotation
  (*caturvimsaty amsais cakram ubhayato gacchet*) given by Colebrooke from Munisvara, and
  states plainly: *"No such statement is found in our Aryastasata … The quotation should be
  verified in the unpublished text in order to determine whether Colebrooke was mistaken."*
  An unverified attribution is not a definition. Both variants go.

- **The remaining galactic variants** — Gil Brand's, Raymond Mardyks' 1991 "galactic
  alignment", "galactic equator true", "mid-Mula", and Chandra Hari's True Mula. Named
  proposers exist, which is more than Tier E had, but **no primary publication was located
  for any of them**, and unlike Sgr A*-at-240° their conditions are not self-describing —
  you cannot compute them without reading the proposer. Per the ratified rule they drop.
  (Attribution here came from secondary web sources, which is itself disqualifying for a
  provenance file.)

- ~~Pushya-paksha~~ — **investigated further 2026-08-17; the earlier recommendation to drop
  was WRONG and is withdrawn.** It rested on secondary commentary rather than the primary.
  See §1.1 item 8: the primary is locatable, free, complete, and needs no declared
  assumption. It is retained.

**Doubtful; expected to fall out.**

- **All galactic variants.** A star or pole *position* is not an ayanamsha definition —
  someone must have proposed the zero point. The IAU 1958 pole (Blaauw et al. 1960) and
  Sgr A* have citable primaries for the *astronomy*; the systems built on them need a
  locatable proposer, and Brand / "galactic alignment" / "true" / "mid-Mula" read as labels.
- **Hipparchos** — a contested reconstruction of Hipparchus's catalogue, not a definition.
  Recommend dropping.

**Expected surviving surface: 6–9 systems**, not the ~22 previously projected. Smaller than
v5 advertises, and every entry traceable to a chapter, a star, or a committee.

## 2. Frames and conventions

These must be fixed before any value is computed, because a frame error is
indistinguishable from a wrong constant.

**2.1 Mean vs true equinox.** The engine's `general_precession_in_longitude` is a
**mean-equinox** quantity: precession only, no nutation. Any anchor value stated against
the *true* equinox of its date must therefore have the nutation in longitude Δψ at that
instant removed before it enters the formula. This is a required frame conversion, not an
adjustment, and it must be applied only where the primary establishes that the anchor is a
true-equinox value. **Where the primary does not say, this must be resolved from the
primary and not assumed in either direction** (see §9.2).

**2.2 Time scale.** All Julian Days internal to the derivation are TT. Anchors given in
civil time convert via the ΔT model already in `vedaksha-ephem-core`. For anchors in the
1900–1960 window ΔT is ~25–35 s, which is worth ≪0.01″ of precession and cannot matter;
record the conversion anyway so the chain is complete.

**2.3 Propagation model.** Each tradition is propagated with the precession theory its own
definition implies, not with a single house model:

- IAU 1976 — Lieske, Lederle, Fricke & Morando (1977), *A&A* 58, 1–16, eq. (A2).
- Newcomb — Newcomb (1898), *A Compendium of Spherical Astronomy*, p. 226. Public domain.
- IAU 2006 P03 — Capitaine, Wallace & Chapront (2003), *A&A* 412, 567–586.

Where a tradition states its own rate (KP does), the derivation must use the stated rate
and record the divergence from the modern model as a documented property of that system —
not silently substitute the better model.

**2.4 Proper motion — the "true" vs "mean" distinction.** For star-anchored systems this is
the definition, not a detail. A *true* variant fixes the star at its assigned sidereal
longitude **at every epoch**, so the ayanamsha tracks the star's actual apparent motion,
proper motion included. A *mean* variant fixes the relationship once and propagates by
precession alone. Which one each system is must be taken from its primary, and the
`True…` prefix in the enum is **not** evidence — the enum is part of the artefact under
review.

---

### 2.6 Declared assumptions

Some derivations rest on inferences the primaries do not themselves state. **These are
declared in the source and in the audit dir, never absorbed silently.** A hidden assumption
is indistinguishable from a fabricated constant to anyone auditing later, which is how the
current file got into trouble.

**DA-1 — Revati is ζ Piscium.** The Surya Siddhanta gives coordinates, not modern catalogue
identities. Identifying the yogatārā of Revati with ζ Piscium is an identification made by
the commentary, not a statement of the text — Burgess records that the star "is by all
authorities identified with ζ Piscium", so it is universally attested, but attested
identification is still identification. Revati-paksha cannot be computed without it.
Vedaksha asserts it, on the authority of the commentary, and says so in the doc comment.

Systems needing **no** such assumption, and therefore stronger: the Indian official
ayanamsha (IAE polynomial, J2000-anchored), the Surya Siddhanta (its own trepidation), and
Fagan-Bradley (a published SVP). The Galactic Centre system needs none either — Sgr A* is a
radio source with its own astrometry, not a modern guess at an ancient referent.

### 2.7 SS Ch. 8 — complete, derived from the verse rules

No table was transcribed. `dhruva(n) = 10 × share(n) + 800′ × n` from v. 1, shares from
vv. 2–3 and 5–6, positional rules from vv. 4–5, vikṣepa from vv. 6–9. The four positional
cases resolve to the canonical values, which is a check on the reading:

| U-Ashadha | Abhijit | Sravana | Dhanishtha |
|---|---|---|---|
| 260°00′ | 266°40′ | 280°00′ | 290°00′ |

U-Ashadha's junction star falls inside P-Ashadha's portion; the text permits a yogatārā to
sit outside its own bhoga, and this is not an error to be corrected.

8. **Pushya-paksha — ESTABLISHED.** Primary: P.V.R. Narasimha Rao, *"Introducing
   Pushya-paksha Ayanamsa"*, freely published at
   <https://vedicastrologer.org/articles/pp_ayanamsa.pdf> (6 pp.). Citing the proposer's own
   statement of his own system is the correct citation for a modern system — exactly
   parallel to citing Fagan & Firebrace for Fagan-Bradley. Verbatim:

   > "As per Surya Siddhanta, yogatara of Pushya is the star at the centre of the
   > constellation and it is at a longitude of 16Cn0 and a latitude of 0°. In Cancer
   > constellation, Delta Cancri (Asellus Australis) is at the centre and it is indeed right
   > on the ecliptic plane, i.e. at a latitude of 0°! **So we define Pushya-paksha ayanamsa
   > by fixing the sidereal longitude of Delta Cancri at 16Cn0.** It is derived at any
   > date/time as follows: (1) Take the tropical longitude of Delta Cancri star at the
   > desired date/time. (2) Subtract 106° (16Cn0) from it. (3) The result is the ayanamsa…"

   **It needs no declared assumption.** Unlike Revati-paksha, where the text says "Revati"
   and we must infer ζ Piscium (DA-1), Rao names δ Cancri himself as part of the definition.
   There is no inference gap — on that axis it is *better* sourced than Revati-paksha.

   **Independent corroboration:** Rao's stated Surya Siddhanta basis — Pushya at 106°,
   latitude 0° — is exactly what §2.7's derivation from the v. 1 encoding rule produces,
   arrived at without reference to his article.

   Implementation notes: it is a **true star-tracking** system — Rao is explicit that it
   "finds ayanamsa at any given time by finding the exact tropical longitude of Delta Cancri
   star and does not approximate with any linear formula" — so it requires a catalogue
   position with proper motion. Rao does **not** specify mean vs apparent tropical
   longitude. His published reference values may be used **diagnostically to establish
   authorial intent** on that point, but **not as a conformance oracle** (worked values from
   a publication, excluded by the citation gate and by §6.5).

### 2.8 Normative conventions (A4) — an implementer may not guess these

Every item below was underspecified in the 2026-08-17 draft. Each is a contract, not a
preference: silence on any of them reproduces the class of error this work exists to remove.

**2.8.1 Output is MEAN ayanamsha. Nutation is never included.** Vedaksha returns the mean
ayanamsha for every system; a caller wanting true ayanamsha adds nutation in longitude
themselves. This must be stated in the doc comment of every public function, in the audit
dir, and in the migration note.

*Why this is not a detail:* IAE 2022 itself prints **"True Ayanamsa = Mean Ayanamsa +
nutation in longitude"**, and the daily tables panchanga-makers actually consume are the
*true* values — which differ from the mean polynomial by up to ~17″. The single largest
unexplained term in the constant this whole effort began with was a nutation removal. An
engine that is silent about which one it returns will be compared against the wrong column.

**2.8.2 Star-tracking longitude = mean geometric place of date.** No annual aberration, no
nutation, consistent with 2.8.1. "Tropical longitude of the star" is ambiguous by roughly
**20″ (aberration) plus ~17″ (nutation)** — *larger than every discrepancy that triggered
this project*, so leaving it to the implementer would silently dominate the result.
Applies to any system defined by tracking a live star. Proper motion **is** applied (it is
part of the star's position); aberration and nutation are observer effects and are not.

Where a proposer published reference values, those may be used **diagnostically, once, to
establish which convention the author intended** — and the finding recorded in the audit
dir. They may never become a conformance oracle (§6.5, and the citation gate excludes worked
values from publications).

**2.8.3 "Per year" means Julian year (365.25 d) throughout.** Constant-rate systems state a
rate without units — KP's 50.2388475″/yr, Raman's 50⅓″/yr. Julian vs tropical year differ by
~0.4″/century of accumulated ayanamsha. Adopt the Julian year, state the adoption, and record
it as a declared assumption (§2.6) wherever the primary does not specify.

**2.8.4 NORMATIVE ANCHOR TABLE.** This is the spec's acceptance surface: §6.1 requires each
system to reproduce its own anchor here to ≤1e-9°. Values are as published by the primary, at
full stated precision. All output is mean (§2.8.1); all rates are per Julian year (§2.8.3);
all star-anchored systems use the mean geometric place of date (§2.8.2).

**Group A — anchor value at a fixed epoch, propagated by a precession model**

| # | System | Anchor (as published) | Epoch, JD (TT) | Propagation | DA |
|---|---|---|---|---|---|
| 1 | Indian official (Chitra-paksha) | 23°51′25″.53 | 2451545.0 (J2000.0) | Capitaine, Wallace & Chapront 2003, P03 general precession in longitude | — |
| 2 | Fagan-Bradley | SVP 335°57′28″.64 ⇒ ayanamsha 24°02′31″.36 | 2433282.42346 (B1950.0) | **see DA-3** | DA-3 |
| 3 | Krishnamurti (KP) | 22°22′00″ | 2415122.5 or 2415123.5 | 50.2388475″/yr, constant (KSK's own stated rate) | DA-4, DA-5 |
| 6 | Yukteshwar | 20°54′36″ | vernal equinox 1894 — **see DA-6** | 54″/yr, constant (his own 24,000-yr cycle) | DA-6 |
| 9 | Raman | zero point 397 CE ⇒ ayanamsha = (year − 397) × 50⅓″ | year-granular — **see DA-7** | 151/3 ″/yr exactly, constant | DA-7, DA-8 |

**Group B — star or source fixed at an assigned sidereal longitude (tracked live)**

| # | System | Object | Assigned sidereal longitude | Position source | DA |
|---|---|---|---|---|---|
| 5 | Revati-paksha | Revati | 359°50′00″ | ζ Piscium, catalogue — **see DA-1** | DA-1 |
| 7 | Galactic Centre 0° Sagittarius | Sgr A* | 240°00′00″ | VLBI astrometry, ICRF3 — **see DA-9** | DA-9 |
| 8 | Pushya-paksha | δ Cancri (Asellus Australis) | 106°00′00″ (16 Cn 00) | catalogue | — |
| 10 | True Mula (Chandra Hari) | λ Scorpii | 240°00′00″ | catalogue | — |
| 11 | True Chitra | Spica (α Virginis) | 180°00′00″ | catalogue | — |

Note 7 and 10 both assign 240° but to **different objects**; they are different systems and
must not be collapsed.

**Group C — the text supplies its own model, not an anchor**

| # | System | Model |
|---|---|---|
| 4 | Surya Siddhanta | Zigzag libration, amplitude 27°, 600 revolutions per Mahayuga of 1,577,917,828 days (period ≈ 2,629,863.05 d), phase measured as ahargana from the Kali epoch JD 588465.75. **Linear folding, not sinusoidal** (Ch. 3 v. 10's *bhuja* rule, 90 → 27). DA-2, DA-10. |

**Declared assumptions referenced above** (extending §2.6):

- **DA-1** — Revati is ζ Piscium. Identification by the commentary, not by the text.
- **DA-2** — Surya Siddhanta's Ch. 3 phase (zero crossing 499 CE) vs Ch. 8's star-table epoch
  (~560–570 CE); the chapters disagree by ~1°. State which is adopted.
- **DA-3 — NEW, surfaced by building this table.** Fagan & Firebrace state the anchor and the
  relation (ayanamsha = 360° − SVP) but **never state a precession model** for propagating the
  SVP. Fagan wrote in the Newcomb era; adopting P03 is defensible as the faithful reading of
  "mean longitude of the vernal point" but is a choice, and it is *our* choice. §11 previously
  marked Fagan-Bradley "no declared assumption" — that was wrong, and only became visible
  when the propagation column had to be filled in.
- **DA-4** — KP's mid-April epoch is inferred from the flat +2′ correlation with Rajan, not
  stated by KSK, whose table header gives only "English Year".
- **DA-5** — Rajan's own header says "the 13th **or** 14th April"; the ambiguity is 0.14″.
  Pick one, state it.
- **DA-6** — "the vernal equinox of 1894" needs a JD. Candidates: the actual equinox instant,
  or 1894-03-20/21 00:00 (2412907.5 / 2412908.5). His model is year-granular so any is within
  its own resolution; the choice must still be recorded.
- **DA-7** — Raman's rule takes "the year of birth" with no point within the year. Recommend
  1 January, stated. Art. 50 says the intra-year change "can conveniently be neglected".
- **DA-8** — Raman titles his own article "Determination of (**Approximate**) Ayanamsa" and
  declines to defend 397 CE over 361/394/498/559. Ship it as he characterises it.
- **DA-9** — which Sgr A* determination is adopted (positions differ at the mas level, far
  below the stated precision, but the paper must be named not implied).
- **DA-10** — the sign and direction of travel of the SS libration after its 499 CE zero
  crossing. My scratch implementation got this wrong, which is precisely what an implementer
  left to infer would also do. Specify it with a worked check value.

**Six of eleven now carry a declared assumption.** That is the table doing its job: DA-3 did
not exist as a known gap until a column had to be filled, and it was hiding inside a system
this spec had twice called fully sourced.

**2.8.4a Decisions on the open assumptions (2026-08-17).** Each is a fork in the derivation,
not a caveat on it; the implementation agent cannot compute without them.

**DA-2 — Surya Siddhanta phase: adopt the Ch. 3 verse-literal reading (Kali-epoch ahargana),
NOT a re-anchoring to Ch. 8.** Chapter 3 vv. 9–12 *is* the ayanamsa definition; Chapter 8 is a
star catalogue. Shifting the libration's phase to make the two chapters agree would be fitting
the model to a second dataset — the same move as "adjusted to match", one abstraction up. The
text is incoherent here and Burgess says so; we implement what it states and record the ~1°
disagreement as a property of the source. A repaired Surya Siddhanta would be our own system
shipped under its name.

**DA-3 — Fagan-Bradley propagation: adopt P03.** Four reasons, the last decisive:
(i) Fagan defines the SVP as the *mean longitude of the vernal point*, whose motion is general
precession, and P03 is the best current determination of it; (ii) he states no model, so
adopting one contradicts nothing he wrote; (iii) it matches system #1, where IAE itself
specifies P03; (iv) the tempting alternative — Newcomb, to match his era — would **not** buy
internal consistency, because his own p. 16 worked value for 1963 fails to follow from his
p. 13 anchor under *any* precession model. Era-matching would trade a defensible modern choice
for a speculation about intent and gain nothing.

**DA-6 — Yukteshwar epoch: the vernal equinox instant of 1894, computed by the engine.** His
text says "the position of the Vernal Equinox at spring in the year 1894"; the equinox instant
is the literal reading, it is computable from machinery we already have, and it makes the
anchor a defined instant rather than an arbitrary midnight. His 54″/yr runs from it and the
zero falls at the corresponding instant in 500 CE. Record the computed JD in the manifest.

**DA-7 — Raman epoch: 1 January of the stated year, 00:00 TT.** Art. 50 tells the reader to
*add* precession for "odd days", which implies accumulating forward from a year boundary, and
the boundary of the "year (a.d.)" his rule takes is 1 January. **Verified: this convention
reproduces both of Art. 49's printed examples exactly** — 1912 → 21°10′55″, 1918 → 21°15′57″.

**DA-9 — Sgr A*: adopt the ICRF3-frame VLBI absolute-astrometry determination**, named
explicitly in the fetch manifest with position *and* proper motion, cross-checked against
Reid & Brunthaler. The implementer must read the values out of the paper, not inherit them
from this spec. The choice is immaterial at our precision (mas-level), but an unnamed source
is not a source.

**DA-10 — Surya Siddhanta sign: after the 499 CE zero crossing the ayanamsa is POSITIVE and
INCREASES at a constant 54″/yr**, reaching the +27° extremum at 2299 CE and reversing after.

*Worked check value, normative:* `ayanamsha_SS(J2000) = (2000 − 499) × 54″ = 81,054″` =
**22°30′54″**. A negative result at J2000 means the folding direction is inverted — which is
what my own scratch implementation produced, and why this is stated rather than inferred.

*Unexpected corroboration.* The three quantities the text gives independently — amplitude 27°,
rate 54″/yr, and 600 revolutions per Mahayuga — are **mutually consistent only under the
linear folding**: 27° at 54″/yr takes exactly 1800 years, and the quarter-period is
2,629,863.05 / 4 / 365.25 = **1800.04 years**. Under a sinusoidal reading 54″/yr would be a
maximum rather than a constant rate, and the amplitude would not land on the quarter period.
Burgess's zigzag is not merely his interpretation; it is the only reading under which the
Surya Siddhanta's own three numbers agree with each other.

**2.8.5 Enum surface and deserialization.** The spec must list the normative variant names —
they propagate to seven locale files, the MCP tool schema enum, the Python and WASM bindings,
and serde. A removed v5 variant name **must hard-error on deserialization** with a pointer to
the disposition table, never silently fall back to a default: a silent remap would move a
caller's chart without telling them, which is the failure mode of the whole episode in
miniature. State explicitly what the crate's default ayanamsha is.

**2.8.6 The fixture is generated, not hand-written.** `scripts/generate_ayanamsha.py` emits
it post-freeze; a hand-typed fixture is another unsourced constant table. Pin its tolerance
and location. Regeneration under `--verify` is an acceptance criterion (§6.7).

## 3. Permitted inputs

**Primary/definitional**

- *Report of the Calendar Reform Committee*, CSIR, Government of India (signed 13 Sept
  1954). <https://archive.org/details/calendar_reform_comittee_report>
- Fagan, C. & Firebrace, R. C., *Primer of Sidereal Astrology*, AFA, 1971.
- Krishnamurti, K. S., *Krishnamurti Padhdhati Vol-I*.
  <https://archive.org/details/in.ernet.dli.2015.128044>
- Rajan, C. G., *Astrological Tables of Lagna and Other Houses*, 1st edn 1941.
  <https://archive.org/details/in.ernet.dli.2015.48595>
- Indian Astronomical Ephemeris 1989 — **not yet obtained**, see §9.1.
- Classical texts by chapter (Surya Siddhanta, Aryabhatiya); Yukteshwar, *The Holy Science*
  (1894, public domain).
- IAU resolutions and the academic precession papers in §2.3.

**Astrometric data** (permissive, redistributable with attribution)

- Hipparcos / Gaia via VizieR (ESA) — star positions, parallaxes, proper motions.
- The IAU 1958 galactic coordinate definition — to be read from the defining publication
  (Blaauw, Gum, Pawsey & Westerhout, *MNRAS* 121, 123), **not from memory or a secondary
  summary**.
- Sgr A* position from the radio-astrometry literature, cited to the paper.

**Verification tooling** (astronomy only, never ayanamsha values)

- ERFA (BSD-3), the IAU SOFA board's approved relicensing. Astropy (BSD-3), Skyfield (MIT).

## 3.1 Forbidden inputs

Enumerated so the firewall can be checked mechanically, exactly as `ytliu` was for ELP:

- `sweph.h`, any Swiss Ephemeris source, and any Swiss Ephemeris **output** including
  `swetest`.
- Any derivative or wrapper: pyswisseph, swephR, the `sweph` npm package, SwissEphNet,
  VedAstro, or any service backed by them.
- **`crates/vedaksha-astro/src/sidereal.rs`** — both the shipped constants and the
  "original" values named in its comments.
- `the internal provenance audit note`.
- Any other astrology software or its output, at any point, for any purpose.

Search-inside on a lending-restricted archive.org title is permitted (it is a public
feature). Circumventing a borrow restriction is not.

---

## 4. Derivation procedures

### 4.1 Tier A — published definitional constants

**Lahiri (Chitrapaksha) — SOLVED, and it is a no-change outcome.**

The operative definition is not a reconstruction from 1956. It is published directly by the
issuing authority. *The Indian Astronomical Ephemeris 2022*, Positional Astronomy Centre,
Government of India, **p. 380, "AYANAMSA"** — free and unrestricted at
<https://archive.org/details/Indian_Astronomical_Ephemeris> (2018–2022 held) — reads, in
typeset print (verified from the rendered page, not OCR):

> "The ayanamsa value has been calculated from the polynomial of precession in longitude
> published by N.Capitaine et. al. (2003) in journal Astronomy and Astrophysics. The
> polynomial for ayanamsa has been introduced in this publication from the year 2021. …
> Where T=(JD-2451545)/36525. **Ayanamsha for J2000.0 is taken as 23°51'25".53**"

So the derivation is: **anchor 23°51'25".53 at J2000.0, propagated by Capitaine, Wallace &
Chapront (2003) general precession in longitude** — the polynomial
`vedaksha_ephem_core::precession::general_precession_in_longitude` already implements and
already cites.

Two things follow, and both are load-bearing:

1. **Structure.** Implement Lahiri in the official J2000-anchored form, *not* the current
   1956-epoch + IAU 1976 form. The code then mirrors the primary one-to-one, which is what
   makes it auditable — and it is simpler. (Note that `10f8834` moved Lahiri *away* from a
   J2000-flat structure toward the sweph reconstruction, while the official definition is
   J2000-anchored.)
2. **Transcription.** IAE's printed T⁴ coefficient is −0".00023857; Capitaine's published
   P03 T⁴ is −0.000023857. Since IAE states it uses Capitaine's polynomial, **use
   Capitaine's coefficients** and record the IAE printing discrepancy in the audit dir.
   The difference is ~1e-6″ and cannot matter numerically; copying a typo would matter for
   the chain's integrity.

The CRC 1954 anchor (23°15'00" at 0h, 21 March 1956) is retained as **historical context
only**, not as the computational anchor. IAE 2022 restates it on the same page, and it is
internally ~16″ from IAE's own operative J2000 value — the 1956 figure is nominal. IAE also
states the initial point "coincides with the vernal equinoctial point of vernal equinox day
of 285 A.D.", which the §6.2 inversion of the J2000 anchor independently reproduces
(285.71 CE).

**KP (Krishnamurti).** Anchor from KSK, *Vol-I* p. 140, whose table is headed `deg. mt`
and derives from Rajan 1941 p. 110 — headed *"Ayanamsa on the 1st of Chitra (i.e. on the
13th or 14th April.)"* — by a flat +2′ that KSK applies across the whole range and whose
provenance KSK acknowledges in prose (*"the difference between what I follow, what Lahiri
and C.G Rajan follow is negligible"*).

Two properties of this chain constrain the result and must be carried, not smoothed:

- **The primaries publish arcminutes only.** No value in either table, 1840–2001, carries
  an arcsecond. A derived KP constant may therefore state no more precision than the
  propagation legitimately produces from a minute-precision anchor, and the derivation must
  record the anchor's ±30″ quantisation as its stated uncertainty.
- **The anchor epoch is mid-April, not a year boundary and not a Besselian epoch.** The
  13th-vs-14th ambiguity in Rajan's own header is ~0.14″ and is recorded, not resolved.

Rate: KSK states 50.2388475″/yr, twice, attributing it to Newcomb. Independent research
(Senthilathiban) finds no basis for it in Newcomb and identifies it as the rate for B1821;
M. G. G. Nayar reported the same failure to trace it in 1980. **Use KSK's stated rate** —
it is the definition of the system — and record the divergence from real Newcomb
(~1.8″/century) as a documented property.

**Fagan-Bradley.** Already derived and verified; no computational work remains. Anchor:
Fagan & Firebrace p. 13, *"for the epoch 1950.0 he proposed as the mean longitude of the
vernal point 335° 57' 28.64″"*. Relation, p. 16: *"The ayanamsa for a given date is found
by subtracting the sidereal longitude of the vernal point … from 360 degrees."* Epoch is
B1950.0, a standard epoch. The deliverable here is a **citation swap only**.

Note for the record: p. 16 also gives a worked ayanamsha for "the beginning of 1963 A.D."
that does not follow from p. 13 under any precession model (it implies ~49.13″/yr). It is
inconsistent with Fagan's own definition and **must not be used as a check**; it is in any
case a worked value from a commercial book, which our citation rule excludes.

### 4.2 Tier B — star-anchored

General procedure. For a system defining star *S* at sidereal longitude λ_s:

1. Obtain *S*'s ICRS position, proper motion and parallax from Hipparcos/Gaia via VizieR.
   Record catalogue, identifier, epoch and query in the manifest.
2. Propagate to the required epoch; apply proper motion per §2.4 as the system's own
   definition requires.
3. Convert ICRS → ecliptic longitude of date using the obliquity and frame bias of the
   adopted IAU model.
4. ayanamsha(t) = λ_tropical(S, t) − λ_s.

Systems whose star **and** assigned longitude are established from a primary may proceed.
For the remainder the defining longitude **must be located in a primary first**; several
are currently known only from the enum's own doc comments, which are part of the artefact
under review and carry no authority. `Hipparchos` is flagged separately: it rests on a
contested reconstruction of Hipparchus's catalogue, not on a definition, and is a
candidate for Tier E.

### 4.3 Tier C — galactic

Same shape, with the anchor being a galactic-frame condition rather than a star: obtain the
pole/centre definition from its defining publication, transform to ecliptic of date, and
subtract the assigned sidereal longitude. The IAU 1958 pole and Sgr A* both have citable
primaries. The Brand, "galactic alignment", "true" and "mid-Mula" variants do not yet have
located definitions and may fall to Tier E.

### 4.4 Tier D — classical Indian texts

Anchor is the text's own stated epoch and rate, cited **by chapter, never by page, edition
or translator** (project citation gate). Note that Surya Siddhanta specifies a *trepidation*
(libration), not a linear precession; a derivation that silently linearises it has changed
the definition. Aryabhata's two variants differ by epoch (499 CE and 528 CE) and the
distinction must come from the text. Yukteshwar's *The Holy Science* (1894) is public
domain and states its own cycle in the introduction.

### 4.5 Tier E — removed

No derivation is possible, so **these are deleted, not hedged**. A value that cannot be
reached from any primary can only have come from another implementation; retaining it with
a "value as published by X" note would preserve the exposure while adding a citation that
does not actually support it.

Removal is mechanical but must be complete: enum variants, `name()` arms, the locale
strings in `crates/vedaksha/src/locale/` that name them, the MCP tool schema enum in
`tools/mcp-tools.json` (regenerate with `dump-tools-list`), the WASM surface, the Python
binding, and the README's "44 ayanamshas" claim. The count appears in more places than the
enum.

Shipping ~22 ayanamshas each traceable to a star, a chapter or a committee is a stronger
claim than 44 of unknown provenance — and unlike the current claim, the README can make it
truthfully.

---

## 5. Process — two-agent firewall

Unchanged from the ELP precedent, which worked.

1. **Spec agent.** Permitted inputs §3 only; forbidden list §3.1 enumerated in its system
   prompt. Produces the final spec. Transcript committed.
2. **Implementation agent.** Reads *only* the final spec, plus the repo's existing code
   style. Works in an isolated git worktree on `cleanroom/ayanamsha`. Never sees the
   current constants, this note's parent audit, or any forbidden source. Transcript
   committed.
3. **Firewall check before merge.** `grep -rniE 'sweph|swisseph|swetest|astrodienst'` over
   the new code returns empty, excluding the audit dir's own provenance prose.

The implementation agent will need `sidereal.rs` replaced wholesale rather than edited,
since editing it means reading it. Write the new module beside the old one and delete the
old one in the same commit.

---

## 6. Acceptance criteria

**6.1 Anchor reproduction.** At its defining epoch each system's formula returns its
defining value exactly (≤1e-9°). Non-negotiable; a failure here is an implementation bug.

**6.2 Zero-year inversion.** Inverting the derived constant to the epoch at which the
ayanamsha is zero must land on the tradition's own documented zero year, within the
precision the primary states. This is **primary-against-primary with no software in the
loop**, and it has already been shown to discriminate: under IAU 1976 the CRC anchor
inverts to 285.5 CE against Lahiri's documented 285; under Newcomb, KSK's published anchor
inverts to 291.7 CE against his stated 291 AD. Systems with no documented zero year are
exempt.

**6.3 Astronomy cross-check.** Star positions, obliquity and precession — *not* ayanamsha
values — verified against ERFA or Astropy to ≤0.01″. Verifying our own computation against
a permissive implementation taints nothing.

**6.4 Post-freeze comparison — NOT a gate.** After derived values are frozen and committed,
compute the delta against the currently shipped constants **once**, and record it in the
migration note. Purpose: sizing the breaking change. It has no pass/fail. It must be run
after the freeze commit so it cannot influence the values, and the script that does it must
be separate from the generator.

**6.5 Explicitly not criteria.** Agreement with the shipped constants; agreement with Swiss
Ephemeris or any other implementation; "reasonableness" of a value relative to its
neighbours.

**6.7 Regeneration reproducibility — the criterion with real teeth.**
`scripts/generate_ayanamsha.py --verify` regenerates every constant from the anchors in the
normative table (§2.8.4) and diffs against the committed fixture; wired into full-validation
exactly like the existing `coefficient-blob-drift` / `generate_vsop87a.py --verify` jobs.

This matters because **§6.1–6.3 cannot, by themselves, catch a reverse-engineered value**,
and the spec should say so rather than imply otherwise. §6.1 tests self-consistency — a
reverse-engineered anchor reproduces itself perfectly. §6.2 is the only criterion with
discriminating power and only covers systems with a documented zero year, and the adversarial review showed
it can be run against the wrong rate model and prove nothing. §6.3 validates astronomy, not
ayanamshas. **Derivation integrity is established by the audit trail — sanitized spec, agent
transcripts, runnable generator — not by the test suite.** No one should later claim the
tests prove independence.

§6.7 is what keeps that true over time: the derivation stays executable and auditable
independent of any agent, and a constant can no longer enter by being typed.

**6.6 Where this deliberately diverges from the ELP precedent — no legacy oracle as a
gate.** ELP captured `lunar_legacy_oracle.bin` (10,000 pre-rederivation tuples) and used it
as a Tier-3 regression oracle. That was sound there: ELP's contamination was *structural*,
while the coefficient values existed independently in the IMCCE primary, so the old outputs
were a legitimate fact-check. **Here the values themselves are the contaminated artefact.**
Regression-testing against them would reinstate exactly the "aim at the known answer"
failure this work exists to undo. A snapshot of the current values may be captured for the
§6.4 migration delta, but it must be quarantined from the implementation agent and must
never appear in a test.

---

## 10. Standing constraints

- Never open `sweph.h`, at any point, for any reason — including "just to check". It
  deepens the exposure and creates a record of consulting the source.
- This document is normative. Any change to an anchor, model, convention or declared
  assumption is a change to the derivation and must be recorded here first, not in the
  generator.
- I am not counsel. This spec is engineering; the disclosure and licensing questions in the
  parent audit note remain the project owner's.

---

## 11. Corrections made during implementation — 2026-08-18

§10 makes this document normative: any change to an anchor, model, convention or
declared assumption is a change to the derivation and must be recorded here, not in the
generator. Six changes were needed. Each is recorded with the evidence that forced it.

### 11.1 The engine does not implement the polynomial §4.1 says it does

**§2.3 and §4.1 both assert that `vedaksha_ephem_core::precession::general_precession_in_longitude`
is the Capitaine et al. (2003) general precession in longitude. It is not.** That function
returns the Fukushima-Williams angle `ψ̄_A` — precession in longitude referred to the *fixed*
J2000 ecliptic, which exists to build the precession matrix. The ayanamsha needs `p_A`, general
precession accumulated along the *moving* ecliptic of date.

| | linear coefficient |
|---|---|
| `ψ̄_A`, what the engine had | 5038.481484″/century |
| `p_A`, what an ayanamsha needs | 5028.796195″/century |

They differ by **9.7 arcseconds per century**, and neither is distinguishable from the other by
inspecting a result — both grow at roughly 50″/year. §4.1's own transcription note is the tell:
it states Capitaine's T⁴ coefficient as −0.000023857, which belongs to `p_A`, while the function
in the tree carries −0.000026452, which belongs to `ψ̄_A`.

Resolution: a new `general_precession_p03` was added, pinned against ERFA's `eraP06e` at six
epochs from −51 to +10 centuries, where it agrees to machine precision. The two functions are
documented together, and a test asserts they are not interchangeable.

*This is exactly the failure §2 warns about — "a frame error is indistinguishable from a wrong
constant" — and it was inside a sentence this spec used to argue that no work was needed.*

### 11.2 The §2.8.4 anchor table gives Revati-paksha the wrong longitude

The normative table's Group B row 5 assigns ζ Piscium **359°50′00″**. The prose in
"Revati-paksha vs Surya Siddhanta — RESOLVED" says the opposite, and quotes a primary for it:

> "all authorities agree in placing it upon the ecliptic and **all excepting our treatise and the
> Cakalya** make its position exactly mark the initial point of the fixed sidereal sphere"

so Revati-paksha puts Revati at **0°00′**, and 359°50′ is the *Surya Siddhanta's* minority
reading. The table row appears to have copied the SS dhruva. The two readings are not
interchangeable: they are the 10′ by which the section says the systems differ, and adopting the
table row would have made Revati-paksha and the Surya Siddhanta share a zero point while the
prose said they differ.

**The derivation settles it numerically, and the check is the one the spec itself set up.** The
same section records two dates from the commentary: the SS zero point met the vernal equinox
about A.D. 560, and ζ Piscium itself did so in A.D. 572 — twelve years apart, being 10′ at
~50.2″/yr. Inverting the derived system to its zero year:

| assigned longitude | derived zero year | commentary |
|---|---|---|
| **0°00′** (adopted) | **575 CE** | ζ Piscium at the equinox, **A.D. 572** |
| 359°50′ (table row) | 563 CE | SS zero point at the equinox, **A.D. 560** |

Both readings land within three years of a date the commentary states, and reproduce the
twelve-year separation exactly. That confirms the *relationship* and identifies which reading
belongs to which system. Revati-paksha is implemented at **0°00′00″**; the table row is a
transcription slip and is corrected here.

### 11.3 DA-9: ICRF3 does not contain Sgr A*, and §1.1(7)'s position is not a measurement

DA-9 asks for "the ICRF3-frame VLBI absolute-astrometry determination, named explicitly". The
ICRF3 catalogue has no Sgr A* entry — it is undetectable in S band, only smeared in X band, and
resolved out at X/Ka. A 1° cone search on all three ICRF3 band tables returns zero rows.

The citable determination is **Gordon, D., de Witt, A. & Jacobs, C. S. (2023), AJ 165, 49**
(doi:10.3847/1538-3881/aca65b), a no-net-rotation solution against 258 ICRF3 defining sources.
Adopted, with Reid & Brunthaler (2020) as the cross-check; their proper motions agree within 1σ.

Separately: the position stated in §1.1(7) is **not a measurement**. It appears in Reid &
Brunthaler only as a table note giving the arbitrary origin their offsets are measured from, and
Gordon+2023 Table 1 puts it 40.1 mas in RA from the ICRF3 position. DA-9's instruction that "the
implementer must read the values out of the paper, not inherit them from this spec" was therefore
load-bearing rather than ceremonial.

*Effect on §1.1(7)'s own check value: the spec's position gives ayanamsha(J2000) = 26°51′06.24″
against the 26°51′06.2″ it states, and the adopted Gordon+2023 astrometry gives 26°51′06.19″. The
check did its job; the 0.04″ between them is the frame difference, not an error.*

### 11.4 Raman's rate is per calendar year, and §2.8.3's escape clause is why

§2.8.3 adopts the Julian year "wherever the primary does not specify". Raman's primary *does*
specify, through its own arithmetic: Art. 49 subtracts one year number from another and
multiplies by 50⅓″, so the unit it counts is the calendar year. Implementing it at 50⅓″ per
Julian year would miss Art. 49's printed 1912 example by ~1.6″.

Adopted: the value is anchored at 1 January of the stated year (DA-7) and interpolated linearly
across the actual length of that calendar year — which is Art. 50's optional "odd days"
correction, and makes the function continuous while leaving every 1 January exact. **Both printed
examples reproduce to better than a microarcsecond.**

### 11.5 §2.8.4's Surya Siddhanta check value carries a stated rounding

DA-10's normative check is `(2000 − 499) × 54″ = 22°30′54″`. The verse-literal computation gives
**+22°30′38.4″** at J2000. The whole 15.6″ is accounted for by the two roundings inside the check
itself: the exact half-period crossing falls at 499.22 CE, not 499.0, and the exact rate implied
by the text's own three numbers is 53.9987″/yr, not 54.0000.

No change to the derivation. Recorded because the check's stated purpose is to pin the **sign and
direction**, and it does: the derived value is positive, the extremum is exactly +27°, and it
falls in 2299 CE — all three as DA-10 states.

### 11.6 §4.5 overstates what the MCP schema contained

§4.5 lists "the MCP tool schema enum in `tools/mcp-tools.json`" among the surfaces to prune. There
was no enum: both `ayanamsha` properties were free-form strings whose descriptions named three
systems while the engine had forty-four. A generated JSON-Schema `enum` was added, built from
`Ayanamsha::ALL`, so the schema can no longer drift from the engine.

Also corrected while there: `compute_vargas` advertised an `ayanamsha` parameter with a default,
and its handler never read the field. The tool takes an already-sidereal longitude, so the
parameter was meaningless; it has been removed rather than wired up.

### 11.7 A pre-existing engine defect, found while validating the transform

Not a spec correction, recorded because the derivation rests on it.
`precession_matrix` composed the four Fukushima-Williams rotations in the wrong order —
`Rz(γ̄)·Rx(−φ̄)·Rz(−ψ̄)·Rx(ε_A)` instead of `Rx(−ε_A)·Rz(−ψ̄)·Rx(φ̄)·Rz(γ̄)`. The error is
**0.014 mas at J2000 and 0.56 arcsecond at 499 CE**: invisible in the era the Horizons oracle
covers, and growing without bound outside it. It was found by checking the star transform against
ERFA at epochs 1500 years either side of J2000, which is the only reason it surfaced at all.

Fixed, and pinned by an ERFA-derived test spanning six millennia. Three `riseset` tests carried
scan-oracle literals produced under the old matrix; they were **regenerated from the scan
reference**, and no tolerance was relaxed — `AGREEMENT_TOL_DAYS` is still 1e-9 d.

### 11.8 §2.8.5's locale claim does not hold, and the honest response is to do nothing

§2.8.5 states that variant names "propagate to seven locale files". They do not:
`crates/vedaksha/src/locale/` has no ayanamsha module and never had one — its ten tables cover
aspects, dashas, deities, dignities, houses, karanas, nakshatras, panchanga yogas, planets and
signs. §4.5's instruction to remove "the locale strings that name them" therefore had nothing to
remove.

**No ayanamsha locale table was added, and the reason is the project's own fabrication rule.**
Populating one would mean inventing Hindi, Sanskrit, Tamil, Telugu, Kannada and Bengali renderings
of "Fagan-Bradley", "Galactic Centre at 0° Sagittarius" and "True Mula (Chandra Hari)". Some of
those have no traditional rendering because the systems are twentieth- and twenty-first-century
Western or scientific constructions. Shipping invented translations to fill a table would be
exactly the failure mode this whole re-derivation exists to remove, one layer up.

If localised ayanamsha names are wanted, they need a source per name per language, which is a
research task and not part of this derivation.

### 11.9 A limit of the hygiene gate, recorded so nobody over-trusts it

`scripts/check_spec_hygiene.py` catches two things: the previously shipped digit
strings, and the comparison phrasings that make a delta recoverable when a derived value
sits in the same paragraph. (The phrase list stays in the script — writing an example of one
here made this very paragraph fail the gate on its first run, which is §0's point about a
document matching its own check, demonstrated.) It does **not** model a delta *table*. The migration note contains one by
design — §6.4 requires it — and passes the gate cleanly.

That is the correct outcome for that file, but it means the gate is a check on the spec's
own hygiene, not a general leak detector. CI runs it over `spec.md`,
`derivation-inputs.json` and the audit `README.md`, and deliberately not over
`migration-note.md`, whose whole job is to state the delta. Anyone adding a new document to
this directory should decide which side of that line it sits on rather than assuming a green
gate means the document is target-free.
