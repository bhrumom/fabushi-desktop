use std::sync::Arc;

use crate::r#box::box_transfer::{
    BoxTransferError, SAND_BOX_UPLOADS_DIR, TransferBox, TransferEndpoint,
    resolve_box_workspace_path, transfer_file_between_boxes,
};
use crate::ports::r#box::SAND_BOX_NOT_READY_MESSAGE;

#[derive(Debug, Clone)]
pub struct UserComputerHandle<BoxType> {
    pub id: String,
    pub label: String,
    pub connected: bool,
    pub box_: Arc<BoxType>,
}

#[derive(Debug, Clone)]
pub struct FileTransferController<BoxType> {
    pub agent_box: Arc<BoxType>,
    pub user_computers: Vec<UserComputerHandle<BoxType>>,
    pub default_computer_id: Option<String>,
    pub computer_agent_id: String,
    pub box_id: String,
    pub box_preparing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyToBoxArgs {
    pub computer_path: String,
    pub box_path: Option<String>,
    pub computer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyFromBoxArgs {
    pub box_path: String,
    pub computer_path: Option<String>,
    pub computer: Option<String>,
}

pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1_024 {
        return format!("{bytes} bytes");
    }
    let units = ["KB", "MB", "GB"];
    let mut value = bytes as f64 / 1_024.0;
    let mut unit_index = 0usize;
    while value >= 1_024.0 && unit_index < units.len() - 1 {
        value /= 1_024.0;
        unit_index += 1;
    }
    if value >= 10.0 || value.fract() == 0.0 {
        format!("{value:.0} {}", units[unit_index])
    } else {
        format!("{value:.1} {}", units[unit_index])
    }
}

fn posix_basename(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    trimmed
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

impl<BoxType> FileTransferController<BoxType> {
    pub fn resolve_computer_or_throw(
        &self,
        computer_id: Option<&str>,
    ) -> Result<&UserComputerHandle<BoxType>, BoxTransferError> {
        let requested = computer_id.map(str::trim).filter(|value| !value.is_empty());
        let resolved = if let Some(id) = requested {
            self.user_computers
                .iter()
                .find(|computer| computer.id == id && computer.connected)
        } else if let Some(default_id) = self.default_computer_id.as_deref() {
            self.user_computers
                .iter()
                .find(|computer| computer.id == default_id && computer.connected)
        } else {
            let mut connected = self.user_computers.iter().filter(|computer| computer.connected);
            let first = connected.next();
            if connected.next().is_none() { first } else { None }
        };
        if let Some(handle) = resolved {
            return Ok(handle);
        }

        let known = if self.user_computers.is_empty() {
            "none connected".to_string()
        } else {
            self.user_computers
                .iter()
                .map(|computer| {
                    if computer.connected {
                        computer.id.clone()
                    } else {
                        format!("{} (offline)", computer.id)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        match requested {
            None => Err(BoxTransferError(format!(
                "No computer is connected right now (the Grok Bot desktop app must be open and online to transfer files). Connected computers: {known}."
            ))),
            Some(id) => Err(BoxTransferError(format!(
                "Unknown computer \"{id}\". Connected computers: {known}."
            ))),
        }
    }

    pub fn assert_box_ready(&self) -> Result<(), BoxTransferError> {
        if self.box_preparing {
            Err(BoxTransferError(SAND_BOX_NOT_READY_MESSAGE.to_string()))
        } else {
            Ok(())
        }
    }
}

pub fn copy_file_to_box<Ctx, BoxType>(
    context: &Ctx,
    args: &CopyToBoxArgs,
    controller: &FileTransferController<BoxType>,
) -> Result<String, BoxTransferError>
where
    BoxType: TransferBox<Ctx>,
{
    controller.assert_box_ready()?;
    let computer = controller.resolve_computer_or_throw(args.computer.as_deref())?;
    let box_path = resolve_box_workspace_path(
        args.box_path
            .as_deref()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                format!("{SAND_BOX_UPLOADS_DIR}/{}", posix_basename(&args.computer_path))
            })
            .as_str(),
    );
    let bytes = transfer_file_between_boxes(
        context,
        TransferEndpoint {
            box_: computer.box_.as_ref(),
            agent_id: &controller.computer_agent_id,
            path: &args.computer_path,
            label: &computer.label,
        },
        TransferEndpoint {
            box_: controller.agent_box.as_ref(),
            agent_id: &controller.box_id,
            path: &box_path,
            label: "your box",
        },
        None,
    )?;
    Ok(format!(
        "Copied {} from {} into your box at {} ({}). Open it with Shell.",
        args.computer_path,
        computer.label,
        box_path,
        format_bytes(bytes),
    ))
}

pub fn copy_file_from_box<Ctx, BoxType>(
    context: &Ctx,
    args: &CopyFromBoxArgs,
    controller: &FileTransferController<BoxType>,
) -> Result<String, BoxTransferError>
where
    BoxType: TransferBox<Ctx>,
{
    controller.assert_box_ready()?;
    let computer = controller.resolve_computer_or_throw(args.computer.as_deref())?;
    let box_path = resolve_box_workspace_path(&args.box_path);
    let computer_path = args
        .computer_path
        .clone()
        .unwrap_or_else(|| posix_basename(&box_path));
    let bytes = transfer_file_between_boxes(
        context,
        TransferEndpoint {
            box_: controller.agent_box.as_ref(),
            agent_id: &controller.box_id,
            path: &box_path,
            label: "your box",
        },
        TransferEndpoint {
            box_: computer.box_.as_ref(),
            agent_id: &controller.computer_agent_id,
            path: &computer_path,
            label: &computer.label,
        },
        None,
    )?;
    Ok(format!(
        "Copied {} from your box to {} at {} ({}). The user can open it there with ExternalShell.",
        box_path,
        computer.label,
        computer_path,
        format_bytes(bytes),
    ))
}
