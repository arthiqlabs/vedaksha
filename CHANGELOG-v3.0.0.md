# Vedākṣha v3.0.0 — Surface Consolidation

**Release date:** 2026-05-12 (target)
**Type:** Breaking — published-crate surface change. No runtime semantics
or numerical-accuracy changes.

## Why

The v2.x line published 10 separate crates on crates.io. Maintaining 10
crates incurs constant operations cost — version-bump propagation,
release pipeline complexity, `[workspace.dependencies]` churn, and 10
pages of metadata to keep in sync with the website + MCP listings.

Three of those crates had near-zero real-world adoption:

| Crate           | Downloads (first 3 weeks) | Real users (excl. bots) |
|-----------------|--------------------------:|------------------------:|
| `vedaksha-emit`   | ~170                    | ~0 |
| `vedaksha-locale` | ~200                    | ~0 |
| `vedaksha-wasm`   | ~190 (npm: separate)    | ~0 (Rust); npm is the real channel |

v3.0.0 consolidates them away. The Rust API surface for downstream
consumers shrinks from 10 published crates to 7, with no loss of
functionality and no behavioural changes.

## Surface changes

| v2.x                            | v3.0.0 equivalent                                   |
|----------------------------------|------------------------------------------------------|
| `vedaksha-emit` crate            | `vedaksha-graph` with `features = ["emitters"]`     |
| `vedaksha-locale` crate          | `vedaksha` umbrella with `features = ["locale"]`    |
| `vedaksha-wasm` on crates.io     | `vedaksha-wasm` on **npm only** (this crate stops publishing to crates.io) |

The seven crates still published to crates.io in v3.0.0:

1. `vedaksha-math`
2. `vedaksha-ephem-core`
3. `vedaksha-graph` *(now includes the emitters)*
4. `vedaksha-astro`
5. `vedaksha-vedic`
6. `vedaksha-mcp`
7. `vedaksha` *(umbrella; now hosts the locale tables under a feature)*

## Migration

### 1. `vedaksha-emit` → `vedaksha-graph[emitters]`

**Cargo.toml**

```diff
-vedaksha-emit = "2"
+vedaksha-graph = { version = "3", features = ["emitters"] }
```

Or via `cargo`:

```sh
cargo remove vedaksha-emit
cargo add vedaksha-graph --features emitters
```

**Source**

```diff
-use vedaksha_emit::GraphEmitter;
-use vedaksha_emit::cypher::CypherEmitter;
-use vedaksha_emit::surreal::SurrealEmitter;
-use vedaksha_emit::jsonld::JsonLdEmitter;
-use vedaksha_emit::json_graph::JsonGraphEmitter;
-use vedaksha_emit::embedding_text::EmbeddingTextEmitter;
+use vedaksha_graph::emitters::GraphEmitter;
+use vedaksha_graph::emitters::cypher::CypherEmitter;
+use vedaksha_graph::emitters::surreal::SurrealEmitter;
+use vedaksha_graph::emitters::jsonld::JsonLdEmitter;
+use vedaksha_graph::emitters::json_graph::JsonGraphEmitter;
+use vedaksha_graph::emitters::embedding_text::EmbeddingTextEmitter;
```

Trait + struct identities, method signatures, and emitted output are
unchanged.

### 2. `vedaksha-locale` → `vedaksha[locale]`

**Cargo.toml**

```diff
-vedaksha-locale = "2"
+vedaksha = { version = "3", features = ["locale"] }
```

Or via `cargo`:

```sh
cargo remove vedaksha-locale
cargo add vedaksha --features locale
```

**Source**

```diff
-use vedaksha_locale::Language;
-use vedaksha_locale::planets;
-use vedaksha_locale::signs;
-use vedaksha_locale::nakshatras;
+use vedaksha::locale::Language;
+use vedaksha::locale::planets;
+use vedaksha::locale::signs;
+use vedaksha::locale::nakshatras;
```

All 11 lookup-table modules (planets, signs, nakshatras, houses, dashas,
deities, dignities, karanas, panchanga_yogas, yogas, aspects) keep their
exact names and APIs. The seven Tier-1 languages and the `Language`
enum are unchanged.

If you were already using the `vedaksha` umbrella and reaching locale
via `vedaksha::locale::*`, just add the `locale` feature flag — your
imports already work.

### 3. `vedaksha-wasm` — npm only

The Rust crate `vedaksha-wasm` is no longer published to crates.io.
The **npm** package `vedaksha-wasm` is unchanged and remains the
canonical delivery channel for browser / Node consumers.

**npm consumers:** no action needed.

**Rust consumers who really want to build the wasm bindings from
source:**

```toml
[dependencies]
vedaksha-wasm = { git = "https://github.com/arthiqlabs/vedaksha", tag = "v3.0.0" }
# or, vendoring:
vedaksha-wasm = { path = "path/to/vedaksha/crates/vedaksha-wasm" }
```

The crate stays in-tree; it just isn't on crates.io anymore.

## v2.x line

The 2.x line will not receive any new feature releases. The final
2.x version (v2.6.0) of `vedaksha-emit`, `vedaksha-locale`, and the
Rust `vedaksha-wasm` crate will be **yanked** from crates.io shortly
after v3.0.0 ships — switch to v3.0.0 to keep building.

v3.0.0 is the canonical path forward. There are no other behaviour or
numerical-accuracy changes in this release; only the published-crate
shape moved.

## License

BSL 1.1 (unchanged). Copyright © 2026 ArthIQ Labs LLC.
