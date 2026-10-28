use std::process::Command;
use std::time::Duration;

use mahayana_host_runtime::runner::process_tree::terminate_process_tree;

#[cfg(unix)]
#[test]
fn production_terminator_kills_real_shell_and_descendant() {
    use std::os::unix::process::CommandExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    let pid_file = std::env::temp_dir().join(format!(
        "fabushi-process-tree-{}-{}.pid",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let command = format!("sleep 30 & printf '%s' $! > '{}' ; wait", pid_file.display());

    let mut child = Command::new("sh")
        .args(["-c", &command])
        .process_group(0)
        .spawn()
        .expect("spawn shell with descendant");

    let descendant_pid = (0..100).find_map(|_| {
        let value = std::fs::read_to_string(&pid_file).ok()?;
        let pid = value.trim().parse::<u32>().ok();
        if pid.is_none() {
            std::thread::sleep(Duration::from_millis(10));
        }
        pid
    });
    let descendant_pid = descendant_pid.or_else(|| {
        std::thread::sleep(Duration::from_millis(10));
        std::fs::read_to_string(&pid_file)
            .ok()?
            .trim()
            .parse::<u32>()
            .ok()
    });
    let descendant_pid = descendant_pid.expect("shell should publish descendant pid");
    let pid = child.id();
    let forced = terminate_process_tree(
        &mut child,
        pid,
        Duration::from_millis(500),
        "contract shell",
    )
    .expect("terminate process tree");

    assert!(!forced, "SIGTERM should stop the contract shell within grace");
    assert!(child.try_wait().expect("child status").is_some());

    let descendant_gone = (0..100).any(|_| {
        let output = Command::new("ps")
            .args(["-p", &descendant_pid.to_string(), "-o", "pid="])
            .output()
            .expect("ps");
        if String::from_utf8_lossy(&output.stdout).trim().is_empty() {
            true
        } else {
            std::thread::sleep(Duration::from_millis(10));
            false
        }
    });
    let _ = std::fs::remove_file(pid_file);
    assert!(descendant_gone, "descendant pid {descendant_pid} survived cleanup");
}

#[cfg(unix)]
#[test]
fn production_terminator_escalates_when_root_ignores_graceful_signal() {
    use std::os::unix::process::CommandExt;

    let mut child = Command::new("sh")
        .args(["-c", "trap '' TERM; sleep 30 & wait"])
        .process_group(0)
        .spawn()
        .expect("spawn signal-ignoring shell");
    let pid = child.id();
    let forced = terminate_process_tree(
        &mut child,
        pid,
        Duration::from_millis(50),
        "contract force shell",
    )
    .expect("force terminate process tree");

    assert!(forced, "ignored graceful signal should require force cleanup");
    assert!(child.try_wait().expect("child status").is_some());
}
