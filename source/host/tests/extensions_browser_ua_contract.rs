use std::cell::{Cell, RefCell};
use std::fs;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::browser_ua::{
    BrowserUaAuthApi, BrowserUaAuthRenewalEvent, BrowserUaExperimentsApi, BrowserUaHostLog,
    BROWSER_UA_DEPENDENCIES, browser_ua_extension_id, start_browser_ua_extension,
};
use mahayana_host_runtime::extensions::browser_ua::ua_owner_stamp_service::{
    UA_OWNER_STAMP_LENGTH, ua_owner_stamp_for_access_token,
};
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;

struct FakeAuth {
    token: RefCell<Option<String>>,
    listener: RefCell<Option<Rc<dyn Fn(BrowserUaAuthRenewalEvent)>>>,
    stops: Rc<Cell<usize>>,
}

impl BrowserUaAuthApi for FakeAuth {
    fn peek_access_token(&self) -> Option<String> {
        self.token.borrow().clone()
    }

    fn subscribe_to_renewal(
        &self,
        listener: Rc<dyn Fn(BrowserUaAuthRenewalEvent)>,
    ) -> Box<dyn FnOnce()> {
        self.listener.replace(Some(listener));
        let stops = Rc::clone(&self.stops);
        Box::new(move || stops.set(stops.get() + 1))
    }
}

impl FakeAuth {
    fn renew(&self, token: &str) {
        self.token.replace(Some(token.to_string()));
        if let Some(listener) = self.listener.borrow().as_ref() {
            listener(BrowserUaAuthRenewalEvent {
                outcome: "renewed".to_string(),
            });
        }
    }
}

struct FakeExperiments {
    enabled: Cell<bool>,
    listener: RefCell<Option<Rc<dyn Fn()>>>,
    stops: Rc<Cell<usize>>,
}

impl BrowserUaExperimentsApi for FakeExperiments {
    fn is_ua_token_kill_switch_enabled(&self) -> bool {
        self.enabled.get()
    }

    fn subscribe(&self, listener: Rc<dyn Fn()>) -> Box<dyn FnOnce()> {
        self.listener.replace(Some(listener));
        let stops = Rc::clone(&self.stops);
        Box::new(move || stops.set(stops.get() + 1))
    }
}

impl FakeExperiments {
    fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
        if let Some(listener) = self.listener.borrow().as_ref() {
            listener();
        }
    }
}

#[derive(Default)]
struct FakeLog {
    messages: RefCell<Vec<String>>,
}

impl BrowserUaHostLog for FakeLog {
    fn log(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_string());
    }
}

#[test]
fn browser_ua_extension_wires_auth_and_experiments_to_marker_services() {
    assert_eq!(browser_ua_extension_id(), HostExtensionId::BrowserUa);
    assert_eq!(
        BROWSER_UA_DEPENDENCIES,
        &[HostExtensionId::Auth, HostExtensionId::Experiments]
    );

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("fabushi-browser-ua-{suffix}"));
    fs::create_dir_all(&root).expect("create test root");
    let owner_path = root.join("owner");
    let disabled_path = root.join("disabled");

    let initial_token = "e30.eyJzdWIiOiJ1c2VyLTEyMyJ9.sig";
    let renewed_token = "e30.eyJzdWIiOiJ1c2VyLTQ1NiJ9.sig";
    let auth_stops = Rc::new(Cell::new(0usize));
    let experiment_stops = Rc::new(Cell::new(0usize));
    let auth = Rc::new(FakeAuth {
        token: RefCell::new(Some(initial_token.to_string())),
        listener: RefCell::new(None),
        stops: Rc::clone(&auth_stops),
    });
    let experiments = Rc::new(FakeExperiments {
        enabled: Cell::new(false),
        listener: RefCell::new(None),
        stops: Rc::clone(&experiment_stops),
    });
    let log = Rc::new(FakeLog::default());

    let runtime = start_browser_ua_extension(
        Rc::clone(&auth),
        Rc::clone(&experiments),
        Rc::clone(&log),
        Some(owner_path.clone()),
        Some(disabled_path.clone()),
    );

    let initial_stamp = ua_owner_stamp_for_access_token(initial_token).expect("initial stamp");
    assert_eq!(initial_stamp.len(), UA_OWNER_STAMP_LENGTH);
    assert_eq!(
        fs::read_to_string(&owner_path).expect("owner stamp"),
        format!("{initial_stamp}\n")
    );
    assert!(!disabled_path.exists());

    experiments.set_enabled(true);
    assert_eq!(fs::read_to_string(&disabled_path).expect("disabled marker"), "1\n");

    auth.renew(renewed_token);
    let renewed_stamp = ua_owner_stamp_for_access_token(renewed_token).expect("renewed stamp");
    assert_eq!(
        fs::read_to_string(&owner_path).expect("renewed owner stamp"),
        format!("{renewed_stamp}\n")
    );

    drop(runtime);
    assert_eq!(auth_stops.get(), 1);
    assert_eq!(experiment_stops.get(), 1);
    assert!(log.messages.borrow().is_empty());

    fs::remove_dir_all(root).expect("remove test root");
}
