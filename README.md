# provenact-spec

Public contract repository for Provenact.

This repo is implementation-neutral: it defines normative protocol behavior,
schemas, and conformance vectors that runtimes, verifiers, and SDKs can pin.

## Contents

- `SPEC.md`: top-level normative index.
- `spec/`: normative documents and JSON schemas.
- `test-vectors/`: deterministic conformance fixtures (good/bad).
- `docs/versioning-policy.md`: compatibility and release semantics.
- `docs/compatibility-matrix.md`: implementation compatibility tracking.
- `docs/repo-remediation-backlog.md`: prioritized cross-repo remediation plan.
- `docs/source-of-truth-sync-policy.md`: sync governance for shared artifacts.
- `docs/remediation-execution-board.md`: issue-ready execution board.
- `tools/check-sync-parity.sh`: CI parity checker for mirrored artifacts.
- `tools/generate-sync-manifest.sh`: machine-readable sync metadata generator.

## Stability

Current stable line: `v0`.

Experimental drafts are explicitly marked with `experimental` in file names
or are placed under `spec/rfcs/`.

## Conformance

```bash
npm install
npm run check
```

`npm run check` performs:
- JSON syntax linting for the whole repository.
- Schema-vs-vector validation for all v0 and draft schema sets.
- Capability-evaluation vector execution with expected allow/deny assertions.

Rust users can run the crate-based harness:

```bash
cargo run -p provenact-conformance-harness --bin provenact-conformance
```

## Consumers

- `provenact-cli`: reference implementation and release gate.
- `provenact-sdk`: SDK-side parsers, validators, and protocol bindings.
- Third-party runtimes/verifiers: pin schema files and vectors by tag.

Note: several Rust crate IDs still use `provenact-*` naming for compatibility.

Rust convenience crates are documented in `docs/rust-crates.md`.

## License

Dual licensed under MIT OR Apache-2.0. See `LICENSE`.

Security reporting and contribution expectations are documented in
`SECURITY.md` and `CONTRIBUTING.md`.
