use std::path::{Path, PathBuf};

pub const SNAPSHOT_OUT_LARGE_CONCURRENCY: usize = 2;
pub const BOX_STORE_SNAPSHOT_TMP_SUFFIX: &str = ".box-store-snap-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOutcome {
    Uploaded,
    Unchanged,
    Oversize,
    MetadataError,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransferSummary {
    pub files_uploaded: usize,
    pub bytes_uploaded: u64,
    pub skipped_unchanged: usize,
    pub oversize: usize,
    pub failures: usize,
    pub metadata_failures: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CategoryTransferSummary {
    pub name: String,
    pub files_scanned: usize,
    pub files_uploaded: usize,
    pub bytes_uploaded: u64,
    pub skipped_unchanged: usize,
    pub removed: usize,
    pub oversize: usize,
    pub failures: usize,
    pub metadata_failures: usize,
    pub excluded_files: usize,
    pub excluded_bytes: u64,
    pub pruned_dirs: usize,
}

pub fn tally_file_outcome(summary: &mut TransferSummary, outcome: FileOutcome, size: u64) {
    match outcome {
        FileOutcome::Uploaded => {
            summary.files_uploaded += 1;
            summary.bytes_uploaded = summary.bytes_uploaded.saturating_add(size);
        }
        FileOutcome::Unchanged => summary.skipped_unchanged += 1,
        FileOutcome::Oversize => summary.oversize += 1,
        FileOutcome::MetadataError => {
            summary.failures += 1;
            summary.metadata_failures += 1;
        }
        FileOutcome::Error => summary.failures += 1,
    }
}

pub fn tally_metadata_failure(summary: &mut TransferSummary) {
    summary.failures += 1;
    summary.metadata_failures += 1;
}

pub fn dedupe_nested_roots<I, P>(roots: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut unique = Vec::<PathBuf>::new();
    for root in roots {
        let root = root.as_ref().to_path_buf();
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
        .iter()
        .filter(|root| {
            !unique
                .iter()
                .any(|other| other != *root && root.starts_with(other))
        })
        .cloned()
        .collect()
}

pub fn glob_matches_path(glob: &str, path: &str) -> bool {
    let glob = glob.trim_start_matches('/');
    let path = path.trim_start_matches('/');
    glob_match(glob.as_bytes(), path.as_bytes())
}

fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if pattern.starts_with(b"**") {
        let rest = &pattern[2..];
        return (0..=text.len()).any(|index| glob_match(rest, &text[index..]));
    }
    if pattern[0] == b'*' {
        let rest = &pattern[1..];
        return (0..=text.len())
            .take_while(|index| *index == 0 || text[*index - 1] != b'/')
            .any(|index| glob_match(rest, &text[index..]));
    }
    if text.is_empty() || pattern[0] != text[0] {
        return false;
    }
    glob_match(&pattern[1..], &text[1..])
}
