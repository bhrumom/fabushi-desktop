use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::r#box::box_transfer::TransferBox;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError,
};
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest,
    RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::runner_prompt_glue::RunnerPromptGlue;
use mahayana_host_runtime::runner::tools::sand_file_transfer_tools::{
    FileTransferController, UserComputerHandle,
};
use serde_json::{Value, json};

#[derive(Default)]
struct MemoryBox {
    files: Mutex<HashMap<String, Vec<u8>>>,
}
impl TransferBox<()> for MemoryBox {
    type Error = io::Error;
    fn download_file(
        &self,
        _ctx: &(),
        _agent_id: &str,
        path: &str,
    ) -> Result<Vec<u8>, Self::Error> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing"))
    }
    fn upload_file(
        &self,
        _ctx: &(),
        _agent_id: &str,
        path: &str,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), data.to_vec());
        Ok(())
    }
}

#[derive(Default)]
struct BoxResources {
    writes: Mutex<Vec<RunnerBoxWriteRequest>>,
}
impl RunnerBoxResourcePort for BoxResources {
    fn execute_shell(
        &self,
        _request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!()
    }
    fn execute_read(
        &self,
        _request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!()
    }
    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().unwrap().push(request);
        Ok(())
    }
}

fn transfer_controller() -> FileTransferController<MemoryBox> {
    let agent = Arc::new(MemoryBox::default());
    let computer = Arc::new(MemoryBox::default());
    FileTransferController {
        agent_box: agent,
        user_computers: vec![UserComputerHandle {
            id: "mac".into(),
            label: "Mac".into(),
            connected: true,
            box_: computer,
        }],
        default_computer_id: Some("mac".into()),
        computer_agent_id: "computer-agent".into(),
        box_id: "box-agent".into(),
        box_preparing: false,
    }
}

#[test]
fn glue_reuses_file_transfer_owner_and_prompt_collector_projection() {
    let glue = RunnerPromptGlue::new(transfer_controller(), false);
    assert_eq!(glue.file_transfer_controller().box_id, "box-agent");
    let projected = glue.project_provider_messages(
        &json!({
            "messageId":"user-7",
            "attachmentPaths":["/tmp/a.txt"],
            "boxPathByHostPath":{"/tmp/a.txt":"/workspace/uploads/a.txt"}
        }),
        &[ProviderMessage {
            role: "user".into(),
            content: "Read this".into(),
        }],
    );
    let user = projected
        .messages
        .iter()
        .find(|message| message.role == "user")
        .expect("projected user");
    assert!(user.content.contains("user-7"));
    assert!(user.content.contains("/workspace/uploads/a.txt"));
}

#[test]
fn glue_applies_single_large_output_spill_policy() {
    let resources = BoxResources::default();
    let result = json!({
        "content":[{"type":"text","text":"0123456789abcdef"}]
    });

    let disabled = RunnerPromptGlue::new(transfer_controller(), false);
    let unchanged = disabled.spill_mcp_text_result_with_threshold(
        &resources,
        result.clone(),
        "tool-1",
        4,
    );
    assert_eq!(unchanged, result);
    assert!(resources.writes.lock().unwrap().is_empty());

    let enabled = RunnerPromptGlue::new(transfer_controller(), true);
    let spilled = enabled.spill_mcp_text_result_with_threshold(
        &resources,
        result,
        "tool-2",
        4,
    );
    assert_eq!(resources.writes.lock().unwrap().len(), 1);
    assert_eq!(spilled["content"][0]["type"], "text");
    assert_eq!(spilled["content"][0]["text"], "");
    assert!(spilled["content"][0]["outputLocation"]["filePath"]
        .as_str()
        .unwrap()
        .starts_with(".sand/tools/"));
}
