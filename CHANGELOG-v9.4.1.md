# v9.4.1

Patch release. No behaviour change. The v9.4.0 tag's tree failed
`release.yml`'s Clippy step under CI's newer stable toolchain
(`missing_panics_doc` on an `assert_eq!` the pinned 1.97.1 toolchain does
not flag), so v9.4.0 never published anything and this version carries the
fix.

## Fixed

- `varnada_lagna` and `indu_lagna` document their panic contracts with
  `# Panics` sections. Doc comments only; no computed value moves.

## Compatibility

Identical behaviour to the v9.4.0 tree. A consumer pinning `9.4.0` by
registry version finds nothing there (it was never published); use `9.4.1`.
