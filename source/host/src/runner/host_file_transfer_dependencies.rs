use std::sync::Arc;

use crate::extensions::forever_box::forever_box_service::ForeverBoxService;
use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::extensions::local_exec::extension::HostLocalExecExtension;
use crate::extensions::local_exec::gateway_local_exec_sand_box::{
    GatewayLocalExecSandBox, GatewayLocalToolGate, LocalExecTransferContext,
};
use crate::extensions::local_tool_permission::extension::HostLocalToolPermissionExtension;
use crate::runner::tools::sand_file_transfer_tools::{
    CopyFromBoxArgs, CopyToBoxArgs, FileTransferController, FileTransferExecutor,
    UserComputerHandle, copy_file_from_box, copy_file_to_box,
};

#[derive(Clone)]
pub struct ProductionFileTransferExecutor {
    forever_box: Arc<ForeverBoxService>,
    local_exec: Arc<HostLocalExecExtension>,
    local_tool_permission: Arc<HostLocalToolPermissionExtension>,
    agent_id: String,
}

impl ProductionFileTransferExecutor {
    pub fn new(
        forever_box: Arc<ForeverBoxService>,
        local_exec: Arc<HostLocalExecExtension>,
        local_tool_permission: Arc<HostLocalToolPermissionExtension>,
        agent_id: impl Into<String>,
    ) -> Self {
        Self {
            forever_box,
            local_exec,
            local_tool_permission,
            agent_id: agent_id.into(),
        }
    }

    fn controller(
        &self,
    ) -> FileTransferController<ForeverBoxService, GatewayLocalExecSandBox> {
        let gate: Arc<dyn GatewayLocalToolGate> = self.local_tool_permission.clone();
        let bridge = self.local_exec.bridge();
        let user_computers = self
            .local_exec
            .list_computers()
            .into_iter()
            .map(|computer| UserComputerHandle {
                id: computer.id.clone(),
                label: computer.label,
                connected: computer.connected,
                box_: Arc::new(
                    GatewayLocalExecSandBox::new(bridge.clone(), Arc::clone(&gate))
                        .for_computer(computer.id),
                ),
            })
            .collect::<Vec<_>>();
        let default_computer_id = self.local_exec.active_computer().map(|computer| computer.id);
        let box_preparing = self.forever_box.get_status(&self.agent_id).state == "preparing";
        FileTransferController {
            agent_box: Arc::clone(&self.forever_box),
            user_computers,
            default_computer_id,
            computer_agent_id: self.agent_id.clone(),
            box_id: self.agent_id.clone(),
            box_preparing,
        }
    }

    fn context(&self, tool_call_id: &str) -> LocalExecTransferContext {
        LocalExecTransferContext {
            agent_id: self.agent_id.clone(),
            tool_call_id: tool_call_id.to_string(),
        }
    }
}

impl FileTransferExecutor for ProductionFileTransferExecutor {
    fn copy_to_box(
        &self,
        args: CopyToBoxArgs,
        tool_call_id: &str,
    ) -> Result<String, ProviderSessionError> {
        copy_file_to_box(&self.context(tool_call_id), &args, &self.controller())
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))
    }

    fn copy_from_box(
        &self,
        args: CopyFromBoxArgs,
        tool_call_id: &str,
    ) -> Result<String, ProviderSessionError> {
        copy_file_from_box(&self.context(tool_call_id), &args, &self.controller())
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))
    }
}
