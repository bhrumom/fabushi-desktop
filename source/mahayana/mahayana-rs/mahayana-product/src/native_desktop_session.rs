//! Read-only projection of the existing Electron Native credential owner.
//! No pipe consumption, credential persistence, refresh, or session cache.

use crate::ProductError;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const PRODUCTION_ISSUER: &str = "https://api.ombhrum.com";
pub const ISOLATED_ISSUER: &str = "https://fabushi-ci-isolated-auth.bhrumom.workers.dev";
const MAX_BYTES: u64 = 64 * 1024;
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
const EXPIRY_MARGIN_MS: i64 = 30_000;

pub fn enabled() -> bool {
    cfg!(unix) && std::env::var("FABUSHI_PRODUCT_MODE").ok().as_deref() == Some("1")
}

fn invalid(code: &'static str) -> ProductError {
    ProductError::Session(code.into())
}

pub fn normalize_origin(raw: &str) -> Result<String, ProductError> {
    let url = url::Url::parse(raw).map_err(|_| invalid("NATIVE_ISSUER_INVALID"))?;
    let origin = url.origin().ascii_serialization();
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || !matches!(origin.as_str(), PRODUCTION_ISSUER | ISOLATED_ISSUER)
    {
        return Err(invalid("NATIVE_ISSUER_INVALID"));
    }
    Ok(origin)
}

/// Default clients resolve Native configuration at use time, including clients
/// constructed before the Desktop environment was installed.
pub fn target_origin(explicit_client_base: Option<&str>) -> Result<String, ProductError> {
    let configured =
        std::env::var("FABUSHI_API_BASE_URL").unwrap_or_else(|_| PRODUCTION_ISSUER.into());
    let override_base = std::env::var("MAHAYANA_API_BASE_URL").ok();
    resolve_target(&configured, override_base.as_deref(), explicit_client_base)
}

fn resolve_target(
    configured: &str,
    override_base: Option<&str>,
    explicit_client_base: Option<&str>,
) -> Result<String, ProductError> {
    let origin = normalize_origin(configured)?;
    for candidate in [override_base, explicit_client_base].into_iter().flatten() {
        if normalize_origin(candidate)? != origin {
            return Err(invalid("NATIVE_TARGET_MISMATCH"));
        }
    }
    Ok(origin)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireCredential {
    version: u32,
    issuer: String,
    access_token: String,
    expires_at_ms: i64,
    session_id: String,
    device_id: String,
    user_id: Value,
}

/// Intentionally has no Serialize or derived Debug implementation.
pub struct NativeCredential {
    issuer: String,
    access_token: String,
    expires_at_ms: i64,
    session_id: String,
    device_id: String,
    raw_user_id: String,
}

impl std::fmt::Debug for NativeCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("NativeCredential(<private>)")
    }
}

impl NativeCredential {
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub(crate) fn token(&self) -> &str {
        &self.access_token
    }

    pub fn account_id(&self) -> String {
        if self.issuer == PRODUCTION_ISSUER {
            return self.raw_user_id.clone();
        }
        let digest = Sha256::digest(
            format!(
                "fabushi-native-auth-id-v1\0{}\0{}",
                self.issuer, self.raw_user_id
            )
            .as_bytes(),
        );
        format!("fabushi-ci-isolated:v1:{digest:x}")
    }

    fn binding(&self) -> String {
        let value = json!([
            self.issuer,
            self.access_token,
            self.expires_at_ms,
            self.session_id,
            self.device_id,
            self.raw_user_id,
        ]);
        format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
    }

    pub fn same_binding(&self, other: &Self) -> bool {
        self.binding() == other.binding()
    }

    pub fn ui_status(&self, restored: bool) -> Value {
        json!({
            "@type": if restored { "mahayana.auth.session" } else { "mahayana.auth.status" },
            "loggedIn": true,
            "provider": "native",
            "sessionStored": false,
            "user": { "id": self.account_id() },
        })
    }

    pub(crate) fn private_session(&self) -> Value {
        json!({
            "accessToken": self.access_token,
            "accessTokenExpiresAt": self.expires_at_ms,
            "issuer": self.issuer,
            "sessionId": self.session_id,
            "deviceId": self.device_id,
            "provider": "native",
            "user": { "id": self.account_id() },
            "_nativeRawUserId": self.raw_user_id,
            "_nativeBinding": self.binding(),
        })
    }

    pub fn matches_private_session(&self, session: &Value) -> bool {
        session.get("_nativeBinding").and_then(Value::as_str) == Some(self.binding().as_str())
    }

    pub fn validate_user_info(&self, value: &Value) -> Result<(), ProductError> {
        let data = value.get("data").unwrap_or(value);
        let user = data.get("user").unwrap_or(data);
        let id = [
            data.get("id"),
            data.get("userId"),
            data.get("user_id"),
            user.get("id"),
            user.get("userId"),
            user.get("user_id"),
        ]
        .into_iter()
        .flatten()
        .find_map(|value| raw_id(value).ok());
        if id.as_deref() != Some(self.raw_user_id.as_str()) {
            return Err(invalid("NATIVE_USER_INFO_IDENTITY_MISMATCH"));
        }
        Ok(())
    }

    pub fn guard_request(&self, target: &str, token: &str) -> Result<(), ProductError> {
        let url = url::Url::parse(target).map_err(|_| invalid("NATIVE_TARGET_INVALID"))?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.origin().ascii_serialization() != self.issuer
            || token != self.access_token
        {
            return Err(invalid("NATIVE_BEARER_BINDING_MISMATCH"));
        }
        Ok(())
    }
}

fn bounded(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn raw_id(value: &Value) -> Result<String, ProductError> {
    match value {
        Value::String(value) if bounded(value, 200) => Ok(value.clone()),
        Value::Number(value) => value
            .as_u64()
            .filter(|value| *value <= MAX_SAFE_INTEGER as u64)
            .map(|value| value.to_string())
            .ok_or_else(|| invalid("NATIVE_USER_ID_INVALID")),
        _ => Err(invalid("NATIVE_USER_ID_INVALID")),
    }
}

fn parse(
    bytes: &[u8],
    expected_issuer: &str,
    now_ms: i64,
) -> Result<NativeCredential, ProductError> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("NATIVE_CREDENTIAL_SIZE"));
    }
    let wire: WireCredential =
        serde_json::from_slice(bytes).map_err(|_| invalid("NATIVE_CREDENTIAL_SCHEMA"))?;
    let issuer = normalize_origin(&wire.issuer)?;
    if wire.version != 1
        || issuer != expected_issuer
        || !bounded(&wire.access_token, 16_384)
        || wire.access_token.len() < 32
        || !wire
            .access_token
            .bytes()
            .all(|byte| byte.is_ascii_graphic())
        || !bounded(&wire.session_id, 200)
        || !bounded(&wire.device_id, 200)
        || wire.expires_at_ms > MAX_SAFE_INTEGER
    {
        return Err(invalid("NATIVE_CREDENTIAL_INVALID"));
    }
    if wire.expires_at_ms <= now_ms.saturating_add(EXPIRY_MARGIN_MS) {
        return Err(ProductError::SessionExpired);
    }
    Ok(NativeCredential {
        issuer,
        access_token: wire.access_token,
        expires_at_ms: wire.expires_at_ms,
        session_id: wire.session_id,
        device_id: wire.device_id,
        raw_user_id: raw_id(&wire.user_id)?,
    })
}

pub fn read_current() -> Result<Option<NativeCredential>, ProductError> {
    let origin = target_origin(None)?;
    let Some(path) =
        std::env::var_os("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE").filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid("NATIVE_CLOCK_INVALID"))?
        .as_millis();
    let now_ms = i64::try_from(now_ms).map_err(|_| invalid("NATIVE_CLOCK_INVALID"))?;
    read_at(Path::new(&path), &origin, now_ms)
}

#[cfg(unix)]
fn validate_metadata(
    mode: u32,
    uid: u32,
    expected_uid: u32,
    size: i64,
) -> Result<(), ProductError> {
    if mode & 0o170000 != 0o100000
        || mode & 0o7777 != 0o600
        || uid != expected_uid
        || size <= 0
        || size as u64 > MAX_BYTES
    {
        return Err(invalid("NATIVE_CREDENTIAL_PRIVACY_OR_SIZE"));
    }
    Ok(())
}

#[cfg(unix)]
pub fn read_at(
    path: &Path,
    issuer: &str,
    now_ms: i64,
) -> Result<Option<NativeCredential>, ProductError> {
    use rustix::fs::{Mode, OFlags, fstat, open};
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let fd = match open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(_) => return Err(invalid("NATIVE_CREDENTIAL_OPEN")),
    };
    let stat = fstat(&fd).map_err(|_| invalid("NATIVE_CREDENTIAL_STAT"))?;
    validate_metadata(
        stat.st_mode as u32,
        stat.st_uid as u32,
        rustix::process::geteuid().as_raw(),
        stat.st_size as i64,
    )?;
    let file = std::fs::File::from(fd);
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid("NATIVE_CREDENTIAL_READ"))?;
    // Verify the final path still names the opened regular inode. Atomic Native
    // replacement/removal or symlink substitution fails closed for this call.
    let current =
        std::fs::symlink_metadata(path).map_err(|_| invalid("NATIVE_CREDENTIAL_CHANGED"))?;
    if current.file_type().is_symlink()
        || current.ino() != stat.st_ino as u64
        || current.dev() != stat.st_dev as u64
        || current.uid() != stat.st_uid as u32
        || current.mode() & 0o7777 != 0o600
        || current.len() != bytes.len() as u64
    {
        return Err(invalid("NATIVE_CREDENTIAL_CHANGED"));
    }
    parse(&bytes, issuer, now_ms).map(Some)
}

#[cfg(not(unix))]
pub fn read_at(
    _path: &Path,
    _issuer: &str,
    _now_ms: i64,
) -> Result<Option<NativeCredential>, ProductError> {
    Err(invalid("NATIVE_CREDENTIAL_ACL_UNSUPPORTED"))
}

pub fn reject_auth_mutation(path: &str, method: &str) -> Result<(), ProductError> {
    // The API resource category is canonical ASCII. Never decode aliases here:
    // encoded/case/matrix categories and repeated separators fail closed.
    let ambiguous_category = path.strip_prefix("/api/").is_some_and(|rest| {
        let category = rest.split('/').next().unwrap_or("");
        category.is_empty()
            || !category.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
    });
    if !path.starts_with('/')
        || path.contains("//")
        || path.contains(['\\', '?', '#'])
        || ambiguous_category
        || path.split('/').any(|part| matches!(part, "." | ".."))
        || ((path == "/api/auth" || path.starts_with("/api/auth/"))
            && !(method == "GET" && path == "/api/auth/user-info"))
    {
        return Err(invalid("NATIVE_AUTH_MUTATION_UNSUPPORTED"));
    }
    Ok(())
}

/// Server failures must not echo a private token, even under an unexpected key.
pub fn redact(value: &Value, token: Option<&str>) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter_map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    if lower.contains("token")
                        || lower.contains("secret")
                        || lower == "authorization"
                        || key.starts_with("_native")
                        || token.is_some_and(|token| !token.is_empty() && key.contains(token))
                    {
                        return None;
                    }
                    Some((key.clone(), redact(value, token)))
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(|item| redact(item, token)).collect()),
        Value::String(value) => Value::String(match token.filter(|token| !token.is_empty()) {
            Some(token) => value.replace(token, "[redacted]"),
            None => value.clone(),
        }),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const NOW: i64 = 1_800_000_000_000;
    const TOKEN: &str = "synthetic-native-access-token-for-contract-only";

    fn fixture(issuer: &str) -> Value {
        json!({"version":1, "issuer":issuer, "accessToken":TOKEN,
            "expiresAtMs":NOW + 86_400_000, "sessionId":"session-contract",
            "deviceId":"device-contract", "userId":42})
    }

    fn credential(issuer: &str) -> NativeCredential {
        parse(fixture(issuer).to_string().as_bytes(), issuer, NOW).expect("synthetic credential")
    }

    #[test]
    fn native_schema_is_not_ci_export_and_has_no_ci_ttl_cap() {
        assert!(
            parse(
                fixture(PRODUCTION_ISSUER).to_string().as_bytes(),
                PRODUCTION_ISSUER,
                NOW
            )
            .is_ok()
        );
        let mut value = fixture(PRODUCTION_ISSUER);
        value["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
        assert!(parse(value.to_string().as_bytes(), PRODUCTION_ISSUER, NOW).is_ok());
        for (key, bad) in [
            ("version", json!(2)),
            ("issuer", json!(ISOLATED_ISSUER)),
            ("accessToken", json!("weak")),
            ("accessToken", json!(" ".repeat(32))),
            ("sessionId", json!("")),
            ("deviceId", json!("")),
            ("userId", json!(-1)),
            ("userId", json!("")),
            ("expiresAtMs", json!(NOW + 30_000)),
            ("expiresAtMs", json!(MAX_SAFE_INTEGER + 1)),
            ("refreshToken", json!("forbidden")),
        ] {
            let mut value = fixture(PRODUCTION_ISSUER);
            value[key] = bad;
            assert!(
                parse(value.to_string().as_bytes(), PRODUCTION_ISSUER, NOW).is_err(),
                "field {key}"
            );
        }
        assert!(parse(b"{", PRODUCTION_ISSUER, NOW).is_err());
        assert!(parse(&vec![b' '; MAX_BYTES as usize + 1], PRODUCTION_ISSUER, NOW).is_err());
    }

    #[test]
    fn fixed_origins_and_explicit_overrides_are_bound() {
        for bad in [
            "http://api.ombhrum.com",
            "https://api.ombhrum.com.evil.invalid",
            "https://api.ombhrum.com/path",
            "https://user@api.ombhrum.com",
            "https://api.ombhrum.com?query=1",
            "https://api.ombhrum.com#fragment",
        ] {
            assert!(normalize_origin(bad).is_err());
        }
        assert_eq!(
            normalize_origin("https://api.ombhrum.com:443/").unwrap(),
            PRODUCTION_ISSUER
        );
        assert!(resolve_target(ISOLATED_ISSUER, Some(PRODUCTION_ISSUER), None).is_err());
        assert!(resolve_target(ISOLATED_ISSUER, None, Some(PRODUCTION_ISSUER)).is_err());
        assert_eq!(
            resolve_target(ISOLATED_ISSUER, None, None).unwrap(),
            ISOLATED_ISSUER
        );
    }

    #[test]
    fn account_projection_preserves_production_and_isolates_equal_raw_ids() {
        let production = credential(PRODUCTION_ISSUER);
        let isolated = credential(ISOLATED_ISSUER);
        assert_eq!(production.account_id(), "42");
        let expected = format!(
            "fabushi-ci-isolated:v1:{:x}",
            Sha256::digest(
                format!("fabushi-native-auth-id-v1\0{ISOLATED_ISSUER}\0{}", "42").as_bytes()
            )
        );
        assert_eq!(isolated.account_id(), expected);
        assert_ne!(production.account_id(), isolated.account_id());
        let projection = isolated.ui_status(true);
        assert_eq!(projection["user"], json!({"id": expected}));
        assert_eq!(projection["sessionStored"], false);
        let text = projection.to_string();
        for private in [
            TOKEN,
            "session-contract",
            "device-contract",
            "_native",
            "RawUserId",
        ] {
            assert!(!text.contains(private));
        }
        assert!(
            isolated
                .validate_user_info(&json!({"data":{"user":{"id":42}}}))
                .is_ok()
        );
        assert!(
            isolated
                .validate_user_info(&json!({"id":isolated.account_id()}))
                .is_err()
        );
        assert!(isolated.validate_user_info(&json!({"id":43})).is_err());
        let mut private = isolated.private_session().as_object().unwrap().clone();
        crate::strip_credentials(&mut private);
        assert!(!private.contains_key("_nativeRawUserId"));
        assert!(!private.contains_key("_nativeBinding"));
        assert!(!private.contains_key("accessToken"));
        assert!(!private.contains_key("sessionId"));
        assert!(!private.contains_key("deviceId"));
    }

    #[test]
    fn bearer_binding_mutation_and_response_privacy_contracts() {
        let credential = credential(ISOLATED_ISSUER);
        assert!(
            credential
                .guard_request(&format!("{ISOLATED_ISSUER}/api/auth/user-info"), TOKEN)
                .is_ok()
        );
        assert!(
            credential
                .guard_request(&format!("{PRODUCTION_ISSUER}/v1/messages"), TOKEN)
                .is_err()
        );
        assert!(
            credential
                .guard_request(&format!("{ISOLATED_ISSUER}/v1/messages"), "stale")
                .is_err()
        );
        for path in [
            "/api/auth/login",
            "/api/auth/logout",
            "/api/auth/refresh",
            "/api/%61uth/logout",
            "/api/auth/../auth/login",
            "//api/auth/login",
            "/api//auth/logout",
            "/api/Auth/logout",
            "/api/auth;alias/logout",
            "/api/%2561uth/logout",
        ] {
            assert!(reject_auth_mutation(path, "POST").is_err());
        }
        assert!(reject_auth_mutation("/api/auth/user-info", "GET").is_ok());
        assert!(reject_auth_mutation("/api/auth/user-info", "POST").is_err());
        assert!(reject_auth_mutation("/api/agents/opaque%20id", "GET").is_ok());
        let value = redact(
            &json!({"accessToken":TOKEN,"nested":[{"echo":TOKEN}],
            "_nativeRawUserId":"42", (TOKEN):"key-echo"}),
            Some(TOKEN),
        );
        assert!(!value.to_string().contains(TOKEN));
        assert!(!value.to_string().contains("_native"));
    }

    #[cfg(not(unix))]
    #[test]
    fn non_unix_never_selects_posix_adapter() {
        assert!(!enabled());
    }

    #[cfg(unix)]
    fn private_file(path: &Path, value: &Value) {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .expect("create synthetic private fixture");
        file.write_all(value.to_string().as_bytes()).unwrap();
    }

    #[cfg(unix)]
    fn temporary_root() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "native-product-contract-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        root
    }

    #[cfg(unix)]
    #[test]
    fn private_file_metadata_symlinks_logout_and_replacement() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let root = temporary_root();
        let path = root.join("native");
        assert!(read_at(&path, PRODUCTION_ISSUER, NOW).unwrap().is_none());
        private_file(&path, &fixture(PRODUCTION_ISSUER));
        let first = read_at(&path, PRODUCTION_ISSUER, NOW).unwrap().unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_at(&path, PRODUCTION_ISSUER, NOW).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&path, root.join("alias")).unwrap();
        assert!(read_at(&root.join("alias"), PRODUCTION_ISSUER, NOW).is_err());
        assert!(read_at(&root, PRODUCTION_ISSUER, NOW).is_err());
        let fifo = root.join("invalid-native-fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        assert!(read_at(&fifo, PRODUCTION_ISSUER, NOW).is_err());
        let mut next = fixture(PRODUCTION_ISSUER);
        next["userId"] = json!(43);
        private_file(&root.join("next"), &next);
        std::fs::rename(root.join("next"), &path).unwrap();
        let second = read_at(&path, PRODUCTION_ISSUER, NOW).unwrap().unwrap();
        assert!(!first.same_binding(&second));
        assert_eq!(second.account_id(), "43");
        std::fs::remove_file(&path).unwrap();
        assert!(read_at(&path, PRODUCTION_ISSUER, NOW).unwrap().is_none());
        assert!(validate_metadata(0o100600, 2, 1, 50).is_err());
        assert!(validate_metadata(0o010600, 1, 1, 50).is_err());
        assert!(validate_metadata(0o100600, 1, 1, MAX_BYTES as i64 + 1).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A subprocess owns its environment, so parallel library tests need no
    /// global env mutation or unsafe Rust. No server/real account is contacted.
    #[cfg(unix)]
    #[test]
    fn native_client_ignores_ci_fifo_and_reloads_owner_file() {
        let root = temporary_root();
        let path = root.join("native");
        let mut value = fixture(ISOLATED_ISSUER);
        value["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
        private_file(&path, &value);
        let fifo = root.join("native-only-fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "native_desktop_session::tests::native_client_child",
                "--nocapture",
            ])
            .env("NATIVE_PRODUCT_CONTRACT_CHILD", "1")
            .env("FABUSHI_PRODUCT_MODE", "1")
            .env("FABUSHI_API_BASE_URL", ISOLATED_ISSUER)
            .env_remove("MAHAYANA_API_BASE_URL")
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &path)
            .env("FABUSHI_CI_ACCOUNT_SESSION_FILE", &fifo)
            .env("GITHUB_ACTIONS", "true")
            .env("NATIVE_PRODUCT_CONTRACT_ROOT", &root)
            .spawn()
            .unwrap();
        // Bound a regression that tries to open the FIFO rather than hanging CI.
        let mut child = child;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                let _ = child.wait();
                panic!("Native product attempted blocking bootstrap");
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn native_client_child() {
        if std::env::var("NATIVE_PRODUCT_CONTRACT_CHILD")
            .ok()
            .as_deref()
            != Some("1")
        {
            return;
        }
        let root =
            std::path::PathBuf::from(std::env::var_os("NATIVE_PRODUCT_CONTRACT_ROOT").unwrap());
        let client = crate::MahayanaProductClient::new_with_default_api_base_url(
            root.join("session.json"),
            root.join("surface.json"),
        );
        assert!(enabled());
        assert!(client.bootstrap_ci_test_account_session().unwrap());
        assert_eq!(client.effective_api_base_url().unwrap(), ISOLATED_ISSUER);
        assert_eq!(
            client.restore_session().unwrap()["user"]["id"],
            credential(ISOLATED_ISSUER).account_id()
        );
        assert!(client.session_token().is_err());
        assert!(client.auth_status(&json!({"accessToken":"stale"})).is_err());
        assert!(
            client
                .get_json("/api/auth/user-info", &[], Some("stale"))
                .is_err()
        );
        assert!(
            client
                .post_json("/api/auth/login", json!({}), Some(TOKEN))
                .is_err()
        );
        assert!(
            client
                .platform_request(&json!({"method":"POST", "path":"/api/auth/logout"}))
                .is_err()
        );
        for method in [
            "mahayana.auth.logout",
            "mahayana.auth.login",
            "mahayana.auth.refresh",
        ] {
            assert!(client.execute(method, &json!({})).is_err());
        }
        let explicit =
            crate::MahayanaProductClient::new(PRODUCTION_ISSUER, root.join("explicit.json"));
        assert!(
            explicit
                .get_json("/api/auth/user-info", &[], Some(TOKEN))
                .is_err()
        );
        let old_session = client.load_session().unwrap().unwrap();
        let path = root.join("native");
        let mut next = fixture(ISOLATED_ISSUER);
        next["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
        next["userId"] = json!(43);
        next["accessToken"] = json!("synthetic-replacement-access-token-contract-only");
        private_file(&root.join("next"), &next);
        std::fs::rename(root.join("next"), &path).unwrap();
        assert!(client.active_session_token(old_session).is_err());
        assert!(client.get_json("/v1/messages", &[], Some(TOKEN)).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(!client.bootstrap_ci_test_account_session().unwrap());
        assert_eq!(client.auth_status(&json!({})).unwrap()["loggedIn"], false);
        assert!(client.load_session().unwrap().is_none());
        assert!(!root.join("session.json").exists());
    }
}
