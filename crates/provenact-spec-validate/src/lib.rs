use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SchemaStore {
    root: PathBuf,
    schemas: HashMap<String, Value>,
}

impl SchemaStore {
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut schemas = HashMap::new();
        for rel in [
            "spec/v0/skill-manifest.schema.json",
            "spec/v0/pipeline.schema.json",
            "spec/policy/policy.schema.json",
            "spec/execution-receipt.schema.json",
            "spec/execution-receipt.v1.experimental.schema.json",
            "spec/registry/snapshot.schema.json",
            "spec/skill-format/manifest.schema.json",
            "spec/skill-format/provenance.schema.json",
            "spec/skill-format/signatures.schema.json",
            "spec/skill-format/manifest.v1.experimental.schema.json",
            "test-vectors/capability-eval/schema.json",
        ] {
            let schema_path = root.join(rel);
            let schema_text = fs::read_to_string(&schema_path)
                .with_context(|| format!("reading schema {}", schema_path.display()))?;
            let schema_json: Value = serde_json::from_str(&schema_text)
                .with_context(|| format!("parsing schema {}", schema_path.display()))?;
            schemas.insert(rel.to_string(), schema_json);
        }
        Ok(Self { root, schemas })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn validate_value(&self, schema_rel: &str, value: &Value) -> Result<()> {
        let schema = self
            .schemas
            .get(schema_rel)
            .ok_or_else(|| anyhow!("unknown schema {}", schema_rel))?;
        if !jsonschema::is_valid(schema, value) {
            bail!("schema {} validation failed", schema_rel);
        }
        Ok(())
    }

    pub fn parse_doc_file(&self, rel: &str) -> Result<Value> {
        let path = self.root.join(rel);
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        if rel.ends_with(".yaml") || rel.ends_with(".yml") {
            let yaml: serde_yaml::Value = serde_yaml::from_str(&raw)
                .with_context(|| format!("parsing yaml {}", path.display()))?;
            let json = serde_json::to_value(yaml)?;
            Ok(json)
        } else {
            let json = serde_json::from_str(&raw)
                .with_context(|| format!("parsing json {}", path.display()))?;
            Ok(json)
        }
    }

    pub fn validate_file(&self, schema_rel: &str, doc_rel: &str) -> Result<()> {
        let value = self.parse_doc_file(doc_rel)?;
        self.validate_value(schema_rel, &value)
    }
}

pub fn discover_repo_root(from: impl AsRef<Path>) -> Result<PathBuf> {
    let mut cur = from.as_ref().canonicalize()?;
    loop {
        let candidate = cur
            .join("spec")
            .join("v0")
            .join("skill-manifest.schema.json");
        if candidate.exists() {
            return Ok(cur);
        }
        if !cur.pop() {
            bail!(
                "could not discover provenact-spec repository root from {}",
                from.as_ref().display()
            );
        }
    }
}
