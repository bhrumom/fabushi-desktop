use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::telemetry::pressure_cpu_profiler::{
    CpuProfilerBackend, PressureCpuProfiler, PressureCpuProfilerKnobs,
    create_production_pressure_cpu_profiler,
};

#[derive(Default)]
struct Backend {
    starts: AtomicUsize,
    stops: AtomicUsize,
    disposes: AtomicUsize,
    profile: Mutex<String>,
}

impl CpuProfilerBackend for Backend {
    fn start(&self) -> Result<(), String> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn stop(&self) -> Result<String, String> {
        self.stops.fetch_add(1, Ordering::SeqCst);
        Ok(self.profile.lock().unwrap().clone())
    }
    fn dispose(&self) {
        self.disposes.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn sustained_pressure_is_required_and_min_interval_is_enforced() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-pressure-profiler-{}",
        uuid::Uuid::new_v4()
    ));
    let backend = Arc::new(Backend::default());
    *backend.profile.lock().unwrap() = "{\"nodes\":[]}".into();
    let mut profiler = PressureCpuProfiler::new(
        &root,
        backend.clone(),
        PressureCpuProfilerKnobs {
            sustained_pressure_window_ms: Some(100),
            profile_duration_ms: Some(50),
            min_interval_ms: Some(1_000),
            max_retained_profiles: Some(2),
        },
    );
    profiler.on_pressure(0);
    assert_eq!(backend.starts.load(Ordering::SeqCst), 0);
    profiler.on_pressure(50);
    assert_eq!(backend.starts.load(Ordering::SeqCst), 1);
    assert!(profiler.is_profiling());
    assert!(profiler.on_tick(99).unwrap().is_none());
    let captured = profiler.on_tick(100).unwrap().unwrap();
    assert!(captured.exists());
    assert_eq!(backend.stops.load(Ordering::SeqCst), 1);

    profiler.on_pressure(200);
    profiler.on_pressure(250);
    assert_eq!(backend.starts.load(Ordering::SeqCst), 1);

    profiler.on_pressure(1_100);
    profiler.on_pressure(1_150);
    assert_eq!(backend.starts.load(Ordering::SeqCst), 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn retention_keeps_latest_profiles_and_dispose_is_idempotent() {
    let root =
        std::env::temp_dir().join(format!("fabushi-pressure-retain-{}", uuid::Uuid::new_v4()));
    let backend = Arc::new(Backend::default());
    *backend.profile.lock().unwrap() = "{}".into();
    let mut profiler = PressureCpuProfiler::new(
        &root,
        backend.clone(),
        PressureCpuProfilerKnobs {
            sustained_pressure_window_ms: Some(100),
            profile_duration_ms: Some(0),
            min_interval_ms: Some(0),
            max_retained_profiles: Some(2),
        },
    );
    for base in [100_i64, 300, 500] {
        profiler.on_pressure(base);
        profiler.on_pressure(base + 1);
        profiler.on_tick(base + 1).unwrap();
    }
    let names = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 2);
    assert!(!names.iter().any(|name| name.contains("101")));
    profiler.on_pressure(1_000);
    profiler.on_pressure(1_001);
    assert!(profiler.is_profiling());
    profiler.dispose();
    profiler.dispose();
    assert_eq!(backend.disposes.load(Ordering::SeqCst), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn live_overrides_are_fail_closed_and_max_retention_is_at_least_one() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-pressure-overrides-{}",
        uuid::Uuid::new_v4()
    ));
    let backend = Arc::new(Backend::default());
    *backend.profile.lock().unwrap() = "{}".into();
    let mut profiler = PressureCpuProfiler::new(
        &root,
        backend,
        PressureCpuProfilerKnobs {
            sustained_pressure_window_ms: Some(100),
            profile_duration_ms: Some(0),
            min_interval_ms: Some(0),
            max_retained_profiles: Some(3),
        },
    )
    .with_overrides(Arc::new(|| {
        Some(PressureCpuProfilerKnobs {
            max_retained_profiles: Some(0),
            ..Default::default()
        })
    }));
    for base in [10_i64, 20, 30] {
        profiler.on_pressure(base);
        profiler.on_pressure(base + 1);
        profiler.on_tick(base + 1).unwrap();
    }
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    let _ = fs::remove_dir_all(root);
}


#[test]
fn production_profiler_owner_consumes_live_experiment_knobs_without_starting_capture() {
    const SERVICE: &str = include_str!("../src/extensions/telemetry/host_telemetry_service.rs");
    const PROFILER: &str = include_str!("../src/extensions/telemetry/pressure_cpu_profiler.rs");
    assert!(SERVICE.contains("create_production_pressure_cpu_profiler(Arc::clone(&experiments))"));
    assert!(PROFILER.contains("sand_pressure_cpu_profiler_config"));
    assert!(PROFILER.contains("ProductionCpuProfilerBackend::default()"));
}
