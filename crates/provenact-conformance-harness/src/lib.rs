use anyhow::{bail, Context, Result};
use provenact_spec_rs::{
    evaluate_capability, parse_json, verify_receipt_hash, verify_snapshot_hash,
    CapabilityEvalVector, ExecutionReceipt, RegistrySnapshot,
};
use provenact_spec_validate::SchemaStore;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Report {
    pub checks: usize,
}

fn files(root: &Path, rel_dir: &str) -> Result<Vec<String>> {
    let dir = root.join(rel_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let p = entry.path();
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or_default();
            if matches!(ext, "json" | "yaml" | "yml") {
                out.push(
                    p.strip_prefix(root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    out.sort();
    Ok(out)
}

fn check_schema_group(
    store: &SchemaStore,
    schema: &str,
    good_dir: &str,
    bad_dir: &str,
    semantic_bad: &[&str],
) -> Result<usize> {
    let mut checks = 0usize;
    let good = files(store.root(), good_dir)?;
    let bad = files(store.root(), bad_dir)?;

    for f in good {
        store
            .validate_file(schema, &f)
            .with_context(|| format!("expected valid: {f}"))?;
        checks += 1;
    }

    for f in bad {
        if semantic_bad.iter().any(|x| *x == f) {
            continue;
        }
        let result = store.validate_file(schema, &f);
        if result.is_ok() {
            bail!("expected invalid: {f}");
        }
        checks += 1;
    }

    Ok(checks)
}

fn check_capability_vectors(store: &SchemaStore) -> Result<usize> {
    let mut checks = 0usize;
    for file in files(store.root(), "test-vectors/capability-eval")? {
        if file.ends_with("schema.json") {
            continue;
        }
        let value = store.parse_doc_file(&file)?;
        store.validate_value("test-vectors/capability-eval/schema.json", &value)?;
        let vector: CapabilityEvalVector = serde_json::from_value(value)?;
        checks += 1;
        for case in vector.cases {
            let got = if evaluate_capability(&vector.policy, &case.capability) {
                "allow"
            } else {
                "deny"
            };
            if got != case.expect {
                bail!(
                    "capability mismatch in {} for {}:{} expected={} actual={}",
                    file,
                    case.capability.kind,
                    case.capability.value,
                    case.expect,
                    got
                );
            }
            checks += 1;
        }
    }
    Ok(checks)
}

fn check_hash_semantics(store: &SchemaStore) -> Result<usize> {
    let mut checks = 0usize;

    for good in files(store.root(), "test-vectors/receipt/good")? {
        let raw = std::fs::read_to_string(store.root().join(&good))?;
        let receipt: ExecutionReceipt = parse_json(&raw)?;
        verify_receipt_hash(&receipt)?;
        checks += 1;
    }
    for bad in files(store.root(), "test-vectors/receipt/bad")? {
        let raw = std::fs::read_to_string(store.root().join(&bad))?;
        let receipt: ExecutionReceipt = parse_json(&raw)?;
        if verify_receipt_hash(&receipt).is_ok() {
            bail!("expected receipt hash verification failure: {bad}");
        }
        checks += 1;
    }

    for good in files(store.root(), "test-vectors/registry/snapshot/good")? {
        let raw = std::fs::read_to_string(store.root().join(&good))?;
        let snapshot: RegistrySnapshot = parse_json(&raw)?;
        verify_snapshot_hash(&snapshot)?;
        checks += 1;
    }

    for bad in files(store.root(), "test-vectors/registry/snapshot/bad")? {
        let raw = std::fs::read_to_string(store.root().join(&bad))?;
        let snapshot: RegistrySnapshot = parse_json(&raw)?;
        if bad.ends_with("hash-mismatch.json") {
            if verify_snapshot_hash(&snapshot).is_ok() {
                bail!("expected snapshot hash mismatch: {bad}");
            }
        }
        checks += 1;
    }

    Ok(checks)
}

pub fn run_all(root: &Path) -> Result<Report> {
    let store = SchemaStore::load(root)?;
    let mut checks = 0usize;

    checks += check_schema_group(
        &store,
        "spec/v0/skill-manifest.schema.json",
        "test-vectors/v0/skill-manifest/good",
        "test-vectors/v0/skill-manifest/bad",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/v0/pipeline.schema.json",
        "test-vectors/v0/pipeline/good",
        "test-vectors/v0/pipeline/bad",
        &["test-vectors/v0/pipeline/bad/cycle.json"],
    )?;
    checks += check_schema_group(
        &store,
        "spec/policy/policy.schema.json",
        "test-vectors/policy/valid",
        "test-vectors/policy/invalid",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/execution-receipt.schema.json",
        "test-vectors/receipt/good",
        "test-vectors/receipt/bad",
        &["test-vectors/receipt/bad/hash-mismatch.json"],
    )?;
    checks += check_schema_group(
        &store,
        "spec/execution-receipt.v1.experimental.schema.json",
        "test-vectors/receipt-v1/good",
        "test-vectors/receipt-v1/bad",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/registry/snapshot.schema.json",
        "test-vectors/registry/snapshot/good",
        "test-vectors/registry/snapshot/bad",
        &["test-vectors/registry/snapshot/bad/hash-mismatch.json"],
    )?;
    checks += check_schema_group(
        &store,
        "spec/skill-format/manifest.schema.json",
        "test-vectors/skill-format/manifest/good",
        "test-vectors/skill-format/manifest/bad",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/skill-format/provenance.schema.json",
        "test-vectors/skill-format/provenance/good",
        "test-vectors/skill-format/provenance/bad",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/skill-format/signatures.schema.json",
        "test-vectors/skill-format/signatures/good",
        "test-vectors/skill-format/signatures/bad",
        &[],
    )?;
    checks += check_schema_group(
        &store,
        "spec/skill-format/manifest.v1.experimental.schema.json",
        "test-vectors/skill-format/manifest-v1/good",
        "test-vectors/skill-format/manifest-v1/bad",
        &[],
    )?;

    checks += check_capability_vectors(&store)?;
    checks += check_hash_semantics(&store)?;

    Ok(Report { checks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use provenact_spec_validate::discover_repo_root;

    #[test]
    fn conformance_suite_passes() {
        let root = discover_repo_root(".").unwrap();
        let report = run_all(&root).unwrap();
        assert!(report.checks > 40);
    }
}
