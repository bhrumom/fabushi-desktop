use std::sync::Arc;

use serde_json::{Value, json};

use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::extensions::local_exec::extension::HostLocalExecExtension;
use crate::extensions::local_exec::gateway_local_exec_sand_box::{
    GatewayLocalExecManager, GatewayLocalToolGate, GatewayLocalToolScope,
};
use crate::extensions::local_tool_permission::extension::HostLocalToolPermissionExtension;
use crate::runner::tools::sand_external_machine_tools::{
    ExternalMachineExecutor, ExternalMachineReadArgs, ExternalMachineShellArgs,
};

#[derive(Clone)]
pub struct ProductionExternalMachineExecutor {
    manager: Arc<GatewayLocalExecManager>,
    agent_id: String,
}

impl ProductionExternalMachineExecutor {
    pub fn new(
        local_exec: Arc<HostLocalExecExtension>,
        local_tool_permission: Arc<HostLocalToolPermissionExtension>,
        agent_id: impl Into<String>,
    ) -> Self {
        let gate: Arc<dyn GatewayLocalToolGate> = local_tool_permission;
        let manager = local_exec
            .sandbox(gate)
            .remote_resource_accessor()
            .manager();
        Self {
            manager,
            agent_id: agent_id.into(),
        }
    }

    fn scope(&self, tool_call_id: &str, action: &str) -> GatewayLocalToolScope {
        GatewayLocalToolScope {
            agent_id: Some(self.agent_id.clone()),
            tool_call_id: Some(tool_call_id.to_string()),
            action: Some(action.to_string()),
        }
    }
}

impl ExternalMachineExecutor for ProductionExternalMachineExecutor {
    fn execute_shell(
        &self,
        args: &ExternalMachineShellArgs,
    ) -> Result<Value, ProviderSessionError> {
        let result = self
            .manager
            .execute_shell(
                &self.scope(&args.tool_call_id, "run-command"),
                &args.command,
                &args.working_directory,
                &args.tool_call_id,
            )
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        Ok(json!({
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exitCode": result.exit_code,
            "cwd": result.cwd,
            "aborted": result.aborted
        }))
    }

    fn execute_read(
        &self,
        args: &ExternalMachineReadArgs,
    ) -> Result<Value, ProviderSessionError> {
        self.manager
            .execute_read(
                &self.scope(&args.tool_call_id, "read-file"),
                &args.path,
                &args.tool_call_id,
                args.offset,
                args.limit,
                args.encoding_hint.as_deref(),
            )
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))
    }
}
