use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, RecvTimeoutError},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::agents::agent_avatar::{
    invalidate_avatar_data_url_cache, is_conventional_avatar_filename,
};
use crate::agents::agent_profile::{
    SAND_PROFILE_FILENAME, get_sand_profile_path, read_sand_profile_file,
};
use crate::agents::settings_file::{
    SAND_SETTINGS_FILENAME, get_sand_settings_path,
};
use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::session::session_paths::ACTIVE_AGENT_FILENAME;

use super::roster_emit::ProductionRosterEmit;

pub const PROFILE_WATCH_DEBOUNCE_MS: u64 = 50;
pub const SAND_DEFAULT_AGENT_NAME: &str = "Grok";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDisplayProfile {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAgentProfile {
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
    pub settings_file_path: PathBuf,
}

pub struct ProductionProfileWatch {
    _watcher: RecommendedWatcher,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    watched_agent_id: Arc<Mutex<Option<String>>>,
}

impl ProductionProfileWatch {
    pub fn start(
        sessions: Arc<ProductionSessionWorkers>,
        roster_emit: Arc<ProductionRosterEmit>,
    ) -> notify::Result<Self> {
        let agents_root = sessions.agents_root().to_path_buf();
        let _ = fs::create_dir_all(&agents_root);
        let (event_tx, event_rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = event_tx.send(event);
        })?;
        watcher.watch(&agents_root, RecursiveMode::Recursive)?;

        let watched_agent_id = Arc::new(Mutex::new(
            SandAgentSessionStore::new(Arc::clone(&sessions)).read_active_agent_id(),
        ));
        let last_known_names = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        if let Some(agent_id) = watched_agent_id
            .lock()
            .ok()
            .and_then(|value| value.clone())
        {
            seed_known_agent_name(&sessions, &last_known_names, &agent_id);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_watched = Arc::clone(&watched_agent_id);
        let worker_names = Arc::clone(&last_known_names);
        let worker_sessions = Arc::clone(&sessions);
        let worker_roster = Arc::clone(&roster_emit);
        let worker_root = agents_root.clone();

        let worker = thread::spawn(move || {
            let mut pending = HashSet::<String>::new();
            while !worker_stop.load(Ordering::Acquire) {
                match event_rx.recv_timeout(Duration::from_millis(PROFILE_WATCH_DEBOUNCE_MS)) {
                    Ok(Ok(event)) => {
                        for path in event.paths {
                            if path == worker_root.join(ACTIVE_AGENT_FILENAME) {
                                let active = SandAgentSessionStore::new(Arc::clone(&worker_sessions))
                                    .read_active_agent_id();
                                if let Ok(mut watched) = worker_watched.lock() {
                                    *watched = active.clone();
                                }
                                if let Some(agent_id) = active {
                                    seed_known_agent_name(
                                        &worker_sessions,
                                        &worker_names,
                                        &agent_id,
                                    );
                                }
                                continue;
                            }
                            let watched = worker_watched
                                .lock()
                                .ok()
                                .and_then(|value| value.clone());
                            let Some(watched) = watched else {
                                continue;
                            };
                            if watched_profile_path_agent_id(&worker_root, &path).as_deref()
                                == Some(watched.as_str())
                            {
                                pending.insert(watched);
                            }
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(RecvTimeoutError::Timeout) => {
                        if pending.is_empty() {
                            continue;
                        }
                        let pending_agents = pending.drain().collect::<Vec<_>>();
                        for agent_id in pending_agents {
                            let agent_dir = worker_root.join(&agent_id);
                            invalidate_avatar_data_url_cache(&agent_dir);
                            let _ = worker_roster.emit_agent_update(&agent_id);
                            record_name_change(
                                &worker_sessions,
                                &worker_names,
                                &worker_roster,
                                &agent_id,
                            );
                            worker_roster.publish_profile_changed(&agent_id);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            stop,
            worker: Some(worker),
            watched_agent_id,
        })
    }

    pub fn watched_agent_id(&self) -> Option<String> {
        self.watched_agent_id
            .lock()
            .ok()
            .and_then(|value| value.clone())
    }
}

impl Drop for ProductionProfileWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn is_profile_watch_filename(name: &str) -> bool {
    name == SAND_PROFILE_FILENAME
        || name == SAND_SETTINGS_FILENAME
        || is_conventional_avatar_filename(name)
}

pub fn watched_profile_path_agent_id(agents_root: &Path, path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if !is_profile_watch_filename(name) {
        return None;
    }
    let agent_dir = path.parent()?;
    if agent_dir.parent()? != agents_root {
        return None;
    }
    agent_dir.file_name()?.to_str().map(ToOwned::to_owned)
}

pub fn get_agent_display_profile(
    sessions: &Arc<ProductionSessionWorkers>,
    agent_id: &str,
) -> Option<AgentDisplayProfile> {
    let dir = sessions.agents_root().join(agent_id);
    if !dir.is_dir() {
        return None;
    }
    let profile = read_sand_profile_file(get_sand_profile_path(&dir));
    Some(AgentDisplayProfile {
        name: profile
            .as_ref()
            .map(|profile| profile.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or(SAND_DEFAULT_AGENT_NAME)
            .to_string(),
        description: profile
            .map(|profile| profile.description)
            .unwrap_or_default(),
    })
}

pub fn resolve_agent_profile(db_path: &Path) -> ResolvedAgentProfile {
    let dir = db_path.parent().unwrap_or_else(|| Path::new("."));
    let file_path = get_sand_profile_path(dir);
    let profile = read_sand_profile_file(&file_path);
    ResolvedAgentProfile {
        name: profile
            .as_ref()
            .map(|profile| profile.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or(SAND_DEFAULT_AGENT_NAME)
            .to_string(),
        description: profile
            .map(|profile| profile.description)
            .unwrap_or_default(),
        file_path,
        settings_file_path: get_sand_settings_path(dir),
    }
}

fn seed_known_agent_name(
    sessions: &Arc<ProductionSessionWorkers>,
    names: &Arc<Mutex<HashMap<String, String>>>,
    agent_id: &str,
) {
    let Some(profile) = get_agent_display_profile(sessions, agent_id) else {
        return;
    };
    if let Ok(mut names) = names.lock() {
        names.entry(agent_id.to_string()).or_insert(profile.name);
    }
}

fn record_name_change(
    sessions: &Arc<ProductionSessionWorkers>,
    names: &Arc<Mutex<HashMap<String, String>>>,
    roster_emit: &Arc<ProductionRosterEmit>,
    agent_id: &str,
) {
    let Some(profile) = get_agent_display_profile(sessions, agent_id) else {
        return;
    };
    if profile.name.trim().is_empty() {
        return;
    }
    let previous = names
        .lock()
        .ok()
        .and_then(|mut names| names.insert(agent_id.to_string(), profile.name.clone()));
    if let Some(previous) = previous {
        if previous != profile.name {
            roster_emit.publish_name_changed(agent_id, &previous, &profile.name);
        }
    }
}
