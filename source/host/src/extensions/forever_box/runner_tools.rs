use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::r#box::box_file_transfer::{FileTransferAccessor, WriteExecResult};
use crate::r#box::box_shell_command::{
    HostShellArgsInput, build_host_shell_args,
};
use crate::r#box::box_windows::{ShellAccessor, ShellExecutionOutcome};
use crate::r#box::generated_production::{
    ProductionReadArgs, ProductionReadOutput, ProductionReadResult,
};
use crate::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest,
};
use crate::runner::remote_box_resources::{
    RemoteBoxResourceCoordinator, RemoteConnection,
};

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
        }
    }

    fn connection(
        &self,
    ) -> Result<RemoteConnection<Arc<Mutex<crate::r#box::generated_production::ProductionBoxResourceAccessor>>>, ProviderSessionError>
    {
        let status = self.service.get_status(&self.agent_id);
        let box_preparing = matches!(status.state.as_str(), "starting" | "preparing");
        let service = Arc::clone(&self.service);
        let agent_id = self.agent_id.clone();
        self.coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
        let args = build_host_shell_args(HostShellArgsInput {
            command: request.command,
            name: executable_name,
            working_directory: request.working_directory,
            tool_call_id: request.tool_call_id,
        });
        let accessor = self.production_accessor()?;
        let mut accessor = accessor.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let result = accessor.execute(&(), args).map_err(|error| {
            ProviderSessionError::Tool(format!("Box Shell failed: {error}"))
        })?;
        Ok(match result.result {
            ShellExecutionOutcome::Success { exit_code, stderr } => json!({
                "kind": "success",
                "exitCode": exit_code,
                "stderr": stderr,
            }),
            ShellExecutionOutcome::Failure { case } => json!({
                "kind": "failure",
                "case": case,
            }),
        })
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
