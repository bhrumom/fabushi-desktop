use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use thiserror::Error;

use crate::host_secret_store::get_or_create_host_machine_id;

use super::credential_renewer::{
    CredentialRenewalBackend, CredentialRenewerHooks, HttpCredentialRenewalBackend,
    InferenceCredential, RenewalOutcome, RenewalResult, SAND_DEV_INFERENCE_TOKEN_FILE_ENV,
    SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV, SandCredentialRenewalError,
    SandInferenceCredentialRenewer, get_configured_backend_url,
    read_dev_inference_credential_file,
};

pub const EXPIRY_LEEWAY_MS: u64 = 30_000;
pub const FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV: &str = "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE";
pub const SAND_SHORTLIVED_CREDS_WAITING_MESSAGE: &str =
    "Waiting for an inference credential. Fabushi's computer renews this automatically (no desktop required); this resolves on its own shortly.";

pub const FABUSHI_CORE_CREDENTIAL_WAITING_MESSAGE: &str =
    "Fabushi first-party Core credential is unavailable or expired.";

#[derive(Clone, PartialEq, Eq)]
pub struct FabushiCoreCredential {
    access_token: String,
    issuer: String,
    pub expires_at_ms: u64,
    pub session_id: String,
    pub device_id: String,
    pub user_id: String,
}

/// Non-secret account/session identity from the same validated credential image.
/// Token rotation and expiry changes do not create a new context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FabushiCredentialContext {
    issuer: String,
    user_id: String,
    device_id: String,
    session_id: String,
}

impl std::fmt::Debug for FabushiCoreCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FabushiCoreCredential")
            .field("access_token", &"<redacted>")
            .field("issuer", &self.issuer)
            .field("expires_at_ms", &self.expires_at_ms)
            .field("session_id", &self.session_id)
            .field("device_id", &self.device_id)
            .field("user_id", &self.user_id)
            .finish()
    }
}

impl FabushiCoreCredential {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub(crate) fn context(&self) -> FabushiCredentialContext {
        FabushiCredentialContext {
            issuer: self.issuer.clone(),
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
        }
    }

    pub fn authorize_origin(&self, target: &str) -> Result<(), FabushiCoreCredentialError> {
        let validated = super::credential_renewer::configured_product_backend_url(Some(target), None)
            .map_err(|_| FabushiCoreCredentialError::unavailable())?;
        let target = url::Url::parse(&validated).map_err(|_| FabushiCoreCredentialError::unavailable())?;
        if target.origin().ascii_serialization() != self.issuer {
            return Err(FabushiCoreCredentialError::unavailable());
        }
        Ok(())
    }
}

pub trait FabushiCoreCredentialProvider: Send + Sync {
    fn get_fabushi_core_credential(
        &self,
    ) -> Result<FabushiCoreCredential, FabushiCoreCredentialError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{message}")]
pub struct FabushiCoreCredentialError {
    message: String,
}

impl FabushiCoreCredentialError {
    fn unavailable() -> Self {
        Self {
            message: FABUSHI_CORE_CREDENTIAL_WAITING_MESSAGE.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FabushiCoreCredentialFile {
    version: u32,
    #[serde(default)]
    issuer: Option<String>,
    access_token: String,
    expires_at_ms: u64,
    session_id: String,
    device_id: String,
    user_id: serde_json::Value,
}

/// Decode one Native account-owner document. Callers retain their own file and
/// clock policy; malformed document values never enter error diagnostics.
pub(crate) fn parse_fabushi_core_credential(
    raw: &str,
    now_ms: impl FnOnce() -> u64,
) -> Result<FabushiCoreCredential, FabushiCoreCredentialError> {
    let parsed: FabushiCoreCredentialFile =
        serde_json::from_str(raw).map_err(|_| FabushiCoreCredentialError::unavailable())?;
    if parsed.version != 1
        || parsed.access_token.trim().is_empty()
        || parsed.session_id.trim().is_empty()
        || parsed.device_id.trim().is_empty()
        || now_ms() >= parsed.expires_at_ms.saturating_sub(EXPIRY_LEEWAY_MS)
    {
        return Err(FabushiCoreCredentialError::unavailable());
    }
    let user_id = match parsed.user_id {
        serde_json::Value::String(value) if !value.trim().is_empty() => value,
        serde_json::Value::Number(value) => value.to_string(),
        _ => return Err(FabushiCoreCredentialError::unavailable()),
    };
    let issuer = super::credential_renewer::configured_product_backend_url(
        parsed.issuer.as_deref(), None,
    ).map_err(|_| FabushiCoreCredentialError::unavailable())?;
    let issuer = url::Url::parse(&issuer)
        .map_err(|_| FabushiCoreCredentialError::unavailable())?
        .origin().ascii_serialization();
    Ok(FabushiCoreCredential {
        access_token: parsed.access_token,
        issuer,
        expires_at_ms: parsed.expires_at_ms,
        session_id: parsed.session_id,
        device_id: parsed.device_id,
        user_id,
    })
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{message}")]
pub struct SandCredentialsWaitingError {
    message: String,
}

impl SandCredentialsWaitingError {
    pub fn new() -> Self {
        Self {
            message: SAND_SHORTLIVED_CREDS_WAITING_MESSAGE.into(),
        }
    }
}

impl Default for SandCredentialsWaitingError {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct InferenceCredentialStore {
    credential: Mutex<Option<InferenceCredential>>,
}

impl InferenceCredentialStore {
    pub fn set_credential(&self, credential: InferenceCredential) {
        if credential.access_token.is_empty() {
            return;
        }
        *self
            .credential
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(credential);
    }

    pub fn get_valid_access_token(&self, observed_now_ms: u64) -> Option<String> {
        let guard = self
            .credential
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let credential = guard.as_ref()?;
        (observed_now_ms < credential.expires_at_ms.saturating_sub(EXPIRY_LEEWAY_MS))
            .then(|| credential.access_token.clone())
    }

    pub fn has_valid_credential(&self, observed_now_ms: u64) -> bool {
        self.get_valid_access_token(observed_now_ms).is_some()
    }

    pub fn clear(&self) {
        *self
            .credential
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostAuthRenewalEvent {
    pub result: RenewalResult,
    pub is_first_credential: bool,
}

pub type RenewalListener = Arc<dyn Fn(HostAuthRenewalEvent) + Send + Sync>;
pub type AuthLog = Arc<dyn Fn(&str) + Send + Sync>;
pub type MachineIdProvider =
    Arc<dyn Fn() -> Result<String, std::io::Error> + Send + Sync>;

struct DevFileCredentialBackend;

impl CredentialRenewalBackend for DevFileCredentialBackend {
    fn renew(&self, credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError> {
        read_dev_inference_credential_file(Path::new(credential))
    }
}

#[derive(Debug, Clone)]
pub struct HostAuthEnvironment {
    values: BTreeMap<String, String>,
}

impl HostAuthEnvironment {
    pub fn from_process_env() -> Self {
        Self {
            values: env::vars().collect(),
        }
    }

    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            values: pairs.into_iter().collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

pub struct HostAuthServiceOptions {
    pub environment: HostAuthEnvironment,
    pub backend_url: Option<String>,
    pub backend: Option<Arc<dyn CredentialRenewalBackend>>,
    pub machine_id: MachineIdProvider,
    pub log: AuthLog,
    pub now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
}

impl HostAuthServiceOptions {
    pub fn production(log: impl Fn(&str) + Send + Sync + 'static) -> Result<Self, SandCredentialRenewalError> {
        Ok(Self {
            environment: HostAuthEnvironment::from_process_env(),
            backend_url: Some(get_configured_backend_url()?),
            backend: None,
            machine_id: Arc::new(|| get_or_create_host_machine_id(None)),
            log: Arc::new(log),
            now_ms: Arc::new(super::credential_renewer::system_now_ms),
        })
    }
}

pub struct HostAuthService {
    store: Arc<InferenceCredentialStore>,
    listeners: Arc<Mutex<BTreeMap<u64, RenewalListener>>>,
    next_listener_id: AtomicU64,
    last_renewal_event: Arc<Mutex<Option<HostAuthRenewalEvent>>>,
    renewer: Mutex<Option<SandInferenceCredentialRenewer>>,
    has_renewal_credential: bool,
    product_credential_file: Option<PathBuf>,
    machine_id: MachineIdProvider,
    now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
    log: AuthLog,
}

impl FabushiCoreCredentialProvider for HostAuthService {
    fn get_fabushi_core_credential(
        &self,
    ) -> Result<FabushiCoreCredential, FabushiCoreCredentialError> {
        HostAuthService::get_fabushi_core_credential(self)
    }
}

impl HostAuthService {
    pub fn new(options: HostAuthServiceOptions) -> Result<Self, SandCredentialRenewalError> {
        let dev_token_file = options
            .environment
            .get(SAND_DEV_INFERENCE_TOKEN_FILE_ENV)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let product_token_file = options
            .environment
            .get(FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let renewal_credential = options
            .environment
            .get(SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        // Fabushi account credentials are scoped to the first-party Fabushi API and
        // Responses transport. They are not Cursor/Sand Connect credentials. Treating
        // FABUSHI_HOST_ACCESS_CREDENTIAL_FILE as a HostAuth renewal source sends a
        // valid Fabushi bearer token to Cursor endpoints and produces misleading
        // ERROR_NOT_LOGGED_IN failures on every turn.
        let has_renewal_credential =
            dev_token_file.is_some() || renewal_credential.is_some();

        let backend: Arc<dyn CredentialRenewalBackend> = match options.backend {
            Some(backend) => backend,
            None if dev_token_file.is_some() => Arc::new(DevFileCredentialBackend),
            None => Arc::new(HttpCredentialRenewalBackend::new(
                options
                    .backend_url
                    .clone()
                    .unwrap_or_else(|| get_configured_backend_url().unwrap_or_else(|_| super::credential_renewer::DEFAULT_CURSOR_BACKEND_URL.into())),
            )),
        };

        let source_value = dev_token_file
            .clone()
            .or_else(|| renewal_credential.clone());
        let store = Arc::new(InferenceCredentialStore::default());
        let listeners = Arc::new(Mutex::new(BTreeMap::<u64, RenewalListener>::new()));
        let last_renewal_event = Arc::new(Mutex::new(None));
        let wrote_first_credential = Arc::new(AtomicBool::new(false));

        let source = Arc::new(move || source_value.clone());
        let store_for_set = Arc::clone(&store);
        let now_for_set = Arc::clone(&options.now_ms);
        let first_for_set = Arc::clone(&wrote_first_credential);
        let set_credential = Arc::new(move |credential: InferenceCredential| {
            first_for_set.store(
                !store_for_set.has_valid_credential(now_for_set()),
                Ordering::Release,
            );
            store_for_set.set_credential(credential);
        });

        let listeners_for_result = Arc::clone(&listeners);
        let event_for_result = Arc::clone(&last_renewal_event);
        let first_for_result = Arc::clone(&wrote_first_credential);
        let log_for_result = Arc::clone(&options.log);
        let on_result = Arc::new(move |result: RenewalResult| {
            let is_first_credential = result.outcome == RenewalOutcome::Renewed
                && first_for_result.swap(false, Ordering::AcqRel);
            if result.outcome == RenewalOutcome::Failed {
                log_for_result(&format!(
                    "inference-credential renewal failed (streak {}): {}",
                    result.consecutive_failures,
                    result.error_summary.as_deref().unwrap_or("unknown error"),
                ));
            }
            let event = HostAuthRenewalEvent {
                result,
                is_first_credential,
            };
            *event_for_result
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(event.clone());
            let snapshot = listeners_for_result
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .values()
                .cloned()
                .collect::<Vec<_>>();
            for listener in snapshot {
                listener(event.clone());
            }
        });

        let mut renewer = SandInferenceCredentialRenewer::new(
            backend,
            CredentialRenewerHooks {
                get_credential: source,
                set_credential,
                on_result: Some(on_result),
                now_ms: Arc::clone(&options.now_ms),
            },
        );
        renewer.start();

        if let Some(path) = dev_token_file {
            (options.log)(&format!(
                "DEV inference-credential renewer started, reading short-lived tokens from {path} (dev:box-docker local loop)"
            ));
        } else if product_token_file.is_some() && !has_renewal_credential {
            (options.log)(
                "Fabushi account credential is reserved for first-party Fabushi account/Responses transport; Cursor/Sand HostAuth remains unavailable without a Cursor renewal credential",
            );
        } else if has_renewal_credential {
            (options.log)(
                "inference-credential renewer started (backend self-renewal is the sole inference-credential source)",
            );
        } else {
            (options.log)(
                "inference-credential renewer started, but no renewal credential was delivered into the box; inference is unavailable until the box is re-provisioned with one",
            );
        }

        Ok(Self {
            store,
            listeners,
            next_listener_id: AtomicU64::new(1),
            last_renewal_event,
            renewer: Mutex::new(Some(renewer)),
            has_renewal_credential,
            product_credential_file: product_token_file.map(PathBuf::from),
            machine_id: options.machine_id,
            now_ms: options.now_ms,
            log: options.log,
        })
    }

    pub fn production(
        log: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<Self, SandCredentialRenewalError> {
        Self::new(HostAuthServiceOptions::production(log)?)
    }

    pub fn get_access_token(&self) -> Result<String, SandCredentialsWaitingError> {
        if let Some(token) = self.peek_access_token() {
            return Ok(token);
        }

        if self.has_renewal_credential {
            let renewed = self
                .renewer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .is_some_and(SandInferenceCredentialRenewer::request_immediate_renewal);
            if renewed {
                if let Some(token) = self.peek_access_token() {
                    return Ok(token);
                }
            }
        }

        Err(SandCredentialsWaitingError::new())
    }

    pub fn peek_access_token(&self) -> Option<String> {
        self.store.get_valid_access_token((self.now_ms)())
    }

    pub fn get_last_renewal_event(&self) -> Option<HostAuthRenewalEvent> {
        self.last_renewal_event
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn get_machine_id(&self) -> Result<String, std::io::Error> {
        (self.machine_id)()
    }

    /// Read the current first-party Fabushi product credential on demand.
    ///
    /// The Electron account owner atomically replaces this file on refresh/account
    /// switch. Host deliberately does not cache it so an old account identity cannot
    /// survive a replacement. This credential is never a Cursor/Sand renewal source.
    pub fn get_fabushi_core_credential(
        &self,
    ) -> Result<FabushiCoreCredential, FabushiCoreCredentialError> {
        let path = self
            .product_credential_file
            .as_deref()
            .ok_or_else(FabushiCoreCredentialError::unavailable)?;
        let raw = fs::read_to_string(path).map_err(|_| FabushiCoreCredentialError::unavailable())?;
        parse_fabushi_core_credential(&raw, || (self.now_ms)())
    }

    pub fn subscribe_to_renewal(&self, listener: RenewalListener) -> u64 {
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(id, listener);
        id
    }

    pub fn unsubscribe_from_renewal(&self, id: u64) -> bool {
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&id)
            .is_some()
    }

    pub fn dispose(&self) {
        if let Some(mut renewer) = self
            .renewer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            renewer.close();
        }
        self.listeners
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    pub fn has_renewal_credential(&self) -> bool {
        self.has_renewal_credential
    }

    pub fn log(&self, message: &str) {
        (self.log)(message);
    }
}

impl Drop for HostAuthService {
    fn drop(&mut self) {
        if let Ok(slot) = self.renewer.get_mut() {
            if let Some(mut renewer) = slot.take() {
                renewer.close();
            }
        }
    }
}

#[cfg(test)]
mod worker_exit_tests {
    use super::super::credential_renewer::RenewerTestControl;
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
    use std::sync::TryLockError;
    use std::thread;
    use std::time::Duration;

    const DEADLINE: Duration = Duration::from_secs(5);
    const INITIAL_EXPIRY_MS: u64 = 600_000;
    const STORED_TOKEN: &str = "credential-stored-before-listener-panic";

    struct GatedBackend {
        calls: AtomicUsize,
        entered: Sender<usize>,
        release: Mutex<Receiver<()>>,
    }

    impl CredentialRenewalBackend for GatedBackend {
        fn renew(&self, credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError> {
            assert_eq!(credential, "worker-exit-renewal-fixture");
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = self.entered.send(call);
            let released = self.release.lock().unwrap().recv_timeout(DEADLINE);
            released.expect("release the owned auth backend attempt");
            Ok(match call {
                1 => InferenceCredential {
                    access_token: "initial-credential".into(),
                    expires_at_ms: INITIAL_EXPIRY_MS,
                },
                2 => InferenceCredential {
                    access_token: STORED_TOKEN.into(),
                    expires_at_ms: 1_200_000,
                },
                _ => panic!("a terminated auth worker must not renew again"),
            })
        }
    }

    struct AuthWorkerCleanup<'a> {
        service: &'a HostAuthService,
        control: &'a RenewerTestControl,
        backend_release: &'a Sender<()>,
        listener_release: &'a Sender<()>,
    }

    impl Drop for AuthWorkerCleanup<'_> {
        fn drop(&mut self) {
            // This runs before thread::scope joins its requesters, including
            // when a deadline assertion fails with the exit guard removed.
            // Cleanup never supplies a successful observation to the test.
            self.control.close_waiters();
            let _ = self.backend_release.send(());
            let _ = self.listener_release.send(());
            self.service.dispose();
        }
    }

    #[test]
    fn listener_panic_releases_auth_waiter_preserves_token_and_allows_disposal() {
        let (backend_entered_tx, backend_entered) = mpsc::channel();
        let (backend_release, backend_release_rx) = mpsc::channel();
        let backend = Arc::new(GatedBackend {
            calls: AtomicUsize::new(0),
            entered: backend_entered_tx,
            release: Mutex::new(backend_release_rx),
        });
        let now_ms = Arc::new(AtomicU64::new(10_000));
        let clock = Arc::clone(&now_ms);
        let service = Arc::new(
            HostAuthService::new(HostAuthServiceOptions {
                environment: HostAuthEnvironment::from_pairs([(
                    SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV.into(),
                    "worker-exit-renewal-fixture".into(),
                )]),
                backend_url: Some("http://127.0.0.1:9".into()),
                backend: Some(backend.clone()),
                machine_id: Arc::new(|| Ok("worker-exit-machine".into())),
                log: Arc::new(|_| {}),
                now_ms: Arc::new(move || clock.load(Ordering::SeqCst)),
            })
            .expect("construct the real Host Auth owner"),
        );
        // The real get_access_token waiter retains service.renewer. Capture
        // this observation/cleanup handle before that invocation owns it.
        let control = service
            .renewer
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .test_control();
        let (events_tx, events) = mpsc::channel();
        let (listener_release, listener_release_rx) = mpsc::channel();
        let listener_release_rx = Mutex::new(listener_release_rx);
        let callback_calls = Arc::new(AtomicUsize::new(0));
        let intentional_panic = Arc::new(AtomicBool::new(false));

        thread::scope(|scope| {
            let _cleanup = AuthWorkerCleanup {
                service: &service,
                control: &control,
                backend_release: &backend_release,
                listener_release: &listener_release,
            };
            let calls = Arc::clone(&callback_calls);
            let panic_released = Arc::clone(&intentional_panic);
            service.subscribe_to_renewal(Arc::new(move |event| {
                let call = calls.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = events_tx.send((call, event));
                if call == 2 {
                    let released = listener_release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(DEADLINE);
                    released.expect("release the owned renewal listener fault");
                    panic_released.store(true, Ordering::SeqCst);
                    panic!("intentional subscribed Host Auth listener panic");
                }
            }));

            assert_eq!(backend_entered.recv_timeout(DEADLINE).unwrap(), 1);
            backend_release.send(()).unwrap();
            let (first_call, first_event) = events.recv_timeout(DEADLINE).unwrap();
            assert_eq!(first_call, 1);
            assert_eq!(first_event.result.outcome, RenewalOutcome::Renewed);
            assert_eq!(first_event.result.consecutive_failures, 0);
            assert!(first_event.is_first_credential);
            // Publication precedes idle; only the real state predicate proves
            // that the following immediate-renewal request can be accepted.
            assert!(control.wait_until_waiting(DEADLINE));
            assert_eq!(control.snapshot(), (false, true, 0, 0));
            assert_eq!(service.peek_access_token(), Some("initial-credential".into()));

            // The worker already selected its long ordinary refresh delay.
            // Advancing the injected clock now expires the initial credential
            // without waiting for a timer or clearing the real credential store.
            now_ms.store(INITIAL_EXPIRY_MS - EXPIRY_LEEWAY_MS, Ordering::SeqCst);
            assert!(service.peek_access_token().is_none());
            let request_service = Arc::clone(&service);
            let (request_tx, request_result) = mpsc::channel();
            let requester = scope.spawn(move || {
                let _ = request_tx.send(request_service.get_access_token());
            });

            assert_eq!(backend_entered.recv_timeout(DEADLINE).unwrap(), 2);
            assert_eq!(control.snapshot(), (false, false, 1, 0));
            assert!(matches!(
                service.renewer.try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            assert!(matches!(request_result.try_recv(), Err(TryRecvError::Empty)));
            backend_release.send(()).unwrap();
            let (fault_call, fault_event) = events.recv_timeout(DEADLINE).unwrap();
            assert_eq!(fault_call, 2);
            assert_eq!(fault_event.result.outcome, RenewalOutcome::Renewed);
            assert_eq!(fault_event.result.consecutive_failures, 0);
            assert!(fault_event.result.error_summary.is_none());
            assert!(fault_event.is_first_credential);
            assert_eq!(service.peek_access_token(), Some(STORED_TOKEN.into()));
            assert_eq!(service.get_last_renewal_event(), Some(fault_event.clone()));
            assert_eq!(control.snapshot(), (false, false, 1, 0));
            assert!(matches!(request_result.try_recv(), Err(TryRecvError::Empty)));

            listener_release.send(()).unwrap();
            let waiting_result = request_result
                .recv_timeout(DEADLINE)
                .expect("the real Auth waiter must settle after its renewal worker unwinds");
            requester.join().expect("join the owned Auth requester");
            assert!(intentional_panic.load(Ordering::SeqCst));
            assert_eq!(waiting_result, Err(SandCredentialsWaitingError::new()));
            assert_eq!(control.snapshot(), (true, false, 1, 0));
            assert_eq!(service.peek_access_token(), Some(STORED_TOKEN.into()));

            // A later ordinary call may use the already stored valid token.
            // Its successful fast path does not retroactively complete epoch 1
            // or turn the earlier waiting invocation into a success.
            let later_service = Arc::clone(&service);
            let (disposed_tx, disposed) = mpsc::channel();
            let lifecycle = scope.spawn(move || {
                let token = later_service.get_access_token();
                later_service.dispose();
                later_service.dispose();
                let _ = disposed_tx.send(token);
            });
            let later_result = disposed
                .recv_timeout(DEADLINE)
                .expect("the later fast path and repeated Auth disposal must finish");
            lifecycle.join().expect("join the owned Auth disposal caller");
            assert_eq!(later_result, Ok(STORED_TOKEN.into()));
            assert_eq!(service.peek_access_token(), Some(STORED_TOKEN.into()));
            assert_eq!(service.get_last_renewal_event(), Some(fault_event));
            assert!(service.renewer.lock().unwrap().is_none());
            assert!(service.listeners.lock().unwrap().is_empty());
            assert_eq!(control.snapshot(), (true, false, 1, 0));
            assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
            assert_eq!(callback_calls.load(Ordering::SeqCst), 2);
            assert!(events.try_recv().is_err(), "listeners must not be replayed");
        });
    }
}
