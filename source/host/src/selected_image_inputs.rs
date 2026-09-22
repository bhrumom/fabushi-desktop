use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedImageInput {
    pub data: Vec<u8>,
    pub path: String,
    pub mime_type: Option<&'static str>,
}

pub fn image_mime_from_path(file_path: &str) -> Option<&'static str> {
    let base = file_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_path);
    let dot = base.rfind('.')?;
    if dot == 0 {
        return None;
    }
    match base[dot..].to_ascii_lowercase().as_str() {
        ".avif" => Some("image/avif"),
        ".bmp" => Some("image/bmp"),
        ".gif" => Some("image/gif"),
        ".ico" => Some("image/x-icon"),
        ".jpeg" | ".jpg" => Some("image/jpeg"),
        ".png" => Some("image/png"),
        ".svg" => Some("image/svg+xml"),
        ".webp" => Some("image/webp"),
        _ => None,
    }
}

pub fn load_selected_image_inputs<I, S>(attachment_paths: I) -> Vec<SelectedImageInput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    attachment_paths
        .into_iter()
        .filter_map(|path| {
            let path = path.as_ref();
            let data = fs::read(path).ok()?;
            Some(SelectedImageInput {
                data,
                path: path.to_string(),
                mime_type: image_mime_from_path(path),
            })
        })
        .collect()
}
