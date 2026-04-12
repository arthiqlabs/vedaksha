FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --release -p vedaksha-mcp

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/vedaksha-mcp /usr/local/bin/vedaksha-mcp

EXPOSE 3100

ENTRYPOINT ["vedaksha-mcp"]
CMD ["--http"]
