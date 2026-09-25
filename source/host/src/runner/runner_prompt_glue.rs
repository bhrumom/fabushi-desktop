use serde_json::Value;

use crate::r#box::box_transfer::TransferBox;
use crate::extensions::inference::provider_session::ProviderMessage;

use super::box_tool_access::RunnerBoxResourcePort;
use super::large_output_spill::{
    MCP_TEXT_FILE_THRESHOLD_BYTES, is_large_output_spill_enabled, maybe_spill_mcp_text_result,
};
use super::prompt_collector_glue::{
    ProviderPromptProjection, project_provider_messages_for_turn,
};
use super::tools::sand_file_transfer_tools::FileTransferController;

#[derive(Debug, Clone)]
pub struct RunnerPromptGlue<BoxType> {
    file_transfer_controller: FileTransferController<BoxType>,
    mcp_text_spill_enabled: bool,
}

impl<BoxType> RunnerPromptGlue<BoxType> {
    pub fn new(
        file_transfer_controller: FileTransferController<BoxType>,
        mcp_text_spill_enabled: bool,
    ) -> Self {
        Self {
            file_transfer_controller,
            mcp_text_spill_enabled,
        }
    }

    pub fn from_environment(
        file_transfer_controller: FileTransferController<BoxType>,
    ) -> Self {
        Self::new(file_transfer_controller, is_large_output_spill_enabled())
    }

    pub fn file_transfer_controller(&self) -> &FileTransferController<BoxType> {
        &self.file_transfer_controller
    }

    pub fn project_provider_messages(
        &self,
        args: &Value,
        messages: &[ProviderMessage],
    ) -> ProviderPromptProjection {
        project_provider_messages_for_turn(args, messages)
    }

    pub fn mcp_text_spill_enabled(&self) -> bool {
        self.mcp_text_spill_enabled
    }

    pub fn spill_mcp_text_result(
        &self,
        box_resources: &dyn RunnerBoxResourcePort,
        result: Value,
        tool_call_id: &str,
    ) -> Value {
        self.spill_mcp_text_result_with_threshold(
            box_resources,
            result,
            tool_call_id,
            MCP_TEXT_FILE_THRESHOLD_BYTES,
        )
    }

    pub fn spill_mcp_text_result_with_threshold(
        &self,
        box_resources: &dyn RunnerBoxResourcePort,
        result: Value,
        tool_call_id: &str,
        threshold_bytes: usize,
    ) -> Value {
        if !self.mcp_text_spill_enabled {
            return result;
        }
        maybe_spill_mcp_text_result(
            box_resources,
            result,
            tool_call_id,
            threshold_bytes,
        )
    }
}

pub fn create_runner_prompt_glue<BoxType>(
    file_transfer_controller: FileTransferController<BoxType>,
) -> RunnerPromptGlue<BoxType> {
    RunnerPromptGlue::from_environment(file_transfer_controller)
}

pub fn assert_transfer_box_bound<Ctx, BoxType>(
    glue: &RunnerPromptGlue<BoxType>,
) where
    BoxType: TransferBox<Ctx>,
{
    let _ = glue.file_transfer_controller();
}
