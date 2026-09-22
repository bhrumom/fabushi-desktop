use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::webauthn::{
    ApprovedWebAuthnConsent, WebAuthnCeremony, WebAuthnSigner, WebAuthnSignerError,
    WebAuthnSignerResult,
};

pub const SIGNER_EVENT_PREFIX: &str = "[signer-event] ";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SignerEvent {
    PresenceRequired,
    SelectDevice,
    PinNotSet,
    PinBlocked,
    UvBlocked,
    UvInvalid,
    PinRequired,
    PinInvalid {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retries: Option<u32>,
    },
}

pub fn describe_signer_event_as_status(event: &SignerEvent) -> Option<&'static str> {
    match event {
        SignerEvent::PresenceRequired => Some("Touch your security key now"),
        SignerEvent::SelectDevice => Some("Touch the security key you want to use"),
        SignerEvent::PinNotSet => Some("This security key has no PIN set, but the site asked for one"),
        SignerEvent::PinBlocked => Some("Your security key is locked"),
        SignerEvent::UvBlocked => Some("Your security key's fingerprint check is locked"),
        SignerEvent::UvInvalid => Some("That didn't match — try again on the key"),
        SignerEvent::PinRequired | SignerEvent::PinInvalid { .. } => None,
    }
}

pub fn parse_signer_event_line(line: &str) -> Option<SignerEvent> {
    let body = line.strip_prefix(SIGNER_EVENT_PREFIX)?;
    serde_json::from_str(body).ok()
}

pub fn signer_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "sand-webauthn-signer.exe"
    } else {
        "sand-webauthn-signer"
    }
}

pub fn resolve_web_authn_signer_path(
    is_packaged: bool,
    resources_path: &Path,
    repo_root: &Path,
    override_path: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(path) = override_path {
        return path.is_file().then(|| path.to_path_buf());
    }

    let candidates = if is_packaged {
        vec![
            resources_path
                .join("app.asar.unpacked")
                .join("dist")
                .join("native")
                .join(signer_binary_name()),
        ]
    } else {
        vec![
            repo_root.join("target").join("release").join(signer_binary_name()),
            repo_root.join("target").join("debug").join(signer_binary_name()),
        ]
    };
    candidates.into_iter().find(|path| path.is_file())
}

#[derive(Debug, Clone)]
pub struct SpawnedWebAuthnSigner {
    pub binary_path: PathBuf,
}

impl SpawnedWebAuthnSigner {
    fn failure(
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> WebAuthnSignerResult {
        WebAuthnSignerResult::Failed {
            error: WebAuthnSignerError {
                name: "NotAllowedError".into(),
                code: Some(code.into()),
                message: message.into(),
            },
        }
    }
}

impl WebAuthnSigner for SpawnedWebAuthnSigner {
    fn sign(
        &mut self,
        ceremony: &WebAuthnCeremony,
        approved: Option<&ApprovedWebAuthnConsent>,
    ) -> WebAuthnSignerResult {
        let mut child = match Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                return Self::failure(
                    "helper_spawn_failed",
                    format!("could not start the security key helper: {error}"),
                );
            }
        };

        let mut request = serde_json::to_value(ceremony).unwrap_or_else(|_| json!({}));
        if let Some(window_handle) = approved.and_then(|value| value.window_handle.as_ref()) {
            if let Some(object) = request.as_object_mut() {
                object.insert("windowHandle".into(), Value::String(window_handle.clone()));
            }
        }

        let Some(stdin) = child.stdin.as_mut() else {
            return Self::failure(
                "helper_spawn_failed",
                "security key helper stdin is unavailable",
            );
        };
        if writeln!(stdin, "{}", request).is_err() {
            return Self::failure(
                "helper_io_failed",
                "could not write the security key request",
            );
        }
        drop(child.stdin.take());

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => {
                return Self::failure(
                    "helper_wait_failed",
                    format!("security key helper wait failed: {error}"),
                );
            }
        };
        if output.stdout.is_empty() {
            return Self::failure(
                "helper_no_result",
                format!(
                    "the security key helper exited with code {:?} and no result",
                    output.status.code()
                ),
            );
        }

        let value = match serde_json::from_slice::<Value>(&output.stdout) {
            Ok(value) => value,
            Err(error) => {
                return Self::failure(
                    "helper_no_result",
                    format!("unreadable security key helper result: {error}"),
                );
            }
        };
        if value.get("ok").and_then(Value::as_bool) == Some(true) {
            return WebAuthnSignerResult::Success {
                credential_json: value
                    .get("credentialJson")
                    .cloned()
                    .unwrap_or(Value::Null),
            };
        }

        let error = value
            .get("error")
            .cloned()
            .and_then(|value| serde_json::from_value::<WebAuthnSignerError>(value).ok())
            .unwrap_or_else(|| WebAuthnSignerError {
                name: "NotAllowedError".into(),
                code: Some("helper_failed".into()),
                message: "the security key helper reported a failure".into(),
            });
        WebAuthnSignerResult::Failed { error }
    }
}
