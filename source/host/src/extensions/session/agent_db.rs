use std::path::Path;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use super::agent_db_schema::GET_KV_SQL;

#[derive(Debug, thiserror::Error)]
pub enum AgentDbProjectionError {
    #[error("agent session sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("agent metadata hex is invalid: {0}")]
    MetadataHex(String),
    #[error("agent metadata json is invalid utf-8: {0}")]
    MetadataUtf8(#[from] std::string::FromUtf8Error),
    #[error("agent metadata json is invalid: {0}")]
    MetadataJson(#[from] serde_json::Error),
    #[error("latestRootBlobId hex is invalid: {0}")]
    LatestRootHex(String),
}

pub fn read_persisted_latest_root_blob_id(
    db_path: &Path,
    busy_timeout_ms: u64,
) -> Result<Vec<u8>, AgentDbProjectionError> {
    let db = Connection::open(db_path)?;
    db.busy_timeout(Duration::from_millis(busy_timeout_ms))?;

    let has_kv = db
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'kv' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !has_kv {
        return Ok(Vec::new());
    }

    let raw = db
        .query_row(
            GET_KV_SQL,
            params!["metadata"],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    let metadata_bytes = decode_hex(&raw).map_err(AgentDbProjectionError::MetadataHex)?;
    let metadata_json = String::from_utf8(metadata_bytes)?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)?;
    let Some(root_hex) = metadata
        .get("latestRootBlobId")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(Vec::new());
    };
    decode_hex(root_hex).map_err(AgentDbProjectionError::LatestRootHex)
}

fn decode_hex(raw: &str) -> Result<Vec<u8>, String> {
    let clean = raw.trim();
    if clean.len() % 2 != 0 {
        return Err("odd-length hex string".into());
    }
    let mut output = Vec::with_capacity(clean.len() / 2);
    for index in (0..clean.len()).step_by(2) {
        let pair = &clean[index..index + 2];
        let byte = u8::from_str_radix(pair, 16)
            .map_err(|_| format!("invalid hex pair at offset {index}: {pair}"))?;
        output.push(byte);
    }
    Ok(output)
}
