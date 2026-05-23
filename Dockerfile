FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . .

# Build for AVX2 (x86-64-v3, Haswell 2013+) so the SIMD lunar kernel runs
# 4-wide. This image is amd64; hosts without AVX2 will SIGILL — drop this ENV
# (or use x86-64-v2) if pre-2013 / non-AVX2 hosts must be supported. A future
# multi-arch (arm64) build would need this flag made target-conditional.
ENV RUSTFLAGS="-C target-cpu=x86-64-v3"

RUN cargo build --release -p vedaksha-mcp

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/vedaksha-mcp /usr/local/bin/vedaksha-mcp

EXPOSE 3100

ENTRYPOINT ["vedaksha-mcp"]
CMD ["--http"]
