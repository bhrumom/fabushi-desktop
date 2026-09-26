use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::extensions::auth::auth_service::{
    EXPIRY_LEEWAY_MS, HostAuthEnvironment, HostAuthService, HostAuthServiceOptions,
    InferenceCredentialStore, SAND_SHORTLIVED_CREDS_WAITING_MESSAGE,
};
use mahayana_host_runtime::extensions::auth::credential_renewer::{
    CredentialRenewalBackend, InferenceCredential, SandCredentialRenewalError,
    SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV,
};

struct StaticBackend {
    token: String,
    expires_at_ms: u64,
}

impl CredentialRenewalBackend for StaticBackend {
    fn renew(&self, credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError> {
        assert_eq!(credential, "renewal-secret");
        // Grok's renewal backend is async: even an immediately-resolved Promise
        // yields before onResult fires, so callers can subscribe after service
        // construction without racing the first renewal event. Preserve that
        // scheduling boundary in this synchronous Rust test double.
        thread::sleep(Duration::from_millis(5));
        Ok(InferenceCredential {
            access_token: self.token.clone(),
            expires_at_ms: self.expires_at_ms,
        })
    }
}

fn options(
    env: BTreeMap<String, String>,
    now: Arc<Mutex<u64>>,
    logs: Arc<Mutex<Vec<String>>>,
    backend: Arc<dyn CredentialRenewalBackend>,
) -> HostAuthServiceOptions {
    let now_for_clock = Arc::clone(&now);
    let logs_for_sink = Arc::clone(&logs);
    HostAuthServiceOptions {
        environment: HostAuthEnvironment::from_pairs(env),
        backend_url: Some("http://127.0.0.1:9".into()),
        backend: Some(backend),
        machine_id: Arc::new(|| Ok("machine-test".into())),
        log: Arc::new(move |message| {
            logs_for_sink.lock().unwrap().push(message.to_string());
        }),
        now_ms: Arc::new(move || *now_for_clock.lock().unwrap()),
    }
}

#[test]
fn credential_store_honors_the_grok_expiry_leeway() {
    let store = InferenceCredentialStore::default();
    store.set_credential(InferenceCredential {
        access_token: "token".into(),
        expires_at_ms: 100_000,
    });
    assert_eq!(
        store.get_valid_access_token(100_000 - EXPIRY_LEEWAY_MS - 1),
        Some("token".into()),
    );
    assert_eq!(
        store.get_valid_access_token(100_000 - EXPIRY_LEEWAY_MS),
        None,
    );
    store.clear();
    assert!(!store.has_valid_credential(0));
}

#[test]
fn auth_service_fails_closed_without_a_renewal_source() {
    let now = Arc::new(Mutex::new(10_000));
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = HostAuthService::new(options(
        BTreeMap::new(),
        now,
        Arc::clone(&logs),
        Arc::new(StaticBackend {
            token: "unused".into(),
            expires_at_ms: 100_000,
        }),
    ))
    .unwrap();

    let error = service.get_access_token().unwrap_err();
    assert_eq!(error.to_string(), SAND_SHORTLIVED_CREDS_WAITING_MESSAGE);
    assert!(!service.has_renewal_credential());
    assert!(logs.lock().unwrap()[0].contains("no renewal credential"));
    service.dispose();
}

#[test]
fn auth_service_renews_stores_emits_first_credential_and_machine_id() {
    let now = Arc::new(Mutex::new(10_000));
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = HostAuthService::new(options(
        BTreeMap::from([(
            SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV.into(),
            "renewal-secret".into(),
        )]),
        Arc::clone(&now),
        Arc::clone(&logs),
        Arc::new(StaticBackend {
            token: "short-lived-token".into(),
            expires_at_ms: 600_000,
        }),
    ))
    .unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_sink = Arc::clone(&events);
    let subscription = service.subscribe_to_renewal(Arc::new(move |event| {
        events_sink.lock().unwrap().push(event);
    }));

    for _ in 0..100 {
        if service.peek_access_token().is_some()
            && service.get_last_renewal_event().is_some()
            && events.lock().unwrap().len() == 1
        {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    assert_eq!(service.get_access_token().unwrap(), "short-lived-token");
    assert_eq!(service.get_machine_id().unwrap(), "machine-test");
    let last = service.get_last_renewal_event().expect("renewal event");
    assert!(last.is_first_credential);
    assert_eq!(
        events.lock().unwrap().len(),
        1,
        "the subscribed first-renewal listener must observe exactly one event"
    );
    assert!(service.unsubscribe_from_renewal(subscription));
    assert!(logs.lock().unwrap()[0].contains("sole inference-credential source"));

    *now.lock().unwrap() = 600_000 - EXPIRY_LEEWAY_MS;
    assert!(service.peek_access_token().is_none());
    service.dispose();
}
