//! Read-only projection of the existing Electron Native credential owner.
//! No pipe consumption, credential persistence, refresh, or session cache.

use crate::ProductError;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
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

#[cfg(any(debug_assertions, test))]
fn development_loopback_flags(
    github_actions: Option<&str>,
    fabushi_e2e: Option<&str>,
    feature_host_mode: Option<&str>,
) -> bool {
    github_actions == Some("true") && fabushi_e2e == Some("1") && feature_host_mode == Some("test")
}

#[cfg(debug_assertions)]
fn development_loopback_enabled() -> bool {
    development_loopback_flags(
        std::env::var("GITHUB_ACTIONS").ok().as_deref(),
        std::env::var("FABUSHI_E2E").ok().as_deref(),
        std::env::var("FABUSHI_FEATURE_HOST_MODE").ok().as_deref(),
    )
}

#[cfg(not(debug_assertions))]
fn development_loopback_enabled() -> bool {
    false
}

fn invalid(code: &'static str) -> ProductError {
    ProductError::Session(code.into())
}

pub fn normalize_origin(raw: &str) -> Result<String, ProductError> {
    normalize_origin_with_loopback(raw, development_loopback_enabled())
}

fn normalize_origin_with_loopback(raw: &str, allow_loopback: bool) -> Result<String, ProductError> {
    let url = url::Url::parse(raw).map_err(|_| invalid("NATIVE_ISSUER_INVALID"))?;
    let origin = url.origin().ascii_serialization();
    if allow_loopback && is_development_loopback_origin(raw) {
        return Ok(origin);
    }
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
    resolve_target_with_loopback(
        configured,
        override_base,
        explicit_client_base,
        development_loopback_enabled(),
    )
}

fn resolve_target_with_loopback(
    configured: &str,
    override_base: Option<&str>,
    explicit_client_base: Option<&str>,
    allow_loopback: bool,
) -> Result<String, ProductError> {
    let origin = normalize_origin_with_loopback(configured, allow_loopback)?;
    for candidate in [override_base, explicit_client_base].into_iter().flatten() {
        if normalize_origin_with_loopback(candidate, allow_loopback)? != origin {
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
    #[serde(default)]
    telegram_storage_key: Option<String>,
    #[serde(default)]
    telegram_api_id: Option<i32>,
    #[serde(default)]
    telegram_api_hash: Option<String>,
}

/// Intentionally has no Serialize or derived Debug implementation.
pub struct NativeCredential {
    issuer: String,
    access_token: String,
    expires_at_ms: i64,
    session_id: String,
    device_id: String,
    raw_user_id: String,
    telegram_storage_key: Option<[u8; 32]>,
    telegram_api_id: Option<i32>,
    telegram_api_hash: Option<String>,
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
        if self.issuer == PRODUCTION_ISSUER || is_development_loopback_origin(&self.issuer) {
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

    pub fn telegram_storage_key(&self) -> Option<[u8; 32]> {
        self.telegram_storage_key
    }

    pub fn telegram_application_credentials(&self) -> Option<(i32, &str)> {
        self.telegram_api_id.zip(self.telegram_api_hash.as_deref())
    }

    fn binding(&self) -> String {
        let value = json!([
            self.issuer,
            self.access_token,
            self.expires_at_ms,
            self.session_id,
            self.device_id,
            self.raw_user_id,
            self.telegram_storage_key,
            self.telegram_api_id,
            self.telegram_api_hash.as_deref(),
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

fn is_development_loopback_origin(origin: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    let canonical = url.origin().ascii_serialization();
    let raw_is_canonical = origin == canonical || origin == format!("{canonical}/");
    url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1")
        && url.port().is_some_and(|port| port != 0)
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none()
        && raw_is_canonical
}

#[cfg(test)]
fn development_loopback_policy(
    raw: &str,
    debug_assertions: bool,
    github_actions: Option<&str>,
    fabushi_e2e: Option<&str>,
    feature_host_mode: Option<&str>,
) -> bool {
    debug_assertions
        && development_loopback_flags(github_actions, fabushi_e2e, feature_host_mode)
        && is_development_loopback_origin(raw)
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
    let telegram_storage_key = wire
        .telegram_storage_key
        .as_deref()
        .map(|encoded| {
            let decoded = URL_SAFE_NO_PAD
                .decode(encoded.as_bytes())
                .map_err(|_| invalid("NATIVE_TELEGRAM_STORAGE_KEY_INVALID"))?;
            decoded
                .try_into()
                .map_err(|_| invalid("NATIVE_TELEGRAM_STORAGE_KEY_INVALID"))
        })
        .transpose()?;
    let (telegram_api_id, telegram_api_hash) =
        match (wire.telegram_api_id, wire.telegram_api_hash.as_deref()) {
            (None, None) => (None, None),
            (Some(api_id), Some(api_hash))
                if api_id > 0
                    && api_hash.len() == 32
                    && api_hash.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
            {
                (Some(api_id), Some(api_hash.to_ascii_lowercase()))
            }
            _ => return Err(invalid("NATIVE_TELEGRAM_APPLICATION_CREDENTIAL_INVALID")),
        };
    Ok(NativeCredential {
        issuer,
        access_token: wire.access_token,
        expires_at_ms: wire.expires_at_ms,
        session_id: wire.session_id,
        device_id: wire.device_id,
        raw_user_id: raw_id(&wire.user_id)?,
        telegram_storage_key,
        telegram_api_id,
        telegram_api_hash,
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
        let mut valid_telegram_key = fixture(PRODUCTION_ISSUER);
        valid_telegram_key["telegramStorageKey"] = json!(URL_SAFE_NO_PAD.encode([7_u8; 32]));
        assert_eq!(
            parse(
                valid_telegram_key.to_string().as_bytes(),
                PRODUCTION_ISSUER,
                NOW
            )
            .unwrap()
            .telegram_storage_key(),
            Some([7_u8; 32])
        );
        let mut invalid_telegram_key = fixture(PRODUCTION_ISSUER);
        invalid_telegram_key["telegramStorageKey"] = json!("not-a-32-byte-storage-key");
        assert!(
            parse(
                invalid_telegram_key.to_string().as_bytes(),
                PRODUCTION_ISSUER,
                NOW
            )
            .is_err()
        );

        let mut telegram_application = fixture(PRODUCTION_ISSUER);
        telegram_application["telegramApiId"] = json!(123456);
        telegram_application["telegramApiHash"] = json!("ABCDEF0123456789ABCDEF0123456789");
        let telegram_application = parse(
            telegram_application.to_string().as_bytes(),
            PRODUCTION_ISSUER,
            NOW,
        )
        .expect("bounded Telegram application credential");
        assert_eq!(
            telegram_application.telegram_application_credentials(),
            Some((123456, "abcdef0123456789abcdef0123456789"))
        );
        for (api_id, api_hash) in [
            (json!(0), json!("abcdef0123456789abcdef0123456789")),
            (json!(123456), json!("short")),
            (json!(123456), json!(null)),
        ] {
            let mut invalid = fixture(PRODUCTION_ISSUER);
            invalid["telegramApiId"] = api_id;
            invalid["telegramApiHash"] = api_hash;
            assert!(parse(invalid.to_string().as_bytes(), PRODUCTION_ISSUER, NOW).is_err());
        }
        let mut missing_id = fixture(PRODUCTION_ISSUER);
        missing_id["telegramApiHash"] = json!("abcdef0123456789abcdef0123456789");
        assert!(parse(missing_id.to_string().as_bytes(), PRODUCTION_ISSUER, NOW).is_err());

        assert!(parse(b"{", PRODUCTION_ISSUER, NOW).is_err());
        assert!(parse(&vec![b' '; MAX_BYTES as usize + 1], PRODUCTION_ISSUER, NOW).is_err());
    }

    #[test]
    fn fixed_origins_and_explicit_overrides_are_bound() {
        const LOOPBACK: &str = "http://127.0.0.1:49226/";

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

        assert!(development_loopback_policy(
            LOOPBACK,
            true,
            Some("true"),
            Some("1"),
            Some("test")
        ));
        assert!(!development_loopback_policy(
            LOOPBACK,
            false,
            Some("true"),
            Some("1"),
            Some("test")
        ));
        for flags in [
            (None, Some("1"), Some("test")),
            (Some("false"), Some("1"), Some("test")),
            (Some("TRUE"), Some("1"), Some("test")),
            (Some("true"), None, Some("test")),
            (Some("true"), Some("0"), Some("test")),
            (Some("true"), Some("1"), None),
            (Some("true"), Some("1"), Some("Test")),
        ] {
            assert!(!development_loopback_flags(flags.0, flags.1, flags.2));
        }
        assert!(development_loopback_flags(
            Some("true"),
            Some("1"),
            Some("test")
        ));
        for bad in [
            "http://127.0.0.1/",
            "http://127.0.0.1:0/",
            "http://127.0.0.1:49226/path",
            "https://127.0.0.1:49226/",
            "http://localhost:49226/",
            "http://127.0.0.2:49226/",
            "http://127.1:49226/",
            "http://2130706433:49226/",
            "http://user@127.0.0.1:49226/",
            "http://127.0.0.1:49226/?query=1",
            "http://127.0.0.1:49226/#fragment",
        ] {
            assert!(!development_loopback_policy(
                bad,
                true,
                Some("true"),
                Some("1"),
                Some("test")
            ));
            assert!(normalize_origin_with_loopback(bad, true).is_err());
        }
        assert_eq!(
            normalize_origin_with_loopback(LOOPBACK, true).unwrap(),
            LOOPBACK.trim_end_matches('/')
        );
        assert!(
            resolve_target_with_loopback(LOOPBACK, Some(PRODUCTION_ISSUER), None, true).is_err()
        );
        assert!(
            resolve_target_with_loopback(PRODUCTION_ISSUER, Some(LOOPBACK), None, true).is_err()
        );
        assert!(
            resolve_target_with_loopback(LOOPBACK, None, Some(PRODUCTION_ISSUER), true).is_err()
        );
        assert!(
            resolve_target_with_loopback(PRODUCTION_ISSUER, None, Some(LOOPBACK), true).is_err()
        );
        assert_eq!(
            resolve_target_with_loopback(LOOPBACK, Some(LOOPBACK), None, true).unwrap(),
            LOOPBACK.trim_end_matches('/')
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

    #[cfg(all(unix, debug_assertions))]
    #[test]
    fn native_development_client_uses_raw_id_and_rereads_owner_file() {
        const LOOPBACK: &str = "http://127.0.0.1:49226";
        let root = temporary_root();
        let path = root.join("native");
        let mut value = fixture(LOOPBACK);
        value["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
        private_file(&path, &value);
        let fifo = root.join("native-development-fifo");
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
                "native_desktop_session::tests::native_development_client_child",
                "--nocapture",
            ])
            .env("NATIVE_PRODUCT_CONTRACT_CHILD", "1")
            .env("NATIVE_DEVELOPMENT_CONTRACT_CHILD", "1")
            .env("FABUSHI_PRODUCT_MODE", "1")
            .env("FABUSHI_API_BASE_URL", LOOPBACK)
            .env("FABUSHI_E2E", "1")
            .env("FABUSHI_FEATURE_HOST_MODE", "test")
            .env_remove("MAHAYANA_API_BASE_URL")
            .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &path)
            .env("FABUSHI_CI_ACCOUNT_SESSION_FILE", &fifo)
            .env("GITHUB_ACTIONS", "true")
            .env("NATIVE_PRODUCT_CONTRACT_ROOT", &root)
            .spawn()
            .unwrap();
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
                panic!("Native development client attempted blocking bootstrap");
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Keep process-wide Native configuration inside an exact child test. The
    /// parent owns a real HTTP server, private credential changes and all waits.
    #[cfg(all(unix, debug_assertions))]
    mod native_http_completion_contracts {
        use super::*;
        use std::io::{Read, Write};
        use std::net::{TcpListener, TcpStream};
        use std::process::{Child, Command, ExitStatus};
        use std::time::{Duration, Instant};

        const CHILD_TEST: &str = "native_desktop_session::tests::native_http_completion_contracts::native_http_completion_child";
        const MARKER: &str = "bound-response-marker";
        const OBSOLETE_DATA: &str = "obsolete-account-payload";

        #[derive(Clone, Copy, Debug)]
        enum BodyKind {
            Json,
            InvalidJson,
            InvalidUtf8,
            Truncated,
            Oversized,
        }

        impl BodyKind {
            fn name(self) -> &'static str {
                match self {
                    Self::Json => "json",
                    Self::InvalidJson => "invalid-json",
                    Self::InvalidUtf8 => "invalid-utf8",
                    Self::Truncated => "truncated",
                    Self::Oversized => "oversized",
                }
            }

            fn bytes(self) -> Vec<u8> {
                match self {
                    Self::Json | Self::Truncated => json!({
                        "marker": MARKER,
                        "ownerData": OBSOLETE_DATA,
                        "accessToken": TOKEN,
                        "data": {"user": {"id": 42}},
                    })
                    .to_string()
                    .into_bytes(),
                    Self::InvalidJson => b"{not-valid-json".to_vec(),
                    Self::InvalidUtf8 => vec![0xff, 0xfe, b'x'],
                    Self::Oversized => vec![b'x'; 1024 * 1024 + 1],
                }
            }
        }

        struct CompletionChild(Child);

        impl CompletionChild {
            fn wait_bounded(&mut self) -> ExitStatus {
                let deadline = Instant::now() + Duration::from_secs(20);
                loop {
                    if let Some(status) = self.0.try_wait().expect("poll completion child") {
                        return status;
                    }
                    assert!(
                        Instant::now() < deadline,
                        "Native completion child exceeded its deadline"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        impl Drop for CompletionChild {
            fn drop(&mut self) {
                if self.0.try_wait().ok().flatten().is_none() {
                    let _ = self.0.kill();
                }
                let _ = self.0.wait();
            }
        }

        fn request_bytes(stream: &mut TcpStream) -> Vec<u8> {
            stream
                .set_nonblocking(false)
                .expect("blocking accepted Native stream");
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .expect("request read deadline");
            stream
                .set_write_timeout(Some(Duration::from_secs(10)))
                .expect("response write deadline");
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            let header_end = loop {
                let read = stream.read(&mut buffer).expect("read Native request");
                assert_ne!(read, 0, "Native request ended before its headers");
                bytes.extend_from_slice(&buffer[..read]);
                assert!(bytes.len() <= 256 * 1024, "bounded request fixture");
                if let Some(offset) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                    break offset + 4;
                }
            };
            let headers = std::str::from_utf8(&bytes[..header_end]).expect("HTTP request headers");
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("request body length"))
                })
                .unwrap_or(0);
            assert!(
                !headers.to_ascii_lowercase().contains("transfer-encoding:"),
                "this bounded multipart fixture must declare its complete length"
            );
            assert!(content_length <= 128 * 1024, "bounded request body fixture");
            while bytes.len() < header_end + content_length {
                let read = stream.read(&mut buffer).expect("read Native request body");
                assert_ne!(read, 0, "Native request body was truncated");
                bytes.extend_from_slice(&buffer[..read]);
            }
            assert_eq!(
                bytes.len(),
                header_end + content_length,
                "one complete request"
            );
            bytes
        }

        fn replace_binding(root: &Path, issuer: &str, change: &str) {
            let path = root.join("native");
            if matches!(change, "logout" | "anonymous-logout") {
                std::fs::remove_file(path).expect("remove Native credential");
                return;
            }
            if change == "unchanged" {
                return;
            }
            let mut next = fixture(issuer);
            next["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
            match change {
                "account" => next["userId"] = json!(43),
                "token" => {
                    next["accessToken"] = json!("synthetic-rotated-native-token-for-contract")
                }
                "device" => next["deviceId"] = json!("rotated-device"),
                "session" => next["sessionId"] = json!("rotated-session"),
                "issuer" => next["issuer"] = json!(ISOLATED_ISSUER),
                "expiry" => next["expiresAtMs"] = json!(MAX_SAFE_INTEGER - 1),
                "telegram-key" => {
                    next["telegramStorageKey"] = json!(URL_SAFE_NO_PAD.encode([8_u8; 32]))
                }
                "telegram-application" => {
                    next["telegramApiId"] = json!(123456);
                    next["telegramApiHash"] = json!("abcdef0123456789abcdef0123456789");
                }
                "invalid" | "anonymous-invalid" => next = json!({"version":999}),
                "anonymous-login" => next["userId"] = json!(43),
                _ => panic!("unexpected credential mutation: {change}"),
            }
            private_file(&root.join("replacement"), &next);
            std::fs::rename(root.join("replacement"), path)
                .expect("atomically replace Native owner file");
        }

        fn run_contract(entry: &str, change: &str, status: u16, kind: BodyKind) {
            let root = temporary_root();
            let listener = TcpListener::bind("127.0.0.1:0").expect("Native completion listener");
            listener
                .set_nonblocking(true)
                .expect("nonblocking Native listener");
            let origin = format!("http://{}", listener.local_addr().unwrap());
            let path = root.join("native");
            if change != "anonymous-login" {
                let mut value = fixture(&origin);
                value["expiresAtMs"] = json!(MAX_SAFE_INTEGER);
                if change == "anonymous-invalid" {
                    value = json!({"version":999});
                }
                private_file(&path, &value);
            }
            let mut child = CompletionChild(
                Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", CHILD_TEST, "--nocapture"])
                    .env("NATIVE_HTTP_COMPLETION_CHILD", "1")
                    .env("NATIVE_HTTP_COMPLETION_ROOT", &root)
                    .env("NATIVE_HTTP_COMPLETION_ENTRY", entry)
                    .env("NATIVE_HTTP_COMPLETION_CHANGE", change)
                    .env("NATIVE_HTTP_COMPLETION_STATUS", status.to_string())
                    .env("NATIVE_HTTP_COMPLETION_KIND", kind.name())
                    .env("FABUSHI_PRODUCT_MODE", "1")
                    .env("FABUSHI_API_BASE_URL", &origin)
                    .env("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE", &path)
                    .env("GITHUB_ACTIONS", "true")
                    .env("FABUSHI_E2E", "1")
                    .env("FABUSHI_FEATURE_HOST_MODE", "test")
                    .env_remove("MAHAYANA_API_BASE_URL")
                    .env_remove("FABUSHI_CI_ACCOUNT_SESSION_FILE")
                    .env_remove("MAHAYANA_TEST_ACCOUNT_TOKEN")
                    .spawn()
                    .expect("spawn isolated Native completion child"),
            );

            let deadline = Instant::now() + Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            child.0.try_wait().expect("poll before request").is_none(),
                            "child exited before making the Native request"
                        );
                        assert!(Instant::now() < deadline, "Native request did not arrive");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept Native request: {error}"),
                }
            };
            let request = request_bytes(&mut stream);
            let header_end = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .unwrap()
                + 4;
            let headers = std::str::from_utf8(&request[..header_end])
                .unwrap()
                .to_ascii_lowercase();
            let anonymous = entry.ends_with("-anonymous");
            if anonymous {
                assert!(
                    !headers.contains("authorization:"),
                    "anonymous request acquired a bearer"
                );
            } else {
                assert!(
                    headers.contains(&format!("authorization: bearer {TOKEN}")),
                    "dispatch must use the original credential"
                );
            }
            let expected_start = match entry {
                "platform-get" | "platform-anonymous" => "GET /api/social/friends",
                "platform-post" => "POST /api/social/friends",
                "platform-put" => "PUT /api/social/friends",
                "platform-patch" => "PATCH /api/social/friends",
                "platform-delete" => "DELETE /api/social/friends",
                "get" | "get-anonymous" => "GET /v1/wallet/balance",
                "post" | "post-anonymous" => "POST /v1/purchases/restore",
                "auth-status" => "GET /api/auth/user-info",
                "multipart" => "POST /v1/marketplace/releases",
                "body-barrier" => "GET /v1/wallet/balance",
                _ => panic!("unexpected Native entry: {entry}"),
            };
            assert!(
                std::str::from_utf8(&request)
                    .unwrap()
                    .starts_with(expected_start),
                "actual Native route: {entry}"
            );
            if entry == "multipart" {
                assert!(headers.contains("content-type: multipart/form-data;"));
                assert!(
                    std::str::from_utf8(&request[header_end..])
                        .unwrap()
                        .contains("completion-package-bytes"),
                    "actual multipart package bytes"
                );
            }
            let body = kind.bytes();
            let declared_length = body.len()
                + if matches!(kind, BodyKind::Truncated) {
                    10
                } else {
                    0
                };
            write!(stream,
                "HTTP/1.1 {status} Native fixture\r\nContent-Type: application/json\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n"
            ).expect("response headers");
            stream.write_all(&body[..1]).expect("partial response body");
            stream
                .flush()
                .expect("flush headers before credential replacement");

            if entry == "body-barrier" {
                // The child obtained a real reqwest Response and enters the
                // shared completion callback before releasing this barrier.
                // Checking only before that callback would see the old binding.
                let deadline = Instant::now() + Duration::from_secs(10);
                while !root.join("release-body").exists() {
                    assert!(
                        child.0.try_wait().expect("poll body barrier").is_none(),
                        "child returned before releasing the response body"
                    );
                    assert!(
                        Instant::now() < deadline,
                        "body completion callback did not release the barrier"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            replace_binding(&root, &origin, change);
            stream
                .write_all(&body[1..])
                .expect("finish response after owner change");
            stream.flush().expect("complete Native response");
            drop(stream);
            let exit = child.wait_bounded();
            assert!(
                exit.success(),
                "completion contract failed: {entry}/{change}/{status}/{}",
                kind.name()
            );

            // Keep the listener alive until the request has completed. A retry
            // cannot be hidden by a refused connection or a closed fixture.
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "Native completion must not replay a dispatched request"
            );
            assert!(
                !root.join("session.json").exists(),
                "completion must not persist a second identity owner"
            );
            println!(
                "NATIVE_RESPONSE_COMPLETION_CASE {}",
                json!({
                    "entry": entry, "change": change, "status": status, "body": kind.name(), "requests": 1,
                })
            );
            drop(child);
            drop(listener);
            std::fs::remove_dir_all(root).expect("remove completion fixture");
        }

        #[test]
        fn native_response_completion_platform_bindings() {
            for (entry, change) in [
                ("platform-get", "account"),
                ("platform-post", "token"),
                ("platform-put", "device"),
                ("platform-patch", "session"),
                ("platform-delete", "issuer"),
                ("platform-get", "logout"),
                ("platform-post", "invalid"),
                ("platform-get", "expiry"),
            ] {
                run_contract(entry, change, 200, BodyKind::Json);
            }
        }

        #[test]
        fn native_response_completion_json_bindings() {
            for entry in ["get", "post"] {
                for change in [
                    "account",
                    "logout",
                    "token",
                    "device",
                    "session",
                    "issuer",
                    "expiry",
                    "invalid",
                    "telegram-key",
                    "telegram-application",
                ] {
                    run_contract(entry, change, 200, BodyKind::Json);
                }
            }
        }

        #[test]
        fn native_response_completion_multipart_bindings() {
            for change in [
                "account", "logout", "token", "device", "session", "issuer", "invalid",
            ] {
                run_contract("multipart", change, 200, BodyKind::Json);
            }
        }

        #[test]
        fn native_response_completion_unchanged_and_anonymous() {
            for entry in [
                "platform-get",
                "platform-post",
                "platform-put",
                "platform-patch",
                "platform-delete",
                "get",
                "post",
                "multipart",
                "body-barrier",
            ] {
                run_contract(entry, "unchanged", 200, BodyKind::Json);
            }
            for entry in ["platform-anonymous", "get-anonymous", "post-anonymous"] {
                for change in ["anonymous-login", "anonymous-invalid", "anonymous-logout"] {
                    run_contract(entry, change, 200, BodyKind::Json);
                }
            }
        }

        #[test]
        fn native_response_completion_body_boundary_and_errors() {
            for (change, status, kind) in [
                ("logout", 200, BodyKind::Json),
                ("account", 403, BodyKind::Json),
                ("token", 200, BodyKind::InvalidJson),
                ("session", 200, BodyKind::InvalidUtf8),
                ("device", 200, BodyKind::Truncated),
                ("expiry", 200, BodyKind::Oversized),
                ("unchanged", 403, BodyKind::Json),
                ("unchanged", 200, BodyKind::InvalidJson),
                ("unchanged", 200, BodyKind::InvalidUtf8),
                ("unchanged", 200, BodyKind::Truncated),
                ("unchanged", 200, BodyKind::Oversized),
            ] {
                run_contract("body-barrier", change, status, kind);
            }
            for (entry, change, status) in [
                ("platform-get", "account", 403),
                ("get", "logout", 403),
                ("post", "token", 403),
                ("multipart", "session", 403),
                ("auth-status", "account", 401),
            ] {
                run_contract(entry, change, status, BodyKind::Json);
            }
        }

        // Deliberately excluded from the native_response_completion_ filter:
        // every selected parent test must really spawn and observe this child.
        #[test]
        fn native_http_completion_child() {
            if std::env::var("NATIVE_HTTP_COMPLETION_CHILD")
                .ok()
                .as_deref()
                != Some("1")
            {
                return;
            }
            let root = std::path::PathBuf::from(
                std::env::var_os("NATIVE_HTTP_COMPLETION_ROOT").expect("completion root"),
            );
            let entry = std::env::var("NATIVE_HTTP_COMPLETION_ENTRY").unwrap();
            let change = std::env::var("NATIVE_HTTP_COMPLETION_CHANGE").unwrap();
            let status: u16 = std::env::var("NATIVE_HTTP_COMPLETION_STATUS")
                .unwrap()
                .parse()
                .unwrap();
            let kind = std::env::var("NATIVE_HTTP_COMPLETION_KIND").unwrap();
            let client = crate::MahayanaProductClient::new_with_default_api_base_url(
                root.join("session.json"),
                root.join("surface.json"),
            );
            assert!(enabled());
            let result = match entry.as_str() {
                name if name.starts_with("platform-") => {
                    let method = match name {
                        "platform-post" => "POST",
                        "platform-put" => "PUT",
                        "platform-patch" => "PATCH",
                        "platform-delete" => "DELETE",
                        _ => "GET",
                    };
                    client.execute_session_owned_platform_request(&json!({
                        "method": method, "path": "/api/social/friends",
                        "authenticated": name != "platform-anonymous",
                        "body": if method == "GET" { Value::Null } else { json!({"fixture":true}) },
                    }))
                }
                "get" | "get-anonymous" => {
                    client.get_json("/v1/wallet/balance", &[], (entry == "get").then_some(TOKEN))
                }
                "post" | "post-anonymous" => client.post_json(
                    "/v1/purchases/restore",
                    json!({"fixture":true}),
                    (entry == "post").then_some(TOKEN),
                ),
                "auth-status" => client.auth_status(&json!({})),
                "multipart" => {
                    let package = b"completion-package-bytes";
                    let digest = format!("{:x}", Sha256::digest(package));
                    client.publish_plugin(
                        "completion-contract",
                        "1.0.0",
                        "https://native-completion-contract.workers.dev",
                        &digest,
                        package.len() as u64,
                        &["desktop".into()],
                        package,
                        &json!({"fixture":true}),
                        &json!({"fixture":true}),
                    )
                }
                "body-barrier" => {
                    let url = url::Url::parse(&format!(
                        "{}/v1/wallet/balance",
                        client.effective_api_base_url().unwrap()
                    ))
                    .unwrap();
                    let (response, credential) = client
                        .native_send(reqwest::Method::GET, url, None, Some(TOKEN))
                        .expect("obtain actual HTTP headers before body release");
                    assert_eq!(response.status().as_u16(), status);
                    assert!(credential.is_some());
                    client.complete_native_response(credential.as_ref(), || {
                        std::fs::write(root.join("release-body"), b"ready")
                            .expect("release body from inside the actual completion callback");
                        crate::decode_native_response(response, Some(TOKEN))
                    })
                }
                other => panic!("unexpected child entry: {other}"),
            };
            let stale = change != "unchanged" && !entry.ends_with("-anonymous");
            if stale {
                let error =
                    result.expect_err("obsolete response must not escape its dispatch binding");
                assert!(
                    matches!(&error, ProductError::Session(code) if code == "NATIVE_CREDENTIAL_CHANGED"),
                    "fixed credential-completion failure: {error:?}"
                );
                let rendered = format!("{error:?} {error}");
                assert!(!rendered.contains(OBSOLETE_DATA));
                assert!(!rendered.contains(TOKEN));
                assert!(!rendered.contains("127.0.0.1"));
                return;
            }
            if status >= 300 {
                let error = result.expect_err("JSON endpoints retain their fixed HTTP failure");
                assert!(matches!(&error,
                    ProductError::HttpStatus { status: actual, message }
                    if *actual == status && message == "Native request rejected"
                ));
                assert!(!format!("{error:?}").contains(OBSOLETE_DATA));
                return;
            }
            if kind != "json" {
                let error = result.expect_err("unchanged binding preserves bounded decode failure");
                match kind.as_str() {
                    "invalid-json" => assert!(
                        matches!(error, ProductError::Response(code) if code == "NATIVE_RESPONSE_JSON")
                    ),
                    "invalid-utf8" => assert!(
                        matches!(error, ProductError::Response(code) if code == "NATIVE_RESPONSE_ENCODING")
                    ),
                    "oversized" => assert!(
                        matches!(error, ProductError::Response(code) if code == "NATIVE_RESPONSE_SIZE")
                    ),
                    "truncated" => assert!(
                        matches!(error, ProductError::Transport(code) if code == "NATIVE_RESPONSE_READ")
                    ),
                    other => panic!("unexpected body kind: {other}"),
                }
                return;
            }
            let value = result.expect("current or anonymous response remains usable");
            let data = if entry.starts_with("platform-") {
                assert_eq!(value["statusCode"], 200);
                assert_eq!(value["ok"], true);
                &value["data"]
            } else {
                &value
            };
            assert_eq!(data["marker"], MARKER);
            assert!(
                !value.to_string().contains(TOKEN),
                "existing token redaction remains active"
            );
        }
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

    #[cfg(all(unix, debug_assertions))]
    #[test]
    fn native_development_client_child() {
        const LOOPBACK: &str = "http://127.0.0.1:49226";
        if std::env::var("NATIVE_DEVELOPMENT_CONTRACT_CHILD")
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
        assert_eq!(
            client.effective_api_base_url().unwrap(),
            "http://127.0.0.1:49226"
        );
        assert_eq!(client.restore_session().unwrap()["user"]["id"], "42");
        assert!(client.session_token().is_err());
        let native = read_current().unwrap().unwrap();
        assert_eq!(native.account_id(), "42");
        assert!(
            native
                .guard_request(&format!("{PRODUCTION_ISSUER}/v1/messages"), TOKEN)
                .is_err()
        );
        assert!(
            native
                .guard_request(&format!("{ISOLATED_ISSUER}/v1/messages"), TOKEN)
                .is_err()
        );
        assert!(
            native
                .guard_request(&format!("{LOOPBACK}/v1/messages"), TOKEN)
                .is_ok()
        );
        assert!(
            credential(PRODUCTION_ISSUER)
                .guard_request(&format!("{LOOPBACK}/v1/messages"), TOKEN)
                .is_err()
        );
        assert!(
            credential(ISOLATED_ISSUER)
                .guard_request(&format!("{LOOPBACK}/v1/messages"), TOKEN)
                .is_err()
        );
        let explicit_production =
            crate::MahayanaProductClient::new(PRODUCTION_ISSUER, root.join("explicit-prod.json"));
        assert!(explicit_production.effective_api_base_url().is_err());
        std::fs::remove_file(root.join("native")).unwrap();
        assert!(!client.bootstrap_ci_test_account_session().unwrap());
        assert_eq!(client.auth_status(&json!({})).unwrap()["loggedIn"], false);
        assert!(client.load_session().unwrap().is_none());
        assert!(!root.join("session.json").exists());
    }
}
