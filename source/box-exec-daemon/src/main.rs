use std::env;
use std::path::PathBuf;
use std::sync::mpsc;

use serde_json::json;

use crate::server::{BoxExecDaemonOptions, start_box_exec_daemon};

pub fn run_box_exec_daemon_entrypoint() -> Result<(), String> {
    let workspace_root = match env::var_os("SAND_BOX_WORKSPACE_ROOT") {
        Some(value) => PathBuf::from(value),
        None => env::current_dir().map_err(|error| error.to_string())?,
    };
    let port = match env::var("SAND_BOX_EXEC_DAEMON_PORT") {
        Ok(raw) => Some(
            raw.trim()
                .parse::<u16>()
                .map_err(|_| format!("invalid SAND_BOX_EXEC_DAEMON_PORT: {raw}"))?,
        ),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.to_string()),
    };
    let terminals_directory =
        env::var_os("SAND_BOX_TERMINALS_DIRECTORY").map(PathBuf::from);
    let auth_token = env::var("SAND_BOX_EXEC_DAEMON_AUTH_TOKEN").ok();

    let handle = start_box_exec_daemon(BoxExecDaemonOptions {
        host: None,
        port,
        auth_token,
        workspace_root,
        terminals_directory,
        environment: None,
    })?;

    println!(
        "{}",
        json!({
            "event": "box-exec-daemon-ready",
            "url": handle.url,
            "workspaceRoot": handle.workspace_root,
            "terminalsDirectory": handle.terminals_directory,
        })
    );

    let (shutdown_tx, shutdown_rx) = mpsc::sync_channel::<()>(1);
    ctrlc::set_handler(move || {
        let _ = shutdown_tx.try_send(());
    })
    .map_err(|error| format!("could not install box-exec daemon signal handler: {error}"))?;

    let _ = shutdown_rx.recv();
    handle.stop()?;
    Ok(())
}
