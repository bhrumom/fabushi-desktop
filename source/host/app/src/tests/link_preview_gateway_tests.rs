use super::super::{HostLaneRequest, call_native_attachments_or_host_lane};
use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService, LINK_CACHE_DIRNAME, LINK_CACHE_TTL_MS,
    LINK_CACHE_VERSION, LinkMetadata,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use mahayana_host_runtime::sha256::sha256_hex;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// LPG-07 / LPG-LANE-01: exercise the production composition helper with the
// actual service. Only unrelated image-generation and Box dependencies are
// inert. These app-unit controls do not exercise the full desktop or HTTP app.
struct UnusedPreviewDependencies;

impl GenerateImageAuth for UnusedPreviewDependencies {
    fn get_access_token(&self) -> Result<String, String> {
        panic!("cached preview must not request image-generation credentials")
    }

    fn get_machine_id(&self) -> Result<String, String> {
        panic!("cached preview must not request an image-generation machine id")
    }
}

impl AttachmentsBox for UnusedPreviewDependencies {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        panic!("cached preview must not inspect a Box")
    }

    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        panic!("cached preview must not upload to a Box")
    }
}

struct CacheRecord {
    raw_url: &'static str,
    private: bool,
    metadata: LinkMetadata,
    path: PathBuf,
    bytes: Vec<u8>,
}

struct PreviewFixture {
    root: PathBuf,
    cache_dir: PathBuf,
    attachments: Arc<AttachmentsService>,
    records: Vec<CacheRecord>,
}

fn now_ms() -> u64 {
    u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH).expect("fixture clock").as_millis(),
    ).expect("fixture timestamp")
}

fn directory_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(directory).expect("fixture directory")
        .map(|entry| entry.expect("fixture directory entry").path()).collect::<Vec<_>>();
    paths.sort();
    paths
}

impl PreviewFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-app-preview-lane-{}", uuid::Uuid::new_v4(),
        ));
        // Own the directory before installing cleanup; a failed exclusive
        // create must never authorize removal of an existing directory.
        fs::create_dir(&root).expect("create owned app-preview fixture");
        let mut fixture = Self {
            cache_dir: root.join("desktop-user-data"),
            attachments: Arc::new(AttachmentsService::new(
                root.join("sand"),
                Arc::new(UnusedPreviewDependencies),
                Arc::new(UnusedPreviewDependencies),
                None,
            )),
            root,
            records: Vec::new(),
        };
        let records_dir = fixture.cache_dir.join(LINK_CACHE_DIRNAME);
        fs::create_dir_all(&records_dir).expect("create app-preview cache");
        let fetched_at = now_ms();
        // Independent canonical URL oracles: case/default-port/dot-segment
        // normalization and IPv4-mapped IPv6 serialization are explicit. In
        // particular, seeding private entries never calls preview admission.
        // The app already depends on the public Host SHA helper, not url/sha2.
        for (label, raw_url, canonical_url, hostname, private) in [
            (
                "public-host",
                " \thttps://PREVIEW.example.test:443/a/../public?fixture=app-host\n",
                "https://preview.example.test/public?fixture=app-host",
                "preview.example.test",
                false,
            ),
            (
                "public-mapped",
                "https://[::ffff:1.1.1.1]:443/a/../public?fixture=app-mapped",
                "https://[::ffff:101:101]/public?fixture=app-mapped",
                "[::ffff:101:101]",
                false,
            ),
            (
                "private-dotted",
                "https://[::ffff:127.0.0.1]:443/a/../private?fixture=app-dotted",
                "https://[::ffff:7f00:1]/private?fixture=app-dotted",
                "[::ffff:7f00:1]",
                true,
            ),
            (
                "private-hex",
                "https://[::ffff:7f00:1]/private?fixture=app-hex",
                "https://[::ffff:7f00:1]/private?fixture=app-hex",
                "[::ffff:7f00:1]",
                true,
            ),
        ] {
            let metadata = LinkMetadata {
                url: canonical_url.to_string(),
                canonical_url: format!("{canonical_url}#cached-canonical-{label}"),
                title: format!("App cached preview {label} ☃"),
                description: format!("Cached description for {label}"),
                site_name: format!("App preview site {label}"),
                hostname: hostname.to_string(),
                image_data_url: Some(concat!(
                    "data:image/png;base64,",
                    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8",
                    "AAwMCAO+aVBEAAAAASUVORK5CYII=",
                ).to_string()),
                favicon_data_url: None,
                fetched_at,
            };
            let mut cached = serde_json::to_value(&metadata).expect("typed cache metadata");
            cached.as_object_mut().expect("metadata object")
                .insert("cacheVersion".into(), json!(LINK_CACHE_VERSION));
            let bytes = serde_json::to_vec_pretty(&cached).expect("serialized cache record");
            let path = records_dir.join(format!("{}.json", sha256_hex(canonical_url.as_bytes())));
            fs::write(&path, &bytes).expect("seed fresh app-preview cache");
            fixture.records.push(CacheRecord { raw_url, private, metadata, path, bytes });
        }
        fixture.assert_unchanged_and_fresh();
        fixture
    }

    fn args(&self, record: &CacheRecord) -> Value {
        json!({"cacheDir": self.cache_dir.to_string_lossy(), "url": record.raw_url})
    }

    fn assert_public_preview(&self, host_tx: &mpsc::Sender<HostLaneRequest>, record: &CacheRecord) {
        assert!(!record.private, "positive control must use a public URL");
        let result = call_native_attachments_or_host_lane(
            host_tx, self.attachments.as_ref(), "fetchLinkMetadata", self.args(record),
        ).expect("real cached preview must succeed with a closed Host lane");
        assert_eq!(result, serde_json::to_value(&record.metadata).expect("expected metadata"));
        let metadata: LinkMetadata = serde_json::from_value(result).expect("typed preview reply");
        assert_eq!(metadata, record.metadata);
    }

    fn assert_unchanged_and_fresh(&self) {
        let records_dir = self.cache_dir.join(LINK_CACHE_DIRNAME);
        assert_eq!(directory_paths(&self.root), vec![self.cache_dir.clone()]);
        assert_eq!(directory_paths(&self.cache_dir), vec![records_dir.clone()]);
        let mut expected_paths = self.records.iter().map(|record| record.path.clone())
            .collect::<Vec<_>>();
        expected_paths.sort();
        assert_eq!(directory_paths(&records_dir), expected_paths, "cache record set changed");
        let now = now_ms();
        for record in &self.records {
            let bytes = fs::read(&record.path).expect("cache record remains readable");
            assert_eq!(bytes, record.bytes, "preview must preserve cache bytes");
            let cached: Value = serde_json::from_slice(&bytes).expect("cache JSON");
            assert_eq!(cached["cacheVersion"], json!(LINK_CACHE_VERSION));
            let metadata: LinkMetadata = serde_json::from_slice(&bytes).expect("typed cache record");
            assert_eq!(metadata, record.metadata);
            let age = now.checked_sub(metadata.fetched_at).expect("cache timestamp is not future");
            assert!(age < LINK_CACHE_TTL_MS, "cache fixture must remain fresh");
        }
    }
}

impl Drop for PreviewFixture {
    fn drop(&mut self) {
        let cleanup = fs::remove_dir_all(&self.root);
        if !thread::panicking() {
            cleanup.expect("remove owned app-preview fixture");
        }
    }
}

fn assert_native_preview_source_wiring() {
    // Source anchors complement the runtime helper oracle. They do not claim
    // construction/execution of the full UnifiedGatewayApi or desktop app.
    let source = include_str!("../main.rs")
        .split_once("\n#[cfg(test)]\nmod tests {").expect("production/test boundary").0;
    let fields = source.split_once("struct UnifiedGatewayApi {").expect("Gateway API fields").1
        .split_once("\n}").expect("Gateway API fields end").0;
    assert!(fields.contains("attachments: Arc<AttachmentsService>,"));
    let service = "let attachments_service = box_extensions.attachments.service();";
    assert_eq!(source.matches(service).count(), 1, "retain the existing service owner");
    let compose = source.split_once("let gateway_api = Arc::new(UnifiedGatewayApi {")
        .expect("production Gateway construction").1
        .split_once("\n    let channel_inbound_gateway =").expect("Gateway construction end").0;
    assert_eq!(compose.matches("attachments: Arc::clone(&attachments_service),").count(), 1);
    assert!(!compose.contains("AttachmentsService::new("));
    assert!(!compose.contains("AttachmentsService::production("));
    let api_call = source.split_once("impl GatewayApi for UnifiedGatewayApi {")
        .expect("production Gateway implementation").1
        .split_once("\n    fn health(").expect("Gateway call boundary").0;
    assert_eq!(api_call.matches(
        "call_native_attachments_or_host_lane(&self.host_tx, &self.attachments, method, args)",
    ).count(), 1, "the actual API fallback must use the tested helper and shared service");
    assert!(!api_call.contains("call_host_lane("), "ordinary fallback stays inside the helper");
}

#[test]
fn link_preview_gateway_cached_native_call_bypasses_closed_host_lane() {
    let fixture = PreviewFixture::new();
    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    drop(host_rx);
    for record in fixture.records.iter().filter(|record| !record.private) {
        fixture.assert_public_preview(&host_tx, record);
    }
    fixture.assert_unchanged_and_fresh();
    assert_native_preview_source_wiring();
}

#[test]
fn link_preview_gateway_native_errors_and_private_cache_preserve_contract() {
    let fixture = PreviewFixture::new();
    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    drop(host_rx);
    let public = fixture.records.iter().find(|record| !record.private).expect("public control");
    for record in fixture.records.iter().filter(|record| record.private) {
        let result = call_native_attachments_or_host_lane(
            &host_tx, fixture.attachments.as_ref(), "fetchLinkMetadata", fixture.args(record),
        ).expect("private-address refusal must not depend on the Host lane");
        assert_eq!(result, Value::Null, "fresh private cache must not bypass admission");
        fixture.assert_public_preview(&host_tx, public);
        fixture.assert_unchanged_and_fresh();
    }
    for (args, expected) in [
        (json!({}), "fetchLinkMetadata.cacheDir must be a string"),
        (
            json!({"cacheDir": false, "url": public.raw_url}),
            "fetchLinkMetadata.cacheDir must be a string",
        ),
        (
            json!({"cacheDir": fixture.cache_dir.to_string_lossy()}),
            "fetchLinkMetadata.url must be a string",
        ),
        (
            json!({"cacheDir": fixture.cache_dir.to_string_lossy(), "url": [public.raw_url]}),
            "fetchLinkMetadata.url must be a string",
        ),
    ] {
        let error = call_native_attachments_or_host_lane(
            &host_tx, fixture.attachments.as_ref(), "fetchLinkMetadata", args,
        ).expect_err("service argument error");
        assert_eq!(error.to_string(), expected, "retain the service's exact error string");
        assert!(matches!(&error, GatewayCommandError::Internal(message) if message == expected));
        fixture.assert_public_preview(&host_tx, public);
        fixture.assert_unchanged_and_fresh();
    }
}

fn call_with_host_lane_reply(
    fixture: &PreviewFixture,
    method: &str,
    args: Value,
    reply_value: Result<Value, GatewayCommandError>,
) -> (Result<Value, GatewayCommandError>, (String, Value)) {
    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    thread::scope(|scope| {
        let receiver = scope.spawn(move || -> Result<(String, Value), String> {
            let request = host_rx.recv_timeout(Duration::from_secs(5))
                .map_err(|error| format!("ordinary request did not reach Host lane: {error}"))?;
            let HostLaneRequest::Gateway { method, args, reply } = request else {
                return Err("ordinary Gateway call became a non-Gateway Host request".into());
            };
            reply.send(reply_value).map_err(|_| "Gateway caller dropped its reply".to_string())?;
            // The caller drops its sender after receiving this reply. This
            // also detects a duplicate request without a timing-based absence.
            match host_rx.recv_timeout(Duration::from_secs(5)) {
                Err(mpsc::RecvTimeoutError::Disconnected) => Ok((method, args)),
                Err(mpsc::RecvTimeoutError::Timeout) => Err("Host sender was not released".into()),
                Ok(_) => Err("one Gateway call enqueued more than one Host request".into()),
            }
        });
        let result = call_native_attachments_or_host_lane(
            &host_tx, fixture.attachments.as_ref(), method, args,
        );
        drop(host_tx);
        // Join the owned receiver before asserting any request/result oracle;
        // scoped ownership also joins it if the production helper panics.
        let observed = receiver.join().expect("Host receiver thread").expect("Host lane exchange");
        (result, observed)
    })
}

#[test]
fn link_preview_gateway_other_methods_preserve_host_lane_request_and_reply() {
    let fixture = PreviewFixture::new();
    let args = json!({"agentId": "lane-agent", "nested": [null, true, "unchanged ☃"]});
    let expected = json!({"from": "existing-host-lane", "result": [1, {"text": "reply ☃"}]});
    let (result, observed) = call_with_host_lane_reply(
        &fixture, "getAgentTranscript", args.clone(), Ok(expected.clone()),
    );
    assert_eq!(observed, ("getAgentTranscript".to_string(), args.clone()));
    assert_eq!(result.expect("ordinary Host lane success"), expected);

    let message = "ordinary Host lane denied this request\nretain exact detail ☃";
    let (result, observed) = call_with_host_lane_reply(
        &fixture, "getAgentMemories", args.clone(), Err(GatewayCommandError::Forbidden(message.into())),
    );
    assert_eq!(observed, ("getAgentMemories".to_string(), args));
    let error = result.expect_err("ordinary Host lane error");
    assert!(matches!(&error, GatewayCommandError::Forbidden(detail) if detail == message));
    assert_eq!(error.to_string(), format!("forbidden: {message}"));

    let neighbor = "fetchLinkMetadataUnexpected";
    let preview_args = fixture.args(&fixture.records[0]);
    let (result, observed) = call_with_host_lane_reply(
        &fixture, neighbor, preview_args.clone(), Err(GatewayCommandError::UnknownMethod(neighbor.into())),
    );
    assert_eq!(observed, (neighbor.to_string(), preview_args.clone()));
    let error = result.expect_err("unknown neighbor remains an ordinary Host lane request");
    assert!(matches!(&error, GatewayCommandError::UnknownMethod(method) if method == neighbor));
    assert_eq!(error.to_string(), format!("unknown gateway method: {neighbor}"));

    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    drop(host_rx);
    for method in ["getAgentTranscript", neighbor, "saveRemoteMedia"] {
        let error = call_native_attachments_or_host_lane(
            &host_tx, fixture.attachments.as_ref(), method, preview_args.clone(),
        ).expect_err("ordinary methods still require the Host lane");
        assert!(matches!(&error, GatewayCommandError::Internal(message)
            if message == "Mahayana Host lane is closed"));
        assert_eq!(error.to_string(), "Mahayana Host lane is closed");
    }
    fixture.assert_public_preview(&host_tx, &fixture.records[0]);
    fixture.assert_unchanged_and_fresh();
}
