use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("invalid json: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("canonical json serialization failed")]
    CanonicalJson,
    #[error("invalid sha256 format: {0}")]
    InvalidSha256(String),
    #[error("hash mismatch: expected={expected} actual={actual}")]
    HashMismatch { expected: String, actual: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    pub artifact: String,
    pub capabilities: Vec<Value>,
    pub signers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub version: u64,
    #[serde(default)]
    pub trusted_signers: Vec<String>,
    #[serde(default)]
    pub capability_ceiling: CapabilityCeiling,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityCeiling {
    #[serde(default)]
    pub fs: FsCeiling,
    #[serde(default)]
    pub net: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub kv: KvCeiling,
    #[serde(default)]
    pub queue: QueueCeiling,
    #[serde(default)]
    pub exec: bool,
    #[serde(default)]
    pub time: bool,
    #[serde(default)]
    pub random: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FsCeiling {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KvCeiling {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueCeiling {
    #[serde(default)]
    pub publish: Vec<String>,
    #[serde(default)]
    pub consume: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub artifact: String,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub caps_used: Vec<String>,
    pub timestamp: u64,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub timestamp: u64,
    pub entries: BTreeMap<String, SnapshotEntry>,
    pub snapshot_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub sha256: String,
    pub md5: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvalVector {
    pub name: String,
    pub policy: Policy,
    pub cases: Vec<CapabilityCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCase {
    pub capability: Capability,
    pub expect: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub kind: String,
    pub value: String,
}

pub fn parse_json<T: for<'de> Deserialize<'de>>(raw: &str) -> Result<T, SpecError> {
    Ok(serde_json::from_str(raw)?)
}

pub fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex::encode(digest))
}

pub fn validate_sha256_prefixed(value: &str) -> Result<(), SpecError> {
    if !value.starts_with("sha256:") || value.len() != 71 {
        return Err(SpecError::InvalidSha256(value.to_string()));
    }
    if !value.as_bytes()[7..].iter().all(u8::is_ascii_hexdigit) {
        return Err(SpecError::InvalidSha256(value.to_string()));
    }
    if value.chars().skip(7).any(|c| c.is_ascii_uppercase()) {
        return Err(SpecError::InvalidSha256(value.to_string()));
    }
    Ok(())
}

pub fn to_jcs_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, SpecError> {
    serde_jcs::to_vec(value).map_err(|_| SpecError::CanonicalJson)
}

pub fn compute_manifest_hash(manifest: &Manifest) -> Result<String, SpecError> {
    Ok(sha256_prefixed(&to_jcs_bytes(manifest)?))
}

pub fn compute_policy_hash(policy: &Policy) -> Result<String, SpecError> {
    Ok(sha256_prefixed(&to_jcs_bytes(policy)?))
}

pub fn compute_receipt_hash(receipt: &ExecutionReceipt) -> Result<String, SpecError> {
    let payload = serde_json::json!({
        "artifact": receipt.artifact,
        "inputs_hash": receipt.inputs_hash,
        "outputs_hash": receipt.outputs_hash,
        "caps_used": receipt.caps_used,
        "timestamp": receipt.timestamp,
    });
    Ok(sha256_prefixed(&to_jcs_bytes(&payload)?))
}

pub fn verify_receipt_hash(receipt: &ExecutionReceipt) -> Result<(), SpecError> {
    validate_sha256_prefixed(&receipt.receipt_hash)?;
    let actual = compute_receipt_hash(receipt)?;
    if actual == receipt.receipt_hash {
        Ok(())
    } else {
        Err(SpecError::HashMismatch {
            expected: receipt.receipt_hash.clone(),
            actual,
        })
    }
}

pub fn compute_snapshot_hash(snapshot: &RegistrySnapshot) -> Result<String, SpecError> {
    let payload = serde_json::json!({
        "timestamp": snapshot.timestamp,
        "entries": snapshot.entries,
    });
    Ok(sha256_prefixed(&to_jcs_bytes(&payload)?))
}

pub fn verify_snapshot_hash(snapshot: &RegistrySnapshot) -> Result<(), SpecError> {
    validate_sha256_prefixed(&snapshot.snapshot_hash)?;
    let actual = compute_snapshot_hash(snapshot)?;
    if actual == snapshot.snapshot_hash {
        Ok(())
    } else {
        Err(SpecError::HashMismatch {
            expected: snapshot.snapshot_hash.clone(),
            actual,
        })
    }
}

fn normalize_fs_path(path: &str) -> Option<String> {
    if !path.starts_with('/') || path.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return None;
        }
        parts.push(segment);
    }
    Some(format!("/{}", parts.join("/")))
}

fn is_within_prefix(candidate: &str, prefix: &str) -> bool {
    if prefix == "/" {
        return candidate.starts_with('/');
    }
    candidate == prefix || candidate.starts_with(&format!("{prefix}/"))
}

fn normalize_uri_path(path: &str) -> Option<String> {
    let raw = if path.is_empty() { "/" } else { path };
    normalize_fs_path(raw)
}

fn net_uri_within_prefix(requested: &Url, allowed: &Url) -> bool {
    if !requested.has_authority() || !allowed.has_authority() {
        return false;
    }
    if requested.scheme() != allowed.scheme() {
        return false;
    }
    if requested.host_str() != allowed.host_str() {
        return false;
    }
    if requested.port_or_known_default() != allowed.port_or_known_default() {
        return false;
    }
    if requested.username() != allowed.username() || requested.password() != allowed.password() {
        return false;
    }
    if requested.fragment().is_some() || allowed.query().is_some() || allowed.fragment().is_some() {
        return false;
    }
    let Some(requested_path) = normalize_uri_path(requested.path()) else {
        return false;
    };
    let Some(allowed_path) = normalize_uri_path(allowed.path()) else {
        return false;
    };
    is_within_prefix(&requested_path, &allowed_path)
}

pub fn evaluate_capability(policy: &Policy, capability: &Capability) -> bool {
    match capability.kind.as_str() {
        "exec" => capability.value == "true" && policy.capability_ceiling.exec,
        "exec.safe" => !capability.value.is_empty() && policy.capability_ceiling.exec,
        "time" => policy.capability_ceiling.time,
        "time.now" => !capability.value.is_empty() && policy.capability_ceiling.time,
        "random.bytes" => !capability.value.is_empty() && policy.capability_ceiling.random,
        "env" => policy
            .capability_ceiling
            .env
            .iter()
            .any(|x| x == &capability.value),
        "net" | "net.http" => {
            let Ok(requested) = Url::parse(&capability.value) else {
                return false;
            };
            policy.capability_ceiling.net.iter().any(|allowed| {
                Url::parse(allowed)
                    .ok()
                    .map(|prefix| net_uri_within_prefix(&requested, &prefix))
                    .unwrap_or(false)
            })
        }
        "fs.read" | "fs.write" => {
            let value = match normalize_fs_path(&capability.value) {
                Some(v) => v,
                None => return false,
            };
            let allowed = if capability.kind == "fs.read" {
                &policy.capability_ceiling.fs.read
            } else {
                &policy.capability_ceiling.fs.write
            };
            allowed.iter().any(|prefix| {
                normalize_fs_path(prefix)
                    .map(|p| is_within_prefix(&value, &p))
                    .unwrap_or(false)
            })
        }
        "kv.read" => policy
            .capability_ceiling
            .kv
            .read
            .iter()
            .any(|item| item == "*" || item == &capability.value),
        "kv.write" => policy
            .capability_ceiling
            .kv
            .write
            .iter()
            .any(|item| item == "*" || item == &capability.value),
        "queue.publish" => policy
            .capability_ceiling
            .queue
            .publish
            .iter()
            .any(|item| item == "*" || item == &capability.value),
        "queue.consume" => policy
            .capability_ceiling
            .queue
            .consume
            .iter()
            .any(|item| item == "*" || item == &capability.value),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_hash_round_trip() {
        let mut receipt = ExecutionReceipt {
            artifact: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into(),
            inputs_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .into(),
            outputs_hash: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .into(),
            caps_used: vec!["fs.read".into()],
            timestamp: 1,
            receipt_hash: String::new(),
        };
        receipt.receipt_hash = compute_receipt_hash(&receipt).unwrap();
        verify_receipt_hash(&receipt).unwrap();
    }
}
