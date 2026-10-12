use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, RecvTimeoutError},
};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zeroize::Zeroizing;

use crate::webauthn::{
    ApprovedWebAuthnConsent, WebAuthnCeremony, WebAuthnSigner, WebAuthnSignerError,
    WebAuthnSignerResult,
};

pub const SIGNER_EVENT_PREFIX: &str = "[signer-event] ";
const MAX_SIGNER_STDOUT_BYTES: usize = 1024 * 1024;
const MAX_SIGNER_EVENT_STREAM_BYTES: usize = 256 * 1024;
const MIN_WEBAUTHN_CEREMONY_TIMEOUT_MS: u64 = 300_000;
const MAX_WEBAUTHN_CEREMONY_TIMEOUT_MS: u64 = 600_000;
const DEFAULT_WEBAUTHN_CEREMONY_TIMEOUT_MS: u64 = 300_000;
const MAX_WEBAUTHN_OPTIONS_JSON_BYTES: usize = 256 * 1024;

fn timeout_hint_ms(value: &Value) -> Option<u64> {
    value
        .get("timeout")
        .and_then(Value::as_u64)
        .or_else(|| {
            value
                .get("publicKey")
                .and_then(|public_key| public_key.get("timeout"))
                .and_then(Value::as_u64)
        })
}

fn ceremony_timeout(ceremony: &WebAuthnCeremony) -> Duration {
    let timeout_hint = timeout_hint_ms(&ceremony.payload).or_else(|| {
        let options_json = ceremony.payload.get("optionsJson")?.as_str()?;
        if options_json.len() > MAX_WEBAUTHN_OPTIONS_JSON_BYTES {
            return None;
        }
        let options = serde_json::from_str::<Value>(options_json).ok()?;
        timeout_hint_ms(&options)
    });
    // WebAuthn Level 3 recommends a client range of 5-10 minutes and a
    // 5-minute default. The RP value is a hint, so clamp rather than trusting
    // an unbounded or too-short remote timeout.
    Duration::from_millis(
        timeout_hint
            .unwrap_or(DEFAULT_WEBAUTHN_CEREMONY_TIMEOUT_MS)
            .clamp(
                MIN_WEBAUTHN_CEREMONY_TIMEOUT_MS,
                MAX_WEBAUTHN_CEREMONY_TIMEOUT_MS,
            ),
    )
}

#[derive(Debug)]
enum BoundedReadError {
    Io(std::io::Error),
    LimitExceeded,
}

fn read_bounded(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, BoundedReadError> {
    let mut bytes = Vec::new();
    let limit_with_sentinel = limit.saturating_add(1);
    reader
        .by_ref()
        .take(limit_with_sentinel as u64)
        .read_to_end(&mut bytes)
        .map_err(BoundedReadError::Io)?;
    if bytes.len() > limit {
        return Err(BoundedReadError::LimitExceeded);
    }
    Ok(bytes)
}

#[derive(Debug, Clone, Default)]
pub struct WebAuthnSignCancellation {
    cancelled: Arc<AtomicBool>,
}

impl WebAuthnSignCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAuthnPinRequest {
    pub invalid: bool,
    pub retries: Option<u32>,
}

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

fn parse_signer_event_bytes(line: &[u8]) -> Result<Option<SignerEvent>, &'static str> {
    let line = match std::str::from_utf8(line) {
        Ok(line) => line.trim(),
        Err(_) if line.starts_with(SIGNER_EVENT_PREFIX.as_bytes()) => {
            return Err("security key helper event is not valid UTF-8");
        }
        Err(_) => return Ok(None),
    };
    if !line.starts_with(SIGNER_EVENT_PREFIX) {
        return Ok(None);
    }
    parse_signer_event_line(line)
        .map(Some)
        .ok_or("security key helper emitted a malformed signer event")
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


fn parse_helper_result(
    exit_success: bool,
    exit_code: Option<i32>,
    output: &[u8],
) -> WebAuthnSignerResult {
    if output.is_empty() {
        return SpawnedWebAuthnSigner::failure(
            "helper_no_result",
            format!("the security key helper exited with code {exit_code:?} and no result"),
        );
    }

    let value = match serde_json::from_slice::<Value>(output) {
        Ok(value) => value,
        Err(error) => {
            return SpawnedWebAuthnSigner::failure(
                "helper_no_result",
                format!("unreadable security key helper result: {error}"),
            );
        }
    };

    if value.get("ok").and_then(Value::as_bool) == Some(true) {
        if !exit_success {
            return SpawnedWebAuthnSigner::failure(
                "helper_failed",
                format!(
                    "the security key helper reported success but exited with code {exit_code:?}"
                ),
            );
        }
        let Some(credential_json) = value
            .get("credentialJson")
            .filter(|value| value.is_object())
            .cloned()
        else {
            return SpawnedWebAuthnSigner::failure(
                "helper_result_invalid",
                "the security key helper reported success without an object credentialJson",
            );
        };
        return WebAuthnSignerResult::Success { credential_json };
    }

    let error = value
        .get("error")
        .cloned()
        .and_then(|value| serde_json::from_value::<WebAuthnSignerError>(value).ok())
        .unwrap_or_else(|| WebAuthnSignerError {
            name: "NotAllowedError".into(),
            code: Some("helper_failed".into()),
            message: if exit_success {
                "the security key helper reported a failure".into()
            } else {
                format!("the security key helper exited with code {exit_code:?}")
            },
        });
    WebAuthnSignerResult::Failed { error }
}

impl SpawnedWebAuthnSigner {
    pub fn sign_interactive<Status, Pin>(
        &self,
        ceremony: &WebAuthnCeremony,
        approved: Option<&ApprovedWebAuthnConsent>,
        cancellation: &WebAuthnSignCancellation,
        on_status: Status,
        on_pin_request: Pin,
    ) -> WebAuthnSignerResult
    where
        Status: FnMut(&str),
        Pin: FnMut(WebAuthnPinRequest, &str) -> Option<String>,
    {
        self.sign_interactive_with_timeout(
            ceremony,
            approved,
            cancellation,
            ceremony_timeout(ceremony),
            on_status,
            on_pin_request,
        )
    }

    fn sign_interactive_with_timeout<Status, Pin>(
        &self,
        ceremony: &WebAuthnCeremony,
        approved: Option<&ApprovedWebAuthnConsent>,
        cancellation: &WebAuthnSignCancellation,
        timeout: Duration,
        mut on_status: Status,
        mut on_pin_request: Pin,
    ) -> WebAuthnSignerResult
    where
        Status: FnMut(&str),
        Pin: FnMut(WebAuthnPinRequest, &str) -> Option<String>,
    {
        let started_at = Instant::now();

        #[derive(Debug)]
        enum ProcessEvent {
            Stdout(Vec<u8>),
            Signer(SignerEvent),
            Failure {
                code: &'static str,
                message: String,
            },
        }

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

        let Some(mut stdin) = child.stdin.take() else {
            let _ = child.kill();
            return Self::failure(
                "helper_spawn_failed",
                "security key helper stdin is unavailable",
            );
        };
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            return Self::failure(
                "helper_spawn_failed",
                "security key helper stdout is unavailable",
            );
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            return Self::failure(
                "helper_spawn_failed",
                "security key helper stderr is unavailable",
            );
        };

        let mut request = serde_json::to_value(ceremony).unwrap_or_else(|_| json!({}));
        if let Some(window_handle) = approved.and_then(|value| value.window_handle) {
            if let Some(object) = request.as_object_mut() {
                object.insert("windowHandle".into(), Value::from(window_handle));
            }
        }
        if writeln!(stdin, "{request}")
            .and_then(|_| stdin.flush())
            .is_err()
        {
            let _ = child.kill();
            return Self::failure(
                "helper_io_failed",
                "could not write the security key request",
            );
        }

        let (events_tx, events_rx) = mpsc::channel::<ProcessEvent>();
        let stdout_tx = events_tx.clone();
        let stdout_thread = thread::spawn(move || {
            let event = match read_bounded(stdout, MAX_SIGNER_STDOUT_BYTES) {
                Ok(bytes) => ProcessEvent::Stdout(bytes),
                Err(BoundedReadError::LimitExceeded) => ProcessEvent::Failure {
                    code: "helper_output_too_large",
                    message: format!(
                        "security key helper stdout exceeded {MAX_SIGNER_STDOUT_BYTES} bytes"
                    ),
                },
                Err(BoundedReadError::Io(error)) => ProcessEvent::Failure {
                    code: "helper_io_failed",
                    message: format!("could not read security key helper stdout: {error}"),
                },
            };
            let _ = stdout_tx.send(event);
        });
        let stderr_tx = events_tx.clone();
        let stderr_thread = thread::spawn(move || {
            let mut reader = BufReader::new(stderr)
                .take((MAX_SIGNER_EVENT_STREAM_BYTES.saturating_add(1)) as u64);
            let mut consumed = 0usize;
            loop {
                let mut line = Vec::new();
                let read = match reader.read_until(b'\n', &mut line) {
                    Ok(read) => read,
                    Err(error) => {
                        let _ = stderr_tx.send(ProcessEvent::Failure {
                            code: "helper_io_failed",
                            message: format!(
                                "could not read security key helper event stream: {error}"
                            ),
                        });
                        break;
                    }
                };
                if read == 0 {
                    break;
                }
                consumed = consumed.saturating_add(read);
                if consumed > MAX_SIGNER_EVENT_STREAM_BYTES {
                    let _ = stderr_tx.send(ProcessEvent::Failure {
                        code: "helper_output_too_large",
                        message: format!(
                            "security key helper event stream exceeded {MAX_SIGNER_EVENT_STREAM_BYTES} bytes"
                        ),
                    });
                    break;
                }
                match parse_signer_event_bytes(&line) {
                    Ok(Some(event)) => {
                        let _ = stderr_tx.send(ProcessEvent::Signer(event));
                    }
                    Ok(None) => {}
                    Err(message) => {
                        let _ = stderr_tx.send(ProcessEvent::Failure {
                            code: "helper_event_invalid",
                            message: message.into(),
                        });
                        break;
                    }
                }
            }
        });
        drop(events_tx);

        let mut stdout_bytes: Option<Vec<u8>> = None;
        let process_event = |event: ProcessEvent,
                             stdin: &mut std::process::ChildStdin,
                             stdout_bytes: &mut Option<Vec<u8>>,
                             on_status: &mut Status,
                             on_pin_request: &mut Pin|
         -> Result<(), WebAuthnSignerResult> {
            match event {
                ProcessEvent::Stdout(bytes) => {
                    *stdout_bytes = Some(bytes);
                    Ok(())
                }
                ProcessEvent::Failure { code, message } => Err(Self::failure(code, message)),
                ProcessEvent::Signer(event) => {
                    let request = match event {
                        SignerEvent::PinRequired => Some(WebAuthnPinRequest {
                            invalid: false,
                            retries: None,
                        }),
                        SignerEvent::PinInvalid { retries } => Some(WebAuthnPinRequest {
                            invalid: true,
                            retries,
                        }),
                        other => {
                            if let Some(status) = describe_signer_event_as_status(&other) {
                                on_status(status);
                            }
                            None
                        }
                    };
                    let Some(request) = request else {
                        return Ok(());
                    };
                    let pin = approved
                        .and_then(|value| value.prompt_id.as_deref())
                        .and_then(|prompt_id| on_pin_request(request, prompt_id))
                        .filter(|pin| !pin.is_empty());
                    match pin {
                        Some(pin) => {
                            #[derive(Serialize)]
                            struct PinReply<'a> {
                                kind: &'static str,
                                pin: &'a str,
                            }

                            let pin = Zeroizing::new(pin);
                            let reply = serde_json::to_vec(&PinReply {
                                kind: "pin",
                                pin: pin.as_str(),
                            })
                            .map(Zeroizing::new)
                            .map_err(|error| {
                                Self::failure(
                                    "helper_io_failed",
                                    format!("could not serialize security key PIN reply: {error}"),
                                )
                            })?;
                            stdin
                                .write_all(reply.as_slice())
                                .and_then(|_| stdin.write_all(b"\n"))
                                .and_then(|_| stdin.flush())
                                .map_err(|error| {
                                    Self::failure(
                                        "helper_io_failed",
                                        format!("could not answer security key helper: {error}"),
                                    )
                                })
                        }
                        None => stdin
                            .write_all(b"{\"kind\":\"cancel\"}\n")
                            .and_then(|_| stdin.flush())
                            .map_err(|error| {
                                Self::failure(
                                    "helper_io_failed",
                                    format!("could not answer security key helper: {error}"),
                                )
                            }),
                    }
                }
            }
        };

        let exit_status = loop {
            if cancellation.is_cancelled() {
                let _ = writeln!(stdin, "{}", json!({ "kind": "cancel" }));
                let _ = stdin.flush();
                let _ = child.kill();
                let _ = child.wait();
                drop(stdin);
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Self::failure(
                    "cancelled_or_timeout",
                    "the security key request was cancelled",
                );
            }

            if started_at.elapsed() >= timeout {
                let _ = writeln!(stdin, "{}", json!({ "kind": "cancel" }));
                let _ = stdin.flush();
                let _ = child.kill();
                let _ = child.wait();
                drop(stdin);
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Self::failure(
                    "cancelled_or_timeout",
                    "the security key request timed out",
                );
            }

            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    drop(stdin);
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();
                    return Self::failure(
                        "helper_wait_failed",
                        format!("security key helper wait failed: {error}"),
                    );
                }
            }

            match events_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(event) => {
                    if let Err(result) = process_event(
                        event,
                        &mut stdin,
                        &mut stdout_bytes,
                        &mut on_status,
                        &mut on_pin_request,
                    ) {
                        let _ = child.kill();
                        let _ = child.wait();
                        drop(stdin);
                        let _ = stdout_thread.join();
                        let _ = stderr_thread.join();
                        return result;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {}
            }
        };

        drop(stdin);
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();
        while let Ok(event) = events_rx.try_recv() {
            match event {
                ProcessEvent::Stdout(bytes) => stdout_bytes = Some(bytes),
                ProcessEvent::Failure { code, message } => {
                    return Self::failure(code, message);
                }
                ProcessEvent::Signer(_) => {}
            }
        }

        parse_helper_result(exit_status.success(), exit_status.code(), stdout_bytes.as_deref().unwrap_or_default())
    }
}

impl WebAuthnSigner for SpawnedWebAuthnSigner {
    fn sign(
        &mut self,
        ceremony: &WebAuthnCeremony,
        approved: Option<&ApprovedWebAuthnConsent>,
    ) -> WebAuthnSignerResult {
        let cancellation = WebAuthnSignCancellation::default();
        self.sign_interactive(
            ceremony,
            approved,
            &cancellation,
            |_| {},
            |_, _| None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error, ErrorKind};

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::Other, "synthetic read failure"))
        }
    }

    #[test]
    fn ceremony_timeout_uses_webauthn_default_and_clamps_remote_hint() {
        let make = |options_json: &str| WebAuthnCeremony {
            kind: "get".into(),
            origin: "https://example.test".into(),
            payload: json!({ "optionsJson": options_json }),
        };

        assert_eq!(
            ceremony_timeout(&make(r#"{"challenge":"abc"}"#)),
            Duration::from_millis(DEFAULT_WEBAUTHN_CEREMONY_TIMEOUT_MS),
        );
        assert_eq!(
            ceremony_timeout(&make(r#"{"challenge":"abc","timeout":10}"#)),
            Duration::from_millis(MIN_WEBAUTHN_CEREMONY_TIMEOUT_MS),
        );
        assert_eq!(
            ceremony_timeout(&make(r#"{"challenge":"abc","timeout":450000}"#)),
            Duration::from_millis(450_000),
        );
        assert_eq!(
            ceremony_timeout(&make(r#"{"challenge":"abc","timeout":900000}"#)),
            Duration::from_millis(MAX_WEBAUTHN_CEREMONY_TIMEOUT_MS),
        );
        assert_eq!(
            ceremony_timeout(&make("{not-json}")),
            Duration::from_millis(DEFAULT_WEBAUTHN_CEREMONY_TIMEOUT_MS),
        );
    }

    #[test]
    fn ceremony_timeout_accepts_nested_public_key_and_flat_future_shape() {
        let nested = WebAuthnCeremony {
            kind: "create".into(),
            origin: "https://example.test".into(),
            payload: json!({
                "optionsJson": "{\"publicKey\":{\"timeout\":420000}}"
            }),
        };
        assert_eq!(
            ceremony_timeout(&nested),
            Duration::from_millis(420_000),
        );

        let flat = WebAuthnCeremony {
            kind: "get".into(),
            origin: "https://example.test".into(),
            payload: json!({ "timeout": 480000 }),
        };
        assert_eq!(
            ceremony_timeout(&flat),
            Duration::from_millis(480_000),
        );
    }

    #[cfg(unix)]
    #[test]
    fn interactive_signer_kills_a_stuck_helper_at_the_deadline() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let helper = std::env::temp_dir().join(format!(
            "fabushi-webauthn-timeout-{}",
            uuid::Uuid::new_v4()
        ));
        fs::write(&helper, "#!/bin/sh\nexec sleep 5\n")
            .expect("write synthetic helper");
        let mut permissions = fs::metadata(&helper)
            .expect("stat synthetic helper")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&helper, permissions).expect("make synthetic helper executable");

        let signer = SpawnedWebAuthnSigner {
            binary_path: helper.clone(),
        };
        let ceremony = WebAuthnCeremony {
            kind: "get".into(),
            origin: "https://example.test".into(),
            payload: json!({ "optionsJson": "{\"challenge\":\"abc\"}" }),
        };
        let cancellation = WebAuthnSignCancellation::default();
        let result = signer.sign_interactive_with_timeout(
            &ceremony,
            None,
            &cancellation,
            Duration::from_millis(50),
            |_| {},
            |_, _| None,
        );
        let _ = fs::remove_file(helper);

        assert!(matches!(
            result,
            WebAuthnSignerResult::Failed {
                error: WebAuthnSignerError {
                    code: Some(ref code),
                    ref message,
                    ..
                }
            } if code == "cancelled_or_timeout" && message.contains("timed out")
        ));
    }

    #[test]
    fn signer_event_bytes_accept_valid_protocol_event() {
        let event = parse_signer_event_bytes(
            br#"[signer-event] {"kind":"pin-invalid","retries":2}
"#,
        )
        .expect("valid signer event should parse");
        assert_eq!(
            event,
            Some(SignerEvent::PinInvalid { retries: Some(2) })
        );
    }

    #[test]
    fn signer_event_bytes_ignore_unstructured_diagnostics() {
        assert_eq!(
            parse_signer_event_bytes(b"diagnostic output that is not protocol\n")
                .expect("ordinary diagnostics should be ignored"),
            None
        );
    }

    #[test]
    fn signer_event_bytes_reject_malformed_prefixed_json() {
        assert_eq!(
            parse_signer_event_bytes(b"[signer-event] {not-json}\n"),
            Err("security key helper emitted a malformed signer event")
        );
    }

    #[test]
    fn signer_event_bytes_reject_non_utf8_prefixed_protocol_frame() {
        let mut line = SIGNER_EVENT_PREFIX.as_bytes().to_vec();
        line.extend_from_slice(&[0xff, b'\n']);
        assert_eq!(
            parse_signer_event_bytes(&line),
            Err("security key helper event is not valid UTF-8")
        );
    }

    #[test]
    fn signer_event_bytes_ignore_non_utf8_unstructured_diagnostics() {
        assert_eq!(
            parse_signer_event_bytes(&[0xff, b'\n'])
                .expect("non-protocol diagnostics are not signer events"),
            None
        );
    }

    #[test]
    fn helper_result_accepts_success_only_with_object_credential() {
        let result = parse_helper_result(
            true,
            Some(0),
            br#"{"ok":true,"credentialJson":{"id":"credential-1"}}"#,
        );
        assert_eq!(
            result,
            WebAuthnSignerResult::Success {
                credential_json: json!({"id":"credential-1"}),
            }
        );
    }

    #[test]
    fn helper_result_rejects_missing_or_non_object_credential() {
        for output in [
            br#"{"ok":true}"#.as_slice(),
            br#"{"ok":true,"credentialJson":null}"#.as_slice(),
            br#"{"ok":true,"credentialJson":"not-an-object"}"#.as_slice(),
        ] {
            assert!(matches!(
                parse_helper_result(true, Some(0), output),
                WebAuthnSignerResult::Failed {
                    error: WebAuthnSignerError {
                        code: Some(ref code),
                        ..
                    }
                } if code == "helper_result_invalid"
            ));
        }
    }

    #[test]
    fn helper_result_rejects_success_from_nonzero_exit() {
        assert!(matches!(
            parse_helper_result(
                false,
                Some(7),
                br#"{"ok":true,"credentialJson":{"id":"credential-1"}}"#,
            ),
            WebAuthnSignerResult::Failed {
                error: WebAuthnSignerError {
                    code: Some(ref code),
                    ..
                }
            } if code == "helper_failed"
        ));
    }

    #[test]
    fn helper_result_preserves_structured_failure() {
        assert_eq!(
            parse_helper_result(
                false,
                Some(9),
                br#"{"ok":false,"error":{"name":"NotAllowedError","code":"pin_blocked","message":"locked"}}"#,
            ),
            WebAuthnSignerResult::Failed {
                error: WebAuthnSignerError {
                    name: "NotAllowedError".into(),
                    code: Some("pin_blocked".into()),
                    message: "locked".into(),
                },
            }
        );
    }

    #[test]
    fn bounded_reader_accepts_exact_limit() {
        let bytes = read_bounded(Cursor::new(vec![7_u8; 4]), 4)
            .expect("exact limit should be accepted");
        assert_eq!(bytes, vec![7_u8; 4]);
    }

    #[test]
    fn bounded_reader_rejects_limit_plus_one() {
        assert!(matches!(
            read_bounded(Cursor::new(vec![7_u8; 5]), 4),
            Err(BoundedReadError::LimitExceeded)
        ));
    }

    #[test]
    fn bounded_reader_preserves_io_failure() {
        assert!(matches!(
            read_bounded(FailingReader, 4),
            Err(BoundedReadError::Io(_))
        ));
    }
}
