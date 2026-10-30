use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::gateway_command_error::GatewayCommandError;

pub struct DevicePairingOwner {
    conn: Mutex<Connection>,
    db_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceTokenAuthorization {
    pub device_id: String,
    pub role: String,
    pub scopes: Vec<String>,
}

const PENDING_PAIRING_TTL_MS: i64 = 900_000;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn new_device_token() -> Result<String, GatewayCommandError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|_| GatewayCommandError::Internal("failed to generate device token".into()))?;
    Ok(format!(
        "sand-device-tok-{}",
        URL_SAFE_NO_PAD.encode(bytes)
    ))
}

fn scopes_for_role(
    scopes_json: Option<&str>,
    role: &str,
) -> Result<Value, GatewayCommandError> {
    let scopes = scopes_json
        .unwrap_or("[]")
        .parse::<Value>()
        .map_err(|error| {
            GatewayCommandError::BadRequest(format!("invalid approved device scopes: {error}"))
        })?;
    let Some(scopes) = scopes.as_array() else {
        return Err(GatewayCommandError::BadRequest(
            "approved device scopes must be an array".into(),
        ));
    };
    let mut normalized = Vec::new();
    for scope in scopes.iter().filter_map(Value::as_str).map(str::trim) {
        if scope.is_empty()
            || (role == "operator" && !scope.starts_with("operator."))
            || (role != "operator" && scope.starts_with("operator."))
        {
            continue;
        }
        if !normalized.iter().any(|known| known == scope) {
            normalized.push(scope.to_string());
        }
    }
    Ok(Value::Array(
        normalized.into_iter().map(Value::String).collect(),
    ))
}

fn parse_string_set(raw: &str) -> Option<BTreeSet<String>> {
    let values = serde_json::from_str::<Value>(raw).ok()?.as_array()?.clone();
    let mut result = BTreeSet::new();
    for value in values {
        let value = value.as_str()?.trim();
        if value.is_empty() {
            return None;
        }
        result.insert(value.to_string());
    }
    Some(result)
}

fn pending_request_is_compatible(
    existing_public_key: &str,
    existing_browser_origin: Option<&str>,
    existing_role: Option<&str>,
    existing_roles_json: Option<&str>,
    existing_scopes_json: Option<&str>,
    public_key: &str,
    browser_origin: Option<&str>,
    role: &str,
    roles_json: &str,
    scopes_json: &str,
) -> bool {
    if existing_public_key != public_key
        || existing_browser_origin != browser_origin
        || existing_role != Some(role)
    {
        return false;
    }
    let (Some(existing_roles), Some(existing_scopes), Some(requested_roles), Some(requested_scopes)) = (
        existing_roles_json.and_then(parse_string_set),
        existing_scopes_json.and_then(parse_string_set),
        parse_string_set(roles_json),
        parse_string_set(scopes_json),
    ) else {
        return false;
    };
    requested_roles.is_subset(&existing_roles) && requested_scopes.is_subset(&existing_scopes)
}

fn bump_revision(conn: &Connection) -> Result<u64, rusqlite::Error> {
    let now = now_ms();
    let current_val: Option<String> = conn
        .query_row(
            "SELECT value_json FROM config_machine_state WHERE state_key = 'devicePairing.revision'",
            [],
            |row| row.get(0),
        )
        .ok();
    let current_rev: u64 = current_val
        .and_then(|v| v.trim_matches('"').parse().ok())
        .unwrap_or(0);
    let next_rev = current_rev + 1;
    conn.execute(
        "INSERT INTO config_machine_state (state_key, value_json, updated_at_ms)
         VALUES ('devicePairing.revision', ?1, ?2)
         ON CONFLICT(state_key) DO UPDATE SET value_json = ?1, updated_at_ms = ?2",
        params![format!("\"{next_rev}\""), now],
    )?;
    Ok(next_rev)
}

fn purge_expired_pending_requests(
    conn: &Connection,
    now: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM device_pairing_pending
         WHERE COALESCE(refreshed_at_ms, ts) <= ?1 - ?2",
        params![now, PENDING_PAIRING_TTL_MS],
    )?;
    Ok(())
}

impl DevicePairingOwner {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path).map_err(|e| e.to_string())?;
        let owner = Self {
            conn: Mutex::new(conn),
            db_path: Some(path),
        };
        owner.init_schema().map_err(|e| e.to_string())?;
        Ok(owner)
    }

    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let owner = Self {
            conn: Mutex::new(conn),
            db_path: None,
        };
        owner.init_schema().map_err(|e| e.to_string())?;
        Ok(owner)
    }

    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS device_pairing_pending (
                request_id TEXT PRIMARY KEY NOT NULL,
                device_id TEXT NOT NULL,
                public_key TEXT NOT NULL,
                display_name TEXT,
                platform TEXT,
                device_family TEXT,
                client_id TEXT,
                client_mode TEXT,
                browser_origin TEXT,
                role TEXT,
                roles_json TEXT,
                scopes_json TEXT,
                remote_ip TEXT,
                silent INTEGER,
                is_repair INTEGER,
                ts INTEGER NOT NULL,
                refreshed_at_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_device_pairing_pending_device_id ON device_pairing_pending(device_id);

            CREATE TABLE IF NOT EXISTS device_pairing_paired (
                device_id TEXT PRIMARY KEY NOT NULL,
                public_key TEXT NOT NULL,
                display_name TEXT,
                operator_label TEXT,
                platform TEXT,
                device_family TEXT,
                client_id TEXT,
                client_mode TEXT,
                browser_origin TEXT,
                role TEXT,
                roles_json TEXT,
                scopes_json TEXT,
                approved_scopes_json TEXT,
                remote_ip TEXT,
                tokens_json TEXT NOT NULL,
                approved_via TEXT,
                node_surface_json TEXT,
                pending_node_surface_json TEXT,
                created_at_ms INTEGER NOT NULL,
                approved_at_ms INTEGER NOT NULL,
                last_seen_at_ms INTEGER,
                last_seen_reason TEXT
            );

            CREATE TABLE IF NOT EXISTS device_bootstrap_tokens (
                token_key TEXT PRIMARY KEY NOT NULL,
                token TEXT NOT NULL,
                setup_id TEXT,
                ts INTEGER NOT NULL,
                device_id TEXT,
                public_key TEXT,
                profile_json TEXT,
                redeemed_profile_json TEXT,
                pending_profile_json TEXT,
                issued_at_ms INTEGER NOT NULL,
                last_used_at_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_device_bootstrap_tokens_setup_id ON device_bootstrap_tokens(setup_id);

            CREATE TABLE IF NOT EXISTS device_pairing_join_codes (
                shortcode TEXT PRIMARY KEY NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                expires_at_ms INTEGER NOT NULL,
                redeemed_at_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_device_pairing_join_codes_expiry
                ON device_pairing_join_codes(expires_at_ms);

            CREATE TABLE IF NOT EXISTS device_pair_setup_completions (
                setup_id TEXT PRIMARY KEY NOT NULL,
                device_id TEXT NOT NULL,
                device_name TEXT,
                access TEXT NOT NULL,
                completed_at_ms INTEGER NOT NULL,
                delivery_state TEXT NOT NULL,
                retain_until_ms INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS config_machine_state (
                state_key TEXT PRIMARY KEY NOT NULL,
                value_json TEXT NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn current_revision(&self) -> u64 {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value_json FROM config_machine_state WHERE state_key = 'devicePairing.revision'",
            [],
            |row| row.get(0),
        )
        .ok()
        .and_then(|v: String| v.trim_matches('"').parse().ok())
        .unwrap_or(0)
    }

    /// Resolves a durable device token to its approved role and scopes.
    ///
    /// Token verification belongs to the pairing owner; Gateway ingress only
    /// consumes this authorization result and must not inspect pairing rows.
    pub fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        if token.is_empty() || token.len() > 256 {
            return Ok(None);
        }

        let conn = self.conn.lock().unwrap();
        let mut statement = conn
            .prepare("SELECT device_id, tokens_json FROM device_pairing_paired")
            .map_err(|error| {
                GatewayCommandError::Internal(format!("failed to read device tokens: {error}"))
            })?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| {
                GatewayCommandError::Internal(format!("failed to enumerate device tokens: {error}"))
            })?;

        for row in rows {
            let (device_id, tokens_json) = row.map_err(|error| {
                GatewayCommandError::Internal(format!("failed to read device token row: {error}"))
            })?;
            let entries = match serde_json::from_str::<Value>(&tokens_json) {
                Ok(Value::Object(entries)) => entries,
                Ok(_) | Err(_) => continue,
            };
            for entry in entries.values() {
                let Some(candidate) = entry.get("token").and_then(Value::as_str) else {
                    continue;
                };
                if candidate.len() != token.len()
                    || !bool::from(candidate.as_bytes().ct_eq(token.as_bytes()))
                {
                    continue;
                }
                let Some(role) = entry.get("role").and_then(Value::as_str) else {
                    continue;
                };
                let scopes = entry
                    .get("scopes")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                return Ok(Some(DeviceTokenAuthorization {
                    device_id,
                    role: role.to_string(),
                    scopes,
                }));
            }
        }
        Ok(None)
    }

    pub fn handle_pair_request(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args
            .get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.request requires deviceId".into())
            })?;
        let public_key = args
            .get("publicKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.request requires publicKey".into())
            })?;

        let display_name = args
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or(device_id);
        let platform = args
            .get("platform")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let device_family = args.get("deviceFamily").and_then(Value::as_str);
        let client_id = args.get("clientId").and_then(Value::as_str);
        let client_mode = args.get("clientMode").and_then(Value::as_str);
        let browser_origin = args.get("browserOrigin").and_then(Value::as_str);
        let role = args
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("operator");
        let roles_json = args
            .get("roles")
            .map(|v| v.to_string())
            .unwrap_or_else(|| json!([role]).to_string());
        let scopes_json = args
            .get("scopes")
            .map(|v| v.to_string())
            .unwrap_or_else(|| json!(["operator.read", "operator.write"]).to_string());
        let silent = if args.get("silent").and_then(Value::as_bool) == Some(true) {
            1
        } else {
            0
        };

        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        purge_expired_pending_requests(&conn, now).map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to expire pending pairings: {error}"))
        })?;

        // Check if device is already paired (is_repair)
        let is_repair: bool = conn
            .query_row(
                "SELECT COUNT(1) FROM device_pairing_paired WHERE device_id = ?1",
                params![device_id],
                |row| row.get(0),
            )
            .map(|c: i64| c > 0)
            .unwrap_or(false);

        let pending_candidates = {
            let mut statement = conn
                .prepare(
                    "SELECT request_id, public_key, browser_origin, role, roles_json, scopes_json
                     FROM device_pairing_pending
                     WHERE device_id = ?1
                     ORDER BY ts DESC",
                )
                .map_err(|error| {
                    GatewayCommandError::Internal(format!(
                        "Failed to inspect pending pairing requests: {error}"
                    ))
                })?;
            let rows = statement
                .query_map(params![device_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                })
                .map_err(|error| {
                    GatewayCommandError::Internal(format!(
                        "Failed to enumerate pending pairing requests: {error}"
                    ))
                })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
                GatewayCommandError::Internal(format!(
                    "Failed to read pending pairing request: {error}"
                ))
            })?
        };

        if let Some((request_id, ..)) =
            pending_candidates.iter().find(|(_, existing_public_key, existing_browser_origin, existing_role, existing_roles, existing_scopes)| {
                pending_request_is_compatible(
                    existing_public_key,
                    existing_browser_origin.as_deref(),
                    existing_role.as_deref(),
                    existing_roles.as_deref(),
                    existing_scopes.as_deref(),
                    public_key,
                    browser_origin,
                    role,
                    &roles_json,
                    &scopes_json,
                )
            })
        {
            conn.execute(
                "UPDATE device_pairing_pending
                 SET display_name = ?1, platform = ?2, device_family = ?3,
                     client_id = ?4, client_mode = ?5, browser_origin = ?6,
                     silent = ?7, refreshed_at_ms = ?8
                 WHERE request_id = ?9",
                params![
                    display_name,
                    platform,
                    device_family,
                    client_id,
                    client_mode,
                    browser_origin,
                    silent,
                    now,
                    request_id,
                ],
            )
            .map_err(|error| {
                GatewayCommandError::Internal(format!(
                    "Failed to refresh pending pairing request: {error}"
                ))
            })?;
            return Ok(json!({
                "status": "pending",
                "requestId": request_id,
                "deviceId": device_id,
                "created": false,
                "refreshedAtMs": now,
                "isRepair": is_repair,
                "expiresAtMs": now + PENDING_PAIRING_TTL_MS
            }));
        }

        if !pending_candidates.is_empty() {
            conn.execute(
                "DELETE FROM device_pairing_pending WHERE device_id = ?1",
                params![device_id],
            )
            .map_err(|error| {
                GatewayCommandError::Internal(format!(
                    "Failed to supersede pending pairing requests: {error}"
                ))
            })?;
        }

        let request_id = format!("pair-{}", Uuid::new_v4());
        let is_repair_int = if is_repair { 1 } else { 0 };

        conn.execute(
            r#"INSERT INTO device_pairing_pending (
                request_id, device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, browser_origin, role, roles_json, scopes_json,
                silent, is_repair, ts, refreshed_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)"#,
            params![
                request_id, device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, browser_origin, role, roles_json, scopes_json,
                silent, is_repair_int, now
            ],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to store pending pairing: {e}")))?;

        Ok(json!({
            "status": "pending",
            "requestId": request_id,
            "deviceId": device_id,
            "created": true,
            "isRepair": is_repair,
            "expiresAtMs": now + PENDING_PAIRING_TTL_MS
        }))
    }

    pub fn handle_pair_status(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let request_id = args.get("requestId").and_then(Value::as_str).map(str::trim);
        let device_id = args.get("deviceId").and_then(Value::as_str).map(str::trim);

        let conn = self.conn.lock().unwrap();
        purge_expired_pending_requests(&conn, now_ms()).map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to expire pending pairings: {error}"))
        })?;

        if let Some(req_id) = request_id {
            let pending_exists: bool = conn
                .query_row(
                    "SELECT COUNT(1) FROM device_pairing_pending WHERE request_id = ?1",
                    params![req_id],
                    |row| row.get(0),
                )
                .map(|c: i64| c > 0)
                .unwrap_or(false);

            if pending_exists {
                return Ok(json!({
                    "status": "pending",
                    "requestId": req_id
                }));
            }
        }

        if let Some(dev_id) = device_id {
            let paired_row: Option<(String, Option<String>, Option<String>, Option<String>)> = conn
                .query_row(
                    "SELECT device_id, display_name, platform, device_family FROM device_pairing_paired WHERE device_id = ?1",
                    params![dev_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .ok();

            if let Some((d_id, name, platform, device_family)) = paired_row {
                return Ok(json!({
                    "status": "approved",
                    "device": {
                        "deviceId": d_id,
                        "displayName": name.unwrap_or_else(|| d_id.clone()),
                        "platform": platform,
                        "deviceFamily": device_family
                    }
                }));
            }
        }

        Ok(json!({ "status": "rejected" }))
    }

    pub fn handle_pair_approve(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let request_id = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.approve requires requestId".into())
            })?;

        let mut conn = self.conn.lock().unwrap();
        purge_expired_pending_requests(&conn, now_ms()).map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to expire pending pairings: {error}"))
        })?;

        // 1. Fetch pending record
        let pending = conn
            .query_row(
                r#"SELECT device_id, public_key, display_name, platform, device_family,
                      client_id, client_mode, browser_origin, role, roles_json, scopes_json, ts
               FROM device_pairing_pending WHERE request_id = ?1"#,
                params![request_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, i64>(11)?,
                    ))
                },
            )
            .map_err(|_| {
                GatewayCommandError::BadRequest(format!("unknown requestId: {request_id}"))
            })?;

        let (
            device_id,
            public_key,
            display_name,
            platform,
            device_family,
            client_id,
            client_mode,
            browser_origin,
            role,
            roles_json,
            scopes_json,
            created_at,
        ) = pending;

        let now = now_ms();
        let requested_roles = roles_json
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|error| {
                GatewayCommandError::BadRequest(format!("invalid requested device roles: {error}"))
            })?
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
        let mut token_entries = serde_json::Map::new();
        for role in requested_roles
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|role| !role.is_empty())
        {
            if token_entries.contains_key(role) {
                continue;
            }
            token_entries.insert(
                role.to_string(),
                json!({
                    "token": new_device_token()?,
                    "role": role,
                    "scopes": scopes_for_role(scopes_json.as_deref(), role)?,
                    "createdAtMs": now,
                    "rotatedAtMs": now
                }),
            );
        }
        if token_entries.is_empty() {
            return Err(GatewayCommandError::BadRequest(
                "device pairing approval requires at least one non-empty role".into(),
            ));
        }
        let tokens_json = Value::Object(token_entries).to_string();

        // Atomic transaction: delete from pending, upsert into paired, bump revision.
        let tx = conn.transaction().map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to begin pairing approval: {error}"))
        })?;
        tx.execute(
            "DELETE FROM device_pairing_pending WHERE request_id = ?1",
            params![request_id],
        )
        .map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to delete pending request: {e}"))
        })?;

        tx.execute(
            r#"INSERT INTO device_pairing_paired (
                device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, browser_origin, role, roles_json, scopes_json, approved_scopes_json,
                tokens_json, approved_via, created_at_ms, approved_at_ms, last_seen_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12, 'owner', ?13, ?14, ?14)
            ON CONFLICT(device_id) DO UPDATE SET
                public_key = ?2, display_name = ?3, platform = ?4, device_family = ?5,
                client_id = ?6, client_mode = ?7, browser_origin = ?8, role = ?9,
                roles_json = ?10, scopes_json = ?11, approved_scopes_json = ?11,
                tokens_json = ?12, node_surface_json = NULL, pending_node_surface_json = NULL,
                approved_at_ms = ?14, last_seen_at_ms = ?14"#,
            params![
                device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, browser_origin, role, roles_json, scopes_json,
                tokens_json, created_at, now
            ],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to persist paired device: {e}")))?;

        let new_rev = bump_revision(&tx).map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}"))
        })?;
        tx.commit().map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to commit pairing approval: {error}"))
        })?;

        Ok(json!({
            "requestId": request_id,
            "deviceId": device_id,
            "decision": "approved",
            "revision": new_rev,
            "device": {
                "deviceId": device_id,
                "displayName": display_name,
                "platform": platform,
                "approvedAtMs": now
            }
        }))
    }

    pub fn handle_pair_reject(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let request_id = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.reject requires requestId".into())
            })?;

        let conn = self.conn.lock().unwrap();
        purge_expired_pending_requests(&conn, now_ms()).map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to expire pending pairings: {error}"))
        })?;

        let device_id: String = conn
            .query_row(
                "SELECT device_id FROM device_pairing_pending WHERE request_id = ?1",
                params![request_id],
                |row| row.get(0),
            )
            .map_err(|_| {
                GatewayCommandError::BadRequest(format!("unknown requestId: {request_id}"))
            })?;

        conn.execute(
            "DELETE FROM device_pairing_pending WHERE request_id = ?1",
            params![request_id],
        )
        .map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to delete pending request: {e}"))
        })?;

        Ok(json!({
            "requestId": request_id,
            "deviceId": device_id,
            "decision": "rejected"
        }))
    }

    pub fn handle_pair_list(&self, _args: &Value) -> Result<Value, GatewayCommandError> {
        let conn = self.conn.lock().unwrap();
        purge_expired_pending_requests(&conn, now_ms()).map_err(|error| {
            GatewayCommandError::Internal(format!("Failed to expire pending pairings: {error}"))
        })?;

        let mut pending_stmt = conn.prepare(
            "SELECT request_id, device_id, display_name, platform, role, ts FROM device_pairing_pending"
        ).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let pending_rows = pending_stmt
            .query_map([], |row| {
                Ok(json!({
                    "requestId": row.get::<_, String>(0)?,
                    "deviceId": row.get::<_, String>(1)?,
                    "displayName": row.get::<_, Option<String>>(2)?,
                    "platform": row.get::<_, Option<String>>(3)?,
                    "role": row.get::<_, Option<String>>(4)?,
                    "ts": row.get::<_, i64>(5)?
                }))
            })
            .map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let pending: Vec<Value> = pending_rows.filter_map(Result::ok).collect();

        let mut paired_stmt = conn.prepare(
            "SELECT device_id, display_name, platform, role, approved_at_ms, last_seen_at_ms FROM device_pairing_paired"
        ).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let paired_rows = paired_stmt
            .query_map([], |row| {
                Ok(json!({
                    "deviceId": row.get::<_, String>(0)?,
                    "displayName": row.get::<_, Option<String>>(1)?,
                    "platform": row.get::<_, Option<String>>(2)?,
                    "role": row.get::<_, Option<String>>(3)?,
                    "approvedAtMs": row.get::<_, i64>(4)?,
                    "lastSeenAtMs": row.get::<_, Option<i64>>(5)?
                }))
            })
            .map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let paired: Vec<Value> = paired_rows.filter_map(Result::ok).collect();

        Ok(json!({
            "pending": pending,
            "paired": paired
        }))
    }

    pub fn handle_pair_revoke(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args
            .get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.revoke requires deviceId".into())
            })?;

        let role = args.get("role").and_then(Value::as_str);

        let mut conn = self.conn.lock().unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(1) FROM device_pairing_paired WHERE device_id = ?1",
                params![device_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if count == 0 {
            return Err(GatewayCommandError::BadRequest(format!(
                "unknown deviceId: {device_id}"
            )));
        }

        let tx = conn.transaction().map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to begin pairing revoke transaction: {e}"))
        })?;

        tx.execute(
            "DELETE FROM device_pairing_paired WHERE device_id = ?1",
            params![device_id],
        )
        .map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to delete paired device: {e}"))
        })?;

        let new_rev = bump_revision(&tx).map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}"))
        })?;
        tx.commit().map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to commit pairing revoke transaction: {e}"))
        })?;

        Ok(json!({
            "deviceId": device_id,
            "role": role.unwrap_or("all"),
            "revokedAtMs": now_ms(),
            "revision": new_rev
        }))
    }

    pub fn handle_pair_rotate_token(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args
            .get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("device.pair.rotateToken requires deviceId".into())
            })?;

        let role = args
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("operator");
        let mut conn = self.conn.lock().unwrap();

        let (roles_json, scopes_json): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT roles_json, scopes_json FROM device_pairing_paired WHERE device_id = ?1",
                params![device_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| {
                GatewayCommandError::BadRequest(format!("unknown deviceId: {device_id}"))
            })?;

        let approved_roles = roles_json
            .as_deref()
            .map(serde_json::from_str::<Value>)
            .transpose()
            .map_err(|error| {
                GatewayCommandError::Internal(format!("invalid approved device roles: {error}"))
            })?
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
        if !approved_roles
            .iter()
            .filter_map(Value::as_str)
            .any(|approved| approved == role)
        {
            return Err(GatewayCommandError::BadRequest(format!(
                "role {role} is not approved for deviceId: {device_id}"
            )));
        }
        let role_scopes = scopes_for_role(scopes_json.as_deref(), role)?;

        let now = now_ms();
        let new_token = new_device_token()?;
        let tokens_json = json!({
            role: {
                "token": new_token,
                "role": role,
                "scopes": role_scopes,
                "createdAtMs": now,
                "rotatedAtMs": now
            }
        })
        .to_string();

        let tx = conn.transaction().map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to begin token rotation transaction: {e}"))
        })?;

        tx.execute(
            "UPDATE device_pairing_paired
             SET tokens_json = ?1, node_surface_json = NULL, pending_node_surface_json = NULL
             WHERE device_id = ?2",
            params![tokens_json, device_id],
        )
        .map_err(|e| GatewayCommandError::Internal(format!("Failed to update tokens: {e}")))?;

        let new_rev = bump_revision(&tx).map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}"))
        })?;
        tx.commit().map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to commit token rotation transaction: {e}"))
        })?;

        let is_cross_device = args
            .get("crossDevice")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if is_cross_device {
            Ok(json!({
                "deviceId": device_id,
                "role": role,
                "rotatedAtMs": now,
                "tokenDelivery": "withheld-cross-device",
                "revision": new_rev
            }))
        } else {
            Ok(json!({
                "deviceId": device_id,
                "role": role,
                "token": new_token,
                "rotatedAtMs": now,
                "tokenDelivery": "in-band",
                "revision": new_rev
            }))
        }
    }

    pub fn handle_pair_setup_code(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let gateway_url = args
            .get("gatewayUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(url) = gateway_url {
            validate_mobile_pairing_url(url)
                .map_err(GatewayCommandError::BadRequest)?;
        }
        let setup_id = format!("setup-{}", Uuid::new_v4());
        let setup_code = format!("sand-setup-{}", Uuid::new_v4());
        let now = now_ms();
        let expires_at = now + 900_000;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO device_bootstrap_tokens (token_key, token, setup_id, ts, issued_at_ms)
               VALUES (?1, ?2, ?3, ?4, ?4)"#,
            params![setup_code, setup_code, setup_id, now],
        )
        .map_err(|e| {
            GatewayCommandError::Internal(format!("Failed to store bootstrap token: {e}"))
        })?;

        let qr_data_url = crate::extensions::attachments::media_qr_png::render_qr_png_data_url(
            &setup_code,
            &crate::extensions::attachments::media_qr_png::QrPngRenderOptions::default(),
        )
        .ok();
        let qr_terminal =
            crate::extensions::attachments::media_qr_png::render_qr_terminal(&setup_code, true)
                .ok();

        let mut res = json!({
            "setupId": setup_id,
            "setupCode": setup_code,
            "auth": "token",
            "urlSource": "local",
            "access": "full",
            "expiresAtMs": expires_at
        });
        if let Some(url) = gateway_url {
            res["gatewayUrl"] = Value::String(url.to_string());
        }
        if let Some(qr) = qr_data_url {
            res["qrDataUrl"] = Value::String(qr);
        }
        if let Some(term) = qr_terminal {
            res["qrTerminal"] = Value::String(term);
        }
        Ok(res)
    }

    /// Stores a short-lived pairing setup payload under a cryptographically random
    /// 128-bit base64url shortcode. The payload is only returned once by
    /// `redeem_device_pairing_join_code`.
    pub fn register_device_pairing_join_code(
        &self,
        payload: &Value,
        expires_at_ms: i64,
    ) -> Result<String, GatewayCommandError> {
        let now = now_ms();
        if expires_at_ms <= now {
            return Err(GatewayCommandError::BadRequest(
                "device pairing join code requires a future expiry".into(),
            ));
        }
        let payload_json = serde_json::to_string(payload).map_err(|error| {
            GatewayCommandError::BadRequest(format!("invalid device pairing payload: {error}"))
        })?;
        if payload_json.len() > 64 * 1024 {
            return Err(GatewayCommandError::BadRequest(
                "device pairing payload exceeds 64 KiB".into(),
            ));
        }

        let mut bytes = [0_u8; DEVICE_PAIRING_JOIN_CODE_BYTES];
        let conn = self.conn.lock().unwrap();
        for _ in 0..4 {
            getrandom::fill(&mut bytes).map_err(|_| {
                GatewayCommandError::Internal("failed to generate device pairing join code".into())
            })?;
            let shortcode = URL_SAFE_NO_PAD.encode(bytes);
            let inserted = conn
                .execute(
                    "INSERT OR IGNORE INTO device_pairing_join_codes
                     (shortcode, payload_json, created_at_ms, expires_at_ms)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![shortcode, payload_json, now, expires_at_ms],
                )
                .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
            if inserted == 1 {
                return Ok(shortcode);
            }
        }
        Err(GatewayCommandError::Internal(
            "failed to allocate a unique device pairing join code".into(),
        ))
    }

    /// Atomically burns a live shortcode and returns its validated JSON payload.
    pub fn redeem_device_pairing_join_code(
        &self,
        shortcode: &str,
    ) -> Result<Option<Value>, GatewayCommandError> {
        let shortcode = shortcode.trim();
        if !is_device_pairing_join_code(shortcode) {
            return Ok(None);
        }
        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        let payload_json: Option<String> = conn
            .query_row(
                "SELECT payload_json FROM device_pairing_join_codes
                 WHERE shortcode = ?1 AND redeemed_at_ms IS NULL AND expires_at_ms > ?2",
                params![shortcode, now],
                |row| row.get(0),
            )
            .ok();
        let Some(payload_json) = payload_json else {
            return Ok(None);
        };
        let updated = conn
            .execute(
                "UPDATE device_pairing_join_codes SET redeemed_at_ms = ?1
                 WHERE shortcode = ?2 AND redeemed_at_ms IS NULL AND expires_at_ms > ?1",
                params![now, shortcode],
            )
            .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
        if updated != 1 {
            return Ok(None);
        }
        let payload = serde_json::from_str(&payload_json).map_err(|error| {
            GatewayCommandError::Internal(format!(
                "stored device pairing payload is invalid: {error}"
            ))
        })?;
        Ok(Some(payload))
    }

    pub fn handle_pairing_command(
        &self,
        method: &str,
        args: &Value,
    ) -> Result<Value, GatewayCommandError> {
        match method {
            "device.pair.request" => self.handle_pair_request(args),
            "device.pair.status" => self.handle_pair_status(args),
            "device.pair.approve" => self.handle_pair_approve(args),
            "device.pair.reject" => self.handle_pair_reject(args),
            "device.pair.list" => self.handle_pair_list(args),
            "device.pair.revoke" => self.handle_pair_revoke(args),
            "device.pair.rotateToken" => self.handle_pair_rotate_token(args),
            "device.pair.setupCode" => self.handle_pair_setup_code(args),
            _ => Err(GatewayCommandError::UnknownMethod(format!(
                "Unknown device pairing method: {method}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// OpenClaw Pairing Domain Helpers (`src/pairing/`)
// ---------------------------------------------------------------------------

pub const DEVICE_PAIRING_JOIN_CODE_BYTES: usize = 16;

/// Validates whether a string is a 22-character base64url device pairing join code.
pub fn is_device_pairing_join_code(value: &str) -> bool {
    if value.len() != 22 {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Parses the pairing join request path from a URL pathname.
///
/// Matches OpenClaw `parseDevicePairingJoinRequestPath`:
/// - Terminal `/j` returns `Some("")`
/// - Last `/j/<shortcode>` returns `Some("<shortcode>")`
/// - Otherwise returns `None`
pub fn parse_device_pairing_join_request_path(pathname: &str) -> Option<&str> {
    if pathname.ends_with("/j") {
        return Some("");
    }
    if let Some(pos) = pathname.rfind("/j/") {
        return Some(&pathname[pos + 3..]);
    }
    None
}

/// Sanitizes pairing key tokens (channel or account ID).
fn normalize_pairing_key(value: &str, kind: &str) -> Result<String, String> {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(format!("invalid pairing {kind}: expected non-empty string"));
    }
    let sanitized: String = trimmed
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    let safe = sanitized.replace("..", "_");
    if safe.is_empty() || safe == "_" {
        return Err(format!("invalid pairing {kind}: sanitized key is empty"));
    }
    Ok(safe)
}

/// Sanitizes channel key for pairing store.
pub fn safe_channel_key(channel: &str) -> Result<String, String> {
    normalize_pairing_key(channel, "channel")
}

/// Sanitizes account ID key for pairing store.
pub fn safe_account_key(account_id: &str) -> Result<String, String> {
    normalize_pairing_key(account_id, "account id")
}

/// Resolves pairing id label shown in approval prompts.
pub fn resolve_pairing_id_label(custom_label: Option<&str>) -> &str {
    custom_label.unwrap_or("userId")
}

/// Formats the user-facing pairing challenge reply text.
pub fn build_pairing_reply(channel: &str, id_line: &str, code: &str) -> String {
    format!(
        "OpenClaw: access not configured.\n\n{}\nPairing code:\n```\n{}\n```\n\nAsk the bot owner to approve with:\n```\nopenclaw pairing approve {} {}\n```",
        id_line, code, channel, code
    )
}

/// Checks whether a hostname belongs to a private LAN or local domain.
pub fn is_private_lan_host(host: &str) -> bool {
    let host = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_lowercase();
    let host = host.split('%').next().unwrap_or(&host);
    if host.ends_with(".local") {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                let octets = v4.octets();
                octets[0] == 10
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
                    || (octets[0] == 169 && octets[1] == 254) // Link-local
            }
            std::net::IpAddr::V6(v6) => {
                let segs = v6.segments();
                (segs[0] & 0xffc0) == 0xfe80 // Link-local (fe80::/10)
                    || (segs[0] & 0xfe00) == 0xfc00 // ULA (fc00::/7, covers fc00 and fd00)
            }
        }
    } else {
        false
    }
}

/// Checks whether cleartext ws:// is allowed for this mobile pairing host.
pub fn is_mobile_pairing_cleartext_allowed_host(host: &str) -> bool {
    let host = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_lowercase();
    let host = host.split('%').next().unwrap_or(&host);
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "10.0.2.2" // Android emulator loopback alias
        || is_private_lan_host(host)
}

/// Validates mobile pairing URL security.
/// Rejects cleartext `ws://` on public internet; allows `wss://` and LAN/localhost `ws://`.
pub fn validate_mobile_pairing_url(url: &str) -> Result<(), String> {
    let parsed =
        url::Url::parse(url).map_err(|_| "Resolved mobile pairing URL is invalid.".to_string())?;
    let scheme = parsed.scheme().to_lowercase();
    if scheme == "wss" || scheme == "https" {
        return Ok(());
    }
    if scheme == "ws" || scheme == "http" {
        if let Some(host) = parsed.host_str() {
            if is_mobile_pairing_cleartext_allowed_host(host) {
                return Ok(());
            }
        }
        return Err(
            "Tailscale and public mobile pairing require a secure gateway URL (wss://) or Tailscale Serve/Funnel. ws:// is only valid for localhost, private LAN addresses, .local hosts, or the Android emulator.".to_string()
        );
    }
    Err("Invalid mobile pairing URL protocol; expected wss:// or ws://".to_string())
}

#[cfg(test)]
mod tests {
    use super::{DevicePairingOwner, PENDING_PAIRING_TTL_MS};
    use rusqlite::params;
    use serde_json::json;

    #[test]
    fn expired_pending_request_is_rejected_before_status_or_approval() {
        let owner = DevicePairingOwner::in_memory().expect("pairing owner");
        let expired_at = super::now_ms() - PENDING_PAIRING_TTL_MS - 1;
        {
            let conn = owner.conn.lock().expect("pairing database lock");
            conn.execute(
                "INSERT INTO device_pairing_pending
                 (request_id, device_id, public_key, ts, refreshed_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params!["expired-request", "expired-device", "expired-key", expired_at],
            )
            .expect("insert expired pending request");
        }

        let status = owner
            .handle_pair_status(&json!({ "requestId": "expired-request" }))
            .expect("status lookup");
        assert_eq!(status["status"], "rejected");

        let approval = owner.handle_pair_approve(&json!({
            "requestId": "expired-request"
        }));
        assert!(approval.is_err());
    }

    #[test]
    fn token_rotation_clears_retained_node_surfaces() {
        let owner = DevicePairingOwner::in_memory().expect("pairing owner");
        let request = owner
            .handle_pair_request(&json!({
                "deviceId": "rotation-surface-device",
                "publicKey": "rotation-surface-key"
            }))
            .expect("pair request");
        owner
            .handle_pair_approve(&json!({
                "requestId": request["requestId"]
            }))
            .expect("pair approval");

        {
            let conn = owner.conn.lock().expect("pairing database lock");
            conn.execute(
                "UPDATE device_pairing_paired
                 SET node_surface_json = ?1, pending_node_surface_json = ?2
                 WHERE device_id = ?3",
                params!["{\"generation\":1}", "{\"generation\":2}", "rotation-surface-device"],
            )
            .expect("seed retained node surfaces");
        }

        owner
            .handle_pair_rotate_token(&json!({
                "deviceId": "rotation-surface-device"
            }))
            .expect("token rotation");

        let conn = owner.conn.lock().expect("pairing database lock");
        let surfaces: (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT node_surface_json, pending_node_surface_json
                 FROM device_pairing_paired WHERE device_id = ?1",
                params!["rotation-surface-device"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read rotated pairing");
        assert_eq!(surfaces, (None, None));
    }
}
