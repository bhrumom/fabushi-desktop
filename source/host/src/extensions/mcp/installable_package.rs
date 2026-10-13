//! Passive catalog bundle verification inside the existing Host Plugins owner.
//! No download, execution, signing, install authority or renderer API lives here.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{Cursor, Read},
};
use thiserror::Error;

const DOMAIN: &[u8] = b"fabushi.fabapp.manifest.v1\0";
const MAX_ARCHIVE: usize = 100 * 1024 * 1024;
const MAX_EXPANDED: u64 = 250 * 1024 * 1024;
const MAX_MANIFEST: usize = 2 * 1024 * 1024;
const MAX_FILES: usize = 10_000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PackageError {
    #[error("package archive is malformed or unsafe")]
    Archive,
    #[error("package resource or archive limit exceeded")]
    Limit,
    #[error("package manifest is malformed or unsupported")]
    Manifest,
    #[error("package does not match trusted catalog authorization")]
    Authorization,
    #[error("package signature is invalid")]
    Signature,
    #[error("package resource integrity mismatch")]
    Integrity,
}

/// Internal policy input, never deserialized from Web content or renderer IPC.
/// Caller must verify release approval, trusted feed, revocation and current scope.
pub struct CatalogAuthorization {
    pub app_id: String,
    pub publisher_id: String,
    pub version: String,
    pub package_sha256: String,
    pub key_id: String,
    pub public_key: [u8; 32],
    pub trust_epoch: u64,
    pub host_api: u64,
}

pub struct VerifiedPackage {
    manifest: Manifest,
    digest: String,
    resources: BTreeMap<String, Vec<u8>>,
    trust_epoch: u64,
}

impl VerifiedPackage {
    pub fn app_id(&self) -> &str {
        &self.manifest.app_id
    }
    pub fn publisher_id(&self) -> &str {
        &self.manifest.publisher_id
    }
    pub fn version(&self) -> &str {
        &self.manifest.version
    }
    pub fn entrypoint(&self) -> &str {
        &self.manifest.entrypoint
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn trust_epoch(&self) -> u64 {
        self.trust_epoch
    }
    pub fn resource(&self, name: &str) -> Option<(&[u8], &str)> {
        Some((
            self.resources.get(name)?.as_slice(),
            self.manifest.files.get(name)?.mime.as_str(),
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    manifest_version: u64,
    package_class: String,
    app_id: String,
    publisher_id: String,
    version: String,
    display_name: String,
    entrypoint: String,
    runtime: String,
    min_host_api: u64,
    offline_core: bool,
    requested_capabilities: Vec<String>,
    network_policy: NetworkPolicy,
    files: BTreeMap<String, Resource>,
    signature: ManifestSignature,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NetworkPolicy {
    mode: String,
    allowed_origins: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resource {
    sha256: String,
    bytes: u64,
    mime: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ManifestSignature {
    algorithm: String,
    key_id: String,
    value: String,
}

// serde_json::Value normally silently replaces duplicate object keys. Reject
// duplicates recursively before strict typed parsing or canonical signing.
struct UniqueJson(Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("unambiguous JSON")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Bool(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::String(v.into())))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                if v > MAX_SAFE_INTEGER {
                    return Err(E::custom("unsafe integer"));
                }
                Ok(UniqueJson(Value::Number(v.into())))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                if v < 0 {
                    return Err(E::custom("negative integer"));
                }
                self.visit_u64(v as u64)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueJson(value)) = seq.next_element()? {
                    if values.len() >= MAX_FILES {
                        return Err(de::Error::custom("array limit"));
                    }
                    values.push(value);
                }
                Ok(UniqueJson(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((key, UniqueJson(value))) = map.next_entry::<String, UniqueJson>()? {
                    if !key.is_ascii() || values.contains_key(&key) || values.len() >= MAX_FILES {
                        return Err(de::Error::custom("ambiguous object or limit"));
                    }
                    values.insert(key, value);
                }
                Ok(UniqueJson(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && value.is_ascii()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-/".contains(&b))
        && value.split('/').all(|p| {
            let stem = p.split('.').next().unwrap_or_default().to_ascii_lowercase();
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with('.')
                && !["con", "prn", "aux", "nul"].contains(&stem.as_str())
                && !(stem.len() == 4
                    && (stem.starts_with("com") || stem.starts_with("lpt"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}
fn semver(value: &str) -> bool {
    if value.len() > 128 {
        return false;
    }
    let (release, metadata) = value
        .split_once('+')
        .map_or((value, None), |(a, b)| (a, Some(b)));
    let (core, prerelease) = release
        .split_once('-')
        .map_or((release, None), |(a, b)| (a, Some(b)));
    let parts: Vec<_> = core.split('.').collect();
    let ids = |text: &str, pre: bool| {
        text.split('.').all(|p| {
            !p.is_empty()
                && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && !(pre
                    && p.len() > 1
                    && p.starts_with('0')
                    && p.bytes().all(|b| b.is_ascii_digit()))
        })
    };
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.bytes().all(|b| b.is_ascii_digit())
                && (p.len() == 1 || !p.starts_with('0'))
                && p.parse::<u64>().is_ok()
        })
        && prerelease.is_none_or(|s| ids(s, true))
        && metadata.is_none_or(|s| ids(s, false))
}
fn expected_mime(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next()? {
        "html" => Some("text/html"),
        "css" => Some("text/css"),
        "js" => Some("text/javascript"),
        "wasm" => Some("application/wasm"),
        "txt" => Some("text/plain"),
        "json" => Some("application/json"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}
fn canonical(value: &Value) -> Result<Vec<u8>, PackageError> {
    // ASCII field/path keys sort identically in UTF-16 (JCS) and UTF-8.
    // Integers were bounded by UniqueJson. Do not depend on Map feature ordering.
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let ordered: BTreeMap<_, _> =
                    map.iter().map(|(k, v)| (k.clone(), sorted(v))).collect();
                Value::Object(ordered.into_iter().collect())
            }
            Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
            _ => value.clone(),
        }
    }
    serde_json::to_vec(&sorted(value)).map_err(|_| PackageError::Manifest)
}

pub fn verify_catalog_bundle(
    bytes: &[u8],
    policy: &CatalogAuthorization,
) -> Result<VerifiedPackage, PackageError> {
    if bytes.len() > MAX_ARCHIVE {
        return Err(PackageError::Limit);
    }
    let package_digest = digest(bytes);
    if package_digest != policy.package_sha256 || policy.trust_epoch == 0 {
        return Err(PackageError::Authorization);
    }
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| PackageError::Archive)?;
    if archive.len() < 2 || archive.len() > MAX_FILES + 1 {
        return Err(PackageError::Limit);
    }
    let mut expanded = 0u64;
    let mut names = BTreeSet::new();
    let mut resources = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|_| PackageError::Archive)?;
        let name = file.name().to_string();
        let mode = file.unix_mode().unwrap_or(0o100644);
        if !safe_path(&name)
            || !names.insert(name.to_ascii_lowercase())
            || mode & 0o170000 != 0o100000
            || mode & 0o111 != 0
        {
            return Err(PackageError::Archive);
        }
        expanded = expanded
            .checked_add(file.size())
            .ok_or(PackageError::Limit)?;
        if expanded > MAX_EXPANDED
            || file.size() > file.compressed_size().saturating_mul(100)
            || (name == "manifest.json" && file.size() > MAX_MANIFEST as u64)
        {
            return Err(PackageError::Limit);
        }
        let size = file.size();
        let mut data = Vec::new();
        file.by_ref()
            .take(size + 1)
            .read_to_end(&mut data)
            .map_err(|_| PackageError::Archive)?;
        if data.len() as u64 != size {
            return Err(PackageError::Integrity);
        }
        resources.insert(name, data);
    }
    let raw = resources
        .remove("manifest.json")
        .ok_or(PackageError::Manifest)?;
    let UniqueJson(mut value) = serde_json::from_slice(&raw).map_err(|_| PackageError::Manifest)?;
    let manifest: Manifest =
        serde_json::from_value(value.clone()).map_err(|_| PackageError::Manifest)?;
    if manifest.manifest_version != 1
        || manifest.package_class != "catalog"
        || manifest.runtime != "web"
        || !manifest.offline_core
        || !manifest.requested_capabilities.is_empty()
        || manifest.network_policy.mode != "deny"
        || !manifest.network_policy.allowed_origins.is_empty()
        || manifest.min_host_api == 0
        || manifest.min_host_api > policy.host_api
        || !identifier(&manifest.app_id)
        || !identifier(&manifest.publisher_id)
        || !semver(&manifest.version)
        || manifest.display_name.is_empty()
        || manifest.display_name.chars().count() > 160
        || !safe_path(&manifest.entrypoint)
        || manifest.files.is_empty()
    {
        return Err(PackageError::Manifest);
    }
    if manifest.app_id != policy.app_id
        || manifest.publisher_id != policy.publisher_id
        || manifest.version != policy.version
        || manifest.signature.key_id != policy.key_id
    {
        return Err(PackageError::Authorization);
    }
    if manifest.signature.algorithm != "Ed25519" {
        return Err(PackageError::Signature);
    }
    let encoded = URL_SAFE_NO_PAD
        .decode(&manifest.signature.value)
        .map_err(|_| PackageError::Signature)?;
    let signature = Signature::from_slice(&encoded).map_err(|_| PackageError::Signature)?;
    let key = VerifyingKey::from_bytes(&policy.public_key).map_err(|_| PackageError::Signature)?;
    value
        .as_object_mut()
        .ok_or(PackageError::Manifest)?
        .remove("signature");
    let mut payload = DOMAIN.to_vec();
    payload.extend(canonical(&value)?);
    key.verify_strict(&payload, &signature)
        .map_err(|_| PackageError::Signature)?;
    if resources.len() != manifest.files.len() {
        return Err(PackageError::Integrity);
    }
    for (name, item) in &manifest.files {
        if !safe_path(name)
            || expected_mime(name) != Some(item.mime.as_str())
            || item.bytes > MAX_EXPANDED
            || item.sha256.len() != 64
            || !item
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(PackageError::Manifest);
        }
        let data = resources.get(name).ok_or(PackageError::Integrity)?;
        if data.len() as u64 != item.bytes || digest(data) != item.sha256 {
            return Err(PackageError::Integrity);
        }
    }
    if manifest
        .files
        .get(&manifest.entrypoint)
        .map(|r| r.mime.as_str())
        != Some("text/html")
    {
        return Err(PackageError::Manifest);
    }
    Ok(VerifiedPackage {
        manifest,
        digest: package_digest,
        resources,
        trust_epoch: policy.trust_epoch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn archive(items: Vec<(String, Vec<u8>)>) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in items {
            writer
                .start_file(
                    name,
                    SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored)
                        .unix_permissions(0o644),
                )
                .unwrap();
            writer.write_all(&data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    fn fixture(
        mutate: impl FnOnce(&mut Value),
        resource: &[u8],
    ) -> (Vec<u8>, CatalogAuthorization) {
        let key = SigningKey::from_bytes(&[7; 32]); // Test fixture, never production trust.
        let expected = b"<!doctype html><p>offline</p>";
        let mut manifest = serde_json::json!({
            "manifestVersion":1,"packageClass":"catalog","appId":"org.test.app",
            "publisherId":"test-publisher","version":"1.0.0","displayName":"测试 app",
            "entrypoint":"index.html","runtime":"web","minHostApi":1,"offlineCore":true,
            "requestedCapabilities":[],"networkPolicy":{"mode":"deny","allowedOrigins":[]},
            "files":{"index.html":{"sha256":digest(expected),"bytes":expected.len(),"mime":"text/html"}}
        });
        mutate(&mut manifest);
        let mut payload = DOMAIN.to_vec();
        payload.extend(canonical(&manifest).unwrap());
        manifest["signature"] = serde_json::json!({"algorithm":"Ed25519","keyId":"fixture-only",
            "value":URL_SAFE_NO_PAD.encode(key.sign(&payload).to_bytes())});
        let bytes = archive(vec![
            (
                "manifest.json".into(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
            ("index.html".into(), resource.to_vec()),
        ]);
        let policy = CatalogAuthorization {
            app_id: "org.test.app".into(),
            publisher_id: "test-publisher".into(),
            version: "1.0.0".into(),
            package_sha256: digest(&bytes),
            key_id: "fixture-only".into(),
            public_key: key.verifying_key().to_bytes(),
            trust_epoch: 1,
            host_api: 1,
        };
        (bytes, policy)
    }
    const HTML: &[u8] = b"<!doctype html><p>offline</p>";
    #[test]
    fn verifies_signed_content_and_descriptor() {
        let (bytes, policy) = fixture(|_| {}, HTML);
        let verified = verify_catalog_bundle(&bytes, &policy).unwrap();
        assert_eq!(verified.resource("index.html"), Some((HTML, "text/html")));
        assert_eq!(verified.app_id(), "org.test.app");
        assert_eq!(verified.trust_epoch(), 1);
        assert!(verified.resource("../host-token").is_none());
    }
    #[test]
    fn rejects_digest_publisher_and_key_swaps() {
        let (bytes, mut policy) = fixture(|_| {}, HTML);
        policy.package_sha256 = "0".repeat(64);
        assert!(matches!(
            verify_catalog_bundle(&bytes, &policy),
            Err(PackageError::Authorization)
        ));
        policy.package_sha256 = digest(&bytes);
        policy.publisher_id = "other".into();
        assert!(matches!(
            verify_catalog_bundle(&bytes, &policy),
            Err(PackageError::Authorization)
        ));
        policy.publisher_id = "test-publisher".into();
        policy.public_key = SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes();
        assert!(matches!(
            verify_catalog_bundle(&bytes, &policy),
            Err(PackageError::Signature)
        ));
    }
    #[test]
    fn rejects_unknown_fields_capabilities_network_and_host() {
        for field in ["unknown", "capabilities", "network", "host"] {
            let (bytes, policy) = fixture(
                |m| match field {
                    "unknown" => m["admin"] = Value::Bool(true),
                    "capabilities" => {
                        m["requestedCapabilities"] = serde_json::json!(["shell.exec"])
                    }
                    "network" => m["networkPolicy"]["mode"] = Value::String("allowlist".into()),
                    _ => m["minHostApi"] = serde_json::json!(2),
                },
                HTML,
            );
            assert!(matches!(
                verify_catalog_bundle(&bytes, &policy),
                Err(PackageError::Manifest)
            ));
        }
    }
    #[test]
    fn rejects_resource_corruption_and_mime() {
        let (bytes, policy) = fixture(|_| {}, b"changed");
        assert!(matches!(
            verify_catalog_bundle(&bytes, &policy),
            Err(PackageError::Integrity)
        ));
        let (bytes, policy) = fixture(
            |m| m["files"]["index.html"]["mime"] = Value::String("application/octet-stream".into()),
            HTML,
        );
        assert!(matches!(
            verify_catalog_bundle(&bytes, &policy),
            Err(PackageError::Manifest)
        ));
    }
    #[test]
    fn rejects_duplicate_json_and_unsafe_paths() {
        assert!(serde_json::from_str::<UniqueJson>(r#"{"files":{"x":{},"x":{}}}"#).is_err());
        assert!(serde_json::from_str::<UniqueJson>(r#"{"n":9007199254740992}"#).is_err());
        for path in [
            "../x",
            "/x",
            "a/../x",
            "C:/x",
            "a\\x",
            "CON.txt",
            "aux/x",
            "a./x",
            "x//y",
            "中文.js",
        ] {
            assert!(!safe_path(path), "{path}");
        }
        assert!(safe_path("pkg/adapter_bg.wasm"));
    }
    #[test]
    fn rejects_unlisted_and_case_colliding_members() {
        let (bytes, mut policy) = fixture(|_| {}, HTML);
        let mut source = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut manifest = Vec::new();
        source
            .by_name("manifest.json")
            .unwrap()
            .read_to_end(&mut manifest)
            .unwrap();
        for extra in ["rogue.js", "INDEX.HTML", "../escape"] {
            let data = archive(vec![
                ("manifest.json".into(), manifest.clone()),
                ("index.html".into(), HTML.to_vec()),
                (extra.into(), b"x".to_vec()),
            ]);
            policy.package_sha256 = digest(&data);
            assert!(verify_catalog_bundle(&data, &policy).is_err());
        }
    }
    #[test]
    fn strict_versions() {
        for version in ["1.0.0", "1.2.3-alpha.1+build.2"] {
            assert!(semver(version));
        }
        for version in ["1.0", "01.0.0", "1.0.0-", "1.0.0+", "1.0.0-01", "1.0.0+x+y"] {
            assert!(!semver(version));
        }
    }
}
