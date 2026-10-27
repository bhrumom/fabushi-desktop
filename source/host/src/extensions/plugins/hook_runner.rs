use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HookName {
    BeforeInit,
    AfterInit,
    BeforeReply,
    AfterReply,
    BeforeCommand,
    AfterCommand,
    BeforeToolCall,
}

pub type HookHandler = Arc<dyn Fn(&Value) -> Result<Option<Value>, String> + Send + Sync>;

#[derive(Clone)]
pub struct RegisteredHook {
    pub plugin_id: String,
    pub handler: HookHandler,
    pub priority: i32,
}

#[derive(Clone, Default)]
pub struct PluginHookRunner {
    hooks: Arc<Mutex<HashMap<HookName, Vec<RegisteredHook>>>>,
}

impl PluginHookRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_hook<F>(&self, hook_name: HookName, plugin_id: &str, priority: i32, handler: F)
    where
        F: Fn(&Value) -> Result<Option<Value>, String> + Send + Sync + 'static,
    {
        let mut hooks = self.hooks.lock().unwrap();
        let list = hooks.entry(hook_name).or_default();
        list.push(RegisteredHook {
            plugin_id: plugin_id.to_string(),
            handler: Arc::new(handler),
            priority,
        });
        // Sort by priority descending (higher priority runs first)
        list.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn run_hooks(&self, hook_name: HookName, initial_payload: &Value) -> Result<Value, String> {
        let handlers = {
            let hooks = self.hooks.lock().unwrap();
            hooks.get(&hook_name).cloned().unwrap_or_default()
        };

        let mut current_payload = initial_payload.clone();

        for registered in handlers {
            match (registered.handler)(&current_payload) {
                Ok(Some(modified)) => {
                    current_payload = modified;
                }
                Ok(None) => {
                    // Retain current payload
                }
                Err(err) => {
                    // Fail-soft: record warning/error but continue pipeline unless payload is invalid
                    eprintln!(
                        "[plugin_hook] Error running hook {:?} for plugin '{}': {err}",
                        hook_name, registered.plugin_id
                    );
                }
            }
        }

        Ok(current_payload)
    }

    pub fn hook_count(&self, hook_name: HookName) -> usize {
        let hooks = self.hooks.lock().unwrap();
        hooks.get(&hook_name).map(|v| v.len()).unwrap_or(0)
    }
}

pub type ProviderExtensionHandler = Arc<dyn Fn(&str, &Value) -> Result<Value, String> + Send + Sync>;

#[derive(Clone, Default)]
pub struct ProviderExtensionBridge {
    providers: Arc<Mutex<HashMap<String, ProviderExtensionHandler>>>,
}

impl ProviderExtensionBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider<F>(&self, provider_id: &str, handler: F)
    where
        F: Fn(&str, &Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(provider_id.to_ascii_lowercase(), Arc::new(handler));
    }

    pub fn invoke_provider(
        &self,
        provider_id: &str,
        operation: &str,
        params: &Value,
        _timeout: Duration,
    ) -> Result<Value, String> {
        let handler = {
            let providers = self.providers.lock().unwrap();
            providers
                .get(&provider_id.to_ascii_lowercase())
                .cloned()
                .ok_or_else(|| format!("Unknown provider extension: '{provider_id}'"))?
        };

        handler(operation, params)
    }
}

// Boundary invariant validations (from src/plugins/contracts/boundary-invariants.test.ts)

pub struct HookBoundaryInvariantGuard;

impl HookBoundaryInvariantGuard {
    pub const ALLOWED_BUNDLED_TYPED_HOOKS: &'static [&'static str] = &[
        "agent_end",
        "before_agent_reply",
        "before_agent_run",
        "before_prompt_build",
        "before_tool_call",
        "cron_changed",
        "cron_reconciled",
        "gateway_start",
        "gateway_stop",
        "inbound_claim",
        "reply_dispatch",
        "session_end",
        "subagent_delivery_target",
        "subagent_ended",
    ];

    pub fn is_allowed_typed_hook(hook_name: &str) -> bool {
        Self::ALLOWED_BUNDLED_TYPED_HOOKS.contains(&hook_name)
    }

    pub fn assert_no_raw_register_hook_calls(sources: &[(&str, &str)]) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();
        for (file_path, content) in sources {
            if content.contains("registerHook(") {
                violations.push(file_path.to_string());
            }
        }
        if violations.is_empty() {
            Ok(())
        } else {
            violations.sort();
            Err(violations)
        }
    }

    pub fn is_allowed_bundled_extension_import(specifier: &str) -> bool {
        if !specifier.contains("extensions/") {
            return true;
        }
        specifier.ends_with("/api.js") || specifier.ends_with("/runtime-api.js")
    }
}
