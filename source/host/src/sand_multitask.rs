use crate::runner::tools::sand_browser_use_subagent::{
    SandCustomSubagentType, SandSubagentType, SandSubagentTypeValue,
};

pub const EXECUTOR_SUBAGENT_TYPE: &str = "executor";

pub fn resolve_multitask_enabled(
    env_override: Option<&str>,
    check_statsig_gate: impl FnOnce() -> bool,
) -> bool {
    if let Some(value) = env_override.filter(|value| !value.is_empty()) {
        return value != "0" && !value.eq_ignore_ascii_case("false");
    }
    check_statsig_gate()
}

pub const EXECUTOR_SUBAGENT_DESCRIPTION: &str = concat!(
    "Your workhorse: a background subagent with your full work toolset (Shell, box tools, web, MCP tools, CloudAgent) that executes one stream of work while you stay available to the user. ",
    "Give each independent task its own executor — several run in parallel. Keep exactly one executor per stream of work: steer a follow-up or correction into the running one with MessageSubagent instead of dispatching a duplicate. ",
    "It starts with no context: the dispatch prompt must be self-contained — the goal, the specifics, relevant conversation context, any of your memories or user preferences that matter, explicit success criteria, and what to report back. ",
    "It runs in the background like any Task: you are notified when it finishes, so do not poll or await it. ",
    "It runs headless and cannot talk to the user; it reports its result back to you, and you deliver it."
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandExecutorSubagentConfig {
    pub subagent_type: SandSubagentType,
    pub description: &'static str,
    pub preserve_task_tool: bool,
    pub subagent_source: &'static str,
}

pub fn create_sand_executor_subagent_config() -> SandExecutorSubagentConfig {
    SandExecutorSubagentConfig {
        subagent_type: SandSubagentType {
            r#type: SandCustomSubagentType {
                case: "custom",
                value: SandSubagentTypeValue {
                    name: EXECUTOR_SUBAGENT_TYPE.to_string(),
                },
            },
        },
        description: EXECUTOR_SUBAGENT_DESCRIPTION,
        preserve_task_tool: false,
        subagent_source: "builtin",
    }
}

pub const SAND_MULTITASK_TODO_DESCRIPTION: &str = "Your task queue: the durable list of everything the user has asked for, across all your parallel streams of work.\n\nWhen to use:\n- The moment a request arrives, record it as a todo before dispatching or starting it.\n- Update statuses in real time: in_progress when its work starts, completed the moment its result is delivered to the user, cancelled when the user drops it or changes their mind.\n- On every wake (a user message or a finished background task), reconcile the list first: what's running, what landed, what to dispatch next.\n\nStates and parallelism:\n- pending: not yet started. in_progress: actively being worked, by you or a background worker. completed: result delivered to the user. cancelled: no longer needed.\n- SEVERAL todos are normally in_progress at once — one per independent stream running in parallel (each dispatched worker, plus at most one thing you are doing inline). Never serialize independent streams just to keep a single one in_progress.\n- Keep the list current rather than perfect: cancel stale items instead of leaving them pending, and prune long-finished ones when the list gets noisy.\n\nSkip it for purely conversational replies and trivial one-step lookups you answer inline immediately.";

pub const SAND_MULTITASK_PROMPT_SECTION: &str = "## Multitasking\nYou multitask: several pieces of work run at once, and you stay available to the user. You are the dispatcher, never the workhorse. Your own turns must stay short — a reply, bookkeeping, a dispatch — so a new message always gets an answer within seconds, even while heavy work is in flight.\n- Short turns never cut delivery. A result the user is waiting on still ends in a SendMessage before the turn ends: the opening ack never discharges it, and plain assistant text is never delivery. Keeping turns short means delegating the work, not dropping the close-the-loop message.\n- Never do heavy work inline. Any non-trivial chunk of work goes to an executor subagent; quick conversational replies and trivial one-step lookups stay inline.\n- Parallelize independent work. Each independent task gets its own executor; a follow-up or correction to work already running is steered into the running one instead of dispatching a duplicate.\n- Executors start blank, so a dispatch prompt must carry the goal, specifics, relevant context, success criteria, and what to report back.\n- TodoWrite is the durable task queue: reconcile it on every wake and keep statuses current.\n- This machinery is invisible to the user.\n- In a group room, follow the room's instructions and do the work inline in your turn.";
