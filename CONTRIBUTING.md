# Contributing

`provenact-spec` defines normative contracts and vectors. Changes here affect every consumer.

## Rules

1. Keep normative language explicit and testable.
2. Every schema or vector change must include conformance updates.
3. Breaking changes must be marked and justified in docs.
4. Keep examples deterministic; avoid environment-dependent fixtures.
5. Preserve backward compatibility for stable lines unless explicitly version-bumped.

## Required Checks

Before opening a PR:

1. `npm run check`
2. `cargo run -p provenact-conformance-harness --bin provenact-conformance`
3. Update changelog/docs for contract-impacting changes.
