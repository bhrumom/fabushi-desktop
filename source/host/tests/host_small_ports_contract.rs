use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use mahayana_host_runtime::r#box::box_env::{
    BoxEnvironmentControlClient, BoxEnvironmentUpdate, apply_box_environment_via_transport,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedEnvironmentCall {
    ctx: String,
    update: BoxEnvironmentUpdate,
}

struct RecordingTransport {
    calls: Rc<RefCell<Vec<RecordedEnvironmentCall>>>,
}

struct RecordingControlClient {
    calls: Rc<RefCell<Vec<RecordedEnvironmentCall>>>,
}

impl BoxEnvironmentControlClient<String> for RecordingControlClient {
    type Error = &'static str;

    fn update_environment_variables(
        &mut self,
        ctx: &String,
        request: BoxEnvironmentUpdate,
    ) -> Result<(), Self::Error> {
        self.calls.borrow_mut().push(RecordedEnvironmentCall {
            ctx: ctx.clone(),
            update: request,
        });
        Ok(())
    }
}

#[test]
fn box_environment_port_is_wired_and_forwards_a_cloned_update() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let transport = RecordingTransport {
        calls: Rc::clone(&calls),
    };
    let ctx = "request-context".to_string();
    let update = BoxEnvironmentUpdate {
        env: BTreeMap::from([
            ("FABUSHI_AGENT".to_string(), "enabled".to_string()),
            ("SHELL".to_string(), "/bin/zsh".to_string()),
        ]),
        replace: true,
    };
    let original = update.clone();

    apply_box_environment_via_transport(&ctx, &transport, &update, |transport| {
        RecordingControlClient {
            calls: Rc::clone(&transport.calls),
        }
    })
    .expect("box environment transport should succeed");

    assert_eq!(update, original, "the caller-owned update must remain unchanged");
    assert_eq!(
        calls.borrow().as_slice(),
        &[RecordedEnvironmentCall {
            ctx,
            update: original,
        }]
    );
}

#[test]
fn box_shell_command_builder_matches_grok_host_contract() {
    use mahayana_host_runtime::r#box::box_shell_command::{
        HostShellArgsInput, ShellCommandExecutable, ShellCommandParsingResult, build_host_shell_args,
    };

    let args = build_host_shell_args(HostShellArgsInput {
        command: "git status --short".to_string(),
        name: "git".to_string(),
        working_directory: "/workspace".to_string(),
        tool_call_id: "tool-42".to_string(),
    });

    assert_eq!(args.command, "git status --short");
    assert_eq!(args.working_directory, "/workspace");
    assert_eq!(args.tool_call_id, "tool-42");
    assert!(args.skip_approval);
    assert_eq!(
        args.parsing_result,
        ShellCommandParsingResult {
            parsing_failed: false,
            executable_commands: vec![ShellCommandExecutable {
                name: "git".to_string(),
                args: Vec::new(),
                full_text: "git status --short".to_string(),
            }],
            has_redirects: false,
            has_command_substitution: false,
        }
    );
}

#[test]
fn host_request_context_prefers_injected_timezone_and_normalizes_identity() {
    use mahayana_host_runtime::host_request_context::create_host_request_context;

    let provider = create_host_request_context(
        "/tmp/transcripts",
        || Some("Asia/Shanghai".to_string()),
        || vec!["rule-a", "rule-b"],
        || Some("  Gloria   Chan  ".to_string()),
    );

    let resolved = provider.resolve();
    assert_eq!(resolved.transcripts_folder, "/tmp/transcripts");
    assert_eq!(resolved.time_zone.as_deref(), Some("Asia/Shanghai"));
    assert_eq!(resolved.user_full_name.as_deref(), Some("Gloria Chan"));
    assert!(!resolved.os_version.trim().is_empty());
    assert_eq!(provider.resolve_rules(), vec!["rule-a", "rule-b"]);
}

#[test]
fn host_request_context_omits_blank_user_identity() {
    use mahayana_host_runtime::host_request_context::create_host_request_context;

    let provider = create_host_request_context(
        "/tmp/transcripts",
        || Some("UTC".to_string()),
        || Vec::<String>::new(),
        || Some("   ".to_string()),
    );

    assert_eq!(provider.resolve().user_full_name, None);
}

#[test]
fn ua_token_kill_switch_retries_failures_and_reconciles_marker() {
    use std::cell::Cell;
    use std::fs;
    use std::rc::Rc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use mahayana_host_runtime::extensions::browser_ua::ua_token_kill_switch_service::create_ua_token_kill_switch_reconciler;

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("fabushi-ua-kill-switch-{suffix}"));
    let marker = root.join("nested").join("disabled");
    let enabled = Rc::new(Cell::new(true));
    let log_count = Rc::new(Cell::new(0usize));

    let enabled_for_reconciler = Rc::clone(&enabled);
    let logs_for_reconciler = Rc::clone(&log_count);
    let mut reconcile = create_ua_token_kill_switch_reconciler(
        Some(marker.clone()),
        move || enabled_for_reconciler.get(),
        move |_message| logs_for_reconciler.set(logs_for_reconciler.get() + 1),
    );

    reconcile.reconcile();
    assert_eq!(log_count.get(), 1);
    assert_eq!(reconcile.last_applied(), None);

    fs::create_dir_all(marker.parent().expect("marker parent")).expect("create marker parent");
    reconcile.reconcile();
    assert_eq!(fs::read_to_string(&marker).expect("marker text"), "1\n");
    assert_eq!(reconcile.last_applied(), Some(true));

    enabled.set(false);
    reconcile.reconcile();
    assert!(!marker.exists());
    assert_eq!(reconcile.last_applied(), Some(false));

    fs::remove_dir_all(root).expect("remove UA test root");
}
