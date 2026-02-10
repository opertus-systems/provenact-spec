# Repository Remediation Backlog

Date: 2026-02-07
Scope: `provenact-agent-kit`, `provenact-cli`, `provenact-control`, `provenact-control-web`, `provenact-examples`, `provenact-sdk`, `provenact-skills`, `provenact-spec`.

This backlog turns audit findings into trackable cross-repo work with severity, ownership, and concrete pull request gates.

## P0 (Blocker)

### P0-1: Patch `time` DoS advisory in Rust lockfiles
- Severity: P0
- Owner repo(s): `provenact-cli`, `provenact-spec`
- Finding: `RUSTSEC-2026-0009` in `time 0.3.36` (published 2026-02-05), upgrade to `>=0.3.47`.
- PR checklist:
1. Upgrade dependency graph so lockfiles no longer resolve `time 0.3.36`.
2. Commit updated lockfiles.
3. Run `cargo test` in each impacted workspace.
4. Run `cargo audit` and attach clean output.
5. Add CHANGELOG entries noting advisory remediation.

### P0-2: Fix broken `provenact-agent-kit` unit contract
- Severity: P0
- Owner repo(s): `provenact-agent-kit`
- Finding: `adapter_executes_verify_then_run` fails because `keys_digest` is required but test passes `None`.
- PR checklist:
1. Decide contract: either make `keys_digest` required in request type, or permit `None` consistently.
2. Update test and adapter behavior to match the chosen contract.
3. Add at least one negative test proving invalid/missing digest behavior.
4. Run `cargo test` and include passing output in PR.
5. Document the contract clearly in README example.

## P1 (High)

### P1-1: Unify substrate pin policy across ecosystem
- Severity: P1
- Owner repo(s): `provenact-sdk`, `provenact-control`, `provenact-skills`, `provenact-cli`
- Finding: `provenact-skills` pin differs from `provenact-sdk`/`provenact-control`; "tested" claims are inconsistent cross-repo.
- PR checklist:
1. Choose shared pin strategy: single commit pin per release train.
2. Update all compatibility matrices to the same substrate pin for a train.
3. Add cross-repo pin validation in release contract checks.
4. Fail CI when compatibility docs and manifest pins diverge.
5. Record exceptions with date and rationale if divergence is intentional.

### P1-2: Move replay/rate limits from memory to shared store
- Severity: P1
- Owner repo(s): `provenact-control`
- Finding: replay cache and rate windows are per-process; not robust for multi-instance deployments.
- PR checklist:
1. Implement replay cache in durable/shared store (for example Postgres or Redis).
2. Implement distributed rate limiting keyed by authenticated subject.
3. Add migration/schema/docs for TTL and cleanup behavior.
4. Add integration tests for multi-instance semantics.
5. Document operational defaults and failure modes.

### P1-3: Add web auth abuse protections
- Severity: P1
- Owner repo(s): `provenact-control-web`
- Finding: registration endpoint lacks explicit throttle/abuse controls.
- PR checklist:
1. Add IP/user-agent throttling for `/api/auth/register`.
2. Add optional challenge or email verification mode toggle.
3. Add structured audit logs for auth events.
4. Add tests for brute-force and flood behavior.
5. Document production hardening defaults in README.

### P1-4: Close baseline web header gap
- Severity: P1
- Owner repo(s): `provenact-control-web`
- Finding: headers do not include explicit CSP/HSTS baseline.
- PR checklist:
1. Define CSP for app routes and API routes.
2. Enable HSTS in production mode.
3. Add regression test/check that required headers are present.
4. Document local-dev exceptions.
5. Validate compatibility with NextAuth flows.

## P2 (Medium)

### P2-1: Standardize legal/security doc coverage
- Severity: P2
- Owner repo(s): all repos
- Finding: inconsistent presence of `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`.
- PR checklist:
1. Adopt repo baseline policy (required docs by repo type).
2. Add missing docs in `provenact-agent-kit`, `provenact-control-web`, `provenact-examples`, `provenact-spec`.
3. Ensure all docs include security reporting route and support window.
4. Add CI check for required doc files.
5. Link each repo README to those docs.

### P2-2: Remove stale monorepo/legacy links
- Severity: P2
- Owner repo(s): `provenact-control-web`, `provenact-control`, `provenact-cli`, `provenact-spec`
- Finding: mixed references to historical `opertus-systems/provenact` paths.
- PR checklist:
1. Canonicalize repository URLs in docs and site pages.
2. Replace stale links to current split-repo locations.
3. Add link checker in CI.
4. Add redirect map doc for old references.
5. Confirm all public docs resolve correctly.

### P2-3: Align naming conventions for binaries/services
- Severity: P2
- Owner repo(s): `provenact-control`, `provenact-control-web`
- Finding: backend binary named `provenact-control-web` causes role confusion.
- PR checklist:
1. Decide naming convention: `*-api`, `*-web`, `*-cli`.
2. Rename binary and docs consistently.
3. Provide compatibility shim/alias during migration.
4. Update Docker and compose assets.
5. Note change in changelog.

## Cross-Repo PR Template (Use For Every Remediation PR)

1. Problem statement with direct finding reference.
2. Threat/impact statement.
3. Scope boundaries (what is explicitly out of scope).
4. Tests run and results.
5. Docs updated (list files).
6. Compatibility impact and migration notes.
7. Rollback plan.
