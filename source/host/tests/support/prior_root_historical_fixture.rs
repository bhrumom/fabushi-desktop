//! PRID-v1 test-only carrier for actual closed historical ORT states.
//!
//! This file is also overlaid, unchanged, on the pinned BASE integration
//! crate. Keep its dependencies on the existing parent harness: no checkpoint
//! builder call, replacement producer, runtime schema, or deserialized proof.
//! The Actions capture owner verifies the complete source/build overlay and
//! binds the inventory/archive outside the payload they identify.

use super::*;

const BASE_COMMIT: &str = "e19a0a265e400a4c9b1d96520c84fb612a3843e8";
const BASE_TREE: &str = "5be2d7ca128dae5fefc8f7551e7088934daf19ac";
const BASE_CHECKPOINT_SHA256: &str = "09e38fa7e471037e7a46ccf679815b4cc969e32ff2c9a23146ff5d9a340b376c";
const CONTRACT: &str = "OCRM-CANONICAL-PRIOR-ROOT-IDENTITY-v1";
const EXPORT_ENV: &str = "FABUSHI_PRIOR_ROOT_FIXTURE_EXPORT_DIR";
const IMPORT_ENV: &str = "FABUSHI_PRIOR_ROOT_FIXTURE_IMPORT_DIR";
const CONSUMER_ENV: &str = "FABUSHI_PRIOR_ROOT_FIXTURE_CONSUMER_DIR";
const PROVENANCE_ENV: &str = "FABUSHI_PRIOR_ROOT_FIXTURE_PROVENANCE_FILE";
const INVENTORY_SHA256_ENV: &str = "FABUSHI_PRIOR_ROOT_FIXTURE_INVENTORY_SHA256";
const INVOCATIONS: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureProvenance {
    format: String,
    phase: String,
    base_commit: String,
    base_tree: String,
    head_commit: String,
    head_tree: String,
    support_module_sha256: String,
    parent_harness_sha256: String,
    host_binary_sha256: String,
    app_binary_sha256: Option<String>,
    capture_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutingCapture {
    supplied: CaptureProvenance,
    host_test_executable: PathBuf,
    checkpoint_source_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum HistoricalCaseKind {
    PresentEmpty,
    LegacyNonemptyNoAlias,
}

impl HistoricalCaseKind {
    fn name(self) -> &'static str {
        match self {
            Self::PresentEmpty => "present-empty",
            Self::LegacyNonemptyNoAlias => "legacy-nonempty-no-alias",
        }
    }

    fn initial(self) -> InitialState {
        match self {
            Self::PresentEmpty => InitialState::EmptyCheckpoint,
            Self::LegacyNonemptyNoAlias => InitialState::UserCheckpoint,
        }
    }
}

#[derive(Clone, Copy)]
enum WriterImage { Historical, Current }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorImage {
    root_blob_id: Vec<u8>,
    checkpoint_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorObservation {
    phase: String,
    actual_prior: Option<PriorImage>,
    content_sha256_alias: Vec<u8>,
    content_sha256_alias_present: bool,
    current_root_blob_id: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProcessEvidence {
    label: String,
    run_directory: String,
    token: String,
    pid: u32,
    parent_pid: u32,
    expectation: ReopenExpectation,
    exit_code: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpEvidence {
    // original_wire_request captures only JSON bodies, never HTTP headers.
    request_bodies: Vec<Value>,
    stages: Vec<(usize, String)>,
    requests: usize,
    invocations: usize,
    actual_effects: usize,
    model: String,
    worker_joined: bool,
    worker_outcome: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistentImage {
    kv: Vec<(String, Vec<u8>)>,
    transcript: Vec<(i64, String, Vec<u8>)>,
    blobs: Vec<(String, String, Vec<u8>)>,
    databases: BTreeMap<String, DatabaseImage>,
    other_files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum SqlCell {
    Null,
    Integer(i64),
    RealBits(u64),
    Text(Vec<u8>),
    Blob(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DatabaseImage {
    schema: Vec<(String, String, String, Option<String>)>,
    tables: BTreeMap<String, Vec<Vec<SqlCell>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDigest {
    path: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TreeInventory {
    format: String,
    directories: Vec<String>,
    files: Vec<FileDigest>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseManifest {
    kind: HistoricalCaseKind,
    producer_root: PathBuf,
    producer_root_retained_for_job: bool,
    agent_id: String,
    initial_expected: ExpectedCheckpoint,
    prior: Option<PriorImage>,
    expected: ExpectedCheckpoint,
    original: OriginalTurnReference,
    original_marker: UpgradeResumeMarker,
    invocations: usize,
    marker_path: String,
    processes: Vec<ProcessEvidence>,
    prior_observations: Vec<PriorObservation>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportManifest {
    format: String,
    contract: String,
    capture: ExecutingCapture,
    cases: Vec<CaseManifest>,
}

struct ProducedCase {
    fixture: Fixture,
    initial_expected: ExpectedCheckpoint,
    prior: Option<PriorImage>,
    processes: Vec<ProcessEvidence>,
    prior_observations: Vec<PriorObservation>,
    http: HttpEvidence,
    credential_value: Vec<u8>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_sha256(path: &Path) -> String {
    let mut file = File::open(path).expect("open exact evidence file for hashing");
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let length = file.read(&mut buffer).expect("hash exact evidence file");
        if length == 0 { break; }
        digest.update(&buffer[..length]);
    }
    format!("{:x}", digest.finalize())
}

fn assert_hex(value: &str, length: usize, label: &str) {
    assert!(value.len() == length && value.bytes().all(|byte|
        byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)), "invalid {label}");
}

fn absolute_env_path(name: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(name)
        .unwrap_or_else(|| panic!("{name} is required; this utility never silently skips")));
    assert!(path.is_absolute(), "{name} must be an absolute path");
    path
}

fn required_directory(name: &str, empty: bool) -> PathBuf {
    let path = absolute_env_path(name);
    assert!(path.is_dir(), "{name} must name an existing owned directory");
    let path = path.canonicalize().expect("canonical owned capture directory");
    if empty {
        assert!(fs::read_dir(&path).expect("read owned output directory").next().is_none(),
            "{name} must be empty; existing evidence is never overwritten");
    }
    path
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_slice(&fs::read(path).expect("read required test evidence"))
        .expect("decode strict test-only evidence envelope")
}

fn write_json(path: &Path, value: &impl Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize test-only evidence");
    bytes.push(b'\n');
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .expect("create fresh evidence without replacing an earlier result");
    file.write_all(&bytes).and_then(|()| file.sync_all())
        .expect("persist complete test-only evidence");
}

fn validate_capture_shape(provenance: &CaptureProvenance, phase: &str) {
    assert_eq!(provenance.format, "prid-v1-capture");
    assert_eq!(provenance.phase, phase);
    assert_eq!(provenance.base_commit, BASE_COMMIT);
    assert_eq!(provenance.base_tree, BASE_TREE);
    assert_hex(&provenance.head_commit, 40, "carrier H commit");
    assert_hex(&provenance.head_tree, 40, "carrier H tree");
    assert_ne!(provenance.head_commit, BASE_COMMIT);
    for digest in [&provenance.support_module_sha256, &provenance.parent_harness_sha256,
        &provenance.host_binary_sha256]
    {
        assert_hex(digest, 64, "bound source/binary SHA-256");
    }
    assert!(!provenance.capture_id.is_empty() && provenance.capture_id.len() <= 512
        && !provenance.capture_id.chars().any(char::is_control), "bounded Actions capture identity");
    if phase == "base-export" {
        assert!(provenance.app_binary_sha256.is_none(), "BASE exports do not claim an App build");
    } else {
        assert_hex(provenance.app_binary_sha256.as_deref().expect("bound H App digest"), 64,
            "H App SHA-256");
    }
}

fn executing_capture(phase: &str) -> ExecutingCapture {
    let path = absolute_env_path(PROVENANCE_ENV);
    let supplied: CaptureProvenance = read_json(&path);
    validate_capture_shape(&supplied, phase);
    assert_eq!(supplied.support_module_sha256, sha256(include_bytes!("prior_root_historical_fixture.rs")),
        "capture must identify the exact compiled support overlay");
    assert_eq!(supplied.parent_harness_sha256, sha256(include_bytes!("../production_checkpoint_restart_contract.rs")),
        "capture must identify the exact compiled parent harness");
    let host_test_executable = std::env::current_exe().expect("exact currently executing Host test artifact");
    assert!(host_test_executable.is_absolute() && host_test_executable.is_file());
    assert_eq!(supplied.host_binary_sha256, file_sha256(&host_test_executable),
        "the utility must execute the exact artifact bound by Actions");
    let checkpoint_source_sha256 = sha256(include_bytes!("../../src/runner/production_agent_checkpoint.rs"));
    if phase == "base-export" {
        assert_eq!(checkpoint_source_sha256, BASE_CHECKPOINT_SHA256,
            "historical export requires the compiled BASE writer, never future bytes rewritten as old data");
    } else {
        assert_ne!(checkpoint_source_sha256, BASE_CHECKPOINT_SHA256,
            "the new-H consumer cannot be the historical production source");
        let app_digest = std::env::var(APP_CHILD_SHA256_ENV)
            .expect("existing App compiler-artifact digest binding");
        assert_eq!(supplied.app_binary_sha256.as_deref(), Some(app_digest.as_str()));
        let app = absolute_env_path(APP_CHILD_EXECUTABLE_ENV);
        assert!(app.is_file(), "existing App compiler-artifact binding must exist");
    }
    ExecutingCapture { supplied, host_test_executable, checkpoint_source_sha256 }
}

fn relative_path(value: &str) -> PathBuf {
    assert!(!value.is_empty() && !value.contains('\\') && !value.contains(':')
        && !value.chars().any(char::is_control), "portable relative evidence path");
    assert!(value.split('/').all(|part| !part.is_empty() && part != "." && part != ".."));
    let path = PathBuf::from(value);
    assert!(!path.is_absolute() && path.components().all(|part|
        matches!(part, std::path::Component::Normal(_))), "evidence path must remain in its owned root");
    path
}

fn relative_name(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).expect("evidence belongs to this exact owned root");
    let value = relative.components().map(|part| match part {
        std::path::Component::Normal(value) => value.to_str().expect("UTF-8 fixture path"),
        _ => panic!("non-normal fixture path"),
    }).collect::<Vec<_>>().join("/");
    relative_path(&value);
    value
}

fn inventory(root: &Path, omit_root_inventory: bool) -> TreeInventory {
    fn visit(root: &Path, directory: &Path, omit: bool, output: &mut TreeInventory) {
        for entry in fs::read_dir(directory).expect("enumerate complete closed fixture") {
            let path = entry.expect("fixture directory entry").path();
            let metadata = fs::symlink_metadata(&path).expect("inspect fixture entry without following links");
            assert!(!metadata.file_type().is_symlink(), "fixture carriers do not follow symbolic links");
            let name = relative_name(root, &path);
            if metadata.is_dir() {
                output.directories.push(name);
                visit(root, &path, omit, output);
            } else {
                assert!(metadata.is_file(), "fixture carrier contains only ordinary files and directories");
                if omit && name == "inventory.json" { continue; }
                output.files.push(FileDigest { path: name, size: metadata.len(), sha256: file_sha256(&path) });
            }
        }
    }
    let mut output = TreeInventory { format: "prid-v1-tree-inventory".into(), directories: Vec::new(), files: Vec::new() };
    visit(root, root, omit_root_inventory, &mut output);
    output.directories.sort();
    output.files.sort_by(|left, right| left.path.cmp(&right.path));
    output
}

fn copy_closed_tree(source: &Path, destination: &Path) {
    let before = inventory(source, false);
    assert!(destination.is_absolute() && !destination.exists());
    fs::create_dir(destination).expect("create uniquely owned immutable-state copy");
    for directory in &before.directories {
        fs::create_dir_all(destination.join(relative_path(directory))).expect("copy closed-state directories");
    }
    for file in &before.files {
        let relative = relative_path(&file.path);
        fs::copy(source.join(&relative), destination.join(&relative)).expect("copy original bytes without rewriting persisted data");
    }
    assert_eq!(inventory(source, false), before, "the closed source must not change while copied");
    assert_eq!(inventory(destination, false), before, "the copy must retain every exact original file byte");
}

fn verify_inventory(root: &Path, expected_digest: &str) -> TreeInventory {
    assert_hex(expected_digest, 64, "externally bound inventory SHA-256");
    assert_eq!(file_sha256(&root.join("inventory.json")), expected_digest,
        "inventory must match the external Actions binding");
    let expected: TreeInventory = read_json(&root.join("inventory.json"));
    for directory in &expected.directories { relative_path(directory); }
    for file in &expected.files {
        relative_path(&file.path);
        assert_hex(&file.sha256, 64, "payload file SHA-256");
    }
    assert_eq!(inventory(root, true), expected,
        "raw payload verification includes sorted membership, sizes, hashes, and unexpected files");
    expected
}

fn process_directories(processes: &[ProcessEvidence]) -> Vec<String> {
    let mut names = Vec::new();
    for process in processes {
        let path = relative_path(&process.run_directory);
        assert_eq!(path.components().count(), 1);
        assert!(process.run_directory.starts_with("process-"));
        assert!(!names.contains(&process.run_directory), "every launch has a distinct owned directory");
        names.push(process.run_directory.clone());
    }
    names
}

fn complete_database_image(path: &Path) -> DatabaseImage {
    let db = open_existing(path, true);
    let schema: Vec<(String, String, String, Option<String>)> = db.prepare(
        "SELECT name,type,tbl_name,sql FROM sqlite_master ORDER BY name,type,tbl_name",
    ).expect("read complete owner schema without physical page numbers")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .expect("read owner schema rows").collect::<Result<_, _>>().expect("complete owner schema");
    let mut tables = BTreeMap::new();
    for (name, kind, _, _) in &schema {
        if kind != "table" { continue; }
        let mut statement = db.prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
            .expect("read every owner table including independent custody and sequence state");
        let columns = statement.column_count();
        let mut rows = statement.query([]).expect("read all actual table rows");
        let mut image = Vec::new();
        while let Some(row) = rows.next().expect("read complete actual row") {
            let mut values = Vec::new();
            for column in 0..columns {
                values.push(match row.get_ref(column).expect("actual unnormalized SQLite value") {
                    rusqlite::types::ValueRef::Null => SqlCell::Null,
                    rusqlite::types::ValueRef::Integer(value) => SqlCell::Integer(value),
                    rusqlite::types::ValueRef::Real(value) => SqlCell::RealBits(value.to_bits()),
                    rusqlite::types::ValueRef::Text(value) => SqlCell::Text(value.to_vec()),
                    rusqlite::types::ValueRef::Blob(value) => SqlCell::Blob(value.to_vec()),
                });
            }
            image.push(values);
        }
        image.sort();
        tables.insert(name.clone(), image);
    }
    DatabaseImage { schema, tables }
}

fn persistent_image(fixture: &Fixture, processes: &[ProcessEvidence]) -> PersistentImage {
    let state = durable_state(fixture);
    let process_directories = process_directories(processes);
    let databases = [relative_name(&fixture.root, &fixture.db_path()), relative_name(&fixture.root, &fixture.blob_path())];
    let mut other_files = BTreeMap::new();
    for file in inventory(&fixture.root, false).files {
        let top = file.path.split('/').next().unwrap();
        if process_directories.iter().any(|name| name == top) { continue; }
        // Logical raw row values above cover the two SQLite owners. Their
        // page/WAL/SHM/journal layouts may change when a fresh owner closes.
        if databases.iter().any(|database| file.path == *database
            || ["-wal", "-shm", "-journal"].iter().any(|suffix| file.path == format!("{database}{suffix}")))
        { continue; }
        other_files.insert(file.path.clone(), fs::read(fixture.root.join(relative_path(&file.path)))
            .expect("read every other persistent file exactly"));
    }
    let database_images = [fixture.db_path(), fixture.blob_path()].iter().map(|path|
        (relative_name(&fixture.root, path), complete_database_image(path))).collect();
    PersistentImage { kv: state.kv, transcript: state.transcript, blobs: state.blobs,
        databases: database_images, other_files }
}

fn finish_child(fixture: &Fixture, label: &str, child: ChildRun) -> (ProcessEvidence, ChildReceipt) {
    let exit_code = child.result.as_ref().ok().and_then(ExitStatus::code);
    let evidence = ProcessEvidence {
        label: label.into(), run_directory: relative_name(&fixture.root, &child.run_dir),
        token: child.token.clone(), pid: child.child_pid, parent_pid: std::process::id(),
        expectation: child.expectation, exit_code: exit_code.unwrap_or(-1),
    };
    let receipt = child.assert_completed();
    assert_eq!(evidence.exit_code, 0, "retain the actual successful command exit");
    (evidence, receipt)
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value.as_bytes().chunks_exact(2).map(|pair| u8::from_str_radix(
        std::str::from_utf8(pair).expect("stored metadata hex"), 16,
    ).expect("stored metadata byte")).collect()
}

fn stored_metadata(fixture: &Fixture) -> Value {
    serde_json::from_slice(&decode_hex(&read_stored_kv(fixture, "metadata")))
        .expect("actual stored metadata image")
}

fn stored_blob(fixture: &Fixture, id: &[u8]) -> Vec<u8> {
    open_existing(&fixture.blob_path(), true).query_row("SELECT data FROM blobs WHERE id=?1",
        params![hex(id)], |row| row.get(0)).expect("actual recorded blob remains available")
}

fn observe_prior(fixture: &Fixture, kind: HistoricalCaseKind, prior: &Option<PriorImage>, phase: &str) -> PriorObservation {
    let prior_image = prior.as_ref().expect("both historical controls have an actual present prior");
    assert!(!prior_image.root_blob_id.is_empty());
    assert_eq!(stored_blob(fixture, &prior_image.root_blob_id), prior_image.checkpoint_bytes,
        "read the actual retained prior identity, never a guessed alias");
    let alias = Sha256::digest(&prior_image.checkpoint_bytes).to_vec();
    let aliases: i64 = open_existing(&fixture.blob_path(), true).query_row(
        "SELECT COUNT(*) FROM blobs WHERE id=?1", params![hex(&alias)], |row| row.get(0),
    ).expect("observe actual content-hash alias availability");
    match kind {
        HistoricalCaseKind::PresentEmpty => {
            assert!(prior_image.checkpoint_bytes.is_empty());
            assert_eq!(prior_image.root_blob_id, alias);
            assert_eq!(aliases, 1);
        }
        HistoricalCaseKind::LegacyNonemptyNoAlias => {
            assert!(!prior_image.checkpoint_bytes.is_empty());
            assert_eq!(prior_image.root_blob_id, LEGACY_FIXED_ROOT_ID);
            assert_ne!(prior_image.root_blob_id, alias);
            assert_eq!(aliases, 0, "the SHA alias must remain absent before production and after every reader");
        }
    }
    PriorObservation {
        phase: phase.into(), actual_prior: prior.clone(), content_sha256_alias: alias,
        content_sha256_alias_present: aliases != 0,
        current_root_blob_id: decode_hex(stored_metadata(fixture)["latestRootBlobId"].as_str().expect("actual current root ID")),
    }
}

fn remove_legacy_alias_before_production(fixture: &Fixture) {
    let bytes = fixture.expected.state_bytes.as_ref().expect("actual legacy root bytes");
    let alias = Sha256::digest(bytes).to_vec();
    assert_ne!(fixture.expected.root_blob_id, alias);
    let db = open_existing(&fixture.blob_path(), false);
    let actual: Vec<u8> = db.query_row("SELECT data FROM blobs WHERE id=?1", params![hex(&alias)], |row| row.get(0))
        .expect("initial SHA alias must exist before its explicit fixture removal");
    assert_eq!(&actual, bytes);
    assert_eq!(db.execute("DELETE FROM blobs WHERE id=?1", params![hex(&alias)])
        .expect("remove only the legacy prior's SHA alias while owners are closed"), 1);
    drop(db);
    assert_eq!(stored_blob(fixture, LEGACY_FIXED_ROOT_ID), *bytes);
}

fn user_fields(bytes: &[u8]) -> Vec<(u64, Vec<u8>)> {
    let mut remaining = bytes;
    let mut fields = Vec::new();
    while !remaining.is_empty() {
        let tag = prost::encoding::decode_varint(&mut remaining).expect("actual generated user field tag");
        assert_eq!(tag & 7, 2, "these real user controls contain only length-delimited fields");
        let length = usize::try_from(prost::encoding::decode_varint(&mut remaining)
            .expect("actual generated user field length")).expect("bounded user field length");
        assert!(length <= remaining.len());
        let (value, rest) = remaining.split_at(length);
        fields.push((tag >> 3, value.to_vec()));
        remaining = rest;
    }
    fields
}

fn closed_facts(fixture: &Fixture, prior: &Option<PriorImage>, writer: WriterImage) -> Value {
    let original = fixture.original.as_ref().expect("genuine producer reference");
    original.validate().expect("actual accepted and bound original reference");
    let record = original_terminal(fixture);
    assert_eq!(record["version"], 1);
    assert_eq!(record["original"], serde_json::to_value(original).unwrap());
    assert_eq!(record["original"]["modelId"], ORIGINAL_MODEL);
    assert!(record["compaction"].is_null());
    assert_eq!(record["settlementLifecycleRevision"], record["original"]["originalLifecycleRevision"]);
    let admission: Value = serde_json::from_str(&read_stored_kv(fixture, "sessionWorkAdmission")).unwrap();
    assert_eq!(record["settlementLifecycleRevision"], admission["lifecycle_revision"]);
    assert_ne!(stored_metadata(fixture)["lifecycleRevision"], record["settlementLifecycleRevision"]);
    let canonical = record["canonical"].as_array().expect("actual ordered canonical receipt");
    assert_eq!(canonical.len(), INVOCATIONS);
    let expected_prior = prior.as_ref().map(|image| image.root_blob_id.clone());
    assert_eq!(canonical[0]["priorRootBlobId"], serde_json::to_value(&expected_prior).unwrap());
    assert_eq!(record["finalResultSha256"], serde_json::to_value(original_turn_result_digest("completed invocation 2")).unwrap());
    let custody: ToolEffectCustodyReport = serde_json::from_str(record["custodyJson"].as_str().unwrap())
        .expect("observe the genuinely issued custody data; never construct proof from it");
    assert_eq!(custody.invocations().len(), INVOCATIONS);
    assert!(!custody.requires_retention());
    for (ordinal, observation) in custody.invocations().iter().enumerate() {
        let ToolEffectCustodyObservation::Known { run_generation, snapshot } = observation else {
            panic!("the genuine generated producer must retain Known custody");
        };
        assert!(*run_generation > 0 && snapshot.unsettled_reason().is_none());
        assert_eq!(snapshot.started_revision, u64::from(ordinal == 0));
        assert_eq!(snapshot.checkpoint_revision, u64::from(ordinal == 0));
    }
    let mut users = Vec::new();
    for (ordinal, commit) in canonical.iter().enumerate() {
        let actual_prior: Option<Vec<u8>> = serde_json::from_value(commit["priorRootBlobId"].clone()).unwrap();
        let prior_bytes = actual_prior.as_deref().map(|id| stored_blob(fixture, id)).unwrap_or_default();
        let user_id: Vec<u8> = serde_json::from_value(commit["userBlobId"].clone()).unwrap();
        let user_bytes = stored_blob(fixture, &user_id);
        assert_eq!(user_id, Sha256::digest(&user_bytes).to_vec());
        let user = decode_user_message_identity_fields(&user_bytes).expect("actual canonical user");
        assert_eq!(user.message_id, if ordinal == 0 { original.user_message_id.as_str() } else { ORIGINAL_STREAM });
        if ordinal == 0 { assert_eq!(user.text, USER_TEXT); }
        let embedded = match writer {
            WriterImage::Historical => (!prior_bytes.is_empty()).then(|| Sha256::digest(&prior_bytes).to_vec()),
            WriterImage::Current => actual_prior.clone(),
        };
        let mut expected_fields = vec![(1, user.text.as_bytes().to_vec()), (2, user.message_id.as_bytes().to_vec())];
        if let Some(id) = &embedded { expected_fields.push((10, id.clone())); }
        assert_eq!(user_fields(&user_bytes), expected_fields,
            "actual writer image, including omission versus present-empty and exact legacy identity");
        users.push(json!({"ordinal":ordinal, "user_blob_id":user_id,
            "actual_prior_root_blob_id":actual_prior, "embedded_prior_root_blob_id":embedded}));
    }
    let raw_user: String = open_existing(&fixture.db_path(), true).query_row(
        "SELECT entry FROM transcript_entries WHERE id=?1", params![original.user_message_id], |row| row.get(0),
    ).expect("actual accepted user row");
    let user: Value = serde_json::from_str(&raw_user).unwrap();
    assert_eq!(user["confirmed"], true);
    assert_eq!(user["content"], USER_TEXT);
    assert_eq!(fs::read(fixture.root.join("original-effect.txt")).expect("actual single effect receipt"), b"original-terminal-effect");
    assert!(!fixture.root.join("original-turn-credential.json").exists());
    let providers = mirror_files(&fixture.root.join("runner-provider-checkpoints"));
    assert_eq!(providers.len(), 1, "one actual checkpoint for the one returned tool effect");
    let (provider_path, provider_bytes) = providers.iter().next().unwrap();
    let RoutedProviderCheckpoint::Fabushi(provider) = serde_json::from_slice(provider_bytes)
        .expect("actual persisted provider checkpoint") else { panic!("actual Native provider identity"); };
    assert_eq!(provider.model.as_deref(), Some(ORIGINAL_MODEL));
    assert_eq!(provider.tool_calls_completed, 1);
    let journal_path = fixture.root.join("transcripts").join(&fixture.agent_id).join(format!("{}.jsonl", fixture.agent_id));
    let journal = fs::read_to_string(&journal_path).expect("actual generated Journal");
    let entries = journal.lines().map(|line| serde_json::from_str::<Value>(line).expect("actual Journal occurrence"))
        .collect::<Vec<_>>();
    let prior_turns = decode_conversation_state_recovery_fields(&prior.as_ref().unwrap().checkpoint_bytes)
        .expect("actual retained prior wire image").turns.len();
    assert_eq!(entries.len(), prior_turns * 2 + 6, "prior text history plus actual user/tool/result/two invocation history");
    assert_eq!(entries.last().unwrap(), &json!({"role":"assistant", "message":{"content":[{
        "type":"text", "text":"completed invocation 2",
    }]}}));
    json!({"terminal":record, "confirmed_user":user, "canonical_user_images":users,
        "provider_checkpoint":{"path":provider_path, "sha256":sha256(provider_bytes),
            "model":provider.model, "tool_calls_completed":provider.tool_calls_completed},
        "journal":{"path":relative_name(&fixture.root, &journal_path), "sha256":sha256(journal.as_bytes()),
            "occurrences":entries.len()}, "actual_effects":1})
}

fn produce_case(kind: HistoricalCaseKind, writer: WriterImage) -> ProducedCase {
    let mut fixture = Fixture::new(&format!("prid-{}", kind.name()), kind.initial());
    if kind == HistoricalCaseKind::LegacyNonemptyNoAlias {
        fixture.rebind_to_legacy_fixed_root();
        remove_legacy_alias_before_production(&fixture);
    }
    let initial_expected = fixture.expected.clone();
    let prior = fixture.expected.state_bytes.as_ref().map(|bytes| PriorImage {
        root_blob_id: fixture.expected.root_blob_id.clone(), checkpoint_bytes: bytes.clone(),
    });
    let before = observe_prior(&fixture, kind, &prior, "before-genuine-producer");
    assert_eq!(before.current_root_blob_id, prior.as_ref().unwrap().root_blob_id);
    fixture.original_invocations = INVOCATIONS;
    fixture.original_compaction = false;
    let mut server = OriginalTurnWireServer::new(&fixture.root, INVOCATIONS, false);
    let credential: Value = read_json(&server.credential);
    let credential_value = credential["accessToken"].as_str().expect("controlled fixture credential").as_bytes().to_vec();
    assert!(!credential_value.is_empty());
    let child = fixture.run_child("prid-produce-original", ReopenExpectation::ProduceOriginal, Some(&server));
    let server_result = server.finish();
    eprintln!("PRID genuine producer pid={} server={server_result:?} {}", child.child_pid, server.diagnostics());
    assert!(server_result.is_ok() && server.worker.is_none(), "the actual provider worker must finish and join");
    let (process, receipt) = finish_child(&fixture, "genuine-producer", child);
    let request_bodies = server.requests.lock().expect("actual bounded HTTP capture").clone();
    assert_eq!(request_bodies.len(), INVOCATIONS + 1);
    assert!(request_bodies.iter().all(|request| request["model"] == ORIGINAL_MODEL));
    let stages = server.stages.lock().expect("actual provider stages").iter()
        .map(|(ordinal, stage)| (*ordinal, (*stage).to_string())).collect();
    fs::remove_file(&server.credential).expect("remove actual credential before any recovery or export");
    drop(server);
    let produced = receipt.produced.expect("actual post-shutdown producer completion receipt");
    fixture.original = Some(produced.original);
    fixture.expected = produced.checkpoint;
    let marker_store = SandUpgradeResumeStore::new(&fixture.root);
    let markers = marker_store.list_pending_checked().expect("actual producer marker");
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].agent_id, fixture.agent_id);
    assert_eq!(markers[0].source.as_deref(), Some("turn"));
    assert_eq!(markers[0].original_work, fixture.original);
    let marker_image: Value = read_json(marker_store.file_path());
    assert_eq!(marker_image["version"], 2);
    fixture.original_marker = Some(markers[0].clone());
    closed_facts(&fixture, &prior, writer);
    let after = observe_prior(&fixture, kind, &prior, "after-genuine-producer");
    assert_eq!(after.current_root_blob_id, fixture.expected.root_blob_id);
    ProducedCase {
        fixture, initial_expected, prior, processes: vec![process], prior_observations: vec![before, after],
        http: HttpEvidence { requests: request_bodies.len(), request_bodies, stages,
            invocations: INVOCATIONS, actual_effects: 1, model: ORIGINAL_MODEL.into(),
            worker_joined: true, worker_outcome: "ok".into() },
        credential_value,
    }
}

fn fresh_readers(case: &mut ProducedCase, kind: HistoricalCaseKind, prefix: &str) {
    let before = persistent_image(&case.fixture, &case.processes);
    for attempt in 1..=2 {
        let label = format!("{prefix}-reader-{attempt}");
        let child = case.fixture.reopen(&label, ReopenExpectation::OriginalReady);
        let (process, receipt) = finish_child(&case.fixture, &label, child);
        assert!(receipt.produced.is_none() && receipt.canonical_turns.is_none());
        case.processes.push(process);
        assert_eq!(persistent_image(&case.fixture, &case.processes), before,
            "fresh readers cannot rewrite owner rows, Journal, provider, acceptance, effect or marker bytes");
        case.prior_observations.push(observe_prior(&case.fixture, kind, &case.prior, &label));
    }
}

fn assert_no_credential(root: &Path, credential_value: &[u8]) {
    assert!(!credential_value.is_empty());
    for file in inventory(root, false).files {
        let bytes = fs::read(root.join(relative_path(&file.path))).expect("inspect exact outgoing fixture bytes");
        assert!(!bytes.windows(credential_value.len()).any(|window| window == credential_value),
            "credential material must not enter exported file {}", file.path);
    }
}

fn validate_historical_processes(root: &Path, case: &CaseManifest) {
    assert_eq!(case.processes.len(), 3, "one actual producer and two actual old-reader processes");
    process_directories(&case.processes);
    let mut tokens = Vec::new();
    for (ordinal, process) in case.processes.iter().enumerate() {
        assert_eq!(process.exit_code, 0);
        assert_ne!(process.pid, 0);
        assert_ne!(process.parent_pid, 0);
        assert_ne!(process.pid, process.parent_pid);
        assert!(!process.token.is_empty() && !tokens.contains(&process.token));
        tokens.push(process.token.clone());
        let directory = root.join(relative_path(&process.run_directory));
        let request: ChildRequest = read_json(&directory.join("request.json"));
        let receipt: ChildReceipt = read_json(&directory.join("completed.json"));
        assert_eq!(request.token, process.token);
        assert_eq!(receipt.token, process.token);
        assert_eq!(receipt.pid, process.pid);
        assert_eq!(request.parent_pid, process.parent_pid);
        assert_eq!(receipt.parent_pid, process.parent_pid);
        assert_eq!(request.fixture_root, case.producer_root, "historical requests remain byte-exact, never relocated in place");
        assert_eq!(request.agent_id, case.agent_id);
        assert_eq!(request.original_invocations, INVOCATIONS);
        assert!(!request.original_compaction && request.generated_scenario.is_none() && request.canonical_scenario.is_none());
        let expected = if ordinal == 0 { ReopenExpectation::ProduceOriginal } else { ReopenExpectation::OriginalReady };
        assert_eq!(process.expectation, expected);
        assert_eq!(request.expectation, expected);
        assert_eq!(receipt.expectation, expected);
        if ordinal == 0 {
            assert_eq!(request.expected, case.initial_expected);
            assert!(request.original.is_none() && request.original_marker.is_none());
            let produced = receipt.produced.expect("original genuine producer result");
            assert_eq!(produced.original, case.original);
            assert_eq!(produced.checkpoint, case.expected);
        } else {
            assert_eq!(request.expected, case.expected);
            assert_eq!(request.original.as_ref(), Some(&case.original));
            assert_eq!(request.original_marker.as_ref(), Some(&case.original_marker));
            assert!(receipt.produced.is_none());
        }
        assert!(receipt.canonical_turns.is_none());
        for name in ["stdout.log", "stderr.log"] { assert!(directory.join(name).is_file()); }
    }
}

#[test]
#[ignore = "explicit Actions BASE utility; requires absolute output and bound provenance"]
fn export_base_writer_fixtures() {
    let output = required_directory(EXPORT_ENV, true);
    let capture = executing_capture("base-export");
    fs::create_dir(output.join("cases")).expect("create test-only historical cases directory");
    let mut cases = Vec::new();
    for kind in [HistoricalCaseKind::PresentEmpty, HistoricalCaseKind::LegacyNonemptyNoAlias] {
        let mut produced = produce_case(kind, WriterImage::Historical);
        fresh_readers(&mut produced, kind, "base");
        let directory = output.join("cases").join(kind.name());
        fs::create_dir(&directory).expect("new historical case carrier");
        let evidence = directory.join("evidence");
        fs::create_dir(&evidence).expect("new historical case evidence");
        let marker_path = relative_name(&produced.fixture.root,
            SandUpgradeResumeStore::new(&produced.fixture.root).file_path());
        write_json(&evidence.join("http.json"), &produced.http);
        write_json(&evidence.join("closed-facts.json"), &closed_facts(&produced.fixture, &produced.prior, WriterImage::Historical));
        write_json(&evidence.join("persistent-image.json"), &persistent_image(&produced.fixture, &produced.processes));
        assert_no_credential(&produced.fixture.root, &produced.credential_value);
        copy_closed_tree(&produced.fixture.root, &directory.join("state"));
        let case = CaseManifest {
            kind, producer_root: produced.fixture.root.clone(), producer_root_retained_for_job: true,
            agent_id: produced.fixture.agent_id.clone(), initial_expected: produced.initial_expected,
            prior: produced.prior, expected: produced.fixture.expected.clone(),
            original: produced.fixture.original.clone().unwrap(),
            original_marker: produced.fixture.original_marker.clone().unwrap(), invocations: INVOCATIONS,
            marker_path, processes: produced.processes, prior_observations: produced.prior_observations,
        };
        validate_historical_processes(&directory.join("state"), &case);
        assert_no_credential(&directory, &produced.credential_value);
        eprintln!("PRID exported {} from closed BASE location {}; retained until Actions job cleanup for relocation diagnosis",
            kind.name(), case.producer_root.display());
        cases.push(case);
        // Fixture contains paths/expectations only; every child and server was
        // reaped/joined and all Session owners closed above. Transfer this exact
        // directory's cleanup to the Actions job. The independent state copy
        // already outlives Fixture::Drop, even if relocation later fails.
        std::mem::forget(produced.fixture);
    }
    write_json(&output.join("manifest.json"), &ExportManifest {
        format: "prid-v1-historical-fixtures".into(), contract: CONTRACT.into(), capture, cases,
    });
    write_json(&output.join("inventory.json"), &inventory(&output, true));
    eprintln!("PRID export inventory_sha256={}", file_sha256(&output.join("inventory.json")));
}

#[test]
#[ignore = "explicit Actions H utility; requires immutable input, separate output, provenance and bound shipping App"]
fn consume_base_writer_fixtures() {
    let input = required_directory(IMPORT_ENV, false);
    let output = required_directory(CONSUMER_ENV, true);
    assert!(!input.starts_with(&output) && !output.starts_with(&input), "immutable carrier and consumer evidence must be disjoint");
    let digest = std::env::var(INVENTORY_SHA256_ENV).expect("external inventory digest is required before any fixture is opened");
    let input_inventory = verify_inventory(&input, &digest);
    let manifest: ExportManifest = read_json(&input.join("manifest.json"));
    assert_eq!(manifest.format, "prid-v1-historical-fixtures");
    assert_eq!(manifest.contract, CONTRACT);
    validate_capture_shape(&manifest.capture.supplied, "base-export");
    assert_eq!(manifest.capture.checkpoint_source_sha256, BASE_CHECKPOINT_SHA256);
    let capture = executing_capture("head-consume");
    assert_eq!(capture.supplied.head_commit, manifest.capture.supplied.head_commit);
    assert_eq!(capture.supplied.head_tree, manifest.capture.supplied.head_tree);
    assert_eq!(capture.supplied.support_module_sha256, manifest.capture.supplied.support_module_sha256,
        "BASE and H must compile the same support module");
    assert_eq!(manifest.cases.len(), 2);
    fs::create_dir(output.join("cases")).expect("create owned consumer evidence root");
    let mut results = Vec::new();
    for (case, kind) in manifest.cases.iter().zip([HistoricalCaseKind::PresentEmpty, HistoricalCaseKind::LegacyNonemptyNoAlias]) {
        assert_eq!(case.kind, kind);
        assert_eq!(case.invocations, INVOCATIONS);
        assert!(case.producer_root.is_absolute() && case.producer_root_retained_for_job);
        assert_eq!(relative_path(&case.agent_id).components().count(), 1, "one actual agent directory");
        relative_path(&case.marker_path);
        assert_eq!(case.prior_observations.len(), 4);
        for (ordinal, observation) in case.prior_observations.iter().enumerate() {
            assert_eq!(observation.actual_prior, case.prior);
            let prior = case.prior.as_ref().expect("recorded present prior Option");
            assert_eq!(observation.content_sha256_alias, Sha256::digest(&prior.checkpoint_bytes).to_vec());
            assert_eq!(observation.content_sha256_alias_present, kind == HistoricalCaseKind::PresentEmpty);
            let expected_current = if ordinal == 0 {
                &prior.root_blob_id
            } else { &case.expected.root_blob_id };
            assert_eq!(&observation.current_root_blob_id, expected_current);
        }
        let source = input.join("cases").join(kind.name());
        let directory = output.join("cases").join(kind.name());
        fs::create_dir(&directory).expect("new owned consumer case");
        let root = directory.join("working-state");
        assert_ne!(root, case.producer_root);
        copy_closed_tree(&source.join("state"), &root);
        validate_historical_processes(&root, case);
        let fixture = Fixture {
            root, agent_id: case.agent_id.clone(), expected: case.expected.clone(),
            original: Some(case.original.clone()), original_invocations: INVOCATIONS,
            original_marker: Some(case.original_marker.clone()), original_compaction: false,
            generated_scenario: None, canonical_scenario: None, canonical_turns: None,
        };
        eprintln!("PRID relocating {} to {}; original BASE root {} is retained in the producing job. Any relocation failure remains non-passing; do not rewrite persisted paths.",
            kind.name(), fixture.root.display(), case.producer_root.display());
        let mut processes = case.processes.clone();
        let before: PersistentImage = read_json(&source.join("evidence/persistent-image.json"));
        assert_eq!(persistent_image(&fixture, &processes), before, "immutable copy has the actual complete persistent image");
        assert_eq!(closed_facts(&fixture, &case.prior, WriterImage::Historical),
            read_json::<Value>(&source.join("evidence/closed-facts.json")));
        let http: HttpEvidence = read_json(&source.join("evidence/http.json"));
        assert_eq!(http.requests, 3);
        assert_eq!(http.request_bodies.len(), 3);
        assert_eq!(http.invocations, INVOCATIONS);
        assert_eq!(http.actual_effects, 1);
        assert_eq!(http.model, ORIGINAL_MODEL);
        assert!(http.worker_joined && http.worker_outcome == "ok");
        assert!(http.request_bodies.iter().all(|body| body["model"] == ORIGINAL_MODEL));
        let mut observations = vec![observe_prior(&fixture, kind, &case.prior, "copied-before-H-readers")];
        let mut new_processes = Vec::new();
        for (label, expectation) in [
            ("head-reader-1", ReopenExpectation::OriginalReady),
            ("head-reader-2", ReopenExpectation::OriginalReady),
            ("head-app-ack", ReopenExpectation::OriginalMarkerAck),
            ("head-app-noop", ReopenExpectation::OriginalMarkerNoop),
        ] {
            let child = fixture.reopen(label, expectation);
            let (process, receipt) = finish_child(&fixture, label, child);
            assert!(receipt.produced.is_none() && receipt.canonical_turns.is_none());
            processes.push(process.clone());
            new_processes.push(process);
            let after = persistent_image(&fixture, &processes);
            if matches!(expectation, ReopenExpectation::OriginalMarkerAck | ReopenExpectation::OriginalMarkerNoop) {
                let mut expected_files = before.other_files.clone();
                assert!(expected_files.remove(&case.marker_path).is_some(), "actual whole producer marker was present");
                assert_eq!(after.kv, before.kv);
                assert_eq!(after.transcript, before.transcript);
                assert_eq!(after.blobs, before.blobs);
                assert_eq!(after.databases, before.databases);
                assert_eq!(after.other_files, expected_files,
                    "shipping App may remove only the exact marker; every other persistent byte is unchanged");
                assert!(!fixture.root.join(relative_path(&case.marker_path)).exists());
            } else {
                assert_eq!(after, before, "fresh H reconciliation is strictly read-only");
            }
            observations.push(observe_prior(&fixture, kind, &case.prior, label));
        }
        write_json(&directory.join("persistent-image-after.json"), &persistent_image(&fixture, &processes));
        copy_closed_tree(&fixture.root, &directory.join("state-after"));
        results.push(json!({"kind":kind, "producer_root":case.producer_root,
            "consumer_root":fixture.root, "relocated_without_persisted_rewrite":true,
            "new_processes":new_processes, "prior_observations":observations,
            "only_persistent_change":"exact original marker removed", "additional_tool_effects":0}));
        drop(fixture);
    }
    assert_eq!(verify_inventory(&input, &digest), input_inventory,
        "consumption never changes the immutable historical carrier");
    write_json(&output.join("manifest.json"), &json!({
        "format":"prid-v1-historical-consumption", "contract":CONTRACT,
        "capture":capture, "input_inventory_sha256":digest, "cases":results,
    }));
    write_json(&output.join("inventory.json"), &inventory(&output, true));
}

#[test]
fn current_writer_present_empty_root_has_exact_identity_and_fresh_completion() {
    let mut case = produce_case(HistoricalCaseKind::PresentEmpty, WriterImage::Current);
    fresh_readers(&mut case, HistoricalCaseKind::PresentEmpty, "current-present-empty");
}

#[test]
fn current_writer_legacy_without_alias_has_exact_identity_and_fresh_completion() {
    let mut case = produce_case(HistoricalCaseKind::LegacyNonemptyNoAlias, WriterImage::Current);
    fresh_readers(&mut case, HistoricalCaseKind::LegacyNonemptyNoAlias, "current-legacy");
}

#[derive(Clone, Copy, Debug)]
enum UserImageFault { WrongExistingId, DuplicatePrior, UnsupportedField, MissingPrior }

impl UserImageFault {
    fn diagnostic(self) -> &'static str {
        match self {
            Self::WrongExistingId | Self::MissingPrior => "canonical user differs from the checkpoint writer",
            Self::DuplicatePrior | Self::UnsupportedField => "generated envelope has the wrong field",
        }
    }
}

fn append_wire_field(field: u64, bytes: &[u8], output: &mut Vec<u8>) {
    prost::encoding::encode_varint((field << 3) | 2, output);
    prost::encoding::encode_varint(bytes.len() as u64, output);
    output.extend_from_slice(bytes);
}

fn adversarial_turn_bytes(user: &[u8], steps: &[Vec<u8>], request: &str) -> Vec<u8> {
    let mut agent = Vec::new();
    append_wire_field(1, user, &mut agent);
    for step in steps { append_wire_field(2, step, &mut agent); }
    append_wire_field(3, request.as_bytes(), &mut agent);
    let mut turn = Vec::new();
    append_wire_field(1, &agent, &mut turn);
    turn
}

fn mutate_user_image_copy(seed: &ProducedCase, fault: UserImageFault) -> Fixture {
    // This is explicitly an adversarial copy of genuinely issued state. It is
    // never exported as historical writer output or used as a positive proof.
    // Rehash the dependent second invocation as well, retaining all original
    // timing bytes, steps/results, accepted user, custody and lifecycle fences.
    let root = std::env::temp_dir().join(format!("fabushi-checkpoint-restart-prid-negative-{}-{}",
        std::process::id(), uuid::Uuid::new_v4()));
    let root = root.parent().unwrap().canonicalize().unwrap().join(root.file_name().unwrap());
    copy_closed_tree(&seed.fixture.root, &root);
    let mut fixture = Fixture {
        root, agent_id: seed.fixture.agent_id.clone(), expected: seed.fixture.expected.clone(),
        original: seed.fixture.original.clone(), original_invocations: INVOCATIONS,
        original_marker: seed.fixture.original_marker.clone(), original_compaction: false,
        generated_scenario: None, canonical_scenario: None, canonical_turns: None,
    };
    let original_record = original_terminal(&fixture);
    let mut record = original_record.clone();
    let wrong_existing_id: Vec<u8> = serde_json::from_value(
        original_record["canonical"][INVOCATIONS - 1]["rootBlobId"].clone()).unwrap();
    assert!(!stored_blob(&fixture, &wrong_existing_id).is_empty(), "wrong semantic reference still resolves");
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut mutations = Vec::new();
    for ordinal in 0..INVOCATIONS {
        let old = &original_record["canonical"][ordinal];
        let old_prior: Vec<u8> = serde_json::from_value::<Option<Vec<u8>>>(old["priorRootBlobId"].clone()).unwrap().unwrap();
        let old_prior_bytes = stored_blob(&fixture, &old_prior);
        let (new_prior, new_prior_bytes) = previous.clone().unwrap_or_else(|| (old_prior.clone(), old_prior_bytes.clone()));
        let old_user: Vec<u8> = serde_json::from_value(old["userBlobId"].clone()).unwrap();
        let old_turn: Vec<u8> = serde_json::from_value(old["turnBlobId"].clone()).unwrap();
        let old_root: Vec<u8> = serde_json::from_value(old["rootBlobId"].clone()).unwrap();
        let steps: Vec<Vec<u8>> = serde_json::from_value(old["stepBlobIds"].clone()).unwrap();
        let request = old["requestId"].as_str().unwrap();
        let mut fields = user_fields(&stored_blob(&fixture, &old_user));
        assert_eq!(fields.len(), 3, "this generated seed has text, identity and exact prior only");
        assert_eq!(fields[2], (10, old_prior.clone()));
        if ordinal == 0 {
            assert_eq!(old_prior, LEGACY_FIXED_ROOT_ID);
            assert_ne!(wrong_existing_id, old_prior);
            assert_ne!(wrong_existing_id, Sha256::digest(&old_prior_bytes).to_vec());
            match fault {
                UserImageFault::WrongExistingId => fields[2].1 = wrong_existing_id.clone(),
                UserImageFault::DuplicatePrior => fields.push(fields[2].clone()),
                UserImageFault::UnsupportedField => fields.insert(2, (11, b"unsupported-user-field".to_vec())),
                UserImageFault::MissingPrior => { fields.pop(); }
            }
        } else {
            fields[2].1 = new_prior.clone();
        }
        let mut user_bytes = Vec::new();
        for (field, bytes) in &fields { append_wire_field(*field, bytes, &mut user_bytes); }
        let new_user = Sha256::digest(&user_bytes).to_vec();
        assert_ne!(new_user, old_user);
        assert_eq!(stored_blob(&fixture, &old_turn), adversarial_turn_bytes(&old_user, &steps, request),
            "the seed's complete turn image is accounted for before reference replacement");
        let turn_bytes = adversarial_turn_bytes(&new_user, &steps, request);
        let new_turn = Sha256::digest(&turn_bytes).to_vec();
        let old_root_bytes = stored_blob(&fixture, &old_root);
        let mut old_append = Vec::new();
        append_wire_field(8, &old_turn, &mut old_append);
        let timing_suffix = old_root_bytes.strip_prefix(old_prior_bytes.as_slice())
            .and_then(|tail| tail.strip_prefix(old_append.as_slice()))
            .expect("retain the actual settled timing suffix after prior-plus-one");
        let mut root_bytes = new_prior_bytes;
        append_wire_field(8, &new_turn, &mut root_bytes);
        root_bytes.extend_from_slice(timing_suffix);
        let new_root = Sha256::digest(&root_bytes).to_vec();
        let blobs = open_existing(&fixture.blob_path(), false);
        for (id, bytes) in [(&new_user, &user_bytes), (&new_turn, &turn_bytes), (&new_root, &root_bytes)] {
            assert_eq!(blobs.execute("INSERT INTO blobs(id,data) VALUES(?1,?2)", params![hex(id), bytes])
                .expect("explicit adversarial insertion under the actual new content digest"), 1);
        }
        drop(blobs);
        record["canonical"][ordinal]["priorRootBlobId"] = json!(Some(new_prior));
        record["canonical"][ordinal]["userBlobId"] = json!(new_user);
        record["canonical"][ordinal]["turnBlobId"] = json!(new_turn);
        record["canonical"][ordinal]["rootBlobId"] = json!(new_root);
        mutations.push(json!({"ordinal":ordinal, "old":old, "mutated":record["canonical"][ordinal]}));
        previous = Some((new_root, root_bytes));
    }
    let (root_id, root_bytes) = previous.unwrap();
    let mut metadata = stored_metadata(&fixture);
    metadata["latestRootBlobId"] = json!(hex(&root_id));
    replace_stored_kv(&fixture, "metadata", &hex(&serde_json::to_vec(&metadata).unwrap()));
    replace_stored_kv(&fixture, ORIGINAL_TERMINAL_KEY, &serde_json::to_string(&record).unwrap());
    let mut other_record_fields = record.clone();
    other_record_fields["canonical"] = original_record["canonical"].clone();
    assert_eq!(other_record_fields, original_record,
        "the fault preserves every original input, result, custody, lifecycle and compaction field");
    fixture.expected.root_blob_id = root_id.clone();
    fixture.expected.state_bytes = Some(root_bytes.clone());
    fixture.expected.critical_blobs = vec![(root_id, root_bytes)];
    write_json(&fixture.root.join("prid-adversarial-mutation.json"), &json!({
        "kind":"adversarial-user-image-mutation", "fault":format!("{fault:?}"),
        "genuine_seed_root":seed.fixture.root, "expected_diagnostic":fault.diagnostic(), "mutations":mutations,
    }));
    observe_prior(&fixture, HistoricalCaseKind::LegacyNonemptyNoAlias, &seed.prior, "after-adversarial-mutation");
    fixture
}

#[test]
fn rehashed_user_image_faults_reach_the_specific_fresh_process_semantic_refusal() {
    let seed = produce_case(HistoricalCaseKind::LegacyNonemptyNoAlias, WriterImage::Current);
    let seed_before = persistent_image(&seed.fixture, &seed.processes);
    for fault in [UserImageFault::WrongExistingId, UserImageFault::DuplicatePrior,
        UserImageFault::UnsupportedField, UserImageFault::MissingPrior]
    {
        let fixture = mutate_user_image_copy(&seed, fault);
        let mut processes = seed.processes.clone();
        let before = persistent_image(&fixture, &processes);
        let label = format!("prid-user-image-{fault:?}");
        let child = fixture.reopen(&label, ReopenExpectation::OriginalRefuse);
        let expected_error = ProviderSessionError::Protocol(format!(
            "original turn completion is unavailable: {}", fault.diagnostic(),
        ));
        let expected = format!("original-turn reconciliation refused: {expected_error:?}");
        let diagnostic_count = child.logs.lines().filter(|line| *line == expected.as_str()).count();
        let logs = child.logs.clone();
        let (process, receipt) = finish_child(&fixture, &label, child);
        assert!(receipt.produced.is_none());
        // The parent OriginalRefuse arm prints the actual error. Requiring one
        // exact line makes a missing hook, earlier hash/root/fence failure, or
        // generic refusal non-passing, including on an unmodified old harness.
        assert_eq!(diagnostic_count, 1, "the fresh process must reach the intended user-image check: {logs}");
        processes.push(process);
        assert_eq!(persistent_image(&fixture, &processes), before,
            "semantic refusal cannot repair the adversarial bytes, clear a marker, or replay an effect");
        observe_prior(&fixture, HistoricalCaseKind::LegacyNonemptyNoAlias, &seed.prior, "after-semantic-refusal");
        assert_eq!(persistent_image(&seed.fixture, &seed.processes), seed_before,
            "adversarial copies never mutate the actual genuinely produced positive seed");
    }
}
