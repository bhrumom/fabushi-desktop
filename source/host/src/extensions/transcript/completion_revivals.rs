use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use super::sand_pending_wake_store::{
    DurablePendingWakeMarker, PendingWakeKind, QuietWakeOrigin,
};

pub const QUIET_REVIVAL_INSTRUCTION: &str = "Pick the work back up. Everything above came out of your own quiet standing order(s) — the user did not ask to hear about it, so the saved instruction's delivery rule governs. If the outcome is a genuine change, a new actionable result, or a real blocker the user must know about, tell them once with a single useful SendMessage. If it amounts to no change, nothing new, or still waiting, end the turn with no SendMessage at all — no \"still waiting\" or progress notes; if the standing order says to keep watching, just keep the watch going quietly. Keep your status current, and clear it once everything is done and you're idle.";

#[derive(Debug, Clone, PartialEq)]
pub struct SubagentCompletion {
    pub parent_agent_id: String,
    pub subagent_agent_id: String,
    pub title: String,
    pub subagent_type: String,
    pub status: String,
    pub result: String,
    pub quiet_origin: Option<QuietWakeOrigin>,
    pub pending_wake_marker: Option<DurablePendingWakeMarker>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellCompletion {
    pub agent_id: String,
    pub shell_id: String,
    pub title: String,
    pub status: String,
    pub detail: Option<String>,
    pub output_path: Option<String>,
    pub quiet_origin: Option<QuietWakeOrigin>,
    pub pending_wake_marker: Option<DurablePendingWakeMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevivalExecution {
    pub aborted: bool,
    pub quiesced_for_upgrade: bool,
    pub sent_message_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevivalReport {
    pub agent_id: String,
    pub kind: &'static str,
    pub outcome: String,
    pub completion_count: usize,
    pub subagent_type: Option<String>,
    pub subagent_agent_id: Option<String>,
    pub is_quiet_origin: bool,
    pub reason: Option<String>,
    pub sent_message_count: Option<usize>,
}

pub trait CompletionRevivalRuntimePort: Send + Sync {
    fn can_execute(&self) -> bool;
    fn is_agent_deleted(&self, agent_id: &str) -> bool;
    fn is_agent_gone(&self, agent_id: &str) -> bool;
    fn clear_pending_wake(&self, marker: DurablePendingWakeMarker);
    fn clear_pending_wake_by_key(&self, agent_id: &str, kind: PendingWakeKind, work_id: &str);
    fn run_background_revival(
        &self,
        agent_id: &str,
        source: &str,
        prompt: &str,
        is_silence_allowed: bool,
        auto_review_epoch: &str,
    ) -> Result<RevivalExecution, String>;
    fn mark_resume_pending_for_quiesced_revival(&self, agent_id: &str);
    fn report_revival(&self, report: RevivalReport);
    fn report_revival_error(&self, agent_id: &str, title: &str, error: &str);
}

#[derive(Default)]
struct CompletionRevivalState {
    pending_subagent: HashMap<String, Vec<SubagentCompletion>>,
    reviving_subagent: HashSet<String>,
    pending_shell: HashMap<String, Vec<ShellCompletion>>,
    reviving_shell: HashSet<String>,
}

impl CompletionRevivalState {
    fn take_subagent_batch_or_end_revival(&mut self, agent_id: &str) -> Option<Vec<SubagentCompletion>> {
        let completions = self.pending_subagent.remove(agent_id).unwrap_or_default();
        if completions.is_empty() {
            self.reviving_subagent.remove(agent_id);
            None
        } else {
            Some(completions)
        }
    }

    fn take_shell_batch_or_end_revival(&mut self, agent_id: &str) -> Option<Vec<ShellCompletion>> {
        let completions = self.pending_shell.remove(agent_id).unwrap_or_default();
        if completions.is_empty() {
            self.reviving_shell.remove(agent_id);
            None
        } else {
            Some(completions)
        }
    }
}

pub struct CompletionRevivals {
    runtime: Arc<dyn CompletionRevivalRuntimePort>,
    state: Mutex<CompletionRevivalState>,
}

impl CompletionRevivals {
    pub fn new(runtime: Arc<dyn CompletionRevivalRuntimePort>) -> Self {
        Self { runtime, state: Mutex::new(CompletionRevivalState::default()) }
    }

    pub fn pending_subagent_completions(&self, agent_id: &str) -> Vec<SubagentCompletion> {
        self.state.lock().unwrap().pending_subagent.get(agent_id).cloned().unwrap_or_default()
    }

    pub fn pending_shell_completions(&self, agent_id: &str) -> Vec<ShellCompletion> {
        self.state.lock().unwrap().pending_shell.get(agent_id).cloned().unwrap_or_default()
    }

    pub fn clear_agent_pending_completions(&self, agent_id: &str) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending_subagent.remove(agent_id);
        state.pending_shell.remove(agent_id);
    }

    /// Frozen SandHost health treats a completion revival that is actively
    /// draining as background work even after its durable marker has been
    /// removed. Read the single CompletionRevivals owner instead of inventing
    /// a parallel Host-side busy flag.
    pub fn has_mid_drain_revival(&self) -> bool {
        !self.mid_drain_revival_agent_ids().is_empty()
    }

    pub fn mid_drain_revival_agent_ids(&self) -> Vec<String> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut agent_ids = state
            .reviving_subagent
            .iter()
            .chain(state.reviving_shell.iter())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        agent_ids.sort();
        agent_ids
    }

    pub fn handle_background_subagent_completion(&self, completion: SubagentCompletion) {
        if self.runtime.is_agent_deleted(&completion.parent_agent_id) {
            self.runtime.report_revival(RevivalReport {
                agent_id: completion.parent_agent_id.clone(),
                kind: "subagent",
                outcome: "dropped".into(),
                completion_count: 1,
                subagent_type: Some(completion.subagent_type.clone()),
                subagent_agent_id: Some(completion.subagent_agent_id.clone()),
                is_quiet_origin: completion.quiet_origin.is_some(),
                reason: Some("agent_deleted".into()),
                sent_message_count: None,
            });
            return;
        }

        let agent_id = completion.parent_agent_id.clone();
        {
            let mut state = self.state.lock().unwrap();
            let queue = state.pending_subagent.entry(agent_id.clone()).or_default();
            if queue.iter().any(|item| item.subagent_agent_id == completion.subagent_agent_id) {
                return;
            }
            queue.push(completion);
        }
        self.revive_for_subagent_completions(&agent_id);
    }

    pub fn revive_for_subagent_completions(&self, agent_id: &str) {
        self.revive_for_subagent_completions_with_idle_return(agent_id, || {});
    }

    fn revive_for_subagent_completions_with_idle_return(
        &self,
        agent_id: &str,
        on_idle_return: impl FnOnce(),
    ) {
        if !self.runtime.can_execute() {
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            if !state.reviving_subagent.insert(agent_id.to_string()) {
                return;
            }
        }

        loop {
            let completions = {
                let mut state = self.state.lock().unwrap();
                state.take_subagent_batch_or_end_revival(agent_id)
            };
            let Some(completions) = completions else {
                // Ownership was released with the empty observation. A
                // successor may already own it when this worker returns.
                on_idle_return();
                return;
            };
            for completion in &completions {
                if let Some(marker) = completion.pending_wake_marker.clone() {
                    self.runtime.clear_pending_wake(marker);
                } else {
                    self.runtime.clear_pending_wake_by_key(
                        agent_id,
                        if completion.subagent_type == "cursor-agent" {
                            PendingWakeKind::CloudAgent
                        } else {
                            PendingWakeKind::Subagent
                        },
                        &completion.subagent_agent_id,
                    );
                }
            }
            let report = self.run_subagent_revival(agent_id, &completions);
            self.runtime.report_revival(RevivalReport {
                agent_id: agent_id.to_string(),
                kind: "subagent",
                completion_count: completions.len(),
                subagent_type: completions.first().map(|item| item.subagent_type.clone()),
                subagent_agent_id: None,
                is_quiet_origin: is_all_quiet_subagent(&completions),
                ..report
            });
        }
    }

    fn run_subagent_revival(
        &self,
        agent_id: &str,
        completions: &[SubagentCompletion],
    ) -> RevivalReport {
        if completions.is_empty() {
            return base_report(agent_id, "subagent", "delivered");
        }
        if !self.runtime.can_execute() {
            return dropped_report(agent_id, "subagent", "no_runner");
        }
        if self.runtime.is_agent_gone(agent_id) {
            return dropped_report(agent_id, "subagent", "session_unavailable");
        }
        let prompt = build_subagent_revival_prompt(completions);
        match self.runtime.run_background_revival(
            agent_id,
            "subagent-revival",
            &prompt,
            true,
            "continue",
        ) {
            Ok(run) if run.aborted => base_report(agent_id, "subagent", "superseded"),
            Ok(run) if run.quiesced_for_upgrade => {
                self.runtime.mark_resume_pending_for_quiesced_revival(agent_id);
                dropped_report(agent_id, "subagent", "quiesced")
            }
            Ok(run) => RevivalReport {
                sent_message_count: Some(run.sent_message_count),
                ..base_report(agent_id, "subagent", "delivered")
            },
            Err(error) => {
                self.runtime.report_revival_error(
                    agent_id,
                    "Background task follow-up failed",
                    &error,
                );
                dropped_report(agent_id, "subagent", "error")
            }
        }
    }

    pub fn handle_background_shell_completion(&self, completion: ShellCompletion) {
        if self.runtime.is_agent_gone(&completion.agent_id) {
            self.runtime.report_revival(RevivalReport {
                agent_id: completion.agent_id,
                kind: "shell",
                outcome: "dropped".into(),
                completion_count: 1,
                subagent_type: None,
                subagent_agent_id: None,
                is_quiet_origin: completion.quiet_origin.is_some(),
                reason: Some("agent_gone".into()),
                sent_message_count: None,
            });
            return;
        }
        let agent_id = completion.agent_id.clone();
        self.state.lock().unwrap().pending_shell.entry(agent_id.clone()).or_default().push(completion);
        self.revive_for_shell_completions(&agent_id);
    }

    pub fn revive_for_shell_completions(&self, agent_id: &str) {
        self.revive_for_shell_completions_with_idle_return(agent_id, || {});
    }

    fn revive_for_shell_completions_with_idle_return(
        &self,
        agent_id: &str,
        on_idle_return: impl FnOnce(),
    ) {
        if !self.runtime.can_execute() {
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            if !state.reviving_shell.insert(agent_id.to_string()) {
                return;
            }
        }
        loop {
            let completions = {
                let mut state = self.state.lock().unwrap();
                state.take_shell_batch_or_end_revival(agent_id)
            };
            let Some(completions) = completions else {
                on_idle_return();
                return;
            };
            for completion in &completions {
                if let Some(marker) = completion.pending_wake_marker.clone() {
                    self.runtime.clear_pending_wake(marker);
                } else {
                    self.runtime.clear_pending_wake_by_key(
                        agent_id,
                        PendingWakeKind::Shell,
                        &completion.shell_id,
                    );
                }
            }
            let report = self.run_shell_revival(agent_id, &completions);
            self.runtime.report_revival(report);
        }
    }

    fn run_shell_revival(&self, agent_id: &str, completions: &[ShellCompletion]) -> RevivalReport {
        if completions.is_empty() {
            return base_report(agent_id, "shell", "delivered");
        }
        if !self.runtime.can_execute() {
            return shell_report(agent_id, completions, "dropped", Some("no_runner"), None);
        }
        if self.runtime.is_agent_gone(agent_id) {
            return shell_report(agent_id, completions, "dropped", Some("session_unavailable"), None);
        }
        let prompt = build_shell_revival_prompt(completions);
        match self.runtime.run_background_revival(
            agent_id,
            "shell-revival",
            &prompt,
            true,
            "continue",
        ) {
            Ok(run) if run.aborted => shell_report(agent_id, completions, "superseded", None, None),
            Ok(run) if run.quiesced_for_upgrade => {
                self.runtime.mark_resume_pending_for_quiesced_revival(agent_id);
                shell_report(agent_id, completions, "dropped", Some("quiesced"), None)
            }
            Ok(run) => shell_report(
                agent_id,
                completions,
                "delivered",
                None,
                Some(run.sent_message_count),
            ),
            Err(error) => {
                self.runtime.report_revival_error(
                    agent_id,
                    "Background command follow-up failed",
                    &error,
                );
                shell_report(agent_id, completions, "dropped", Some("error"), None)
            }
        }
    }
}

pub fn describe_quiet_origin_note(origin: &QuietWakeOrigin) -> String {
    let source = origin
        .automation
        .as_ref()
        .map(|automation| {
            format!(
                "your routine \"{}\" (folder {})",
                automation.name, automation.id
            )
        })
        .unwrap_or_else(|| "one of your own quiet self-initiated runs".into());
    format!("(You started this during {source} — nobody is waiting on it.)")
}

pub fn is_all_quiet_subagent(completions: &[SubagentCompletion]) -> bool {
    completions.iter().all(|completion| completion.quiet_origin.is_some())
}

pub fn is_all_quiet_shell(completions: &[ShellCompletion]) -> bool {
    completions.iter().all(|completion| completion.quiet_origin.is_some())
}

pub fn build_subagent_revival_prompt(completions: &[SubagentCompletion]) -> String {
    let blocks = completions
        .iter()
        .map(|completion| {
            let label = if completion.status == "error" {
                format!(
                    "Background task \"{}\" ({}) failed:",
                    completion.title, completion.subagent_type
                )
            } else {
                format!(
                    "Background task \"{}\" ({}) finished:",
                    completion.title, completion.subagent_type
                )
            };
            let mut value = format!("{label}\n{}", completion.result);
            if let Some(origin) = &completion.quiet_origin {
                value.push('\n');
                value.push_str(&describe_quiet_origin_note(origin));
            }
            value
        })
        .collect::<Vec<_>>();
    let intro = if completions.len() == 1 {
        "A background task you started has finished.".to_string()
    } else {
        format!("{} background tasks you started have finished.", completions.len())
    };
    let instruction = "Pick the work back up: review the result(s), then either keep going or wrap up. If this result is genuinely new and relevant to the user, or the user asked to be told when this finished, tell them with a SendMessage. Lead with the concrete thing that finished, not a bare pronoun like \"That\" (they cannot see the background task). If it is stale, irrelevant, already handled, or a duplicate, and the user was not waiting on it, just stay silent and end the turn with no SendMessage rather than narrating it. Keep your status current, and clear it once everything is done and you're idle.";
    [
        format!("[A background task just completed] {intro}"),
        String::new(),
        blocks.join("\n\n"),
        String::new(),
        if is_all_quiet_subagent(completions) {
            QUIET_REVIVAL_INSTRUCTION.to_string()
        } else {
            instruction.to_string()
        },
    ]
    .join("\n")
}

pub fn describe_shell_outcome(status: &str) -> &'static str {
    match status {
        "success" => "finished",
        "aborted" => "was stopped",
        _ => "failed",
    }
}

pub fn build_shell_revival_prompt(completions: &[ShellCompletion]) -> String {
    let blocks = completions
        .iter()
        .map(|completion| {
            let mut lines = vec![format!(
                "Background command \"{}\" {}.",
                completion.title,
                describe_shell_outcome(&completion.status)
            )];
            if let Some(detail) = &completion.detail {
                lines.push(detail.clone());
            }
            if let Some(path) = &completion.output_path {
                lines.push(format!("Full output: {path}"));
            }
            if let Some(origin) = &completion.quiet_origin {
                lines.push(describe_quiet_origin_note(origin));
            }
            lines.join("\n")
        })
        .collect::<Vec<_>>();
    let intro = if completions.len() == 1 {
        "A command you started in the background has finished.".to_string()
    } else {
        format!(
            "{} commands you started in the background have finished.",
            completions.len()
        )
    };
    let instruction = "Pick the work back up: check the result (read the output file if you need the full logs), then either keep going or wrap up. If this result is genuinely new and relevant to the user, or the user asked to be told when this finished, tell them with a SendMessage. Lead with the concrete thing that finished, not a bare pronoun like \"That\" (they cannot see the background task). If it is stale, irrelevant, already handled, or a duplicate, and the user was not waiting on it, just stay silent and end the turn with no SendMessage rather than narrating it. Keep your status current, and clear it once everything is done and you're idle.";
    [
        format!("[A background command just completed] {intro}"),
        String::new(),
        blocks.join("\n\n"),
        String::new(),
        if is_all_quiet_shell(completions) {
            QUIET_REVIVAL_INSTRUCTION.to_string()
        } else {
            instruction.to_string()
        },
    ]
    .join("\n")
}

fn base_report(agent_id: &str, kind: &'static str, outcome: &str) -> RevivalReport {
    RevivalReport {
        agent_id: agent_id.into(),
        kind,
        outcome: outcome.into(),
        completion_count: 0,
        subagent_type: None,
        subagent_agent_id: None,
        is_quiet_origin: false,
        reason: None,
        sent_message_count: None,
    }
}

fn dropped_report(agent_id: &str, kind: &'static str, reason: &str) -> RevivalReport {
    RevivalReport {
        reason: Some(reason.into()),
        ..base_report(agent_id, kind, "dropped")
    }
}

fn shell_report(
    agent_id: &str,
    completions: &[ShellCompletion],
    outcome: &str,
    reason: Option<&str>,
    sent_message_count: Option<usize>,
) -> RevivalReport {
    RevivalReport {
        agent_id: agent_id.into(),
        kind: "shell",
        outcome: outcome.into(),
        completion_count: completions.len(),
        subagent_type: None,
        subagent_agent_id: None,
        is_quiet_origin: is_all_quiet_shell(completions),
        reason: reason.map(str::to_string),
        sent_message_count,
    }
}

#[cfg(test)]
mod idle_handoff_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    #[derive(Default)]
    struct Runtime {
        can_execute: AtomicBool,
        pause_once: Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>,
        runs: Mutex<Vec<(String, String, String)>>,
        cleared: Mutex<Vec<(String, PendingWakeKind, String)>>,
        reports: Mutex<Vec<RevivalReport>>,
    }

    impl CompletionRevivalRuntimePort for Runtime {
        fn can_execute(&self) -> bool {
            self.can_execute.load(Ordering::SeqCst)
        }

        fn is_agent_deleted(&self, _: &str) -> bool {
            false
        }

        fn is_agent_gone(&self, _: &str) -> bool {
            false
        }

        fn clear_pending_wake(&self, marker: DurablePendingWakeMarker) {
            self.clear_pending_wake_by_key(&marker.agent_id, marker.kind, &marker.work_id);
        }

        fn clear_pending_wake_by_key(&self, agent_id: &str, kind: PendingWakeKind, work_id: &str) {
            self.cleared.lock().unwrap().push((agent_id.into(), kind, work_id.into()));
        }

        fn run_background_revival(
            &self,
            agent_id: &str,
            source: &str,
            prompt: &str,
            is_silence_allowed: bool,
            auto_review_epoch: &str,
        ) -> Result<RevivalExecution, String> {
            assert!(is_silence_allowed);
            assert_eq!(auto_review_epoch, "continue");
            self.runs.lock().unwrap().push((agent_id.into(), source.into(), prompt.into()));
            let pause_once = self.pause_once.lock().unwrap().take();
            if let Some((entered, release)) = pause_once {
                entered.send(()).unwrap();
                release.recv_timeout(Duration::from_secs(10)).unwrap();
            }
            Ok(RevivalExecution {
                aborted: false,
                quiesced_for_upgrade: false,
                sent_message_count: 3,
            })
        }

        fn mark_resume_pending_for_quiesced_revival(&self, _: &str) {
            panic!("successful revival must not mark upgrade recovery");
        }

        fn report_revival(&self, report: RevivalReport) {
            self.reports.lock().unwrap().push(report);
        }

        fn report_revival_error(&self, _: &str, title: &str, error: &str) {
            panic!("unexpected revival error: {title}: {error}");
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Family {
        Subagent,
        Shell,
    }

    impl Family {
        fn kind(self) -> &'static str {
            match self {
                Self::Subagent => "subagent",
                Self::Shell => "shell",
            }
        }

        fn source(self) -> &'static str {
            match self {
                Self::Subagent => "subagent-revival",
                Self::Shell => "shell-revival",
            }
        }

        fn wake_kind(self) -> PendingWakeKind {
            match self {
                Self::Subagent => PendingWakeKind::Subagent,
                Self::Shell => PendingWakeKind::Shell,
            }
        }

        fn other(self) -> Self {
            match self {
                Self::Subagent => Self::Shell,
                Self::Shell => Self::Subagent,
            }
        }

        fn enqueue(self, service: &CompletionRevivals, agent_id: &str, work_id: &str) {
            match self {
                Self::Subagent => service.handle_background_subagent_completion(subagent(agent_id, work_id)),
                Self::Shell => service.handle_background_shell_completion(shell(agent_id, work_id)),
            }
        }

        fn drain_with_idle_return(
            self,
            service: &CompletionRevivals,
            agent_id: &str,
            on_idle_return: impl FnOnce(),
        ) {
            match self {
                Self::Subagent => service.revive_for_subagent_completions_with_idle_return(agent_id, on_idle_return),
                Self::Shell => service.revive_for_shell_completions_with_idle_return(agent_id, on_idle_return),
            }
        }

        fn pending_ids(self, service: &CompletionRevivals, agent_id: &str) -> Vec<String> {
            match self {
                Self::Subagent => service.pending_subagent_completions(agent_id).into_iter().map(|item| item.subagent_agent_id).collect(),
                Self::Shell => service.pending_shell_completions(agent_id).into_iter().map(|item| item.shell_id).collect(),
            }
        }
    }

    fn subagent(agent_id: &str, work_id: &str) -> SubagentCompletion {
        SubagentCompletion {
            parent_agent_id: agent_id.into(),
            subagent_agent_id: work_id.into(),
            title: work_id.into(),
            subagent_type: "task".into(),
            status: "success".into(),
            result: "done".into(),
            quiet_origin: None,
            pending_wake_marker: None,
        }
    }

    fn shell(agent_id: &str, work_id: &str) -> ShellCompletion {
        ShellCompletion {
            agent_id: agent_id.into(),
            shell_id: work_id.into(),
            title: work_id.into(),
            status: "success".into(),
            detail: Some("done".into()),
            output_path: None,
            quiet_origin: None,
            pending_wake_marker: None,
        }
    }

    #[test]
    fn predecessor_idle_return_preserves_successor_for_each_completion_family() {
        for family in [Family::Subagent, Family::Shell] {
            let runtime = Arc::new(Runtime::default());
            let service = Arc::new(CompletionRevivals::new(runtime.clone()));
            family.enqueue(&service, "p", "first");
            assert_eq!(family.pending_ids(&service, "p"), vec!["first"]);
            runtime.can_execute.store(true, Ordering::SeqCst);
            let (idle_entered_tx, idle_entered_rx) = mpsc::channel();
            let (idle_release_tx, idle_release_rx) = mpsc::channel();
            let predecessor_service = Arc::clone(&service);
            let predecessor = std::thread::spawn(move || {
                family.drain_with_idle_return(&predecessor_service, "p", || {
                    idle_entered_tx.send(()).unwrap();
                    idle_release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                });
            });
            idle_entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            assert!(family.pending_ids(&service, "p").is_empty());
            assert!(!service.has_mid_drain_revival(), "idle release is complete before the old worker returns");

            let (successor_entered_tx, successor_entered_rx) = mpsc::channel();
            let (successor_release_tx, successor_release_rx) = mpsc::channel();
            *runtime.pause_once.lock().unwrap() = Some((successor_entered_tx, successor_release_rx));
            let successor_service = Arc::clone(&service);
            let successor = std::thread::spawn(move || {
                family.enqueue(&successor_service, "p", "successor");
            });
            successor_entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(service.mid_drain_revival_agent_ids(), vec!["p"]);
            assert!(family.pending_ids(&service, "p").is_empty());

            idle_release_tx.send(()).unwrap();
            predecessor.join().unwrap();
            assert_eq!(service.mid_drain_revival_agent_ids(), vec!["p"], "predecessor return must not clear the successor's occupancy");

            family.enqueue(&service, "q", "other-agent");
            family.other().enqueue(&service, "p", "other-family");
            assert!(family.pending_ids(&service, "q").is_empty());
            assert!(family.other().pending_ids(&service, "p").is_empty());
            assert_eq!(service.mid_drain_revival_agent_ids(), vec!["p"], "other agents and queue families do not release this owner");

            family.enqueue(&service, "p", "late");
            assert_eq!(family.pending_ids(&service, "p"), vec!["late"]);
            let runs_before_release = runtime.runs.lock().unwrap().iter()
                .filter(|run| run.0 == "p" && run.1 == family.source()).count();
            assert_eq!(runs_before_release, 2, "late enqueue must join the held successor, not start a third worker");
            successor_release_tx.send(()).unwrap();
            successor.join().unwrap();

            assert!(!service.has_mid_drain_revival());
            for agent_id in ["p", "q"] {
                assert!(service.pending_subagent_completions(agent_id).is_empty());
                assert!(service.pending_shell_completions(agent_id).is_empty());
            }
            let runs = runtime.runs.lock().unwrap();
            assert_eq!(runs.len(), 5);
            let same_owner_runs = runs.iter().filter(|run| run.0 == "p" && run.1 == family.source()).collect::<Vec<_>>();
            assert_eq!(same_owner_runs.len(), 3);
            for (run, title) in same_owner_runs.iter().zip(["first", "successor", "late"]) {
                assert!(run.2.contains(&format!("\"{title}\"")));
            }
            assert!(runs.iter().any(|run| run.0 == "q" && run.1 == family.source() && run.2.contains("\"other-agent\"")));
            assert!(runs.iter().any(|run| run.0 == "p" && run.1 == family.other().source() && run.2.contains("\"other-family\"")));
            let cleared = runtime.cleared.lock().unwrap();
            let same_owner_cleared = cleared.iter().filter(|item| item.0 == "p" && item.1 == family.wake_kind())
                .map(|item| item.2.as_str()).collect::<Vec<_>>();
            assert_eq!(same_owner_cleared, vec!["first", "successor", "late"]);
            assert_eq!(cleared.len(), 5);
            let reports = runtime.reports.lock().unwrap();
            assert_eq!(reports.len(), 5);
            assert_eq!(reports.iter().filter(|report| report.agent_id == "p" && report.kind == family.kind()).count(), 3);
            assert_eq!(reports.iter().filter(|report| report.agent_id == "q" && report.kind == family.kind()).count(), 1);
            assert_eq!(reports.iter().filter(|report| report.agent_id == "p" && report.kind == family.other().kind()).count(), 1);
            for report in reports.iter() {
                assert_eq!(report.outcome, "delivered");
                assert_eq!(report.completion_count, 1);
                assert_eq!(report.sent_message_count, Some(3));
                assert_eq!(report.reason, None);
                assert!(!report.is_quiet_origin);
                assert_eq!(report.subagent_agent_id, None);
                assert_eq!(report.subagent_type.as_deref(), if report.kind == "subagent" { Some("task") } else { None });
            }
        }
    }

    #[test]
    fn empty_batch_release_is_local_to_agent_and_family() {
        let mut state = CompletionRevivalState::default();
        for agent_id in ["p", "q"] {
            state.reviving_subagent.insert(agent_id.into());
            state.reviving_shell.insert(agent_id.into());
        }
        state.pending_subagent.insert("p".into(), Vec::new());
        state.pending_subagent.insert("q".into(), vec![subagent("q", "sub-q")]);
        state.pending_shell.insert("p".into(), vec![shell("p", "shell-first"), shell("p", "shell-second")]);
        state.pending_shell.insert("q".into(), vec![shell("q", "shell-q")]);

        assert_eq!(state.take_subagent_batch_or_end_revival("p"), None);
        assert!(!state.reviving_subagent.contains("p"));
        assert!(state.reviving_subagent.contains("q"));
        assert!(state.reviving_shell.contains("p"));
        assert!(state.reviving_shell.contains("q"));
        assert_eq!(state.pending_subagent["q"][0].subagent_agent_id, "sub-q");
        assert_eq!(state.pending_shell["q"][0].shell_id, "shell-q");
        let batch = state.take_shell_batch_or_end_revival("p").unwrap();
        assert_eq!(batch.iter().map(|item| item.shell_id.as_str()).collect::<Vec<_>>(), vec!["shell-first", "shell-second"]);
        assert!(state.reviving_shell.contains("p"), "a nonempty batch retains its drain owner");
        assert_eq!(state.take_shell_batch_or_end_revival("p"), None);
        assert!(!state.reviving_shell.contains("p"));
        state.reviving_shell.insert("p".into());
        state.pending_shell.insert("p".into(), Vec::new());
        assert_eq!(state.take_shell_batch_or_end_revival("p"), None);
        assert!(!state.reviving_shell.contains("p"));
        assert!(state.reviving_shell.contains("q"));
        assert!(state.reviving_subagent.contains("q"));
        assert_eq!(state.pending_subagent["q"][0].subagent_agent_id, "sub-q");
        assert_eq!(state.pending_shell["q"][0].shell_id, "shell-q");
    }
}
