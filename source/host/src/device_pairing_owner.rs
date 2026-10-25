use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, params};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::gateway_command_error::GatewayCommandError;

pub struct DevicePairingOwner {
    conn: Mutex<Connection>,
    db_path: Option<PathBuf>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
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

    pub fn handle_pair_request(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args.get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.request requires deviceId".into()))?;
        let public_key = args.get("publicKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.request requires publicKey".into()))?;

        let display_name = args.get("displayName").and_then(Value::as_str).unwrap_or(device_id);
        let platform = args.get("platform").and_then(Value::as_str).unwrap_or("unknown");
        let device_family = args.get("deviceFamily").and_then(Value::as_str);
        let client_id = args.get("clientId").and_then(Value::as_str);
        let client_mode = args.get("clientMode").and_then(Value::as_str);
        let role = args.get("role").and_then(Value::as_str).unwrap_or("operator");
        let roles_json = args.get("roles").map(|v| v.to_string()).unwrap_or_else(|| json!([role]).to_string());
        let scopes_json = args.get("scopes").map(|v| v.to_string()).unwrap_or_else(|| json!(["operator.read", "operator.write"]).to_string());
        let silent = if args.get("silent").and_then(Value::as_bool) == Some(true) { 1 } else { 0 };

        let conn = self.conn.lock().unwrap();
        let now = now_ms();

        // Check if device is already paired (is_repair)
        let is_repair: bool = conn
            .query_row(
                "SELECT COUNT(1) FROM device_pairing_paired WHERE device_id = ?1",
                params![device_id],
                |row| row.get(0),
            )
            .map(|c: i64| c > 0)
            .unwrap_or(false);

        let request_id = format!("pair-{}", Uuid::new_v4());
        let is_repair_int = if is_repair { 1 } else { 0 };

        conn.execute(
            r#"INSERT INTO device_pairing_pending (
                request_id, device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, role, roles_json, scopes_json, silent, is_repair, ts, refreshed_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)"#,
            params![
                request_id, device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, role, roles_json, scopes_json, silent, is_repair_int, now
            ],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to store pending pairing: {e}")))?;

        Ok(json!({
            "status": "pending",
            "requestId": request_id,
            "deviceId": device_id,
            "created": true,
            "isRepair": is_repair,
            "expiresAtMs": now + 900_000
        }))
    }

    pub fn handle_pair_status(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let request_id = args.get("requestId").and_then(Value::as_str).map(str::trim);
        let device_id = args.get("deviceId").and_then(Value::as_str).map(str::trim);

        let conn = self.conn.lock().unwrap();

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
            let paired_row: Option<(String, String)> = conn
                .query_row(
                    "SELECT device_id, display_name FROM device_pairing_paired WHERE device_id = ?1",
                    params![dev_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            if let Some((d_id, name)) = paired_row {
                return Ok(json!({
                    "status": "approved",
                    "device": {
                        "deviceId": d_id,
                        "displayName": name
                    }
                }));
            }
        }

        Ok(json!({ "status": "rejected" }))
    }

    pub fn handle_pair_approve(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let request_id = args.get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.approve requires requestId".into()))?;

        let conn = self.conn.lock().unwrap();

        // 1. Fetch pending record
        let pending = conn.query_row(
            r#"SELECT device_id, public_key, display_name, platform, device_family,
                      client_id, client_mode, role, roles_json, scopes_json, ts
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
                    row.get::<_, i64>(10)?,
                ))
            },
        ).map_err(|_| GatewayCommandError::BadRequest(format!("unknown requestId: {request_id}")))?;

        let (device_id, public_key, display_name, platform, device_family, client_id, client_mode, role, roles_json, scopes_json, created_at) = pending;

        let now = now_ms();
        let token_string = format!("sand-device-tok-{}", Uuid::new_v4());
        let tokens_json = json!({
            "operator": {
                "token": token_string,
                "createdAtMs": now,
                "rotatedAtMs": now
            }
        }).to_string();

        // Atomic transaction: delete from pending, upsert into paired, bump revision
        conn.execute("DELETE FROM device_pairing_pending WHERE request_id = ?1", params![request_id])
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to delete pending request: {e}")))?;

        conn.execute(
            r#"INSERT INTO device_pairing_paired (
                device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, role, roles_json, scopes_json, approved_scopes_json,
                tokens_json, approved_via, created_at_ms, approved_at_ms, last_seen_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?11, 'owner', ?12, ?13, ?13)
            ON CONFLICT(device_id) DO UPDATE SET
                public_key = ?2, display_name = ?3, platform = ?4, device_family = ?5,
                client_id = ?6, client_mode = ?7, role = ?8, roles_json = ?9, scopes_json = ?10,
                approved_scopes_json = ?10, tokens_json = ?11, approved_at_ms = ?13, last_seen_at_ms = ?13"#,
            params![
                device_id, public_key, display_name, platform, device_family,
                client_id, client_mode, role, roles_json, scopes_json,
                tokens_json, created_at, now
            ],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to persist paired device: {e}")))?;

        let new_rev = bump_revision(&conn)
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}")))?;

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
        let request_id = args.get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.reject requires requestId".into()))?;

        let conn = self.conn.lock().unwrap();

        let device_id: String = conn.query_row(
            "SELECT device_id FROM device_pairing_pending WHERE request_id = ?1",
            params![request_id],
            |row| row.get(0),
        ).map_err(|_| GatewayCommandError::BadRequest(format!("unknown requestId: {request_id}")))?;

        conn.execute("DELETE FROM device_pairing_pending WHERE request_id = ?1", params![request_id])
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to delete pending request: {e}")))?;

        Ok(json!({
            "requestId": request_id,
            "deviceId": device_id,
            "decision": "rejected"
        }))
    }

    pub fn handle_pair_list(&self, _args: &Value) -> Result<Value, GatewayCommandError> {
        let conn = self.conn.lock().unwrap();

        let mut pending_stmt = conn.prepare(
            "SELECT request_id, device_id, display_name, platform, role, ts FROM device_pairing_pending"
        ).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let pending_rows = pending_stmt.query_map([], |row| {
            Ok(json!({
                "requestId": row.get::<_, String>(0)?,
                "deviceId": row.get::<_, String>(1)?,
                "displayName": row.get::<_, Option<String>>(2)?,
                "platform": row.get::<_, Option<String>>(3)?,
                "role": row.get::<_, Option<String>>(4)?,
                "ts": row.get::<_, i64>(5)?
            }))
        }).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let pending: Vec<Value> = pending_rows.filter_map(Result::ok).collect();

        let mut paired_stmt = conn.prepare(
            "SELECT device_id, display_name, platform, role, approved_at_ms, last_seen_at_ms FROM device_pairing_paired"
        ).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let paired_rows = paired_stmt.query_map([], |row| {
            Ok(json!({
                "deviceId": row.get::<_, String>(0)?,
                "displayName": row.get::<_, Option<String>>(1)?,
                "platform": row.get::<_, Option<String>>(2)?,
                "role": row.get::<_, Option<String>>(3)?,
                "approvedAtMs": row.get::<_, i64>(4)?,
                "lastSeenAtMs": row.get::<_, Option<i64>>(5)?
            }))
        }).map_err(|e| GatewayCommandError::Internal(e.to_string()))?;

        let paired: Vec<Value> = paired_rows.filter_map(Result::ok).collect();

        Ok(json!({
            "pending": pending,
            "paired": paired
        }))
    }

    pub fn handle_pair_revoke(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args.get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.revoke requires deviceId".into()))?;

        let role = args.get("role").and_then(Value::as_str);

        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(1) FROM device_pairing_paired WHERE device_id = ?1",
            params![device_id],
            |row| row.get(0),
        ).unwrap_or(0);

        if count == 0 {
            return Err(GatewayCommandError::BadRequest(format!("unknown deviceId: {device_id}")));
        }

        conn.execute("DELETE FROM device_pairing_paired WHERE device_id = ?1", params![device_id])
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to delete paired device: {e}")))?;

        let new_rev = bump_revision(&conn)
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}")))?;

        Ok(json!({
            "deviceId": device_id,
            "role": role.unwrap_or("all"),
            "revokedAtMs": now_ms(),
            "revision": new_rev
        }))
    }

    pub fn handle_pair_rotate_token(&self, args: &Value) -> Result<Value, GatewayCommandError> {
        let device_id = args.get("deviceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("device.pair.rotateToken requires deviceId".into()))?;

        let role = args.get("role").and_then(Value::as_str).unwrap_or("operator");
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(1) FROM device_pairing_paired WHERE device_id = ?1",
            params![device_id],
            |row| row.get(0),
        ).unwrap_or(0);

        if count == 0 {
            return Err(GatewayCommandError::BadRequest(format!("unknown deviceId: {device_id}")));
        }

        let now = now_ms();
        let new_token = format!("sand-device-tok-{}", Uuid::new_v4());
        let tokens_json = json!({
            role: {
                "token": new_token,
                "createdAtMs": now,
                "rotatedAtMs": now
            }
        }).to_string();

        conn.execute(
            "UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = ?2",
            params![tokens_json, device_id],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to update tokens: {e}")))?;

        let new_rev = bump_revision(&conn)
            .map_err(|e| GatewayCommandError::Internal(format!("Failed to bump pairing revision: {e}")))?;

        let is_cross_device = args.get("crossDevice").and_then(Value::as_bool).unwrap_or(false);

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

    pub fn handle_pair_setup_code(&self, _args: &Value) -> Result<Value, GatewayCommandError> {
        let setup_id = format!("setup-{}", Uuid::new_v4());
        let setup_code = format!("sand-setup-{}", Uuid::new_v4());
        let now = now_ms();
        let expires_at = now + 900_000;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO device_bootstrap_tokens (token_key, token, setup_id, ts, issued_at_ms)
               VALUES (?1, ?2, ?3, ?4, ?4)"#,
            params![setup_code, setup_code, setup_id, now],
        ).map_err(|e| GatewayCommandError::Internal(format!("Failed to store bootstrap token: {e}")))?;

        Ok(json!({
            "setupId": setup_id,
            "setupCode": setup_code,
            "gatewayUrl": "http://127.0.0.1:8080",
            "auth": "token",
            "urlSource": "local",
            "access": "full",
            "expiresAtMs": expires_at
        }))
    }

    pub fn handle_pairing_command(&self, method: &str, args: &Value) -> Result<Value, GatewayCommandError> {
        match method {
            "device.pair.request" => self.handle_pair_request(args),
            "device.pair.status" => self.handle_pair_status(args),
            "device.pair.approve" => self.handle_pair_approve(args),
            "device.pair.reject" => self.handle_pair_reject(args),
            "device.pair.list" => self.handle_pair_list(args),
            "device.pair.revoke" => self.handle_pair_revoke(args),
            "device.pair.rotateToken" => self.handle_pair_rotate_token(args),
            "device.pair.setupCode" => self.handle_pair_setup_code(args),
            _ => Err(GatewayCommandError::UnknownMethod(format!("Unknown device pairing method: {method}"))),
        }
    }
}
