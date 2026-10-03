use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::r#box::box_file_transfer::{FileTransferAccessor, WriteExecResult};
use crate::r#box::box_shell_command::{
    HostShellArgsInput, build_host_shell_args,
};
use crate::r#box::box_windows::{ShellAccessor, ShellExecutionOutcome};
use crate::r#box::generated_production::{
    ProductionBackgroundShellSpawnResult, ProductionReadArgs, ProductionReadOutput,
    ProductionReadResult, ProductionShellResult, ProductionShellStreamArgs,
    ProductionShellStreamEvent,
};
use crate::runner::background_work::{
    BackgroundShellWatchOptions, RunnerBackgroundShellWatches,
};
use crate::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest,
};
use crate::runner::remote_box_resources::{
    RemoteBoxResourceCoordinator, RemoteConnection,
};
use crate::runner::shell_terminal_watch::{ShellTerminalPollRead, TerminalReadResult};

use super::forever_box_service::ForeverBoxService;

/// Host-side adapter for the Runner's narrow Box resource port.
///
/// The adapter intentionally keeps ForeverBox lifecycle ownership in Host.
/// Every tool invocation obtains a fresh guarded production accessor from the
/// live HostBox; Runner never owns Box lifecycle, transport credentials, or
/// shared-desktop assignment state.
#[derive(Clone)]
pub struct ForeverBoxRunnerResourcePort {
    service: Arc<ForeverBoxService>,
    agent_id: String,
    coordinator: Arc<Mutex<RemoteBoxResourceCoordinator<Arc<Mutex<crate::r#box::generated_production::ProductionBoxResourceAccessor>>>>>,
    background_shell_watches: Option<Arc<RunnerBackgroundShellWatches>>,
}

impl ForeverBoxRunnerResourcePort {
    pub fn new(
        service: Arc<ForeverBoxService>,
        agent_id: impl Into<String>,
    ) -> Self {
        let desktop_capable = service.box_().inner().shared_desktop().is_some();
        Self {
            service,
            agent_id: agent_id.into(),
            coordinator: Arc::new(Mutex::new(RemoteBoxResourceCoordinator::new(
                desktop_capable,
                None,
            ))),
            background_shell_watches: None,
        }
    }

    pub fn with_background_shell_watches(
        mut self,
        watches: Arc<RunnerBackgroundShellWatches>,
    ) -> Self {
        self.background_shell_watches = Some(watches);
        self
    }

    fn connection(
        &self,
    ) -> Result<RemoteConnection<Arc<Mutex<crate::r#box::generated_production::ProductionBoxResourceAccessor>>>, ProviderSessionError>
    {
        let status = self.service.get_status(&self.agent_id);
        let box_preparing = matches!(status.state.as_str(), "starting" | "preparing");
        let service = Arc::clone(&self.service);
        let agent_id = self.agent_id.clone();
        let mut coordinator = self
            .coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // HostBox owns readiness/authentication and returns a guarded accessor.
        // Do not reuse that transport across Runner tool invocations: a stale
        // cached accessor can outlive box recreation, credential rotation, or
        // daemon readiness changes and bypass the shipping readiness probe.
        coordinator.clear_connection();
        coordinator
            .connect(box_preparing, move || {
                let ready = service
                    .box_()
                    .ensure_ready(&agent_id)
                    .map_err(|error| Arc::new(error) as crate::runner::remote_box_resources::RemoteConnectError)?;
                let window_index = service
                    .box_()
                    .get_agent_window_index(&agent_id)
                    .unwrap_or(1);
                let owns_monitor = service.box_().inner().shared_desktop().is_some()
                    && !ready.vnc_url.trim().is_empty();
                Ok(RemoteConnection {
                    terminals_folder: ready.terminals_folder.to_string(),
                    resource: Arc::new(Mutex::new(ready.remote_accessor)),
                    owns_monitor,
                    window_index,
                })
            })
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))
    }

    fn production_accessor(
        &self,
    ) -> Result<Arc<Mutex<crate::r#box::generated_production::ProductionBoxResourceAccessor>>, ProviderSessionError>
    {
        self.connection().map(|connection| connection.resource)
    }

    /// Read one background-shell terminal snapshot through the shipping
    /// Host-owned ForeverBox resource. Box readiness is re-resolved on every
    /// poll so recreate/credential rotation cannot leave a stale accessor.
    pub fn read_background_shell_terminal(&self, shell_id: &str) -> ShellTerminalPollRead {
        let shell_id = shell_id.trim();
        let connection = match self.connection() {
            Ok(connection) => connection,
            Err(error) => return ShellTerminalPollRead::TransientFailure(error.to_string()),
        };
        let folder = connection.terminals_folder.trim_end_matches('/');
        let output_path = format!("{folder}/{shell_id}.txt");
        let mut accessor = connection
            .resource
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = accessor.execute_read(
            &(),
            ProductionReadArgs {
                path: output_path.clone(),
                tool_call_id: String::new(),
                offset: None,
                limit: None,
                encoding_hint: None,
            },
        );
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                return ShellTerminalPollRead::TransientFailure(error.to_string());
            }
        };
        match result {
            ProductionReadResult::Success { output, .. } => {
                let result = match output {
                    ProductionReadOutput::Content(content) => {
                        TerminalReadResult::SuccessText(content)
                    }
                    ProductionReadOutput::Data(data) => {
                        TerminalReadResult::SuccessData(data)
                    }
                    ProductionReadOutput::None => {
                        TerminalReadResult::SuccessText(String::new())
                    }
                };
                ShellTerminalPollRead::Snapshot { output_path, result }
            }
            ProductionReadResult::FileNotFound { .. } => ShellTerminalPollRead::Snapshot {
                output_path,
                result: TerminalReadResult::FileNotFound,
            },
            ProductionReadResult::PermissionDenied { .. } => {
                ShellTerminalPollRead::PermissionDenied {
                    output_path: Some(output_path),
                }
            }
            ProductionReadResult::Error { error, .. } => {
                ShellTerminalPollRead::TransientFailure(error)
            }
            ProductionReadResult::Rejected { reason, .. } => {
                ShellTerminalPollRead::TransientFailure(reason)
            }
            ProductionReadResult::InvalidFile { reason, .. } => {
                ShellTerminalPollRead::TransientFailure(reason)
            }
            ProductionReadResult::Other { case } => {
                ShellTerminalPollRead::TransientFailure(case)
            }
        }
    }
}

impl RunnerBoxResourcePort for ForeverBoxRunnerResourcePort {
    fn execute_shell(
        &self,
        request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        let executable_name = request
            .command
            .split_whitespace()
            .next()
            .unwrap_or("shell")
            .to_string();
        let should_start_in_background = request.should_start_in_background();
        let requested_block_until_ms = request.block_until_ms;
        let command = request.command.clone();
        let working_directory = request.working_directory.clone();
        let args = build_host_shell_args(HostShellArgsInput {
            command: request.command,
            name: executable_name,
            working_directory: request.working_directory,
            tool_call_id: request.tool_call_id,
        });
        let accessor = self.production_accessor()?;
        let mut accessor = accessor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        if should_start_in_background {
            let watches = self.background_shell_watches.as_ref().ok_or_else(|| {
                ProviderSessionError::Tool(
                    "Box background Shell is unavailable without the Runner background-shell owner"
                        .into(),
                )
            })?;
            let result = accessor
                .execute_background_shell_spawn(&(), args.into())
                .map_err(|error| {
                    ProviderSessionError::Tool(format!("Box background Shell failed: {error}"))
                })?;
            return Ok(match result {
                ProductionBackgroundShellSpawnResult::Success {
                    shell_id,
                    command,
                    working_directory,
                    pid,
                } => {
                    if shell_id == 0 {
                        return Err(ProviderSessionError::Tool(
                            "Box background Shell returned an invalid shellId=0".into(),
                        ));
                    }
                    let shell_work_id = shell_id.to_string();
                    watches.watch_background_shell(
                        &self.agent_id,
                        &shell_work_id,
                        BackgroundShellWatchOptions::new(Some(command.clone()), None),
                    );
                    json!({
                        "kind": "backgrounded",
                        "shellId": shell_id,
                        "command": command,
                        "workingDirectory": working_directory,
                        "pid": pid,
                    })
                }
                ProductionBackgroundShellSpawnResult::Error {
                    command,
                    working_directory,
                    error,
                } => json!({
                    "kind": "error",
                    "command": command,
                    "workingDirectory": working_directory,
                    "error": error,
                }),
                ProductionBackgroundShellSpawnResult::Rejected {
                    command,
                    working_directory,
                    reason,
                    is_readonly,
                } => json!({
                    "kind": "rejected",
                    "command": command,
                    "workingDirectory": working_directory,
                    "reason": reason,
                    "isReadonly": is_readonly,
                }),
                ProductionBackgroundShellSpawnResult::PermissionDenied {
                    command,
                    working_directory,
                    error,
                    is_readonly,
                } => json!({
                    "kind": "permissionDenied",
                    "command": command,
                    "workingDirectory": working_directory,
                    "error": error,
                    "isReadonly": is_readonly,
                }),
                ProductionBackgroundShellSpawnResult::SandboxUnsupported {
                    command,
                    working_directory,
                    sandbox_policy_type,
                    reason,
                    is_readonly,
                } => json!({
                    "kind": "sandboxUnsupported",
                    "command": command,
                    "workingDirectory": working_directory,
                    "sandboxPolicyType": sandbox_policy_type,
                    "reason": reason,
                    "isReadonly": is_readonly,
                }),
                ProductionBackgroundShellSpawnResult::Other { case } => json!({
                    "kind": "failure",
                    "case": case,
                }),
            });
        }

        if let Some(block_until_ms) = requested_block_until_ms {
            let watches = self.background_shell_watches.as_ref().ok_or_else(|| {
                ProviderSessionError::Tool(
                    "Box timed-background Shell is unavailable without the Runner background-shell owner"
                        .into(),
                )
            })?;
            let timeout_ms = i32::try_from(block_until_ms).map_err(|_| {
                ProviderSessionError::Tool(format!(
                    "Box Shell block_until_ms={block_until_ms} exceeds the frozen ShellStream int32 range"
                ))
            })?;
            if timeout_ms <= 0 {
                return Err(ProviderSessionError::Tool(
                    "Box Shell positive block_until_ms must be greater than zero on the ShellStream path"
                        .into(),
                ));
            }
            let events = accessor
                .execute_shell_stream(
                    &(),
                    ProductionShellStreamArgs {
                        shell_args: args,
                        timeout_ms,
                        hard_timeout_ms: None,
                    },
                )
                .map_err(|error| {
                    ProviderSessionError::Tool(format!("Box timed Shell failed: {error}"))
                })?;
            let mut stdout = String::new();
            let mut stderr = String::new();
            let mut last_other = None::<String>;
            for event in events {
                match event {
                    ProductionShellStreamEvent::Start => {}
                    ProductionShellStreamEvent::Stdout(delta) => stdout.push_str(&delta),
                    ProductionShellStreamEvent::Stderr(delta) => stderr.push_str(&delta),
                    ProductionShellStreamEvent::Exit {
                        code,
                        cwd,
                        aborted,
                        local_execution_time_ms,
                    } => {
                        if code == 0 && !aborted {
                            return Ok(json!({
                                "kind": "success",
                                "exitCode": code,
                                "stdout": stdout,
                                "stderr": stderr,
                                "workingDirectory": cwd,
                                "aborted": aborted,
                                "elapsedMs": local_execution_time_ms,
                            }));
                        }
                        return Ok(json!({
                            "kind": "failure",
                            "exitCode": code,
                            "stdout": stdout,
                            "stderr": stderr,
                            "workingDirectory": cwd,
                            "aborted": aborted,
                            "elapsedMs": local_execution_time_ms,
                        }));
                    }
                    ProductionShellStreamEvent::Rejected {
                        command,
                        working_directory,
                        reason,
                        is_readonly,
                    } => {
                        return Ok(json!({
                            "kind": "rejected",
                            "command": command,
                            "workingDirectory": working_directory,
                            "reason": reason,
                            "isReadonly": is_readonly,
                        }));
                    }
                    ProductionShellStreamEvent::PermissionDenied {
                        command,
                        working_directory,
                        error,
                        is_readonly,
                    } => {
                        return Ok(json!({
                            "kind": "permissionDenied",
                            "command": command,
                            "workingDirectory": working_directory,
                            "error": error,
                            "isReadonly": is_readonly,
                        }));
                    }
                    ProductionShellStreamEvent::Backgrounded {
                        shell_id,
                        command,
                        working_directory,
                        pid,
                        ms_to_wait,
                        reason,
                    } => {
                        if shell_id == 0 {
                            return Err(ProviderSessionError::Tool(
                                "Box timed Shell returned an invalid shellId=0".into(),
                            ));
                        }
                        let shell_work_id = shell_id.to_string();
                        watches.watch_background_shell(
                            &self.agent_id,
                            &shell_work_id,
                            BackgroundShellWatchOptions::new(Some(command.clone()), None),
                        );
                        return Ok(json!({
                            "kind": "backgrounded",
                            "shellId": shell_id,
                            "command": command,
                            "workingDirectory": working_directory,
                            "pid": pid,
                            "blockUntilMs": ms_to_wait,
                            "backgroundReason": reason,
                            "stdout": stdout,
                            "stderr": stderr,
                        }));
                    }
                    ProductionShellStreamEvent::Other { case } => {
                        last_other = Some(case);
                    }
                }
            }
            return Err(ProviderSessionError::Tool(format!(
                "Box timed Shell stream closed without exit/backgrounded settlement for command {command:?} in {working_directory:?}: {}",
                last_other.unwrap_or_else(|| "no terminal event".into())
            )));
        }

        let result = accessor.execute_shell_result(&(), args).map_err(|error| {
            ProviderSessionError::Tool(format!("Box Shell failed: {error}"))
        })?;
        Ok(project_shell_result(result))
    }

    fn execute_read(
        &self,
        request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        let accessor = self.production_accessor()?;
        let mut accessor = accessor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = accessor
            .execute_read(
                &(),
                ProductionReadArgs {
                    path: request.path,
                    tool_call_id: request.tool_call_id,
                    offset: request.offset,
                    limit: request.limit,
                    encoding_hint: request.encoding_hint,
                },
            )
            .map_err(|error| {
                ProviderSessionError::Tool(format!("Box Read failed: {error}"))
            })?;
        Ok(match result {
            ProductionReadResult::Success {
                path,
                output,
                total_lines,
                file_size,
                truncated,
                output_blob_id,
                range_applied,
            } => {
                let output = match output {
                    ProductionReadOutput::Content(content) => {
                        json!({"kind":"content","content":content})
                    }
                    ProductionReadOutput::Data(data) => {
                        json!({"kind":"data","data":data})
                    }
                    ProductionReadOutput::None => Value::Null,
                };
                json!({
                    "kind": "success",
                    "path": path,
                    "output": output,
                    "totalLines": total_lines,
                    "fileSize": file_size,
                    "truncated": truncated,
                    "outputBlobId": output_blob_id,
                    "rangeApplied": range_applied,
                })
            }
            ProductionReadResult::Error { path, error } => {
                json!({"kind":"error","path":path,"error":error})
            }
            ProductionReadResult::Rejected { path, reason } => {
                json!({"kind":"rejected","path":path,"reason":reason})
            }
            ProductionReadResult::FileNotFound { path } => {
                json!({"kind":"fileNotFound","path":path})
            }
            ProductionReadResult::PermissionDenied { path } => {
                json!({"kind":"permissionDenied","path":path})
            }
            ProductionReadResult::InvalidFile { path, reason } => {
                json!({"kind":"invalidFile","path":path,"reason":reason})
            }
            ProductionReadResult::Other { case } => {
                json!({"kind":"other","case":case})
            }
        })
    }

    fn execute_computer_use_protobuf(
        &self,
        protobuf_args: Vec<u8>,
    ) -> Result<Vec<u8>, ProviderSessionError> {
        let connection = self.connection()?;
        self.coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .computer_use_plan(&connection, None)
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        let mut accessor = connection
            .resource
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        accessor
            .execute_computer_use_protobuf(&(), protobuf_args)
            .map_err(|error| {
                if error.to_string().to_ascii_lowercase().contains("monitor") {
                    self.coordinator
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .clear_connection();
                }
                ProviderSessionError::Tool(format!("Computer use failed: {error}"))
            })
    }

    fn browser_window_index(&self) -> Result<u32, ProviderSessionError> {
        self.service
            .box_()
            .ensure_ready(&self.agent_id)
            .map_err(|error| {
                ProviderSessionError::Tool(format!(
                    "Box browser is not ready for {}: {error}",
                    self.agent_id
                ))
            })?;
        self.service
            .box_()
            .get_agent_window_index(&self.agent_id)
            .ok_or_else(|| {
                ProviderSessionError::Tool(format!(
                    "Box has not assigned {} a browser window yet",
                    self.agent_id
                ))
            })
    }

    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        let accessor = self.production_accessor()?;
        let mut accessor = accessor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = accessor
            .execute_write(&(), &request.path, &request.data, &request.tool_call_id)
            .map_err(|error| {
                ProviderSessionError::Tool(format!("Box Write failed: {error}"))
            })?;
        match result {
            WriteExecResult::Success => Ok(()),
            WriteExecResult::Error { error } => Err(ProviderSessionError::Tool(format!(
                "Box Write failed: {error}"
            ))),
            WriteExecResult::Rejected { reason } => Err(ProviderSessionError::Tool(format!(
                "Box Write rejected: {reason}"
            ))),
            WriteExecResult::Other { case } => Err(ProviderSessionError::Tool(format!(
                "Box Write failed: {case}"
            ))),
        }
    }

}
fn project_shell_result(result: ProductionShellResult) -> Value {
    match result {
        ProductionShellResult::Success {
            exit_code,
            stdout,
            stderr,
        } => json!({
            "kind": "success",
            "exitCode": exit_code,
            "stdout": stdout,
            "stderr": stderr,
        }),
        ProductionShellResult::Failure {
            exit_code,
            signal,
            stdout,
            stderr,
            aborted,
        } => json!({
            "kind": "failure",
            "exitCode": exit_code,
            "signal": signal,
            "stdout": stdout,
            "stderr": stderr,
            "aborted": aborted,
        }),
        ProductionShellResult::SpawnError { error } => json!({
            "kind": "spawnError",
            "error": error,
        }),
        ProductionShellResult::PermissionDenied { error } => json!({
            "kind": "permissionDenied",
            "error": error,
        }),
        ProductionShellResult::Rejected { reason } => json!({
            "kind": "rejected",
            "reason": reason,
        }),
        ProductionShellResult::Timeout { timeout_ms } => json!({
            "kind": "timeout",
            "timeoutMs": timeout_ms,
        }),
        ProductionShellResult::Other { case } => json!({
            "kind": "failure",
            "case": case,
        }),
    }
}



#[cfg(test)]
mod tests {
    use super::project_shell_result;
    use crate::r#box::generated_production::ProductionShellResult;

    #[test]
    fn normal_shell_projection_preserves_stdout_for_navigation_probes() {
        let value = project_shell_result(ProductionShellResult::Success {
            exit_code: 0,
            stdout: r#"[{\"url\":\"https://example.com/path\"}]"#.into(),
            stderr: String::new(),
        });
        assert_eq!(value["kind"], "success");
        assert_eq!(value["exitCode"], 0);
        assert_eq!(
            value["stdout"],
            r#"[{\"url\":\"https://example.com/path\"}]"#
        );
    }

    #[test]
    fn normal_shell_projection_preserves_failure_diagnostics() {
        let value = project_shell_result(ProductionShellResult::Failure {
            exit_code: 7,
            signal: "SIGTERM".into(),
            stdout: "partial".into(),
            stderr: "failed".into(),
            aborted: true,
        });
        assert_eq!(value["kind"], "failure");
        assert_eq!(value["exitCode"], 7);
        assert_eq!(value["stdout"], "partial");
        assert_eq!(value["stderr"], "failed");
        assert_eq!(value["aborted"], true);
    }
}
