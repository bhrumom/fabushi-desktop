use std::sync::Arc;

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
}

impl ForeverBoxRunnerResourcePort {
    pub fn new(
        service: Arc<ForeverBoxService>,
        agent_id: impl Into<String>,
    ) -> Self {
        Self {
            service,
            agent_id: agent_id.into(),
        }
    }

    fn production_accessor(
        &self,
    ) -> Result<crate::r#box::generated_production::ProductionBoxResourceAccessor, ProviderSessionError>
    {
        self.service
            .box_()
            .ensure_ready(&self.agent_id)
            .map(|ready| ready.remote_accessor)
            .map_err(|error| {
                ProviderSessionError::Tool(format!(
                    "Box is not ready for {}: {error}",
                    self.agent_id
                ))
            })
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
        let mut accessor = self.production_accessor()?;
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
        let mut accessor = self.production_accessor()?;
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
        let mut accessor = self.production_accessor()?;
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
