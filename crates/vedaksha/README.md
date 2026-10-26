# vedaksha

[![crates.io](https://img.shields.io/crates/v/vedaksha.svg)](https://crates.io/crates/vedaksha) [![docs.rs](https://img.shields.io/docsrs/vedaksha)](https://docs.rs/vedaksha)

Rust ephemeris and Vedic astrology (Jyotish) engine: natal charts and kundli (kundali), panchanga, dashas, nakshatras, vargas, shadbala, ashtakavarga, muhurta and transits, from an ephemeris that agrees with JPL Horizons to 0.096″ mean in longitude (1900 to 2025). Clean-room, with every algorithm traced to a primary source, and no AGPL obligations. This umbrella crate re-exports every Vedaksha crate under one dependency.

```toml
[dependencies]
vedaksha = "10"
```

```rust
use vedaksha::prelude::*;

fn main() {
    // 2024-03-20 12:00 UT1. The day is a fraction: noon on the 20th is 20.5.
    let jd = calendar_to_jd(2024, 3, 20.5);
    let moon = apparent_position(&AnalyticalProvider, Body::Moon, jd).expect("in range");
    println!("Moon {:.4}°", moon.ecliptic.longitude.to_degrees());
}
```

A complete sidereal chart with houses is the crate-level example on [docs.rs](https://docs.rs/vedaksha). Every Julian Day on the public surface is UT1.

The same engine ships as a Python package ([`vedaksha`](https://pypi.org/project/vedaksha/)), a WebAssembly package ([`vedaksha-wasm`](https://www.npmjs.com/package/vedaksha-wasm)) and an MCP server of 17 tools ([`vedaksha-mcp`](https://crates.io/crates/vedaksha-mcp)).

## Links

[Repository and full README](https://github.com/arthiqlabs/vedaksha) · [Website](https://vedaksha.net) · [Accuracy evidence](https://vedaksha.net/accuracy) · [Changelog for v10.0.0](https://github.com/arthiqlabs/vedaksha/blob/main/CHANGELOG-v10.0.0.md)

## Licence

BUSL-1.1, source-available. Non-commercial use is free; commercial use (a product or service earning revenue from external customers) needs a one-time licence of USD $500 per organisation. Each release converts to Apache-2.0 four years after it ships. See [LICENSE][license]. By ArthIQ Labs.

[license]: https://github.com/arthiqlabs/vedaksha/blob/main/LICENSE
