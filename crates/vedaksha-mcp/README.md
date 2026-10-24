# vedaksha-mcp

[![crates.io](https://img.shields.io/crates/v/vedaksha-mcp.svg)](https://crates.io/crates/vedaksha-mcp) [![docs.rs](https://img.shields.io/docsrs/vedaksha-mcp)](https://docs.rs/vedaksha-mcp)

An MCP (Model Context Protocol) server that gives AI agents Vedic and Western astrology: natal charts and kundli, panchanga, eight dasha systems, vargas, shadbala, ashtakavarga, transits, transit and muhurta search, synastry and composite charts, and a property-graph export, as 17 tools. Stdio or Streamable HTTP with bearer-token auth. Listed in the official MCP Registry as `io.github.arthiqlabs/vedaksha`.

```bash
cargo install vedaksha-mcp
vedaksha-mcp                                   # stdio, for Claude Desktop, Cursor, VS Code
VEDAKSHA_MCP_TOKEN=... vedaksha-mcp --http     # HTTP on :3100, auth on
```

Or with Docker: `docker run --rm -i ghcr.io/arthiqlabs/vedaksha-mcp --stdio`.

Part of [Vedaksha](https://github.com/arthiqlabs/vedaksha), a clean-room Rust ephemeris and Vedic astrology (Jyotish) engine.

## Links

[Repository and full README](https://github.com/arthiqlabs/vedaksha) · [Website](https://vedaksha.net) · [Accuracy evidence](https://vedaksha.net/accuracy) · [Changelog for v10.0.0](https://github.com/arthiqlabs/vedaksha/blob/main/CHANGELOG-v10.0.0.md)

## Licence

BUSL-1.1, source-available. Non-commercial use is free; commercial use (a product or service earning revenue from external customers) needs a one-time licence of USD $500 per organisation. Each release converts to Apache-2.0 four years after it ships. See [LICENSE](https://github.com/arthiqlabs/vedaksha/blob/main/LICENSE). By ArthIQ Labs.
