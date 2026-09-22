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
