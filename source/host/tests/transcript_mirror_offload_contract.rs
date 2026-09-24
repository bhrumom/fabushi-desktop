use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use mahayana_host_runtime::agent_isolation::{
    TranscriptMirrorOffloadPool, TranscriptMirrorOffloadPoolOptions,
    TranscriptMirrorWorkerFn, TranscriptMirrorWorkerJob,
};

fn job(conversation_id: &str, marker: u8) -> TranscriptMirrorWorkerJob {
    TranscriptMirrorWorkerJob {
        conversation_id: conversation_id.to_string(),
        state_blob_id: vec![marker],
        blob_db_paths: vec![PathBuf::from("conversation-blobs.db")],
        transcripts_dir: PathBuf::from("transcripts"),
    }
}

fn wait_until(timeout: Duration, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("condition did not become true before timeout");
}

#[test]
fn worker_sharding_matches_frozen_js_utf16_hashing() {
    let worker: TranscriptMirrorWorkerFn = Arc::new(|_| Ok(true));
    let pool = TranscriptMirrorOffloadPool::with_worker(
        TranscriptMirrorOffloadPoolOptions { max_workers: 3 },
        worker,
    );

    assert_eq!(pool.worker_index_for("agent-a"), 0);
    assert_eq!(pool.worker_index_for("agent-b"), 2);
    assert_eq!(pool.worker_index_for("conversation-1"), 2);
    assert_eq!(pool.worker_index_for("conversation-2"), 1);
    assert_eq!(pool.worker_index_for("😀"), 1);
    pool.close_all();
}

#[test]
fn running_conversation_coalesces_to_latest_job_and_resolves_all_waiters() {
    let calls = Arc::new(Mutex::new(Vec::<u8>::new()));
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));

    let worker: TranscriptMirrorWorkerFn = {
        let calls = Arc::clone(&calls);
        let release_rx = Arc::clone(&release_rx);
        Arc::new(move |job| {
            let marker = *job.state_blob_id.first().unwrap_or(&0);
            calls.lock().expect("calls").push(marker);
            if marker == 1 {
                entered_tx.send(()).expect("entered");
                release_rx
                    .lock()
                    .expect("release receiver")
                    .recv()
                    .expect("release");
            }
            Ok(true)
        })
    };
    let pool = Arc::new(TranscriptMirrorOffloadPool::with_worker(
        TranscriptMirrorOffloadPoolOptions { max_workers: 1 },
        worker,
    ));

    let first_pool = Arc::clone(&pool);
    let first = thread::spawn(move || first_pool.write(job("conversation-a", 1)));
    entered_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("first job entered");

    let second_pool = Arc::clone(&pool);
    let second = thread::spawn(move || second_pool.write(job("conversation-a", 2)));
    wait_until(Duration::from_secs(3), || {
        pool.queued_waiter_count("conversation-a") == 1
    });

    let third_pool = Arc::clone(&pool);
    let third = thread::spawn(move || third_pool.write(job("conversation-a", 3)));
    wait_until(Duration::from_secs(3), || {
        pool.queued_waiter_count("conversation-a") == 2
    });

    release_tx.send(()).expect("release first");
    assert_eq!(first.join().expect("first join"), Ok(true));
    assert_eq!(second.join().expect("second join"), Ok(true));
    assert_eq!(third.join().expect("third join"), Ok(true));
    wait_until(Duration::from_secs(3), || pool.active_lane_count() == 0);

    assert_eq!(*calls.lock().expect("calls"), vec![1, 3]);
    assert_eq!(pool.active_worker_count(), 1);
    pool.close_all();
}

#[test]
fn different_shards_own_independent_long_lived_workers() {
    let worker: TranscriptMirrorWorkerFn = Arc::new(|_| Ok(true));
    let pool = TranscriptMirrorOffloadPool::with_worker(
        TranscriptMirrorOffloadPoolOptions { max_workers: 2 },
        worker,
    );

    assert_eq!(pool.write(job("agent-a", 1)), Ok(true));
    assert_eq!(pool.write(job("agent-b", 2)), Ok(true));
    assert_eq!(pool.active_worker_count(), 2);
    pool.close_all();
    assert_eq!(pool.active_worker_count(), 0);
}

#[test]
fn closed_pool_rejects_new_writes() {
    let worker: TranscriptMirrorWorkerFn = Arc::new(|_| Ok(true));
    let pool = TranscriptMirrorOffloadPool::with_worker(
        TranscriptMirrorOffloadPoolOptions { max_workers: 1 },
        worker,
    );
    pool.close_all();
    assert_eq!(
        pool.write(job("conversation-a", 1)),
        Err("mirror offload pool is closed".to_string())
    );
}
