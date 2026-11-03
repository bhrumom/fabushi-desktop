use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use mahayana_host_runtime::durable_file_policy::{
    HOST_CONFIG_REVISION_KEY_FILE_NAME, is_host_config_revision_key_file_name,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());

use mahayana_host_runtime::host_secret_store::{
    clear_host_machine_id_cache_for_tests, get_or_create_host_machine_id, read_machine_id,
    clear_host_node_token, load_or_create_host_node_identity, read_host_node_token,
    load_or_create_host_config_revision_key, write_host_node_token, write_machine_id,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("fabushi-{label}-{suffix}"))
}

#[test]
fn host_secret_store_reads_writes_and_process_caches_machine_id() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    clear_host_machine_id_cache_for_tests();
    let root = temp_root("host-secret-store");
    let path = root.join("nested").join("host-secrets.json");

    assert_eq!(read_machine_id(&path), None);
    write_machine_id(&path, "machine-existing").expect("write existing id");
    assert_eq!(read_machine_id(&path).as_deref(), Some("machine-existing"));

    let first = get_or_create_host_machine_id(Some(&path)).expect("read existing id");
    assert_eq!(first, "machine-existing");

    write_machine_id(&path, "machine-changed-on-disk").expect("replace id on disk");
    let cached = get_or_create_host_machine_id(Some(&path)).expect("cached id");
    assert_eq!(cached, "machine-existing");

    clear_host_machine_id_cache_for_tests();
    let reloaded = get_or_create_host_machine_id(Some(&path)).expect("reload id");
    assert_eq!(reloaded, "machine-changed-on-disk");

    fs::remove_dir_all(root).expect("remove test root");
    clear_host_machine_id_cache_for_tests();
}

#[test]
fn host_secret_store_creates_uuid_and_ignores_invalid_json() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    clear_host_machine_id_cache_for_tests();
    let root = temp_root("host-secret-create");
    fs::create_dir_all(&root).expect("create root");
    let path = root.join("host-secrets.json");
    fs::write(&path, "{not-json").expect("write invalid file");
    assert_eq!(read_machine_id(&path), None);

    let created = get_or_create_host_machine_id(Some(&path)).expect("create machine id");
    assert!(!created.is_empty());
    assert_eq!(read_machine_id(&path).as_deref(), Some(created.as_str()));

    fs::remove_dir_all(root).expect("remove test root");
    clear_host_machine_id_cache_for_tests();
}

#[test]
fn node_identity_is_persisted_and_signs_without_exposing_private_bytes() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    let root = temp_root("node-identity");
    let path = root.join("node-identity.json");

    let first = load_or_create_host_node_identity(Some(&path)).expect("create node identity");
    let second = load_or_create_host_node_identity(Some(&path)).expect("reload node identity");
    assert_eq!(first.device_id(), second.device_id());
    assert_eq!(
        first.public_key_raw_base64_url(),
        second.public_key_raw_base64_url()
    );
    let public_key_bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(first.public_key_raw_base64_url())
        .expect("decode public key")
        .try_into()
        .expect("public key length");
    let public_key = VerifyingKey::from_bytes(&public_key_bytes).expect("public key");
    let signature_bytes: [u8; 64] = URL_SAFE_NO_PAD
        .decode(first.sign_payload_base64_url("node-challenge"))
        .expect("decode signature")
        .try_into()
        .expect("signature length");
    public_key
        .verify(
            b"node-challenge",
            &Signature::from_bytes(&signature_bytes),
        )
        .expect("signature verifies");
    assert!(!fs::read_to_string(&path)
        .expect("read persisted identity")
        .contains("signingKey"));

    fs::remove_dir_all(root).expect("remove test root");
}

#[test]
fn node_tokens_are_origin_scoped_and_rotation_fenced() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    let root = temp_root("node-auth");
    let path = root.join("node-auth.json");
    let scopes = vec!["node.invoke".to_string(), "node.read".to_string()];

    assert!(write_host_node_token(
        Some(&path),
        "device-1",
        " node ",
        Some("wss://gateway.example:443"),
        "token-a",
        &scopes,
        None,
    )
    .expect("write token"));
    let stored = read_host_node_token(
        Some(&path),
        "device-1",
        "node",
        Some("wss://gateway.example:443"),
    )
    .expect("read token")
    .expect("stored token");
    assert_eq!(stored.secret(), "token-a");
    assert_eq!(
        stored.scopes().iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["node.invoke", "node.read"]
    );
    assert!(read_host_node_token(Some(&path), "device-1", "node", None)
        .expect("read unscoped token")
        .is_none());
    assert!(!write_host_node_token(
        Some(&path),
        "device-1",
        "node",
        Some("wss://gateway.example:443"),
        "token-b",
        &[],
        Some("stale-token"),
    )
    .expect("reject stale rotation"));
    assert!(write_host_node_token(
        Some(&path),
        "device-1",
        "node",
        Some("wss://gateway.example:443"),
        "token-b",
        &[],
        Some("token-a"),
    )
    .expect("rotate token"));
    assert!(!clear_host_node_token(
        Some(&path),
        "device-1",
        "node",
        Some("wss://gateway.example:443"),
        Some("token-a"),
    )
    .expect("reject stale clear"));
    assert!(clear_host_node_token(
        Some(&path),
        "device-1",
        "node",
        Some("wss://gateway.example:443"),
        Some("token-b"),
    )
    .expect("clear token"));

    fs::remove_dir_all(root).expect("remove test root");
}

struct RevisionKeyFixture(std::path::PathBuf);

impl RevisionKeyFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-config-revision-{}", uuid::Uuid::new_v4().simple()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).expect("create private fixture root");
        Self(root)
    }

    fn key_path(&self) -> std::path::PathBuf {
        self.0.join(HOST_CONFIG_REVISION_KEY_FILE_NAME)
    }

    fn assert_no_key_temporary_files(&self) {
        for entry in fs::read_dir(&self.0).expect("list owned fixture") {
            let entry = entry.expect("read fixture entry");
            let name = entry.file_name();
            let name = name.to_string_lossy();
            assert!(
                name == HOST_CONFIG_REVISION_KEY_FILE_NAME
                    || !is_host_config_revision_key_file_name(&name),
                "publication left a key temporary file"
            );
        }
    }
}

impl Drop for RevisionKeyFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn config_revision_key_reopens_and_separates_installations_and_domains() {
    let fixture = RevisionKeyFixture::new();
    let other_installation = RevisionKeyFixture::new();
    let settings_path = fixture.0.join("settings.json");
    let machine_id_path = fixture.0.join("host-secrets.json");
    let settings_before = br#"{"timezone":"UTC"}"#;
    let machine_before = br#"{"machineId":"synthetic-machine"}"#;
    fs::write(&settings_path, settings_before).expect("seed settings control");
    fs::write(&machine_id_path, machine_before).expect("seed machine-id control");

    let path = fixture.key_path();
    let first = load_or_create_host_config_revision_key(Some(&path)).expect("create revision key");
    let key_before = fs::read(&path).expect("read owned key fixture");
    assert_eq!(key_before.len(), 32);
    let reopened = load_or_create_host_config_revision_key(Some(&path)).expect("reopen revision key");
    let separate = load_or_create_host_config_revision_key(Some(&other_installation.key_path()))
        .expect("create separate installation key");
    let normalized = first.project_normalized_hash("same-internal-hash");
    assert_eq!(normalized, reopened.project_normalized_hash("same-internal-hash"));
    assert_ne!(normalized, separate.project_normalized_hash("same-internal-hash"));
    assert_ne!(normalized, first.project_raw_hash("same-internal-hash"));
    assert_ne!(normalized, first.project_normalized_hash("changed-internal-hash"));
    assert!(fs::read(&path).expect("reread owned key") == key_before, "reopen changed key material");
    assert!(fs::read(&settings_path).expect("read settings control") == settings_before);
    assert!(fs::read(&machine_id_path).expect("read machine-id control") == machine_before);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&path).expect("key metadata").permissions().mode() & 0o777, 0o600);
    }
    fixture.assert_no_key_temporary_files();
    other_installation.assert_no_key_temporary_files();
}

#[test]
fn config_revision_key_wire_mac_covers_exact_json_domains() {
    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    let synthetic_key = [0x37_u8; 32];
    fs::write(&path, synthetic_key).expect("seed synthetic key");
    let key = load_or_create_host_config_revision_key(Some(&path)).expect("load synthetic key");
    let cases: [(String, &[u8]); 2] = [
        (
            key.project_normalized_hash("fixture-internal-hash"),
            br#"["fabushi.gateway.config-revision.normalized.v1","fixture-internal-hash"]"#,
        ),
        (
            key.project_raw_hash("fixture-internal-hash"),
            br#"["fabushi.gateway.config-revision.raw.v1","fixture-internal-hash"]"#,
        ),
    ];
    for (token, literal_json) in cases {
        let encoded = token.strip_prefix("hmac-sha256:v1:").expect("versioned HMAC token");
        assert_eq!(encoded.len(), 43);
        assert!(!encoded.contains('='));
        let signature = URL_SAFE_NO_PAD.decode(encoded).expect("base64url token");
        assert_eq!(signature.len(), 32);
        let mut verifier = <Hmac<Sha256> as Mac>::new_from_slice(&synthetic_key)
            .expect("synthetic HMAC key");
        verifier.update(literal_json);
        verifier.verify_slice(&signature).expect("token authenticates exact frozen JSON input");
    }
}

#[test]
fn config_revision_key_rejects_corruption_without_cache_or_replacement() {
    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    let _first = load_or_create_host_config_revision_key(Some(&path)).expect("create initial key");
    for size in [0, 31, 33, 4096] {
        let corrupt = vec![0x52_u8; size];
        fs::write(&path, &corrupt).expect("write corrupt owned key");
        let error = load_or_create_host_config_revision_key(Some(&path))
            .err().expect("corruption must fail even after an earlier successful load");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(fs::read(&path).expect("read corrupt fixture") == corrupt, "corrupt key was replaced");
        fixture.assert_no_key_temporary_files();
    }
}

#[test]
fn config_revision_key_rejects_nonregular_entries_and_creation_io_failure() {
    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    fs::create_dir(&path).expect("create nonregular key entry");
    let marker = path.join("owned-marker");
    fs::write(&marker, b"preserve").expect("write nonregular control");
    let error = load_or_create_host_config_revision_key(Some(&path))
        .err().expect("directory cannot be key material");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(fs::read(&marker).expect("read preserved marker") == b"preserve");

    let blocked_parent = fixture.0.join("parent-is-a-file");
    fs::write(&blocked_parent, b"keep-parent").expect("seed IO refusal");
    assert!(load_or_create_host_config_revision_key(
        Some(&blocked_parent.join(HOST_CONFIG_REVISION_KEY_FILE_NAME))
    ).is_err(), "non-directory parent must fail instead of substituting another key");
    assert!(fs::read(&blocked_parent).expect("read preserved parent") == b"keep-parent");
    fixture.assert_no_key_temporary_files();
}

#[cfg(unix)]
#[test]
fn config_revision_key_permission_errors_never_fall_back_or_create_material() {
    use std::os::unix::fs::PermissionsExt;

    // SAFETY: geteuid has no preconditions and does not change process identity.
    assert_ne!(unsafe { libc::geteuid() }, 0, "permission oracle requires an unprivileged Actions user");
    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    let synthetic_key = [0x58_u8; 32];
    fs::write(&path, synthetic_key).expect("seed owned permission fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("make key unreadable");
    let error = load_or_create_host_config_revision_key(Some(&path))
        .err().expect("unreadable key must fail");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore owned fixture permission");
    assert!(fs::read(&path).expect("read retained material") == synthetic_key, "unreadable key was replaced");

    let blocked_directory = fixture.0.join("unwritable");
    fs::create_dir(&blocked_directory).expect("create publication refusal fixture");
    fs::set_permissions(&blocked_directory, fs::Permissions::from_mode(0o500))
        .expect("refuse new publication files");
    let error = load_or_create_host_config_revision_key(
        Some(&blocked_directory.join(HOST_CONFIG_REVISION_KEY_FILE_NAME))
    ).err().expect("unwritable directory must fail");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(fs::read_dir(&blocked_directory).expect("inspect blocked directory").count(), 0);
    fs::set_permissions(&blocked_directory, fs::Permissions::from_mode(0o700))
        .expect("restore owned directory permission");
    fixture.assert_no_key_temporary_files();
}

#[cfg(unix)]
#[test]
fn config_revision_key_rejects_symlinks_including_dangling_links() {
    use std::os::unix::fs::symlink;

    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    let target = fixture.0.join("synthetic-target");
    let synthetic_key = [0x61_u8; 32];
    fs::write(&target, synthetic_key).expect("write owned symlink target");
    symlink(&target, &path).expect("link owned key entry");
    assert!(load_or_create_host_config_revision_key(Some(&path)).is_err());
    assert!(fs::symlink_metadata(&path).expect("link metadata").file_type().is_symlink());
    assert!(fs::read(&target).expect("preserved target") == synthetic_key);
    fs::remove_file(&target).expect("make link dangling");
    assert!(load_or_create_host_config_revision_key(Some(&path)).is_err());
    assert!(!target.exists(), "dangling link target must not be created");
    assert!(fs::symlink_metadata(&path).expect("dangling link metadata").file_type().is_symlink());
    fixture.assert_no_key_temporary_files();
}

#[test]
fn config_revision_key_concurrent_first_use_converges_and_cleans_candidates() {
    let fixture = RevisionKeyFixture::new();
    let path = fixture.key_path();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let workers = (0..8).map(|_| {
        let barrier = std::sync::Arc::clone(&barrier);
        let path = path.clone();
        std::thread::spawn(move || {
            barrier.wait();
            load_or_create_host_config_revision_key(Some(&path))
                .expect("load concurrent revision key")
                .project_normalized_hash("concurrent-revision")
        })
    }).collect::<Vec<_>>();
    let tokens = workers.into_iter().map(|worker| worker.join().expect("revision worker"))
        .collect::<Vec<_>>();
    let reopened = load_or_create_host_config_revision_key(Some(&path)).expect("reopen race winner");
    assert!(tokens.iter().all(|token| token == &reopened.project_normalized_hash("concurrent-revision")));
    assert_eq!(fs::metadata(&path).expect("winner metadata").len(), 32);
    fixture.assert_no_key_temporary_files();
}
