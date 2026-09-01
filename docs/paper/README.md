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
