# Remediation Execution Board

Date: 2026-02-07
Source backlog: `docs/repo-remediation-backlog.md`

Use this board to open and track concrete remediation issues across repositories.

## Board

| ID | Priority | Owner Repo(s) | Status | Target Date | Issue Link |
| --- | --- | --- | --- | --- | --- |
| P0-1 | P0 | `provenact-cli`, `provenact-spec` | Todo | 2026-02-10 | _create issue_ |
| P0-2 | P0 | `provenact-agent-kit` | Todo | 2026-02-10 | _create issue_ |
| P1-1 | P1 | `provenact-sdk`, `provenact-control`, `provenact-skills`, `provenact-cli` | Todo | 2026-02-14 | _create issue_ |
| P1-2 | P1 | `provenact-control` | Todo | 2026-02-18 | _create issue_ |
| P1-3 | P1 | `provenact-control-web` | Todo | 2026-02-18 | _create issue_ |
| P1-4 | P1 | `provenact-control-web` | Todo | 2026-02-18 | _create issue_ |
| P2-1 | P2 | all repos | Todo | 2026-02-24 | _create issue_ |
| P2-2 | P2 | `provenact-control-web`, `provenact-control`, `provenact-cli`, `provenact-spec` | Todo | 2026-02-24 | _create issue_ |
| P2-3 | P2 | `provenact-control`, `provenact-control-web` | Todo | 2026-02-24 | _create issue_ |

## Issue Template (Copy/Paste)

Title:
`[<ID>] <short remediation title>`

Body:

```md
## Problem
Finding ID: <ID>
Backlog reference: docs/repo-remediation-backlog.md

## Impact
- Security/consistency/elegance risk:
- Affected repos:
- Release impact:

## Scope
In scope:
- ...

Out of scope:
- ...

## Acceptance Criteria
1. ...
2. ...
3. ...

## PR Checklist
1. Problem statement with direct finding reference.
2. Threat/impact statement.
3. Scope boundaries (what is explicitly out of scope).
4. Tests run and results.
5. Docs updated (list files).
6. Compatibility impact and migration notes.
7. Rollback plan.
```

## Starter Issue Bodies

### P0-1
- Title: `[P0-1] Patch time crate DoS advisory in provenact-cli and provenact-spec`
- Acceptance criteria:
1. No `time 0.3.36` in lockfiles.
2. `cargo audit` clean for advisory `RUSTSEC-2026-0009`.
3. Tests pass in both repos.

### P0-2
- Title: `[P0-2] Resolve keys_digest contract drift in provenact-agent-kit`
- Acceptance criteria:
1. Unit tests pass in `provenact-agent-kit`.
2. Behavior and README contract match.
3. Negative test covers missing/invalid digest behavior.

### P1-1
- Title: `[P1-1] Enforce cross-repo substrate pin consistency for release train`
- Acceptance criteria:
1. Compatibility docs align on release-train pin policy.
2. Release contract check fails on pin divergence.
3. Exception process documented.

### P1-2
- Title: `[P1-2] Make replay and rate-limit controls multi-instance safe`
- Acceptance criteria:
1. Replay and rate state are no longer in-process only.
2. Integration tests cover multi-instance semantics.
3. Ops docs include failure modes and TTL behavior.

### P1-3
- Title: `[P1-3] Add registration abuse protections in provenact-control-web`
- Acceptance criteria:
1. Registration route has explicit throttling.
2. Tests cover flood/brute-force cases.
3. README documents production defaults.

### P1-4
- Title: `[P1-4] Add CSP and HSTS baseline headers in provenact-control-web`
- Acceptance criteria:
1. CSP and HSTS are enforced in production mode.
2. Header presence is tested.
3. Local-dev exceptions are documented.

### P2-1
- Title: `[P2-1] Standardize minimum legal/security docs across all repos`
- Acceptance criteria:
1. Required docs exist in each repo class.
2. CI checks required files.
3. READMEs link to policy docs.

### P2-2
- Title: `[P2-2] Eliminate stale monorepo and legacy organization links`
- Acceptance criteria:
1. Links updated to canonical locations.
2. Link checker enabled in CI.
3. Redirect map documented.

### P2-3
- Title: `[P2-3] Align control-plane binary and service naming conventions`
- Acceptance criteria:
1. Naming convention documented and applied.
2. Transitional alias/shim provided.
3. Docker/compose/docs updated.
