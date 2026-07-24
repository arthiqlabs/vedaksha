# Multi-arch build. `docker buildx build --platform linux/amd64,linux/arm64 .`
# builds each platform in its target-arch container, so TARGETARCH selects the
# right codegen below.
FROM rust:1.85-slim AS builder

# Populated by buildx: "amd64", "arm64", …
ARG TARGETARCH

WORKDIR /app
COPY . .

# Arch-appropriate codegen for the 4-wide SIMD lunar kernel:
#   amd64 → AVX2 (x86-64-v3, Haswell 2013+). Hosts without AVX2 will SIGILL;
#           relax to x86-64-v2 if pre-2013 x86 must be supported.
#   arm64 → NEON is the aarch64 baseline, so no target-cpu flag is needed.
RUN case "$TARGETARCH" in \
      amd64) export RUSTFLAGS="-C target-cpu=x86-64-v3" ;; \
      *)     export RUSTFLAGS="" ;; \
    esac; \
    cargo build --release -p vedaksha-mcp

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/vedaksha-mcp /usr/local/bin/vedaksha-mcp

EXPOSE 3100

# HTTP mode now requires auth: set VEDAKSHA_MCP_TOKEN at runtime, e.g.
#   docker run -e VEDAKSHA_MCP_TOKEN=… -p 3100:3100 <image>
# The container exits immediately if the token is absent (pass
# --insecure-no-auth only behind a trusted network boundary).
ENTRYPOINT ["vedaksha-mcp"]
CMD ["--http"]
