use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::r#box::box_transfer::TransferBox;
use mahayana_host_runtime::ports::r#box::SAND_BOX_NOT_READY_MESSAGE;
use mahayana_host_runtime::runner::tools::sand_file_transfer_tools::{
    CopyFromBoxArgs, CopyToBoxArgs, FileTransferController, UserComputerHandle,
    copy_file_from_box, copy_file_to_box, format_bytes,
};

#[derive(Debug, Default)]
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

fn controller() -> (
    Arc<MemoryBox>,
    Arc<MemoryBox>,
    FileTransferController<MemoryBox>,
) {
    let agent = Arc::new(MemoryBox::default());
    let computer = Arc::new(MemoryBox::default());
    computer
        .files
        .lock()
        .unwrap()
        .insert("/Users/me/report.pdf".into(), vec![0, 1, 2, 255]);
    let controller = FileTransferController {
        agent_box: agent.clone(),
        user_computers: vec![
            UserComputerHandle {
                id: "mac".into(),
                label: "My Mac".into(),
                connected: true,
                box_: computer.clone(),
            },
            UserComputerHandle {
                id: "old".into(),
                label: "Old Mac".into(),
                connected: false,
                box_: Arc::new(MemoryBox::default()),
            },
        ],
        default_computer_id: Some("mac".into()),
        computer_agent_id: "computer-agent".into(),
        box_id: "box-agent".into(),
        box_preparing: false,
    };
    (agent, computer, controller)
}

#[test]
fn frozen_byte_formatting_is_binary_and_human_readable() {
    assert_eq!(format_bytes(0), "0 bytes");
    assert_eq!(format_bytes(1_023), "1023 bytes");
    assert_eq!(format_bytes(1_024), "1 KB");
    assert_eq!(format_bytes(1_536), "1.5 KB");
    assert_eq!(format_bytes(10 * 1_024), "10 KB");
    assert_eq!(format_bytes(2 * 1_024 * 1_024), "2 MB");
}

#[test]
fn copy_to_box_preserves_bytes_and_uses_uploads_default() {
    let (agent, _computer, controller) = controller();
    let message = copy_file_to_box(
        &(),
        &CopyToBoxArgs {
            computer_path: "/Users/me/report.pdf".into(),
            box_path: None,
            computer: None,
        },
        &controller,
    )
    .expect("copy to box");
    assert_eq!(
        agent
            .files
            .lock()
            .unwrap()
            .get("/workspace/uploads/report.pdf")
            .cloned(),
        Some(vec![0, 1, 2, 255])
    );
    assert!(message.contains("from My Mac into your box"));
    assert!(message.contains("(4 bytes)"));
}

#[test]
fn copy_from_box_resolves_workspace_path_and_default_filename() {
    let (agent, computer, controller) = controller();
    agent
        .files
        .lock()
        .unwrap()
        .insert("/workspace/out/report.csv".into(), b"a,b\n1,2\n".to_vec());
    let message = copy_file_from_box(
        &(),
        &CopyFromBoxArgs {
            box_path: "out/report.csv".into(),
            computer_path: None,
            computer: Some("mac".into()),
        },
        &controller,
    )
    .expect("copy from box");
    assert_eq!(
        computer.files.lock().unwrap().get("report.csv").cloned(),
        Some(b"a,b\n1,2\n".to_vec())
    );
    assert!(message.contains("/workspace/out/report.csv"));
    assert!(message.contains("at report.csv"));
}

#[test]
fn unavailable_computers_and_preparing_box_fail_closed() {
    let (_agent, _computer, mut controller) = controller();
    let unknown = controller
        .resolve_computer_or_throw(Some("missing"))
        .unwrap_err()
        .to_string();
    assert!(unknown.contains("Unknown computer \"missing\""));
    assert!(unknown.contains("old (offline)"));

    controller.box_preparing = true;
    assert_eq!(
        controller.assert_box_ready().unwrap_err().to_string(),
        SAND_BOX_NOT_READY_MESSAGE
    );
}
