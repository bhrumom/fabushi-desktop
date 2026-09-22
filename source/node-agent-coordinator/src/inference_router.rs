use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceRoute {
    pub provider: String,
    pub host_slot: String,
}

#[derive(Debug, Default)]
pub struct InferenceRouter {
    by_agent: HashMap<String, InferenceRoute>,
    default: Option<InferenceRoute>,
}

impl InferenceRouter {
    pub fn set_default(&mut self, route: InferenceRoute) { self.default = Some(route); }
    pub fn bind_agent(&mut self, agent_id: impl Into<String>, route: InferenceRoute) {
        self.by_agent.insert(agent_id.into(), route);
    }
    pub fn resolve(&self, agent_id: &str) -> Option<&InferenceRoute> {
        self.by_agent.get(agent_id).or(self.default.as_ref())
    }
}
