use mahayana_host_runtime::extensions::host_upgrade::openclaw_update_startup::{
    is_update_check_due, UpdateTriageReport,
};
use mahayana_host_runtime::extensions::local_exec::openclaw_platform_sandbox::{
    escape_powershell_argument, is_wsl_version_output, is_zstd_compressed, ZSTD_MAGIC_HEADER,
};
use mahayana_host_runtime::extensions::box_lifecycle::openclaw_worker_pool::{
    WorkerPoolConfig, WorkerTaskPool,
};

#[test]
fn test_ocrm_infra_update_startup_and_triage() {
    // Check due calculation
    assert!(is_update_check_due(1000, 5000, 3000));
    assert!(!is_update_check_due(1000, 2000, 3000));

    // Update triage for supervised service
    let triage_supervised = UpdateTriageReport::from_error(
        "Update failed: system supervisor service required",
        Some(1),
        true,
    );
    assert!(triage_supervised.requires_supervisor);
    assert!(triage_supervised.suggested_remedy.contains("systemctl restart openclaw"));

    // Update triage for manual user run
    let triage_manual = UpdateTriageReport::from_error("Network timeout during fetch", Some(2), false);
    assert!(!triage_manual.requires_supervisor);
    assert!(triage_manual.suggested_remedy.contains("openclaw update"));
}

#[test]
fn test_ocrm_infra_platform_sandbox_and_zstd() {
    // Zstd magic header detection
    let mut zstd_data = ZSTD_MAGIC_HEADER.to_vec();
    zstd_data.extend_from_slice(b"sample compressed payload");
    assert!(is_zstd_compressed(&zstd_data));

    let non_zstd = b"plain text header payload";
    assert!(!is_zstd_compressed(non_zstd));

    // PowerShell argument escaping
    assert_eq!(escape_powershell_argument("simple"), "simple");
    assert_eq!(escape_powershell_argument(""), "''");
    assert_eq!(
        escape_powershell_argument("arg with space"),
        "\"arg with space\""
    );
    assert_eq!(
        escape_powershell_argument("$VAR"),
        "\"`$VAR\""
    );

    // WSL version detection
    assert!(is_wsl_version_output("Linux version 5.15.153.1-microsoft-standard-WSL2"));
    assert!(!is_wsl_version_output("Linux version 6.1.0-28-amd64 (Debian 6.1.119-1)"));
}

#[test]
fn test_ocrm_infra_worker_pool_lifecycle() {
    let config = WorkerPoolConfig {
        min_workers: 1,
        max_workers: 2,
        idle_timeout_ms: 5000,
        max_memory_bytes_per_worker: 256 * 1024 * 1024,
    };

    let mut pool = WorkerTaskPool::new(config);
    pool.register_worker("worker-1", 1000);
    pool.register_worker("worker-2", 1000);

    // Assign and complete tasks
    assert!(pool.assign_task("worker-1").is_ok());
    assert!(pool.complete_task("worker-1", 2000).is_ok());

    // Sweep at t=4000: neither has been idle for > 5000ms
    assert!(pool.sweep_idle_workers(4000).is_empty());

    // Sweep at t=8000: worker-1 was idle since t=2000 (diff 6000 > 5000), worker-2 was idle since t=1000 (diff 7000 > 5000)
    let idle = pool.sweep_idle_workers(8000);
    assert_eq!(idle.len(), 2);
}
