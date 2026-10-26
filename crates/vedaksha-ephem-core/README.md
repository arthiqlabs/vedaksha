# vedaksha-ephem-core

[![crates.io](https://img.shields.io/crates/v/vedaksha-ephem-core.svg)](https://crates.io/crates/vedaksha-ephem-core) [![docs.rs](https://img.shields.io/docsrs/vedaksha-ephem-core)](https://docs.rs/vedaksha-ephem-core)

Ephemeris core: a JPL DE440s/DE441 SPK reader, the VSOP87A and ELP/MPP02 analytical series compiled to constants (no data files), IAU 2006 precession, IAU 2000B nutation, light time, aberration, solar light deflection and Delta T. Agreement with JPL Horizons over 1900 to 2025, in apparent geocentric longitude: 0.096″ mean on the SPK path and 0.160″ mean on the analytical path.

Part of [Vedaksha](https://github.com/arthiqlabs/vedaksha), a clean-room Rust ephemeris and Vedic astrology (Jyotish) engine. Most users want the umbrella crate [`vedaksha`](https://crates.io/crates/vedaksha), which re-exports this one.

## Links

[Repository and full README](https://github.com/arthiqlabs/vedaksha) · [Website](https://vedaksha.net) · [Accuracy evidence](https://vedaksha.net/accuracy) · [Changelog for v10.0.0](https://github.com/arthiqlabs/vedaksha/blob/main/CHANGELOG-v10.0.0.md)

## Licence

BUSL-1.1, source-available. Non-commercial use is free; commercial use (a product or service earning revenue from external customers) needs a one-time licence of USD $500 per organisation. Each release converts to Apache-2.0 four years after it ships. See [LICENSE][license]. By ArthIQ Labs.

[license]: https://github.com/arthiqlabs/vedaksha/blob/main/LICENSE
