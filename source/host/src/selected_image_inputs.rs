use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaDimensions {
    pub width: u32,
    pub height: u32,
}

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

pub fn video_mime_from_path(file_path: &str) -> Option<&'static str> {
    let base = file_path.rsplit(['/', '\\']).next().unwrap_or(file_path);
    let dot = base.rfind('.')?;
    if dot == 0 {
        return None;
    }
    match base[dot..].to_ascii_lowercase().as_str() {
        ".m4v" | ".mp4" => Some("video/mp4"),
        ".mov" => Some("video/quicktime"),
        ".ogv" => Some("video/ogg"),
        ".webm" => Some("video/webm"),
        _ => None,
    }
}

pub fn read_image_file_dimensions(path: &str) -> Option<MediaDimensions> {
    let bytes = fs::read(path).ok()?;
    read_image_dimensions(&bytes)
}

pub fn read_image_dimensions(bytes: &[u8]) -> Option<MediaDimensions> {
    read_webp_dimensions(bytes)
        .or_else(|| read_png_dimensions(bytes))
        .or_else(|| read_gif_dimensions(bytes))
        .or_else(|| read_jpeg_dimensions(bytes))
}

fn dimensions(width: u32, height: u32) -> Option<MediaDimensions> {
    (width > 0 && height > 0).then_some(MediaDimensions { width, height })
}

fn read_u16_be(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*bytes.get(offset)?, *bytes.get(offset + 1)?]))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*bytes.get(offset)?, *bytes.get(offset + 1)?]))
}

fn read_u24_le(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(
        u32::from(*bytes.get(offset)?)
            | (u32::from(*bytes.get(offset + 1)?) << 8)
            | (u32::from(*bytes.get(offset + 2)?) << 16),
    )
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}

fn tag(bytes: &[u8], offset: usize) -> Option<&[u8]> {
    bytes.get(offset..offset + 4)
}

fn read_png_dimensions(bytes: &[u8]) -> Option<MediaDimensions> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || bytes.get(..8)? != SIGNATURE || tag(bytes, 12)? != b"IHDR" {
        return None;
    }
    dimensions(read_u32_be(bytes, 16)?, read_u32_be(bytes, 20)?)
}

fn read_gif_dimensions(bytes: &[u8]) -> Option<MediaDimensions> {
    if bytes.len() < 10
        || !matches!(bytes.get(..6), Some(b"GIF87a") | Some(b"GIF89a"))
    {
        return None;
    }
    dimensions(
        u32::from(read_u16_le(bytes, 6)?),
        u32::from(read_u16_le(bytes, 8)?),
    )
}

fn read_jpeg_dimensions(bytes: &[u8]) -> Option<MediaDimensions> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut offset = 2usize;
    while offset + 3 < bytes.len() {
        if bytes[offset] != 0xff {
            offset += 1;
            continue;
        }
        let marker = bytes[offset + 1];
        if marker == 0xff {
            offset += 1;
            continue;
        }
        if matches!(marker, 0x01 | 0xd0..=0xd9) {
            offset += 2;
            continue;
        }
        if marker == 0xda {
            return None;
        }
        let segment_len = usize::from(read_u16_be(bytes, offset + 2)?);
        if segment_len < 2 {
            return None;
        }
        let frame = (0xc0..=0xcf).contains(&marker)
            && !matches!(marker, 0xc4 | 0xc8 | 0xcc);
        if frame {
            if offset + 9 > bytes.len() {
                return None;
            }
            return dimensions(
                u32::from(read_u16_be(bytes, offset + 7)?),
                u32::from(read_u16_be(bytes, offset + 5)?),
            );
        }
        offset = offset.checked_add(2 + segment_len)?;
    }
    None
}

fn read_webp_dimensions(bytes: &[u8]) -> Option<MediaDimensions> {
    if bytes.len() < 30 || tag(bytes, 0)? != b"RIFF" || tag(bytes, 8)? != b"WEBP" {
        return None;
    }
    match tag(bytes, 12)? {
        b"VP8 " => dimensions(
            u32::from(read_u16_le(bytes, 26)? & 0x3fff),
            u32::from(read_u16_le(bytes, 28)? & 0x3fff),
        ),
        b"VP8L" => {
            let packed = read_u32_le(bytes, 21)?;
            dimensions((packed & 0x3fff) + 1, ((packed >> 14) & 0x3fff) + 1)
        }
        b"VP8X" => dimensions(
            read_u24_le(bytes, 24)? + 1,
            read_u24_le(bytes, 27)? + 1,
        ),
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
