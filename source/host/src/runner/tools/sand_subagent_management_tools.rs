use crate::runner::subagent_runtime::{
    ControlResult, RunningSubagentInfo, SubagentRuntime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteerReview {
    pub allowed: bool,
    pub reason: String,
}

pub fn elapsed_label(elapsed_ms: u64) -> String {
    let total_seconds = elapsed_ms.saturating_add(500) / 1_000;
    if total_seconds < 90 {
        return format!("{total_seconds}s");
    }
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    if seconds == 0 {
        format!("{minutes}m")
    } else {
        format!("{minutes}m {seconds}s")
    }
}

pub fn describe_running_subagent(info: &RunningSubagentInfo, detailed: bool) -> String {
    let header = format!(
        "- {} [{}] \"{}\" — running for {}, {} tool call(s)",
        info.subagent_id,
        info.subagent_type,
        info.title,
        elapsed_label(info.elapsed_ms),
        info.tool_call_count,
    );
    if !detailed {
        return header;
    }
    let mut lines = vec![header];
    if info.recent_activity.is_empty() {
        lines.push("  No tool activity recorded yet.".to_string());
    } else {
        lines.push("  Recent activity (oldest → newest):".to_string());
        lines.extend(
            info.recent_activity
                .iter()
                .map(|entry| format!("    {entry}")),
        );
    }
    if let Some(path) = info.transcript_path.as_deref() {
        lines.push(format!(
            "  Full transcript (read it for the complete play-by-play): {path}"
        ));
    }
    lines.join("\n")
}

pub fn not_running_message(
    subagent_id: &str,
    running: &[RunningSubagentInfo],
) -> String {
    let base = format!(
        "No subagent \"{subagent_id}\" is currently running. It may have already finished (you're revived automatically with a finished subagent's result), or the id is wrong."
    );
    if running.is_empty() {
        format!("{base} No subagents are running right now.")
    } else {
        format!(
            "{base} Currently running: {}.",
            running
                .iter()
                .map(|info| info.subagent_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub fn check_subagent(
    runtime: &SubagentRuntime,
    subagent_id: Option<&str>,
    now_ms: u64,
) -> String {
    let id = subagent_id.map(str::trim).filter(|id| !id.is_empty());
    let running = runtime.list_running_subagents(now_ms);
    let Some(id) = id else {
        if running.is_empty() {
            return "No background subagents are running right now.".to_string();
        }
        let mut lines = vec![format!("{} subagent(s) running:", running.len())];
        lines.extend(
            running
                .iter()
                .map(|info| describe_running_subagent(info, false)),
        );
        lines.push(
            "Pass a subagent_id to see its recent activity and transcript path.".to_string(),
        );
        return lines.join("\n");
    };
    runtime
        .get_running_subagent(id, now_ms)
        .map(|info| describe_running_subagent(&info, true))
        .unwrap_or_else(|| not_running_message(id, &running))
}

pub fn message_subagent(
    runtime: &mut SubagentRuntime,
    subagent_id: &str,
    message: &str,
    review: Option<&SteerReview>,
    now_ms: u64,
) -> String {
    if let Some(review) = review.filter(|review| !review.allowed) {
        return review.reason.clone();
    }
    if matches!(
        runtime.steer_subagent(subagent_id, message),
        ControlResult::NotRunning
    ) {
        return not_running_message(
            subagent_id,
            &runtime.list_running_subagents(now_ms),
        );
    }
    format!(
        "Message delivered to subagent {subagent_id}. It will interrupt what it's doing, take your message into account, and keep working. You'll be revived with its result when it finishes — don't wait on it."
    )
}

pub fn stop_subagent(
    runtime: &mut SubagentRuntime,
    subagent_id: &str,
    now_ms: u64,
) -> String {
    if matches!(
        runtime.abort_subagent(subagent_id),
        ControlResult::NotRunning
    ) {
        return not_running_message(
            subagent_id,
            &runtime.list_running_subagents(now_ms),
        );
    }
    format!(
        "Stopping subagent {subagent_id}. It will be torn down and won't report back."
    )
}
