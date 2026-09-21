use mahayana_core::capability::{CapabilityAuditRecord, ComputerControlLease};
use mahayana_core::{AskUserRequest, ExecutionRun, HandoffIntent, LogicalTurn, RunId, TurnId, TurnState};
use serde_json::Value;
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{params, Connection, OptionalExtension};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;

/// Local-first durable store for Agent runtime state.
///
/// Desktop uses SQLite WAL so renderer lifecycle, cloud availability and app
/// restarts cannot become the source of truth for turns, runs or permissions.
/// Web/WASM builds keep the same API but intentionally do not open SQLite.
pub struct RuntimeStore {
    #[cfg(not(target_arch = "wasm32"))]
    connection: Option<Mutex<Connection>>,
}

impl RuntimeStore {
    pub fn open(data_dir: Option<&Path>) -> Result<Self, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = data_dir;
            Ok(Self {})
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(data_dir) = data_dir else {
                return Ok(Self { connection: None });
            };
            std::fs::create_dir_all(data_dir)
                .map_err(|error| RuntimeStoreError::Io(error.to_string()))?;
            let path = data_dir.join("runtime.sqlite3");
            let connection =
                Connection::open(path).map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            connection
                .execute_batch(
                    r#"
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA busy_timeout=5000;
                    PRAGMA foreign_keys=ON;

                    CREATE TABLE IF NOT EXISTS conversations (
                        conversation_id TEXT PRIMARY KEY,
                        updated_at_ms INTEGER NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS turns (
                        turn_id TEXT PRIMARY KEY,
                        conversation_id TEXT NOT NULL,
                        user_message_id TEXT,
                        created_at_ms INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        active_run_id TEXT,
                        FOREIGN KEY(conversation_id) REFERENCES conversations(conversation_id)
                    );
                    CREATE INDEX IF NOT EXISTS turns_conversation_created_idx
                    ON turns(conversation_id, created_at_ms);

                    CREATE TABLE IF NOT EXISTS runs (
                        run_id TEXT PRIMARY KEY,
                        turn_id TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        provider TEXT NOT NULL,
                        started_at_ms INTEGER NOT NULL,
                        finished_at_ms INTEGER,
                        state TEXT NOT NULL,
                        FOREIGN KEY(turn_id) REFERENCES turns(turn_id)
                    );
                    CREATE INDEX IF NOT EXISTS runs_turn_generation_idx
                    ON runs(turn_id, generation);

                    CREATE TABLE IF NOT EXISTS activities (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        run_id TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        title TEXT NOT NULL,
                        detail TEXT,
                        state TEXT NOT NULL,
                        created_at_ms INTEGER NOT NULL,
                        FOREIGN KEY(run_id) REFERENCES runs(run_id)
                    );

                    CREATE TABLE IF NOT EXISTS permissions (
                        permission_key TEXT PRIMARY KEY,
                        decision TEXT NOT NULL,
                        scope_json TEXT NOT NULL DEFAULT '{}',
                        updated_at_ms INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS ui_state (
                        key TEXT PRIMARY KEY,
                        value_json TEXT NOT NULL,
                        updated_at_ms INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS pending_intents (
                        intent_id TEXT PRIMARY KEY,
                        turn_id TEXT NOT NULL,
                        run_id TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        payload_json TEXT NOT NULL,
                        created_at_ms INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS sync_journal (
                        sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                        object_kind TEXT NOT NULL,
                        object_id TEXT NOT NULL,
                        mutation_json TEXT NOT NULL,
                        created_at_ms INTEGER NOT NULL,
                        acknowledged_at_ms INTEGER
                    );

                    CREATE TABLE IF NOT EXISTS capability_audit (
                        sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                        actor TEXT NOT NULL,
                        agent_id TEXT,
                        conversation_id TEXT NOT NULL,
                        run_id TEXT,
                        capability TEXT NOT NULL,
                        intent TEXT NOT NULL,
                        decision TEXT NOT NULL,
                        reason TEXT,
                        record_json TEXT NOT NULL,
                        decided_at_ms INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS computer_leases (
                        device_id TEXT PRIMARY KEY,
                        agent_id TEXT NOT NULL,
                        run_id TEXT NOT NULL,
                        mode TEXT NOT NULL,
                        acquired_at_ms INTEGER NOT NULL,
                        expires_at_ms INTEGER NOT NULL
                    );
                    "#,
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(Self {
                connection: Some(Mutex::new(connection)),
            })
        }
    }

    pub fn record_turn(&self, turn: &LogicalTurn) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = turn;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let connection = connection.lock().map_err(|_| RuntimeStoreError::Poisoned)?;
            connection
                .execute(
                    "INSERT INTO conversations(conversation_id, updated_at_ms) VALUES (?1, ?2)
                     ON CONFLICT(conversation_id) DO UPDATE SET updated_at_ms = excluded.updated_at_ms",
                    params![turn.conversation_id.as_str(), turn.created_at_ms],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            connection
                .execute(
                    "INSERT INTO turns(turn_id, conversation_id, user_message_id, created_at_ms, state, active_run_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(turn_id) DO UPDATE SET
                       state = excluded.state,
                       active_run_id = excluded.active_run_id",
                    params![
                        turn.id.as_str(),
                        turn.conversation_id.as_str(),
                        turn.user_message_id.as_ref().map(|id| id.as_str()),
                        turn.created_at_ms,
                        turn.state.as_str(),
                        turn.active_run_id.as_ref().map(|id| id.as_str()),
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn record_run(&self, run: &ExecutionRun) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = run;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let connection = connection.lock().map_err(|_| RuntimeStoreError::Poisoned)?;
            connection
                .execute(
                    "INSERT INTO runs(run_id, turn_id, generation, provider, started_at_ms, finished_at_ms, state)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(run_id) DO UPDATE SET
                       finished_at_ms = excluded.finished_at_ms,
                       state = excluded.state",
                    params![
                        run.id.as_str(),
                        run.turn_id.as_str(),
                        i64::from(run.generation),
                        run.provider,
                        run.started_at_ms,
                        run.finished_at_ms,
                        run.state.as_str(),
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn set_turn_state(
        &self,
        turn_id: &TurnId,
        state: TurnState,
        active_run_id: Option<&RunId>,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (turn_id, state, active_run_id);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "UPDATE turns SET state = ?2, active_run_id = ?3 WHERE turn_id = ?1",
                    params![turn_id.as_str(), state.as_str(), active_run_id.map(|id| id.as_str())],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn set_run_state(
        &self,
        run_id: &RunId,
        state: TurnState,
        finished_at_ms: Option<i64>,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (run_id, state, finished_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "UPDATE runs SET state = ?2, finished_at_ms = COALESCE(?3, finished_at_ms) WHERE run_id = ?1",
                    params![run_id.as_str(), state.as_str(), finished_at_ms],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn enqueue_ask_user(
        &self,
        request: &AskUserRequest,
        created_at_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (request, created_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let payload = serde_json::to_string(request)
                .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))?;
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                     VALUES (?1, ?2, ?3, 'ask-user', ?4, ?5)
                     ON CONFLICT(intent_id) DO NOTHING",
                    params![
                        request.id.as_str(),
                        request.turn_id.as_str(),
                        request.run_id.as_str(),
                        payload,
                        created_at_ms,
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn enqueue_handoff(
        &self,
        intent: &HandoffIntent,
        turn_id: &TurnId,
        created_at_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (intent, turn_id, created_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let payload = serde_json::to_string(intent)
                .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))?;
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO pending_intents(intent_id, turn_id, run_id, kind, payload_json, created_at_ms)
                     VALUES (?1, ?2, ?3, 'agent-handoff', ?4, ?5)
                     ON CONFLICT(intent_id) DO NOTHING",
                    params![
                        intent.id.as_str(),
                        turn_id.as_str(),
                        intent.origin_run.as_str(),
                        payload,
                        created_at_ms,
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn append_audit(&self, record: &CapabilityAuditRecord) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = record;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let encoded = serde_json::to_string(record)
                .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))?;
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO capability_audit(
                       actor, agent_id, conversation_id, run_id, capability, intent,
                       decision, reason, record_json, decided_at_ms
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        record.request.actor,
                        record.request.agent_id,
                        record.request.conversation_id.as_str(),
                        record.request.run_id.as_ref().map(|id| id.as_str()),
                        record.request.capability,
                        record.request.intent,
                        format!("{:?}", record.decision).to_lowercase(),
                        record.reason,
                        encoded,
                        record.decided_at_ms,
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn acquire_computer_lease(
        &self,
        lease: &ComputerControlLease,
        now_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (lease, now_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let mut connection = connection.lock().map_err(|_| RuntimeStoreError::Poisoned)?;
            let transaction = connection
                .transaction()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            transaction
                .execute("DELETE FROM computer_leases WHERE expires_at_ms <= ?1", params![now_ms])
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            let existing = transaction
                .query_row(
                    "SELECT run_id FROM computer_leases WHERE device_id = ?1",
                    params![lease.device_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            if existing.as_deref().is_some_and(|run_id| run_id != lease.run_id.as_str()) {
                return Err(RuntimeStoreError::ComputerLeaseBusy(lease.device_id.clone()));
            }
            transaction
                .execute(
                    "INSERT INTO computer_leases(device_id, agent_id, run_id, mode, acquired_at_ms, expires_at_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(device_id) DO UPDATE SET
                       agent_id = excluded.agent_id,
                       run_id = excluded.run_id,
                       mode = excluded.mode,
                       acquired_at_ms = excluded.acquired_at_ms,
                       expires_at_ms = excluded.expires_at_ms",
                    params![
                        lease.device_id,
                        lease.agent_id,
                        lease.run_id.as_str(),
                        lease.mode,
                        lease.acquired_at_ms,
                        lease.expires_at_ms,
                    ],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            transaction
                .commit()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))
        }
    }
    /// Read renderer projection state that is owned durably by the Rust runtime.
    /// UI code may keep an in-memory mirror, but SQLite remains authoritative.
    pub fn read_ui_state(&self, key: &str) -> Result<Option<Value>, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = key;
            Ok(None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(None); };
            let encoded: Option<String> = connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .query_row(
                    "SELECT value_json FROM ui_state WHERE key = ?1",
                    params![key],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            encoded
                .map(|value| {
                    serde_json::from_str(&value)
                        .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))
                })
                .transpose()
        }
    }

    pub fn write_ui_state(
        &self,
        key: &str,
        value: &Value,
        updated_at_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (key, value, updated_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let encoded = serde_json::to_string(value)
                .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))?;
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO ui_state(key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)
                     ON CONFLICT(key) DO UPDATE SET
                       value_json = excluded.value_json,
                       updated_at_ms = excluded.updated_at_ms",
                    params![key, encoded, updated_at_ms],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeStoreError {
    #[error("runtime store I/O failed: {0}")]
    Io(String),
    #[error("runtime store SQLite failed: {0}")]
    Sqlite(String),
    #[error("runtime store serialization failed: {0}")]
    Serialization(String),
    #[error("runtime store mutex is poisoned")]
    Poisoned,
    #[error("computer {0} already has an active controller lease")]
    ComputerLeaseBusy(String),
}