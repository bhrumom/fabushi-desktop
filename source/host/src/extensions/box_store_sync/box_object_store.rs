use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::agent_store_sand_files::normalize_rel_path;

pub trait BoxObjectStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    fn put(&self, key: &str, bytes: &[u8]) -> Result<(), String>;
    fn get_to_file(
        &self,
        key: &str,
        dest_path: &Path,
        max_bytes: Option<u64>,
    ) -> Result<Option<u64>, String>;
    fn put_from_file(&self, key: &str, src_path: &Path) -> Result<(), String>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}

pub trait BoxObjectStoreProvider: Send + Sync {
    fn for_store(&self, store_id: &str) -> Box<dyn BoxObjectStore>;
}

#[derive(Debug, Clone)]
pub struct LocalFsObjectStoreProvider {
    base_dir: PathBuf,
}

impl LocalFsObjectStoreProvider {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }
}

impl BoxObjectStoreProvider for LocalFsObjectStoreProvider {
    fn for_store(&self, store_id: &str) -> Box<dyn BoxObjectStore> {
        let normalized = normalize_rel_path(store_id)
            .unwrap_or_else(|_| "invalid-store-id".to_string());
        Box::new(LocalFsObjectStore::new(self.base_dir.join(normalized)))
    }
}

#[derive(Debug, Clone)]
pub struct LocalFsObjectStore {
    root: PathBuf,
}

impl LocalFsObjectStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, key: &str) -> Result<PathBuf, String> {
        let normalized = normalize_rel_path(key)?;
        Ok(self.root.join(normalized))
    }
}

impl BoxObjectStore for LocalFsObjectStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        match fs::read(self.path_for(key)?) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<(), String> {
        let path = self.path_for(key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, bytes).map_err(|error| error.to_string())
    }

    fn get_to_file(
        &self,
        key: &str,
        dest_path: &Path,
        max_bytes: Option<u64>,
    ) -> Result<Option<u64>, String> {
        let src = self.path_for(key)?;
        let metadata = match fs::metadata(&src) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        if max_bytes.is_some_and(|max| metadata.len() > max) {
            return Err(format!(
                "object {} exceeds maximum restore bytes",
                metadata.len()
            ));
        }
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(src, dest_path)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn put_from_file(&self, key: &str, src_path: &Path) -> Result<(), String> {
        let dest = self.path_for(key)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(src_path, dest)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, String> {
        let normalized_prefix = if prefix.trim().is_empty() {
            String::new()
        } else {
            normalize_rel_path(prefix)?
        };
        let mut files = Vec::new();
        if !self.root.exists() {
            return Ok(files);
        }
        walk_files(&self.root, &self.root, &mut files).map_err(|error| error.to_string())?;
        files.retain(|key| is_under_prefix(key, &normalized_prefix));
        files.sort();
        Ok(files)
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        match fs::remove_file(self.path_for(key)?) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub fn is_under_prefix(key: &str, prefix: &str) -> bool {
    prefix.is_empty() || key == prefix || key.starts_with(&format!("{prefix}/"))
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_files(root, &path, out)?;
        } else if file_type.is_file() {
            if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}
