use std::sync::Arc;

use crate::extensions::auth::extension::HostAuthExtension;

use super::cloud_agents_service::{
    CloudConversationTraceConverter, SandCloudAgentManager,
};

struct CloudAgentsExtensionInner {
    service: Arc<SandCloudAgentManager>,
}

impl Drop for CloudAgentsExtensionInner {
    fn drop(&mut self) {
        self.service.dispose();
    }
}

#[derive(Clone)]
pub struct CloudAgentsExtension {
    inner: Arc<CloudAgentsExtensionInner>,
}

impl CloudAgentsExtension {
    pub fn new(service: Arc<SandCloudAgentManager>) -> Self {
        Self {
            inner: Arc::new(CloudAgentsExtensionInner { service }),
        }
    }

    pub fn service(&self) -> Arc<SandCloudAgentManager> {
        Arc::clone(&self.inner.service)
    }
}

/// Starts the Host-owned Cloud Agents extension.
///
/// Auth and the generated ConversationMessage -> trace adapter stay explicit
/// dependencies, matching the frozen Host extension boundary. The service is
/// started once, eagerly refreshes team-admin policy, and is disposed when the
/// final extension owner is dropped.
pub fn start_cloud_agents_extension(
    backend_url: String,
    auth: Arc<HostAuthExtension>,
    convert_conversation: CloudConversationTraceConverter,
) -> CloudAgentsExtension {
    let service = SandCloudAgentManager::production(
        backend_url,
        auth,
        convert_conversation,
    );
    service.prefetch_team_admin_policy();
    CloudAgentsExtension::new(service)
}
