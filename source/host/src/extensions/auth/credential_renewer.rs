use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::blocking::Client;
use serde_json::Value;
use thiserror::Error;
use url::Url;

pub const SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV: &str = "SAND_INFERENCE_RENEWAL_CREDENTIAL";
pub const SAND_DEV_INFERENCE_TOKEN_FILE_ENV: &str = "SAND_DEV_INFERENCE_TOKEN_FILE";
pub const REFRESH_LEEWAY_MS: u64 = 2 * 60 * 1_000;
pub const MIN_REFRESH_INTERVAL_MS: u64 = 30 * 1_000;
pub const MAX_REFRESH_INTERVAL_MS: u64 = 30 * 60 * 1_000;
pub const CREDENTIAL_RETRY_BASE_DELAY_MS: u64 = 5 * 1_000;
pub const CREDENTIAL_RETRY_MAX_DELAY_MS: u64 = 5 * 60 * 1_000;
pub const DEFAULT_TTL_MS: u64 = 10 * 60 * 1_000;
pub const RENEWAL_PATH: &str = "/sand-box/inference-credential";
// Historical symbol retained for source compatibility; no foreign service fallback.
pub const DEFAULT_CURSOR_BACKEND_URL: &str = "https://api.ombhrum.com";
pub const SAND_CLIENT_TYPE: &str = "sand";
pub const SAND_CLIENT_FALLBACK_BASE_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceCredential {
    pub access_token: String,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenewalOutcome {
    Renewed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalResult {
    pub outcome: RenewalOutcome,
    pub consecutive_failures: u32,
    pub duration_ms: u64,
    pub error_summary: Option<String>,
}

#[derive(Debug, Error)]
pub enum SandCredentialRenewalError {
    #[error("{0}")]
    InvalidPayload(String),
    #[error("invalid backend URL: {0}")]
    InvalidBackendUrl(String),
    #[error("Sand inference-credential renewal request failed: {0}")]
    Transport(String),
    #[error("Sand inference-credential renewal failed (HTTP {0}).")]
    HttpStatus(u16),
    #[error("read inference credential file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("parse inference credential JSON: {0}")]
    ParseJson(#[from] serde_json::Error),
}

pub fn system_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub fn get_access_token_expiry_ms(token: &str) -> Option<u64> {
    let payload = token.split('.').nth(1)?;
    if payload.is_empty() {
        return None;
    }
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let parsed: Value = serde_json::from_slice(&decoded).ok()?;
    let exp = parsed.as_object()?.get("exp")?.as_f64()?;
    if !exp.is_finite() || exp < 0.0 {
        return None;
    }
    let millis = exp * 1_000.0;
    (millis <= u64::MAX as f64).then_some(millis as u64)
}

fn credential_from_payload(
    parsed: Value,
    absent_message: &str,
    observed_now_ms: u64,
) -> Result<InferenceCredential, SandCredentialRenewalError> {
    let object = parsed
        .as_object()
        .ok_or_else(|| SandCredentialRenewalError::InvalidPayload(absent_message.into()))?;
    let access_token = object
        .get("accessToken")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if access_token.is_empty() {
        return Err(SandCredentialRenewalError::InvalidPayload(absent_message.into()));
    }
    let expires_at_ms = object
        .get("expiresAtMs")
        .and_then(Value::as_u64)
        .or_else(|| get_access_token_expiry_ms(&access_token))
        .unwrap_or_else(|| observed_now_ms.saturating_add(DEFAULT_TTL_MS));
    Ok(InferenceCredential {
        access_token,
        expires_at_ms,
    })
}

pub fn read_dev_inference_credential_file(
    path: &Path,
) -> Result<InferenceCredential, SandCredentialRenewalError> {
    let raw = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<Value>(&raw)?;
    credential_from_payload(
        parsed,
        &format!("Dev inference token file {} has no accessToken yet.", path.display()),
        system_now_ms(),
    )
}

pub fn get_configured_backend_url() -> Result<String, SandCredentialRenewalError> {
    configured_product_backend_url(
        env::var("FABUSHI_API_BASE_URL").ok().as_deref(),
        env::var("SAND_BACKEND_URL").ok().as_deref(),
    )
}

pub fn configured_product_backend_url(
    native: Option<&str>,
    legacy: Option<&str>,
) -> Result<String, SandCredentialRenewalError> {
    let raw = native.or(legacy).unwrap_or(DEFAULT_CURSOR_BACKEND_URL).trim();
    let url = Url::parse(raw)
        .map_err(|_| SandCredentialRenewalError::InvalidBackendUrl("invalid product service origin".into()))?;
    let host = url.host_str().unwrap_or_default();
    let loopback = matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]");
    let approved = matches!(url.origin().ascii_serialization().as_str(),
        "https://api.ombhrum.com" | "https://fabushi-ci-isolated-auth.bhrumom.workers.dev");
    if (!url.username().is_empty() || url.password().is_some())
        || url.query().is_some() || url.fragment().is_some()
        || (url.scheme() != "https" && !(loopback && url.scheme() == "http"))
        || (!loopback && (!approved || url.path() != "/" || url.port().is_some()))
    {
        return Err(SandCredentialRenewalError::InvalidBackendUrl(
            "product services require an approved Fabushi origin or loopback".into(),
        ));
    }
    Ok(url.to_string())
}

pub fn sand_box_namespace() -> &'static str {
    match env::var("SAND_BOX_OWNER_NAMESPACE").ok().as_deref() {
        Some("dev") => "dev",
        Some("lab") => "lab",
        _ if env::var("SAND_PACKAGED").ok().as_deref() != Some("1") => "dev",
        _ if env::var("SAND_LAB").ok().as_deref() == Some("1") => "lab",
        _ => "prod",
    }
}

fn sand_client_base_version() -> String {
    let stamped = env::var("SAND_CLIENT_APP_VERSION").unwrap_or_default();
    let base = stamped.split('-').next().unwrap_or_default();
    let valid = {
        let parts = base.split('.').collect::<Vec<_>>();
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    };
    if valid {
        base.to_string()
    } else {
        SAND_CLIENT_FALLBACK_BASE_VERSION.into()
    }
}

pub fn sand_client_version() -> String {
    let base = sand_client_base_version();
    match sand_box_namespace() {
        "dev" => format!("{base}-dev"),
        "lab" => format!("{base}-lab"),
        _ => base,
    }
}

pub fn renew_sand_box_inference_credential(
    backend_url: &str,
    credential: &str,
) -> Result<InferenceCredential, SandCredentialRenewalError> {
    let backend = Url::parse(backend_url)
        .map_err(|error| SandCredentialRenewalError::InvalidBackendUrl(error.to_string()))?;
    let url = backend
        .join(RENEWAL_PATH)
        .map_err(|error| SandCredentialRenewalError::InvalidBackendUrl(error.to_string()))?;
    let response = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| SandCredentialRenewalError::Transport(error.to_string()))?
        .post(url)
        .header("content-type", "application/json")
        .header("x-cursor-client-type", SAND_CLIENT_TYPE)
        .header("x-cursor-client-version", sand_client_version())
        .header("x-sand-box-namespace", sand_box_namespace())
        .json(&serde_json::json!({ "credential": credential }))
        .send()
        .map_err(|error| SandCredentialRenewalError::Transport(error.to_string()))?;

    if !response.status().is_success() {
        return Err(SandCredentialRenewalError::HttpStatus(response.status().as_u16()));
    }
    let parsed = response
        .json::<Value>()
        .map_err(|error| SandCredentialRenewalError::Transport(error.to_string()))?;
    credential_from_payload(
        parsed,
        "Sand inference-credential renewal returned no token.",
        system_now_ms(),
    )
}

pub fn refresh_delay_ms(expires_at_ms: u64, observed_now_ms: u64) -> u64 {
    expires_at_ms
        .saturating_sub(observed_now_ms)
        .saturating_sub(REFRESH_LEEWAY_MS)
        .clamp(MIN_REFRESH_INTERVAL_MS, MAX_REFRESH_INTERVAL_MS)
}

pub fn retry_delay_ms(attempt: u32) -> u64 {
    let exponent = attempt.saturating_sub(1).min(31);
    let factor = 1_u64.checked_shl(exponent).unwrap_or(u64::MAX);
    CREDENTIAL_RETRY_BASE_DELAY_MS
        .saturating_mul(factor)
        .min(CREDENTIAL_RETRY_MAX_DELAY_MS)
}

pub fn redact_renewal_error_for_report(raw: &str) -> String {
    let mut words = Vec::new();
    for word in raw.split_whitespace() {
        let redacted = if word.starts_with("http://") || word.starts_with("https://") {
            "<url>".to_string()
        } else if word.starts_with('/') {
            "<path>".to_string()
        } else if word.len() >= 24
            && word
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            "<id>".to_string()
        } else {
            word.to_string()
        };
        words.push(redacted);
    }
    words.join(" ").chars().take(160).collect()
}

pub trait CredentialRenewalBackend: Send + Sync + 'static {
    fn renew(&self, credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError>;
}

#[derive(Debug, Clone)]
pub struct HttpCredentialRenewalBackend {
    backend_url: String,
}

impl HttpCredentialRenewalBackend {
    pub fn new(backend_url: impl Into<String>) -> Self {
        Self {
            backend_url: backend_url.into(),
        }
    }
}

impl CredentialRenewalBackend for HttpCredentialRenewalBackend {
    fn renew(&self, credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError> {
        renew_sand_box_inference_credential(&self.backend_url, credential)
    }
}

pub struct CredentialRenewerHooks {
    pub get_credential: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    pub set_credential: Arc<dyn Fn(InferenceCredential) + Send + Sync>,
    pub on_result: Option<Arc<dyn Fn(RenewalResult) + Send + Sync>>,
    pub now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
}

impl CredentialRenewerHooks {
    pub fn production(
        get_credential: Arc<dyn Fn() -> Option<String> + Send + Sync>,
        set_credential: Arc<dyn Fn(InferenceCredential) + Send + Sync>,
        on_result: Option<Arc<dyn Fn(RenewalResult) + Send + Sync>>,
    ) -> Self {
        Self {
            get_credential,
            set_credential,
            on_result,
            now_ms: Arc::new(system_now_ms),
        }
    }
}

#[derive(Debug)]
struct RenewerState {
    closed: bool,
    waiting: bool,
    requested_epoch: u64,
    completed_epoch: u64,
}

struct RenewerWorkerExitGuard<'a> {
    state: &'a (Mutex<RenewerState>, Condvar),
}

impl Drop for RenewerWorkerExitGuard<'_> {
    fn drop(&mut self) {
        let (lock, wake) = self.state;
        let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.closed = true;
        guard.waiting = false;
        // Worker exit releases waiters; it does not complete a renewal epoch.
        wake.notify_all();
    }
}

#[cfg(test)]
#[derive(Clone)]
pub(super) struct RenewerTestControl {
    state: Arc<(Mutex<RenewerState>, Condvar)>,
}

#[cfg(test)]
impl RenewerTestControl {
    pub(super) fn wait_until_waiting(&self, timeout: Duration) -> bool {
        let (lock, wake) = &*self.state;
        let guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let (guard, _) = wake
            .wait_timeout_while(guard, timeout, |current| !current.closed && !current.waiting)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        !guard.closed && guard.waiting
    }

    pub(super) fn snapshot(&self) -> (bool, bool, u64, u64) {
        let guard = self
            .state
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (
            guard.closed,
            guard.waiting,
            guard.requested_epoch,
            guard.completed_epoch,
        )
    }

    pub(super) fn close_waiters(&self) {
        let (lock, wake) = &*self.state;
        let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.closed = true;
        guard.waiting = false;
        wake.notify_all();
    }
}

pub struct SandInferenceCredentialRenewer {
    state: Arc<(Mutex<RenewerState>, Condvar)>,
    worker: Option<JoinHandle<()>>,
    backend: Arc<dyn CredentialRenewalBackend>,
    hooks: Arc<CredentialRenewerHooks>,
}

impl SandInferenceCredentialRenewer {
    pub fn new(
        backend: Arc<dyn CredentialRenewalBackend>,
        hooks: CredentialRenewerHooks,
    ) -> Self {
        Self {
            state: Arc::new((
                Mutex::new(RenewerState {
                    closed: false,
                    waiting: false,
                    requested_epoch: 0,
                    completed_epoch: 0,
                }),
                Condvar::new(),
            )),
            worker: None,
            backend,
            hooks: Arc::new(hooks),
        }
    }

    pub fn start(&mut self) {
        if self.worker.is_some() {
            return;
        }
        let state = Arc::clone(&self.state);
        let backend = Arc::clone(&self.backend);
        let hooks = Arc::clone(&self.hooks);
        self.worker = Some(thread::spawn(move || {
            // Outlive every worker-local state lock, including during unwinding.
            let _exit_guard = RenewerWorkerExitGuard { state: &state };
            let mut consecutive_failures = 0_u32;
            loop {
                {
                    let guard = state.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if guard.closed {
                        break;
                    }
                }

                let started = Instant::now();
                let renewal_credential = (hooks.get_credential)();
                let delay_ms = match renewal_credential.filter(|value| !value.is_empty()) {
                    None => MAX_REFRESH_INTERVAL_MS,
                    Some(renewal_credential) => match backend.renew(&renewal_credential) {
                        Ok(credential) => {
                            let expires_at_ms = credential.expires_at_ms;
                            (hooks.set_credential)(credential);
                            consecutive_failures = 0;
                            if let Some(on_result) = &hooks.on_result {
                                on_result(RenewalResult {
                                    outcome: RenewalOutcome::Renewed,
                                    consecutive_failures,
                                    duration_ms: started.elapsed().as_millis() as u64,
                                    error_summary: None,
                                });
                            }
                            refresh_delay_ms(expires_at_ms, (hooks.now_ms)())
                        }
                        Err(error) => {
                            consecutive_failures = consecutive_failures.saturating_add(1);
                            if let Some(on_result) = &hooks.on_result {
                                on_result(RenewalResult {
                                    outcome: RenewalOutcome::Failed,
                                    consecutive_failures,
                                    duration_ms: started.elapsed().as_millis() as u64,
                                    error_summary: Some(redact_renewal_error_for_report(
                                        &error.to_string(),
                                    )),
                                });
                            }
                            retry_delay_ms(consecutive_failures)
                        }
                    },
                };

                let (lock, wake) = &*state;
                let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.completed_epoch = guard.requested_epoch;
                wake.notify_all();
                if guard.closed {
                    break;
                }
                guard.waiting = true;
                let requested = guard.requested_epoch;
                let duration = Duration::from_millis(delay_ms);
                let (next_guard, _) = wake
                    .wait_timeout_while(guard, duration, |current| {
                        !current.closed && current.requested_epoch == requested
                    })
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard = next_guard;
                guard.waiting = false;
                if guard.closed {
                    break;
                }
            }
        }));
    }

    pub fn request_immediate_renewal(&self) -> bool {
        let (lock, wake) = &*self.state;
        let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.closed || !guard.waiting {
            return false;
        }
        guard.requested_epoch = guard.requested_epoch.saturating_add(1);
        let requested = guard.requested_epoch;
        wake.notify_all();
        while !guard.closed && guard.completed_epoch < requested {
            guard = wake
                .wait(guard)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        !guard.closed
    }

    #[cfg(test)]
    pub(super) fn test_control(&self) -> RenewerTestControl {
        RenewerTestControl {
            state: Arc::clone(&self.state),
        }
    }

    pub fn close(&mut self) {
        let (lock, wake) = &*self.state;
        {
            let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.closed = true;
            wake.notify_all();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for SandInferenceCredentialRenewer {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod worker_exit_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};

    const DEADLINE: Duration = Duration::from_secs(5);
    const NOW_MS: u64 = 1_000;

    fn credential() -> InferenceCredential {
        InferenceCredential {
            access_token: "worker-exit-test-token".into(),
            expires_at_ms: NOW_MS + 2 * MAX_REFRESH_INTERVAL_MS,
        }
    }

    enum BackendStep {
        Renew,
        Error,
        Unwind,
    }

    struct ControlledBackend {
        calls: AtomicUsize,
        control_failures: AtomicUsize,
        entered: Sender<usize>,
        steps: Mutex<Receiver<BackendStep>>,
    }

    impl CredentialRenewalBackend for ControlledBackend {
        fn renew(
            &self,
            _credential: &str,
        ) -> Result<InferenceCredential, SandCredentialRenewalError> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = self.entered.send(call);
            let step = self
                .steps
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .recv_timeout(DEADLINE);
            match step {
                Ok(BackendStep::Renew) => Ok(credential()),
                Ok(BackendStep::Error) => Err(SandCredentialRenewalError::Transport(
                    "injected ordinary backend failure".into(),
                )),
                Ok(BackendStep::Unwind) => panic!("injected renewal backend unwind"),
                Err(_) => {
                    self.control_failures.fetch_add(1, Ordering::SeqCst);
                    Err(SandCredentialRenewalError::Transport(
                        "test controller did not release backend".into(),
                    ))
                }
            }
        }
    }

    struct PendingRequest {
        result: Receiver<bool>,
        handle: Option<JoinHandle<()>>,
        control: RenewerTestControl,
    }

    struct RequestReceipt {
        outcome: Result<bool, RecvTimeoutError>,
        state: (bool, bool, u64, u64),
        joined: bool,
    }

    impl PendingRequest {
        fn finish(mut self) -> RequestReceipt {
            let outcome = self.result.recv_timeout(DEADLINE);
            let state = self.control.snapshot();
            // Keep the failure and pre-cleanup state. Cleanup cannot make a
            // missing worker notification count as a successful observation.
            if outcome.is_err() {
                self.control.close_waiters();
            }
            let joined = self.handle.take().unwrap().join().is_ok();
            RequestReceipt {
                outcome,
                state,
                joined,
            }
        }
    }

    impl Drop for PendingRequest {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                // Also reap the exact requester if the test itself unwinds.
                self.control.close_waiters();
                let _ = handle.join();
            }
        }
    }

    struct Fixture {
        renewer: Option<Arc<SandInferenceCredentialRenewer>>,
        control: RenewerTestControl,
        source_available: Arc<AtomicBool>,
        backend: Arc<ControlledBackend>,
        attempts: Receiver<usize>,
        steps: Sender<BackendStep>,
        stored: Arc<Mutex<Vec<InferenceCredential>>>,
        reports: Arc<Mutex<Vec<RenewalResult>>>,
    }

    impl Fixture {
        fn new(on_result: impl Fn(RenewalResult) + Send + Sync + 'static) -> Self {
            let (entered, attempts) = mpsc::channel();
            let (steps, backend_steps) = mpsc::channel();
            let backend = Arc::new(ControlledBackend {
                calls: AtomicUsize::new(0),
                control_failures: AtomicUsize::new(0),
                entered,
                steps: Mutex::new(backend_steps),
            });
            let source_available = Arc::new(AtomicBool::new(false));
            let source = Arc::clone(&source_available);
            let stored = Arc::new(Mutex::new(Vec::new()));
            let store = Arc::clone(&stored);
            let reports = Arc::new(Mutex::new(Vec::new()));
            let report = Arc::clone(&reports);
            let mut renewer = SandInferenceCredentialRenewer::new(
                backend.clone(),
                CredentialRenewerHooks {
                    get_credential: Arc::new(move || {
                        source
                            .load(Ordering::Acquire)
                            .then(|| "test-renewal-source".into())
                    }),
                    set_credential: Arc::new(move |credential| {
                        store.lock().unwrap().push(credential);
                    }),
                    on_result: Some(Arc::new(move |result| {
                        report.lock().unwrap().push(result.clone());
                        on_result(result);
                    })),
                    now_ms: Arc::new(|| NOW_MS),
                },
            );
            let control = renewer.test_control();
            renewer.start();
            Self {
                renewer: Some(Arc::new(renewer)),
                control,
                source_available,
                backend,
                attempts,
                steps,
                stored,
                reports,
            }
        }

        fn request(&self) -> PendingRequest {
            let renewer = Arc::clone(self.renewer.as_ref().unwrap());
            let (completed, result) = mpsc::channel();
            let handle = thread::spawn(move || {
                let outcome = renewer.request_immediate_renewal();
                let _ = completed.send(outcome);
            });
            PendingRequest {
                result,
                handle: Some(handle),
                control: self.control.clone(),
            }
        }

        fn close(&mut self) -> (Result<(bool, bool), RecvTimeoutError>, bool) {
            let Some(renewer) = self.renewer.take() else {
                return (Ok((true, true)), true);
            };
            let (completed, result) = mpsc::channel();
            let closer = thread::spawn(move || {
                let mut renewer = Arc::try_unwrap(renewer)
                    .ok()
                    .expect("requester must be joined before closing its owner");
                renewer.close();
                renewer.close();
                let _ = completed.send((
                    renewer.worker.is_none(),
                    !renewer.request_immediate_renewal(),
                ));
            });
            let outcome = result.recv_timeout(DEADLINE);
            if outcome.is_err() {
                // Retain the timeout as failure before releasing this fixture's
                // state. close() still joins the actual owned renewal worker.
                self.control.close_waiters();
            }
            let joined = closer.join().is_ok();
            (outcome, joined)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.renewer.is_some() {
                self.control.close_waiters();
                let _ = self.close();
            }
        }
    }

    fn assert_request(receipt: RequestReceipt, expected: bool, state: (bool, bool, u64, u64)) {
        assert_eq!(receipt.outcome, Ok(expected));
        assert_eq!(receipt.state, state);
        assert!(receipt.joined, "the owned requester must be joined");
    }

    fn assert_close(receipt: (Result<(bool, bool), RecvTimeoutError>, bool)) {
        assert_eq!(receipt.0, Ok((true, true)));
        assert!(receipt.1, "the owned closer must be joined");
    }

    #[derive(Clone, Copy)]
    enum UnwindSite {
        Backend,
        Callback,
    }

    fn check_admitted_unwind(site: UnwindSite) {
        let mut fixture = Fixture::new(move |_| {
            if matches!(site, UnwindSite::Callback) {
                panic!("injected renewal callback unwind");
            }
        });
        let was_waiting = fixture.control.wait_until_waiting(DEADLINE);
        fixture.source_available.store(true, Ordering::Release);
        let request = fixture.request();
        let entered = fixture.attempts.recv_timeout(DEADLINE);
        let admitted = fixture.control.snapshot();
        let released = fixture.steps.send(match site {
            UnwindSite::Backend => BackendStep::Unwind,
            UnwindSite::Callback => BackendStep::Renew,
        });
        let first = request.finish();
        let later = fixture.request().finish();
        let close = fixture.close();

        assert!(was_waiting, "admission requires the actual idle state");
        assert_eq!(entered, Ok(1));
        assert_eq!(admitted, (false, false, 1, 0));
        assert!(released.is_ok());
        assert_request(first, false, (true, false, 1, 0));
        assert_request(later, false, (true, false, 1, 0));
        assert_eq!(fixture.backend.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.backend.control_failures.load(Ordering::SeqCst), 0);
        let stored = fixture.stored.lock().unwrap();
        let reports = fixture.reports.lock().unwrap();
        match site {
            UnwindSite::Backend => {
                assert!(stored.is_empty());
                assert!(reports.is_empty());
            }
            UnwindSite::Callback => {
                assert_eq!(*stored, vec![credential()]);
                assert_eq!(reports.len(), 1, "a failed callback must not be replayed");
                assert_eq!(reports[0].outcome, RenewalOutcome::Renewed);
            }
        }
        assert_close(close);
    }

    #[test]
    fn admitted_request_is_released_on_callback_unwind_without_completing_epoch() {
        check_admitted_unwind(UnwindSite::Callback);
    }

    #[test]
    fn admitted_request_is_released_on_backend_unwind_without_completing_epoch() {
        check_admitted_unwind(UnwindSite::Backend);
    }

    #[test]
    fn held_callback_keeps_the_admitted_request_pending_until_return() {
        let (callback_entered, callback_entry) = mpsc::channel();
        let (release_callback, callback_release) = mpsc::channel();
        let callback_release = Mutex::new(callback_release);
        let callback_timed_out = Arc::new(AtomicBool::new(false));
        let timed_out = Arc::clone(&callback_timed_out);
        let mut fixture = Fixture::new(move |_| {
            let _ = callback_entered.send(());
            if callback_release
                .lock()
                .unwrap()
                .recv_timeout(DEADLINE)
                .is_err()
            {
                timed_out.store(true, Ordering::SeqCst);
            }
        });
        let was_waiting = fixture.control.wait_until_waiting(DEADLINE);
        fixture.source_available.store(true, Ordering::Release);
        let request = fixture.request();
        let entered = fixture.attempts.recv_timeout(DEADLINE);
        let released_backend = fixture.steps.send(BackendStep::Renew);
        let callback_held = callback_entry.recv_timeout(DEADLINE);
        let held_state = fixture.control.snapshot();
        let before_release = request.result.try_recv();
        let released_callback = release_callback.send(());
        let completed = request.finish();
        let close = fixture.close();

        assert!(was_waiting);
        assert_eq!(entered, Ok(1));
        assert!(released_backend.is_ok());
        assert_eq!(callback_held, Ok(()));
        assert_eq!(held_state, (false, false, 1, 0));
        assert_eq!(before_release, Err(TryRecvError::Empty));
        assert!(released_callback.is_ok());
        assert!(!callback_timed_out.load(Ordering::SeqCst));
        assert_request(completed, true, (false, true, 1, 1));
        assert_eq!(fixture.backend.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fixture.backend.control_failures.load(Ordering::SeqCst), 0);
        assert_eq!(*fixture.stored.lock().unwrap(), vec![credential()]);
        assert_eq!(fixture.reports.lock().unwrap().len(), 1);
        assert_close(close);
    }

    #[test]
    fn ordinary_backend_error_completes_its_epoch_and_allows_immediate_retry() {
        let mut fixture = Fixture::new(|_| {});
        let was_waiting = fixture.control.wait_until_waiting(DEADLINE);
        fixture.source_available.store(true, Ordering::Release);
        let first = fixture.request();
        let first_entry = fixture.attempts.recv_timeout(DEADLINE);
        let first_admitted = fixture.control.snapshot();
        let failed_backend = fixture.steps.send(BackendStep::Error);
        let failed = first.finish();

        let waiting_after_error = fixture.control.wait_until_waiting(DEADLINE);
        let second = fixture.request();
        let second_entry = fixture.attempts.recv_timeout(DEADLINE);
        let second_admitted = fixture.control.snapshot();
        let successful_backend = fixture.steps.send(BackendStep::Renew);
        let renewed = second.finish();
        let close = fixture.close();

        assert!(was_waiting);
        assert_eq!(first_entry, Ok(1));
        assert_eq!(first_admitted, (false, false, 1, 0));
        assert!(failed_backend.is_ok());
        assert_request(failed, true, (false, true, 1, 1));
        assert!(waiting_after_error);
        assert_eq!(second_entry, Ok(2));
        assert_eq!(second_admitted, (false, false, 2, 1));
        assert!(successful_backend.is_ok());
        assert_request(renewed, true, (false, true, 2, 2));
        assert_eq!(fixture.backend.calls.load(Ordering::SeqCst), 2);
        assert_eq!(fixture.backend.control_failures.load(Ordering::SeqCst), 0);
        assert_eq!(*fixture.stored.lock().unwrap(), vec![credential()]);
        let reports = fixture.reports.lock().unwrap();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].outcome, RenewalOutcome::Failed);
        assert_eq!(reports[0].consecutive_failures, 1);
        assert_eq!(
            reports[0].error_summary.as_deref(),
            Some("Sand inference-credential renewal request failed: injected ordinary backend failure"),
        );
        assert_eq!(reports[1].outcome, RenewalOutcome::Renewed);
        assert_eq!(reports[1].consecutive_failures, 0);
        assert_close(close);
    }

    #[test]
    fn explicit_close_joins_the_idle_worker_and_remains_idempotent() {
        let mut fixture = Fixture::new(|_| {});
        let was_waiting = fixture.control.wait_until_waiting(DEADLINE);
        let close = fixture.close();

        assert!(was_waiting);
        assert_close(close);
        assert_eq!(fixture.control.snapshot(), (true, false, 0, 0));
        assert_eq!(fixture.backend.calls.load(Ordering::SeqCst), 0);
        assert!(fixture.stored.lock().unwrap().is_empty());
        assert!(fixture.reports.lock().unwrap().is_empty());
    }
}
