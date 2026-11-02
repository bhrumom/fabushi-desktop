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
    guard: Mutex<Option<File>>,
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
        if guard.is_none() {
            return Ok(());
        }
        if read_lock_pid(&self.path) == Some(self.pid) {
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        // Remove the projection before closing physical custody. Taking the
        // resource also makes later release/Drop harmless to a same-PID owner.
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

fn open_guard(path: &Path) -> io::Result<File> {
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
    options.open(PathBuf::from(guard_path))
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
    let guard = open_guard(path)?;
    let mut guard_held = try_acquire_guard(&guard)?;
    let mut outcome = HostLockOutcome::Created;
    let mut previous_pid = None;

    for _ in 0..MAX_ACQUIRE_ATTEMPTS {
        if guard_held && try_create_lock(path, pid)? {
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
            if !guard_held {
                return Err(guard_contention(path));
            }
            outcome = HostLockOutcome::ReclaimedDead;
            remove_lock_if_pid_matches(path, None)?;
            continue;
        };
        if holder == pid {
            if !guard_held {
                return Err(guard_contention(path));
            }
            remove_lock_if_pid_matches(path, Some(holder))?;
            continue;
        }
        if !is_alive(holder) {
            if !guard_held {
                return Err(guard_contention(path));
            }
            outcome = HostLockOutcome::ReclaimedDead;
            previous_pid = Some(holder);
            remove_lock_if_pid_matches(path, Some(holder))?;
            continue;
        }
        if !is_host_process(holder) {
            if !guard_held {
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
        if !guard_held {
            // A liveness observation does not release another process's file
            // lock. No PID removal or publication may precede this acquisition.
            guard_held = try_acquire_guard(&guard)?;
            if !guard_held {
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
