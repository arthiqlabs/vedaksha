# vedaksha-wasm

Vedaksha's ephemeris and Vedic astrology (Jyotish) engine compiled to WebAssembly, for the browser and the edge with no data files: natal charts and kundli (kundali), nakshatras, vargas, dashas, shadbala, ashtakavarga, panchanga, houses and aspects.

```bash
npm install vedaksha-wasm
```

```js
import init, { compute_natal_chart } from "vedaksha-wasm";

await init();
const chart = JSON.parse(compute_natal_chart(JSON.stringify({
  year: 2024, month: 3, day: 20, hour: 12, minute: 0,   // UTC
  latitude: 28.6139, longitude: 77.2090,
  ayanamsha: "IndianOfficial", house_system: "WholeSign",
})));
```

Every function takes and returns JSON strings. The package is built with `wasm-pack --target web`, so `init()` fetches the `.wasm` file next to the module; under Node, pass its bytes: `await init({ module_or_path: readFileSync(".../vedaksha_wasm_bg.wasm") })`.

Part of [Vedaksha](https://github.com/arthiqlabs/vedaksha), a clean-room Rust ephemeris and Vedic astrology engine, also available for [Rust](https://crates.io/crates/vedaksha), [Python](https://pypi.org/project/vedaksha/) and as an [MCP server](https://crates.io/crates/vedaksha-mcp).

## Links

[Repository and full README](https://github.com/arthiqlabs/vedaksha) · [Website](https://vedaksha.net) · [Accuracy evidence](https://vedaksha.net/accuracy) · [Changelog for v10.0.0](https://github.com/arthiqlabs/vedaksha/blob/main/CHANGELOG-v10.0.0.md)

## Licence

BUSL-1.1, source-available. Non-commercial use is free; commercial use (a product or service earning revenue from external customers) needs a one-time licence of USD $500 per organisation. Each release converts to Apache-2.0 four years after it ships. See [LICENSE][license]. By ArthIQ Labs.

[license]: https://github.com/arthiqlabs/vedaksha/blob/main/LICENSE
