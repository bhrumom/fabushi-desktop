use std::collections::{BTreeMap, HashMap, HashSet};

use super::box_store_manifest_format::BoxStoreManifestEntry;

pub const AGENT_STORE_DB_BASENAMES: [&str; 2] = ["store.db", "conversation-blobs.db"];
pub type BoxManifestMap = BTreeMap<String, BoxStoreManifestEntry>;

pub fn count_store_db_manifest_entries(manifest: Option<&BoxManifestMap>) -> usize {
    manifest
        .into_iter()
        .flat_map(|manifest| manifest.iter())
        .filter(|(path, entry)| {
            extract_store_db_agent_id(path).is_some() && !entry.is_symlink()
        })
        .count()
}

pub fn get_store_db_manifest_agent_ids(manifest: Option<&BoxManifestMap>) -> HashSet<String> {
    manifest
        .into_iter()
        .flat_map(|manifest| manifest.iter())
        .filter_map(|(path, entry)| {
            (!entry.is_symlink())
                .then(|| extract_store_db_agent_id(path))
                .flatten()
                .map(str::to_string)
        })
        .collect()
}

pub fn count_agent_dir_manifest_entries(manifest: Option<&BoxManifestMap>) -> usize {
    manifest
        .into_iter()
        .flat_map(|manifest| manifest.keys())
        .filter_map(|path| extract_agent_dir_id(path).map(str::to_string))
        .collect::<HashSet<_>>()
        .len()
}

fn extract_store_db_agent_id(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("home/box/sand-data/agents/")?;
    let (agent_id, basename) = rest.split_once('/')?;
    (basename == "store.db" && !agent_id.is_empty()).then_some(agent_id)
}

fn extract_agent_dir_id(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("home/box/sand-data/agents/")?;
    let (agent_id, _) = rest.split_once('/')?;
    (!agent_id.is_empty()).then_some(agent_id)
}

pub fn has_manifest_path_conflict<'a>(paths: impl IntoIterator<Item = &'a str>) -> bool {
    let paths = paths.into_iter().collect::<HashSet<_>>();
    for path in &paths {
        for (separator, _) in path.match_indices('/') {
            if paths.contains(&path[..separator]) {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCapture {
    pub revision: u64,
    pub entries: BoxManifestMap,
}

#[derive(Debug, Default)]
pub struct BoxStoreManifestRevisionState {
    revision: u64,
    persisted: Option<(u64, Option<bool>)>,
    pending_revision_counts: HashMap<u64, usize>,
    construction_count: u64,
}

impl BoxStoreManifestRevisionState {
    pub fn with_persisted(revision: u64, fully_hydrated: Option<bool>) -> Self {
        Self {
            persisted: Some((revision, fully_hydrated)),
            ..Self::default()
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn persisted_revision(&self) -> Option<u64> {
        self.persisted.map(|value| value.0)
    }

    pub fn snapshot_construction_count(&self) -> u64 {
        self.construction_count
    }

    pub fn has_pending_write_for_revision(&self) -> bool {
        self.pending_revision_counts
            .get(&self.revision)
            .copied()
            .unwrap_or_default()
            > 0
    }

    pub fn mark_changed(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn capture(
        &mut self,
        manifest: &BoxManifestMap,
        fully_hydrated: Option<bool>,
        is_required_write: bool,
    ) -> Option<ManifestCapture> {
        let pending = self
            .pending_revision_counts
            .get(&self.revision)
            .copied()
            .unwrap_or_default();
        if !is_required_write
            && (pending > 0 || self.persisted == Some((self.revision, fully_hydrated)))
        {
            return None;
        }
        self.construction_count = self.construction_count.saturating_add(1);
        *self
            .pending_revision_counts
            .entry(self.revision)
            .or_default() += 1;
        Some(ManifestCapture {
            revision: self.revision,
            entries: manifest.clone(),
        })
    }

    pub fn mark_persisted(&mut self, capture: &ManifestCapture, fully_hydrated: Option<bool>) {
        self.persisted = Some((capture.revision, fully_hydrated));
    }

    pub fn mark_failed(&mut self) {
        self.persisted = None;
    }

    pub fn release(&mut self, capture: &ManifestCapture) {
        let Some(count) = self.pending_revision_counts.get_mut(&capture.revision) else {
            return;
        };
        if *count <= 1 {
            self.pending_revision_counts.remove(&capture.revision);
        } else {
            *count -= 1;
        }
    }
}

pub fn set_manifest_entry(
    manifest: &mut BoxManifestMap,
    rel_path: &str,
    entry: BoxStoreManifestEntry,
) -> bool {
    let mut changed = false;
    for ancestor in ancestor_paths(rel_path) {
        changed |= manifest.remove(&ancestor).is_some();
    }
    let prefix = format!("{rel_path}/");
    let descendants = manifest
        .keys()
        .filter(|path| path.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    for descendant in descendants {
        changed |= manifest.remove(&descendant).is_some();
    }
    if manifest.get(rel_path) == Some(&entry) {
        return changed;
    }
    manifest.insert(rel_path.to_string(), entry);
    true
}

fn ancestor_paths(rel_path: &str) -> Vec<String> {
    rel_path
        .match_indices('/')
        .map(|(index, _)| rel_path[..index].to_string())
        .collect()
}
