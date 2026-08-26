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

.PHONY: gate fmt lint test guards portability wasm validate clean-check help

help:
	@echo "make gate        — everything CI gates. Run before every commit."
	@echo "make fmt         — rustfmt, write mode"
	@echo "make lint        — clippy over all targets, warnings denied"
	@echo "make test        — full workspace suite (~50s)"
	@echo "make guards      — publish order, licence sync, SPDX headers"
	@echo "make portability — no-default-features builds (the no_std surface)"
	@echo "make wasm        — rebuild the Python binding's wasm blob"
	@echo "make validate    — release + ignored tests. Tens of minutes; pre-tag only."

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
