use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

pub const SAND_BOX_STORE_MULTIPART_THRESHOLD_BYTES: u64 = 64 * 1024 * 1024;
pub const SAND_BOX_STORE_MULTIPART_PART_SIZE_BYTES: u64 = 128 * 1024 * 1024;
pub const S3_MAX_SINGLE_PUT_BYTES: u64 = 5 * 1024 * 1024 * 1024;
pub const MULTIPART_COMPLETE_MAX_ATTEMPTS: usize = 3;
pub const READ_PRESIGN_BATCH_MAX: usize = 500;
pub const WRITE_PRESIGN_BATCH_MAX: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultipartPart {
    pub part_number: usize,
    pub offset_bytes: u64,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultipartPlan {
    pub parts: Vec<MultipartPart>,
    pub whole_sha256: String,
}

pub fn plan_sand_box_store_multipart_parts(
    src_path: &Path,
    part_size_bytes: u64,
) -> Result<MultipartPlan, String> {
    let bytes = fs::read(src_path).map_err(|error| error.to_string())?;
    plan_sand_box_store_multipart_parts_from_bytes(&bytes, part_size_bytes)
}

pub fn plan_sand_box_store_multipart_parts_from_bytes(
    bytes: &[u8],
    part_size_bytes: u64,
) -> Result<MultipartPlan, String> {
    if bytes.is_empty() {
        return Err("multipart plan requires a positive size".into());
    }
    let part_size = usize::try_from(part_size_bytes)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "multipart plan requires a positive part size".to_string())?;

    let mut parts = Vec::new();
    for (index, chunk) in bytes.chunks(part_size).enumerate() {
        parts.push(MultipartPart {
            part_number: index + 1,
            offset_bytes: (index * part_size) as u64,
            size_bytes: chunk.len() as u64,
            sha256: sha256_hex(chunk),
        });
    }
    Ok(MultipartPlan {
        parts,
        whole_sha256: sha256_hex(bytes),
    })
}

pub fn should_use_multipart(size_bytes: u64, threshold_bytes: u64) -> bool {
    size_bytes >= threshold_bytes || size_bytes > S3_MAX_SINGLE_PUT_BYTES
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}
