use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::agent_db::SandAgentDb;

const FORWARD_BATCHES_KEY: &str = "humanForwardBatches";
const FORWARD_BATCH_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct HumanForwardTarget {
    pub conversation_id: String,
    pub client_nonce: String,
    pub completed: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct HumanForwardBatch {
    pub version: u32,
    pub account_id: String,
    pub issuer: String,
    pub request_id: String,
    pub source_conversation_id: String,
    pub source_entry_id: String,
    pub text: String,
    pub attachments: Vec<Value>,
    pub context: Value,
    pub destinations: Vec<HumanForwardTarget>,
}

pub(super) fn forward_destination_nonce(
    account: &str,
    issuer: &str,
    request: &str,
    destination: &str,
) -> String {
    let mut digest = Sha256::new();
    for part in [account, issuer, request, destination] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    format!("human-forward-{:x}", digest.finalize())
}

impl HumanForwardBatch {
    pub(super) fn new(
        account_id: String,
        issuer: String,
        request_id: String,
        source_conversation_id: String,
        source_entry_id: String,
        text: String,
        attachments: Vec<Value>,
        context: Value,
        destination_ids: &[String],
    ) -> Self {
        let destinations = destination_ids.iter().map(|id| HumanForwardTarget {
            conversation_id: id.clone(),
            client_nonce: forward_destination_nonce(&account_id, &issuer, &request_id, id),
            completed: false,
            error: None,
        }).collect();
        Self {
            version: FORWARD_BATCH_VERSION,
            account_id,
            issuer,
            request_id,
            source_conversation_id,
            source_entry_id,
            text,
            attachments,
            context,
            destinations,
        }
    }

    pub(super) fn validate(&self, expected_source: &str) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        if self.version != FORWARD_BATCH_VERSION
            || self.account_id.is_empty()
            || self.issuer.is_empty()
            || self.request_id.is_empty()
            || self.request_id.len() > 200
            || self.source_conversation_id != expected_source
            || self.source_entry_id.is_empty()
            || self.destinations.is_empty()
            || self.destinations.len() > 32
            || (self.text.trim().is_empty() && self.attachments.is_empty())
            || self.context.get("batchVersion").and_then(Value::as_u64) != Some(1)
            || self.context.get("requestId").and_then(Value::as_str) != Some(self.request_id.as_str())
            || self.context.get("sourceConversationId").and_then(Value::as_str) != Some(self.source_conversation_id.as_str())
            || self.context.get("sourceEntryId").and_then(Value::as_str) != Some(self.source_entry_id.as_str())
            || self.context.pointer("/privacy/dropSenderNames").and_then(Value::as_bool).is_none()
            || self.context.pointer("/privacy/dropCaptions").and_then(Value::as_bool).is_none()
        {
            return Err("persisted Human forward batch is invalid or has an unsupported version".into());
        }
        for target in &self.destinations {
            if target.conversation_id.is_empty()
                || !seen.insert(target.conversation_id.as_str())
                || target.client_nonce != forward_destination_nonce(
                    &self.account_id, &self.issuer, &self.request_id, &target.conversation_id,
                )
                || (target.completed && target.error.is_some())
            {
                return Err("persisted Human forward destination is invalid".into());
            }
        }
        Ok(())
    }

    pub(super) fn contains_requested(&self, requested: &[String]) -> bool {
        requested.iter().all(|id| self.destinations.iter().any(|dest| &dest.conversation_id == id))
    }

    pub(super) fn response(&self, requested: &[String]) -> Value {
        let mut results = Vec::new();
        for id in requested {
            if let Some(dest) = self.destinations.iter().find(|dest| &dest.conversation_id == id) {
                results.push(serde_json::json!({
                    "conversationId": id,
                    "clientNonce": dest.client_nonce,
                    "status": if dest.completed { "sent" } else { "failed" },
                    "error": if dest.completed { None } else { dest.error.as_deref().or(Some("forward delivery remains pending")) },
                }));
            }
        }
        serde_json::json!({
            "requestId": self.request_id,
            "privacy": self.context.get("privacy"),
            "destinations": results,
        })
    }
}

pub(super) fn load_forward_batches(owner: &SandAgentDb, source: &str) -> Result<Vec<HumanForwardBatch>, String> {
    let observed = owner.get_metadata(FORWARD_BATCHES_KEY).map_err(|error| error.to_string())?;
    decode_forward_batches(observed, source)
}

fn decode_forward_batches(
    observed: Option<Value>,
    source: &str,
) -> Result<Vec<HumanForwardBatch>, String> {
    let Some(value) = observed else { return Ok(Vec::new()); };
    let batches: Vec<HumanForwardBatch> = serde_json::from_value(value)
        .map_err(|error| format!("persisted Human forward batches are invalid: {error}"))?;
    let mut seen = std::collections::BTreeSet::new();
    for batch in &batches {
        batch.validate(source)?;
        if !seen.insert((batch.account_id.as_str(), batch.issuer.as_str(), batch.request_id.as_str())) {
            return Err("persisted Human forward batches contain duplicate request identities".into());
        }
    }
    Ok(batches)
}

pub(super) fn store_forward_batch(owner: &SandAgentDb, batch: &HumanForwardBatch) -> Result<(), String> {
    batch.validate(&batch.source_conversation_id)?;
    // Read the exact key value once. A second writer may commit between
    // observation and this merge; the SandAgentDb conditional CAS detects
    // that interleaving instead of erasing another batch's journal.
    let observed = owner.get_metadata(FORWARD_BATCHES_KEY).map_err(|error| error.to_string())?;
    let mut batches = decode_forward_batches(observed.clone(), &batch.source_conversation_id)?;
    if let Some(existing) = batches.iter_mut().find(|prior|
        prior.account_id == batch.account_id && prior.issuer == batch.issuer && prior.request_id == batch.request_id
    ) {
        if existing.source_entry_id != batch.source_entry_id
            || existing.text != batch.text
            || existing.attachments != batch.attachments
            || existing.context != batch.context
            || existing.destinations.iter().map(|item| (&item.conversation_id, &item.client_nonce))
                .ne(batch.destinations.iter().map(|item| (&item.conversation_id, &item.client_nonce)))
        {
            return Err("forwardHumanMessage clientNonce already identifies a different immutable batch".into());
        }
        *existing = batch.clone();
    } else {
        batches.push(batch.clone());
    }
    let value = serde_json::to_value(batches).map_err(|error| error.to_string())?;
    if !owner.set_metadata_if_unchanged(FORWARD_BATCHES_KEY, observed.as_ref(), value)
        .map_err(|error| error.to_string())?
    {
        return Err("Human forward batch progress could not be durably stored".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forward_nonce_is_account_issuer_and_destination_scoped() {
        let a = forward_destination_nonce("a", "prod", "request", "b");
        assert_eq!(a, forward_destination_nonce("a", "prod", "request", "b"));
        assert_ne!(a, forward_destination_nonce("a", "isolated", "request", "b"));
        assert_ne!(a, forward_destination_nonce("b", "prod", "request", "b"));
        assert_ne!(a, forward_destination_nonce("a", "prod", "request", "c"));
    }


    #[test]
    fn stale_forward_metadata_observation_cannot_erase_other_batch() {
        let dir = std::env::temp_dir().join(format!(
            "forward-metadata-cas-{}", uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("cas fixture dir");
        let db_path = dir.join("store.db");
        let first = SandAgentDb::open(&db_path, 500).expect("first owner");
        let second = SandAgentDb::open(&db_path, 500).expect("independent owner");
        let first_observed = second.get_metadata(FORWARD_BATCHES_KEY).expect("observe absence");
        assert!(first_observed.is_none(), "absence is not JSON null");
        let first_value = serde_json::json!([{"requestId": "first"}]);
        assert!(first.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, first_observed.as_ref(), first_value.clone()
        ).expect("commit first value"));
        let stale_second = serde_json::json!([{"requestId": "second"}]);
        assert!(!second.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, first_observed.as_ref(), stale_second
        ).expect("reject stale merge"));
        assert_eq!(first.get_metadata(FORWARD_BATCHES_KEY).unwrap(), Some(first_value.clone()));

        // An unrelated metadata update must not invalidate a journal writer
        // that has observed the same journal value.
        assert!(first.set_metadata("unrelated", serde_json::json!(5)).unwrap());
        let second_value = serde_json::json!([{"requestId": "first"}, {"requestId": "second"}]);
        assert!(second.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, Some(&first_value), second_value.clone()
        ).expect("retry from fresh journal"));
        assert_eq!(first.get_metadata(FORWARD_BATCHES_KEY).unwrap(), Some(second_value.clone()));
        assert!(!first.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, None, serde_json::Value::Null
        ).expect("absent differs from present"));
        assert!(!first.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, Some(&serde_json::Value::Null), serde_json::Value::Null
        ).expect("null differs from journal"));
        assert!(first.set_metadata_if_unchanged(
            FORWARD_BATCHES_KEY, Some(&second_value), second_value.clone()
        ).expect("idempotent no-change journal"));
        assert_eq!(first.get_metadata("unrelated").unwrap(), Some(serde_json::json!(5)));
        first.close(false);
        second.close(false);
        drop(first);
        drop(second);
        std::fs::remove_dir_all(&dir).expect("cleanup fixture");
    }
    #[test]
    fn bad_forward_journal_versions_and_destinations_fail_closed() {
        let context = serde_json::json!({"batchVersion":1,"requestId":"r","sourceConversationId":"s","sourceEntryId":"e","privacy":{"dropSenderNames":false,"dropCaptions":false}});
        let batch = HumanForwardBatch::new("a".into(),"prod".into(),"r".into(),"s".into(),"e".into(),"text".into(),vec![],context,&["b".into(),"c".into()]);
        assert!(batch.validate("s").is_ok());
        let mut invalid = batch.clone();
        invalid.version = 2;
        assert!(invalid.validate("s").is_err());
        invalid = batch.clone();
        invalid.destinations[1].client_nonce = invalid.destinations[0].client_nonce.clone();
        assert!(invalid.validate("s").is_err());
        invalid = batch.clone();
        invalid.destinations[1].conversation_id = "b".into();
        assert!(invalid.validate("s").is_err());
    }
}
