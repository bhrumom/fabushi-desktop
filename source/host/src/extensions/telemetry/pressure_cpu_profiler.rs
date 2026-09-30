use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::extensions::experiments::HostExperimentsExtension;

pub const DEFAULT_PROFILE_DURATION_MS: i64 = 15_000;
pub const DEFAULT_MIN_INTERVAL_MS: i64 = 6 * 60 * 60_000;
pub const DEFAULT_MAX_RETAINED_PROFILES: usize = 3;
pub const DEFAULT_SUSTAINED_PRESSURE_WINDOW_MS: i64 = 150_000;
pub const PROFILER_SAMPLING_INTERVAL_US: u64 = 10_000;
pub const PRESSURE_CPU_PROFILE_PREFIX: &str = "sand-host-pressure-";
pub const PRESSURE_CPU_PROFILE_DIR: &str = "/tmp/sand-host-profiles";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct SandProfilerCaptureError(pub String);

pub trait CpuProfilerBackend: Send + Sync {
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<String, String>;
    fn dispose(&self);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PressureCpuProfilerKnobs {
    pub sustained_pressure_window_ms: Option<i64>,
    pub profile_duration_ms: Option<i64>,
    pub min_interval_ms: Option<i64>,
    pub max_retained_profiles: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EffectiveKnobs {
    sustained_pressure_window_ms: i64,
    profile_duration_ms: i64,
    min_interval_ms: i64,
    max_retained_profiles: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureState {
    Idle,
    Starting,
    Profiling,
    Stopping,
}

pub struct PressureCpuProfiler {
    directory: PathBuf,
    backend: Arc<dyn CpuProfilerBackend>,
    configured: PressureCpuProfilerKnobs,
    overrides: Option<Arc<dyn Fn() -> Option<PressureCpuProfilerKnobs> + Send + Sync>>,
    on_captured: Option<Arc<dyn Fn(&Path) + Send + Sync>>,
    state: CaptureState,
    capture_deadline_ms: i64,
    last_capture_started_at_ms: Option<i64>,
    previous_pressure_at_ms: Option<i64>,
    disposed: bool,
}

impl PressureCpuProfiler {
    pub fn new(
        directory: impl Into<PathBuf>,
        backend: Arc<dyn CpuProfilerBackend>,
        configured: PressureCpuProfilerKnobs,
    ) -> Self {
        Self {
            directory: directory.into(),
            backend,
            configured,
            overrides: None,
            on_captured: None,
            state: CaptureState::Idle,
            capture_deadline_ms: 0,
            last_capture_started_at_ms: None,
            previous_pressure_at_ms: None,
            disposed: false,
        }
    }

    pub fn with_overrides(
        mut self,
        overrides: Arc<dyn Fn() -> Option<PressureCpuProfilerKnobs> + Send + Sync>,
    ) -> Self {
        self.overrides = Some(overrides);
        self
    }

    pub fn with_on_captured(mut self, listener: Arc<dyn Fn(&Path) + Send + Sync>) -> Self {
        self.on_captured = Some(listener);
        self
    }

    fn knobs(&self) -> EffectiveKnobs {
        let live = self
            .overrides
            .as_ref()
            .and_then(|overrides| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| overrides())).ok()
            })
            .flatten();
        let pick_i64 = |live: Option<i64>, configured: Option<i64>, default: i64| {
            live.or(configured).unwrap_or(default).max(0)
        };
        EffectiveKnobs {
            sustained_pressure_window_ms: pick_i64(
                live.and_then(|value| value.sustained_pressure_window_ms),
                self.configured.sustained_pressure_window_ms,
                DEFAULT_SUSTAINED_PRESSURE_WINDOW_MS,
            ),
            profile_duration_ms: pick_i64(
                live.and_then(|value| value.profile_duration_ms),
                self.configured.profile_duration_ms,
                DEFAULT_PROFILE_DURATION_MS,
            ),
            min_interval_ms: pick_i64(
                live.and_then(|value| value.min_interval_ms),
                self.configured.min_interval_ms,
                DEFAULT_MIN_INTERVAL_MS,
            ),
            max_retained_profiles: live
                .and_then(|value| value.max_retained_profiles)
                .or(self.configured.max_retained_profiles)
                .unwrap_or(DEFAULT_MAX_RETAINED_PROFILES)
                .max(1),
        }
    }

    pub fn on_pressure(&mut self, now_ms: i64) {
        if self.disposed || self.state != CaptureState::Idle {
            return;
        }
        let effective = self.knobs();
        if self
            .last_capture_started_at_ms
            .is_some_and(|last| now_ms.saturating_sub(last) < effective.min_interval_ms)
        {
            return;
        }

        let sustained = self.previous_pressure_at_ms.is_some_and(|previous| {
            now_ms.saturating_sub(previous) <= effective.sustained_pressure_window_ms
        });
        if !sustained {
            self.previous_pressure_at_ms = Some(now_ms);
            return;
        }

        self.previous_pressure_at_ms = None;
        self.state = CaptureState::Starting;
        self.last_capture_started_at_ms = Some(now_ms);
        match self.backend.start() {
            Ok(()) if !self.disposed => {
                self.capture_deadline_ms = now_ms.saturating_add(effective.profile_duration_ms);
                self.state = CaptureState::Profiling;
            }
            Ok(()) => {
                self.backend.dispose();
                self.state = CaptureState::Idle;
            }
            Err(_) => {
                self.state = CaptureState::Idle;
                self.last_capture_started_at_ms = None;
            }
        }
    }

    pub fn on_tick(&mut self, now_ms: i64) -> Result<Option<PathBuf>, SandProfilerCaptureError> {
        if self.disposed
            || self.state != CaptureState::Profiling
            || now_ms < self.capture_deadline_ms
        {
            return Ok(None);
        }
        self.finish_capture(now_ms).map(Some)
    }

    fn finish_capture(&mut self, now_ms: i64) -> Result<PathBuf, SandProfilerCaptureError> {
        self.state = CaptureState::Stopping;
        let result = (|| {
            let profile_json = self.backend.stop().map_err(SandProfilerCaptureError)?;
            if self.disposed {
                return Err(SandProfilerCaptureError(
                    "profiler disposed during capture".into(),
                ));
            }
            fs::create_dir_all(&self.directory)
                .map_err(|error| SandProfilerCaptureError(error.to_string()))?;
            let path = self
                .directory
                .join(format!("{PRESSURE_CPU_PROFILE_PREFIX}{now_ms}.cpuprofile"));
            fs::write(&path, profile_json)
                .map_err(|error| SandProfilerCaptureError(error.to_string()))?;
            self.prune_old_profiles(self.knobs().max_retained_profiles)
                .map_err(|error| SandProfilerCaptureError(error.to_string()))?;
            if let Some(listener) = &self.on_captured {
                listener(&path);
            }
            Ok(path)
        })();
        self.state = CaptureState::Idle;
        result
    }

    fn prune_old_profiles(&self, max: usize) -> std::io::Result<()> {
        let mut profiles = fs::read_dir(&self.directory)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(PRESSURE_CPU_PROFILE_PREFIX))
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        profiles.sort();
        let remove_count = profiles.len().saturating_sub(max);
        for path in profiles.into_iter().take(remove_count) {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }

    pub fn dispose(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
        if matches!(self.state, CaptureState::Starting | CaptureState::Profiling) {
            self.backend.dispose();
        }
        self.state = CaptureState::Idle;
    }

    pub fn is_profiling(&self) -> bool {
        self.state == CaptureState::Profiling
    }

    pub fn last_capture_started_at_ms(&self) -> Option<i64> {
        self.last_capture_started_at_ms
    }
}


struct ProductionCpuProfilerState {
    child: Option<Child>,
    capture_path: Option<PathBuf>,
    active: bool,
}

pub struct ProductionCpuProfilerBackend {
    state: Mutex<ProductionCpuProfilerState>,
}

impl Default for ProductionCpuProfilerBackend {
    fn default() -> Self {
        Self {
            state: Mutex::new(ProductionCpuProfilerState {
                child: None,
                capture_path: None,
                active: false,
            }),
        }
    }
}

impl ProductionCpuProfilerBackend {
    fn fresh_capture_path(extension: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sand-host-pressure-source-{}-{}.{}",
            std::process::id(),
            uuid::Uuid::new_v4(),
            extension
        ))
    }

    #[cfg(target_os = "macos")]
    fn start_platform(state: &mut ProductionCpuProfilerState) -> Result<(), String> {
        let path = Self::fresh_capture_path("sample.txt");
        let child = Command::new("/usr/bin/sample")
            .arg(std::process::id().to_string())
            .arg("86400")
            .arg("10")
            .arg("-file")
            .arg(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("could not start macOS sample profiler: {error}"))?;
        state.child = Some(child);
        state.capture_path = Some(path);
        state.active = true;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn start_platform(state: &mut ProductionCpuProfilerState) -> Result<(), String> {
        let path = Self::fresh_capture_path("perf.data");
        let child = Command::new("perf")
            .args(["record", "-F", "99", "-g", "-p"])
            .arg(std::process::id().to_string())
            .arg("-o")
            .arg(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("could not start perf profiler: {error}"))?;
        state.child = Some(child);
        state.capture_path = Some(path);
        state.active = true;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn start_platform(state: &mut ProductionCpuProfilerState) -> Result<(), String> {
        let status = Command::new("wpr.exe")
            .args(["-start", "CPU", "-filemode"])
            .status()
            .map_err(|error| format!("could not start WPR CPU profile: {error}"))?;
        if !status.success() {
            return Err(format!("wpr start failed with status {status}"));
        }
        state.capture_path = Some(Self::fresh_capture_path("etl"));
        state.active = true;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn start_platform(_state: &mut ProductionCpuProfilerState) -> Result<(), String> {
        Err("pressure CPU profiling is unsupported on this platform".into())
    }

    #[cfg(target_os = "macos")]
    fn stop_platform(state: &mut ProductionCpuProfilerState) -> Result<String, String> {
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let path = state
            .capture_path
            .take()
            .ok_or_else(|| "macOS sample output path is missing".to_string())?;
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("could not read macOS sample profile: {error}"))?;
        let _ = fs::remove_file(&path);
        Ok(json!({
            "format": "macos-sample",
            "samplingIntervalUs": PROFILER_SAMPLING_INTERVAL_US,
            "raw": raw,
        })
        .to_string())
    }

    #[cfg(target_os = "linux")]
    fn stop_platform(state: &mut ProductionCpuProfilerState) -> Result<String, String> {
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let path = state
            .capture_path
            .take()
            .ok_or_else(|| "perf output path is missing".to_string())?;
        let output = Command::new("perf")
            .args(["script", "-i"])
            .arg(&path)
            .output()
            .map_err(|error| format!("could not decode perf profile: {error}"))?;
        let _ = fs::remove_file(&path);
        if !output.status.success() {
            return Err(format!("perf script failed with status {}", output.status));
        }
        Ok(json!({
            "format": "linux-perf-script",
            "samplingIntervalUs": PROFILER_SAMPLING_INTERVAL_US,
            "raw": String::from_utf8_lossy(&output.stdout),
        })
        .to_string())
    }

    #[cfg(target_os = "windows")]
    fn stop_platform(state: &mut ProductionCpuProfilerState) -> Result<String, String> {
        let path = state
            .capture_path
            .take()
            .ok_or_else(|| "WPR output path is missing".to_string())?;
        let status = Command::new("wpr.exe")
            .arg("-stop")
            .arg(&path)
            .status()
            .map_err(|error| format!("could not stop WPR CPU profile: {error}"))?;
        if !status.success() {
            return Err(format!("wpr stop failed with status {status}"));
        }
        Ok(json!({
            "format": "windows-wpr-etl",
            "path": path.to_string_lossy(),
            "samplingIntervalUs": PROFILER_SAMPLING_INTERVAL_US,
        })
        .to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn stop_platform(_state: &mut ProductionCpuProfilerState) -> Result<String, String> {
        Err("pressure CPU profiling is unsupported on this platform".into())
    }
}

impl CpuProfilerBackend for ProductionCpuProfilerBackend {
    fn start(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "pressure profiler backend mutex poisoned".to_string())?;
        if state.active {
            return Ok(());
        }
        Self::start_platform(&mut state)
    }

    fn stop(&self) -> Result<String, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "pressure profiler backend mutex poisoned".to_string())?;
        if !state.active {
            return Err("profiler session not started".into());
        }
        let result = Self::stop_platform(&mut state);
        state.active = false;
        result
    }

    fn dispose(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        #[cfg(target_os = "windows")]
        if state.active {
            let _ = Command::new("wpr.exe").arg("-cancel").status();
        }
        if let Some(path) = state.capture_path.take() {
            let _ = fs::remove_file(path);
        }
        state.active = false;
    }
}

fn config_i64(config: &std::collections::BTreeMap<String, Value>, key: &str) -> Option<i64> {
    config.get(key).and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
            .or_else(|| value.as_f64().map(|value| value.round() as i64))
    })
}

fn config_usize(config: &std::collections::BTreeMap<String, Value>, key: &str) -> Option<usize> {
    config_i64(config, key).and_then(|value| usize::try_from(value.max(0)).ok())
}

fn pressure_knobs_from_dynamic_config(
    config: &std::collections::BTreeMap<String, Value>,
) -> PressureCpuProfilerKnobs {
    PressureCpuProfilerKnobs {
        sustained_pressure_window_ms: config_i64(config, "sustainedPressureWindowMs")
            .or_else(|| config_i64(config, "sustained_pressure_window_ms")),
        profile_duration_ms: config_i64(config, "profileDurationMs")
            .or_else(|| config_i64(config, "profile_duration_ms")),
        min_interval_ms: config_i64(config, "minIntervalMs")
            .or_else(|| config_i64(config, "min_interval_ms")),
        max_retained_profiles: config_usize(config, "maxRetainedProfiles")
            .or_else(|| config_usize(config, "max_retained_profiles")),
    }
}

pub fn create_production_pressure_cpu_profiler(
    experiments: Arc<HostExperimentsExtension>,
) -> Arc<Mutex<PressureCpuProfiler>> {
    let directory = if cfg!(windows) {
        std::env::temp_dir().join("sand-host-profiles")
    } else {
        PathBuf::from(PRESSURE_CPU_PROFILE_DIR)
    };
    let experiments_for_overrides = Arc::clone(&experiments);
    let profiler = PressureCpuProfiler::new(
        directory,
        Arc::new(ProductionCpuProfilerBackend::default()),
        PressureCpuProfilerKnobs::default(),
    )
    .with_overrides(Arc::new(move || {
        Some(pressure_knobs_from_dynamic_config(
            &experiments_for_overrides
                .get_dynamic_config("sand_pressure_cpu_profiler_config"),
        ))
    }));
    Arc::new(Mutex::new(profiler))
}
