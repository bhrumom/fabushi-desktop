use crate::gateway_server::GatewayEventHub;
use super::mcp_service::{
    BoxServerStatus, CatalogField, CatalogPlugin, CatalogSkill, EffectivePlugin,
    McpManagerBackend, McpPluginUninstallOutcome, McpServerSummary,
};
use super::plugin_skills::{LoadedPlugins, PluginSkillsLoader};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, RecvTimeoutError, SyncSender},
};
use std::time::Duration;
use thiserror::Error;

pub const HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL: &str = "host-mcp-lifecycle-request";
pub const MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD: &str = "mcp.resolveLifecycleRequest";
pub const MCP_LIST_SERVERS_METHOD: &str = "listHostMcpServers";
pub const MCP_LIST_CATALOG_METHOD: &str = "listHostMcpCatalog";
pub const MCP_LIST_EFFECTIVE_PLUGINS_METHOD: &str = "listHostEffectiveMcpPlugins";
pub const MCP_LOAD_PLUGIN_SKILLS_METHOD: &str = "loadHostPluginSkills";
pub const MCP_INSTALL_PLUGIN_METHOD: &str = "installHostMcpPlugin";
pub const MCP_UNINSTALL_PLUGIN_METHOD: &str = "uninstallHostMcpPlugin";
pub const MCP_ADD_SERVER_METHOD: &str = "addHostMcpServer";
pub const MCP_REMOVE_SERVER_METHOD: &str = "removeHostMcpServer";
pub const MCP_RESTART_SERVERS_METHOD: &str = "restartHostMcpServers";
pub const MCP_SET_INSTRUCTIONS_METHOD: &str = "setHostMcpInstructions";
pub const MCP_AUTHENTICATE_SERVER_METHOD: &str = "authenticateHostMcpServer";
pub const MCP_LOGOUT_ACCOUNT_METHOD: &str = "logoutHostMcpAccount";
pub const MCP_REMOVE_ACCOUNT_METHOD: &str = "removeHostMcpAccount";
pub const MCP_RENAME_ACCOUNT_METHOD: &str = "renameHostMcpAccount";
pub const MCP_NOTE_AUTH_COMPLETED_METHOD: &str = "noteHostMcpAuthCompleted";

const DEFAULT_MCP_LIFECYCLE_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Error, PartialEq)]
pub enum CoordinatorMcpLifecycleRelayError {
    #[error("unsupported Host MCP lifecycle relay method: {0}")]
    UnsupportedMethod(String),
    #[error("Host MCP lifecycle relay request {0} timed out")]
    Timeout(String),
    #[error("Host MCP lifecycle relay request {request_id} failed: {message}")]
    Remote { request_id: String, message: String },
    #[error("Host MCP lifecycle relay is closed: {0}")]
    Closed(String),
    #[error("invalid Host MCP lifecycle relay response: {0}")]
    Protocol(String),
}

type PendingReply = SyncSender<Result<Value, CoordinatorMcpLifecycleRelayError>>;

pub fn is_host_mcp_lifecycle_method(method: &str) -> bool {
    matches!(
        method,
        MCP_LIST_SERVERS_METHOD
            | MCP_LIST_CATALOG_METHOD
            | MCP_LIST_EFFECTIVE_PLUGINS_METHOD
            | MCP_LOAD_PLUGIN_SKILLS_METHOD
            | MCP_INSTALL_PLUGIN_METHOD
            | MCP_UNINSTALL_PLUGIN_METHOD
            | MCP_ADD_SERVER_METHOD
            | MCP_REMOVE_SERVER_METHOD
            | MCP_RESTART_SERVERS_METHOD
            | MCP_SET_INSTRUCTIONS_METHOD
            | MCP_AUTHENTICATE_SERVER_METHOD
            | MCP_LOGOUT_ACCOUNT_METHOD
            | MCP_REMOVE_ACCOUNT_METHOD
            | MCP_RENAME_ACCOUNT_METHOD
            | MCP_NOTE_AUTH_COMPLETED_METHOD
    )
}

#[derive(Clone)]
pub struct CoordinatorPluginSkillsLoader {
    relay: Arc<CoordinatorMcpLifecycleRelay>,
}

impl CoordinatorPluginSkillsLoader {
    pub fn new(relay: Arc<CoordinatorMcpLifecycleRelay>) -> Self {
        Self { relay }
    }
}

impl PluginSkillsLoader for CoordinatorPluginSkillsLoader {
    fn load(&self) -> Result<LoadedPlugins, String> {
        let payload = self
            .relay
            .request(MCP_LOAD_PLUGIN_SKILLS_METHOD, json!({}))
            .map_err(|error| error.to_string())?;
        serde_json::from_value(payload)
            .map_err(|error| format!("invalid Plugin Skills loader response: {error}"))
    }
}

pub struct CoordinatorMcpLifecycleRelay {
    events: GatewayEventHub,
    pending: Mutex<HashMap<String, PendingReply>>,
    incarnation: uuid::Uuid,
    next_request_id: AtomicU64,
    timeout: Duration,
    closed: AtomicBool,
    #[cfg(test)]
    before_pending_registration: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl CoordinatorMcpLifecycleRelay {
    pub fn new(events: GatewayEventHub) -> Self {
        Self::with_timeout(events, DEFAULT_MCP_LIFECYCLE_REQUEST_TIMEOUT)
    }

    pub fn with_timeout(events: GatewayEventHub, timeout: Duration) -> Self {
        Self {
            events,
            pending: Mutex::new(HashMap::new()),
            incarnation: uuid::Uuid::new_v4(),
            next_request_id: AtomicU64::new(0),
            timeout,
            closed: AtomicBool::new(false),
            #[cfg(test)]
            before_pending_registration: None,
        }
    }

    pub fn request(&self, method: &str, args: Value) -> Result<Value, CoordinatorMcpLifecycleRelayError> {
        if !is_host_mcp_lifecycle_method(method) {
            return Err(CoordinatorMcpLifecycleRelayError::UnsupportedMethod(method.to_string()));
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(CoordinatorMcpLifecycleRelayError::Closed("Mahayana Host is shutting down".into()));
        }

        // A late Coordinator reply may reach a replacement Host. Namespace
        // each relay's requests independently, and never recycle its sequence.
        let sequence = self
            .next_request_id
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |sequence| {
                sequence.checked_add(1)
            })
            .map_err(|_| {
                CoordinatorMcpLifecycleRelayError::Protocol(
                    "Host MCP lifecycle request sequence exhausted".into(),
                )
            })?
            + 1;
        let request_id = format!("host-mcp-{}-{sequence}", self.incarnation);
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        #[cfg(test)]
        if let Some(before_registration) = &self.before_pending_registration {
            before_registration();
        }
        {
            let mut pending = self.pending
                .lock()
                .map_err(|_| CoordinatorMcpLifecycleRelayError::Protocol("pending relay lock poisoned".into()))?;
            // Closure drains this same lock. A request must either register
            // before that drain or observe closure without leaving a waiter.
            if self.closed.load(Ordering::Acquire) {
                return Err(CoordinatorMcpLifecycleRelayError::Closed("Mahayana Host is shutting down".into()));
            }
            pending.insert(request_id.clone(), reply_tx);
        }

        // Publication invokes listeners synchronously; they may resolve or
        // close the relay. An admitted request can still publish after close.
        self.events.publish(json!({
            "channel": HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL,
            "payload": {
                "requestId": request_id,
                "method": method,
                "args": args,
            }
        }));

        match reply_rx.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorMcpLifecycleRelayError::Timeout(request_id))
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                Err(CoordinatorMcpLifecycleRelayError::Closed(format!("reply channel disconnected for {request_id}")))
            }
        }
    }

    pub fn resolve(&self, args: &Value) -> Result<Value, CoordinatorMcpLifecycleRelayError> {
        let request_id = args
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol("mcp.resolveLifecycleRequest requires requestId".into()))?
            .to_string();
        let ok = args
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol("mcp.resolveLifecycleRequest requires boolean ok".into()))?;
        let sender = self.pending
            .lock()
            .map_err(|_| CoordinatorMcpLifecycleRelayError::Protocol("pending relay lock poisoned".into()))?
            .remove(&request_id)
            .ok_or_else(|| CoordinatorMcpLifecycleRelayError::Protocol(format!("unknown or already-settled Host MCP lifecycle request {request_id}")))?;

        let outcome = if ok {
            Ok(args.get("result").cloned().unwrap_or(Value::Null))
        } else {
            Err(CoordinatorMcpLifecycleRelayError::Remote {
                request_id: request_id.clone(),
                message: args
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Host MCP lifecycle request failed")
                    .to_string(),
            })
        };
        sender
            .send(outcome)
            .map_err(|_| CoordinatorMcpLifecycleRelayError::Closed(format!("request waiter disappeared for {request_id}")))?;
        Ok(json!({"resolved": true, "requestId": request_id}))
    }

    pub fn cancel_all(&self, reason: impl Into<String>) {
        self.closed.store(true, Ordering::Release);
        let reason = reason.into();
        let pending = self.pending
            .lock()
            .map(|mut pending| pending.drain().collect::<Vec<_>>())
            .unwrap_or_default();
        for (_request_id, sender) in pending {
            let _ = sender.send(Err(CoordinatorMcpLifecycleRelayError::Closed(reason.clone())));
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.lock().map(|pending| pending.len()).unwrap_or_default()
    }
}


pub type BoxServerStatusLoader =
    Arc<dyn Fn(&[String], bool) -> Result<Vec<BoxServerStatus>, String> + Send + Sync>;

pub struct CoordinatorMcpManagerBackend {
    relay: Arc<CoordinatorMcpLifecycleRelay>,
    list_box_servers: BoxServerStatusLoader,
}

impl CoordinatorMcpManagerBackend {
    pub fn new(
        relay: Arc<CoordinatorMcpLifecycleRelay>,
        list_box_servers: BoxServerStatusLoader,
    ) -> Self {
        Self { relay, list_box_servers }
    }
}

fn required_string(row: &Value, key: &str, context: &str) -> Result<String, String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("{context} requires string {key}"))
}

fn optional_string(row: &Value, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn decode_mcp_server_state(value: Value) -> Result<Vec<McpServerSummary>, String> {
    let rows = value
        .get("servers")
        .and_then(Value::as_array)
        .ok_or_else(|| "desktop MCP owner listServers response requires servers".to_string())?;
    rows.iter()
        .map(|row| {
            Ok(McpServerSummary {
                id: required_string(row, "id", "MCP server")?,
                name: required_string(row, "name", "MCP server")?,
                server_identifier: required_string(row, "serverIdentifier", "MCP server")?,
                account_key: required_string(row, "accountKey", "MCP server")?,
                plugin_id: optional_string(row, "pluginId"),
                is_team_server: row.get("isTeamServer").and_then(Value::as_bool).unwrap_or(false),
                status: required_string(row, "status", "MCP server")?,
                status_detail: optional_string(row, "statusDetail"),
                transport: required_string(row, "transport", "MCP server")?,
                tool_count: row.get("toolCount").and_then(Value::as_u64).unwrap_or_default() as usize,
                disabled_tool_count: row.get("disabledToolCount").and_then(Value::as_u64).map(|value| value as usize),
                custom_instructions: row
                    .get("customInstructions")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect()
}

pub fn decode_mcp_catalog(value: Value) -> Result<Vec<CatalogPlugin>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "desktop MCP owner catalog response must be an array".to_string())?;
    rows.iter()
        .map(|row| {
            let fields = row
                .get("fields")
                .and_then(Value::as_array)
                .map(|fields| {
                    fields.iter()
                        .map(|field| {
                            Ok(CatalogField {
                                key: required_string(field, "key", "MCP catalog field")?,
                                label: field.get("label").and_then(Value::as_str).unwrap_or_default().to_string(),
                                hint: field.get("hint").and_then(Value::as_str).unwrap_or_default().to_string(),
                                is_required: field.get("isRequired").and_then(Value::as_bool).unwrap_or(false),
                                is_secret: field.get("isSecret").and_then(Value::as_bool).unwrap_or(false),
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
                .transpose()?
                .unwrap_or_default();
            let skills = row
                .get("skills")
                .and_then(Value::as_array)
                .map(|skills| {
                    skills.iter()
                        .map(|skill| {
                            Ok(CatalogSkill {
                                name: required_string(skill, "name", "MCP catalog skill")?,
                                description: optional_string(skill, "description"),
                                source_url: optional_string(skill, "sourceUrl"),
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
                .transpose()?
                .unwrap_or_default();
            Ok(CatalogPlugin {
                id: required_string(row, "id", "MCP catalog plugin")?,
                name: required_string(row, "name", "MCP catalog plugin")?,
                display_name: optional_string(row, "displayName"),
                description: optional_string(row, "description"),
                category: optional_string(row, "category"),
                fields,
                connector_count: row.get("connectors").and_then(Value::as_array).map(Vec::len).unwrap_or_default(),
                skills,
            })
        })
        .collect()
}

pub fn decode_effective_plugins(value: Value) -> Result<Vec<EffectivePlugin>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "desktop MCP owner effective plugins response must be an array".to_string())?;
    rows.iter()
        .map(|row| {
            Ok(EffectivePlugin {
                plugin_id: required_string(row, "pluginId", "effective MCP plugin")?,
                install_mode: optional_string(row, "installMode"),
                is_enabled: row.get("isEnabled").and_then(Value::as_bool).unwrap_or(false),
                has_team_configured_variables: row
                    .get("hasTeamConfiguredVariables")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

impl McpManagerBackend for CoordinatorMcpManagerBackend {
    fn list_servers(&self) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_LIST_SERVERS_METHOD, json!({}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn list_catalog(&self, force_refresh: bool) -> Result<Vec<CatalogPlugin>, String> {
        self.relay
            .request(MCP_LIST_CATALOG_METHOD, json!({"forceRefresh": force_refresh}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_catalog)
    }

    fn list_effective_plugins(&self) -> Result<Vec<EffectivePlugin>, String> {
        self.relay
            .request(MCP_LIST_EFFECTIVE_PLUGINS_METHOD, json!({}))
            .map_err(|error| error.to_string())
            .and_then(decode_effective_plugins)
    }

    fn uninstall_plugin(&self, plugin_id: &str) -> Result<(bool, Option<String>), String> {
        self.uninstall_plugin_outcome(plugin_id)
            .map(|outcome| (outcome.removed, outcome.reason))
    }

    fn uninstall_plugin_outcome(&self, plugin_id: &str) -> Result<McpPluginUninstallOutcome, String> {
        let result = self.relay
            .request(MCP_UNINSTALL_PLUGIN_METHOD, json!({"pluginId": plugin_id}))
            .map_err(|error| error.to_string())?;
        let removed = result.get("removed").and_then(Value::as_bool)
            .ok_or_else(|| "desktop MCP owner uninstallPlugin response requires boolean removed".to_string())?;
        let reason = optional_string(&result, "reason");
        let desktop_response = result.get("state").cloned()
            .ok_or_else(|| "desktop MCP owner uninstallPlugin response requires state".to_string())
            .and_then(decode_mcp_server_state)
            // Validate with the existing decoder, but return the original reply
            // without dropping warnings, row policy or future display fields.
            .map(|_| result);
        Ok(McpPluginUninstallOutcome { removed, reason, desktop_response })
    }

    fn install_plugin(
        &self,
        plugin_id: &str,
        values: &HashMap<String, String>,
    ) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_INSTALL_PLUGIN_METHOD, json!({"entryId": plugin_id, "values": values}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn add_server(&self, name: &str, config_json: &str) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_ADD_SERVER_METHOD, json!({"name": name, "configJson": config_json}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn remove_server(&self, server_id: &str) -> Result<(bool, Option<String>, Vec<McpServerSummary>), String> {
        let result = self.relay
            .request(MCP_REMOVE_SERVER_METHOD, json!({"serverId": server_id}))
            .map_err(|error| error.to_string())?;
        let removed = result.get("removed").and_then(Value::as_bool).unwrap_or(false);
        let reason = optional_string(&result, "reason");
        let servers = result
            .get("state")
            .cloned()
            .or_else(|| result.get("servers").map(|servers| json!({"servers": servers})))
            .ok_or_else(|| "desktop MCP owner removeServer response requires state".to_string())
            .and_then(decode_mcp_server_state)?;
        Ok((removed, reason, servers))
    }

    fn restart_servers(&self) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_RESTART_SERVERS_METHOD, json!({}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn set_server_custom_instructions(&self, server_id: &str, instructions: &str) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_SET_INSTRUCTIONS_METHOD, json!({"serverId": server_id, "instructions": instructions}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn authenticate_server(
        &self,
        server_id: &str,
        account_key: &str,
        requesting_agent_id: Option<&str>,
        force_reauth: bool,
    ) -> Result<Value, String> {
        self.relay
            .request(MCP_AUTHENTICATE_SERVER_METHOD, json!({
                "serverId": server_id,
                "accountKey": account_key,
                "requestingAgentId": requesting_agent_id,
                "forceReauth": force_reauth,
            }))
            .map_err(|error| error.to_string())
    }

    fn logout_account(&self, server_id: &str, account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_LOGOUT_ACCOUNT_METHOD, json!({"serverId": server_id, "accountKey": account_key}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn remove_account(&self, server_id: &str, account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_REMOVE_ACCOUNT_METHOD, json!({"serverId": server_id, "accountKey": account_key}))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn rename_account(&self, server_id: &str, account_key: &str, new_account_key: &str) -> Result<Vec<McpServerSummary>, String> {
        self.relay
            .request(MCP_RENAME_ACCOUNT_METHOD, json!({
                "serverId": server_id,
                "accountKey": account_key,
                "newAccountKey": new_account_key,
            }))
            .map_err(|error| error.to_string())
            .and_then(decode_mcp_server_state)
    }

    fn list_box_servers(
        &self,
        ids: &[String],
        kick_only: bool,
    ) -> Result<Vec<BoxServerStatus>, String> {
        (self.list_box_servers)(ids, kick_only)
    }

    fn note_auth_completed_elsewhere(&self, server_id: &str, account_key: &str) {
        if let Err(error) = self.relay.request(
            MCP_NOTE_AUTH_COMPLETED_METHOD,
            json!({"serverId": server_id, "accountKey": account_key}),
        ) {
            eprintln!("Host MCP external-auth completion relay failed: {error}");
        }
    }

    fn dispose(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::extensions::mcp::extension::{
        AuthRenewalEvent, AuthRenewalPort, McpExtensionRuntime,
        PluginSkillsAuthenticatedStartup, PollingHandle, PollingPort,
    };
    use crate::extensions::mcp::mcp_service::McpHostService;
    use crate::extensions::mcp::plugin_skills::SandPluginSkillsService;
    use crate::extensions::mcp::plugin_skills_cache::{
        PluginSkillsCacheWriteIndex, get_plugin_skills_cache_path,
        get_plugin_skills_dir, write_plugin_skills_cache,
    };
    use crate::extensions::mcp::production::RealPluginSkillsPolling;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::thread;

    const ORACLE_TIMEOUT: Duration = Duration::from_secs(5);
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

    fn completes_without_deadlock(check: impl FnOnce() + Send + 'static) {
        let (done_tx, done_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            check();
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(20))
            .expect("MCP lifecycle contract did not complete before the request timeout");
        worker.join().expect("MCP lifecycle contract worker");
    }

    struct RegistrationPause {
        entered: mpsc::Receiver<()>,
        release: mpsc::Sender<()>,
        attempts: Arc<AtomicUsize>,
    }

    impl Drop for RegistrationPause {
        fn drop(&mut self) {
            let _ = self.release.send(());
        }
    }

    fn paused_relay(
        events: GatewayEventHub,
    ) -> (Arc<CoordinatorMcpLifecycleRelay>, RegistrationPause) {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let attempts = Arc::new(AtomicUsize::new(0));
        let hook_attempts = Arc::clone(&attempts);
        let mut relay = CoordinatorMcpLifecycleRelay::with_timeout(events, REQUEST_TIMEOUT);
        relay.before_pending_registration = Some(Arc::new(move || {
            hook_attempts.fetch_add(1, Ordering::SeqCst);
            entered_tx.send(()).expect("registration pause entered");
            release_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(30))
                .expect("release actual request registration");
        }));
        (
            Arc::new(relay),
            RegistrationPause {
                entered: entered_rx,
                release: release_tx,
                attempts,
            },
        )
    }

    #[test]
    fn close_before_pending_registration_refuses_the_actual_request() {
        completes_without_deadlock(|| {
            let events = GatewayEventHub::default();
            let receiver = events.subscribe();
            let (relay, pause) = paused_relay(events);
            let (result_tx, result_rx) = mpsc::channel();
            let worker_relay = Arc::clone(&relay);
            let worker = thread::spawn(move || {
                result_tx
                    .send(worker_relay.request(MCP_LOAD_PLUGIN_SKILLS_METHOD, json!({})))
                    .unwrap();
            });
            pause.entered.recv_timeout(ORACLE_TIMEOUT).unwrap();
            assert_eq!(relay.pending_count(), 0);

            relay.cancel_all("Host teardown before registration");
            pause.release.send(()).unwrap();
            assert_eq!(
                result_rx
                    .recv_timeout(ORACLE_TIMEOUT)
                    .expect("losing admission must return Closed without waiting for a reply"),
                Err(CoordinatorMcpLifecycleRelayError::Closed(
                    "Mahayana Host is shutting down".into()
                ))
            );
            worker.join().unwrap();
            assert_eq!(pause.attempts.load(Ordering::SeqCst), 1);
            assert_eq!(relay.pending_count(), 0);
            assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert!(matches!(
                relay.request(MCP_LOAD_PLUGIN_SKILLS_METHOD, json!({})),
                Err(CoordinatorMcpLifecycleRelayError::Closed(_))
            ));
            assert_eq!(pause.attempts.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn exhausted_request_sequence_never_registers_or_publishes() {
        completes_without_deadlock(|| {
            let events = GatewayEventHub::default();
            let receiver = events.subscribe();
            let relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
                events,
                REQUEST_TIMEOUT,
            ));
            // Exercise the real allocation/admission path at its final ID.
            relay.next_request_id.store(u64::MAX - 1, Ordering::Release);
            let (result_tx, result_rx) = mpsc::channel();
            let worker_relay = Arc::clone(&relay);
            let worker = thread::spawn(move || {
                let _ = result_tx.send(worker_relay.request(MCP_LIST_CATALOG_METHOD, json!({})));
            });
            let event = receiver
                .recv_timeout(ORACLE_TIMEOUT)
                .expect("last sequence must still admit a real request");
            let request_id = event["payload"]["requestId"].as_str().unwrap();
            assert_eq!(relay.pending_count(), 1);

            for _ in 0..2 {
                assert_eq!(
                    relay.request(MCP_LIST_CATALOG_METHOD, json!({})),
                    Err(CoordinatorMcpLifecycleRelayError::Protocol(
                        "Host MCP lifecycle request sequence exhausted".into()
                    ))
                );
                assert_eq!(relay.pending_count(), 1);
                assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
                assert!(matches!(result_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            }
            assert_eq!(relay.next_request_id.load(Ordering::Acquire), u64::MAX);
            assert_eq!(
                request_id,
                format!("host-mcp-{}-{}", relay.incarnation, u64::MAX)
            );
            assert!(request_id.len() < 256);

            let response = json!({
                "requestId": request_id,
                "ok": true,
                "result": [{"id": "last-plugin"}],
            });
            relay
                .resolve(&response)
                .expect("last waiter keeps its own reply");
            assert_eq!(
                result_rx.recv_timeout(ORACLE_TIMEOUT).unwrap(),
                Ok(json!([{"id": "last-plugin"}]))
            );
            worker.join().unwrap();
            assert_eq!(relay.pending_count(), 0);
            assert!(matches!(
                relay.resolve(&response),
                Err(CoordinatorMcpLifecycleRelayError::Protocol(_))
            ));
            assert_eq!(
                relay.request(MCP_LIST_CATALOG_METHOD, json!({})),
                Err(CoordinatorMcpLifecycleRelayError::Protocol(
                    "Host MCP lifecycle request sequence exhausted".into()
                ))
            );
            assert_eq!(relay.pending_count(), 0);
            assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
        });
    }

    struct TestRoot(PathBuf);

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct Authenticated {
        unsubscriptions: Arc<AtomicUsize>,
    }

    impl AuthRenewalPort for Authenticated {
        fn peek_access_token(&self) -> Option<String> {
            Some("fixture-access-token".into())
        }

        fn subscribe_to_renewal(
            &self,
            _listener: Arc<dyn Fn(AuthRenewalEvent) + Send + Sync>,
        ) -> Box<dyn FnOnce() + Send> {
            let unsubscriptions = Arc::clone(&self.unsubscriptions);
            Box::new(move || {
                unsubscriptions.fetch_add(1, Ordering::SeqCst);
            })
        }
    }

    struct JoinObservedPolling {
        joining: mpsc::Sender<()>,
        callbacks: Arc<AtomicUsize>,
        disposals: Arc<AtomicUsize>,
    }

    struct JoinObservedHandle {
        inner: Box<dyn PollingHandle>,
        joining: mpsc::Sender<()>,
        disposals: Arc<AtomicUsize>,
    }

    impl PollingHandle for JoinObservedHandle {
        fn dispose(&mut self) {
            self.disposals.fetch_add(1, Ordering::SeqCst);
            let _ = self.joining.send(());
            self.inner.dispose();
        }
    }

    impl PollingPort for JoinObservedPolling {
        fn start(&self, callback: Arc<dyn Fn() + Send + Sync>) -> Box<dyn PollingHandle> {
            let callbacks = Arc::clone(&self.callbacks);
            Box::new(JoinObservedHandle {
                inner: RealPluginSkillsPolling::daily().start(Arc::new(move || {
                    callbacks.fetch_add(1, Ordering::SeqCst);
                    callback();
                })),
                joining: self.joining.clone(),
                disposals: Arc::clone(&self.disposals),
            })
        }
    }

    #[test]
    fn closed_relay_does_not_strand_real_plugin_skills_poll_shutdown() {
        completes_without_deadlock(|| {
            let root = TestRoot(std::env::temp_dir().join(format!(
                "mcp-relay-close-{}",
                uuid::Uuid::new_v4()
            )));
            let cache_dir = get_plugin_skills_dir(&root.0);
            write_plugin_skills_cache(
                &cache_dir,
                &PluginSkillsCacheWriteIndex {
                    current_user_id: Some(Some(11)),
                    skills: vec![json!({
                        "id": "retained-skill",
                        "pluginId": "41",
                        "pluginName": "Retained plugin",
                        "name": "Retained skill",
                        "description": "Available before shutdown",
                        "filePath": root.0.join("retained/SKILL.md"),
                    })],
                    auth_blocked: None,
                },
                || 1000.0,
            )
            .unwrap();
            let cache_path = get_plugin_skills_cache_path(&cache_dir);
            let cache_before = fs::read(&cache_path).unwrap();
            let events = GatewayEventHub::default();
            let receiver = events.subscribe();
            let (relay, pause) = paused_relay(events);
            let skills = Arc::new(SandPluginSkillsService::new(
                root.0.clone(),
                Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&relay))),
            ));
            let retained_skills = skills.current();
            assert_eq!(retained_skills.len(), 1);
            let service = Arc::new(McpHostService::new(
                Arc::new(CoordinatorMcpManagerBackend::new(
                    Arc::clone(&relay),
                    Arc::new(|_, _| Ok(Vec::new())),
                )),
                Some(skills.clone()),
            ));
            let callbacks = Arc::new(AtomicUsize::new(0));
            let disposals = Arc::new(AtomicUsize::new(0));
            let unsubscriptions = Arc::new(AtomicUsize::new(0));
            let sweeps = Arc::new(AtomicUsize::new(0));
            let (joining_tx, joining_rx) = mpsc::channel();
            let startup = PluginSkillsAuthenticatedStartup::start(
                Arc::new(Authenticated {
                    unsubscriptions: Arc::clone(&unsubscriptions),
                }),
                skills.clone(),
                Arc::new(JoinObservedPolling {
                    joining: joining_tx,
                    callbacks: Arc::clone(&callbacks),
                    disposals: Arc::clone(&disposals),
                }),
                Some({
                    let sweeps = Arc::clone(&sweeps);
                    Arc::new(move || {
                        sweeps.fetch_add(1, Ordering::SeqCst);
                    })
                }),
            );
            let runtime = Arc::new(McpExtensionRuntime::new(startup, service, Arc::new(())));
            pause.entered.recv_timeout(ORACLE_TIMEOUT).unwrap();

            // Match shipping teardown: close the relay before stopping MCP.
            relay.cancel_all("Host shutdown");
            let (stopped_tx, stopped_rx) = mpsc::channel();
            let stopping_runtime = Arc::clone(&runtime);
            let stopping = thread::spawn(move || {
                stopped_tx.send(stopping_runtime.stop()).unwrap();
            });
            joining_rx.recv_timeout(ORACLE_TIMEOUT).unwrap();
            assert!(matches!(stopped_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            pause.release.send(()).unwrap();
            stopped_rx
                .recv_timeout(ORACLE_TIMEOUT)
                .expect("real poll join must finish before the relay request timeout")
                .unwrap();
            stopping.join().unwrap();
            runtime.stop().unwrap();

            assert_eq!(callbacks.load(Ordering::SeqCst), 1);
            assert_eq!(disposals.load(Ordering::SeqCst), 1);
            assert_eq!(unsubscriptions.load(Ordering::SeqCst), 1);
            assert_eq!(sweeps.load(Ordering::SeqCst), 0);
            assert_eq!(skills.sync("after-stop").unwrap(), retained_skills);
            assert_eq!(fs::read(&cache_path).unwrap(), cache_before);
            assert_eq!(pause.attempts.load(Ordering::SeqCst), 1);
            assert_eq!(relay.pending_count(), 0);
            assert!(matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)));
        });
    }

    #[test]
    fn desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff() {
        use crate::extensions::session::agent_session::SandAgentSessionStore;
        use crate::extensions::session::production::ProductionSessionWorkers;

        completes_without_deadlock(|| {
            let root = TestRoot(std::env::temp_dir().join(format!(
                "desktop-mcp-handoff-{}", uuid::Uuid::new_v4(),
            )));
            let mut records = Vec::new();
            let mut files = Vec::new();
            for id in ["41", "42"] {
                let file = root.0.join(format!("plugins/cache/cursor/p{id}/v1/skills/retained/SKILL.md"));
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, format!(
                    "---\nname: Retained\ndescription: DESCRIPTION_{id}\n---\nBODY_{id}\n",
                )).unwrap();
                records.push(json!({
                    "id":format!("plugin-{id}-retained"), "pluginId":id,
                    "pluginName":format!("Plugin {id}"), "name":"Retained",
                    "description":format!("DESCRIPTION_{id}"), "filePath":file,
                }));
                files.push(file);
            }
            let cache_dir = get_plugin_skills_dir(&root.0);
            write_plugin_skills_cache(&cache_dir, &PluginSkillsCacheWriteIndex {
                current_user_id: Some(Some(11)), skills: records, auth_blocked: Some(Vec::new()),
            }, || 1_000.0).unwrap();
            let cache_path = get_plugin_skills_cache_path(&cache_dir);
            let before = fs::read(&cache_path).unwrap();
            let file_before = fs::read(&files[0]).unwrap();
            let events = GatewayEventHub::default();
            let requests = events.subscribe();
            let old_relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
                events.clone(), REQUEST_TIMEOUT,
            ));
            let old_skills = Arc::new(SandPluginSkillsService::new(
                &root.0, Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&old_relay))),
            ));
            let old_service = Arc::new(McpHostService::new(
                Arc::new(CoordinatorMcpManagerBackend::new(
                    Arc::clone(&old_relay), Arc::new(|_, _| Ok(Vec::new())),
                )), Some(old_skills.clone()),
            ));
            let workers = Arc::new(ProductionSessionWorkers::with_agents_root(root.0.join("agents"), 500));
            let session = SandAgentSessionStore::new(Arc::clone(&workers));
            let agent = session.create_session(None, "user", None).unwrap();
            let retained = workers.open_workflow_store(&agent.id).unwrap();
            workers.bind_plugin_skills_admission(old_skills.admission_reader()).unwrap();
            assert_eq!(retained.get("plugin-41-retained").unwrap().body, "BODY_41");

            let settle = |relay: &Arc<CoordinatorMcpLifecycleRelay>, method: &str, value: Value| {
                let event = requests.recv_timeout(ORACLE_TIMEOUT).unwrap();
                assert_eq!(event["payload"]["method"], method);
                relay.resolve(&json!({
                    "requestId":event["payload"]["requestId"], "ok":true, "result":value,
                })).unwrap();
            };
            let dispatch = |service: Arc<McpHostService>, id: &'static str| {
                let (tx, rx) = mpsc::channel();
                let worker = thread::spawn(move || {
                    let _ = tx.send(service.dispatch_desktop_command(
                        "uninstallMcpPlugin", &json!({"pluginId":id}),
                    ).unwrap());
                });
                (worker, rx)
            };
            let (worker, result) = dispatch(Arc::clone(&old_service), "41");
            settle(&old_relay, MCP_LIST_CATALOG_METHOD, json!([]));
            settle(&old_relay, MCP_UNINSTALL_PLUGIN_METHOD, json!({"removed":true,"state":null}));
            assert!(result.recv_timeout(ORACLE_TIMEOUT).unwrap().unwrap_err().contains("requires servers"));
            worker.join().unwrap();
            assert!(retained.get("plugin-41-retained").is_none());

            let successor_relay = Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
                events, REQUEST_TIMEOUT,
            ));
            let successor = Arc::new(old_skills.successor_with_loader(
                Arc::new(CoordinatorPluginSkillsLoader::new(Arc::clone(&successor_relay))),
            ).unwrap());
            assert!(Arc::ptr_eq(&old_skills.admission_reader(), &successor.admission_reader()));
            workers.bind_plugin_skills_admission(successor.admission_reader()).unwrap();
            assert!(successor.admission_reader().is_denied("41"));
            assert!(retained.get("plugin-41-retained").is_none());
            assert!(workers.open_workflow_store(&agent.id).unwrap().get("plugin-41-retained").is_none());
            assert_eq!(retained.get("plugin-42-retained").unwrap().body, "BODY_42");

            // A retained predecessor can still receive a remote confirmation;
            // the retired actual Skills port must refuse it, without writing
            // through to its successor's live observation state.
            let (worker, result) = dispatch(Arc::clone(&old_service), "42");
            settle(&old_relay, MCP_LIST_CATALOG_METHOD, json!([]));
            settle(&old_relay, MCP_UNINSTALL_PLUGIN_METHOD, json!({"removed":true,"state":null}));
            let error = result.recv_timeout(ORACLE_TIMEOUT).unwrap().unwrap_err();
            worker.join().unwrap();
            assert!(error.contains("uninstall confirmed") && error.contains("disposed"));
            assert!(!successor.admission_reader().is_denied("42"));
            assert_eq!(retained.get("plugin-42-retained").unwrap().body, "BODY_42");
            assert_eq!(fs::read(&cache_path).unwrap(), before);
            assert_eq!(fs::read(&files[0]).unwrap(), file_before);
            assert!(matches!(requests.try_recv(), Err(mpsc::TryRecvError::Empty)));
            old_relay.cancel_all("fixture complete");
            successor_relay.cancel_all("fixture complete");
            old_service.dispose().unwrap();
            successor.dispose();
            workers.shutdown();
        });
    }
}
