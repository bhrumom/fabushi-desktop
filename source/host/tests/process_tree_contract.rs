use std::process::Command;
use std::time::Duration;

use mahayana_host_runtime::runner::process_tree::terminate_process_tree;

#[cfg(unix)]
#[test]
fn production_terminator_kills_real_shell_and_descendant() {
    let mut child = Command::new("sh")
        .args(["-c", "sleep 30 & wait"])
        .spawn()
        .expect("spawn shell with descendant");

    std::thread::sleep(Duration::from_millis(50));
    let pid = child.id();
    let forced = terminate_process_tree(
        &mut child,
        pid,
        Duration::from_millis(500),
        "contract shell",
    )
    .expect("terminate process tree");

    assert!(
        !forced,
        "SIGTERM should stop the contract shell within grace"
    );
    assert!(child.try_wait().expect("child status").is_some());
}
