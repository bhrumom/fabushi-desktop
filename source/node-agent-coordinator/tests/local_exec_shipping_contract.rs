use std::fs;
use std::path::{Path, PathBuf};

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("coordinator crate must live below source")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = source_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(
        haystack.contains(needle),
        "{label} is missing shipping contract marker {needle:?}"
    );
}

#[test]
fn shipping_local_exec_chain_is_bound_from_electron_through_the_coordinator_to_the_daemon() {
    let native = read("electron-main/local-exec/local-exec-native.ts");
    assert_contains(
        &native,
        r#""local-exec-daemon", "main.cjs""#,
        "Electron local-exec native bridge",
    );
    assert_contains(&native, "export async function spawnLocalExecDaemon", "Electron local-exec native bridge");
    assert_contains(&native, "LOCAL_EXEC_GENERATION_TOKEN_ARG", "Electron local-exec native bridge");
    assert_contains(&native, "LOCAL_EXEC_GENERATION_TOKEN_ENV", "Electron local-exec native bridge");

    let production_provider = read("electron-main/coordinator/production-provider.ts");
    assert_contains(
        &production_provider,
        "requiredFunction(ports.localExecNative?.spawnLocalExecDaemon",
        "Electron Coordinator production provider",
    );
    assert_contains(
        &production_provider,
        "requiredFunction(ports.localExecNative?.terminateProcess",
        "Electron Coordinator production provider",
    );
    assert_contains(
        &production_provider,
        "requiredFunction(ports.localExecNative?.isProcessAlive",
        "Electron Coordinator production provider",
    );
    assert_contains(
        &production_provider,
        "requiredFunction(ports.localExecNative?.readProcessIdentity",
        "Electron Coordinator production provider",
    );
    assert_contains(
        &production_provider,
        "connector.issueLocalExecDaemonCredential",
        "Electron Coordinator production provider",
    );

    let coordinator = read("node-agent-coordinator/src/main.rs");
    assert_contains(&coordinator, "LocalExecDaemonRuntime::new", "Mahayana Coordinator");
    assert_contains(&coordinator, r#""mintLocalExecDaemonCredential""#, "Mahayana Coordinator");
    assert_contains(&coordinator, r#""spawnLocalExecDaemon""#, "Mahayana Coordinator");
    assert_contains(&coordinator, r#""getProcessIdentity""#, "Mahayana Coordinator");
    assert_contains(&coordinator, r#""terminateProcess""#, "Mahayana Coordinator");

    let entry = read("local-exec-daemon/main.ts");
    assert_contains(&entry, "runLocalExecDaemon", "local-exec daemon entry");
    assert_contains(
        &entry,
        "createDefaultProductionLocalExecExecutor",
        "local-exec daemon entry",
    );
    assert_contains(&entry, "LOCAL_EXEC_GENERATION_TOKEN_ENV", "local-exec daemon entry");
    assert_contains(&entry, "entryRealpath: realpathSync(invokedEntry)", "local-exec daemon entry");
}

#[test]
fn shipping_local_exec_daemon_composes_the_frozen_provider_machine_approvals_and_sentry_modules() {
    let daemon = read("host/local-exec/local-exec-daemon.ts");
    assert_contains(&daemon, "SandLocalExecProvider", "frozen local-exec daemon");
    assert_contains(&daemon, "readLiveLocalToolApprovals", "frozen local-exec daemon");
    assert_contains(&daemon, "retireLocalToolApproval", "frozen local-exec daemon");
    assert_contains(&daemon, "publishDaemonDiscovery", "frozen local-exec daemon");

    let machine = read("host/local-exec/local-exec-machine.ts");
    assert_contains(&machine, "export function buildLocalExecManager", "frozen local-exec machine");
    assert_contains(&machine, "containPath", "frozen local-exec machine");
    assert_contains(&machine, "resolveShellWorkingDirectory", "frozen local-exec machine");

    let provider = read("host/local-exec/local-exec-provider.ts");
    assert_contains(&provider, "export class SandLocalExecProvider", "frozen local-exec provider");
    assert_contains(&provider, "GATEWAY_LOCAL_EXEC_REQUESTS_PATH", "frozen local-exec provider");
    assert_contains(&provider, "GATEWAY_LOCAL_EXEC_RESPONSES_PATH", "frozen local-exec provider");
    assert_contains(&provider, "this.options.executor.execute", "frozen local-exec provider");
    assert_contains(&provider, "this.options.executor.cancel", "frozen local-exec provider");

    let approvals = read("host/local-exec/local-tool-approvals.ts");
    assert_contains(
        &approvals,
        "export async function readLiveLocalToolApprovals",
        "frozen local-tool approvals",
    );
    assert_contains(
        &approvals,
        "export async function retireLocalToolApproval",
        "frozen local-tool approvals",
    );

    let sentry = read("host/local-exec/sentry.ts");
    assert_contains(&sentry, "SandSentryPrivacyGate", "frozen local-exec sentry");
    assert_contains(&sentry, "export function initSandSentryDaemon", "frozen local-exec sentry");
    assert_contains(&sentry, "export function flushSandSentry", "frozen local-exec sentry");

    let production_executor = read("local-exec-daemon/production-executor.ts");
    assert_contains(
        &production_executor,
        "buildLocalExecManager",
        "production local-exec executor",
    );
    assert_contains(
        &production_executor,
        "PRODUCTION_LOCAL_EXEC_RUNTIME_BINDINGS",
        "production local-exec executor",
    );
    assert_contains(
        &production_executor,
        "createDefaultProductionLocalExecExecutor",
        "production local-exec executor",
    );

    let invariant_log = read("local-exec-daemon/invariant-violation-log.ts");
    assert_contains(
        &invariant_log,
        "writeInvariantViolationLog",
        "local-exec invariant violation log",
    );
}
