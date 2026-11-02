use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub const DEFAULT_TAKEOVER_TIMEOUT_MS: u64 = 3_000;
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 100;
pub const MAX_ACQUIRE_ATTEMPTS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostLockOutcome {
    Created,
    ReclaimedDead,
    ReclaimedForeign,
    TookOver,
}

#[derive(Debug)]
pub struct HostLockHandle {
    path: PathBuf,
    pid: u32,
    guard: Mutex<Option<PhysicalGuard>>,
}

// This is the resource retained by the existing Host lock owner. Closing its
// descriptor alone is insufficient when a concurrent fork has inherited the
// same open file description; release the native lock explicitly on every exit.
#[derive(Debug)]
struct PhysicalGuard {
    file: File,
    held: bool,
}

impl PhysicalGuard {
    fn try_acquire(&mut self) -> io::Result<bool> {
        if !self.held {
            self.held = try_acquire_guard(&self.file)?;
        }
        Ok(self.held)
    }

    fn unlock(&mut self) -> io::Result<()> {
        if self.held {
            fs2::FileExt::unlock(&self.file)?;
            self.held = false;
        }
        Ok(())
    }
}

impl Drop for PhysicalGuard {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

impl HostLockHandle {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn release(&self) -> io::Result<()> {
        let mut guard = self
            .guard
            .lock()
            .map_err(|_| io::Error::other("Host lock release state is poisoned"))?;
        let Some(physical_guard) = guard.as_mut() else {
            return Ok(());
        };
        if read_lock_pid(&self.path) == Some(self.pid) {
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        // Remove the projection before releasing physical custody. Taking the
        // resource also makes later release/Drop harmless to a same-PID owner.
        physical_guard.unlock()?;
        drop(guard.take());
        Ok(())
    }
}

impl Drop for HostLockHandle {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[derive(Debug)]
pub struct HostLockAcquisition {
    pub outcome: HostLockOutcome,
    pub lock: HostLockHandle,
    pub previous_pid: Option<u32>,
}

pub fn read_lock_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|pid| *pid > 0)
}

fn open_guard(path: &Path) -> io::Result<PhysicalGuard> {
    let mut guard_path = path.as_os_str().to_os_string();
    guard_path.push(".guard");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // Never unlink this file: a waiter must keep referring to the same inode.
    Ok(PhysicalGuard {
        file: options.open(PathBuf::from(guard_path))?,
        held: false,
    })
}

fn try_acquire_guard(guard: &File) -> io::Result<bool> {
    let contended = fs2::lock_contended_error().raw_os_error();
    match fs2::FileExt::try_lock_exclusive(guard) {
        Ok(()) => Ok(true),
        Err(error) if contended.is_some() && error.raw_os_error() == contended => Ok(false),
        Err(error) => Err(error),
    }
}

fn guard_contention(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::WouldBlock,
        format!("Host physical custody is still held for {}", path.display()),
    )
}

fn remove_lock_if_pid_matches(path: &Path, expected_pid: Option<u32>) -> io::Result<bool> {
    // A liveness probe or takeover can outlast the PID file it inspected.
    // Leave a visibly newer holder for the next exclusive acquisition attempt.
    // Participating owners serialize this recheck/removal under the OS guard;
    // this is not an atomic CAS against older writers or external replacement.
    if read_lock_pid(path) != expected_pid {
        return Ok(false);
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn try_create_lock(path: &Path, pid: u32) -> io::Result<bool> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            write!(file, "{pid}")?;
            file.flush()?;
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error),
    }
}

pub fn read_process_command(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    if let Ok(value) = fs::read(format!("/proc/{pid}/cmdline")) {
        if !value.is_empty() {
            return Some(
                String::from_utf8_lossy(&value)
                    .replace('\0', " ")
                    .trim()
                    .to_string(),
            );
        }
    }

    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "command="])
            .output()
            .ok()?;
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .ok()?;
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() && !value.starts_with("INFO:") {
                return Some(value);
            }
        }
    }

    None
}

pub fn default_is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .is_ok_and(|status| status.success())
    }
    #[cfg(windows)]
    {
        read_process_command(pid).is_some()
    }
}

pub fn is_sand_host_process(pid: u32) -> bool {
    read_process_command(pid).is_some_and(|command| {
        command.contains("host-main") || command.contains("mahayana-app-host")
    })
}

fn terminate_process(pid: u32, force: bool) {
    #[cfg(unix)]
    {
        let signal = if force { "-KILL" } else { "-TERM" };
        let _ = Command::new("kill")
            .args([signal, &pid.to_string()])
            .status();
    }
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill");
        command.args(["/PID", &pid.to_string(), "/T"]);
        if force {
            command.arg("/F");
        }
        let _ = command.status();
    }
}

pub fn acquire_host_lock_with<IsAlive, IsHost, Terminate, Delay>(
    path: &Path,
    pid: u32,
    mut is_alive: IsAlive,
    mut is_host_process: IsHost,
    mut terminate: Terminate,
    mut delay: Delay,
    takeover_timeout_ms: u64,
    poll_interval_ms: u64,
) -> io::Result<HostLockAcquisition>
where
    IsAlive: FnMut(u32) -> bool,
    IsHost: FnMut(u32) -> bool,
    Terminate: FnMut(u32, bool),
    Delay: FnMut(Duration),
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut guard = open_guard(path)?;
    guard.try_acquire()?;
    let mut outcome = HostLockOutcome::Created;
    let mut previous_pid = None;

    for _ in 0..MAX_ACQUIRE_ATTEMPTS {
        if guard.held && try_create_lock(path, pid)? {
            return Ok(HostLockAcquisition {
                outcome,
                lock: HostLockHandle {
                    path: path.to_path_buf(),
                    pid,
                    guard: Mutex::new(Some(guard)),
                },
                previous_pid,
            });
        }

        let holder = read_lock_pid(path);
        let Some(holder) = holder else {
            if !guard.held {
                return Err(guard_contention(path));
            }
            outcome = HostLockOutcome::ReclaimedDead;
            remove_lock_if_pid_matches(path, None)?;
            continue;
        };
        if holder == pid {
            if !guard.held {
                return Err(guard_contention(path));
            }
            remove_lock_if_pid_matches(path, Some(holder))?;
            continue;
        }
        if !is_alive(holder) {
            if !guard.held {
                return Err(guard_contention(path));
            }
            outcome = HostLockOutcome::ReclaimedDead;
            previous_pid = Some(holder);
            remove_lock_if_pid_matches(path, Some(holder))?;
            continue;
        }
        if !is_host_process(holder) {
            if !guard.held {
                return Err(guard_contention(path));
            }
            outcome = HostLockOutcome::ReclaimedForeign;
            previous_pid = Some(holder);
            remove_lock_if_pid_matches(path, Some(holder))?;
            continue;
        }

        outcome = HostLockOutcome::TookOver;
        previous_pid = Some(holder);
        terminate(holder, false);
        let poll = poll_interval_ms.max(1);
        let attempts = takeover_timeout_ms.div_ceil(poll).max(1);
        for _ in 0..attempts {
            if !is_alive(holder) {
                break;
            }
            delay(Duration::from_millis(poll));
        }
        if is_alive(holder) {
            terminate(holder, true);
            for _ in 0..attempts {
                if !is_alive(holder) {
                    break;
                }
                delay(Duration::from_millis(poll));
            }
        }
        if is_alive(holder) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "Host process {holder} is still running after graceful and forced takeover polling"
                ),
            ));
        }
        if !guard.held {
            // A liveness observation does not release another process's file
            // lock. No PID removal or publication may precede this acquisition.
            if !guard.try_acquire()? {
                return Err(guard_contention(path));
            }
        }
        remove_lock_if_pid_matches(path, Some(holder))?;
    }

    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        format!("Host lock acquisition exhausted {MAX_ACQUIRE_ATTEMPTS} exclusive attempts"),
    ))
}

pub fn acquire_host_lock(path: &Path, pid: u32) -> io::Result<HostLockAcquisition> {
    acquire_host_lock_with(
        path,
        pid,
        default_is_process_alive,
        is_sand_host_process,
        terminate_process,
        thread::sleep,
        DEFAULT_TAKEOVER_TIMEOUT_MS,
        DEFAULT_POLL_INTERVAL_MS,
    )
}

#[cfg(test)]
mod physical_guard_tests {
    use super::*;
    use uuid::Uuid;

    fn root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "fabushi-host-guard-{name}-{}", Uuid::new_v4(),
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    // A retained duplicate makes the close-only defect deterministic without
    // relying on scheduling another test's fork between release and reacquire.
    #[cfg(unix)]
    #[test]
    fn native_close_only_control_keeps_lock_until_the_duplicate_is_unlocked() {
        let root = root("native-close");
        let path = root.join("guard");
        let file = OpenOptions::new().read(true).write(true).create_new(true)
            .open(&path).unwrap();
        fs2::FileExt::try_lock_exclusive(&file).unwrap();
        let duplicate = file.try_clone().unwrap();
        drop(file);
        let probe = OpenOptions::new().read(true).write(true).open(&path).unwrap();
        assert!(!try_acquire_guard(&probe).unwrap(), "the native prerequisite must hold");
        fs2::FileExt::unlock(&duplicate).unwrap();
        assert!(try_acquire_guard(&probe).unwrap());
        fs2::FileExt::unlock(&probe).unwrap();
        drop(probe);
        drop(duplicate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn public_release_and_drop_unlock_even_with_a_retained_duplicate() {
        for explicit_release in [true, false] {
            let root = root("public-release");
            let path = root.join("host.lock");
            let owner = acquire_host_lock(&path, 41).unwrap();
            let duplicate = owner.lock.guard.lock().unwrap().as_ref().unwrap()
                .file.try_clone().unwrap();
            let mut probe = open_guard(&path).unwrap();
            assert!(!probe.try_acquire().unwrap());
            if explicit_release {
                owner.lock.release().unwrap();
                owner.lock.release().unwrap();
            }
            drop(owner);
            assert!(!path.exists());

            // The duplicate deliberately remains open through successor entry.
            let successor = acquire_host_lock(&path, 99).unwrap();
            assert_eq!(read_lock_pid(&path), Some(99));
            drop(duplicate);
            assert!(!probe.try_acquire().unwrap(), "old cleanup must preserve the successor");
            assert_eq!(read_lock_pid(&path), Some(99));
            successor.lock.release().unwrap();
            assert!(probe.try_acquire().unwrap());
            drop(probe);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn unacquired_guard_cleanup_preserves_the_current_holder() {
        let root = root("unacquired");
        let path = root.join("host.lock");
        let owner = acquire_host_lock(&path, 41).unwrap();
        let mut contender = open_guard(&path).unwrap();
        assert!(!contender.try_acquire().unwrap());
        contender.unlock().unwrap();
        drop(contender);
        let mut probe = open_guard(&path).unwrap();
        assert!(!probe.try_acquire().unwrap());
        assert_eq!(read_lock_pid(&path), Some(41));
        owner.lock.release().unwrap();
        assert!(probe.try_acquire().unwrap());
        drop(probe);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn acquired_guard_cleanup_unlocks_with_a_retained_duplicate() {
        let root = root("acquired-cleanup");
        let path = root.join("host.lock");
        let mut acquired = open_guard(&path).unwrap();
        assert!(acquired.try_acquire().unwrap());
        let duplicate = acquired.file.try_clone().unwrap();
        let mut probe = open_guard(&path).unwrap();
        assert!(!probe.try_acquire().unwrap());
        // The same resource Drop runs when any admission step returns an error.
        drop(acquired);
        assert!(probe.try_acquire().unwrap());
        drop(duplicate);
        let mut third = open_guard(&path).unwrap();
        assert!(!third.try_acquire().unwrap());
        drop(probe);
        assert!(third.try_acquire().unwrap());
        drop(third);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_admission_leaves_the_guard_available_without_removing_bad_projection() {
        let root = root("failed-admission");
        let path = root.join("host.lock");
        fs::create_dir(&path).unwrap();
        let error = acquire_host_lock(&path, 41).unwrap_err();
        assert_ne!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(path.is_dir(), "failed PID removal must preserve the invalid projection");
        let mut probe = open_guard(&path).unwrap();
        assert!(probe.try_acquire().unwrap(), "failed admission must release its acquired guard");
        drop(probe);
        fs::remove_dir(&path).unwrap();
        let successor = acquire_host_lock(&path, 99).unwrap();
        successor.lock.release().unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    struct RestoreDirectoryPermissions {
        path: PathBuf,
        original: Option<fs::Permissions>,
    }

    #[cfg(unix)]
    impl RestoreDirectoryPermissions {
        fn deny_unlink(path: &Path) -> Self {
            use std::os::unix::fs::PermissionsExt;
            let restore = Self {
                path: path.to_path_buf(),
                original: Some(fs::metadata(path).unwrap().permissions()),
            };
            fs::set_permissions(path, fs::Permissions::from_mode(0o500))
                .expect("make owned directory readable/searchable but not writable");
            restore
        }

        fn restore(&mut self) {
            if let Some(original) = self.original.as_ref() {
                fs::set_permissions(&self.path, original.clone())
                    .expect("restore owned directory permissions");
                self.original = None;
            }
        }
    }

    #[cfg(unix)]
    impl Drop for RestoreDirectoryPermissions {
        fn drop(&mut self) {
            if let Some(original) = self.original.as_ref() {
                let _ = fs::set_permissions(&self.path, original.clone());
            }
        }
    }

    #[cfg(unix)]
    fn assert_pid_unlink_failure_with_live_duplicate(explicit_retry: bool) {
        let root = root(if explicit_retry { "unlink-retry" } else { "unlink-drop" });
        let path = root.join("host.lock");
        let pid = std::process::id();
        let mut owner = Some(acquire_host_lock(&path, pid).unwrap());
        let duplicate = owner.as_ref().unwrap().lock.guard.lock().unwrap()
            .as_ref().unwrap().file.try_clone().unwrap();
        let projection = fs::read(&path).unwrap();
        let mut permissions = RestoreDirectoryPermissions::deny_unlink(&root);

        assert_eq!(read_lock_pid(&path), Some(pid));
        let error = owner.as_ref().unwrap().lock.release()
            .expect_err("fixture must establish an actual matching-PID unlink refusal");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(fs::read(&path).unwrap(), projection);
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        assert!(owner.as_ref().unwrap().lock.guard.lock().unwrap()
            .as_ref().unwrap().held, "explicit failure must retain an owned retry capability");

        if explicit_retry {
            permissions.restore();
            owner.as_ref().unwrap().lock.release().expect("retry after actual unlink repair");
            assert!(!path.exists());
        } else {
            drop(owner.take());
            assert_eq!(fs::read(&path).unwrap(), projection,
                "destruction cannot remove the still-protected PID projection");
            permissions.restore();
        }

        let successor = acquire_host_lock(&path, pid)
            .expect("successor must enter while the old duplicate remains open");
        if let Some(old) = owner.as_ref() {
            old.lock.release().expect("repeated old release must preserve the successor");
        }
        drop(owner);
        assert_eq!(read_lock_pid(&path), Some(pid));
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        drop(duplicate);
        assert_eq!(read_lock_pid(&path), Some(pid));
        assert_eq!(acquire_host_lock(&path, pid).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        successor.lock.release().unwrap();
        assert!(!path.exists());
        drop(successor);
        drop(permissions);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn failed_explicit_pid_removal_retains_custody_until_owned_retry_succeeds() {
        assert_pid_unlink_failure_with_live_duplicate(true);
    }

    #[cfg(unix)]
    #[test]
    fn ordinary_drop_unlocks_when_matching_pid_removal_still_fails() {
        assert_pid_unlink_failure_with_live_duplicate(false);
    }
}
