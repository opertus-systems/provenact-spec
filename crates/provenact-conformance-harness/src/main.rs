use anyhow::Result;
use provenact_conformance_harness::run_all;
use provenact_spec_validate::discover_repo_root;

fn main() -> Result<()> {
    let root = discover_repo_root(".")?;
    let report = run_all(&root)?;
    println!("OK rust-conformance checks={}", report.checks);
    Ok(())
}
