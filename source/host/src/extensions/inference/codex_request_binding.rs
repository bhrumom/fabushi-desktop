//! Process-local credential source and workspace continuity for one Codex turn.
//!
//! The parent owns credential loading, refresh persistence and cancellation.
//! This binding retains identity only; every transport still loads fresh tokens.

use std::env;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::Value;

use super::{CodexCredentials, ProviderSessionError, codex_home, read_codex_credentials};

const AUTH_CLAIM: &str = "https://api.openai.com/auth";

/// Opaque identity shared by the requests belonging to one admitted Codex turn.
/// Tokens, the auth document and permission state remain with their existing owners.
#[derive(Clone)]
pub struct CodexResponsesRequestBinding {
    identity: Arc<OnceLock<CodexResponsesRequestIdentity>>,
}

#[derive(PartialEq, Eq)]
struct CodexResponsesRequestIdentity {
    credential_path: PathBuf,
    workspace: String,
}

impl fmt::Debug for CodexResponsesRequestBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexResponsesRequestBinding")
            .field("initialized", &self.is_initialized())
            .finish()
    }
}

impl CodexResponsesRequestBinding {
    /// Allocation performs no I/O. The caller checks cancellation and the
    /// provider/model/checkpoint contract before admitting the first image.
    pub(crate) fn for_runner_invocation() -> Self {
        Self { identity: Arc::new(OnceLock::new()) }
    }

    pub(super) fn is_initialized(&self) -> bool {
        self.identity.get().is_some()
    }

    /// Load through the existing reader, which validates the actual access-token
    /// workspace against the same image's account_id before returning credentials.
    /// The caller retains cancellation-before-error precedence around this I/O.
    pub(super) fn credentials_for_transport(
        &self,
    ) -> Result<CodexCredentials, ProviderSessionError> {
        let path = match self.identity.get() {
            Some(identity) => identity.credential_path.clone(),
            None => configured_absolute_credential_path()?,
        };
        let credentials = read_codex_credentials(&path)?;
        self.accept_validated_credentials(&credentials)?;
        Ok(credentials)
    }

    fn accept_validated_credentials(
        &self,
        credentials: &CodexCredentials,
    ) -> Result<(), ProviderSessionError> {
        if !credentials.path.is_absolute() {
            return Err(source_path_error());
        }
        let candidate = CodexResponsesRequestIdentity {
            credential_path: credentials.path.clone(),
            workspace: credentials.account_id.clone(),
        };
        // Both first readers may hold valid images. A losing reader must compare
        // with the accepted winner, never replace it or borrow its authorization.
        if let Err(candidate) = self.identity.set(candidate) {
            if self.identity.get() != Some(&candidate) {
                return Err(ProviderSessionError::Authentication(
                    "Codex login workspace or credential source changed during this turn. Start a new turn after signing in again."
                        .into(),
                ));
            }
        }
        Ok(())
    }
}

/// Capture identity under the caller's existing admission/cancellation guard.
/// Discard the loaded secrets; later transports reread the captured source.
pub fn capture_codex_responses_request_binding(
) -> Result<CodexResponsesRequestBinding, ProviderSessionError> {
    let binding = CodexResponsesRequestBinding::for_runner_invocation();
    let _ = binding.credentials_for_transport()?;
    Ok(binding)
}

fn configured_absolute_credential_path() -> Result<PathBuf, ProviderSessionError> {
    let path = codex_home().join("auth.json");
    let path = if path.is_absolute() {
        path
    } else {
        env::current_dir().map_err(|_| source_path_error())?.join(path)
    };
    // A drive-relative Windows path can retain its prefix after PathBuf::join.
    // Never admit a source that would still be reinterpreted against a later cwd.
    if !path.is_absolute() {
        return Err(source_path_error());
    }
    Ok(path)
}

fn source_path_error() -> ProviderSessionError {
    ProviderSessionError::Authentication(
        "Could not resolve an absolute Codex login source. Sign in again, then start a new turn."
            .into(),
    )
}

fn invalid_workspace_error() -> ProviderSessionError {
    ProviderSessionError::Authentication(
        "Codex login workspace identity is invalid. Run `codex login`, then start a new turn."
            .into(),
    )
}

fn valid_workspace(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

/// Inspect access-token metadata without verifying a signature or a principal.
/// The same gate applies to file admission and a refresh result before writeback.
pub(super) fn validate_codex_access_workspace(
    access_token: &str,
    expected_workspace: &str,
) -> Result<(), ProviderSessionError> {
    if !valid_workspace(expected_workspace) {
        return Err(invalid_workspace_error());
    }
    let mut parts = access_token.split('.');
    let payload = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(header), Some(payload), Some(signature), None)
            if !header.is_empty() && !payload.is_empty() && !signature.is_empty() => payload,
        _ => return Err(invalid_workspace_error()),
    };
    let decoded = URL_SAFE_NO_PAD.decode(payload).map_err(|_| invalid_workspace_error())?;
    let payload: Value = serde_json::from_slice(&decoded).map_err(|_| invalid_workspace_error())?;
    let workspace = payload.as_object()
        .and_then(|claims| claims.get(AUTH_CLAIM))
        .and_then(Value::as_object)
        .and_then(|claim| claim.get("chatgpt_account_id"))
        .and_then(Value::as_str)
        .filter(|workspace| valid_workspace(workspace))
        .ok_or_else(invalid_workspace_error)?;
    if workspace != expected_workspace {
        return Err(ProviderSessionError::Authentication(
            "Codex login workspace does not match its access token or this turn. Start a new turn after signing in again."
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::persist_refreshed_credentials;
    use serde_json::json;
    use std::fs::{self, File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::Path;
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::sync::Barrier;
    use std::thread;
    use std::time::{Duration, Instant};

    const CHILD_TEST: &str = "CWB_BINDING_CHILD_TEST";
    const CHILD_ROOT: &str = "CWB_BINDING_CHILD_ROOT";
    const CHILD_NONCE: &str = "CWB_BINDING_CHILD_NONCE";
    const WORKSPACE_A: &str = "CWB/Workspace-A";
    const WORKSPACE_B: &str = "CWB/Workspace-B";

    struct FixtureRoot {
        path: PathBuf,
        clean: bool,
    }

    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            if self.clean {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }

    struct OwnedChild {
        process: Child,
        terminal: bool,
        retirement: Option<String>,
    }

    impl OwnedChild {
        fn retire_once_bounded(&mut self) -> String {
            if let Some(observation) = &self.retirement {
                return observation.clone();
            }
            self.retirement = Some("retirement started; terminal outcome unknown".into());
            let kill = self.process.kill();
            let deadline = Instant::now() + Duration::from_secs(3);
            let outcome = loop {
                let observation = match self.process.try_wait() {
                    Ok(Some(status)) => {
                        self.terminal = true;
                        break format!("terminal exit observed: {status:?}");
                    }
                    Ok(None) => "child still running".to_string(),
                    Err(error) => format!("terminal observation failed: {error}"),
                };
                if Instant::now() >= deadline {
                    break format!("retirement budget exhausted; {observation}; terminal=false");
                }
                thread::sleep(Duration::from_millis(10));
            };
            let observation = format!("kill request={kill:?}; {outcome}");
            self.retirement = Some(observation.clone());
            observation
        }

        fn wait_bounded(&mut self) -> Result<ExitStatus, String> {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                match self.process.try_wait() {
                    Ok(Some(status)) => {
                        self.terminal = true;
                        return Ok(status);
                    }
                    Ok(None) if Instant::now() < deadline => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    result => {
                        let retirement = self.retire_once_bounded();
                        return Err(format!("child did not complete: {result:?}; {retirement}"));
                    }
                }
            }
        }
    }

    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if !self.terminal && self.retirement.is_none() {
                let observation = self.retire_once_bounded();
                eprintln!("CWB owned child retirement during Drop: {observation}");
            }
        }
    }

    fn private_file(path: &Path) -> File {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(path).expect("create owned private fixture file")
    }

    fn write_image(path: &Path, document: &Value) {
        write_bytes(path, &serde_json::to_vec(document).expect("serialize synthetic image"));
    }

    fn write_bytes(path: &Path, bytes: &[u8]) {
        let temporary = path.with_file_name(format!("cwb-{}.tmp", uuid::Uuid::new_v4()));
        private_file(&temporary).write_all(bytes).expect("write private fixture image");
        fs::rename(temporary, path).expect("replace owned credential image");
    }

    fn bounded_log(path: &Path) -> String {
        let mut text = String::new();
        File::open(path).expect("open owned child log")
            .take(64 * 1024).read_to_string(&mut text).expect("read bounded child log");
        text
    }

    fn isolated(name: &str, body: impl FnOnce(&Path)) {
        let (_, module) = module_path!().split_once("::").expect("library test module path");
        let exact = format!("{module}::{name}");
        if let Ok(child_test) = env::var(CHILD_TEST) {
            assert_eq!(child_test, exact, "run only the selected owned child test");
            let root = PathBuf::from(env::var_os(CHILD_ROOT).expect("owned child root"));
            let nonce = env::var(CHILD_NONCE).expect("owned child nonce");
            assert!(root.is_absolute(), "fixture root must already be absolute");
            assert!(root.file_name().unwrap().to_string_lossy().starts_with("cwb-binding-"));
            uuid::Uuid::parse_str(&nonce).expect("fixture nonce must have the owned UUID shape");
            assert_eq!(fs::read_to_string(root.join("owner-nonce")).unwrap(), nonce);
            assert_eq!(env::var_os("CODEX_HOME"), Some(root.join("codex").into_os_string()));
            assert_eq!(env::var_os("HOME"), Some(root.join("home").into_os_string()));
            assert_eq!(env::var_os("USERPROFILE"), Some(root.join("home").into_os_string()));
            assert_eq!(fs::canonicalize(env::current_dir().unwrap()).unwrap(), root);
            body(&root);
            let receipt = json!({
                "test": exact, "nonce": nonce, "pid": std::process::id(), "completed": true,
            });
            private_file(&root.join("completed.json"))
                .write_all(receipt.to_string().as_bytes()).expect("write child completion receipt");
            return;
        }

        let path = env::temp_dir().join(format!("cwb-binding-{}", uuid::Uuid::new_v4()));
        let mut directory = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            directory.mode(0o700);
        }
        directory.create(&path).expect("create unique private fixture root");
        let mut root = FixtureRoot {
            path: fs::canonicalize(path).expect("resolve owned fixture root"),
            clean: false,
        };
        fs::create_dir(root.path.join("codex")).unwrap();
        fs::create_dir(root.path.join("home")).unwrap();
        let nonce = uuid::Uuid::new_v4().to_string();
        private_file(&root.path.join("owner-nonce")).write_all(nonce.as_bytes()).unwrap();
        let stdout_path = root.path.join("stdout.log");
        let stderr_path = root.path.join("stderr.log");
        let process = Command::new(env::current_exe().expect("unit test executable"))
            .args(["--exact", &exact, "--nocapture", "--test-threads=1"])
            .env(CHILD_TEST, &exact).env(CHILD_ROOT, &root.path).env(CHILD_NONCE, &nonce)
            .env("CODEX_HOME", root.path.join("codex"))
            .env("HOME", root.path.join("home"))
            .env("USERPROFILE", root.path.join("home"))
            .env_remove("OPENAI_API_KEY").env_remove("CODEX_API_KEY")
            .current_dir(&root.path)
            .stdout(Stdio::from(private_file(&stdout_path)))
            .stderr(Stdio::from(private_file(&stderr_path)))
            .spawn().expect("spawn exact owned child");
        let mut child = OwnedChild { process, terminal: false, retirement: None };
        let pid = child.process.id();
        let status = child.wait_bounded();
        if !matches!(&status, Ok(status) if status.success()) {
            panic!(
                "CWB child failed: {status:?}; terminal={}; retained at {}\nstdout:\n{}\nstderr:\n{}",
                child.terminal, root.path.display(), bounded_log(&stdout_path), bounded_log(&stderr_path),
            );
        }
        assert!(child.terminal, "cleanup requires observed child retirement");
        let receipt: Value = serde_json::from_slice(
            &fs::read(root.path.join("completed.json")).expect("non-vacuous child completion receipt"),
        ).expect("parse child completion receipt");
        assert_eq!(receipt.get("test").and_then(Value::as_str), Some(exact.as_str()));
        assert_eq!(receipt.get("nonce").and_then(Value::as_str), Some(nonce.as_str()));
        assert_eq!(receipt.get("pid").and_then(Value::as_u64), Some(u64::from(pid)));
        assert_eq!(receipt.get("completed").and_then(Value::as_bool), Some(true));
        // Successful child libtest output stays in its private logs so the
        // enclosing command retains one summary and one named result per oracle.
        root.clean = true;
    }

    fn jwt(payload: &Value) -> String {
        format!("{}.{}.{}",
            URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#),
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap()),
            URL_SAFE_NO_PAD.encode(b"synthetic-signature"),
        )
    }

    fn access_token(workspace: &str, marker: &str) -> String {
        jwt(&json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": workspace,
                "chatgpt_user_id": format!("synthetic-user-{marker}"),
            },
            "sub": format!("synthetic-subject-{marker}"),
            "marker": marker,
        }))
    }

    fn document(workspace: &str, marker: &str) -> Value {
        json!({
            "auth_mode": "chatgpt",
            "tokens": {
                "access_token": access_token(workspace, marker),
                "refresh_token": format!("synthetic-refresh-{marker}"),
                "id_token": jwt(&json!({"aud":"synthetic-client","marker":marker})),
                "account_id": workspace,
            },
            "document_marker": format!("synthetic-document-{marker}"),
        })
    }

    fn authentication<T>(result: Result<T, ProviderSessionError>) -> ProviderSessionError {
        let error = match result {
            Ok(_) => panic!("expected a credential Authentication refusal"),
            Err(error) => error,
        };
        assert!(matches!(&error, ProviderSessionError::Authentication(_)), "{error:?}");
        error
    }

    fn assert_redacted(error: &ProviderSessionError, markers: &[&str]) {
        let rendered = format!("{error}\n{error:?}");
        for marker in markers {
            assert!(!rendered.contains(*marker), "credential diagnostic disclosed a synthetic marker");
        }
    }

    #[test]
    fn cwb_o1_private_file_rotation_and_workspace_refusal() {
        isolated("cwb_o1_private_file_rotation_and_workspace_refusal", |root| {
            let path = root.join("codex/auth.json");
            write_image(&path, &document(WORKSPACE_A, "initial"));
            let binding = capture_codex_responses_request_binding().unwrap();
            let retained = binding.clone();
            assert!(binding.is_initialized());
            assert_eq!(binding.credentials_for_transport().unwrap().path, path);

            let rotated = document(WORKSPACE_A, "rotated");
            write_image(&path, &rotated);
            let next = retained.credentials_for_transport().unwrap();
            assert_eq!(next.account_id, WORKSPACE_A);
            assert_eq!(next.access_token, rotated["tokens"]["access_token"].as_str().unwrap());
            assert_eq!(next.refresh_token, "synthetic-refresh-rotated");

            write_image(&path, &document(WORKSPACE_B, "later-login"));
            authentication(binding.credentials_for_transport());
            authentication(retained.credentials_for_transport());
            let independent = capture_codex_responses_request_binding().unwrap();
            assert_eq!(independent.credentials_for_transport().unwrap().account_id, WORKSPACE_B);

            write_image(&path, &document(WORKSPACE_A, "restored"));
            assert_eq!(binding.credentials_for_transport().unwrap().account_id, WORKSPACE_A);
            authentication(independent.credentials_for_transport());
        });
    }

    #[test]
    fn cwb_o1_invalid_sources_do_not_initialize_binding() {
        isolated("cwb_o1_invalid_sources_do_not_initialize_binding", |root| {
            let path = root.join("codex/auth.json");
            let binding = CodexResponsesRequestBinding::for_runner_invocation();
            assert!(!binding.is_initialized());
            authentication(binding.credentials_for_transport());
            assert!(!binding.is_initialized());
            fs::create_dir(&path).unwrap();
            authentication(binding.credentials_for_transport());
            fs::remove_dir(&path).unwrap();
            write_bytes(&path, b"{invalid synthetic auth JSON");
            authentication(binding.credentials_for_transport());

            let valid = document(WORKSPACE_A, "invalid-source-control");
            let mut invalid = Vec::new();
            let mut mode = valid.clone();
            mode["auth_mode"] = json!("chatgptAuthTokens");
            invalid.push(mode);
            for name in ["access_token", "refresh_token", "id_token", "account_id"] {
                let mut missing = valid.clone();
                missing["tokens"].as_object_mut().unwrap().remove(name);
                invalid.push(missing);
            }
            for token in ["opaque-synthetic-access".to_string(), jwt(&json!({})), access_token(WORKSPACE_B, "conflict")] {
                let mut image = valid.clone();
                image["tokens"]["access_token"] = json!(token);
                invalid.push(image);
            }
            let mut blank = document(" \t ", "blank-workspace");
            blank["tokens"]["account_id"] = json!(" \t ");
            invalid.push(blank);
            for image in invalid {
                write_image(&path, &image);
                authentication(binding.credentials_for_transport());
                assert!(!binding.is_initialized(), "an invalid first image cannot bind identity");
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::{PermissionsExt, symlink};
                write_image(&path, &valid);
                fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
                authentication(binding.credentials_for_transport());
                fs::remove_file(&path).unwrap();
                let target = root.join("owned-symlink-target.json");
                write_image(&target, &valid);
                symlink(&target, &path).unwrap();
                authentication(binding.credentials_for_transport());
                fs::remove_file(&path).unwrap();
                assert!(!binding.is_initialized());
            }

            write_image(&path, &valid);
            assert_eq!(binding.credentials_for_transport().unwrap().account_id, WORKSPACE_A);
            fs::remove_file(&path).unwrap();
            authentication(binding.credentials_for_transport());
            assert!(binding.is_initialized(), "a missing replacement cannot erase admission");
            write_image(&path, &valid);
            assert_eq!(binding.credentials_for_transport().unwrap().account_id, WORKSPACE_A);
        });
    }

    #[test]
    fn cwb_o1_source_path_survives_environment_and_cwd_changes() {
        isolated("cwb_o1_source_path_survives_environment_and_cwd_changes", |root| {
            let first = root.join("working-a");
            let later = root.join("working-b");
            fs::create_dir_all(first.join("relative-codex")).unwrap();
            fs::create_dir_all(later.join("relative-codex")).unwrap();
            let first_path = first.join("relative-codex/auth.json");
            let later_path = later.join("relative-codex/auth.json");
            write_image(&first_path, &document(WORKSPACE_A, "relative-first"));
            write_image(&later_path, &document(WORKSPACE_B, "relative-later"));
            env::set_current_dir(&first).unwrap();
            // Only the exact selected test runs in this owned child. No reader
            // threads are active during its process-global environment changes.
            unsafe { env::set_var("CODEX_HOME", "relative-codex"); }
            let expected_first_path = env::current_dir().unwrap().join("relative-codex/auth.json");
            let binding = capture_codex_responses_request_binding().unwrap();
            env::set_current_dir(&later).unwrap();
            assert_eq!(binding.credentials_for_transport().unwrap().path, expected_first_path);
            let expected_later_path = env::current_dir().unwrap().join("relative-codex/auth.json");
            let later_binding = capture_codex_responses_request_binding().unwrap();
            assert_eq!(later_binding.credentials_for_transport().unwrap().path, expected_later_path);
            unsafe { env::set_var("CODEX_HOME", later.join("relative-codex")); }
            assert_eq!(binding.credentials_for_transport().unwrap().account_id, WORKSPACE_A);
            fs::remove_file(&first_path).unwrap();
            authentication(binding.credentials_for_transport());
            write_image(&first_path, &document(WORKSPACE_A, "relative-rotation"));
            assert_eq!(binding.credentials_for_transport().unwrap().path, expected_first_path);

            let home = root.join("home");
            let profile = root.join("profile");
            fs::create_dir(home.join(".codex")).unwrap();
            fs::create_dir_all(profile.join(".codex")).unwrap();
            write_image(&home.join(".codex/auth.json"), &document(WORKSPACE_A, "home"));
            write_image(&profile.join(".codex/auth.json"), &document(WORKSPACE_B, "profile"));
            unsafe {
                env::set_var("CODEX_HOME", "");
                env::set_var("HOME", &home);
                env::set_var("USERPROFILE", &profile);
            }
            let home_binding = capture_codex_responses_request_binding().unwrap();
            assert_eq!(home_binding.credentials_for_transport().unwrap().path, home.join(".codex/auth.json"));
            unsafe {
                env::remove_var("CODEX_HOME");
                env::remove_var("HOME");
            }
            let profile_binding = capture_codex_responses_request_binding().unwrap();
            assert_eq!(profile_binding.credentials_for_transport().unwrap().path, profile.join(".codex/auth.json"));
            assert_eq!(home_binding.credentials_for_transport().unwrap().account_id, WORKSPACE_A);
        });
    }

    type ReaderResult = (PathBuf, String, Result<(), ProviderSessionError>);

    fn race_valid_images(
        first: CodexCredentials,
        second: CodexCredentials,
    ) -> (CodexResponsesRequestBinding, Vec<ReaderResult>) {
        // Real private-file reads have already produced both valid snapshots;
        // the barrier isolates the first-reader identity acceptance race.
        let binding = CodexResponsesRequestBinding::for_runner_invocation();
        let start = Arc::new(Barrier::new(3));
        let mut readers = Vec::new();
        for credentials in [first, second] {
            let binding = binding.clone();
            let start = Arc::clone(&start);
            readers.push(thread::spawn(move || {
                start.wait();
                let accepted = binding.accept_validated_credentials(&credentials);
                (credentials.path, credentials.account_id, accepted)
            }));
        }
        start.wait();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !readers.iter().all(|reader| reader.is_finished()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(readers.iter().all(|reader| reader.is_finished()), "first readers did not retire");
        let results = readers.into_iter().map(|reader| reader.join().expect("reader completed without panic")).collect();
        (binding, results)
    }

    #[test]
    fn cwb_o1_concurrent_first_readers_keep_one_identity() {
        isolated("cwb_o1_concurrent_first_readers_keep_one_identity", |root| {
            let path = root.join("codex/auth.json");
            write_image(&path, &document(WORKSPACE_A, "race-a"));
            let first = read_codex_credentials(&path).unwrap();
            write_image(&path, &document(WORKSPACE_A, "race-a-rotated"));
            let rotated = read_codex_credentials(&path).unwrap();
            let (same, same_results) = race_valid_images(first.clone(), rotated);
            assert!(same_results.iter().all(|(_, _, result)| result.is_ok()));
            assert_eq!(same.credentials_for_transport().unwrap().account_id, WORKSPACE_A);

            write_image(&path, &document(WORKSPACE_B, "race-b"));
            let other = read_codex_credentials(&path).unwrap();
            let (different, results) = race_valid_images(first, other);
            assert_eq!(results.iter().filter(|(_, _, result)| result.is_ok()).count(), 1);
            let (winning_path, winning_workspace, _) = results.iter().find(|(_, _, result)| result.is_ok()).unwrap();
            let (_, losing_workspace, rejected) = results.iter().find(|(_, _, result)| result.is_err()).unwrap();
            assert!(matches!(rejected, Err(ProviderSessionError::Authentication(_))));
            let identity = different.identity.get().unwrap();
            assert_eq!(&identity.credential_path, winning_path);
            assert_eq!(&identity.workspace, winning_workspace);
            write_image(&path, &document(losing_workspace, "losing-replacement"));
            authentication(different.credentials_for_transport());
            write_image(&path, &document(winning_workspace, "winning-rotation"));
            assert_eq!(&different.credentials_for_transport().unwrap().account_id, winning_workspace);

            let alternate = root.join("alternate-auth.json");
            write_image(&path, &document(WORKSPACE_A, "source-one"));
            write_image(&alternate, &document(WORKSPACE_A, "source-two"));
            let (source, results) = race_valid_images(
                read_codex_credentials(&path).unwrap(), read_codex_credentials(&alternate).unwrap(),
            );
            assert_eq!(results.iter().filter(|(_, _, result)| result.is_ok()).count(), 1);
            let (winning_path, _, _) = results.iter().find(|(_, _, result)| result.is_ok()).unwrap();
            assert_eq!(&source.credentials_for_transport().unwrap().path, winning_path);
        });
    }

    #[test]
    fn cwb_o1_refresh_workspace_gate_preserves_source_and_redacts() {
        isolated("cwb_o1_refresh_workspace_gate_preserves_source_and_redacts", |root| {
            let path = root.join("codex/auth.json");
            write_image(&path, &document(WORKSPACE_A, "before-refresh"));
            let binding = capture_codex_responses_request_binding().unwrap();
            let current = binding.credentials_for_transport().unwrap();
            let original = fs::read(&path).unwrap();
            for access in [access_token(WORKSPACE_B, "refused-refresh"), jwt(&json!({})), "opaque-refreshed-access".into()] {
                let error = authentication(persist_refreshed_credentials(
                    &current, access.clone(), "refused-refresh-secret".into(), "refused-id-secret".into(),
                ));
                assert_redacted(&error, &[&access, "refused-refresh-secret", "refused-id-secret", WORKSPACE_A, WORKSPACE_B]);
                assert_eq!(fs::read(&path).unwrap(), original, "identity refusal must precede persistence");
                assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1, "refusal cannot leave a temporary auth image");
            }
            let access = access_token(WORKSPACE_A, "accepted-refresh");
            let next = persist_refreshed_credentials(
                &current, access.clone(), "accepted-refresh-secret".into(),
                jwt(&json!({"aud":"synthetic-client","marker":"accepted-refresh"})),
            ).unwrap();
            assert_eq!(next.account_id, WORKSPACE_A);
            let reread = binding.credentials_for_transport().unwrap();
            assert_eq!(reread.access_token, access);
            assert_eq!(reread.refresh_token, "accepted-refresh-secret");
            assert_ne!(fs::read(&path).unwrap(), original);
        });
    }

    #[test]
    fn cwb_o1_jwt_shape_claim_and_workspace_validation() {
        let valid = access_token(WORKSPACE_A, "principal-one");
        validate_codex_access_workspace(&valid, WORKSPACE_A).unwrap();
        validate_codex_access_workspace(&access_token(WORKSPACE_A, "principal-two"), WORKSPACE_A).unwrap();
        let payload = valid.split('.').nth(1).unwrap();
        for token in [
            String::new(), "opaque-access".into(), format!("h.{payload}"),
            format!(".{payload}.s"), "h..s".into(), format!("h.{payload}."),
            format!("h.{payload}.s.extra"), format!("h.{payload}=.s"),
            format!("h.{payload} .s"), "h.+/8.s".into(), "h._w.s".into(),
        ] {
            let error = authentication(validate_codex_access_workspace(&token, WORKSPACE_A));
            if !token.is_empty() {
                assert_redacted(&error, &[&token]);
            }
        }
        for payload in [
            json!(null), json!(true), json!("string"), json!([]), json!({}),
            json!({"https://api.openai.com/auth": []}),
            json!({"https://api.openai.com/auth": {"chatgpt_account_id": null}}),
            json!({"https://api.openai.com/auth": {"chatgpt_account_id": 42}}),
            json!({"https://api.openai.com/auth": {"chatgpt_account_id": [WORKSPACE_A]}}),
        ] {
            authentication(validate_codex_access_workspace(&jwt(&payload), WORKSPACE_A));
        }
        for workspace in ["", "  ", "\t\n", "\u{2003}", "A\0B", "A\nB", "A\u{7f}B", "A\u{85}B"] {
            authentication(validate_codex_access_workspace(&access_token(workspace, "invalid-workspace"), workspace));
        }
        let spaced = " CWB/Workspace-A ";
        validate_codex_access_workspace(&access_token(spaced, "exact-bytes"), spaced).unwrap();
        authentication(validate_codex_access_workspace(&access_token(spaced, "exact-bytes"), spaced.trim()));
        authentication(validate_codex_access_workspace(&valid, "cwb/workspace-a"));
        authentication(validate_codex_access_workspace(&valid, WORKSPACE_B));

        // Retain a valid JSON claim while changing only discarded base64 bits.
        let mut bytes = serde_json::to_vec(&json!({"https://api.openai.com/auth":{"chatgpt_account_id":WORKSPACE_A}})).unwrap();
        while bytes.len() % 3 == 0 { bytes.push(b' '); }
        let canonical = URL_SAFE_NO_PAD.encode(&bytes);
        validate_codex_access_workspace(&format!("h.{canonical}.s"), WORKSPACE_A).unwrap();
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut noncanonical = canonical.into_bytes();
        let last = noncanonical.last_mut().unwrap();
        let index = alphabet.iter().position(|byte| *byte == *last).unwrap();
        assert_eq!(index & 1, 0);
        *last = alphabet[index | 1];
        authentication(validate_codex_access_workspace(
            &format!("h.{}.s", String::from_utf8(noncanonical).unwrap()), WORKSPACE_A,
        ));
    }

    #[test]
    fn cwb_o1_binding_debug_redacts_source_and_credentials() {
        isolated("cwb_o1_binding_debug_redacts_source_and_credentials", |root| {
            let path = root.join("codex/auth.json");
            let image = document(WORKSPACE_A, "redaction-marker");
            write_image(&path, &image);
            let binding = capture_codex_responses_request_binding().unwrap();
            let rendered = format!("{binding:?}");
            assert!(rendered.contains("initialized: true"));
            assert!(!rendered.contains(WORKSPACE_A));
            assert!(!rendered.contains(path.to_string_lossy().as_ref()));
            assert!(!rendered.contains("redaction-marker"));
            for name in ["access_token", "refresh_token", "id_token"] {
                assert!(!rendered.contains(image["tokens"][name].as_str().unwrap()));
            }
            let error = authentication(validate_codex_access_workspace(
                image["tokens"]["access_token"].as_str().unwrap(), WORKSPACE_B,
            ));
            assert_redacted(&error, &[
                image["tokens"]["access_token"].as_str().unwrap(),
                image["tokens"]["refresh_token"].as_str().unwrap(),
                image["tokens"]["id_token"].as_str().unwrap(),
                "redaction-marker", WORKSPACE_A, WORKSPACE_B,
            ]);
        });
    }
}
