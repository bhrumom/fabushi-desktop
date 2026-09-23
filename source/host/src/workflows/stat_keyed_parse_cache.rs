use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_STAT_PARSE_CACHE_CAPACITY: usize = 2_048;
pub const RACY_MTIME_TICK_WINDOW_MS: u128 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint { stat_key: String, newest_mtime_ms: u128 }

pub fn mtime_tick_could_still_hide_an_edit(newest_mtime_ms: u128, now_ms: u128) -> bool {
    now_ms.saturating_sub(newest_mtime_ms) < RACY_MTIME_TICK_WINDOW_MS
}
fn modified_ns(metadata: &fs::Metadata) -> u128 {
    metadata.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or_default()
}
#[cfg(unix)]
fn file_identity(metadata: &fs::Metadata) -> u64 { use std::os::unix::fs::MetadataExt; metadata.ino() }
#[cfg(not(unix))]
fn file_identity(_metadata: &fs::Metadata) -> u64 { 0 }
fn fingerprint_of(paths: &[PathBuf]) -> Option<Fingerprint> {
    let mut parts=Vec::with_capacity(paths.len()); let mut newest=0u128;
    for path in paths { let metadata=fs::metadata(path).ok()?; let ns=modified_ns(&metadata); newest=newest.max(ns/1_000_000); parts.push(format!("{}:{}:{}:{}",path.display(),file_identity(&metadata),ns,metadata.len())); }
    Some(Fingerprint{stat_key:parts.join("|"),newest_mtime_ms:newest})
}
struct CacheEntry<T>{key:String,stat_key:String,value:T}
pub struct StatKeyedParseCache<T:Clone>{capacity:usize,entries:Mutex<VecDeque<CacheEntry<T>>>}
impl<T:Clone> Default for StatKeyedParseCache<T>{fn default()->Self{Self::new(DEFAULT_STAT_PARSE_CACHE_CAPACITY)}}
impl<T:Clone> StatKeyedParseCache<T>{
    pub fn new(capacity:usize)->Self{Self{capacity:capacity.max(1),entries:Mutex::new(VecDeque::new())}}
    pub fn read<F>(&self,stat_paths:&[PathBuf],parse:F)->Option<T> where F:FnOnce()->T{
        let key=stat_paths.iter().map(|p|p.to_string_lossy()).collect::<Vec<_>>().join("\0");
        let Some(fp)=fingerprint_of(stat_paths) else {if let Ok(mut e)=self.entries.lock(){e.retain(|x|x.key!=key);}return None;};
        if let Ok(e)=self.entries.lock(){if let Some(hit)=e.iter().find(|x|x.key==key&&x.stat_key==fp.stat_key){return Some(hit.value.clone());}}
        let value=parse(); let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        if mtime_tick_could_still_hide_an_edit(fp.newest_mtime_ms,now){return Some(value);}
        if let Ok(mut e)=self.entries.lock(){e.retain(|x|x.key!=key);while e.len()>=self.capacity{e.pop_front();}e.push_back(CacheEntry{key,stat_key:fp.stat_key,value:value.clone()});}
        Some(value)
    }
}
pub fn stat_paths(paths:impl IntoIterator<Item=impl AsRef<Path>>)->Vec<PathBuf>{paths.into_iter().map(|p|p.as_ref().to_path_buf()).collect()}
