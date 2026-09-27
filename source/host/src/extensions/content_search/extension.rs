use std::sync::{Arc, Mutex};

pub const CONTENT_SEARCH_EXTENSION_ID: &str = "content-search";
pub const GLOBAL_SEARCH_GATE: &str = "sand_global_search";
pub const SEARCH_INDEX_FILENAME: &str = "search-index.db";
pub const SEARCH_INDEX_DISPOSE_TIMEOUT_MS: u64 = 2_000;
pub const AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT: usize = 5;
pub const AGENT_CONTENT_SEARCH_MAX_RESULTS: usize = 50;
pub const CONTENT_SEARCH_DEPENDENCIES: &[&str] = &["experiments", "telemetry"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptMutation {
    pub agent_id: String,
    pub entry_id: String,
}

pub trait ContentSearchIndex: Send + Sync + 'static {
    fn start(&self);
    fn dispose(&self);
    fn is_search_ready(&self) -> bool;
    fn search_messages(&self, query: &str, limit: usize) -> Vec<String>;
    fn search_media(&self, query: &str, limit: usize) -> Vec<String>;
    fn apply_mutation(&self, mutation: TranscriptMutation);
}

pub trait ContentSearchIndexFactory: Send + Sync {
    type Index: ContentSearchIndex;
    fn create(&self) -> Arc<Self::Index>;
}

pub trait FeatureGateSubscription: Send {
    fn dispose(&mut self);
}

pub trait FeatureGateProperty: Send + Sync {
    fn get(&self) -> bool;
    fn subscribe(
        &self,
        listener: Arc<dyn Fn(bool) + Send + Sync>,
    ) -> Box<dyn FeatureGateSubscription>;
}

pub trait TranscriptMutationSubscription: Send {
    fn dispose(&mut self);
}

pub trait TranscriptMutationSource: Send + Sync {
    fn subscribe(
        &self,
        listener: Arc<dyn Fn(TranscriptMutation) + Send + Sync>,
    ) -> Box<dyn TranscriptMutationSubscription>;
}

struct RuntimeState {
    mutation_subscription: Option<Box<dyn TranscriptMutationSubscription>>,
}

pub struct ContentSearchExtension<I: ContentSearchIndex> {
    index: Arc<I>,
    check_gate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    state: Arc<Mutex<RuntimeState>>,
    gate_subscription: Mutex<Option<Box<dyn FeatureGateSubscription>>>,
}

impl<I: ContentSearchIndex> ContentSearchExtension<I> {
    pub fn is_enabled(&self) -> bool {
        (self.check_gate)(GLOBAL_SEARCH_GATE)
    }

    pub fn is_search_ready(&self) -> bool {
        self.index.is_search_ready()
    }

    pub fn max_matches_per_agent(&self) -> usize {
        AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT
    }

    pub fn max_results(&self) -> usize {
        AGENT_CONTENT_SEARCH_MAX_RESULTS
    }

    pub fn search_messages(&self, query: &str, limit: usize) -> Vec<String> {
        self.index.search_messages(query, limit)
    }

    pub fn search_media(&self, query: &str, limit: usize) -> Vec<String> {
        self.index.search_media(query, limit)
    }
}

impl<I: ContentSearchIndex> Drop for ContentSearchExtension<I> {
    fn drop(&mut self) {
        if let Ok(mut gate) = self.gate_subscription.lock() {
            if let Some(mut subscription) = gate.take() {
                subscription.dispose();
            }
        }
        if let Ok(mut state) = self.state.lock() {
            if let Some(mut subscription) = state.mutation_subscription.take() {
                subscription.dispose();
            }
        }
        self.index.dispose();
    }
}

pub fn start_content_search_extension<F>(
    factory: &F,
    gate: Arc<dyn FeatureGateProperty>,
    check_gate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    mutations: Arc<dyn TranscriptMutationSource>,
) -> ContentSearchExtension<F::Index>
where
    F: ContentSearchIndexFactory,
{
    let index = factory.create();
    let state = Arc::new(Mutex::new(RuntimeState {
        mutation_subscription: None,
    }));

    let apply_gate: Arc<dyn Fn(bool) + Send + Sync> = {
        let index = Arc::clone(&index);
        let state = Arc::clone(&state);
        let mutations = Arc::clone(&mutations);
        Arc::new(move |enabled| {
            let Ok(mut runtime) = state.lock() else {
                return;
            };
            if !enabled {
                if let Some(mut subscription) = runtime.mutation_subscription.take() {
                    subscription.dispose();
                }
                return;
            }
            if runtime.mutation_subscription.is_none() {
                let target = Arc::clone(&index);
                runtime.mutation_subscription = Some(mutations.subscribe(Arc::new(move |mutation| {
                    target.apply_mutation(mutation);
                })));
            }
            index.start();
        })
    };

    let gate_subscription = gate.subscribe(Arc::clone(&apply_gate));
    apply_gate(gate.get());

    ContentSearchExtension {
        index,
        check_gate,
        state,
        gate_subscription: Mutex::new(Some(gate_subscription)),
    }
}
