# SECURITY.md

## Security Policy — provenact-spec

`provenact-spec` is a contract repository. Security issues in schemas/vectors can propagate ecosystem-wide.

### Scope

This policy applies to:
- normative schemas in `spec/`
- conformance vectors in `test-vectors/`
- validation tooling and harness crates

### Reporting

Report vulnerabilities privately:
- Email: security@opertus.systems
- Include impacted schema/vector paths and exploit scenario.

Do not open public issues for unpatched vulnerabilities.

### High-Risk Issue Types

- schema ambiguity that permits unsafe interpretation
- vectors that validate insecure behavior as acceptable
- hash/signature contract regressions
- compatibility changes shipped without version boundary
