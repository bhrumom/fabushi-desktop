//! One existing-owner maintenance snapshot and all of its summary phases.
//! Admission happens inside the caller's bounded worker, never at allocation.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use super::ProductionInferenceExtension;
use crate::extensions::inference::inference_service::InferenceRoute;
use crate::extensions::inference::provider_session::{
    CodexResponsesRequestBinding, FabushiResponsesRequestBinding, ProviderSessionError,
    RoutedProvider, capture_codex_responses_request_binding,
    capture_fabushi_responses_request_binding,
};

#[cfg(test)]
mod admission_test_hook {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::thread::{self, ThreadId};

    struct HookState {
        owner: ThreadId,
        alive: Cell<bool>,
        observe: Box<dyn Fn(bool)>,
    }

    thread_local! {
        static ADMISSION_HOOK: RefCell<Option<Rc<HookState>>> = RefCell::new(None);
    }

    /// Worker-affine observation only: no identity, error or credential can be
    /// supplied by the fixture. Rc keeps the scope on the installing thread.
    pub(crate) struct SummaryAdmissionHookScope {
        state: Rc<HookState>,
        previous: Option<Rc<HookState>>,
    }

    impl SummaryAdmissionHookScope {
        pub(crate) fn install(observe: impl Fn(bool) + 'static) -> Self {
            let state = Rc::new(HookState {
                owner: thread::current().id(),
                alive: Cell::new(true),
                observe: Box::new(observe),
            });
            let previous = ADMISSION_HOOK.with(|slot| slot.replace(Some(state.clone())));
            Self { state, previous }
        }
    }

    impl Drop for SummaryAdmissionHookScope {
        fn drop(&mut self) {
            assert_eq!(thread::current().id(), self.state.owner, "summary admission scope changed threads");
            self.state.alive.set(false);
            ADMISSION_HOOK.with(|slot| {
                let mut slot = slot.borrow_mut();
                assert!(slot.as_ref().is_some_and(|active| Rc::ptr_eq(active, &self.state)),
                    "summary admission scope retirement must follow its owned stack");
                *slot = self.previous.take();
            });
        }
    }

    pub(super) fn observe_capture(succeeded: bool) {
        let state = ADMISSION_HOOK.with(|slot| slot.borrow().clone());
        if let Some(state) = state {
            assert_eq!(thread::current().id(), state.owner, "summary admission changed threads");
            assert!(state.alive.get(), "summary admission outlived its fixture scope");
            // No TLS borrow spans the callback or its bounded fixture barrier.
            (state.observe)(succeeded);
            assert!(state.alive.get(), "summary admission scope retired during observation");
            assert!(ADMISSION_HOOK.with(|slot| slot.borrow().as_ref()
                .is_some_and(|active| Rc::ptr_eq(active, &state))),
                "summary admission observation changed its active scope");
        }
    }
}

#[cfg(test)]
pub(crate) use admission_test_hook::SummaryAdmissionHookScope;

struct SummaryIdentity {
    provider: RoutedProvider,
    fabushi_model: Option<String>,
    codex_model: Option<String>,
    codex_binding: Option<CodexResponsesRequestBinding>,
    fabushi_binding: Option<FabushiResponsesRequestBinding>,
}

/// Opaque, process-local identity for one independently admitted summary job.
/// It holds neither secrets nor a serializable continuation/permission grant.
pub struct SummaryInferenceJob {
    inference: Arc<ProductionInferenceExtension>,
    data_dir: PathBuf,
    admission: OnceLock<Result<SummaryIdentity, ProviderSessionError>>,
}

impl ProductionInferenceExtension {
    /// Allocation only. The first worker to execute a phase admits the route,
    /// model and credential identity; every phase and retry shares its result.
    pub fn create_summarization_job(
        self: &Arc<Self>,
        data_dir: PathBuf,
    ) -> Arc<SummaryInferenceJob> {
        Arc::new(SummaryInferenceJob {
            inference: Arc::clone(self),
            data_dir,
            admission: OnceLock::new(),
        })
    }
}

impl SummaryInferenceJob {
    fn capture_identity(&self) -> Result<SummaryIdentity, ProviderSessionError> {
        let provider = match self.inference.route() {
            InferenceRoute::Cursor => RoutedProvider::Cursor,
            InferenceRoute::Routed(provider) => provider,
        };
        let mut identity = SummaryIdentity {
            provider,
            fabushi_model: None,
            codex_model: None,
            codex_binding: None,
            fabushi_binding: None,
        };
        match provider {
            RoutedProvider::Fabushi => {
                identity.fabushi_model = Some(
                    self.inference.port().resolve_fabushi_turn_model(None, None, None)?,
                );
                identity.fabushi_binding = Some(capture_fabushi_responses_request_binding()?);
            }
            RoutedProvider::Codex => {
                identity.codex_model = Some(
                    self.inference.port().resolve_codex_turn_model(None, None, None)?,
                );
                identity.codex_binding = Some(capture_codex_responses_request_binding()?);
            }
            // Other routes keep their existing provider-specific summary policy.
            // Only their route is fixed by this bounded Native/Codex contract.
            _ => {}
        }
        Ok(identity)
    }

    pub fn run_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<String, ProviderSessionError> {
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled("summary job cancelled".into()));
        }
        // Initialization is sticky even when the owning attempt times out while
        // synchronous settings/credential I/O is finishing. Its post-admission
        // cancellation check prohibits dispatch, and a later attempt cannot
        // replace that identity or erase an admission refusal.
        let admitted = self.admission.get_or_init(|| {
            let result = self.capture_identity();
            #[cfg(test)]
            admission_test_hook::observe_capture(result.is_ok());
            result
        });
        if should_cancel() {
            return Err(ProviderSessionError::Cancelled("summary job cancelled".into()));
        }
        let identity = admitted.as_ref().map_err(Clone::clone)?;
        self.inference.run_summarization_prompt_with_identity(
            &self.data_dir,
            identity.provider,
            identity.fabushi_model.as_deref(),
            identity.codex_model.as_deref(),
            identity.codex_binding.as_ref(),
            identity.fabushi_binding.as_ref(),
            system_prompt,
            user_prompt,
            should_cancel,
        )
    }
}
