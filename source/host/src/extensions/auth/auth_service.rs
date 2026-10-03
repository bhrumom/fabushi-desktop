use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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
pub const SAND_SHORTLIVED_CREDS_WAITING_MESSAGE: &str =
    "Waiting for an inference credential. Grok Bot's computer renews this automatically (no desktop required); this resolves on its own shortly.";

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
    machine_id: MachineIdProvider,
    now_ms: Arc<dyn Fn() -> u64 + Send + Sync>,
    log: AuthLog,
}

impl HostAuthService {
    pub fn new(options: HostAuthServiceOptions) -> Result<Self, SandCredentialRenewalError> {
        let dev_token_file = options
            .environment
            .get(SAND_DEV_INFERENCE_TOKEN_FILE_ENV)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let renewal_credential = options
            .environment
            .get(SAND_INFERENCE_RENEWAL_CREDENTIAL_ENV)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let has_renewal_credential = dev_token_file.is_some() || renewal_credential.is_some();

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
