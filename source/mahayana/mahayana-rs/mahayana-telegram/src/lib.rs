//! Telegram conversation adapter for the shared Mahayana runtime.

use async_trait::async_trait;
use fabushi_telegram_core::MessageContent;
use fabushi_telegram_core::TelegramState;
use fabushi_telegram_runtime::authorization_snapshot;
use fabushi_telegram_runtime::close_client;
use fabushi_telegram_runtime::create_client;
use fabushi_telegram_runtime::create_persistent_client;
use fabushi_telegram_runtime::execute_json;
pub use fabushi_telegram_runtime::AuthorizationState as TelegramAuthorizationState;
use mahayana_conversation::ConversationError;
use mahayana_conversation::ConversationProvider;
use mahayana_conversation::ResolveApprovalRequest;
use mahayana_conversation::SendMessageRequest;
use mahayana_conversation::SharedConversationEventSink;
use mahayana_core::Conversation;
use mahayana_core::ConversationId;
use mahayana_core::Message;
use mahayana_core::MessageId;
use mahayana_core::MessageRole;
use mahayana_core::OperationId;
use mahayana_core::PeerKind;
use mahayana_core::RuntimeEvent;
use serde_json::Value;
use serde_json::json;
use std::path::Path;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

const CONVERSATION_PREFIX: &str = "telegram:chat:";
const MAX_AUTH_BACKOFF_SECONDS: u32 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TelegramAuthIssue {
    code: String,
    category: &'static str,
    action: &'static str,
    recoverable: bool,
    retry_after_seconds: Option<u32>,
}

impl TelegramAuthIssue {
    fn as_value(&self) -> Value {
        json!({
            "code": self.code,
            "category": self.category,
            "action": self.action,
            "recoverable": self.recoverable,
            "retryAfterSeconds": self.retry_after_seconds,
        })
    }
}

fn remote_rpc_error_type(message: &str) -> Option<&str> {
    let candidate = message
        .rsplit_once(": ")
        .map(|(_, value)| value)
        .unwrap_or(message)
        .trim();
    (!candidate.is_empty()
        && candidate.len() <= 128
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'))
    .then_some(candidate)
}

fn flood_wait_seconds(code: &str) -> Option<u32> {
    ["FLOOD_WAIT_", "FLOOD_PREMIUM_WAIT_"]
        .into_iter()
        .find_map(|prefix| code.strip_prefix(prefix))
        .and_then(|value| value.parse::<u32>().ok())
        .map(|seconds| seconds.clamp(1, MAX_AUTH_BACKOFF_SECONDS))
}

fn classify_telegram_auth_issue(runtime_code: &str, message: &str) -> Option<TelegramAuthIssue> {
    if runtime_code == "auth_context_missing" {
        return Some(TelegramAuthIssue {
            code: "AUTH_CONTEXT_MISSING".into(),
            category: "state",
            action: "restartAuthorization",
            recoverable: true,
            retry_after_seconds: None,
        });
    }
    if runtime_code != "telegram_rpc_error" {
        return None;
    }

    let code = remote_rpc_error_type(message)
        .unwrap_or("TELEGRAM_RPC_ERROR")
        .to_string();
    let retry_after_seconds = flood_wait_seconds(&code);
    let (category, action, recoverable) = if retry_after_seconds.is_some()
        || code == "PHONE_NUMBER_FLOOD"
    {
        ("rateLimit", "retryLater", true)
    } else {
        match code.as_str() {
            "PHONE_NUMBER_INVALID" => ("input", "editPhoneNumber", true),
            "PHONE_NUMBER_BANNED" => ("policy", "contactSupport", false),
            "PHONE_CODE_EMPTY" | "PHONE_CODE_INVALID" => ("input", "retryCode", true),
            "PHONE_CODE_EXPIRED" => ("state", "restartAuthorization", true),
            "PASSWORD_HASH_INVALID" | "SRP_PASSWORD_CHANGED" => {
                ("input", "retryPassword", true)
            }
            "PASSWORD_EMPTY" | "AUTH_KEY_UNREGISTERED" | "PHONE_NUMBER_OCCUPIED" => {
                ("state", "restartAuthorization", true)
            }
            "SRP_ID_INVALID" => ("state", "refreshPasswordChallenge", true),
            "FIRSTNAME_INVALID" | "LASTNAME_INVALID" => {
                ("input", "retryRegistration", true)
            }
            "SEND_CODE_UNAVAILABLE" => ("availability", "retryLater", true),
            "EMAIL_NOT_ALLOWED" | "EMAIL_INVALID" => ("input", "editEmail", true),
            "EMAIL_HASH_EXPIRED" => ("state", "restartEmailVerification", true),
            "PASSWORD_RECOVERY_NA" => ("state", "restartAuthorization", true),
            "PASSWORD_RECOVERY_EXPIRED" => ("state", "restartPasswordRecovery", true),
            _ => ("remote", "retryCurrentStep", true),
        }
    };
    Some(TelegramAuthIssue {
        code,
        category,
        action,
        recoverable,
        retry_after_seconds,
    })
}

#[derive(Clone, PartialEq, Eq)]
pub struct TelegramApplicationCredentials {
    api_id: i32,
    api_hash: String,
}

impl std::fmt::Debug for TelegramApplicationCredentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TelegramApplicationCredentials")
            .field("api_id", &self.api_id)
            .field("api_hash", &"<redacted>")
            .finish()
    }
}

impl TelegramApplicationCredentials {
    pub fn new(api_id: i32, api_hash: impl Into<String>) -> Result<Self, ConversationError> {
        let api_hash = api_hash.into().trim().to_ascii_lowercase();
        if api_id <= 0
            || api_hash.len() != 32
            || !api_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ConversationError::Provider(
                "Telegram application credentials failed bounded validation".into(),
            ));
        }
        Ok(Self { api_id, api_hash })
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProxyKind {
    Socks5,
    Http,
    Mtproto,
    Web,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NetworkProxyEndpointProjection {
    pub kind: NetworkProxyKind,
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NetworkProxyProjection {
    pub enabled: bool,
    pub selected: Option<NetworkProxyEndpointProjection>,
    pub try_ipv6: bool,
}

impl NetworkProxyKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Socks5 => "socks5",
            Self::Http => "http",
            Self::Mtproto => "mtproto",
            Self::Web => "web",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramClientHandoff {
    pub client_id: u64,
    pub authorization_state: TelegramAuthorizationState,
    pub self_user_id: Option<i64>,
}

/// Process-lifetime owner for the one embedded Telegram client selected by the
/// canonical platform account lifecycle. ConversationProvider receives only a
/// non-owning typed handoff; no second Telegram account/runtime is created.
pub struct TelegramClientLease {
    client_id: u64,
    application_credentials: Option<TelegramApplicationCredentials>,
    authorization_backoff_until_ms: Mutex<Option<u64>>,
}

impl TelegramClientLease {
    pub fn open_persistent(
        database_path: &Path,
        storage_key: &[u8],
    ) -> Result<Self, ConversationError> {
        Self::open_persistent_with_application_credentials(database_path, storage_key, None)
    }

    pub fn open_persistent_with_application_credentials(
        database_path: &Path,
        storage_key: &[u8],
        application_credentials: Option<TelegramApplicationCredentials>,
    ) -> Result<Self, ConversationError> {
        let database_path = database_path.to_str().ok_or_else(|| {
            ConversationError::Provider("Telegram database path is not UTF-8".into())
        })?;
        let client_id = create_persistent_client(database_path, storage_key)
            .map_err(|error| ConversationError::Provider(error.to_string()))?;
        Ok(Self {
            client_id,
            application_credentials,
            authorization_backoff_until_ms: Mutex::new(None),
        })
    }

    pub fn handoff(&self) -> Result<TelegramClientHandoff, ConversationError> {
        let authorization = authorization_snapshot(self.client_id)
            .map_err(|error| ConversationError::Provider(error.to_string()))?;
        Ok(TelegramClientHandoff {
            client_id: self.client_id,
            authorization_state: authorization.state,
            self_user_id: authorization.self_user_id,
        })
    }

    fn runtime_envelope(&self, request: Value) -> Result<Value, ConversationError> {
        serde_json::from_str(&execute_json(self.client_id, &request.to_string()))
            .map_err(|error| ConversationError::Provider(error.to_string()))
    }

    fn runtime_failure(response: &Value) -> ConversationError {
        let code = response
            .get("errorCode")
            .and_then(Value::as_str)
            .unwrap_or("telegram_error");
        let message = response
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Telegram command failed");
        ConversationError::Provider(format!("{code}: {message}"))
    }

    fn execute(&self, request: Value) -> Result<Value, ConversationError> {
        let response = self.runtime_envelope(request)?;
        if response.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(Self::runtime_failure(&response));
        }
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ConversationError::Provider("Telegram response data is missing".into()))
    }

    fn authorization_backoff_remaining_seconds(&self) -> Result<Option<u32>, ConversationError> {
        let now = now_ms().max(0) as u64;
        let mut deadline = self.authorization_backoff_until_ms.lock().map_err(|_| {
            ConversationError::Provider("Telegram authorization backoff lock is unavailable".into())
        })?;
        let Some(until_ms) = *deadline else {
            return Ok(None);
        };
        if until_ms <= now {
            *deadline = None;
            return Ok(None);
        }
        let remaining_ms = until_ms - now;
        let remaining_seconds = (remaining_ms.saturating_add(999) / 1000)
            .min(u64::from(MAX_AUTH_BACKOFF_SECONDS)) as u32;
        Ok(Some(remaining_seconds.max(1)))
    }

    fn set_authorization_backoff(&self, retry_after_seconds: u32) -> Result<(), ConversationError> {
        let seconds = retry_after_seconds.clamp(1, MAX_AUTH_BACKOFF_SECONDS);
        let now = now_ms().max(0) as u64;
        let deadline_ms = now.saturating_add(u64::from(seconds).saturating_mul(1000));
        let mut deadline = self.authorization_backoff_until_ms.lock().map_err(|_| {
            ConversationError::Provider("Telegram authorization backoff lock is unavailable".into())
        })?;
        *deadline = Some(deadline_ms);
        Ok(())
    }

    fn decorate_authorization(&self, mut response: Value) -> Result<Value, ConversationError> {
        let handoff = self.handoff()?;
        let object = response.as_object_mut().ok_or_else(|| {
            ConversationError::Provider("Telegram authorization response is not an object".into())
        })?;
        object.insert(
            "authorizationState".into(),
            serde_json::to_value(&handoff.authorization_state)
                .map_err(|error| ConversationError::Provider(error.to_string()))?,
        );
        object.insert("selfUserId".into(), json!(handoff.self_user_id));
        if let Some(retry_after_seconds) = self.authorization_backoff_remaining_seconds()? {
            object.insert(
                "backoff".into(),
                json!({"retryAfterSeconds": retry_after_seconds}),
            );
        }
        Ok(response)
    }

    fn authorization_issue_projection(
        &self,
        issue: TelegramAuthIssue,
    ) -> Result<Value, ConversationError> {
        if let Some(retry_after_seconds) = issue.retry_after_seconds {
            self.set_authorization_backoff(retry_after_seconds)?;
        }
        self.decorate_authorization(json!({
            "@type": "telegram.authorizationIssue",
            "error": issue.as_value(),
        }))
    }

    fn authorization_backoff_projection(&self) -> Result<Option<Value>, ConversationError> {
        let Some(retry_after_seconds) = self.authorization_backoff_remaining_seconds()? else {
            return Ok(None);
        };
        let issue = TelegramAuthIssue {
            code: "AUTH_BACKOFF_ACTIVE".into(),
            category: "rateLimit",
            action: "retryLater",
            recoverable: true,
            retry_after_seconds: Some(retry_after_seconds),
        };
        Ok(Some(self.decorate_authorization(json!({
            "@type": "telegram.authorizationIssue",
            "error": issue.as_value(),
        }))?))
    }

    pub fn authorization_status(&self) -> Result<Value, ConversationError> {
        self.decorate_authorization(json!({}))
    }


    fn network_proxy_request(projection: &NetworkProxyProjection) -> Value {
        let selected = projection.selected.as_ref().map(|proxy| {
            json!({
                "type": proxy.kind.as_str(),
                "host": proxy.host,
                "port": proxy.port,
                "user": proxy.user,
                "password": proxy.password,
            })
        });
        json!({
            "@type": "telegram.applyNetworkProxy",
            "enabled": projection.enabled,
            "tryIpv6": projection.try_ipv6,
            "selected": selected,
        })
    }

    pub fn apply_network_proxy(
        &self,
        projection: &NetworkProxyProjection,
    ) -> Result<Value, ConversationError> {
        self.execute(Self::network_proxy_request(projection))
    }

    fn special_config_refresh_request(proxy_enabled: bool) -> Value {
        json!({
            "@type": "telegram.refreshSpecialConfig",
            "proxyEnabled": proxy_enabled,
        })
    }

    pub fn refresh_special_config(&self, proxy_enabled: bool) -> Result<Value, ConversationError> {
        self.execute(Self::special_config_refresh_request(proxy_enabled))
    }

    fn execute_authorization(&self, request: Value) -> Result<Value, ConversationError> {
        if let Some(projection) = self.authorization_backoff_projection()? {
            return Ok(projection);
        }
        let response = self.runtime_envelope(request)?;
        if response.get("ok").and_then(Value::as_bool) == Some(true) {
            let data = response.get("data").cloned().ok_or_else(|| {
                ConversationError::Provider("Telegram response data is missing".into())
            })?;
            return self.decorate_authorization(data);
        }
        let runtime_code = response
            .get("errorCode")
            .and_then(Value::as_str)
            .unwrap_or("telegram_error");
        let message = response
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Telegram command failed");
        if let Some(issue) = classify_telegram_auth_issue(runtime_code, message) {
            return self.authorization_issue_projection(issue);
        }
        Err(Self::runtime_failure(&response))
    }

    pub fn send_authentication_code(&self, phone_number: &str) -> Result<Value, ConversationError> {
        if phone_number.trim().is_empty() {
            return Err(ConversationError::Provider(
                "Telegram phone number is required".into(),
            ));
        }
        if matches!(self.handoff()?.authorization_state, TelegramAuthorizationState::Ready) {
            return Err(ConversationError::Provider(
                "Telegram account is already authorized".into(),
            ));
        }
        if let Some(projection) = self.authorization_backoff_projection()? {
            return Ok(projection);
        }
        let credentials = self.application_credentials.as_ref().ok_or_else(|| {
            ConversationError::Provider(
                "Telegram application credentials are unavailable".into(),
            )
        })?;
        let status = self.execute(json!({"@type": "telegram.getStatus"}))?;
        let transport_available = status
            .get("transportConnected")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || status
                .get("transportRestorable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        if !transport_available {
            self.execute(json!({
                "@type": "telegram.bootstrapTransport",
                "dcId": 2,
                "testMode": false,
            }))?;
        }
        self.execute(json!({
            "@type": "telegram.initializeConnection",
            "apiId": credentials.api_id,
            "deviceModel": "Fabushi Desktop",
            "systemVersion": std::env::consts::OS,
            "appVersion": env!("CARGO_PKG_VERSION"),
            "systemLangCode": "en",
            "langPack": "",
            "langCode": "en",
        }))?;
        self.execute_authorization(json!({
            "@type": "telegram.sendAuthenticationCode",
            "phoneNumber": phone_number.trim(),
            "apiId": credentials.api_id,
            "apiHash": credentials.api_hash.as_str(),
        }))
    }

    pub fn submit_authentication_code(&self, code: &str) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.submitAuthenticationCode",
            "code": code.trim(),
        }))
    }

    pub fn submit_authentication_password(
        &self,
        password: &str,
    ) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.submitAuthenticationPassword",
            "password": password,
        }))
    }

    pub fn submit_registration(
        &self,
        first_name: &str,
        last_name: Option<&str>,
    ) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.submitRegistration",
            "firstName": first_name.trim(),
            "lastName": last_name.unwrap_or("").trim(),
        }))
    }

    pub fn resend_authentication_code(
        &self,
        reason: Option<&str>,
    ) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.resendAuthenticationCode",
            "reason": reason.map(str::trim).filter(|value| !value.is_empty()),
        }))
    }

    pub fn cancel_authentication_code(&self) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.cancelAuthenticationCode",
        }))
    }

    pub fn request_password_recovery(&self) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.requestPasswordRecovery",
        }))
    }

    pub fn check_recovery_password(&self, code: &str) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.checkRecoveryPassword",
            "code": code.trim(),
        }))
    }

    pub fn send_verify_email_code(&self, email: &str) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.sendVerifyEmailCode",
            "email": email.trim(),
        }))
    }

    pub fn verify_email(&self, code: &str) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.verifyEmail",
            "code": code.trim(),
        }))
    }

    pub fn refresh_password_challenge(&self) -> Result<Value, ConversationError> {
        self.execute_authorization(json!({
            "@type": "telegram.refreshPasswordChallenge",
        }))
    }
}

impl Drop for TelegramClientLease {
    fn drop(&mut self) {
        let _ = close_client(self.client_id);
    }
}

/// Adapts the embedded Telegram command/event core to the product-level
/// conversation contract. The client remains alive for the lifetime of this
/// provider; no helper subprocess or cloud Agent gateway is involved.
pub struct TelegramConversationProvider {
    client_id: u64,
    self_user_id: i64,
    runtime_identity: bool,
    next_local_message_id: AtomicI64,
    owns_client: bool,
}

impl TelegramConversationProvider {
    pub fn new_ephemeral(self_user_id: i64) -> Self {
        Self {
            client_id: create_client(),
            self_user_id,
            runtime_identity: false,
            next_local_message_id: AtomicI64::new(-1),
            owns_client: true,
        }
    }

    pub fn open_persistent(
        database_path: &Path,
        storage_key: &[u8],
        self_user_id: i64,
    ) -> Result<Self, ConversationError> {
        let database_path = database_path.to_str().ok_or_else(|| {
            ConversationError::Provider("Telegram database path is not UTF-8".into())
        })?;
        let client_id = create_persistent_client(database_path, storage_key)
            .map_err(|error| ConversationError::Provider(error.to_string()))?;
        Ok(Self {
            client_id,
            self_user_id,
            runtime_identity: false,
            next_local_message_id: AtomicI64::new(-1),
            owns_client: true,
        })
    }

    /// Wraps an existing Telegram client owned by another platform component.
    pub fn from_client_id(client_id: u64, self_user_id: Option<i64>) -> Self {
        Self {
            client_id,
            self_user_id: self_user_id.unwrap_or_default(),
            runtime_identity: true,
            next_local_message_id: AtomicI64::new(-1),
            owns_client: false,
        }
    }

    pub fn client_id(&self) -> u64 {
        self.client_id
    }

    fn canonical_self_user_id(&self) -> Result<i64, ConversationError> {
        if !self.runtime_identity {
            return Ok(self.self_user_id);
        }
        let authorization = authorization_snapshot(self.client_id)
            .map_err(|error| ConversationError::Provider(error.to_string()))?;
        match (authorization.state, authorization.self_user_id) {
            (TelegramAuthorizationState::Ready, Some(self_user_id)) => Ok(self_user_id),
            _ => Err(ConversationError::Provider(
                "Telegram account is not authorized with a canonical self user identity".into(),
            )),
        }
    }

    fn state(&self) -> Result<TelegramState, ConversationError> {
        self.canonical_self_user_id()?;
        let response = self.execute(json!({"@type": "telegram.getState"}))?;
        serde_json::from_value(
            response
                .get("state")
                .cloned()
                .ok_or_else(|| ConversationError::Provider("Telegram state is missing".into()))?,
        )
        .map_err(|error| ConversationError::Provider(error.to_string()))
    }

    fn execute(&self, request: Value) -> Result<Value, ConversationError> {
        let response: Value =
            serde_json::from_str(&execute_json(self.client_id, &request.to_string()))
                .map_err(|error| ConversationError::Provider(error.to_string()))?;
        if response.get("ok").and_then(Value::as_bool) != Some(true) {
            let code = response
                .get("errorCode")
                .and_then(Value::as_str)
                .unwrap_or("telegram_error");
            let message = response
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Telegram command failed");
            return Err(ConversationError::Provider(format!("{code}: {message}")));
        }
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ConversationError::Provider("Telegram response data is missing".into()))
    }
}

impl Drop for TelegramConversationProvider {
    fn drop(&mut self) {
        if self.owns_client {
            let _ = close_client(self.client_id);
        }
    }
}

#[async_trait]
impl ConversationProvider for TelegramConversationProvider {
    fn key(&self) -> &'static str {
        "telegram"
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, ConversationError> {
        let state = self.state()?;
        Ok(state
            .chats
            .values()
            .map(|chat| {
                let updated_at_ms = chat
                    .last_message_id
                    .and_then(|message_id| state.messages.get(&(chat.id, message_id)))
                    .map(|message| message.date_unix_ms)
                    .unwrap_or_default();
                Conversation {
                    id: conversation_id(chat.id.0),
                    title: chat.title.clone(),
                    peer: PeerKind::TelegramContact { user_id: chat.id.0 },
                    pinned: chat.pinned_message_id.is_some(),
                    unread_count: chat.unread_count,
                    updated_at_ms,
                }
            })
            .collect())
    }

    async fn history(
        &self,
        conversation_id: &ConversationId,
        limit: u32,
    ) -> Result<Vec<Message>, ConversationError> {
        let chat_id = parse_chat_id(conversation_id)?;
        let state = self.state()?;
        if !state.chats.keys().any(|candidate| candidate.0 == chat_id) {
            return Err(ConversationError::ConversationNotFound(
                conversation_id.clone(),
            ));
        }
        let self_user_id = self.canonical_self_user_id()?;
        let mut messages: Vec<_> = state
            .messages
            .values()
            .filter(|message| message.chat_id.0 == chat_id && !message.is_deleted)
            .map(|message| Message {
                id: telegram_message_id(chat_id, message.id.0),
                conversation_id: conversation_id.clone(),
                role: if message.sender_user_id.0 == self_user_id || message.is_outgoing {
                    MessageRole::User
                } else {
                    MessageRole::Contact
                },
                text: message_content_text(&message.content),
                created_at_ms: message.date_unix_ms,
                metadata: json!({
                    "telegramMessageId": message.id.0,
                    "senderUserId": message.sender_user_id.0,
                    "deliveryState": message.delivery_state,
                }),
            })
            .collect();
        messages.sort_by_key(|message| message.created_at_ms);
        let start = messages.len().saturating_sub(limit as usize);
        Ok(messages[start..].to_vec())
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
        events: SharedConversationEventSink,
    ) -> Result<(), ConversationError> {
        let chat_id = parse_chat_id(&request.conversation_id)?;
        let self_user_id = self.canonical_self_user_id()?;
        let local_message_id = self.next_local_message_id.fetch_sub(1, Ordering::Relaxed);
        let now_ms = now_ms();
        let client_request_id = request
            .client_message_id
            .as_deref()
            .filter(|value| !value.trim().is_empty() && value.len() <= 128)
            .unwrap_or_else(|| request.operation_id.as_str());
        self.execute(json!({
            "@type": "telegram.executeCoreCommand",
            "command": {
                "type": "queueMessage",
                "chatId": chat_id,
                "localMessageId": local_message_id,
                "senderUserId": self_user_id,
                "clientRequestId": client_request_id,
                "dateUnixMs": now_ms,
                "content": {
                    "type": "text",
                    "data": {"text": request.text, "entities": []}
                },
                "replyToMessageId": null,
                "messageThreadId": null
            }
        }))?;
        events.emit(RuntimeEvent::MessageCompleted {
            operation_id: request.operation_id,
            message: Message {
                id: telegram_message_id(chat_id, local_message_id),
                conversation_id: request.conversation_id,
                role: MessageRole::User,
                text: request.text,
                created_at_ms: now_ms,
                metadata: json!({"telegramMessageId": local_message_id, "queued": true}),
            },
        })
    }

    async fn interrupt(&self, _operation_id: &OperationId) -> Result<(), ConversationError> {
        Ok(())
    }

    async fn resolve_approval(
        &self,
        request: ResolveApprovalRequest,
    ) -> Result<(), ConversationError> {
        Err(ConversationError::ApprovalNotFound(request.approval_id))
    }
}

fn conversation_id(chat_id: i64) -> ConversationId {
    ConversationId(format!("{CONVERSATION_PREFIX}{chat_id}"))
}

fn parse_chat_id(conversation_id: &ConversationId) -> Result<i64, ConversationError> {
    conversation_id
        .as_str()
        .strip_prefix(CONVERSATION_PREFIX)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| ConversationError::UnsupportedConversation(conversation_id.to_string()))
}

fn telegram_message_id(chat_id: i64, message_id: i64) -> MessageId {
    MessageId(format!("telegram:chat:{chat_id}:message:{message_id}"))
}

fn message_content_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(text) => text.text.clone(),
        MessageContent::Photo { caption, .. } => media_text("[图片]", &caption.text),
        MessageContent::Video { caption, .. } => media_text("[视频]", &caption.text),
        MessageContent::Animation { caption, .. } => media_text("[动图]", &caption.text),
        MessageContent::Audio { caption, .. } => media_text("[音频]", &caption.text),
        MessageContent::VoiceNote { caption, .. } => media_text("[语音]", &caption.text),
        MessageContent::VideoNote { .. } => "[视频消息]".into(),
        MessageContent::Document { caption, .. } => media_text("[文件]", &caption.text),
        MessageContent::Sticker { emoji, .. } => format!("[贴纸] {emoji}"),
        MessageContent::Poll { question, .. } => format!("[投票] {}", question.text),
        MessageContent::Location { .. } => "[位置]".into(),
        MessageContent::Venue { title, address, .. } => format!("[地点] {title} {address}"),
        MessageContent::Dice { emoji, value } => format!("{emoji} {value}"),
        MessageContent::Story { story_id, .. } => format!("[动态] {}", story_id.0),
        MessageContent::Invoice {
            title,
            currency,
            total_amount_minor,
            ..
        } => format!("[账单] {title} {currency} {total_amount_minor}"),
        MessageContent::Contact {
            first_name,
            last_name,
            ..
        } => format!("[联系人] {first_name} {last_name}")
            .trim_end()
            .to_string(),
        MessageContent::Service { action } => format!("[系统消息] {action:?}"),
        MessageContent::Unsupported { constructor } => format!("[不支持的消息] {constructor}"),
    }
}

fn media_text(label: &str, caption: &str) -> String {
    if caption.is_empty() {
        label.to_string()
    } else {
        format!("{label} {caption}")
    }
}

fn now_ms() -> i64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabushi_telegram_core::Chat;
    use fabushi_telegram_core::ChatId;
    use fabushi_telegram_core::ChatKind;
    use fabushi_telegram_core::DeliveryState;
    use fabushi_telegram_core::FormattedText;
    use fabushi_telegram_core::Message as TelegramMessage;
    use fabushi_telegram_core::MessageId as TelegramMessageId;
    use fabushi_telegram_core::UserId;
    use std::sync::Arc;
    use std::sync::Mutex;


    #[test]
    fn network_proxy_request_carries_only_transient_carrier_projection() {
        let projection = NetworkProxyProjection {
            enabled: true,
            try_ipv6: true,
            selected: Some(NetworkProxyEndpointProjection {
                kind: NetworkProxyKind::Mtproto,
                host: "proxy.example".into(),
                port: 443,
                user: String::new(),
                password: "00112233445566778899aabbccddeeff".into(),
            }),
        };
        let request = TelegramClientLease::network_proxy_request(&projection);
        assert_eq!(request.get("@type").and_then(Value::as_str), Some("telegram.applyNetworkProxy"));
        assert_eq!(request.get("enabled").and_then(Value::as_bool), Some(true));
        assert_eq!(request.get("tryIpv6").and_then(Value::as_bool), Some(true));
        assert_eq!(
            request.pointer("/selected/type").and_then(Value::as_str),
            Some("mtproto")
        );
        assert!(request.get("settings").is_none());
        assert!(request.get("list").is_none());
        assert!(request.get("rotation").is_none());
    }

    #[test]
    fn special_config_refresh_request_carries_authoritative_proxy_projection() {
        assert_eq!(
            TelegramClientLease::special_config_refresh_request(true),
            json!({
                "@type": "telegram.refreshSpecialConfig",
                "proxyEnabled": true,
            })
        );
        assert_eq!(
            TelegramClientLease::special_config_refresh_request(false),
            json!({
                "@type": "telegram.refreshSpecialConfig",
                "proxyEnabled": false,
            })
        );
    }

    #[derive(Default)]
    struct EventCollector(Mutex<Vec<RuntimeEvent>>);

    impl mahayana_conversation::ConversationEventSink for EventCollector {
        fn emit(&self, event: RuntimeEvent) -> Result<(), ConversationError> {
            self.0.lock().expect("event mutex").push(event);
            Ok(())
        }
    }

    #[test]
    fn auth_rpc_errors_have_typed_actions_and_bounded_backoff() {
        let flood = classify_telegram_auth_issue(
            "telegram_rpc_error",
            "Telegram RPC failed with 420: FLOOD_WAIT_37",
        )
        .expect("flood issue");
        assert_eq!(flood.code, "FLOOD_WAIT_37");
        assert_eq!(flood.category, "rateLimit");
        assert_eq!(flood.action, "retryLater");
        assert_eq!(flood.retry_after_seconds, Some(37));

        let bounded = classify_telegram_auth_issue(
            "telegram_rpc_error",
            "Telegram RPC failed with 420: FLOOD_WAIT_999999999",
        )
        .expect("bounded flood issue");
        assert_eq!(bounded.retry_after_seconds, Some(MAX_AUTH_BACKOFF_SECONDS));

        let invalid_code = classify_telegram_auth_issue(
            "telegram_rpc_error",
            "Telegram RPC failed with 400: PHONE_CODE_INVALID",
        )
        .expect("invalid-code issue");
        assert_eq!(invalid_code.category, "input");
        assert_eq!(invalid_code.action, "retryCode");
        assert_eq!(invalid_code.retry_after_seconds, None);

        let missing_context = classify_telegram_auth_issue(
            "auth_context_missing",
            "Telegram authentication code context is unavailable",
        )
        .expect("missing-context issue");
        assert_eq!(missing_context.action, "restartAuthorization");
        assert!(missing_context.recoverable);

        assert!(classify_telegram_auth_issue("network_failed", "offline").is_none());
    }

    #[tokio::test]
    async fn exposes_telegram_chats_history_and_queued_messages() {
        let provider = TelegramConversationProvider::new_ephemeral(7);
        provider
            .execute(json!({
                "@type": "telegram.executeCoreCommand",
                "command": {
                    "type": "upsertChat",
                    "chat": Chat::new(ChatId(42), ChatKind::Private, "善友")
                }
            }))
            .expect("upsert chat");
        provider
            .execute(json!({
                "@type": "telegram.executeCoreCommand",
                "command": {
                    "type": "upsertRemoteMessage",
                    "message": TelegramMessage {
                        id: TelegramMessageId(9),
                        chat_id: ChatId(42),
                        sender_user_id: UserId(42),
                        date_unix_ms: 100,
                        edit_date_unix_ms: None,
                        content: MessageContent::Text(FormattedText::plain("南无阿弥陀佛")),
                        reply_to_message_id: None,
                        message_thread_id: None,
                        delivery_state: DeliveryState::Sent,
                        reactions: vec![],
                        is_outgoing: false,
                        is_pinned: false,
                        is_deleted: false,
                    }
                }
            }))
            .expect("upsert message");

        let conversations = provider.list_conversations().await.expect("list chats");
        assert_eq!(conversations[0].id.as_str(), "telegram:chat:42");
        let history = provider
            .history(&conversation_id(42), 50)
            .await
            .expect("history");
        assert_eq!(history[0].text, "南无阿弥陀佛");
        assert_eq!(history[0].role, MessageRole::Contact);

        let events = Arc::new(EventCollector::default());
        provider
            .send_message(
                SendMessageRequest {
                    conversation_id: conversation_id(42),
                    operation_id: OperationId("operation:test".into()),
                    text: "收到".into(),
                    client_message_id: Some("client:test".into()),
                    inference_provider: None,
                    hidden: false,
                },
                events.clone(),
            )
            .await
            .expect("queue message");
        assert!(matches!(
            events.0.lock().expect("events").as_slice(),
            [RuntimeEvent::MessageCompleted { .. }]
        ));
        assert_eq!(
            provider
                .history(&conversation_id(42), 50)
                .await
                .expect("history")
                .last()
                .expect("queued message")
                .text,
            "收到"
        );
    }
}
