# Vedaksha — developer entry points.
#
# `make gate` is the contract: if it is green, the change is verified. It
# mirrors what .github/workflows/ci.yml gates on every push, so a green gate
# here and a red CI there should not be possible.
#
# Everything runs unpiped and in order. A piped command reports the PIPE's exit
# status, not the command's, which is how a `cargo test` that exited 101 was
# once read as passing.

CARGO ?= cargo
PY    ?= python3

.PHONY: gate fmt lint test guards portability wasm validate bench clean-check help

help:
	@echo "make gate        — everything CI gates. Run before every commit."
	@echo "make fmt         — rustfmt, write mode"
	@echo "make lint        — clippy over all targets, warnings denied"
	@echo "make test        — full workspace suite (~50s)"
	@echo "make guards      — publish order, licence sync, SPDX headers"
	@echo "make portability — no-default-features builds (the no_std surface)"
	@echo "make wasm        — rebuild the Python binding's wasm blob"
	@echo "make validate    — release + ignored tests. Tens of minutes; pre-tag only."
	@echo "make bench       — criterion suite under the shipped RUSTFLAGS for this arch"

# ── The gate ────────────────────────────────────────────────────────────────
# Ordered cheapest-first so a formatting slip fails in two seconds rather than
# after the suite.
gate:
	@echo "── format ──"
	$(CARGO) fmt --all -- --check
	@echo "── clippy (all targets, all features, -D warnings) ──"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings
	@echo "── portability ──"
	$(CARGO) check -p vedaksha-math --no-default-features --locked
	$(CARGO) check --workspace --no-default-features --locked
	@echo "── guards ──"
	$(PY) scripts/check_publish_order.py
	$(PY) scripts/check_license_sync.py
	$(PY) scripts/check_spdx_headers.py
	$(PY) scripts/check_mcp_image_label.py
	$(PY) scripts/check_paper_pdf_fresh.py
	@echo "── tests ──"
	$(CARGO) test --workspace --locked -- --skip analytical_oracle_regression
	@echo ""
	@echo "GATE GREEN — mirrors ci.yml. Full validation is a separate, weekly tier."

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

test:
	$(CARGO) test --workspace --locked -- --skip analytical_oracle_regression

guards:
	$(PY) scripts/check_publish_order.py
	$(PY) scripts/check_license_sync.py
	$(PY) scripts/check_spdx_headers.py
	$(PY) scripts/check_mcp_image_label.py
	$(PY) scripts/check_paper_pdf_fresh.py

portability:
	$(CARGO) check -p vedaksha-math --no-default-features --locked
	$(CARGO) check --workspace --no-default-features --locked

wasm:
	./bindings/python/scripts/build-wasm.sh

# Not part of `gate`: needs data/de440s.bsp, reaches the network, and takes
# tens of minutes. This is the pre-tag tier, and the tag IS the release —
# release.yml runs no tests of its own.
validate:
	$(CARGO) test --workspace --release --locked -- --include-ignored

# Not part of `gate`: criterion benchmarks take a while and are not a
# pass/fail check. Reproduces the shipped build's codegen locally so numbers
# here are comparable to the tracked trend in benchmarks.yml, instead of a
# plain-scalar build that never exercises the 4-wide SIMD lunar kernel.
# RUSTFLAGS matches Dockerfile's per-arch case and release.yml's build-mcp
# matrix: x86-64-v3 (AVX2) on x86_64, nothing extra on aarch64 (NEON is
# already the baseline there). See
# docs/audit/2026-08-29-perf-investigation.md #7a.
bench:
	@arch="$$(uname -m)"; \
	case "$$arch" in \
	  x86_64) flags="-C target-cpu=x86-64-v3" ;; \
	  *)      flags="" ;; \
	esac; \
	echo "── bench (arch=$$arch, RUSTFLAGS='$$flags') ──"; \
	RUSTFLAGS="$$flags" $(CARGO) bench -p vedaksha-ephem-core
