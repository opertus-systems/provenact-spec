# Cross-Repo Source-Of-Truth And Sync Policy

Date: 2026-02-07
Status: Draft (recommended for immediate adoption)

This policy prevents silent drift for artifacts duplicated across Inactu repositories.

## Goals

1. One authoritative source per shared artifact set.
2. Deterministic sync from source to mirrors.
3. CI failure on any unauthorized divergence.
4. Explicit ownership and release cadence.

## Authoritative Sources

### 1) Normative spec + vectors
- Source repo: `inactu-spec`
- Source paths:
  - `spec/`
  - `test-vectors/`
- Mirrors:
  - `inactu-cli/spec/`
  - `inactu-cli/test-vectors/`

### 2) Control plane OpenAPI contract
- Source repo: `inactu-control`
- Source path:
  - `openapi.yaml`
- Mirror:
  - `inactu-control-web/public/openapi.yaml`

## Non-Negotiable Rules

1. No direct edits in mirror paths.
2. Mirror updates only via sync workflow commits.
3. Mirror commits must reference source commit SHA.
4. CI blocks merges when source and mirror differ.
5. Release notes must mention source SHA for synced artifacts.

## Sync Workflow

### Spec/vector sync (`inactu-spec` -> `inactu-cli`)

1. Change lands in `inactu-spec`.
2. `inactu-spec` tags or publishes source commit.
3. Sync PR in `inactu-cli` copies `spec/` + `test-vectors/` verbatim.
4. Sync PR includes machine-generated manifest:
   - source repo
   - source commit
   - copied paths
   - file count
   - checksum summary
   - command: `tools/generate-sync-manifest.sh spec-cli <inactu-cli-path>`
5. CI in `inactu-cli` validates:
   - parity with declared source commit
   - conformance tests still pass

### OpenAPI sync (`inactu-control` -> `inactu-control-web`)

1. API contract change merges in `inactu-control`.
2. Sync PR copies `openapi.yaml` to web mirror path.
   - command: `tools/generate-sync-manifest.sh openapi <inactu-control-web-path>`
3. CI in `inactu-control-web` verifies checksum parity.
4. Docs/examples regenerated in same PR if needed.

## Required CI Gates

### In `inactu-cli`
1. `sync-spec-check`: compare mirrored `spec/` + `test-vectors/` with pinned `inactu-spec` commit.
2. `conformance-check`: run spec validation + relevant runtime tests.
3. `release-contract-check`: fail if source pin and compatibility matrices diverge.

### In `inactu-control-web`
1. `sync-openapi-check`: compare `public/openapi.yaml` with pinned `inactu-control` commit.
2. `typed-client-check` (if generated client exists): regenerate and assert no diff.

### In `inactu-spec`
1. `consumer-notify`: emit machine-readable metadata for downstream sync jobs.
2. `breaking-change-guard`: require explicit compatibility annotation for schema-breaking diffs.

## Ownership Model

1. `inactu-spec` maintainers own normative contract acceptance.
2. `inactu-control` maintainers own API contract acceptance.
3. Consumer repos own sync latency SLO:
   - P0/security contract changes: within 24 hours.
   - P1/high changes: within 3 business days.
   - P2/routine changes: within 7 business days.

## Versioning And Pins

1. Every consumer repo stores source pin in a single machine-readable file.
2. Human-readable compatibility docs are generated from that file.
3. Release checks validate:
   - pin consistency across repos for same release train
   - source commit existence and immutability

## PR Policy For Shared Artifacts

A PR that changes mirrored artifacts must include:
1. Source repo + commit SHA.
2. Diff summary (file adds/removes/changes).
3. Regenerated outputs/tests.
4. Compatibility impact statement.
5. Rollback method if consumer regressions appear.

## Minimal Implementation Plan

1. Add `sync-manifest.json` in each consumer repo (source repo + commit + paths + checksums).
2. Add `scripts/check-sync-parity.sh` in each consumer repo.
3. Wire checks into existing CI workflows.
4. Add CODEOWNERS rules for mirror paths requiring designated owners.
5. Add a recurring release checklist step: "verify sync manifests across ecosystem."
