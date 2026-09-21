use mahayana_core::capability::{CapabilityAuditRecord, ComputerControlLease};
use mahayana_core::{
    AskUserRequest, ConversationId, ExecutionRun, HandoffIntent, LogicalTurn, MessageId, RunId,
    TurnId, TurnState,
};
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

#[derive(Debug, Clone)]
pub(crate) struct StoredTurnSnapshot {
    pub turn_id: TurnId,
    pub state: TurnState,
    pub last_run_id: Option<RunId>,
    pub generation: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RecoverableHandoff {
    pub origin_turn_id: TurnId,
    pub intent: HandoffIntent,
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

                    CREATE TABLE IF NOT EXISTS workspace_state (
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

                    CREATE TABLE IF NOT EXISTS handoff_dispatch (
                        intent_id TEXT PRIMARY KEY,
                        target_operation_id TEXT,
                        state TEXT NOT NULL,
                        updated_at_ms INTEGER NOT NULL,
                        FOREIGN KEY(intent_id) REFERENCES pending_intents(intent_id)
                    );
                    CREATE INDEX IF NOT EXISTS handoff_dispatch_state_idx
                    ON handoff_dispatch(state, updated_at_ms);

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

    /// Resolve one terminal logical turn to the next execution generation.
    ///
    /// Regenerate/retry is explicit: ordinary duplicate transport requests do
    /// not select an existing turn. The caller supplies the original stable
    /// client user-message id and receives the same turn id plus N+1.
    pub fn retry_turn_generation(
        &self,
        conversation_id: &str,
        user_message_id: &str,
    ) -> Result<Option<(TurnId, i64, u32)>, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (conversation_id, user_message_id);
            Ok(None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(None); };
            let row = connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .query_row(
                    "SELECT t.turn_id, t.created_at_ms, COALESCE(MAX(r.generation), 0)
                     FROM turns t
                     LEFT JOIN runs r ON r.turn_id = t.turn_id
                     WHERE t.conversation_id = ?1
                       AND t.user_message_id = ?2
                       AND t.state IN ('completed', 'failed', 'cancelled', 'recovering')
                     GROUP BY t.turn_id, t.created_at_ms
                     ORDER BY t.created_at_ms DESC
                     LIMIT 1",
                    params![conversation_id, user_message_id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                        ))
                    },
                )
                .optional()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            row.map(|(turn_id, created_at_ms, generation)| {
                let next = generation
                    .checked_add(1)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| RuntimeStoreError::Sqlite("run generation overflow".to_string()))?;
                Ok((TurnId(turn_id), created_at_ms, next))
            })
            .transpose()
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

    pub fn find_turn_by_client_message(
        &self,
        conversation_id: &ConversationId,
        message_id: &MessageId,
    ) -> Result<Option<StoredTurnSnapshot>, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (conversation_id, message_id);
            Ok(None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(None); };
            let row = connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .query_row(
                    "SELECT t.turn_id, t.state, r.run_id, COALESCE(r.generation, 0)
                     FROM turns t
                     LEFT JOIN runs r ON r.turn_id = t.turn_id
                     WHERE t.conversation_id = ?1 AND t.user_message_id = ?2
                     ORDER BY r.generation DESC
                     LIMIT 1",
                    params![conversation_id.as_str(), message_id.as_str()],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    },
                )
                .optional()
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            row.map(|(turn_id, state, run_id, generation)| {
                Ok(StoredTurnSnapshot {
                    turn_id: TurnId(turn_id),
                    state: parse_turn_state(&state)?,
                    last_run_id: run_id.map(RunId),
                    generation: generation.max(0).min(i64::from(u32::MAX)) as u32,
                })
            })
            .transpose()
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

    pub fn mark_handoff_started(
        &self,
        intent_id: &str,
        operation_id: &str,
        updated_at_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (intent_id, operation_id, updated_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO handoff_dispatch(intent_id, target_operation_id, state, updated_at_ms)
                     VALUES (?1, ?2, 'running', ?3)
                     ON CONFLICT(intent_id) DO UPDATE SET
                       target_operation_id = CASE
                         WHEN handoff_dispatch.state IN ('completed', 'failed')
                           THEN handoff_dispatch.target_operation_id
                         ELSE excluded.target_operation_id
                       END,
                       state = CASE
                         WHEN handoff_dispatch.state IN ('completed', 'failed')
                           THEN handoff_dispatch.state
                         ELSE 'running'
                       END,
                       updated_at_ms = excluded.updated_at_ms",
                    params![intent_id, operation_id, updated_at_ms],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn mark_handoff_terminal(
        &self,
        intent_id: &str,
        completed: bool,
        updated_at_ms: i64,
    ) -> Result<(), RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (intent_id, completed, updated_at_ms);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(()); };
            let state = if completed { "completed" } else { "failed" };
            connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .execute(
                    "INSERT INTO handoff_dispatch(intent_id, target_operation_id, state, updated_at_ms)
                     SELECT ?1, NULL, ?2, ?3
                     WHERE EXISTS (
                       SELECT 1 FROM pending_intents
                       WHERE intent_id = ?1 AND kind = 'agent-handoff'
                     )
                     ON CONFLICT(intent_id) DO UPDATE SET
                       state = excluded.state,
                       updated_at_ms = excluded.updated_at_ms",
                    params![intent_id, state, updated_at_ms],
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(())
        }
    }

    pub fn recoverable_handoffs(&self) -> Result<Vec<RecoverableHandoff>, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Vec::new())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(Vec::new()); };
            let connection = connection.lock().map_err(|_| RuntimeStoreError::Poisoned)?;
            let mut statement = connection
                .prepare(
                    "SELECT p.turn_id, p.payload_json
                     FROM pending_intents p
                     LEFT JOIN handoff_dispatch d ON d.intent_id = p.intent_id
                     WHERE p.kind = 'agent-handoff'
                       AND (d.state IS NULL OR d.state IN ('queued', 'running'))
                     ORDER BY p.created_at_ms ASC",
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            let mut recovered = Vec::new();
            for row in rows {
                let (turn_id, payload) =
                    row.map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
                let intent = serde_json::from_str::<HandoffIntent>(&payload)
                    .map_err(|error| RuntimeStoreError::Serialization(error.to_string()))?;
                recovered.push(RecoverableHandoff {
                    origin_turn_id: TurnId(turn_id),
                    intent,
                });
            }
            Ok(recovered)
        }
    }

    pub fn count_handoffs_for_run(&self, run_id: &RunId) -> Result<u64, RuntimeStoreError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = run_id;
            Ok(0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(connection) = &self.connection else { return Ok(0); };
            let count = connection
                .lock()
                .map_err(|_| RuntimeStoreError::Poisoned)?
                .query_row(
                    "SELECT COUNT(*) FROM pending_intents WHERE kind = 'agent-handoff' AND run_id = ?1",
                    params![run_id.as_str()],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| RuntimeStoreError::Sqlite(error.to_string()))?;
            Ok(count.max(0) as u64)
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
    pub fn read_workspace_state(&self, key: &str) -> Result<Option<Value>, RuntimeStoreError> {
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
                    "SELECT value_json FROM workspace_state WHERE key = ?1",
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

    pub fn write_workspace_state(
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
                    "INSERT INTO workspace_state(key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)
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

fn parse_turn_state(value: &str) -> Result<TurnState, RuntimeStoreError> {
    match value {
        "accepted" => Ok(TurnState::Accepted),
        "queued" => Ok(TurnState::Queued),
        "preparing" => Ok(TurnState::Preparing),
        "thinking" => Ok(TurnState::Thinking),
        "tool-running" => Ok(TurnState::ToolRunning),
        "streaming" => Ok(TurnState::Streaming),
        "waiting-user" => Ok(TurnState::WaitingUser),
        "completed" => Ok(TurnState::Completed),
        "failed" => Ok(TurnState::Failed),
        "cancelled" => Ok(TurnState::Cancelled),
        "recovering" => Ok(TurnState::Recovering),
        other => Err(RuntimeStoreError::Serialization(format!(
            "unknown turn state: {other}"
        ))),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use mahayana_core::IntentId;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_store() -> (std::path::PathBuf, RuntimeStore) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "mahayana-runtime-store-{}-{nonce}",
            std::process::id()
        ));
        let store = RuntimeStore::open(Some(&path)).expect("open runtime store");
        (path, store)
    }

    #[test]
    fn handoff_dispatch_recovers_running_but_not_terminal_work() {
        let (path, store) = temp_store();
        let intent = HandoffIntent {
            id: IntentId("handoff:test".to_string()),
            target_agent: "research".to_string(),
            target_conversation_id: Some(ConversationId(
                "codex:agent:research".to_string(),
            )),
            inference_provider: Some("codex".to_string()),
            task: "summarize".to_string(),
            constraints: Value::Null,
            expected_output: None,
            origin_run: RunId("run:origin".to_string()),
            depth: 1,
        };
        let turn_id = TurnId("turn:origin".to_string());
        store.enqueue_handoff(&intent, &turn_id, 1).expect("enqueue");
        assert_eq!(store.recoverable_handoffs().expect("recover").len(), 1);

        store
            .mark_handoff_started(intent.id.as_str(), "run:target", 2)
            .expect("mark running");
        assert_eq!(store.recoverable_handoffs().expect("recover").len(), 1);

        store
            .mark_handoff_terminal(intent.id.as_str(), true, 3)
            .expect("mark complete");
        assert!(store.recoverable_handoffs().expect("recover").is_empty());

        drop(store);
        let _ = std::fs::remove_dir_all(path);
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