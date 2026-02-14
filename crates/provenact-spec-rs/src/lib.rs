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
    #[error("invalid md5 format: {0}")]
    InvalidMd5(String),
    #[error("hash mismatch: expected={expected} actual={actual}")]
    HashMismatch { expected: String, actual: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    pub artifact: String,
    pub capabilities: Vec<Value>,
    pub signers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub version: u64,
    #[serde(default)]
    pub trusted_signers: Vec<String>,
    #[serde(default)]
    pub capability_ceiling: CapabilityCeiling,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct FsCeiling {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct KvCeiling {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct QueueCeiling {
    #[serde(default)]
    pub publish: Vec<String>,
    #[serde(default)]
    pub consume: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReceipt {
    pub artifact: String,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub caps_used: Vec<String>,
    pub timestamp: u64,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySnapshot {
    pub timestamp: u64,
    pub entries: BTreeMap<String, SnapshotEntry>,
    pub snapshot_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotEntry {
    pub sha256: String,
    pub md5: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityEvalVector {
    pub name: String,
    pub policy: Policy,
    pub cases: Vec<CapabilityCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityCase {
    pub capability: Capability,
    pub expect: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

fn validate_md5_hex(value: &str) -> Result<(), SpecError> {
    if value.len() != 32 {
        return Err(SpecError::InvalidMd5(value.to_string()));
    }
    if !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(SpecError::InvalidMd5(value.to_string()));
    }
    if value.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(SpecError::InvalidMd5(value.to_string()));
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
    validate_sha256_prefixed(&receipt.artifact)?;
    validate_sha256_prefixed(&receipt.inputs_hash)?;
    validate_sha256_prefixed(&receipt.outputs_hash)?;
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
    for entry in snapshot.entries.values() {
        validate_sha256_prefixed(&entry.sha256)?;
        validate_md5_hex(&entry.md5)?;
    }
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
        if segment.is_empty() {
            continue;
        }
        if segment == "." {
            return None;
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
    if raw.contains('\\') || contains_pct_encoded_triplet(raw) {
        return None;
    }
    normalize_fs_path(raw)
}

fn contains_pct_encoded_triplet(value: &str) -> bool {
    value.as_bytes().windows(3).any(|window| {
        window[0] == b'%' && window[1].is_ascii_hexdigit() && window[2].is_ascii_hexdigit()
    })
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
        "time.now" => !capability.value.is_empty() && policy.capability_ceiling.time,
        "random.bytes" => !capability.value.is_empty() && policy.capability_ceiling.random,
        "env" => {
            if !is_valid_env_name(&capability.value) {
                return false;
            }
            policy
                .capability_ceiling
                .env
                .iter()
                .any(|x| x == &capability.value)
        }
        "net.http" => {
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
        "kv.read" => {
            policy.capability_ceiling.kv.read.iter().any(|item| {
                !capability.value.is_empty() && (item == "*" || item == &capability.value)
            })
        }
        "kv.write" => {
            policy.capability_ceiling.kv.write.iter().any(|item| {
                !capability.value.is_empty() && (item == "*" || item == &capability.value)
            })
        }
        "queue.publish" => {
            policy.capability_ceiling.queue.publish.iter().any(|item| {
                !capability.value.is_empty() && (item == "*" || item == &capability.value)
            })
        }
        "queue.consume" => {
            policy.capability_ceiling.queue.consume.iter().any(|item| {
                !capability.value.is_empty() && (item == "*" || item == &capability.value)
            })
        }
        _ => false,
    }
}

fn is_valid_env_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_uppercase() || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sha(c: char) -> String {
        format!("sha256:{}", c.to_string().repeat(64))
    }

    #[test]
    fn receipt_hash_round_trip() {
        let mut receipt = ExecutionReceipt {
            artifact: sha('a'),
            inputs_hash: sha('b'),
            outputs_hash: sha('c'),
            caps_used: vec!["fs.read".into()],
            timestamp: 1,
            receipt_hash: String::new(),
        };
        receipt.receipt_hash = compute_receipt_hash(&receipt).unwrap();
        verify_receipt_hash(&receipt).unwrap();
    }

    #[test]
    fn receipt_hash_rejects_invalid_component_digests() {
        let mut receipt = ExecutionReceipt {
            artifact: sha('a'),
            inputs_hash: sha('b'),
            outputs_hash: sha('c'),
            caps_used: vec!["fs.read".into()],
            timestamp: 1,
            receipt_hash: String::new(),
        };
        receipt.receipt_hash = compute_receipt_hash(&receipt).unwrap();

        let invalid_artifact = format!("sha256:{}A", "a".repeat(63));
        let mut artifact_bad = receipt.clone();
        artifact_bad.artifact = invalid_artifact.clone();
        assert!(matches!(
            verify_receipt_hash(&artifact_bad),
            Err(SpecError::InvalidSha256(value)) if value == invalid_artifact
        ));

        let invalid_inputs = "sha256:short".to_string();
        let mut inputs_bad = receipt.clone();
        inputs_bad.inputs_hash = invalid_inputs.clone();
        assert!(matches!(
            verify_receipt_hash(&inputs_bad),
            Err(SpecError::InvalidSha256(value)) if value == invalid_inputs
        ));

        let invalid_outputs = "md5:ffffffffffffffffffffffffffffffff".to_string();
        let mut outputs_bad = receipt.clone();
        outputs_bad.outputs_hash = invalid_outputs.clone();
        assert!(matches!(
            verify_receipt_hash(&outputs_bad),
            Err(SpecError::InvalidSha256(value)) if value == invalid_outputs
        ));
    }

    #[test]
    fn parse_json_rejects_unknown_fields() {
        let raw = serde_json::json!({
            "artifact": sha('a'),
            "inputs_hash": sha('b'),
            "outputs_hash": sha('c'),
            "caps_used": ["fs.read"],
            "timestamp": 1,
            "receipt_hash": sha('d'),
            "unexpected": true
        })
        .to_string();
        assert!(matches!(
            parse_json::<ExecutionReceipt>(&raw),
            Err(SpecError::InvalidJson(_))
        ));
    }

    #[test]
    fn snapshot_hash_rejects_malformed_entry_digests() {
        let mut snapshot_bad_sha = RegistrySnapshot {
            timestamp: 1,
            entries: BTreeMap::from([(
                "skill-a".to_string(),
                SnapshotEntry {
                    sha256: "sha256:not-a-real-digest".to_string(),
                    md5: "0123456789abcdef0123456789abcdef".to_string(),
                },
            )]),
            snapshot_hash: String::new(),
        };
        snapshot_bad_sha.snapshot_hash = compute_snapshot_hash(&snapshot_bad_sha).unwrap();
        assert!(matches!(
            verify_snapshot_hash(&snapshot_bad_sha),
            Err(SpecError::InvalidSha256(value)) if value == "sha256:not-a-real-digest"
        ));

        let invalid_md5 = "0123456789abcdef0123456789abcdeF".to_string();
        let mut snapshot_bad_md5 = RegistrySnapshot {
            timestamp: 1,
            entries: BTreeMap::from([(
                "skill-b".to_string(),
                SnapshotEntry {
                    sha256: sha('a'),
                    md5: invalid_md5.clone(),
                },
            )]),
            snapshot_hash: String::new(),
        };
        snapshot_bad_md5.snapshot_hash = compute_snapshot_hash(&snapshot_bad_md5).unwrap();
        assert!(matches!(
            verify_snapshot_hash(&snapshot_bad_md5),
            Err(SpecError::InvalidMd5(value)) if value == invalid_md5
        ));
    }

    #[test]
    fn time_now_capability_requires_non_empty_value() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                time: true,
                ..CapabilityCeiling::default()
            },
        };
        let allowed = Capability {
            kind: "time.now".to_string(),
            value: "utc".to_string(),
        };
        let denied = Capability {
            kind: "time.now".to_string(),
            value: "".to_string(),
        };
        assert!(evaluate_capability(&policy, &allowed));
        assert!(!evaluate_capability(&policy, &denied));
    }

    #[test]
    fn env_capability_requires_posix_style_name() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                env: vec!["HOME".to_string(), "PATH".to_string()],
                ..CapabilityCeiling::default()
            },
        };
        let allowed = Capability {
            kind: "env".to_string(),
            value: "HOME".to_string(),
        };
        let denied = Capability {
            kind: "env".to_string(),
            value: "home".to_string(),
        };
        assert!(evaluate_capability(&policy, &allowed));
        assert!(!evaluate_capability(&policy, &denied));
    }

    #[test]
    fn net_http_rejects_percent_encoded_path_bytes() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                net: vec!["https://api.example.test/v1".to_string()],
                ..CapabilityCeiling::default()
            },
        };
        let escaped = Capability {
            kind: "net.http".to_string(),
            value: "https://api.example.test/v1/%2f..%2fadmin".to_string(),
        };
        assert!(!evaluate_capability(&policy, &escaped));
    }

    #[test]
    fn net_http_accepts_equivalent_default_https_port() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                net: vec!["https://api.example.test/v1".to_string()],
                ..CapabilityCeiling::default()
            },
        };
        let requested = Capability {
            kind: "net.http".to_string(),
            value: "https://api.example.test:443/v1/forecast".to_string(),
        };
        assert!(evaluate_capability(&policy, &requested));
    }

    #[test]
    fn fs_capability_rejects_dot_segment_paths() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                fs: FsCeiling {
                    read: vec!["/tmp".to_string()],
                    write: vec!["/tmp".to_string()],
                },
                ..CapabilityCeiling::default()
            },
        };
        let requested = Capability {
            kind: "fs.read".to_string(),
            value: "/tmp/./report.json".to_string(),
        };
        assert!(!evaluate_capability(&policy, &requested));
    }

    #[test]
    fn kv_and_queue_capabilities_require_non_empty_values() {
        let policy = Policy {
            version: 1,
            trusted_signers: vec!["alice.dev".to_string()],
            capability_ceiling: CapabilityCeiling {
                kv: KvCeiling {
                    read: vec!["*".to_string()],
                    write: vec!["*".to_string()],
                },
                queue: QueueCeiling {
                    publish: vec!["*".to_string()],
                    consume: vec!["*".to_string()],
                },
                ..CapabilityCeiling::default()
            },
        };

        let kv_empty = Capability {
            kind: "kv.read".to_string(),
            value: String::new(),
        };
        let queue_empty = Capability {
            kind: "queue.publish".to_string(),
            value: String::new(),
        };
        let kv_non_empty = Capability {
            kind: "kv.read".to_string(),
            value: "user-profile".to_string(),
        };
        let queue_non_empty = Capability {
            kind: "queue.publish".to_string(),
            value: "jobs".to_string(),
        };

        assert!(!evaluate_capability(&policy, &kv_empty));
        assert!(!evaluate_capability(&policy, &queue_empty));
        assert!(evaluate_capability(&policy, &kv_non_empty));
        assert!(evaluate_capability(&policy, &queue_non_empty));
    }
}
