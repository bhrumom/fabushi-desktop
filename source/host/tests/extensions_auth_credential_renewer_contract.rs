use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use mahayana_host_runtime::extensions::auth::credential_renewer::{
    CREDENTIAL_RETRY_BASE_DELAY_MS, CREDENTIAL_RETRY_MAX_DELAY_MS, DEFAULT_TTL_MS,
    InferenceCredential, MAX_REFRESH_INTERVAL_MS, MIN_REFRESH_INTERVAL_MS,
    REFRESH_LEEWAY_MS, RenewalOutcome, SandInferenceCredentialRenewer,
    CredentialRenewalBackend, CredentialRenewerHooks, SandCredentialRenewalError,
    get_access_token_expiry_ms, read_dev_inference_credential_file,
    redact_renewal_error_for_report, refresh_delay_ms, renew_sand_box_inference_credential,
    retry_delay_ms,
};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[test]
fn credential_payload_uses_exp_claim_then_default_ttl() {
    let payload = URL_SAFE_NO_PAD.encode(br#"{"exp":2000000000}"#);
    let token = format!("header.{payload}.signature");
    assert_eq!(get_access_token_expiry_ms(&token), Some(2_000_000_000_000));

    let path = PathBuf::from(format!(
        "{}/fabushi-auth-token-{}-{}.json",
        std::env::temp_dir().display(),
        std::process::id(),
        now_ms(),
    ));
    std::fs::write(&path, format!(r#"{{"accessToken":"{token}"}}"#)).unwrap();
    let parsed = read_dev_inference_credential_file(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(parsed.access_token, token);
    assert_eq!(parsed.expires_at_ms, 2_000_000_000_000);

    assert_eq!(DEFAULT_TTL_MS, 10 * 60 * 1_000);
}

#[test]
fn refresh_and_retry_schedules_match_grok_bounds() {
    let now = 1_000_000_u64;
    assert_eq!(
        refresh_delay_ms(now + REFRESH_LEEWAY_MS + 1, now),
        MIN_REFRESH_INTERVAL_MS,
    );
    assert_eq!(
        refresh_delay_ms(now + REFRESH_LEEWAY_MS + MAX_REFRESH_INTERVAL_MS * 2, now),
        MAX_REFRESH_INTERVAL_MS,
    );
    assert_eq!(retry_delay_ms(1), CREDENTIAL_RETRY_BASE_DELAY_MS);
    assert_eq!(retry_delay_ms(2), CREDENTIAL_RETRY_BASE_DELAY_MS * 2);
    assert_eq!(retry_delay_ms(100), CREDENTIAL_RETRY_MAX_DELAY_MS);
}

#[test]
fn renewal_error_report_redacts_urls_paths_and_long_ids() {
    let redacted = redact_renewal_error_for_report(
        "POST https://secret.example/api failed /Users/name/token ABCDEFGHIJKLMNOPQRSTUVWXYZ012345",
    );
    assert_eq!(redacted, "POST <url> failed <path> <id>");
}

#[test]
fn production_renewer_posts_backend_metadata_and_reads_credential() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut received = Vec::new();
        let mut buffer = [0_u8; 2048];
        let header_end;
        loop {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            received.extend_from_slice(&buffer[..count]);
            if let Some(index) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = index;
                break;
            }
        }
        let headers = std::str::from_utf8(&received[..header_end]).unwrap();
        let length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        while received.len() < header_end + 4 + length {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            received.extend_from_slice(&buffer[..count]);
        }
        let body = br#"{"accessToken":"short-lived","expiresAtMs":424242}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(body).unwrap();
        stream.flush().unwrap();
        received
    });

    let credential = renew_sand_box_inference_credential(
        &format!("http://127.0.0.1:{port}/nested/base"),
        "renewal-secret",
    )
    .unwrap();
    assert_eq!(
        credential,
        InferenceCredential {
            access_token: "short-lived".into(),
            expires_at_ms: 424242,
        }
    );

    let request = server.join().unwrap();
    let text = String::from_utf8_lossy(&request);
    assert!(text.starts_with("POST /sand-box/inference-credential HTTP/1.1\r\n"));
    assert!(text.to_ascii_lowercase().contains("x-cursor-client-type: sand\r\n"));
    assert!(text.to_ascii_lowercase().contains("x-cursor-client-version:"));
    assert!(text.to_ascii_lowercase().contains("x-sand-box-namespace:"));
    assert!(text.contains(r#"{"credential":"renewal-secret"}"#));
}

struct SequenceBackend {
    calls: Mutex<u32>,
}

impl CredentialRenewalBackend for SequenceBackend {
    fn renew(&self, _credential: &str) -> Result<InferenceCredential, SandCredentialRenewalError> {
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        if *calls == 1 {
            return Err(SandCredentialRenewalError::Transport(
                "https://secret.example/ ABCDEFGHIJKLMNOPQRSTUVWXYZ".into(),
            ));
        }
        Ok(InferenceCredential {
            access_token: format!("token-{calls}"),
            expires_at_ms: now_ms() + 600_000,
        })
    }
}

#[test]
fn background_renewer_reports_failure_then_supports_immediate_retry() {
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc;
    use std::time::Instant;

    const CASE: &str = "background_renewer_reports_failure_then_supports_immediate_retry";
    const CHILD_NONCE_ENV: &str = "FABUSHI_AUTH_RETRY_CONTRACT_CHILD_NONCE";
    const RECEIPT_PREFIX: &str = "FABUSHI_AUTH_RETRY_CONTRACT_COMPLETED:";
    const WORK_LIMIT: Duration = Duration::from_millis(CREDENTIAL_RETRY_BASE_DELAY_MS / 2);
    const CHILD_LIMIT: Duration = Duration::from_secs(20);

    if std::env::var_os(CHILD_NONCE_ENV).is_none() {
        struct OwnedRetryChild(Child);

        impl Drop for OwnedRetryChild {
            fn drop(&mut self) {
                if self.0.try_wait().ok().flatten().is_none() {
                    let _ = self.0.kill();
                }
                let _ = self.0.wait();
            }
        }

        let nonce = uuid::Uuid::new_v4().to_string();
        let mut child = OwnedRetryChild(
            Command::new(std::env::current_exe().expect("current Auth contract executable"))
                .args(["--exact", CASE, "--nocapture", "--test-threads=1"])
                .env(CHILD_NONCE_ENV, &nonce)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("start the exact owned Auth retry case"),
        );
        let started = Instant::now();
        let observed = loop {
            match child.0.try_wait() {
                Ok(Some(status)) if started.elapsed() < CHILD_LIMIT => break Ok(status),
                Ok(None) if started.elapsed() < CHILD_LIMIT => {
                    // Backoff for process supervision only; exit plus the
                    // per-run receipt below is the execution oracle.
                    thread::sleep(Duration::from_millis(5));
                }
                Ok(_) => break Err("owned Auth retry case exceeded its deadline".to_string()),
                Err(error) => break Err(format!("wait for owned Auth retry case: {error}")),
            }
        };
        let status = match observed {
            Ok(status) => status,
            Err(failure) => {
                // Preserve the failed observation before stopping only this
                // child. Cleanup can never turn a timeout into success.
                let killed = match child.0.try_wait() {
                    Ok(Some(_)) => Ok(()),
                    _ => child.0.kill(),
                };
                let reaped = child.0.wait();
                if reaped.is_ok() {
                    let mut output = String::new();
                    let _ = child.0.stdout.take().unwrap().read_to_string(&mut output);
                    eprintln!("owned Auth retry child output:\n{output}");
                }
                panic!("{failure}; exact-child kill={killed:?}, wait={reaped:?}");
            }
        };
        let mut output = String::new();
        child.0.stdout.take().unwrap().read_to_string(&mut output).unwrap();
        assert!(status.success(), "owned Auth retry case failed: {status}\n{output}");
        let receipt = format!("{RECEIPT_PREFIX}{nonce}");
        assert!(
            output.lines().any(|line| line == receipt.as_str()),
            "the exact child must run all retry assertions, not zero filtered tests:\n{output}",
        );
        return;
    }

    // The parent supervises this same single case; there is no extra helper
    // test whose default execution could be mistaken for owner coverage.
    let child_nonce = std::env::var(CHILD_NONCE_ENV).expect("owned Auth retry child nonce");
    let backend = Arc::new(SequenceBackend {
        calls: Mutex::new(0),
    });
    let source = Arc::new(|| Some("renewal-secret".to_string()));
    let stored = Arc::new(Mutex::new(Vec::<InferenceCredential>::new()));
    let stored_sink = Arc::clone(&stored);
    let reports = Arc::new(Mutex::new(Vec::new()));
    let reports_sink = Arc::clone(&reports);
    let (report_published, report_events) = mpsc::channel();

    let mut renewer = SandInferenceCredentialRenewer::new(
        backend,
        CredentialRenewerHooks {
            get_credential: source,
            set_credential: Arc::new(move |credential| {
                stored_sink.lock().unwrap().push(credential);
            }),
            on_result: Some(Arc::new(move |result| {
                reports_sink.lock().unwrap().push(result);
                let _ = report_published.send(());
            })),
            now_ms: Arc::new(now_ms),
        },
    );
    // Share one pre-start budget, strictly shorter than the first automatic
    // retry delay. A late report cannot open a fresh request window after a
    // timer-driven renewal has already produced token-2.
    let work_deadline = Instant::now() + WORK_LIMIT;
    renewer.start();

    let report_remaining = work_deadline.saturating_duration_since(Instant::now());
    let first_report = if report_remaining.is_zero() {
        Err(mpsc::RecvTimeoutError::Timeout)
    } else {
        report_events.recv_timeout(report_remaining)
    };
    let first_report_on_time = first_report.is_ok() && Instant::now() < work_deadline;
    if !first_report_on_time {
        eprintln!("first Auth retry report missing or late before owner cleanup: {first_report:?}");
        renewer.close();
    }
    assert!(first_report_on_time, "the first report must arrive within the shared work deadline");
    assert_eq!(first_report, Ok(()), "the first failure report must arrive");
    assert_eq!(reports.lock().unwrap()[0].outcome, RenewalOutcome::Failed);

    // The report is not idle admission. A false public request has not been
    // admitted; only a true return proves the real owner admitted and completed
    // this retry. The child supervisor also bounds an admitted call that hangs.
    let retry_completed = loop {
        if Instant::now() >= work_deadline {
            break false;
        }
        if renewer.request_immediate_renewal() {
            break Instant::now() < work_deadline;
        }
        thread::yield_now();
    };
    if !retry_completed {
        eprintln!("public Auth retry did not complete within its deadline before owner cleanup");
    }
    renewer.close();
    assert!(retry_completed, "the real public retry must complete before the deadline");

    let reports = reports.lock().unwrap();
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[1].outcome, RenewalOutcome::Renewed);
    assert_eq!(stored.lock().unwrap()[0].access_token, "token-2");
    println!("\n{RECEIPT_PREFIX}{child_nonce}");
}

#[test]
fn product_service_origin_has_no_cursor_default_or_foreign_override() {
    use mahayana_host_runtime::extensions::auth::credential_renewer::configured_product_backend_url;
    assert_eq!(configured_product_backend_url(None, None).unwrap(), "https://api.ombhrum.com/");
    assert_eq!(configured_product_backend_url(Some("http://127.0.0.1:4200"), Some("https://api2.cursor.sh")).unwrap(), "http://127.0.0.1:4200/");
    for origin in ["https://api2.cursor.sh", "https://cursor.com", "https://api3.cursor.sh", "https://example.com", "http://api.ombhrum.com", "https://token@api.ombhrum.com", "https://api.ombhrum.com/?secret=x"] {
        assert!(configured_product_backend_url(Some(origin), None).is_err(), "origin must be refused");
        assert!(configured_product_backend_url(None, Some(origin)).is_err(), "legacy override must be refused");
    }
    assert!(configured_product_backend_url(Some("https://fabushi-ci-isolated-auth.bhrumom.workers.dev"), None).is_ok());
}
