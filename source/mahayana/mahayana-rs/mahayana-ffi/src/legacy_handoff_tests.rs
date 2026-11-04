//! NMR-36: exercise the exported C ABI, not a JSON/source-code surrogate.

use super::*;
use std::fs;
use std::path::{Path, PathBuf};

const REFUSAL: &str = "legacy Telegram handoff is unavailable in the Fabushi host; use the native Session/Transcript owner";

struct OwnedRoot(PathBuf);

impl OwnedRoot {
    fn new() -> Self {
        static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "fabushi-ffi-handoff-{}-{nonce}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        )))
    }

    fn config(&self) -> Value {
        json!({
            "dataDir": self.0.join("runtime"),
            "productSessionPath": self.0.join("product-session.json"),
            "productSurfaceStatePath": self.0.join("product-surfaces.json"),
            "cwd": self.0.join("workspace"),
            "inheritInstalledPlugins": false,
        })
    }
}

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedRuntime(u64);

impl Drop for OwnedRuntime {
    fn drop(&mut self) {
        unsafe {
            mahayana_runtime_free_string(mahayana_runtime_close(self.0));
        }
    }
}

fn take(pointer: *mut c_char) -> Value {
    assert!(!pointer.is_null());
    let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes().to_vec();
    unsafe { mahayana_runtime_free_string(pointer) };
    serde_json::from_slice(&bytes).expect("owned JSON response")
}

fn create(config: &Value) -> u64 {
    let source = CString::new(serde_json::to_string(config).unwrap()).unwrap();
    unsafe { mahayana_runtime_create(source.as_ptr()) }
}

fn status(runtime: &OwnedRuntime) -> Value {
    let command = CString::new(r#"{"@type":"mahayana.runtime.status"}"#).unwrap();
    take(unsafe { mahayana_runtime_execute(runtime.0, command.as_ptr()) })
}

fn assert_rejected(config: &Value) {
    assert_eq!(create(config), 0, "legacy handoff must not acquire a handle");
    let error = take(unsafe { mahayana_runtime_last_error() });
    assert_eq!(error["ok"], false);
    assert_eq!(error["message"], REFUSAL);
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn visit(root: &Path, current: &Path, entries: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if entry.file_type().unwrap().is_dir() {
                entries.push((relative, None));
                visit(root, &path, entries);
            } else {
                entries.push((relative, Some(fs::read(path).unwrap())));
            }
        }
    }
    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

#[test]
fn rejects_legacy_value_matrix_before_creating_any_storage() {
    let root = OwnedRoot::new();
    let cases = [
        (Some(0_u64), None),
        (Some(1), None),
        (Some(u64::MAX), None),
        (None, Some(0_i64)),
        (None, Some(i64::MIN)),
        (None, Some(i64::MAX)),
        (Some(17), Some(29)),
    ];
    for (client, user) in cases {
        let mut config = root.config();
        config["telegramClientId"] = json!(client);
        config["telegramSelfUserId"] = json!(user);
        assert_rejected(&config);
        assert!(!root.0.exists(), "rejection created a configured root");
    }
}

#[test]
fn rejection_preserves_existing_opaque_data_and_open_native_handle() {
    let native_root = OwnedRoot::new();
    fs::create_dir_all(native_root.0.join("workspace")).unwrap();
    let handle = create(&native_root.config());
    assert_ne!(handle, 0, "native runtime is a required positive control");
    let native = OwnedRuntime(handle);
    let before = status(&native);
    assert_eq!(before["ok"], true);
    assert_eq!(before["data"]["runtimeAbiVersion"], 1);

    let retained = OwnedRoot::new();
    let files: [(&str, &[u8]); 6] = [
        ("product-session.json", b"opaque retained session; not JSON"),
        ("product-surfaces.json", b"opaque retained surface; not JSON"),
        ("runtime/telegram.sqlite", b"retained encrypted database\0\xff"),
        ("runtime/telegram.sqlite-wal", b"retained write-ahead bytes"),
        ("runtime/telegram.sqlite-shm", b"retained shared memory bytes"),
        ("runtime/unknown.bin", b"unknown retained format\0\xfe"),
    ];
    for (name, bytes) in files {
        let path = retained.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    let retained_before = snapshot(&retained.0);
    let mut config = retained.config();
    config["telegramClientId"] = json!(u64::MAX);
    config["telegramSelfUserId"] = json!(i64::MIN);
    assert_rejected(&config);
    for (name, bytes) in files {
        assert_eq!(fs::read(retained.0.join(name)).unwrap(), bytes, "{name}");
    }
    assert_eq!(snapshot(&retained.0), retained_before, "no new or removed data");
    assert!(!retained.0.join("workspace").exists());
    assert_eq!(status(&native)["data"], before["data"]);
    let ready = take(unsafe { mahayana_runtime_receive(native.0, 10) });
    assert_eq!(ready["ok"], true);
    assert_eq!(ready["data"]["@type"], "mahayana.runtime.ready");
}

#[test]
fn null_legacy_fields_allow_native_creation_and_clear_previous_error() {
    let root = OwnedRoot::new();
    let mut config = root.config();
    config["telegramSelfUserId"] = json!(0);
    assert_rejected(&config);
    assert!(!root.0.exists());

    fs::create_dir_all(root.0.join("workspace")).unwrap();
    config["telegramClientId"] = Value::Null;
    config["telegramSelfUserId"] = Value::Null;
    let handle = create(&config);
    assert_ne!(handle, 0);
    let native = OwnedRuntime(handle);
    let error = take(unsafe { mahayana_runtime_last_error() });
    assert_eq!(error["ok"], true);
    assert!(error["message"].is_null());
    assert_eq!(status(&native)["ok"], true);
}

#[test]
fn rejection_error_is_owned_by_the_calling_thread() {
    let main_root = OwnedRoot::new();
    fs::create_dir_all(main_root.0.join("workspace")).unwrap();
    let handle = create(&main_root.config());
    assert_ne!(handle, 0);
    let native = OwnedRuntime(handle);
    assert_eq!(take(unsafe { mahayana_runtime_last_error() })["ok"], true);

    std::thread::spawn(|| {
        let root = OwnedRoot::new();
        let mut config = root.config();
        config["telegramClientId"] = json!(42);
        assert_rejected(&config);
        // The error JSON can be independently retrieved and freed again.
        assert_eq!(take(unsafe { mahayana_runtime_last_error() })["message"], REFUSAL);
        assert!(!root.0.exists());
    })
    .join()
    .expect("worker thread");

    assert_eq!(take(unsafe { mahayana_runtime_last_error() })["ok"], true);
    assert_eq!(status(&native)["ok"], true);
}
