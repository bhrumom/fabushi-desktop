use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::mcp::extension::{
    AuthRenewalEvent, AuthRenewalPort, MCP_EXTENSION_DEPENDENCIES, MCP_EXTENSION_ID,
    McpExtensionRuntime, McpHostServicePort, McpPluginSkillsService,
    PluginSkillsAuthenticatedStartup, PollingHandle, PollingPort,
};

#[derive(Default)]
struct AuthState {
    token: Option<String>,
    listener: Option<Arc<dyn Fn(AuthRenewalEvent) + Send + Sync>>,
    unsubscribed: usize,
}
struct Auth(Arc<Mutex<AuthState>>);
impl AuthRenewalPort for Auth {
    fn peek_access_token(&self) -> Option<String> {
        self.0.lock().unwrap().token.clone()
    }
    fn subscribe_to_renewal(
        &self,
        listener: Arc<dyn Fn(AuthRenewalEvent) + Send + Sync>,
    ) -> Box<dyn FnOnce() + Send> {
        self.0.lock().unwrap().listener = Some(listener);
        let state = self.0.clone();
        Box::new(move || state.lock().unwrap().unsubscribed += 1)
    }
}

#[derive(Default)]
struct SkillsState {
    syncs: Vec<String>,
    auth_changes: usize,
    disposals: usize,
}
struct Skills(Arc<Mutex<SkillsState>>);
impl McpPluginSkillsService for Skills {
    fn sync(&self, trigger: &str) -> Result<(), String> {
        self.0.lock().unwrap().syncs.push(trigger.into());
        Ok(())
    }
    fn handle_auth_change(&self) {
        self.0.lock().unwrap().auth_changes += 1;
    }
    fn dispose(&self) {
        self.0.lock().unwrap().disposals += 1;
    }
}

#[derive(Default)]
struct PollState {
    callback: Option<Arc<dyn Fn() + Send + Sync>>,
    disposed: usize,
}
struct Handle(Arc<Mutex<PollState>>);
impl PollingHandle for Handle {
    fn dispose(&mut self) {
        self.0.lock().unwrap().disposed += 1;
    }
}
struct Poll(Arc<Mutex<PollState>>);
impl PollingPort for Poll {
    fn start(&self, callback: Arc<dyn Fn() + Send + Sync>) -> Box<dyn PollingHandle> {
        self.0.lock().unwrap().callback = Some(callback);
        Box::new(Handle(self.0.clone()))
    }
}

#[derive(Default)]
struct Service(Arc<Mutex<usize>>);
impl McpHostServicePort for Service {
    fn dispose(&self) -> Result<(), String> {
        *self.0.lock().unwrap() += 1;
        Ok(())
    }
}

#[test]
fn frozen_identity_and_dependencies_are_preserved() {
    assert_eq!(MCP_EXTENSION_ID, "mcp");
    assert_eq!(
        MCP_EXTENSION_DEPENDENCIES,
        [
            "auth",
            "experiments",
            "forever-box",
            "settings",
            "telemetry"
        ]
    );
}

#[test]
fn authentication_gates_startup_and_first_success_runs_legacy_sweep() {
    let auth_state = Arc::new(Mutex::new(AuthState::default()));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let poll_state = Arc::new(Mutex::new(PollState::default()));
    let swept = Arc::new(Mutex::new(0usize));
    let startup = PluginSkillsAuthenticatedStartup::start(
        Arc::new(Auth(auth_state.clone())),
        Arc::new(Skills(skills_state.clone())),
        Arc::new(Poll(poll_state.clone())),
        Some({
            let swept = swept.clone();
            Arc::new(move || *swept.lock().unwrap() += 1)
        }),
    );
    assert!(!startup.is_started());
    auth_state.lock().unwrap().token = Some("token".into());
    let renewal = auth_state.lock().unwrap().listener.clone().unwrap();
    renewal(AuthRenewalEvent {
        outcome: "renewed".into(),
        is_first_credential: true,
    });
    assert!(startup.is_started());

    let callback = poll_state.lock().unwrap().callback.clone().unwrap();
    callback();
    callback();
    assert_eq!(
        skills_state.lock().unwrap().syncs,
        vec!["startup", "refresh"]
    );
    assert_eq!(*swept.lock().unwrap(), 1);

    let renewal = auth_state.lock().unwrap().listener.clone().unwrap();
    renewal(AuthRenewalEvent {
        outcome: "renewed".into(),
        is_first_credential: true,
    });
    assert_eq!(skills_state.lock().unwrap().auth_changes, 1);
    startup.dispose();
    startup.dispose();
    assert_eq!(skills_state.lock().unwrap().disposals, 1);
    assert_eq!(poll_state.lock().unwrap().disposed, 1);
    assert_eq!(auth_state.lock().unwrap().unsubscribed, 1);
}

#[test]
fn extension_stop_disposes_startup_before_service_and_is_idempotent() {
    let auth_state = Arc::new(Mutex::new(AuthState {
        token: Some("token".into()),
        ..Default::default()
    }));
    let skills_state = Arc::new(Mutex::new(SkillsState::default()));
    let poll_state = Arc::new(Mutex::new(PollState::default()));
    let startup = PluginSkillsAuthenticatedStartup::start(
        Arc::new(Auth(auth_state)),
        Arc::new(Skills(skills_state.clone())),
        Arc::new(Poll(poll_state)),
        None,
    );
    let service = Arc::new(Service::default());
    let runtime = McpExtensionRuntime::new(startup, service.clone(), Arc::new(()));
    runtime.stop().unwrap();
    runtime.stop().unwrap();
    assert_eq!(skills_state.lock().unwrap().disposals, 1);
    assert_eq!(*service.0.lock().unwrap(), 1);
}
