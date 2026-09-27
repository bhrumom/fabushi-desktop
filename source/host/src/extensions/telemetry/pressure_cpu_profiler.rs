use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
