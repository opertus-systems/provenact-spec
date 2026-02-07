use anyhow::Result;
use inactu_conformance_harness::run_all;
use inactu_spec_validate::discover_repo_root;

fn main() -> Result<()> {
    let root = discover_repo_root(".")?;
    let report = run_all(&root)?;
    println!("OK rust-conformance checks={}", report.checks);
    Ok(())
}
