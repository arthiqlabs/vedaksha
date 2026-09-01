# arXiv submission metadata

This is a record of the submission choices for the Vedaksha preprint, drafted
for Amit's review before anything is typed into arXiv. It does not itself
submit anything — no account login, endorsement request, or submit click has
been performed. See `.superpowers/sdd/task-1-report.md` for full sourcing.

## Categories

- **Primary category: `astro-ph.IM`** (Instrumentation and Methods for
  Astrophysics). arXiv's own taxonomy describes this category as "Detector and
  telescope design, experiment proposals. Laboratory Astrophysics. Methods for
  data analysis, statistical methods. Software, database design" — a direct
  match for a software/methods paper on an ephemeris engine.

- **Cross-list category: `cs.MS` (Mathematical Software) — conditional,
  recommend attempting but be ready to drop it.** Vedaksha is a published
  software library, so `cs.MS` is plausible as a cross-list. However, Task 1
  could not confirm on any arXiv-owned page whether a cross-list category
  requires its own, separate endorsement in that category's endorsement
  domain (checked `info.arxiv.org/help/submit/index.html` and
  `.../help/cross.html` directly — neither addresses it; secondhand forum
  reports suggest cross-listing after initial posting may avoid a second
  endorsement gate, but that is not arXiv-confirmed).

  **Recommendation:** attempt `astro-ph.IM` + `cs.MS` together at submission
  time; if arXiv's submission UI flags `cs.MS` as needing its own
  endorsement, drop it from the initial submission and add the cross-list
  later rather than seeking two personal endorsements up front.

## Endorsement path

Personal endorsement is **required with certainty**, not just "likely" —
arXiv's Jan 21 2026 policy update
(`blog.arxiv.org/2026/01/21/attention-authors-updated-endorsement-policy`)
made automatic endorsement require BOTH an institutional/academic email AND
prior authorship of an arXiv-accepted paper in the target endorsement domain.
Amit fails both independently: `info@arthiq.net` is not an institutional
email, and there is no prior arXiv paper to claim.

**Fallback path** (confirmed from `https://arxiv.org/help/endorsement`,
canonical text mirrored at `github.com/arXiv/arxiv-docs/.../endorsement.md`):
use arXiv's in-flow finder once this submission package is ready — start a
submission draft in `astro-ph.IM`, arXiv emails an "endorsement request"
link, and the submitter finds a candidate endorser via the "Which authors of
this paper are endorsers?" link on a recent related `astro-ph.IM` abstract
page, then emails that person the request link.

**Open question for Amit, unresolved as of this writing (relay verbatim):
"Amit needs to confirm: does he know an arXiv author in astro-ph.IM or
cs.MS?"** This cannot be researched further — only Amit can answer it. Per
the plan's design, this does not block producing this submission package; it
only blocks the actual click-submit action, which is out of scope for this
plan regardless (Amit's action).

Note: `https://arxiv.org/help/prevent-endorsement`, which was expected to
explain how to avoid needing an endorsement, currently 404s on every path
tried and appears to have been retired during the endorsement-policy
overhaul.

## Abstract

Copied verbatim from `main.tex` (lines 16-39 of the current draft, commit
`ca8657c`) — not retyped:

> Vedaksha is a clean-room ephemeris and Vedic astrology (jyotish) computation engine written
> in Rust. It computes planetary positions from the analytical VSOP87A and ELP/MPP02 series and
> from the JPL DE440s SPK numerical kernel --- validated against an independent JPL DE441
> ephemeris served by the NASA/JPL Horizons system to a mean residual of 0.103 arcseconds (maximum
> 1.187 arcseconds) over 15,350 comparisons spanning 1900--2025, re-measured for this paper -- and
> derives from them the constructs used in
> Vedic astrology --- nakshatras, dashas, vargas (divisional charts), panchanga, shadbala, and
> muhurta --- exposed through native Rust, Python, WebAssembly, and Model Context Protocol (MCP)
> interfaces.
> Every algorithm is derived directly from a cited primary source rather than from an existing
> implementation: the ephemeris and sidereal-conversion algorithms as shipped in the current
> release contain no source code, header, or documentation from the GNU Affero General Public
> Licensed (AGPL) Swiss Ephemeris library, on which prior Vedic astrology software has commonly
> relied. (Three early sidereal-offset constants were briefly cross-referenced against a published
> Swiss Ephemeris header solely to identify a numeric value; all three were independently re-derived
> from cited primary sources in a subsequent clean-room pass, documented below.) This clean-room
> provenance is not merely asserted: it is documented per data source, with fetch dates and,
> where established, content hashes, in a public provenance ledger, and specific re-derivations
> carry dated, auditable clean-room evidence trails. Vedaksha is released under the Business
> Source License 1.1 (converting to Apache License
> 2.0 four years after each version's release) and is distributed via crates.io, PyPI, npm, and
> GitHub.

## License for the arXiv submission itself

**Recommend CC BY 4.0 for the paper text.** This is distinct from and
independent of the software's license (BUSL 1.1, converting to Apache 2.0
four years after each version's release) — the paper text and the software
are separately licensable works, and arXiv requires the submitter to pick
one of its own supported license options for the paper deposit regardless of
what license covers any software described in it.

**This is flagged explicitly as Amit's choice to confirm, not a decision this
plan makes.** arXiv's other options at submission time include arXiv's own
non-exclusive distribution license (with no reuse rights granted beyond
arXiv's own use), CC BY-SA 4.0, and CC0. CC BY 4.0 is recommended here only
because it is the most common choice for permissive open-access reuse of the
paper text while imposing no restriction that could be read as bearing on the
separately-licensed software.

## Bundle contents

`docs/paper/build/vedaksha-preprint-submission.tar.gz` contains exactly:

- `main.tex`
- `references.bib`

No compiled PDF is included in the tar, per arXiv's own TeX pipeline
(it compiles `main.tex` itself). A compiled `docs/paper/build/main.pdf` is
kept alongside the tar, unpacked, for Amit's own pre-submission read. Both
files under `docs/paper/build/` are gitignored build artifacts, not source.
