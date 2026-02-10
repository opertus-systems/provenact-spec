# Rust Convenience Crates

These crates make adoption easier for Rust users while keeping JSON Schema and
spec docs as the normative source.

## Crates

- `provenact-spec-rs`
  - typed serde models for stable v0 contracts
  - canonical JCS + SHA-256 digest helpers
  - semantic helpers (`verify_receipt_hash`, `verify_snapshot_hash`, capability evaluation)

- `provenact-spec-validate`
  - schema loading and file/value validation helpers
  - YAML/JSON parsing support for policy vectors
  - repository root discovery helper

- `provenact-conformance-harness`
  - reusable conformance runner library and `provenact-conformance` CLI
  - validates schema vectors, capability vectors, and hash semantics

## Usage

```bash
cargo run -p provenact-conformance-harness --bin provenact-conformance
```

## Versioning

Crate versions should track spec tags and compatibility matrix updates.
