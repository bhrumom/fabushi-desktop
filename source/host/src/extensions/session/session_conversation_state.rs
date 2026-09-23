use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde_json::Value;

use crate::agent_isolation::{AgentWorkerPool, ProductionAgentStoreWorkerBackend};
use crate::storage::store_db::live_db_handle_count;

use super::agent_db::{
    read_persisted_latest_root_blob_id,
};
use super::agent_db_recovery::{AgentDbRecoveryError, DbRecoveryOptions, open_configured_db};
use super::agent_db_schema::{
    GET_TRANSCRIPT_ENTRY_SQL, LIST_BRANCHED_ENTRIES_SQL, LIST_TRANSCRIPT_ENTRIES_SQL,
};
use super::agent_db_serde::parse_transcript_entry;
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
    read_transcript_page, read_transcript_tail, read_transcript_window,
};
use super::conversation_recovery::{
    ConversationStructureRefs, OutlineItem as RecoveryOutlineItem,
    conversation_structure_fully_resolves, parse_conversation_state_structure,
};

#[derive(Debug, thiserror::Error)]
pub enum SessionConversationStateError {
    #[error("session conversation-state database error: {0}")]
    Database(#[from] AgentDbRecoveryError),
    #[error("session conversation-state sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("session conversation-state blob error: {0}")]
    Blob(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptThread {
    pub entries: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum ConversationOutlineItem {
    #[serde(rename = "user")]
    User {
        id: String,
        text: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        hidden: bool,
    },
    #[serde(rename = "assistant-text")]
    AssistantText {
        id: String,
        text: String,
    },
    #[serde(rename = "thinking")]
    Thinking {
        id: String,
        text: String,
        #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    #[serde(rename = "send-message")]
    SendMessage {
        id: String,
        message: Value,
    },
    #[serde(rename = "tool-call")]
    ToolCall {
        id: String,
        name: String,
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
    },
}

impl ConversationOutlineItem {
    fn to_recovery_item(&self) -> Option<RecoveryOutlineItem> {
        match self {
            Self::User { id, text, hidden } => Some(RecoveryOutlineItem::User {
                id: id.clone(),
                hidden: *hidden,
                text: text.clone(),
                timestamp_ms: None,
            }),
            Self::SendMessage { id, message } => Some(RecoveryOutlineItem::SendMessage {
                id: id.clone(),
                message: message.clone(),
                timestamp_ms: None,
            }),
            Self::ToolCall { id, name, status, summary } => Some(RecoveryOutlineItem::ToolCall {
                id: id.clone(),
                name: name.clone(),
                status: status.clone(),
                summary: summary.clone(),
                timestamp_ms: None,
            }),
            Self::AssistantText { .. } | Self::Thinking { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationOutlineTurn {
    pub raw_user_text: String,
    pub user_message_id: String,
    pub items: Vec<ConversationOutlineItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationTodoItem {
    pub id: String,
    pub content: String,
    pub status: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConversationState {
    pub turns: Vec<ConversationOutlineTurn>,
    pub todos: Vec<ConversationTodoItem>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionConversationState {
    busy_timeout_ms: u64,
}

impl SessionConversationState {
    pub fn new(busy_timeout_ms: u64) -> Self {
        Self { busy_timeout_ms }
    }

    fn open_read_db(&self, db_path: &Path) -> Result<Connection, SessionConversationStateError> {
        let agent_id = db_path
            .parent()
            .and_then(Path::file_name)
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let options = DbRecoveryOptions {
            recover_on_corruption: false,
            busy_timeout_ms: self.busy_timeout_ms,
            ..DbRecoveryOptions::default()
        };
        Ok(open_configured_db(
            db_path,
            &agent_id,
            &options,
            live_db_handle_count(db_path) > 0,
        )?)
    }

    pub fn read_agent_transcript_entries(
        &self,
        db_path: &Path,
    ) -> Result<Vec<Value>, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        read_entries(&db)
    }

    pub fn read_agent_transcript_page(
        &self,
        db_path: &Path,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        Ok(read_transcript_page(&db, query)?)
    }

    pub fn read_agent_transcript_window(
        &self,
        db_path: &Path,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<BTreeMap<String, usize>>, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        let branched = read_branched_entries(&db)?;
        let counts = branch_reply_counts(&branched);
        Ok(read_transcript_window(&db, query, |entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let id = entry.get("id").and_then(Value::as_str)?;
                    counts.get(id).copied().map(|count| (id.to_string(), count))
                })
                .collect()
        })?)
    }

    pub fn read_agent_transcript_tail(
        &self,
        db_path: &Path,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        Ok(read_transcript_tail(&db, query)?)
    }

    pub fn read_agent_thread(
        &self,
        db_path: &Path,
        root_id: &str,
    ) -> Result<TranscriptThread, SessionConversationStateError> {
        let db = self.open_read_db(db_path)?;
        let root_raw = db
            .query_row(GET_TRANSCRIPT_ENTRY_SQL, params![root_id], |row| row.get::<_, String>(0))
            .optional()?;
        let root = root_raw.as_deref().and_then(parse_transcript_entry);
        let branched = read_branched_entries(&db)?;
        let mut entries = Vec::new();
        if let Some(root) = root {
            entries.push(root);
        }
        entries.extend(thread_descendants(root_id, &branched));
        Ok(TranscriptThread { entries })
    }
    pub fn read_agent_outline(
        &self,
        pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
        agent_id: &str,
        db_path: &Path,
        blob_db_path: &Path,
    ) -> Result<Vec<ConversationOutlineItem>, SessionConversationStateError> {
        Ok(self
            .read_agent_outline_turns(pool, agent_id, db_path, blob_db_path)?
            .into_iter()
            .flat_map(|turn| turn.items)
            .collect())
    }

    pub fn read_agent_conversation_state(
        &self,
        pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
        agent_id: &str,
        db_path: &Path,
        blob_db_path: &Path,
    ) -> Result<Option<ResolvedConversationState>, SessionConversationStateError> {
        let root_id = read_persisted_latest_root_blob_id(db_path, self.busy_timeout_ms)
            .map_err(|error| SessionConversationStateError::Blob(error.to_string()))?;
        if root_id.is_empty() {
            return Ok(None);
        }
        let root_blob = block_on_blob(
            Arc::clone(&pool),
            agent_id,
            blob_db_path,
            db_path,
            &root_id,
        )?
        .ok_or_else(|| SessionConversationStateError::Blob(
            "latest conversation root blob is missing".into(),
        ))?;
        let structure = parse_conversation_state_structure(&root_blob)
            .ok_or_else(|| SessionConversationStateError::Blob(
                "latest conversation root blob is not a ConversationStateStructure".into(),
            ))?;
        let refs = structure.refs();
        let resolves = futures::executor::block_on(conversation_structure_fully_resolves(
            &refs,
            |blob_id| {
                let pool = Arc::clone(&pool);
                let agent_id = agent_id.to_string();
                let blob_db_path = blob_db_path.to_path_buf();
                let db_path = db_path.to_path_buf();
                async move {
                    pool.get_blob(
                        &agent_id,
                        &blob_db_path,
                        &blob_id,
                        Some(&db_path),
                    )
                    .await
                    .map_err(|error| error.to_string())
                }
            },
        ));
        if !resolves {
            return Ok(None);
        }

        let mut turns = Vec::with_capacity(structure.turns.len());
        for (turn_index, turn_id) in structure.turns.iter().enumerate() {
            let Some(turn_blob) = block_on_blob(
                Arc::clone(&pool),
                agent_id,
                blob_db_path,
                db_path,
                turn_id,
            )? else {
                return Ok(None);
            };
            let Some(turn) = decode_outline_turn(
                &pool,
                agent_id,
                db_path,
                blob_db_path,
                turn_index,
                &turn_blob,
            )? else {
                return Ok(None);
            };
            turns.push(turn);
        }

        let mut todos = Vec::with_capacity(structure.todos.len());
        for todo_id in &structure.todos {
            let Some(todo_blob) = block_on_blob(
                Arc::clone(&pool),
                agent_id,
                blob_db_path,
                db_path,
                todo_id,
            )? else {
                return Ok(None);
            };
            let Some(todo) = decode_todo_item(&todo_blob) else {
                return Ok(None);
            };
            todos.push(todo);
        }

        let summary = match structure.summary.as_deref() {
            Some(summary_id) if !summary_id.is_empty() => {
                let Some(summary_blob) = block_on_blob(
                    Arc::clone(&pool),
                    agent_id,
                    blob_db_path,
                    db_path,
                    summary_id,
                )? else {
                    return Ok(None);
                };
                Some(first_string_field(&summary_blob, 1).unwrap_or_default())
            }
            _ => None,
        };

        Ok(Some(ResolvedConversationState {
            turns,
            todos,
            summary,
        }))
    }

    pub fn read_agent_outline_turns(
        &self,
        pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
        agent_id: &str,
        db_path: &Path,
        blob_db_path: &Path,
    ) -> Result<Vec<ConversationOutlineTurn>, SessionConversationStateError> {
        Ok(self
            .read_agent_conversation_state(pool, agent_id, db_path, blob_db_path)?
            .map(|state| state.turns)
            .unwrap_or_default())
    }

    pub fn read_agent_recovery_outline_turns(
        &self,
        pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
        agent_id: &str,
        db_path: &Path,
        blob_db_path: &Path,
    ) -> Result<Vec<Vec<RecoveryOutlineItem>>, SessionConversationStateError> {
        Ok(self
            .read_agent_outline_turns(pool, agent_id, db_path, blob_db_path)?
            .into_iter()
            .map(|turn| {
                turn.items
                    .iter()
                    .filter_map(ConversationOutlineItem::to_recovery_item)
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}

fn read_entries(db: &Connection) -> Result<Vec<Value>, SessionConversationStateError> {
    let mut statement = db.prepare(LIST_TRANSCRIPT_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        if let Some(entry) = parse_transcript_entry(&row?) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn read_branched_entries(db: &Connection) -> Result<Vec<Value>, SessionConversationStateError> {
    let mut statement = db.prepare(LIST_BRANCHED_ENTRIES_SQL)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut entries = Vec::new();
    for row in rows {
        if let Some(entry) = parse_transcript_entry(&row?) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn branch_reply_counts(branched: &[Value]) -> HashMap<String, usize> {
    let by_id = branched
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("id")?.as_str()?.to_string(),
                entry,
            ))
        })
        .collect::<HashMap<_, _>>();
    let mut counts = HashMap::new();
    for entry in branched {
        if let Some(root) = resolve_branch_root(entry, &by_id) {
            *counts.entry(root).or_insert(0) += 1;
        }
    }
    counts
}

fn thread_descendants(root_id: &str, branched: &[Value]) -> Vec<Value> {
    let by_id = branched
        .iter()
        .filter_map(|entry| {
            Some((
                entry.get("id")?.as_str()?.to_string(),
                entry,
            ))
        })
        .collect::<HashMap<_, _>>();
    branched
        .iter()
        .filter(|entry| resolve_branch_root(entry, &by_id).as_deref() == Some(root_id))
        .cloned()
        .collect()
}

fn resolve_branch_root(
    entry: &Value,
    branched_by_id: &HashMap<String, &Value>,
) -> Option<String> {
    let mut current = entry;
    let mut seen = std::collections::HashSet::new();
    if let Some(id) = current.get("id").and_then(Value::as_str) {
        seen.insert(id.to_string());
    }
    loop {
        let parent_id = current.get("replyTo").and_then(Value::as_str)?;
        let Some(parent) = branched_by_id.get(parent_id) else {
            return Some(parent_id.to_string());
        };
        if !seen.insert(parent_id.to_string()) {
            return None;
        }
        current = parent;
    }
}


const SAND_HIDDEN_PROMPT_MARKER: &str = "[SAND_HIDDEN_PROMPT]";
const SAND_TRUSTED_AUTOMATION_PROMPT_MARKER: &str = "[SAND_TRUSTED_AUTOMATION_PROMPT]";

fn strip_hidden_marker(text: &str) -> String {
    let without_hidden = text
        .strip_prefix(SAND_HIDDEN_PROMPT_MARKER)
        .unwrap_or(text);
    without_hidden
        .strip_prefix(SAND_TRUSTED_AUTOMATION_PROMPT_MARKER)
        .unwrap_or(without_hidden)
        .to_string()
}

fn block_on_blob(
    pool: Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    blob_db_path: &Path,
    db_path: &Path,
    blob_id: &[u8],
) -> Result<Option<Vec<u8>>, SessionConversationStateError> {
    futures::executor::block_on(pool.get_blob(
        agent_id,
        blob_db_path,
        blob_id,
        Some(db_path),
    ))
    .map_err(|error| SessionConversationStateError::Blob(error.to_string()))
}

fn decode_todo_item(data: &[u8]) -> Option<ConversationTodoItem> {
    Some(ConversationTodoItem {
        id: first_string_field(data, 1).unwrap_or_default(),
        content: first_string_field(data, 2).unwrap_or_default(),
        status: first_varint_field(data, 3).unwrap_or_default(),
        created_at: first_varint_field(data, 4).unwrap_or_default() as i64,
        updated_at: first_varint_field(data, 5).unwrap_or_default() as i64,
        dependencies: all_string_fields(data, 6),
    })
}

fn decode_outline_turn(
    pool: &Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    turn_index: usize,
    data: &[u8],
) -> Result<Option<ConversationOutlineTurn>, SessionConversationStateError> {
    let Some((field, payload)) = last_oneof_bytes(data, &[1, 2]) else {
        return Ok(None);
    };
    if field == 1 {
        decode_agent_outline_turn(
            pool,
            agent_id,
            db_path,
            blob_db_path,
            turn_index,
            payload,
        )
    } else {
        decode_shell_outline_turn(
            pool,
            agent_id,
            db_path,
            blob_db_path,
            turn_index,
            payload,
        )
    }
}

fn decode_agent_outline_turn(
    pool: &Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    turn_index: usize,
    data: &[u8],
) -> Result<Option<ConversationOutlineTurn>, SessionConversationStateError> {
    let Some(user_message_id) = first_bytes_field(data, 1) else {
        return Ok(None);
    };
    let Some(user_blob) = block_on_blob(
        Arc::clone(pool),
        agent_id,
        blob_db_path,
        db_path,
        user_message_id,
    )? else {
        return Ok(None);
    };
    let raw_user_text = first_string_field(&user_blob, 1).unwrap_or_default();
    let user_message_id = first_string_field(&user_blob, 2).unwrap_or_default();
    let hidden = raw_user_text.starts_with(SAND_HIDDEN_PROMPT_MARKER);
    let user_text = strip_hidden_marker(&raw_user_text);
    let mut items = Vec::new();
    if !user_text.trim().is_empty() {
        items.push(ConversationOutlineItem::User {
            id: format!("outline-user-{turn_index}"),
            text: user_text,
            hidden,
        });
    }

    for (step_index, step_id) in all_bytes_fields(data, 2).into_iter().enumerate() {
        let Some(step_blob) = block_on_blob(
            Arc::clone(pool),
            agent_id,
            blob_db_path,
            db_path,
            step_id,
        )? else {
            continue;
        };
        if let Some(item) = decode_step(&step_blob, &format!("outline-{turn_index}-{step_index}")) {
            items.push(item);
        }
    }
    Ok(Some(ConversationOutlineTurn {
        raw_user_text,
        user_message_id,
        items,
    }))
}

fn decode_shell_outline_turn(
    pool: &Arc<AgentWorkerPool<ProductionAgentStoreWorkerBackend>>,
    agent_id: &str,
    db_path: &Path,
    blob_db_path: &Path,
    turn_index: usize,
    data: &[u8],
) -> Result<Option<ConversationOutlineTurn>, SessionConversationStateError> {
    let Some(command_id) = first_bytes_field(data, 1) else {
        return Ok(None);
    };
    let Some(output_id) = first_bytes_field(data, 2) else {
        return Ok(None);
    };
    let Some(command_blob) = block_on_blob(
        Arc::clone(pool),
        agent_id,
        blob_db_path,
        db_path,
        command_id,
    )? else {
        return Ok(None);
    };
    if block_on_blob(
        Arc::clone(pool),
        agent_id,
        blob_db_path,
        db_path,
        output_id,
    )?.is_none() {
        return Ok(None);
    }
    let command = first_string_field(&command_blob, 1).unwrap_or_default();
    Ok(Some(ConversationOutlineTurn {
        raw_user_text: String::new(),
        user_message_id: String::new(),
        items: vec![ConversationOutlineItem::ToolCall {
            id: format!("outline-shell-{turn_index}"),
            name: "shellToolCall".into(),
            status: "done".into(),
            summary: (!command.is_empty()).then_some(command),
        }],
    }))
}

fn decode_step(data: &[u8], id: &str) -> Option<ConversationOutlineItem> {
    let (field, payload) = last_oneof_bytes(data, &[1, 2, 3])?;
    match field {
        1 => {
            let text = first_string_field(payload, 1).unwrap_or_default();
            (!text.is_empty()).then(|| ConversationOutlineItem::AssistantText {
                id: id.to_string(),
                text,
            })
        }
        3 => {
            let text = first_string_field(payload, 1).unwrap_or_default();
            if text.is_empty() {
                return None;
            }
            let duration_ms = first_varint_field(payload, 2).filter(|value| *value > 0);
            Some(ConversationOutlineItem::Thinking {
                id: id.to_string(),
                text,
                duration_ms,
            })
        }
        2 => decode_tool_call(payload, id),
        _ => None,
    }
}

fn decode_tool_call(data: &[u8], id: &str) -> Option<ConversationOutlineItem> {
    let (field, payload) = last_tool_oneof(data)?;
    if field == 55 {
        let message = decode_send_message(payload)?;
        return Some(ConversationOutlineItem::SendMessage {
            id: id.to_string(),
            message,
        });
    }

    let mut name = tool_case_name(field)?.to_string();
    let mut status = "done".to_string();
    let mut summary = None;
    if field == 19 {
        name = "Task".into();
        let args = first_bytes_field(payload, 1);
        let result = first_bytes_field(payload, 2);
        if let Some(result) = result {
            if let Some(error_payload) = last_oneof_bytes(result, &[1, 2])
                .filter(|(case, _)| *case == 2)
                .map(|(_, payload)| payload)
            {
                let error = first_string_field(error_payload, 1).unwrap_or_default();
                if !error.is_empty() {
                    summary = Some(error);
                }
                status = "failed".into();
            }
        }
        if summary.is_none() {
            if let Some(args) = args {
                summary = first_string_field(args, 1)
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .or_else(|| {
                        first_string_field(args, 2)
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty())
                    });
            }
        }
    } else if field == 30 && computer_use_is_single_screenshot(payload) {
        name = "Screenshot".into();
    }

    Some(ConversationOutlineItem::ToolCall {
        id: id.to_string(),
        name,
        status,
        summary,
    })
}

fn decode_send_message(data: &[u8]) -> Option<Value> {
    let args = first_bytes_field(data, 1)?;
    let (case, payload) = last_oneof_bytes(args, &[1, 2])?;
    match case {
        1 => {
            let content = first_string_field(payload, 1)?;
            Some(serde_json::json!({"type":"text","content":content}))
        }
        2 => {
            let url = first_string_field(payload, 1)?;
            let alt = first_string_field(payload, 2);
            Some(match alt.filter(|value| !value.is_empty()) {
                Some(alt) => serde_json::json!({"type":"attachment","url":url,"alt":alt}),
                None => serde_json::json!({"type":"attachment","url":url}),
            })
        }
        _ => None,
    }
}

fn computer_use_is_single_screenshot(data: &[u8]) -> bool {
    let Some(args) = first_bytes_field(data, 1) else {
        return false;
    };
    let actions = all_bytes_fields(args, 2);
    actions.len() == 1
        && last_oneof_bytes(actions[0], &[1,2,3,4,5,6,7,8,9,10,11])
            .is_some_and(|(case, _)| case == 10)
}

fn last_tool_oneof(data: &[u8]) -> Option<(u64, &[u8])> {
    let mut last = None;
    let mut position = 0usize;
    while position < data.len() {
        let (field, wire, value) = read_wire_field(data, &mut position)?;
        if wire == 2 && tool_case_name(field).is_some() {
            if let WireValue::Bytes(bytes) = value {
                last = Some((field, bytes));
            }
        }
    }
    last
}

fn tool_case_name(field: u64) -> Option<&'static str> {
    Some(match field {
        1 => "shellToolCall",
        3 => "deleteToolCall",
        4 => "globToolCall",
        5 => "grepToolCall",
        8 => "readToolCall",
        9 => "updateTodosToolCall",
        10 => "readTodosToolCall",
        12 => "editToolCall",
        13 => "lsToolCall",
        14 => "readLintsToolCall",
        15 => "mcpToolCall",
        16 => "semSearchToolCall",
        17 => "createPlanToolCall",
        18 => "webSearchToolCall",
        19 => "taskToolCall",
        20 => "listMcpResourcesToolCall",
        21 => "readMcpResourceToolCall",
        22 => "applyAgentDiffToolCall",
        23 => "askQuestionToolCall",
        24 => "fetchToolCall",
        25 => "switchModeToolCall",
        28 => "generateImageToolCall",
        29 => "recordScreenToolCall",
        30 => "computerUseToolCall",
        31 => "writeShellStdinToolCall",
        32 => "reflectToolCall",
        33 => "setupVmEnvironmentToolCall",
        34 => "truncatedToolCall",
        35 => "startGrindExecutionToolCall",
        36 => "startGrindPlanningToolCall",
        37 => "webFetchToolCall",
        38 => "reportBugfixResultsToolCall",
        39 => "aiAttributionToolCall",
        40 => "prManagementToolCall",
        41 => "mcpAuthToolCall",
        42 => "awaitToolCall",
        43 => "blameByFilePathToolCall",
        44 => "getMcpToolsToolCall",
        45 => "reportBugToolCall",
        46 => "setActiveBranchToolCall",
        48 => "communicateUpdateToolCall",
        49 => "sendFinalSummaryToolCall",
        50 => "updatePrCodeTourToolCall",
        51 => "replaceEnvToolCall",
        52 => "editPrLabelsToolCall",
        53 => "recordCiInvestigationFindingsToolCall",
        55 => "sendMessageToolCall",
        56 => "fetchCloudAgentDataToolCall",
        58 => "sendToUserToolCall",
        61 => "piReadToolCall",
        62 => "piBashToolCall",
        63 => "piEditToolCall",
        64 => "piWriteToolCall",
        65 => "piGrepToolCall",
        66 => "piFindToolCall",
        67 => "piLsToolCall",
        68 => "connectScmToolCall",
        69 => "searchConversationsToolCall",
        70 => "createGoalToolCall",
        71 => "updateGoalToolCall",
        72 => "adoptToolCall",
        73 => "getAgentStatusToolCall",
        74 => "sendToAgentToolCall",
        75 => "readAgentTranscriptToolCall",
        76 => "createAgentToolCall",
        77 => "stopAgentToolCall",
        _ => return None,
    })
}

enum WireValue<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Fixed,
}

fn read_wire_field<'a>(
    data: &'a [u8],
    position: &mut usize,
) -> Option<(u64, u8, WireValue<'a>)> {
    let tag = read_varint(data, position)?;
    let field = tag >> 3;
    let wire = (tag & 0x07) as u8;
    if field == 0 {
        return None;
    }
    let value = match wire {
        0 => WireValue::Varint(read_varint(data, position)?),
        1 => {
            *position = position.checked_add(8)?;
            if *position > data.len() { return None; }
            WireValue::Fixed
        }
        2 => {
            let len: usize = read_varint(data, position)?.try_into().ok()?;
            let end = position.checked_add(len)?;
            let bytes = data.get(*position..end)?;
            *position = end;
            WireValue::Bytes(bytes)
        }
        5 => {
            *position = position.checked_add(4)?;
            if *position > data.len() { return None; }
            WireValue::Fixed
        }
        _ => return None,
    };
    Some((field, wire, value))
}

fn read_varint(data: &[u8], position: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *data.get(*position)?;
        *position += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn first_bytes_field(data: &[u8], wanted: u64) -> Option<&[u8]> {
    all_bytes_fields(data, wanted).into_iter().next()
}

fn all_bytes_fields(data: &[u8], wanted: u64) -> Vec<&[u8]> {
    let mut values = Vec::new();
    let mut position = 0usize;
    while position < data.len() {
        let Some((field, wire, value)) = read_wire_field(data, &mut position) else {
            break;
        };
        if field == wanted && wire == 2 {
            if let WireValue::Bytes(bytes) = value {
                values.push(bytes);
            }
        }
    }
    values
}

fn first_string_field(data: &[u8], wanted: u64) -> Option<String> {
    let bytes = first_bytes_field(data, wanted)?;
    std::str::from_utf8(bytes).ok().map(ToOwned::to_owned)
}

fn all_string_fields(data: &[u8], wanted: u64) -> Vec<String> {
    all_bytes_fields(data, wanted)
        .into_iter()
        .filter_map(|bytes| std::str::from_utf8(bytes).ok().map(ToOwned::to_owned))
        .collect()
}

fn first_varint_field(data: &[u8], wanted: u64) -> Option<u64> {
    let mut position = 0usize;
    while position < data.len() {
        let (field, wire, value) = read_wire_field(data, &mut position)?;
        if field == wanted && wire == 0 {
            if let WireValue::Varint(value) = value {
                return Some(value);
            }
        }
    }
    None
}

fn last_oneof_bytes<'a>(data: &'a [u8], fields: &[u64]) -> Option<(u64, &'a [u8])> {
    let mut last = None;
    let mut position = 0usize;
    while position < data.len() {
        let (field, wire, value) = read_wire_field(data, &mut position)?;
        if wire == 2 && fields.contains(&field) {
            if let WireValue::Bytes(bytes) = value {
                last = Some((field, bytes));
            }
        }
    }
    last
}
