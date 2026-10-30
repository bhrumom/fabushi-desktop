use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::host_paths::get_host_secrets_path;

static MACHINE_ID_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn machine_id_cache() -> &'static Mutex<Option<String>> {
    MACHINE_ID_CACHE.get_or_init(|| Mutex::new(None))
}

pub fn get_or_create_host_machine_id(path: Option<&Path>) -> io::Result<String> {
    let mut cache = machine_id_cache()
        .lock()
        .map_err(|_| io::Error::other("host machine-id cache poisoned"))?;
    if let Some(machine_id) = cache.as_ref() {
        return Ok(machine_id.clone());
    }

    let owned_path;
    let path = match path {
        Some(path) => path,
        None => {
            owned_path = get_host_secrets_path();
            &owned_path
        }
    };

    if let Some(machine_id) = read_machine_id(path) {
        *cache = Some(machine_id.clone());
        return Ok(machine_id);
    }

    let machine_id = Uuid::new_v4().to_string();
    write_machine_id(path, &machine_id)?;
    *cache = Some(machine_id.clone());
    Ok(machine_id)
}

pub fn read_machine_id(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let object = parsed.as_object()?;
    let machine_id = object.get("machineId")?.as_str()?;
    (!machine_id.is_empty()).then(|| machine_id.to_string())
}

pub fn write_machine_id(path: &Path, machine_id: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = temp_path_for(path);
    let bytes = serde_json::to_vec_pretty(&json!({ "machineId": machine_id }))
        .map_err(|error| io::Error::other(format!("serialize host secrets: {error}")))?;
    fs::write(&temp_path, bytes)?;
    fs::rename(temp_path, path)
}

#[derive(Debug, Clone)]
pub struct HostNodeDeviceIdentity {
    device_id: String,
    public_key_raw: [u8; 32],
    signing_key: SigningKey,
}

impl HostNodeDeviceIdentity {
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn public_key_raw_base64_url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.public_key_raw)
    }

    pub fn sign_payload_base64_url(&self, payload: &str) -> String {
        URL_SAFE_NO_PAD.encode(self.signing_key.sign(payload.as_bytes()).to_bytes())
    }
}

fn node_identity_path(path: Option<&Path>) -> PathBuf {
    path.map(Path::to_path_buf)
        .unwrap_or_else(crate::host_paths::get_host_node_identity_path)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn identity_from_signing_key(signing_key: SigningKey) -> HostNodeDeviceIdentity {
    let public_key_raw = signing_key.verifying_key().to_bytes();
    HostNodeDeviceIdentity {
        device_id: sha256_hex(&public_key_raw),
        public_key_raw,
        signing_key,
    }
}

fn parse_node_identity(value: Value) -> io::Result<HostNodeDeviceIdentity> {
    let object = value
        .as_object()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "node identity must be an object"))?;
    if object.get("version").and_then(Value::as_u64) != Some(1) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "node identity version is unsupported",
        ));
    }
    let private_key = object
        .get("privateKeyRaw")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "node identity private key is missing"))?;
    let raw = URL_SAFE_NO_PAD.decode(private_key).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "node identity private key is invalid")
    })?;
    let bytes: [u8; 32] = raw.try_into().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "node identity private key length is invalid")
    })?;
    let signing_key = SigningKey::from_bytes(&bytes);
    let public_key_raw = signing_key.verifying_key().to_bytes();
    let stored_public_key = object
        .get("publicKeyRaw")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "node identity public key is missing"))?;
    if URL_SAFE_NO_PAD.decode(stored_public_key).ok().as_deref() != Some(public_key_raw.as_slice()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "node identity public key does not match its private key",
        ));
    }
    let device_id = sha256_hex(&public_key_raw);
    if object.get("deviceId").and_then(Value::as_str) != Some(device_id.as_str()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "node identity device id does not match its public key",
        ));
    }
    Ok(HostNodeDeviceIdentity {
        device_id,
        public_key_raw,
        signing_key,
    })
}

fn read_node_identity(path: &Path) -> io::Result<Option<HostNodeDeviceIdentity>> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let value = serde_json::from_str(&raw).map_err(|error| {
        io::Error::new(io::ErrorKind::InvalidData, format!("invalid node identity JSON: {error}"))
    })?;
    parse_node_identity(value).map(Some)
}

fn persist_node_identity(path: &Path, signing_key: &SigningKey) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let public_key_raw = signing_key.verifying_key().to_bytes();
    let device_id = sha256_hex(&public_key_raw);
    let value = json!({
        "version": 1,
        "deviceId": device_id,
        "publicKeyRaw": URL_SAFE_NO_PAD.encode(public_key_raw),
        "privateKeyRaw": URL_SAFE_NO_PAD.encode(signing_key.to_bytes()),
    });
    let temp_path = temp_path_for(path);
    fs::write(&temp_path, serde_json::to_vec(&value).map_err(|error| {
        io::Error::other(format!("serialize node identity: {error}"))
    })?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))?;
    }
    // Link the fully written temporary file into place without replacing a
    // concurrently created identity. A loser rereads the winner in the
    // caller, so two processes converge on one installation identity.
    match fs::hard_link(&temp_path, path) {
        Ok(()) => {
            let _ = fs::remove_file(&temp_path);
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(error)
        }
    }
}

/// Load or atomically create the installation-local NodeHost signing identity.
/// The private key is retained inside this owner and is never returned as bytes.
pub fn load_or_create_host_node_identity(
    path: Option<&Path>,
) -> io::Result<HostNodeDeviceIdentity> {
    let path = node_identity_path(path);
    if let Some(identity) = read_node_identity(&path)? {
        return Ok(identity);
    }
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed)
        .map_err(|error| io::Error::other(format!("generate node identity: {error}")))?;
    let signing_key = SigningKey::from_bytes(&seed);
    if persist_node_identity(&path, &signing_key).is_err() {
        if let Some(identity) = read_node_identity(&path)? {
            return Ok(identity);
        }
        return Err(io::Error::other("persist node identity failed"));
    }
    Ok(identity_from_signing_key(signing_key))
}

fn temp_path_for(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.{}.tmp", path.to_string_lossy(), std::process::id()))
}

pub fn clear_host_machine_id_cache_for_tests() {
    if let Ok(mut cache) = machine_id_cache().lock() {
        *cache = None;
    }
}
