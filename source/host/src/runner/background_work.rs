use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::Value;

pub const SHELL_REWATCH_POLL_DEFAULT_MS: u64 = 10_000;
pub const SHELL_REWATCH_MAX_WAIT_MS: u64 = 5 * 60 * 60 * 1_000;
pub const SHELL_REWATCH_MISSING_FILE_GIVE_UP: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellTerminalFooter {
    pub is_complete: bool,
    pub exit_code: Option<i64>,
    pub is_stream_failure: bool,
}

impl Default for ShellTerminalFooter {
    fn default() -> Self {
        Self {
            is_complete: false,
            exit_code: None,
            is_stream_failure: false,
        }
    }
}

#[derive(Clone)]
pub struct BackgroundWorkRecord {
    pub id: String,
    pub kind: String,
    pub state: String,
    pub owner_id: Option<String>,
    pub metadata: Option<Value>,
    pub abort: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for BackgroundWorkRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BackgroundWorkRecord")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("state", &self.state)
            .field("owner_id", &self.owner_id)
            .field("metadata", &self.metadata)
            .field("has_abort", &self.abort.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundWakeupPayload {
    pub kind: String,
    pub reason: Option<String>,
    pub task_id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub detail: Option<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundWakeup {
    pub id: Option<String>,
    pub conversation_id: Option<String>,
    pub payload: BackgroundWakeupPayload,
}

pub type ShellCompletionCallback =
    Arc<dyn Fn(&BackgroundWakeupPayload, Option<&str>) + Send + Sync>;
pub type ShellWorkRegisteredCallback =
    Arc<dyn Fn(&BackgroundWorkRecord, Option<&str>) + Send + Sync>;
pub type WorkSetChangedCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
pub struct RevivingBackgroundWorkRegistry {
    work: HashMap<String, BackgroundWorkRecord>,
    wakeups: Vec<BackgroundWakeup>,
    completions: Vec<BackgroundWakeupPayload>,
    quiet_origins: HashMap<String, String>,
    on_shell_completion: Option<ShellCompletionCallback>,
    on_shell_work_registered: Option<ShellWorkRegisteredCallback>,
    on_work_set_changed: Option<WorkSetChangedCallback>,
}

impl RevivingBackgroundWorkRegistry {
    pub fn with_callbacks(
        on_shell_completion: Option<ShellCompletionCallback>,
        on_shell_work_registered: Option<ShellWorkRegisteredCallback>,
        on_work_set_changed: Option<WorkSetChangedCallback>,
    ) -> Self {
        Self {
            on_shell_completion,
            on_shell_work_registered,
            on_work_set_changed,
            ..Self::default()
        }
    }

    pub fn upsert_work(&mut self, record: BackgroundWorkRecord) {
        self.work.insert(record.id.clone(), record);
        self.changed();
    }

    pub fn upsert_work_for_turn(
        &mut self,
        record: BackgroundWorkRecord,
        quiet_origin: Option<&str>,
    ) {
        self.record_work_origin(&record.id, quiet_origin);
        let is_running_shell = record.kind == "shell" && record.state == "running";
        self.upsert_work(record.clone());
        if is_running_shell {
            if let Some(callback) = self.on_shell_work_registered.as_ref() {
                callback(&record, quiet_origin);
            }
        }
    }

    pub fn clear_work(&mut self, id: &str) -> Option<BackgroundWorkRecord> {
        let value = self.work.remove(id);
        self.changed();
        value
    }

    pub fn abort_work(&mut self, id: &str) -> bool {
        let Some(record) = self.work.remove(id) else {
            return false;
        };
        if let Some(abort) = record.abort {
            abort();
        }
        self.changed();
        true
    }

    pub fn abort_all_work(&mut self, kind: Option<&str>) -> usize {
        let ids = self
            .list_work(kind)
            .into_iter()
            .map(|record| record.id.clone())
            .collect::<Vec<_>>();
        for id in &ids {
            if let Some(record) = self.work.remove(id) {
                if let Some(abort) = record.abort {
                    abort();
                }
            }
        }
        self.changed();
        ids.len()
    }

    pub fn has_running_work(&self, kind: Option<&str>) -> bool {
        self.list_work(kind)
            .into_iter()
            .any(|record| record.state == "running")
    }

    pub fn list_work(&self, kind: Option<&str>) -> Vec<&BackgroundWorkRecord> {
        self.work
            .values()
            .filter(|record| kind.is_none_or(|kind| record.kind == kind))
            .collect()
    }

    pub fn enqueue(&mut self, wakeup: BackgroundWakeup) {
        let payload = &wakeup.payload;
        if payload.kind == "shell" && payload.reason.as_deref() != Some("task_progress") {
            let origin = self.quiet_origins.remove(&payload.task_id);
            if let Some(callback) = self.on_shell_completion.as_ref() {
                callback(payload, origin.as_deref());
            }
            return;
        }
        self.wakeups.push(wakeup);
    }

    pub fn pull(&self, conversation_id: &str) -> Vec<BackgroundWakeup> {
        self.wakeups
            .iter()
            .filter(|item| {
                item.conversation_id
                    .as_deref()
                    .is_none_or(|candidate| candidate == conversation_id)
            })
            .cloned()
            .collect()
    }

    pub fn ack(&mut self, ids: &[String]) {
        let selected = ids.iter().map(String::as_str).collect::<HashSet<_>>();
        self.wakeups
            .retain(|wakeup| !wakeup.id.as_deref().is_some_and(|id| selected.contains(id)));
    }

    pub fn nack(&mut self, _ids: &[String]) {}

    pub fn suppress(&mut self, conversation_id: &str, id: &str) {
        self.wakeups.retain(|wakeup| {
            !(wakeup.conversation_id.as_deref() == Some(conversation_id)
                && wakeup.id.as_deref() == Some(id))
        });
    }

    pub fn enqueue_completion(&mut self, item: BackgroundWakeupPayload) {
        self.completions.push(item);
    }

    pub fn drain_completions(&mut self) -> Vec<BackgroundWakeupPayload> {
        std::mem::take(&mut self.completions)
    }

    pub fn has_pending_completions(&self, _conversation_id: &str) -> bool {
        !self.completions.is_empty()
    }

    pub fn mark_awaited_completion(&mut self, _task_id: &str) {}

    pub fn record_work_origin(&mut self, id: &str, origin: Option<&str>) {
        match origin {
            Some(origin) => {
                self.quiet_origins.insert(id.to_string(), origin.to_string());
            }
            None => {
                self.quiet_origins.remove(id);
            }
        }
    }

    fn changed(&self) {
        if let Some(callback) = self.on_work_set_changed.as_ref() {
            callback();
        }
    }
}

pub fn shell_rewatch_poll_ms(raw: Option<&str>) -> u64 {
    raw.and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(SHELL_REWATCH_POLL_DEFAULT_MS)
}

pub fn parse_shell_terminal_footer(content: &str) -> ShellTerminalFooter {
    let Some(marker_start) = content.rfind("\n---\n") else {
        return ShellTerminalFooter::default();
    };
    let footer_and_end = &content[marker_start + 5..];
    let Some(footer) = footer_and_end.strip_suffix("\n---")
        .or_else(|| footer_and_end.strip_suffix("\n---\n"))
        .or_else(|| footer_and_end.strip_suffix("\n---\r\n"))
    else {
        return ShellTerminalFooter::default();
    };

    for line in footer.lines() {
        if let Some(raw) = line.strip_prefix("exit_code:") {
            let raw = raw.trim();
            return ShellTerminalFooter {
                is_complete: true,
                exit_code: (!raw.is_empty())
                    .then(|| raw.parse::<i64>().ok())
                    .flatten(),
                is_stream_failure: false,
            };
        }
    }

    let has_error = footer.lines().any(|line| line.starts_with("error:"));
    let has_ended_at = footer
        .lines()
        .any(|line| line.strip_prefix("ended_at:").is_some_and(|value| !value.trim().is_empty()));
    if has_error && has_ended_at {
        ShellTerminalFooter {
            is_complete: true,
            exit_code: None,
            is_stream_failure: true,
        }
    } else {
        ShellTerminalFooter::default()
    }
}

pub fn derive_background_subagent_title(prompt: &str) -> String {
    let one_line = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.is_empty() {
        return "Background task".to_string();
    }
    if one_line.chars().count() <= 80 {
        return one_line;
    }
    let prefix = one_line.chars().take(79).collect::<String>();
    format!("{prefix}…")
}

pub fn format_steer_prompt(message: &str) -> String {
    [
        "[Steering message from the parent agent that dispatched you]",
        message.trim(),
        "Take this into account and continue your task from where you are — do not start over.",
    ]
    .join("\n\n")
}
