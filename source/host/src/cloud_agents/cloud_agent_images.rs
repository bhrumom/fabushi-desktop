use std::fs;
use std::path::{Path, PathBuf};

use url::Url;

use crate::attachment_paths::get_agent_media_store_roots;

pub const SAND_BOX_WORKSPACE_ROOT: &str = "/workspace";
pub const ATTACHMENT_BYTE_LIMIT: u64 = 25 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudAgentImage {
    pub data: Vec<u8>,
    pub path: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudAgentImagesError {
    NotFileUrl(String),
    NotImage(String),
    Unreadable(String),
    Refused(String),
    TooLarge(String),
}

fn image_mime_from_path(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_string_lossy().to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "avif" => Some("image/avif"),
        "heic" | "heif" => Some("image/heic"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

fn is_within(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = fs::canonicalize(root) else {
        return false;
    };
    let Ok(candidate) = fs::canonicalize(candidate) else {
        return false;
    };
    candidate != root && candidate.starts_with(root)
}

fn allowed_host_path(roots: &[PathBuf; 2], path: &Path) -> bool {
    roots.iter().any(|root| is_within(root, path))
}

fn format_too_large_notice(path: &str) -> String {
    format!(""{path}" is too large to attach (max 25 MB).")
}

pub fn load_cloud_agent_images(
    urls: &[String],
    agent_dir: &Path,
    mut read_box_file: Option<&mut dyn FnMut(&str) -> Result<Vec<u8>, String>>,
) -> Result<Vec<CloudAgentImage>, CloudAgentImagesError> {
    let roots = get_agent_media_store_roots(agent_dir);
    let mut images = Vec::new();

    for raw in urls {
        let parsed = Url::parse(raw)
            .ok()
            .filter(|url| url.scheme() == "file")
            .ok_or_else(|| CloudAgentImagesError::NotFileUrl(format!(
                "'{raw}' is not a file:// url. Attach an image the cloud agent can be shown by passing an absolute file:// path."
            )))?;

        let path = parsed
            .to_file_path()
            .map_err(|_| CloudAgentImagesError::Unreadable(raw.clone()))?;
        let mime = image_mime_from_path(&path).ok_or_else(|| {
            CloudAgentImagesError::NotImage(format!(
                "'{raw}' is not a recognized image. Only images ride the cloud agent's vision channel."
            ))
        })?;

        if allowed_host_path(&roots, &path) {
            let metadata = fs::metadata(&path)
                .map_err(|_| CloudAgentImagesError::Unreadable(raw.clone()))?;
            if !metadata.is_file() {
                return Err(CloudAgentImagesError::Unreadable(raw.clone()));
            }
            if metadata.len() > ATTACHMENT_BYTE_LIMIT {
                return Err(CloudAgentImagesError::TooLarge(format_too_large_notice(raw)));
            }
            let data = fs::read(&path)
                .map_err(|_| CloudAgentImagesError::Unreadable(raw.clone()))?;
            images.push(CloudAgentImage {
                data,
                path: path.to_string_lossy().into_owned(),
                mime_type: mime.into(),
            });
            continue;
        }

        let box_path = path.to_string_lossy().replace('\\', "/");
        let in_workspace = box_path == SAND_BOX_WORKSPACE_ROOT
            || box_path.starts_with(&format!("{SAND_BOX_WORKSPACE_ROOT}/"));
        if !in_workspace {
            return Err(CloudAgentImagesError::Refused(format!(
                "Refused to read '{raw}': it is neither inside your own attachments/assets folder nor under /workspace in your box."
            )));
        }

        let Some(reader) = read_box_file.as_deref_mut() else {
            return Err(CloudAgentImagesError::Unreadable(raw.clone()));
        };
        let data = reader(&box_path)
            .map_err(|_| CloudAgentImagesError::Unreadable(raw.clone()))?;
        if data.len() as u64 > ATTACHMENT_BYTE_LIMIT {
            return Err(CloudAgentImagesError::TooLarge(format_too_large_notice(raw)));
        }
        images.push(CloudAgentImage {
            data,
            path: box_path,
            mime_type: mime.into(),
        });
    }

    Ok(images)
}
