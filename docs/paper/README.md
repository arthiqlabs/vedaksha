# Vedaksha technical report

Source for the software/methods paper describing Vedaksha's ephemeris pipelines, their measured
accuracy, and the clean-room provenance of every algorithm.

**Status: not peer reviewed, and not submitted anywhere.** It is a technical report published by
ArthIQ Labs, not a preprint -- "preprint" would assert an intent to submit for publication that
does not currently exist. arXiv submission was explored and is blocked on securing a personal
endorser, which arXiv began requiring for this subject class; that route stays open, and posting
here does not foreclose it.

Every figure in it is version-specific. The accuracy residuals, test counts and tool counts are
measurements of one engine version, named on the title page. Re-measure and re-date before
republishing against a newer release rather than assuming they carry forward -- several of them
have moved between releases.

Build: `make pdf` (requires `tectonic`, installed via `brew install tectonic`). Output lands in
`build/main.pdf`.

The built `main.pdf` is also committed next to `main.tex` so it can be read on GitHub without a
LaTeX toolchain. `main.tex.sha256` records the hash of the `main.tex` it was built from;
`scripts/check_paper_pdf_fresh.py` (part of `make gate`) fails if the two drift apart. After
editing `main.tex`, refresh both:

```
make pdf && cp build/main.pdf main.pdf && shasum -a 256 main.tex > main.tex.sha256
```
